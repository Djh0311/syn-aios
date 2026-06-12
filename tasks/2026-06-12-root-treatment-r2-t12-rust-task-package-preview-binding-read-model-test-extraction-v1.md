# Root Treatment / R2-T12 Rust Task Package Preview Binding Read Model Test Extraction v1

日期：2026-06-12

Supervisor：Claude（claude-fable-5，临时代班，依据 `handoffs/2026-06-12-supervisor-line-temporary-takeover-codex-to-claude-v1.md`）

状态：已完成并复核通过，checkpoint 已同步。

Planning baseline commit：`435c21471ad056bd7ed1b44681ad52f285883b5c`

Takeover docs commit：`6fba75ef3facbaf03f81f19c3f235df5981b2875`

Task package commit：`7d5333936b46c2532c4811c70c99233e31125b9d`

Implementation commit：`a3fce1f7385616bae3d0b19a1ec0907b5943ea47`

复核清除 commit：`bcb8864b0f4684f44bce6819e81b7ac5c5cbd9fe`

Authority sync commit：`cf47fb8524f6ed795b3eee8a712573449cd752e1`

Evidence commit：`86827770f13f6c91155f415fd9b2ff7c28e83158`

Review result：`CLEAR`；复核结论文件 `evidence/2026-06-12-root-treatment-r2-t12-rust-task-package-preview-binding-read-model-test-extraction-v1-review-claude-v1.md`（Reviewer：claude-opus-4-8，复核线临时代班）；P0/P1/P2 无

本文是 Root Treatment / Stage R 的 R2-T12 任务包，承接 R2-T11 和 2026-06-12 执行策略（`handoffs/2026-06-12-root-treatment-execution-strategy-review-claude-to-codex-v1.md` P1 规则），只迁移能实际降低 `lib.rs` 棘轮指标的低风险 Rust inline tests。本包同时完成 checkpoint 要求的 R2-T12 候选切片评估（第 6 节）。

## 1. 目标

把 `prototypes/productized-desktop-shell/src-tauri/src/lib.rs` 中 work item state update / workflow node session binding / task package preview / task package read model / project blackboard read model 相关 inline tests 迁出到 crate-root test include 文件，继续降低 `lib.rs` 历史最低水位线。该切片是 R2-T11 task package dispatch preparation 域的同域延续：T11 + T12 后，task package 准备链路（draft → fields → preview → file generation → readiness → binding → read model 派生）的低风险 inline tests 全部迁出。

目标文件：

- 新增：`prototypes/productized-desktop-shell/src-tauri/src/lib_task_package_preview_binding_read_model_tests.rs`
- 修改：`prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- 修改：`scripts/harness/workbench-shape-gate.js`

预计收益：

- `lib.rs` 预计下降约 538 行（6,544 → 约 6,006）。
- 新增 `.rs` test include 预计约 539 行，远低于 3,000 行新文件上限。
- 该切片降低 `lib.rs` 棘轮指标，符合"不得立项不降低棘轮指标的拆分包"规则。

## 2. 允许范围

允许迁移以下 11 个 tests（`lib.rs` 3411-3949 行连续区段），保持测试体和断言语义不变：

- `work_item_state_update_rejects_non_index_project`
- `workflow_node_session_binding_binds_rebinds_and_unbinds`
- `workflow_node_session_binding_rejects_non_index_session_and_missing_node`
- `task_package_preview_rejects_non_index_project`
- `task_package_preview_rejects_missing_state_file`
- `task_package_preview_rejects_missing_workflow`
- `task_package_preview_rejects_missing_work_item`
- `task_package_preview_renders_markdown_from_draft`
- `task_package_preview_uses_placeholders_for_missing_fields`
- `workflow_task_package_read_model_derives_v1_objects_from_v0_state`
- `project_blackboard_read_model_derives_candidates_without_state_promotion`

允许：

- 在 `lib.rs` 原位置插入 `include!("lib_task_package_preview_binding_read_model_tests.rs");`
- 让共享 helper 继续留在 `lib.rs`，包括 `fixture_project`、`fixture_session`、`fixture_task_draft_request`、`fixture_work_item_state_update_request`、`fixture_node_session_bind_request`、`fixture_node_session_unbind_request`、`fixture_task_preview_request`、`fixture_task_file_generation_request`、`ready_fields_update_request`、`mark_task_package_fixture_ready`、`append_fixture_dispatch` 等。
- 更新 shape gate 中 `lib.rs` waterline 为本轮完成后的历史最低收口值。
- 新增 evidence / handoff。

## 3. 禁止范围

禁止：

- 修改产品函数签名、可见性、语义或断言口径。
- 迁移 helper / fixture builder / stub runner / runner fake。
- 迁移 K3-B runtime prompt guard、workflow node dispatch prepare/execute/readback、workflow execution runner、workflow machine、offline role dispatch、ignored real-state tests、cross-store memory adoption、memory candidate adoption、formal memory adoption、legacy dispatch execution 或真实 execution tests。
- 修改 task package / task memory injection / dispatch readiness 产品语义。
- 修改 workflow state JSON schema、Tauri command、DB schema 或 sidecar schema。
- 修改前端 UI / CSS / TS。
- 执行真实 `codex exec` / `codex exec resume`、发送 prompt、读写 `/Users/yoyi/.codex`。
- 启动 Tauri / Browser / Chrome / Vite / 截图工具。

## 4. 验收

必须通过：

- `cargo test --lib work_item_state_update`
- `cargo test --lib workflow_node_session_binding`
- `cargo test --lib task_package_preview`
- `cargo test --lib workflow_task_package_read_model`
- `cargo test --lib project_blackboard_read_model`
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
- task package / task memory injection / dispatch readiness 产品语义变更或新增能力
- workflow node dispatch execute/readback 迁移完成
- workflow execution runner / workflow machine / K3-B guard 迁移完成
- memory candidate adoption 迁移完成
- formal memory adoption 迁移完成
- UI / 产品行为修改
- backlog 功能解冻

## 6. R2-T12 候选切片评估记录

本包开工前对 `lib.rs` 剩余 55 个 inline tests 做了全量分类（行号以 planning baseline 为准）：

选中迁移（11 个，见第 2 节）：3411-3949 连续区段，全部为 temp-dir fixture 驱动的 store-local / read-model 派生测试，不依赖 stub runner，不涉及 adoption、dispatch execute/readback、K3-B guard。

禁迁（既定清单命中，本包不触碰，共 33 个）：

- K3-B runtime prompt guard：`k3_b_tauri_command_guard_*` 2 个（1757、1770）。
- real-state：`reads_real_static_index_summary` 1 个（1782，读取真实静态 index）；`#[ignore]` real task package file generation confirmation 1 个（约 5794，写真实 product-line 文件）。
- cross-store memory adoption / formal memory adoption 相邻组：`memory_candidate_adoption_*` 7 个 + `memory_candidate_rejection_does_not_create_formal_memory` 1 个 + `formal_memory_store_*` 5 个（2943-3409；其中 formal_memory_store 5 个为 R2-T9 刻意留下的 adoption 相邻簇，含 cross-store 隔离断言，不赌）。
- workflow node dispatch prepare/started/execute/readback/failure/timeout/permission 组 + legacy real execution guard：12 个（3954-4884，依赖 stub runner 与真实执行边界语义）。
- workflow machine：`workflow_machine_runs_four_role_loop_to_acceptance` 1 个（4885）。
- offline role 端到端组：`offline_role_orchestration_records_dispatch_handoff_and_review`、`offline_role_dispatch_rejects_missing_ready_work_item`、`offline_role_dispatch_rejects_duplicate_prepared_dispatch` 3 个（5585-5744+）。

