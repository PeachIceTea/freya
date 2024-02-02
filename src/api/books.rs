use axum::{
    body::Bytes,
    extract::{Multipart, State},
    response::IntoResponse,
    routing::get,
    Router,
};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};

use crate::{
    axum_json,
    models::Book,
    state::FreyaState,
    util::{cover::Cover, session::AdminSession},
};

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

#[derive(TryFromMultipart)]
pub struct UploadBook {
    title: String,
    author: String,
    cover: Option<FieldData<Bytes>>,
    files: Vec<String>,
}

pub async fn upload_book(
    State(state): State<FreyaState>,
    AdminSession(_): AdminSession,
    TypedMultipart(UploadBook {
        title,
        author,
        cover,
        files,
    }): TypedMultipart<UploadBook>,
) -> impl IntoResponse {
    // Trim user inputs.
    let title = title.trim();
    let author = author.trim();

    // Check if title, author, and files vector are not empty.
    if title.is_empty() || author.is_empty() || files.is_empty() {
        return axum_json!({
            "error_code": "server-books--missing-data",
        });
    }

    // Check if each file path exists.
    for file in &files {
        if !std::path::Path::new(file).exists() {
            return axum_json!({
                "error_code": "server-books--invalid-file-path",
                "value": file,
            });
        }
    }

    // Extract cover image.
    let cover = if let Some(cover) = cover {
        todo!();
        Some(())
    } else {
        None
    };

    axum_json!({
        "id": "TODO",
    })
}
