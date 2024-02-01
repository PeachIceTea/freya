mod authentication;
mod books;
mod fs;

use axum::{http::StatusCode, middleware, response::IntoResponse, Router};
use tower_cookies::CookieManagerLayer;
use tower_http::trace::TraceLayer;

use crate::{
    axum_json,
    state::FreyaState,
    util::{response::IntoResponseWithStatus, session::get_session},
};

pub async fn build_router(state: FreyaState) -> Router {
    let api = Router::new()
        .merge(authentication::build_router())
        .nest("/book", books::router())
        .nest("/fs", fs::router());

    Router::new()
        .nest("/api", api)
        .fallback(route_not_found)
        .route_layer(middleware::from_fn_with_state(state.clone(), get_session))
        .route_layer(CookieManagerLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

pub async fn route_not_found() -> impl IntoResponse {
    axum_json!({
        "error": "Route not found"
    })
    .into_response_with_status(StatusCode::NOT_FOUND)
}
