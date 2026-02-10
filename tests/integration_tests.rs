//! 集成测试 - 测试完整的处理器流程和框架功能

use astrea::prelude::*;
use astrea::Event;
use axum::http::{HeaderMap, HeaderValue, Method};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

// ============================================================================
// 完整的处理器场景测试
// ============================================================================

#[tokio::test]
async fn test_simple_handler_flow() {
    // 模拟一个简单的 GET 请求处理器
    async fn handler(event: Event) -> Result<Response> {
        let name = get_param(&event, "name").unwrap_or("World");
        json(json!({
            "message": format!("Hello, {}!", name),
            "path": get_path(&event)
        }))
    }

    let mut params = HashMap::new();
    params.insert("name".to_string(), "Astrea".to_string());

    let event = Event::new(
        Method::GET,
        "/hello/Astrea".to_string(),
        "/hello/Astrea".parse().unwrap(),
        HeaderMap::new(),
        params,
        HashMap::new(),
    );

    let result = handler(event).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.status, StatusCode::OK);

    let body_str = String::from_utf8_lossy(&response.body);
    assert!(body_str.contains("Hello, Astrea!"));
}

#[tokio::test]
async fn test_post_handler_with_json_body() {
    #[derive(Deserialize, Serialize)]
    #[allow(dead_code)]
    struct CreateUserRequest {
        name: String,
        email: String,
    }

    async fn create_user_handler(_event: Event) -> Result<Response> {
        // 在真实场景中，这里会从请求体获取数据
        // 但我们直接构造来测试逻辑
        Ok(json(json!({
            "id": 1,
            "name": "Test User",
            "email": "test@example.com",
            "created": true
        }))?
        .status(StatusCode::CREATED))
    }

    let event = Event::new(
        Method::POST,
        "/api/users".to_string(),
        "/api/users".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
    );

    let result = create_user_handler(event).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.status, StatusCode::CREATED);
}

#[tokio::test]
async fn test_handler_with_query_params() {
    async fn search_handler(event: Event) -> Result<Response> {
        let query = get_query_param(&event, "q")
            .ok_or_else(|| RouteError::bad_request("Missing search query"))?;

        let page = get_query_param(&event, "page")
            .and_then(|p| p.parse::<u32>().ok())
            .unwrap_or(1);

        json(json!({
            "query": query,
            "page": page,
            "results": []
        }))
    }

    let mut query = HashMap::new();
    query.insert("q".to_string(), "rust".to_string());
    query.insert("page".to_string(), "2".to_string());

    let event = Event::new(
        Method::GET,
        "/search".to_string(),
        "/search?q=rust&page=2".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        query,
    );

    let result = search_handler(event).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    let body_str = String::from_utf8_lossy(&response.body);
    assert!(body_str.contains("rust"));
    assert!(body_str.contains("\"page\":2"));
}

#[tokio::test]
async fn test_handler_with_headers() {
    async fn auth_handler(event: Event) -> Result<Response> {
        let auth_header = get_header(&event, "authorization")
            .ok_or_else(|| RouteError::unauthorized("Missing authorization header"))?;

        if !auth_header.starts_with("Bearer ") {
            return Err(RouteError::unauthorized("Invalid authorization format"));
        }

        json(json!({
            "authenticated": true,
            "message": "Access granted"
        }))
    }

    // 测试成功的认证
    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_static("Bearer valid_token"),
    );

    let event = Event::new(
        Method::GET,
        "/api/protected".to_string(),
        "/api/protected".parse().unwrap(),
        headers,
        HashMap::new(),
        HashMap::new(),
    );

    let result = auth_handler(event).await;
    assert!(result.is_ok());

    // 测试缺少认证头
    let event_no_auth = Event::new(
        Method::GET,
        "/api/protected".to_string(),
        "/api/protected".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
    );

    let result_no_auth = auth_handler(event_no_auth).await;
    assert!(result_no_auth.is_err());
    assert!(matches!(result_no_auth, Err(RouteError::Unauthorized(_))));
}

