//! Error types for Astrea
//!
//! Provides a structured error type that integrates with Axum's response system
//! while being compatible with third-party error types via `anyhow`.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response as AxumResponse},
    Json,
};
use serde_json::json;
use std::fmt;

/// Main error type for route handlers
///
/// This error type is designed to be used in route handlers and automatically
/// converts to HTTP responses with appropriate status codes.
#[derive(thiserror::Error, Debug)]
pub enum RouteError {
    /// Bad request (400)
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// Not found (404)
    #[error("Not found: {0}")]
    NotFound(String),

    /// Unauthorized (401)
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// Forbidden (403)
    #[error("Forbidden: {0}")]
    Forbidden(String),

    /// Method not allowed (405)
    #[error("Method not allowed: {0}")]
    MethodNotAllowed(String),

    /// Conflict (409)
    #[error("Conflict: {0}")]
    Conflict(String),

    /// Validation error (422)
    #[error("Validation error: {0}")]
    Validation(String),

    /// Too many requests (429)
    #[error("Too many requests: {0}")]
    RateLimit(String),

    /// Internal server error (500)
    ///
    /// This variant automatically converts from `anyhow::Error`, allowing
    /// third-party errors to be propagated with `?` operator.
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),

    /// Custom error with specific status code
    #[error("Error {status}: {message}")]
    Custom { status: StatusCode, message: String },
}

impl RouteError {
    /// Create a new bad request error
    pub fn bad_request<M: fmt::Display>(message: M) -> Self {
        Self::BadRequest(message.to_string())
    }

    /// Create a new not found error
    pub fn not_found<M: fmt::Display>(message: M) -> Self {
        Self::NotFound(message.to_string())
    }

    /// Create a new unauthorized error
    pub fn unauthorized<M: fmt::Display>(message: M) -> Self {
        Self::Unauthorized(message.to_string())
    }

    /// Create a new forbidden error
    pub fn forbidden<M: fmt::Display>(message: M) -> Self {
        Self::Forbidden(message.to_string())
    }

    /// Create a new validation error
    pub fn validation<M: fmt::Display>(message: M) -> Self {
        Self::Validation(message.to_string())
    }

    /// Create a custom error with a specific status code
    pub fn custom<M: fmt::Display>(status: StatusCode, message: M) -> Self {
        Self::Custom {
            status,
            message: message.to_string(),
        }
    }

    /// Get the HTTP status code for this error
    #[must_use]
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::MethodNotAllowed(_) => StatusCode::METHOD_NOT_ALLOWED,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::RateLimit(_) => StatusCode::TOO_MANY_REQUESTS,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Custom { status, .. } => *status,
        }
    }

    /// Get the error message
    #[must_use]
    pub fn message(&self) -> String {
        match self {
            Self::BadRequest(msg)
            | Self::NotFound(msg)
            | Self::Unauthorized(msg)
            | Self::Forbidden(msg)
            | Self::MethodNotAllowed(msg)
            | Self::Conflict(msg)
            | Self::Validation(msg)
            | Self::RateLimit(msg)
            | Self::Custom { message: msg, .. } => msg.clone(),
            Self::Internal(e) => e.to_string(),
        }
    }
}

impl IntoResponse for RouteError {
    fn into_response(self) -> AxumResponse {
        let status = self.status_code();
        let body = json!({
            "error": self.message(),
            "status": status.as_u16(),
        });

        (status, Json(body)).into_response()
    }
}

/// Type alias for Result with `RouteError`
pub type Result<T> = std::result::Result<T, RouteError>;
