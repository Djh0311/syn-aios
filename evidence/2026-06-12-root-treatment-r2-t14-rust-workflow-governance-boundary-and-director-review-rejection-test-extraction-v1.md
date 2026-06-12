# Evidence: Root Treatment / R2-T14 Rust Workflow Governance Boundary And Director Review Rejection Test Extraction v1

日期：2026-06-12

Supervisor：Claude（claude-fable-5，临时代班，依据 `handoffs/2026-06-12-supervisor-line-temporary-takeover-codex-to-claude-v1.md`）

状态：已完成并通过复核线 `STATUS: CLEAR`；用户已放行，commit 序列执行中，checkpoint 同步随后。

任务包：`tasks/2026-06-12-root-treatment-r2-t14-rust-workflow-governance-boundary-and-director-review-rejection-test-extraction-v1.md`

Planning baseline commit：`cdfd7f225bc182287ee58cfe765067a1aedb9916`

Task package commit：`bd4522a80b092b54cf6fef89fcfb1eac93f0a534`

Implementation commit：`a61c8f973a2b4a274fdf0aaf8f1b0c027a385b0c`

Review result：`CLEAR`；复核结论文件 `evidence/2026-06-12-root-treatment-r2-t14-rust-workflow-governance-boundary-and-director-review-rejection-test-extraction-v1-review-claude-v1.md`（Reviewer：claude-opus-4-8，复核线临时代班）；P0/P1/P2 无；详见第 9 节

## 1. 本轮目标

执行 R2-T13 裁决的可迁 9 个 inline tests 迁移（用户已同意立项）：workflow governance boundary 带 8 个（`lib.rs` 4490-4828）与 director review 拒绝路径 1 个（4943-5044），两区段非连续，各迁入独立 include 文件。本包是 T 系列最后一个迁移包。

## 2. 改动范围

修改：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `scripts/harness/workbench-shape-gate.js`

新增：

- `prototypes/productized-desktop-shell/src-tauri/src/lib_workflow_governance_boundary_tests.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib_director_review_rejection_tests.rs`
- `tasks/2026-06-12-root-treatment-r2-t14-rust-workflow-governance-boundary-and-director-review-rejection-test-extraction-v1.md`
- `evidence/2026-06-12-root-treatment-r2-t14-rust-workflow-governance-boundary-and-director-review-rejection-test-extraction-v1.md`
- `handoffs/2026-06-12-root-treatment-r2-t14-rust-workflow-governance-boundary-and-director-review-rejection-test-extraction-v1-result.md`

本轮未修改：

- 产品函数签名、可见性或行为。
- workflow governance / director review 产品语义或断言口径。
- helper / fixture builder / stub runner / runner fake（`fixture_director_review_request`、`append_fixture_dispatch` 等仍留 `lib.rs`）。
- 紧邻的禁迁/deferred 测试：workflow machine marker（deferred 维持）、`workflow_dispatch_director_review_records_completed_dispatch`（禁迁，StubCodexResumeRunner）、offline role 组及其余全部禁迁测试，原位保留。
- Tauri command、DB schema、sidecar schema、workflow state JSON schema。
- 前端 UI / CSS / TS。

## 3. 实现说明

- 自底向上做两段字节级提取：先 4943-5044（102 行）→ `lib_director_review_rejection_tests.rs`，再 4490-4828（339 行）→ `lib_workflow_governance_boundary_tests.rs`，保证行号坐标有效。
- 两个原位置各保留一行 `include!(...)`，测试仍运行在 crate-root `tests` module 内，不扩大生产函数可见性。
- shape gate `lib.rs` waterline 从 `6006` 更新为 `5567`。

## 4. 形状收益

