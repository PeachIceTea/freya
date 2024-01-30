use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub trait IntoResponseWithStatus
where
    Self: IntoResponse + Sized,
{
    fn into_error_response(self) -> Response {
        (StatusCode::UNPROCESSABLE_ENTITY, self).into_response()
    }
    fn into_response_with_status(self, status: StatusCode) -> Response {
        (status, self).into_response()
    }
}

impl<T> IntoResponseWithStatus for T where T: IntoResponse + Sized {}
