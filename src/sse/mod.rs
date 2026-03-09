//! Server-Sent Events (SSE) support
//!
//! / Server-Sent Events (SSE) 支持
//!
//! This module provides SSE types for real-time unidirectional server-to-client
//! event streaming.
//!
//! 此模块提供用于实时单向服务端到客户端事件流的 SSE 类型。
//!
//! # Usage
//!
//! # 用法
//!
//! ```rust,ignore
//! use astrea::prelude::*;
//! use astrea::sse::{SseSender, SseEvent};
//!
//! // routes/events.get.rs
//! #[route(sse)]
//! async fn handler(event: Event, sender: SseSender) {
//!     let mut id = 0u64;
//!     loop {
//!         if sender.send(SseEvent::new().event("update").data("hello")).await.is_err() {
//!             break; // client disconnected
//!         }
//!         id += 1;
//!         tokio::time::sleep(std::time::Duration::from_secs(1)).await;
//!     }
//! }
//! ```

use std::time::Duration;

use axum::response::sse::Event as AxumSseEvent;

/// SSE event to be sent to the client
///
/// / 要发送给客户端的 SSE 事件
///
/// Builder-style API for constructing SSE events with optional fields.
///
/// 构建器风格的 API，用于构造带有可选字段的 SSE 事件。
///
/// # Example
///
/// # 示例
///
/// ```rust,ignore
/// SseEvent::new()
///     .event("message")
///     .data("Hello, world!")
///     .id("1")
/// ```
#[derive(Debug, Clone, Default)]
pub struct SseEvent {
    data: Option<String>,
    event: Option<String>,
    id: Option<String>,
    retry: Option<Duration>,
    comment: Option<String>,
}

impl SseEvent {
    /// Create a new empty SSE event
    ///
    /// / 创建新的空 SSE 事件
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the event data (plain text)
    ///
    /// / 设置事件数据（纯文本）
    #[must_use]
    pub fn data(mut self, data: impl Into<String>) -> Self {
        self.data = Some(data.into());
        self
    }

    /// Set the event data from a serializable value (JSON)
    ///
    /// / 从可序列化的值设置事件数据（JSON）
    ///
    /// Serializes the value to JSON string. If serialization fails,
    /// stores the error message as data.
    ///
    /// 将值序列化为 JSON 字符串。如果序列化失败，将错误信息存储为数据。
    #[must_use]
    pub fn json<T: serde::Serialize>(mut self, value: &T) -> Self {
        self.data = Some(
            serde_json::to_string(value).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
        );
        self
    }

    /// Set the event type
    ///
    /// / 设置事件类型
    ///
    /// Corresponds to the `event:` field in the SSE protocol.
    /// Clients can listen for specific event types using `EventSource.addEventListener()`.
    ///
    /// 对应 SSE 协议中的 `event:` 字段。
    /// 客户端可以使用 `EventSource.addEventListener()` 监听特定事件类型。
    #[must_use]
    pub fn event(mut self, event: impl Into<String>) -> Self {
        self.event = Some(event.into());
        self
    }

    /// Set the event ID
    ///
    /// / 设置事件 ID
    ///
    /// Used by the client for reconnection (`Last-Event-ID` header).
    ///
    /// 客户端用于断线重连（`Last-Event-ID` 请求头）。
    #[must_use]
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the retry interval suggestion
    ///
    /// / 设置重连间隔建议
    ///
    /// Suggests to the client how long to wait before reconnecting.
    ///
    /// 建议客户端在重连前等待多长时间。
    #[must_use]
    pub fn retry(mut self, duration: Duration) -> Self {
        self.retry = Some(duration);
        self
    }

    /// Set a comment
    ///
    /// / 设置注释
    ///
    /// Comments are prefixed with `:` in the SSE protocol and are
    /// typically used as keep-alive signals.
    ///
    /// 注释在 SSE 协议中以 `:` 为前缀，通常用作保活信号。
    #[must_use]
    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    /// Return the data payload, if set.
    ///
    /// / 返回数据内容（如已设置）。
    pub fn get_data(&self) -> Option<&str> {
        self.data.as_deref()
    }

    /// Return the event type, if set.
    ///
    /// / 返回事件类型（如已设置）。
    pub fn get_event(&self) -> Option<&str> {
        self.event.as_deref()
    }

    /// Return the event ID, if set.
    ///
    /// / 返回事件 ID（如已设置）。
    pub fn get_id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    /// Return the retry duration, if set.
    ///
    /// / 返回重连间隔（如已设置）。
    pub fn get_retry(&self) -> Option<Duration> {
        self.retry
    }

    /// Return the comment, if set.
    ///
    /// / 返回注释（如已设置）。
    pub fn get_comment(&self) -> Option<&str> {
        self.comment.as_deref()
    }

    /// Convert to Axum SSE Event (internal use)
    ///
    /// / 转换为 Axum SSE Event（内部使用）
    pub fn into_axum_event(self) -> AxumSseEvent {
        let mut evt = AxumSseEvent::default();
        if let Some(data) = self.data {
            evt = evt.data(data);
        }
        if let Some(event) = self.event {
            evt = evt.event(event);
        }
        if let Some(id) = self.id {
            evt = evt.id(id);
        }
        if let Some(retry) = self.retry {
            evt = evt.retry(retry);
        }
        if let Some(comment) = self.comment {
            evt = evt.comment(comment);
        }
        evt
    }
}

/// SSE sender for pushing events to the client
///
/// / SSE 发送端，用于向客户端推送事件
///
/// Obtained as the second parameter of an `#[route(sse)]` handler.
/// When `send()` returns `Err`, the client has disconnected.
///
/// 作为 `#[route(sse)]` 处理函数的第二个参数获得。
/// 当 `send()` 返回 `Err` 时，表示客户端已断开。
pub struct SseSender {
    tx: tokio::sync::mpsc::Sender<SseEvent>,
}

impl SseSender {
    /// Create a new SSE sender (internal use)
    ///
    /// / 创建新的 SSE 发送端（内部使用）
    pub fn new(tx: tokio::sync::mpsc::Sender<SseEvent>) -> Self {
        Self { tx }
    }

    /// Send an SSE event to the client
    ///
    /// / 向客户端发送 SSE 事件
    ///
    /// Returns `Err` when the client has disconnected (channel closed).
    ///
    /// 当客户端已断开（通道关闭）时返回 `Err`。
    pub async fn send(&self, event: SseEvent) -> Result<(), tokio::sync::mpsc::error::SendError<SseEvent>> {
        self.tx.send(event).await
    }
}
