//! Streaming response support
//!
//! / 流式响应支持

use axum::{
    body::Body,
    response::{IntoResponse, Response as AxumResponse},
};

/// Create a streaming response
///
/// / 创建流式响应
///
/// Use this for streaming data, Server-Sent Events, or large files.
///
/// 用于流式数据、服务器发送事件或大文件。
///
/// # Example
///
/// # 示例
///
/// ```rust,ignore
/// use axum::body::Body;
/// use futures_util::stream::once;
///
/// async fn stream_handler() -> Response {
///     let stream = once(async { Ok::<_, std::io::Error>(bytes::Bytes::from("Hello")) });
///     stream(Body::from_stream(stream))
/// }
/// ```
#[must_use]
pub fn stream(body: Body) -> AxumResponse {
    body.into_response()
}
