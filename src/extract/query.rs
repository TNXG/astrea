//! Query parameter extraction
//!
//! / 查询参数提取

use crate::{
    Event,
    error::{Result, RouteError},
};

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
