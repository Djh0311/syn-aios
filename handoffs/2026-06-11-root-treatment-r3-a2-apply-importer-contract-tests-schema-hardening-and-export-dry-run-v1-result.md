# Root Treatment R3-A2 Apply Importer Contract Tests Schema Hardening And Export Dry Run v1 Result

日期：2026-06-11

## STATUS

`DONE_WITH_CONCERNS`

R3-A2 开发线完成 temp DB apply importer、schema hardening、transaction failure injection 和 export dry-run。未提交，未 stage；请主管线复核后决定回收。

主管线回收时补充了 `BeforeDbBegin` 独立 failure injection 测试，用于证明 begin 前失败不会创建 temp DB。

## CHANGED_FILES

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_importer.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_apply.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_exporter.rs`
- `prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a2/**`
- `evidence/2026-06-11-root-treatment-r3-a2-apply-importer-contract-tests-schema-hardening-and-export-dry-run-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-a2-apply-importer-contract-tests-schema-hardening-and-export-dry-run-v1-result.md`

## SUMMARY

- `lib.rs` 只新增两个模块声明。
- schema v0 增加 CHECK / index hardening 和 focused tests。
- apply importer 显式接收 fixture root + temp DB path，只写 temp / R3 fixture DB。
- apply importer 通过 SQLite transaction 写 metadata/source/record 和最小 domain tables。
- reapply 幂等，conflict / revision / corrupt / sensitive 均拒绝且无 partial domain rows。
- export dry-run 只返回内存 manifest / projection hash / redaction manifest，不写 sidecar。

## SCHEMA HARDENING SUMMARY

- 加固 `import_batches`、`import_sources`、`source_records`、`export_batches` 状态和模式 CHECK。
- 增加 importer/source/record、workflow core、memory、product command、continuation 和 runtime summary 核心索引。
- `cargo test --lib sqlite_schema` 覆盖 CHECK / index 断言和 invalid mode 拒绝。

## TEMP DB APPLY IMPORTER SUMMARY

- 新增 `workbench_sqlite_apply.rs`。
- `apply_fixture_dir_to_temp_db` 仅允许 temp / `fixtures/r3-a2` DB path。
- 使用 R3-A1 dry-run report 作为 apply 前置分类，不重新定义敏感/冲突规则。
- canonical runtime log 才进入 runtime domain table；legacy singular alias 不作为 export domain row。

## TRANSACTION / FAILURE INJECTION SUMMARY

- 覆盖 before begin、after begin、after import batch before domain、after first domain before commit、before commit、after commit before export manifest。
- commit 前 failure injection 均 rollback，无 import batch / domain partial rows。
- commit 后 failure injection 保留 committed rows，用于证明错误发生在 export manifest 前而非回滚。

## EXPORT DRY-RUN SUMMARY

- 新增 `workbench_sqlite_exporter.rs`。
- 生成 `workflow-state.v0.json`、`formal-memories.v1.json`、`runtime-logs.v1.json`、`real-execution-product-commands.v1.json`、`session-continuations.v1.json` projection manifest。
- projection hash deterministic。
- projected files 递归省略 forbidden sensitive keys；redaction manifest 只记录 policy。
- 不输出 `runtime-log.v1.json` legacy alias。

## FIXTURE COVERAGE MATRIX

- `apply-valid-core-chain`
- `apply-idempotent-reapply`
- `apply-conflict-rollback`
- `apply-revision-conflict-rollback`
- `apply-corrupt-primary-reject`
- `apply-sensitive-reject`
- `crash-after-source-before-domain`
- `crash-after-domain-before-commit`
- `export-dry-run-workflow-runtime`
- `runtime-log-alias-export-policy`

共 22 个 R3-A2 fixture 文件。

## FORBIDDEN SENSITIVE FIELD HANDLING

Importer 继续拒绝 forbidden sensitive key / marker；apply 在 dry-run 前置拒绝 sensitive batch；export dry-run 从 projected files 中递归省略 forbidden keys。敏感扫描命中的是拒绝/脱敏清单、合法 plan authorization 命名和 forbidden fixture `prompt_body` 测试值；精确 no-real-Codex 扫描无输出。

## RUNTIME-LOG ALIAS EXPORT HANDLING

- `runtime-logs.v1.json` 是 canonical export。
- `runtime-log.v1.json` 只作为 legacy source/ref label。
- export manifest 只包含 canonical plural file。

## EVIDENCE / CHECKS RUN

- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors / 0 warnings。
- `cargo test --lib sqlite_schema`：pass，3 passed。
- `cargo test --lib sqlite_importer_dry_run`：pass，6 passed。
- `cargo test --lib sqlite_apply_importer`：pass，6 passed。
- `cargo test --lib sqlite_export_dry_run`：pass，3 passed。
- `cargo test --lib workflow_state`：pass，11 passed。
- `cargo test --lib`：pass，354 passed / 0 failed / 16 ignored。
- `cargo fmt -- --check`：pass。
- `git diff --check`：pass。
- `git status --short`：completed，仅包含 R3-A2 允许范围文件。
- required sensitive scan：completed，命中均为拒绝/脱敏清单、合法 sidecar/table 命名或 forbidden fixture。
- required sidecar/source scan：completed，命中 importer/apply/export allowed sidecar handling 和 R3-A2 fixtures。

Filtered cargo tests 均有匹配测试；未发生 no-match。

## METRICS

- start commit：`556efb023601cb7f59b0cb44aad1d563e02bad5d`
- end commit：无，开发线不提交。
- `lib.rs`：13953 行。
- `workbench_sqlite_schema.rs`：303 行。
- `workbench_sqlite_importer.rs`：1236 行。
- `workbench_sqlite_apply.rs`：1026 行。
- `workbench_sqlite_exporter.rs`：420 行。
- Tauri commands：96 total / 0 in `lib.rs`。
- 新增 Tauri command：0。
- 新增 sidecar JSON 种类：0。

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：apply importer 仍是 fixture/temp DB 合同实现，未接产品写路径。
- P2：schema v0 仍可继续细化 FK / typed columns / table normalization。
- P2：export dry-run 不写盘，不代表 rollback/export production workflow 完成。
- P2：Product Command / continuation / runtime log 的真实单事务产品写路径仍待后续 R3 task。

## BOUNDARY CONFIRMATION

- 未创建生产 DB。
- 未写用户真实数据目录。
- 未迁移真实 JSON / sidecar。
- 未双写。
- 未切 DB 读写路径。
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

- 请主管线复核后决定是否提交 R3-A2。
- 后续 R3-A3 建议继续保持非生产路径，优先补 typed FK / importer coverage / rollback export rehearsal，不直接切产品读写。
