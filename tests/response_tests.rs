//! 全面测试 Response 模块的所有响应构建功能

use astrea::prelude::*;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;

// ============================================================================
// JSON 响应测试
// ============================================================================

#[test]
fn test_json_response_simple() {
    let data = json!({
        "message": "Hello, World!",
        "status": "success"
    });

    let response = json(data).unwrap();

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(
        response.headers.get("content-type").unwrap(),
        "application/json"
    );

    let body_str = String::from_utf8_lossy(&response.body);
    assert!(body_str.contains("Hello, World!"));
    assert!(body_str.contains("success"));
}

#[test]
fn test_json_response_complex_struct() {
    #[derive(Serialize)]
    struct User {
        id: u64,
        name: String,
        email: String,
        roles: Vec<String>,
    }

    let user = User {
        id: 123,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        roles: vec!["admin".to_string(), "user".to_string()],
    };

    let response = json(user).unwrap();

    assert_eq!(response.status, StatusCode::OK);

    let body_str = String::from_utf8_lossy(&response.body);
    assert!(body_str.contains("123"));
    assert!(body_str.contains("Alice"));
    assert!(body_str.contains("alice@example.com"));
    assert!(body_str.contains("admin"));
}

#[test]
fn test_json_response_array() {
    let data = json!([
        {"id": 1, "name": "Item 1"},
        {"id": 2, "name": "Item 2"},
        {"id": 3, "name": "Item 3"}
    ]);

    let response = json(data).unwrap();

    assert_eq!(response.status, StatusCode::OK);

    let body_str = String::from_utf8_lossy(&response.body);
    assert!(body_str.contains("Item 1"));
    assert!(body_str.contains("Item 2"));
    assert!(body_str.contains("Item 3"));
}

#[test]
fn test_json_response_empty_object() {
    let data = json!({});
    let response = json(data).unwrap();

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(String::from_utf8_lossy(&response.body), "{}");
}

// ============================================================================
// Text 响应测试
// ============================================================================

#[test]
fn test_text_response_simple() {
    let response = text("Hello, World!");

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(
        response.headers.get("content-type").unwrap(),
        "text/plain; charset=utf-8"
    );
    assert_eq!(String::from_utf8_lossy(&response.body), "Hello, World!");
}

#[test]
fn test_text_response_empty() {
    let response = text("");

    assert_eq!(response.status, StatusCode::OK);
    assert!(response.body.is_empty());
}

#[test]
fn test_text_response_multiline() {
    let content = "Line 1\nLine 2\nLine 3";
    let response = text(content);

    assert_eq!(String::from_utf8_lossy(&response.body), content);
}

#[test]
fn test_text_response_unicode() {
    let content = "Hello 世界 🌍 مرحبا мир";
    let response = text(content);

    assert_eq!(String::from_utf8_lossy(&response.body), content);
}

#[test]
fn test_text_response_from_string() {
    let content = String::from("Dynamic string content");
    let response = text(content.clone());

    assert_eq!(String::from_utf8_lossy(&response.body), content);
}

// ============================================================================
// HTML 响应测试
// ============================================================================

#[test]
fn test_html_response_simple() {
    let html_content = "<h1>Hello, World!</h1>";
    let response = html(html_content);

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(
        response.headers.get("content-type").unwrap(),
        "text/html; charset=utf-8"
    );
    assert_eq!(String::from_utf8_lossy(&response.body), html_content);
}

#[test]
fn test_html_response_complete_page() {
    let html_content = r"<!DOCTYPE html>
<html>
<head>
    <title>Test Page</title>
</head>
<body>
    <h1>Welcome</h1>
    <p>This is a test page.</p>
</body>
</html>";

    let response = html(html_content);

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(String::from_utf8_lossy(&response.body), html_content);
}

#[test]
fn test_html_response_with_special_chars() {
    let html_content = r#"<div>&lt;script&gt;alert("XSS")&lt;/script&gt;</div>"#;
    let response = html(html_content);

    assert_eq!(String::from_utf8_lossy(&response.body), html_content);
}

// ============================================================================
// Redirect 响应测试
// ============================================================================

