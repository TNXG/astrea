// 创建文章路由

use astrea::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct CreatePostRequest {
    title: String,
    content: String,
}

/// 创建新文章
#[route]
pub async fn handler(_event: Event) -> Result<Response> {
    // TODO: 实现实际的创建逻辑
    json(serde_json::json!({
        "message": "Post created successfully",
        "id": 1
    }))
    .map(|r| r.status(StatusCode::CREATED))
}
