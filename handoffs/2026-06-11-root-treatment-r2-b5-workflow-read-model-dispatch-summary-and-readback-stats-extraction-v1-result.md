# Root Treatment R2-B5 Workflow Read Model Dispatch Summary And Readback Stats Extraction v1 Result

日期：2026-06-11

## 结论

R2-B5 本轮可接受为：任务包点名的 workflow read model、dispatch summary、readback stats 和相邻 workflow read surface 派生逻辑已从 `lib.rs` 物理抽出到 `workflow_read_model_entrypoints.rs`，并通过必需验证。行为保持不变，command 总量仍为 96，`lib.rs` 内 `#[tauri::command]` 仍为 0。

不接受为：R2 全部完成、`lib.rs <= 15,000` 水位线完成、C4-C6 自动化执行逻辑 / workflow machine / memory / runtime diagnostics / provider adapter / tests 巨石拆分完成、SQLite 迁移完成、workflow state schema 迁移或真实 Codex 执行恢复。

## 已完成

- 新增 `prototypes/productized-desktop-shell/src-tauri/src/workflow_read_model_entrypoints.rs`。
- 将 workflow state snapshot/read model、project blackboard、ledger/result/exception/interface/state-machine/acceptance scenarios、dispatch summary 和 readback stats helper 从 `lib.rs` 搬到新 helper 文件。
- `lib.rs` 原位置改为 `include!("workflow_read_model_entrypoints.rs")`。
- 新增 R2-B5 evidence：`evidence/2026-06-11-root-treatment-r2-b5-workflow-read-model-dispatch-summary-and-readback-stats-extraction-v1.md`。
- 未同步入口文档。

## 改动文件

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_read_model_entrypoints.rs`
- `evidence/2026-06-11-root-treatment-r2-b5-workflow-read-model-dispatch-summary-and-readback-stats-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r2-b5-workflow-read-model-dispatch-summary-and-readback-stats-extraction-v1-result.md`

## 抽出函数 / 类型

- `workflow_state_counts`
- `empty_workflow_state_snapshot`
- `project_workflow_summaries`
- `project_blackboards_from_workflows`
- `project_blackboard_from_workflow`
- `blackboard_candidate_entry`
- `blackboard_source_ref`
- `blackboard_promotion_decision`
- `task_draft_summaries`
- `derive_workflow_read_model`
- `derive_workflow_nodes`
- `derive_task_packages`
- `derive_workflow_ledger_entries`
- `derive_subagent_reports`
- `dispatch_role_from_node`
- `derive_review_results`
- `derive_workflow_result_summary_read_model`
- `pending_stage_c_acceptance_summary`
- `review_result_label`
- `derive_workflow_exceptions`
- `ledger_entry_type_from_audit`
- `compact_ledger_summary`
- `workflow_state_machine_summary`
- `workflow_transition_allowed`
- `workflow_node_transition_allowed`
- `director_completion_gate`
- `workflow_interface_boundaries`
- `interface_boundary`
- `workflow_acceptance_scenarios`
- `recent_audit_events_for`
- `workflow_node_session_binding_summaries`
- `workflow_node_dispatch_summaries`
- `workflow_dispatch_director_review_summaries`
- `workflow_execution_control_summaries`
- `workflow_permission_request_summaries`
- `workflow_execution_attempt_summaries`
- `parse_workflow_dispatch_director_review_record`
- `parse_workflow_execution_control_record`
- `parse_workflow_user_reviewed_instruction`
- `parse_workflow_permission_request_record`
- `parse_workflow_execution_attempt_record`
- `parse_workflow_node_dispatch_record`
- `dispatch_result_from_state`
- `dispatch_readback_stats`
- `dispatch_readback_stats_native`
- `dispatch_readback_stats_from_transcript`

常量 / test-only static 同步移入新 helper：

- `REVIEW_RETURN_EXCEPTION_THRESHOLD`
- `WORKFLOW_ALLOWED_TRANSITIONS`
- `NODE_ALLOWED_TRANSITIONS`
- `DISPATCH_READBACK_NATIVE_READ_COUNT`

本轮未迁移 Rust 类型定义；相关类型仍按既有 `types.rs` crate-root include 暴露。

## Shape 指标

- `lib.rs`：23,524 lines -> 21,463 lines。
- `workflow_read_model_entrypoints.rs`：新增 2,066 lines。
- Tauri command registry：96 total -> 96 total。
- `lib.rs` 内 `#[tauri::command]`：0 -> 0。
- Sidecar JSON kinds：14 detected / 0 unknown。

## 验证

已通过：

- `node scripts/harness/workbench-shape-gate.js --mode baseline`：pass，0 errors / 0 warnings；`lib.rs` 21,463 / 25,925，status `decreased`；Tauri commands 96 total / 0 in `lib.rs`。
- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors / 0 warnings；`lib.rs` 21,463 / 25,925，status `decreased`；Tauri commands 96 total / 0 in `lib.rs`。
- `cargo test --lib workflow_task_package_read_model`：1 passed / 0 failed / 351 filtered out。
- `cargo test --lib workflow_ledger`：1 passed / 0 failed / 351 filtered out。
- `cargo test --lib workflow_exception`：1 passed / 0 failed / 351 filtered out。
- `cargo test --lib workflow_interfaces`：1 passed / 0 failed / 351 filtered out。
- `cargo test --lib dispatch_readback_stats`：6 passed / 0 failed / 346 filtered out。
- `cargo test --lib workbench_snapshot`：1 passed / 0 failed / 351 filtered out。
- `cargo test --lib`：336 passed / 0 failed / 16 ignored。
- `cargo fmt -- --check`：通过。
- `git diff --check`：通过。
- `git status --short`：提交前仅包含 R2-B5 范围文件。

已知 warning：

- Rust 测试仍有既有 dead_code warning：`JsonRpcError::invalid_params` 未使用。

## Commit 记录

- Start commit：`86ce04032cce9ec1b1bd2970c78cd6be587b3cd9`
- End commit：本 R2-B5 提交承载；提交后真实 hash 由回交记录。

## 边界

- 未同步 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`。
- 未改产品业务逻辑。
- 未改函数语义、返回值、错误文案、公开 Tauri command 契约或 workflow state schema。
- 未新增 Tauri command。
- 未新增 sidecar / sidecar JSON kind。
- 未迁移 SQLite。
- 未做 UI。
- 未做 C4-C6 自动化执行逻辑 / workflow machine / memory / runtime diagnostics / provider adapter / tests 巨石其他拆分。
- 未执行真实 Codex。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript 或 rollout。
- 未启动 Tauri / Browser / Chrome / Vite / 截图工具。

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：`include!` 是保守过渡，后续 R2 可再收敛为正式模块边界。
- P2：本轮只抽出 workflow read model / dispatch summary / readback stats，不代表 R2 水位线、C4-C6 自动化执行逻辑、workflow machine、storage migration、memory、runtime diagnostics、provider adapter 或 tests 巨石拆分完成。
- P2：read-model / dispatch readback 相关测试仍主要留在 `lib.rs` inline tests，后续 R2 后段再迁移 tests。
