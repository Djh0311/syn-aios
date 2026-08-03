# FND-006 隔离 App 验收指南

## 前置条件

1. 已安装 Rust + Node.js + Tauri CLI
2. 工作目录: `/Users/yoyi/workspace/product-line-syn-fnd-002/prototypes/productized-desktop-shell`
3. 使用隔离 profile（不碰真实 store/项目/凭据）

## 验收场景（8 类）

### 场景 1: 跨项目访问拒绝
- **操作**: 用 Project A 的 dispatch 尝试读写 Project B 的 workflow
- **预期**: 返回 `fnd004a_rejected` 或 `identity_kernel_rejected`
- **验证命令**:
```bash
# 启动隔离 App 后，在控制台执行
# 用 Project A 的 project_root 尝试加载 Project B 的 workflow nodes
```

### 场景 2: 路径逃逸拒绝
- **操作**: 传入 `../../etc/passwd` 作为 canvas_id
- **预期**: 返回 `path_guard_rejected`
- **验证**:
```bash
# canvas_load("../../etc/passwd") 应被拒绝
```

### 场景 3: 伪造 report 拒绝
- **操作**: 提交一个没有 grant_id 的 worker report
- **预期**: 返回 "无有效执行授权 grant_id"
- **验证**:
```bash
# consume_worker_report_after_completion(grant_id=None) 应被拒绝
```

### 场景 4: 伪造 grant 拒绝
- **操作**: 提交一个 grant_id 格式无效的 report
- **预期**: 返回 "grant_id 格式无效"
- **验证**:
```bash
# consume_worker_report_after_completion(grant_id="invalid") 应被拒绝
```

### 场景 5: Station 3b 写入拒绝
- **操作**: 尝试在 Station 3b（只读站）写入文件
- **预期**: 返回写入被拒绝
- **验证**:
```bash
# 在 Station 3b profile 下尝试写入应被拒绝
```

### 场景 6: 脱敏 UI 验证
- **操作**: 查看审计日志中的 report 内容
- **预期**: 敏感内容（token、密钥等）显示为 `[REDACTED]`
- **验证**:
```bash
# 检查 audit_events 中的 executed_what/changed_what/reason 字段
# 不应包含原始敏感内容
```

### 场景 7: 重启后守卫仍生效
- **操作**: 启动 App → 执行一些操作 → 关闭 App → 重新启动 → 再次尝试违规操作
- **预期**: 重启后 path-lock/identity/grant 守卫仍然有效
- **验证**:
```bash
# 重启后再次尝试跨项目访问/路径逃逸等，应仍被拒绝
```

### 场景 8: 身份解析验证
- **操作**: 用不同角色（worker/user/secretary）提交 report
- **预期**: 身份解析正确，未知角色使用默认兜底
- **验证**:
```bash
# 不同角色的 report 应被正确解析
# 未知角色（如 "hacker"）应使用 TemporaryAgent 兜底
```

## 隔离启动命令

> 订正（2026-08-03）：此前版本写的 `SYN_ISOLATED_PROFILE=1` 在代码库中不存在。
> 真实隔离机制是改 `HOME`（应用数据目录全部由 `$HOME` 派生），并把 rustup/cargo
> 的家指回真实路径（rustup 按 `$HOME/.rustup` 找 toolchain）。

```bash
cd /Users/yoyi/workspace/product-line-syn-fnd-002/prototypes/productized-desktop-shell

# 使用隔离 HOME 启动（不碰真实 store）
HOME=/tmp/fnd006-isolated-home \
RUSTUP_HOME=/Users/yoyi/.rustup \
CARGO_HOME=/Users/yoyi/.cargo \
tauri dev

# 验收前后对真实 HOME 应用目录做指纹比对，必须 IDENTICAL：
find ~/Library/Application\ Support/CodexGovernanceWorkbench \
  -exec stat -f "%N|%m|%z" {} \; | sort > /tmp/real-appdir.txt

# 或者只运行单元测试验证
cargo test --lib
```

## 验收记录模板

```
验收日期: ____
验收环境: ____
验收人: ____

场景 1: 跨项目访问拒绝 → □通过 □失败 □不适用
场景 2: 路径逃逸拒绝 → □通过 □失败 □不适用
场景 3: 伪造 report 拒绝 → □通过 □失败 □不适用
场景 4: 伪造 grant 拒绝 → □通过 □失败 □不适用
场景 5: Station 3b 写入拒绝 → □通过 □失败 □不适用
场景 6: 脱敏 UI 验证 → □通过 □失败 □不适用
场景 7: 重启后守卫仍生效 → □通过 □失败 □不适用
场景 8: 身份解析验证 → □通过 □失败 □不适用

总结: ____
遗留问题: ____
```
