//! Response type and builders
//!
//! / 响应类型和构建器
//!
//! This module provides convenient functions for building HTTP responses
//! with proper content types and status codes.
//!
//! 此模块提供便捷的函数来构建具有正确内容类型和状态码的 HTTP 响应。
//!
//! # Overview
//!
//! # 概述
//!
//! The `Response` type provides a simple chainable API for building responses:
//!
//! `Response` 类型提供了简单的链式 API 来构建响应：
//!
//! ```rust,ignore
//! use astrea::prelude::*;
//!
//! json(json!({ "message": "Hello" }))?
//!     .status(StatusCode::CREATED)
//!     .header("X-Request-Id", "abc123")
//! ```
//!
//! # Response Helpers
//!
//! # 响应辅助函数
//!
//! - [`json`] - JSON responses (application/json)
//!   [`json`] - JSON 响应 (application/json)
//! - [`text`] - Plain text responses (text/plain)
//!   [`text`] - 纯文本响应 (text/plain)
//! - [`html`] - HTML responses (text/html)
//!   [`html`] - HTML 响应 (text/html)
//! - [`redirect`] - HTTP redirects (302 Found)
//!   [`redirect`] - HTTP 重定向 (302 Found)
//! - [`no_content`] - Empty responses (204 No Content)
//!   [`no_content`] - 空响应 (204 No Content)
//! - [`bytes`] - Raw byte responses
//!   [`bytes`] - 原始字节响应
//! - [`stream`] - Streaming responses
//!   [`stream`] - 流式响应
//!
//! # Server Header
//!
//! # Server 头
//!
//! All responses automatically include a `Server: Astrea` header unless
//! explicitly overridden.
//!
//! 所有响应自动包含 `Server: Astrea` 头，除非明确覆盖。

use crate::error::{Result, RouteError};
use axum::{
    body::Body,
    http::{header, HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::{IntoResponse, Response as AxumResponse},
};
use serde::Serialize;

/// HTTP response type
///
/// / HTTP 响应类型
///
/// Provides a simple API for building responses with support for
/// method chaining to set status code and headers.
///
/// 提供简单的 API 来构建响应，支持链式调用设置状态码和响应头。
///
/// # Example
///
/// # 示例
///
/// ```rust,ignore
/// use astrea::prelude::*;
///
/// json(json!({ "message": "Hello" }))?
///     .status(StatusCode::CREATED)
///     .header("X-Request-Id", "abc123");
/// ```

#[derive(Debug, Clone)]
pub struct Response {
    /// HTTP status code
    /// / HTTP 状态码
    pub status: StatusCode,
    /// Response headers
    /// / 响应头
    pub headers: HeaderMap,
    /// Response body
    /// / 响应体
    pub body: Vec<u8>,
}

impl Response {
    /// Create a default response
    ///
    /// / 创建默认响应
    ///
    /// Defaults to 200 OK with no headers and empty body.
    ///
    /// 默认为 200 OK，无响应头和空响应体。
    ///
    /// # Example
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// let response = Response::new();
    /// assert_eq!(response.status, StatusCode::OK);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the status code (chainable)
    ///
    /// / 设置状态码（可链式调用）
    ///
    /// # Example
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// json(data)?.status(StatusCode::CREATED)
    /// ```
    #[must_use]
    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    /// Add a response header (chainable)
    ///
    /// / 添加响应头（可链式调用）
    ///
    /// # Example
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// json(data)?
    ///     .status(StatusCode::CREATED)
    ///     .header("X-Request-Id", "abc123")
    ///     .header("X-Rate-Limit", "100")
    /// ```
    #[must_use]
    pub fn header(mut self, key: &str, value: &str) -> Self {
        if let Ok(name) = HeaderName::try_from(key) {
            if let Ok(v) = HeaderValue::try_from(value) {
                self.headers.insert(name, v);
            }
        }
        self
    }

    /// Set the Content-Type header
    ///
    /// / 设置 Content-Type 响应头
    ///
    /// # Example
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// bytes(data).content_type("image/png")
    /// ```
    pub fn content_type(mut self, content_type: &str) -> Self {
        if let Ok(v) = HeaderValue::try_from(content_type) {
            self.headers.insert(header::CONTENT_TYPE, v);
        }
        self
    }

    /// Convert to Axum Response
    ///
    /// / 转换为 Axum Response
    ///
    /// Automatically adds `Server: Astrea` header if not already set.
    ///
    /// 如果未设置，自动添加 `Server: Astrea` 头。
    ///
    /// # Note
    ///
    /// # 注意
    ///
    /// This method is called automatically by the `IntoResponse` trait.
    /// You typically don't need to call it directly.
    ///
    /// 此方法由 `IntoResponse` trait 自动调用。通常不需要直接调用。
    pub fn into_axum_response(mut self) -> AxumResponse {
        // Add Server header if not manually set
        // 添加 Server 头（如果未手动设置）
        if !self.headers.contains_key(header::SERVER) {
            self.headers
                .insert(header::SERVER, HeaderValue::from_static("Astrea"));
        }
        (self.status, self.headers, self.body).into_response()
    }
}

impl Default for Response {
    fn default() -> Self {
        Self {
            status: StatusCode::OK,
            headers: HeaderMap::new(),
            body: Vec::new(),
        }
    }
}

impl IntoResponse for Response {
    fn into_response(self) -> AxumResponse {
        self.into_axum_response()
    }
}

/// Create a JSON response
///
/// / 创建 JSON 响应
///
/// Automatically sets `Content-Type: application/json`.
///
/// 自动设置 `Content-Type: application/json`。
///
/// # Type Parameters
///
/// # 类型参数
///
/// - `T` - The type to serialize (must implement `Serialize`)
///   要序列化的类型（必须实现 `Serialize`）
///
/// # Errors
///
/// # 错误
///
/// Returns `RouteError::Internal` if serialization fails.
///
/// 如果序列化失败，返回 `RouteError::Internal`。
///
/// # Example
///
/// # 示例
///
/// ```rust,ignore
/// json(json!({
///     "message": "Hello, World!",
///     "status": "success",
/// }))
/// ```
///
/// # See Also
///
/// # 另请参阅
///
/// - [`serde_json::json`] macro for creating JSON values
///   [`serde_json::json`] 宏 - 用于创建 JSON 值
pub fn json<T: Serialize>(data: T) -> Result<Response> {
    let body = serde_json::to_vec(&data)
        .map_err(|e| RouteError::Internal(anyhow::anyhow!("Failed to serialize JSON: {e}")))?;

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );

    Ok(Response {
        status: StatusCode::OK,
        headers,
        body,
    })
}

