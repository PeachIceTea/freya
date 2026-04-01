use axum::{
    Router,
    extract::{Path, State},
    routing::get,
};

use super::response::{ApiResult, DataResponse};
use crate::{
    auth::session::Session, data_response, database::library::LibraryResponse, state::FelaState,
};

/// Build router for user routes.
/// Is attached to `/user`.
pub fn router() -> Router<FelaState> {
    Router::new().route("/{id}/library", get(get_library))
}

/// Get library of a user.
/// TODO: Allow other users to see the progress others have made.
pub async fn get_library(
    Session(_): Session,
    State(state): State<FelaState>,
    Path(id): Path<i64>,
) -> ApiResult<DataResponse<Vec<LibraryResponse>>> {
    // Get library by user id.
    let library = state.database.get_user_library(id).await?;

    data_response!(library)
}
