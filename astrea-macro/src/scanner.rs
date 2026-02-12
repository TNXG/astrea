//! Directory scanning and middleware scope tree building
//!
//! / 目录扫描和中间件作用域树构建

use std::path::Path;

use crate::parser::parse_route_file;
use crate::utils::{sanitize_ident, sanitize_ident_part};

// ────────────────────────────────────────────────────
// Data structures / 数据结构
// ────────────────────────────────────────────────────

/// Route scanning result
///
/// / 路由扫描结果
pub struct ScannedRoute {
    /// HTTP method (GET, POST, ...)
    /// / HTTP 方法 (GET, POST, ...)
    pub method: String,
    /// Axum route path (e.g., /users/{id})
    /// / Axum 路由路径 (如 /users/{id})
    pub axum_path: String,
    /// Source file absolute path
    /// / 源文件绝对路径
    pub file_path: String,
    /// Generated module name
    /// / 生成的模块名
    pub module_name: String,
}

/// Information about a `_middleware.rs` file
///
/// / `_middleware.rs` 文件信息
pub struct MiddlewareFileInfo {
    /// Path relative to CARGO_MANIFEST_DIR, for `include!()`
    /// / 相对于 CARGO_MANIFEST_DIR 的路径，用于 `include!()`
    pub rel_path: String,
    /// Valid Rust module identifier
    /// / 合法的 Rust 模块标识符
    pub module_name: String,
    /// Display path for logging (e.g., "/" or "/api")
    /// / 用于日志的显示路径（如 "/" 或 "/api"）
    pub scope_path: String,
}

/// A middleware scope in the directory tree
///
/// / 目录树中的中间件作用域
///
/// A scope is created for every directory that contains `_middleware.rs`.
/// Directories without `_middleware.rs` have their routes absorbed into
/// the nearest parent scope.
///
/// 每个包含 `_middleware.rs` 的目录都会创建一个作用域。
/// 没有 `_middleware.rs` 的目录会将其路由吸收到最近的父作用域中。
pub struct MiddlewareScope {
    /// Middleware config if `_middleware.rs` exists in this directory
    /// / 此目录的中间件配置（如果存在 `_middleware.rs`）
    pub middleware: Option<MiddlewareFileInfo>,
    /// Routes directly in this scope (not in child scopes)
    /// / 直接属于此作用域的路由（不包含子作用域的路由）
    pub routes: Vec<ScannedRoute>,
    /// Child scopes (sub-directories that have their own `_middleware.rs`)
    /// / 子作用域（拥有自己 `_middleware.rs` 的子目录）
    pub children: Vec<MiddlewareScope>,
}

// ────────────────────────────────────────────────────
// Directory scanning / 目录扫描
// ────────────────────────────────────────────────────

/// Recursively scan a directory and build a middleware scope tree
///
/// / 递归扫描目录并构建中间件作用域树
///
/// Directories containing `_middleware.rs` become their own scope.
/// Directories without middleware merge into the nearest parent scope.
///
/// 包含 `_middleware.rs` 的目录成为独立作用域。
/// 没有中间件的目录合并到最近的父作用域。
pub fn scan_and_build_scope(
    dir: &Path,
    path_parts: &[String],
    manifest_dir: &str,
) -> MiddlewareScope {
    // Check for _middleware.rs
    // 检查 _middleware.rs
    let mw_file = dir.join("_middleware.rs");
    let middleware = if mw_file.exists() && mw_file.is_file() {
        let abs = mw_file.to_string_lossy().to_string();
        let rel = Path::new(&abs)
            .strip_prefix(manifest_dir)
            .map(|p| format!("/{}", p.to_string_lossy()))
            .unwrap_or_else(|_| abs.clone());

        let module_name = if path_parts.is_empty() {
            "mw".to_string()
        } else {
            let parts: Vec<String> = path_parts.iter().map(|s| sanitize_ident_part(s)).collect();
            let raw = format!("mw_{}", parts.join("_"));
            sanitize_ident(&raw)
        };

        let scope_path = if path_parts.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", path_parts.join("/"))
        };

        Some(MiddlewareFileInfo {
            rel_path: rel,
            module_name,
            scope_path,
        })
    } else {
        None
    };

    let mut scope = MiddlewareScope {
        middleware,
        routes: Vec::new(),
        children: Vec::new(),
    };

    let mut entries: Vec<_> = match std::fs::read_dir(dir) {
        Ok(entries) => entries.filter_map(|e| e.ok()).collect(),
        Err(_) => return scope,
    };
    // Sort for deterministic output
    // 排序以保证确定性输出
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();

        // Already handled _middleware.rs above; skip it and other special files
        // _middleware.rs 已在上方处理，跳过它和其他特殊文件
        if name == "_middleware.rs" || name.starts_with('.') || name.starts_with('_') {
            continue;
        }

        if path.is_dir() {
            let component = dir_name_to_path_part(&name);
            let mut child_parts = path_parts.to_vec();
            child_parts.push(component);

            let child_scope = scan_and_build_scope(&path, &child_parts, manifest_dir);

            if child_scope.middleware.is_some() {
                // Child directory has its own middleware → separate scope
                // 子目录有自己的中间件 → 独立作用域
                scope.children.push(child_scope);
            } else if !child_scope.children.is_empty() {
                // No middleware here, but grandchildren have middleware.
                // Absorb direct routes; promote grandchildren as our children.
                // 这里没有中间件，但孙级目录有中间件。
                // 吸收直接路由；将孙级提升为我们的子级。
                scope.routes.extend(child_scope.routes);
                scope.children.extend(child_scope.children);
            } else {
                // No middleware anywhere in subtree → absorb all routes
                // 子树中没有任何中间件 → 吸收所有路由
                scope.routes.extend(child_scope.routes);
            }
        } else if path.is_file()
            && name.ends_with(".rs")
            && let Some(route) = parse_route_file(&path, &name, path_parts)
        {
            scope.routes.push(route);
        }
    }

    // Sort routes within scope: longer (more specific) paths first
    // 作用域内路由排序：更长（更具体）的路径优先
    scope.routes.sort_by(|a, b| {
        let len_cmp = b.axum_path.len().cmp(&a.axum_path.len());
        if len_cmp != std::cmp::Ordering::Equal {
            return len_cmp;
        }
        a.axum_path.cmp(&b.axum_path)
    });

    scope
}

