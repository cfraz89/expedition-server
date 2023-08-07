use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub struct ResponseError(Response);

impl IntoResponse for ResponseError {
    fn into_response(self) -> Response {
        self.0
    }
}

impl<E> From<E> for ResponseError
where
    E: Into<color_eyre::eyre::Error>,
{
    fn from(value: E) -> Self {
        Self(
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Into::<color_eyre::eyre::Error>::into(value).to_string(),
            )
                .into_response(),
        )
    }
}

impl ResponseError {
    pub fn with_status<T>(status_code: StatusCode, data: T) -> Self
    where
        (StatusCode, T): IntoResponse,
    {
        ResponseError((status_code, data).into_response())
    }

    pub fn internal_server_error<T>(data: T) -> Self
    where
        (StatusCode, T): IntoResponse,
    {
        ResponseError((StatusCode::INTERNAL_SERVER_ERROR, data).into_response())
    }

    pub fn not_found<T>(data: T) -> Self
    where
        (StatusCode, T): IntoResponse,
    {
        ResponseError((StatusCode::NOT_FOUND, data).into_response())
    }
}

pub type Result<T, E = ResponseError> = axum::response::Result<T, E>;
