mod account;
mod admin;
mod authentication;
mod books;
mod fs;
pub mod response;
mod user;

use axum::{Router, middleware, routing::get};
use tower_http::trace::TraceLayer;

use crate::{
    api::response::{ApiResult, SuccessResponse},
    api_bail, api_response,
    auth::session::get_session,
    state::FelaState,
};

pub async fn build_router(state: FelaState) -> Router {
    Router::new()
        .fallback(route_not_found)
        .route("/", get(greet))
        .merge(authentication::router())
        .nest("/book", books::router())
        .nest("/user", user::router())
        .nest("/fs", fs::router())
        .nest("/account", account::router())
        .nest("/admin", admin::router())
        .route_layer(middleware::from_fn_with_state(state.clone(), get_session))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
pub async fn greet() -> ApiResult<SuccessResponse> {
    api_response!("Welcome to Fela!")
}

pub async fn route_not_found() -> ApiResult<()> {
    api_bail!(NotFound)
}
