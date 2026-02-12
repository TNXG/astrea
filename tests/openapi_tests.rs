//! Integration tests for OpenAPI generation
//!
//! Tests document annotation parsing and metadata extraction
//! in real user scenarios with #[route] macro.

#![cfg(feature = "openapi")]

use astrea::openapi::ParamLocation;
use astrea::prelude::*;

// ---------------------------------------------------------------------------
// Test 1: Auto-summary from first doc line
// 测试 1: 从首行文档自动提取摘要
// ---------------------------------------------------------------------------

#[test]
fn test_auto_summary() {
    mod handler {
        use super::*;

        /// Get user by ID
        /// Retrieves detailed user information from the database.
        #[route]
        pub async fn auto_summary_handler(event: Event) -> Result<Response> {
            let id = get_param_required(&event, "id")?;
            json(json!({ "id": id }))
        }
    }

    let meta = handler::__openapi_meta();

    assert_eq!(meta.summary, Some("Get user by ID".to_string()));
    assert_eq!(
        meta.description,
        Some("Retrieves detailed user information from the database.".to_string())
    );
    assert!(meta.tags.is_empty());
    assert!(!meta.deprecated);
}

// ---------------------------------------------------------------------------
// Test 2: Explicit annotations
// 测试 2: 显式标注
// ---------------------------------------------------------------------------

#[test]
fn test_explicit_annotations() {
    mod handler {
        use super::*;

        /// @summary List all users
        /// @description Returns a paginated list of all users in the system.
        /// @tag Users
        /// @tag Admin
        /// @security bearer
        #[route]
        pub async fn explicit_annotations_handler(event: Event) -> Result<Response> {
            let page: u32 = get_query_param(&event, "page")
                .and_then(|s| s.parse().ok())
                .unwrap_or(1);
            json(json!({ "page": page, "users": [] }))
        }
    }

    let meta = handler::__openapi_meta();

    assert_eq!(meta.summary, Some("List all users".to_string()));
    assert_eq!(
        meta.description,
        Some("Returns a paginated list of all users in the system.".to_string())
    );
    assert_eq!(meta.tags, vec!["Users", "Admin"]);
    assert_eq!(meta.security, vec!["bearer"]);
    assert!(!meta.deprecated);
}

// ---------------------------------------------------------------------------
// Test 3: Deprecated endpoint
// 测试 3: 已弃用的端点
// ---------------------------------------------------------------------------

#[test]
fn test_deprecated_endpoint() {
    mod handler {
        use super::*;

        /// Old API endpoint
        /// This endpoint is deprecated. Use /v2/users instead.
        /// @deprecated
        /// @tag Legacy
        #[route]
        pub async fn deprecated_handler(_event: Event) -> Result<Response> {
            Ok::<_, RouteError>(text("This endpoint is deprecated"))
        }
    }

    let meta = handler::__openapi_meta();

    assert_eq!(meta.summary, Some("Old API endpoint".to_string()));
    assert!(meta.deprecated);
    assert_eq!(meta.tags, vec!["Legacy"]);
}

// ---------------------------------------------------------------------------
// Test 4: Multiple response codes
// 测试 4: 多状态码响应
// ---------------------------------------------------------------------------

#[test]
fn test_multiple_responses() {
    mod handler {
        use super::*;

        /// Update user profile
        /// @tag Users
        /// @security bearer
        /// @response 200 Profile updated successfully
        /// @response 400 Invalid input data
        /// @response 401 Authentication required
        /// @response 404 User not found
        #[route]
        pub async fn multiple_responses_handler(event: Event) -> Result<Response> {
            let id = get_param_required(&event, "id")?;
            json(json!({ "id": id, "updated": true }))
        }
    }

    let meta = handler::__openapi_meta();

    assert_eq!(meta.summary, Some("Update user profile".to_string()));
    assert_eq!(meta.responses.len(), 4);
    assert_eq!(
        meta.responses[0],
        (
            "200".to_string(),
            "Profile updated successfully".to_string()
        )
    );
    assert_eq!(
        meta.responses[1],
        ("400".to_string(), "Invalid input data".to_string())
    );
    assert_eq!(
        meta.responses[2],
        ("401".to_string(), "Authentication required".to_string())
    );
    assert_eq!(
        meta.responses[3],
        ("404".to_string(), "User not found".to_string())
    );
}

// ---------------------------------------------------------------------------
// Test 5: Path and query parameters
// 测试 5: 路径和查询参数
// ---------------------------------------------------------------------------

