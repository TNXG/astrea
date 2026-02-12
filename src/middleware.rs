//! Middleware system for Astrea
//!
//! / Astrea 中间件系统
//!
//! Provides file-based middleware support through `_middleware.rs` convention files.
//! Middleware functions are scoped to their directory and all subdirectories.
//!
//! 通过 `_middleware.rs` 约定文件提供基于文件的中间件支持。
//! 中间件函数作用于其所在目录及所有子目录。
//!
//! # Middleware Scoping — 就近原则 (Proximity Principle)
//!
//! # 中间件作用域 — 就近原则
//!
//! Middleware is applied based on directory proximity:
//!
//! 中间件根据目录就近原则应用：
//!
//! ```text
//! routes/
//! ├── _middleware.rs          # Applies to ALL routes / 作用于所有路由
//! ├── index.get.rs            # ← root middleware
//! ├── api/
//! │   ├── _middleware.rs      # Applies to /api/* routes / 作用于 /api/* 路由
//! │   ├── users.get.rs        # ← root + api middleware
//! │   └── admin/
//! │       ├── _middleware.rs   # Applies to /api/admin/* / 作用于 /api/admin/*
//! │       └── dashboard.get.rs # ← root + api + admin middleware
//! └── public/
//!     └── health.get.rs       # ← root middleware only
//! ```
//!
//! # Middleware Modes — 覆盖与叠加
//!
//! # 中间件模式 — 覆盖与叠加
//!
//! ## Extend (叠加, default)
//!
//! Child middleware **stacks on top of** parent middleware. Both apply.
//! Execution order (onion model): parent → child → handler → child → parent
//!
//! 子中间件**叠加在**父中间件之上。两者同时生效。
//! 执行顺序（洋葱模型）：父 → 子 → 处理函数 → 子 → 父
//!
//! ## Override (覆盖)
//!
//! Child middleware **replaces** parent middleware. Only child's middleware applies.
//! Use this for routes that need completely different middleware (e.g., public endpoints
//! that should skip authentication).
//!
//! 子中间件**替换**父中间件。仅子中间件生效。
//! 适用于需要完全不同中间件的路由（如应跳过认证的公开端点）。
//!
//! # Example
//!
//! # 示例
//!
//! ```rust,ignore
//! // routes/_middleware.rs — root middleware (applies to all routes)
//! use astrea::middleware::*;
//!
//! pub fn middleware() -> Middleware {
//!     Middleware::new()
//!         .wrap(|router| {
//!             router
//!                 .layer(tower_http::trace::TraceLayer::new_for_http())
//!                 .layer(tower_http::cors::CorsLayer::permissive())
//!         })
//! }
//! ```
//!
//! ```rust,ignore
//! // routes/api/_middleware.rs — extends root middleware (叠加)
//! use astrea::middleware::*;
//!
//! pub fn middleware() -> Middleware {
//!     Middleware::new()  // default: Extend mode
//!         .wrap(|router| {
//!             router.layer(axum::middleware::from_fn(auth_check))
//!         })
//! }
//!
//! async fn auth_check(
//!     req: axum::extract::Request,
//!     next: axum::middleware::Next,
//! ) -> axum::response::Response {
//!     use axum::response::IntoResponse;
//!     match req.headers().get("authorization") {
//!         Some(_) => next.run(req).await,
//!         None => (
//!             axum::http::StatusCode::UNAUTHORIZED,
//!             axum::Json(serde_json::json!({"error": "unauthorized"})),
//!         ).into_response(),
//!     }
//! }
//! ```
//!
//! ```rust,ignore
//! // routes/api/public/_middleware.rs — overrides parent middleware (覆盖)
//! use astrea::middleware::*;
//!
//! pub fn middleware() -> Middleware {
//!     Middleware::override_parent()
//!         .wrap(|router| {
//!             // Only rate limiting, no auth
//!             router.layer(tower::limit::ConcurrencyLimitLayer::new(100))
//!         })
//! }
//! ```

// ============================================================================
// MiddlewareMode
// ============================================================================
// 中间件模式
// ============================================================================

/// How child middleware relates to parent middleware
///
/// / 子中间件与父中间件的关系
///
/// This determines whether a middleware scope **inherits** parent middleware
/// (stacking/叠加) or **replaces** it (override/覆盖).
///
/// 决定中间件作用域是**继承**父中间件（叠加）还是**替换**它（覆盖）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MiddlewareMode {
    /// Inherit parent middleware and add these on top (default — stacking/叠加)
    ///
    /// / 继承父中间件并在其上叠加当前中间件（默认 — 叠加）
    ///
    /// Execution order: parent middleware → this middleware → handler
    ///
    /// 执行顺序：父中间件 → 当前中间件 → 处理函数
    #[default]
    Extend,

    /// Discard parent middleware, only apply this scope's middleware (override/覆盖)
    ///
    /// / 丢弃父中间件，仅应用当前作用域的中间件（覆盖）
    ///
    /// Use when you need completely different middleware for certain routes,
    /// such as public endpoints that should skip authentication.
    ///
    /// 当某些路由需要完全不同的中间件时使用，
    /// 例如应跳过认证的公开端点。
    Override,
}

