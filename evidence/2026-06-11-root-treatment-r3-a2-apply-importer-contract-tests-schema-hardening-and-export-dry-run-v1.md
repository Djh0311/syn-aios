# Root Treatment R3-A2 Apply Importer Contract Tests Schema Hardening And Export Dry Run v1 Evidence

日期：2026-06-11

## STATUS

`DONE_WITH_CONCERNS`

R3-A2 已完成临时 DB apply importer 合同测试、schema constraint / index hardening、transaction failure injection fixtures 和 DB -> JSON export dry-run。所有实现仍限制在 temp DB / fixture / dry-run / contract tests；未创建生产 DB，未迁移真实 JSON / sidecar，未双写，未切产品读写路径，未新增 Tauri command。

主管线回收时补充了 `BeforeDbBegin` 独立 failure injection 测试，用于证明 begin 前失败不会创建 temp DB。

## Commit / Worktree

- start commit：`556efb023601cb7f59b0cb44aad1d563e02bad5d`
- end commit：无。本开发线按任务包要求不运行 `git add` / `git commit`。
- 初始 `git status --short`：无输出。
- 当前变更仅限 R3-A2 允许写入范围，最终 `git status --short` 见本文件末尾。

## 读取文件

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `AGENTS.md`
- `codex-multi-agent-safe-collaboration.md`
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`
- `docs/plans/2026-06-10-root-treatment-r0-shape-gate-and-governance-task-package-rule-v1.md`
- `docs/plans/2026-06-11-root-treatment-r3-sqlite-schema-importer-rollback-contract-v1.md`
- `tasks/2026-06-11-root-treatment-r3-a1-sqlite-schema-and-idempotent-importer-dry-run-v1.md`
- `tasks/2026-06-11-root-treatment-r3-a2-apply-importer-contract-tests-schema-hardening-and-export-dry-run-v1.md`
- `evidence/2026-06-11-root-treatment-r3-a1-sqlite-schema-and-idempotent-importer-dry-run-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-a1-sqlite-schema-and-idempotent-importer-dry-run-v1-result.md`
- `evidence/2026-06-11-root-treatment-r3-a1-supervisor-checkpoint-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-a1-supervisor-checkpoint-v1-result.md`
- `prototypes/productized-desktop-shell/src-tauri/Cargo.toml`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_importer.rs`
- `prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a1/**`

