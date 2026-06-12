# Evidence: Root Treatment / R2-T4 Rust Memory Lint Mature Pattern Test Extraction v1

日期：2026-06-12

状态：复核通过，hash 待回填。

任务包：`tasks/2026-06-12-root-treatment-r2-t4-rust-memory-lint-mature-pattern-test-extraction-v1.md`

Planning baseline commit：`fe1b69c70058e3316c7a6f3020552279757f7397`

Implementation commit：`4d35b3b3e042cd738c0ebd92267508ed5e93fe31`

Review result：`CLEAR`，P0/P1/P2 无；复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`

Checkpoint commit：`TBD`

## 1. 本轮目标

按最新策略继续 R2 后段 inline tests 迁移，只抽能降低 `lib.rs` 棘轮指标的低风险测试切片。本轮迁移 memory lint、maintenance 和 mature pattern 相关 Rust inline tests。

## 2. 改动范围

修改：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `scripts/harness/workbench-shape-gate.js`
- `tasks/2026-06-12-root-treatment-r2-t4-rust-memory-lint-mature-pattern-test-extraction-v1.md`

新增：

- `prototypes/productized-desktop-shell/src-tauri/src/lib_memory_lint_mature_pattern_tests.rs`
- `evidence/2026-06-12-root-treatment-r2-t4-rust-memory-lint-mature-pattern-test-extraction-v1.md`
- `handoffs/2026-06-12-root-treatment-r2-t4-rust-memory-lint-mature-pattern-test-extraction-v1-result.md`

本轮未修改：

- 产品函数签名、可见性或行为。
- K3-B runtime prompt guard 测试。
- workflow execution runner / workflow machine / ignored real-state tests。
- cross-store memory adoption 或共享 stub runner / factory。
- blackboard、observation、task memory packet、workflow state bootstrap / task draft / work item state tests。
- Tauri command、DB schema、sidecar schema、workflow state JSON schema。
- 前端 UI / CSS / TS。

## 3. 实现说明

- 将 16 个 memory lint / maintenance / mature pattern tests 和 2 个仅供该簇使用的 M12 helper 原样迁入 `lib_memory_lint_mature_pattern_tests.rs`。
- 在 `lib.rs` 原位置保留 `include!("lib_memory_lint_mature_pattern_tests.rs");`，测试仍运行在 crate-root `tests` module 内，不扩大生产函数可见性。
- 将 shape gate `lib.rs` waterline 从 `12019` 更新为 `10943`。

## 4. 形状收益

- `lib.rs`：`12019 -> 10943`，下降 `1076` 行。
- 新增 `lib_memory_lint_mature_pattern_tests.rs`：`1078` 行，低于 `.rs` 新文件上限 `3000`。
- shape gate 输出：`lib.rs: 10943/10943 (same)`，0 errors，0 warnings。

## 5. 验证

已通过：

- `cargo test --lib memory_lint`：9 passed，0 failed。
- `cargo test --lib mature_pattern`：5 passed，0 failed。
- `cargo test --lib task_memory_packet`：10 passed，0 failed。
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

- 新增 Rust include 文件未命中 `codex exec`、`codex exec resume`、`/Users/yoyi/.codex`、`K3-B`、`workflow machine`、`workflow_execution`、`real Codex`、`真实 Codex`。
- 命中仅出现在任务包禁止项说明中。

## 7. 复核结果

复核线只读审查结论：

- `STATUS: CLEAR`
- P0：无。
- P1：无。
- P2：无。
- 复核确认 diff 范围符合 R2-T4：`lib.rs`、shape gate、新 test include，以及 task/evidence/handoff。
- 复核确认新 include 文件为 16 个目标 tests 加 2 个 M12 helper，且无 `use/mod/pub/struct/enum/impl` 新形状，无 `std::process`、网络、Tauri command、真实 Codex 或 `.codex` 访问。
- 复核确认 `lib.rs` 仅保留 include，后续 `missing_workflow_state_returns_empty_without_creating_file`、bootstrap/task draft/work item state、blackboard、observation、task memory packet tests 仍留在 `lib.rs`。
- 复核确认 shape gate waterline `10943` 与当前 `wc -l lib.rs` 一致，新 include `1078` 行低于 `3000` 上限。

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