/// Convert a directory name to a path component for route building
///
/// / 将目录名转换为路由构建用的路径组件
fn dir_name_to_path_part(name: &str) -> String {
    if name.starts_with("[...") && name.ends_with(']') {
        let param = &name[4..name.len() - 1];
        format!("[...{}]", param)
    } else if name.starts_with('[') && name.ends_with(']') {
        let param = &name[1..name.len() - 1];
        format!("[{}]", param)
    } else {
        name.to_string()
    }
}

// ────────────────────────────────────────────────────
// Log collection helpers / 日志收集辅助函数
// ────────────────────────────────────────────────────

/// Detailed route information for TUI display
///
/// / 用于 TUI 显示的详细路由信息
pub struct RouteDetailLog {
    /// HTTP method
    /// / HTTP 方法
    pub method: String,
    /// Route path
    /// / 路由路径
    pub path: String,
    /// Middleware scope chain applied to this route (e.g., ["/", "/api"])
    /// / 作用于此路由的中间件作用域链（如 ["/", "/api"]）
    pub middleware_chain: Vec<String>,
}

/// Detailed middleware scope information for TUI display
///
/// / 用于 TUI 显示的详细中间件作用域信息
pub struct MiddlewareDetailLog {
    /// Scope display path (e.g., "/" or "/api")
    /// / 作用域显示路径
    pub scope_path: String,
    /// Parent scope path, if any
    /// / 父作用域路径（如果有）
    pub parent_path: Option<String>,
    /// Module name — used at runtime to call `middleware()` and read mode
    /// / 模块名 — 在运行时调用 `middleware()` 并读取 mode
    pub module_name: String,
}

/// Collect detailed route information including middleware chain
///
/// / 收集包含中间件链的详细路由信息
pub fn collect_route_detail_logs(
    scope: &MiddlewareScope,
    parent_chain: &[String],
) -> Vec<RouteDetailLog> {
    // Build the middleware chain for this scope
    // 构建此作用域的中间件链
    let mut chain = parent_chain.to_vec();
    if let Some(mw) = &scope.middleware {
        chain.push(mw.scope_path.clone());
    }

    let mut logs: Vec<RouteDetailLog> = scope
        .routes
        .iter()
        .map(|r| RouteDetailLog {
            method: r.method.clone(),
            path: r.axum_path.clone(),
            middleware_chain: chain.clone(),
        })
        .collect();

    for child in &scope.children {
        // Child scopes will determine their own chain based on mode at runtime,
        // but at compile time we pass the full parent chain — the codegen will
        // decide at runtime whether to show the inherited chain or just the child.
        // 子作用域在运行时根据 mode 决定自己的链，但编译时传递完整的父链 —
        // codegen 将在运行时决定是显示继承链还是仅显示子级。
        logs.extend(collect_route_detail_logs(child, &chain));
    }

    // Sort: shorter paths first (more natural reading order), then alphabetically
    // 排序：较短路径优先（更自然的阅读顺序），然后按字母排序
    logs.sort_by(|a, b| {
        let len_cmp = a.path.len().cmp(&b.path.len());
        if len_cmp != std::cmp::Ordering::Equal {
            return len_cmp;
        }
        a.path.cmp(&b.path).then(a.method.cmp(&b.method))
    });
    logs
}

/// Collect detailed middleware scope information including parent relationships
///
/// / 收集包含父级关系的详细中间件作用域信息
pub fn collect_middleware_detail_logs(
    scope: &MiddlewareScope,
    parent_path: Option<&str>,
) -> Vec<MiddlewareDetailLog> {
    let mut logs = Vec::new();
    let current_path = scope.middleware.as_ref().map(|mw| mw.scope_path.as_str());

    if let Some(mw) = &scope.middleware {
        logs.push(MiddlewareDetailLog {
            scope_path: mw.scope_path.clone(),
            parent_path: parent_path.map(|s| s.to_string()),
            module_name: mw.module_name.clone(),
        });
    }

    for child in &scope.children {
        logs.extend(collect_middleware_detail_logs(
            child,
            current_path.or(parent_path),
        ));
    }
    logs
}
