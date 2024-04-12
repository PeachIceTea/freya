use anyhow::Context;
use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    response::IntoResponse,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};

use crate::{
    api_bail, data_response,
    state::FreyaState,
    util::{
        ffmpeg::{ffprobe_book_details, FileInfo},
        list_fs::{get_file_system_list, Entry, IMAGE_EXTENSIONS},
        response::{ApiError, ApiFileResult, ApiResult, DataResponse},
        send_file::send_file,
        session::{AdminSession, Session},
        storage::TMP_PATH,
    },
};

// Default directory to list.
// Read DEFAULT_DIRECTORY from environment variable using a lazy once_cell.
// Default to /.
pub static DEFAULT_DIRECTORY: once_cell::sync::Lazy<String> = once_cell::sync::Lazy::new(|| {
    std::env::var("DEFAULT_DIRECTORY").unwrap_or_else(|_| "/".to_string())
});

pub fn router() -> Router<FreyaState> {
    Router::new()
        .route("/", get(fs))
        .route("/info", get(ffprobe))
        .route("/tmp-cover", get(get_tmp_cover))
        .route("/audio/:file_id", get(get_audio_file))
}

// Query for the file system list.
#[derive(Deserialize)]
pub struct FsQuery {
    path: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FsResponse {
    path: String,
    parent_path: String,
    directory: Vec<Entry>,
}

// List a directory in the file system.
pub async fn fs(
    Session(_): Session,
    Query(FsQuery { path }): Query<FsQuery>,
) -> ApiResult<DataResponse<FsResponse>> {
    let path = path.map_or_else(
        || DEFAULT_DIRECTORY.to_string(),
        |p| match p.trim() {
            "" => DEFAULT_DIRECTORY.to_string(),
            p => p.to_string(),
        },
    );

    let list = get_file_system_list(&path)
        .await
        .context(ApiError::CouldNotListDirectory)?;

    let parent_path = std::path::Path::new(&path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "/".to_string());

    data_response!(FsResponse {
        path,
        parent_path,
        directory: list,
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FfprobeResponse {
    path: String,
    info: FileInfo,
}

// Get ffprobe info for a file.
pub async fn ffprobe(
    Session(_): Session,
    Query(FsQuery { path }): Query<FsQuery>,
) -> ApiResult<DataResponse<FfprobeResponse>> {
    let path = path.context(ApiError::InvalidPath)?;

    let info = ffprobe_book_details(&path)
        .await
        .with_context(|| ApiError::FFProbeFailed(path.to_string()))?;

    data_response!(FfprobeResponse { path, info })
}

// If PathBuf::join() is called with an absolute path, it will ignore the previous path. This allows
// the use of this function for both extracted files and files selected from the file system. It
// also theoretically allows the use of this function to download arbitrary files from the server.
// Not sure that is an actual attack vector, but it is something to keep in mind. To limit the
// potential damage, this function can only be called by an admin and checks if the file has an
// image extension.
#[derive(Deserialize)]
pub struct TemporaryCoverQuery {
    name: String,
}

pub async fn get_tmp_cover(
    AdminSession(_): AdminSession,
    Query(TemporaryCoverQuery { name }): Query<TemporaryCoverQuery>,
) -> ApiFileResult<Vec<u8>> {
    // Read the file.
    let path = TMP_PATH.join(name);

    tracing::debug!("Reading temporary cover image: {}", path.to_string_lossy());

    // Check if the file is an image.
    let ext = path
        .extension()
        .context(ApiError::InvalidPath)?
        .to_string_lossy();
    if !IMAGE_EXTENSIONS.contains(&ext.as_ref()) {
        api_bail!(InvalidPath)
    }

    let data = std::fs::read(&path)
        .with_context(|| format!("Failed to read image file: {}", path.to_string_lossy()))?;

    Ok(data)
}

// Send user the requested audio file.
// We use the path stored in the database to get the file.

pub async fn get_audio_file(
    Session(_): Session,
    Path(file_id): Path<String>,
    headers: HeaderMap,
    State(state): State<FreyaState>,
) -> ApiFileResult<impl IntoResponse> {
    // Get file path from database.
    let file = state
        .database
        .get_file_path(&file_id)
        .await?
        .context(ApiError::InvalidPath)?;

    Ok(send_file(&file, Some(&headers)).await)
}
