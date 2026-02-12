//! Code generation logic for route macros
//!
//! / 路由宏的代码生成逻辑

mod openapi;
mod router;
mod scope;

pub use scope::generate_scope_code;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::Ident;

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
    let routes_dir = std::path::PathBuf::from(&manifest_dir).join(&routes_dir_name);

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

    // Collect info for TUI logging
    // 收集 TUI 日志信息
    let route_detail_logs = crate::scanner::collect_route_detail_logs(&root_scope, &[]);
    let mw_detail_logs = crate::scanner::collect_middleware_detail_logs(&root_scope, None);
    let route_count = route_detail_logs.len();
    let mw_count = mw_detail_logs.len();

    // ── Build TUI route table data (compile-time) ──
    // ── 构建 TUI 路由表数据（编译时）──
    let route_methods: Vec<&str> = route_detail_logs
        .iter()
        .map(|r| r.method.as_str())
        .collect();
    let route_paths: Vec<&str> = route_detail_logs.iter().map(|r| r.path.as_str()).collect();
    let route_mw_chains: Vec<String> = route_detail_logs
        .iter()
        .map(|r| {
            if r.middleware_chain.is_empty() {
                "(none)".to_string()
            } else {
                r.middleware_chain.join(" → ")
            }
        })
        .collect();
    let route_mw_chain_refs: Vec<&str> = route_mw_chains.iter().map(|s| s.as_str()).collect();

    // ── Build TUI middleware table data (compile-time static parts) ──
    // ── 构建 TUI 中间件表数据（编译时静态部分）──
    let mw_scope_paths: Vec<&str> = mw_detail_logs
        .iter()
        .map(|m| m.scope_path.as_str())
        .collect();
    let mw_parent_paths: Vec<String> = mw_detail_logs
        .iter()
        .map(|m| match &m.parent_path {
            Some(p) => p.clone(),
            None => String::new(),
        })
        .collect();
    let mw_parent_path_refs: Vec<&str> = mw_parent_paths.iter().map(|s| s.as_str()).collect();

    // Generate runtime mode probe code for each middleware scope
    // 为每个中间件作用域生成运行时 mode 探测代码
    let mw_mode_probes: Vec<proc_macro2::TokenStream> = mw_detail_logs
        .iter()
        .map(|m| {
            let mod_ident = Ident::new(&m.module_name, Span::call_site());
            quote! {
                {
                    let __probe = #mod_ident::middleware::<S>();
                    if __probe.mode == ::astrea::middleware::MiddlewareMode::Override {
                        "override"
                    } else {
                        "extend"
                    }
                }
            }
        })
        .collect();

    // Phase 2: Generate module declarations and router expression
    // 阶段2: 生成模块声明和路由器表达式
    let (mod_decls, router_expr, openapi_regs) = generate_scope_code(&root_scope, &manifest_dir);

    // OpenAPI registration section (only when openapi feature is enabled)
    // OpenAPI 注册部分（仅当启用 openapi feature 时）
    let openapi_section = if openapi_regs.is_empty() {
        quote! {}
    } else {
        quote! { #(#openapi_regs)* }
    };

    // OpenAPI TUI section (only when openapi feature is enabled and there are registrations)
    // OpenAPI TUI 部分（仅当启用 openapi feature 且有注册时）
    let openapi_tui_section = if cfg!(feature = "openapi") && !openapi_regs.is_empty() {
        quote! {
            // OpenAPI summary / OpenAPI 摘要
            {
                let entries = ::astrea::openapi::registry::get_entries();
                let op_count = entries.len();

                if op_count > 0 {
                    let mut openapi_lines: Vec<String> = Vec::new();
                    for entry in &entries {
                        let summary = entry.handler_meta.summary.as_deref().unwrap_or("-");
                        let param_count = entry.handler_meta.parameters.len();
                        let body = if entry.handler_meta.request_body.is_some() { " +body" } else { "" };
                        openapi_lines.push(format!(
                            "  {:<6} {:<28} {} | {}p{}",
                            entry.method, entry.path, summary, param_count, body,
                        ));
                    }

                    ::astrea::tracing::info!("");
                    ::astrea::tracing::info!("📄 OpenAPI: {} operation(s) registered", op_count);
                    for line in &openapi_lines {
                        ::astrea::tracing::info!("{}", line);
                    }
                }
            }
        }
    } else {
        quote! {}
    };

    let expanded = quote! {
        #(#mod_decls)*

        /// Create a Router with all file-based routes and middleware
        /// / 创建包含所有文件路由和中间件的 Router
        pub fn create_router<S: Clone + Send + Sync + 'static>() -> ::astrea::axum::Router<S> {
            // ── TUI Logging with comfy_table ──
            // ── 使用 comfy_table 进行 TUI 日志输出 ──
            {
                use::astrea::comfy_table::{Table, Row, Cell, presets, Attribute, CellAlignment, ContentArrangement};

                // 1. 构建路由表 (Routes Table)
                let mut table = Table::new();
                table.load_preset(presets::UTF8_FULL);
                table.set_content_arrangement(ContentArrangement::Dynamic);
                table.force_no_tty(); // 禁用 ANSI 颜色代码

                // 标题行 / Title Row
                let mut title_row = Row::new();
                title_row.add_cell(
                    Cell::new("🚀 Astrea Router")
                        .add_attribute(Attribute::Bold)
                        .set_alignment(CellAlignment::Center)

                );
                table.add_row(title_row);

                // 表头 / Headers
                table.set_header(vec![
                    Cell::new("Method").add_attribute(Attribute::Bold),
                    Cell::new("Path").add_attribute(Attribute::Bold),
                    Cell::new("Middleware").add_attribute(Attribute::Bold),
                ]);

                // 数据准备 / Data Preparation
                let __methods: &[&str] = &[#(#route_methods),*];
                let __paths: &[&str] = &[#(#route_paths),*];
                let __mw_chains: &[&str] = &[#(#route_mw_chain_refs),*];

                // 填充路由数据 / Fill Route Data
                for __i in 0..#route_count {
                    table.add_row(vec![
                        __methods[__i],
                        __paths[__i],
                        __mw_chains[__i],
                    ]);
                }

                // 打印路由表 / Print Routes
                // Split by newline to preserve log formatting per line
                // 按换行符分割以保持每行的日志格式整齐
                ::astrea::tracing::info!("");
                for line in table.to_string().lines() {
                    ::astrea::tracing::info!("{}", line);
                }

                // 2. 构建中间件表 (Middleware Table)
                if #mw_count > 0 {
                    let mut mw_table = Table::new();
                    mw_table.load_preset(presets::UTF8_FULL);
                    mw_table.set_content_arrangement(ContentArrangement::Dynamic);
                    mw_table.force_no_tty(); // 禁用 ANSI 颜色代码

                    // 中间件标题 / Middleware Title
                    let mut mw_title_row = Row::new();
                    mw_title_row.add_cell(
                        Cell::new("📦 Middleware Scopes")
                            .add_attribute(Attribute::Bold)
                            .set_alignment(CellAlignment::Center)
                    );
                    mw_table.add_row(mw_title_row);

                    // 中间件表头 / Middleware Headers
                    mw_table.set_header(vec![
                        Cell::new("Scope").add_attribute(Attribute::Bold),
                        Cell::new("Mode").add_attribute(Attribute::Bold),
                        Cell::new("Inherits").add_attribute(Attribute::Bold),
                    ]);

                    let __scope_paths: &[&str] = &[#(#mw_scope_paths),*];
                    let __parent_paths: &[&str] = &[#(#mw_parent_path_refs),*];
                    let __modes: &[&str] = &[#(#mw_mode_probes),*];

                    for __i in 0..#mw_count {
                        let __mode_display = if __parent_paths[__i].is_empty() {
                            "─"
                        } else if __modes[__i] == "override" {
                            "override"
                        } else {
                            "extend"
                        };

                        let __inherit_display = if __parent_paths[__i].is_empty() {
                            "(root)".to_string()
                        } else if __modes[__i] == "override" {
                            "⚡ standalone".to_string()
                        } else {
                            format!("← {}", __parent_paths[__i])
                        };

                        mw_table.add_row(vec![
                            __scope_paths[__i],
                            __mode_display,
                            __inherit_display.as_str(),
                        ]);
                    }

                    // 打印中间件表 / Print Middleware Table
                    // 为了视觉上的连贯性，这里也可以选择不打印头部空行，紧贴着上一个表
                    for line in mw_table.to_string().lines() {
                        ::astrea::tracing::info!("{}", line);
                    }
                }

                ::astrea::tracing::info!("✅ {} route(s), {} middleware scope(s) loaded", #route_count, #mw_count);
                ::astrea::tracing::info!("");
            }

            #openapi_section

            // OpenAPI TUI (after registration)
            #openapi_tui_section

            #router_expr
        }
    };
    expanded.into()
}
