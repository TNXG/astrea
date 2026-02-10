//! Extract 辅助函数性能测试
//!
//! 测试各种提取函数的性能

use astrea::prelude::*;
use astrea::Event;
use axum::http::{HeaderMap, Method};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::HashMap;
use std::hint::black_box;

/// 创建带有参数的 Event
fn create_event_with_params() -> Event {
    let mut params = HashMap::new();
    params.insert("id".to_string(), "123".to_string());
    params.insert("name".to_string(), "alice".to_string());

    Event::new(
        Method::GET,
        "/users/123".to_string(),
        "/users/123".parse().unwrap(),
        HeaderMap::new(),
        params,
        HashMap::new(),
    )
}

/// 创建带有查询参数的 Event
fn create_event_with_query() -> Event {
    Event::new(
        Method::GET,
        "/search".to_string(),
        "/search?q=rust&lang=en&page=1".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
    )
}

/// 创建带有请求头的 Event
fn create_event_with_headers() -> Event {
    let mut headers = HeaderMap::new();
    headers.insert("authorization", "Bearer token123".parse().unwrap());
    headers.insert("content-type", "application/json".parse().unwrap());
    headers.insert("x-request-id", "abc-123".parse().unwrap());

    Event::new(
        Method::POST,
        "/api/data".to_string(),
        "/api/data".parse().unwrap(),
        headers,
        HashMap::new(),
        HashMap::new(),
    )
}

fn bench_get_param(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_param");

    let event = create_event_with_params();

    group.bench_function("existing_param", |b| {
        b.iter(|| black_box(get_param(black_box(&event), "id")));
    });

    group.bench_function("missing_param", |b| {
        b.iter(|| black_box(get_param(black_box(&event), "missing")));
    });

    // 测试参数缓存效果
    group.bench_function("cached_access", |b| {
        // 第一次访问触发初始化
        let _ = event.params();
        b.iter(|| black_box(get_param(black_box(&event), "id")));
    });

    group.finish();
}

fn bench_get_param_required(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_param_required");

    let event = create_event_with_params();

    group.bench_function("existing_param", |b| {
        b.iter(|| black_box(get_param_required(black_box(&event), "id")));
    });

    group.finish();
}

fn bench_get_query_param(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_query_param");

    let event = create_event_with_query();

    group.bench_function("existing_query_param", |b| {
        b.iter(|| black_box(get_query_param(black_box(&event), "q")));
    });

    group.bench_function("missing_query_param", |b| {
        b.iter(|| black_box(get_query_param(black_box(&event), "missing")));
    });

    // 测试查询字符串解析缓存
    group.bench_function("cached_query_access", |b| {
        let _ = event.query();
        b.iter(|| black_box(get_query_param(black_box(&event), "q")));
    });

    // 测试不同数量的查询参数
    for param_count in &[1, 5, 10, 20] {
        group.throughput(Throughput::Elements(*param_count as u64));

        let query_string: String = (0..*param_count)
            .map(|i| format!("key{i}=value{i}"))
            .collect::<Vec<_>>()
            .join("&");

        let event = Event::new(
            Method::GET,
            "/test".to_string(),
            format!("/test?{query_string}").parse().unwrap(),
            HeaderMap::new(),
            HashMap::new(),
            HashMap::new(),
        );

        group.bench_with_input(
            BenchmarkId::from_parameter(param_count),
            param_count,
            |b, _| b.iter(|| black_box(get_query_param(black_box(&event), "key5"))),
        );
    }

    group.finish();
}

fn bench_get_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_query");

    let event = create_event_with_query();

    group.bench_function("get_all_query", |b| {
        b.iter(|| black_box(get_query(black_box(&event))));
    });

    group.finish();
}

fn bench_get_header(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_header");

    let event = create_event_with_headers();

    group.bench_function("existing_header", |b| {
        b.iter(|| black_box(get_header(black_box(&event), "authorization")));
    });

    group.bench_function("missing_header", |b| {
        b.iter(|| black_box(get_header(black_box(&event), "missing")));
    });

    group.bench_function("case_insensitive", |b| {
        b.iter(|| black_box(get_header(black_box(&event), "Content-Type")));
    });

    group.finish();
}

