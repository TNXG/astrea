//! Event type and related functionality
//!
//! / Event 类型及相关功能
//!
//! The `Event` type encapsulates all request information and provides
//! lazy, cached access to parsed data.
//!
//! `Event` 类型封装了所有请求信息，并提供延迟缓存的数据访问。
//!
//! # Overview
//!
//! # 概述
//!
//! The `Event` type is the main way to access request data in Astrea handlers.
//! It provides:
//!
//! `Event` 类型是在 Astrea 处理函数中访问请求数据的主要方式。它提供：
//!
//! - **Lazy evaluation** - Query parameters are only parsed when accessed
//!   **延迟求值** - 查询参数仅在访问时解析
//! - **Cached access** - Parsed data is cached for efficient repeated access
//!   **缓存访问** - 解析后的数据被缓存，可高效重复访问
//! - **Type-safe state** - Application state with type erasure
//!   **类型安全的状态** - 带类型擦除的应用状态
//! - **Body parsing** - Convenience methods for parsing request bodies
//!   **请求体解析** - 解析请求体的便捷方法
//!
//! # Example
//!
//! # 示例
//!
//! ```rust,ignore
//! use astrea::prelude::*;
//!
//! #[route]
//! async fn handler(event: Event, bytes: Bytes) -> Result<Response> {
//!     // Access path parameters
//!     let user_id = get_param(&event, "id");
//!
//!     // Access query parameters (lazy parsed)
//!     let search = get_query_param(&event, "q");
//!
//!     // Get request method and path
//!     let method = event.method();
//!     let path = event.path();
//!
//!     json(json!({ "user_id": user_id }))
//! }
//! ```

use crate::error::{Result, RouteError};
use axum::http::{HeaderMap, Method, Uri};
use once_cell::sync::OnceCell;
use std::{collections::HashMap, sync::Arc};

/// Inner event data shared via Arc
///
/// / 通过 Arc 共享的内部事件数据
///
/// This struct is separated to allow efficient cloning of `Event` while
/// sharing the parsed data.
///
/// 此结构体被分离以便在共享解析数据时高效克隆 `Event`。
#[derive(Debug)]
pub struct EventInner {
    /// HTTP method
    /// / HTTP 方法
    pub method: Method,
    /// Request path
    /// / 请求路径
    pub path: String,
    /// Original URI for query parsing
    /// / 用于查询解析的原始 URI
    pub raw_uri: Uri,
    /// Request headers
    /// / 请求头
    pub headers: HeaderMap,
    /// Lazy cached path parameters
    /// / 延迟缓存的路径参数
    pub params: OnceCell<HashMap<String, String>>,
    /// Lazy cached query parameters
    /// / 延迟缓存的查询参数
    pub query: OnceCell<HashMap<String, String>>,
}

/// Request event containing all request information
///
/// / 包含所有请求信息的请求事件
///
/// The `Event` type provides lazy, cached access to request data through
/// helper functions, avoiding the need for complex Axum extractor signatures.
///
/// `Event` 类型通过辅助函数提供延迟缓存的数据访问，无需复杂的 Axum 提取器签名。
///
/// # Cloning
///
/// # 克隆
///
/// `Event` is cheap to clone because it uses `Arc` internally.
/// Clones share the same inner data.
///
/// `Event` 克隆成本低，因为内部使用 `Arc`。克隆共享相同的内部数据。
///
/// # Thread Safety
///
/// # 线程安全
///
/// `Event` can be safely shared between threads using `Arc`.
///
/// `Event` 可以使用 `Arc` 在线程间安全共享。
#[derive(Debug, Clone)]
pub struct Event {
    /// Inner event data
    /// / 内部事件数据
    pub inner: Arc<EventInner>,
    /// Application state (type-erased, stored as Arc<dyn Any + Send + Sync>)
    /// / 应用状态（类型擦除，存储为 Arc<dyn Any + Send + Sync>）
    pub state: Option<std::sync::Arc<dyn std::any::Any + Send + Sync>>,
}

