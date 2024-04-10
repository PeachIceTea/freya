use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;

use crate::{
    api_bail, api_response, data_response,
    database::user::User,
    state::FreyaState,
    util::{
        password::hash_password,
        response::{ApiResult, DataResponse, SuccessResponse},
        session::{AdminSession, Session},
    },
};

pub fn router() -> Router<FreyaState> {
    Router::new()
        .route("/", get(get_users).post(create_user))
        .route("/:id", get(get_user).patch(update_user))
}

pub async fn get_users(
    Session(_): Session,
    State(state): State<FreyaState>,
) -> ApiResult<DataResponse<Vec<User>>> {
    // Get all users.
    let users = state.database.get_all_users().await?;

    data_response!(users)
}

pub async fn get_user(
    Session(_): Session,
    State(state): State<FreyaState>,
    Path(id): Path<i64>,
) -> ApiResult<DataResponse<User>> {
    // Get user by id.
    let user = state.database.get_user(id).await?;

    data_response!(user)
}

#[derive(Deserialize)]
pub struct CreateUserRequest {
    name: String,
    password: String,
    admin: bool,
}
pub async fn create_user(
    AdminSession(_): AdminSession,
    State(state): State<FreyaState>,
    Json(body): Json<CreateUserRequest>,
) -> ApiResult<SuccessResponse> {
    // Noramlize username.
    let username = body.name.trim().to_lowercase();

    // Check if both username and password exists.
    if username.is_empty() || body.password.is_empty() {
        api_bail!(DataMissing)
    }

    // Hash password.
    let password = hash_password(&body.password)?;

    // Insert user into database.
    state
        .database
        .create_user(&username, &password, body.admin)
        .await?;

    api_response!("user-create--success")
}
#[derive(Deserialize)]
pub struct UpdateUserRequest {
    name: Option<String>,
    password: Option<String>,
    admin: Option<bool>,
}

pub async fn update_user(
    Session(session): Session,
    State(state): State<FreyaState>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateUserRequest>,
) -> ApiResult<SuccessResponse> {
    // Check if user is allowed to update user.
    if session.user_id != id && !session.admin {
        api_bail!(NotAdmin);
    }

    // Normalize username if it's provided.
    let username = body.name.map(|name| name.trim().to_lowercase());

    // Hash password if it's provided.
    let password = if let Some(password) = body.password {
        Some(hash_password(&password)?)
    } else {
        None
    };

    // Update user by id.
    state
        .database
        .update_user(id, username, password, body.admin)
        .await?;

    api_response!("user-edit--success")
}