// ============================================================================
// 状态管理测试
// ============================================================================

#[tokio::test]
async fn test_handler_with_state() {
    #[derive(Clone)]
    struct AppState {
        db_url: String,
        api_key: String,
    }

    async fn handler_with_state(event: Event) -> Result<Response> {
        let state = get_state::<AppState>(&event)?;

        json(json!({
            "db_configured": !state.db_url.is_empty(),
            "api_configured": !state.api_key.is_empty()
        }))
    }

    let state = AppState {
        db_url: "postgresql://localhost/db".to_string(),
        api_key: "secret_key".to_string(),
    };

    let mut event = Event::new(
        Method::GET,
        "/status".to_string(),
        "/status".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
    );

    event.state = Some(Arc::new(state));

    let result = handler_with_state(event).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    let body_str = String::from_utf8_lossy(&response.body);
    assert!(body_str.contains("true"));
}

// ============================================================================
// 错误处理集成测试
// ============================================================================

#[tokio::test]
async fn test_handler_error_propagation() {
    async fn failing_handler(_event: Event) -> Result<Response> {
        Err(RouteError::not_found("Resource not found"))
    }

    let event = Event::new(
        Method::GET,
        "/missing".to_string(),
        "/missing".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
    );

    let result = failing_handler(event).await;
    assert!(result.is_err());

    match result {
        Err(RouteError::NotFound(msg)) => {
            assert_eq!(msg, "Resource not found");
        }
        _ => panic!("Expected NotFound error"),
    }
}

#[tokio::test]
async fn test_handler_with_validation_errors() {
    #[derive(Deserialize)]
    #[allow(dead_code)]
    struct LoginRequest {
        username: String,
        password: String,
    }

    async fn validate_and_login(username: &str, password: &str) -> Result<Response> {
        if username.is_empty() {
            return Err(RouteError::validation("Username is required"));
        }

        if password.len() < 8 {
            return Err(RouteError::validation(
                "Password must be at least 8 characters",
            ));
        }

        json(json!({
            "token": "jwt_token_here",
            "user": username
        }))
    }

    // 测试空用户名
    let result1 = validate_and_login("", "password123").await;
    assert!(matches!(result1, Err(RouteError::Validation(_))));

    // 测试短密码
    let result2 = validate_and_login("user", "short").await;
    assert!(matches!(result2, Err(RouteError::Validation(_))));

    // 测试有效输入
    let result3 = validate_and_login("user", "password123").await;
    assert!(result3.is_ok());
}

// ============================================================================
// RESTful API 场景测试
// ============================================================================

