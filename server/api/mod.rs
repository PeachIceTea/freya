mod account;
mod admin;
mod authentication;
mod books;
mod fs;
pub mod response;
mod user;

use axum::{Json, Router, http::StatusCode, middleware, response::IntoResponse, routing::get};
use serde::Serialize;
use tower_http::trace::TraceLayer;

use crate::{auth::session::get_session, state::FelaState};

pub async fn build_router(state: FelaState) -> Router {
    Router::new()
        .fallback(route_not_found)
        .route("/", get(greet))
        .merge(authentication::build_router())
        .nest("/book", books::router())
        .nest("/user", user::router())
        .nest("/fs", fs::router())
        .nest("/account", account::router())
        .nest("/admin", admin::router())
        .route_layer(middleware::from_fn_with_state(state.clone(), get_session))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[derive(Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
struct Greeting {
    success: bool,
    message: &'static str,
}

static GREETING: Json<Greeting> = Json(Greeting {
    success: true,
    message: "Welcome to Fela!",
});
pub async fn greet() -> impl IntoResponse {
    GREETING
}

#[derive(Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
struct NotFound {
    success: bool,
    error_code: &'static str,
}
static NOT_FOUND: (StatusCode, Json<NotFound>) = (
    StatusCode::NOT_FOUND,
    Json(NotFound {
        success: false,
        error_code: "NotFound",
    }),
);

pub async fn route_not_found() -> impl IntoResponse {
    NOT_FOUND
}