#[test]
fn test_parameters_extraction() {
    mod handler {
        use super::*;

        /// Get user posts
        /// Returns all posts by a specific user with optional filtering.
        /// @tag Users
        /// @tag Posts
        #[route]
        pub async fn params_handler(event: Event) -> Result<Response> {
            let user_id = get_param_required(&event, "user_id")?;
            let limit: u32 = get_query_param(&event, "limit")
                .unwrap_or("10".to_string())
                .parse::<u32>()
                .unwrap_or(10);
            let offset: u32 = get_query_param(&event, "offset")
                .unwrap_or("0".to_string())
                .parse::<u32>()
                .unwrap_or(0);

            json(json!({
                "user_id": user_id,
                "limit": limit,
                "offset": offset,
                "posts": []
            }))
        }
    }

    let meta = handler::__openapi_meta();

    assert_eq!(meta.parameters.len(), 3);

    // Path parameter
    let user_id_param = &meta.parameters[0];
    assert_eq!(user_id_param.name, "user_id");
    assert!(user_id_param.required);
    match user_id_param.location {
        ParamLocation::Path => {}
        _ => panic!("Expected path parameter"),
    }

    // Query parameters (detected from parse::<u32>())
    let limit_param = &meta.parameters[1];
    assert_eq!(limit_param.name, "limit");
    assert!(!limit_param.required); // get_query_param is optional
    match limit_param.location {
        ParamLocation::Query => {}
        _ => panic!("Expected query parameter"),
    }
    assert_eq!(limit_param.schema_type, "integer");
    assert_eq!(limit_param.schema_format, Some("uint32".to_string()));
}

// ---------------------------------------------------------------------------
// Test 6: Request body detection
// 测试 6: 请求体检测
// ---------------------------------------------------------------------------

#[test]
fn test_request_body_detection() {
    mod handler {
        use super::*;
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Serialize, Deserialize)]
        #[allow(dead_code)] // Struct is used by #[route] macro expansion
        pub struct CreateUserRequest {
            pub username: String,
            pub email: String,
        }

        /// Create a new user
        /// @tag Users
        /// @tag Authentication
        /// @response 201 User created successfully
        /// @response 409 Email already exists
        #[route]
        pub async fn create_user_handler(event: Event) -> Result<Response> {
            let body: CreateUserRequest = get_body(&event, &__body_bytes)?;
            json(json!({
                "username": body.username,
                "email": body.email,
                "id": 123
            }))
        }
    }

    let meta = handler::__openapi_meta();

    assert!(meta.request_body.is_some());
    let body = meta.request_body.unwrap();
    assert_eq!(body.schema_type_name, "CreateUserRequest");
    assert_eq!(body.content_type, "application/json");
}

// ---------------------------------------------------------------------------
// Test 7: Response content type detection
// 测试 7: 响应内容类型检测
// ---------------------------------------------------------------------------

#[test]
fn test_response_content_types() {
    mod html_mod {
        use super::*;

        /// Get HTML page
        /// @tag Frontend
        #[route]
        pub async fn html_handler(_event: Event) -> Result<Response> {
            Ok::<_, RouteError>(html("<h1>Hello World</h1>"))
        }
    }

    mod text_mod {
        use super::*;

        /// Get plain text
        /// @tag Frontend
        #[route]
        pub async fn text_handler(_event: Event) -> Result<Response> {
            Ok::<_, RouteError>(text("Hello World"))
        }
    }

    mod bytes_mod {
        use super::*;

        /// Download binary file
        /// @tag Files
        #[route]
        pub async fn bytes_handler(_event: Event) -> Result<Response> {
            Ok::<_, RouteError>(bytes(vec![0x48, 0x65, 0x6c, 0x6c, 0x6f]))
        }
    }

    let html_meta = html_mod::__openapi_meta();
    assert_eq!(html_meta.response_content_type, "text/html");

    let text_meta = text_mod::__openapi_meta();
    assert_eq!(text_meta.response_content_type, "text/plain");

    let bytes_meta = bytes_mod::__openapi_meta();
    assert_eq!(bytes_meta.response_content_type, "application/octet-stream");
}

// ---------------------------------------------------------------------------
// Test 8: JSON response schema fields
// 测试 8: JSON 响应模式字段
// ---------------------------------------------------------------------------

#[test]
fn test_json_response_schema() {
    mod handler {
        use super::*;

        /// Get user statistics
        /// @tag Analytics
        #[route]
        pub async fn json_schema_handler(event: Event) -> Result<Response> {
            let id = get_param_required(&event, "id")?;
            json(json!({
                "user_id": id,
                "total_posts": 42,
                "total_likes": 128,
                "joined_at": "2024-01-01T00:00:00Z"
            }))
        }
    }

    let meta = handler::__openapi_meta();

    assert_eq!(meta.response_content_type, "application/json");
    assert_eq!(meta.response_schema_fields.len(), 4);
    assert!(meta.response_schema_fields.contains(&"user_id".to_string()));
    assert!(
        meta.response_schema_fields
            .contains(&"total_posts".to_string())
    );
    assert!(
        meta.response_schema_fields
            .contains(&"total_likes".to_string())
    );
    assert!(
        meta.response_schema_fields
            .contains(&"joined_at".to_string())
    );
}

