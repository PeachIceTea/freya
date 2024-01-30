use axum::{
    extract::State,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};

use crate::{
    axum_json,
    models::User,
    state::FreyaState,
    util::{
        password::verify_password,
        session::{create_session_id, Session},
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
    pub username: String,
    pub password: String,
}

pub async fn login(
    State(state): State<FreyaState>,
    Json(data): Json<LoginRequest>,
) -> impl IntoResponse {
    // Normalize inputs.
    let username = data.username.trim().to_lowercase();
    let password = data.password.trim();

    // Check if both username and password are not empty.
    if username.is_empty() || password.is_empty() {
        return axum_json!({
            "error_code": "server-authentication--missing-data",
        });
    }

    // Get the user from the database.
    let user = match sqlx::query_as!(
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
    {
        Ok(user) => user,
        Err(_) => {
            return axum_json!({
                "error_code": "server-authentication--invalid-credentials",
            });
        }
    };

    // Check if the password is correct.
    if !verify_password(&user.password, &data.password) {
        return axum_json!({
            "error_code": "server-authentication--invalid-credentials",
        });
    }

    // Create a new session.
    let session_id = create_session_id();
    if let Err(err) = sqlx::query!(
        r#"
            INSERT INTO sessions (id, user_id)
            VALUES ($1, $2)
        "#,
        session_id,
        user.id
    )
    .execute(&state.db)
    .await
    {
        tracing::error!("Could not create session: {}", err);
        return axum_json!({
            "error_code": "server-error--internal",
        });
    }

    axum_json!({
        "session_id": session_id,
    })
}

pub async fn logout(
    State(state): State<FreyaState>,
    Session(session): Session,
) -> impl IntoResponse {
    if let Err(err) = sqlx::query!(
        r#"
            DELETE FROM sessions
            WHERE id = $1
        "#,
        session.session_id
    )
    .execute(&state.db)
    .await
    {
        tracing::error!("Could not delete session: {}", err);
        return axum_json!({
            "error_code": "server-error--internal",
        });
    }

    axum_json!({
        "success": true,
        "msg": "server-authentication--logged-out"
    })
}

pub async fn info(Session(session): Session) -> impl IntoResponse {
    axum_json!({
        "session": session,
    })
}
