use anyhow::Context;
use axum::{extract::State, routing::get, Router};

use crate::{
    data_response,
    models::User,
    state::FreyaState,
    util::{
        response::{ApiResult, DataResponse},
        session::Session,
    },
};

pub fn router() -> Router<FreyaState> {
    Router::new().route("/", get(get_users))
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
