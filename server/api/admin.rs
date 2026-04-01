use std::path::Path;

use anyhow::Context;
use axum::{Router, extract::State, routing::post};

use super::response::{ApiResult, SuccessResponse};
use crate::{
    api_response, auth::session::AdminSession, media::ffmpeg::ffprobe_chapters, state::FelaState,
};

/// Build router for admin routes.
/// Is attached to `/admin`.
pub fn router() -> Router<FelaState> {
    Router::new().route("/rediscover-chapters", post(rediscover_chapters))
}

/// Utility function that iterates through audio files and creates new chapter markers.
/// Run on request by an admin user.
pub async fn rediscover_chapters(
    AdminSession(_): AdminSession,
    State(state): State<FelaState>,
) -> ApiResult<SuccessResponse> {
    // Get all books with only one file.
    // Files that are split up are almost always split by chapter already.
    let entries = sqlx::query!(
        r#"
            SELECT book_id, path
            FROM files
            WHERE book_id IN (
                SELECT book_id
                FROM files
                GROUP BY book_id
                HAVING COUNT(1) = 1
            )
        "#,
    )
    .fetch_all(&state.database.pool)
    .await
    .context("Failed to fetch books with only one file")?;

    // For each book, get the file and parse it.
    let mut chapter_info = Vec::new();
    for entry in entries {
        let path = Path::new(&entry.path);
        let chapters = ffprobe_chapters(path).await;
        if let Ok(chapters) = chapters {
            chapter_info.push((entry.book_id, chapters));
        }
    }

    // In case we don't find any chapter markers we are already done.
    if chapter_info.is_empty() {
        return api_response!("admin--chapters-rediscovered");
    }

    // Update the database with the new chapter info.
    let mut trx = state
        .database
        .pool
        .begin()
        .await
        .context("Failed to start transaction")?;

    for (book_id, chapters) in chapter_info {
        // Delete old chapters.
        sqlx::query!(
            r#"
                    DELETE FROM chapters
                    WHERE book_id = $1
                "#,
            book_id,
        )
        .execute(&mut *trx)
        .await
        .context("Failed to delete old chapters")?;

        // Insert new ones.
        for chapter in chapters {
            sqlx::query!(
                r#"
                        INSERT INTO chapters (book_id, name, start, end)
                        VALUES ($1, $2, $3, $4)
                    "#,
                book_id,
                chapter.name,
                chapter.start,
                chapter.end,
            )
            .execute(&mut *trx)
            .await
            .context("Failed to insert chapters")?;
        }
    }

    trx.commit().await.context("Failed to commit transaction")?;

    api_response!("admin--chapters-rediscovered")
}
