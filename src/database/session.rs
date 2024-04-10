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
