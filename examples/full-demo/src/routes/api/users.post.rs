// 创建用户 - POST /api/users
use astrea::prelude::*;

#[derive(Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

#[route]
pub async fn handler(event: Event) -> Result<Response> {
    // 演示提取 JSON 请求体
    let body: CreateUserRequest = get_body(&event)?;

    // 简单的验证
    if body.name.is_empty() {
        return Err(RouteError::bad_request("Name is required"));
    }

    if !body.email.contains('@') {
        return Err(RouteError::validation("Invalid email format"));
    }

    // 模拟创建用户
    let user = json!({
        "id": 4,
        "name": body.name,
        "email": body.email,
        "created_at": "2025-01-15T10:30:00Z"
    });

    // 返回 201 Created 状态码
    let response = json(user)?.status(StatusCode::CREATED);
    Ok(response)
}
