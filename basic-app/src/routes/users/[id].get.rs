// 获取单个用户路由

use astrea::prelude::*;

/// 根据用户 ID 获取用户信息
#[route]
pub async fn handler(event: Event) -> Result<Response> {
    let user_id = get_param(&event, "id").unwrap_or("0");

    json(serde_json::json!({
        "id": user_id,
        "name": format!("User {}", user_id)
    }))
}