impl Event {
    /// Create a new Event with manual data
    ///
    /// / 使用手动数据创建新 Event
    ///
    /// This function is typically called by the `#[route]` macro generated wrapper code.
    ///
    /// 此函数通常由 `#[route]` 宏生成的包装代码调用。
    ///
    /// # Parameters
    ///
    /// # 参数
    ///
    /// - `method` - HTTP method / HTTP 方法
    /// - `path` - Request path / 请求路径
    /// - `raw_uri` - Original URI (for query parsing) / 原始 URI（用于查询解析）
    /// - `headers` - Request headers / 请求头
    /// - `params` - Path parameters / 路径参数
    /// - `query` - Query parameters / 查询参数
    pub fn new(
        method: Method,
        path: String,
        raw_uri: Uri,
        headers: HeaderMap,
        params: HashMap<String, String>,
        query: HashMap<String, String>,
    ) -> Self {
        let inner = EventInner {
            method,
            path,
            raw_uri,
            headers,
            params: OnceCell::from(params),
            query: OnceCell::from(query),
        };

        Self {
            inner: Arc::new(inner),
            state: None,
        }
    }

    /// Get the HTTP method
    ///
    /// / 获取 HTTP 方法
    ///
    /// # Example
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// if event.method() == Method::POST {
    ///     // Handle POST request
    /// }
    /// ```
    #[must_use]
    pub fn method(&self) -> &Method {
        &self.inner.method
    }

    /// Get the request path
    ///
    /// / 获取请求路径
    ///
    /// Returns the path part of the URI (without query string).
    ///
    /// 返回 URI 的路径部分（不含查询字符串）。
    ///
    /// # Example
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// let path = event.path(); // "/users/123"
    /// ```
    #[must_use]
    pub fn path(&self) -> &str {
        &self.inner.path
    }

    /// Get the original URI
    ///
    /// / 获取原始 URI
    ///
    /// Returns the complete URI including path and query string.
    ///
    /// 返回完整的 URI，包括路径和查询字符串。
    #[must_use]
    pub fn uri(&self) -> &Uri {
        &self.inner.raw_uri
    }

    /// Get request headers
    ///
    /// / 获取请求头
    ///
    /// Returns a reference to the complete header map.
    ///
    /// 返回完整请求头的引用。
    #[must_use]
    pub fn headers(&self) -> &HeaderMap {
        &self.inner.headers
    }

    /// Get path parameters (lazy cached)
    ///
    /// / 获取路径参数（延迟缓存）
    ///
    /// Returns a reference to the path parameters map.
    ///
    /// 返回路径参数映射的引用。
    ///
    /// Note: For more convenient access, use [`get_param`](crate::extract::get_param)
    /// or [`get_param_required`](crate::extract::get_param_required).
    ///
    /// 注意：为了更方便的访问，请使用 [`get_param`](crate::extract::get_param)
    /// 或 [`get_param_required`](crate::extract::get_param_required)。
    #[must_use]
    pub fn params(&self) -> &HashMap<String, String> {
        self.inner.params.get_or_init(HashMap::new)
    }

    /// Get query parameters (lazy cached)
    ///
    /// / 获取查询参数（延迟缓存）
    ///
    /// Query parameters are parsed from the URI on first access and cached.
    ///
    /// 查询参数在首次访问时从 URI 解析并缓存。
    ///
    /// Note: For more convenient access, use [`get_query`](crate::extract::get_query)
    /// or [`get_query_param`](crate::extract::get_query_param).
    ///
    /// 注意：为了更方便的访问，请使用 [`get_query`](crate::extract::get_query)
    /// 或 [`get_query_param`](crate::extract::get_query_param)。
    #[must_use]
    pub fn query(&self) -> &HashMap<String, String> {
        self.inner.query.get_or_init(|| {
            self.inner
                .raw_uri
                .query()
                .map(|q| serde_urlencoded::from_str(q).unwrap_or_else(|_| HashMap::new()))
                .unwrap_or_default()
        })
    }

