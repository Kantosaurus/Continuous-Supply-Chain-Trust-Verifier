//! API error types and handling.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

/// API error type.
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Authentication required")]
    Unauthorized,

    #[error("Access denied")]
    Forbidden,

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Rate limit exceeded")]
    RateLimited,

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Database error: {0}")]
    Database(String),
}

impl ApiError {
    /// Returns the HTTP status code for this error.
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            Self::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Returns the error code for this error.
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Unauthorized => "UNAUTHORIZED",
            Self::Forbidden => "FORBIDDEN",
            Self::NotFound(_) => "NOT_FOUND",
            Self::BadRequest(_) => "BAD_REQUEST",
            Self::Conflict(_) => "CONFLICT",
            Self::RateLimited => "RATE_LIMITED",
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
            Self::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE",
            Self::Database(_) => "DATABASE_ERROR",
        }
    }
}

/// Error response body.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorDetails,
}

/// Error details.
#[derive(Debug, Serialize)]
pub struct ErrorDetails {
    pub code: &'static str,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = ErrorResponse {
            error: ErrorDetails {
                code: self.error_code(),
                message: self.to_string(),
                details: None,
            },
        };

        (status, Json(body)).into_response()
    }
}

/// Result type alias for API operations.
pub type ApiResult<T> = Result<T, ApiError>;

impl From<sctv_core::traits::RepositoryError> for ApiError {
    fn from(err: sctv_core::traits::RepositoryError) -> Self {
        use sctv_core::traits::RepositoryError;
        match err {
            RepositoryError::NotFound => Self::NotFound("Resource not found".to_string()),
            RepositoryError::AlreadyExists => Self::Conflict("Resource already exists".to_string()),
            RepositoryError::Database(msg) => Self::Database(msg),
            RepositoryError::Serialization(msg) => {
                Self::Internal(format!("Serialization error: {}", msg))
            }
            RepositoryError::InvalidData(msg) => Self::Validation(msg),
        }
    }
}

impl From<jsonwebtoken::errors::Error> for ApiError {
    fn from(_err: jsonwebtoken::errors::Error) -> Self {
        Self::Unauthorized
    }
}
