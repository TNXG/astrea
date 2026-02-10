//! 全面测试 Extract 模块的所有提取函数

use astrea::Event;
use astrea::prelude::*;
use axum::http::{HeaderMap, HeaderValue, Method, Uri};
use std::collections::HashMap;
use std::sync::Arc;

// ============================================================================
// 路径参数提取测试
// ============================================================================

#[test]
fn test_get_param_exists() {
    let mut params = HashMap::new();
    params.insert("id".to_string(), "123".to_string());
    params.insert("slug".to_string(), "hello-world".to_string());

    let event = Event::new(
        Method::GET,
        "/posts/123/hello-world".to_string(),
        "/posts/123/hello-world".parse().unwrap(),
        HeaderMap::new(),
        params,
        HashMap::new(),
    );

    assert_eq!(get_param(&event, "id"), Some("123"));
    assert_eq!(get_param(&event, "slug"), Some("hello-world"));
}

#[test]
fn test_get_param_not_exists() {
    let event = Event::new(
        Method::GET,
        "/".to_string(),
        "/".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
    );

    assert_eq!(get_param(&event, "id"), None);
    assert_eq!(get_param(&event, "any_key"), None);
}

#[test]
fn test_get_param_required_exists() {
    let mut params = HashMap::new();
    params.insert("user_id".to_string(), "456".to_string());

    let event = Event::new(
        Method::GET,
        "/users/456".to_string(),
        "/users/456".parse().unwrap(),
        HeaderMap::new(),
        params,
        HashMap::new(),
    );

    let result = get_param_required(&event, "user_id");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "456");
}

#[test]
fn test_get_param_required_not_exists() {
    let event = Event::new(
        Method::GET,
        "/".to_string(),
        "/".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
    );

    let result = get_param_required(&event, "missing_param");
    assert!(result.is_err());

    match result {
        Err(RouteError::BadRequest(msg)) => {
            assert!(msg.contains("Missing required parameter"));
            assert!(msg.contains("missing_param"));
        }
        _ => panic!("Expected BadRequest error"),
    }
}

// ============================================================================
// 查询参数提取测试
// ============================================================================

#[test]
fn test_get_query() {
    let mut query = HashMap::new();
    query.insert("page".to_string(), "2".to_string());
    query.insert("limit".to_string(), "50".to_string());
    query.insert("search".to_string(), "rust".to_string());

    let event = Event::new(
        Method::GET,
        "/api/items".to_string(),
        "/api/items?page=2&limit=50&search=rust".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        query,
    );

    let query_map = get_query(&event);
    assert_eq!(query_map.get("page").unwrap(), "2");
    assert_eq!(query_map.get("limit").unwrap(), "50");
    assert_eq!(query_map.get("search").unwrap(), "rust");
}

#[test]
fn test_get_query_empty() {
    let event = Event::new(
        Method::GET,
        "/api/items".to_string(),
        "/api/items".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
    );

    let query_map = get_query(&event);
    assert!(query_map.is_empty());
}

#[test]
fn test_get_query_param_exists() {
    let mut query = HashMap::new();
    query.insert("filter".to_string(), "active".to_string());

    let event = Event::new(
        Method::GET,
        "/api/users".to_string(),
        "/api/users?filter=active".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        query,
    );

    let result = get_query_param(&event, "filter");
    assert_eq!(result, Some("active".to_string()));
}

#[test]
fn test_get_query_param_not_exists() {
    let event = Event::new(
        Method::GET,
        "/api/users".to_string(),
        "/api/users".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
    );

    let result = get_query_param(&event, "missing");
    assert_eq!(result, None);
}

#[test]
fn test_get_query_param_required_exists() {
    let mut query = HashMap::new();
    query.insert("token".to_string(), "abc123".to_string());

    let event = Event::new(
        Method::GET,
        "/verify".to_string(),
        "/verify?token=abc123".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        query,
    );

    let result = get_query_param_required(&event, "token");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "abc123");
}

#[test]
fn test_get_query_param_required_not_exists() {
    let event = Event::new(
        Method::GET,
        "/verify".to_string(),
        "/verify".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
    );

    let result = get_query_param_required(&event, "token");
    assert!(result.is_err());

    match result {
        Err(RouteError::BadRequest(msg)) => {
            assert!(msg.contains("Missing required query parameter"));
            assert!(msg.contains("token"));
        }
        _ => panic!("Expected BadRequest error"),
    }
}

