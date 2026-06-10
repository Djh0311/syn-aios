# Root Treatment R2-B5 Workflow Read Model Dispatch Summary And Readback Stats Extraction v1

日期：2026-06-11

## 结论

R2-B5 本轮完成行为不变的第五批 `lib.rs` 形状治理：

- 新增 `prototypes/productized-desktop-shell/src-tauri/src/workflow_read_model_entrypoints.rs`。
- 将 workflow state snapshot/read model、project blackboard、workflow ledger/result/exception/interface/state-machine/acceptance scenarios、dispatch summary、readback stats 及相邻 workflow surface readback helper 从 `lib.rs` 物理移入新 helper 文件。
- `lib.rs` 原位置保留 `include!("workflow_read_model_entrypoints.rs")`，继续在 crate root 展开，避免修改 helper 可见性。
- 未修改函数语义、返回值、错误文案、公开 Tauri command 契约或 workflow state schema。

R2-B5 可接受为：workflow read model / dispatch summary / readback stats 相关派生逻辑已从 `lib.rs` 抽出，`lib.rs` 行数继续下降，command 总量和 `lib.rs` 内 command 数量保持不变，并通过 shape gate、任务包指定 Rust 聚焦测试、全量库测试和格式检查。

R2-B5 不接受为：R2 全部完成、`lib.rs <= 15,000` 或 `<= 3,000` 目标完成、C4-C6 自动化执行逻辑拆分完成、workflow machine / memory / runtime diagnostics / provider adapter / tests 巨石拆分完成、SQLite 迁移完成、workflow state schema 迁移完成、Stage L / K3-B1 / K3-B2 恢复或新的真实 Codex 执行授权。

## Commit 记录

- Start commit：`86ce04032cce9ec1b1bd2970c78cd6be587b3cd9`
- End commit：本 R2-B5 提交承载；提交后真实 hash 由回交记录。

## 改动文件

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_read_model_entrypoints.rs`
- `evidence/2026-06-11-root-treatment-r2-b5-workflow-read-model-dispatch-summary-and-readback-stats-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r2-b5-workflow-read-model-dispatch-summary-and-readback-stats-extraction-v1-result.md`

未修改：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_read_model.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_run_dispatch_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_lifecycle_task_package.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_json_helpers.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs`

## 抽出清单

以下函数 / 常量 / test-only static 已从 `lib.rs` 移入 `workflow_read_model_entrypoints.rs`：

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
- `REVIEW_RETURN_EXCEPTION_THRESHOLD`
- `ledger_entry_type_from_audit`
- `compact_ledger_summary`
- `workflow_state_machine_summary`
- `WORKFLOW_ALLOWED_TRANSITIONS`
- `NODE_ALLOWED_TRANSITIONS`
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
- `DISPATCH_READBACK_NATIVE_READ_COUNT`

本轮未迁移 Rust 类型定义；相关类型仍按既有 `types.rs` crate-root include 暴露。

仍保留在 `lib.rs` 的相邻 dispatch / app assembly 边界：

- `next_workflow_node_dispatch_id`
- `safe_probe_target`
- `safe_probe_prompt`
- `compact_last_message_summary`
- `default_workflow_node_dispatch_output_dir`
- `atomic_write_json`
- `default_workflow_state_path`
- `workspace_id`
- `unix_timestamp_string`
- `unix_timestamp_ms`
- `unix_timestamp_nanos`
- `build_snapshot`
- `build_snapshot_with_session_source`

## Shape 指标

| 指标 | R2-B5 前 | R2-B5 后 |
| --- | ---: | ---: |
| `lib.rs` 行数 | 23,524 | 21,463 |
| `workflow_read_model_entrypoints.rs` 行数 | 0 | 2,066 |
| Tauri command registry 总量 | 96 | 96 |
| `lib.rs` 内 `#[tauri::command]` 数量 | 0 | 0 |
| Sidecar JSON kinds | 14 allowed / 0 unknown | 14 allowed / 0 unknown |

说明：

- `lib.rs` 较 R2-B4 checkpoint 继续减少 2,061 行。
- 新 helper 文件 2,066 行，低于 Rust 3,000 行治理阈值。
- 本轮选择 crate-root `include!`，没有扩展既有 `workflow_read_model.rs`。原因是抽出块依赖大量 crate-root private helper 和既有 inline tests；直接并入正式模块需要扩大可见性修改，`include!` 更符合本任务的行为不变和小风险要求。
- 本轮只搬移 workflow read model / dispatch summary / readback stats 及相邻 readback result surface；未拆 C4-C6 自动化执行逻辑、workflow machine、memory、runtime diagnostics、provider adapter、SQLite、UI 或 tests 巨石。

