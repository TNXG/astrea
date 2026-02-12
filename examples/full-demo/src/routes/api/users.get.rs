// 获取用户列表 - GET /api/users
use astrea::prelude::*;

#[route]
pub async fn handler(event: Event) -> Result<Response> {
    // 演示查询参数提取
    let page = get_query_param(&event, "page").unwrap_or_else(|| "1".to_string());
    let limit = get_query_param(&event, "limit").unwrap_or_else(|| "10".to_string());
    let search = get_query_param(&event, "q");

    // 演示获取所有查询参数
    let all_query = get_query(&event);

    // 演示获取请求头
    let user_agent = get_header(&event, "user-agent").unwrap_or("Unknown");

    // 演示获取请求方法和路径
    let method = get_method(&event);
    let path = get_path(&event);

    let users = vec![
        json!({
            "id": 1,
            "name": "Alice",
            "email": "alice@example.com"
        }),
        json!({
            "id": 2,
            "name": "Bob",
            "email": "bob@example.com"
        }),
        json!({
            "id": 3,
            "name": "Charlie",
            "email": "charlie@example.com"
        }),
    ];

    // 构建响应
    let response = json(json!({
        "users": users,
        "pagination": {
            "page": page,
            "limit": limit,
            "total": 3
        },
        "search": search,
        "request_info": {
            "method": method.to_string(),
            "path": path,
            "user_agent": user_agent,
            "all_query_params": all_query
        }
    }))?
        .status(StatusCode::OK)
        .header("X-Page", &page)
        .header("X-Limit", &limit);

    Ok(response)
}