    /// Get a value from the application state
    ///
    /// / 从应用状态获取值
    ///
    /// # Type Parameters
    ///
    /// # 类型参数
    ///
    /// - `T` - The type to retrieve (must be `Clone + Send + Sync + 'static`)
    ///   要检索的类型（必须是 `Clone + Send + Sync + 'static`）
    ///
    /// # Example
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// use astrea::prelude::*;
    ///
    /// struct DatabasePool {
    ///     // ...
    /// }
    ///
    /// #[route]
    /// async fn handler(event: Event) -> Result<Response> {
    ///     let pool = event.state::<DatabasePool>()
    ///         .ok_or_else(|| RouteError::internal("Database pool not found"))?;
    ///     // Use pool...
    /// }
    /// ```
    #[must_use]
    pub fn state<T: Clone + Send + Sync + 'static>(&self) -> Option<T> {
        self.state
            .as_ref()
            .and_then(|s| s.downcast_ref::<T>())
            .cloned()
    }

    /// Parse JSON body from bytes
    ///
    /// / 从字节解析 JSON 请求体
    ///
    /// This is a convenience method typically used by the generated wrapper code.
    ///
    /// 这是一个便捷方法，通常由生成的包装代码使用。
    ///
    /// # Errors
    ///
    /// # 错误
    ///
    /// Returns `RouteError::BadRequest` if the JSON is invalid.
    ///
    /// 如果 JSON 无效，返回 `RouteError::BadRequest`。
    ///
    /// # Example
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// #[derive(Deserialize)]
    /// struct CreateUserRequest {
    ///     name: String,
    ///     email: String,
    /// }
    ///
    /// #[route]
    /// async fn handler(event: Event, bytes: Bytes) -> Result<Response> {
    ///     let body: CreateUserRequest = event.parse_json(&bytes)?;
    ///     json(json!({ "message": format!("User {} created", body.name) }))
    /// }
    /// ```
    pub fn parse_json<T: serde::de::DeserializeOwned>(&self, bytes: &[u8]) -> Result<T> {
        serde_json::from_slice(bytes)
            .map_err(|e| RouteError::bad_request(format!("Invalid JSON: {e}")))
    }

    /// Parse form data from bytes
    ///
    /// / 从字节解析表单数据
    ///
    /// Parses URL-encoded form data from the request body.
    ///
    /// 从请求体解析 URL 编码的表单数据。
    ///
    /// # Errors
    ///
    /// # 错误
    ///
    /// Returns `RouteError::BadRequest` if the form data is invalid.
    ///
    /// 如果表单数据无效，返回 `RouteError::BadRequest`。
    ///
    /// # Example
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// #[derive(Deserialize)]
    /// struct LoginForm {
    ///     username: String,
    ///     password: String,
    /// }
    ///
    /// #[route]
    /// async fn handler(event: Event, bytes: Bytes) -> Result<Response> {
    ///     let form: LoginForm = event.parse_form(&bytes)?;
    ///     // Process login...
    /// }
    /// ```
    pub fn parse_form<T: serde::de::DeserializeOwned>(&self, bytes: &[u8]) -> Result<T> {
        let text = self.parse_text(bytes)?;
        serde_urlencoded::from_str(&text)
            .map_err(|e| RouteError::bad_request(format!("Invalid form data: {e}")))
    }

    /// Parse text body from bytes
    ///
    /// / 从字节解析文本请求体
    ///
    /// Validates that the body is valid UTF-8.
    ///
    /// 验证请求体是有效的 UTF-8。
    ///
    /// # Errors
    ///
    /// # 错误
    ///
    /// Returns `RouteError::BadRequest` if the body is not valid UTF-8.
    ///
    /// 如果请求体不是有效的 UTF-8，返回 `RouteError::BadRequest`。
    pub fn parse_text(&self, bytes: &[u8]) -> Result<String> {
        std::str::from_utf8(bytes)
            .map(std::string::ToString::to_string)
            .map_err(|e| RouteError::bad_request(format!("Invalid UTF-8: {e}")))
    }
}
