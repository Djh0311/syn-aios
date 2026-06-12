# 复核结论：Root Treatment / R2-T14 Rust Workflow Governance Boundary And Director Review Rejection Test Extraction v1

日期：2026-06-12

Reviewer：Claude（claude-opus-4-8，复核线临时代班，依据 `handoffs/2026-06-12-review-line-temporary-takeover-claude-v1.md`）

复核对象：

- 任务包：`tasks/2026-06-12-root-treatment-r2-t14-rust-workflow-governance-boundary-and-director-review-rejection-test-extraction-v1.md`
- 实现 evidence：`evidence/2026-06-12-root-treatment-r2-t14-rust-workflow-governance-boundary-and-director-review-rejection-test-extraction-v1.md`
- 结果 handoff：`handoffs/2026-06-12-root-treatment-r2-t14-rust-workflow-governance-boundary-and-director-review-rejection-test-extraction-v1-result.md`

复核基线：当前 git HEAD `cdfd7f2`（R2-T13 hash 回填，T13 已收口入库）；T14 改动未提交，commit 序列待用户放行后由主管线执行。

性质：只读独立复核（实现包，双段非连续提取）。本结论文件是复核线唯一产出；复核线不改产品代码/任务包/evidence/权威文档，不跑 `git commit`，发现问题只列不修。

---

## STATUS: CLEAR

- P0：无
- P1：无
- P2：无

T14 把 T13 裁决的 9 个可迁测试抽到两个非连续 include 文件，经反向重构验证为字节级纯搬运；最高风险的"同组一迁一禁"边界正确落地——禁迁的 runner 端到端测试原位保留，可迁的 store-local 拒绝测试是新 director 文件唯一测试。脚本门全绿。未发现任何 P 级问题。

---

## 1. Recovery 与基线完整性

- 按代班纪律先重读磁盘：HEAD `cdfd7f2`；T13 五连 commit 全部落库（`82d3075` 任务包 → `f3ad6e3` evidence → `0f99690` 复核清除 → `0895f56` checkpoint → `cdfd7f2` hash 回填）。
- 复核线 T13 结论文件已入库：`git cat-file -e HEAD:evidence/...-r2-t13-...-review-claude-v1.md` → 存在。
- 工作区改动面 = T14：`lib.rs`、`workbench-shape-gate.js` 修改；新增 `lib_workflow_governance_boundary_tests.rs`、`lib_director_review_rejection_tests.rs`、T14 三份文档。无旁逸改动。

## 2. 独立重跑的脚本门（本地全量复跑，非转述 evidence）

- `cargo test --lib`：`471 passed; 0 failed; 16 ignored`——与 T12/T13 收口基线一致，迁出 9 个测试经 `include!` 仍全部运行、无丢失。
- 9 个聚焦过滤器：`workflow_ledger` / `subagent_report` / `review_result` / `workflow_exception` / `workflow_state_transition` / `workflow_node_state_transition` / `director_completion_gate` / `workflow_interfaces` 各 1 passed；`workflow_dispatch_director_review` **2 passed**（迁走的 rejects + 留下的 records 各 1，均通过）；全部 0 failed。
- `cargo fmt --manifest-path .../Cargo.toml -- --check`：FMT_CLEAN。
- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors / 0 warnings；`lib.rs: 5567 lines`。
- `git diff --check`：干净（exit 0）。
- 方法注：`git diff --check` 不扫未跟踪的两个新 include 文件；其清洁性由 §3 字节对账 + 尾随空白/EOF 独立扫描确认。

## 3. 双段对账（声明 vs 真实工作区）

- 行数收益：`git diff --numstat` 显示 `lib.rs` `2 added / 441 deleted`（净 −439）；2 added = 两行 `include!`，441 deleted = 339 + 102 两区段。`wc -l lib.rs` = 5567（6006 − 439）。
- 新文件规模：governance 339 行、director rejection 102 行，均远低于 `.rs` 3000 上限。
- waterline：shape gate diff 仅 `lib.rs` 一行 `6006 → 5567`；实测 5567/5567，锁历史新低、未放松。
- 区段字节对账：`HEAD:lib.rs` 4490-4828 == `lib_workflow_governance_boundary_tests.rs`（GOV_BYTE_EXACT）；4943-5044 == `lib_director_review_rejection_tests.rs`（DIR_BYTE_EXACT）。
- **反向重构对账（最强证据）**：把工作区 `lib.rs` 的两行 `include!` 分别展开为对应文件内容后，与 `HEAD:lib.rs` 逐字节比对 = `RECONSTRUCT_EXACT_MATCH`。即两个抽出文件恰好填满两个洞、且 `lib.rs` 其余部分零改动——纯搬运 + 产品代码零改动一次锁死。

