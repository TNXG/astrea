//! Event type and related functionality
//!
//! The Event type encapsulates all request information and provides
//! lazy, cached access to parsed data.

use crate::error::{Result, RouteError};
use axum::http::{HeaderMap, Method, Uri};
use once_cell::sync::OnceCell;
use std::{collections::HashMap, sync::Arc};

/// Inner event data shared via Arc
#[derive(Debug)]
pub struct EventInner {
    /// HTTP method
    pub method: Method,
    /// Request path
    pub path: String,
    /// Original URI for query parsing
    pub raw_uri: Uri,
    /// Request headers
    pub headers: HeaderMap,
    /// Lazy cached path parameters
    pub params: OnceCell<HashMap<String, String>>,
    /// Lazy cached query parameters
    pub query: OnceCell<HashMap<String, String>>,
}

/// Request event containing all request information
///
/// The Event type provides lazy, cached access to request data through
/// helper functions, avoiding the need for complex Axum extractor signatures.
#[derive(Debug, Clone)]
pub struct Event {
    /// Inner event data
    pub inner: Arc<EventInner>,
    /// Application state (type-erased, stored as Arc<dyn Any + Send + Sync>)
    pub state: Option<std::sync::Arc<dyn std::any::Any + Send + Sync>>,
}

impl Event {
    /// Create a new Event with manual data
    ///
    /// Called by the `#[route]` macro generated wrapper code.
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
    pub fn method(&self) -> &Method {
        &self.inner.method
    }

    /// Get the request path
    pub fn path(&self) -> &str {
        &self.inner.path
    }

    /// Get the original URI
    pub fn uri(&self) -> &Uri {
        &self.inner.raw_uri
    }

    /// Get request headers
    pub fn headers(&self) -> &HeaderMap {
        &self.inner.headers
    }

    /// Get path parameters (lazy cached)
    pub fn params(&self) -> &HashMap<String, String> {
        self.inner.params.get_or_init(|| HashMap::new())
    }

    /// Get query parameters (lazy cached)
    pub fn query(&self) -> &HashMap<String, String> {
        self.inner.query.get_or_init(|| {
            self.inner
                .raw_uri
                .query()
                .map(|q| {
                    serde_urlencoded::from_str(q)
                        .unwrap_or_else(|_| HashMap::new())
                })
                .unwrap_or_default()
        })
    }

    /// Get a value from the application state
    pub fn state<T: Clone + Send + Sync + 'static>(&self) -> Option<T> {
        self.state
            .as_ref()
            .and_then(|s| s.downcast_ref::<T>())
            .cloned()
    }

    /// Parse JSON body from bytes
    ///
    /// This is a convenience method for the generated wrapper code.
    pub fn parse_json<T: serde::de::DeserializeOwned>(&self, bytes: &[u8]) -> Result<T> {
        serde_json::from_slice(bytes)
            .map_err(|e| RouteError::bad_request(format!("Invalid JSON: {}", e)))
    }

    /// Parse form data from bytes
    pub fn parse_form<T: serde::de::DeserializeOwned>(&self, bytes: &[u8]) -> Result<T> {
        let text = self.parse_text(bytes)?;
        serde_urlencoded::from_str(&text)
            .map_err(|e| RouteError::bad_request(format!("Invalid form data: {}", e)))
    }

    /// Parse text body from bytes
    pub fn parse_text(&self, bytes: &[u8]) -> Result<String> {
        std::str::from_utf8(bytes)
            .map(|s| s.to_string())
            .map_err(|e| RouteError::bad_request(format!("Invalid UTF-8: {}", e)))
    }
}
