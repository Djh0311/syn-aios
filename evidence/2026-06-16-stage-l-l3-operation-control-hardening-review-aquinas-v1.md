# Stage L / L3 Operation Control Hardening Review - Aquinas v1

日期：2026-06-16

复核线：Aquinas / Agent E

Agent id：`019ece6b-4b39-7830-9553-86b979ec322c`

STATUS: CLEAR

P0：none

P1：none

P2：none

P3：none

## 复核范围

- 任务包：`tasks/2026-06-16-stage-l-l3-operation-control-hardening-v1.md`
- 证据草稿：`evidence/2026-06-16-stage-l-l3-operation-control-hardening-v1.md`
- 交接草稿：`handoffs/2026-06-16-stage-l-l3-operation-control-hardening-v1-result.md`
- 后端：`operation_control.rs`、`commands.rs`、`command_registry.rs`、`lib.rs`、`workflow_audit.rs`、`runtime_log_store.rs`、`memory_capture_bus.rs`
- 前端：`App.tsx`、`PermissionDialog.tsx`、`RunningWorkflowsView.tsx`、`tauri.ts`、相关 TS types 与 offline helper/tests

## 初审发现与修复

初审结论为 `STATUS: FINDINGS`，唯一阻断项：

- P1：确认 `record-operation-control-decision` 后只调用 `recordOperationControlDecision` 与 `setWorkflowState(result.snapshot)`，没有刷新 `snapshot.operation_control`；运行页四操作卡片来自 `snapshot.operation_control`，可能继续显示旧的 `available` 状态。

已修复：

- `App.tsx` 在 `record-operation-control-decision` 分支中，成功写入 audit 后调用 `loadWorkbenchSnapshotFromPageQueries(queryWorkbenchPageReadModel)` 并 `setSnapshot(nextSnapshot)`，随后 `setWorkflowState(result.snapshot)`。
- `offlineL3OperationControlScenario.tsx` 新增断言：confirmed snapshot 下运行页显示 `决策已登记`，仍提示 `仍未执行真实操作`，且 `已记录 重试 决策` 按钮 disabled，防止重复发起。

复审确认 P1 已关闭。

## 复核结论

- 未发现 L3 新增真实执行路径。
- `record_operation_control_decision` 是窄 workflow-state audit 写入，不调用 runner / process-control / real Codex。
- 未发现新增 `Command::new("codex")`、真实 `codex exec` / `codex exec resume`、真实 stop / restart / kill、`.codex` 读写或 K3-B2 解锁。
- `confirmed_recorded` 仍是“决策已登记”，不是已执行 / 已成功。
- `does_execute_in_l3`、`readback_result_count`、`requires_separate_authorized_window`、`blocks_k3_b2` 有后端 guard。
- Level-B / real-resume 既有门未放宽；`session_continuation_store.rs::run_real_resume_phase_b_with_runner()` 未被修改或新调用。

## 证据说明

复核线只读核验了最新 diff，并用 `rg` / `git diff --name-only` / `git ls-files --others` 检查 tracked 与 untracked 范围。复核线未修改文件，未运行真实 Codex / runner，未触碰真实数据或 `/Users/yoyi/.codex`。

主管线提供的验证证据：

- `cargo test --lib operation_control`：6 passed / 0 ignored
- `cargo test --lib`：516 passed / 21 ignored
- `npm run typecheck`：passed
- `npm run test:offline-interaction`：passed
- `npm run build`：passed，只有既有 Vite chunk-size warning
- shape gate：pass，0 errors / 1 warning；warning 为 Tauri command 总数 97 -> 98，且 0 in `lib.rs`
- `git diff --check`：passed

备注：主管线尝试真实浏览器 smoke，Vite 需提升权限后可启动，但 Playwright browser binary 缺失且系统 Chrome headless SIGABRT；该环境限制不改变本次复核结论，真实浏览器 / Tauri 可视化验收仍可作为后续 L4 残余项补做。