## 代码地图摘要

R2 代码地图中 R2-B5 对应原 `lib.rs` 中段连续块：

- workflow state snapshot counts / empty snapshot。
- project workflow summary 和 project blackboard read model。
- workflow nodes、task packages、ledger、subagent reports、review results、result summary、exceptions。
- workflow state machine / interface / acceptance scenarios read model。
- workflow node binding / dispatch / review / control / permission / attempt summary parsing。
- dispatch result readback helper 和 native transcript readback stats。

R2-B5 已将上述函数抽入 `workflow_read_model_entrypoints.rs`，`lib.rs` 原位置仅保留 crate-root `include!`。后续 C4-C6 自动化执行控制、workflow machine、memory、diagnostics 和 tests 迁移仍等待后续 R2 批次。

## 验证记录

已运行：

```bash
node scripts/harness/workbench-shape-gate.js --mode baseline
node scripts/harness/workbench-shape-gate.js --mode check
cargo test --lib workflow_task_package_read_model
cargo test --lib workflow_ledger
cargo test --lib workflow_exception
cargo test --lib workflow_interfaces
cargo test --lib dispatch_readback_stats
cargo test --lib workbench_snapshot
cargo test --lib
cargo fmt -- --check
git diff --check
git status --short
```

结果：

- `node scripts/harness/workbench-shape-gate.js --mode baseline`：通过，Status `pass`，0 errors / 0 warnings / 12 info。
- baseline key metrics：`lib.rs` 21,463 lines；Tauri commands 96 total / 0 in `lib.rs`；Sidecar JSON kinds 14 detected / 0 unknown；`lib.rs` ratchet 21,463 / 25,925，status `decreased`。
- `node scripts/harness/workbench-shape-gate.js --mode check`：通过，Status `pass`，0 errors / 0 warnings / 12 info。
- check key metrics：`lib.rs` 21,463 lines；Tauri commands 96 total / 0 in `lib.rs`；Sidecar JSON kinds 14 detected / 0 unknown；`lib.rs` ratchet 21,463 / 25,925，status `decreased`。
- `cargo test --lib workflow_task_package_read_model`：通过，1 passed / 0 failed / 351 filtered out；保留既有 warning：`JsonRpcError::invalid_params` dead_code。
- `cargo test --lib workflow_ledger`：通过，1 passed / 0 failed / 351 filtered out；保留同一既有 warning。
- `cargo test --lib workflow_exception`：通过，1 passed / 0 failed / 351 filtered out；保留同一既有 warning。
- `cargo test --lib workflow_interfaces`：通过，1 passed / 0 failed / 351 filtered out；保留同一既有 warning。
- `cargo test --lib dispatch_readback_stats`：通过，6 passed / 0 failed / 346 filtered out；保留同一既有 warning。
- `cargo test --lib workbench_snapshot`：通过，1 passed / 0 failed / 351 filtered out；保留同一既有 warning。
- `cargo test --lib`：通过，336 passed / 0 failed / 16 ignored；保留同一既有 warning。
- `cargo fmt -- --check`：通过，无输出。
- `git diff --check`：通过，无输出。
- `git status --short`：提交前仅包含 R2-B5 范围文件。

所有任务包点名 filters 均有匹配测试并通过；没有将无匹配或环境失败冒充通过。

## 边界确认

- 未改产品业务逻辑。
- 未改函数语义、返回值、错误文案或公开 Tauri command 契约。
- 未改 workflow state 顶层 schema。
- 未新增 `#[tauri::command]`。
- 未新增 sidecar store 或 sidecar JSON 种类。
- 未迁移 SQLite。
- 未做 UI。
- 未做 C4-C6 自动化执行逻辑 / workflow machine / memory / runtime diagnostics / provider adapter / tests 巨石其他拆分。
- 未改真实 Codex runner。
- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript 或 rollout。
- 未启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 未启动 Stage L / K3-B1 retry / K3-B2。
- 未解冻 backlog 功能。

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：继续使用 `include!` 作为保守过渡，后续 R2 可再收敛为正式模块边界。
- P2：本轮只抽出 workflow read model / dispatch summary / readback stats，不代表 C4-C6 自动化执行逻辑、workflow machine、storage migration、memory、runtime diagnostics、provider adapter 或 tests 巨石拆分完成。
- P2：read-model / dispatch readback 相关测试仍主要保留在 `lib.rs` inline tests 中，后续 R2 后段可按领域迁移 tests。
