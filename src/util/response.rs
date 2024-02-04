use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

extern crate proc_macro;

pub type ApiResult<T> = Result<Json<T>, ApiError>;

#[derive(Debug, Error)]
pub enum ApiError {
    // Authentication errors.
    #[error("server-authentication--already-logged-in")]
    AlreadyLoggedIn,

    #[error("server-authentication--invalid-credentials")]
    InvalidCredentials,

    #[error("server-authentication--not-logged-in")]
    NotLoggedIn,

    #[error("server-authentication--not-admin")]
    NotAdmin,

    // File system errors.
    #[error("server-fs--could-not-list-directory")]
    CouldNotListDirectory,

    #[error("server-fs--ffprobe-failed")]
    FFProbeFailed(String),

    #[error("server-fs--invalid-path")]
    InvalidPath,

    // Book errors.
    #[error("server-upload--missing-data")]
    UploadMissingData,

    #[error("server-upload--invalid-file-path")]
    UploadInvalidFilePath(String),

    #[error("server-books--failed-to-get-cover-image")]
    FailedToGetCoverImage,

    // Internal server errors.
    #[error(transparent)]
    AnyhowError(#[from] anyhow::Error),
}

// Turn ApiError into a ApiResult.
impl<T> From<ApiError> for ApiResult<T> {
    fn from(err: ApiError) -> Self {
        Err(err)
    }
}

// Turn ApiError into an axum response.
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        // Try to extract an ApiError from an ApiError::AnyhowError.
        let api_error = match self {
            Self::AnyhowError(err) => match err.downcast::<Self>() {
                Ok(err) => err,
                Err(err) => Self::AnyhowError(err),
            },
            _ => self,
        };

        // Match the error to a status code.
        let status = match api_error {
            Self::InvalidCredentials
            | Self::AlreadyLoggedIn
            | Self::UploadMissingData
            | Self::UploadInvalidFilePath(_)
            | Self::InvalidPath => StatusCode::BAD_REQUEST,
            Self::CouldNotListDirectory | Self::FailedToGetCoverImage | Self::FFProbeFailed(_) => {
                StatusCode::UNPROCESSABLE_ENTITY
            }
            Self::NotLoggedIn | Self::NotAdmin => StatusCode::UNAUTHORIZED,
            Self::AnyhowError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        // Create the response
        let body = match &api_error {
            Self::AlreadyLoggedIn
            | Self::InvalidCredentials
            | Self::CouldNotListDirectory
            | Self::UploadMissingData
            | Self::FailedToGetCoverImage
            | Self::InvalidPath
            | Self::NotLoggedIn
            | Self::NotAdmin => ErrorResponse::new(api_error.to_string(), None),

            Self::UploadInvalidFilePath(value) | Self::FFProbeFailed(value) => {
                ErrorResponse::new(api_error.to_string(), Some(value.to_string()))
            }

            Self::AnyhowError(err) => {
                ErrorResponse::new(api_error.to_string(), Some(err.to_string()))
            }
        };
        (status, body).into_response()
    }
}

#[derive(Serialize)]
pub struct SuccessResponse {
    success: bool,
    pub message: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

impl SuccessResponse {
    pub fn new(message: &'static str, value: Option<String>) -> Self {
        Self {
            success: true,
            message,
            value,
        }
    }
}

#[derive(Serialize)]
pub struct DataResponse<T>
where
    T: Serialize + Sized,
{
    success: bool,
    pub data: T,
}

impl<T> DataResponse<T>
where
    T: Serialize + Sized,
{
    pub fn new(data: T) -> Self {
        Self {
            success: true,
            data,
        }
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    success: bool,
    error_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<String>,
}

impl ErrorResponse {
    fn new(error_code: String, value: Option<String>) -> Json<Self> {
        Json(Self {
            success: false,
            error_code,
            value,
        })
    }
}

// Macro for creating a success response.
#[macro_export]
macro_rules! api_response {
    ($message:expr) => {
        Ok(axum::Json($crate::util::response::SuccessResponse::new(
            $message, None,
        )))
    };
    ($message:expr, $value:expr) => {
        Ok(axum::Json($crate::util::response::SuccessResponse::new(
            $message,
            Some($value),
        )))
    };
}

// Macro for creating a success response with data.
#[macro_export]
macro_rules! data_response {
    ($data:expr) => {
        Ok(axum::Json($crate::util::response::DataResponse::new($data)))
    };
}

// Macros for creating ApiErrors.
#[macro_export]
macro_rules! api_error {
    ($err:ident) => {
        $crate::util::response::ApiError::$err
    };
    ($err:ident, $arg:expr) => {
        $crate::util::response::ApiError::$err(format!("{}", $arg))
    };
    ($err:ident, $($arg:tt)*) => {
        $crate::util::response::ApiError::$err(format!($($arg)*))
    };
}

#[macro_export]
macro_rules! api_error_result {
    ($err:ident) => {
        Err($crate::util::response::ApiError::$err)
    };
    ($err:ident, $arg:expr) => {
        Err($crate::util::response::ApiError::$err(format!("{}", $arg)))
    };
    ($err:ident, $($arg:tt)*) => {
        Err($crate::util::response::ApiError::$err(format!($($arg)*)))
    };
}

#[macro_export]
macro_rules! api_bail {
    ($err:ident) => {
        return Err($crate::util::response::ApiError::$err)
    };
    ($err:ident, $arg:expr) => {
        return Err($crate::util::response::ApiError::$err(format!("{}", $arg)))
    };
    ($err:ident, $($arg:tt)*) => {
        return Err($crate::util::response::ApiError::$err(format!($($arg)*)))
    };
}
