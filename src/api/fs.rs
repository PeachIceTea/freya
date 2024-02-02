use axum::{extract::Query, response::IntoResponse, routing::get, Router};
use serde::Deserialize;

use crate::{
    axum_json,
    state::FreyaState,
    util::{ffmpeg::ffprobe_book_details, list_fs::get_file_system_list, session::Session},
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

// List a directory in the file system.
pub async fn fs(Session(_): Session, Query(FsQuery { path }): Query<FsQuery>) -> impl IntoResponse {
    let path = path.unwrap_or(DEFAULT_DIRECTORY.to_string());

    let list = match get_file_system_list(&path).await {
        Ok(list) => list,
        Err(e) => {
            tracing::error!("Failed to get file system list: {}", e);
            return axum_json!({
                "error_code": "fs--list-failed",
            });
        }
    };

    axum_json!({
        "path": path,
        "directory": list,
    })
}

// Get ffprobe info for a file.
pub async fn ffprobe(
    Session(_): Session,
    Query(FsQuery { path }): Query<FsQuery>,
) -> impl IntoResponse {
    let path = match path {
        Some(path) => path,
        None => {
            return axum_json!({
                "error_code": "fs--ffprobe--path-missing",
            });
        }
    };

    let info = match ffprobe_book_details(&path).await {
        Ok(info) => info,
        Err(e) => {
            tracing::error!("Failed to get ffprobe info: {}", e);
            return axum_json!({
                "error_code": "fs--ffprobe--failed",
            });
        }
    };

    axum_json!({
        "path": path,
        "info": info,
    })
}
