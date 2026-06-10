# Root Treatment R2-B6 Workflow Execution Control Offline Role And Machine Extraction v1

日期：2026-06-11

## 结论

R2-B6 本轮完成行为不变的第六批 `lib.rs` 形状治理：

- 新增 `prototypes/productized-desktop-shell/src-tauri/src/workflow_execution_entrypoints.rs`。
- 将 workflow dispatch execution control、offline role dispatch、workflow machine run loop 和相邻 execution result helper 从 `lib.rs` 物理移入新 helper 文件。
- `lib.rs` 原位置保留 `include!("workflow_execution_entrypoints.rs")`，继续在 crate root 展开，避免修改 helper 可见性。
- 未修改函数语义、返回值、错误文案、公开 Tauri command 契约或 workflow state schema。

R2-B6 可接受为：workflow dispatch execution control / offline role dispatch / workflow machine 相关 helper 已从 `lib.rs` 抽出，`lib.rs` 行数继续下降，command 总量和 `lib.rs` 内 command 数量保持不变，并通过 shape gate、任务包指定 Rust 聚焦测试、全量库测试和格式检查。

R2-B6 不接受为：R2 全部完成、`lib.rs <= 15,000` 或 `<= 3,000` 目标完成、memory / runtime diagnostics / provider adapter / tests 巨石拆分完成、SQLite 迁移完成、workflow state schema 迁移完成、Stage L / K3-B1 / K3-B2 恢复或新的真实 Codex 执行授权。

## Commit 记录

- Start commit：`93c20bb5b515ced0f0306ec57d61b222a592a08a`
- End commit：本文件随 R2-B6 completion commit 一起提交；提交创建后由最终回交记录实际 hash。

## 改动文件

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_execution_entrypoints.rs`
- `evidence/2026-06-11-root-treatment-r2-b6-workflow-execution-control-offline-role-and-machine-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r2-b6-workflow-execution-control-offline-role-and-machine-extraction-v1-result.md`

未修改：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_run_dispatch_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_read_model_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_lifecycle_task_package.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_json_helpers.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs`

## 抽出清单

以下函数 / 类型已从 `lib.rs` 移入 `workflow_execution_entrypoints.rs`：

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

本轮未迁移 Rust command wrapper、公开类型定义或 tests；相关类型仍按既有 `types.rs` crate-root include 暴露，测试仍保留在既有 inline tests 中。

## Shape 指标

| 指标 | R2-B6 前 | R2-B6 后 |
| --- | ---: | ---: |
| `lib.rs` 行数 | 21,463 | 19,401 |
| `workflow_execution_entrypoints.rs` 行数 | 0 | 2,068 |
| Tauri command registry 总量 | 96 | 96 |
| `lib.rs` 内 `#[tauri::command]` 数量 | 0 | 0 |
| Sidecar JSON kinds | 14 allowed / 0 unknown | 14 allowed / 0 unknown |

说明：

- `lib.rs` 较 R2-B5 checkpoint 继续减少 2,062 行。
- 新 helper 文件 2,068 行，低于 Rust 3,000 行治理阈值。
- 本轮选择 crate-root `include!`。原因是抽出块依赖大量 crate-root private helper 和既有 inline tests；正式 `mod` 会要求扩大可见性修改，不符合本任务的行为不变和小风险边界。
- 本轮只搬移 workflow dispatch execution control / offline role dispatch / workflow machine 相关 helper；未拆 memory、runtime diagnostics、provider adapter、SQLite、UI 或 tests 巨石。

## 代码地图摘要

R2-B6 对应原 `lib.rs` 中段连续块：

- workflow node dispatch authorization、prepared / started / completed / failed / readback dispatch 写入链。
- workflow dispatch director review 和 workflow permission decision 写入链。
- offline role dispatch、handoff、director review 和 pending guard。
- workflow machine request validation、round steps、step instruction、prompt render、execution root、final acceptance、started / finished append 和 result assembly。

R2-B6 已将上述函数抽入 `workflow_execution_entrypoints.rs`，`lib.rs` 原位置仅保留 crate-root `include!`。后续 memory、diagnostics、provider adapter、SQLite、UI、tests 迁移仍等待后续批次。

## 验证记录

已运行：

```bash
node scripts/harness/workbench-shape-gate.js --mode baseline
node scripts/harness/workbench-shape-gate.js --mode check
cargo test --lib workflow_dispatch
cargo test --lib workflow_node_dispatch
cargo test --lib offline_role
cargo test --lib workflow_machine
cargo test --lib workflow_permission
cargo test --lib
cargo fmt -- --check
git diff --check
git status --short
```

结果：

- `node scripts/harness/workbench-shape-gate.js --mode baseline`：通过，Status `pass`，0 errors / 0 warnings / 12 info。
- baseline key metrics：`lib.rs` 19,401 lines；Tauri commands 96 total / 0 in `lib.rs`；Sidecar JSON kinds 14 detected / 0 unknown；`lib.rs` ratchet 19,401 / 25,925，status `decreased`。
- `node scripts/harness/workbench-shape-gate.js --mode check`：通过，Status `pass`，0 errors / 0 warnings / 12 info。
- check key metrics：`lib.rs` 19,401 lines；Tauri commands 96 total / 0 in `lib.rs`；Sidecar JSON kinds 14 detected / 0 unknown；`lib.rs` ratchet 19,401 / 25,925，status `decreased`。
- `cargo test --lib workflow_dispatch`：通过，2 passed / 0 failed / 1 ignored / 349 filtered out；保留既有 warning：`JsonRpcError::invalid_params` dead_code。
- `cargo test --lib workflow_node_dispatch`：通过，11 passed / 0 failed / 341 filtered out；保留同一既有 warning。
- `cargo test --lib offline_role`：通过，3 passed / 0 failed / 349 filtered out；保留同一既有 warning。
- `cargo test --lib workflow_machine`：通过，2 passed / 0 failed / 350 filtered out；保留同一既有 warning。
- `cargo test --lib workflow_permission`：通过，1 passed / 0 failed / 351 filtered out；保留同一既有 warning。
- `cargo test --lib`：通过，336 passed / 0 failed / 16 ignored；保留同一既有 warning。
- `cargo fmt -- --check`：通过，无输出。
- `git diff --check`：通过，无输出。
- `git status --short`：提交前仅包含 R2-B6 范围文件。

所有任务包点名 filters 均有匹配测试并通过；没有将无匹配或环境失败冒充通过。

## 边界确认

- 未改产品业务逻辑。
- 未改函数语义、返回值、错误文案或公开 Tauri command 契约。
- 未改 workflow state 顶层 schema。
- 未新增 `#[tauri::command]`。
- 未新增 sidecar store 或 sidecar JSON 种类。
- 未迁移 SQLite。
- 未做 UI。
- 未做 memory / runtime diagnostics / provider adapter / tests 巨石其他拆分。
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
- P2：本轮只抽出 workflow dispatch execution control / offline role dispatch / workflow machine，不代表 R2 水位线、memory、runtime diagnostics、provider adapter、storage migration、UI 或 tests 巨石拆分完成。
- P2：相关测试仍主要保留在 `lib.rs` inline tests 中，后续 R2 后段可按领域迁移 tests。