- `lib.rs`：`6006 -> 5567`，下降 `439` 行。
- 新增 `lib_workflow_governance_boundary_tests.rs`：`339` 行（8 个 tests）；`lib_director_review_rejection_tests.rs`：`102` 行（1 个 test）；均低于 `.rs` 新文件上限 `3000`。
- shape gate 输出：`lib.rs: 5567/5567 (same)`，0 errors，0 warnings。
- 字节级对比：从 planning baseline（HEAD `cdfd7f2`）的旧 `lib.rs` 分别提取 4490-4828 与 4943-5044，与两新文件 `cmp` 完全一致；两文件 EOF 单一换行、无尾部空白。
- `#[test]` 守恒：44 = 35（`lib.rs` 剩余）+ 8 + 1，与 R2-T13 预测一致。

## 5. 验证

已通过：

- `cargo test --lib workflow_ledger`：1 passed。
- `cargo test --lib subagent_report`：1 passed。
- `cargo test --lib review_result`：1 passed。
- `cargo test --lib workflow_exception`：1 passed。
- `cargo test --lib workflow_state_transition`：1 passed。
- `cargo test --lib workflow_node_state_transition`：1 passed。
- `cargo test --lib director_completion_gate`：1 passed。
- `cargo test --lib workflow_interfaces`：1 passed。
- `cargo test --lib workflow_dispatch_director_review`：2 passed（迁出的拒绝路径 + 留在 `lib.rs` 的禁迁回收路径，均通过）。
- `cargo test --lib`：471 passed，0 failed，16 ignored（与 R2-T13 基线一致）。
- `cargo fmt -- --check`：通过。
- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors，0 warnings。
- `git diff --check`：通过。

保留既有 warning：

- `src/mcp/protocol.rs` 中 `JsonRpcError::invalid_params` dead_code warning。本轮未触碰该文件。

## 6. 范围扫描

- 禁迁关键词扫描（`workflow_node_dispatch_execute_|workflow_machine|memory_candidate_adoption_|formal_memory_adoption_|k3_b_|stub|runner|real_state|prepare_offline_role_dispatch|record_offline_role|CodexResumeRunner`）：两新文件零命中。
- 两新文件未迁移任何 helper / fixture builder；helper 仍留 `lib.rs`。
- 禁迁/deferred 测试确认未触碰：workflow machine marker（4479-4488）、director review 回收路径（原 4830-4941）、offline role 组、其余 33 个禁迁测试全部原位保留在 `lib.rs`。

## 7. 边界确认

本轮没有：

- 执行真实 `codex exec` / `codex exec resume`。
- 发送 prompt。
- 读写 `/Users/yoyi/.codex`。
- 读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript。
- 启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 修改 UI / CSS / TS。
- 修改 Tauri command、DB/schema、sidecar schema、workflow state JSON schema。

## 8. 不接受为

本轮不接受为：

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
- R2 后段收口方式或转 R4 已决定

## 9. 复核结论

复核线只读复核已通过，用户已放行：

- 复核结论文件：`evidence/2026-06-12-root-treatment-r2-t14-rust-workflow-governance-boundary-and-director-review-rejection-test-extraction-v1-review-claude-v1.md`（Reviewer：claude-opus-4-8，复核线临时代班，与主管线 claude-fable-5 构成跨模型复核）。
- 最终结论：`STATUS: CLEAR`；P0/P1/P2：无。
- 复核线独立重跑脚本门（471/0/16、9 个聚焦过滤器 8×1 + 2、fmt、shape gate 0 errors、`git diff --check` 干净）。
- 复核线双段字节对账（GOV_BYTE_EXACT / DIR_BYTE_EXACT）外加反向重构对账：把工作区 `lib.rs` 两行 include 展开后与 HEAD `lib.rs` 逐字节比对 `RECONSTRUCT_EXACT_MATCH`——纯搬运与产品代码零改动一次锁死。
- 复核确认同组一迁一禁正确落地：禁迁 records 测试（StubCodexResumeRunner）原位保留于 `lib.rs`，可迁 rejects 测试为 director 文件唯一测试；deferred marker 测试原位保留；`#[test]` 守恒 44 = 35 + 8 + 1；禁迁/helper 扫描零命中。
