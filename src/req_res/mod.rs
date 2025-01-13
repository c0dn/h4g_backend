pub mod auth;
pub mod me;
pub mod users;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use diesel::result::Error;
use diesel_async::pooled_connection::bb8::RunError;
use diesel_async::pooled_connection::PoolError;
use log::error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataValidationError {
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum ClientErrorMessages {
    DataValidationError(DataValidationError),
}

impl From<DataValidationError> for ClientErrorMessages {
    fn from(value: DataValidationError) -> Self {
        Self::DataValidationError(value)
    }
}

#[derive(Debug, Clone)]
enum ErrorKind {
    Unauthorized,
    Forbidden,
    UnprocessableEntity,
    BadRequest(Option<ClientErrorMessages>),
    InternalError(String),
    MethodNotAllowed,
    NoContent,
    ServiceUnavailable,
    NotFound,
}
#[derive(Debug)]
pub struct AppError(ErrorKind);

impl AppError {
    pub(crate) fn unauthorized() -> Self {
        AppError(ErrorKind::Unauthorized)
    }

    pub(crate) fn forbidden() -> Self {
        AppError(ErrorKind::Forbidden)
    }

    pub(crate) fn internal_error(msg: String) -> Self {
        AppError(ErrorKind::InternalError(msg))
    }
    pub(crate) fn unprocessable_entity() -> Self {
        AppError(ErrorKind::UnprocessableEntity)
    }

    pub(crate) fn no_content() -> Self {
        AppError(ErrorKind::NoContent)
    }

    pub(crate) fn service_unavailable() -> Self {
        AppError(ErrorKind::ServiceUnavailable)
    }
    pub(crate) fn bad_request<E: Into<Option<ClientErrorMessages>>>(errors: E) -> Self {
        AppError(ErrorKind::BadRequest(errors.into()))
    }

    pub(crate) fn not_found() -> Self {
        AppError(ErrorKind::NotFound)
    }

    pub(crate) fn method_not_allowed() -> Self {
        AppError(ErrorKind::MethodNotAllowed)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match &self.0 {
            ErrorKind::Unauthorized => (StatusCode::UNAUTHORIZED, ()).into_response(),
            ErrorKind::Forbidden => (StatusCode::FORBIDDEN, ()).into_response(),
            ErrorKind::InternalError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()).into_response()
            }
            ErrorKind::ServiceUnavailable => (StatusCode::SERVICE_UNAVAILABLE, ()).into_response(),
            ErrorKind::UnprocessableEntity => {
                (StatusCode::UNPROCESSABLE_ENTITY, ()).into_response()
            }
            ErrorKind::BadRequest(errors) => errors
                .clone()
                .map_or((StatusCode::BAD_REQUEST, ()).into_response(), |errors| {
                    (StatusCode::BAD_REQUEST, Json(errors)).into_response()
                }),
            ErrorKind::NotFound => (StatusCode::NOT_FOUND, ()).into_response(),
            ErrorKind::NoContent => (StatusCode::NO_CONTENT, ()).into_response(),
            ErrorKind::MethodNotAllowed => (StatusCode::METHOD_NOT_ALLOWED, ()).into_response(),
        }
    }
}

impl From<Error> for AppError {
    fn from(v: Error) -> Self {
        error!("Diesel: {}", v.to_string());
        Self::internal_error("DB error".to_string())
    }
}

impl From<fred::error::Error> for AppError {
    fn from(value: fred::error::Error) -> Self {
        error!("Redis: {}", value.to_string());
        Self::internal_error("Redis error".to_string())
    }
}

impl From<RunError> for AppError {
    fn from(v: RunError) -> Self {
        error!("Diesel: {}", v.to_string());
        Self::internal_error("DB error".to_string())
    }
}
