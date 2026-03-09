//! `#[route]` attribute macro implementation
//!
//! / `#[route]` 属性宏实现

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

/// Route handler type derived from `#[route(...)]` argument
///
/// / 从 `#[route(...)]` 参数推导的路由处理函数类型
#[allow(dead_code)]
enum RouteKind {
    /// Standard HTTP handler: `#[route]`
    /// / 标准 HTTP 处理函数: `#[route]`
    Http,
    /// All-method HTTP handler: `#[route()]`
    /// / 全方法 HTTP 处理函数: `#[route()]`
    Any,
    /// WebSocket handler: `#[route(ws)]`
    /// / WebSocket 处理函数: `#[route(ws)]`
    Ws,
    /// SSE handler: `#[route(sse)]`
    /// / SSE 处理函数: `#[route(sse)]`
    Sse,
}

/// Parse the `#[route(...)]` attribute argument to determine route kind
///
/// / 解析 `#[route(...)]` 属性参数以确定路由类型
fn parse_route_kind(args: TokenStream) -> Result<RouteKind, String> {
    let args_str = args.to_string();
    let trimmed = args_str.trim();

    if trimmed.is_empty() {
        return Ok(RouteKind::Http);
    }

    match trimmed {
        "ws" => Ok(RouteKind::Ws),
        "sse" => Ok(RouteKind::Sse),
        _ => Err(format!(
            "unknown #[route] argument: `{trimmed}`. \
             Expected `ws`, `sse`, or empty. / \
             未知的 #[route] 参数: `{trimmed}`。期望 `ws`、`sse` 或为空。"
        )),
    }
}

/// Implementation of the `#[route]` attribute macro
///
/// / `#[route]` 属性宏的实现
pub fn impl_route(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);

    let fn_name = &input_fn.sig.ident;

    if input_fn.sig.asyncness.is_none() {
        return syn::Error::new_spanned(
            fn_name,
            "#[route] 函数必须是 async fn / #[route] function must be async fn",
        )
        .to_compile_error()
        .into();
    }

    let kind = match parse_route_kind(args) {
        Ok(k) => k,
        Err(msg) => {
            return syn::Error::new_spanned(fn_name, msg)
                .to_compile_error()
                .into();
        }
    };

    match kind {
        RouteKind::Http | RouteKind::Any => generate_http_handler(&input_fn),
        RouteKind::Ws => generate_ws_handler(&input_fn),
        RouteKind::Sse => generate_sse_handler(&input_fn),
    }
}

/// Find the `event` parameter name from function inputs
///
/// / 从函数输入中找到 `event` 参数名
fn find_event_param(input_fn: &ItemFn) -> syn::Ident {
    for input in &input_fn.sig.inputs {
        if let syn::FnArg::Typed(arg) = input
            && let syn::Pat::Ident(ident) = &*arg.pat
            && ident.ident == "event"
        {
            return ident.ident.clone();
        }
    }
    syn::Ident::new("event", proc_macro2::Span::call_site())
}

/// Find the second parameter name (for WebSocket socket / SSE sender)
///
/// / 找到第二个参数名（WebSocket socket / SSE sender）
fn find_second_param(input_fn: &ItemFn) -> syn::Ident {
    let mut count = 0;
    for input in &input_fn.sig.inputs {
        if let syn::FnArg::Typed(arg) = input {
            count += 1;
            if count == 2 {
                if let syn::Pat::Ident(ident) = &*arg.pat {
                    return ident.ident.clone();
                }
            }
        }
    }
    syn::Ident::new("_conn", proc_macro2::Span::call_site())
}

