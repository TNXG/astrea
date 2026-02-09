//! 基础示例应用
//!
//! 演示如何使用 Astrea 文件路由器构建 Web 应用

// 由 generate_routes! 宏在编译时扫描 routes/ 目录并生成路由代码
#[allow(dead_code, unused_imports)]
mod routes {
    astrea::generate_routes!();
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let app = routes::create_router();

    let addr = "127.0.0.1:3000";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to port 3000");

    astrea::tracing::info!("Listening on http://{}", addr);

    astrea::serve(listener, app).await.unwrap();
}
