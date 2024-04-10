use std::fmt::{Display, Formatter};

use crate::{
    api_bail, api_response, data_response,
    database::{
        book::Book,
        chapter::Chapter,
        file::{File, FileData},
        library::LibraryEntry,
    },
    state::FreyaState,
    util::{
        cover::get_cover_bytes,
        ffmpeg::{ffprobe_chapters, ffprobe_duration},
        response::{ApiError, ApiFileResult, ApiResult, DataResponse, SuccessResponse},
        session::{AdminSession, Session},
    },
};
use anyhow::Context;
use axum::{
    body::Bytes,
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

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
    let books = state.database.get_all_books().await?;

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
    let book = state
        .database
        .get_book_details(book_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    // Get files from the database.
    let files = state.database.get_files_for_book(book_id).await?;

    // Get chapters from the database.
    let chapters = state.database.get_chapters_for_book(book_id).await?;

    // Get library entry from the database.
    let library = state
        .database
        .get_library_entry(session.user_id, book_id)
        .await?;

    data_response!(BookResponse {
        book,
        files,
        library,
        chapters,
    })
}

static PLACEHOLDER_COVER: Lazy<Vec<u8>> = Lazy::new(|| {
    let placeholder_cover = include_bytes!("../../placeholder-cover.jpg");
    placeholder_cover.to_vec()
});

pub async fn get_book_cover(
    Session(_): Session,
    Path(book_id): Path<i64>,
    State(state): State<FreyaState>,
) -> ApiFileResult<Vec<u8>> {
    // Get cover image from the database.
    let result = state.database.get_book_cover(book_id).await;

    match result {
        Ok(Some(cover)) => Ok(cover),
        Ok(None) => Ok(PLACEHOLDER_COVER.clone()),
        Err(err) => {
            tracing::error!("Failed to get cover image from database: {:?}", err);
            Ok(PLACEHOLDER_COVER.clone())
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
    let book_id = state
        .database
        .create_book(
            &title,
            &author,
            cover.as_ref(),
            &file_data,
            chapters.as_ref(),
        )
        .await?;

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
    state
        .database
        .manage_library_entry(session.user_id, book_id, &list, file_id, progress)
        .await?;
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
    state
        .database
        .update_progress(session.user_id, book_id, file_id, progress)
        .await?;
    api_response!("library--progress-updated")
}
