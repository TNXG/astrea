# Astrea 性能测试

本目录包含 Astrea Web 框架的性能测试套件，使用 [Criterion.rs](https://github.com/bheisler/criterion.rs) 进行基准测试。

## 性能测试结果

基于 Apple Silicon (M系列) 的测试结果：

### Event 性能

| 操作 | 平均时间 | 说明 |
|------|----------|------|
| 空 Event 创建 | ~81 ns | 基础 Event 对象创建 |
| 带参数 Event 创建 | ~196 ns | 包含路径参数 |
| 带查询参数 Event 创建 | ~250 ns | 包含查询字符串 |
| Event 克隆 (Arc) | ~16 ns | 共享指针克隆 |
| 参数访问 | ~7 ns | 预填充参数获取 |
| 查询参数解析（懒加载） | ~0.8 ns | 已缓存时 |
| JSON 反序列化（小） | ~52 ns | 简单 JSON 对象 |
| JSON 反序列化（中） | ~510 ns | 嵌套 JSON 结构 |
| JSON 反序列化（大） | ~810 ns | 复杂 JSON 对象 |
| 文本解析 | ~24 ns | UTF-8 验证 |
| 状态访问 | ~0.77 ns | 应用状态获取 |

### Response 性能

| 操作 | 平均时间 | 说明 |
|------|----------|------|
| JSON 响应（小） | ~150 ns | `{"message":"ok"}` |
| JSON 响应（中） | ~520 ns | 10个对象的数组 |
| JSON 响应（大） | ~7.6 µs | 100个对象的数组 |
| 文本响应（短） | ~100 ns | 简单字符串 |
| HTML 响应 | ~100 ns | HTML 内容 |
| 重定向响应 | ~100 ns | URL 重定向 |
| 状态码设置 | ~50 ns | 链式调用 |

### Extract 性能

| 操作 | 平均时间 | 说明 |
|------|----------|------|
| get_param（存在） | ~8.8 ns | 获取路径参数 |
| get_param（缺失） | ~7.7 ns | 返回 None |
| get_query_param | ~8 ns | 获取查询参数 |
| get_header（存在） | ~8 ns | 获取请求头 |
| JSON body 解析 | ~50 ns | 反序列化请求体 |

### Error 性能

| 操作 | 平均时间 | 说明 |
|------|----------|------|
| 错误创建 | ~16-20 ns | 各种错误类型 |
| 错误消息获取 | ~20 ns | 获取错误文本 |
| 状态码映射 | ~0.5 ps | 获取 HTTP 状态码 |
| 错误转响应 | ~330 ns | 转换为 HTTP 响应 |
| 错误显示 | ~59 ns | 格式化输出 |
| anyhow 转换 | ~54 ns | anyhow::Error 转换 |
| 动态错误消息 | ~69 ns | 带上下文的错误 |

### 综合场景性能

| 场景 | 平均时间 | 说明 |
|------|----------|------|
| 用户列表处理 | ~4.5 µs | 分页查询 + JSON 响应 |
| 用户详情处理 | ~388 ns | 路径参数提取 |
| 创建用户处理 | ~390 ns | JSON body 解析 |
| 认证请求处理 | ~245 ns | Header 验证 |
| 完整请求生命周期 | ~507 ns | Event → 提取 → 响应 |
| 多请求处理 | ~341 ns | 单线程处理 4 个请求 |

## 性能特点

1. **Event 创建非常高效** - 空对象仅需 81ns，带参数也只需 200ns 左右
2. **参数访问极快** - 预填充参数访问 ~7ns，得益于 HashMap 缓存
3. **查询参数懒加载** - 首次访问后缓存，后续访问 < 1ns
4. **Arc 克销廉价** - Event 使用 Arc 共享，克隆仅 ~16ns
5. **JSON 处理高效** - 小 JSON 对象序列化 ~150ns
6. **错误处理开销小** - 错误创建 ~20ns，转响应 ~330ns

## 运行测试

### 运行所有性能测试

```bash
cargo bench
```

### 运行特定的性能测试

```bash
# 只运行 Event 性能测试
cargo bench --bench event

# 只运行 Response 性能测试
cargo bench --bench response

# 只运行 Extract 性能测试
cargo bench --bench extract

# 只运行 Error 性能测试
cargo bench --bench error

# 只运行综合场景测试
cargo bench --bench comprehensive
```

### 运行特定测试组

```bash
# 运行 event_access 测试组
cargo bench --bench event -- event_access

# 运行 json_response 测试组
cargo bench --bench response -- json_response

# 运行 get_param 测试组
cargo bench --bench extract -- get_param
```

### 查看输出格式

```bash
# 安静模式（只输出摘要）
cargo bench --quiet

# 详细输出
cargo bench --verbose

# 保存基线
cargo bench -- --save-baseline main

# 与基线对比
cargo bench -- --baseline main
```

## 测试文件概览

### 1. event.rs
测试 Event 类型的性能，包括：
- **创建性能** - 空 Event、带参数的 Event、带查询参数的 Event
- **克隆性能** - Arc 克隆性能
- **访问性能** - method、path、uri、headers 访问
- **参数访问** - 路径参数访问、缓存效果
- **查询解析** - 懒加载查询参数解析、缓存效果
- **JSON 解析** - 不同大小的 JSON 反序列化
- **文本解析** - UTF-8 文本解析
- **状态访问** - 应用状态获取

### 2. response.rs
测试响应构建的性能，包括：
- **JSON 响应** - 小型、中型、大型、嵌套 JSON
- **文本响应** - 不同大小的文本响应
- **HTML 响应** - 简单和复杂的 HTML
- **重定向响应** - 相对和绝对 URL 重定向
- **状态码** - 不同 HTTP 状态码
- **响应链式调用** - 多个 header 设置、状态码设置
- **Content-Type** - 自定义内容类型
- **字节响应** - 不同大小的二进制响应

### 3. extract.rs
测试提取函数的性能，包括：
- **参数提取** - `get_param`、`get_param_required`
- **查询参数提取** - `get_query_param`、`get_query`
- **请求头提取** - `get_header`、`get_headers`
- **请求体提取** - JSON、文本、字节
- **HTTP 方法/路径/URI** - 基本请求信息提取
- **组合提取** - 模拟真实场景的多字段提取

### 4. error.rs
测试错误处理的性能，包括：
- **错误创建** - 各种错误类型的创建
- **错误消息** - 消息获取、不同长度消息
- **状态码获取** - 错误状态码映射
- **响应转换** - 错误转换为 HTTP 响应
- **错误显示** - 格式化输出
- **错误转换** - anyhow::Error 转换
- **构建模式** - 直接构造 vs 构造函数
- **上下文错误** - 带上下文的错误创建
- **常见场景** - 验证、认证、权限等场景

### 5. comprehensive.rs
综合场景测试，模拟真实应用：
- **用户列表场景** - 带分页的列表查询
- **用户详情场景** - 路径参数提取
- **创建用户场景** - JSON 请求体解析
- **认证请求场景** - Header 验证
- **完整请求生命周期** - Event 创建 → 参数提取 → 响应构建
- **并发式处理** - 单线程处理多个请求
- **错误处理路径** - 成功路径 vs 各种错误路径
- **响应大小影响** - 不同大小响应的性能

## 测试结果

测试运行后，结果会保存在 `target/criterion/` 目录中。可以在浏览器中查看详细报告：

```bash
open target/criterion/report/index.html
```

报告包含：
- 每个测试的平均执行时间
- 不同运行之间的比较
- 性能趋势图表
- 与基线的对比

## 编写新的性能测试

要添加新的性能测试，参考以下模板：

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_my_function(c: &mut Criterion) {
    let mut group = c.benchmark_group("my_test_group");

    group.bench_function("my_test", |b| {
        b.iter(|| {
            // 使用 black_box 防止编译器优化掉测试代码
            black_box(my_function(black_box(test_input)))
        })
    });

    group.finish();
}

criterion_group!(benches, bench_my_function);
criterion_main!(benches);
```

## 性能测试最佳实践

1. **使用 `black_box`** - 防止编译器优化掉测试代码
2. **合理分组** - 使用 `benchmark_group` 组织相关测试
3. **输入多样性** - 测试不同大小的输入
4. **真实场景** - 尽量模拟真实使用场景
5. **可重复性** - 确保测试结果稳定可重复
6. **独立测试** - 每个测试应该独立运行

## CI/CD 集成

可以在 CI 中运行性能测试来检测性能退化：

```yaml
- name: Run benchmarks
  run: cargo bench -- --save-baseline main

- name: Compare with baseline
  run: cargo bench -- --baseline main
```

## 更多信息

- [Criterion.rs 文档](https://bheisler.github.io/criterion.rs/book/index.html)
- [Rust 性能测试指南](https://doc.rust-lang.org/1.81.0/unstable-book/library-features/test.html)
