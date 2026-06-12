# Root Treatment / R2-T14 Rust Workflow Governance Boundary And Director Review Rejection Test Extraction v1

日期：2026-06-12

Supervisor：Claude（claude-fable-5，临时代班，依据 `handoffs/2026-06-12-supervisor-line-temporary-takeover-codex-to-claude-v1.md`）

状态：已完成并通过复核线 `STATUS: CLEAR`；用户已放行 commit 序列，序列执行中，hash 随序列回填。

Planning baseline commit：`cdfd7f225bc182287ee58cfe765067a1aedb9916`

Task package commit：随 commit 序列回填

Implementation commit：随 commit 序列回填

Review result：`CLEAR`；复核结论文件 `evidence/2026-06-12-root-treatment-r2-t14-rust-workflow-governance-boundary-and-director-review-rejection-test-extraction-v1-review-claude-v1.md`（Reviewer：claude-opus-4-8，复核线临时代班）；P0/P1/P2 无。复核线以反向重构对账（两行 include 展开后与 HEAD `lib.rs` 逐字节比对 RECONSTRUCT_EXACT_MATCH）锁死纯搬运与产品代码零改动；确认禁迁 records 测试与 deferred marker 测试原位保留、`#[test]` 守恒 44 = 35 + 8 + 1

本文是 Root Treatment / Stage R 的 R2-T14 任务包，执行 R2-T13 裁决的可迁 9 个 inline tests 迁移（用户 2026-06-12 已同意立项）。本包是 T 系列最后一个迁移包：收口后 T 系列可迁切片到底（`lib.rs` 剩余 35 个 inline tests = 禁迁 34 + deferred 1），主管线按接管档案 §6 写代班清单收尾停下。

## 1. 目标

把 `lib.rs` 中 R2-T13 裁决为可迁的 workflow governance boundary 带 8 个 tests 与 director review 拒绝路径 1 个 test 迁出到 crate-root test include 文件，继续降低 `lib.rs` 历史最低水位线。

目标文件：

- 新增：`prototypes/productized-desktop-shell/src-tauri/src/lib_workflow_governance_boundary_tests.rs`
- 新增：`prototypes/productized-desktop-shell/src-tauri/src/lib_director_review_rejection_tests.rs`
- 修改：`prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- 修改：`scripts/harness/workbench-shape-gate.js`

预计收益：

- `lib.rs` 预计下降约 439 行（6,006 → 约 5,567）。
- 新增两个 `.rs` test include：约 339 行与约 102 行，均远低于 3,000 行新文件上限。
- 该切片降低 `lib.rs` 棘轮指标，符合"不得立项不降低棘轮指标的拆分包"规则。

## 2. 允许范围

允许迁移以下 9 个 tests（行号按 planning baseline），保持测试体和断言语义不变：

governance 带（`lib.rs` 4490-4828 连续区段 → `lib_workflow_governance_boundary_tests.rs`）：

- `workflow_ledger_derives_summary_entries_without_tool_output_fulltext`
- `subagent_report_derives_required_fields_and_direction_risk`
- `review_result_cannot_directly_complete_node`
- `workflow_exception_detects_timeout_permission_review_direction_and_harness`
- `workflow_state_transition_enforces_confirmed_table`
- `workflow_node_state_transition_enforces_actor_boundaries`
- `director_completion_gate_requires_evidence_review_and_no_risk`
- `workflow_interfaces_keep_conservative_boundaries`

director review 拒绝路径（`lib.rs` 4943-5044 区段 → `lib_director_review_rejection_tests.rs`）：

- `workflow_dispatch_director_review_rejects_invalid_state_and_dispatch`

允许：

- 在两个原位置各插入一行 `include!(...)`（区段非连续，各文件字节级对应一个区段）。
- 让共享 helper 继续留在 `lib.rs`，包括 `fixture_project`、`fixture_task_draft_request`、`fixture_work_item_state_update_request`、`fixture_director_review_request`、`append_fixture_dispatch` 等。
- 更新 shape gate 中 `lib.rs` waterline 为本轮完成后的历史最低收口值。
- 新增 evidence / handoff。

## 3. 禁止范围

禁止：

- 修改产品函数签名、可见性、语义或断言口径。
- 迁移 helper / fixture builder / stub runner / runner fake。
- 触碰紧邻的禁迁/deferred 测试：`compact_last_message_summary_preserves_workflow_machine_control_marker`（4479-4488，deferred 维持）、`workflow_machine_runs_four_role_loop_to_acceptance`、`workflow_dispatch_director_review_records_completed_dispatch`（4830-4941，禁迁，StubCodexResumeRunner）、offline role 组及其余全部禁迁测试。
- 迁移 K3-B runtime prompt guard、workflow node dispatch prepare/execute/readback、workflow execution runner、workflow machine、offline role dispatch、ignored real-state tests、cross-store memory adoption、memory candidate adoption、formal memory adoption、legacy dispatch execution 或真实 execution tests。
- 修改 workflow state JSON schema、Tauri command、DB schema 或 sidecar schema。
- 修改前端 UI / CSS / TS。
- 执行真实 `codex exec` / `codex exec resume`、发送 prompt、读写 `/Users/yoyi/.codex`。
- 启动 Tauri / Browser / Chrome / Vite / 截图工具。

## 4. 验收

必须通过：

- `cargo test --lib workflow_ledger`
- `cargo test --lib subagent_report`
- `cargo test --lib review_result`
- `cargo test --lib workflow_exception`
- `cargo test --lib workflow_state_transition`
- `cargo test --lib workflow_node_state_transition`
- `cargo test --lib director_completion_gate`
- `cargo test --lib workflow_interfaces`
- `cargo test --lib workflow_dispatch_director_review`（含留在 `lib.rs` 的禁迁兄弟测试，一并通过）
- `cargo test --lib`
- `cargo fmt -- --check`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`
- 两区段分别字节级对比一致。
- 复核线只读审查，结论不得有 P0/P1。

