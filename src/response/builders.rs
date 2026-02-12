//! Response builder functions
//!
//! / 响应构建器函数

use crate::error::{Result, RouteError};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use serde::Serialize;

use super::Response;

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
