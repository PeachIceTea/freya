use axum::{
    Router,
    extract::{Path, State},
    routing::get,
};

use crate::{
    data_response,
    auth::session::Session,
    database::library::LibraryResponse,
    state::FreyaState,
};
use super::response::{ApiResult, DataResponse};

pub fn router() -> Router<FreyaState> {
    Router::new().route("/{id}/library", get(get_library))
}

pub async fn get_library(
    Session(_): Session,
    State(state): State<FreyaState>,
    Path(id): Path<i64>,
) -> ApiResult<DataResponse<Vec<LibraryResponse>>> {
    // Get library by user id.
    let library = state.database.get_user_library(id).await?;

    data_response!(library)
}
