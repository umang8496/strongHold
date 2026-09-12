//! ==============================================================================
//! Centralized Application Error Types
//! ==============================================================================

use actix_web::{
    http::{header::ContentType, StatusCode},
    HttpResponse, ResponseError,
};
use diesel::result::Error as DieselError;
use serde::Serialize;
use thiserror::Error;

/// The primary error enum representing all failure modes in the application.
#[derive(Debug, Error)]
pub enum AppError {
    /// Returned when a requested entity does not exist (Maps to HTTP 404).
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Returned when a business rule or invariant is violated (Maps to HTTP 400).
    #[error("Validation error: {0}")]
    BadRequest(String),

    /// Returned when a state collision occurs (Maps to HTTP 409).
    #[error("Conflict: {0}")]
    Conflict(String),

    /// Returned when a generic database or pool operation fails (Maps to HTTP 500).
    #[error("Database error occurred: {0}")]
    DatabaseError(String),

    /// Direct wrapper around Diesel execution errors for transactions and queries.
    #[error("Database error: {0}")]
    Database(#[from] DieselError),

    /// Catch-all for unhandled system failures or internal bugs (Maps to HTTP 500).
    #[error("Internal server error: {0}")]
    Internal(String),
}

/// Standardized JSON structure sent back to API clients on failure.
#[derive(Serialize)]
struct ErrorResponseBody {
    error_code: &'static str,
    message: String,
    status: u16,
}

impl ResponseError for AppError {
    /// Maps our domain error variants to explicit HTTP Status Codes.
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Database(DieselError::NotFound) => StatusCode::NOT_FOUND,
            AppError::Database(_) | AppError::DatabaseError(_) | AppError::Internal(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    /// Constructs the final HTTP response sent over the wire.
    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();

        let error_code = match self {
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::BadRequest(_) => "BAD_REQUEST",
            AppError::Conflict(_) => "CONFLICT",
            AppError::Database(DieselError::NotFound) => "NOT_FOUND",
            AppError::Database(_) | AppError::DatabaseError(_) => "DATABASE_ERROR",
            AppError::Internal(_) => "INTERNAL_SERVER_ERROR",
        };

        // Log 500-level failures with internal context, warning on 4xx errors
        match self {
            AppError::Database(err) => {
                log::error!("Diesel database failure: {:?}", err);
            }
            AppError::DatabaseError(details) => {
                log::error!("Database infrastructure failure: {}", details);
            }
            AppError::Internal(details) => {
                log::error!("Internal error: {}", details);
            }
            _ => {
                log::warn!("Client error ({}): {}", status.as_u16(), self);
            }
        }

        // Return a clean JSON response with Content-Type: application/json
        HttpResponse::build(status)
            .content_type(ContentType::json())
            .json(ErrorResponseBody {
                error_code,
                message: self.to_string(),
                status: status.as_u16(),
            })
    }
}

/// Helper type alias so domain functions can return `AppResult<T>`
pub type AppResult<T> = Result<T, AppError>;