# Evidence / Root Treatment R3-A13 Transaction Acceptance And Cutover Gap Matrix v1

日期：2026-06-11

状态：Level A 已完成，Level B 未执行。

Planning baseline commit：`d1e2ce1c139f392437928a367a9744411eb7ecc4`

Implementation commit：待回填

## Scope

本轮实现 R3-A13 Level A：

- 在 fixture / temp DB 内验证 memory candidate adoption、formal memory record、formal memory version、formal memory audit event 和 workflow audit event 可以在同一个 SQLite transaction 内提交。
- 验证 before-commit failure injection 不产生 half-adopted state。
- 验证 after-commit-before-report 被分类为 `committed_but_report_failed`，保留 DB rows for audit，但不冒充完整 completed。
- 输出 cutover gap matrix，明确 R3-A9 / A10 / A11 / A12 Level B 仍未执行。

## Files

- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_transaction_acceptance.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a13/transaction-acceptance-core/workflow-state.v0.json`
- `prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a13/transaction-acceptance-core/memory-candidates.v1.json`
- `tasks/2026-06-11-root-treatment-r3-a13-transaction-acceptance-and-cutover-gap-matrix-v1.md`

## Transaction Acceptance

Covered in `cargo test --lib sqlite_transaction_acceptance`:

- success path commits candidate adoption, formal memory record, formal memory version, memory audit, and workflow audit in one transaction.
- `BeforeTransactionBegin` fails before DB creation.
- `AfterCandidateUpdateBeforeFormalMemoryInsert` rolls back candidate update and all downstream rows.
- `AfterFormalMemoryInsertBeforeVersionInsert` rolls back candidate update and formal memory row.
- `AfterVersionInsertBeforeMemoryAuditInsert` rolls back candidate update, formal memory row, and version row.
- `AfterMemoryAuditInsertBeforeWorkflowAuditInsert` rolls back candidate update, formal memory row, version row, and memory audit row.
- `BeforeCommit` rolls back every in-transaction row.
- `AfterCommitBeforeReport` records `committed_but_report_failed`, preserving committed rows for audit.
- non-temp DB path is rejected.
- non-R3-A13 fixture root is rejected.

## Cutover Gap Matrix

R3-A13 report records:

| Item | Level A | Level B | Acceptance |
| --- | --- | --- | --- |
| R3-A9 production DB apply | complete | pending | level_a_only |
| R3-A10 limited read-cut | complete | pending | level_a_only |
| R3-A11 production observation | complete | pending | level_a_only |
| R3-A12 stop-write decision | complete | pending | level_a_only |
| R3-A13 transaction acceptance | complete | not_requested | level_a_transaction_verified |
| production DB apply | not_applicable | pending | deferred |
| production read-cut | not_applicable | pending | deferred |
| production observation | not_applicable | pending | deferred |
| JSON / sidecar stop-write | not_applicable | pending | deferred |
| app startup / Tauri command / UI product path cutover | not_applicable | pending | deferred |
| multi-agent parallel real execution unlock | not_applicable | pending | blocked_until_real_cutover |

## Verification

- `node scripts/harness/workbench-shape-gate.js --mode check`：通过。
- `cargo test --lib sqlite_transaction_acceptance`：通过，5 passed。
- `cargo test --lib sqlite_stop_write`：通过，16 passed。
- `cargo test --lib sqlite_observation`：通过，24 passed。
- `cargo test --lib sqlite_read_cut`：通过，26 passed。
- `cargo test --lib sqlite_production`：通过，21 passed。
- `cargo test --lib sqlite_export`：通过，3 passed。
- `cargo test --lib sqlite_apply`：通过，6 passed。
- `cargo test --lib workflow_state`：通过，11 passed。
- `cargo test --lib`：通过，468 passed，16 ignored。
- `cargo fmt -- --check`：通过。

Common warning:

- Rust 测试保留既有 warning：`JsonRpcError::invalid_params` never used。

## Review Input

A12 checkpoint 复核线 `019eb51c-61fe-7fc3-8973-b22a4ce58911` 回交：

- A12 checkpoint 无 P0/P1，可以继续 A13。
- P2：历史 A11 task/evidence 仍有“准备 R3-A12”历史下一步口径；不是当前 authority。
- P2：少量旧 evidence/handoff 的“多 agent 并行真实执行已解锁”字面串位于“不接受为 / 不得声明”段落，属于 grep 噪声。

## Boundary Confirmation

本轮没有：

- 读取真实 workbench state root。
- 创建真实 workbench-owned production DB。
- 写真实 production DB。
- 修改真实 JSON / sidecar。
- 停写 JSON / sidecar。
- 切 app startup / Tauri command / UI / 产品全局读写路径。
- 新增 Tauri command。
- 新增 sidecar JSON 种类。
- 执行真实 `codex exec` / `codex exec resume`。
- 发送 prompt。
- 读写 `/Users/yoyi/.codex`。
- 读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 启动 Stage L / K3-B1 retry / K3-B2。
- 解冻 backlog 功能。

## Do Not Claim

不能声明：

- R3 全量完成。
- 生产 SQLite 迁移完成。
- 真实 workbench-owned production DB 已创建。
- 真实 production apply 已执行。
- 真实 production read-cut 已执行。
- 真实 production observation 已执行。
- JSON / sidecar 已停写。
- app startup / Tauri command / UI 产品路径已切 DB。
- rollback production workflow 已执行。
- 多 agent 并行真实执行已解锁。
- Stage L / K3-B1 / K3-B2 已恢复。
