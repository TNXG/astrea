<p align="center">
  <h1 align="center">Astrea</h1>
  <p align="center">
    基于文件系统的 <a href="https://github.com/tokio-rs/axum">Axum</a> 路由框架。
    <br />
    灵感来自 <a href="https://nitro.unjs.io/">Nitro</a> 和 <a href="https://h3.unjs.io/">H3</a>。
  </p>
</p>

<p align="center">
  <a href="https://crates.io/crates/astrea"><img src="https://img.shields.io/crates/v/astrea.svg" alt="crates.io" /></a>
  <a href="https://docs.rs/astrea"><img src="https://docs.rs/astrea/badge.svg" alt="docs.rs" /></a>
  <a href="https://github.com/TNXG/astrea/blob/main/LICENSE"><img src="https://img.shields.io/crates/l/astrea.svg" alt="MIT License" /></a>
</p>

<p align="center">
  <a href="README.md">English</a>
</p>

---

## Astrea 是什么？

Astrea 把你的 **文件结构** 变成 **API 路由**——在编译时完成，零运行时开销。把一个 `.rs` 文件丢进 `src/routes/` 文件夹，它就变成了一个 HTTP 接口。不需要手动注册路由，不需要 `build.rs`，不需要模板代码。

每个处理函数长这样：

```rust
#[route]
async fn handler(event: Event) -> Result<Response> {
    // 你的逻辑
}
```

就这样。不用记复杂的提取器签名，不用为每种参数类型学新语法。

## 特性

- 📁 **基于文件的路由** — 文件名 = 路由路径，编译时自动生成
- 🎯 **统一的处理函数签名** — 所有处理函数都是 `async fn(Event) -> Result<Response>`
- 🔧 **简单的提取器** — `get_param()`、`get_query_param()`、`get_body()` — 调函数就行
- 🧅 **作用域中间件** — `_middleware.rs` 文件支持叠加和覆盖两种模式
- 📝 **自动生成 OpenAPI** — 可选的 Swagger UI + OpenAPI 3.0 规范（feature flag `openapi`）
- 🔄 **兼容 Axum 生态** — 与所有现有 Axum 中间件和 Tower 生态无缝协作
- 📦 **零额外依赖** — 自动 re-export `axum`、`tokio`、`serde`、`tower` 等，只需依赖 `astrea`

## 快速开始

### 1. 创建项目

```bash
cargo new my-api
cd my-api
```

### 2. 添加 Astrea

```bash
cargo add astrea
```

或者在 `Cargo.toml` 里写：

```toml
[package]
name = "my-api"
edition = "2024"

[dependencies]
astrea = "0.0.1"
```

> **注意：** Astrea 需要 Rust edition 2024（Rust ≥ 1.85）。

### 3. 创建路由文件

```
my-api/
├── src/
│   ├── main.rs
│   └── routes/
│       ├── index.get.rs          # GET /
│       └── users/
│           ├── index.get.rs      # GET /users
│           ├── index.post.rs     # POST /users
│           └── [id].get.rs       # GET /users/:id
```

#### `src/routes/index.get.rs`

```rust
use astrea::prelude::*;

#[route]
pub async fn handler(event: Event) -> Result<Response> {
    json(json!({ "message": "Hello, World!" }))
}
```

#### `src/routes/users/[id].get.rs`

```rust
use astrea::prelude::*;

#[route]
pub async fn handler(event: Event) -> Result<Response> {
    let id = get_param_required(&event, "id")?;
    json(json!({ "user_id": id }))
}
```

### 4. 写 `main.rs`

```rust
mod routes {
    astrea::generate_routes!();
}

#[tokio::main]
async fn main() {
    let app = routes::create_router();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on http://localhost:3000");
    astrea::serve(listener, app).await.unwrap();
}
```

### 5. 运行

```bash
cargo run
```

搞定。你会看到一个漂亮的启动日志：