deferred（拿不准，留待 T13 评估，共 11 个，理由如下）：

- `compact_last_message_summary_preserves_workflow_machine_control_marker`（5017）：断言对象是 workflow machine 控制标记，与 workflow machine 冻结语义相邻。
- workflow governance boundary 带（5029-5360，8 个）：`workflow_ledger_derives_summary_entries_without_tool_output_fulltext`、`subagent_report_derives_required_fields_and_direction_risk`、`review_result_cannot_directly_complete_node`、`workflow_exception_detects_timeout_permission_review_direction_and_harness`、`workflow_state_transition_enforces_confirmed_table`、`workflow_node_state_transition_enforces_actor_boundaries`、`director_completion_gate_requires_evidence_review_and_no_risk`、`workflow_interfaces_keep_conservative_boundaries`。多数疑似纯内存边界表测试，但其中 director gate / review result 与 R2-T0 暂缓的 "workflow machine / director review / offline role 端到端组" 边界需逐测试核对是否依赖 runner fixture；代班期间按 takeover handoff §3 "拿不准跳过不赌" 原则整带 deferred。
- director review 组（5369、5482，2 个）：`workflow_dispatch_director_review_*` 使用 dispatch fixture，与 T0 暂缓的 director review 组直接同名，deferred。

T13 复评触发点：T12 收口且用户确认节奏后，对 deferred 带逐测试核对 runner fixture 依赖，再决定是否立项。

## 7. 执行记录

本轮已完成实现、本地验证、主管线自查与复核线独立只读复核（`STATUS: CLEAR`，P0/P1/P2 无）；用户已放行 commit 序列。

过程事件（如实留痕）：实现期间用户更新了主管线接管档案 §5 并新增复核线职位档案（`handoffs/2026-06-12-review-line-temporary-takeover-claude-v1.md`）：复核改由独立复核线会话（claude-opus-4-8，跨模型）承担，触发权在用户。主管线在档案更新前曾按旧版 §5 口径自派一个同模型只读审查 agent——该审查降级记录为主管线自查，不充当正式复核；其抓出的任务包 §6 计数笔误（禁迁 32 应为 33）已修正。主管线曾因持有旧版档案上下文而短暂误判"复核线职位档案不存在"，经重读文件系统纠正；预防措施：收口各步骤前重读权威档案最新状态，不依赖开机时的上下文快照。

实际改动：

- 新增 `prototypes/productized-desktop-shell/src-tauri/src/lib_task_package_preview_binding_read_model_tests.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs` 原测试块替换为 `include!("lib_task_package_preview_binding_read_model_tests.rs");`
- `scripts/harness/workbench-shape-gate.js` 的 `lib.rs` waterline 更新为 `6006`

实际形状收益：

- `lib.rs`：`6544 -> 6006`，下降 `538` 行。
- 新增 include 文件：`539` 行，低于 `.rs` 新文件上限 `3000`。
- 旧测试块与新 include 字节级一致。

验证结果与复核结论：见 evidence 第 5、9 节。
