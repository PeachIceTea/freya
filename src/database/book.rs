use anyhow::{Context, Result};
use serde::Serialize;
use time::OffsetDateTime;

use crate::util::ffmpeg::Chapters;

use super::{file::FileData, Database};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Book {
    pub id: i64,

    pub title: String,
    pub author: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,

    #[serde(with = "time::serde::iso8601")]
    pub created: OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    pub modified: OffsetDateTime,
}

impl Database {
    // Get all books.
    pub async fn get_all_books(&self) -> Result<Vec<Book>> {
        // Get all books from the database.
        sqlx::query_as!(
            Book,
            r#"
                SELECT
                    id,
                    title,
                    author,
                    created,
                    modified,
                    NULL AS "duration: f64"
                FROM books
                ORDER BY title ASC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .context("Unable to get all books")
    }

    // Get book details.
    pub async fn get_book_details(&self, book_id: i64) -> Result<Option<Book>> {
        sqlx::query_as!(
            Book,
            r#"
                SELECT
                    id,
                    title,
                    author,
                    created,
                    modified,
                    (
                        SELECT SUM(duration)
                        FROM files
                        WHERE book_id = books.id
                    ) AS "duration: f64"
                FROM books
                WHERE id = ?
            "#,
            book_id
        )
        .fetch_optional(&self.pool)
        .await
        .context("Unable to get book details")
    }

    // Create new book.
    pub async fn create_book(
        &self,
        title: &str,
        author: &str,
        cover: Option<&Vec<u8>>,
        file_data: &[FileData],
        chapters: Option<&Vec<Chapters>>,
    ) -> Result<i64> {
        let mut trx = self
            .pool
            .begin()
            .await
            .context("Failed to start transaction")?;

        // Insert book.
        let book_id = sqlx::query!(
            r#"
            INSERT INTO books (title, author, cover)
            VALUES (?, ?, ?)
            RETURNING id
        "#,
            title,
            author,
            cover,
        )
        .fetch_one(&mut *trx)
        .await
        .context("Failed to insert book into database")?
        .id;

        // Insert files into the database.
        for (position, file) in file_data.iter().enumerate() {
            let position = position as i64 + 1;
            sqlx::query!(
                r#"
                INSERT INTO files (book_id, path, name, position, duration)
                VALUES (?, ?, ?, ?, ?)
            "#,
                book_id,
                file.path,
                file.name,
                position,
                file.duration,
            )
            .execute(&mut *trx)
            .await
            .context("Failed to insert file into database")?;
        }

        // Insert chapters into the database.
        if let Some(chapters) = chapters {
            for chapter in chapters {
                sqlx::query!(
                    r#"
                    INSERT INTO chapters (book_id, name, start, end)
                    VALUES (?, ?, ?, ?)
                "#,
                    book_id,
                    chapter.name,
                    chapter.start,
                    chapter.end,
                )
                .execute(&mut *trx)
                .await
                .context("Failed to insert chapter into database")?;
            }
        }

        // Commit transaction.
        trx.commit().await.context("Failed to commit transaction")?;

        Ok(book_id)
    }
}
