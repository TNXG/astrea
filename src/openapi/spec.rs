//! OpenAPI 3.0 specification generation
//!
//! / OpenAPI 3.0 规范生成

use serde_json::{Value, json};

use super::registry::get_entries;
use super::types::*;

/// Generate an OpenAPI 3.0.3 specification document
///
/// / 生成 OpenAPI 3.0.3 规范文档
pub fn generate_spec(title: &str, version: &str) -> Value {
    let entries = get_entries();
    let mut paths = serde_json::Map::new();

    for entry in &entries {
        let path_item = paths
            .entry(&entry.path)
            .or_insert_with(|| Value::Object(serde_json::Map::new()));

        let method_key = entry.method.to_lowercase();
        let operation = build_operation(entry);

        if let Value::Object(map) = path_item {
            map.insert(method_key, operation);
        }
    }

    // Check if any route uses bearer security
    // 检查是否有路由使用 bearer 安全性
    let has_bearer = entries
        .iter()
        .any(|e| e.handler_meta.security.contains(&"bearer".to_string()));

    let mut spec = json!({
        "openapi": "3.0.3",
        "info": {
            "title": title,
            "version": version,
        },
        "paths": paths,
    });

    // Add securitySchemes if bearer is used
    // 如果使用了 bearer，添加安全方案
    if has_bearer {
        spec["components"] = json!({
            "securitySchemes": {
                "bearerAuth": {
                    "type": "http",
                    "scheme": "bearer",
                    "bearerFormat": "JWT",
                }
            }
        });
    }

    // Add request body type references as component schemas (placeholder)
    // 添加请求体类型引用作为组件模式（占位符）
    let body_types: Vec<String> = entries
        .iter()
        .filter_map(|e| e.handler_meta.request_body.as_ref())
        .map(|b| b.schema_type_name.clone())
        .collect();

    if !body_types.is_empty() {
        let components = spec.get_mut("components").and_then(|c| c.as_object_mut());

        let components = if let Some(c) = components {
            c
        } else {
            spec["components"] = json!({});
            spec["components"].as_object_mut().unwrap()
        };

        let mut schemas = serde_json::Map::new();
        for type_name in body_types {
            schemas.entry(type_name).or_insert_with(|| {
                json!({
                    "type": "object",
                    "description": "Auto-detected request body type (schema details require manual definition or a derive macro)",
                })
            });
        }
        components.insert("schemas".to_string(), Value::Object(schemas));
    }

    spec
}

/// Build an OpenAPI operation object for a single route entry
///
/// / 为单个路由条目构建 OpenAPI 操作对象
fn build_operation(entry: &RouteEntry) -> Value {
    let meta = &entry.handler_meta;
    let mut operation = serde_json::Map::new();

    // operationId
    operation.insert("operationId".to_string(), json!(entry.operation_id));

    // summary
    if let Some(s) = &meta.summary {
        operation.insert("summary".to_string(), json!(s));
    }

    // description
    if let Some(d) = &meta.description {
        operation.insert("description".to_string(), json!(d));
    }

    // tags
    if !meta.tags.is_empty() {
        operation.insert("tags".to_string(), json!(meta.tags));
    }

    // deprecated
    if meta.deprecated {
        operation.insert("deprecated".to_string(), json!(true));
    }

    // parameters
    if !meta.parameters.is_empty() {
        let params: Vec<Value> = meta
            .parameters
            .iter()
            .map(|p| {
                let location = match p.location {
                    ParamLocation::Path => "path",
                    ParamLocation::Query => "query",
                };

                let mut schema = serde_json::Map::new();
                schema.insert("type".to_string(), json!(p.schema_type));
                if let Some(fmt) = &p.schema_format {
                    schema.insert("format".to_string(), json!(fmt));
                }

                json!({
                    "name": p.name,
                    "in": location,
                    "required": p.required,
                    "schema": Value::Object(schema),
                })
            })
            .collect();
        operation.insert("parameters".to_string(), json!(params));
    }

    // requestBody
    if let Some(body) = &meta.request_body {
        operation.insert(
            "requestBody".to_string(),
            json!({
                "required": true,
                "content": {
                    &body.content_type: {
                        "schema": {
                            "$ref": format!("#/components/schemas/{}", body.schema_type_name),
                        }
                    }
                }
            }),
        );
    }

    // responses
    build_responses(meta, &mut operation);

    // security
    if !meta.security.is_empty() {
        let sec: Vec<Value> = meta
            .security
            .iter()
            .map(|s| match s.as_str() {
                "bearer" => json!({ "bearerAuth": [] }),
                other => json!({ other: [] }),
            })
            .collect();
        operation.insert("security".to_string(), json!(sec));
    }

    Value::Object(operation)
}

/// Build the responses section of an operation
///
/// / 构建操作的 responses 部分
fn build_responses(meta: &HandlerMeta, operation: &mut serde_json::Map<String, Value>) {
    let ct = &meta.response_content_type;
    let mut responses = serde_json::Map::new();

    if ct.is_empty() || ct == "none" {
        // 204 No Content
        responses.insert("204".to_string(), json!({ "description": "No Content" }));
    } else {
        // Build response schema from detected json!() fields
        // 从检测到的 json!() 字段构建响应模式
        let response_schema = if !meta.response_schema_fields.is_empty() {
            let props: serde_json::Map<String, Value> = meta
                .response_schema_fields
                .iter()
                .map(|k| (k.clone(), json!({})))
                .collect();
            json!({
                "type": "object",
                "properties": Value::Object(props),
            })
        } else {
            json!({})
        };

        responses.insert(
            "200".to_string(),
            json!({
                "description": "Successful response",
                "content": {
                    ct: {
                        "schema": response_schema,
                    }
                }
            }),
        );
    }

    // Additional responses from @response annotations
    // 来自 @response 标注的额外响应
    for (code, desc) in &meta.responses {
        responses.insert(code.clone(), json!({ "description": desc }));
    }

    operation.insert("responses".to_string(), Value::Object(responses));
}
