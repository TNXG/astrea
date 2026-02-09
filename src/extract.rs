//! 从 Event 提取数据的辅助函数
//!
//! 这些函数提供便捷的 API 来访问请求数据，
//! 无需复杂的 Axum 提取器签名。

use crate::{error::{Result, RouteError}, Event};
use axum::http::HeaderMap;

/// 获取路径参数
///
/// # 示例
///
/// ```rust,ignore
/// let user_id = get_param(&event, "id").unwrap();
/// ```
pub fn get_param<'a>(event: &'a Event, key: &str) -> Option<&'a str> {
    event.params().get(key).map(|s| s.as_str())
}

/// 获取必需的路径参数
///
/// 如果参数不存在则返回错误。
///
/// # 示例
///
/// ```rust,ignore
/// let user_id = get_param_required(&event, "id")?;
/// ```
pub fn get_param_required<'a>(event: &'a Event, key: &str) -> Result<&'a str> {
    get_param(event, key).ok_or_else(|| {
        RouteError::bad_request(format!("Missing required parameter: {}", key))
    })
}

/// 获取所有查询参数
///
/// # 示例
///
/// ```rust,ignore
/// let query = get_query(&event);
/// let search = query.get("q").unwrap_or(&"".to_string());
/// ```
pub fn get_query(event: &Event) -> &std::collections::HashMap<String, String> {
    event.query()
}

/// 获取查询参数
///
/// # 示例
///
/// ```rust,ignore
/// let code = get_query_param(&event, "code")
///     .ok_or_else(|| error("Missing code"))?;
/// ```
pub fn get_query_param(event: &Event, key: &str) -> Option<String> {
    event.query().get(key).cloned()
}

/// 获取必需的查询参数
///
/// 如果参数不存在则返回错误。
pub fn get_query_param_required(event: &Event, key: &str) -> Result<String> {
    get_query_param(event, key).ok_or_else(|| {
        RouteError::bad_request(format!("Missing required query parameter: {}", key))
    })
}

/// 将请求体解析为 JSON
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
/// let body: CreateUserRequest = get_body(&event, bytes)?;
/// ```
pub fn get_body<T: serde::de::DeserializeOwned>(event: &Event, bytes: &[u8]) -> Result<T> {
    event.parse_json(bytes)
}

/// 获取原始请求体字节
///
/// # 示例
///
/// ```rust,ignore
/// let bytes = get_body_bytes(&event, bytes)?;
/// ```
pub fn get_body_bytes<'a>(_event: &'a Event, bytes: &'a [u8]) -> Result<&'a [u8]> {
    Ok(bytes)
}

/// 获取请求体文本
///
/// # 示例
///
/// ```rust,ignore
/// let text = get_body_text(&event, bytes)?;
/// ```
pub fn get_body_text(event: &Event, bytes: &[u8]) -> Result<String> {
    event.parse_text(bytes)
}

/// 获取请求头
///
/// # 示例
///
/// ```rust,ignore
/// let auth = get_header(&event, "authorization")
///     .ok_or_else(|| error("Missing authorization header"))?;
/// ```
pub fn get_header<'a>(event: &'a Event, name: &str) -> Option<&'a str> {
    event
        .headers()
        .get(name)
        .and_then(|v| v.to_str().ok())
}

/// 获取所有请求头
///
/// # 示例
///
/// ```rust,ignore
/// let headers = get_headers(&event);
/// ```
pub fn get_headers(event: &Event) -> &HeaderMap {
    event.headers()
}

/// 获取应用状态
///
/// # 示例
///
/// ```rust,ignore
/// let pool = get_state::<PgPool>(&event)?;
/// ```
pub fn get_state<T: Clone + Send + Sync + 'static>(event: &Event) -> Result<T> {
    event.state().ok_or_else(|| {
        RouteError::Internal(anyhow::anyhow!("State not found"))
    })
}

/// 获取 HTTP 方法
///
/// # 示例
///
/// ```rust,ignore
/// let method = get_method(&event);
/// ```
pub fn get_method(event: &Event) -> &axum::http::Method {
    event.method()
}

/// 获取请求路径
///
/// # 示例
///
/// ```rust,ignore
/// let path = get_path(&event);
/// ```
pub fn get_path(event: &Event) -> &str {
    event.path()
}

/// 获取请求 URI
///
/// # 示例
///
/// ```rust,ignore
/// let uri = get_uri(&event);
/// ```
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
