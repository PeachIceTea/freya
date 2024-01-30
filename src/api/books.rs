use axum::{extract::State, response::IntoResponse, routing::get, Router};

use crate::{axum_json, models::Book, state::FreyaState};

pub fn router() -> Router<FreyaState> {
    Router::new().route("/", get(get_books))
}

pub async fn get_books(State(state): State<FreyaState>) -> impl IntoResponse {
    // Get all books from the database.
    let books = match sqlx::query_as!(
        Book,
        r#"
            SELECT *
            FROM books
        "#
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(books) => books,
        Err(e) => {
            tracing::error!("Failed to get books: {}", e);
            return axum_json!({
                "error_code": "server-database--query-failed",
            });
        }
    };

    axum_json!({
        "books": books,
    })
}
