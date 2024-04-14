use anyhow::{Context, Result};
use serde::Serialize;

use super::Database;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Chapter {
    pub id: i64,

    #[serde(skip)]
    pub book_id: i64,

    pub name: String,
    pub start: f64,
    pub end: f64,
}

impl Database {
    pub async fn get_chapters_for_book(&self, book_id: i64) -> Result<Vec<Chapter>> {
        sqlx::query_as!(
            Chapter,
            r#"
                SELECT
                    id,
                    book_id,
                    name,
                    start,
                    end
                FROM chapters
                WHERE book_id = ?
                ORDER BY start ASC
            "#,
            book_id
        )
        .fetch_all(&self.pool)
        .await
        .context("Unable to get chapters for book")
    }
}
