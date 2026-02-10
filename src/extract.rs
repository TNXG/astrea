//! Helper functions for extracting data from Event
//!
//! / 从 Event 提取数据的辅助函数
//!
//! This module provides convenient functions to access request data
//! without complex Axum extractor signatures.
//!
//! 此模块提供便捷的函数来访问请求数据，无需复杂的 Axum 提取器签名。
//!
//! # Overview
//!
//! # 概述
//!
//! Instead of using Axum's extractors like:
//!
//! 不使用 Axum 的提取器，如：
//!
//! ```rust,ignore
//! async fn handler(
//!     Path(id): Path<String>,
//!     Query(params): Query<HashMap<String, String>>,
//!     Json(body): Json<MyData>,
//! ) -> Result<Response>
//! ```
//!
//! Astrea provides a simple, unified interface:
//!
//! Astrea 提供了简单统一的接口：
//!
//! ```rust,ignore
//! use astrea::prelude::*;
//!
//! #[route]
//! async fn handler(event: Event, bytes: Bytes) -> Result<Response> {
//!     let id = get_param_required(&event, "id")?;
//!     let search = get_query_param(&event, "q");
//!     let body: MyData = get_body(&event, &bytes)?;
//!     json(json!({ "id", "search": search, "body": body }))
//! }
//! ```
//!
//! # Available Extractors
//!
//! # 可用的提取器
//!
//! - **Path parameters**: [`get_param`], [`get_param_required`]
//!   **路径参数**：[`get_param`], [`get_param_required`]
//! - **Query parameters**: [`get_query`], [`get_query_param`], [`get_query_param_required`]
//!   **查询参数**：[`get_query`], [`get_query_param`], [`get_query_param_required`]
//! - **Request body**: [`get_body`], [`get_body_bytes`], [`get_body_text`]
//!   **请求体**：[`get_body`], [`get_body_bytes`], [`get_body_text`]
//! - **Headers**: [`get_header`], [`get_headers`]
//!   **请求头**：[`get_header`], [`get_headers`]
//! - **Metadata**: [`get_method`], [`get_path`], [`get_uri`]
//!   **元数据**：[`get_method`], [`get_path`], [`get_uri`]
//! - **State**: [`get_state`]
//!   **状态**：[`get_state`]

use crate::{
    error::{Result, RouteError},
    Event,
};
use axum::http::HeaderMap;

/// Get a path parameter by key
///
/// / 根据键获取路径参数
///
/// Returns `None` if the parameter doesn't exist.
///
/// 如果参数不存在，返回 `None`。
///
/// # Example
///
/// # 示例
///
/// ```rust,ignore
/// // Route: /users/[id]
/// let user_id = get_param(&event, "id").unwrap_or("default");
/// ```
///
/// # See Also
///
/// # 另请参阅
///
/// - [`get_param_required`] for a version that returns an error if the parameter is missing
///   [`get_param_required`] - 参数缺失时返回错误的版本
#[must_use]
pub fn get_param<'a>(event: &'a Event, key: &str) -> Option<&'a str> {
    event.params().get(key).map(std::string::String::as_str)
}

/// Get a required path parameter
///
/// / 获取必需的路径参数
///
/// Returns an error if the parameter doesn't exist.
///
/// 如果参数不存在，返回错误。
///
/// # Errors
///
/// # 错误
///
/// Returns `RouteError::BadRequest` if the parameter is missing.
///
/// 如果参数缺失，返回 `RouteError::BadRequest`。
///
/// # Example
///
/// # 示例
///
/// ```rust,ignore
/// // Route: /users/[id]
/// let user_id = get_param_required(&event, "id")?;
/// ```
pub fn get_param_required<'a>(event: &'a Event, key: &str) -> Result<&'a str> {
    get_param(event, key)
        .ok_or_else(|| RouteError::bad_request(format!("Missing required parameter: {key}")))
}

