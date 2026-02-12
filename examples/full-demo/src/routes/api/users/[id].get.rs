// 获取单个用户 - GET /api/users/:id
use astrea::prelude::*;

#[route]
pub async fn handler(event: Event) -> Result<Response> {
    // 演示提取路径参数
    let id = get_param_required(&event, "id")?;

    // 演示错误处理 - 如果 ID 为 "404"，返回 404 错误
    if id == "404" {
        return Err(RouteError::not_found(format!("User {} not found", id)));
    }

    // 演示错误处理 - 如果 ID 为 "401"，返回 401 错误
    if id == "401" {
        return Err(RouteError::unauthorized("Authentication required"));
    }

    // 演示错误处理 - 如果 ID 为 "403"，返回 403 错误
    if id == "403" {
        return Err(RouteError::forbidden("Access denied"));
    }

    // 演示错误处理 - 如果 ID 为 "409"，返回 409 冲突错误
    if id == "409" {
        return Err(RouteError::conflict("User already exists"));
    }

    // 演示错误处理 - 如果 ID 为 "429"，返回 429 速率限制错误
    if id == "429" {
        return Err(RouteError::rate_limit("Too many requests"));
    }

    // 模拟数据库查询
    let user = match id {
        "1" => Some(json!({
            "id": 1,
            "name": "Alice",
            "email": "alice@example.com",
            "bio": "Software Engineer"
        })),
        "2" => Some(json!({
            "id": 2,
            "name": "Bob",
            "email": "bob@example.com",
            "bio": "Product Manager"
        })),
        "3" => Some(json!({
            "id": 3,
            "name": "Charlie",
            "email": "charlie@example.com",
            "bio": "Designer"
        })),
        _ => None,
    };

    match user {
        Some(user) => Ok(json(user)?),
        None => Err(RouteError::not_found(format!("User {} not found", id))),
    }
}
