use anyhow::Context;
use axum::{extract::Query, routing::get, Router};
use serde::{Deserialize, Serialize};

use crate::{
    data_response,
    state::FreyaState,
    util::{
        ffmpeg::{ffprobe_book_details, FileInfo},
        list_fs::{get_file_system_list, Entry},
        response::{ApiError, ApiResult, DataResponse},
        session::Session,
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
        .route("/ffprobe", get(ffprobe))
}

// Query for the file system list.
#[derive(Deserialize)]
pub struct FsQuery {
    path: Option<String>,
}

#[derive(Serialize)]
pub struct FsResponse {
    path: String,
    directory: Vec<Entry>,
}

// List a directory in the file system.
pub async fn fs(
    Session(_): Session,
    Query(FsQuery { path }): Query<FsQuery>,
) -> ApiResult<DataResponse<FsResponse>> {
    let path = path.unwrap_or(DEFAULT_DIRECTORY.to_string());

    let list = get_file_system_list(&path)
        .await
        .context(ApiError::CouldNotListDirectory)?;

    data_response!(FsResponse {
        path,
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