```text
┌─────────────────────────────────────────────────────────────────────┐
│                        🚀 Astrea Router                            │
├────────┬──────────────────────────────┬─────────────────────────────┤
│ Method │ Path                         │ Middleware                  │
├────────┼──────────────────────────────┼─────────────────────────────┤
│ GET    │ /                            │ (none)                      │
│ GET    │ /users                       │ (none)                      │
│ POST   │ /users                       │ (none)                      │
│ GET    │ /users/:id                   │ (none)                      │
└────────┴──────────────────────────────┴─────────────────────────────┘
✅ 4 route(s), 0 middleware scope(s) loaded
```

访问 `GET http://localhost:3000/` 返回 `{"message":"Hello, World!"}`。

---

## 路由文件命名规则

| 文件名 | 路由 |
|---|---|
| `src/routes/index.get.rs` | `GET /` |
| `src/routes/users.get.rs` | `GET /users` |
| `src/routes/users/index.post.rs` | `POST /users` |
| `src/routes/users/[id].get.rs` | `GET /users/:id` |
| `src/routes/users/[id].delete.rs` | `DELETE /users/:id` |
| `src/routes/posts/[...slug].get.rs` | `GET /posts/*slug`（全匹配） |

**规则：**
- 文件名格式：`<名称>.<HTTP方法>.rs`
- `index` 是特殊名——它映射到目录本身（不会多一个路径段）
- `[param]` → 动态路径参数
- `[...param]` → 全匹配参数（匹配后面所有内容）

---

## 提取请求数据

Astrea 用简单的函数调用替代了 Axum 复杂的提取器签名：

```rust
#[route]
pub async fn handler(event: Event, bytes: Bytes) -> Result<Response> {
    // 路径参数: /users/:id
    let id = get_param(&event, "id");                   // Option<&str>
    let id = get_param_required(&event, "id")?;          // &str（缺少则返回 400）

    // 查询参数: /search?q=rust&page=2
    let q = get_query_param(&event, "q");                // Option<String>
    let all_query = get_query(&event);                   // &HashMap<String, String>

    // 请求体（JSON）
    let body: MyStruct = get_body(&event, &bytes)?;      // 反序列化后的结构体

    // 请求头
    let auth = get_header(&event, "authorization");      // Option<String>

    // 元信息
    let method = get_method(&event);                     // &Method
    let path = get_path(&event);                         // &str

    // 应用状态
    let db = get_state::<DatabasePool>(&event)?;         // 你的自定义状态

    json(json!({ "ok": true }))
}
```

---

## 响应辅助函数

```rust
// JSON（application/json）
json(json!({ "key": "value" }))?

// 纯文本（text/plain）
text("Hello!")

// HTML（text/html）
html("<h1>Hello</h1>")

// 重定向（302 Found）
redirect("/login")?

// 无内容（204 No Content）
no_content()

// 原始字节
bytes(vec![0x89, 0x50, 0x4E, 0x47]).content_type("image/png")

// 流式响应
stream(Body::from_stream(my_stream))
```

所有响应都支持链式调用：

```rust
json(data)?
    .status(StatusCode::CREATED)
    .header("X-Request-Id", "abc123")
```

---

## 错误处理

自然地返回错误——它们会自动变成合适的 HTTP 响应：

```rust
#[route]
pub async fn handler(event: Event) -> Result<Response> {
    let id = get_param_required(&event, "id")?;       // 缺少则返回 400

    if id == "0" {
        return Err(RouteError::not_found("用户不存在"));  // 404
    }

    // 第三方错误通过 anyhow 自动转换为 500
    let data = some_fallible_operation()?;

    json(data)
}
```

内置错误变体：

| 方法 | 状态码 |
|---|---|
| `RouteError::bad_request(msg)` | 400 |
| `RouteError::unauthorized(msg)` | 401 |
| `RouteError::forbidden(msg)` | 403 |
| `RouteError::not_found(msg)` | 404 |
| `RouteError::conflict(msg)` | 409 |
| `RouteError::validation(msg)` | 422 |
| `RouteError::rate_limit(msg)` | 429 |
| `RouteError::custom(StatusCode, msg)` | 任意 |
| 对任何兼容 `anyhow` 的错误使用 `?` | 500 |

