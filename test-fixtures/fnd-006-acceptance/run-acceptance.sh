#!/bin/bash
# FND-006 快速验收脚本
# 在隔离环境下运行，验证核心安全属性

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TAURI_DIR="/Users/yoyi/workspace/product-line-syn-fnd-002/prototypes/productized-desktop-shell/src-tauri"

echo "=== FND-006 快速验收 ==="
echo "工作目录: $TAURI_DIR"
echo ""

cd "$TAURI_DIR"

# 1. 编译检查
echo "--- 1. 编译检查 ---"
cargo check --lib 2>&1 | grep -E "^error|generated.*warnings"
echo ""

# 2. 单元测试
echo "--- 2. 单元测试 ---"
cargo test --lib 2>&1 | grep "^test result:"
echo ""

# 3. grep 断言
echo "--- 3. grep 断言 ---"
echo "identity_kernel 调用: $(grep -rn 'identity_kernel::resolve_identity\|identity_kernel::IdentityResolution' src/ --include='*.rs' | grep -v 'identity_kernel.rs' | wc -l | tr -d ' ')"
echo "execution_grant 调用: $(grep -rn 'execution_grant::mint_grant\|execution_grant::verify_grant\|execution_grant::GrantMintInput\|execution_grant::GrantVerification' src/ --include='*.rs' | grep -v 'execution_grant.rs' | wc -l | tr -d ' ')"
echo "event_audit_boundary 调用: $(grep -rn 'event_audit_boundary::scrub_content\|event_audit_boundary::classify_content' src/ --include='*.rs' | grep -v 'event_audit_boundary.rs' | wc -l | tr -d ' ')"
echo ""

# 4. 路径守卫测试
echo "--- 4. 路径守卫测试 ---"
cargo test --lib path_guard 2>&1 | grep "^test result:"
echo ""

# 5. 身份内核测试
echo "--- 5. 身份内核测试 ---"
cargo test --lib identity_kernel 2>&1 | grep "^test result:"
echo ""

# 6. 执行授权测试
echo "--- 6. 执行授权测试 ---"
cargo test --lib execution_grant 2>&1 | grep "^test result:"
echo ""

# 7. 事件边界测试
echo "--- 7. 事件边界测试 ---"
cargo test --lib event_audit_boundary 2>&1 | grep "^test result:"
echo ""

# 8. worker_report 测试
echo "--- 8. worker_report 测试 ---"
cargo test --lib worker_report 2>&1 | grep "^test result:"
echo ""

# 9. FND-004A 测试
echo "--- 9. FND-004A 归属测试 ---"
cargo test --lib fnd004a_ownership_tests 2>&1 | grep "^test result:"
echo ""

echo "=== 验收完成 ==="
echo "如果所有测试通过，可以启动隔离 App 进行运行时验收。"
