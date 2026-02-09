#!/bin/bash
# 快速验证所有性能测试能够编译通过
# 用于 CI/CD 或开发过程中快速检查

set -e

echo "Checking all benchmarks compile..."
cargo check --benches

echo ""
echo "All benchmarks compiled successfully!"
echo ""
echo "To run full benchmarks:"
echo "  cargo bench"
echo ""
echo "To run specific benchmark:"
echo "  cargo bench --bench event"
echo "  cargo bench --bench response"
echo "  cargo bench --bench extract"
echo "  cargo bench --bench error"
echo "  cargo bench --bench comprehensive"
