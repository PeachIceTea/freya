use anyhow::Context;
use axum::{extract::State, routing::post, Router};

use crate::{
    api_response,
    state::FreyaState,
    util::{
        ffmpeg::ffprobe_chapters,
        response::{ApiResult, SuccessResponse},
        session::AdminSession,
    },
};

pub fn router() -> Router<FreyaState> {
    Router::new().route("/rediscover-chapters", post(rediscover_chapters))
}

pub async fn rediscover_chapters(
    AdminSession(_): AdminSession,
    State(state): State<FreyaState>,
) -> ApiResult<SuccessResponse> {
    // Get all books with only one file.
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
    .fetch_all(&state.db)
    .await
    .context("Failed to fetch books with only one file")?;

    // For each book, get the file and parse it.
    let mut chapter_info = Vec::new();
    for entry in entries {
        let path = &entry.path;
        let chapters = ffprobe_chapters(path).await;
        if let Ok(chapters) = chapters {
            chapter_info.push((entry.book_id, chapters));
        }
    }

    // Update the database with the new chapter info.
    if !chapter_info.is_empty() {
        let mut trx = state
            .db
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
    }

    api_response!("admin--chapters-rediscovered")
}
