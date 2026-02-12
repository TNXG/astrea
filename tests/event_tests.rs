//! 全面测试 Event 模块的功能

use astrea::Event;
use astrea::prelude::*;
use axum::http::{HeaderMap, HeaderValue, Method, Uri};
use std::collections::HashMap;
use std::sync::Arc;

#[test]
fn test_event_creation() {
    let params = HashMap::new();
    let query = HashMap::new();
    let headers = HeaderMap::new();

    let event = Event::new(
        Method::GET,
        "/api/users".to_string(),
        "/api/users".parse().unwrap(),
        headers,
        params,
        query,
        bytes::Bytes::new(),
    );

    assert_eq!(event.method(), &Method::GET);
    assert_eq!(event.path(), "/api/users");
}

#[test]
fn test_event_with_params() {
    let mut params = HashMap::new();
    params.insert("id".to_string(), "123".to_string());
    params.insert("name".to_string(), "john".to_string());

    let event = Event::new(
        Method::GET,
        "/users/123".to_string(),
        "/users/123".parse().unwrap(),
        HeaderMap::new(),
        params,
        HashMap::new(),
        bytes::Bytes::new(),
    );

    let params = event.params();
    assert_eq!(params.get("id").unwrap(), "123");
    assert_eq!(params.get("name").unwrap(), "john");
    assert_eq!(params.get("missing"), None);
}

#[test]
fn test_event_query_params_from_uri() {
    // 在实际使用中，#[route] 宏会从 URI 提取查询参数
    // 这里我们模拟这个过程
    let uri: Uri = "/api/search?q=rust&page=1&limit=10".parse().unwrap();

    // 模拟宏会做的事：从 URI 解析查询参数
    let query_str = uri.query().unwrap();
    let parsed_query: HashMap<String, String> = serde_urlencoded::from_str(query_str).unwrap();

    let event = Event::new(
        Method::GET,
        "/api/search".to_string(),
        uri,
        HeaderMap::new(),
        HashMap::new(),
        parsed_query,
        bytes::Bytes::new(),
    );

    let query = event.query();
    assert_eq!(query.get("q").unwrap(), "rust");
    assert_eq!(query.get("page").unwrap(), "1");
    assert_eq!(query.get("limit").unwrap(), "10");
}

#[test]
fn test_event_with_preloaded_query() {
    let mut query = HashMap::new();
    query.insert("status".to_string(), "active".to_string());
    query.insert("sort".to_string(), "desc".to_string());

    let event = Event::new(
        Method::GET,
        "/api/items".to_string(),
        "/api/items?status=active&sort=desc".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        query,
        bytes::Bytes::new(),
    );

    let query_params = event.query();
    assert_eq!(query_params.get("status").unwrap(), "active");
    assert_eq!(query_params.get("sort").unwrap(), "desc");
}

#[test]
fn test_event_headers() {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    headers.insert("Authorization", HeaderValue::from_static("Bearer token123"));
    headers.insert("X-Request-Id", HeaderValue::from_static("abc-123"));

    let event = Event::new(
        Method::POST,
        "/api/data".to_string(),
        "/api/data".parse().unwrap(),
        headers.clone(),
        HashMap::new(),
        HashMap::new(),
        bytes::Bytes::new(),
    );

    let event_headers = event.headers();
    assert_eq!(
        event_headers.get("content-type").unwrap(),
        "application/json"
    );
    assert_eq!(
        event_headers.get("authorization").unwrap(),
        "Bearer token123"
    );
    assert_eq!(event_headers.get("x-request-id").unwrap(), "abc-123");
}

