// 捕获所有路由 - GET /posts/*slug
// 演示 catch-all 参数，匹配 /posts 后的所有路径
use astrea::prelude::*;

#[route]
pub async fn handler(event: Event) -> Result<Response> {
    // 提取 catch-all 参数
    let slug = get_param_required(&event, "slug")?;

    // slug 会包含完整的路径片段，例如 "2024/01/hello-world"
    let parts: Vec<&str> = slug.split('/').collect();

    json(json!({
        "slug": slug,
        "parts": parts,
        "year": parts.first(),
        "month": parts.get(1),
        "post_name": parts.get(2),
        "message": format!("This is a catch-all route matching: /posts/{}", slug)
    }))
}
