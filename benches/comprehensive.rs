//! 综合性能测试
//!
//! 测试完整的请求处理流程，模拟真实应用场景

use astrea::prelude::*;
use astrea::{error::Result, Event};
use axum::http::{HeaderMap, Method};
use criterion::{criterion_group, criterion_main, Criterion};
use std::collections::HashMap;
use std::hint::black_box;

/// 模拟用户列表处理函数
fn simulate_user_list_handler(event: &Event) -> Result<Response> {
    let page = get_query_param(event, "page").unwrap_or("1".to_string());
    let limit = get_query_param(event, "limit").unwrap_or("10".to_string());

    let users: Vec<serde_json::Value> = (0..10)
        .map(|i| {
            serde_json::json!({
                "id": i,
                "name": format!("User {}", i),
                "email": format!("user{}@example.com", i)
            })
        })
        .collect();

    json(serde_json::json!({
        "users": users,
        "page": page,
        "limit": limit,
        "total": 100
    }))
}

/// 模拟用户详情处理函数
fn simulate_user_detail_handler(event: &Event) -> Result<Response> {
    let user_id = get_param_required(event, "id")?;

    json(serde_json::json!({
        "id": user_id,
        "name": format!("User {}", user_id),
        "email": format!("user{}@example.com", user_id),
        "active": true
    }))
}

/// 模拟创建用户处理函数
#[derive(serde::Deserialize)]
#[allow(dead_code)]
struct CreateUserRequest {
    name: String,
    email: String,
    password: String,
}

fn simulate_create_user_handler(event: &Event, body_bytes: &[u8]) -> Result<Response> {
    let body: CreateUserRequest = get_body(event, body_bytes)?;

    json(serde_json::json!({
        "id": 123,
        "name": body.name,
        "email": body.email
    }))
    .map(|r| r.status(StatusCode::CREATED))
}

/// 模拟需要认证的处理函数
fn simulate_authenticated_handler(event: &Event) -> Result<Response> {
    let _auth = get_header(event, "authorization")
        .ok_or_else(|| RouteError::unauthorized("Missing authorization header"))?;

    json(serde_json::json!({
        "message": "Authenticated successfully",
        "data": "secret data"
    }))
}

fn bench_user_list_scenario(c: &mut Criterion) {
    let mut group = c.benchmark_group("scenario_user_list");

    let event = Event::new(
        Method::GET,
        "/users".to_string(),
        "/users?page=1&limit=10".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
    );

    group.bench_function("handle_user_list", |b| {
        b.iter(|| black_box(simulate_user_list_handler(black_box(&event))));
    });

    group.finish();
}

fn bench_user_detail_scenario(c: &mut Criterion) {
    let mut group = c.benchmark_group("scenario_user_detail");

    let mut params = HashMap::new();
    params.insert("id".to_string(), "123".to_string());

    let event = Event::new(
        Method::GET,
        "/users/123".to_string(),
        "/users/123".parse().unwrap(),
        HeaderMap::new(),
        params,
        HashMap::new(),
    );

    group.bench_function("handle_user_detail", |b| {
        b.iter(|| black_box(simulate_user_detail_handler(black_box(&event))));
    });

    group.finish();
}

fn bench_create_user_scenario(c: &mut Criterion) {
    let mut group = c.benchmark_group("scenario_create_user");

    let event = Event::new(
        Method::POST,
        "/users".to_string(),
        "/users".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
    );

    let body = br#"{"name":"Alice","email":"alice@example.com","password":"secret123"}"#;

    group.bench_function("handle_create_user", |b| {
        b.iter(|| black_box(simulate_create_user_handler(black_box(&event), body)));
    });

    group.finish();
}

fn bench_authenticated_request_scenario(c: &mut Criterion) {
    let mut group = c.benchmark_group("scenario_authenticated");

    let mut headers = HeaderMap::new();
    headers.insert("authorization", "Bearer secret_token".parse().unwrap());

    let event = Event::new(
        Method::GET,
        "/protected".to_string(),
        "/protected".parse().unwrap(),
        headers,
        HashMap::new(),
        HashMap::new(),
    );

    group.bench_function("handle_authenticated", |b| {
        b.iter(|| black_box(simulate_authenticated_handler(black_box(&event))));
    });

    group.finish();
}

