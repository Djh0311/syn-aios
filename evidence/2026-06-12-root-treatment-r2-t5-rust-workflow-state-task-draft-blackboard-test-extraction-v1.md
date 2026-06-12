# Evidence: Root Treatment / R2-T5 Rust Workflow State Task Draft Blackboard Test Extraction v1

日期：2026-06-12

状态：已完成并复核通过，checkpoint 已回填。

任务包：`tasks/2026-06-12-root-treatment-r2-t5-rust-workflow-state-task-draft-blackboard-test-extraction-v1.md`

Planning baseline commit：`8cd4ee6569dd6131c46ae5ed3e4ead7a3d5e6fb3`

Implementation commit：`3465e4dc96c5141861513d9add37cf7cbddf1440`

Implementation fixup commit：`f2a5e36c450e8ab4275d7d356c6cc78925bb93b7`

Review result：`CLEAR`；复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`；P0/P1/P2 无。初次复核发现新 include EOF 空行 P2，已由 fixup commit `f2a5e36c450e8ab4275d7d356c6cc78925bb93b7` 修复并通过最终复核。

Checkpoint commit：`7c7898a02b8ab6c6c9197afd30da4a7c7babbdd3`

## 1. 本轮目标

按最新策略继续 R2 后段 inline tests 迁移，只抽能降低 `lib.rs` 棘轮指标的低风险测试切片。本轮迁移 workflow state bootstrap、task draft、work item state、workflow audit helper 和 blackboard candidate boundary 相关 Rust inline tests。

## 2. 改动范围

修改：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `scripts/harness/workbench-shape-gate.js`
- `tasks/2026-06-12-root-treatment-r2-t5-rust-workflow-state-task-draft-blackboard-test-extraction-v1.md`

新增：

- `prototypes/productized-desktop-shell/src-tauri/src/lib_workflow_state_task_draft_blackboard_tests.rs`
- `evidence/2026-06-12-root-treatment-r2-t5-rust-workflow-state-task-draft-blackboard-test-extraction-v1.md`
- `handoffs/2026-06-12-root-treatment-r2-t5-rust-workflow-state-task-draft-blackboard-test-extraction-v1-result.md`

本轮未修改：

- 产品函数签名、可见性或行为。
- K3-B runtime prompt guard 测试。
- workflow execution runner / workflow machine / ignored real-state tests。
- cross-store memory adoption 或共享 stub runner / factory。
- observation、task memory packet、formal memory adoption、dispatch execution、offline role 或真实状态相关 tests。
- Tauri command、DB schema、sidecar schema、workflow state JSON schema。
- 前端 UI / CSS / TS。

## 3. 实现说明

- 将 21 个 workflow state / task draft / work item state / workflow audit / blackboard candidate tests 原样迁入 `lib_workflow_state_task_draft_blackboard_tests.rs`。
- 在 `lib.rs` 原位置保留 `include!("lib_workflow_state_task_draft_blackboard_tests.rs");`，测试仍运行在 crate-root `tests` module 内，不扩大生产函数可见性。
- 将 shape gate `lib.rs` waterline 从 `10943` 更新为 `10279`。

## 4. 形状收益

- `lib.rs`：`10943 -> 10279`，下降 `664` 行。
- 新增 `lib_workflow_state_task_draft_blackboard_tests.rs`：`664` 行，低于 `.rs` 新文件上限 `3000`。
- shape gate 输出：`lib.rs: 10279/10279 (same)`，0 errors，0 warnings。

## 5. 验证

已通过：

- `cargo test --lib workflow_state`：11 passed，0 failed。
- `cargo test --lib task_draft`：6 passed，0 failed。
- `cargo test --lib work_item_state`：4 passed，0 failed。
- `cargo test --lib workflow_audit`：2 passed，0 failed。
- `cargo test --lib blackboard_candidate`：2 passed，0 failed。
- `cargo test --lib`：471 passed，16 ignored。
- `cargo fmt -- --check`：通过。
- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors，0 warnings。
- `git diff --check`：通过。
- `git diff --check 3465e4d^`：EOF fixup 后通过。

保留既有 warning：

- `src/mcp/protocol.rs` 中 `JsonRpcError::invalid_params` dead_code warning。本轮未触碰该文件。

## 6. 边界确认

本轮没有：

- 执行真实 `codex exec` / `codex exec resume`。
- 发送 prompt。
- 读写 `/Users/yoyi/.codex`。
- 读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript。
- 启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 修改 UI / CSS / TS。
- 修改 Tauri command、DB/schema、sidecar schema、workflow state JSON schema。

关键词扫描：

- 新增 Rust include 文件未命中 `codex exec`、`codex exec resume`、`/Users/yoyi/.codex`、`K3-B`、`workflow_machine`、`workflow execution`、`std::process`、`#[tauri::command]`、`pub struct`、`pub enum` 或 `impl`。
- 命中仅出现在任务包禁止项说明中。

## 7. 复核状态

复核线只读审查已通过：

- 复核线程：`019eb850-0698-7f70-a9b2-e7d0d668ccf5`
- 最终结论：`STATUS: CLEAR`
- P0/P1/P2：无。
- 初次复核发现新 include EOF 空行 P2；主管线已用 `f2a5e36c450e8ab4275d7d356c6cc78925bb93b7` 修复，最终复核确认 clean worktree、新 include 664 行、shape gate waterline `10279`，无边界越界。

## 8. 不接受为

本轮不接受为：

- `lib.rs <= 3,000`
- R2 全部完成
- R3 Level B 执行
- 生产 SQLite 迁移 / read-cut / stop-write
- 多 agent 并行真实执行解锁
- 真实 Codex 执行
- UI / 产品行为修改
- backlog 功能解冻
