# Root Treatment R2-B10 C4-C6 Automation Workflow Governance Extraction v1 Evidence

日期：2026-06-11

## 结论

R2-B10 已完成行为不变的 C4-C6 自动化工作流治理区块物理抽出：

- 将 `ProjectDirectorAuthorizationContext` 到 `normalize_c4_symbol` 的连续区块搬入 `prototypes/productized-desktop-shell/src-tauri/src/c4_c6_workflow_governance_entrypoints.rs`。
- `lib.rs` 原位置保留 `include!("c4_c6_workflow_governance_entrypoints.rs")`，继续在 crate root 展开 helper，避免函数可见性改动。
- 未迁移 inline tests；`#[cfg(test)] mod tests` 仍留在 `lib.rs`。
- 未改函数语义、返回值、错误文案、公开 command/type/schema。

R2-B10 可接受为：C4-C6 project director plan、authorized dispatch、worker report、process fact、final review、user decision、acceptance summary 相关治理区块已从 `lib.rs` 抽出，`lib.rs` 行数继续下降，command 总量和 `lib.rs` 内 command 数量保持不变，并通过 shape gate、任务包指定 Rust 测试、全量库测试和格式检查。

## Commit

- Start commit：`b3392b09b1a2907fd75f6d81f75199d1a2da2b7b`
- End commit：本文件随 R2-B10 completion commit 一起提交；提交创建后由最终回交记录实际 hash。

说明：completion commit 的实际 hash 无法稳定写入同一 commit 内的文件内容；本 evidence 记录 start commit 和 completion commit 关系，实际 end hash 以开发线最终回交和主管线 checkpoint / backfill 为准。

## 改动文件

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/c4_c6_workflow_governance_entrypoints.rs`
- `evidence/2026-06-11-root-treatment-r2-b10-c4-c6-automation-workflow-governance-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r2-b10-c4-c6-automation-workflow-governance-extraction-v1-result.md`

## 形状指标

| 指标 | R2-B9 / start | R2-B10 / current |
| --- | ---: | ---: |
| `lib.rs` 行数 | 16,457 | 13,949 |
| `c4_c6_workflow_governance_entrypoints.rs` 行数 | 0 | 2,509 |
| Tauri command 总量 | 96 | 96 |
| `lib.rs` 内 `#[tauri::command]` | 0 | 0 |
| Sidecar JSON kinds | 14 detected / 0 unknown | 14 detected / 0 unknown |

说明：

- 新 helper 文件 2,509 行，低于 Rust 3,000 行治理阈值。
- 本轮后 `lib.rs` 已低于 15,000 行，这是事实记录；不自动声明 R2 完成或后续水位线完成。
- 本轮选择 crate-root `include!`。原因是抽出区块仍依赖 crate-root private helper、workflow state JSON helper、task memory injection、plan authorization store、observation bridge 和 inline tests；正式 `mod` 会要求扩大可见性修改，不符合本任务的行为不变和小风险边界。
- 本轮只搬移 C4-C6 自动化工作流治理区块；未拆 workflow execution、task package render、shared workflow utility、snapshot assembly、atomic helper 或 inline tests。

## 抽出范围

本轮抽出 `workflow_run_dispatch_entrypoints.rs` include 之后、`workflow_execution_entrypoints.rs` include 之前的连续 C4-C6 区块：

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

留在本批次外：

- `read_index` / transcript fallback loader。
- `workflow_execution_entrypoints.rs` 内部重构。
- task package render / finder helper。
- shared workflow utility。
- workbench snapshot assembly。
- atomic path / time helper。
- diagnostics/provider/session/index host app helper 的内部重构。
- SQLite migration。
- UI / TypeScript。
- worker_protocol / real_execution_command / project_workflow_automation 模块内部重构。
- inline tests 巨石。

## 验证命令

已运行：

