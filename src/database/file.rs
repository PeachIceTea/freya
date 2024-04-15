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
                SELECT
                    id,
                    book_id,
                    path,
                    name,
                    position,
                    duration,
                    created,
                    modified
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
        .map(|result| result.and_then(|result| result.cover))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test(fixtures("book"))]
    async fn test_get_files_for_book(pool: sqlx::Pool<sqlx::Sqlite>) {
        // Test case: Verify that get_files_for_book fetches files correctly
        let db = Database::new_test(pool);

        let files = db
            .get_files_for_book(15)
            .await
            .expect("Should be able to get files");

        assert_eq!(files.len(), 1);

        assert_eq!(files[0].id, 336);
        assert_eq!(files[0].book_id, 15);
        assert_eq!(files[0].name, "A Witch's Sin");
        assert_eq!(files[0].position, 1);
    }

    #[sqlx::test(fixtures("book"))]
    async fn test_get_files_for_book_empty(pool: sqlx::Pool<sqlx::Sqlite>) {
        // Test case: Verify that get_files_for_book returns nothing for a book_id that doesn't exist
        let db = Database::new_test(pool);

        let files = db
            .get_files_for_book(999)
            .await
            .expect("Should be able to get files");

        assert_eq!(files.len(), 0);
    }

    #[sqlx::test(fixtures("book"))]
    async fn test_get_file_path(pool: sqlx::Pool<sqlx::Sqlite>) {
        // Test case: Verify that get_file_path fetches the file path correctly
        let db = Database::new_test(pool);

        let path = db
            .get_file_path("336")
            .await
            .expect("Should be able to get file path")
            .expect("File path should exist");

        assert_eq!(
            path,
            "/media/Daniel B. Greene - A Witch's Sin/A Witch's Sin.m4b"
        );
    }

    #[sqlx::test(fixtures("book"))]
    async fn test_get_file_path_not_found(pool: sqlx::Pool<sqlx::Sqlite>) {
        // Test case: Verify that get_file_path returns None for a file_id that doesn't exist
        let db = Database::new_test(pool);

        let path = db
            .get_file_path("999")
            .await
            .expect("Should be able to get file path");

        assert!(path.is_none());
    }

    #[sqlx::test(fixtures("book"))]
    async fn test_get_book_cover(pool: sqlx::Pool<sqlx::Sqlite>) {
        // Test case: Verify that get_book_cover fetches the book cover correctly
        let db = Database::new_test(pool);

        let cover = db
            .get_book_cover(15)
            .await
            .expect("Should be able to get book cover");

        assert!(cover.is_none()); // The cover is NULL in the fixture data
    }

    #[sqlx::test(fixtures("book"))]
    async fn test_get_book_cover_not_found(pool: sqlx::Pool<sqlx::Sqlite>) {
        // Test case: Verify that get_book_cover returns None for a book_id that doesn't exist
        let db = Database::new_test(pool);

        let cover = db
            .get_book_cover(999)
            .await
            .expect("Should be able to get book cover");

        assert!(cover.is_none());
    }
}
