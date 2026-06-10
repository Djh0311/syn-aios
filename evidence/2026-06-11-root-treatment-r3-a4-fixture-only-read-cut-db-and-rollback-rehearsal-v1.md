# Root Treatment R3-A4 Fixture Only Read-Cut DB And Rollback Rehearsal v1

日期：2026-06-11

## STATUS

`DONE_WITH_CONCERNS`

R3-A4 fixture-only read-cut DB and rollback rehearsal 已实现并完成验证。本结果只接受为临时 DB + 临时 JSON projection root + R3-A4 fixture root 内的 read-cut / fallback / rollback recovery dry-run 演练；不接受为生产 read-cut、生产 DB、真实 JSON / sidecar 迁移、JSON / sidecar 停写、产品读写路径切 DB、rollback production workflow 或多 agent 并行真实执行解锁。

## Start / End Commit

- start commit：`221232cedc8e7cd2dc326005820eb575c1a40544`
- end commit：未提交；本开发线按任务要求不执行 `git add` / `git commit`
- 当前变更等待主管线回收

## Read / Write Scope

### 读取

- R3-A4 任务包、当前权威入口、R3 合同、R3-A1 / R3-A2 / R3-A3 任务包与 R3-A3 supervisor checkpoint。
- 现有 R3 SQLite modules：schema / importer / apply / exporter / dual-write。

