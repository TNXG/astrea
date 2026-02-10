# Astrea 测试文档

本项目包含完整的测试套件，覆盖所有用户可能使用的功能。

## 测试统计

### 总览
- **总测试数**: 154个
- **通过率**: 100%
- **测试文件**: 7个

### 各模块测试详情

#### 1. Event 模块测试 (`tests/event_tests.rs`)
**测试数量**: 19个

测试覆盖范围：
- ✅ Event 创建和基本属性访问
- ✅ 路径参数的获取和访问
- ✅ 查询参数的解析和访问
- ✅ 请求头的访问和处理
- ✅ URI 和路径的获取
- ✅ 多种 HTTP 方法支持
- ✅ JSON 数据解析（成功和失败场景）
- ✅ 文本数据解析（包括 UTF-8 验证）
- ✅ Form 数据解析（成功和失败场景）
- ✅ 应用状态的存储和读取
- ✅ 状态类型安全和错误处理
- ✅ Event 对象的克隆
- ✅ 空查询字符串处理
- ✅ 复杂查询参数解析

#### 2. Extract 模块测试 (`tests/extract_tests.rs`)
**测试数量**: 26个

测试覆盖范围：
- ✅ `get_param()` - 获取路径参数
- ✅ `get_param_required()` - 获取必需的路径参数及错误处理
- ✅ `get_query()` - 获取所有查询参数
- ✅ `get_query_param()` - 获取单个查询参数
- ✅ `get_query_param_required()` - 获取必需的查询参数及错误处理
- ✅ `get_body()` - JSON 请求体解析（成功和失败场景）
- ✅ `get_body_bytes()` - 获取原始字节数据
- ✅ `get_body_text()` - 文本数据解析及 UTF-8 验证
- ✅ `get_header()` - 获取请求头（大小写不敏感）
- ✅ `get_headers()` - 获取所有请求头
- ✅ `get_state()` - 获取应用状态及错误处理
- ✅ `get_method()` - 获取 HTTP 方法
- ✅ `get_path()` - 获取请求路径
- ✅ `get_uri()` - 获取 URI
- ✅ 组合场景测试（多个提取器一起使用）
- ✅ POST 请求与复杂 JSON 结构

#### 3. Response 模块测试 (`tests/response_tests.rs`)
**测试数量**: 41个（模块内）

测试覆盖范围：
- ✅ `json()` - JSON 响应创建
  - 简单对象、复杂结构体、数组、空对象
  - null、number、boolean、string 等基本类型
  - 嵌套结构
- ✅ `text()` - 文本响应
  - 简单文本、空文本、多行文本
  - Unicode 字符支持
  - String 类型转换
- ✅ `html()` - HTML 响应
  - 简单 HTML、完整页面、特殊字符
- ✅ `redirect()` - 重定向响应
  - 相对路径、绝对 URL、带查询参数
  - 无效 URL 错误处理
- ✅ `no_content()` - 204 响应
- ✅ `bytes()` - 二进制数据响应
- ✅ Response 链式调用
  - `status()` - 设置状态码
  - `header()` - 添加响应头
  - `content_type()` - 设置 Content-Type
  - 多个方法组合
- ✅ Response 默认值和创建
- ✅ 转换为 Axum Response
- ✅ 各种 HTTP 状态码
- ✅ 响应克隆
- ✅ 大数据体处理
- ✅ 响应头覆盖和错误处理

#### 4. Error 模块测试 (`tests/error_tests.rs`)
**测试数量**: 41个

测试覆盖范围：
- ✅ 所有错误类型的创建
  - `bad_request()` - 400 错误
  - `not_found()` - 404 错误
  - `unauthorized()` - 401 错误
  - `forbidden()` - 403 错误
  - `validation()` - 422 验证错误
  - `custom()` - 自定义状态码错误
  - `MethodNotAllowed` - 405 错误
  - `Conflict` - 409 冲突错误
  - `RateLimit` - 429 限流错误
  - `Internal` - 500 内部错误
- ✅ 错误转换
  - 从 `anyhow::Error` 自动转换
  - `?` 操作符支持
