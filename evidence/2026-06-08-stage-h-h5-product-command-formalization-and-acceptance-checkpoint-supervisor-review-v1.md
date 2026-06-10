# Evidence: Stage H / H5 Product Command Formalization And Acceptance Checkpoint Supervisor Review v1

日期：2026-06-08

## 结论

H5 product command formalization and acceptance checkpoint 已完成主管复核，结论为：

```text
accepted_as_h5_checkpoint_after_supervisor_review
```

本轮接受为：

- H5 checkpoint 已把 Level A 非真实 preview / readiness / permission envelope、B1 read-only 真实 probe、B2 workspace-write 真实 probe 收敛成一份产品 command / bridge 验收矩阵。
- `preview_h5_project_workflow_dispatch` 仍是非执行路径，固定不发送 prompt、不执行真实 Codex、不读写 `/Users/yoyi/.codex`。
- 显式批准后的真实执行路径当前复用 continuation Phase B 后端 runner / `RealCodexLocalPhaseBProcessRunner`，B1/B2 证明单项目 read-only 与 workspace-write 真实 `resume` 可追溯到 continuation / runtime log / audit / readback。
- 本 checkpoint 未新增 UI、未新增真实执行按钮、未新增新的真实执行授权。

本轮不接受为：

- H5 通用项目工作流真实派发产品化完成。
- 任意项目 / 任意 session / 任意写入范围自由执行开放。
- H3-B retry 成功或 `new_session` 产品化完成。
- H4-Level-B 真实失败 / 超时探针完成。
- 自动重试、自动恢复、stop / kill / restart 产品化完成。
- planned adapters 真实接入。
- provider credential / model verification 完成。
- worker report、readback、tool output、observation 或 candidate 自动写正式事实 / 正式记忆。
- 阶段 H 完成。

## 复核依据

开发线产物：

```text
tasks/2026-06-08-stage-h-h5-product-command-formalization-and-acceptance-checkpoint-v1.md
evidence/2026-06-08-stage-h-h5-product-command-formalization-and-acceptance-checkpoint-v1.md
handoffs/2026-06-08-stage-h-h5-product-command-formalization-and-acceptance-checkpoint-v1-result.md
```

只读复核线指出的原始风险：

```text
H5 preview bridge 和真实 execute 路径曾像两套路径；
旧 execute_workflow_node_dispatch 不能冒充 H5 product command；
B1/B2 只能作为单项目 probe，不能写成 H5 通用产品化。
```

本轮主管判断：

```text
H5 checkpoint 可接受为产品 command / bridge 边界收束；
不要求本 checkpoint 新增可见 H5 execute UI 按钮；
真实执行仍只能在后续执行点授权后走后端 runner；
B1/B2 只能作为 acceptance matrix 的真实证据，不扩大为通用完成。
```

## 本轮主管验证

已重新执行：

```text
cargo test --lib
rustfmt --check src/h5_project_dispatch_bridge.rs src/session_continuation_store.rs src/codex_local_runner.rs src/runtime_log_store.rs src/types.rs src/commands.rs
```

结果：

```text
cargo test --lib: 258 passed; 0 failed; 5 ignored
rustfmt --check: passed
```

保留既有 warning：

```text
JsonRpcError::invalid_params is never used
```

5 个 ignored 测试均为显式真实执行授权测试，本轮未运行。

## 主管修正

本轮仅做文档主管收口：

- 新增本主管复核 evidence / handoff。
- 将 checkpoint 任务包状态从“已创建，待执行”修正为“已完成并通过主管复核”。
- 将权威入口中的 H5 checkpoint stale 口径从“待执行”修正为“已完成并通过主管复核”。
- 下一步指向更大的 H6 checkpoint：真实执行 UI 产品化和 Tauri 验收准备 / 执行，而不是继续拆 B3/B4 小 probe。

## 边界确认

本轮没有：

- 修改产品代码。
- 执行新的真实 `codex exec`。
- 执行新的真实 `codex exec resume`。
- 发送 prompt。
- 读写 `/Users/yoyi/.codex`。
- 读取 auth / token / secret / `.env` / keychain / OAuth / provider credential。
- 读取 full transcript / rollout。
- 创建真实 `new_session`。
- 执行 H3-B retry。
- 执行 H4-Level-B 真实失败 / 超时探针。
- 接 planned adapters 真实执行。
- 改 UI 或声称 UI / Tauri 验收完成。

## 下一步

建议进入 H6 合并型 checkpoint：真实执行 UI 产品化和 Tauri 验收。H6 应继续采用 checkpoint 节奏，不再为小探针反复同步入口文档；如果需要真实执行或真实 Tauri 截图，必须在任务包内明确执行点授权、截图路径、降级规则和停止条件。