#[test]
fn test_event_uri() {
    let uri: Uri = "/api/users/123?include=profile".parse().unwrap();

    let event = Event::new(
        Method::GET,
        "/api/users/123".to_string(),
        uri.clone(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
        bytes::Bytes::new(),
    );

    assert_eq!(event.uri(), &uri);
    assert_eq!(event.uri().path(), "/api/users/123");
    assert_eq!(event.uri().query(), Some("include=profile"));
}

#[test]
fn test_event_method_types() {
    let methods = vec![
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::DELETE,
        Method::PATCH,
        Method::HEAD,
        Method::OPTIONS,
    ];

    for method in methods {
        let event = Event::new(
            method.clone(),
            "/test".to_string(),
            "/test".parse().unwrap(),
            HeaderMap::new(),
            HashMap::new(),
            HashMap::new(),
            bytes::Bytes::new(),
        );

        assert_eq!(event.method(), &method);
    }
}

#[test]
fn test_event_parse_json_valid() {
    let event = Event::new(
        Method::POST,
        "/api/data".to_string(),
        "/api/data".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
        bytes::Bytes::new(),
    );

    #[derive(serde::Deserialize, PartialEq, Debug)]
    struct TestData {
        name: String,
        age: u32,
    }

    let json_bytes = br#"{"name":"Alice","age":30}"#;
    let parsed: TestData = event.parse_json(json_bytes).unwrap();

    assert_eq!(parsed.name, "Alice");
    assert_eq!(parsed.age, 30);
}

#[test]
fn test_event_parse_json_invalid() {
    let event = Event::new(
        Method::POST,
        "/api/data".to_string(),
        "/api/data".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
        bytes::Bytes::new(),
    );

    #[derive(serde::Deserialize)]
    #[allow(dead_code)]
    struct TestData {
        name: String,
    }

    let invalid_json = b"not valid json";
    let result = event.parse_json::<TestData>(invalid_json);

    assert!(result.is_err());
    match result {
        Err(RouteError::BadRequest(msg)) => {
            assert!(msg.contains("Invalid JSON"));
        }
        _ => panic!("Expected BadRequest error"),
    }
}

#[test]
fn test_event_parse_text_valid() {
    let event = Event::new(
        Method::POST,
        "/api/text".to_string(),
        "/api/text".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
        bytes::Bytes::new(),
    );

    let text_bytes = b"Hello, Astrea!";
    let parsed = event.parse_text(text_bytes).unwrap();

    assert_eq!(parsed, "Hello, Astrea!");
}

#[test]
fn test_event_parse_text_invalid_utf8() {
    let event = Event::new(
        Method::POST,
        "/api/text".to_string(),
        "/api/text".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
        bytes::Bytes::new(),
    );

    let invalid_utf8 = &[0xFF, 0xFE, 0xFD]; // Invalid UTF-8 bytes
    let result = event.parse_text(invalid_utf8);

    assert!(result.is_err());
    match result {
        Err(RouteError::BadRequest(msg)) => {
            assert!(msg.contains("Invalid UTF-8"));
        }
        _ => panic!("Expected BadRequest error"),
    }
}

#[test]
fn test_event_parse_form_valid() {
    let event = Event::new(
        Method::POST,
        "/api/form".to_string(),
        "/api/form".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
        bytes::Bytes::new(),
    );

    #[derive(serde::Deserialize, PartialEq, Debug)]
    struct FormData {
        username: String,
        password: String,
    }

    let form_bytes = b"username=alice&password=secret123";
    let parsed: FormData = event.parse_form(form_bytes).unwrap();

    assert_eq!(parsed.username, "alice");
    assert_eq!(parsed.password, "secret123");
}

#[test]
fn test_event_parse_form_invalid() {
    let event = Event::new(
        Method::POST,
        "/api/form".to_string(),
        "/api/form".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
        bytes::Bytes::new(),
    );

    #[derive(serde::Deserialize)]
    #[allow(dead_code)]
    struct FormData {
        required_field: String,
    }

    let invalid_form = b"wrong=data";
    let result: Result<FormData> = event.parse_form(invalid_form);

    assert!(result.is_err());
}

#[test]
fn test_event_state() {
    #[derive(Clone, Debug, PartialEq)]
    struct AppState {
        db_url: String,
        max_connections: u32,
    }

    let state = AppState {
        db_url: "postgresql://localhost".to_string(),
        max_connections: 100,
    };

    let mut event = Event::new(
        Method::GET,
        "/test".to_string(),
        "/test".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
        bytes::Bytes::new(),
    );

    event.state = Some(Arc::new(state.clone()));

    let retrieved_state: AppState = event.state().unwrap();
    assert_eq!(retrieved_state, state);
}

#[test]
fn test_event_state_none() {
    let event = Event::new(
        Method::GET,
        "/test".to_string(),
        "/test".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
        bytes::Bytes::new(),
    );

    #[derive(Clone)]
    #[allow(dead_code)]
    struct AppState {
        value: i32,
    }

    let result: Option<AppState> = event.state();
    assert!(result.is_none());
}

#[test]
fn test_event_state_wrong_type() {
    #[derive(Clone)]
    #[allow(dead_code)]
    struct StateA {
        value: i32,
    }

    #[derive(Clone)]
    #[allow(dead_code)]
    struct StateB {
        value: String,
    }

    let state = StateA { value: 42 };

    let mut event = Event::new(
        Method::GET,
        "/test".to_string(),
        "/test".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
        bytes::Bytes::new(),
    );

    event.state = Some(Arc::new(state));

    // 尝试获取错误的类型
    let result: Option<StateB> = event.state();
    assert!(result.is_none());
}

#[test]
fn test_event_clone() {
    let mut params = HashMap::new();
    params.insert("id".to_string(), "456".to_string());

    let event1 = Event::new(
        Method::POST,
        "/users/456".to_string(),
        "/users/456".parse().unwrap(),
        HeaderMap::new(),
        params,
        HashMap::new(),
        bytes::Bytes::new(),
    );

    let event2 = event1.clone();

    assert_eq!(event1.method(), event2.method());
    assert_eq!(event1.path(), event2.path());
    assert_eq!(event1.params().get("id"), event2.params().get("id"));
}

#[test]
fn test_event_empty_query_string() {
    let uri: Uri = "/api/test?".parse().unwrap();

    let event = Event::new(
        Method::GET,
        "/api/test".to_string(),
        uri,
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
        bytes::Bytes::new(),
    );

    let query = event.query();
    assert!(query.is_empty());
}

#[test]
fn test_event_complex_query_params() {
    let uri: Uri = "/search?tag=rust&tag=web&status=active".parse().unwrap();

    // 解析查询参数（serde_urlencoded 对重复键的处理）
    let query_str = uri.query().unwrap();
    let parsed_query: HashMap<String, String> = serde_urlencoded::from_str(query_str).unwrap();

    let event = Event::new(
        Method::GET,
        "/search".to_string(),
        uri,
        HeaderMap::new(),
        HashMap::new(),
        parsed_query,
        bytes::Bytes::new(),
    );

    let query = event.query();
    // serde_urlencoded 对重复的键只保留最后一个值
    assert!(query.contains_key("tag") || query.contains_key("status"));
    // 验证至少有一个键值对被解析
    assert!(!query.is_empty());
}
