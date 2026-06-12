# Evidence: Root Treatment / R2-T6 Rust Observation Candidate Test Extraction v1

日期：2026-06-12

状态：实现完成，待复核，hash 待回填。

任务包：`tasks/2026-06-12-root-treatment-r2-t6-rust-observation-candidate-test-extraction-v1.md`

Planning baseline commit：`092dceba6f9dd2896053267b8ffc65702484e0a3`

Implementation commit：`abd10f1e6fd11cd94f3ad9d7dca2b5902204c816`

Review result：`TBD`

Checkpoint commit：`TBD`

## 1. 本轮目标

按最新策略继续 R2 后段 inline tests 迁移，只抽能降低 `lib.rs` 棘轮指标的低风险测试切片。本轮迁移 observation store / observation candidate 相关 Rust inline tests。

## 2. 改动范围

修改：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `scripts/harness/workbench-shape-gate.js`
- `tasks/2026-06-12-root-treatment-r2-t6-rust-observation-candidate-test-extraction-v1.md`

新增：

- `prototypes/productized-desktop-shell/src-tauri/src/lib_observation_candidate_tests.rs`
- `evidence/2026-06-12-root-treatment-r2-t6-rust-observation-candidate-test-extraction-v1.md`
- `handoffs/2026-06-12-root-treatment-r2-t6-rust-observation-candidate-test-extraction-v1-result.md`

本轮未修改：

- 产品函数签名、可见性或行为。
- observation helper 语义；共享 helper 仍留在 `lib.rs`，继续供后续 task memory packet tests 复用。
- task memory packet、formal memory adoption、memory lint、K3-B runtime prompt guard、workflow execution runner、workflow machine、ignored real-state tests、cross-store memory adoption 或共享 stub runner / factory。
- Tauri command、DB schema、sidecar schema、workflow state JSON schema。
- 前端 UI / CSS / TS。

## 3. 实现说明

- 将 9 个 observation store / observation candidate tests 原样迁入 `lib_observation_candidate_tests.rs`。
- 在 `lib.rs` 原位置保留 `include!("lib_observation_candidate_tests.rs");`，测试仍运行在 crate-root `tests` module 内，不扩大生产函数可见性。
- 将 shape gate `lib.rs` waterline 从 `10279` 更新为 `9996`。

## 4. 形状收益

- `lib.rs`：`10279 -> 9996`，下降 `283` 行。
- 新增 `lib_observation_candidate_tests.rs`：`284` 行，低于 `.rs` 新文件上限 `3000`。
- shape gate 输出：`lib.rs: 9996/9996 (same)`，0 errors，0 warnings。

## 5. 验证

已通过：

- `cargo test --lib observation`：40 passed，0 failed。
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

- 新增 Rust include 文件未命中 `codex exec`、`codex exec resume`、`/Users/yoyi/.codex`、`K3-B`、`workflow_machine`、`workflow execution`、`std::process`、`#[tauri::command]`、`pub struct`、`pub enum`、`impl`、`task_memory_packet`、`adopt_memory_candidate` 或 `run_workflow_machine`。
- 命中仅出现在任务包禁止项 / 验收项说明中。

## 7. 复核状态

待复核线只读审查。

## 8. 不接受为

本轮不接受为：

- `lib.rs <= 3,000`
- R2 全部完成
- R3 Level B 执行
- 生产 SQLite 迁移 / read-cut / stop-write
- 多 agent 并行真实执行解锁
- 真实 Codex 执行
- task memory packet / formal memory adoption 迁移完成
- UI / 产品行为修改
- backlog 功能解冻
