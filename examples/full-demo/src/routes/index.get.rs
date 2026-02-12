// 首页路由 - GET /
use astrea::prelude::*;
use crate::AppState;

#[route]
pub async fn handler(event: Event) -> Result<Response> {
    // 获取应用状态
    let state = get_state::<AppState>(&event)?;

    Ok(html(format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>{app_name}</title>
    <style>
        body {{ font-family: Arial, sans-serif; max-width: 800px; margin: 50px auto; padding: 20px; }}
        h1 {{ color: #333; }}
        .endpoint {{ background: #f4f4f4; padding: 10px; margin: 10px 0; border-radius: 5px; }}
        .method {{ display: inline-block; padding: 3px 8px; border-radius: 3px; color: white; font-weight: bold; }}
        .get {{ background: #61affe; }}
        .post {{ background: #49cc90; }}
        .put {{ background: #fca130; }}
        .delete {{ background: #f93e3e; }}
        code {{ background: #eee; padding: 2px 5px; border-radius: 3px; }}
    </style>
</head>
<body>
    <h1>🚀 {app_name} v{version}</h1>
    <p>Welcome to the Astrea Full Demo! This example showcases all features of Astrea.</p>

    <h2>Available Endpoints</h2>

    <div class="endpoint">
        <span class="method get">GET</span> <code>/</code>
        <p>This page</p>
    </div>

    <div class="endpoint">
        <span class="method get">GET</span> <code>/api/users</code>
        <p>List all users (with optional query parameters)</p>
    </div>

    <div class="endpoint">
        <span class="method post">POST</span> <code>/api/users</code>
        <p>Create a new user</p>
    </div>

    <div class="endpoint">
        <span class="method get">GET</span> <code>/api/users/:id</code>
        <p>Get a specific user by ID</p>
    </div>

    <div class="endpoint">
        <span class="method put">PUT</span> <code>/api/users/:id</code>
        <p>Update a user by ID</p>
    </div>

    <div class="endpoint">
        <span class="method delete">DELETE</span> <code>/api/users/:id</code>
        <p>Delete a user by ID</p>
    </div>

    <div class="endpoint">
        <span class="method get">GET</span> <code>/posts/*slug</code>
        <p>Catch-all route for posts</p>
    </div>

    <h2>API Documentation</h2>
    <p>
        <a href="/swagger">Swagger UI</a> |
        <a href="/openapi.json">OpenAPI Spec</a>
    </p>

    <h2>Features Demonstrated</h2>
    <ul>
        <li>✅ File-based routing (file name = route path)</li>
        <li>✅ Unified handler signature <code>(Event) -> Result&lt;Response&gt;</code></li>
        <li>✅ Path parameters: <code>get_param(&event, "id")</code></li>
        <li>✅ Query parameters: <code>get_query_param(&event, "q")</code></li>
        <li>✅ Request body: <code>get_body(&event)</code></li>
        <li>✅ Response helpers: <code>json()</code>, <code>text()</code>, <code>html()</code></li>
        <li>✅ Error handling: <code>RouteError::not_found()</code></li>
        <li>✅ Scoped middleware (extend and override modes)</li>
        <li>✅ Application state: <code>get_state(&event)</code></li>
        <li>✅ OpenAPI auto-generation (Swagger UI)</li>
        <li>✅ Catch-all routes: <code>[...slug]</code></li>
    </ul>
</body>
</html>
        "#,
        app_name = state.app_name,
        version = state.version
    )))
}