## CHANGED_FILES

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_importer.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_apply.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_exporter.rs`
- `prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a2/**`
- `evidence/2026-06-11-root-treatment-r3-a2-apply-importer-contract-tests-schema-hardening-and-export-dry-run-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-a2-apply-importer-contract-tests-schema-hardening-and-export-dry-run-v1-result.md`

未写入口文档、Root Treatment 官方计划、UI、TypeScript、migration 文件、生产 DB 初始化路径或 Tauri command。

## SUMMARY

- `lib.rs` 只新增 `mod workbench_sqlite_apply;` / `mod workbench_sqlite_exporter;`。
- `workbench_sqlite_schema.rs` 补 `CHECK`、`UNIQUE` 已有约束验证和核心索引断言；temp initializer 扩展为允许 R3 fixture root。
- `workbench_sqlite_importer.rs` 只把 R3-A1 dry-run constants / hash helper 放宽到 `pub(crate)`，供 apply/export 复用；未改变 dry-run 行为。
- 新增 `workbench_sqlite_apply.rs`：显式 fixture root + temp DB path apply importer，使用 SQLite transaction，支持 failure injection。
- 新增 `workbench_sqlite_exporter.rs`：从 temp DB 生成内存 export manifest、projection hashes、record counts 和 redaction manifest，不写 JSON / sidecar 文件。
- 新增 `fixtures/r3-a2/**` 22 个 fixture 文件，覆盖 apply success、idempotent reapply、conflict rollback、revision conflict、corrupt primary、sensitive reject、crash injection、export dry-run 和 runtime log alias policy。

## SCHEMA HARDENING SUMMARY

- `import_batches.mode` 增加 `CHECK(mode IN ('dry_run', 'apply'))`。
- `import_batches.status` 增加 `accepted / applied / rejected / rolled_back` 检查。
- `import_sources.status` 增加 accepted / missing / rejected / skipped 状态检查。
- `source_records.status` 增加 accepted / skipped_duplicate / conflict 检查。
- `export_batches.status` 增加 planned / completed / failed / dry_run 检查。
- 新增核心索引：
  - `idx_import_batches_source_mode`
  - `idx_import_sources_batch_kind`
  - `idx_source_records_kind_natural`
  - workflow / work item / memory / observation / product command / continuation / runtime summary 相关索引。
- focused test `sqlite_schema_hardens_core_constraints_and_indexes` 断言 CHECK / index 存在，并验证 invalid import mode 被 SQLite 拒绝。

## TEMP DB APPLY IMPORTER SUMMARY

`apply_fixture_dir_to_temp_db(fixture_root, db_path, failure_point)`：

- 只接受显式传入的 temp path 或仓库内 `fixtures/r3-a2` DB path。
- 先运行 R3-A1 `dry_run_import_fixture_dir`，只有 accepted / accepted_with_rejections 且无 conflict / sensitive reject 才允许 apply。
- 调用 `initialize_temp_workbench_sqlite_db` 初始化 temp DB。
- 使用单个 SQLite transaction 写入 `import_batches`、`import_sources`、`source_records` 和最小 domain tables。
- 同一 fixture 重复 apply 依靠 primary key / `ON CONFLICT DO NOTHING` 幂等；第二次不新增 domain rows。
- legacy `runtime-log.v1.json` 只作为 source/alias 被 dry-run 识别；apply domain runtime table 只吸收 canonical `runtime-logs.v1.json`。

## TRANSACTION / FAILURE INJECTION SUMMARY

failure injection 覆盖：

- `BeforeDbBegin`：DB begin 前失败，不建 transaction。
- `AfterDbBeginBeforeFirstInsert`：begin 后、第一条 insert 前失败，rollback，无 rows。
- `AfterImportBatchBeforeDomainInsert`：import batch/source 写入后、domain insert 前失败，rollback，无 rows。
- `AfterFirstDomainInsertBeforeCommit`：首条 domain insert 后、commit 前失败，rollback，无 partial domain rows。
- `BeforeCommit`：commit 前失败，rollback，无 rows。
- `AfterCommitBeforeExportManifest`：commit 后、export manifest 前失败；用于证明 commit 后错误不会伪装为 rollback，本测试保留已提交 rows。

focused test：

- `sqlite_apply_importer_rolls_back_failure_injection_before_commit`
- `sqlite_apply_importer_after_commit_injection_keeps_committed_rows`
- `sqlite_apply_importer_rejects_conflicts_sensitive_and_corrupt_without_partial_rows`

## EXPORT DRY-RUN SUMMARY

`export_temp_db_to_json_dry_run(db_path, target_root_ref)`：

- 只读取显式 temp / R3 fixture DB path。
- 返回内存 `SqliteExportDryRunManifest`，包含 `export_id`、`mode = dry_run`、`status = planned`、`export_hash`、projected files、record counts 和 redaction manifest。
- 不写 `export-manifest.json`，不写任何 JSON / sidecar 文件。
- 可投影：
  - `workflow-state.v0.json`
  - `formal-memories.v1.json`
  - `runtime-logs.v1.json`
  - `real-execution-product-commands.v1.json`
  - `session-continuations.v1.json`
- projection 内容递归删除 forbidden export keys；redaction manifest 只记录 omission policy。
- runtime alias policy：export 只输出 canonical `runtime-logs.v1.json`，不输出 `runtime-log.v1.json`。

## FIXTURE COVERAGE MATRIX

| 要求 | fixture / test |
| --- | --- |
| apply-valid-core-chain | `fixtures/r3-a2/apply-valid-core-chain/**` + `sqlite_apply_importer_applies_valid_chain_and_reapply_is_idempotent` |
| apply-idempotent-reapply | `fixtures/r3-a2/apply-idempotent-reapply/workflow-state.v0.json`；核心幂等由同一 valid chain reapply 测试覆盖 |
| apply-conflict-rollback | `fixtures/r3-a2/apply-conflict-rollback/workflow-state.v0.json` + reject/rollback test |
| apply-revision-conflict-rollback | `fixtures/r3-a2/apply-revision-conflict-rollback/workflow-state.v0.json` + reject/rollback test |
| apply-corrupt-primary-reject | `fixtures/r3-a2/apply-corrupt-primary-reject/workflow-state.v0.json` + reject/rollback test |
| apply-sensitive-reject | `fixtures/r3-a2/apply-sensitive-reject/workflow-state.v0.json` + reject/rollback test |
| crash-after-source-before-domain | `fixtures/r3-a2/crash-after-source-before-domain/workflow-state.v0.json` + after-commit injection test |
| crash-after-domain-before-commit | `fixtures/r3-a2/crash-after-domain-before-commit/workflow-state.v0.json` + before-commit rollback test |
| export-dry-run-workflow-runtime | `fixtures/r3-a2/export-dry-run-workflow-runtime/**` + export projection test |
| runtime-log-alias-export-policy | `fixtures/r3-a2/runtime-log-alias-export-policy/**` + canonical alias export test |

## FORBIDDEN SENSITIVE FIELD HANDLING

- importer 继续拒绝 `prompt_body`、secret/token/credential/keychain/OAuth/provider credential/full transcript/transcript body/rollout body 等 key 或 marker。
- R3-A2 apply 在 dry-run 前置发现 sensitive / conflict / corrupt primary 时拒绝 batch，不进入 DB transaction domain rows。
- export dry-run 对 projected files 递归省略 forbidden export keys，并在 redaction manifest 记录 policy。
- 敏感扫描命中项为 importer/exporter 拒绝 / redaction 清单、合法 `plan_authorization(s)` 名称和 R3-A2 forbidden fixture 的 `prompt_body` 测试字段；精确 no-real-Codex 扫描无输出。

## RUNTIME-LOG ALIAS EXPORT HANDLING

- `runtime-logs.v1.json` 是 canonical persisted / export name。
- `runtime-log.v1.json` 只作为 legacy alias / source ref label 被 dry-run 识别。
- apply 不把 legacy alias 写入 `runtime_log_entries` domain table。
- export dry-run manifest 只包含 `runtime-logs.v1.json`，并断言不输出 `runtime-log.v1.json`。

## Shape / Line Metrics

- `lib.rs`：13953 行，只新增 2 行 module declaration。
- `workbench_sqlite_schema.rs`：303 行。
- `workbench_sqlite_importer.rs`：1236 行。
- `workbench_sqlite_apply.rs`：1026 行，低于 3000 行。
- `workbench_sqlite_exporter.rs`：420 行，低于 3000 行。
- R3-A2 fixture 文件：22 个。
- Tauri command 总量：96；`lib.rs` 内 command 数量：0。
- shape gate sidecar：14 detected / 0 unknown。

## EVIDENCE / CHECKS RUN

- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors / 0 warnings；`lib.rs` 13953 行，Tauri commands 96 total / 0 in `lib.rs`，sidecar 14 detected / 0 unknown。
- `cargo test --lib sqlite_schema`：pass，3 passed / 0 failed / 366 filtered。
- `cargo test --lib sqlite_importer_dry_run`：pass，6 passed / 0 failed / 363 filtered。
- `cargo test --lib sqlite_apply_importer`：pass，6 passed / 0 failed / 364 filtered。
- `cargo test --lib sqlite_export_dry_run`：pass，3 passed / 0 failed / 366 filtered。
- `cargo test --lib workflow_state`：pass，11 passed / 0 failed / 358 filtered。
- `cargo test --lib`：pass，354 passed / 0 failed / 16 ignored。
- `cargo fmt -- --check`：pass。
- `git diff --check`：pass，无输出。
- `git status --short`：completed，见本文件末尾。
- `rg -n 'Command::new\("codex"\)|codex exec|codex exec resume|/Users/yoyi/\.codex|auth|token|provider credential|prompt_body|full transcript|rollout' ...`：completed；命中合法拒绝/脱敏清单、`plan_authorization(s)` 命名和 R3-A2 forbidden fixture `prompt_body`。
- 精确扫描 `Command::new("codex")|codex exec|codex exec resume|/Users/yoyi/.codex`：无输出。
- `rg -n 'workflow-state|formal-memories|memory-candidates|observations|runtime-log|runtime-logs|plan-authorizations|project-proposals|real-execution-product-commands|session-continuations' ...`：completed；命中 importer/apply/export allowed sidecar handling 和 R3-A2 fixture coverage。

Filtered cargo tests 均有匹配测试；未发生 no-match。

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：apply importer 仍是 fixture/temp DB contract implementation，未接产品写路径。
- P2：schema v0 仍偏 coarse；更多 FK / typed columns / table normalization 可放到 R3-A3。
- P2：export dry-run 覆盖核心 projection，不覆盖所有历史 sidecar corner case。
- P2：Product Command / continuation / runtime log 的真实单事务产品写路径仍待后续 R3 task。
- P2：DB -> JSON export 仍不写盘；rollback/export production workflow 尚未实现。

## BOUNDARY CONFIRMATION

- 未创建生产 DB。
- 未写用户真实数据目录。
- 未迁移真实 `workflow-state.v0.json` 或 sidecar。
- 未修改任何真实 JSON / sidecar。
- 未双写 DB + JSON。
- 未切任何产品读写路径到 DB。
- 未在 app startup / Tauri command / UI 中接入 DB initializer、apply importer 或 exporter。
- 未改 workflow state 顶层 schema。
- 未新增 sidecar store 或 sidecar JSON 种类。
- 未新增 Tauri command。
- 未改真实 Codex runner。
- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript 或 rollout。
- 未启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 未启动 Stage L / K3-B1 retry / K3-B2。
- 未解冻 backlog 功能。
- 未迁移 R2 inline tests 巨石。
- 未做 R4 前端按页读模型或 UI 瘦身。
- 未运行 `git add` / `git commit`。

## 不能声明完成

- 不能声明 R3 SQLite 迁移开始或完成。
- 不能声明生产 DB 创建完成。
- 不能声明双写期开始。
- 不能声明读切 DB 完成。
- 不能声明 JSON / sidecar 停写。
- 不能声明 DB -> JSON export 写盘完成。
- 不能声明 transaction boundary 全部产品化完成。
- 不能声明多 agent 并行真实执行解锁。
- 不能声明 Stage L / K3-B1 / K3-B2 已恢复。

## Final git status --short

```text
 M prototypes/productized-desktop-shell/src-tauri/src/lib.rs
 M prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_importer.rs
 M prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs
?? evidence/2026-06-11-root-treatment-r3-a2-apply-importer-contract-tests-schema-hardening-and-export-dry-run-v1.md
?? handoffs/2026-06-11-root-treatment-r3-a2-apply-importer-contract-tests-schema-hardening-and-export-dry-run-v1-result.md
?? prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a2/
?? prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_apply.rs
?? prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_exporter.rs
```