- ✅ 错误消息格式化和显示
- ✅ `IntoResponse` 转换
- ✅ 状态码映射
- ✅ 错误消息提取
- ✅ `Result<T>` 类型别名使用
- ✅ 复杂错误链和上下文
- ✅ 多种错误类型组合使用
- ✅ 动态错误消息生成
- ✅ 多行错误消息
- ✅ Unicode 字符支持
- ✅ 错误调试格式
- ✅ 真实场景模拟
  - 认证检查
  - 权限验证
  - 资源冲突

#### 5. 集成测试 (`tests/integration_tests.rs`)
**测试数量**: 18个

测试覆盖范围：
- ✅ 完整的处理器流程
  - 简单 GET 请求处理
  - POST 请求与 JSON 请求体
  - 查询参数处理
  - 请求头验证
- ✅ 应用状态管理
- ✅ 错误传播和处理
- ✅ 验证逻辑
- ✅ RESTful API 场景
  - 列表资源（带分页）
  - 获取单个资源
  - 删除资源
  - 权限保护
- ✅ 不同响应类型
  - JSON、Text、HTML
  - 重定向
- ✅ 复杂业务逻辑
  - 多重验证
  - 状态检查
  - 文件上传模拟
- ✅ 中间件风格处理
- ✅ 边界情况
  - 空响应体
  - 自定义状态码
  - 多个响应头

#### 6. 宏测试 (`tests/macro_test.rs`)
**测试数量**: 1个
- ✅ `#[route]` 宏编译测试

#### 7. 模块内置测试
**测试数量**: 8个 (extract.rs) + 41个 (response.rs)
- ✅ Extract 模块单元测试
- ✅ Response 模块单元测试

## 测试覆盖的用户场景

### 1. 路由处理器开发
- ✅ 定义处理器函数
- ✅ 访问请求数据
- ✅ 返回各种响应类型
- ✅ 错误处理和传播

### 2. 请求数据提取
- ✅ 路径参数（`/users/:id`）
- ✅ 查询参数（`?page=1&limit=10`）
- ✅ 请求头（Authorization 等）
- ✅ JSON 请求体
- ✅ 文本和二进制数据

### 3. 响应构建
- ✅ JSON API 响应
- ✅ HTML 页面渲染
- ✅ 纯文本响应
- ✅ 重定向
- ✅ 自定义响应头
- ✅ 自定义状态码

### 4. 错误处理
- ✅ 标准 HTTP 错误
- ✅ 自定义错误
- ✅ 验证错误
- ✅ 第三方错误集成（anyhow）
- ✅ 错误到响应的自动转换

### 5. 状态管理
- ✅ 应用级共享状态
- ✅ 类型安全的状态访问
- ✅ 状态错误处理

### 6. RESTful API
- ✅ CRUD 操作
- ✅ 分页
- ✅ 过滤和查询
- ✅ 认证和授权
- ✅ 资源冲突处理

## 运行测试

### 运行所有测试
```bash
cargo test --lib --tests
```

### 运行特定模块测试
```bash
# Event 模块测试
cargo test --test event_tests

# Extract 模块测试
cargo test --test extract_tests

# Response 模块测试
cargo test --test response_tests

# Error 模块测试
cargo test --test error_tests

# 集成测试
cargo test --test integration_tests
```

### 运行单个测试
```bash
cargo test test_simple_handler_flow
```

### 查看测试输出
```bash
cargo test -- --nocapture
```

### 运行测试并显示详细信息
```bash
cargo test -- --test-threads=1 --nocapture
```

## 测试质量指标

- ✅ **功能覆盖**: 100% - 所有公开 API 都有测试
- ✅ **边界情况**: 覆盖空值、错误输入、大数据等
- ✅ **错误路径**: 所有错误情况都有对应测试
- ✅ **集成场景**: 测试真实使用场景
- ✅ **类型安全**: 利用 Rust 类型系统确保正确性

## 持续改进

测试套件会随着框架的发展持续更新：
- 新功能添加时补充相应测试
- 发现 bug 时添加回归测试
- 用户反馈场景补充集成测试

## 贡献指南

添加新功能时，请确保：
1. 为新的公开 API 添加单元测试
2. 添加边界情况和错误处理测试
3. 如果是重要功能，添加集成测试
4. 确保所有测试通过
5. 更新本文档

---

最后更新: 2026-02-10
测试框架: Rust 内置测试 + tokio::test
