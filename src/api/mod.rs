pub mod authentication;
pub mod books;

use axum::{http::StatusCode, middleware::from_fn_with_state, response::IntoResponse, Router};
use tower_http::trace::TraceLayer;

use crate::{
    axum_json,
    state::FreyaState,
    util::{response::IntoResponseWithStatus, session::get_session},
};

pub async fn build_router(state: FreyaState) -> Router {
    let api = Router::new()
        .merge(authentication::build_router())
        .nest("/book", books::router());

    Router::new()
        .nest("/api", api)
        .fallback(route_not_found)
        .layer(from_fn_with_state(state.clone(), get_session))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

pub async fn route_not_found() -> impl IntoResponse {
    axum_json!({
        "error": "Route not found"
    })
    .into_response_with_status(StatusCode::NOT_FOUND)
}
