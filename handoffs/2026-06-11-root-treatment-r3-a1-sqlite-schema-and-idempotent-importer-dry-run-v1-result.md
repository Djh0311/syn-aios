# Root Treatment R3-A1 SQLite Schema And Idempotent Importer Dry Run v1 Result

日期：2026-06-11

## STATUS

`DONE_WITH_CONCERNS`

R3-A1 开发线完成最小 SQLite schema module、临时 / fixture DB initializer、idempotent dry-run importer 和 R3-A1 fixture 矩阵。未提交，未 stage；请主管线复核后决定回收。

## CHANGED_FILES

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_importer.rs`
- `prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a1/**`
- `evidence/2026-06-11-root-treatment-r3-a1-sqlite-schema-and-idempotent-importer-dry-run-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-a1-sqlite-schema-and-idempotent-importer-dry-run-v1-result.md`

## SUMMARY

- `lib.rs` 只新增 `mod workbench_sqlite_importer;` / `mod workbench_sqlite_schema;`。
- 新增 schema DDL 常量和 temp DB initializer。
- 新增 dry-run importer report、source inventory、record summaries、natural key / hash / conflict / warning 分类。
- 新增 15 组 R3-A1 fixtures，覆盖任务包要求矩阵。
- 新增 focused Rust tests；filters 有匹配，没有 no-match。

## SCHEMA MODULE SUMMARY

- DDL 覆盖 metadata/source/export/rollback、workflow、memory/observation、workflow governance、runtime/continuation/product command/readback。
- `initialize_temp_workbench_sqlite_db` 只接受 temp dir 或 R3-A1 fixture 路径。
- 非 temp / fixture 路径返回 `temp_or_fixture_path_required`。
- 未新增 migration 文件，未创建生产 DB。

## TEMP DB INITIALIZER SUMMARY

- 显式使用调用方传入路径。
- 创建 parent dir、打开 SQLite、启用 foreign keys、执行 DDL、记录 `schema_migrations`。
- 不从 app state / startup 推导路径。
- 不接 Tauri command，不接 UI，不写真实数据目录。

## DRY-RUN IMPORTER / IDEMPOTENCY SUMMARY

- 只读 fixture 目录的 `workflow-state.v0.json` 与允许 sidecar。
- report deterministic，包含 batch id、mode、source root hash、source inventory、record hashes、natural keys、counts、warnings、conflicts。
- same natural key + same hash：`skipped_duplicate`。
- same natural key + different hash：`conflict`。
- second pass with previous report：unchanged records become `skipped_duplicate`。
- corrupt primary：batch `rejected_corrupt_primary`。
- corrupt optional sidecar：source `rejected_corrupt` + warning。
- unknown sidecar：source `rejected_unknown`。
- forbidden sensitive field：source / batch `rejected_sensitive`，proposed inserts 0。

## FIXTURE COVERAGE MATRIX

- `valid-empty-workflow`
- `valid-workflow-core`
- `memory-adoption-chain`
- `memory-capture-chain`
- `proposal-authorization-chain`
- `process-fact-observation`
- `product-command-runtime-chain`
- `runtime-log-alias`
- `corrupt-primary`
- `corrupt-optional-sidecar`
- `duplicate-same-hash`
- `duplicate-different-hash`
- `revision-conflict`
- `unknown-sidecar`
- `forbidden-sensitive-field`

## FORBIDDEN SENSITIVE FIELD HANDLING

Importer 拒绝 `prompt_body`、secret/token/credential/keychain/OAuth/provider credential/full transcript/transcript body/rollout body 等 key 或 marker。扫描命中的是 importer 拒绝清单和 R3-A1 forbidden fixture 的 `prompt_body`，不是外部 secret，也没有读取 `/Users/yoyi/.codex`。

## RUNTIME-LOG ALIAS HANDLING

- `runtime-logs.v1.json`：canonical，source kind `runtime_log`。
- `runtime-log.v1.json`：legacy alias / ref label，source kind `runtime_log_legacy_alias`。
- Dry-run report 输出 alias policy；R3-A1 不做 export。

## EVIDENCE / CHECKS RUN

- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors / 0 warnings。
- `cargo test --lib sqlite_schema`：pass，2 passed / 0 failed / 358 filtered。
- `cargo test --lib sqlite_importer_dry_run`：pass，6 passed / 0 failed / 354 filtered。
- `cargo test --lib workflow_state`：pass，11 passed / 0 failed / 349 filtered。
- `cargo test --lib`：pass，344 passed / 0 failed / 16 ignored。
- `cargo fmt -- --check`：pass。
- `git diff --check`：pass。
- required sensitive scan：完成；命中拒绝清单和 forbidden fixture，精确 no-Codex scan 无输出。
- required sidecar scan：完成；命中 importer allowed list 和 R3-A1 fixture coverage。

TDD red / green 已记录在 evidence：初始 focused tests 先因 unimplemented 失败，完成实现后通过。

## METRICS

- start commit：`183f30e40c1a89071942e26f486c2396eba4a0b3`
- end commit：无，开发线不提交。
- `lib.rs`：13951 行。
- `workbench_sqlite_schema.rs`：242 行。
- `workbench_sqlite_importer.rs`：1236 行。
- Tauri commands：96 total / 0 in `lib.rs`。
- 新增 Tauri command：0。
- 新增 sidecar JSON 种类：0。

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：dry-run importer 尚未实现 apply mode。
- P2：schema v0 字段约束仍是最小合同表域，后续需要细化索引、FK 和 transaction tests。
- P2：DB -> JSON export / rollback 仍未实现。
- P2：Product Command / continuation / runtime log transaction 仍待 R3 后续实现。

## BOUNDARY CONFIRMATION

- 未创建生产 DB。
- 未写用户真实数据目录。
- 未迁移真实 JSON / sidecar。
- 未双写。
- 未切 DB 读路径。
- 未接 app startup / Tauri command / UI。
- 未改 workflow state 顶层 schema。
- 未新增 sidecar store / sidecar JSON 种类。
- 未新增 Tauri command。
- 未改真实 Codex runner。
- 未执行真实 Codex。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 未启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 未启动 Stage L / K3-B1 / K3-B2。
- 未解冻 backlog 功能。
- 未 `git add` / `git commit`。

## REQUESTS

- 请主管线复核 R3-A1 后再决定是否创建 R3-A2。
- R3-A2 建议不要直接切读写路径；优先做 apply importer contract tests、schema constraint hardening、transaction crash fixture 和 DB -> JSON export dry-run。
