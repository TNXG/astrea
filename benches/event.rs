//! Event 性能测试
//!
//! 测试 Event 创建、克隆、访问等操作的性能

use astrea::Event;
use axum::http::{HeaderMap, Method, Uri};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::HashMap;
use std::hint::black_box;

/// 创建测试用的 Event
fn create_test_event(path: &str, with_params: bool, with_query: bool) -> Event {
    let mut params = HashMap::new();
    if with_params {
        params.insert("id".to_string(), "123".to_string());
        params.insert("name".to_string(), "test_user".to_string());
    }

    let mut query = HashMap::new();
    if with_query {
        query.insert("page".to_string(), "1".to_string());
        query.insert("limit".to_string(), "10".to_string());
        query.insert("sort".to_string(), "desc".to_string());
    }

    let uri: Uri = format!("{}?page=1&limit=10", path).parse().unwrap();

    Event::new(
        Method::GET,
        path.to_string(),
        uri,
        HeaderMap::new(),
        params,
        query,
    )
}

fn bench_event_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_creation");

    // 基础 Event 创建
    group.bench_function("empty_event", |b| {
        b.iter(|| {
            black_box(Event::new(
                Method::GET,
                "/test".to_string(),
                "/test".parse().unwrap(),
                HeaderMap::new(),
                HashMap::new(),
                HashMap::new(),
            ))
        })
    });

    // 带参数的 Event 创建
    group.bench_function("with_params", |b| {
        b.iter(|| {
            let mut params = HashMap::new();
            params.insert("id".to_string(), "123".to_string());
            params.insert("name".to_string(), "test".to_string());
            black_box(Event::new(
                Method::POST,
                "/users/123".to_string(),
                "/users/123".parse().unwrap(),
                HeaderMap::new(),
                params,
                HashMap::new(),
            ))
        })
    });

    // 带查询参数的 Event 创建
    group.bench_function("with_query", |b| {
        b.iter(|| {
            let mut query = HashMap::new();
            query.insert("page".to_string(), "1".to_string());
            query.insert("limit".to_string(), "10".to_string());
            query.insert("sort".to_string(), "desc".to_string());
            black_box(Event::new(
                Method::GET,
                "/users".to_string(),
                "/users?page=1&limit=10&sort=desc".parse().unwrap(),
                HeaderMap::new(),
                HashMap::new(),
                query,
            ))
        })
    });

    group.finish();
}

fn bench_event_cloning(c: &mut Criterion) {
    let event = create_test_event("/users/123", true, true);

    let mut group = c.benchmark_group("event_cloning");

    // Event 是 Arc 包裹的，克隆应该非常快
    group.bench_function("clone_arc", |b| b.iter(|| black_box(event.clone())));

    group.finish();
}

fn bench_event_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_access");

    // 访问方法
    group.bench_function("method_access", |b| {
        let event = create_test_event("/users/123", false, false);
        b.iter(|| black_box(event.method()))
    });

    // 访问路径
    group.bench_function("path_access", |b| {
        let event = create_test_event("/users/123", false, false);
        b.iter(|| black_box(event.path()))
    });

    // 访问 URI
    group.bench_function("uri_access", |b| {
        let event = create_test_event("/users/123", false, false);
        b.iter(|| black_box(event.uri()))
    });

    // 访问 headers
    group.bench_function("headers_access", |b| {
        let event = create_test_event("/users/123", false, false);
        b.iter(|| black_box(event.headers()))
    });

    group.finish();
}

fn bench_param_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("param_access");

    // params 是预填充的，访问应该很快
    group.bench_function("existing_param", |b| {
        let event = create_test_event("/users/123", true, false);
        b.iter(|| black_box(event.params().get("id")))
    });

    group.bench_function("params_get_all", |b| {
        let event = create_test_event("/users/123", true, false);
        b.iter(|| black_box(event.params().clone()))
    });

    // 测试不同数量的参数
    for param_count in [1, 5, 10, 20].iter() {
        group.throughput(Throughput::Elements(*param_count as u64));

        let mut params = HashMap::new();
        for i in 0..*param_count {
            params.insert(format!("param_{}", i), format!("value_{}", i));
        }

        group.bench_with_input(
            BenchmarkId::from_parameter(param_count),
            param_count,
            |b, _| {
                let event = Event::new(
                    Method::GET,
                    "/test".to_string(),
                    "/test".parse().unwrap(),
                    HeaderMap::new(),
                    params.clone(),
                    HashMap::new(),
                );
                b.iter(|| {
                    for key in event.params().keys() {
                        black_box(event.params().get(key));
                    }
                })
            },
        );
    }

    group.finish();
}

