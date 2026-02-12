//! Helper functions for OpenAPI analysis
//!
//! / OpenAPI 分析的辅助函数

use proc_macro2::TokenStream;
use syn::{Expr, Type};

// ---------------------------------------------------------------------------
// Configuration-driven parameter detection (phf)
// 配置驱动的参数检测
// ---------------------------------------------------------------------------

/// Configuration for a parameter extraction function
/// / 参数提取函数的配置
pub struct ParamFuncConfig {
    /// Whether this is a path parameter (vs query)
    /// / 是否为路径参数（否则为查询参数）
    pub is_path: bool,
    /// Whether the parameter is required
    /// / 参数是否必需
    pub required: bool,
}

/// Lookup table: function name → parameter config
/// / 查找表：函数名 → 参数配置
pub static PARAM_FUNC_MAP: phf::Map<&'static str, ParamFuncConfig> = phf::phf_map! {
    "get_param" => ParamFuncConfig { is_path: true, required: false },
    "get_param_required" => ParamFuncConfig { is_path: true, required: true },
    "get_query_param" => ParamFuncConfig { is_path: false, required: false },
    "get_query_param_required" => ParamFuncConfig { is_path: false, required: true },
};

// ---------------------------------------------------------------------------
// Response builder → content type mapping (phf)
// 响应构建器 → 内容类型映射
// ---------------------------------------------------------------------------

/// Lookup table: response builder name → HTTP content type
/// / 查找表：响应构建器名 → HTTP 内容类型
pub static RESPONSE_CONTENT_TYPE_MAP: phf::Map<&'static str, &'static str> = phf::phf_map! {
    "json" => "application/json",
    "text" => "text/plain",
    "html" => "text/html",
    "no_content" => "none",
    "redirect" => "none",
    "bytes" => "application/octet-stream",
};

/// Set of known response builder function names
/// / 已知的响应构建器函数名集合
pub static RESPONSE_BUILDER_SET: phf::Set<&'static str> = phf::phf_set! {
    "json", "text", "html", "no_content", "redirect", "bytes",
};

// ---------------------------------------------------------------------------
// Rust type → OpenAPI type mapping (phf)
// Rust 类型 → OpenAPI 类型映射
// ---------------------------------------------------------------------------

/// OpenAPI schema type and optional format
/// / OpenAPI 模式类型及可选格式
pub struct OpenApiType {
    pub schema_type: &'static str,
    pub format: Option<&'static str>,
}

/// Lookup table: Rust type name → OpenAPI schema type/format
/// / 查找表：Rust 类型名 → OpenAPI 模式类型/格式
static RUST_TYPE_MAP: phf::Map<&'static str, OpenApiType> = phf::phf_map! {
    "u8"    => OpenApiType { schema_type: "integer", format: Some("uint8") },
    "u16"   => OpenApiType { schema_type: "integer", format: Some("uint16") },
    "u32"   => OpenApiType { schema_type: "integer", format: Some("uint32") },
    "u64"   => OpenApiType { schema_type: "integer", format: Some("uint64") },
    "u128"  => OpenApiType { schema_type: "integer", format: Some("uint128") },
    "usize" => OpenApiType { schema_type: "integer", format: Some("uint64") },
    "i8"    => OpenApiType { schema_type: "integer", format: Some("int8") },
    "i16"   => OpenApiType { schema_type: "integer", format: Some("int16") },
    "i32"   => OpenApiType { schema_type: "integer", format: Some("int32") },
    "i64"   => OpenApiType { schema_type: "integer", format: Some("int64") },
    "i128"  => OpenApiType { schema_type: "integer", format: Some("int128") },
    "isize" => OpenApiType { schema_type: "integer", format: Some("int64") },
    "f32"   => OpenApiType { schema_type: "number",  format: Some("float") },
    "f64"   => OpenApiType { schema_type: "number",  format: Some("double") },
    "bool"  => OpenApiType { schema_type: "boolean", format: None },
    "String"=> OpenApiType { schema_type: "string",  format: None },
    "str"   => OpenApiType { schema_type: "string",  format: None },
};

/// Map a Rust type name to OpenAPI schema type and format
///
/// / 将 Rust 类型名映射为 OpenAPI schema 类型和格式
pub fn rust_type_to_openapi(ty: &str) -> (String, Option<String>) {
    match RUST_TYPE_MAP.get(ty) {
        Some(t) => (t.schema_type.to_string(), t.format.map(|f| f.to_string())),
        None => ("string".to_string(), None),
    }
}

// ---------------------------------------------------------------------------
// Recursive expression traversal macro
// 递归表达式遍历宏
// ---------------------------------------------------------------------------

