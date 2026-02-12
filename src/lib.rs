//! # Astrea - File-based Router for Axum
//!
//! / # Astrea - 基于 Axum 的文件路由器
//!
//! Astrea is a file-system based router for Axum, inspired by [Nitro] and [H3].
//!
//! Astrea 是一个受 [Nitro] 和 [H3] 启发的 Axum 文件路由器。
//!
//! ## Features
//!
//! ## 特性
//!
//! - **Simple unified handler signature** - All handlers follow the same pattern:
//!   `async fn handler(event: Event) -> Result<Response>`
//!   **简单统一的处理函数签名** - 所有处理函数遵循相同模式：
//!   `async fn handler(event: Event) -> Result<Response>`
//!
//! - **Declarative parameter extraction** - Access request data through helper functions
//!   instead of complex Axum extractor signatures
//!   **声明式参数提取** - 通过辅助函数访问请求数据，无需复杂的 Axum 提取器签名
//!
//! - **File-based routing** - Routes are automatically generated from your filesystem
//!   structure at compile time
//!   **基于文件的路由** - 在编译时根据文件系统结构自动生成路由
//!
//! - **Type-safe** - Full Rust type safety with compile-time route generation
//!   **类型安全** - 完整的 Rust 类型安全，编译时生成路由
//!
//! - **Axum ecosystem compatible** - Works seamlessly with Axum middleware
//!   **兼容 Axum 生态** - 与 Axum 中间件无缝协作
//!
//! ## Quick Start
//!
//! ## 快速开始
//!
//! ```rust,no_run
//! use astrea::prelude::*;
//! use serde_json::json;
//!
//! // routes/index.get.rs
//! #[route]
//! async fn handler(event: Event) -> Result<Response> {
//!     let name = get_param(&event, "name").unwrap_or("World");
//!     json(json!({ "message": format!("Hello, {}!", name) }))
//! }
//! ```
//!
//! ## Route File Convention
//!
//! ## 路由文件规则
//!
//! Routes are generated from the `routes/` directory:
//!
//! 路由从 `routes/` 目录生成：
//!
//! - `routes/index.get.rs` → `GET /`
//! - `routes/users.get.rs` → `GET /users`
//! - `routes/users/[id].get.rs` → `GET /users/:id`
//! - `routes/posts/[...slug].get.rs` → `GET /posts/*slug`
//!
//! ## Module Organization
//!
//! ## 模块组织
//!
//! - [`event`] - Request event type that encapsulates all request data
//!   [`event`] - 封装所有请求数据的请求事件类型
//! - [`extract`] - Helper functions for extracting request data
//!   [`extract`] - 提取请求数据的辅助函数
//! - [`response`] - Response builders and helpers
//!   [`response`] - 响应构建器和辅助函数
//! - [`error`] - Error types and result handling
//!   [`error`] - 错误类型和结果处理
//!
//! [Nitro]: https://nitro.unjs.io/
//! [H3]: https://h3.unjs.io/

pub mod error;
pub mod event;
pub mod extract;
pub mod middleware;
pub mod response;
pub mod router;

// ============================================================================
// Re-export dependencies - users don't need to depend on these crates directly
// ============================================================================
// Re-export 依赖库 — 用户无需在 Cargo.toml 中直接依赖这些 crate
// ============================================================================

/// Re-export of `axum` - users don't need to explicitly depend on it
/// / Re-export axum — 用户无需显式依赖
pub use axum;
/// Re-export of `bytes`
/// / Re-export bytes
pub use bytes;
/// Re-export of `serde`
/// / Re-export serde
pub use serde;
/// Re-export of `serde_json`
/// / Re-export serde_json
pub use serde_json;
/// Re-export of `tokio`
/// / Re-export tokio
pub use tokio;
/// Re-export of `tower`
/// / Re-export tower
pub use tower;
/// Re-export of `tower_http`
/// / Re-export tower_http
pub use tower_http;
/// Re-export of `tracing`
/// / Re-export tracing
pub use tracing;

// Convenience re-export: axum::serve
// 便捷 re-export: axum::serve
pub use axum::serve;

// Re-export commonly used types
// 重新导出常用类型
pub use error::RouteError;
pub use event::Event;
pub use response::Response;

// Re-export procedural macros
// 重新导出过程宏
pub use astrea_macro::generate_routes;

/// Prelude module with common imports
///
/// / 包含常用导入的预导出模块
///
/// # Example
///
/// # 示例
///
/// ```rust,ignore
/// use astrea::prelude::*;
/// ```
pub mod prelude {
    pub use crate::error::{Result, RouteError};
    pub use crate::event::Event;
    pub use crate::extract::*;
    pub use crate::middleware::{Middleware, MiddlewareMode};
    pub use crate::response::{Response, bytes, html, json, no_content, redirect, text};

    // Re-export common Axum types
    // Re-export 常用 Axum 类型
    pub use axum::http::StatusCode;

    // Re-export common serde macros
    // Re-export serde 常用 derive 宏和 json! 宏
    pub use serde::{Deserialize, Serialize};
    pub use serde_json::json;

    // Re-export #[route] macro
    // Re-export #[route] 宏
    pub use astrea_macro::route;

    // Re-export generate_routes! macro
    // Re-export generate_routes! 宏
    pub use astrea_macro::generate_routes;
}
