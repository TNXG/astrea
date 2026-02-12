//! `#[route]` attribute macro implementation
//!
//! / `#[route]` 属性宏实现

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

/// Implementation of the `#[route]` attribute macro
///
/// / `#[route]` 属性宏的实现
pub fn impl_route(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);

    let vis = &input_fn.vis;
    let fn_name = &input_fn.sig.ident;
    let inputs = &input_fn.sig.inputs;
    let block = &input_fn.block;

    if input_fn.sig.asyncness.is_none() {
        return syn::Error::new_spanned(
            fn_name,
            "#[route] 函数必须是 async fn / #[route] function must be async fn",
        )
        .to_compile_error()
        .into();
    }

    // 解析参数名 / Parse parameter name
    let mut event_param_name = None;
    for input in inputs {
        if let syn::FnArg::Typed(arg) = input
            && let syn::Pat::Ident(ident) = &*arg.pat
            && ident.ident == "event"
        {
            event_param_name = Some(ident.ident.clone());
        }
    }

    let event_name = event_param_name
        .unwrap_or_else(|| syn::Ident::new("event", proc_macro2::Span::call_site()));

    // 生成 OpenAPI 元数据函数（仅当启用 openapi feature 时）
    // Generate OpenAPI metadata function (only when openapi feature is enabled)
    #[cfg(feature = "openapi")]
    let openapi_fn = {
        let meta_tokens = crate::openapi::analyze_handler(&input_fn);
        quote! {
            pub fn __openapi_meta() -> ::astrea::openapi::HandlerMeta {
                #meta_tokens
            }
        }
    };
    #[cfg(not(feature = "openapi"))]
    let openapi_fn = quote! {};

    // 生成包装函数 — 所有外部类型通过 ::astrea:: 引用，用户无需直接依赖 axum / bytes
    // Generate wrapper function - all external types referenced via ::astrea::
    let expanded = quote! {
        #vis async fn #fn_name(
            __method: ::astrea::axum::http::Method,
            __uri: ::astrea::axum::http::Uri,
            __headers: ::astrea::axum::http::HeaderMap,
            __path_params: ::astrea::axum::extract::Path<std::collections::HashMap<String, String>>,
            __query_params: ::astrea::axum::extract::Query<std::collections::HashMap<String, String>>,
            __body_bytes: ::astrea::bytes::Bytes,
        ) -> impl ::astrea::axum::response::IntoResponse {
            use ::astrea::{Event, Response};
            use ::astrea::axum::response::IntoResponse;

            let __path = __uri.path().to_string();

            let #event_name = Event::new(
                __method,
                __path,
                __uri,
                __headers,
                __path_params.0,
                __query_params.0,
            );

            let result: ::std::result::Result<::astrea::Response, ::astrea::RouteError> =
                async move #block.await;

            match result {
                Ok(response) => response.into_axum_response(),
                Err(error) => error.into_response(),
            }
        }

        #openapi_fn
    };

    TokenStream::from(expanded)
}
