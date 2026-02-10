# Astrea

[![Crates.io](https://img.shields.io/crates/v/astrea)](https://crates.io/crates/astrea)
[![Documentation](https://docs.rs/astrea/badge.svg)](https://docs.rs/astrea)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![Axum](https://img.shields.io/badge/axum-0.8%2B-blue.svg)](https://github.com/tokio-rs/axum)

> A file-system based routing framework for Axum, inspired by [Nitro](https://nitro.unjs.io/) and [H3](https://h3.unjs.io/).

**[crates.io](https://crates.io/crates/astrea)** | **[GitHub](https://github.com/TNXG/astrea)** | **[Documentation](https://docs.rs/astrea)**

[中文文档](README_ZH.md)

## Features

- **Simple, unified function signature** - All handlers follow the same pattern: `async fn handler(event: Event) -> Result<Response>`
- **Declarative parameter extraction** - Access request data through helper functions instead of complex extractors
- **File-based routing** - Routes are automatically generated from your filesystem structure
- **Type-safe** - Full Rust type safety with compile-time route generation
- **Axum ecosystem compatible** - Works seamlessly with Axum middleware and ecosystem
- **Zero runtime overhead** - Route generation happens at compile time via procedural macros

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
astrea = "0.0"
tokio = { version = "1", features = ["full"] }
```

Or use cargo-edit:

```bash
cargo add astrea
```

## Quick Start

### 1. Create your route files

Create a `routes/` directory in your project root and add route files:

```
routes/
├── index.get.rs          # GET /
├── users/
│   ├── index.get.rs      # GET /users
│   └── [id].get.rs       # GET /users/:id
└── posts/
    └── index.post.rs     # POST /posts
```

### 2. Define your handlers

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

### 3. Generate routes and run

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

## Route File Naming Convention

Routes are generated based on the file structure and naming:

| File Pattern | Route | Method |
|--------------|-------|--------|
| `index.get.rs` | `/` | GET |
| `index.post.rs` | `/` | POST |
| `users.get.rs` | `/users` | GET |
| `users/[id].get.rs` | `/users/:id` | GET |
| `posts/[...slug].get.rs` | `/posts/*slug` | GET |

Dynamic parameters use square brackets:
- `[id]` → `:id` (single path parameter)
- `[...slug]` → `*slug` (catch-all parameter)

## Request Data Extraction

Astrea provides helper functions to access request data:

```rust
use astrea::prelude::*;

#[route]
pub async fn handler(event: Event, bytes: Bytes) -> Result<Response> {
    // Path parameters - Approach 1: get optional param
    let params = get_param(&event, "id"); // Option<&str>

    // Path parameters - Approach 2: get required param (returns error if missing)
    let user_id = get_param_required(&event, "id")?;

    // Query parameters - Approach 1: get the full query map
    let query = get_query(&event);
    let search = query.get("q").cloned().unwrap_or_default();

    // Query parameters - Approach 2: get a single query param directly
    let search = get_query_param(&event, "q"); // Option<String>

    // Request headers
    let auth = get_header(&event, "authorization");

    // JSON body
    #[derive(Deserialize)]
    struct CreateUserRequest {
        name: String,
        email: String,
    }
    let body: CreateUserRequest = get_body(&event, &bytes)?;

    json(json!({ "user_id": user_id }))
}
```

## Response Helpers

```rust
use astrea::prelude::*;

// JSON response
json(json!({ "message": "Hello" }))?

// Text response
text("Hello, World!")

// HTML response
html("<h1>Hello</h1>")

// Redirect
redirect("/login")?

// Custom status and headers
json(json!({ "status": "created" }))?
    .status(StatusCode::CREATED)
    .header("X-Request-Id", "abc123")
```

## Examples

Run the examples with `cargo run --example <name>`:

```bash
# Simple hello world
cargo run --example hello

# JSON API with route parameters
cargo run --example json_api

# Request data extraction (query params, headers, JSON body)
cargo run --example request_data
```

For a complete application example, see the `basic-app/` directory.

## Project Structure

```
astrea/
├── astrea-macro/         # Procedural macros
│   └── src/lib.rs        # #[route] and generate_routes! macros
├── basic-app/            # Complete example application (independent project)
│   └── src/routes/       # File-based route examples
├── examples/             # Simple code examples
│   ├── hello.rs          # Minimal hello world
│   ├── json_api.rs       # JSON API with parameters
│   └── request_data.rs   # Request data extraction
├── benches/              # Performance benchmarks
├── src/
│   ├── lib.rs            # Main library exports
│   ├── event.rs          # Event type
│   ├── extract.rs        # Helper extraction functions
│   ├── response.rs       # Response builders
│   ├── error.rs          # Error types
│   └── router.rs         # Router utilities
└── tests/                # Integration tests
```

## Author

**TNXG** (Asahi Shiori)

- GitHub: [@TNXG](https://github.com/TNXG)
- Blog: https://tnxg.moe
- Email: tnxg@outlook.jp

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by [Nitro](https://nitro.unjs.io/) and [H3](https://h3.unjs.io/)
- Built on top of the excellent [Axum](https://github.com/tokio-rs/axum) framework
- Part of the Rust web ecosystem
