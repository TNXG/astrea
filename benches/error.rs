//! RouteError 性能测试
//!
//! 测试错误创建、转换等操作的性能

use astrea::RouteError;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use criterion::{ criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;

fn bench_error_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_creation");

    group.bench_function("bad_request", |b| {
        b.iter(|| {
            black_box(RouteError::bad_request("Invalid input"))
        })
    });

    group.bench_function("not_found", |b| {
        b.iter(|| {
            black_box(RouteError::not_found("Resource not found"))
        })
    });

    group.bench_function("unauthorized", |b| {
        b.iter(|| {
            black_box(RouteError::unauthorized("Not authenticated"))
        })
    });

    group.bench_function("forbidden", |b| {
        b.iter(|| {
            black_box(RouteError::forbidden("Access denied"))
        })
    });

    group.bench_function("validation", |b| {
        b.iter(|| {
            black_box(RouteError::validation("Validation failed"))
        })
    });

    group.bench_function("rate_limit", |b| {
        b.iter(|| {
            black_box(RouteError::RateLimit("Too many requests".to_string()))
        })
    });

    group.bench_function("internal_from_anyhow", |b| {
        b.iter(|| {
            let anyhow_error = anyhow::anyhow!("Internal server error");
            black_box(RouteError::from(anyhow_error))
        })
    });

    group.bench_function("custom_error", |b| {
        b.iter(|| {
            black_box(RouteError::custom(StatusCode::IM_A_TEAPOT, "I'm a teapot"))
        })
    });

    group.finish();
}

fn bench_error_message(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_message");

    let error = RouteError::bad_request("Invalid input data");

    group.bench_function("get_message", |b| {
        b.iter(|| black_box(error.message()))
    });

    // 测试不同长度的错误消息
    for size in [10, 50, 100, 500].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, size| {
            let msg = "x".repeat(*size);
            let error = RouteError::bad_request(msg);
            b.iter(|| black_box(error.message()))
        });
    }

    group.finish();
}

fn bench_error_status_code(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_status_code");

    let errors = vec![
        RouteError::bad_request("test"),
        RouteError::not_found("test"),
        RouteError::unauthorized("test"),
        RouteError::forbidden("test"),
        RouteError::validation("test"),
        RouteError::RateLimit("test".to_string()),
        RouteError::Internal(anyhow::anyhow!("test")),
        RouteError::custom(StatusCode::IM_A_TEAPOT, "test"),
    ];

    for error in errors {
        let status = error.status_code();
        let name = format!("{:?}", status);
        group.bench_function(name, |b| {
            b.iter(|| black_box(error.status_code()))
        });
    }

    group.finish();
}

fn bench_error_into_response(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_into_response");

    group.bench_function("bad_request_response", |b| {
        b.iter(|| {
            let error = RouteError::bad_request("Invalid input");
            black_box(error.into_response())
        })
    });

    group.bench_function("not_found_response", |b| {
        b.iter(|| {
            let error = RouteError::not_found("Resource not found");
            black_box(error.into_response())
        })
    });

    group.bench_function("internal_error_response", |b| {
        b.iter(|| {
            let error = RouteError::from(anyhow::anyhow!("Internal error"));
            black_box(error.into_response())
        })
    });

    group.bench_function("custom_error_response", |b| {
        b.iter(|| {
            let error = RouteError::custom(StatusCode::IM_A_TEAPOT, "Custom error");
            black_box(error.into_response())
        })
    });

    group.finish();
}

// RouteError 没有实现 Clone，跳过此测试
// fn bench_error_clone(c: &mut Criterion) {
//     let mut group = c.benchmark_group("error_clone");
//     let error = RouteError::bad_request("Test error message");
//     group.bench_function("clone_error", |b| {
//         b.iter(|| black_box(error.clone()))
//     });
//     group.finish();
// }

fn bench_error_display(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_display");

    let error = RouteError::bad_request("Invalid input data");

    group.bench_function("to_string", |b| {
        b.iter(|| black_box(error.to_string()))
    });

    group.bench_function("fmt_display", |b| {
        b.iter(|| black_box(format!("{}", error)))
    });

    group.finish();
}

fn bench_error_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_conversion");

    // anyhow::Error 到 RouteError 的转换
    group.bench_function("anyhow_to_route_error", |b| {
        b.iter(|| {
            let anyhow_err = anyhow::anyhow!("Something went wrong: {}", 42);
            black_box(RouteError::from(anyhow_err))
        })
    });

    // 使用 ? 操作符的转换场景
    group.bench_function("question_mark_conversion", |b| {
        fn inner_function() -> Result<(), RouteError> {
            Err(anyhow::anyhow!("Inner error"))?;
            Ok(())
        }
        b.iter(|| black_box(inner_function()))
    });

    group.finish();
}

fn bench_error_builder_pattern(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_builder_pattern");

    // 比较不同的错误创建方式
    group.bench_function("direct_variant", |b| {
        b.iter(|| {
            black_box(RouteError::BadRequest("Invalid input".to_string()))
        })
    });

    group.bench_function("constructor_function", |b| {
        b.iter(|| {
            black_box(RouteError::bad_request("Invalid input"))
        })
    });

    group.finish();
}

fn bench_error_with_context(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_with_context");

    // 模拟带上下文的错误创建
    group.bench_function("error_with_dynamic_message", |b| {
        b.iter(|| {
            let user_id = 123;
            let resource = "post";
            black_box(RouteError::not_found(format!(
                "{} {} not found for user {}",
                resource, user_id, user_id
            )))
        })
    });

    // 链式错误
    group.bench_function("chained_anyhow_error", |b| {
        b.iter(|| {
            let original = anyhow::anyhow!("Database connection failed");
            let context = original.context("Failed to fetch user data");
            black_box(RouteError::from(context))
        })
    });

    group.finish();
}

fn bench_common_error_scenarios(c: &mut Criterion) {
    let mut group = c.benchmark_group("common_error_scenarios");

    // 场景 1: 验证失败
    group.bench_function("validation_error", |b| {
        b.iter(|| {
            let field = "email";
            let reason = "invalid format";
            black_box(RouteError::validation(format!(
                "Field '{}' failed validation: {}",
                field, reason
            )))
        })
    });

    // 场景 2: 认证失败
    group.bench_function("auth_error", |b| {
        b.iter(|| {
            black_box(RouteError::unauthorized("Invalid or missing authentication token"))
        })
    });

    // 场景 3: 权限不足
    group.bench_function("permission_error", |b| {
        b.iter(|| {
            let resource = "admin_panel";
            black_box(RouteError::forbidden(format!(
                "Insufficient permissions to access {}",
                resource
            )))
        })
    });

    // 场景 4: 资源冲突
    group.bench_function("conflict_error", |b| {
        b.iter(|| {
            black_box(RouteError::Conflict(
                "Resource already exists".to_string(),
            ))
        })
    });

    // 场景 5: 速率限制
    group.bench_function("rate_limit_error", |b| {
        b.iter(|| {
            black_box(RouteError::RateLimit(
                "Rate limit exceeded: 100 requests per minute".to_string(),
            ))
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_error_creation,
    bench_error_message,
    bench_error_status_code,
    bench_error_into_response,
    bench_error_display,
    bench_error_conversion,
    bench_error_builder_pattern,
    bench_error_with_context,
    bench_common_error_scenarios
);
criterion_main!(benches);
