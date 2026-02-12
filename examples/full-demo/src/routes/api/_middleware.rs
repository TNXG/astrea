// API 中间件 - 继承根中间件并添加额外的层
use astrea::middleware::*;

pub fn middleware<S: Clone + Send + Sync + 'static>() -> Middleware<S> {
    // 使用 new() 来继承父中间件（extend mode）
    Middleware::new().wrap(|router| {
        router
            // 添加请求日志
            .layer(astrea::tower_http::trace::TraceLayer::new_for_http())
    })
}
