use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use domain::errors::DomainError;

use api_types::errors::ErrorResponse;

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug)]
pub enum ApiError {
    NotFound(String),
    Forbidden(String),
    Conflict(String),
    Validation(String),
    Unauthorized,
    Internal(String),
}

impl ApiError {
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}

impl From<DomainError> for ApiError {
    fn from(e: DomainError) -> Self {
        match e {
            DomainError::NotFound(msg) => Self::NotFound(msg),
            DomainError::Forbidden(msg) => Self::Forbidden(msg),
            DomainError::Conflict(msg) => Self::Conflict(msg),
            DomainError::Validation(msg) => Self::Validation(msg),
            DomainError::Repository(msg) => {
                tracing::error!("repository error: {msg}");
                Self::Internal("database error".into())
            }
            DomainError::Infrastructure(msg) => {
                tracing::error!("infrastructure error: {msg}");
                Self::Internal("service unavailable".into())
            }
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, ErrorResponse::not_found(msg)),
            Self::Forbidden(msg) => (StatusCode::FORBIDDEN, ErrorResponse::forbidden(msg)),
            Self::Conflict(msg) => (StatusCode::CONFLICT, ErrorResponse::conflict(msg)),
            Self::Validation(msg) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                ErrorResponse::validation(msg),
            ),
            Self::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                ErrorResponse::new("UNAUTHORIZED", "authentication required"),
            ),
            Self::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::internal(msg),
            ),
        };

        (status, Json(body)).into_response()
    }
}
