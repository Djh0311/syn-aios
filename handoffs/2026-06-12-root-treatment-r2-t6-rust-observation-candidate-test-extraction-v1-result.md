# Handoff: Root Treatment / R2-T6 Rust Observation Candidate Test Extraction v1

日期：2026-06-12

状态：实现完成，待复核，hash 待回填。

任务包：`tasks/2026-06-12-root-treatment-r2-t6-rust-observation-candidate-test-extraction-v1.md`

Evidence：`evidence/2026-06-12-root-treatment-r2-t6-rust-observation-candidate-test-extraction-v1.md`

Planning baseline commit：`092dceba6f9dd2896053267b8ffc65702484e0a3`

Implementation commit：`abd10f1e6fd11cd94f3ad9d7dca2b5902204c816`

Review result：`TBD`

Checkpoint commit：`TBD`

## 1. 完成内容

R2-T6 按新策略继续做能降低 `lib.rs` 棘轮指标的低风险 inline tests 迁移：

- 新增 `prototypes/productized-desktop-shell/src-tauri/src/lib_observation_candidate_tests.rs`
- `lib.rs` 原位置保留 `include!("lib_observation_candidate_tests.rs");`
- `scripts/harness/workbench-shape-gate.js` 的 `lib.rs` waterline 更新为 `9996`

迁移内容为 9 个 observation store / observation candidate 相关 tests。共享 observation helper 继续留在 `lib.rs`，供后续 task memory packet tests 复用。

## 2. 形状指标

- `lib.rs`：`10279 -> 9996`，下降 `283` 行。
- 新 include 文件：`284` 行，低于 `.rs` 新文件上限 `3000`。
- shape gate：pass，0 errors，0 warnings；`lib.rs: 9996/9996 (same)`。

## 3. 验证

已通过：

- `cargo test --lib observation`：40 passed。
- `cargo test --lib task_memory_packet`：10 passed。
- `cargo test --lib`：471 passed，16 ignored。
- `cargo fmt -- --check`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

保留既有 warning：

- `src/mcp/protocol.rs` 中 `JsonRpcError::invalid_params` dead_code warning。本轮未触碰该文件。

## 4. 边界确认

本轮没有：

- 执行真实 `codex exec` / `codex exec resume`
- 发送 prompt
- 读写 `/Users/yoyi/.codex`
- 读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript
- 启动 Tauri / Browser / Chrome / Vite / 截图工具
- 修改 UI / CSS / TS
- 修改 Tauri command、DB/schema、sidecar schema、workflow state JSON schema

新增 Rust include 文件关键词扫描无命中：`codex exec`、`codex exec resume`、`/Users/yoyi/.codex`、`K3-B`、`workflow_machine`、`workflow execution`、`std::process`、`#[tauri::command]`、`pub struct`、`pub enum`、`impl`、`task_memory_packet`、`adopt_memory_candidate`、`run_workflow_machine`。

## 5. 等待复核

请复核线只读审查：

- diff 范围是否只限 R2-T6。
- 新 include 是否只包含任务包允许的 9 个 tests。
- `lib.rs` 是否只保留 include，且 task memory packet / formal memory adoption / workflow machine / runner tests 仍留在原文件。
- observation helper 是否仍在 `lib.rs` 且语义未改。
- shape gate waterline `9996` 是否与当前 `wc -l lib.rs` 一致。
- 是否存在真实 Codex、`.codex`、Tauri command、DB/schema、UI 或产品语义越界。

## 6. 不接受为

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