#[tokio::test]
async fn test_rest_api_list_resources() {
    async fn list_users(event: Event) -> Result<Response> {
        let page = get_query_param(&event, "page")
            .and_then(|p| p.parse::<u32>().ok())
            .unwrap_or(1);

        let limit = get_query_param(&event, "limit")
            .and_then(|l| l.parse::<u32>().ok())
            .unwrap_or(10);

        json(json!({
            "users": [
                {"id": 1, "name": "Alice"},
                {"id": 2, "name": "Bob"}
            ],
            "pagination": {
                "page": page,
                "limit": limit,
                "total": 2
            }
        }))
    }

    let mut query = HashMap::new();
    query.insert("page".to_string(), "1".to_string());
    query.insert("limit".to_string(), "10".to_string());

    let event = Event::new(
        Method::GET,
        "/api/users".to_string(),
        "/api/users?page=1&limit=10".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        query,
    );

    let result = list_users(event).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_rest_api_get_single_resource() {
    async fn get_user(event: Event) -> Result<Response> {
        let user_id = get_param_required(&event, "id")?;

        // 模拟数据库查询
        if user_id == "999" {
            return Err(RouteError::not_found(format!("User {user_id} not found")));
        }

        json(json!({
            "id": user_id,
            "name": "Test User",
            "email": "test@example.com"
        }))
    }

    // 测试存在的用户
    let mut params = HashMap::new();
    params.insert("id".to_string(), "123".to_string());

    let event = Event::new(
        Method::GET,
        "/api/users/123".to_string(),
        "/api/users/123".parse().unwrap(),
        HeaderMap::new(),
        params,
        HashMap::new(),
    );

    let result = get_user(event).await;
    assert!(result.is_ok());

    // 测试不存在的用户
    let mut params_not_found = HashMap::new();
    params_not_found.insert("id".to_string(), "999".to_string());

    let event_not_found = Event::new(
        Method::GET,
        "/api/users/999".to_string(),
        "/api/users/999".parse().unwrap(),
        HeaderMap::new(),
        params_not_found,
        HashMap::new(),
    );

    let result_not_found = get_user(event_not_found).await;
    assert!(result_not_found.is_err());
}

#[tokio::test]
async fn test_rest_api_delete_resource() {
    async fn delete_user(event: Event) -> Result<Response> {
        let user_id = get_param_required(&event, "id")?;

        // 模拟删除操作
        if user_id == "1" {
            return Err(RouteError::forbidden("Cannot delete admin user"));
        }

        Ok(no_content())
    }

    // 测试正常删除
    let mut params = HashMap::new();
    params.insert("id".to_string(), "123".to_string());

    let event = Event::new(
        Method::DELETE,
        "/api/users/123".to_string(),
        "/api/users/123".parse().unwrap(),
        HeaderMap::new(),
        params,
        HashMap::new(),
    );

    let result = delete_user(event).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().status, StatusCode::NO_CONTENT);

    // 测试受保护的资源
    let mut params_protected = HashMap::new();
    params_protected.insert("id".to_string(), "1".to_string());

    let event_protected = Event::new(
        Method::DELETE,
        "/api/users/1".to_string(),
        "/api/users/1".parse().unwrap(),
        HeaderMap::new(),
        params_protected,
        HashMap::new(),
    );

    let result_protected = delete_user(event_protected).await;
    assert!(matches!(result_protected, Err(RouteError::Forbidden(_))));
}

// ============================================================================
// 不同响应类型测试
// ============================================================================

#[tokio::test]
async fn test_handler_returning_html() {
    async fn html_handler(_event: Event) -> Result<Response> {
        Ok(html(
            "<h1>Welcome to Astrea</h1><p>A file-based router for Axum.</p>",
        ))
    }

    let event = Event::new(
        Method::GET,
        "/".to_string(),
        "/".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
    );

    let result = html_handler(event).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(
        response.headers.get("content-type").unwrap(),
        "text/html; charset=utf-8"
    );
}

#[tokio::test]
async fn test_handler_returning_text() {
    async fn text_handler(event: Event) -> Result<Response> {
        let name = get_param(&event, "name").unwrap_or("Guest");
        Ok(text(format!("Hello, {name}!")))
    }

    let mut params = HashMap::new();
    params.insert("name".to_string(), "Rustacean".to_string());

    let event = Event::new(
        Method::GET,
        "/greet/Rustacean".to_string(),
        "/greet/Rustacean".parse().unwrap(),
        HeaderMap::new(),
        params,
        HashMap::new(),
    );

    let result = text_handler(event).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(String::from_utf8_lossy(&response.body), "Hello, Rustacean!");
}

#[tokio::test]
async fn test_handler_with_redirect() {
    async fn redirect_handler(_event: Event) -> Result<Response> {
        redirect("/new-location")
    }

    let event = Event::new(
        Method::GET,
        "/old-location".to_string(),
        "/old-location".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
    );

    let result = redirect_handler(event).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.status, StatusCode::FOUND);
    assert_eq!(response.headers.get("location").unwrap(), "/new-location");
}

// ============================================================================
// 复杂业务逻辑测试
// ============================================================================

