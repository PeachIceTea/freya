use anyhow::Context;
use axum::{
    extract::State,
    routing::{delete, get, post},
    Json, Router,
};
use tower_cookies::Cookies;

use crate::{
    api_bail, api_response, data_response,
    models::User,
    state::FreyaState,
    util::{
        password::verify_password,
        response::{ApiError, ApiResult, DataResponse, SuccessResponse},
        session::{
            create_session_cookie, create_session_id, delete_session_cookie, Session, SessionInfo,
        },
    },
};

pub fn build_router() -> Router<FreyaState> {
    Router::new()
        .route("/login", post(login))
        .route("/logout", delete(logout))
        .route("/info", get(info))
}

#[derive(serde::Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

pub async fn login(
    cookies: Cookies,
    session: Option<Session>,
    State(state): State<FreyaState>,
    Json(data): Json<LoginRequest>,
) -> ApiResult<SuccessResponse> {
    // Check if the user is already logged in.
    if session.is_some() {
        api_bail!(AlreadyLoggedIn)
    }

    // Normalize inputs.
    let username = data.username.trim().to_lowercase();
    let password = data.password.trim();

    // Check if both username and password are not empty.
    if username.is_empty() || password.is_empty() {
        api_bail!(InvalidCredentials)
    }

    // Get the user from the database.
    let user = sqlx::query_as!(
        User,
        r#"
            SELECT *
            FROM users
            WHERE name = $1
        "#,
        username
    )
    .fetch_one(&state.db)
    .await
    .context(ApiError::InvalidCredentials)?;

    // Check if the password is correct.
    if !verify_password(&user.password, &data.password) {
        api_bail!(InvalidCredentials)
    }

    // Create a new session.
    let session_id = create_session_id();
    sqlx::query!(
        r#"
            INSERT INTO sessions (id, user_id)
            VALUES ($1, $2)
        "#,
        session_id,
        user.id
    )
    .execute(&state.db)
    .await
    .context("Failed to create session in database")?;

    // Set the session cookie.
    cookies.add(create_session_cookie(
        &session_id,
        time::OffsetDateTime::now_utc(),
    ));

    api_response!("server-authentication--logged-in")
}

pub async fn logout(
    cookies: Cookies,
    State(state): State<FreyaState>,
    Session(session): Session,
) -> ApiResult<SuccessResponse> {
    // Remove the session from the database.
    sqlx::query!(
        r#"
            DELETE FROM sessions
            WHERE id = $1
        "#,
        session.session_id
    )
    .execute(&state.db)
    .await
    .context("Couldn't delete session from database")?;

    // Delete the session cookie.
    cookies.remove(delete_session_cookie());

    api_response!("server-authentication--logged-out")
}

pub async fn info(Session(session): Session) -> ApiResult<DataResponse<SessionInfo>> {
    data_response!(session)
}
