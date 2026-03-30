use anyhow::Context;
use axum::{
    Router,
    extract::{Path, Query, State},
    http::HeaderMap,
    response::IntoResponse,
    routing::get,
};
use serde::{Deserialize, Serialize};

use crate::{
    api_bail, data_response,
    auth::session::{AdminSession, Session},
    fs::{
        list_fs::{Entry, IMAGE_EXTENSIONS, get_file_system_list},
        path::validate_path_within_bounds,
        send_file::send_file,
        storage::{FREYA_MEDIA_ROOT, TMP_PATH},
    },
    media::ffmpeg::{FileInfo, ffprobe_book_details},
    state::FreyaState,
};
use super::response::{ApiError, ApiFileResult, ApiResult, DataResponse};

pub fn router() -> Router<FreyaState> {
    Router::new()
        .route("/", get(fs))
        .route("/info", get(ffprobe))
        .route("/tmp-cover", get(get_tmp_cover))
        .route("/audio/{file_id}", get(get_audio_file))
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
        || FREYA_MEDIA_ROOT.to_string_lossy().into_owned(),
        |p| match p.trim() {
            "" => FREYA_MEDIA_ROOT.to_string_lossy().into_owned(),
            p => p.to_string(),
        },
    );

    // Validate path stays within allowed bounds; fall back to FREYA_MEDIA_ROOT if invalid.
    let validated_path =
        validate_path_within_bounds(std::path::Path::new(&path), &FREYA_MEDIA_ROOT)
            .unwrap_or_else(|_| FREYA_MEDIA_ROOT.clone());

    let list = get_file_system_list(&validated_path)
        .await
        .context(ApiError::CouldNotListDirectory)?;

    let parent_path = std::path::Path::new(&path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "/".to_string());

    data_response!(FsResponse {
        path: validated_path.to_string_lossy().to_string(),
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

    // Validate the path stays within allowed bounds. Path must be exact for ffprobe.
    let validated_path =
        validate_path_within_bounds(std::path::Path::new(&path), &*FREYA_MEDIA_ROOT)?;
    let validated_path_str = validated_path.to_string_lossy().to_string();

    let info = ffprobe_book_details(&validated_path_str)
        .await
        .with_context(|| ApiError::FFProbeFailed(validated_path_str))?;

    data_response!(FfprobeResponse { path, info })
}

#[derive(Deserialize)]
pub struct TemporaryCoverQuery {
    name: String,
}

pub async fn get_tmp_cover(
    AdminSession(_): AdminSession,
    Query(TemporaryCoverQuery { name }): Query<TemporaryCoverQuery>,
) -> ApiFileResult<Vec<u8>> {
    // Join name with TMP_PATH before validating — name is a bare filename, not a full path.
    let full_path = TMP_PATH.join(&name);
    let validated_path =
        validate_path_within_bounds(&full_path, &TMP_PATH).context(ApiError::InvalidPath)?;

    tracing::debug!(
        "Reading temporary cover image: {}",
        validated_path.to_string_lossy()
    );

    // Check if the file is an image.
    let ext = validated_path
        .extension()
        .context(ApiError::InvalidPath)?
        .to_string_lossy();
    if !IMAGE_EXTENSIONS.contains(&ext.as_ref()) {
        api_bail!(InvalidPath)
    }

    let data = std::fs::read(&validated_path).with_context(|| {
        format!(
            "Failed to read image file: {}",
            validated_path.to_string_lossy()
        )
    })?;

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
