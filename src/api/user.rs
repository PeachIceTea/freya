use anyhow::Context;
use axum::{
    extract::{Path, State},
    routing::get,
    Router,
};
use serde::Serialize;

use crate::{
    data_response,
    state::FreyaState,
    util::{
        response::{ApiResult, DataResponse},
        session::Session,
    },
};

pub fn router() -> Router<FreyaState> {
    Router::new().route("/:id/library", get(get_library))
}

#[derive(Serialize)]
pub struct LibraryResponse {
    id: i64,
    title: String,
    author: String,
    list: String,
    // There isn't actually a way for progress to be null, but sqlx apparently can't guarantee that.
    progress: Option<f64>,
}

pub async fn get_library(
    Session(_): Session,
    State(state): State<FreyaState>,
    Path(id): Path<i64>,
) -> ApiResult<DataResponse<Vec<LibraryResponse>>> {
    // Get library by user id.
    let library = sqlx::query_as!(
        LibraryResponse,
        // Not the prettiest query, but it works.
        r#"
        WITH total_duration AS (
            SELECT book_id, SUM(duration) as total_duration
            FROM files
            GROUP BY book_id
        )
        SELECT
            books.id,
            books.title,
            books.author,
            library_entries.list,
            COALESCE((
                SELECT (SUM(file_sub.duration) + library_entries.progress) / total_duration.total_duration
                FROM files file_sub
                WHERE
                    files.book_id = books.id
                    AND file_sub.position < files.position
            ), library_entries.progress / total_duration.total_duration) as "progress: f64"
        FROM books
        JOIN library_entries ON library_entries.book_id = books.id
        JOIN files ON library_entries.file_id = files.id
        JOIN total_duration ON total_duration.book_id = books.id
        WHERE library_entries.user_id = ?
        ORDER BY library_entries.modified DESC
        "#,
        id
    )
    .fetch_all(&state.db)
    .await
    .context("Failed to fetch library")?;

    data_response!(library)
}
