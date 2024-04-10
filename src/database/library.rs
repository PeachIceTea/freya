use super::Database;

use anyhow::{Context, Result};
use serde::Serialize;
use time::OffsetDateTime;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryEntry {
    pub id: i64,

    #[serde(skip)]
    pub user_id: i64,
    #[serde(skip)]
    pub book_id: i64,

    pub file_id: i64,
    pub progress: f64,

    pub list: String,

    #[serde(with = "time::serde::iso8601")]
    pub created: OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    pub modified: OffsetDateTime,
}

#[derive(Serialize)]
pub struct LibraryResponse {
    id: i64,
    title: String,
    author: String,
    list: String,
    progress: f64,
}

impl Database {
    // Get a users library entry for a given book.
    pub async fn get_library_entry(
        &self,
        user_id: i64,
        book_id: i64,
    ) -> Result<Option<LibraryEntry>> {
        sqlx::query_as!(
            LibraryEntry,
            r#"
                SELECT *
                FROM library_entries
                WHERE user_id = ?
                AND book_id = ?
            "#,
            user_id,
            book_id
        )
        .fetch_optional(&self.pool)
        .await
        .context("Unable to get library entry")
    }

    // Updates a users progress.
    pub async fn update_progress(
        &self,
        user_id: i64,
        book_id: i64,
        file_id: i64,
        progress: f64,
    ) -> Result<()> {
        sqlx::query!(
            r#"
                UPDATE library_entries
                SET progress = ?,
                    file_id = ?,
                    modified = CURRENT_TIMESTAMP
                WHERE user_id = ?
                AND book_id = ?
            "#,
            progress,
            file_id,
            user_id,
            book_id,
        )
        .execute(&self.pool)
        .await
        .context("Unable to update progress")
        .map(|_| ())
    }

    // Manage library entry for a given book.
    pub async fn manage_library_entry(
        &self,
        user_id: i64,
        book_id: i64,
        list: &str,
        file_id: Option<i64>,
        progress: Option<f64>,
    ) -> Result<()> {
        // TODO: This is horrible, can't belive I wrote this. I need to split all of this up.

        // Try to create a new library entry. If it already exists, update it.
        // If file_id is None, the first file will be used.
        // If the user already has a library entry for the book, the library entry will be updated.
        // If the file_id is updated, the progress will be reset to 0.
        sqlx::query!(
            r#"
                INSERT INTO library_entries (user_id, book_id, file_id, list, progress)
                VALUES ($1, $2, COALESCE($4, (
                    SELECT id
                    FROM files
                    WHERE book_id = $2
                    ORDER BY position ASC
                    LIMIT 1
                )), $3, COALESCE($5, 0))
                ON CONFLICT (user_id, book_id) DO UPDATE
                SET list = EXCLUDED.list,
                    file_id = COALESCE(EXCLUDED.file_id, library_entries.file_id),
                    progress = CASE
                        WHEN library_entries.file_id != EXCLUDED.file_id THEN 0
                        WHEN $5 IS NOT NULL THEN $5
                        ELSE library_entries.progress
                    END,
                    modified = CURRENT_TIMESTAMP
            "#,
            user_id,
            book_id,
            list,
            file_id,
            progress,
        )
        .execute(&self.pool)
        .await
        .context("Unable to manage library entry")
        .map(|_| ())
    }

    // Get user library.
    pub async fn get_user_library(&self, user_id: i64) -> Result<Vec<LibraryResponse>> {
        sqlx::query_as!(
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
                    ), library_entries.progress / total_duration.total_duration) as "progress!: f64"
                FROM books
                JOIN library_entries ON library_entries.book_id = books.id
                JOIN files ON library_entries.file_id = files.id
                JOIN total_duration ON total_duration.book_id = books.id
                WHERE library_entries.user_id = ?
                ORDER BY library_entries.modified DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .context("Unable to get user library")
    }
}
