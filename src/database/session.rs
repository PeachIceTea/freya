use super::Database;
use anyhow::{Context, Result};
use serde::Serialize;

// Extract the session from the request.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    #[serde(skip)]
    pub session_id: String,
    pub user_id: i64,
    #[serde(with = "time::serde::iso8601")]
    pub last_accessed: time::OffsetDateTime,
    pub username: String,
    pub admin: bool,
}

impl Database {
    // Get session by id.
    pub async fn get_session(&self, session_id: &str) -> Result<SessionInfo> {
        sqlx::query_as!(
            SessionInfo,
            r#"
                SELECT
                    sessions.id as session_id,
                    sessions.user_id,
                    sessions.last_accessed,
                    users.name as username,
                    users.admin
                FROM sessions
                INNER JOIN users ON sessions.user_id = users.id
                WHERE sessions.id = $1
            "#,
            session_id,
        )
        .fetch_one(&self.pool)
        .await
        .context("Unable to get session info")
    }

    // Create session.
    pub async fn create_session(&self, user_id: i64, session_id: &str) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO sessions (id, user_id)
            VALUES ($1, $2)
        "#,
            session_id,
            user_id
        )
        .execute(&self.pool)
        .await
        .context("Unable to create new session")
        .map(|_| ())
    }

    // Update timestamp of session.
    pub async fn update_session_timestamp(&self, session_id: &str) -> Result<()> {
        sqlx::query!(
            r#"
                UPDATE sessions
                SET last_accessed = CURRENT_TIMESTAMP
                WHERE id = $1
            "#,
            session_id,
        )
        .execute(&self.pool)
        .await
        .context("Unable to update session timestamp")
        .map(|_| ())
    }

    // Delete session by id.
    pub async fn delete_session(&self, session_id: &str) -> Result<()> {
        sqlx::query!(
            r#"
                DELETE FROM sessions
                WHERE id = $1
            "#,
            session_id,
        )
        .execute(&self.pool)
        .await
        .context("Unable to delete session")
        .map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test(fixtures("user"))]
    async fn test_get_session(pool: sqlx::Pool<sqlx::Sqlite>) {
        // Test case: Verify that get_session fetches session info correctly
        let db = Database::new_test(pool);

        // Create a new session for the user
        let session_id = "test_session_id";
        let user_id = 2;
        db.create_session(user_id, session_id)
            .await
            .expect("Should be able to create a new session");

        let session_info = db
            .get_session(session_id)
            .await
            .expect("Should be able to get session info");

        assert_eq!(session_info.session_id, session_id);
        assert_eq!(session_info.user_id, user_id);
        assert_eq!(session_info.username, "user");
        assert!(!session_info.admin);
    }

    #[sqlx::test(fixtures("user"))]
    async fn test_create_session(pool: sqlx::Pool<sqlx::Sqlite>) {
        // Test case: Verify that create_session creates a new session correctly
        let db = Database::new_test(pool);

        let session_id = "new_session_id";
        let user_id = 2;
        db.create_session(user_id, session_id)
            .await
            .expect("Should be able to create a new session");

        let session_info = db
            .get_session(session_id)
            .await
            .expect("Should be able to get session info");

        assert_eq!(session_info.session_id, session_id);
        assert_eq!(session_info.user_id, user_id);
    }

    #[sqlx::test(fixtures("user"))]
    async fn test_update_session_timestamp(pool: sqlx::Pool<sqlx::Sqlite>) {
        // Test case: Verify that update_session_timestamp updates the session timestamp correctly
        let db = Database::new_test(pool);

        let session_id = "test_session_id";
        let user_id = 2;
        db.create_session(user_id, session_id)
            .await
            .expect("Should be able to create a new session");

        let session_info_before = db
            .get_session(session_id)
            .await
            .expect("Should be able to get session info");

        // Wait for a short duration to ensure timestamp change
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

        db.update_session_timestamp(session_id)
            .await
            .expect("Should be able to update session timestamp");

        let session_info_after = db
            .get_session(session_id)
            .await
            .expect("Should be able to get session info");

        assert!(session_info_after.last_accessed > session_info_before.last_accessed);
    }

    #[sqlx::test(fixtures("user"))]
    async fn test_delete_session(pool: sqlx::Pool<sqlx::Sqlite>) {
        // Test case: Verify that delete_session deletes the session correctly
        let db = Database::new_test(pool);

        let session_id = "test_session_id";
        let user_id = 2;
        db.create_session(user_id, session_id)
            .await
            .expect("Should be able to create a new session");

        db.delete_session(session_id)
            .await
            .expect("Should be able to delete session");

        let result = db.get_session(session_id).await;
        assert!(result.is_err(), "Session should be deleted");
    }
}
