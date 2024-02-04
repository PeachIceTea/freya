mod authentication;
mod books;
mod fs;

use axum::{http::StatusCode, middleware, response::IntoResponse, routing::get, Json, Router};
use serde::Serialize;
use tower_cookies::CookieManagerLayer;
use tower_http::trace::TraceLayer;

use crate::{state::FreyaState, util::session::get_session};

pub async fn build_router(state: FreyaState) -> Router {
    Router::new()
        .fallback(route_not_found)
        .route("/", get(greet))
        .merge(authentication::build_router())
        .nest("/book", books::router())
        .nest("/fs", fs::router())
        .route_layer(middleware::from_fn_with_state(state.clone(), get_session))
        .route_layer(CookieManagerLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[derive(Serialize, Clone, Copy)]
struct Greeting {
    success: bool,
    message: &'static str,
}

static GREETING: Json<Greeting> = Json(Greeting {
    success: true,
    message: "Welcome to Freya!",
});
pub async fn greet() -> impl IntoResponse {
    GREETING
}

#[derive(Serialize, Clone, Copy)]
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
