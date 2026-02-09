//! Response 性能测试
//!
//! 测试响应构建、序列化等操作的性能

use astrea::response::{json, text, html, redirect, no_content, bytes};
use axum::http::StatusCode;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct TestData {
    id: u32,
    name: String,
    email: String,
    active: bool,
}

fn bench_json_response(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_response");

    // 小型 JSON 响应
    group.bench_function("small_json", |b| {
        b.iter(|| {
            black_box(
                json(serde_json::json!({
                    "message": "Hello",
                    "status": "ok"
                }))
                .unwrap(),
            )
        })
    });

    // 中型 JSON 响应
    group.bench_function("medium_json", |b| {
        let data = TestData {
            id: 1,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            active: true,
        };
        b.iter(|| black_box(json(&data).unwrap()))
    });

    // 大型 JSON 响应 - 数组
    group.bench_function("large_json_array", |b| {
        let users: Vec<TestData> = (0..100)
            .map(|i| TestData {
                id: i,
                name: format!("User {}", i),
                email: format!("user{}@example.com", i),
                active: true,
            })
            .collect();
        b.iter(|| black_box(json(&users).unwrap()))
    });

    // 深度嵌套 JSON
    group.bench_function("nested_json", |b| {
        b.iter(|| {
            black_box(
                json(serde_json::json!({
                    "user": {
                        "id": 1,
                        "profile": {
                            "name": "Test",
                            "settings": {
                                "theme": "dark",
                                "notifications": true
                            }
                        },
                        "posts": [
                            {"id": 1, "title": "Post 1"},
                            {"id": 2, "title": "Post 2"}
                        ]
                    }
                }))
                .unwrap(),
            )
        })
    });

    group.finish();
}

fn bench_text_response(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_response");

    group.bench_function("short_text", |b| {
        b.iter(|| black_box(text("Hello, World!")))
    });

    for size in [10, 100, 1000, 10000].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, size| {
            let content = "a".repeat(*size);
            b.iter(|| black_box(text(content.clone())))
        });
    }

    group.finish();
}

fn bench_html_response(c: &mut Criterion) {
    let mut group = c.benchmark_group("html_response");

    group.bench_function("simple_html", |b| {
        b.iter(|| black_box(html("<h1>Hello, World!</h1>")))
    });

    group.bench_function("complex_html", |b| {
        let html_content = r#"
            <!DOCTYPE html>
            <html>
            <head><title>Test Page</title></head>
            <body>
                <header><nav>Home | About | Contact</nav></header>
                <main>
                    <h1>Welcome</h1>
                    <p>This is a test page with multiple elements.</p>
                </main>
                <footer>Copyright 2024</footer>
            </body>
            </html>
        "#;
        b.iter(|| black_box(html(html_content)))
    });

    group.finish();
}

fn bench_redirect_response(c: &mut Criterion) {
    let mut group = c.benchmark_group("redirect_response");

    group.bench_function("relative_redirect", |b| {
        b.iter(|| black_box(redirect("/login").unwrap()))
    });

    group.bench_function("absolute_redirect", |b| {
        b.iter(|| black_box(redirect("https://example.com/login").unwrap()))
    });

    group.finish();
}

fn bench_status_codes(c: &mut Criterion) {
    let mut group = c.benchmark_group("status_codes");

    group.bench_function("ok_200", |b| {
        b.iter(|| {
            black_box(
                json(serde_json::json!({"status": "ok"}))
                    .unwrap()
                    .status(StatusCode::OK),
            )
        })
    });

    group.bench_function("created_201", |b| {
        b.iter(|| {
            black_box(
                json(serde_json::json!({"id": 123}))
                    .unwrap()
                    .status(StatusCode::CREATED),
            )
        })
    });

    group.bench_function("no_content_204", |b| {
        b.iter(|| black_box(no_content()))
    });

    group.finish();
}

fn bench_response_chaining(c: &mut Criterion) {
    let mut group = c.benchmark_group("response_chaining");

    group.bench_function("single_header", |b| {
        b.iter(|| {
            black_box(
                json(serde_json::json!({"data": "test"}))
                    .unwrap()
                    .header("X-Custom", "value"),
            )
        })
    });

    group.bench_function("multiple_headers", |b| {
        b.iter(|| {
            black_box(
                json(serde_json::json!({"data": "test"}))
                    .unwrap()
                    .header("X-Request-Id", "123")
                    .header("X-Response-Time", "10ms")
                    .header("X-Cache", "HIT"),
            )
        })
    });

    group.bench_function("status_with_headers", |b| {
        b.iter(|| {
            black_box(
                json(serde_json::json!({"id": 123}))
                    .unwrap()
                    .status(StatusCode::CREATED)
                    .header("Location", "/resource/123")
                    .header("X-Request-Id", "abc123"),
            )
        })
    });

    group.finish();
}

fn bench_content_type(c: &mut Criterion) {
    let mut group = c.benchmark_group("content_type");

    group.bench_function("custom_content_type", |b| {
        b.iter(|| {
            black_box(
                json(serde_json::json!({"data": "test"}))
                    .unwrap()
                    .content_type("application/vnd.api+json"),
            )
        })
    });

    group.finish();
}

fn bench_bytes_response(c: &mut Criterion) {
    let mut group = c.benchmark_group("bytes_response");

    for size in [1024, 10240, 102400, 1024000].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, size| {
            let data = vec![42u8; *size];
            b.iter(|| black_box(bytes(data.clone())))
        });
    }

    group.finish();
}

fn bench_response_clone(c: &mut Criterion) {
    let mut group = c.benchmark_group("response_clone");

    let response = text("Hello, World!");
    let _response_clone = response.clone();

    group.bench_function("clone_small_response", |b| {
        b.iter(|| black_box(text("Hello, World!")))
    });

    group.bench_function("clone_large_response", |b| {
        let large_text = "a".repeat(10000);
        b.iter(|| black_box(text(large_text.clone())))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_json_response,
    bench_text_response,
    bench_html_response,
    bench_redirect_response,
    bench_status_codes,
    bench_response_chaining,
    bench_content_type,
    bench_bytes_response,
    bench_response_clone
);
criterion_main!(benches);
