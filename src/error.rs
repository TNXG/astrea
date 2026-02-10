//! Error types for Astrea
//!
//! / Astrea 的错误类型定义
//!
//! Provides a structured error type that integrates with Axum's response system
//! while being compatible with third-party error types via `anyhow`.
//!
//! 提供结构化的错误类型，与 Axum 响应系统集成，同时通过 `anyhow` 兼容第三方错误类型。
//!
//! # Example
//!
//! # 示例
//!
//! ```rust,ignore
//! use astrea::prelude::*;
//!
//! #[route]
//! pub async fn handler(event: Event) -> Result<Response> {
//!     let user_id = get_param_required(&event, "id")?;
//!     if user_id.is_empty() {
//!         return Err(RouteError::bad_request("User ID cannot be empty"));
//!     }
//!     json(json!({ "user_id": user_id }))
//! }
//! ```

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response as AxumResponse},
    Json,
};
use serde_json::json;
use std::fmt;

/// Main error type for route handlers
///
/// / 路由处理函数的主要错误类型
///
/// This error type is designed to be used in route handlers and automatically
/// converts to HTTP responses with appropriate status codes.
///
/// 此错误类型专为路由处理函数设计，会自动转换为具有适当状态码的 HTTP 响应。
///
/// # Error Variants
///
/// # 错误变体
///
/// - `BadRequest(400)` - Invalid request data / 无效的请求数据
/// - `Unauthorized(401)` - Authentication required / 需要身份验证
/// - `Forbidden(403)` - Insufficient permissions / 权限不足
/// - `NotFound(404)` - Resource not found / 资源未找到
/// - `MethodNotAllowed(405)` - HTTP method not supported / 不支持的 HTTP 方法
/// - `Conflict(409)` - Resource conflict / 资源冲突
/// - `Validation(422)` - Validation failed / 验证失败
/// - `RateLimit(429)` - Too many requests / 请求过多
/// - `Internal(500)` - Internal server error / 内部服务器错误
/// - `Custom` - Custom status code / 自定义状态码
#[derive(thiserror::Error, Debug)]
pub enum RouteError {
    /// Bad request (400) - The request was malformed or contains invalid data
    /// / 错误的请求 (400) - 请求格式错误或包含无效数据
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// Not found (404) - The requested resource was not found
    /// / 未找到 (404) - 请求的资源不存在
    #[error("Not found: {0}")]
    NotFound(String),

    /// Unauthorized (401) - Authentication is required to access this resource
    /// / 未授权 (401) - 需要身份验证才能访问此资源
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// Forbidden (403) - Insufficient permissions to access this resource
    /// / 禁止访问 (403) - 权限不足以访问此资源
    #[error("Forbidden: {0}")]
    Forbidden(String),

    /// Method not allowed (405) - The HTTP method is not supported for this resource
    /// / 方法不允许 (405) - 此资源不支持该 HTTP 方法
    #[error("Method not allowed: {0}")]
    MethodNotAllowed(String),

    /// Conflict (409) - The request conflicts with the current state of the resource
    /// / 冲突 (409) - 请求与资源当前状态冲突
    #[error("Conflict: {0}")]
    Conflict(String),

    /// Validation error (422) - The request failed validation
    /// / 验证错误 (422) - 请求验证失败
    #[error("Validation error: {0}")]
    Validation(String),

    /// Too many requests (429) - Rate limit exceeded
    /// / 请求过多 (429) - 超过速率限制
    #[error("Too many requests: {0}")]
    RateLimit(String),

    /// Internal server error (500) - An unexpected error occurred
    /// / 内部服务器错误 (500) - 发生意外错误
    ///
    /// This variant automatically converts from `anyhow::Error`, allowing
    /// third-party errors to be propagated with the `?` operator.
    ///
    /// 此变体自动从 `anyhow::Error` 转换，允许使用 `?` 操作符传播第三方错误。
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),

    /// Custom error with specific status code
    /// / 带有特定状态码的自定义错误
    #[error("Error {status}: {message}")]
    Custom { status: StatusCode, message: String },
}

