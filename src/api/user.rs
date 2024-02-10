use anyhow::Context;
use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::{
    api_bail, api_response, data_response,
    models::User,
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
        .route("/:id/library", get(get_library))
}

pub async fn get_users(
    Session(_): Session,
    State(state): State<FreyaState>,
) -> ApiResult<DataResponse<Vec<User>>> {
    // Get all users.
    let users = sqlx::query_as!(User, "SELECT * FROM users")
        .fetch_all(&state.db)
        .await
        .context("Failed to fetch users")?;

    data_response!(users)
}

pub async fn get_user(
    Session(_): Session,
    State(state): State<FreyaState>,
    Path(id): Path<i64>,
) -> ApiResult<DataResponse<User>> {
    // Get user by id.
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = ?", id)
        .fetch_one(&state.db)
        .await
        .context("Failed to fetch user")?;

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
    // Hash password.
    let password = hash_password(&body.password)?;

    // Insert user into database.
    sqlx::query!(
        "INSERT INTO users (name, password, admin) VALUES (?, ?, ?)",
        body.name,
        password,
        body.admin
    )
    .execute(&state.db)
    .await
    .context("Failed to create user")?;

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

    // Hash password if it's provided.
    let password = if let Some(password) = body.password {
        Some(hash_password(&password)?)
    } else {
        None
    };

    // Update user by id.
    sqlx::query_as!(
        User,
        r#"
        UPDATE users
        SET
            name = COALESCE(?, name),
            password = COALESCE(?, password),
            admin = COALESCE(?, admin)
        WHERE id = ?"#,
        body.name,
        password,
        body.admin,
        id
    )
    .fetch_one(&state.db)
    .await
    .context("Failed to update user")?;

    api_response!("user-edit--success")
}

#[derive(Serialize)]
pub struct LibraryResponse {
    id: i64,
    title: String,
    author: String,
    list: String,
    // There isn't actually a way for progress to be null, but sqlx apparently can't guarantee that.
    progress: Option<f64>,
}

pub async fn get_library(
    Session(_): Session,
    State(state): State<FreyaState>,
    Path(id): Path<i64>,
) -> ApiResult<DataResponse<Vec<LibraryResponse>>> {
    // Get library by user id.
    let library = sqlx::query_as!(
        LibraryResponse,
        // Not the prettiest query, but it works.
        r#"
        WITH total_duration AS (
            SELECT book_id, SUM(duration) as total_duration
            FROM files
            GROUP BY book_id
        )
        SELECT
            books.id,
            books.title,
            books.author,
            library_entries.list,
            COALESCE((
                SELECT (SUM(file_sub.duration) + library_entries.progress) / total_duration.total_duration
                FROM files file_sub
                WHERE
                    files.book_id = books.id
                    AND file_sub.position < files.position
            ), library_entries.progress / total_duration.total_duration) as "progress: f64"
        FROM books
        JOIN library_entries ON library_entries.book_id = books.id
        JOIN files ON library_entries.file_id = files.id
        JOIN total_duration ON total_duration.book_id = books.id
        WHERE library_entries.user_id = ?
        ORDER BY library_entries.modified DESC
        "#,
        id
    )
    .fetch_all(&state.db)
    .await
    .context("Failed to fetch library")?;

    data_response!(library)
}
