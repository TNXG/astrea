//! Tests for astrea::sse — SseEvent builder and SseSender channel behaviour

use astrea::sse::{SseEvent, SseSender};
use std::time::Duration;

// ── SseEvent::new / default ───────────────────────────────────────────────────

#[test]
fn sse_event_new_all_fields_none() {
    let evt = SseEvent::new();
    assert!(evt.get_data().is_none());
    assert!(evt.get_event().is_none());
    assert!(evt.get_id().is_none());
    assert!(evt.get_retry().is_none());
    assert!(evt.get_comment().is_none());
}

#[test]
fn sse_event_default_same_as_new() {
    let a = SseEvent::new();
    let b = SseEvent::default();
    assert_eq!(format!("{a:?}"), format!("{b:?}"));
}

// ── builder methods ───────────────────────────────────────────────────────────

#[test]
fn sse_event_data_str() {
    let evt = SseEvent::new().data("hello");
    assert_eq!(evt.get_data(), Some("hello"));
}

#[test]
fn sse_event_data_owned_string() {
    let evt = SseEvent::new().data("owned".to_string());
    assert_eq!(evt.get_data(), Some("owned"));
}

#[test]
fn sse_event_event_type() {
    let evt = SseEvent::new().event("update");
    assert_eq!(evt.get_event(), Some("update"));
}

#[test]
fn sse_event_id() {
    let evt = SseEvent::new().id("42");
    assert_eq!(evt.get_id(), Some("42"));
}

#[test]
fn sse_event_retry() {
    let evt = SseEvent::new().retry(Duration::from_secs(5));
    assert_eq!(evt.get_retry(), Some(Duration::from_secs(5)));
}

#[test]
fn sse_event_comment() {
    let evt = SseEvent::new().comment("keep-alive");
    assert_eq!(evt.get_comment(), Some("keep-alive"));
}

#[test]
fn sse_event_builder_full_chain() {
    let evt = SseEvent::new()
        .data("payload")
        .event("msg")
        .id("1")
        .retry(Duration::from_millis(500))
        .comment("ping");
    assert_eq!(evt.get_data(), Some("payload"));
    assert_eq!(evt.get_event(), Some("msg"));
    assert_eq!(evt.get_id(), Some("1"));
    assert_eq!(evt.get_retry(), Some(Duration::from_millis(500)));
    assert_eq!(evt.get_comment(), Some("ping"));
}

// ── SseEvent::json ────────────────────────────────────────────────────────────

#[test]
fn sse_event_json_plain_struct() {
    use serde::Serialize;
    #[derive(Serialize)]
    struct Payload {
        value: u32,
    }
    let evt = SseEvent::new().json(&Payload { value: 99 });
    let data = evt.get_data().unwrap();
    assert!(data.contains("99"), "expected JSON with 99, got: {data}");
}

#[test]
fn sse_event_json_nested_struct() {
    use serde::Serialize;
    #[derive(Serialize)]
    struct Inner {
        x: i32,
    }
    #[derive(Serialize)]
    struct Outer {
        inner: Inner,
        name: String,
    }
    let evt = SseEvent::new().json(&Outer {
        inner: Inner { x: -7 },
        name: "test".into(),
    });
    let data = evt.get_data().unwrap();
    assert!(data.contains("-7"), "missing field: {data}");
    assert!(data.contains("test"), "missing field: {data}");
}

#[test]
fn sse_event_json_serialize_error_does_not_panic() {
    use std::collections::BTreeMap;
    // BTreeMap<u32, _> fails JSON serialization (keys must be strings)
    let mut bad: BTreeMap<u32, &str> = BTreeMap::new();
    bad.insert(1, "a");
    let evt = SseEvent::new().json(&bad);
    let data = evt.get_data().unwrap();
    assert!(!data.is_empty(), "data should contain fallback error message");
}

// ── SseEvent::into_axum_event ────────────────────────────────────────────────

#[test]
fn sse_event_into_axum_event_empty() {
    // Axum's Event fields are opaque — verify no panic
    let _ = SseEvent::new().into_axum_event();
}

#[test]
fn sse_event_into_axum_event_all_fields() {
    let _ = SseEvent::new()
        .data("hello")
        .event("update")
        .id("1")
        .retry(Duration::from_secs(1))
        .comment("ping")
        .into_axum_event();
}

// ── SseSender ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn sse_sender_delivers_event() {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<SseEvent>(4);
    let sender = SseSender::new(tx);

    sender
        .send(SseEvent::new().data("hello").event("greet"))
        .await
        .expect("send should succeed");

    let received = rx.recv().await.expect("should receive");
    assert_eq!(received.get_data(), Some("hello"));
    assert_eq!(received.get_event(), Some("greet"));
}

#[tokio::test]
async fn sse_sender_err_when_receiver_dropped() {
    let (tx, rx) = tokio::sync::mpsc::channel::<SseEvent>(1);
    let sender = SseSender::new(tx);
    drop(rx);

    let result = sender.send(SseEvent::new().data("orphan")).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn sse_sender_preserves_order() {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<SseEvent>(8);
    let sender = SseSender::new(tx);

    for i in 0..5u32 {
        sender
            .send(SseEvent::new().data(i.to_string()))
            .await
            .expect("send");
    }
    drop(sender);

    let mut count = 0u32;
    while let Some(evt) = rx.recv().await {
        assert_eq!(evt.get_data(), Some(count.to_string().as_str()));
        count += 1;
    }
    assert_eq!(count, 5);
}

#[tokio::test]
async fn sse_sender_two_independent_senders() {
    use tokio::sync::mpsc;
    let (tx, mut rx) = mpsc::channel::<SseEvent>(8);
    let s1 = SseSender::new(tx.clone());
    let s2 = SseSender::new(tx);

    s1.send(SseEvent::new().data("from-s1")).await.unwrap();
    s2.send(SseEvent::new().data("from-s2")).await.unwrap();
    drop(s1);
    drop(s2);

    let mut results = Vec::new();
    while let Some(e) = rx.recv().await {
        results.push(e.get_data().unwrap().to_string());
    }
    assert!(results.contains(&"from-s1".to_string()));
    assert!(results.contains(&"from-s2".to_string()));
}
