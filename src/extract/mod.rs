//! Helper functions for extracting data from Event
//!
//! / 从 Event 提取数据的辅助函数
//!
//! This module provides convenient functions to access request data
//! without complex Axum extractor signatures.
//!
//! 此模块提供便捷的函数来访问请求数据，无需复杂的 Axum 提取器签名。
//!
//! # Overview
//!
//! # 概述
//!
//! Instead of using Axum's extractors like:
//!
//! 不使用 Axum 的提取器，如：
//!
//! ```rust,ignore
//! async fn handler(
//!     Path(id): Path<String>,
//!     Query(params): Query<HashMap<String, String>>,
//!     Json(body): Json<MyData>,
//! ) -> Result<Response>
//! ```
//!
//! Astrea provides a simple, unified interface:
//!
//! Astrea 提供了简单统一的接口：
//!
//! ```rust,ignore
//! use astrea::prelude::*;
//!
//! #[route]
//! async fn handler(event: Event, bytes: Bytes) -> Result<Response> {
//!     let id = get_param_required(&event, "id")?;
//!     let search = get_query_param(&event, "q");
//!     let body: MyData = get_body(&event)?;
//!     json(json!({ "id", "search": search, "body": body }))
//! }
//! ```
//!
//! # Available Extractors
//!
//! # 可用的提取器
//!
//! - **Path parameters**: [`get_param`], [`get_param_required`]
//!   **路径参数**：[`get_param`], [`get_param_required`]
//! - **Query parameters**: [`get_query`], [`get_query_param`], [`get_query_param_required`]
//!   **查询参数**：[`get_query`], [`get_query_param`], [`get_query_param_required`]
//! - **Request body**: [`get_body`], [`get_body_bytes`], [`get_body_text`]
//!   **请求体**：[`get_body`], [`get_body_bytes`], [`get_body_text`]
//! - **Headers**: [`get_header`], [`get_headers`]
//!   **请求头**：[`get_header`], [`get_headers`]
//! - **Metadata**: [`get_method`], [`get_path`], [`get_uri`]
//!   **元数据**：[`get_method`], [`get_path`], [`get_uri`]
//! - **State**: [`get_state`]
//!   **状态**：[`get_state`]

// Re-export all submodules
// Re-export 所有子模块

pub mod body;
pub mod headers;
pub mod metadata;
pub mod params;
pub mod query;
pub mod state;

#[cfg(test)]
mod tests;

// Re-export public items from submodules for convenient access
// Re-export 子模块的公共项以便便捷访问

pub use body::{get_body, get_body_bytes, get_body_text};
pub use headers::{get_header, get_headers};
pub use metadata::{get_method, get_path, get_uri};
pub use params::{get_param, get_param_required};
pub use query::{get_query, get_query_param, get_query_param_required};
pub use state::get_state;
