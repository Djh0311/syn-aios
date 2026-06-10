# Root Treatment R2-B10 C4-C6 Automation Workflow Governance Extraction v1 Result

日期：2026-06-11

## 结论

R2-B10 已完成。接受范围是 `src-tauri/src/lib.rs` 中从 `ProjectDirectorAuthorizationContext` 到 `normalize_c4_symbol` 的 C4-C6 自动化工作流治理连续区块，已抽出到 `src-tauri/src/c4_c6_workflow_governance_entrypoints.rs`，并通过 `include!("c4_c6_workflow_governance_entrypoints.rs")` 在 crate root 展开，保持行为和可见性不变。

不接受为 R2 全部完成、R3 SQLite、R4 UI / 按页读模型、Stage L / K3-B1 / K3-B2 恢复或新的真实 Codex 执行授权。`lib.rs` 已低于 15,000 行是事实记录，是否收口第一阶段水位线由主管线确认。

## 做了什么

- 新增 `c4_c6_workflow_governance_entrypoints.rs`。
- `lib.rs` 原 C4-C6 区块替换为 crate-root `include!`。
- 未移动 inline tests；测试巨石仍留在 `lib.rs`。
- 未同步入口文档，入口同步留给主管线 checkpoint。

## 改动文件

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/c4_c6_workflow_governance_entrypoints.rs`
- `evidence/2026-06-11-root-treatment-r2-b10-c4-c6-automation-workflow-governance-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r2-b10-c4-c6-automation-workflow-governance-extraction-v1-result.md`

## 指标

- `lib.rs`：16,457 lines -> 13,949 lines。
- `c4_c6_workflow_governance_entrypoints.rs`：新增 2,509 lines。
- Tauri command registry：96 total，`lib.rs` 内 `#[tauri::command]` 为 0。
- Sidecar JSON kinds：14 detected / 0 unknown。

## 抽出函数 / 类型

- `ProjectDirectorAuthorizationContext`
- `ActiveBindingInfo`
- `preview_project_director_task_plan_for_index_at`
- `prepare_authorized_auto_dispatch_for_index_at`
- `record_worker_structured_report_at`
- `record_project_director_process_fact_decision_at`
- `record_global_final_result_review_at`
- `record_user_result_decision_at`
- `generate_stage_c_acceptance_summary_at`
- `validate_expected_workflow_revision`
- `validate_global_final_result_review_input`
- `validate_user_result_decision_input`
- `validate_generate_stage_c_acceptance_summary_input`
- `validate_c6_prerequisites_for_final_review`
- `has_c4_prepared_dispatch`
- `has_c4_task_package_artifact`
- `artifact_belongs_to_workflow`
- `has_worker_report_for_workflow`
- `process_fact_reviews_for_workflow`
- `has_process_fact_decision_for_workflow`
- `confirmed_process_fact_ids_for_workflow`
- `unresolved_process_fact_decisions`
- `latest_global_final_review`
- `latest_user_result_decision`
- `latest_stage_c_acceptance_artifact`
- `build_stage_c_acceptance_summary`
- `stage_c_gate`
- `evidence_refs_for_c4`
- `evidence_refs_for_c5`
- `validate_worker_structured_report_input`
- `validate_c5_source_ref`
- `validate_process_fact_decision_input`
- `validate_process_fact_candidate`
- `find_worker_report_event`
- `process_fact_decision_exists`
- `read_c4_workflow_value`
- `project_director_authorization_context`
- `deterministic_project_director_planned_tasks`
- `annotate_project_director_planned_tasks`
- `project_director_task_plan_from_tasks`
- `aggregate_project_director_memory_summary`
- `prepared_dispatch_read_models_from_plan`
- `c4_static_task_blocking_reasons`
- `ensure_c4_backup`
- `ensure_project_director_worker_node`
- `ensure_project_director_work_item`
- `ensure_project_director_task_package_artifact`
- `project_director_memory_snapshot`
- `active_binding_for_planned_task`
- `find_task_package_artifact_by_id`
- `existing_prepared_dispatch_for_planned_task`
- `push_authorized_prepared_dispatch_created_audit`
- `push_authorized_prepared_dispatch_blocked_audit`
- `render_project_director_prepared_prompt`
- `c4_work_item_id`
- `c4_task_package_artifact_id`
- `c4_node_id`
- `standard_project_director_report_format`
- `edge_exists`
- `push_unique`
- `normalize_c4_symbol`

## 验证

已通过：

- `node scripts/harness/workbench-shape-gate.js --mode baseline`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `cargo test --lib project_director`，10 passed / 0 failed
- `cargo test --lib worker_structured_report`，2 passed / 0 failed
- `cargo test --lib process_fact`，3 passed / 0 failed
- `cargo test --lib global_final_result_review`，3 passed / 0 failed
- `cargo test --lib user_result_decision`，1 passed / 0 failed
- `cargo test --lib stage_c_acceptance_summary`，1 passed / 0 failed
- `cargo test --lib workflow_state`，11 passed / 0 failed
- `cargo test --lib`，336 passed / 0 failed / 16 ignored
- `cargo fmt -- --check`
- `git diff --check`
- `git status --short`，提交前仅包含 R2-B10 范围文件

所有任务包点名 cargo filters 均有匹配测试并通过；没有使用 fallback 冒充通过。

已知 warning：

- Rust 测试仍有既有 dead_code warning：`JsonRpcError::invalid_params` 未使用。

## Commit

- Start commit：`b3392b09b1a2907fd75f6d81f75199d1a2da2b7b`
- End commit：本文件随 R2-B10 completion commit 一起提交；提交创建后由最终回交记录实际 hash。

说明：completion commit 的实际 hash 无法稳定写入同一 commit 内的文件内容；本 handoff 记录 start commit 和 completion commit 关系，实际 end hash 以开发线最终回交和主管线 checkpoint / backfill 为准。

## 边界

- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript 或 rollout。
- 未启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 未迁移 SQLite，未改 workflow state schema，未新增 sidecar JSON 种类，未新增 Tauri command。
- 未改 UI / TypeScript。
- 未同步入口文档。

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：`include!` 仍是保守过渡，后续 R2 可再收敛正式模块边界。
- P2：R2-B10 不是 R2 全部完成；`lib.rs <= 15,000` 是本轮达到的事实，是否接受为阶段水位线达成需主管线确认。
- P2：inline tests 仍主要留在 `lib.rs`，后续 R2 后段再迁移。

## 不能声明完成

- 不能声明 R2 全部完成。
- 不能声明 R3 SQLite 或 workflow state schema 迁移完成。
- 不能声明 UI / Tauri 截图验收完成。
- 不能声明真实 Codex send / resume / exec、新真实执行授权、K3-B1 retry、K3-B2 或 Stage L 恢复完成。
- 不能声明 workflow execution、task package render、shared workflow utility、snapshot assembly、atomic helper、SQLite、UI 或 inline tests 巨石已经拆完。