/// Create a text response
///
/// / 创建文本响应
///
/// Automatically sets `Content-Type: text/plain; charset=utf-8`.
///
/// 自动设置 `Content-Type: text/plain; charset=utf-8`。
///
/// # Example
///
/// # 示例
///
/// ```rust,ignore
/// text("Hello, World!")
/// ```
pub fn text(content: impl Into<String>) -> Response {
    let body = content.into();

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/plain; charset=utf-8"),
    );

    Response {
        status: StatusCode::OK,
        headers,
        body: body.into_bytes(),
    }
}

/// Create an HTML response
///
/// / 创建 HTML 响应
///
/// Automatically sets `Content-Type: text/html; charset=utf-8`.
///
/// 自动设置 `Content-Type: text/html; charset=utf-8`。
///
/// # Example
///
/// # 示例
///
/// ```rust,ignore
/// html("<h1>Hello, World!</h1>")
/// ```
pub fn html(content: impl Into<String>) -> Response {
    let body = content.into();

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );

    Response {
        status: StatusCode::OK,
        headers,
        body: body.into_bytes(),
    }
}

/// Create a redirect response
///
/// / 创建重定向响应
///
/// Returns a 302 Found response with the Location header set.
///
/// 返回 302 Found 响应，设置 Location 头。
///
/// # Errors
///
/// # 错误
///
/// Returns `RouteError::BadRequest` if the URL is invalid.
///
/// 如果 URL 无效，返回 `RouteError::BadRequest`。
///
/// # Example
///
/// # 示例
///
/// ```rust,ignore
/// redirect("/login")
/// redirect("https://example.com")
/// ```
pub fn redirect(url: &str) -> Result<Response> {
    let value = HeaderValue::try_from(url)
        .map_err(|_| RouteError::bad_request(format!("Invalid redirect URL: {url}")))?;

    let mut headers = HeaderMap::new();
    headers.insert(header::LOCATION, value);

    Ok(Response {
        status: StatusCode::FOUND,
        headers,
        body: Vec::new(),
    })
}

/// Create a 204 No Content response
///
/// / 创建 204 No Content 响应
///
/// Useful for DELETE requests and other operations that don't return data.
///
/// 适用于 DELETE 请求和其他不返回数据的操作。
///
/// # Example
///
/// # 示例
///
/// ```rust,ignore
/// #[route]
/// async fn delete_handler(event: Event) -> Result<Response> {
///     // Delete resource...
///     Ok(no_content())
/// }
/// ```
#[must_use]
pub fn no_content() -> Response {
    Response {
        status: StatusCode::NO_CONTENT,
        headers: HeaderMap::new(),
        body: Vec::new(),
    }
}

/// Create a response from raw bytes
///
/// / 从原始字节创建响应
///
/// Use this for binary data like images, PDFs, etc.
/// You should set the appropriate Content-Type.
///
/// 用于二进制数据，如图像、PDF 等。应设置适当的 Content-Type。
///
/// # Example
///
/// # 示例
///
/// ```rust,ignore
/// let image_data = std::fs::read("image.png")?;
/// bytes(image_data).content_type("image/png")
/// ```
#[must_use]
pub fn bytes(data: Vec<u8>) -> Response {
    Response {
        status: StatusCode::OK,
        headers: HeaderMap::new(),
        body: data,
    }
}

/// Create a streaming response
///
/// / 创建流式响应
///
/// Use this for streaming data, Server-Sent Events, or large files.
///
/// 用于流式数据、服务器发送事件或大文件。
///
/// # Example
///
/// # 示例
///
/// ```rust,ignore
/// use axum::body::Body;
/// use futures_util::stream::once;
///
/// async fn stream_handler() -> Response {
///     let stream = once(async { Ok::<_, std::io::Error>(bytes::Bytes::from("Hello")) });
///     stream(Body::from_stream(stream))
/// }
/// ```
#[must_use]
pub fn stream(body: Body) -> AxumResponse {
    body.into_response()
}
