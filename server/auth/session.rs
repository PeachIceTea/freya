use axum::{
    extract::{FromRequestParts, OptionalFromRequestParts, Request, State},
    http::request::Parts,
    middleware::Next,
    response::Response,
};

use crate::{api_bail, database::session::SessionInfo, state::FelaState};

use super::random::random_string;
use crate::api::response::ApiError;

// Bytes of entropy in the session id.
pub static SESSION_ID_ENTROPY: usize = 32;

// Session lifetime as a time::Duration.
// Read FELA_SESSION_LIFETIME from environment variable using a lazy once_cell.
// FELA_SESSION_LIFETIME is in hours.
// Default to 30 days.
pub static SESSION_LIFETIME: std::sync::LazyLock<time::Duration> = std::sync::LazyLock::new(|| {
    if let Ok(session_lifetime) = std::env::var("FELA_SESSION_LIFETIME") {
        time::Duration::hours(
            session_lifetime
                .parse::<i64>()
                .expect("SESSION_LIFETIME environment variable should be an integer"),
        )
    } else {
        time::Duration::days(30)
    }
});

/// Middleware function to insert SessionInfo into the request extensions.
pub async fn get_session(
    State(state): State<FelaState>,
    mut request: Request,
    next: Next,
) -> Response {
    // Get session id from auth header.
    let session_id = {
        let Some(auth_header) = request.headers().get("Authorization") else {
            return next.run(request).await;
        };

        let Ok(auth_value) = auth_header.to_str() else {
            return next.run(request).await;
        };

        let Some((auth_type, session_id)) = auth_value.split_once(" ") else {
            return next.run(request).await;
        };

        if auth_type != "Bearer" {
            return next.run(request).await;
        }

        session_id
    };

    // Get session from database.
    let Ok(session) = state.database.get_session(session_id).await else {
        return next.run(request).await;
    };

    // Check if the session is expired.
    // If it is, delete it from the database and return.
    if session.last_accessed < time::OffsetDateTime::now_utc() - *SESSION_LIFETIME {
        tokio::spawn(async move {
            if let Err(err) = state.database.delete_session(&session.session_id).await {
                tracing::error!("{}", err)
            }
        });

        return next.run(request).await;
    }

    // Update session last access time in background.
    // To minimize writes we only update if the session was last accessed more than 6 hours ago.
    if session.last_accessed < time::OffsetDateTime::now_utc() - time::Duration::hours(6) {
        // Spawn task to update session last access time.
        let session_id = session.session_id.clone();
        tokio::spawn(async move { state.database.update_session_timestamp(&session_id).await });
    }

    // Insert session into request extensions.
    request.extensions_mut().insert(session);

    // Call next middleware.
    next.run(request).await
}

/// Extract the session from the request.
pub struct Session(pub SessionInfo);

impl FromRequestParts<FelaState> for Session {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &FelaState,
    ) -> Result<Self, Self::Rejection> {
        // Get the session from the request extensions.
        parts.extensions.get::<SessionInfo>().map_or_else(
            || api_bail!(NotLoggedIn),
            |session| Ok(Session(session.to_owned())),
        )
    }
}

impl OptionalFromRequestParts<FelaState> for Session {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &FelaState,
    ) -> Result<Option<Self>, Self::Rejection> {
        Ok(parts
            .extensions
            .get::<SessionInfo>()
            .map(|session| Session(session.to_owned())))
    }
}

/// Extract the session from the request if user is an admin.
pub struct AdminSession(pub SessionInfo);

impl FromRequestParts<FelaState> for AdminSession {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &FelaState,
    ) -> Result<Self, Self::Rejection> {
        // Get session from Session extractor.
        let session =
            <self::Session as axum::extract::FromRequestParts<FelaState>>::from_request_parts(
                parts, _state,
            )
            .await?
            .0;

        // Check if user is an admin.
        if !session.admin {
            api_bail!(NotAdmin)
        }

        Ok(AdminSession(session))
    }
}

/// Create session id.
pub fn create_session_id() -> String {
    random_string(SESSION_ID_ENTROPY)
}
