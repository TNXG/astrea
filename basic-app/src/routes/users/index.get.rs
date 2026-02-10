// 用户列表路由

use astrea::prelude::*;

/// 获取用户列表
#[route]
pub async fn handler(_event: Event) -> Result<Response> {
    json(serde_json::json!({
        "users": [
            {"id": 1, "name": "Alice"},
            {"id": 2, "name": "Bob"}
        ]
    }))
}
