//! Tests for extract module
//!
//! / extract 模块的测试

use super::*;
use crate::Event;
use axum::http::Method;

#[test]
fn test_get_param() {
    let mut params = std::collections::HashMap::new();
    params.insert("id".to_string(), "123".to_string());

    let event = Event::new(
        Method::GET,
        "/users/123".to_string(),
        "/users/123".parse().unwrap(),
        axum::http::HeaderMap::new(),
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
        axum::http::HeaderMap::new(),
        params,
        std::collections::HashMap::new(),
    );

    assert_eq!(get_param_required(&event, "id").unwrap(), "123");
    assert!(get_param_required(&event, "missing").is_err());
}
