<p align="center">
  <h1 align="center">Astrea</h1>
  <p align="center">
    A file-system based routing framework for <a href="https://github.com/tokio-rs/axum">Axum</a>.
    <br />
    Inspired by <a href="https://nitro.unjs.io/">Nitro</a> and <a href="https://h3.unjs.io/">H3</a>.
  </p>
</p>

<p align="center">
  <a href="https://crates.io/crates/astrea"><img src="https://img.shields.io/crates/v/astrea.svg" alt="crates.io" /></a>
  <a href="https://docs.rs/astrea"><img src="https://docs.rs/astrea/badge.svg" alt="docs.rs" /></a>
  <a href="https://github.com/TNXG/astrea/blob/main/LICENSE"><img src="https://img.shields.io/crates/l/astrea.svg" alt="MIT License" /></a>
  <a href="https://deepwiki.com/TNXG/astrea"><img src="https://img.shields.io/badge/DeepWiki-Astrea-blue.svg" alt="DeepWiki" /></a>
</p>

<p align="center">
  <a href="README.zh-CN.md">简体中文</a>
</p>

---

## What Is Astrea?

Astrea turns your **file structure** into **API routes** — at compile time, with zero runtime cost. Drop a `.rs` file into the `src/routes/` folder, and it becomes an HTTP endpoint. No manual route registration, no `build.rs`, no boilerplate.

Every handler looks the same:

```rust
#[route]
async fn handler(event: Event) -> Result<Response> {
    // your logic here
}
```

That's it. No complex extractor signatures. No learning curve for each parameter type.

## Features

- 📁 **File-based routing** — file name = route path, generated at compile time
- 🎯 **Unified handler signature** — every handler is `async fn(Event) -> Result<Response>`
- 🔧 **Simple extractors** — `get_param()`, `get_query_param()`, `get_body()` — just call a function
- 🧅 **Scoped middleware** — `_middleware.rs` files with inherit (extend) or replace (override) modes
- 📝 **OpenAPI auto-gen** — optional Swagger UI + OpenAPI 3.0 spec from your code (feature flag `openapi`)
- 🔄 **Axum compatible** — works with all existing Axum middleware and the Tower ecosystem
- 📦 **Zero extra deps** — re-exports `axum`, `tokio`, `serde`, `tower`, etc. — just depend on `astrea`

## Quick Start

### 1. Create a new project

```bash
cargo new my-api
cd my-api
```

### 2. Add Astrea

```bash
cargo add astrea
```

Or in `Cargo.toml`:

```toml
[package]
name = "my-api"
edition = "2024"

[dependencies]
astrea = "0.0.1"
```

> **Note:** Astrea requires Rust edition 2024 (Rust ≥ 1.85).

### 3. Create your route files

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

### 4. Write `main.rs`

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

### 5. Run

```bash
cargo run
```

Done. You will see a beautiful startup log:

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

And `GET http://localhost:3000/` returns `{"message":"Hello, World!"}`.

---

## Route File Naming Convention

| File name | Route |
|---|---|
| `src/routes/index.get.rs` | `GET /` |
| `src/routes/users.get.rs` | `GET /users` |
| `src/routes/users/index.post.rs` | `POST /users` |
| `src/routes/users/[id].get.rs` | `GET /users/:id` |
| `src/routes/users/[id].delete.rs` | `DELETE /users/:id` |
| `src/routes/posts/[...slug].get.rs` | `GET /posts/*slug` (catch-all) |

**Rules:**
- File name format: `<name>.<method>.rs`
- `index` is a special name — it maps to the directory itself (no extra path segment)
- `[param]` → dynamic path parameter
- `[...param]` → catch-all parameter (matches everything after)

---

## Extracting Request Data

Astrea replaces complex Axum extractor signatures with simple function calls:

```rust
#[route]
pub async fn handler(event: Event, bytes: Bytes) -> Result<Response> {
    // Path parameters: /users/:id
    let id = get_param(&event, "id");                   // Option<&str>
    let id = get_param_required(&event, "id")?;          // &str (or 400 error)

    // Query parameters: /search?q=rust&page=2
    let q = get_query_param(&event, "q");                // Option<String>
    let all_query = get_query(&event);                   // &HashMap<String, String>

    // Request body (JSON)
    let body: MyStruct = get_body(&event, &bytes)?;      // deserialized struct

    // Headers
    let auth = get_header(&event, "authorization");      // Option<String>

    // Metadata
    let method = get_method(&event);                     // &Method
    let path = get_path(&event);                         // &str

    // Application state
    let db = get_state::<DatabasePool>(&event)?;         // your custom state

    json(json!({ "ok": true }))
}
```

---

## Response Helpers

```rust
// JSON (application/json)
json(json!({ "key": "value" }))?

// Plain text (text/plain)
text("Hello!")

// HTML (text/html)
html("<h1>Hello</h1>")

// Redirect (302 Found)
redirect("/login")?

// No Content (204)
no_content()

// Raw bytes
bytes(vec![0x89, 0x50, 0x4E, 0x47]).content_type("image/png")

// Streaming
stream(Body::from_stream(my_stream))
```

