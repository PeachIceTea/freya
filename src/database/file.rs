use super::Database;
use anyhow::{Context, Result};
use serde::Serialize;
use time::OffsetDateTime;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct File {
    pub id: i64,
    pub book_id: i64,

    #[serde(skip)]
    pub path: String,

    pub name: String,
    pub position: i64,
    pub duration: f64,

    #[serde(with = "time::serde::iso8601")]
    pub created: OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    pub modified: OffsetDateTime,
}

pub struct FileData {
    pub path: String,
    pub name: String,
    pub duration: f64,
}

impl Database {
    // Get files for a book.
    pub async fn get_files_for_book(&self, book_id: i64) -> Result<Vec<File>> {
        sqlx::query_as!(
            File,
            r#"
                SELECT *
                FROM files
                WHERE book_id = ?
                ORDER BY position ASC
            "#,
            book_id
        )
        .fetch_all(&self.pool)
        .await
        .context("Unable to get files for book")
    }

    // Get path of file.
    pub async fn get_file_path(&self, file_id: &str) -> Result<Option<String>> {
        sqlx::query!("SELECT path FROM files WHERE id = $1", file_id)
            .fetch_optional(&self.pool)
            .await
            .context("Unable to get path for file")
            .map(|result| result.map(|result| result.path))
    }

    // Get cover of book.
    pub async fn get_book_cover(&self, book_id: i64) -> Result<Option<Vec<u8>>> {
        sqlx::query!(
            r#"
                SELECT cover
                FROM books
                WHERE id = ?
            "#,
            book_id
        )
        .fetch_optional(&self.pool)
        .await
        .context("Unable to get book cover")
        .map(|result| result.map(|result| result.cover).flatten())
    }
}
