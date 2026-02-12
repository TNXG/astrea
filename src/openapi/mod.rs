//! OpenAPI specification auto-generation for Astrea
//!
//! / Astrea 的 OpenAPI 规范自动生成
//!
//! This module provides automatic OpenAPI 3.0 documentation generation
//! from route handlers. Enable with the `openapi` feature flag.
//!
//! 此模块提供从路由处理函数自动生成 OpenAPI 3.0 文档的功能。
//! 通过 `openapi` feature flag 启用。
//!
//! # Overview
//!
//! # 概述
//!
//! The `#[route]` macro analyzes handler function bodies at compile time
//! to extract parameter info, request body types, and response types.
//! `generate_routes!` combines this with filesystem-derived method/path
//! and registers metadata to a global registry. At runtime, you can
//! serve the spec via `/openapi.json` and optional Swagger UI.
//!
//! `#[route]` 宏在编译时分析处理函数体，提取参数信息、请求体类型和响应类型。
//! `generate_routes!` 将此与文件系统推导的方法/路径组合，并注册到全局注册表。
//! 运行时可以通过 `/openapi.json` 和可选的 Swagger UI 提供规范。
//!
//! # Example
//!
//! # 示例
//!
//! ```rust,ignore
//! // In your main.rs:
//! let app = routes::create_router()
//!     .merge(astrea::openapi::router("My API", "1.0.0"));
//!
//! // Now GET /openapi.json returns the spec
//! // And GET /swagger shows Swagger UI
//! ```

pub mod registry;
mod spec;
mod swagger;
pub mod types;

pub use registry::register;
pub use types::*;

/// Generate an OpenAPI 3.0 specification as a JSON value
///
/// / 生成 OpenAPI 3.0 规范的 JSON 值
///
/// Call this after `create_router()` has been invoked, which registers
/// all route metadata.
///
/// 在调用 `create_router()` 之后调用此函数，`create_router()` 会注册所有路由元数据。
///
/// # Example
///
/// # 示例
///
/// ```rust,ignore
/// let openapi_spec = astrea::openapi::spec("My API", "1.0.0");
/// println!("{}", serde_json::to_string_pretty(&openapi_spec).unwrap());
/// ```
pub fn spec(title: &str, version: &str) -> serde_json::Value {
    spec::generate_spec(title, version)
}

/// Create an Axum Router that serves the OpenAPI spec and Swagger UI
///
/// / 创建一个提供 OpenAPI 规范和 Swagger UI 的 Axum Router
///
/// Provides two endpoints:
///
/// 提供两个端点：
///
/// - `GET /openapi.json` — returns the OpenAPI 3.0 spec as JSON
///   返回 OpenAPI 3.0 规范 JSON
/// - `GET /swagger` — returns the Swagger UI HTML page
///   返回 Swagger UI HTML 页面
///
/// # Example
///
/// # 示例
///
/// ```rust,ignore
/// let app = routes::create_router()
///     .merge(astrea::openapi::router("My API", "1.0.0"));
/// ```
pub fn router(title: &str, version: &str) -> axum::Router {
    let spec_json = spec(title, version);
    let swagger_html = swagger::swagger_ui_html("/openapi.json");

    axum::Router::new()
        .route(
            "/openapi.json",
            axum::routing::get({
                let spec = spec_json.clone();
                move || async move { axum::Json(spec) }
            }),
        )
        .route(
            "/swagger",
            axum::routing::get(move || async move { axum::response::Html(swagger_html) }),
        )
}