/// Generate the standard HTTP handler wrapper (for #[route] and #[route()])
///
/// / 生成标准 HTTP 处理函数包装器（用于 #[route] 和 #[route()]）
fn generate_http_handler(input_fn: &ItemFn) -> TokenStream {
    let vis = &input_fn.vis;
    let fn_name = &input_fn.sig.ident;
    let block = &input_fn.block;
    let event_name = find_event_param(input_fn);

    // 生成 OpenAPI 元数据函数（仅当启用 openapi feature 时）
    // Generate OpenAPI metadata function (only when openapi feature is enabled)
    #[cfg(feature = "openapi")]
    let openapi_fn = {
        let meta_tokens = crate::openapi::analyze_handler(input_fn);
        quote! {
            pub fn __openapi_meta() -> ::astrea::openapi::HandlerMeta {
                #meta_tokens
            }
        }
    };
    #[cfg(not(feature = "openapi"))]
    let openapi_fn = quote! {};

    let expanded = quote! {
        #vis async fn #fn_name<S>(
            ::astrea::axum::extract::State(__state): ::astrea::axum::extract::State<S>,
            __method: ::astrea::axum::http::Method,
            __uri: ::astrea::axum::http::Uri,
            __headers: ::astrea::axum::http::HeaderMap,
            __path_params: ::astrea::axum::extract::Path<std::collections::HashMap<String, String>>,
            __query_params: ::astrea::axum::extract::Query<std::collections::HashMap<String, String>>,
            __body_bytes: ::astrea::bytes::Bytes,
        ) -> impl ::astrea::axum::response::IntoResponse
        where
            S: Clone + Send + Sync + 'static,
        {
            use ::astrea::{Event, Response};
            use ::astrea::axum::response::IntoResponse;

            let __path = __uri.path().to_string();

            let mut #event_name = Event::new(
                __method,
                __path,
                __uri,
                __headers,
                __path_params.0,
                __query_params.0,
                __body_bytes,
            );

            // 注入状态 / Inject state
            #event_name.state = Some(::std::sync::Arc::new(__state) as ::std::sync::Arc<dyn ::std::any::Any + Send + Sync>);

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

/// Generate the WebSocket handler wrapper (for #[route(ws)])
///
/// / 生成 WebSocket 处理函数包装器（用于 #[route(ws)]）
fn generate_ws_handler(input_fn: &ItemFn) -> TokenStream {
    let vis = &input_fn.vis;
    let fn_name = &input_fn.sig.ident;
    let block = &input_fn.block;
    let event_name = find_event_param(input_fn);
    let socket_name = find_second_param(input_fn);

    let expanded = quote! {
        #vis async fn #fn_name<S>(
            ::astrea::axum::extract::State(__state): ::astrea::axum::extract::State<S>,
            __ws: ::astrea::axum::extract::WebSocketUpgrade,
            __uri: ::astrea::axum::http::Uri,
            __headers: ::astrea::axum::http::HeaderMap,
            __path_params: ::astrea::axum::extract::Path<std::collections::HashMap<String, String>>,
            __query_params: ::astrea::axum::extract::Query<std::collections::HashMap<String, String>>,
        ) -> impl ::astrea::axum::response::IntoResponse
        where
            S: Clone + Send + Sync + 'static,
        {
            use ::astrea::Event;

            let __path = __uri.path().to_string();

            let mut #event_name = Event::new(
                ::astrea::axum::http::Method::GET,
                __path,
                __uri,
                __headers,
                __path_params.0,
                __query_params.0,
                ::astrea::bytes::Bytes::new(),
            );

            // 注入状态 / Inject state
            #event_name.state = Some(::std::sync::Arc::new(__state) as ::std::sync::Arc<dyn ::std::any::Any + Send + Sync>);

            __ws.on_upgrade(move |__raw_socket| async move {
                let mut #socket_name = ::astrea::ws::WebSocket::new(__raw_socket);
                async move #block.await;
            })
        }

        /// Route type identifier for TUI display
        /// / 路由类型标识符，用于 TUI 显示
        pub const __ROUTE_TYPE: &str = "WS";
    };

    TokenStream::from(expanded)
}

/// Generate the SSE handler wrapper (for #[route(sse)])
///
/// / 生成 SSE 处理函数包装器（用于 #[route(sse)]）
fn generate_sse_handler(input_fn: &ItemFn) -> TokenStream {
    let vis = &input_fn.vis;
    let fn_name = &input_fn.sig.ident;
    let block = &input_fn.block;
    let event_name = find_event_param(input_fn);
    let sender_name = find_second_param(input_fn);

    let expanded = quote! {
        #vis async fn #fn_name<S>(
            ::astrea::axum::extract::State(__state): ::astrea::axum::extract::State<S>,
            __method: ::astrea::axum::http::Method,
            __uri: ::astrea::axum::http::Uri,
            __headers: ::astrea::axum::http::HeaderMap,
            __path_params: ::astrea::axum::extract::Path<std::collections::HashMap<String, String>>,
            __query_params: ::astrea::axum::extract::Query<std::collections::HashMap<String, String>>,
        ) -> impl ::astrea::axum::response::IntoResponse
        where
            S: Clone + Send + Sync + 'static,
        {
            use ::astrea::Event;
            use ::astrea::tokio_stream::StreamExt as _;

            let __path = __uri.path().to_string();

            let mut #event_name = Event::new(
                __method,
                __path,
                __uri,
                __headers,
                __path_params.0,
                __query_params.0,
                ::astrea::bytes::Bytes::new(),
            );

            // 注入状态 / Inject state
            #event_name.state = Some(::std::sync::Arc::new(__state) as ::std::sync::Arc<dyn ::std::any::Any + Send + Sync>);

            let (__tx, __rx) = ::astrea::tokio::sync::mpsc::channel::<::astrea::sse::SseEvent>(32);
            let #sender_name = ::astrea::sse::SseSender::new(__tx);

            ::astrea::tokio::spawn(async move {
                async move #block.await;
            });

            ::astrea::axum::response::sse::Sse::new(
                ::astrea::tokio_stream::wrappers::ReceiverStream::new(__rx)
                    .map(|evt| Ok::<_, ::std::convert::Infallible>(evt.into_axum_event()))
            )
            .keep_alive(::astrea::axum::response::sse::KeepAlive::default())
        }

        /// Route type identifier for TUI display
        /// / 路由类型标识符，用于 TUI 显示
        pub const __ROUTE_TYPE: &str = "SSE";
    };

    TokenStream::from(expanded)
}