// ============================================================================
// 请求体提取测试
// ============================================================================

#[test]
fn test_get_body_json() {
    let event = Event::new(
        Method::POST,
        "/api/users".to_string(),
        "/api/users".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
    );

    #[derive(serde::Deserialize, PartialEq, Debug)]
    struct User {
        name: String,
        email: String,
    }

    let json_bytes = br#"{"name":"Bob","email":"bob@example.com"}"#;
    let result = get_body::<User>(&event, json_bytes);

    assert!(result.is_ok());
    let user = result.unwrap();
    assert_eq!(user.name, "Bob");
    assert_eq!(user.email, "bob@example.com");
}

#[test]
fn test_get_body_json_invalid() {
    let event = Event::new(
        Method::POST,
        "/api/users".to_string(),
        "/api/users".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
    );

    #[derive(serde::Deserialize)]
    #[allow(dead_code)]
    struct User {
        name: String,
        email: String,
    }

    let invalid_json = b"{invalid json}";
    let result = get_body::<User>(&event, invalid_json);

    assert!(result.is_err());
}

#[test]
fn test_get_body_bytes() {
    let event = Event::new(
        Method::POST,
        "/upload".to_string(),
        "/upload".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
    );

    let data = b"Binary data \x00\x01\x02\xFF";
    let result = get_body_bytes(&event, data);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), data);
}

#[test]
fn test_get_body_text() {
    let event = Event::new(
        Method::POST,
        "/api/message".to_string(),
        "/api/message".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
    );

    let text_data = b"Hello, this is a text message!";
    let result = get_body_text(&event, text_data);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello, this is a text message!");
}

#[test]
fn test_get_body_text_invalid_utf8() {
    let event = Event::new(
        Method::POST,
        "/api/message".to_string(),
        "/api/message".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
    );

    let invalid_utf8 = &[0xFF, 0xFE, 0xFD];
    let result = get_body_text(&event, invalid_utf8);

    assert!(result.is_err());
    match result {
        Err(RouteError::BadRequest(msg)) => {
            assert!(msg.contains("Invalid UTF-8"));
        }
        _ => panic!("Expected BadRequest error"),
    }
}

// ============================================================================
// 请求头提取测试
// ============================================================================

#[test]
fn test_get_header_exists() {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    headers.insert("Authorization", HeaderValue::from_static("Bearer token123"));
    headers.insert("X-Custom-Header", HeaderValue::from_static("custom-value"));

    let event = Event::new(
        Method::GET,
        "/api/data".to_string(),
        "/api/data".parse().unwrap(),
        headers,
        HashMap::new(),
        HashMap::new(),
    );

    assert_eq!(get_header(&event, "content-type"), Some("application/json"));
    assert_eq!(get_header(&event, "authorization"), Some("Bearer token123"));
    assert_eq!(get_header(&event, "x-custom-header"), Some("custom-value"));
}

#[test]
fn test_get_header_not_exists() {
    let event = Event::new(
        Method::GET,
        "/".to_string(),
        "/".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
    );

    assert_eq!(get_header(&event, "missing-header"), None);
}

#[test]
fn test_get_header_case_insensitive() {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("text/html"));

    let event = Event::new(
        Method::GET,
        "/".to_string(),
        "/".parse().unwrap(),
        headers,
        HashMap::new(),
        HashMap::new(),
    );

    // HTTP 头名是不区分大小写的
    assert_eq!(get_header(&event, "content-type"), Some("text/html"));
    assert_eq!(get_header(&event, "Content-Type"), Some("text/html"));
    assert_eq!(get_header(&event, "CONTENT-TYPE"), Some("text/html"));
}

#[test]
fn test_get_headers() {
    let mut headers = HeaderMap::new();
    headers.insert("Accept", HeaderValue::from_static("*/*"));
    headers.insert("User-Agent", HeaderValue::from_static("Test/1.0"));

    let event = Event::new(
        Method::GET,
        "/".to_string(),
        "/".parse().unwrap(),
        headers.clone(),
        HashMap::new(),
        HashMap::new(),
    );

    let retrieved_headers = get_headers(&event);
    assert_eq!(retrieved_headers.get("accept").unwrap(), "*/*");
    assert_eq!(retrieved_headers.get("user-agent").unwrap(), "Test/1.0");
}

// ============================================================================
// 状态提取测试
// ============================================================================

