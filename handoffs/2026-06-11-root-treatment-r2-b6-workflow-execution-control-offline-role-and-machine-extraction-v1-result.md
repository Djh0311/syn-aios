# Root Treatment R2-B6 Workflow Execution Control Offline Role And Machine Extraction v1 Result

日期：2026-06-11

## 结论

R2-B6 本轮可接受为：任务包点名的 workflow dispatch execution control、offline role dispatch、workflow machine run loop 和相邻 execution result helper 已从 `lib.rs` 物理抽出到 `workflow_execution_entrypoints.rs`，并通过必需验证。行为保持不变，command 总量仍为 96，`lib.rs` 内 `#[tauri::command]` 仍为 0。

不接受为：R2 全部完成、`lib.rs <= 15,000` 水位线完成、memory / runtime diagnostics / provider adapter / tests 巨石拆分完成、SQLite 迁移完成、workflow state schema 迁移或真实 Codex 执行恢复。

## 已完成

- 新增 `prototypes/productized-desktop-shell/src-tauri/src/workflow_execution_entrypoints.rs`。
- 将 workflow dispatch authorization / execution control、offline role dispatch / handoff / review、workflow machine helpers 从 `lib.rs` 搬到新 helper 文件。
- `lib.rs` 原位置改为 `include!("workflow_execution_entrypoints.rs")`。
- 新增 R2-B6 evidence：`evidence/2026-06-11-root-treatment-r2-b6-workflow-execution-control-offline-role-and-machine-extraction-v1.md`。
- 未同步入口文档。

## 改动文件

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_execution_entrypoints.rs`
- `evidence/2026-06-11-root-treatment-r2-b6-workflow-execution-control-offline-role-and-machine-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r2-b6-workflow-execution-control-offline-role-and-machine-extraction-v1-result.md`

## 抽出函数 / 类型

- `inspect_workflow_node_dispatch_authorization`
- `ensure_authorized_for_prepare`
- `role_id_from_node_id`
- `ensure_valid_dispatch_state`
- `validate_user_reviewed_instruction`
- `user_reviewed_instruction_value`
- `user_reviewed_instruction_input_from_value`
- `codex_resume_options_for_context`
- `render_user_reviewed_business_prompt`
- `classify_codex_resume_failure`
- `compact_failure_warning`
- `dedupe_strings`
- `write_prepared_dispatch`
- `write_started_dispatch`
- `write_completed_dispatch`
- `write_failed_dispatch`
- `write_readback_dispatch`
- `record_workflow_dispatch_director_review_at`
- `normalize_director_review_decision`
- `record_workflow_permission_decision_at`
- `permission_decision_label`
- `prepare_offline_role_dispatch_at`
- `record_offline_role_result_handoff_at`
- `record_offline_director_review_at`
- `validate_offline_role_dispatch_request`
- `validate_non_empty`
- `offline_role_dispatch_value`
- `offline_role_node_id`
- `offline_role_node_suffix`
- `offline_handoff_refs_for_dispatch`
- `has_pending_offline_role_dispatch`
- `run_workflow_machine_at`
- `validate_workflow_machine_request`
- `WorkflowMachineRoleStep`
- `workflow_machine_round_steps`
- `execute_workflow_machine_step`
- `workflow_machine_step_instruction`
- `render_workflow_machine_step_prompt`
- `workflow_machine_execution_root`
- `workflow_machine_final_acceptance`
- `reset_work_item_for_next_machine_step`
- `append_workflow_machine_run_started`
- `append_workflow_machine_run_finished`
- `workflow_machine_result_from_state`

本轮未迁移 Rust command wrapper、公开类型定义或 tests；相关类型仍按既有 `types.rs` crate-root include 暴露。

## Shape 指标

- `lib.rs`：21,463 lines -> 19,401 lines。
- `workflow_execution_entrypoints.rs`：新增 2,068 lines。
- Tauri command registry：96 total -> 96 total。
- `lib.rs` 内 `#[tauri::command]`：0 -> 0。
- Sidecar JSON kinds：14 detected / 0 unknown。

## 验证

已通过：

- `node scripts/harness/workbench-shape-gate.js --mode baseline`：pass，0 errors / 0 warnings；`lib.rs` 19,401 / 25,925，status `decreased`；Tauri commands 96 total / 0 in `lib.rs`。
- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors / 0 warnings；`lib.rs` 19,401 / 25,925，status `decreased`；Tauri commands 96 total / 0 in `lib.rs`。
- `cargo test --lib workflow_dispatch`：2 passed / 0 failed / 1 ignored / 349 filtered out。
- `cargo test --lib workflow_node_dispatch`：11 passed / 0 failed / 341 filtered out。
- `cargo test --lib offline_role`：3 passed / 0 failed / 349 filtered out。
- `cargo test --lib workflow_machine`：2 passed / 0 failed / 350 filtered out。
- `cargo test --lib workflow_permission`：1 passed / 0 failed / 351 filtered out。
- `cargo test --lib`：336 passed / 0 failed / 16 ignored。
- `cargo fmt -- --check`：通过。
- `git diff --check`：通过。
- `git status --short`：提交前仅包含 R2-B6 范围文件。

已知 warning：

- Rust 测试仍有既有 dead_code warning：`JsonRpcError::invalid_params` 未使用。

## Commit 记录

- Start commit：`93c20bb5b515ced0f0306ec57d61b222a592a08a`
- End commit：本文件随 R2-B6 completion commit 一起提交；提交创建后由最终回交记录实际 hash。

## 边界

- 未同步 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`。
- 未改产品业务逻辑。
- 未改函数语义、返回值、错误文案、公开 Tauri command 契约或 workflow state schema。
- 未新增 Tauri command。
- 未新增 sidecar / sidecar JSON kind。
- 未迁移 SQLite。
- 未做 UI。
- 未做 memory / runtime diagnostics / provider adapter / tests 巨石其他拆分。
- 未执行真实 Codex。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript 或 rollout。
- 未启动 Tauri / Browser / Chrome / Vite / 截图工具。

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：`include!` 是保守过渡，后续 R2 可再收敛为正式模块边界。
- P2：本轮只抽出 workflow dispatch execution control / offline role dispatch / workflow machine，不代表 R2 水位线、memory、runtime diagnostics、provider adapter、storage migration、UI 或 tests 巨石拆分完成。
- P2：相关测试仍主要留在 `lib.rs` inline tests，后续 R2 后段再迁移 tests。