fn bench_full_request_lifecycle(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_lifecycle");

    group.bench_function("event_creation_to_response", |b| {
        b.iter(|| {
            // 1. 创建 Event (模拟 Axum 提取器)
            let mut params = HashMap::new();
            params.insert("id".to_string(), "42".to_string());

            let event = Event::new(
                Method::GET,
                "/users/42".to_string(),
                "/users/42?format=json".parse().unwrap(),
                HeaderMap::new(),
                params,
                HashMap::new(),
            );

            // 2. 提取参数
            let _id = get_param(&event, "id");
            let _format = get_query_param(&event, "format");

            // 3. 构建响应
            let response = json(serde_json::json!({
                "id": _id.unwrap(),
                "format": _format
            }))
            .unwrap();

            // 4. 转换为 Axum Response
            black_box(response.into_axum_response())
        });
    });

    group.finish();
}

fn bench_concurrent_style_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_style");

    // 模拟处理多个请求的场景（单线程）
    group.bench_function("process_multiple_requests", |b| {
        let requests = vec![
            ("/", Method::GET),
            ("/users", Method::GET),
            ("/users/123", Method::GET),
            ("/posts", Method::POST),
        ];

        b.iter(|| {
            for (path, method) in &requests {
                let event = Event::new(
                    method.clone(),
                    path.to_string(),
                    path.parse().unwrap(),
                    HeaderMap::new(),
                    HashMap::new(),
                    HashMap::new(),
                );
                black_box(get_path(&event));
            }
        });
    });

    group.finish();
}

fn bench_error_handling_paths(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling");

    // 成功路径
    group.bench_function("success_path", |b| {
        let mut params = HashMap::new();
        params.insert("id".to_string(), "123".to_string());

        let _event = Event::new(
            Method::GET,
            "/users/123".to_string(),
            "/users/123".parse().unwrap(),
            HeaderMap::new(),
            params,
            HashMap::new(),
        );

        b.iter(|| {
            let result: Result<Response> = json(serde_json::json!({"id": "123"}));
            black_box(result)
        });
    });

    // 错误路径 - 参数缺失
    group.bench_function("error_missing_param", |b| {
        let event = Event::new(
            Method::GET,
            "/users/".to_string(),
            "/users/".parse().unwrap(),
            HeaderMap::new(),
            HashMap::new(),
            HashMap::new(),
        );

        b.iter(|| black_box(get_param_required(&event, "id")));
    });

    // 错误路径 - 认证失败
    group.bench_function("error_unauthorized", |b| {
        let event = Event::new(
            Method::GET,
            "/protected".to_string(),
            "/protected".parse().unwrap(),
            HeaderMap::new(),
            HashMap::new(),
            HashMap::new(),
        );

        b.iter(|| {
            black_box(
                get_header(&event, "authorization")
                    .ok_or_else(|| RouteError::unauthorized("Missing authorization header")),
            )
        });
    });

    group.finish();
}

fn bench_response_size_impact(c: &mut Criterion) {
    let mut group = c.benchmark_group("response_size_impact");

    group.bench_function("small_response", |b| {
        b.iter(|| black_box(json(serde_json::json!({"message": "ok"})).unwrap()));
    });

    group.bench_function("medium_response", |b| {
        let data: Vec<serde_json::Value> = (0..10)
            .map(|i| {
                serde_json::json!({
                    "id": i,
                    "name": format!("Item {}", i),
                    "value": i * 2
                })
            })
            .collect();
        b.iter(|| black_box(json(&data).unwrap()));
    });

    group.bench_function("large_response", |b| {
        let data: Vec<serde_json::Value> = (0..100)
            .map(|i| {
                serde_json::json!({
                    "id": i,
                    "name": format!("Item {}", i),
                    "description": format!("This is item number {}", i),
                    "value": i * 2,
                    "active": true
                })
            })
            .collect();
        b.iter(|| black_box(json(&data).unwrap()));
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_user_list_scenario,
    bench_user_detail_scenario,
    bench_create_user_scenario,
    bench_authenticated_request_scenario,
    bench_full_request_lifecycle,
    bench_concurrent_style_processing,
    bench_error_handling_paths,
    bench_response_size_impact
);
criterion_main!(benches);
