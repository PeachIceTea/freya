use std::path::PathBuf;

use anyhow::Context;
use axum::{
    Router,
    extract::{Path, Query, State},
    http::HeaderMap,
    response::IntoResponse,
    routing::get,
};
use serde::{Deserialize, Serialize};

use super::response::{ApiError, ApiFileResult, ApiResult, DataResponse};
use crate::{
    api_bail,
    auth::session::{AdminSession, Session},
    data_response,
    fs::{
        list_fs::{Entry, IMAGE_EXTENSIONS, get_file_system_list},
        path::validate_path_within_bounds,
        send_file::send_file,
        storage::{FELA_MEDIA_ROOT, TMP_PATH},
    },
    media::ffmpeg::{FileInfo, ffprobe_book_details},
    state::FelaState,
};

/// Build router for file system routes.
/// Is attached to `/fs`.
pub fn router() -> Router<FelaState> {
    Router::new()
        .route("/", get(fs))
        .route("/info", get(ffprobe))
        .route("/tmp-cover", get(get_tmp_cover))
        .route("/audio/{file_id}", get(get_audio_file))
}

/// Query for the file system list.
#[derive(Deserialize)]
pub struct FsQuery {
    path: Option<String>,
}

/// Response for a file system query.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FsResponse {
    path: String,
    parent_path: String,
    directory: Vec<Entry>,
}

/// List a directory in the file system.
pub async fn fs(
    AdminSession(_): AdminSession,
    Query(FsQuery { path }): Query<FsQuery>,
) -> ApiResult<DataResponse<FsResponse>> {
    let path = path
        .as_deref()
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| FELA_MEDIA_ROOT.to_owned());

    // Validate path stays within allowed bounds; fall back to FELA_MEDIA_ROOT if invalid.
    let path = validate_path_within_bounds(&path, &FELA_MEDIA_ROOT).unwrap_or_else(|_| {
        tracing::warn!(
            "Path traversal attempt, clamping to media root: {}",
            path.to_string_lossy()
        );
        FELA_MEDIA_ROOT.clone()
    });

    let list = get_file_system_list(&path)
        .await
        .context(ApiError::CouldNotListDirectory)?;

    // Send down the parent directory for easier traversal.
    // This might point outside of FELA_MEDIA_ROOT, but any future calls to `fs` will clamp.
    // There is also no information leakage, the client sees the full path either way.
    let parent_path = path
        .parent()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| "/".to_string());

    data_response!(FsResponse {
        path: path.to_string_lossy().into_owned(),
        parent_path,
        directory: list,
    })
}

/// FFProbe response.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FfprobeResponse {
    path: String,
    info: FileInfo,
}

/// Get ffprobe info for a file.
pub async fn ffprobe(
    AdminSession(_): AdminSession,
    Query(FsQuery { path }): Query<FsQuery>,
) -> ApiResult<DataResponse<FfprobeResponse>> {
    let path = path.context(ApiError::InvalidPath)?;

    // Validate the path stays within allowed bounds. Path must be exact for ffprobe.
    let path = validate_path_within_bounds(std::path::Path::new(&path), &FELA_MEDIA_ROOT)?;

    let info = ffprobe_book_details(&path)
        .await
        .with_context(|| ApiError::FFProbeFailed(path.to_string_lossy().into_owned()))?;

    data_response!(FfprobeResponse {
        path: path.to_string_lossy().into_owned(),
        info
    })
}

/// Querystring for a temporary cover request.
#[derive(Deserialize)]
pub struct TemporaryCoverQuery {
    name: String,
}

/// Return a temporary cover.
/// The path is handed via querystring `&name=`.
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

/// Send user the requested audio file.
/// We use the path stored in the database to get the file.
pub async fn get_audio_file(
    Session(_): Session,
    Path(file_id): Path<String>,
    headers: HeaderMap,
    State(state): State<FelaState>,
) -> ApiFileResult<impl IntoResponse> {
    // Get file path from database.
    let file = state
        .database
        .get_file_path(&file_id)
        .await?
        .context(ApiError::InvalidPath)?;

    Ok(send_file(&file, Some(&headers)).await)
}
