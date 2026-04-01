use anyhow::Context;
use axum::{
    Json, Router, debug_handler,
    extract::State,
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};

use super::response::{ApiError, ApiResult, DataResponse, SuccessResponse};
use crate::{
    api_bail, api_response,
    auth::{
        password::verify_password,
        session::{Session, create_session_id},
    },
    data_response,
    database::session::SessionInfo,
    state::FelaState,
};

/// Build router for authentication.
/// Is attached to `/`, **NOT** `/auth`.
pub fn router() -> Router<FelaState> {
    Router::new()
        .route("/login", post(login))
        .route("/logout", delete(logout))
        .route("/info", get(info))
}

/// Data expected during a login request.
#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

/// Reponse to login request handing over the session id.
#[derive(Serialize)]
pub struct LoginResponse {
    token: String,
}

/// Create new session and get session token.
#[debug_handler]
pub async fn login(
    session: Option<Session>,
    State(state): State<FelaState>,
    Json(data): Json<LoginRequest>,
) -> ApiResult<DataResponse<LoginResponse>> {
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
    let user = state
        .database
        .get_user_with_password(&username)
        .await
        .context(ApiError::InvalidCredentials)?;

    // Check if the password is correct.
    if !verify_password(
        &user
            .password
            .with_context(|| "get_user_with_password did not return a password")?,
        &data.password,
    ) {
        api_bail!(InvalidCredentials)
    }

    // Create a new session.
    let session_id = create_session_id();
    state
        .database
        .create_session(user.id, &session_id)
        .await
        .context("Failed to create session in database")?;

    data_response!(LoginResponse { token: session_id })
}

/// Remove the session from the database.
pub async fn logout(
    State(state): State<FelaState>,
    Session(session): Session,
) -> ApiResult<SuccessResponse> {
    state.database.delete_session(&session.session_id).await?;

    api_response!("server-authentication--logged-out")
}

/// Return session info to client.
/// Used to check if session is still alive.
pub async fn info(Session(session): Session) -> ApiResult<DataResponse<SessionInfo>> {
    data_response!(session)
}
