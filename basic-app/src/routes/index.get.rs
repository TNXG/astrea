// 首页路由
//
// 示例路由文件

use astrea::prelude::*;

/// 首页处理函数
#[route]
pub async fn handler(_event: Event) -> Result<Response> {
    json(serde_json::json!({
        "message": "Welcome to Astrea!",
        "version": "0.1.0"
    }))
}
