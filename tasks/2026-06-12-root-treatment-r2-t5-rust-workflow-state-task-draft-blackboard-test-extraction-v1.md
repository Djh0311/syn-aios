# Root Treatment / R2-T5 Rust Workflow State Task Draft Blackboard Test Extraction v1

日期：2026-06-12

状态：实现完成，待复核，hash 待回填。

Planning baseline commit：`8cd4ee6569dd6131c46ae5ed3e4ead7a3d5e6fb3`

Implementation commit：`3465e4dc96c5141861513d9add37cf7cbddf1440`

Review result：`TBD`

Checkpoint commit：`TBD`

本文是 Root Treatment / Stage R 的 R2-T5 任务包，承接 R4-A50 新策略和 R2-T1 到 R2-T4 inline tests 迁移结果，继续迁移能实际降低 `lib.rs` 棘轮指标的低风险 Rust inline tests。

## 1. 目标

把 `prototypes/productized-desktop-shell/src-tauri/src/lib.rs` 中 workflow state bootstrap、task draft、work item state、workflow audit helper 和 blackboard candidate boundary 相关 inline tests 迁出到 crate-root test include 文件，继续降低 `lib.rs` 历史最低水位线。

目标文件：

- 新增：`prototypes/productized-desktop-shell/src-tauri/src/lib_workflow_state_task_draft_blackboard_tests.rs`
- 修改：`prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- 修改：`scripts/harness/workbench-shape-gate.js`

预计收益：

- `lib.rs` 预计下降约 600 行以上。
- 新增 `.rs` test include 预计低于 3,000 行。
- 该切片降低 `lib.rs` 棘轮指标，符合新策略“不得立项不降低棘轮指标的拆分包”。

## 2. 允许范围

允许迁移以下 tests，保持测试体和断言语义不变：

- `missing_workflow_state_returns_empty_without_creating_file`
- `initializes_workflow_state_with_audit_event`
- `existing_workflow_state_is_backed_up_before_initialize`
- `bootstrap_project_workflow_initializes_missing_state`
- `bootstrap_project_workflow_does_not_duplicate_existing_workflow`
- `bootstrap_project_workflow_rejects_non_index_project`
- `bootstrap_project_workflow_backs_up_existing_state`
- `task_draft_rejects_missing_workflow_state`
- `task_draft_rejects_project_without_workflow`
- `task_draft_creates_work_item_artifact_and_audit`
- `task_draft_rejects_non_index_project`
- `task_draft_backs_up_existing_state_before_write`
- `task_draft_does_not_duplicate_same_workflow_title`
- `work_item_state_update_advances_state_node_and_audit`
- `work_item_state_update_rejects_illegal_transition`
- `workflow_state_store_helpers_preserve_write_and_backup_behavior`
- `workflow_permission_decision_records_audit_through_control_core`
- `workflow_audit_helper_preserves_work_item_state_changed_fields`
- `workflow_audit_helper_preserves_permission_decision_fields`
- `blackboard_candidate_decision_boundary_rejects_direct_promotion`
- `blackboard_candidate_store_records_candidate_only_decisions`

允许：

- 在 `lib.rs` 原位置插入 `include!("lib_workflow_state_task_draft_blackboard_tests.rs");`
- 更新 shape gate 中 `lib.rs` waterline 为本轮完成后的历史最低收口值。
- 新增 evidence / handoff。

## 3. 禁止范围

禁止：

- 修改产品函数签名、可见性、语义或断言口径。
- 迁移 K3-B runtime prompt guard、workflow execution runner、workflow machine、ignored real-state tests、cross-store memory adoption、共享 stub runner / factory。
- 迁移 observation、task memory packet、formal memory adoption、dispatch execution、offline role、workflow machine 或真实状态相关 tests。
- 修改 Tauri command、DB schema、sidecar schema、workflow state JSON schema。
- 修改前端 UI / CSS / TS。
- 执行真实 `codex exec` / `codex exec resume`、发送 prompt、读写 `/Users/yoyi/.codex`。
- 启动 Tauri / Browser / Chrome / Vite / 截图工具。

## 4. 验收

必须通过：

- `cargo test --lib workflow_state`
- `cargo test --lib task_draft`
- `cargo test --lib work_item_state`
- `cargo test --lib workflow_audit`
- `cargo test --lib blackboard_candidate`
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
- UI / 产品行为修改
- backlog 功能解冻

## 6. 执行记录

本轮已完成实现和本地验证，等待复核线只读审查。

实际改动：

- 新增 `prototypes/productized-desktop-shell/src-tauri/src/lib_workflow_state_task_draft_blackboard_tests.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs` 原测试块替换为 `include!("lib_workflow_state_task_draft_blackboard_tests.rs");`
- `scripts/harness/workbench-shape-gate.js` 的 `lib.rs` waterline 更新为 `10279`

实际形状收益：

- `lib.rs`：`10943 -> 10279`，下降 `664` 行。
- 新增 include 文件：`664` 行，低于 `.rs` 新文件上限 `3000`。

验证已通过：

- `cargo test --lib workflow_state`：11 passed。
- `cargo test --lib task_draft`：6 passed。
- `cargo test --lib work_item_state`：4 passed。
- `cargo test --lib workflow_audit`：2 passed。
- `cargo test --lib blackboard_candidate`：2 passed。
- `cargo test --lib`：471 passed，16 ignored。
- `cargo fmt -- --check`：通过。
- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors，0 warnings。
- `git diff --check`：通过。

保留既有 warning：

- `src/mcp/protocol.rs` 中 `JsonRpcError::invalid_params` dead_code warning。本轮未触碰该文件。
