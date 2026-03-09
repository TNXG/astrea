//! Router expression building logic

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::scanner::MiddlewareScope;

/// Build the router expression for a single scope
///
/// / 为单个作用域构建路由器表达式
///
/// Returns a `TokenStream` that evaluates to
/// `(Router<S>, Router<S>)` — `(wrapped_router, override_passthrough)`.
///
/// 返回一个求值为 `(Router<S>, Router<S>)` 的 `TokenStream`
/// — `(包裹后的路由器, 覆盖透传路由器)`。
///
/// Override children bubble up through all ancestor scopes without being
/// wrapped by any ancestor middleware, ensuring `Middleware::override_parent()`
/// truly discards **all** ancestor layers.
///
/// 覆盖子级会逐层向上冒泡，不被任何祖先中间件包裹，
/// 确保 `Middleware::override_parent()` 真正丢弃**所有**祖先层。
///
/// Handles four cases:
///
/// 处理四种情况：
///
/// 1. No middleware, no children → flat `Router::new().route(...)`
///    无中间件，无子级 → 扁平路由
/// 2. No middleware, has children → flat routes + propagate children's override
///    无中间件，有子级 → 扁平路由 + 透传子级的覆盖组
/// 3. Has middleware, no children → routes wrapped by middleware
///    有中间件，无子级 → 路由被中间件包裹
/// 4. Has middleware, has children → wrap extend group, propagate override group
///    有中间件，有子级 → 包裹叠加组，透传覆盖组
pub fn build_router_expr(
    scope: &MiddlewareScope,
    route_regs: &[TokenStream],
    child_blocks: &[TokenStream],
) -> TokenStream {
    let has_mw = scope.middleware.is_some();
    let has_children = !child_blocks.is_empty();

    match (has_mw, has_children) {
        // Case 1: No middleware, no children — simple flat router
        // 情况1：无中间件，无子级 — 简单的扁平路由器
        (false, false) => {
            quote! {
                (
                    ::astrea::axum::Router::new()
                        #(#route_regs)*,
                    ::astrea::axum::Router::new()
                )
            }
        }

        // Case 2: No middleware, has children — flat routes + propagate override
        // 情况2：无中间件，有子级 — 扁平路由 + 透传覆盖组
        (false, true) => {
            quote! {
                {
                    let mut __extend = ::astrea::axum::Router::new()
                        #(#route_regs)*;
                    let mut __override_group = ::astrea::axum::Router::new();
                    #(
                        let (__child_ext, __child_ovr) = #child_blocks;
                        __extend = __extend.merge(__child_ext);
                        __override_group = __override_group.merge(__child_ovr);
                    )*
                    (__extend, __override_group)
                }
            }
        }

        // Case 3: Has middleware, no children — routes wrapped by middleware
        // 情况3：有中间件，无子级 — 路由被中间件包裹
        (true, false) => {
            let mw_mod = Ident::new(
                &scope.middleware.as_ref().unwrap().module_name,
                proc_macro2::Span::call_site(),
            );
            quote! {
                {
                    let __routes = ::astrea::axum::Router::new()
                        #(#route_regs)*;
                    let __scope_mw = #mw_mod::middleware::<S>();
                    (__scope_mw.apply(__routes), ::astrea::axum::Router::new())
                }
            }
        }

        // Case 4: Has middleware + children — wrap extend, propagate override
        // 情况4：有中间件 + 子级 — 包裹叠加组，透传覆盖组
        //
        // Extend children: wrapped by this scope's middleware (stacking/叠加)
        // Override children: bubble up without wrapping (覆盖透传)
        //
        // 叠加子级：被此作用域中间件包裹
        // 覆盖子级：向上冒泡，不被包裹
        (true, true) => {
            let mw_mod = Ident::new(
                &scope.middleware.as_ref().unwrap().module_name,
                proc_macro2::Span::call_site(),
            );
            quote! {
                {
                    let __direct = ::astrea::axum::Router::new()
                        #(#route_regs)*;
                    let mut __extend = __direct;
                    let mut __override_group = ::astrea::axum::Router::new();

                    #(
                        let (__child_ext, __child_ovr) = #child_blocks;
                        __extend = __extend.merge(__child_ext);
                        __override_group = __override_group.merge(__child_ovr);
                    )*

                    let __scope_mw = #mw_mod::middleware::<S>();
                    (__scope_mw.apply(__extend), __override_group)
                }
            }
        }
    }
}