#[test]
fn test_redirect_response() {
    let response = redirect("/login").unwrap();

    assert_eq!(response.status, StatusCode::FOUND);
    assert_eq!(response.headers.get("location").unwrap(), "/login");
    assert!(response.body.is_empty());
}

#[test]
fn test_redirect_response_absolute_url() {
    let url = "https://example.com/auth";
    let response = redirect(url).unwrap();

    assert_eq!(response.status, StatusCode::FOUND);
    assert_eq!(response.headers.get("location").unwrap(), url);
}

#[test]
fn test_redirect_response_with_query() {
    let url = "/search?q=rust&page=2";
    let response = redirect(url).unwrap();

    assert_eq!(response.status, StatusCode::FOUND);
    assert_eq!(response.headers.get("location").unwrap(), url);
}

#[test]
fn test_redirect_response_invalid_url() {
    // 包含无效字符的 URL
    let invalid_url = "/path\nwith\nnewlines";
    let result = redirect(invalid_url);

    assert!(result.is_err());
    match result {
        Err(RouteError::BadRequest(msg)) => {
            assert!(msg.contains("Invalid redirect URL"));
        }
        _ => panic!("Expected BadRequest error"),
    }
}

// ============================================================================
// No Content 响应测试
// ============================================================================

#[test]
fn test_no_content_response() {
    let response = no_content();

    assert_eq!(response.status, StatusCode::NO_CONTENT);
    assert!(response.body.is_empty());
    assert!(response.headers.is_empty());
}

// ============================================================================
// Bytes 响应测试
// ============================================================================

#[test]
fn test_bytes_response() {
    let data = vec![0x89, 0x50, 0x4E, 0x47]; // PNG header
    let response = bytes(data.clone());

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body, data);
}

#[test]
fn test_bytes_response_empty() {
    let response = bytes(vec![]);

    assert_eq!(response.status, StatusCode::OK);
    assert!(response.body.is_empty());
}

#[test]
fn test_bytes_response_with_content_type() {
    let data = vec![0xFF, 0xD8, 0xFF]; // JPEG header
    let response = bytes(data.clone()).content_type("image/jpeg");

    assert_eq!(response.body, data);
    assert_eq!(response.headers.get("content-type").unwrap(), "image/jpeg");
}

// ============================================================================
// Response 链式调用测试
// ============================================================================

#[test]
fn test_response_chaining_status() {
    let response = json(json!({"created": true}))
        .unwrap()
        .status(StatusCode::CREATED);

    assert_eq!(response.status, StatusCode::CREATED);
}

#[test]
fn test_response_chaining_header() {
    let response = text("Test").header("X-Custom-Header", "custom-value");

    assert_eq!(
        response.headers.get("x-custom-header").unwrap(),
        "custom-value"
    );
}

#[test]
fn test_response_chaining_multiple() {
    let response = json(json!({"data": "test"}))
        .unwrap()
        .status(StatusCode::CREATED)
        .header("X-Request-Id", "abc123")
        .header("X-Version", "1.0");

    assert_eq!(response.status, StatusCode::CREATED);
    assert_eq!(response.headers.get("x-request-id").unwrap(), "abc123");
    assert_eq!(response.headers.get("x-version").unwrap(), "1.0");
}

#[test]
fn test_response_chaining_content_type() {
    let response = bytes(vec![1, 2, 3])
        .content_type("application/octet-stream")
        .status(StatusCode::OK);

    assert_eq!(
        response.headers.get("content-type").unwrap(),
        "application/octet-stream"
    );
    assert_eq!(response.status, StatusCode::OK);
}

#[test]
fn test_response_chaining_override_content_type() {
    let response = text("Hello").content_type("application/json");

    // 应该覆盖默认的 text/plain
    assert_eq!(
        response.headers.get("content-type").unwrap(),
        "application/json"
    );
}

// ============================================================================
// Response 默认值测试
// ============================================================================

#[test]
fn test_response_default() {
    let response = Response::default();

    assert_eq!(response.status, StatusCode::OK);
    assert!(response.headers.is_empty());
    assert!(response.body.is_empty());
}

#[test]
fn test_response_new() {
    let response = Response::new();

    assert_eq!(response.status, StatusCode::OK);
    assert!(response.headers.is_empty());
    assert!(response.body.is_empty());
}

