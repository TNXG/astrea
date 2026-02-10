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

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

use std::path::{Path, PathBuf};

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
    let input_fn = parse_macro_input!(input as ItemFn);

    let vis = &input_fn.vis;
    let fn_name = &input_fn.sig.ident;
    let inputs = &input_fn.sig.inputs;
    let block = &input_fn.block;

    if input_fn.sig.asyncness.is_none() {
        return syn::Error::new_spanned(
            fn_name,
            "#[route] 函数必须是 async fn / #[route] function must be async fn",
        )
        .to_compile_error()
        .into();
    }

    // 解析参数名 / Parse parameter name
    let mut event_param_name = None;
    for input in inputs {
        if let syn::FnArg::Typed(arg) = input
            && let syn::Pat::Ident(ident) = &*arg.pat
            && ident.ident == "event"
        {
            event_param_name = Some(ident.ident.clone());
        }
    }

    let event_name = event_param_name
        .unwrap_or_else(|| syn::Ident::new("event", proc_macro2::Span::call_site()));

    // 生成包装函数 — 所有外部类型通过 ::astrea:: 引用，用户无需直接依赖 axum / bytes
    // Generate wrapper function - all external types referenced via ::astrea::
    let expanded = quote! {
        #vis async fn #fn_name(
            __method: ::astrea::axum::http::Method,
            __uri: ::astrea::axum::http::Uri,
            __headers: ::astrea::axum::http::HeaderMap,
            __path_params: ::astrea::axum::extract::Path<std::collections::HashMap<String, String>>,
            __query_params: ::astrea::axum::extract::Query<std::collections::HashMap<String, String>>,
            __body_bytes: ::astrea::bytes::Bytes,
        ) -> impl ::astrea::axum::response::IntoResponse {
            use ::astrea::{Event, Response};
            use ::astrea::axum::response::IntoResponse;

            let __path = __uri.path().to_string();

            let #event_name = Event::new(
                __method,
                __path,
                __uri,
                __headers,
                __path_params.0,
                __query_params.0,
            );

            let result = #block;

            match result {
                Ok(response) => response.into_axum_response(),
                Err(error) => error.into_response(),
            }
        }
    };

    TokenStream::from(expanded)
}

// ============================================================================
// generate_routes! macro
// ============================================================================
// generate_routes! 宏
// ============================================================================

