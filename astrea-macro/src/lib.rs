//! Astrea 过程宏
//!
//! 提供 `#[route]` 属性宏和 `generate_routes!` 宏。

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

use std::path::{Path, PathBuf};

// ============================================================================
// #[route] 属性宏
// ============================================================================

/// #[route] 属性宏
///
/// 将简单的 `async fn handler(event: Event)` 函数转换为 Axum 兼容的处理函数。
///
/// # 示例
///
/// ```rust,ignore
/// use astrea::prelude::*;
///
/// #[route]
/// pub async fn handler(event: Event) -> Result<Response> {
///     json(json!({"message": "Hello"}))
/// }
/// ```
#[proc_macro_attribute]
pub fn route(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);

    let vis = &input_fn.vis;
    let fn_name = &input_fn.sig.ident;
    let inputs = &input_fn.sig.inputs;
    let block = &input_fn.block;

    if input_fn.sig.asyncness.is_none() {
        return syn::Error::new_spanned(fn_name, "#[route] 函数必须是 async fn")
            .to_compile_error()
            .into();
    }

    // 解析参数名
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
// generate_routes! 宏
// ============================================================================

/// 路由扫描结果
struct ScannedRoute {
    /// HTTP 方法 (GET, POST, ...)
    method: String,
    /// Axum 路由路径 (如 /users/:id)
    axum_path: String,
    /// 源文件绝对路径
    file_path: String,
    /// 生成的模块名
    module_name: String,
}

/// `generate_routes!()` 过程宏
///
/// 在编译时扫描 `routes` 目录并生成路由注册代码，
/// 无需用户编写 `build.rs`。
///
/// # 用法
///
/// ```rust,ignore
/// // 默认扫描 src/routes/ 目录
/// mod routes {
///     astrea::generate_routes!();
/// }
///
/// // 或指定自定义目录名
/// mod api {
///     astrea::generate_routes!("api");
/// }
/// ```
///
/// 生成的代码包含:
/// - 每个路由文件对应的 `mod` 声明
/// - `create_router()` 函数，返回配置好路由的 `axum::Router`
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

        // 对齐输出: 方法名左对齐 6 字符宽
        let log_line = format!("  {:<6} {}", method_upper, axum_path);
        route_logs.push(quote! {
            ::astrea::tracing::info!("{}", #log_line);
        });
    }

    let expanded = quote! {
        #(#mod_decls)*

        /// 创建包含所有文件路由的 Router
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
// 路由扫描辅助函数
// ============================================================================

/// 递归扫描目录中的路由文件
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
    // 排序以保证确定性
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let name = entry.file_name().to_string_lossy().to_string();

        // 跳过隐藏文件和 _ 开头的文件 (如 _middleware.rs)
        if name.starts_with('.') || name.starts_with('_') {
            continue;
        }

        let path = entry.path();

        if path.is_dir() {
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
        } else if path.is_file() && name.ends_with(".rs") {
            if let Some(route) = parse_route_file(&path, &name, path_components) {
                routes.push(route);
            }
        }
    }
}

/// 解析单个路由文件，提取 HTTP 方法和路由路径
fn parse_route_file(
    file_path: &Path,
    file_name: &str,
    path_components: &[String],
) -> Option<ScannedRoute> {
    let name_without_ext = file_name.strip_suffix(".rs")?;
    let parts: Vec<&str> = name_without_ext.split('.').collect();

    let is_index = parts[0] == "index";

    // 确定 HTTP 方法
    let method = if is_index && parts.len() == 1 {
        // index.rs → 默认 GET
        "GET".to_string()
    } else if parts.len() >= 2 {
        // name.get.rs / index.post.rs → 取最后一段
        parts[parts.len() - 1].to_uppercase()
    } else {
        return None;
    };

    // 构建路由路径
    let mut route_path = path_components.to_vec();
    if !is_index {
        route_path.push(parts[0].to_string());
    }

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

/// 将单个路径片段转为合法标识符字符
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

/// 清理完整标识符：去除连续下划线和首尾下划线
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
