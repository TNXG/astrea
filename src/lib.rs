//! # Astrea - 基于 Axum 的文件路由器
//!
//! Astrea 是一个受 Nitro/H3 启发的 Axum 文件路由器。
//!
//! ## 特性
//!
//! - 简单统一的函数签名 (`async fn handler(event: Event) -> Result<Response>`)
//! - 通过辅助函数声明式提取参数
//! - 基于文件系统结构自动生成路由
//! - Rust 类型安全
//! - 兼容 Axum 中间件生态
//!
//! ## 示例
//!
//! ```rust,no_run
//! use astrea::prelude::*;
//! use serde_json::json;
//!
//! #[route]
//! async fn handler(event: Event) -> Result<Response> {
//!     let name = get_param(&event, "name").unwrap_or("World");
//!     json(json!({ "message": format!("Hello, {}!", name) }))
//! }
//! ```

pub mod error;
pub mod event;
pub mod extract;
pub mod response;
pub mod router;

// ============================================================================
// Re-export 依赖库 — 用户无需在 Cargo.toml 中直接依赖这些 crate
// ============================================================================

/// Re-export axum，用户无需显式依赖
pub use axum;
/// Re-export bytes
pub use bytes;
/// Re-export serde_json
pub use serde_json;
/// Re-export serde
pub use serde;
/// Re-export tokio
pub use tokio;
/// Re-export tracing
pub use tracing;

// 便捷 re-export: axum::serve
pub use axum::serve;

// 重新导出常用类型
pub use error::RouteError;
pub use event::Event;
pub use response::Response;

// 重新导出过程宏
pub use astrea_macro::generate_routes;

pub mod prelude {
    //! 便捷导入的预导出模块
    pub use crate::error::{RouteError, Result};
    pub use crate::event::Event;
    pub use crate::extract::*;
    pub use crate::response::{json, text, html, redirect, no_content, Response};

    // Re-export 常用 axum 类型
    pub use axum::http::StatusCode;

    // Re-export serde 常用 derive 宏和 json! 宏
    pub use serde::{Serialize, Deserialize};
    pub use serde_json::json;

    // Re-export #[route] 宏
    pub use astrea_macro::route;

    // Re-export generate_routes! 宏
    pub use astrea_macro::generate_routes;
}