/// Get all query parameters
///
/// / 获取所有查询参数
///
/// Returns a reference to the complete query parameter map.
///
/// 返回完整查询参数映射的引用。
///
/// # Note
///
/// # 注意
///
/// Query parameters are lazily parsed from the URI on first access.
///
/// 查询参数在首次访问时从 URI 延迟解析。
///
/// # Example
///
/// # 示例
///
/// ```rust,ignore
/// let query = get_query(&event);
/// let search = query.get("q").unwrap_or(&"".to_string());
/// ```
#[must_use]
pub fn get_query(event: &Event) -> &std::collections::HashMap<String, String> {
    event.query()
}

/// Get a query parameter by key
///
/// / 根据键获取查询参数
///
/// Returns `None` if the parameter doesn't exist.
///
/// 如果参数不存在，返回 `None`。
///
/// # Example
///
/// # 示例
///
/// ```rust,ignore
/// // URL: /search?q=rust&page=1
/// let search = get_query_param(&event, "q"); // Some("rust".to_string())
/// ```
#[must_use]
pub fn get_query_param(event: &Event, key: &str) -> Option<String> {
    event.query().get(key).cloned()
}

/// Get a required query parameter
///
/// / 获取必需的查询参数
///
/// Returns an error if the parameter doesn't exist.
///
/// 如果参数不存在，返回错误。
///
/// # Errors
///
/// # 错误
///
/// Returns `RouteError::BadRequest` if the parameter is missing.
///
/// 如果参数缺失，返回 `RouteError::BadRequest`。
///
/// # Example
///
/// # 示例
///
/// ```rust,ignore
/// let code = get_query_param_required(&event, "code")?;
/// ```
pub fn get_query_param_required(event: &Event, key: &str) -> Result<String> {
    get_query_param(event, key)
        .ok_or_else(|| RouteError::bad_request(format!("Missing required query parameter: {key}")))
}

/// Parse request body as JSON
///
/// / 将请求体解析为 JSON
///
/// # Type Parameters
///
/// # 类型参数
///
/// - `T` - The type to deserialize into (must implement `DeserializeOwned`)
///   要反序列化成的类型（必须实现 `DeserializeOwned`）
///
/// # Errors
///
/// # 错误
///
/// Returns `RouteError::BadRequest` if the JSON is invalid.
///
/// 如果 JSON 无效，返回 `RouteError::BadRequest`。
///
/// # Example
///
/// # 示例
///
/// ```rust,ignore
/// #[derive(Deserialize)]
/// struct CreateUserRequest {
///     name: String,
///     email: String,
/// }
///
/// let body: CreateUserRequest = get_body(&event, &bytes)?;
/// ```
pub fn get_body<T: serde::de::DeserializeOwned>(event: &Event, bytes: &[u8]) -> Result<T> {
    event.parse_json(bytes)
}

/// Get raw request body bytes
///
/// / 获取原始请求体字节
///
/// This is a no-op that simply returns the bytes slice.
/// Use this when you need the raw bytes without parsing.
///
/// 这是一个无操作函数，仅返回字节切片。当您需要未解析的原始字节时使用。
///
/// # Example
///
/// # 示例
///
/// ```rust,ignore
/// let data = get_body_bytes(&event, &bytes)?;
/// // Process raw bytes...
/// ```
pub fn get_body_bytes<'a>(_event: &'a Event, bytes: &'a [u8]) -> Result<&'a [u8]> {
    Ok(bytes)
}

/// Get request body as text
///
/// / 获取请求体文本
///
/// Validates that the body is valid UTF-8.
///
/// 验证请求体是有效的 UTF-8。
///
/// # Errors
///
/// # 错误
///
/// Returns `RouteError::BadRequest` if the body is not valid UTF-8.
///
/// 如果请求体不是有效的 UTF-8，返回 `RouteError::BadRequest`。
///
/// # Example
///
/// # 示例
///
/// ```rust,ignore
/// let text = get_body_text(&event, &bytes)?;
/// ```
pub fn get_body_text(event: &Event, bytes: &[u8]) -> Result<String> {
    event.parse_text(bytes)
}