#[tokio::test]
async fn test_complex_business_logic() {
    #[derive(Clone)]
    struct AppState {
        max_upload_size: usize,
    }

    async fn upload_handler(event: Event) -> Result<Response> {
        // 检查认证
        let _auth = get_header(&event, "authorization")
            .ok_or_else(|| RouteError::unauthorized("Authentication required"))?;

        // 检查文件类型
        let content_type = get_header(&event, "content-type")
            .ok_or_else(|| RouteError::bad_request("Content-Type header required"))?;

        if !content_type.starts_with("image/") {
            return Err(RouteError::validation("Only image files are allowed"));
        }

        // 检查应用状态
        let state = get_state::<AppState>(&event)?;

        // 模拟文件大小检查
        let file_size = 1024; // 假设的文件大小
        if file_size > state.max_upload_size {
            return Err(RouteError::validation("File size exceeds maximum allowed"));
        }

        Ok(json(json!({
            "uploaded": true,
            "file_id": "abc123",
            "size": file_size
        }))?
        .status(StatusCode::CREATED))
    }

    let state = AppState {
        max_upload_size: 10_000_000, // 10MB
    };

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_static("Bearer token"));
    headers.insert("Content-Type", HeaderValue::from_static("image/png"));

    let mut event = Event::new(
        Method::POST,
        "/api/upload".to_string(),
        "/api/upload".parse().unwrap(),
        headers,
        HashMap::new(),
        HashMap::new(),
    );

    event.state = Some(Arc::new(state));

    let result = upload_handler(event).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.status, StatusCode::CREATED);
}

// ============================================================================
// 中间件风格的处理测试
// ============================================================================

#[tokio::test]
async fn test_middleware_style_processing() {
    async fn logging_middleware(event: &Event) -> Result<()> {
        // 模拟日志记录
        let _method = get_method(event);
        let _path = get_path(event);
        Ok(())
    }

    async fn auth_middleware(event: &Event) -> Result<()> {
        get_header(event, "authorization")
            .ok_or_else(|| RouteError::unauthorized("Missing auth"))?;
        Ok(())
    }

    async fn actual_handler(event: Event) -> Result<Response> {
        // 运行中间件
        logging_middleware(&event).await?;
        auth_middleware(&event).await?;

        // 实际业务逻辑
        json(json!({"success": true}))
    }

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_static("Bearer token"));

    let event = Event::new(
        Method::GET,
        "/api/data".to_string(),
        "/api/data".parse().unwrap(),
        headers,
        HashMap::new(),
        HashMap::new(),
    );

    let result = actual_handler(event).await;
    assert!(result.is_ok());
}

// ============================================================================
// 边界情况和特殊场景
// ============================================================================

#[tokio::test]
async fn test_empty_response_body() {
    async fn handler(_event: Event) -> Result<Response> {
        Ok(Response::new())
    }

    let event = Event::new(
        Method::GET,
        "/".to_string(),
        "/".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
    );

    let result = handler(event).await;
    assert!(result.is_ok());
    assert!(result.unwrap().body.is_empty());
}

#[tokio::test]
async fn test_handler_with_custom_status_code() {
    async fn handler(_event: Event) -> Result<Response> {
        json(json!({"message": "Resource created"})).map(|r| r.status(StatusCode::CREATED))
    }

    let event = Event::new(
        Method::POST,
        "/api/resource".to_string(),
        "/api/resource".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
    );

    let result = handler(event).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().status, StatusCode::CREATED);
}

#[tokio::test]
async fn test_handler_with_multiple_headers() {
    async fn handler(_event: Event) -> Result<Response> {
        Ok(json(json!({"data": "test"}))?
            .header("X-Request-Id", "123")
            .header("X-Rate-Limit", "100")
            .header("X-Rate-Limit-Remaining", "95"))
    }

    let event = Event::new(
        Method::GET,
        "/api/test".to_string(),
        "/api/test".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
    );

    let result = handler(event).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert!(response.headers.contains_key("x-request-id"));
    assert!(response.headers.contains_key("x-rate-limit"));
}
