//! Scope code generation logic

use proc_macro2::TokenStream;
use quote::quote;
use std::path::Path;
use syn::Ident;

use super::router::build_router_expr;
use crate::scanner::MiddlewareScope;

/// Generate module declarations and the router expression for a scope
///
/// / 为作用域生成模块声明和路由器表达式
///
/// Returns `(module_declarations, router_expression, openapi_registrations)`.
///
/// 返回 `(模块声明列表, 路由器表达式, OpenAPI 注册列表)`。
pub fn generate_scope_code(
    scope: &MiddlewareScope,
    manifest_dir: &str,
) -> (Vec<TokenStream>, TokenStream, Vec<TokenStream>) {
    let mut mod_decls = Vec::new();
    let mut openapi_regs = Vec::new();

    // ── Module declarations for routes in this scope ──
    // ── 此作用域中路由的模块声明 ──
    for route in &scope.routes {
        let mod_name = Ident::new(&route.module_name, proc_macro2::Span::call_site());
        let rel_path = Path::new(&route.file_path)
            .strip_prefix(manifest_dir)
            .map(|p| format!("/{}", p.to_string_lossy()))
            .unwrap_or_else(|_| route.file_path.clone());
        mod_decls.push(quote! {
            #[allow(unused_imports)]
            mod #mod_name {
                include!(concat!(env!("CARGO_MANIFEST_DIR"), #rel_path));
            }
        });

        // OpenAPI registration (only when openapi feature is enabled)
        // OpenAPI 注册（仅当启用 openapi feature 时）
        #[cfg(feature = "openapi")]
        {
            let method_str = &route.method;
            let openapi_path = super::openapi::axum_path_to_openapi(&route.axum_path);
            let op_id = &route.module_name;
            openapi_regs.push(quote! {
                ::astrea::openapi::register(
                    #method_str,
                    #openapi_path,
                    #op_id,
                    #mod_name::__openapi_meta(),
                );
            });
        }
    }

    // ── Module declaration for this scope's middleware ──
    // ── 此作用域的中间件模块声明 ──
    if let Some(mw) = &scope.middleware {
        let mw_mod = Ident::new(&mw.module_name, proc_macro2::Span::call_site());
        let mw_rel = &mw.rel_path;
        mod_decls.push(quote! {
            #[allow(unused_imports)]
            mod #mw_mod {
                include!(concat!(env!("CARGO_MANIFEST_DIR"), #mw_rel));
            }
        });
    }

    // ── Route registration tokens ──
    // ── 路由注册令牌 ──
    let route_regs: Vec<_> = scope
        .routes
        .iter()
        .map(|r| {
            let axum_path = &r.axum_path;
            let method_fn = Ident::new(&r.method.to_lowercase(), proc_macro2::Span::call_site());
            let mod_name = Ident::new(&r.module_name, proc_macro2::Span::call_site());
            quote! {
                .route(#axum_path, ::astrea::axum::routing::#method_fn(#mod_name::handler))
            }
        })
        .collect();

    // ── Recursively process child scopes ──
    // ── 递归处理子作用域 ──
    let mut child_blocks: Vec<TokenStream> = Vec::new();
    for child in &scope.children {
        let (child_mods, child_router_expr, child_openapi_regs) =
            generate_scope_code(child, manifest_dir);
        mod_decls.extend(child_mods);
        openapi_regs.extend(child_openapi_regs);

        let child_mw_mod = Ident::new(
            &child.middleware.as_ref().unwrap().module_name,
            proc_macro2::Span::call_site(),
        );

        child_blocks.push(quote! {
            {
                let __inner = #child_router_expr;
                let __mw = #child_mw_mod::middleware();
                let __mode = __mw.mode;
                let __built = __mw.apply(__inner);
                (__mode, __built)
            }
        });
    }

    // ── Build router expression ──
    // ── 构建路由器表达式 ──
    let router_expr = build_router_expr(scope, &route_regs, &child_blocks);

    (mod_decls, router_expr, openapi_regs)
}
