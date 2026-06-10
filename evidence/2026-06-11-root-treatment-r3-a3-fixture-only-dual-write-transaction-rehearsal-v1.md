# Root Treatment R3-A3 Fixture Only Dual Write Transaction Rehearsal v1

日期：2026-06-11

## STATUS

`DONE_WITH_CONCERNS`

R3-A3 fixture-only dual-write transaction rehearsal 已实现并完成验证。该结果只接受为临时 DB + 临时 JSON projection root + R3-A3 fixture root 内的双写事务演练、projection cleanup、rollback manifest 和 recovery dry-run 完成；不接受为生产双写期开始、生产 DB 创建、真实 JSON / sidecar 迁移、读写路径切 DB、JSON / sidecar 停写或 R3 SQLite 完成。

## Start / End Commit

- start commit：`c729ecab14df32076c5436d048aa7d4b69efdeea`
- end commit：未提交；本开发线按任务要求不执行 `git add` / `git commit`
- 当前变更等待主管线回收

## Read / Write Scope

### 读取

- R3-A3 任务包、R3 合同、R3-A1 / R3-A2 任务包、R3-A2 supervisor checkpoint / handoff、当前权威入口和本地协作规则。
- 现有 R3-A1 / R3-A2 SQLite 模块与 fixture。

### 写入

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_dual_write.rs`
- `prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a3/**`
- `evidence/2026-06-11-root-treatment-r3-a3-fixture-only-dual-write-transaction-rehearsal-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-a3-fixture-only-dual-write-transaction-rehearsal-v1-result.md`

未更新 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md` 或 Root Treatment 官方计划。

## Changed Files

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
  - 仅新增 `mod workbench_sqlite_dual_write;`。
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_dual_write.rs`
  - 新增 fixture-only dual-write rehearsal API、rollback manifest / recovery dry-run manifest 生成、failure injection 和 focused tests。
- `prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a3/**`
  - 新增 8 组 R3-A3 fixture 目录，共 56 个 JSON 输入文件。
- 本 evidence / handoff。

## Shape Metrics

- `lib.rs` before：13953 行。
- `lib.rs` after：13954 行。
- `lib.rs` delta：+1 行，仅 module declaration。
- `workbench_sqlite_dual_write.rs`：574 行，低于 Rust 新文件 3000 行上限。
- Tauri command 总量：96。
- `lib.rs` 内 command 数量：0。
- sidecar JSON kinds：final shape gate 检测 14 allowed / 0 unknown。

## Implementation Summary

新增 `rehearse_fixture_dual_write(...)`，要求调用方显式传入：

- fixture source root。
- temp DB path。
- temp JSON projection root。
- rollback manifest path。
- optional failure injection point。

实现路径：

1. 校验 fixture root 仅允许 temp 或 `src-tauri/fixtures/r3-a3/**`。
2. 校验 DB path 仅允许 temp path。
3. 校验 projection root / manifest path 仅允许 temp 或 `src-tauri/fixtures/r3-a3/**`，且 manifest 必须位于 projection root 下。
4. 复用 R3-A2 `apply_fixture_dir_to_temp_db` 写入 temp DB。
5. 复用 R3-A2 `export_temp_db_to_json_dry_run` 从 DB 生成 projection manifest。
6. 将 export dry-run projection 写入 temp projection root。
7. 原子写入 `rollback-manifest.json`，记录 DB path、projection root、source root hash、projected files、hashes、counts、redaction policy 和 recovery dry-run instructions。

该模块未接入 app startup、Tauri command、UI 或任何产品读写路径。

## Dual-Write Rehearsal Summary

- success path：DB apply commit -> DB export dry-run -> write projection files -> commit rollback manifest。
- idempotency：同一 fixture / same temp DB / same projection root 重跑后 manifest 文本稳定；A2 apply importer 用 `ON CONFLICT DO NOTHING` 保证 DB row 不重复。
- DB rows evidence：manifest 记录 `import_batches`、`import_sources`、`source_records`、`projects`、`workflows`、`formal_memory_records`、`product_commands`、`session_continuations`、`runtime_log_entries` counts。

## Projection / Export Dry-Run Summary

- projection 只来自 DB export dry-run，不复制 source fixture。
- projection 写盘只发生在 temp projection root 或 R3-A3 fixture root 下；测试使用 temp projection root。
- projection 文件沿用已有 allowed names：
  - `workflow-state.v0.json`
  - `formal-memories.v1.json`
  - `runtime-logs.v1.json`
  - `real-execution-product-commands.v1.json`
  - `session-continuations.v1.json`
- 未新增 sidecar JSON 种类。

## Rollback Manifest / Recovery Dry-Run Summary

- completed manifest：`rollback-manifest.json`。
- manifest commit 前失败：写 `rollback-manifest.incomplete.json`，不写 completed manifest。
- projection partial 失败：清理 partial projection，写 `projection-cleanup-incomplete.json`。
- recovery dry-run：manifest 仅记录 `recovery_dry_run_only`、would-remove projection root、would-preserve DB for audit；不执行真实恢复、不写真实 JSON、不改生产路径。

## Failure Injection Summary

| Failure point | 行为 | 证据 |
| --- | --- | --- |
| `BeforeDbApply` | DB / projection / manifest 均不创建 | `sqlite_dual_write_before_db_apply_failure_creates_no_outputs` |
| `AfterDbApplyBeforeProjectionWrite` | DB committed rows 保留；projection 未写；返回 `projection_failed_after_db_commit` | `sqlite_dual_write_after_db_before_projection_failure_keeps_db_without_projection` |
| `AfterFirstProjectionFileBeforeManifest` | partial projection 清理；manifest 不完成；写 cleanup incomplete marker | `sqlite_dual_write_projection_failure_cleans_partial_files_before_manifest` |
| `BeforeManifestCommit` | projection 可见；写 incomplete manifest；completed manifest 不存在 | `sqlite_dual_write_before_manifest_commit_marks_incomplete_without_completed_manifest` |
| `AfterManifestCommit` | completed manifest 已存在；返回 injected failure | `sqlite_dual_write_after_manifest_commit_keeps_completed_manifest_and_reports_failure` |

## Fixture Coverage Matrix

| Fixture | Coverage |
| --- | --- |
| `dual-write-valid-core-chain` | DB apply + projection + completed rollback manifest success |
| `dual-write-idempotent-rerun` | same fixture rerun idempotent manifest / DB row behavior |
| `dual-write-after-db-before-projection-failure` | DB committed, projection not written, explicit projection failure |
| `dual-write-after-first-projection-before-manifest-failure` | partial projection cleanup before manifest |
| `dual-write-before-manifest-commit-failure` | incomplete manifest without completed manifest |
| `dual-write-after-manifest-commit` | completed manifest remains after post-commit injected failure |
| `dual-write-sensitive-redaction` | safe summary/hash fields only; projection and manifest redaction policy omit forbidden body classes |
| `rollback-manifest-recovery-dry-run` | recovery dry-run instructions without real restore output |

说明：R3-A3 fixtures 复用 R3-A2 合法 core chain JSON 形状作为输入素材，并放入独立 `fixtures/r3-a3/**` 目录。该复用只用于 fixture-only rehearsal，不改变 A2 fixtures，不代表生产数据迁移。

## Forbidden Sensitive Field Handling

- A3 success fixture 不包含 prompt body、full transcript、secret/token/credential/keychain/OAuth/provider credential 或 rollout body。
- `dual-write-sensitive-redaction` 使用 `prompt_hash`、`prompt_summary`、`redaction_status` 等安全替代字段，不写 forbidden body 字段。
- Projection 复用 A2 exporter redaction，输出不包含 `prompt_body`、`full_transcript`、`rollout_body` 或 provider credential body。
- Manifest 记录 redaction policy：`prompt_body:omitted`、`full_transcript:omitted`、secret/token/credential/keychain/OAuth/provider credential omitted、`rollout_body:omitted`。

敏感 / 真实执行扫描结果：命令有预期命中，但命中均为 redaction policy / 测试断言 / `plan_authorization(s)` 合法命名；未命中真实 `Command::new("codex")`、真实 `codex exec` / `codex exec resume`、`/Users/yoyi/.codex` 访问或 fixture 中的敏感 body。

## Runtime-Log Alias Handling

- A3 projection 只输出 canonical `runtime-logs.v1.json`。
- 未输出 legacy singular `runtime-log.v1.json`。
- 未新增 runtime log sidecar kind。

## Evidence / Checks Run

TDD red:

- `cargo test --lib sqlite_dual_write`：初次红灯，0 passed / 9 failed，失败原因为 `sqlite_dual_write_not_implemented` 或预期错误码未实现。

Final required checks:

- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors / 0 warnings；14 allowed sidecar kinds / 0 unknown；Tauri commands 96 total / 0 in `lib.rs`。
- `cargo test --lib sqlite_schema`：pass，3 passed。
- `cargo test --lib sqlite_apply_importer`：pass，6 passed。
- `cargo test --lib sqlite_export_dry_run`：pass，3 passed。
- `cargo test --lib sqlite_dual_write`：pass，10 passed。
- `cargo test --lib workflow_state`：pass，11 passed。
- `cargo test --lib`：pass，364 passed / 0 failed / 16 ignored。
- `cargo fmt -- --check`：pass。
- `git diff --check`：pass。
- Required sensitive / real-exec scan：ran; only expected redaction-policy/test/assertion and `plan_authorization(s)` naming matches.
- Required sidecar/projection scan：ran; only allowed workflow / memory / runtime / product / continuation names in R3-A3 fixtures and exporter/dual-write code.
- `git status --short` after source validation, before evidence / handoff write：only R3-A3 scope (`lib.rs`, `fixtures/r3-a3/**`, `workbench_sqlite_dual_write.rs`).

Process note:

- First shape gate run failed because a test literal used `recovered-workflow-state.v0.json`, which the gate classified as an unknown sidecar kind. The literal was removed and replaced with a generic `recovered-` prefix scan; final shape gate passed.

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：R3-A3 仍是 fixture-only rehearsal；不是生产双写期、生产 migration、read-cutover 或 rollback production workflow。
- P2：A3 fixtures 使用 R3-A2 legal core chain shape under new R3-A3 fixture directories；后续可增加更丰富 domain-specific payload，但本轮已覆盖 required failure/recovery matrix。
- P2：SQLite schema / importer 仍是 v0 prep；FK / typed columns / production transaction boundary 仍待后续 R3 task。

## Boundary Confirmation

- 未创建生产 DB。
- 未写用户真实数据目录。
- 未迁移真实 `workflow-state.v0.json` 或 sidecar。
- 未修改任何真实 JSON / sidecar。
- 未切任何产品读写路径到 DB。
- 未在 app startup / Tauri command / UI 中接入 dual-write rehearsal。
- 未改 workflow state 顶层 schema。
- 未新增 sidecar store 或 sidecar JSON 种类。
- 未新增 Tauri command。
- 未改真实 Codex runner。
- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 未启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 未启动 Stage L / K3-B1 retry / K3-B2。
- 未解冻 backlog 功能。
- 未夹带 R4 前端按页读模型或 UI 瘦身。

## Do Not Claim

- 不声明 R3 SQLite 迁移开始或完成。
- 不声明生产 DB 创建完成。
- 不声明生产双写期开始。
- 不声明读切 DB 完成。
- 不声明 JSON / sidecar 停写。
- 不声明 rollback production workflow 完成。
- 不声明多 agent 并行真实执行解锁。
- 不声明 Stage L / K3-B1 / K3-B2 已恢复。
