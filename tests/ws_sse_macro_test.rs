//! Compile tests for #[route(ws)] and #[route(sse)] macro expansion.
//!
//! Each handler lives in its own module, mirroring how Astrea includes
//! route files: `include!("path/to/file.rs")` inside a `mod { }` block.
//! This avoids `__ROUTE_TYPE` constant name collisions.
//!
//! Note: `#[route(ws/sse)]` replaces the entire function signature, so imports
//! that appear only in parameter-type position are unnecessary after expansion.

// ── #[route(ws)] ──────────────────────────────────────────────────────────────

mod ws_echo {
    use astrea::prelude::*;

    /// Minimal WebSocket handler — receives one message and echoes it back.
    #[route(ws)]
    pub async fn handler(event: Event, mut socket: astrea::ws::WebSocket) {
        let _ = event.path();
        if let Some(Ok(msg)) = socket.recv().await {
            let _ = socket.send(msg).await;
        }
    }
}

mod ws_split {
    use astrea::prelude::*;

    /// WebSocket handler using split() for separate tx/rx.
    #[route(ws)]
    pub async fn handler(event: Event, socket: astrea::ws::WebSocket) {
        let _ = event.path();
        let (mut tx, mut rx) = socket.split();
        if let Some(Ok(msg)) = rx.recv().await {
            let _ = tx.send(msg).await;
        }
    }
}

mod ws_send_text {
    use astrea::prelude::*;
    use astrea::ws::Message;

    /// WebSocket handler that sends a greeting then closes.
    #[route(ws)]
    pub async fn handler(_event: Event, mut socket: astrea::ws::WebSocket) {
        let _ = socket.send(Message::from("hello from server")).await;
        let _ = socket.close().await;
    }
}

// ── #[route(sse)] ─────────────────────────────────────────────────────────────

mod sse_once {
    use astrea::prelude::*;
    use astrea::sse::SseEvent;

    /// SSE handler that pushes a single event.
    #[route(sse)]
    pub async fn handler(event: Event, sender: astrea::sse::SseSender) {
        let _ = event.path();
        let _ = sender
            .send(SseEvent::new().event("hello").data("world"))
            .await;
    }
}

mod sse_counter {
    use astrea::prelude::*;
    use astrea::sse::SseEvent;

    /// SSE handler that pushes a counter stream.
    #[route(sse)]
    pub async fn handler(_event: Event, sender: astrea::sse::SseSender) {
        for i in 0..3u32 {
            if sender
                .send(SseEvent::new().event("tick").data(i.to_string()))
                .await
                .is_err()
            {
                break;
            }
        }
    }
}

mod sse_json {
    use astrea::prelude::*;
    use astrea::sse::SseEvent;
    use serde::Serialize;

    #[derive(Serialize)]
    struct Status {
        code: u32,
        msg: &'static str,
    }

    /// SSE handler that pushes structured JSON.
    #[route(sse)]
    pub async fn handler(_event: Event, sender: astrea::sse::SseSender) {
        let _ = sender
            .send(SseEvent::new().json(&Status { code: 200, msg: "ok" }))
            .await;
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    #[test]
    fn ws_route_type_constant_is_ws() {
        assert_eq!(super::ws_echo::__ROUTE_TYPE, "WS");
        assert_eq!(super::ws_split::__ROUTE_TYPE, "WS");
        assert_eq!(super::ws_send_text::__ROUTE_TYPE, "WS");
    }

    #[test]
    fn sse_route_type_constant_is_sse() {
        assert_eq!(super::sse_once::__ROUTE_TYPE, "SSE");
        assert_eq!(super::sse_counter::__ROUTE_TYPE, "SSE");
        assert_eq!(super::sse_json::__ROUTE_TYPE, "SSE");
    }

    #[test]
    fn ws_handlers_are_callable_functions() {
        fn accept_fn<F>(_: F) {}
        accept_fn(super::ws_echo::handler::<()>);
        accept_fn(super::ws_split::handler::<()>);
        accept_fn(super::ws_send_text::handler::<()>);
    }

    #[test]
    fn sse_handlers_are_callable_functions() {
        fn accept_fn<F>(_: F) {}
        accept_fn(super::sse_once::handler::<()>);
        accept_fn(super::sse_counter::handler::<()>);
        accept_fn(super::sse_json::handler::<()>);
    }
}
