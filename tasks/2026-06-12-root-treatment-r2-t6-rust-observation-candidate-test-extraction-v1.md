# Root Treatment / R2-T6 Rust Observation Candidate Test Extraction v1

日期：2026-06-12

状态：实现完成，待复核，hash 待回填。

Planning baseline commit：`092dceba6f9dd2896053267b8ffc65702484e0a3`

Implementation commit：`TBD`

Review result：`TBD`

Checkpoint commit：`TBD`

本文是 Root Treatment / Stage R 的 R2-T6 任务包，承接 R4-A50 新策略和 R2-T1 到 R2-T5 inline tests 迁移结果，继续迁移能实际降低 `lib.rs` 棘轮指标的低风险 Rust inline tests。

## 1. 目标

把 `prototypes/productized-desktop-shell/src-tauri/src/lib.rs` 中 observation store / observation candidate 相关 inline tests 迁出到 crate-root test include 文件，继续降低 `lib.rs` 历史最低水位线。

目标文件：

- 新增：`prototypes/productized-desktop-shell/src-tauri/src/lib_observation_candidate_tests.rs`
- 修改：`prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- 修改：`scripts/harness/workbench-shape-gate.js`

预计收益：

- `lib.rs` 预计下降约 280 行以上。
- 新增 `.rs` test include 预计低于 3,000 行。
- 该切片降低 `lib.rs` 棘轮指标，符合新策略“不得立项不降低棘轮指标的拆分包”。

## 2. 允许范围

允许迁移以下 tests，保持测试体和断言语义不变：

- `observation_store_records_worker_report`
- `observation_candidate_creation_project_director`
- `observation_candidate_creation_rejects_quarantined`
- `observation_candidate_creation_rejects_ignored`
- `observation_candidate_creation_rejects_duplicate`
- `observation_creation_rejects_missing_source_refs`
- `observation_creation_rejects_ordinary_chat_auto_capture`
- `observation_candidate_does_not_create_formal_memory`
- `observation_context_binding_mismatch_rejected`

允许：

- 在 `lib.rs` 原位置插入 `include!("lib_observation_candidate_tests.rs");`
- 让 observation helper 继续留在 `lib.rs`，因为后续 task memory packet tests 仍复用 `fixture_observation_input`、`create_recorded_observation` 等 helper。
- 更新 shape gate 中 `lib.rs` waterline 为本轮完成后的历史最低收口值。
- 新增 evidence / handoff。

## 3. 禁止范围

禁止：

- 修改产品函数签名、可见性、语义或断言口径。
- 迁移 task memory packet、formal memory adoption、memory lint、K3-B runtime prompt guard、workflow execution runner、workflow machine、ignored real-state tests、cross-store memory adoption、共享 stub runner / factory。
- 修改 observation helper 语义或把共享 helper 移出 `lib.rs`。
- 修改 Tauri command、DB schema、sidecar schema、workflow state JSON schema。
- 修改前端 UI / CSS / TS。
- 执行真实 `codex exec` / `codex exec resume`、发送 prompt、读写 `/Users/yoyi/.codex`。
- 启动 Tauri / Browser / Chrome / Vite / 截图工具。

## 4. 验收

必须通过：

- `cargo test --lib observation`
- `cargo test --lib task_memory_packet`
- `cargo test --lib`
- `cargo fmt -- --check`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`
- 复核线只读审查，结论不得有 P0/P1。

## 5. 不接受为

本任务不接受为：

- `lib.rs <= 3,000`
- R2 全部完成
- R3 Level B 执行
- 生产 SQLite 迁移 / read-cut / stop-write
- 多 agent 并行真实执行解锁
- 真实 Codex 执行
- task memory packet / formal memory adoption 迁移完成
- UI / 产品行为修改
- backlog 功能解冻

## 6. 执行记录

本轮已完成实现和本地验证，等待复核线只读审查。

实际改动：

- 新增 `prototypes/productized-desktop-shell/src-tauri/src/lib_observation_candidate_tests.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs` 原测试块替换为 `include!("lib_observation_candidate_tests.rs");`
- `scripts/harness/workbench-shape-gate.js` 的 `lib.rs` waterline 更新为 `9996`

实际形状收益：

- `lib.rs`：`10279 -> 9996`，下降 `283` 行。
- 新增 include 文件：`284` 行，低于 `.rs` 新文件上限 `3000`。

验证已通过：

- `cargo test --lib observation`：40 passed。
- `cargo test --lib task_memory_packet`：10 passed。
- `cargo test --lib`：471 passed，16 ignored。
- `cargo fmt -- --check`：通过。
- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors，0 warnings。
- `git diff --check`：通过。

保留既有 warning：

- `src/mcp/protocol.rs` 中 `JsonRpcError::invalid_params` dead_code warning。本轮未触碰该文件。
