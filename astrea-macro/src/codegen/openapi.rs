//! OpenAPI helpers and utilities

/// Convert Axum path format to OpenAPI path format
///
/// / 将 Axum 路径格式转换为 OpenAPI 路径格式
///
/// Axum catch-all `{*slug}` → OpenAPI `{slug}`
#[cfg(feature = "openapi")]
pub fn axum_path_to_openapi(axum_path: &str) -> String {
    axum_path.replace("{*", "{")
}
