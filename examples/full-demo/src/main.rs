// 导入路由模块
mod routes {
    // 使用 Astrea 的宏自动生成路由
    astrea::generate_routes!();
}

// 定义应用状态 - 需要公开给路由模块使用
#[derive(Clone)]
pub struct AppState {
    pub app_name: String,
    pub version: String,
}

#[astrea::tokio::main]
async fn main() {
    // 初始化日志系统
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_ansi(true) // 启用 ANSI 颜色支持
        .init();
    // 创建应用状态
    let state = AppState {
        app_name: "Astrea Full Demo".to_string(),
        version: "1.0.0".to_string(),
    };

    // 创建路由器并注入状态，同时添加 OpenAPI 路由
    let app = routes::create_router()
        .with_state(state)
        .merge(astrea::openapi::router("Astrea Full Demo", "1.0.0"));

    // 绑定监听地址
    let listener = astrea::tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    println!("🚀 Server listening on http://localhost:3000");
    println!("📚 Swagger UI: http://localhost:3000/swagger");
    println!("📄 OpenAPI Spec: http://localhost:3000/openapi.json");

    // 启动服务器
    astrea::axum::serve(listener, app).await.unwrap();
}
