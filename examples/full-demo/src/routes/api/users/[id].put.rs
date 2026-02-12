// 更新用户 - PUT /api/users/:id
use astrea::prelude::*;

#[derive(Deserialize)]
struct UpdateUserRequest {
    name: Option<String>,
    email: Option<String>,
    bio: Option<String>,
}

#[route]
pub async fn handler(event: Event) -> Result<Response> {
    // 提取路径参数
    let id = get_param_required(&event, "id")?;

    // 提取 JSON 请求体
    let body: UpdateUserRequest = get_body(&event)?;

    // 模拟更新用户
    let user = json!({
        "id": id,
        "name": body.name.unwrap_or_else(|| "Updated Name".to_string()),
        "email": body.email.unwrap_or_else(|| "updated@example.com".to_string()),
        "bio": body.bio.unwrap_or_else(|| "Updated bio".to_string()),
        "updated_at": "2025-01-15T11:00:00Z"
    });

    Ok(json(user)?)
}