/// Route scanning result
///
/// / 路由扫描结果
struct ScannedRoute {
    /// HTTP method (GET, POST, ...)
    /// / HTTP 方法 (GET, POST, ...)
    method: String,
    /// Axum route path (e.g., /users/:id)
    /// / Axum 路由路径 (如 /users/:id)
    axum_path: String,
    /// Source file absolute path
    /// / 源文件绝对路径
    file_path: String,
    /// Generated module name
    /// / 生成的模块名
    module_name: String,
}

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
/// # Usage
///
/// # 用法
///
/// ```rust,ignore
/// // Default: scan src/routes/ directory
/// // 默认扫描 src/routes/ 目录
/// mod routes {
///     astrea::generate_routes!();
/// }
///
/// // Custom directory name
/// // 自定义目录名
/// mod api {
///     astrea::generate_routes!("api");
/// }
/// ```
///
/// # Generated Code
///
/// # 生成的代码
///
/// The macro generates:
///
/// 宏生成：
///
/// - A `mod` declaration for each route file
///   每个路由文件的 `mod` 声明
/// - A `create_router()` function that returns a configured `axum::Router`
///   返回配置好的 `axum::Router` 的 `create_router()` 函数
///
/// # Route File Convention
///
/// # 路由文件规则
///
/// Routes are generated based on filename patterns:
///
/// 路由根据文件名模式生成：
///
/// - `index.get.rs` → `GET /`
/// - `users.get.rs` → `GET /users`
/// - `users/[id].get.rs` → `GET /users/:id`
/// - `posts/[...slug].get.rs` → `GET /posts/*slug`
///
/// # Dynamic Parameters
///
/// # 动态参数
///
/// - `[id]` → Single path parameter `:id` / 单一路径参数 `:id`
/// - `[...slug]` → Catch-all parameter `*slug` / 全捕获参数 `*slug`
///
/// # Example
///
/// # 示例
///
/// Given this file structure:
///
/// 给定以下文件结构：
///
/// ```text
/// routes/
/// ├── index.get.rs          # GET /
/// ├── users/
/// │   ├── index.get.rs      # GET /users
/// │   └── [id].get.rs       # GET /users/:id
/// └── posts/
///     └── index.post.rs     # POST /posts
/// ```
///
/// The macro generates code equivalent to:
///
/// 宏生成等效于以下的代码：
///
/// ```rust,ignore
/// mod routes {
///     // ... module declarations ...
///
///     pub fn create_router() -> axum::Router {
///         axum::Router::new()
///             .route("/", axum::routing::get(index::handler))
///             .route("/users", axum::routing::get(users_index::handler))
///             .route("/users/:id", axum::routing::get(users_id::handler))
///             .route("/posts", axum::routing::post(posts_index::handler))
///     }
/// }
/// ```
#[proc_macro]
pub fn generate_routes(input: TokenStream) -> TokenStream {
    let routes_dir_name = if input.is_empty() {
        "src/routes".to_string()
    } else {
        let lit = parse_macro_input!(input as syn::LitStr);
        lit.value()
    };

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR environment variable not set");

    let routes_dir = PathBuf::from(&manifest_dir).join(&routes_dir_name);

    if !routes_dir.exists() {
        let msg = format!(
            "astrea: routes directory not found: {}",
            routes_dir.display()
        );
        return quote! { compile_error!(#msg); }.into();
    }

    let routes_dir_str = routes_dir.to_string_lossy().to_string();
    let mut routes = Vec::new();
    scan_directory(&routes_dir, &mut Vec::new(), &routes_dir_str, &mut routes);

    // Sort by path length descending for more specific routes to match first
    // 按路径长度降序排列，让更具体的路由优先匹配
    routes.sort_by(|a, b| {
        let len_cmp = b.axum_path.len().cmp(&a.axum_path.len());
        if len_cmp != std::cmp::Ordering::Equal {
            return len_cmp;
        }
        a.axum_path.cmp(&b.axum_path)
    });

    let route_count = routes.len();
    let mut mod_decls = Vec::new();
    let mut route_registrations = Vec::new();
    let mut route_logs = Vec::new();

    for route in &routes {
        let mod_name = syn::Ident::new(&route.module_name, proc_macro2::Span::call_site());
        // Calculate path relative to CARGO_MANIFEST_DIR, use include! + env! pattern
        // This avoids issues with inline module owning directory
        // 计算相对于 CARGO_MANIFEST_DIR 的路径，用 include! + env! 模式
        // 这样不受内联模块 owning directory 的影响
        let rel_path = Path::new(&route.file_path)
            .strip_prefix(&manifest_dir)
            .map(|p| format!("/{}", p.to_string_lossy()))
            .unwrap_or_else(|_| route.file_path.clone());
        let axum_path = &route.axum_path;
        let method_upper = &route.method;
        let method_fn =
            syn::Ident::new(&route.method.to_lowercase(), proc_macro2::Span::call_site());

        mod_decls.push(quote! {
            mod #mod_name {
                include!(concat!(env!("CARGO_MANIFEST_DIR"), #rel_path));
            }
        });

        route_registrations.push(quote! {
            .route(#axum_path, ::astrea::axum::routing::#method_fn(#mod_name::handler))
        });

        // Align output: method name left-aligned 6 chars wide
        // 对齐输出: 方法名左对齐 6 字符宽
        let log_line = format!("  {:<6} {}", method_upper, axum_path);
        route_logs.push(quote! {
            ::astrea::tracing::info!("{}", #log_line);
        });
    }

    let expanded = quote! {
        #(#mod_decls)*

        /// Create a Router with all file-based routes
        /// / 创建包含所有文件路由的 Router
        pub fn create_router() -> ::astrea::axum::Router {
            ::astrea::tracing::info!("Initializing file router...");
            ::astrea::tracing::info!("Registered {} route(s):", #route_count);
            #(#route_logs)*

            ::astrea::axum::Router::new()
                #(#route_registrations)*
        }
    };

    expanded.into()
}

// ============================================================================
// Route scanning helper functions
// ============================================================================
// 路由扫描辅助函数
// ============================================================================

/// Recursively scan directory for route files
///
/// / 递归扫描目录中的路由文件
///
/// # Skips
///
/// # 跳过
///
/// - Hidden files (starting with `.`)
///   隐藏文件（以 `.` 开头）
/// - Files starting with `_` (e.g., `_middleware.rs`)
///   以 `_` 开头的文件（如 `_middleware.rs`）
///
/// # Directory Handling
///
/// # 目录处理
///
/// Directories are processed in order of specificity:
///
/// 目录按特异性顺序处理：
///
/// 1. `[...param]` - Catch-all parameters (highest priority)
///    `[...param]` - 全捕获参数（最高优先级）
/// 2. `[param]` - Dynamic parameters
///    `[param]` - 动态参数
/// 3. Regular names
///    常规名称
fn scan_directory(
    dir: &Path,
    path_components: &mut Vec<String>,
    _routes_dir: &str,
    routes: &mut Vec<ScannedRoute>,
) {
    let mut entries: Vec<_> = match std::fs::read_dir(dir) {
        Ok(entries) => entries.filter_map(|e| e.ok()).collect(),
        Err(_) => return,
    };
    // Sort for determinism
    // 排序以保证确定性
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files and _ prefixed files (e.g., _middleware.rs)
        // 跳过隐藏文件和 _ 开头的文件 (如 _middleware.rs)
        if name.starts_with('.') || name.starts_with('_') {
            continue;
        }

        let path = entry.path();

        if path.is_dir() {
            // Handle directory: [...param] catch-all > [param] dynamic > regular
            // 处理目录：[...param] catch-all > [param] 动态 > 普通
            if name.starts_with("[...") && name.ends_with(']') {
                let param = &name[4..name.len() - 1];
                path_components.push(format!("[...{}]", param));
            } else if name.starts_with('[') && name.ends_with(']') {
                let param = &name[1..name.len() - 1];
                path_components.push(format!("[{}]", param));
            } else {
                path_components.push(name.clone());
            }

            scan_directory(&path, path_components, _routes_dir, routes);
            path_components.pop();
        } else if path.is_file() && name.ends_with(".rs")
            && let Some(route) = parse_route_file(&path, &name, path_components)
        {
            routes.push(route);
        }
    }
}

/// Parse a single route file to extract HTTP method and route path
///
/// / 解析单个路由文件，提取 HTTP 方法和路由路径
///
/// # Filename Patterns
///
/// # 文件名模式
///
/// - `index.get.rs` → method=GET, path=empty
/// - `name.get.rs` → method=GET, path=`name`
/// - `index.post.rs` → method=POST, path=empty
///
/// Returns `None` for files that don't match the expected pattern.
///
/// 如果文件不匹配预期模式，返回 `None`。
fn parse_route_file(
    file_path: &Path,
    file_name: &str,
    path_components: &[String],
) -> Option<ScannedRoute> {
    let name_without_ext = file_name.strip_suffix(".rs")?;
    let parts: Vec<&str> = name_without_ext.split('.').collect();

    let is_index = parts[0] == "index";

    // Determine HTTP method
    // 确定 HTTP 方法
    let method = if is_index && parts.len() == 1 {
        // index.rs → default GET
        "GET".to_string()
    } else if parts.len() >= 2 {
        // name.get.rs / index.post.rs → take last segment
        // name.get.rs / index.post.rs → 取最后一段
        parts[parts.len() - 1].to_uppercase()
    } else {
        return None;
    };

    // Build route path
    // 构建路由路径
    let mut route_path = path_components.to_vec();
    if !is_index {
        route_path.push(parts[0].to_string());
    }

    // Convert to Axum 0.8 route format
    // 转换为 Axum 0.8 路由格式
    let axum_path = if route_path.is_empty() {
        "/".to_string()
    } else {
        let segments: Vec<String> = route_path
            .iter()
            .map(|seg| {
                if seg.starts_with("[...") && seg.ends_with(']') {
                    // catch-all: [...path] → {*path}
                    let param = &seg[4..seg.len() - 1];
                    format!("{{*{}}}", param)
                } else if seg.starts_with('[') && seg.ends_with(']') {
                    // dynamic param: [id] → {id}
                    // 动态参数: [id] → {id}
                    let param = &seg[1..seg.len() - 1];
                    format!("{{{}}}", param)
                } else {
                    seg.clone()
                }
            })
            .collect();
        format!("/{}", segments.join("/"))
    };

    // Generate valid Rust module identifier
    // 生成合法的 Rust 模块标识符
    let mod_name = {
        let name_parts: Vec<String> = path_components
            .iter()
            .map(|s| sanitize_ident_part(s))
            .chain(std::iter::once(sanitize_ident_part(name_without_ext)))
            .collect();
        let raw = name_parts.join("_");
        let sanitized = sanitize_ident(&raw);
        if sanitized.is_empty() {
            "root_route".to_string()
        } else {
            sanitized
        }
    };

    Some(ScannedRoute {
        method,
        axum_path,
        file_path: file_path.to_string_lossy().to_string(),
        module_name: mod_name,
    })
}

/// Convert a single path segment to valid identifier characters
///
/// / 将单个路径片段转为合法标识符字符
///
/// Replaces non-alphanumeric characters with underscores.
///
/// 将非字母数字字符替换为下划线。
fn sanitize_ident_part(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Sanitize a complete identifier: remove consecutive underscores and leading/trailing underscores
///
/// / 清理完整标识符：去除连续下划线和首尾下划线
fn sanitize_ident(name: &str) -> String {
    let mut result = String::new();
    let mut prev_underscore = false;

    for c in name.chars() {
        if c == '_' {
            if !prev_underscore && !result.is_empty() {
                result.push('_');
                prev_underscore = true;
            }
        } else if c.is_alphanumeric() {
            result.push(c);
            prev_underscore = false;
        }
    }

    result.trim_end_matches('_').to_string()
}
