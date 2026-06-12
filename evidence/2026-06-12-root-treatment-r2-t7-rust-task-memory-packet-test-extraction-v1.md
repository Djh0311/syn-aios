# Evidence: Root Treatment / R2-T7 Rust Task Memory Packet Test Extraction v1

日期：2026-06-12

状态：已完成并复核通过，checkpoint 已同步。

任务包：`tasks/2026-06-12-root-treatment-r2-t7-rust-task-memory-packet-test-extraction-v1.md`

Planning baseline commit：`d7d9d3520495425f1c0e8ce5ce3b681970b360be`

Task package commit：`b417e83f7365d57f963abb3e0fd921cbfb2fa36a`

Implementation commit：`04172eb8f8ee59ee3d311c20552eac02c52bd2ca`

Review result：`CLEAR`；复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`；P0/P1/P2 无。

Checkpoint authority sync commit：`b00d6c80210b2166b66f21214d9e392d00f645e9`

## 1. 本轮目标

按最新策略继续 R2 后段 inline tests 迁移，只抽能降低 `lib.rs` 棘轮指标的低风险测试切片。本轮迁移 task memory packet preview 相关 Rust inline tests。

## 2. 改动范围

修改：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `scripts/harness/workbench-shape-gate.js`
- `tasks/2026-06-12-root-treatment-r2-t7-rust-task-memory-packet-test-extraction-v1.md`

新增：

- `prototypes/productized-desktop-shell/src-tauri/src/lib_task_memory_packet_tests.rs`
- `evidence/2026-06-12-root-treatment-r2-t7-rust-task-memory-packet-test-extraction-v1.md`
- `handoffs/2026-06-12-root-treatment-r2-t7-rust-task-memory-packet-test-extraction-v1-result.md`

本轮未修改：

- 产品函数签名、可见性或行为。
- task memory packet helper 语义；共享 helper 仍留在 `lib.rs`，继续供后续 memory entity / lint / mature pattern 等 tests 复用。
- formal memory adoption、memory lint、memory entity relation、mature pattern、K3-B runtime prompt guard、workflow execution runner、workflow machine、ignored real-state tests、cross-store memory adoption 或共享 stub runner / factory。
- Tauri command、DB schema、sidecar schema、workflow state JSON schema。
- 前端 UI / CSS / TS。

## 3. 实现说明

- 将 10 个 task memory packet preview tests 原样迁入 `lib_task_memory_packet_tests.rs`。
- 在 `lib.rs` 原位置保留 `include!("lib_task_memory_packet_tests.rs");`，测试仍运行在 crate-root `tests` module 内，不扩大生产函数可见性。
- 将 shape gate `lib.rs` waterline 从 `9996` 更新为 `9610`。

## 4. 形状收益

- `lib.rs`：`9996 -> 9610`，下降 `386` 行。
- 新增 `lib_task_memory_packet_tests.rs`：`387` 行，低于 `.rs` 新文件上限 `3000`。
- shape gate 输出：`lib.rs: 9610/9610 (same)`，0 errors，0 warnings。

## 5. 验证

已通过：

- `cargo test --lib task_memory_packet`：10 passed，0 failed。
- `cargo test --lib observation`：40 passed，0 failed。
- `cargo test --lib`：471 passed，16 ignored。
- `cargo fmt -- --check`：通过。
- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors，0 warnings。
- `git diff --check`：通过。

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

- 新增 Rust include 文件未命中 `codex exec`、`codex exec resume`、`/Users/yoyi/.codex`、`K3-B`、`workflow_machine`、`workflow execution`、`std::process`、`#[tauri::command]`、`pub struct`、`pub enum`、`impl`、`adopt_memory_candidate` 或 `run_workflow_machine`。
- 新增 Rust include 文件中 `formal_memory` 命中为允许范围内的 task memory packet fixture 调用，不是 formal memory adoption 迁移。
- `task_memory_packet` 命中为本轮允许迁移的 10 个 preview tests。

## 7. 复核状态

复核线只读审查已通过：

- 复核线程：`019eb850-0698-7f70-a9b2-e7d0d668ccf5`
- 最终结论：`STATUS: CLEAR`
- P0/P1/P2：无。
- 复核确认新 include 只包含 10 个 task memory packet preview tests，未迁移 helper；memory entity relation、formal memory adoption、workflow execution、workflow machine 和 stub runner/factory 仍留在 `lib.rs`；shape gate waterline `9610` 与当前 `wc -l lib.rs` 一致。
- 复核确认旧 `lib.rs` 测试块与新 include 的 `diff -w` 仅有末尾空白行差异，无测试体 / 断言语义差异；`git diff --check` 与 `git diff --check 04172eb^ HEAD` 均无输出。

## 8. 不接受为

本轮不接受为：

- `lib.rs <= 3,000`
- R2 全部完成
- R3 Level B 执行
- 生产 SQLite 迁移 / read-cut / stop-write
- 多 agent 并行真实执行解锁
- 真实 Codex 执行
- task memory packet 产品能力新增或语义变更
- formal memory adoption 迁移完成
- UI / 产品行为修改
- backlog 功能解冻
