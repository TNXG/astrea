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

use axum::{
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response as AxumResponse},
};

pub mod builders;
pub mod stream;

pub use builders::{bytes, html, json, no_content, redirect, text};
pub use stream::stream;

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
        if let Ok(name) = HeaderName::try_from(key)
            && let Ok(v) = HeaderValue::try_from(value)
        {
            self.headers.insert(name, v);
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
