# Stage K / K2 General Codex Resume And New Session Product Entry Level A Handoff v1

日期：2026-06-10

结论：K2 Level A 完成，Phase B 普通入口接线完成，R1/R2/N1/N2 env-gated 真实执行测试 harness 完成，K2 尚未完成。Level A 已包含普通层 `写入准备 / 用户确认 / 记录预检` 的非真实产品流；本轮新增 `确认执行` Phase B 入口接线和真实执行点测试入口。补充：后续 K2-R1 已真实执行并通过，见 `handoffs/2026-06-10-stage-k-k2-r1-mario-test-resume-read-only-real-execution-v1-result.md`；R2/N1/N2 仍未执行。

本轮完成：

- 写好 K2 任务包和 R1/R2/N1/N2 真实执行点字段工作表。
- 暴露 Product Command new-session Phase B 的 Tauri command / TS wrapper。
- Agent 普通输入框接入 K2 结构化发送预览。
- `新建对话` 现在只切换为 new-session preview mode，不直接创建真实 session。
- Agent 发送预览卡接入 `写入准备`、`用户确认`、`记录预检` 三步，分别写入 Product Command prepare / decision / Phase A no-op trace。
- Agent 发送预览卡接入 `确认执行 Codex`：`resume` 走 `runRealExecutionProductCommandPhaseB`，`new_session` 走 `runRealExecutionProductCommandNewSessionPhaseB`；按钮只在 Phase A 无阻断后启用。
- 修改任务正文会清空旧 preview、prepare、decision、Phase A 和 Phase B 状态，避免旧权限预览对应新 prompt body。
- 新增 Stage K 隔离项目 fixture README：`test-fixtures/stage-k-isolated-project/README.md`；hash `cf1289518849fc1a6947c2c034717f5c4e5afaa0726d56b5de9c733bdd1c201c`。
- 固定 R1/R2/N1/N2 canonical prompt 文件和 hash，路径在 `tmp/stage-k-k2-real-execution-prompts/`，对应 hash 已写回 K2 任务包第 6 节。
- 新增 K2 R1/R2/N1/N2 execution point harness helper、ignored/env-gated 真实测试入口和 fake-runner safety tests；默认测试不触发真实 Codex。

验证通过：

- `cargo fmt -- --check`
- `cargo test --lib real_execution_command`：33 passed / 3 ignored
- `cargo test --lib session_continuation`：17 passed / 4 ignored
- `cargo test --lib codex_local_runner`：11 passed
- `npm run typecheck`
- `npm run test:offline-interaction`：14 passed
- `npm run build`：通过，仅既有 Vite chunk size warning

Phase B 普通入口接线后补充验证：

- `npm run typecheck`：通过
- `npm run test:offline-interaction`：14 passed
- `npm run build`：通过，仅既有 Vite chunk size warning

任务正文变更清空旧预览修补后再次验证：

- `npm run typecheck`：通过
- `npm run test:offline-interaction`：14 passed
- `npm run build`：通过，仅既有 Vite chunk size warning

K2 execution point harness 并入后验证：

- `cargo test --lib real_execution_command`：36 passed / 7 ignored
- `cargo test --lib session_continuation`：17 passed / 4 ignored
- `cargo fmt -- --check`：通过
- `cargo test --lib`：323 passed / 14 ignored
- `npm run typecheck`：通过
- `npm run test:offline-interaction`：14 passed
- `npm run build`：通过，仅既有 Vite chunk size warning
- Rust 验证仅保留既有 `JsonRpcError::invalid_params` dead_code warning

边界确认：

- 本 Level A 记录形成时未执行真实 Codex；后续已完成 R1 真实执行，见独立 R1 handoff。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未写 `mario test`；隔离测试项目仅新增 README fixture。
- 本 Level A 记录形成时未做 R1/R2/N1/N2；后续已完成 R1，R2/N1/N2 仍未执行。
- R1/R2/N1/N2 的 ignored/env-gated 测试入口已存在，但本轮没有运行任何 ignored 测试。
- 已接入的 Phase A 只是 no-op 预检记录，不调用 Phase B，不触发 runner。
- 已接入的 Phase B 入口本轮未点击；没有真实 runner 调用，没有 prompt 发送，没有 `.codex` 副作用。
- 隔离测试项目仅新增 README fixture；没有执行 N1/N2，也没有写 `.workbench/stage-k/k2/` 探针文件。
- 过程偏差：一次 `rg` 扫描命令里的 Markdown 反引号触发 `mario` 命令替换尝试，返回 `zsh:1: command not found: mario`；无真实 Codex、无 `.codex`、无产品副作用，随后已用单引号重扫。

下一步：

- 复核线只读审查 K2 Level A。
- 复核线确认 K2 普通产品入口接线后，按 R1/R2/N1/N2 字段逐项进入真实执行验收。
- R2/N1/N2 真实执行仍需单项记录 baseline hash、`.codex` 副作用、项目写入副作用、readback status / result_count、runtime/audit refs，不能用本轮接线结果替代。
