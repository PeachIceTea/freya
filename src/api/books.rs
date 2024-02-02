use axum::{body::Bytes, extract::State, response::IntoResponse, routing::get, Router};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};

use crate::{
    axum_json,
    models::Book,
    state::FreyaState,
    util::{cover::get_cover_bytes, ffmpeg::ffprobe_duration, session::AdminSession},
};

pub fn router() -> Router<FreyaState> {
    Router::new().route("/", get(get_books).post(upload_book))
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

struct FileData {
    path: String,
    name: String,
    duration: f64,
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
        match get_cover_bytes(cover).await {
            Ok(cover) => Some(cover),
            Err(e) => {
                tracing::error!("Failed to get cover image: {}", e);
                return axum_json!({
                    "error_code": "server-books--failed-to-get-cover-image",
                });
            }
        }
    } else {
        None
    };

    // Create FileData vector from files.
    let mut file_data = Vec::with_capacity(files.len());
    for path in files {
        // Use ffprobe to get duration of file.
        let duration = match ffprobe_duration(&path).await {
            Ok(duration) => duration,
            Err(e) => {
                tracing::error!("Failed to get file info: {}", e);
                return axum_json!({
                    "error_code": "server-books--failed-to-get-file-info",
                    "value": path,
                });
            }
        };

        // Get file name from file path.
        let name = std::path::Path::new(&path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        file_data.push(FileData {
            path,
            name,
            duration,
        });
    }

    // Sort file data by name.
    file_data.sort_by(|a, b| a.name.cmp(&b.name));

    // Insert book into the database.
    let mut trx = match state.db.begin().await {
        Ok(trx) => trx,
        Err(e) => {
            tracing::error!("Failed to start transaction: {}", e);
            return axum_json!({
                "error_code": "server-database--transaction-failed",
            });
        }
    };

    // Insert book into the database.
    let book_id = match sqlx::query!(
        r#"
            INSERT INTO books (title, author, cover)
            VALUES (?, ?, ?)
            RETURNING id
        "#,
        title,
        author,
        cover,
    )
    .fetch_one(&mut *trx)
    .await
    {
        Ok(book) => book.id,
        Err(e) => {
            tracing::error!("Failed to insert book: {}", e);
            return axum_json!({
                "error_code": "server-database--query-failed",
            });
        }
    };

    // Insert files into the database.
    for (position, file) in file_data.iter().enumerate() {
        let position = position as i32 + 1;
        if let Err(e) = sqlx::query!(
            r#"
                INSERT INTO files (book_id, path, name, position, duration)
                VALUES (?, ?, ?, ?, ?)
            "#,
            book_id,
            file.path,
            file.name,
            position,
            file.duration,
        )
        .execute(&mut *trx)
        .await
        {
            tracing::error!("Failed to insert file: {}", e);
            return axum_json!({
                "error_code": "server-database--query-failed",
            });
        }
    }

    // Commit transaction.
    if let Err(e) = trx.commit().await {
        tracing::error!("Failed to commit transaction: {}", e);
        return axum_json!({
            "error_code": "server-database--transaction-failed",
        });
    }

    axum_json!({
        "book_id": book_id,
    })
}