// ---------------------------------------------------------------------------
// Test 9: Complex real-world scenario
// 测试 9: 复杂的真实场景
// ---------------------------------------------------------------------------

#[test]
fn test_complex_real_world_scenario() {
    mod handler {
        use super::*;
        use serde::Deserialize;

        /// Update user profile
        ///
        /// Updates the authenticated user's profile information.
        /// Only the fields provided in the request body will be updated.
        /// All other fields remain unchanged.
        ///
        /// @tag Users
        /// @tag Profile
        /// @security bearer
        /// @response 200 Profile updated successfully
        /// @response 400 Invalid input data
        /// @response 401 Authentication required
        /// @response 403 Forbidden
        /// @response 404 User not found
        #[route]
        pub async fn complex_handler(event: Event) -> Result<Response> {
            let user_id = get_param_required(&event, "user_id")?;
            let version: u32 = get_query_param_required(&event, "version")?
                .parse()
                .map_err(|e| RouteError::bad_request(format!("Invalid version: {e}")))?;

            #[derive(Deserialize)]
            struct UpdateRequest {
                display_name: Option<String>,
                bio: Option<String>,
            }

            let req: UpdateRequest = get_body(&event, &__body_bytes)?;

            // Manually list fields that were provided in request
            let mut updated_fields = Vec::new();
            if req.display_name.is_some() {
                updated_fields.push("display_name");
            }
            if req.bio.is_some() {
                updated_fields.push("bio");
            }

            json(json!({
                "user_id": user_id,
                "version": version,
                "updated_fields": updated_fields,
                "success": true
            }))
        }
    }

    let meta = handler::__openapi_meta();

    // Summary and description
    assert_eq!(meta.summary, Some("Update user profile".to_string()));
    assert!(meta.description.is_some());
    let desc = meta.description.unwrap();
    assert!(desc.contains("authenticated user's profile"));
    assert!(desc.contains("Only the fields provided"));

    // Tags
    assert_eq!(meta.tags, vec!["Users", "Profile"]);

    // Security
    assert_eq!(meta.security, vec!["bearer"]);

    // Parameters
    assert_eq!(meta.parameters.len(), 2);
    assert_eq!(meta.parameters[0].name, "user_id");
    assert!(meta.parameters[0].required);
    assert_eq!(meta.parameters[1].name, "version");
    assert!(meta.parameters[1].required);

    // Request body
    assert!(meta.request_body.is_some());

    // Responses
    assert_eq!(meta.responses.len(), 5);
    assert_eq!(meta.responses[0].0, "200");
    assert_eq!(meta.responses[4].0, "404");

    // Response content type and schema
    assert_eq!(meta.response_content_type, "application/json");
    assert_eq!(meta.response_schema_fields.len(), 4);
}

// ---------------------------------------------------------------------------
// Test 10: No documentation at all
// 测试 10: 完全没有文档
// ---------------------------------------------------------------------------

#[test]
fn test_no_documentation() {
    mod handler {
        use super::*;

        #[route]
        pub async fn no_docs_handler(event: Event) -> Result<Response> {
            let id = get_param(&event, "id").unwrap_or("default");
            json(json!({ "id": id }))
        }
    }

    let meta = handler::__openapi_meta();

    assert_eq!(meta.summary, None);
    assert_eq!(meta.description, None);
    assert!(meta.tags.is_empty());
    assert!(meta.security.is_empty());
    assert!(!meta.deprecated);
    assert!(meta.responses.is_empty());

    // But parameters are still extracted from code
    assert_eq!(meta.parameters.len(), 1);
    assert_eq!(meta.parameters[0].name, "id");
}

// ---------------------------------------------------------------------------
// Test 11: Multiple security schemes
// 测试 11: 多种安全方案
// ---------------------------------------------------------------------------

#[test]
fn test_multiple_security_schemes() {
    mod handler {
        use super::*;

        /// Admin-only endpoint
        /// @tag Admin
        /// @security bearer
        /// @security apiKey
        /// @security oauth2
        #[route]
        pub async fn multi_security_handler(_event: Event) -> Result<Response> {
            json(json!({ "status": "authorized" }))
        }
    }

    let meta = handler::__openapi_meta();

    assert_eq!(meta.security, vec!["bearer", "apiKey", "oauth2"]);
}

// ---------------------------------------------------------------------------
// Test 12: No content response
// 测试 12: 无内容响应
// ---------------------------------------------------------------------------

#[test]
fn test_no_content_response() {
    mod handler {
        use super::*;

        /// Delete user
        /// @tag Users
        /// @security bearer
        /// @response 204 User deleted successfully
        /// @response 404 User not found
        #[route]
        pub async fn no_content_handler(_event: Event) -> Result<Response> {
            Ok::<_, RouteError>(no_content())
        }
    }

    let meta = handler::__openapi_meta();

    assert_eq!(meta.response_content_type, "none");
    assert!(meta.response_schema_fields.is_empty());
}
