//! Path parameter extraction
//!
//! / 路径参数提取

use crate::{
    Event,
    error::{Result, RouteError},
};

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
