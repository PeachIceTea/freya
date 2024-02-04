use anyhow::Context;
use axum::{body::Bytes, extract::State, routing::get, Router};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use serde::Serialize;

use crate::{
    api_bail, data_response,
    models::Book,
    state::FreyaState,
    util::{
        cover::get_cover_bytes,
        ffmpeg::ffprobe_duration,
        response::{ApiError, ApiResult, DataResponse},
        session::{AdminSession, Session},
    },
};

pub fn router() -> Router<FreyaState> {
    Router::new().route("/", get(get_books).post(upload_book))
}

#[derive(Serialize)]
pub struct GetBooksResponse {
    books: Vec<Book>,
}

pub async fn get_books(
    State(state): State<FreyaState>,
    Session(_): Session,
) -> ApiResult<DataResponse<GetBooksResponse>> {
    // Get all books from the database.
    let books = sqlx::query_as!(
        Book,
        r#"
            SELECT id, title, author, created, modified
            FROM books
        "#
    )
    .fetch_all(&state.db)
    .await
    .context("Couldn't get books from database")?;

    data_response!(GetBooksResponse { books })
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

#[derive(Serialize)]
pub struct UploadBookResponse {
    book_id: i64,
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
) -> ApiResult<DataResponse<UploadBookResponse>> {
    // Trim user inputs.
    let title = title.trim();
    let author = author.trim();

    // Check if title, author, and files vector are not empty.
    if title.is_empty() || author.is_empty() || files.is_empty() {
        api_bail!(UploadMissingData)
    }

    // Check if each file path exists.
    for file in &files {
        if !std::path::Path::new(file).exists() {
            api_bail!(UploadInvalidFilePath, file.to_string())
        }
    }

    // Extract cover image.
    let cover = if let Some(cover) = cover {
        Some(
            get_cover_bytes(cover)
                .await
                .context(ApiError::FailedToGetCoverImage)?,
        )
    } else {
        None
    };

    // Create FileData vector from files.
    let mut file_data = Vec::with_capacity(files.len());
    for path in files {
        // Use ffprobe to get duration of file.
        let duration = ffprobe_duration(&path)
            .await
            .with_context(|| ApiError::FFProbeFailed(path.to_string()))?;

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
    let mut trx = state
        .db
        .begin()
        .await
        .context("Failed to start transaction")?;

    // Insert book into the database.
    let book_id = sqlx::query!(
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
    .context("Failed to insert book into database")?
    .id;

    // Insert files into the database.
    for (position, file) in file_data.iter().enumerate() {
        let position = position as i32 + 1;
        sqlx::query!(
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
        .context("Failed to insert file into database")?;
    }

    // Commit transaction.
    trx.commit().await.context("Failed to commit transaction")?;

    data_response!(UploadBookResponse { book_id })
}