## 4. 迁移纯度（最高风险点：同组一迁一禁）

- **禁迁 records 测试原位保留**：`workflow_dispatch_director_review_records_completed_dispatch` 仍在工作区 `lib.rs:4493`（夹在 governance include 4490 与 director include 4605 之间），未被迁出。该测试体内 `StubCodexResumeRunner` + `execute_workflow_node_dispatch_for_index_at`（runner 端到端，T13 §3.3 确认禁迁），正确留在 `lib.rs`。
- **可迁 rejects 测试为 director 文件唯一测试**：`lib_director_review_rejection_tests.rs` 仅含 `workflow_dispatch_director_review_rejects_invalid_state_and_dispatch`（#[test] = 1，fn 列表唯一）。该测试 store-local、零 runner（T13 §3.2 已读产品函数 `record_workflow_dispatch_director_review_at` 全函数无 runner 确认），可迁正确。
- **governance 文件 = 8 个裁定可迁测试**：`workflow_ledger_derives` / `subagent_report_derives` / `review_result_cannot_directly_complete_node` / `workflow_exception_detects` / `workflow_state_transition_enforces_confirmed_table` / `workflow_node_state_transition_enforces_actor_boundaries` / `director_completion_gate_requires_evidence_review_and_no_risk` / `workflow_interfaces_keep_conservative_boundaries`（#[test] = 8），与 T13 §3.1 一致，无多无少。
- **deferred 测试原位保留**：`compact_last_message_summary_preserves_workflow_machine_control_marker` 仍在 `lib.rs:4479`，未被迁出。
- **`#[test]` 守恒**：`HEAD:lib.rs` 44 = 工作区 `lib.rs` 35 + governance 8 + director 1。迁后 `lib.rs` 剩 35 = 禁迁 34 + deferred 1，与 T13 预测精确一致。
- **禁迁/helper 扫描**：对两个新文件扫 `StubCodexResumeRunner|CodexResumeRunner|execute_workflow_node_dispatch|Runner\b|k3_b_|memory_candidate_adoption|formal_memory_store|reads_real_static|records_completed_dispatch|fn fixture_|fn append_fixture_dispatch`——零命中（exit 1）。helper（`append_fixture_dispatch` 5628、`fixture_director_review_request` 5613）留 `lib.rs`；迁出文件经 `include!` 仍解析到 helper，由 `cargo test --lib` 全过与 `workflow_dispatch_director_review` 2 passed 证实无悬挂引用。
- **新文件清洁**：尾随空白 / `#[ignore]` 扫描零命中（exit 1）；字节对账已涵盖 EOF。

## 5. 边界与流程确认

- 工作区改动面仅：Rust 测试搬运（`lib.rs` + 2 新 include）+ shape gate waterline + T14 三份文档；无 `~/.codex` 写入、无 schema/command/sidecar/UI/CSS/TS 改动。shape gate `Tauri commands: 0 in lib.rs`、`Sidecar JSON: 0 unknown`。
- T12/T13 既定裁决未被重开；本包仅执行 T13 裁定的 9 个迁移，未触碰 34 禁迁与 1 deferred。
- 无真实 Codex 执行、无 prompt、无 secret 读取、无 Tauri/截图——与边界声明一致。

## 6. 复核边界声明

- 本文件为复核线唯一产出。未改任何产品代码/任务包/evidence/权威文档，未跑 `git commit`。
- §2 五门为复核线本地独立重跑结果；§3 字节对账与反向重构、§4 纯度核对均为复核线独立操作，非转述 evidence。
- 结论仅针对 R2-T14 任务包验收口径；不接受为 `lib.rs <= 3,000`、R2 完成、R3 Level B、生产 SQLite 迁移/read-cut/stop-write、多 agent 并行真实执行解锁、真实 Codex 执行、产品语义变更等任何外延。
- T14 收口即 T 系列可迁切片到底（`lib.rs` 剩 35 = 禁迁 34 + deferred 1）；代班清单与 R2 后段收口/转 R4 属主管线与用户/ Codex 回归后决策，不在本复核裁决范围。
- 本结论将作为 Codex 额度恢复后"换脑抽查"事后复检的输入之一。