impl RouteError {
    /// Create a new bad request error (400)
    /// / 创建一个新的错误请求错误 (400)
    ///
    /// # Example
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// Err(RouteError::bad_request("Invalid user ID"))
    /// ```
    pub fn bad_request<M: fmt::Display>(message: M) -> Self {
        Self::BadRequest(message.to_string())
    }

    /// Create a new not found error (404)
    /// / 创建一个新的未找到错误 (404)
    ///
    /// # Example
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// Err(RouteError::not_found("User not found"))
    /// ```
    pub fn not_found<M: fmt::Display>(message: M) -> Self {
        Self::NotFound(message.to_string())
    }

    /// Create a new unauthorized error (401)
    /// / 创建一个新的未授权错误 (401)
    ///
    /// # Example
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// Err(RouteError::unauthorized("Invalid token"))
    /// ```
    pub fn unauthorized<M: fmt::Display>(message: M) -> Self {
        Self::Unauthorized(message.to_string())
    }

    /// Create a new forbidden error (403)
    /// / 创建一个新的禁止访问错误 (403)
    ///
    /// # Example
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// Err(RouteError::forbidden("Insufficient permissions"))
    /// ```
    pub fn forbidden<M: fmt::Display>(message: M) -> Self {
        Self::Forbidden(message.to_string())
    }

    /// Create a new conflict error (409)
    /// / 创建一个新的冲突错误 (409)
    ///
    /// # Example
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// Err(RouteError::conflict("Email already exists"))
    /// ```
    pub fn conflict<M: fmt::Display>(message: M) -> Self {
        Self::Conflict(message.to_string())
    }

    /// Create a new validation error (422)
    /// / 创建一个新的验证错误 (422)
    ///
    /// # Example
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// Err(RouteError::validation("Invalid email format"))
    /// ```
    pub fn validation<M: fmt::Display>(message: M) -> Self {
        Self::Validation(message.to_string())
    }

    /// Create a new rate limit error (429)
    /// / 创建一个新的速率限制错误 (429)
    ///
    /// # Example
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// Err(RouteError::rate_limit("Too many requests, try again later"))
    /// ```
    pub fn rate_limit<M: fmt::Display>(message: M) -> Self {
        Self::RateLimit(message.to_string())
    }

    /// Create a custom error with a specific status code
    /// / 创建带有特定状态码的自定义错误
    ///
    /// # Example
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// use axum::http::StatusCode;
    /// Err(RouteError::custom(StatusCode::IM_A_TEAPOT, "I'm a teapot"))
    /// ```
    pub fn custom<M: fmt::Display>(status: StatusCode, message: M) -> Self {
        Self::Custom {
            status,
            message: message.to_string(),
        }
    }

    /// Get the HTTP status code for this error
    /// / 获取此错误的 HTTP 状态码
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
    /// / 获取错误消息
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

    /// Check if this is a client error (4xx)
    /// / 检查是否为客户端错误 (4xx)
    #[must_use]
    pub fn is_client_error(&self) -> bool {
        self.status_code().is_client_error()
    }

    /// Check if this is a server error (5xx)
    /// / 检查是否为服务器错误 (5xx)
    #[must_use]
    pub fn is_server_error(&self) -> bool {
        self.status_code().is_server_error()
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

/// Type alias for Result with RouteError
/// / Result 类型的别名，使用 RouteError 作为错误类型
///
/// # Example
///
/// # 示例
///
/// ```rust,ignore
/// use astrea::prelude::*;
///
/// fn get_user(id: &str) -> Result<User> {
///     if id.is_empty() {
///         return Err(RouteError::bad_request("Invalid ID"));
///     }
///     Ok(User { id: id.to_string() })
/// }
/// ```
pub type Result<T> = std::result::Result<T, RouteError>;