/// Get a request header by name
///
/// / 根据名称获取请求头
///
/// Returns `None` if the header doesn't exist or is invalid UTF-8.
///
/// 如果请求头不存在或无效的 UTF-8，返回 `None`。
///
/// # Example
///
/// # 示例
///
/// ```rust,ignore
/// let auth = get_header(&event, "authorization")
///     .ok_or_else(|| RouteError::unauthorized("Missing authorization header"))?;
/// ```
#[must_use]
pub fn get_header<'a>(event: &'a Event, name: &str) -> Option<&'a str> {
    event.headers().get(name).and_then(|v| v.to_str().ok())
}

/// Get all request headers
///
/// / 获取所有请求头
///
/// Returns a reference to the complete header map.
///
/// 返回完整请求头映射的引用。
///
/// # Example
///
/// # 示例
///
/// ```rust,ignore
/// let headers = get_headers(&event);
/// for (name, value) in headers.iter() {
///     println!("{}: {}", name, value.to_str().unwrap_or("?"));
/// }
/// ```
#[must_use]
pub fn get_headers(event: &Event) -> &HeaderMap {
    event.headers()
}

/// Get application state by type
///
/// / 根据类型获取应用状态
///
/// # Type Parameters
///
/// # 类型参数
///
/// - `T` - The type to retrieve (must be `Clone + Send + Sync + 'static`)
///   要检索的类型（必须是 `Clone + Send + Sync + 'static`）
///
/// # Errors
///
/// # 错误
///
/// Returns `RouteError::Internal` if the state is not found.
///
/// 如果未找到状态，返回 `RouteError::Internal`。
///
/// # Example
///
/// # 示例
///
/// ```rust,ignore
/// struct DatabasePool {
///     // ...
/// }
///
/// let pool = get_state::<DatabasePool>(&event)?;
/// ```
pub fn get_state<T: Clone + Send + Sync + 'static>(event: &Event) -> Result<T> {
    event
        .state()
        .ok_or_else(|| RouteError::Internal(anyhow::anyhow!("State not found")))
}

/// Get the HTTP method
///
/// / 获取 HTTP 方法
///
/// # Example
///
/// # 示例
///
/// ```rust,ignore
/// let method = get_method(&event);
/// if method == Method::POST {
///     // Handle POST...
/// }
/// ```
#[must_use]
pub fn get_method(event: &Event) -> &axum::http::Method {
    event.method()
}

/// Get the request path
///
/// / 获取请求路径
///
/// Returns the path without query string.
///
/// 返回不含查询字符串的路径。
///
/// # Example
///
/// # 示例
///
/// ```rust,ignore
/// let path = get_path(&event); // "/users/123"
/// ```
#[must_use]
pub fn get_path(event: &Event) -> &str {
    event.path()
}

/// Get the request URI
///
/// / 获取请求 URI
///
/// Returns the complete URI including path and query string.
///
/// 返回完整的 URI，包括路径和查询字符串。
///
/// # Example
///
/// # 示例
///
/// ```rust,ignore
/// let uri = get_uri(&event); // "/users/123?verbose=true"
/// ```
#[must_use]
pub fn get_uri(event: &Event) -> &axum::http::Uri {
    event.uri()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Method;

    #[test]
    fn test_get_param() {
        let mut params = std::collections::HashMap::new();
        params.insert("id".to_string(), "123".to_string());

        let event = Event::new(
            Method::GET,
            "/users/123".to_string(),
            "/users/123".parse().unwrap(),
            HeaderMap::new(),
            params,
            std::collections::HashMap::new(),
        );

        assert_eq!(get_param(&event, "id"), Some("123"));
        assert_eq!(get_param(&event, "missing"), None);
    }

    #[test]
    fn test_get_param_required() {
        let mut params = std::collections::HashMap::new();
        params.insert("id".to_string(), "123".to_string());

        let event = Event::new(
            Method::GET,
            "/users/123".to_string(),
            "/users/123".parse().unwrap(),
            HeaderMap::new(),
            params,
            std::collections::HashMap::new(),
        );

        assert_eq!(get_param_required(&event, "id").unwrap(), "123");
        assert!(get_param_required(&event, "missing").is_err());
    }
}
