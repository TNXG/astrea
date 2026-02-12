//! Request body extraction
//!
//! / 请求体提取

use crate::{Event, error::Result};

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
/// let body: CreateUserRequest = get_body(&event)?;
/// ```
pub fn get_body<T: serde::de::DeserializeOwned>(event: &Event) -> Result<T> {
    event.parse_json(&event.body)
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
/// let data = get_body_bytes(&event)?;
/// // Process raw bytes...
/// ```
pub fn get_body_bytes(event: &Event) -> Result<&[u8]> {
    Ok(&event.body)
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
/// let text = get_body_text(&event)?;
/// ```
pub fn get_body_text(event: &Event) -> Result<String> {
    event.parse_text(&event.body)
}