/// Simplifies expression unwrapping in recursive traversal functions.
///
/// / 简化递归遍历函数中的表达式解包。
///
/// For expression variants that are simple wrappers (Try, Paren, Reference, etc.),
/// this macro generates match arms that unwrap and recurse.
///
/// 对于简单包装型表达式变体（Try、Paren、Reference 等），
/// 此宏生成解包并递归的 match 分支。
macro_rules! unwrap_expr {
    ($expr:expr, $func:ident, { $($Variant:ident($pat:ident) => $inner:expr),* $(,)? }) => {
        match $expr {
            $(Expr::$Variant($pat) => $func($inner),)*
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Helper functions
// 辅助函数
// ---------------------------------------------------------------------------

/// Extract a string literal argument from a function call at given position
///
/// / 从函数调用中指定位置提取字符串字面量参数
pub fn extract_string_arg(
    args: &syn::punctuated::Punctuated<Expr, syn::Token![,]>,
    index: usize,
) -> Option<String> {
    let arg = args.iter().nth(index)?;
    // Handle reference expressions like &"string"
    // 处理引用表达式如 &"string"
    let expr = match arg {
        Expr::Reference(r) => &*r.expr,
        other => other,
    };
    if let Expr::Lit(expr_lit) = expr {
        if let syn::Lit::Str(s) = &expr_lit.lit {
            return Some(s.value());
        }
    }
    None
}

/// Convert a syn::Type to a simple type name string
///
/// / 将 syn::Type 转换为简单的类型名字符串
pub fn type_to_name(ty: &Type) -> String {
    match ty {
        syn::Type::Path(tp) => tp
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_else(|| "unknown".to_string()),
        syn::Type::Reference(r) => type_to_name(&r.elem),
        _ => "unknown".to_string(),
    }
}

/// Walk an expression recursively to find a get_param*/get_query_param* call
/// and return the parameter name
///
/// / 递归遍历表达式查找 get_param*/get_query_param* 调用并返回参数名
pub fn find_param_in_expr(expr: &Expr) -> Option<String> {
    match expr {
        // Direct call: get_param_required(&event, "id")
        Expr::Call(call) => {
            if let Expr::Path(path) = &*call.func {
                if let Some(seg) = path.path.segments.last() {
                    let name = seg.ident.to_string();
                    if PARAM_FUNC_MAP.contains_key(name.as_str()) {
                        return extract_string_arg(&call.args, 1);
                    }
                }
            }
            // Recurse into arguments
            for arg in &call.args {
                if let Some(name) = find_param_in_expr(arg) {
                    return Some(name);
                }
            }
            None
        }

        // Block: { expr }
        Expr::Block(b) => {
            if let Some(syn::Stmt::Expr(expr, _)) = b.block.stmts.last() {
                find_param_in_expr(expr)
            } else {
                None
            }
        }

        // Simple unwrap variants (recursive macro)
        // 简单解包变体（递归宏）
        _ => unwrap_expr!(expr, find_param_in_expr, {
            MethodCall(mc) => &mc.receiver,
            Try(t) => &t.expr,
            Closure(c) => &c.body,
            Paren(p) => &p.expr,
            Reference(r) => &r.expr,
        }),
    }
}

/// Check if an expression is or contains a `get_body(...)` call
///
/// / 检查表达式是否是或包含 `get_body(...)` 调用
pub fn is_get_body_call(expr: &Expr) -> bool {
    match expr {
        Expr::Call(call) => {
            if let Expr::Path(path) = &*call.func {
                if let Some(seg) = path.path.segments.last() {
                    if seg.ident == "get_body" {
                        return true;
                    }
                }
            }
            false
        }
        // Handle: get_body(&event, &bytes)?
        Expr::Try(t) => is_get_body_call(&t.expr),
        // Handle: (get_body(...))
        Expr::Paren(p) => is_get_body_call(&p.expr),
        _ => false,
    }
}

/// Parse token stream inside a `json!({...})` macro to extract top-level keys
///
/// / 解析 `json!({...})` 宏内部的 token 流以提取顶层键
///
/// Looks for pattern: `"key": value` at the top level of the first brace group.
///
/// 在第一个大括号组的顶层查找 `"key": value` 模式。
pub fn parse_json_macro_keys(tokens: &TokenStream) -> Vec<String> {
    let mut keys = Vec::new();

    for tree in tokens.clone().into_iter() {
        if let proc_macro2::TokenTree::Group(group) = tree {
            if group.delimiter() == proc_macro2::Delimiter::Brace {
                let inner: Vec<proc_macro2::TokenTree> = group.stream().into_iter().collect();
                let mut i = 0;
                while i < inner.len() {
                    if let proc_macro2::TokenTree::Literal(lit) = &inner[i] {
                        let s = lit.to_string();
                        // String literals are quoted: "key"
                        if s.starts_with('"') && s.ends_with('"') && s.len() > 2 {
                            // Check next token is ':'
                            if i + 1 < inner.len() {
                                if let proc_macro2::TokenTree::Punct(p) = &inner[i + 1] {
                                    if p.as_char() == ':' {
                                        keys.push(s[1..s.len() - 1].to_string());
                                    }
                                }
                            }
                        }
                    }
                    i += 1;
                }
                // Only process the first top-level brace group
                // 只处理第一个顶层大括号组
                break;
            }
        }
    }

    keys
}

/// Determine the response content type from detected response builder names
///
/// / 从检测到的响应构建器名称确定响应内容类型
pub fn determine_response_content_type(builders: &[String]) -> &'static str {
    // Priority: first known builder found
    // 优先级：找到的第一个已知构建器
    for builder in builders {
        if let Some(&ct) = RESPONSE_CONTENT_TYPE_MAP.get(builder.as_str()) {
            return ct;
        }
    }
    // Default: assume JSON if we found json!() macro usage
    // 默认：如果发现了 json!() 宏使用则假定为 JSON
    "application/json"
}