```bash
node scripts/harness/workbench-shape-gate.js --mode baseline
node scripts/harness/workbench-shape-gate.js --mode check
cargo test --lib project_director
cargo test --lib worker_structured_report
cargo test --lib process_fact
cargo test --lib global_final_result_review
cargo test --lib user_result_decision
cargo test --lib stage_c_acceptance_summary
cargo test --lib workflow_state
cargo test --lib
cargo fmt -- --check
git diff --check
git status --short
```

结果：

- `node scripts/harness/workbench-shape-gate.js --mode baseline`：通过，Status `pass`，0 errors / 0 warnings / 12 info。
- baseline key metrics：`lib.rs` 13,949 lines；Tauri commands 96 total / 0 in `lib.rs`；Sidecar JSON kinds 14 detected / 0 unknown；`lib.rs` ratchet 13,949 / 25,925，status `decreased`。
- `node scripts/harness/workbench-shape-gate.js --mode check`：通过，Status `pass`，0 errors / 0 warnings / 12 info。
- check key metrics：`lib.rs` 13,949 lines；Tauri commands 96 total / 0 in `lib.rs`；Sidecar JSON kinds 14 detected / 0 unknown；`lib.rs` ratchet 13,949 / 25,925，status `decreased`。
- `cargo test --lib project_director`：通过，10 passed / 0 failed / 342 filtered out；filter 有匹配；保留既有 warning：`JsonRpcError::invalid_params` dead_code。
- `cargo test --lib worker_structured_report`：通过，2 passed / 0 failed / 350 filtered out；filter 有匹配；保留同一既有 warning。
- `cargo test --lib process_fact`：通过，3 passed / 0 failed / 349 filtered out；filter 有匹配；保留同一既有 warning。
- `cargo test --lib global_final_result_review`：通过，3 passed / 0 failed / 349 filtered out；filter 有匹配；保留同一既有 warning。
- `cargo test --lib user_result_decision`：通过，1 passed / 0 failed / 351 filtered out；filter 有匹配；保留同一既有 warning；输出含 Cargo artifact lock 等待提示但最终通过。
- `cargo test --lib stage_c_acceptance_summary`：通过，1 passed / 0 failed / 351 filtered out；filter 有匹配；保留同一既有 warning；输出含 Cargo artifact lock 等待提示但最终通过。
- `cargo test --lib workflow_state`：通过，11 passed / 0 failed / 341 filtered out；filter 有匹配；保留同一既有 warning。
- `cargo test --lib`：通过，336 passed / 0 failed / 16 ignored；ignored 均为显式真实执行授权测试；保留同一既有 warning。
- `cargo fmt -- --check`：通过，无输出。
- `git diff --check`：通过，无输出。
- `git status --short`：提交前仅包含 R2-B10 范围文件。

所有任务包点名 filters 均有匹配测试并通过；没有将无匹配或环境失败冒充通过。

## 边界确认

- 未改产品业务逻辑。
- 未改函数语义、返回值、错误文案或公开 Tauri command 契约。
- 未改 workflow state 顶层 schema。
- 未新增 `#[tauri::command]`。
- 未新增 sidecar store 或 sidecar JSON 种类。
- 未迁移 SQLite。
- 未做 UI / TypeScript / Browser / Tauri window / screenshot。
- 未改真实 Codex runner。
- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript 或 rollout。
- 未启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 未启动 Stage L / K3-B1 retry / K3-B2。
- 未解冻 backlog 功能。
- 未同步 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`。

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：继续使用 `include!` 作为保守过渡；后续 R2 可再收敛为正式模块边界。
- P2：本轮只抽出 C4-C6 自动化工作流治理区块，不代表 R2 全部完成、R3 SQLite、R4 按页读模型、UI、Stage L 恢复或真实 Codex 执行授权完成。
- P2：本轮虽然让 `lib.rs` 低于 15,000 行，但这只是事实记录；是否接受为第一阶段水位线达成需由主管线单独确认。
- P2：inline tests 仍主要留在 `lib.rs`，后续 R2 后段可按领域迁移 tests。
