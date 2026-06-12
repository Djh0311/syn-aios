# Root Treatment / R2-T10 Rust Stage C Governance Test Extraction v1

日期：2026-06-12

状态：待执行。

Planning baseline commit：`bcf17fa72928a4f772022f67194bb67f2d2f08bc`

本文是 Root Treatment / Stage R 的 R2-T10 任务包，承接 R2-T9 和 2026-06-12 新策略，只迁移能实际降低 `lib.rs` 棘轮指标的低风险 Rust inline tests。

## 1. 目标

把 `prototypes/productized-desktop-shell/src-tauri/src/lib.rs` 中 Stage C / C4-C6 治理闭环相关 inline tests 迁出到 crate-root test include 文件，继续降低 `lib.rs` 历史最低水位线。

目标文件：

- 新增：`prototypes/productized-desktop-shell/src-tauri/src/lib_stage_c_governance_tests.rs`
- 修改：`prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- 修改：`scripts/harness/workbench-shape-gate.js`

预计收益：

- `lib.rs` 预计下降约 800-900 行。
- 新增 `.rs` test include 预计低于 3,000 行。
- 该切片降低 `lib.rs` 棘轮指标，符合新策略“不得立项不降低棘轮指标的拆分包”。

## 2. 允许范围

允许迁移以下 tests，保持测试体和断言语义不变：

- `project_director_task_plan_rejects_without_active_c3_authorization`
- `project_director_task_plan_rejects_proposal_authorization_mismatch`
- `project_director_task_plan_blocks_out_of_scope_planned_task`
- `authorized_prepared_dispatch_needs_binding_without_executable_dispatch`
- `authorized_prepared_dispatch_creates_memory_snapshot_and_remains_unexecuted_and_idempotent`
- `worker_structured_report_rejects_missing_evidence_and_ordinary_chat_source`
- `worker_structured_report_records_audit_without_observation_or_formal_memory`
- `project_director_process_fact_confirmation_writes_recorded_observation_only`
- `project_director_process_fact_decision_rejects_wrong_actor_and_unsafe_facts`
- `process_fact_duplicate_is_rejected_and_rework_does_not_write_observation`
- `global_final_result_review_rejects_missing_c2_and_c3_prerequisites`
- `global_final_result_review_records_review_without_memory_or_user_acceptance`
- `global_final_result_review_rejects_wrong_actor`
- `user_result_decision_requires_user_and_does_not_write_memory`
- `stage_c_acceptance_summary_records_gates_and_deferred_items`

允许：

- 在 `lib.rs` 原位置插入 `include!("lib_stage_c_governance_tests.rs");`
- 让共享 helper 继续留在 `lib.rs`，包括 `create_active_project_director_authorization_fixture`、`setup_c5_worker_report_fixture`、`setup_c6_complete_fixture`、`fixture_*` helper 等。
- 更新 shape gate 中 `lib.rs` waterline 为本轮完成后的历史最低收口值。
- 新增 evidence / handoff。

## 3. 禁止范围

禁止：

- 修改产品函数签名、可见性、语义或断言口径。
- 迁移 helper / fixture builder / stub runner / runner fake。
- 迁移 memory candidate adoption、formal memory adoption、workflow execution runner、workflow machine、K3-B runtime prompt guard、ignored real-state tests、cross-store memory adoption、legacy dispatch execution 或真实 execution tests。
- 修改 Stage C 产品语义、workflow state JSON schema、Tauri command、DB schema 或 sidecar schema。
- 修改前端 UI / CSS / TS。
- 执行真实 `codex exec` / `codex exec resume`、发送 prompt、读写 `/Users/yoyi/.codex`。
- 启动 Tauri / Browser / Chrome / Vite / 截图工具。

## 4. 验收

必须通过：

- `cargo test --lib project_director_task_plan`
- `cargo test --lib authorized_prepared_dispatch`
- `cargo test --lib worker_structured_report`
- `cargo test --lib process_fact`
- `cargo test --lib global_final_result_review`
- `cargo test --lib user_result_decision`
- `cargo test --lib stage_c_acceptance_summary`
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
- C4-C6 产品语义变更或新增能力
- memory candidate adoption 迁移完成
- formal memory adoption 迁移完成
- workflow execution runner / workflow machine / K3-B guard 迁移完成
- UI / 产品行为修改
- backlog 功能解冻
