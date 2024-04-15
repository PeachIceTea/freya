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

#[cfg(test)]
mod tests {

    use super::*;

    #[sqlx::test(fixtures("book"))]
    async fn test_get_chapters(pool: sqlx::Pool<sqlx::Sqlite>) {
        // Test case: Verify that get_chapters_for_book fetches chapters correctly
        let db = Database::new_test(pool);

        let chapters = db
            .get_chapters_for_book(15)
            .await
            .expect("Should be able to get chapters");

        assert_eq!(chapters.len(), 25);

        assert_eq!(chapters[0].id, 1174);
        assert_eq!(chapters[0].book_id, 15);
        assert_eq!(chapters[0].name, "Opening Credits");
        assert_eq!(chapters[0].start, 0.0);

        let last_chapter = chapters.last().expect("Should be able to get last chapter");
        assert_eq!(last_chapter.id, 1198);
        assert_eq!(last_chapter.book_id, 15);
        assert_eq!(last_chapter.name, "End Credits");
    }

    #[sqlx::test(fixtures("book"))]
    async fn test_get_chapters_empty(pool: sqlx::Pool<sqlx::Sqlite>) {
        // Test case: Verify that get_chapters_for_book reutrn nothing for a book_id that doesn't
        // exist.
        let db = Database::new_test(pool);

        let chapters = db
            .get_chapters_for_book(999)
            .await
            .expect("Should be able to get chapters");

        assert_eq!(chapters.len(), 0);
    }
}
