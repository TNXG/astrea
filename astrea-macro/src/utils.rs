//! Identifier utility functions
//!
//! / 标识符工具函数

/// Convert a single path segment to valid identifier characters
///
/// / 将单个路径片段转为合法标识符字符
///
/// Replaces non-alphanumeric characters with underscores.
///
/// 将非字母数字字符替换为下划线。
pub fn sanitize_ident_part(s: &str) -> String {
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
pub fn sanitize_ident(name: &str) -> String {
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
