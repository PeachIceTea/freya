use std::fmt::{Display, Formatter};

use anyhow::Context;
use axum::{
    body::Bytes,
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use serde::{Deserialize, Serialize};

use crate::{
    api_bail, api_response, data_response,
    models::{Book, Chapter, File, LibraryEntry},
    state::FreyaState,
    util::{
        cover::get_cover_bytes,
        ffmpeg::{ffprobe_chapters, ffprobe_duration},
        response::{ApiError, ApiFileResult, ApiResult, DataResponse, SuccessResponse},
        session::{AdminSession, Session},
    },
};

pub fn router() -> Router<FreyaState> {
    Router::new()
        .route("/", get(get_books).post(upload_book))
        .route("/:book_id", get(get_book_details))
        .route("/:book_id/cover", get(get_book_cover))
        .route("/:book_id/library", post(set_book_list))
        .route("/:book_id/progress", post(update_progress))
}

pub async fn get_books(
    State(state): State<FreyaState>,
    Session(_): Session,
) -> ApiResult<DataResponse<Vec<Book>>> {
    // Get all books from the database.
    let books = sqlx::query_as!(
        Book,
        r#"
            SELECT
                id,
                title,
                author,
                created,
                modified,
                NULL AS "duration: f64"
            FROM books
            ORDER BY title ASC
        "#
    )
    .fetch_all(&state.db)
    .await
    .context("Couldn't get books from database")?;

    data_response!(books)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookResponse {
    #[serde(flatten)]
    book: Book,
    files: Vec<File>,
    #[serde(skip_serializing_if = "Option::is_none")]
    library: Option<LibraryEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    chapters: Vec<Chapter>,
}

pub async fn get_book_details(
    Session(session): Session,
    Path(book_id): Path<i64>,
    State(state): State<FreyaState>,
) -> ApiResult<DataResponse<BookResponse>> {
    // Get book from the database.
    let book = sqlx::query_as!(
        Book,
        r#"
            SELECT
                id,
                title,
                author,
                created,
                modified,
                (
                    SELECT SUM(duration)
                    FROM files
                    WHERE book_id = books.id
                ) AS "duration: f64"
            FROM books
            WHERE id = ?
        "#,
        book_id
    )
    .fetch_optional(&state.db)
    .await
    .context("Couldn't get book from database")?
    .ok_or(ApiError::NotFound)?;

    // Get files from the database.
    let files = sqlx::query_as!(
        File,
        r#"
            SELECT *
            FROM files
            WHERE book_id = ?
            ORDER BY position ASC
        "#,
        book_id
    )
    .fetch_all(&state.db)
    .await
    .context("Couldn't get files from database")?;

    // Get chapters from the database.
    let chapters = sqlx::query_as!(
        Chapter,
        r#"
            SELECT *
            FROM chapters
            WHERE book_id = ?
            ORDER BY start ASC
        "#,
        book_id
    )
    .fetch_all(&state.db)
    .await
    .context("Couldn't get chapters from database")?;

    // Get library entry from the database.
    let library = sqlx::query_as!(
        LibraryEntry,
        r#"
            SELECT *
            FROM library_entries
            WHERE user_id = ?
            AND book_id = ?
        "#,
        session.user_id,
        book_id
    )
    .fetch_optional(&state.db)
    .await
    .context("Couldn't get library entry from database")?;

    data_response!(BookResponse {
        book,
        files,
        library,
        chapters,
    })
}

pub async fn get_book_cover(
    Session(_): Session,
    Path(book_id): Path<i64>,
    State(state): State<FreyaState>,
) -> ApiFileResult<Vec<u8>> {
    // Get cover image from the database.
    let result = sqlx::query!(
        r#"
            SELECT cover
            FROM books
            WHERE id = ?
        "#,
        book_id
    )
    .fetch_optional(&state.db)
    .await;

    let placeholder_cover = include_bytes!("../../placeholder-cover.jpg");

    // Check if the query failed.
    match result {
        Ok(Some(record)) => match record.cover {
            Some(cover) => Ok(cover.to_vec()),
            None => Ok(placeholder_cover.to_vec()),
        },
        Ok(None) => Ok(placeholder_cover.to_vec()),
        Err(err) => {
            tracing::error!("Failed to get cover image from database: {:?}", err);
            Ok(placeholder_cover.to_vec())
        }
    }
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
#[serde(rename_all = "camelCase")]
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
        let cover_data = get_cover_bytes(cover).await;
        if let Ok(cover_data) = cover_data {
            Some(cover_data)
        } else {
            tracing::debug!("Failed to get cover image bytes: {:?}", cover_data);
            api_bail!(FailedToGetCoverImage)
        }
    } else {
        None
    };

    // Extract chapters if only one file is uploaded.
    // This is entirely optional and will not fail the upload if it fails.
    let chapters = if files.len() == 1 {
        let file = &files[0];
        let chapters = ffprobe_chapters(file).await.ok();
        chapters
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
        // Remove file extension and replace underscores with spaces.
        let name = std::path::Path::new(&path)
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.replace('_', " "))
            .unwrap_or_else(|| path.clone());

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
        let position = position as i64 + 1;
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

    // Insert chapters into the database.
    if let Some(chapters) = chapters {
        for chapter in chapters {
            sqlx::query!(
                r#"
                    INSERT INTO chapters (book_id, name, start, end)
                    VALUES (?, ?, ?, ?)
                "#,
                book_id,
                chapter.name,
                chapter.start,
                chapter.end,
            )
            .execute(&mut *trx)
            .await
            .context("Failed to insert chapter into database")?;
        }
    }

    // Commit transaction.
    trx.commit().await.context("Failed to commit transaction")?;

    data_response!(UploadBookResponse { book_id })
}

// Define the library lists.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum LibraryLists {
    // TODO: Maybe allow users to create their own lists?
    Listening,
    WantToListen,
    Finished,
    Abandoned,
}

