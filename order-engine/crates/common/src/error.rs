//! ==============================================================================
//! Centralized Application Error Types
//! ==============================================================================
//! This module defines the canonical `AppError` enum used across all layers of the
//! application (repositories, domain services, and HTTP handlers).
//!
//! By implementing `actix_web::ResponseError`, any function returning
//! `Result<T, AppError>` can propagate errors directly to Actix using the `?` operator.
//! Actix will automatically intercept the error, choose the appropriate HTTP status
//! code, and serialize a standard JSON error payload for the client.
//! ==============================================================================

use actix_web::{
    http::{header::ContentType, StatusCode},
    HttpResponse, ResponseError,
};
use serde::Serialize;
use thiserror::Error;

/// The primary error enum representing all failure modes in the application.
#[derive(Debug, Error)]
pub enum AppError {
    /// Returned when a requested entity does not exist (Maps to HTTP 404).
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Returned when a business rule or invariant is violated (Maps to HTTP 400).
    /// Examples: Negative price, invalid status transition, malformed input.
    #[error("Validation error: {0}")]
    BadRequest(String),

    /// Returned when a state collision occurs (Maps to HTTP 409).
    /// Examples: Unique constraint violation (duplicate SKU), optimistic lock failure.
    #[error("Conflict: {0}")]
    Conflict(String),

    /// Returned when an external or internal database query fails (Maps to HTTP 500).
    /// We keep the internal reason hidden from the client to prevent information leakage.
    #[error("Database error occurred")]
    DatabaseError(String),

    /// Catch-all for unhandled system failures or internal bugs (Maps to HTTP 500).
    #[error("Internal server error")]
    Internal(String),
}

/// Standardized JSON structure sent back to API clients on failure.
#[derive(Serialize)]
struct ErrorResponseBody {
    /// A machine-readable status string, e.g., "NOT_FOUND", "BAD_REQUEST".
    error_code: &'static str,
    /// A human-readable description of what went wrong.
    message: String,
    /// The numeric HTTP status code matching the header.
    status: u16,
}

impl ResponseError for AppError {
    /// Maps our domain error variants to explicit HTTP Status Codes.
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::DatabaseError(_) | AppError::Internal(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    /// Constructs the final HTTP response sent over the wire.
    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();

        // Categorize into a machine-readable code for frontend consumption
        let error_code = match self {
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::BadRequest(_) => "BAD_REQUEST",
            AppError::Conflict(_) => "CONFLICT",
            AppError::DatabaseError(_) => "DATABASE_ERROR",
            AppError::Internal(_) => "INTERNAL_SERVER_ERROR",
        };

        // If the error is a 500-level error, log the actual internal detail to stdout/logs,
        // but avoid exposing sensitive database or internal messages to the HTTP client.
        match self {
            AppError::DatabaseError(details) => {
                log::error!("Database failure: {}", details);
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
/// instead of writing `Result<T, AppError>` repeatedly.
pub type AppResult<T> = Result<T, AppError>;
