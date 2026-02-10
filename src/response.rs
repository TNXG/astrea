//! 响应类型和构建器
//!
//! 提供便捷的函数来构建 HTTP 响应。

use crate::error::{Result, RouteError};
use axum::{
    body::Body,
    http::{header, HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::{IntoResponse, Response as AxumResponse},
};
use serde::Serialize;

/// HTTP 响应类型
///
/// 提供简单的 API 构建响应，支持链式调用。
#[derive(Debug, Clone)]
pub struct Response {
    /// HTTP 状态码
    pub status: StatusCode,
    /// 响应头
    pub headers: HeaderMap,
    /// 响应体
    pub body: Vec<u8>,
}

impl Response {
    /// 创建默认响应
    ///
    /// 默认为 200 OK，无响应头和空响应体。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置状态码（链式调用）
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

    /// 添加响应头（链式调用）
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// json(data)?
    ///     .status(StatusCode::CREATED)
    ///     .header("X-Request-Id", "abc123")
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

    /// 设置 Content-Type 响应头
    pub fn content_type(mut self, content_type: &str) -> Self {
        if let Ok(v) = HeaderValue::try_from(content_type) {
            self.headers.insert(header::CONTENT_TYPE, v);
        }
        self
    }

    /// 转换为 Axum Response
    pub fn into_axum_response(mut self) -> AxumResponse {
        // 默认添加 Server 头（用户未手动设置时）
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

/// 创建 JSON 响应
///
/// # 示例
///
/// ```rust,ignore
/// json(json!({
///     "message": "Hello, World!",
///     "status": "success",
/// }))
/// ```
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

/// 创建文本响应
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

/// 创建 HTML 响应
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

/// 创建重定向响应
///
/// # 示例
///
/// ```rust,ignore
/// redirect("/login")
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

/// 创建 204 No Content 响应
///
/// # 示例
///
/// ```rust,ignore
/// no_content()
/// ```
#[must_use]
pub fn no_content() -> Response {
    Response {
        status: StatusCode::NO_CONTENT,
        headers: HeaderMap::new(),
        body: Vec::new(),
    }
}

/// 从原始字节创建响应
///
/// # 示例
///
/// ```rust,ignore
/// bytes(data).content_type("image/png")
/// ```
#[must_use]
pub fn bytes(data: Vec<u8>) -> Response {
    Response {
        status: StatusCode::OK,
        headers: HeaderMap::new(),
        body: data,
    }
}

/// 从流创建响应
///
/// # 示例
///
/// ```rust,ignore
/// stream(body)
/// ```
#[must_use]
pub fn stream(body: Body) -> AxumResponse {
    body.into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_response() {
        let response = json(serde_json::json!({"message": "Hello"})).unwrap();
        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(
            response.headers.get("content-type").unwrap(),
            "application/json"
        );
        assert_eq!(
            String::from_utf8_lossy(&response.body),
            r#"{"message":"Hello"}"#
        );
    }

    #[test]
    fn test_text_response() {
        let response = text("Hello, World!");
        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(
            response.headers.get("content-type").unwrap(),
            "text/plain; charset=utf-8"
        );
        assert_eq!(String::from_utf8_lossy(&response.body), "Hello, World!");
    }

    #[test]
    fn test_html_response() {
        let response = html("<h1>Hello</h1>");
        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(
            response.headers.get("content-type").unwrap(),
            "text/html; charset=utf-8"
        );
        assert_eq!(String::from_utf8_lossy(&response.body), "<h1>Hello</h1>");
    }

    #[test]
    fn test_redirect_response() {
        let response = redirect("/login").unwrap();
        assert_eq!(response.status, StatusCode::FOUND);
        assert_eq!(response.headers.get("location").unwrap(), "/login");
    }

    #[test]
    fn test_no_content_response() {
        let response = no_content();
        assert_eq!(response.status, StatusCode::NO_CONTENT);
        assert!(response.body.is_empty());
    }

    #[test]
    fn test_response_chainable() {
        let response = json(serde_json::json!({"status": "ok"}))
            .unwrap()
            .status(StatusCode::CREATED)
            .header("X-Custom", "test");

        assert_eq!(response.status, StatusCode::CREATED);
        assert_eq!(response.headers.get("X-Custom").unwrap(), "test");
    }
}
