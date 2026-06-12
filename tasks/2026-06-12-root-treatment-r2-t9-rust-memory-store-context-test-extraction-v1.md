# Root Treatment / R2-T9 Rust Memory Store Context Test Extraction v1

日期：2026-06-12

状态：已完成并复核通过，checkpoint 已同步。

Planning baseline commit：`83441187fef4f3b6acd1ae67a17174f28d4b3823`

Task package commit：`d564febc857c4a51c97d819b295ee66a29218858`

Implementation commit：`8776e95ef005a3a6e1e8e8ff2a21357818564817`

Review result：`CLEAR`；复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`；P0/P1/P2 无。

Checkpoint authority sync commit：`c1221c0b0179097d0ae919d9a444133b689a2a37`

本文是 Root Treatment / Stage R 的 R2-T9 任务包，承接 R2-T8 和 2026-06-12 新策略，只迁移能实际降低 `lib.rs` 棘轮指标的低风险 Rust inline tests。

## 1. 目标

把 `prototypes/productized-desktop-shell/src-tauri/src/lib.rs` 中 memory candidate store、formal memory store 和 formal memory context 相关 inline tests 迁出到 crate-root test include 文件，继续降低 `lib.rs` 历史最低水位线。

目标文件：

- 新增：`prototypes/productized-desktop-shell/src-tauri/src/lib_memory_store_context_tests.rs`
- 修改：`prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- 修改：`scripts/harness/workbench-shape-gate.js`

预计收益：

- `lib.rs` 预计下降约 320-350 行。
- 新增 `.rs` test include 预计低于 3,000 行。
- 该切片降低 `lib.rs` 棘轮指标，符合新策略“不得立项不降低棘轮指标的拆分包”。

## 2. 允许范围

允许迁移以下 tests，保持测试体和断言语义不变：

- `memory_candidate_store_keeps_candidates_out_of_formal_memory`
- `candidate_sidecars_are_isolated_and_damaged_json_is_not_overwritten`
- `formal_memory_store_creates_record_version_and_audit`
- `formal_memory_context_accepts_matching_project_and_workflow`
- `formal_memory_context_rejects_mismatched_project_id`
- `formal_memory_context_rejects_mismatched_workflow_id`
- `formal_memory_context_rejects_project_director_cross_project`
- `formal_memory_context_rejects_missing_project_in_workflow_state`
- `formal_memory_context_keeps_existing_m1_guards`

允许：

- 在 `lib.rs` 原位置插入 `include!("lib_memory_store_context_tests.rs");`
- 让共享 helper 继续留在 `lib.rs`，包括 `fixture_memory_candidate_input`、`fixture_formal_memory_input`、`fixture_bound_formal_memory_input`、`fixture_bound_memory_scope`、`fixture_bound_memory_candidate_input`、`fixture_project` 等。
- 更新 shape gate 中 `lib.rs` waterline 为本轮完成后的历史最低收口值。
- 新增 evidence / handoff。

## 3. 禁止范围

禁止：

- 修改产品函数签名、可见性、语义或断言口径。
- 迁移 memory candidate adoption、formal memory adoption、task package、dispatch readiness、workflow execution runner、workflow machine、K3-B runtime prompt guard、ignored real-state tests、cross-store memory adoption 或共享 stub runner / factory。
- 迁移 `memory_candidate_adoption_*` tests。
- 修改 memory candidate store、formal memory store、formal memory context binding 或 task memory packet 产品语义。
- 修改 Tauri command、DB schema、sidecar schema、workflow state JSON schema。
- 修改前端 UI / CSS / TS。
- 执行真实 `codex exec` / `codex exec resume`、发送 prompt、读写 `/Users/yoyi/.codex`。
- 启动 Tauri / Browser / Chrome / Vite / 截图工具。

## 4. 验收

必须通过：

- `cargo test --lib memory_candidate_store`
- `cargo test --lib formal_memory_store`
- `cargo test --lib formal_memory_context`
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
- memory candidate adoption 迁移完成
- formal memory adoption 迁移完成
- memory candidate store、formal memory store 或 formal memory context 产品能力新增或语义变更
- task memory packet 产品能力新增或语义变更
- UI / 产品行为修改
- backlog 功能解冻

## 6. 执行记录

本轮已完成实现、本地验证和复核线只读审查。

实际改动：

- 新增 `prototypes/productized-desktop-shell/src-tauri/src/lib_memory_store_context_tests.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs` 原测试块替换为 `include!("lib_memory_store_context_tests.rs");`
- `scripts/harness/workbench-shape-gate.js` 的 `lib.rs` waterline 更新为 `8893`

实际形状收益：

- `lib.rs`：`9232 -> 8893`，下降 `339` 行。
- 新增 include 文件：`340` 行，低于 `.rs` 新文件上限 `3000`。

验证已通过：

- `cargo test --lib memory_candidate_store`：1 passed。
- `cargo test --lib formal_memory_store`：6 passed。
- `cargo test --lib formal_memory_context`：6 passed。
- `cargo test --lib task_memory_packet`：10 passed。
- `cargo test --lib`：471 passed，16 ignored。
- `cargo fmt -- --check`：通过。
- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors，0 warnings。
- `git diff --check`：通过。

保留既有 warning：

- `src/mcp/protocol.rs` 中 `JsonRpcError::invalid_params` dead_code warning。本轮未触碰该文件。

范围扫描：

- 新 include 只包含任务包允许的 9 个 tests。
- `memory_candidate_adoption_project_director_low_risk_project_memory` 仍留在 `lib.rs`。
- 新 include 中 `codex-workbench` 仅为 fixture `project_root` 字符串，不是 Codex 执行路径。

复核结论：

- `STATUS: CLEAR`
- 复核线程：`019eb850-0698-7f70-a9b2-e7d0d668ccf5`
- P0/P1/P2：无。
- 复核确认新 include 只包含任务包允许的 9 个 store/context tests；`memory_candidate_adoption_*`、dispatch readiness、workflow execution、workflow machine、K3-B guard、stub runner/factory 仍留在 `lib.rs`；shape gate waterline `8893` 与当前 `wc -l lib.rs` 一致。
