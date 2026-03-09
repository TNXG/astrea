//! Route file parsing
//!
//! / 路由文件解析

use std::path::Path;

use crate::scanner::ScannedRoute;
use crate::utils::{sanitize_ident, sanitize_ident_part};

/// Detect the route type by reading file content for `#[route(ws)]` / `#[route(sse)]`
///
/// / 通过读取文件内容检测 `#[route(ws)]` / `#[route(sse)]` 以确定路由类型
///
/// Returns `"WS"`, `"SSE"`, or `"ANY"` for method-less files.
///
/// 对于无方法后缀的文件，返回 `"WS"`、`"SSE"` 或 `"ANY"`。
fn detect_route_type(file_path: &Path) -> String {
    let content = match std::fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return "ANY".to_string(),
    };

    // Check for #[route(ws)] or #[route( ws )] patterns
    // 检查 #[route(ws)] 或 #[route( ws )] 模式
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("#[route(") {
            if let Some(arg) = rest.strip_suffix(")]") {
                let arg = arg.trim();
                match arg {
                    "ws" => return "WS".to_string(),
                    "sse" => return "SSE".to_string(),
                    _ => {}
                }
            }
        }
    }

    "ANY".to_string()
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
/// - `name.rs` → method detected from file content (WS/SSE/ANY), path=`name`
/// - `index.rs` → method detected from file content, path=empty
///
/// Returns `None` for files that don't match the expected pattern.
///
/// 如果文件不匹配预期模式，返回 `None`。
pub fn parse_route_file(
    file_path: &Path,
    file_name: &str,
    path_components: &[String],
) -> Option<ScannedRoute> {
    let name_without_ext = file_name.strip_suffix(".rs")?;

    // Handle dynamic routes: split by the last dot before method
    // 处理动态路由：在方法前的最后一个点分割
    let (route_name, method_str) = if let Some(pos) = name_without_ext.rfind('.') {
        let name = &name_without_ext[..pos];
        let method = &name_without_ext[pos + 1..];
        (name, Some(method))
    } else {
        (name_without_ext, None)
    };

    let is_index = route_name == "index";

    // Determine method / route type
    // 确定 HTTP 方法 / 路由类型
    let method = match method_str {
        Some(m) => {
            // name.get.rs / index.post.rs → take method part
            // name.get.rs / index.post.rs → 取方法部分
            m.to_uppercase()
        }
        None => {
            // name.rs / index.rs → read file content to detect WS/SSE/ANY
            // name.rs / index.rs → 读取文件内容检测 WS/SSE/ANY
            detect_route_type(file_path)
        }
    };

    // Build route path
    // 构建路由路径
    let mut route_path = path_components.to_vec();
    if !is_index {
        route_path.push(route_name.to_string());
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// Global counter to produce unique temp-file names even across parallel tests.
    static TMP_ID: AtomicU32 = AtomicU32::new(0);

    /// Write `content` to a uniquely-named temp file and return its path.
    /// Each call produces a fresh, collision-free filename via PID + counter.
    fn temp_rs(content: &str) -> std::path::PathBuf {
        let id = TMP_ID.fetch_add(1, Ordering::Relaxed);
        let name = format!("astrea_macro_test_{}_{}.rs", std::process::id(), id);
        let path = std::env::temp_dir().join(name);
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        path
    }

    // ── detect_route_type ─────────────────────────────────────────────────────

    #[test]
    fn detect_ws_annotation() {
        let p = temp_rs("#[route(ws)]\nasync fn handler() {}");
        assert_eq!(detect_route_type(&p), "WS");
        let _ = std::fs::remove_file(p);
    }

    #[test]
    fn detect_sse_annotation() {
        let p = temp_rs("#[route(sse)]\nasync fn handler() {}");
        assert_eq!(detect_route_type(&p), "SSE");
        let _ = std::fs::remove_file(p);
    }

    #[test]
    fn detect_any_for_plain_route() {
        let p = temp_rs("#[route]\nasync fn handler() {}");
        assert_eq!(detect_route_type(&p), "ANY");
        let _ = std::fs::remove_file(p);
    }

    #[test]
    fn detect_any_for_empty_parens() {
        let p = temp_rs("#[route()]\nasync fn handler() {}");
        // arg after stripping is empty — doesn't match ws/sse → ANY
        assert_eq!(detect_route_type(&p), "ANY");
        let _ = std::fs::remove_file(p);
    }

    #[test]
    fn detect_any_for_missing_file() {
        let p = std::path::Path::new("/tmp/definitely_does_not_exist_astrea_xyz.rs");
        assert_eq!(detect_route_type(p), "ANY");
    }

    #[test]
    fn detect_any_for_no_annotation() {
        let p = temp_rs("async fn handler() {}");
        assert_eq!(detect_route_type(&p), "ANY");
        let _ = std::fs::remove_file(p);
    }

    // ── parse_route_file ──────────────────────────────────────────────────────

    #[test]
    fn parse_index_get() {
        let p = temp_rs("#[route]\nasync fn h() {}");
        let route = parse_route_file(&p, "index.get.rs", &[]).unwrap();
        assert_eq!(route.method, "GET");
        assert_eq!(route.axum_path, "/");
        let _ = std::fs::remove_file(p);
    }

    #[test]
    fn parse_users_post() {
        let p = temp_rs("#[route]\nasync fn h() {}");
        let route = parse_route_file(&p, "users.post.rs", &[]).unwrap();
        assert_eq!(route.method, "POST");
        assert_eq!(route.axum_path, "/users");
        let _ = std::fs::remove_file(p);
    }

    #[test]
    fn parse_nested_route_path() {
        let p = temp_rs("#[route]\nasync fn h() {}");
        let path_parts = vec!["api".to_string()];
        let route = parse_route_file(&p, "items.get.rs", &path_parts).unwrap();
        assert_eq!(route.method, "GET");
        assert_eq!(route.axum_path, "/api/items");
        let _ = std::fs::remove_file(p);
    }

    #[test]
    fn parse_method_less_with_ws_annotation() {
        let p = temp_rs("#[route(ws)]\nasync fn h() {}");
        let route = parse_route_file(&p, "chat.rs", &[]).unwrap();
        assert_eq!(route.method, "WS");
        assert_eq!(route.axum_path, "/chat");
        let _ = std::fs::remove_file(p);
    }

    #[test]
    fn parse_method_less_with_sse_annotation() {
        let p = temp_rs("#[route(sse)]\nasync fn h() {}");
        let route = parse_route_file(&p, "events.rs", &[]).unwrap();
        assert_eq!(route.method, "SSE");
        assert_eq!(route.axum_path, "/events");
        let _ = std::fs::remove_file(p);
    }

    #[test]
    fn parse_method_less_defaults_to_any() {
        let p = temp_rs("#[route]\nasync fn h() {}");
        let route = parse_route_file(&p, "users.rs", &[]).unwrap();
        assert_eq!(route.method, "ANY");
        let _ = std::fs::remove_file(p);
    }

    #[test]
    fn parse_dynamic_path_param() {
        let p = temp_rs("#[route]\nasync fn h() {}");
        let path_parts = vec!["users".to_string(), "[id]".to_string()];
        let route = parse_route_file(&p, "index.get.rs", &path_parts).unwrap();
        assert_eq!(route.axum_path, "/users/{id}");
        let _ = std::fs::remove_file(p);
    }

    #[test]
    fn parse_catch_all_path_param() {
        let p = temp_rs("#[route]\nasync fn h() {}");
        let path_parts = vec!["files".to_string(), "[...path]".to_string()];
        let route = parse_route_file(&p, "index.get.rs", &path_parts).unwrap();
        assert_eq!(route.axum_path, "/files/{*path}");
        let _ = std::fs::remove_file(p);
    }

    #[test]
    fn parse_non_rs_file_returns_none() {
        let p = std::path::Path::new("/tmp/not_a_rust_file.txt");
        assert!(parse_route_file(p, "not_a_rust_file.txt", &[]).is_none());
    }
}
