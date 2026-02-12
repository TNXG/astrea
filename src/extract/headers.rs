//! Request header extraction
//!
//! / 请求头提取

use crate::Event;
use axum::http::HeaderMap;

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
