//! AST analysis for OpenAPI metadata extraction
//!
//! / 用于 OpenAPI 元数据提取的 AST 分析
//!
//! Walks the handler function body using `syn::visit::Visit` to detect:
//! - `get_param` / `get_param_required` calls → path parameters
//! - `get_query_param` / `get_query_param_required` calls → query parameters
//! - `get_body::<T>()` calls → request body type
//! - `.parse::<T>()` calls → parameter type inference
//! - `json()` / `text()` / `html()` calls → response content type
//! - `json!({...})` macros → response field names
//! - `///` doc comment annotations → tags, summary, description, security, deprecated, response

mod doc;
mod helpers;
mod visitor;

pub use doc::parse_doc_annotations;
pub use visitor::{HandlerVisitor, ParamInfo};

use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemFn;
use syn::visit::Visit;

// ---------------------------------------------------------------------------
// Builder pattern for HandlerMeta token generation
// HandlerMeta token 生成的 Builder 模式
// ---------------------------------------------------------------------------

/// Builder that assembles a `HandlerMeta` construction `TokenStream`
///
/// / 组装 `HandlerMeta` 构造 `TokenStream` 的 Builder
struct MetaTokenBuilder {
    summary: Option<String>,
    description: Option<String>,
    tags: Vec<String>,
    security: Vec<String>,
    params: Vec<ParamInfo>,
    request_body: Option<String>,
    response_content_type: String,
    response_fields: Vec<String>,
    deprecated: bool,
    responses: Vec<(String, String)>,
}

impl MetaTokenBuilder {
    fn new() -> Self {
        Self {
            summary: None,
            description: None,
            tags: Vec::new(),
            security: Vec::new(),
            params: Vec::new(),
            request_body: None,
            response_content_type: String::new(),
            response_fields: Vec::new(),
            deprecated: false,
            responses: Vec::new(),
        }
    }

    fn summary(mut self, v: Option<String>) -> Self {
        self.summary = v;
        self
    }

    fn description(mut self, v: Option<String>) -> Self {
        self.description = v;
        self
    }

    fn tags(mut self, v: Vec<String>) -> Self {
        self.tags = v;
        self
    }

    fn security(mut self, v: Vec<String>) -> Self {
        self.security = v;
        self
    }

    fn params(mut self, v: Vec<ParamInfo>) -> Self {
        self.params = v;
        self
    }

    fn request_body(mut self, v: Option<String>) -> Self {
        self.request_body = v;
        self
    }

    fn response_content_type(mut self, v: &str) -> Self {
        self.response_content_type = v.to_string();
        self
    }

    fn response_fields(mut self, v: Vec<String>) -> Self {
        self.response_fields = v;
        self
    }

    fn deprecated(mut self, v: bool) -> Self {
        self.deprecated = v;
        self
    }

    fn responses(mut self, v: Vec<(String, String)>) -> Self {
        self.responses = v;
        self
    }

    /// Generate `Option<String>` token: `Some("...".to_string())` or `None`
    ///
    /// / 生成 `Option<String>` token
    fn option_tokens(opt: &Option<String>) -> TokenStream {
        match opt {
            Some(s) => quote! { Some(#s.to_string()) },
            None => quote! { None },
        }
    }

    /// Generate `Vec<String>` token: `vec!["a".to_string(), ...]`
    ///
    /// / 生成 `Vec<String>` token
    fn vec_tokens(items: &[String]) -> TokenStream {
        quote! { vec![#(#items.to_string()),*] }
    }

    /// Build the final `HandlerMeta { ... }` TokenStream
    ///
    /// / 构建最终的 `HandlerMeta { ... }` TokenStream
    fn build(self) -> TokenStream {
        let summary_tokens = Self::option_tokens(&self.summary);
        let description_tokens = Self::option_tokens(&self.description);
        let tags_tokens = Self::vec_tokens(&self.tags);
        let security_tokens = Self::vec_tokens(&self.security);

        let param_tokens: Vec<TokenStream> = self
            .params
            .iter()
            .map(|p| {
                let name = &p.name;
                let required = p.required;
                let schema_type = &p.schema_type;
                let location = if p.is_path {
                    quote! { ::astrea::openapi::ParamLocation::Path }
                } else {
                    quote! { ::astrea::openapi::ParamLocation::Query }
                };
                let format_tokens = Self::option_tokens(&p.schema_format);
                quote! {
                    ::astrea::openapi::ParamMeta {
                        name: #name.to_string(),
                        location: #location,
                        required: #required,
                        schema_type: #schema_type.to_string(),
                        schema_format: #format_tokens,
                    }
                }
            })
            .collect();

        let request_body_tokens = match &self.request_body {
            Some(type_name) => quote! {
                Some(::astrea::openapi::RequestBodyMeta {
                    content_type: "application/json".to_string(),
                    schema_type_name: #type_name.to_string(),
                })
            },
            None => quote! { None },
        };

        let response_ct = &self.response_content_type;
        let response_ct_tokens = quote! { #response_ct.to_string() };
        let response_fields_tokens = Self::vec_tokens(&self.response_fields);

        let deprecated = self.deprecated;

        let response_codes: Vec<&String> = self.responses.iter().map(|(c, _)| c).collect();
        let response_descs: Vec<&String> = self.responses.iter().map(|(_, d)| d).collect();

        quote! {
            ::astrea::openapi::HandlerMeta {
                summary: #summary_tokens,
                description: #description_tokens,
                tags: #tags_tokens,
                security: #security_tokens,
                parameters: vec![#(#param_tokens),*],
                request_body: #request_body_tokens,
                response_content_type: #response_ct_tokens,
                response_schema_fields: #response_fields_tokens,
                deprecated: #deprecated,
                responses: vec![#((#response_codes.to_string(), #response_descs.to_string())),*],
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Public API
// 公共 API
// ---------------------------------------------------------------------------

/// Analyze a handler function and produce a `TokenStream` that constructs `HandlerMeta`
///
/// / 分析处理函数并生成构造 `HandlerMeta` 的 `TokenStream`
pub fn analyze_handler(input_fn: &ItemFn) -> TokenStream {
    // Parse doc comment annotations
    // 解析文档注释标注
    let doc = parse_doc_annotations(&input_fn.attrs);

    // Walk the function body AST
    // 遍历函数体 AST
    let mut visitor = HandlerVisitor::default();
    visitor.visit_block(&input_fn.block);
    visitor.apply_deferred_type_updates();

    // Capture borrowed values before moving out of visitor
    // 在移动 visitor 字段之前捕获借用值
    let response_ct = visitor.response_content_type();

    MetaTokenBuilder::new()
        .summary(doc.summary)
        .description(doc.description)
        .tags(doc.tags)
        .security(doc.security)
        .params(visitor.params)
        .request_body(visitor.body_type_name)
        .response_content_type(response_ct)
        .response_fields(visitor.json_macro_keys)
        .deprecated(doc.deprecated)
        .responses(doc.responses)
        .build()
}