### 写入

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_read_cut.rs`
- `prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a4/**`
- `evidence/2026-06-11-root-treatment-r3-a4-fixture-only-read-cut-db-and-rollback-rehearsal-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-a4-fixture-only-read-cut-db-and-rollback-rehearsal-v1-result.md`

未更新 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md` 或 Root Treatment 官方计划。

## Changed Files

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
  - 仅新增 `mod workbench_sqlite_read_cut;`。
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_read_cut.rs`
  - 新增 fixture-only read-cut rehearsal API、DB authoritative projection/hash verification、JSON projection fallback、rollback recovery dry-run report、failure injection 和 focused tests。
- `prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a4/**`
  - 新增 9 组 R3-A4 fixture 目录，共 63 个 JSON 输入文件。
- 本 evidence / handoff。

## Shape Metrics

- `lib.rs` before：13954 行。
- `lib.rs` after：13955 行。
- `lib.rs` delta：+1 行，仅 module declaration。
- `workbench_sqlite_read_cut.rs`：966 行，低于 Rust 新文件 3000 行上限。
- Tauri command 总量：96。
- `lib.rs` 内 command 数量：0。
- sidecar JSON kinds：shape gate 检测 14 allowed / 0 unknown。

## Read-Cut Rehearsal Summary

新增 `rehearse_fixture_read_cut(...)`，要求调用方显式传入：

- fixture source root。
- temp DB path。
- temp JSON projection root。
- rollback manifest path。
- read-cut report path。
- optional read-cut / fallback failure injection point。

成功路径：

1. 校验 fixture root 仅允许 temp 或 `src-tauri/fixtures/r3-a4/**`。
2. 校验 DB path 仅允许 temp path。
3. 校验 projection / manifest / report path 仅允许 temp 或 R3-A4 fixture path，且 manifest / report 必须在 projection root 下。
4. 复用 R3-A2 `apply_fixture_dir_to_temp_db` 写入 temp DB。
5. 复用 R3-A2 `export_temp_db_to_json_dry_run` 从 DB 生成 projection。
6. 以 DB export hash 作为 authoritative read hash，并和 projection hash 对齐。
7. 从 DB export dry-run projection 写入 temp projection root，生成 completed rollback manifest。
8. 写 `read-cut-report.json`，记录 DB path、projection root、manifest path、source root hash、DB read hash、projection hash、counts、fallback decision、redaction policy 和 recovery dry-run。

该模块未接入 app startup、Tauri command、UI 或任何产品读写路径。

## JSON Fallback / Degrade Summary

- `DbUnavailable`：删除 temp DB 后读取已验证 projection manifest，状态为 `fallback_degraded`，`read_source=json_projection_fallback`，不显示 DB authoritative success。
- `CorruptDbPathOrSchemaMismatch`：写入 corrupt DB bytes 触发 integrity / schema failure，然后 fallback 到已验证 JSON projection，状态仍为 degraded。
- fallback 验证不会只信 source fixture：读取 completed rollback manifest，校验 `source_root_hash`，并逐个读取 projection 文件重新计算 canonical hash。
- fallback report 中 `db_read_hash=None`，`fallback_decision=selected:<reason>`，和 DB success 的 `fallback_decision=not_used` 明确区分。

## Rollback Recovery Dry-Run Summary

- completed manifest：`rollback-manifest.json`。
- read-cut report recovery block 只记录 dry-run decision：
  - would-use DB when authoritative success。
  - would-use JSON projection when fallback degraded。
  - would-disable DB read-cut on fallback / recovery planning。
  - would-preserve DB for audit。
  - production restore performed = false。
- recovery dry-run 不执行真实恢复、不写 production JSON、不改真实 sidecar。

## Failure Injection Summary

| Failure point | 行为 | 证据 |
| --- | --- | --- |
| `BeforeDbRead` | DB / projection / read-cut report 均不创建 | `sqlite_read_cut_failure_injection_before_db_read_creates_no_report` |
| `AfterDbReadBeforeProjectionVerification` | DB read 后、projection verification 前失败；不写 completed report | `sqlite_read_cut_failure_after_db_read_before_verification_creates_no_report` |
| `ProjectionHashMismatch` | DB export hash 与 injected projection hash 不一致；read-cut blocked | `sqlite_read_cut_projection_hash_mismatch_blocks_without_completed_report` |
| `MissingRollbackManifest` | manifest 被删除后阻断；不写 completed report | `sqlite_read_cut_missing_manifest_blocks_without_completed_report` |
| `IncompleteRollbackManifest` | 写 incomplete manifest 后阻断并清理 completed manifest | `sqlite_read_cut_incomplete_manifest_blocks_without_completed_report` |
| `DbUnavailable` | DB unavailable fallback，状态 degraded | `sqlite_read_cut_db_unavailable_uses_verified_json_projection_fallback` |
| `CorruptDbPathOrSchemaMismatch` | corrupt DB / schema mismatch fallback，状态 degraded | `sqlite_read_cut_schema_mismatch_fallback_is_degraded_not_db_success` |
| `AfterFallbackSelectedBeforeReportCommit` | fallback selected 后、report commit 前失败；不写 report | `sqlite_read_cut_failure_after_fallback_before_report_commit_creates_no_report` |

## Fixture Coverage Matrix

| Fixture | Coverage |
| --- | --- |
| `read-cut-valid-core-chain` | DB apply + DB export dry-run + projection hash verification + completed read-cut report |
| `read-cut-idempotent-rerun` | same fixture / DB / projection root rerun keeps report text stable |
| `read-cut-db-unavailable-json-fallback` | DB unavailable, verified JSON projection fallback, degraded status |
| `read-cut-db-schema-mismatch-fallback` | corrupt DB / schema mismatch fallback, degraded status, not DB success |
| `read-cut-projection-hash-mismatch-blocked` | projection hash mismatch blocks read-cut and writes no completed report |
| `read-cut-missing-manifest-blocked` | missing rollback manifest blocks read-cut |
| `read-cut-incomplete-manifest-blocked` | incomplete rollback manifest blocks read-cut |
| `read-cut-sensitive-redaction` | projection/report omit forbidden sensitive body classes |
| `rollback-read-cut-recovery-dry-run` | recovery dry-run instructions without real restore output |

说明：R3-A4 fixtures 复用 R3-A3 合法 core-chain / sensitive-redaction JSON 形状作为输入素材，并放入独立 `fixtures/r3-a4/**` 目录。该复用只用于 fixture-only rehearsal，不改变 A3 fixtures，不代表生产数据迁移。

## Forbidden Sensitive Field Handling

- A4 success fixture 不包含 prompt body、full transcript、secret/token/credential/keychain/OAuth/provider credential 或 rollout body。
- `read-cut-sensitive-redaction` 使用安全替代字段，projection 复用 A2 exporter redaction，不输出 forbidden body 字段。
- Report 只包含 hash / path ref / counts / redaction policy；不持久化 prompt body、full transcript、provider credential body 或 rollout body。
- 敏感扫描命中均为 redaction policy / 测试断言 / `plan_authorization(s)` 合法 schema/table 名 / `db_authoritative` 状态字符串；未命中真实 `Command::new("codex")`、真实 `codex exec` / `codex exec resume`、`/Users/yoyi/.codex` 访问或 fixture 中的敏感 body。

## Runtime-Log Alias Handling

- A4 projection 只输出 canonical `runtime-logs.v1.json`。
- 未输出 legacy singular `runtime-log.v1.json`。
- 未新增 runtime log sidecar kind。

## Evidence / Checks Run

Required checks:

- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors / 0 warnings；14 allowed sidecar kinds / 0 unknown；Tauri commands 96 total / 0 in `lib.rs`。
- `cargo test --lib sqlite_schema`：pass，3 passed。
- `cargo test --lib sqlite_apply_importer`：pass，6 passed。
- `cargo test --lib sqlite_export_dry_run`：pass，3 passed。
- `cargo test --lib sqlite_dual_write`：pass，10 passed。
- `cargo test --lib sqlite_read_cut`：pass，12 passed。
- `cargo test --lib workflow_state`：pass，11 passed。
- `cargo test --lib`：pass，376 passed / 0 failed / 16 ignored。
- `cargo fmt -- --check`：pass。
- `git diff --check`：pass before evidence / handoff write。
- `git status --short` before evidence / handoff write：only R3-A4 scope (`lib.rs`, `fixtures/r3-a4/**`, `workbench_sqlite_read_cut.rs`)。
- Final `git diff --check` after evidence / handoff write：pass。
- Final `git status --short` after evidence / handoff write：only R3-A4 scope plus this evidence / handoff.

Required scans:

- Sensitive / real-exec scan：ran; only expected redaction-policy/test/assertion and `plan_authorization(s)` naming matches.
- Sidecar / projection scan：ran; only allowed workflow / memory / runtime / product / continuation names in R3-A4 fixtures and exporter / dual-write / read-cut code, plus `read-cut` / `rollback-manifest` rehearsal names.

Cargo warnings:

- Existing warning remains: `JsonRpcError::invalid_params` is never used. This is pre-existing unrelated dead-code warning and was not changed by R3-A4.

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：R3-A4 仍是 fixture-only rehearsal；不是生产 read-cut、生产 migration、生产 DB、JSON / sidecar 停写或 rollback production workflow。
- P2：R3-A4 fixtures 复用 R3-A3 legal core chain / sensitive-redaction shape under new R3-A4 dirs；后续可增加更丰富 domain-specific payload，但本轮已覆盖 required failure/fallback/recovery matrix。
- P2：生产 read path、dual-write observation period、rollback production workflow、SQLite production transaction boundary 仍待后续 R3 task。

## Boundary Confirmation

- 未创建生产 DB。
- 未写用户真实数据目录。
- 未迁移真实 `workflow-state.v0.json` 或 sidecar。
- 未修改任何真实 JSON / sidecar。
- 未切任何产品读写路径到 DB。
- 未让真实 app read model 读 DB。
- 未在 app startup / Tauri command / UI 中接入 read-cut rehearsal。
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
- 不声明生产读切 DB 完成。
- 不声明 JSON / sidecar 停写。
- 不声明 rollback production workflow 完成。
- 不声明多 agent 并行真实执行解锁。
- 不声明 Stage L / K3-B1 / K3-B2 已恢复。
