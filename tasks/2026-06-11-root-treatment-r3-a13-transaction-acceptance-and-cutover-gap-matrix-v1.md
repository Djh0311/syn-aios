# Root Treatment / R3-A13 Transaction Acceptance And Cutover Gap Matrix v1

日期：2026-06-11

状态：Level A 已完成，Level B 未执行。本文是 Root Treatment / Stage R 的 R3-A13 任务包，用于在 R3-A12 Level A stop-write JSON decision / rollback drill 完成后，补齐 R3 合同中的跨记忆 + 审计 SQLite 事务验收，并形成 R3 final acceptance / cutover gap matrix。A13 默认只做 Level A fixture / temp 事务验收和缺口矩阵，不读取真实 workbench state root，不创建真实 workbench-owned production DB，不切 app startup / Tauri command / UI / 产品全局读写路径，不停写 JSON / sidecar，不执行真实 Codex，不读写 `/Users/yoyi/.codex`。

规划基线 commit：`d1e2ce1c139f392437928a367a9744411eb7ecc4`

## 0. 全局主管理解

已知事实：

- R3-A9 Level A 已完成 fixture / temp production DB initializer + apply with backup manifest / no read-cut，implementation commit 为 `52d6b4b73dcb49e4ffc582dac500d9ad6a8ee4df`；Level B 未执行。
- R3-A10 Level A 已完成 `workflow_state_summary` 单一低风险 read model 的 fixture / temp limited read-cut contract，implementation commit 为 `b18424c38bf0f36f8c9b8ee783a0010598ca9683`；Level B 未执行。
- R3-A11 Level A 已完成 production observation / export verification fixture / temp contract，implementation commit 为 `a7d715c49888b9d3ec67c36c3e431f07e14af12a`；Level B 未执行。
- R3-A12 Level A 已完成 stop-write JSON / sidecar supervisor decision contract 和 fixture / temp rollback drill，implementation commit 为 `eacfad7c4a916f1307e633a37a6084a9fc2927e6`；Level B 未执行。
- R3 production cutover contract 明确要求至少一个未来 R3 任务证明 candidate adoption across memory + audit in one SQLite transaction。
- 真实 workbench state root 未读取，真实 workbench-owned production DB 未创建，真实 production apply / read-cut / observation / stop-write 均未执行。
- Stage L / L1-L6、K3-B1 retry、K3-B2 和 backlog 功能仍冻结为 `deferred_during_root_treatment`。

核心判断：

```text
R3-A13 的目标不是宣布 R3 全量完成，而是把 Level A 已完成内容、Level B 未执行内容和跨域事务验收结果形成一个可复核的收口矩阵。若 Level B 仍未执行，A13 只能接受为 R3 Level A acceptance / cutover gap matrix 完成，不能接受为生产 SQLite 迁移完成。
```

## 1. Execution Mode

Execution Mode：Supervisor-led task package with reusable Stage R implementation line。

Multi-Agent Policy：

- 任务包由全局主管冻结和提交。
- A13 实现可由主管线直接完成，避免新增过多线程和上下文维护成本。
- 复核线只读复核，不改文件、不提交。
- 主管线负责 fresh verify、入口同步和 commit。

Level split：

- Level A：fixture / temp transaction acceptance + cutover gap matrix。必须完成；只允许 repo fixture 或 temp DB / temp report root，不读取真实 workbench state root，不创建真实 production DB，不切真实产品读写路径，不停写 JSON / sidecar。
- Level B：optional real workbench-owned production transaction acceptance / final cutover。只有 A9/A10/A11/A12 Level B 或等价真实 production evidence 完成、A13 Level A 通过、主管自审 execution record 完整、备份/回滚点存在后才允许另行执行。

Fallback If Scope Expands：

- 如果实现需要真实 workbench state root、真实 production DB、app startup hook、Tauri command、UI 接入、真实 product global read/write path、真实 stop-write、真实 Codex 执行、`.codex`、secret / full transcript、provider credential，立即停止并拆新任务包。

