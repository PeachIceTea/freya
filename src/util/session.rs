use axum::{
    async_trait,
    extract::{FromRequestParts, Request, State},
    http::request::Parts,
    middleware::Next,
    response::Response,
};
use serde::Serialize;
use time::OffsetDateTime;
use tower_cookies::{cookie::SameSite, Cookie, Cookies};

use crate::{api_bail, state::FreyaState};

use super::{random::random_string, response::ApiError};

// Bytes of entropy in the session id.
pub static SESSION_ID_ENTROPY: usize = 32;

// Cookie name for the session id.
pub static SESSION_COOKIE_NAME: &str = "freya_session";

// Session lifetime as a time::Duration.
// Read SESSION_LIFETIME from environment variable using a lazy once_cell.
// SESSION_LIFETIME is in hours.
// Default to 30 days.
pub static SESSION_LIFETIME: once_cell::sync::Lazy<time::Duration> =
    once_cell::sync::Lazy::new(|| {
        if let Ok(session_lifetime) = std::env::var("SESSION_LIFETIME") {
            time::Duration::hours(
                session_lifetime
                    .parse::<i64>()
                    .expect("SESSION_LIFETIME environment variable should be an integer"),
            )
        } else {
            time::Duration::days(30)
        }
    });

// Cookie secure flag.
// Read COOKIE_ONLY_OVER_HTTPS from environment variable using a lazy once_cell.
// Default to false.
pub static COOKIE_ONLY_OVER_HTTPS: once_cell::sync::Lazy<bool> = once_cell::sync::Lazy::new(|| {
    if let Ok(cookie_only_over_https) = std::env::var("COOKIE_ONLY_OVER_HTTPS") {
        cookie_only_over_https
            .parse()
            .expect("COOKIE_ONLY_OVER_HTTPS should be a boolean")
    } else {
        false
    }
});

// Extract the session from the request.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    #[serde(skip)]
    pub session_id: String,
    pub user_id: i64,
    #[serde(with = "time::serde::iso8601")]
    last_accessed: time::OffsetDateTime,
    username: String,
    pub admin: bool,
}

// Middleware function to insert SessionInfo into the request extensions.ü+üß
pub async fn get_session(
    State(state): State<FreyaState>,
    cookies: Cookies,
    mut request: Request,
    next: Next,
) -> Response {
    // Get session id from cookie.
    let session_id = match cookies
        .get(SESSION_COOKIE_NAME)
        .map(|cookie| cookie.value().to_string())
    {
        Some(session_id) => session_id,
        None => return next.run(request).await,
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
            cookies.remove(Cookie::new(SESSION_COOKIE_NAME, ""));
            return next.run(request).await;
        }
    };

    // Check if the session is expired.
    // If it is, delete it from the database and return.
    if session.last_accessed < time::OffsetDateTime::now_utc() - *SESSION_LIFETIME {
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
        cookies.remove(Cookie::new(SESSION_COOKIE_NAME, ""));
        return next.run(request).await;
    }

    // Update session last access time in background.
    // To minimize writes we only update if the session was last accessed more than 6 hours ago.
    if session.last_accessed < time::OffsetDateTime::now_utc() - time::Duration::hours(6) {
        // Set cookie with  new last accessed time.
        cookies.add(create_session_cookie(
            &session.session_id,
            OffsetDateTime::now_utc(),
        ));

        // Spawn task to update session last access time.
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
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &FreyaState,
    ) -> Result<Self, Self::Rejection> {
        // Get the session from the request extensions.
        parts.extensions.get::<SessionInfo>().map_or_else(
            || api_bail!(NotLoggedIn),
            |session| Ok(Session(session.to_owned())),
        )
    }
}

// Extract the session from the request if user is an admin.
pub struct AdminSession(pub SessionInfo);

#[async_trait]
impl FromRequestParts<FreyaState> for AdminSession {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &FreyaState,
    ) -> Result<Self, Self::Rejection> {
        // Get session from Session extractor.
        let session = Session::from_request_parts(parts, _state).await?.0;

        // Check if user is an admin.
        if !session.admin {
            api_bail!(NotAdmin)
        }

        Ok(AdminSession(session))
    }
}

// Create session id.
pub fn create_session_id() -> String {
    random_string(SESSION_ID_ENTROPY)
}

// Create session cookie.
pub fn create_session_cookie<'a>(session_id: &str, last_accessed: OffsetDateTime) -> Cookie<'a> {
    Cookie::build((SESSION_COOKIE_NAME, session_id.to_string()))
        .path("/")
        .http_only(true)
        .secure(*COOKIE_ONLY_OVER_HTTPS)
        .same_site(SameSite::Lax)
        .expires(last_accessed + *SESSION_LIFETIME)
        .build()
}