fn bench_get_headers(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_headers");

    let event = create_event_with_headers();

    group.bench_function("get_all_headers", |b| {
        b.iter(|| black_box(get_headers(black_box(&event))));
    });

    group.finish();
}

fn bench_get_body(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_body");

    let event = Event::new(
        Method::POST,
        "/api/users".to_string(),
        "/api/users".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
    );

    #[derive(serde::Deserialize)]
    #[allow(dead_code)]
    struct CreateUserRequest {
        name: String,
        email: String,
        age: u32,
    }

    let small_body = br#"{"name":"Alice","email":"alice@example.com","age":30}"#;

    group.bench_function("parse_small_json", |b| {
        b.iter(|| black_box(get_body::<CreateUserRequest>(black_box(&event), small_body)));
    });

    // 测试不同大小的 JSON body
    for size in &[10, 100, 1000] {
        let json_body = format!(r#"{{"data":"{}"}}"#, "x".repeat(*size));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                black_box(get_body::<serde_json::Value>(
                    black_box(&event),
                    json_body.as_bytes(),
                ))
            });
        });
    }

    group.finish();
}

fn bench_get_body_text(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_body_text");

    let event = Event::new(
        Method::POST,
        "/api/text".to_string(),
        "/api/text".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
    );

    group.bench_function("short_text", |b| {
        let text = "Hello, World!";
        b.iter(|| black_box(get_body_text(black_box(&event), text.as_bytes())));
    });

    for size in &[100, 1000, 10000] {
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, size| {
            let text = "a".repeat(*size);
            b.iter(|| black_box(get_body_text(black_box(&event), text.as_bytes())));
        });
    }

    group.finish();
}

fn bench_get_body_bytes(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_body_bytes");

    let event = Event::new(
        Method::POST,
        "/api/data".to_string(),
        "/api/data".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
    );

    group.bench_function("get_bytes", |b| {
        let bytes = b"raw data";
        b.iter(|| black_box(get_body_bytes(black_box(&event), bytes)));
    });

    group.finish();
}

fn bench_get_method(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_method");

    let event = Event::new(
        Method::POST,
        "/api/data".to_string(),
        "/api/data".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
    );

    group.bench_function("get_http_method", |b| {
        b.iter(|| black_box(get_method(black_box(&event))));
    });

    group.finish();
}

fn bench_get_path(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_path");

    let event = Event::new(
        Method::GET,
        "/api/v1/users/123/posts".to_string(),
        "/api/v1/users/123/posts".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
    );

    group.bench_function("get_request_path", |b| {
        b.iter(|| black_box(get_path(black_box(&event))));
    });

    group.finish();
}

fn bench_get_uri(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_uri");

    let event = Event::new(
        Method::GET,
        "/search".to_string(),
        "/search?q=rust&lang=en".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
    );

    group.bench_function("get_request_uri", |b| {
        b.iter(|| black_box(get_uri(black_box(&event))));
    });

    group.finish();
}

fn bench_combined_extract(c: &mut Criterion) {
    let mut group = c.benchmark_group("combined_extract");

    // 模拟真实的处理函数中的提取操作
    let mut params = HashMap::new();
    params.insert("id".to_string(), "123".to_string());

    let mut headers = HeaderMap::new();
    headers.insert("authorization", "Bearer token".parse().unwrap());

    let event = Event::new(
        Method::GET,
        "/users/123".to_string(),
        "/users/123?format=json".parse().unwrap(),
        headers,
        params,
        HashMap::new(),
    );

    group.bench_function("extract_multiple_fields", |b| {
        b.iter(|| {
            let _id = get_param(black_box(&event), "id");
            let _format = get_query_param(black_box(&event), "format");
            let _auth = get_header(black_box(&event), "authorization");
            let _path = get_path(black_box(&event));
            let _method = get_method(black_box(&event));
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_get_param,
    bench_get_param_required,
    bench_get_query_param,
    bench_get_query,
    bench_get_header,
    bench_get_headers,
    bench_get_body,
    bench_get_body_text,
    bench_get_body_bytes,
    bench_get_method,
    bench_get_path,
    bench_get_uri,
    bench_combined_extract
);
criterion_main!(benches);
