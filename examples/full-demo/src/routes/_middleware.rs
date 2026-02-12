// 全局中间件 - 应用于所有路由
use astrea::middleware::*;

pub fn middleware<S: Clone + Send + Sync + 'static>() -> Middleware<S> {
    Middleware::new().wrap(|router| {
        router
            // 添加 HTTP 日志追踪
            .layer(astrea::tower_http::trace::TraceLayer::new_for_http())
            // 添加 CORS 支持（允许所有来源）
            .layer(astrea::tower_http::cors::CorsLayer::permissive())
            // 添加压缩支持
            .layer(astrea::tower_http::compression::CompressionLayer::new())
    })
}
