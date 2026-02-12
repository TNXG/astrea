//! Doc comment parsing for OpenAPI annotations
//!
//! / 用于 OpenAPI 标注的文档注释解析

use syn::{Attribute, Expr, Lit, Meta};

/// Parsed doc annotation data
/// / 解析后的文档标注数据
pub struct DocAnnotations {
    /// Operation summary (from `@summary` or auto-detected first doc line)
    /// / 操作摘要（来自 `@summary` 或自动检测的首行文档）
    pub summary: Option<String>,
    /// Operation description (from `@description` or remaining plain doc lines)
    /// / 操作描述（来自 `@description` 或剩余的普通文档行）
    pub description: Option<String>,
    /// Operation tags (from `@tag`)
    /// / 操作标签（来自 `@tag`）
    pub tags: Vec<String>,
    /// Security requirements (from `@security`)
    /// / 安全要求（来自 `@security`）
    pub security: Vec<String>,
    /// Whether the operation is deprecated (from `@deprecated`)
    /// / 操作是否已弃用（来自 `@deprecated`）
    pub deprecated: bool,
    /// Additional response descriptions: `(status_code, description)`
    /// / 额外的响应描述：`(状态码, 描述)`
    ///
    /// From `@response 404 Not found` annotations.
    /// / 来自 `@response 404 Not found` 标注。
    pub responses: Vec<(String, String)>,
}

/// Parse `///` doc comments for OpenAPI annotations
///
/// / 解析 `///` 文档注释中的 OpenAPI 标注
///
/// Supported annotations:
/// - `@tag TagName` → operation tag
/// - `@summary Short text` → operation summary
/// - `@description Longer text` → operation description (multi-line)
/// - `@security bearer` → security requirement
/// - `@deprecated` → marks the operation as deprecated
/// - `@response 404 Not found` → additional response description
///
/// Plain doc lines (without `@` prefix):
/// - First plain line → auto summary (if no `@summary` provided)
/// - Remaining plain lines → description (if no `@description` provided)
///
/// 普通文档行（无 `@` 前缀）：
/// - 第一行 → 自动摘要（如未提供 `@summary`）
/// - 后续行 → 描述（如未提供 `@description`）
pub fn parse_doc_annotations(attrs: &[Attribute]) -> DocAnnotations {
    let mut annot = DocAnnotations {
        summary: None,
        description: None,
        tags: Vec::new(),
        security: Vec::new(),
        deprecated: false,
        responses: Vec::new(),
    };

    let mut plain_lines: Vec<String> = Vec::new();
    let mut has_explicit_description = false;
    let mut has_explicit_summary = false;

    for attr in attrs {
        if !attr.path().is_ident("doc") {
            continue;
        }

        let text = match &attr.meta {
            Meta::NameValue(nv) => {
                if let Expr::Lit(expr_lit) = &nv.value {
                    if let Lit::Str(s) = &expr_lit.lit {
                        s.value()
                    } else {
                        continue;
                    }
                } else {
                    continue;
                }
            }
            _ => continue,
        };

        let trimmed = text.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Annotation dispatch
        // 标注分发
        if let Some(rest) = trimmed.strip_prefix("@tag ") {
            annot.tags.push(rest.trim().to_string());
        } else if let Some(rest) = trimmed.strip_prefix("@summary ") {
            has_explicit_summary = true;
            annot.summary = Some(rest.trim().to_string());
        } else if let Some(rest) = trimmed.strip_prefix("@description ") {
            has_explicit_description = true;
            if let Some(existing) = &mut annot.description {
                existing.push('\n');
                existing.push_str(rest.trim());
            } else {
                annot.description = Some(rest.trim().to_string());
            }
        } else if let Some(rest) = trimmed.strip_prefix("@security ") {
            annot.security.push(rest.trim().to_string());
        } else if trimmed.starts_with("@deprecated") {
            annot.deprecated = true;
        } else if let Some(rest) = trimmed.strip_prefix("@response ") {
            // Format: @response <code> <description>
            // 格式：@response <状态码> <描述>
            let rest = rest.trim();
            if let Some(space_idx) = rest.find(' ') {
                let code = rest[..space_idx].to_string();
                let desc = rest[space_idx + 1..].trim().to_string();
                annot.responses.push((code, desc));
            } else {
                // Code only, no description
                // 仅状态码，无描述
                annot.responses.push((rest.to_string(), String::new()));
            }
        } else if !trimmed.starts_with('@') {
            plain_lines.push(trimmed.to_string());
        }
    }

    // Auto-summary: first plain line becomes summary if no explicit @summary
    // 自动摘要：如无显式 @summary，第一行普通文档作为摘要
    if !has_explicit_summary && !plain_lines.is_empty() {
        annot.summary = Some(plain_lines.remove(0));
    }

    // Use remaining plain lines as description fallback
    // 使用剩余普通行作为描述的后备
    if !has_explicit_description && !plain_lines.is_empty() {
        annot.description = Some(plain_lines.join("\n"));
    }

    annot
}