## 2. 权威依据

必须读取并服从：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `codex-multi-agent-safe-collaboration.md`
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`
- `docs/plans/2026-06-11-root-treatment-r3-sqlite-schema-importer-rollback-contract-v1.md`
- `docs/plans/2026-06-11-root-treatment-r3-production-cutover-and-rollback-operator-contract-v1.md`
- `tasks/2026-06-11-root-treatment-r3-a9-production-db-initializer-apply-with-backup-manifest-no-read-cut-v1.md`
- `tasks/2026-06-11-root-treatment-r3-a10-limited-read-cut-planning-and-feature-flag-fallback-v1.md`
- `tasks/2026-06-11-root-treatment-r3-a11-production-observation-export-verification-contract-v1.md`
- `tasks/2026-06-11-root-treatment-r3-a12-stop-write-json-decision-and-rollback-drill-v1.md`
- A9 / A10 / A11 / A12 evidence 和 handoff。

建议读取的代码：

- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_apply.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_production_apply.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_read_cut.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_observation_period.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_stop_write.rs`
- `scripts/harness/workbench-shape-gate.js`

## 3. 目标

R3-A13 Level A 必须完成：

- 新增 transaction acceptance helper，建议落在新文件 `workbench_sqlite_transaction_acceptance.rs`，避免继续膨胀 A9-A12 模块。
- `lib.rs` 只允许新增 module declaration，不新增 Tauri command、不接 startup、不接 UI。
- Helper 必须在 temp DB / fixture DB 内证明单个 SQLite transaction 可以同时完成：
  - candidate consumed / adopted。
  - formal memory record created。
  - formal memory version created。
  - formal memory audit event created。
  - workflow or product command audit ref linked。
  - source / report 记录可追溯。
- Helper 必须提供 failure injection，至少覆盖：
  - before transaction begin。
  - after candidate update before formal memory insert。
  - after formal memory insert before version insert。
  - after version insert before memory audit insert。
  - after memory audit insert before workflow audit insert。
  - before commit。
  - after commit before report。
- before commit 的所有 failure injection 必须证明不会留下 half-adopted state。
- after commit before report 必须分类为 committed_but_report_failed，不冒充完整完成，并保留 DB rows for audit。
- Helper 必须输出 report，至少包含：
  - schema_version：`workbench_sqlite_transaction_acceptance.v1`
  - mode：`level_a_fixture_transaction_acceptance`
  - status：`completed` / `blocked` / `failed_classified`
  - db_path_ref / db_path_hash
  - candidate_key / memory_id / memory_version_id / memory_audit_event_id / workflow_audit_event_id
  - before_counts / after_counts
  - rows_changed
  - failure_point
  - transaction_flags
  - rollback_assurance
  - cutover_gap_matrix
  - do_not_claim
- transaction_flags 必须明确：
  - `sqlite_transaction_used=true`
  - `candidate_adopted=true` only on completed / committed report
  - `formal_memory_created=true` only on completed / committed report
  - `memory_audit_written=true` only on completed / committed report
  - `workflow_audit_ref_written=true` only on completed / committed report
  - `source_json_written=false`
  - `sidecar_written=false`
  - `production_db_written=false`
  - `product_global_read_path_changed=false`
  - `product_global_write_path_changed=false`
  - `codex_home_touched=false`
- cutover_gap_matrix 必须逐项列出：
  - R3-A9 Level A complete / Level B pending。
  - R3-A10 Level A complete / Level B pending。
  - R3-A11 Level A complete / Level B pending。
  - R3-A12 Level A complete / Level B pending。
  - R3-A13 Level A transaction acceptance complete。
  - production DB apply pending。
  - production read-cut pending。
  - production observation pending。
  - JSON / sidecar stop-write pending。
  - app startup / Tauri command / UI product path cutover pending。
  - multi-agent parallel real execution unlock pending。