## 5. 不接受为

本任务不接受为：

- `lib.rs <= 3,000`
- R2 全部完成
- R3 Level B 执行
- 生产 SQLite 迁移 / read-cut / stop-write
- 多 agent 并行真实执行解锁
- 真实 Codex 执行
- workflow governance / director review 产品语义变更或新增能力
- workflow node dispatch execute/readback 迁移完成
- workflow execution runner / workflow machine / K3-B guard 迁移完成
- memory candidate adoption / formal memory adoption 迁移完成
- UI / 产品行为修改
- backlog 功能解冻
- R2 后段收口方式或转 R4 已决定（属用户与 Codex 回归后决策）

## 6. 执行记录

本轮已完成实现与离线验证；正式复核与 commit 序列待用户触发与放行。

实际改动：

- 新增 `lib_workflow_governance_boundary_tests.rs`（339 行，8 个 tests）与 `lib_director_review_rejection_tests.rs`（102 行，1 个 test）。
- `lib.rs` 两个原位置各替换为一行 `include!(...)`。
- `scripts/harness/workbench-shape-gate.js` 的 `lib.rs` waterline 更新为 `5567`。

实际形状收益：

- `lib.rs`：`6006 -> 5567`，下降 `439` 行。
- 两区段与 HEAD 原文分别 `cmp` 字节级一致；两新文件 EOF 单一换行无尾部空白。
- `#[test]` 守恒：迁移前 44 = 迁移后 `lib.rs` 35 + governance 8 + rejection 1，与 T13 预测一致。

验证已通过：

- 八个 governance 过滤器各 1 passed；`workflow_dispatch_director_review` 2 passed（含留在 `lib.rs` 的禁迁兄弟测试）。
- `cargo test --lib`：471 passed，0 failed，16 ignored（基线不变）。
- `cargo fmt -- --check`、shape gate（pass，0 errors，0 warnings，`lib.rs: 5567/5567 (same)`）、`git diff --check`：全部通过。
- 禁迁关键词扫描两新文件零命中。
