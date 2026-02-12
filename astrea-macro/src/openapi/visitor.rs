//! AST visitor for analyzing handler function bodies
//!
//! / 用于分析处理函数体的 AST 访问器

use syn::visit::Visit;
use syn::{Expr, Local};

use super::helpers::{
    PARAM_FUNC_MAP, RESPONSE_BUILDER_SET, determine_response_content_type, extract_string_arg,
    find_param_in_expr, is_get_body_call, parse_json_macro_keys, rust_type_to_openapi,
    type_to_name,
};

/// Information about a detected parameter
/// / 检测到的参数信息
#[derive(Debug, Clone)]
pub struct ParamInfo {
    pub name: String,
    pub is_path: bool,
    pub required: bool,
    pub schema_type: String,
    pub schema_format: Option<String>,
}

/// AST visitor that walks the handler function body
///
/// / 遍历处理函数体的 AST 访问器
#[derive(Default)]
pub struct HandlerVisitor {
    /// Detected parameters (path + query)
    /// / 检测到的参数（路径 + 查询）
    pub params: Vec<ParamInfo>,
    /// Detected request body type name
    /// / 检测到的请求体类型名
    pub body_type_name: Option<String>,
    /// Response builder function names found
    /// / 找到的响应构建器函数名
    pub response_builders: Vec<String>,
    /// Top-level keys extracted from json!({...})
    /// / 从 json!({...}) 提取的顶层键
    pub json_macro_keys: Vec<String>,
    /// Deferred type updates from .parse::<T>() detection
    /// / 从 .parse::<T>() 检测中延迟的类型更新
    deferred_type_updates: Vec<(String, String, Option<String>)>,
}

impl<'ast> Visit<'ast> for HandlerVisitor {
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let Expr::Path(expr_path) = &*node.func {
            if let Some(last_segment) = expr_path.path.segments.last() {
                let func_name = last_segment.ident.to_string();

                // Config-driven parameter detection
                // 配置驱动的参数检测
                if let Some(cfg) = PARAM_FUNC_MAP.get(func_name.as_str()) {
                    if let Some(name) = extract_string_arg(&node.args, 1) {
                        self.params.push(ParamInfo {
                            name,
                            is_path: cfg.is_path,
                            required: cfg.required,
                            schema_type: "string".to_string(),
                            schema_format: None,
                        });
                    }
                } else if func_name == "get_body" {
                    // Request body extraction
                    // 请求体提取
                    // Check for turbofish: get_body::<Type>(...)
                    // 检查 turbofish: get_body::<Type>(...)
                    if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                        if let Some(syn::GenericArgument::Type(ty)) = args.args.first() {
                            self.body_type_name = Some(type_to_name(ty));
                        }
                    }
                } else if RESPONSE_BUILDER_SET.contains(func_name.as_str()) {
                    // Response builder detection
                    // 响应构建器检测
                    self.response_builders.push(func_name);
                }
            }
        }

        // Continue recursion
        // 继续递归
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        // Detect .parse::<T>() for parameter type inference
        // 检测 .parse::<T>() 以推断参数类型
        if node.method == "parse" {
            if let Some(turbofish) = &node.turbofish {
                if let Some(syn::GenericArgument::Type(ty)) = turbofish.args.first() {
                    let type_name = type_to_name(ty);
                    let (openapi_type, openapi_format) = rust_type_to_openapi(&type_name);

                    // Walk the receiver to find which param this applies to
                    // 遍历接收器以查找此操作适用于哪个参数
                    if let Some(param_name) = find_param_in_expr(&node.receiver) {
                        self.deferred_type_updates
                            .push((param_name, openapi_type, openapi_format));
                    }
                }
            }
        }

        // Continue recursion
        // 继续递归
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_local(&mut self, node: &'ast Local) {
        // Detect: let body: T = get_body(&event, &bytes)?;
        // 检测: let body: T = get_body(&event, &bytes)?;
        if let Some(init) = &node.init {
            if is_get_body_call(&init.expr) {
                if let syn::Pat::Type(pat_type) = &node.pat {
                    self.body_type_name = Some(type_to_name(&pat_type.ty));
                }
            }
        }

        // Continue recursion
        // 继续递归
        syn::visit::visit_local(self, node);
    }

    fn visit_expr_macro(&mut self, node: &'ast syn::ExprMacro) {
        // Detect json!({...}) and extract top-level keys
        // 检测 json!({...}) 并提取顶层键
        let mac_name = node.mac.path.segments.last().map(|s| s.ident.to_string());

        if mac_name.as_deref() == Some("json") {
            let fields = parse_json_macro_keys(&node.mac.tokens);
            self.json_macro_keys.extend(fields);
        }

        // Continue recursion
        // 继续递归
        syn::visit::visit_expr_macro(self, node);
    }
}

impl HandlerVisitor {
    /// Get the response content type based on detected response builders
    ///
    /// / 根据检测到的响应构建器获取响应内容类型
    pub fn response_content_type(&self) -> &'static str {
        determine_response_content_type(&self.response_builders)
    }

    /// Apply deferred type updates after the full AST traversal
    ///
    /// / 在完整的 AST 遍历后应用延迟的类型更新
    pub fn apply_deferred_type_updates(&mut self) {
        for (param_name, schema_type, schema_format) in
            std::mem::take(&mut self.deferred_type_updates)
        {
            for p in &mut self.params {
                if p.name == param_name {
                    p.schema_type = schema_type;
                    p.schema_format = schema_format;
                    break;
                }
            }
        }
    }
}
