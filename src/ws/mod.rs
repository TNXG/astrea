//! WebSocket support
//!
//! / WebSocket 支持
//!
//! This module provides WebSocket types for real-time bidirectional communication.
//!
//! 此模块提供用于实时双向通信的 WebSocket 类型。
//!
//! # Usage
//!
//! # 用法
//!
//! ```rust,ignore
//! use astrea::prelude::*;
//! use astrea::ws::{WebSocket, Message};
//!
//! // routes/chat.get.rs
//! #[route(ws)]
//! async fn handler(event: Event, mut socket: WebSocket) {
//!     while let Some(Ok(msg)) = socket.recv().await {
//!         if let Message::Text(text) = msg {
//!             let _ = socket.send(Message::Text(format!("Echo: {text}"))).await;
//!         }
//!     }
//! }
//! ```

use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};

/// Re-export WebSocket message type from Axum
///
/// / 从 Axum 重新导出 WebSocket 消息类型
pub use axum::extract::ws::Message;

/// WebSocket connection
///
/// / WebSocket 连接
///
/// Wraps an Axum `WebSocket` to provide a simpler API for bidirectional
/// communication. Supports receiving and sending messages, splitting into
/// independent sender/receiver halves, and closing the connection.
///
/// 封装 Axum 的 `WebSocket`，提供更简单的双向通信 API。
/// 支持接收和发送消息、拆分为独立的发送端/接收端，以及关闭连接。
pub struct WebSocket {
    inner: axum::extract::ws::WebSocket,
}

impl WebSocket {
    /// Create a new `WebSocket` from an Axum WebSocket
    ///
    /// / 从 Axum WebSocket 创建新的 `WebSocket`
    ///
    /// This is called internally by the `#[route(ws)]` macro.
    ///
    /// 此方法由 `#[route(ws)]` 宏内部调用。
    pub fn new(inner: axum::extract::ws::WebSocket) -> Self {
        Self { inner }
    }

    /// Receive a message from the client
    ///
    /// / 从客户端接收消息
    ///
    /// Returns `None` when the connection is closed.
    ///
    /// 连接关闭时返回 `None`。
    pub async fn recv(&mut self) -> Option<Result<Message, axum::Error>> {
        self.inner.next().await
    }

    /// Send a message to the client
    ///
    /// / 向客户端发送消息
    ///
    /// Returns `Err` if the connection is closed or broken.
    ///
    /// 如果连接已关闭或断开则返回 `Err`。
    pub async fn send(&mut self, msg: Message) -> Result<(), axum::Error> {
        self.inner.send(msg).await
    }

    /// Close the WebSocket connection
    ///
    /// / 关闭 WebSocket 连接
    pub async fn close(mut self) -> Result<(), axum::Error> {
        self.inner.close().await
    }

    /// Split into independent sender and receiver halves
    ///
    /// / 拆分为独立的发送端和接收端
    ///
    /// Useful when you need to send and receive concurrently from different tasks.
    ///
    /// 当需要在不同任务中并发发送和接收时很有用。
    ///
    /// # Example
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// let (mut sender, mut receiver) = socket.split();
    ///
    /// let send_task = tokio::spawn(async move {
    ///     sender.send(Message::Text("hello".into())).await.ok();
    /// });
    ///
    /// while let Some(Ok(msg)) = receiver.recv().await {
    ///     // handle messages
    /// }
    /// ```
    pub fn split(self) -> (WsSender, WsReceiver) {
        let (sink, stream) = self.inner.split();
        (WsSender { inner: sink }, WsReceiver { inner: stream })
    }
}

/// WebSocket sender half
///
/// / WebSocket 发送端
///
/// Obtained by calling [`WebSocket::split`]. Can be moved to a separate task
/// for concurrent send/receive operations.
///
/// 通过调用 [`WebSocket::split`] 获得。可移动到独立任务中实现并发收发。
pub struct WsSender {
    inner: SplitSink<axum::extract::ws::WebSocket, Message>,
}

impl WsSender {
    /// Send a message to the client
    ///
    /// / 向客户端发送消息
    pub async fn send(&mut self, msg: Message) -> Result<(), axum::Error> {
        self.inner.send(msg).await
    }

    /// Close the sender
    ///
    /// / 关闭发送端
    pub async fn close(&mut self) -> Result<(), axum::Error> {
        self.inner.close().await
    }
}

/// WebSocket receiver half
///
/// / WebSocket 接收端
///
/// Obtained by calling [`WebSocket::split`]. Can be moved to a separate task
/// for concurrent send/receive operations.
///
/// 通过调用 [`WebSocket::split`] 获得。可移动到独立任务中实现并发收发。
pub struct WsReceiver {
    inner: SplitStream<axum::extract::ws::WebSocket>,
}

impl WsReceiver {
    /// Receive a message from the client
    ///
    /// / 从客户端接收消息
    ///
    /// Returns `None` when the connection is closed.
    ///
    /// 连接关闭时返回 `None`。
    pub async fn recv(&mut self) -> Option<Result<Message, axum::Error>> {
        self.inner.next().await
    }
}