// ============================================================================
// Response 转换为 Axum Response 测试
// ============================================================================

#[test]
fn test_response_into_axum_response() {
    let response = text("Test message");
    let axum_response = response.clone().into_axum_response();

    // 验证转换后的响应
    assert_eq!(axum_response.status(), StatusCode::OK);
}

#[test]
fn test_response_into_axum_response_with_server_header() {
    let response = text("Test");
    let axum_response = response.into_axum_response();

    // 应该自动添加 Server 头
    assert!(axum_response.headers().contains_key("server"));
}

#[test]
fn test_response_into_response_trait() {
    let response = json(json!({"test": true})).unwrap();

    // 测试 IntoResponse trait
    let axum_response = response.into_response();
    assert_eq!(axum_response.status(), StatusCode::OK);
}

// ============================================================================
// 状态码测试
// ============================================================================

#[test]
fn test_response_various_status_codes() {
    let status_codes = vec![
        StatusCode::OK,
        StatusCode::CREATED,
        StatusCode::ACCEPTED,
        StatusCode::NO_CONTENT,
        StatusCode::BAD_REQUEST,
        StatusCode::UNAUTHORIZED,
        StatusCode::FORBIDDEN,
        StatusCode::NOT_FOUND,
        StatusCode::INTERNAL_SERVER_ERROR,
    ];

    for status in status_codes {
        let response = text("Test").status(status);
        assert_eq!(response.status, status);
    }
}

// ============================================================================
// 特殊场景测试
// ============================================================================

#[test]
fn test_json_response_with_null() {
    let data = json!(null);
    let response = json(data).unwrap();

    assert_eq!(String::from_utf8_lossy(&response.body), "null");
}

#[test]
fn test_json_response_with_number() {
    let data = json!(42);
    let response = json(data).unwrap();

    assert_eq!(String::from_utf8_lossy(&response.body), "42");
}

#[test]
fn test_json_response_with_boolean() {
    let data = json!(true);
    let response = json(data).unwrap();

    assert_eq!(String::from_utf8_lossy(&response.body), "true");
}

#[test]
fn test_json_response_with_string() {
    let data = json!("just a string");
    let response = json(data).unwrap();

    assert_eq!(
        String::from_utf8_lossy(&response.body),
        r#""just a string""#
    );
}

#[test]
fn test_response_clone() {
    let response1 = text("Original")
        .status(StatusCode::CREATED)
        .header("X-Test", "value");

    let response2 = response1.clone();

    assert_eq!(response1.status, response2.status);
    assert_eq!(response1.body, response2.body);
    assert_eq!(
        response1.headers.get("x-test"),
        response2.headers.get("x-test")
    );
}

#[test]
fn test_response_large_body() {
    let large_text = "x".repeat(1_000_000); // 1MB
    let response = text(large_text.clone());

    assert_eq!(response.body.len(), 1_000_000);
    assert_eq!(String::from_utf8_lossy(&response.body), large_text);
}

#[test]
fn test_json_nested_structure() {
    let data = json!({
        "user": {
            "id": 1,
            "profile": {
                "name": "Alice",
                "settings": {
                    "theme": "dark",
                    "notifications": true
                }
            }
        }
    });

    let response = json(data).unwrap();
    let body_str = String::from_utf8_lossy(&response.body);

    assert!(body_str.contains("Alice"));
    assert!(body_str.contains("dark"));
    assert!(body_str.contains("true"));
}

#[test]
fn test_response_header_overwrite() {
    let response = text("Test")
        .header("X-Custom", "value1")
        .header("X-Custom", "value2"); // 应该覆盖

    assert_eq!(response.headers.get("x-custom").unwrap(), "value2");
}

#[test]
fn test_response_invalid_header_name() {
    // 包含无效字符的头名
    let response = text("Test").header("Invalid\nHeader", "value");

    // 应该默默失败，不添加这个头
    assert!(!response.headers.contains_key("invalid\nheader"));
}

#[test]
fn test_response_invalid_header_value() {
    // 包含无效字符的头值
    let response = text("Test").header("X-Custom", "value\nwith\nnewlines");

    // 应该默默失败，不添加这个头
    assert!(!response.headers.contains_key("x-custom"));
}