所有错误以 JSON 格式返回：`{"error": "...", "status": 404}`。

---

## 中间件

在 `src/routes/` 目录的任意位置创建 `_middleware.rs` 文件。它的作用范围是所在文件夹 + 所有子文件夹。

```
src/routes/
├── _middleware.rs            # 作用于所有路由
├── api/
│   ├── _middleware.rs        # 作用于 /api/*（叠加在根中间件上）
│   ├── users.get.rs          # ← 根 + api 中间件
│   └── public/
│       ├── _middleware.rs    # 覆盖父中间件
│       └── health.get.rs    # ← 仅 public 中间件
```

### 叠加模式（默认）— 在父中间件之上叠加

```rust
// src/routes/_middleware.rs
use astrea::middleware::*;

pub fn middleware() -> Middleware {
    Middleware::new()
        .wrap(|router| {
            router
                .layer(tower_http::trace::TraceLayer::new_for_http())
                .layer(tower_http::cors::CorsLayer::permissive())
        })
}
```

### 覆盖模式 — 替换父中间件

```rust
// src/routes/api/public/_middleware.rs
use astrea::middleware::*;

pub fn middleware() -> Middleware {
    Middleware::override_parent()
        .wrap(|router| {
            router.layer(tower::limit::ConcurrencyLimitLayer::new(100))
        })
}
```

---

## OpenAPI（可选）

启用 `openapi` feature 自动生成 API 文档：

```toml
[dependencies]
astrea = { version = "0.0.1", features = ["openapi"] }
```

然后合并 OpenAPI 路由：

```rust
let app = routes::create_router()
    .merge(astrea::openapi::router("My API", "1.0.0"));
```

这会给你：
- `GET /openapi.json` — OpenAPI 3.0 规范
- `GET /swagger` — Swagger UI 页面

---

## 应用状态

在处理函数间共享状态（数据库连接池、配置等）：

```rust
#[derive(Clone)]
struct AppState {
    db: DatabasePool,
}

// 在处理函数中：
#[route]
pub async fn handler(event: Event) -> Result<Response> {
    let state = get_state::<AppState>(&event)?;
    // 使用 state.db ...
}
```

---

## 完整示例

```
my-api/
├── Cargo.toml
└── src/
    ├── main.rs
    └── routes/
        ├── _middleware.rs
        ├── index.get.rs
        └── api/
            ├── _middleware.rs
            ├── users.get.rs
            ├── users.post.rs
            └── users/
                ├── [id].get.rs
                ├── [id].put.rs
                └── [id].delete.rs
```

这会生成：
- `GET /` — 根页面
- `GET /api/users` — 获取用户列表
- `POST /api/users` — 创建用户
- `GET /api/users/:id` — 获取单个用户
- `PUT /api/users/:id` — 更新用户
- `DELETE /api/users/:id` — 删除用户

根中间件 → 所有路由。API 中间件 → `/api/*` 路由。

---

## 为什么选择 Astrea？

| | Astrea | 原生 Axum |
|---|---|---|
| **路由定义** | 放一个文件 | 手动写 `.route()` |
| **处理函数签名** | 永远是 `(Event) -> Result<Response>` | 随提取器组合变化 |
| **参数访问** | `get_param(&event, "id")` | `Path(id): Path<String>` |
| **错误处理** | 内置 JSON 错误响应 | 自己实现 |
| **中间件** | 基于文件的作用域 | 手动嵌套 |
| **OpenAPI** | 自动生成 | 手动写或用第三方库 |

---

## 最低支持 Rust 版本

Rust **1.85** 或更高版本（edition 2024）。

## 许可证

MIT © [TNXG (Asahi Shiori)](https://github.com/TNXG)
