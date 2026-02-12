//! Inner event data structure
//!
//! / 内部事件数据结构

use axum::http::{HeaderMap, Method, Uri};
use once_cell::sync::OnceCell;
use std::collections::HashMap;

use crate::error::{Result, RouteError};

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

impl EventInner {
    /// Create a new EventInner
    pub fn new(
        method: Method,
        path: String,
        raw_uri: Uri,
        headers: HeaderMap,
        params: HashMap<String, String>,
        query: HashMap<String, String>,
    ) -> Self {
        Self {
            method,
            path,
            raw_uri,
            headers,
            params: OnceCell::from(params),
            query: OnceCell::from(query),
        }
    }

    /// Get the HTTP method
    pub fn method(&self) -> &Method {
        &self.method
    }

    /// Get the request path
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Get the original URI
    pub fn uri(&self) -> &Uri {
        &self.raw_uri
    }

    /// Get request headers
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Get path parameters (lazy cached)
    pub fn params(&self) -> &HashMap<String, String> {
        self.params.get_or_init(HashMap::new)
    }

    /// Get query parameters (lazy cached)
    pub fn query(&self) -> &HashMap<String, String> {
        self.query.get_or_init(|| {
            self.raw_uri
                .query()
                .map(|q| serde_urlencoded::from_str(q).unwrap_or_else(|_| HashMap::new()))
                .unwrap_or_default()
        })
    }

    /// Parse JSON body from bytes
    pub fn parse_json<T: serde::de::DeserializeOwned>(&self, bytes: &[u8]) -> Result<T> {
        serde_json::from_slice(bytes)
            .map_err(|e| RouteError::bad_request(format!("Invalid JSON: {e}")))
    }

    /// Parse form data from bytes
    pub fn parse_form<T: serde::de::DeserializeOwned>(&self, bytes: &[u8]) -> Result<T> {
        let text = self.parse_text(bytes)?;
        serde_urlencoded::from_str(&text)
            .map_err(|e| RouteError::bad_request(format!("Invalid form data: {e}")))
    }

    /// Parse text body from bytes
    pub fn parse_text(&self, bytes: &[u8]) -> Result<String> {
        std::str::from_utf8(bytes)
            .map(std::string::ToString::to_string)
            .map_err(|e| RouteError::bad_request(format!("Invalid UTF-8: {e}")))
    }
}