// ============================================================================
// Middleware configuration
// ============================================================================
// 中间件配置
// ============================================================================

/// Middleware configuration returned by `_middleware.rs` files
///
/// / `_middleware.rs` 文件返回的中间件配置
///
/// Each `_middleware.rs` file should export a `pub fn middleware() -> Middleware`
/// function that returns this configuration.
///
/// 每个 `_middleware.rs` 文件应导出 `pub fn middleware() -> Middleware` 函数。
///
/// # Example
///
/// # 示例
///
/// ```rust,ignore
/// use astrea::middleware::*;
///
/// pub fn middleware() -> Middleware {
///     Middleware::new()
///         .wrap(|router| {
///             router.layer(tower_http::cors::CorsLayer::permissive())
///         })
/// }
/// ```
pub struct Middleware<S = ()> {
    /// How this middleware interacts with parent middleware
    /// / 此中间件与父中间件的交互方式
    pub mode: MiddlewareMode,

    /// Function that wraps a Router with middleware layers
    /// / 将中间件层应用到路由器的函数
    wrapper: Option<Box<dyn FnOnce(axum::Router<S>) -> axum::Router<S>>>,
}

impl<S> Default for Middleware<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Middleware<S> {
    /// Create a new middleware configuration with Extend mode (叠加)
    ///
    /// / 创建一个新的中间件配置，默认叠加模式
    ///
    /// # Example
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// use astrea::middleware::*;
    ///
    /// pub fn middleware() -> Middleware {
    ///     Middleware::new()
    ///         .wrap(|router| router.layer(my_layer))
    /// }
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            mode: MiddlewareMode::Extend,
            wrapper: None,
        }
    }

    /// Create a middleware configuration that overrides parent middleware (覆盖)
    ///
    /// / 创建一个覆盖父中间件的配置
    ///
    /// Routes in this scope will NOT inherit any parent middleware.
    /// Only middleware defined in this `_middleware.rs` will apply.
    ///
    /// 此作用域中的路由将不会继承任何父中间件。
    /// 仅此 `_middleware.rs` 中定义的中间件会生效。
    ///
    /// # Example
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// use astrea::middleware::*;
    ///
    /// pub fn middleware() -> Middleware {
    ///     Middleware::override_parent()
    ///         .wrap(|router| router.layer(public_only_layer))
    /// }
    /// ```
    #[must_use]
    pub fn override_parent() -> Self {
        Self {
            mode: MiddlewareMode::Override,
            wrapper: None,
        }
    }

    /// Set the middleware mode
    ///
    /// / 设置中间件模式
    #[must_use]
    pub fn mode(mut self, mode: MiddlewareMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set the wrapper function that applies middleware layers to a Router
    ///
    /// / 设置将中间件层应用到路由器的包装函数
    ///
    /// The closure receives an `axum::Router` and should return it with
    /// middleware layers applied via `.layer()`.
    ///
    /// 闭包接收 `axum::Router`，应通过 `.layer()` 应用中间件层后返回。
    ///
    /// # Example
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// Middleware::new()
    ///     .wrap(|router| {
    ///         router
    ///             .layer(tower_http::cors::CorsLayer::permissive())
    ///             .layer(tower_http::trace::TraceLayer::new_for_http())
    ///     })
    /// ```
    #[must_use]
    pub fn wrap<F>(mut self, f: F) -> Self
    where
        F: FnOnce(axum::Router<S>) -> axum::Router<S> + 'static,
    {
        self.wrapper = Some(Box::new(f));
        self
    }

    /// Apply this middleware to a router (consumed)
    ///
    /// / 将此中间件应用到路由器（消耗此配置）
    ///
    /// This is called by the generated `create_router()` code.
    /// You typically don't need to call it directly.
    ///
    /// 由生成的 `create_router()` 代码调用。通常不需要直接调用。
    pub fn apply(self, router: axum::Router<S>) -> axum::Router<S> {
        match self.wrapper {
            Some(f) => f(router),
            None => router,
        }
    }
}

impl<S> std::fmt::Debug for Middleware<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Middleware")
            .field("mode", &self.mode)
            .field("has_wrapper", &self.wrapper.is_some())
            .finish()
    }
}
