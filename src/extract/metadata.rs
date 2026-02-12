//! Request metadata extraction (method, path, URI)
//!
//! / 请求元数据提取（方法、路径、URI）

use crate::Event;

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
