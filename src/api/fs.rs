use std::path::Path;

use anyhow::Context;
use axum::{extract::Query, routing::get, Router};
use serde::{Deserialize, Serialize};

use crate::{
    api_bail, data_response,
    state::FreyaState,
    util::{
        ffmpeg::{ffprobe_book_details, FileInfo},
        list_fs::{get_file_system_list, Entry},
        response::{ApiError, ApiFileResult, ApiResult, DataResponse},
        session::Session,
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
}

// Query for the file system list.
#[derive(Deserialize)]
pub struct FsQuery {
    path: Option<String>,
}

#[derive(Serialize)]
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

    let parent_path = Path::new(&path)
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

#[derive(Deserialize)]
pub struct TemporaryCoverQuery {
    name: String,
}

pub async fn get_tmp_cover(
    Session(_): Session,
    Query(TemporaryCoverQuery { name }): Query<TemporaryCoverQuery>,
) -> ApiFileResult<Vec<u8>> {
    // Read the file.
    let path = TMP_PATH.join(&name);

    // Check if the file is an image.
    let ext = path.extension().context(ApiError::InvalidPath)?;
    if !matches!(ext.to_str(), Some("jpg") | Some("jpeg") | Some("png")) {
        api_bail!(InvalidPath)
    }

    let data = std::fs::read(&path)
        .with_context(|| format!("Failed to read image file: {}", path.to_string_lossy()))?;

    Ok(data)
}
