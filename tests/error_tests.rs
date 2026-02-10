//! 全面测试 Error 模块的错误处理功能

use astrea::error::{RouteError, Result};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use anyhow::anyhow;

// ============================================================================
// 错误创建测试
// ============================================================================

#[test]
fn test_bad_request_error() {
    let error = RouteError::bad_request("Invalid input");
    
    assert_eq!(error.status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(error.message(), "Invalid input");
}

#[test]
fn test_not_found_error() {
    let error = RouteError::not_found("Resource not found");
    
    assert_eq!(error.status_code(), StatusCode::NOT_FOUND);
    assert_eq!(error.message(), "Resource not found");
}

#[test]
fn test_unauthorized_error() {
    let error = RouteError::unauthorized("Authentication required");
    
    assert_eq!(error.status_code(), StatusCode::UNAUTHORIZED);
    assert_eq!(error.message(), "Authentication required");
}

#[test]
fn test_forbidden_error() {
    let error = RouteError::forbidden("Access denied");
    
    assert_eq!(error.status_code(), StatusCode::FORBIDDEN);
    assert_eq!(error.message(), "Access denied");
}

#[test]
fn test_validation_error() {
    let error = RouteError::validation("Email format is invalid");
    
    assert_eq!(error.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(error.message(), "Email format is invalid");
}

#[test]
fn test_custom_error() {
    let error = RouteError::custom(StatusCode::IM_A_TEAPOT, "I'm a teapot");
    
    assert_eq!(error.status_code(), StatusCode::IM_A_TEAPOT);
    assert_eq!(error.message(), "I'm a teapot");
}

// ============================================================================
// 错误变体测试
// ============================================================================

#[test]
fn test_method_not_allowed_error() {
    let error = RouteError::MethodNotAllowed("Only GET is allowed".to_string());
    
    assert_eq!(error.status_code(), StatusCode::METHOD_NOT_ALLOWED);
    assert!(error.message().contains("Only GET is allowed"));
}

#[test]
fn test_conflict_error() {
    let error = RouteError::Conflict("Resource already exists".to_string());
    
    assert_eq!(error.status_code(), StatusCode::CONFLICT);
    assert_eq!(error.message(), "Resource already exists");
}

#[test]
fn test_rate_limit_error() {
    let error = RouteError::RateLimit("Too many requests, please try again later".to_string());
    
    assert_eq!(error.status_code(), StatusCode::TOO_MANY_REQUESTS);
    assert!(error.message().contains("Too many requests"));
}

#[test]
fn test_internal_error_from_anyhow() {
    let anyhow_error = anyhow!("Database connection failed");
    let error = RouteError::Internal(anyhow_error);
    
    assert_eq!(error.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    assert!(error.message().contains("Database connection failed"));
}

// ============================================================================
// 错误转换测试
// ============================================================================

#[test]
fn test_error_from_anyhow() {
    let anyhow_error = anyhow!("Something went wrong");
    let route_error: RouteError = anyhow_error.into();
    
    assert_eq!(route_error.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn test_question_mark_operator_with_anyhow() {
    fn may_fail() -> anyhow::Result<String> {
        Err(anyhow!("Failed operation"))
    }

    fn handler() -> Result<String> {
        let result = may_fail()?; // anyhow::Error 自动转换为 RouteError
        Ok(result)
    }

    let result = handler();
    assert!(result.is_err());
    
    match result {
        Err(RouteError::Internal(_)) => {
            // 预期的错误类型
        }
        _ => panic!("Expected Internal error"),
    }
}

// ============================================================================
// 错误消息格式化测试
// ============================================================================

#[test]
fn test_error_display_bad_request() {
    let error = RouteError::BadRequest("Test message".to_string());
    let display = format!("{}", error);
    
    assert!(display.contains("Bad request"));
    assert!(display.contains("Test message"));
}

#[test]
fn test_error_display_not_found() {
    let error = RouteError::NotFound("User not found".to_string());
    let display = format!("{}", error);
    
    assert!(display.contains("Not found"));
    assert!(display.contains("User not found"));
}

#[test]
fn test_error_display_custom() {
    let error = RouteError::Custom {
        status: StatusCode::SERVICE_UNAVAILABLE,
        message: "Service temporarily unavailable".to_string(),
    };
    let display = format!("{}", error);
    
    assert!(display.contains("503"));
    assert!(display.contains("Service temporarily unavailable"));
}

// ============================================================================
// IntoResponse 转换测试
// ============================================================================

#[test]
fn test_error_into_response_bad_request() {
    let error = RouteError::bad_request("Invalid data");
    let response = error.into_response();
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[test]
fn test_error_into_response_not_found() {
    let error = RouteError::not_found("Page not found");
    let response = error.into_response();
    
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[test]
fn test_error_into_response_unauthorized() {
    let error = RouteError::unauthorized("Login required");
    let response = error.into_response();
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[test]
fn test_error_into_response_forbidden() {
    let error = RouteError::forbidden("Insufficient permissions");
    let response = error.into_response();
    
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[test]
fn test_error_into_response_validation() {
    let error = RouteError::validation("Field is required");
    let response = error.into_response();
    
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[test]
fn test_error_into_response_internal() {
    let error = RouteError::Internal(anyhow!("Internal server error"));
    let response = error.into_response();
    
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn test_error_into_response_custom() {
    let error = RouteError::custom(StatusCode::NOT_IMPLEMENTED, "Feature not implemented");
    let response = error.into_response();
    
    assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
}

// ============================================================================
// 状态码映射测试
// ============================================================================

#[test]
fn test_status_code_mapping() {
    let test_cases = vec![
        (RouteError::bad_request(""), StatusCode::BAD_REQUEST),
        (RouteError::not_found(""), StatusCode::NOT_FOUND),
        (RouteError::unauthorized(""), StatusCode::UNAUTHORIZED),
        (RouteError::forbidden(""), StatusCode::FORBIDDEN),
        (RouteError::validation(""), StatusCode::UNPROCESSABLE_ENTITY),
        (
            RouteError::MethodNotAllowed("".to_string()),
            StatusCode::METHOD_NOT_ALLOWED,
        ),
        (RouteError::Conflict("".to_string()), StatusCode::CONFLICT),
        (
            RouteError::RateLimit("".to_string()),
            StatusCode::TOO_MANY_REQUESTS,
        ),
        (
            RouteError::Internal(anyhow!("")),
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
    ];

    for (error, expected_status) in test_cases {
        assert_eq!(error.status_code(), expected_status);
    }
}

// ============================================================================
// 错误消息提取测试
// ============================================================================

#[test]
fn test_message_extraction() {
    let errors = vec![
        (RouteError::bad_request("Bad request message"), "Bad request message"),
        (RouteError::not_found("Not found message"), "Not found message"),
        (RouteError::unauthorized("Unauthorized message"), "Unauthorized message"),
        (RouteError::forbidden("Forbidden message"), "Forbidden message"),
        (RouteError::validation("Validation message"), "Validation message"),
    ];

    for (error, expected_msg) in errors {
        assert_eq!(error.message(), expected_msg);
    }
}

#[test]
fn test_message_extraction_custom() {
    let error = RouteError::Custom {
        status: StatusCode::PAYMENT_REQUIRED,
        message: "Payment required message".to_string(),
    };
    
    assert_eq!(error.message(), "Payment required message");
}

// ============================================================================
// Result 类型别名测试
// ============================================================================

#[test]
fn test_result_ok() {
    fn returns_ok() -> Result<i32> {
        Ok(42)
    }

    let result = returns_ok();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_result_err() {
    fn returns_err() -> Result<i32> {
        Err(RouteError::not_found("Value not found"))
    }

    let result = returns_err();
    assert!(result.is_err());
}

// ============================================================================
// 复杂场景测试
// ============================================================================

#[test]
fn test_error_chain_with_context() {
    fn inner_function() -> anyhow::Result<()> {
        Err(anyhow!("Database query failed"))
    }

    fn outer_function() -> Result<()> {
        inner_function()
            .map_err(|e| RouteError::Internal(anyhow!("Failed to fetch user: {}", e)))?;
        Ok(())
    }

    let result = outer_function();
    assert!(result.is_err());
    
    match result {
        Err(RouteError::Internal(e)) => {
            let msg = e.to_string();
            assert!(msg.contains("Failed to fetch user"));
        }
        _ => panic!("Expected Internal error"),
    }
}

#[test]
fn test_multiple_error_types_in_function() {
    fn complex_handler(valid: bool) -> Result<String> {
        if !valid {
            return Err(RouteError::bad_request("Invalid input"));
        }

        // 模拟可能失败的数据库操作
        let db_result: anyhow::Result<String> = Err(anyhow!("DB error"));
        let data = db_result?;

        Ok(data)
    }

    // 测试验证错误
    let result1 = complex_handler(false);
    assert!(matches!(result1, Err(RouteError::BadRequest(_))));

    // 测试数据库错误
    let result2 = complex_handler(true);
    assert!(matches!(result2, Err(RouteError::Internal(_))));
}

#[test]
fn test_error_with_dynamic_message() {
    let user_id = 12345;
    let error = RouteError::not_found(format!("User with ID {} not found", user_id));
    
    assert!(error.message().contains("12345"));
    assert!(error.message().contains("User with ID"));
}

#[test]
fn test_error_with_multiline_message() {
    let error = RouteError::validation(
        "Multiple validation errors:\n\
         - Email is required\n\
         - Password must be at least 8 characters\n\
         - Username is already taken"
    );
    
    let msg = error.message();
    assert!(msg.contains("Email is required"));
    assert!(msg.contains("Password must be at least 8 characters"));
    assert!(msg.contains("Username is already taken"));
}

// ============================================================================
// 错误构造器模式测试
// ============================================================================

#[test]
fn test_error_builder_pattern_bad_request() {
    let error = RouteError::bad_request("Test");
    assert_eq!(error.status_code(), StatusCode::BAD_REQUEST);
}

#[test]
fn test_error_builder_pattern_all_variants() {
    let errors = vec![
        RouteError::bad_request("msg"),
        RouteError::not_found("msg"),
        RouteError::unauthorized("msg"),
        RouteError::forbidden("msg"),
        RouteError::validation("msg"),
        RouteError::custom(StatusCode::OK, "msg"),
    ];

    for error in errors {
        // 确保所有构造器都正常工作
        assert!(!error.message().is_empty());
    }
}

// ============================================================================
// 特殊字符和编码测试
// ============================================================================

#[test]
fn test_error_with_unicode_message() {
    let error = RouteError::bad_request("错误：用户名包含非法字符 🚫");
    
    assert!(error.message().contains("错误"));
    assert!(error.message().contains("🚫"));
}

#[test]
fn test_error_with_quotes_in_message() {
    let error = RouteError::bad_request(r#"Invalid JSON: expected "name" field"#);
    
    assert!(error.message().contains("expected"));
    assert!(error.message().contains("name"));
}

#[test]
fn test_error_empty_message() {
    let error = RouteError::bad_request("");
    
    assert_eq!(error.message(), "");
    assert_eq!(error.status_code(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// 错误调试表示测试
// ============================================================================

#[test]
fn test_error_debug_format() {
    let error = RouteError::bad_request("Debug test");
    let debug_str = format!("{:?}", error);
    
    assert!(debug_str.contains("BadRequest"));
    assert!(debug_str.contains("Debug test"));
}

#[test]
fn test_error_debug_format_internal() {
    let error = RouteError::Internal(anyhow!("Internal debug test"));
    let debug_str = format!("{:?}", error);
    
    assert!(debug_str.contains("Internal"));
}

// ============================================================================
// 真实使用场景模拟
// ============================================================================

#[test]
fn test_authentication_scenario() {
    fn check_auth(token: Option<&str>) -> Result<String> {
        let token = token.ok_or_else(|| RouteError::unauthorized("Missing token"))?;
        
        if token != "valid_token" {
            return Err(RouteError::unauthorized("Invalid token"));
        }
        
        Ok("user123".to_string())
    }

    // 无 token
    assert!(matches!(
        check_auth(None),
        Err(RouteError::Unauthorized(_))
    ));

    // 无效 token
    assert!(matches!(
        check_auth(Some("bad_token")),
        Err(RouteError::Unauthorized(_))
    ));

    // 有效 token
    assert!(check_auth(Some("valid_token")).is_ok());
}

#[test]
fn test_permission_check_scenario() {
    fn check_permission(user_role: &str, required_role: &str) -> Result<()> {
        if user_role != required_role {
            return Err(RouteError::forbidden(format!(
                "Requires {} role, but user has {} role",
                required_role, user_role
            )));
        }
        Ok(())
    }

    // 权限不足
    let result = check_permission("user", "admin");
    assert!(result.is_err());
    match result {
        Err(RouteError::Forbidden(msg)) => {
            assert!(msg.contains("admin"));
            assert!(msg.contains("user"));
        }
        _ => panic!("Expected Forbidden error"),
    }

    // 权限匹配
    assert!(check_permission("admin", "admin").is_ok());
}

#[test]
fn test_resource_conflict_scenario() {
    fn create_user(username: &str, existing_users: &[&str]) -> Result<()> {
        if existing_users.contains(&username) {
            return Err(RouteError::Conflict(format!(
                "Username '{}' is already taken",
                username
            )));
        }
        Ok(())
    }

    let users = vec!["alice", "bob"];
    
    // 用户名冲突
    let result = create_user("alice", &users);
    assert!(matches!(result, Err(RouteError::Conflict(_))));

    // 用户名可用
    assert!(create_user("charlie", &users).is_ok());
}
