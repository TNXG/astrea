//! Router expression building logic

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::scanner::MiddlewareScope;

/// Build the router expression for a single scope
///
/// / 为单个作用域构建路由器表达式
///
/// Handles four cases:
///
/// 处理四种情况：
///
/// 1. No middleware, no children → flat `Router::new().route(...)` (backward compatible)
///    无中间件，无子级 → 扁平 `Router::new().route(...)`（向后兼容）
/// 2. No middleware, has children → flat routes merged with child routers
///    无中间件，有子级 → 扁平路由与子路由器合并
/// 3. Has middleware, no children → routes wrapped by middleware
///    有中间件，无子级 → 路由被中间件包裹
/// 4. Has middleware, has children → extend/override grouping with middleware
///    有中间件，有子级 → 叠加/覆盖分组加中间件
pub fn build_router_expr(
    scope: &MiddlewareScope,
    route_regs: &[TokenStream],
    child_blocks: &[TokenStream],
) -> TokenStream {
    let has_mw = scope.middleware.is_some();
    let has_children = !child_blocks.is_empty();

    match (has_mw, has_children) {
        // Case 1: No middleware, no children — simple flat router (backward compatible)
        // 情况1：无中间件，无子级 — 简单的扁平路由器（向后兼容）
        (false, false) => {
            quote! {
                ::astrea::axum::Router::new()
                    #(#route_regs)*
            }
        }

        // Case 2: No middleware, has children — flat routes + merge children
        // 情况2：无中间件，有子级 — 扁平路由 + 合并子级
        // (mode is irrelevant since parent has nothing to inherit)
        // （模式无关紧要，因为父级没有要继承的中间件）
        (false, true) => {
            quote! {
                {
                    let mut __router = ::astrea::axum::Router::new()
                        #(#route_regs)*;
                    #(
                        let (_, __child) = #child_blocks;
                        __router = __router.merge(__child);
                    )*
                    __router
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
                    __scope_mw.apply(__routes)
                }
            }
        }

        // Case 4: Has middleware + children — proximity principle with extend/override
        // 情况4：有中间件 + 子级 — 就近原则，叠加/覆盖分组
        //
        // Extend children: wrapped by this scope's middleware (stacking/叠加)
        // Override children: NOT wrapped, only their own middleware applies (覆盖)
        //
        // 叠加子级：被此作用域中间件包裹
        // 覆盖子级：不被包裹，仅自身中间件生效
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
                        let (__mode, __child) = #child_blocks;
                        if __mode == ::astrea::middleware::MiddlewareMode::Override {
                            __override_group = __override_group.merge(__child);
                        } else {
                            __extend = __extend.merge(__child);
                        }
                    )*

                    let __scope_mw = #mw_mod::middleware::<S>();
                    __scope_mw.apply(__extend).merge(__override_group)
                }
            }
        }
    }
}