- A13 任务包必须新增 evidence / handoff，并在完成后同步权威入口。

## 4. 文件落点

允许修改：

- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_transaction_acceptance.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a13/**`
- `tasks/2026-06-11-root-treatment-r3-a13-transaction-acceptance-and-cutover-gap-matrix-v1.md`
- `evidence/2026-06-11-root-treatment-r3-a13-transaction-acceptance-and-cutover-gap-matrix-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-a13-transaction-acceptance-and-cutover-gap-matrix-v1-result.md`
- 当前权威入口：`CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`
- 计划同步：`docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`、`docs/plans/2026-06-11-root-treatment-r3-production-cutover-and-rollback-operator-contract-v1.md`

禁止修改：

- 不改 Tauri command 列表。
- 不改 app startup。
- 不改 UI。
- 不改 workflow state 顶层 schema。
- 不改真实 product command runner。
- 不改真实 `.codex` 或任何 secret / auth / credential。

## 5. 验收

必须通过：

- `node scripts/harness/workbench-shape-gate.js --mode check`
- `cargo test --lib sqlite_transaction_acceptance`
- `cargo test --lib sqlite_stop_write`
- `cargo test --lib sqlite_observation`
- `cargo test --lib sqlite_read_cut`
- `cargo test --lib sqlite_production`
- `cargo test --lib sqlite_export`
- `cargo test --lib sqlite_apply`
- `cargo test --lib workflow_state`
- `cargo test --lib`
- `cargo fmt -- --check`
- `git diff --check`

扫描：

- 旧口径扫描：不得把 A13 写成 production DB / production read-cut / stop-write / R3 full completion。
- 敏感路径扫描：不得新增 `.codex` / secret / token / credential / full transcript 真实读取路径。
- 真实执行扫描：不得新增 `codex exec` / `codex exec resume` 调用。

## 6. 禁止声明

R3-A13 禁止声明：

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

## 7. 形状预算

- 是否允许新增 Rust 文件：是，1 个。
- 新增 Rust 文件目标行数：`<= 900`。
- `lib.rs` 只允许新增 1 行 module declaration。
- 是否允许新增 Tauri command：否。
- 是否允许新增 sidecar JSON 种类：否。
- 是否需要 shape gate 豁免：否。
- 本任务规划基线 commit：`d1e2ce1c139f392437928a367a9744411eb7ecc4`
- 本任务完成 commit：`d96ed041a16ce05da3219b38053e75843e8339ec`

## 8.1 完成摘要

R3-A13 Level A 已完成：

- 新增 `workbench_sqlite_transaction_acceptance.rs`。
- 新增 `fixtures/r3-a13/transaction-acceptance-core`。
- 单事务内完成 candidate adoption、formal memory record、formal memory version、formal memory audit event、workflow audit event。
- before-commit failure injection 均证明不会留下 half-adopted state。
- after-commit-before-report 分类为 `committed_but_report_failed`，不冒充完整完成。
- cutover gap matrix 明确 R3-A9 / A10 / A11 / A12 仍只有 Level A，Level B pending。

Implementation commit：`d96ed041a16ce05da3219b38053e75843e8339ec`。

## 8. 交接要求

完成后必须写入：

- evidence：`evidence/2026-06-11-root-treatment-r3-a13-transaction-acceptance-and-cutover-gap-matrix-v1.md`
- handoff：`handoffs/2026-06-11-root-treatment-r3-a13-transaction-acceptance-and-cutover-gap-matrix-v1-result.md`

handoff 必须包含：

- 实现文件。
- fixture 路径。
- 事务验收结果。
- failure injection 矩阵。
- cutover gap matrix。
- 验证命令结果。
- Level B 未执行边界。
- 下一步建议：R3 Level B 决策或进入 R4 读模型 / 前端瘦身。
