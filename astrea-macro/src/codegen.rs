//! Code generation logic for route macros
//!
//! / 路由宏的代码生成逻辑

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::path::{Path, PathBuf};
use syn::Ident;

use crate::scanner::{MiddlewareScope, collect_middleware_logs, collect_route_logs};

/// Implementation of the `generate_routes!` procedural macro
///
/// / `generate_routes!` 过程宏的实现
pub fn impl_generate_routes(input: TokenStream) -> TokenStream {
    let routes_dir_name = if input.is_empty() {
        "src/routes".to_string()
    } else {
        let lit = syn::parse_macro_input!(input as syn::LitStr);
        lit.value()
    };

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR environment variable not set");
    let routes_dir = PathBuf::from(&manifest_dir).join(&routes_dir_name);

    if !routes_dir.exists() {
        let msg = format!(
            "astrea: routes directory not found: {}",
            routes_dir.display()
        );
        return quote! { compile_error!(#msg); }.into();
    }

    // Phase 1: Scan directories and build middleware scope tree
    // 阶段1: 扫描目录并构建中间件作用域树
    let root_scope = crate::scanner::scan_and_build_scope(&routes_dir, &[], &manifest_dir);

    // Collect info for logging
    // 收集日志信息
    let route_logs = collect_route_logs(&root_scope);
    let mw_logs = collect_middleware_logs(&root_scope);
    let route_count = route_logs.len();
    let mw_count = mw_logs.len();

    let route_log_stmts: Vec<_> = route_logs
        .iter()
        .map(|(method, path)| {
            let log_line = format!("  {:<6} {}", method, path);
            quote! { ::astrea::tracing::info!("{}", #log_line); }
        })
        .collect();

    let mw_log_section = if mw_count > 0 {
        let mw_log_stmts: Vec<_> = mw_logs
            .iter()
            .map(|path| {
                let log_line = format!("  {}", path);
                quote! { ::astrea::tracing::info!("{}", #log_line); }
            })
            .collect();
        quote! {
            ::astrea::tracing::info!("Middleware scope(s): {}", #mw_count);
            #(#mw_log_stmts)*
        }
    } else {
        quote! {}
    };

    // Phase 2: Generate module declarations and router expression
    // 阶段2: 生成模块声明和路由器表达式
    let (mod_decls, router_expr) = generate_scope_code(&root_scope, &manifest_dir);

    let expanded = quote! {
        #(#mod_decls)*

        /// Create a Router with all file-based routes and middleware
        /// / 创建包含所有文件路由和中间件的 Router
        pub fn create_router() -> ::astrea::axum::Router {
            ::astrea::tracing::info!("Initializing file router...");
            ::astrea::tracing::info!("Registered {} route(s):", #route_count);
            #(#route_log_stmts)*
            #mw_log_section

            #router_expr
        }
    };

    expanded.into()
}

/// Generate module declarations and the router expression for a scope
///
/// / 为作用域生成模块声明和路由器表达式
///
/// Returns `(module_declarations, router_expression)`.
///
/// 返回 `(模块声明列表, 路由器表达式)`。
fn generate_scope_code(
    scope: &MiddlewareScope,
    manifest_dir: &str,
) -> (Vec<proc_macro2::TokenStream>, proc_macro2::TokenStream) {
    let mut mod_decls = Vec::new();

    // ── Module declarations for routes in this scope ──
    // ── 此作用域中路由的模块声明 ──
    for route in &scope.routes {
        let mod_name = Ident::new(&route.module_name, Span::call_site());
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
    }

    // ── Module declaration for this scope's middleware ──
    // ── 此作用域的中间件模块声明 ──
    if let Some(mw) = &scope.middleware {
        let mw_mod = Ident::new(&mw.module_name, Span::call_site());
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
            let method_fn = Ident::new(&r.method.to_lowercase(), Span::call_site());
            let mod_name = Ident::new(&r.module_name, Span::call_site());
            quote! {
                .route(#axum_path, ::astrea::axum::routing::#method_fn(#mod_name::handler))
            }
        })
        .collect();

    // ── Recursively process child scopes ──
    // ── 递归处理子作用域 ──
    let mut child_blocks: Vec<proc_macro2::TokenStream> = Vec::new();
    for child in &scope.children {
        let (child_mods, child_router_expr) = generate_scope_code(child, manifest_dir);
        mod_decls.extend(child_mods);

        let child_mw_mod = Ident::new(
            &child.middleware.as_ref().unwrap().module_name,
            Span::call_site(),
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

    (mod_decls, router_expr)
}

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
fn build_router_expr(
    scope: &MiddlewareScope,
    route_regs: &[proc_macro2::TokenStream],
    child_blocks: &[proc_macro2::TokenStream],
) -> proc_macro2::TokenStream {
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
                Span::call_site(),
            );
            quote! {
                {
                    let __routes = ::astrea::axum::Router::new()
                        #(#route_regs)*;
                    let __scope_mw = #mw_mod::middleware();
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
                Span::call_site(),
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

                    let __scope_mw = #mw_mod::middleware();
                    __scope_mw.apply(__extend).merge(__override_group)
                }
            }
        }
    }
}
