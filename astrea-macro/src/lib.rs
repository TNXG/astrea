//! Astrea procedural macros
//!
//! / Astrea 过程宏
//!
//! This crate provides the procedural macros used by Astrea:
//!
//! 此 crate 提供 Astrea 使用的宏：
//!
//! - [`route`] - Attribute macro for marking route handlers
//!   [`route`] - 标记路由处理函数的属性宏
//! - [`generate_routes!`] - Macro for generating routes from filesystem
//!   [`generate_routes!`] - 从文件系统生成路由的宏
//!
//! # Example
//!
//! # 示例
//!
//! ```rust,ignore
//! // routes/index.get.rs
//! use astrea::prelude::*;
//!
//! #[route]
//! pub async fn handler(event: Event) -> Result<Response> {
//!     json(json!({"message": "Hello"}))
//! }
//! ```

mod codegen;
#[cfg(feature = "openapi")]
mod openapi;
mod parser;
mod route;
mod scanner;
mod utils;

use proc_macro::TokenStream;

// ============================================================================
// #[route] attribute macro
// ============================================================================
// #[route] 属性宏
// ============================================================================

/// Attribute macro for Astrea route handlers
///
/// / Astrea 路由处理函数的属性宏
///
/// Transforms simple `async fn handler(event: Event)` functions into
/// Axum-compatible handlers with automatic error handling.
///
/// 将简单的 `async fn handler(event: Event)` 函数转换为
/// Axum 兼容的处理函数，具有自动错误处理功能。
///
/// # Requirements
///
/// # 要求
///
/// - The function must be `async`
///   函数必须是 `async`
/// - The function must take `event: Event` as the first parameter
///   函数必须以 `event: Event` 作为第一个参数
/// - The function must return `Result<Response>`
///   函数必须返回 `Result<Response>`
///
/// # Example
///
/// # 示例
///
/// ```rust,ignore
/// use astrea::prelude::*;
///
/// #[route]
/// pub async fn handler(event: Event) -> Result<Response> {
///     let name = get_param(&event, "name").unwrap_or("World");
///     json(json!({ "message": format!("Hello, {}!", name) }))
/// }
/// ```
///
/// # Generated Code
///
/// # 生成的代码
///
/// The macro generates a wrapper function that:
///
/// 宏生成一个包装函数，它：
///
/// - Extracts Axum request data (method, URI, headers, params, query, body)
///   提取 Axum 请求数据（方法、URI、头、参数、查询、体）
/// - Creates an `Event` struct
///   创建 `Event` 结构体
/// - Calls your handler function
///   调用你的处理函数
/// - Automatically converts `Result<Response>` to Axum's response type
///   自动将 `Result<Response>` 转换为 Axum 的响应类型
#[proc_macro_attribute]
pub fn route(_args: TokenStream, input: TokenStream) -> TokenStream {
    route::impl_route(_args, input)
}

// ============================================================================
// generate_routes! macro — with middleware support
// ============================================================================
// generate_routes! 宏 — 支持中间件
// ============================================================================

/// Procedural macro to generate routes from filesystem
///
/// / 从文件系统生成路由的过程宏
///
/// Scans the `routes` directory at compile time and generates route
/// registration code, eliminating the need for manual route definitions
/// or `build.rs` scripts.
///
/// 在编译时扫描 `routes` 目录并生成路由注册代码，
/// 无需手动定义路由或编写 `build.rs` 脚本。
///
/// # Middleware Support
///
/// # 中间件支持
///
/// Place `_middleware.rs` files in route directories to define scoped middleware.
/// Each file must export `pub fn middleware() -> astrea::middleware::Middleware`.
///
/// 在路由目录中放置 `_middleware.rs` 文件来定义作用域中间件。
/// 每个文件必须导出 `pub fn middleware() -> astrea::middleware::Middleware`。
///
/// ## Proximity Principle (就近原则)
///
/// Middleware closest to the route handler applies first (innermost layer).
/// Parent middleware wraps child middleware (outermost layer).
///
/// 最靠近路由处理函数的中间件最先应用（最内层）。
/// 父中间件包裹子中间件（最外层）。
///
/// ## Extend vs Override (叠加 vs 覆盖)
///
/// - **Extend** (default): child middleware stacks on parent middleware
///   **叠加**（默认）：子中间件叠加在父中间件之上
/// - **Override**: child middleware replaces parent middleware entirely
///   **覆盖**：子中间件完全替换父中间件
///
/// # Usage
///
/// # 用法
///
/// ```rust,ignore
/// mod routes {
///     astrea::generate_routes!();
/// }
/// ```
///
/// # File Convention
///
/// # 文件约定
///
/// ```text
/// routes/
/// ├── _middleware.rs          # Global middleware / 全局中间件
/// ├── index.get.rs            # GET /
/// ├── api/
/// │   ├── _middleware.rs      # API middleware (extends root) / API 中间件（叠加）
/// │   ├── users.get.rs        # GET /api/users  ← root + API middleware
/// │   └── public/
/// │       ├── _middleware.rs   # Public middleware (overrides) / 公开中间件（覆盖）
/// │       └── health.get.rs   # GET /api/public/health  ← public middleware only
/// └── posts/
///     └── index.post.rs       # POST /posts  ← root middleware only
/// ```
#[proc_macro]
pub fn generate_routes(input: TokenStream) -> TokenStream {
    codegen::impl_generate_routes(input)
}