#[test]
fn test_get_state_success() {
    #[derive(Clone, Debug, PartialEq)]
    struct Database {
        connection_string: String,
    }

    let db = Database {
        connection_string: "postgresql://localhost/mydb".to_string(),
    };

    let mut event = Event::new(
        Method::GET,
        "/".to_string(),
        "/".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
    );

    event.state = Some(Arc::new(db.clone()));

    let result = get_state::<Database>(&event);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), db);
}

#[test]
fn test_get_state_not_found() {
    let event = Event::new(
        Method::GET,
        "/".to_string(),
        "/".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
    );

    #[derive(Clone)]
    #[allow(dead_code)]
    struct AppState {
        value: i32,
    }

    let result = get_state::<AppState>(&event);
    assert!(result.is_err());

    match result {
        Err(RouteError::Internal(_)) => {
            // 预期的错误类型
        }
        _ => panic!("Expected Internal error"),
    }
}

// ============================================================================
// 基本信息提取测试
// ============================================================================

#[test]
fn test_get_method() {
    let methods = vec![
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::DELETE,
        Method::PATCH,
    ];

    for method in methods {
        let event = Event::new(
            method.clone(),
            "/test".to_string(),
            "/test".parse().unwrap(),
            HeaderMap::new(),
            HashMap::new(),
            HashMap::new(),
        );

        assert_eq!(get_method(&event), &method);
    }
}

#[test]
fn test_get_path() {
    let paths = vec!["/", "/api/users", "/posts/123/comments", "/search?q=rust"];

    for path in paths {
        let event = Event::new(
            Method::GET,
            path.to_string(),
            path.parse().unwrap(),
            HeaderMap::new(),
            HashMap::new(),
            HashMap::new(),
        );

        assert_eq!(get_path(&event), path);
    }
}

#[test]
fn test_get_uri() {
    let uri_str = "/api/users?page=1&limit=10";
    let uri: Uri = uri_str.parse().unwrap();

    let event = Event::new(
        Method::GET,
        "/api/users".to_string(),
        uri.clone(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
    );

    assert_eq!(get_uri(&event), &uri);
    assert_eq!(get_uri(&event).path(), "/api/users");
    assert_eq!(get_uri(&event).query(), Some("page=1&limit=10"));
}

// ============================================================================
// 组合场景测试
// ============================================================================

#[test]
fn test_extract_combined_scenario() {
    // 模拟一个真实的 API 请求场景
    let mut params = HashMap::new();
    params.insert("user_id".to_string(), "789".to_string());

    let mut query = HashMap::new();
    query.insert("include".to_string(), "profile".to_string());
    query.insert("format".to_string(), "detailed".to_string());

    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_static("Bearer jwt_token"),
    );
    headers.insert("Accept", HeaderValue::from_static("application/json"));

    let uri: Uri = "/api/users/789?include=profile&format=detailed"
        .parse()
        .unwrap();

    let event = Event::new(
        Method::GET,
        "/api/users/789".to_string(),
        uri,
        headers,
        params,
        query,
    );

    // 验证所有提取功能
    assert_eq!(get_param(&event, "user_id"), Some("789"));
    assert_eq!(
        get_query_param(&event, "include"),
        Some("profile".to_string())
    );
    assert_eq!(
        get_query_param(&event, "format"),
        Some("detailed".to_string())
    );
    assert_eq!(
        get_header(&event, "authorization"),
        Some("Bearer jwt_token")
    );
    assert_eq!(get_method(&event), &Method::GET);
    assert_eq!(get_path(&event), "/api/users/789");
}

#[test]
fn test_extract_post_request_with_body() {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));

    let event = Event::new(
        Method::POST,
        "/api/posts".to_string(),
        "/api/posts".parse().unwrap(),
        headers,
        HashMap::new(),
        HashMap::new(),
    );

    #[derive(serde::Deserialize, Debug, PartialEq)]
    struct CreatePost {
        title: String,
        content: String,
        tags: Vec<String>,
    }

    let json_data = br#"{
        "title": "Rust Web Development",
        "content": "Building web apps with Rust is awesome!",
        "tags": ["rust", "web", "backend"]
    }"#;

    let post: CreatePost = get_body(&event, json_data).unwrap();
    assert_eq!(post.title, "Rust Web Development");
    assert_eq!(post.content, "Building web apps with Rust is awesome!");
    assert_eq!(post.tags, vec!["rust", "web", "backend"]);
}