impl Display for LibraryLists {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LibraryLists::Listening => write!(f, "listening"),
            LibraryLists::WantToListen => write!(f, "want_to_listen"),
            LibraryLists::Finished => write!(f, "finished"),
            LibraryLists::Abandoned => write!(f, "abandoned"),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetBookList {
    list: LibraryLists,
    file_id: Option<i64>,
    progress: Option<f64>,
}

// Move book into a library list.
pub async fn set_book_list(
    Session(session): Session,
    Path(book_id): Path<i64>,
    State(state): State<FreyaState>,
    Json(SetBookList {
        list,
        file_id,
        progress,
    }): Json<SetBookList>,
) -> ApiResult<SuccessResponse> {
    // Upsert a library entry.
    let list = list.to_string();

    // Try to create a new library entry. If it already exists, update it.
    // If file_id is None, the first file will be used.
    // If the user already has a library entry for the book, the library entry will be updated.
    // If the file_id is updated, the progress will be reset to 0.
    let error = sqlx::query!(
        r#"
            INSERT INTO library_entries (user_id, book_id, file_id, list, progress)
            VALUES ($1, $2, COALESCE($4, (
                SELECT id
                FROM files
                WHERE book_id = $2
                ORDER BY position ASC
                LIMIT 1
            )), $3, COALESCE($5, 0))
            ON CONFLICT (user_id, book_id) DO UPDATE
            SET list = EXCLUDED.list,
                file_id = COALESCE(EXCLUDED.file_id, library_entries.file_id),
                progress = CASE
                    WHEN library_entries.file_id != EXCLUDED.file_id THEN 0
                    WHEN $5 IS NOT NULL THEN $5
                    ELSE library_entries.progress
                END,
                modified = CURRENT_TIMESTAMP
        "#,
        session.user_id,
        book_id,
        list,
        file_id,
        progress,
    )
    .execute(&state.db)
    .await;

    tracing::error!(?error, "Upsert library entry");

    error.context("Failed to insert or update library entry")?;

    api_response!("library--list-set")
}

// Update progress.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProgress {
    file_id: i64,
    progress: f64,
}

pub async fn update_progress(
    Session(session): Session,
    Path(book_id): Path<i64>,
    State(state): State<FreyaState>,
    Json(UpdateProgress { file_id, progress }): Json<UpdateProgress>,
) -> ApiResult<SuccessResponse> {
    // Update progress.
    sqlx::query!(
        r#"
            UPDATE library_entries
            SET progress = ?,
                file_id = ?,
                modified = CURRENT_TIMESTAMP
            WHERE user_id = ?
            AND book_id = ?
        "#,
        progress,
        file_id,
        session.user_id,
        book_id,
    )
    .execute(&state.db)
    .await
    .context("Failed to update progress")?;

    api_response!("library--progress-updated")
}
