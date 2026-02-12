// 删除用户 - DELETE /api/users/:id
use astrea::prelude::*;

#[route]
pub async fn handler(event: Event) -> Result<Response> {
    // 提取路径参数
    let id = get_param_required(&event, "id")?;

    // 演示 204 No Content 响应
    let response = no_content().header("X-Deleted-Id", id);

    Ok(response)
}
