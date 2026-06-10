# Stage K / K3-B1.0 Prompt Freeze Repair Before Real Resume v1

日期：2026-06-10

状态：已完成，结论为 `accepted`。本文是 K3-B1 真实 `resume` 执行前的字段一致性修补任务包，不执行真实 Codex，不发送 prompt，不读写 `/Users/yoyi/.codex`。

## 1. 背景

K3-B1 任务包已冻结 read-only `resume` 执行点，但执行前复核发现原 `prompt_hash`：

```text
e963aa00f7ba0cb94d973996794e98db9d019bf8d9e568c330eb272d7ddd9fbf
```

只有 hash 记录，没有可审计的 prompt 正文。全仓库和已知历史 prompt 搜索未找到可复原正文，按 H5 / PCR9 / J2-B 模板推导的候选正文也不能匹配该 hash。因此 K3-B1 不能直接执行，否则会破坏“prompt body 可复现、hash 可验证、执行前 gate 可独立检查”的架构要求。

## 2. 目标

- 冻结新的 K3-B1 runtime-only prompt 正文。
- 计算并同步新的 `prompt_hash`。
- 更新 K3-B1 执行任务包、K3-Level-B 字段冻结任务包和 Rust K3-B1 常量。
- 记录旧 hash 已 superseded，不能再作为 K3-B1 执行 gate。
- 保持 K3-B1 真实执行仍为待执行，不在本文中触发。

## 3. 当前冻结值

Prompt 路径：

```text
/Users/yoyi/workspace/product-line/tmp/k3-b-real-workflow-automation/prompts/k3-b1-mario-test-read-only.prompt.txt
```

当前 hash：

```text
ab0442e86e75900ab47b293328e4a2b46512ae68868799b94e8608ffedd57039
```

旧 hash：

```text
e963aa00f7ba0cb94d973996794e98db9d019bf8d9e568c330eb272d7ddd9fbf
```

旧 hash 状态：`superseded_prompt_body_unrecoverable`。

## 4. 必须同步

- `tmp/k3-b-real-workflow-automation/prompts/k3-b1-mario-test-read-only.prompt.txt`
- `tasks/2026-06-10-stage-k-k3-b1-mario-test-workflow-read-only-real-resume-run-v1.md`
- `tasks/2026-06-10-stage-k-k3-level-b-real-workflow-execution-field-freeze-v1.md`
- `prototypes/productized-desktop-shell/src-tauri/src/project_workflow_automation.rs`
- 当前权威入口文档只在 checkpoint 层说明 K3-B1.0 已完成。

## 5. 验收

必须通过：

```text
shasum -a 256 tmp/k3-b-real-workflow-automation/prompts/k3-b1-mario-test-read-only.prompt.txt
cargo test --lib project_workflow_automation
cargo test --lib real_execution_command
cargo fmt -- --check
rg -n "e963aa00f7ba0cb94d973996794e98db9d019bf8d9e568c330eb272d7ddd9fbf" tasks docs evidence handoffs prototypes/productized-desktop-shell/src-tauri/src/project_workflow_automation.rs
```

扫描允许命中历史 evidence / handoff 中的旧 hash，但当前 K3-B1 执行任务包和 Rust 常量不得再以旧 hash 作为执行 gate。

## 6. 禁止

- 禁止执行真实 `codex exec` / `codex exec resume`。
- 禁止发送 runtime prompt。
- 禁止读写 `/Users/yoyi/.codex`。
- 禁止读取 secret、token、`.env`、auth、keychain、OAuth、provider credential、full transcript、rollout。
- 禁止把 K3-B1.0 说成 K3-B1 已执行。
- 禁止把 prompt body 写入 Product Command sidecar、continuation sidecar、runtime log、audit 或 memory。

## 7. 完成口径

K3-B1.0 只接受为 K3-B1 执行前 prompt freeze 修补完成。K3-B1 仍必须单独执行真实 `resume` 任务包，并在执行后记录 `.codex` 副作用、readback marker、项目 hash 前后对比和 sidecar / runtime / audit 路径。
