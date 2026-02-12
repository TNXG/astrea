//! Route file parsing
//!
//! / 路由文件解析

use std::path::Path;

use crate::scanner::ScannedRoute;
use crate::utils::{sanitize_ident, sanitize_ident_part};

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

    // Determine HTTP method
    // 确定 HTTP 方法
    let method = if is_index && method_str.is_none() {
        // index.rs → default GET
        "GET".to_string()
    } else if let Some(m) = method_str {
        // name.get.rs / index.post.rs → take method part
        // name.get.rs / index.post.rs → 取方法部分
        m.to_uppercase()
    } else {
        return None;
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
