use axum::{
    async_trait,
    extract::{FromRequestParts, Request, State},
    http::{request::Parts, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use once_cell::sync::Lazy;
use serde::Serialize;
use serde_json::json;

use crate::state::FreyaState;

use super::{random::random_string, response::IntoResponseWithStatus};

// Bytes of entropy in the session id.
pub static SESSION_ID_ENTROPY: usize = 32;

// Session lifetime in hours.
// Read SESSION_LIFETIME from environment variable using once_cell.
// Default to 30 days.
pub static SESSION_LIFETIME: Lazy<i32> = Lazy::new(|| {
    std::env::var("SESSION_LIFETIME")
        .ok()
        .and_then(|lifetime| lifetime.parse().ok())
        .unwrap_or(24 * 30)
});

// Extract the session from the request.
#[derive(Clone, Serialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub user_id: i64,
    #[serde(with = "time::serde::iso8601")]
    pub last_accessed: time::OffsetDateTime,
    pub username: String,
    pub admin: bool,
}

// Middleware function to insert SessionInfo into the request extensions.ü+üß
pub async fn get_session(
    State(state): State<FreyaState>,
    mut request: Request,
    next: Next,
) -> Response {
    // Get session id from Authorization header.
    let session_id = match request
        .headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "))
    {
        Some(session_id) => session_id.to_string(),
        None => {
            return next.run(request).await;
        }
    };

    // Get connection from pool.
    let mut conn = match state.db.acquire().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Failed to acquire database connection: {}", e);
            return next.run(request).await;
        }
    };

    // Get session from database.
    let session = match sqlx::query_as!(
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
    .fetch_one(conn.as_mut())
    .await
    {
        Ok(session) => session,
        Err(_) => {
            return next.run(request).await;
        }
    };

    // Check if the session is expired.
    // If it is, delete it from the database and return.
    if session.last_accessed
        < time::OffsetDateTime::now_utc() - time::Duration::hours(*SESSION_LIFETIME as i64)
    {
        if let Err(err) = sqlx::query!(
            r#"
                DELETE FROM sessions
                WHERE id = $1
            "#,
            session_id,
        )
        .execute(conn.as_mut())
        .await
        {
            tracing::error!("Could not delete expired session: {}", err);
        }
        return next.run(request).await;
    }

    // Update session last access time in background.
    // To minimize writes we only update if the session was last accessed more than 6 hours ago.
    if session.last_accessed < time::OffsetDateTime::now_utc() - time::Duration::hours(6) {
        tokio::spawn(async move {
            if let Err(err) = sqlx::query!(
                r#"
                UPDATE sessions
                SET last_accessed = CURRENT_TIMESTAMP
                WHERE id = $1
            "#,
                session_id,
            )
            .execute(conn.as_mut())
            .await
            {
                tracing::error!("Could not update session last access time: {}", err);
            }
        });
    }

    // Insert session into request extensions.
    request.extensions_mut().insert(session);

    // Call next middleware.
    next.run(request).await
}

// Extract the session from the request.
pub struct Session(pub SessionInfo);

#[async_trait]
impl FromRequestParts<FreyaState> for Session {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &FreyaState,
    ) -> Result<Self, Self::Rejection> {
        // Get the session from the request extensions.
        match parts.extensions.get::<SessionInfo>() {
            Some(session) => Ok(Session(session.to_owned())),
            None => Err(
                (Json(json!({"error_code": "sever-authentication--not-logged-in"})))
                    .into_response_with_status(StatusCode::UNAUTHORIZED),
            ),
        }
    }
}

// Create session id.
pub fn create_session_id() -> String {
    random_string(SESSION_ID_ENTROPY)
}