All responses support chaining:

```rust
json(data)?
    .status(StatusCode::CREATED)
    .header("X-Request-Id", "abc123")
```

---

## WebSockets & Server-Sent Events (SSE)

Astrea supports WebSockets and SSE natively via route attributes. Simply use `#[route(ws)]` or `#[route(sse)]` instead of `#[route]`.

### WebSockets (`#[route(ws)]`)

```rust
use astrea::prelude::*;
use astrea::ws::{WebSocket, Message};

#[route(ws)]
pub async fn handler(event: Event, mut socket: WebSocket) {
    // Receive and echo messages
    while let Some(Ok(msg)) = socket.recv().await {
        if let Message::Text(text) = msg {
            let _ = socket.send(Message::from(format!("Echo: {}", text.as_str()))).await;
        }
    }
}
```

### Server-Sent Events (`#[route(sse)]`)

```rust
use astrea::prelude::*;
use astrea::sse::{SseSender, SseEvent};
use std::time::Duration;

#[route(sse)]
pub async fn handler(event: Event, sender: SseSender) {
    let _ = sender.send(
        SseEvent::new()
            .event("greeting")
            .data("Hello from SSE!")
            .retry(Duration::from_secs(5))
    ).await;
}
```

---

## Error Handling

Return errors naturally — they become proper HTTP responses automatically:

```rust
#[route]
pub async fn handler(event: Event) -> Result<Response> {
    let id = get_param_required(&event, "id")?;       // 400 if missing

    if id == "0" {
        return Err(RouteError::not_found("User not found"));  // 404
    }

    // Third-party errors auto-convert to 500 via anyhow
    let data = some_fallible_operation()?;

    json(data)
}
```

Built-in error variants:

| Method | Status Code |
|---|---|
| `RouteError::bad_request(msg)` | 400 |
| `RouteError::unauthorized(msg)` | 401 |
| `RouteError::forbidden(msg)` | 403 |
| `RouteError::not_found(msg)` | 404 |
| `RouteError::conflict(msg)` | 409 |
| `RouteError::validation(msg)` | 422 |
| `RouteError::rate_limit(msg)` | 429 |
| `RouteError::custom(StatusCode, msg)` | any |
| `?` on any `anyhow`-compatible error | 500 |

All errors are returned as JSON: `{"error": "...", "status": 404}`.

---

## Middleware

Create `_middleware.rs` files anywhere in the `src/routes/` directory. They scope to the folder they live in + all subfolders.

```
src/routes/
├── _middleware.rs            # applies to ALL routes
├── api/
│   ├── _middleware.rs        # applies to /api/* (stacks on root)
│   ├── users.get.rs          # ← root + api middleware
│   └── public/
│       ├── _middleware.rs    # OVERRIDES parent middleware
│       └── health.get.rs    # ← public middleware only
```

### Extend mode (default) — stack on parent

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

### Override mode — replace parent middleware

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

## OpenAPI (Optional)

Enable the `openapi` feature to get automatic API documentation:

```toml
[dependencies]
astrea = { version = "0.0.1", features = ["openapi"] }
```

Then merge the OpenAPI router:

```rust
let app = routes::create_router()
    .merge(astrea::openapi::router("My API", "1.0.0"));
```

This gives you:
- `GET /openapi.json` — the OpenAPI 3.0 spec
- `GET /swagger` — Swagger UI

---

## Application State

Share state across handlers (database pools, config, etc.):

```rust
#[derive(Clone)]
struct AppState {
    db: DatabasePool,
}

// In handler:
#[route]
pub async fn handler(event: Event) -> Result<Response> {
    let state = get_state::<AppState>(&event)?;
    // use state.db ...
}
```

---

## Full Example

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

This generates:
- `GET /` — root page
- `GET /api/users` — list users
- `POST /api/users` — create user
- `GET /api/users/:id` — get user by ID
- `PUT /api/users/:id` — update user
- `DELETE /api/users/:id` — delete user

Root middleware → all routes. API middleware → `/api/*` routes.

---

## Why Astrea?

| | Astrea | Plain Axum |
|---|---|---|
| **Route definition** | Drop a file | Manual `.route()` calls |
| **Handler signature** | Always `(Event) -> Result<Response>` | Varies per extractor combo |
| **Parameter access** | `get_param(&event, "id")` | `Path(id): Path<String>` |
| **Error handling** | Built-in JSON errors | DIY |
| **Middleware** | File-based scoping | Manual nesting |
| **OpenAPI** | Auto-generated | Manual or third-party |

---

## AI Agent Support

Astrea provides a built-in guide for AI coding assistants. If you are using an AI agent (like Copilot, Cursor, or Claude) to help you build your application, point them to the [`agent.md`](./agent.md) file in the root of the repository. It contains framework-specific rules, architectural context, and coding conventions to ensure your AI assistant writes idiomatic Astrea code.

---

## Minimum Supported Rust Version

Rust **1.85** or later (edition 2024).

## License

MIT © [TNXG (Asahi Shiori)](https://github.com/TNXG)
