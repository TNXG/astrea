//! 测试 #[route] 宏

use astrea::prelude::*;

#[route]
async fn test_handler(event: Event) -> Result<Response> {
    json(serde_json::json!({
        "message": "Test handler works!",
        "path": event.path()
    }))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_macro_compiles() {
        // 如果宏能正确展开，这个测试应该能编译通过
        assert!(true);
    }
}
