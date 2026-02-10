# Astrea

[![Crates.io](https://img.shields.io/crates/v/astrea)](https://crates.io/crates/astrea)
[![Documentation](https://docs.rs/astrea/badge.svg)](https://docs.rs/astrea)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![Axum](https://img.shields.io/badge/axum-0.8%2B-blue.svg)](https://github.com/tokio-rs/axum)

> 基于 Axum 的文件系统路由框架，受 [Nitro](https://nitro.unjs.io/) 和 [H3](https://h3.unjs.io/) 启发。

**[crates.io](https://crates.io/crates/astrea)** | **[GitHub](https://github.com/TNXG/astrea)** | **[文档](https://docs.rs/astrea)**

## 特性

- **简单统一的函数签名** - 所有处理函数遵循相同模式：`async fn handler(event: Event) -> Result<Response>`
- **声明式参数提取** - 通过辅助函数访问请求数据，无需复杂的提取器
- **基于文件的路由** - 根据文件系统结构自动生成路由
- **类型安全** - 完整的 Rust 类型安全，编译时生成路由
- **兼容 Axum 生态** - 与 Axum 中间件和生态系统无缝协作
- **零运行时开销** - 通过过程宏在编译时生成路由

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
astrea = "0.0"
tokio = { version = "1", features = ["full"] }
```

或使用 cargo-edit：

```bash
cargo add astrea
```

## 快速开始

### 1. 创建路由文件

在项目根目录创建 `routes/` 目录并添加路由文件：

```
routes/
├── index.get.rs          # GET /
├── users/
│   ├── index.get.rs      # GET /users
│   └── [id].get.rs       # GET /users/:id
└── posts/
    └── index.post.rs     # POST /posts
```

### 2. 定义处理函数

```rust
// routes/index.get.rs
use astrea::prelude::*;

#[route]
pub async fn handler(_event: Event) -> Result<Response> {
    json(json!({
        "message": "Welcome to Astrea!",
        "version": "0.1.0"
    }))
}
```

```rust
// routes/users/[id].get.rs
use astrea::prelude::*;

#[route]
pub async fn handler(event: Event) -> Result<Response> {
    let user_id = get_param_required(&event, "id")?;
    json(json!({
        "user_id": user_id,
        "name": "Alice"
    }))
}
```

### 3. 生成路由并运行

```rust
// src/main.rs
#[allow(dead_code, unused_imports)]
mod routes {
    astrea::generate_routes!();
}

#[tokio::main]
async fn main() {
    let app = routes::create_router();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    tracing::info!("Listening on http://127.0.0.1:3000");

    astrea::serve(listener, app).await.unwrap();
}
```

## 路由文件命名规则

路由根据文件结构和命名生成：

| 文件模式 | 路由 | 方法 |
|--------------|-------|--------|
| `index.get.rs` | `/` | GET |
| `index.post.rs` | `/` | POST |
| `users.get.rs` | `/users` | GET |
| `users/[id].get.rs` | `/users/:id` | GET |
| `posts/[...slug].get.rs` | `/posts/*slug` | GET |

动态参数使用方括号：
- `[id]` → `:id` (单一路径参数)
- `[...slug]` → `*slug` (全捕获参数)

## 请求数据提取

Astrea 提供辅助函数访问请求数据：

```rust
use astrea::prelude::*;

#[route]
pub async fn handler(event: Event, bytes: Bytes) -> Result<Response> {
    // 路径参数 - 方式一：获取整个参数 Map
    let params = get_param(&event, "id"); // Option<&str>

    // 路径参数 - 方式二：直接获取必需参数（缺失时自动返回错误）
    let user_id = get_param_required(&event, "id")?;

    // 查询参数 - 方式一：获取整个查询 Map
    let query = get_query(&event);
    let search = query.get("q").cloned().unwrap_or_default();

    // 查询参数 - 方式二：直接获取单个查询参数
    let search = get_query_param(&event, "q"); // Option<String>

    // 请求头
    let auth = get_header(&event, "authorization");

    // JSON 请求体
    #[derive(Deserialize)]
    struct CreateUserRequest {
        name: String,
        email: String,
    }
    let body: CreateUserRequest = get_body(&event, &bytes)?;

    json(json!({ "user_id": user_id }))
}
```

## 响应辅助函数

```rust
use astrea::prelude::*;

// JSON 响应
json(json!({ "message": "Hello" }))?

// 文本响应
text("Hello, World!")

// HTML 响应
html("<h1>Hello</h1>")

// 重定向
redirect("/login")?

// 自定义状态码和响应头
json(json!({ "status": "created" }))?
    .status(StatusCode::CREATED)
    .header("X-Request-Id", "abc123")
```

## 示例

使用 `cargo run --example <name>` 运行示例:

```bash
# 简单的 hello world
cargo run --example hello

# 带路由参数的 JSON API
cargo run --example json_api

# 请求数据提取（查询参数、请求头、JSON body）
cargo run --example request_data
```

完整的应用示例请参见 `basic-app/` 目录。

## 项目结构

```
astrea/
├── astrea-macro/         # 过程宏
│   └── src/lib.rs        # #[route] 和 generate_routes! 宏
├── basic-app/            # 完整示例应用（独立项目）
│   └── src/routes/       # 基于文件的路由示例
├── examples/             # 简单代码示例
│   ├── hello.rs          # 最小 hello world
│   ├── json_api.rs       # 带参数的 JSON API
│   └── request_data.rs   # 请求数据提取
├── benches/              # 性能基准测试
├── src/
│   ├── lib.rs            # 主库导出
│   ├── event.rs          # Event 类型
│   ├── extract.rs        # 辅助提取函数
│   ├── response.rs       # 响应构建器
│   ├── error.rs          # 错误类型
│   └── router.rs         # 路由工具
└── tests/                # 集成测试
```

## 作者

**TNXG** (朝日 栞)

- GitHub: [@TNXG](https://github.com/TNXG)
- 博客: https://tnxg.moe
- 邮箱: tnxg@outlook.jp

## 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件。

## 致谢

- 灵感来源于 [Nitro](https://nitro.unjs.io/) 和 [H3](https://h3.unjs.io/)
- 构建于优秀的 [Axum](https://github.com/tokio-rs/axum) 框架之上
- Rust Web 生态系统的一部分
