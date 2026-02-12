//! OpenAPI metadata types
//!
//! / OpenAPI 元数据类型

/// Where a parameter is located in the request
///
/// / 参数在请求中的位置
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParamLocation {
    /// Path parameter (e.g., `/users/{id}`)
    /// / 路径参数（如 `/users/{id}`）
    Path,
    /// Query parameter (e.g., `?page=1`)
    /// / 查询参数（如 `?page=1`）
    Query,
}

/// Metadata about a single operation parameter
///
/// / 单个操作参数的元数据
#[derive(Debug, Clone)]
pub struct ParamMeta {
    /// Parameter name
    /// / 参数名
    pub name: String,
    /// Where the parameter is located
    /// / 参数位置
    pub location: ParamLocation,
    /// Whether the parameter is required
    /// / 参数是否必需
    pub required: bool,
    /// OpenAPI schema type: "string", "integer", "number", "boolean"
    /// / OpenAPI 模式类型
    pub schema_type: String,
    /// OpenAPI schema format: "uint32", "int64", "float", etc.
    /// / OpenAPI 模式格式
    pub schema_format: Option<String>,
}

/// Metadata about a request body
///
/// / 请求体元数据
#[derive(Debug, Clone)]
pub struct RequestBodyMeta {
    /// Content type, e.g., "application/json"
    /// / 内容类型
    pub content_type: String,
    /// Rust type name used as schema reference, e.g., "CreateUserRequest"
    /// / 用作 schema 引用的 Rust 类型名
    pub schema_type_name: String,
}

/// Metadata extracted from a handler function by the `#[route]` macro
///
/// / 由 `#[route]` 宏从处理函数中提取的元数据
///
/// Contains information gathered from AST analysis and doc comment annotations.
///
/// 包含从 AST 分析和文档注释标注中收集的信息。
#[derive(Debug, Clone, Default)]
pub struct HandlerMeta {
    /// Operation summary (from `@summary` or auto-detected first doc line)
    /// / 操作摘要（来自 `@summary` 或自动检测的首行文档）
    pub summary: Option<String>,
    /// Operation description (from `@description` or remaining plain doc lines)
    /// / 操作描述（来自 `@description` 或剩余的普通文档行）
    pub description: Option<String>,
    /// Operation tags (from `@tag` doc annotations)
    /// / 操作标签（来自 `@tag` 文档标注）
    pub tags: Vec<String>,
    /// Security requirements (from `@security` doc annotations)
    /// / 安全要求（来自 `@security` 文档标注）
    pub security: Vec<String>,
    /// Parameters extracted from handler body
    /// / 从处理函数体中提取的参数
    pub parameters: Vec<ParamMeta>,
    /// Request body metadata (from `get_body::<T>()` detection)
    /// / 请求体元数据（从 `get_body::<T>()` 检测得到）
    pub request_body: Option<RequestBodyMeta>,
    /// Response content type inferred from response builder calls
    /// / 从响应构建器调用推断的响应内容类型
    pub response_content_type: String,
    /// Top-level field names extracted from `json!({...})` macros
    /// / 从 `json!({...})` 宏提取的顶层字段名
    pub response_schema_fields: Vec<String>,
    /// Whether the operation is deprecated (from `@deprecated` doc annotation)
    /// / 操作是否已弃用（来自 `@deprecated` 文档标注）
    pub deprecated: bool,
    /// Additional response descriptions: `(status_code, description)`
    /// / 额外的响应描述：`(状态码, 描述)`
    ///
    /// From `@response 404 Not found` doc annotations.
    /// / 来自 `@response 404 Not found` 文档标注。
    pub responses: Vec<(String, String)>,
}

/// A fully resolved route entry combining file-path info with handler metadata
///
/// / 组合了文件路径信息和处理函数元数据的完整路由条目
#[derive(Debug, Clone)]
pub struct RouteEntry {
    /// HTTP method (GET, POST, PUT, DELETE, etc.)
    /// / HTTP 方法
    pub method: String,
    /// OpenAPI-format path (e.g., `/users/{id}`)
    /// / OpenAPI 格式路径
    pub path: String,
    /// Operation ID derived from module name
    /// / 从模块名派生的操作 ID
    pub operation_id: String,
    /// Handler metadata extracted by the `#[route]` macro
    /// / 由 `#[route]` 宏提取的处理函数元数据
    pub handler_meta: HandlerMeta,
}
