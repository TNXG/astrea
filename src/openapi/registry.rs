//! Global OpenAPI metadata registry
//!
//! / 全局 OpenAPI 元数据注册表

use std::sync::{Mutex, OnceLock};

use super::types::{HandlerMeta, ParamLocation, ParamMeta, RouteEntry};

static REGISTRY: OnceLock<Mutex<Vec<RouteEntry>>> = OnceLock::new();

/// Register a route's OpenAPI metadata
///
/// / 注册路由的 OpenAPI 元数据
///
/// Called from generated `create_router()` code. Automatically supplements
/// path parameters found in the URL pattern that weren't detected in the handler body.
///
/// 从生成的 `create_router()` 代码中调用。自动补充 URL 模式中发现但
/// 处理函数体中未检测到的路径参数。
pub fn register(method: &str, path: &str, operation_id: &str, mut handler_meta: HandlerMeta) {
    // Supplement path params from URL pattern
    // 从 URL 模式补充路径参数
    let path_param_names = extract_path_param_names(path);
    let existing: std::collections::HashSet<String> = handler_meta
        .parameters
        .iter()
        .filter(|p| p.location == ParamLocation::Path)
        .map(|p| p.name.clone())
        .collect();

    for name in path_param_names {
        if !existing.contains(&name) {
            handler_meta.parameters.push(ParamMeta {
                name,
                location: ParamLocation::Path,
                required: true,
                schema_type: "string".to_string(),
                schema_format: None,
            });
        }
    }

    // OpenAPI spec requires all path params to be required
    // OpenAPI 规范要求所有路径参数都是必需的
    for p in &mut handler_meta.parameters {
        if p.location == ParamLocation::Path {
            p.required = true;
        }
    }

    let registry = REGISTRY.get_or_init(|| Mutex::new(Vec::new()));
    registry.lock().unwrap().push(RouteEntry {
        method: method.to_uppercase(),
        path: path.to_string(),
        operation_id: operation_id.to_string(),
        handler_meta,
    });
}

/// Get all registered route entries
///
/// / 获取所有已注册的路由条目
pub fn get_entries() -> Vec<RouteEntry> {
    REGISTRY
        .get_or_init(|| Mutex::new(Vec::new()))
        .lock()
        .unwrap()
        .clone()
}

/// Extract parameter names from an OpenAPI-format path
///
/// / 从 OpenAPI 格式路径中提取参数名
///
/// e.g., `/users/{id}/posts/{post_id}` → `["id", "post_id"]`
fn extract_path_param_names(path: &str) -> Vec<String> {
    path.split('/')
        .filter(|s| s.starts_with('{') && s.ends_with('}'))
        .map(|s| {
            let inner = &s[1..s.len() - 1];
            // Handle catch-all: {*slug} → slug
            inner.strip_prefix('*').unwrap_or(inner).to_string()
        })
        .collect()
}
