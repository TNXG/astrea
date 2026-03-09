//! Tests for astrea::ws — re-exported Message type variants
//!
//! WebSocket/WsSender/WsReceiver wrap Axum's internal WebSocket which requires
//! a real HTTP upgrade and cannot be constructed in unit tests. Their behaviour
//! is therefore verified at compile-time (they implement the expected trait bounds)
//! and through integration with the #[route(ws)] macro (see ws_sse_macro_test.rs).

use astrea::ws::Message;

// ── Message::Text ─────────────────────────────────────────────────────────────

#[test]
fn message_text_from_str() {
    // Message::from(&str) produces a Text variant
    let msg = Message::from("hello");
    assert!(matches!(msg, Message::Text(_)));
}

#[test]
fn message_text_from_string() {
    let msg = Message::from("owned".to_string());
    assert!(matches!(msg, Message::Text(_)));
}

#[test]
fn message_text_content_roundtrip() {
    use axum::extract::ws::Utf8Bytes;
    let msg = Message::Text(Utf8Bytes::from_static("hello, world"));
    if let Message::Text(s) = msg {
        assert_eq!(s.as_str(), "hello, world");
    } else {
        panic!("expected Text");
    }
}

#[test]
fn message_text_unicode() {
    use axum::extract::ws::Utf8Bytes;
    let text: Utf8Bytes = "日本語テスト 🦀".to_string().into();
    let msg = Message::Text(text);
    if let Message::Text(s) = msg {
        assert_eq!(s.as_str(), "日本語テスト 🦀");
    } else {
        panic!("expected Text");
    }
}

// ── Message::Binary ───────────────────────────────────────────────────────────

#[test]
fn message_binary_from_vec() {
    let payload = vec![0u8, 1, 2, 3];
    let msg = Message::from(payload.clone());
    if let Message::Binary(b) = msg {
        assert_eq!(b.as_ref(), payload.as_slice());
    } else {
        panic!("expected Binary");
    }
}

#[test]
fn message_binary_empty() {
    let msg = Message::Binary(bytes::Bytes::new());
    if let Message::Binary(b) = msg {
        assert!(b.is_empty());
    } else {
        panic!("expected Binary");
    }
}

#[test]
fn message_binary_large_payload() {
    let payload: Vec<u8> = (0u8..=255).cycle().take(1024).collect();
    let msg = Message::Binary(bytes::Bytes::from(payload.clone()));
    if let Message::Binary(b) = msg {
        assert_eq!(b.len(), 1024);
        assert_eq!(&b[..4], &payload[..4]);
    } else {
        panic!("expected Binary");
    }
}

// ── Message::Ping / Pong ──────────────────────────────────────────────────────

#[test]
fn message_ping_empty() {
    let msg = Message::Ping(bytes::Bytes::new());
    assert!(matches!(msg, Message::Ping(_)));
}

#[test]
fn message_ping_with_payload() {
    let msg = Message::Ping(bytes::Bytes::from_static(&[1, 2, 3]));
    if let Message::Ping(b) = msg {
        assert_eq!(b.as_ref(), &[1, 2, 3]);
    } else {
        panic!("expected Ping");
    }
}

#[test]
fn message_pong_empty() {
    let msg = Message::Pong(bytes::Bytes::new());
    assert!(matches!(msg, Message::Pong(_)));
}

#[test]
fn message_pong_with_payload() {
    let msg = Message::Pong(bytes::Bytes::from_static(&[42]));
    if let Message::Pong(b) = msg {
        assert_eq!(b.as_ref(), &[42]);
    } else {
        panic!("expected Pong");
    }
}

// ── Message::Close ────────────────────────────────────────────────────────────

#[test]
fn message_close_none() {
    let msg = Message::Close(None);
    assert!(matches!(msg, Message::Close(None)));
}

#[test]
fn message_close_with_frame() {
    use axum::extract::ws::{CloseFrame, close_code};
    let msg = Message::Close(Some(CloseFrame {
        code: close_code::NORMAL,
        reason: "done".to_string().into(),
    }));
    if let Message::Close(Some(frame)) = msg {
        assert_eq!(frame.code, close_code::NORMAL);
        assert_eq!(frame.reason.as_str(), "done");
    } else {
        panic!("expected Close(Some(_))");
    }
}

#[test]
fn message_close_error_code() {
    use axum::extract::ws::{CloseFrame, close_code};
    let msg = Message::Close(Some(CloseFrame {
        code: close_code::ERROR,
        reason: "internal error".to_string().into(),
    }));
    assert!(matches!(msg, Message::Close(Some(_))));
}