fn bench_query_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_parsing");

    // 测试不同复杂度的查询字符串
    group.bench_function("empty_query", |b| {
        let event = Event::new(
            Method::GET,
            "/test".to_string(),
            "/test".parse().unwrap(),
            HeaderMap::new(),
            HashMap::new(),
            HashMap::new(),
        );
        b.iter(|| black_box(event.query()))
    });

    group.bench_function("single_param", |b| {
        let event = Event::new(
            Method::GET,
            "/test".to_string(),
            "/test?key=value".parse().unwrap(),
            HeaderMap::new(),
            HashMap::new(),
            HashMap::new(),
        );
        b.iter(|| black_box(event.query()))
    });

    group.bench_function("multiple_params", |b| {
        let event = Event::new(
            Method::GET,
            "/test".to_string(),
            "/test?key1=value1&key2=value2&key3=value3".parse().unwrap(),
            HeaderMap::new(),
            HashMap::new(),
            HashMap::new(),
        );
        b.iter(|| black_box(event.query()))
    });

    // 测试懒加载缓存 - 第二次访问应该更快
    group.bench_function("cached_query_access", |b| {
        let event = Event::new(
            Method::GET,
            "/test".to_string(),
            "/test?key1=value1&key2=value2&key3=value3".parse().unwrap(),
            HeaderMap::new(),
            HashMap::new(),
            HashMap::new(),
        );
        // 第一次调用触发解析
        let _ = event.query();
        b.iter(|| black_box(event.query()))
    });

    for param_count in [1, 5, 10, 20].iter() {
        group.throughput(Throughput::Elements(*param_count as u64));

        let query_string: String = (0..*param_count)
            .map(|i| format!("key{}=value{}", i, i))
            .collect::<Vec<_>>()
            .join("&");

        group.bench_with_input(
            BenchmarkId::from_parameter(param_count),
            param_count,
            |b, _| {
                let event = Event::new(
                    Method::GET,
                    "/test".to_string(),
                    format!("/test?{}", query_string).parse().unwrap(),
                    HeaderMap::new(),
                    HashMap::new(),
                    HashMap::new(),
                );
                b.iter(|| black_box(event.query()))
            },
        );
    }

    group.finish();
}

fn bench_json_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_parsing");

    let small_json = br#"{"name":"test","value":123}"#;
    let medium_json = br#"{"name":"test","value":123,"nested":{"field1":"data1","field2":"data2","field3":"data3"}}"#;
    let large_json = br#"{"name":"test","value":123,"nested":{"field1":"data1","field2":"data2","field3":"data3"},"array":[1,2,3,4,5,6,7,8,9,10],"extra":{"key":"value"}}"#;

    let event = Event::new(
        Method::POST,
        "/test".to_string(),
        "/test".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
    );

    #[derive(serde::Deserialize)]
    #[allow(dead_code)]
    struct TestData {
        name: String,
        value: i32,
    }

    group.bench_function("small_json", |b| {
        b.iter(|| black_box(event.parse_json::<TestData>(small_json)))
    });

    group.bench_function("medium_json", |b| {
        b.iter(|| black_box(event.parse_json::<serde_json::Value>(medium_json)))
    });

    group.bench_function("large_json", |b| {
        b.iter(|| black_box(event.parse_json::<serde_json::Value>(large_json)))
    });

    group.finish();
}

fn bench_text_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_parsing");

    let event = Event::new(
        Method::POST,
        "/test".to_string(),
        "/test".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
    );

    group.bench_function("valid_utf8", |b| {
        let text = "Hello, World!";
        b.iter(|| black_box(event.parse_text(text.as_bytes())))
    });

    group.bench_function("long_text", |b| {
        let text = "a".repeat(10000);
        b.iter(|| black_box(event.parse_text(text.as_bytes())))
    });

    group.finish();
}

fn bench_state_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_access");

    #[derive(Clone, Debug)]
    struct TestState;

    let state = TestState;

    let mut event = Event::new(
        Method::GET,
        "/test".to_string(),
        "/test".parse().unwrap(),
        HeaderMap::new(),
        HashMap::new(),
        HashMap::new(),
    );
    event.state = Some(std::sync::Arc::new(state));

    group.bench_function("get_state", |b| {
        b.iter(|| black_box(event.state::<TestState>()))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_event_creation,
    bench_event_cloning,
    bench_event_access,
    bench_param_access,
    bench_query_parsing,
    bench_json_parsing,
    bench_text_parsing,
    bench_state_access
);
criterion_main!(benches);
