# Root Treatment R3-A5 Fixture Only Observation Export And Rollback Verification v1

日期：2026-06-11

## STATUS

`DONE_WITH_CONCERNS`

R3-A5 fixture-only observation / export / rollback verification rehearsal 已实现并完成验证。该结果只接受为临时 DB + 临时 JSON projection root + R3-A5 fixture root 内的 observation period、export verification、rollback recovery verification 和 failure injection 演练；不接受为生产 DB、生产 read-cut、JSON / sidecar stop-write、真实数据迁移、产品读写路径切 DB 或多 agent 并行真实执行解锁。

## Start / End Commit

- start commit：`6a9b5b7433f2bd50fc80e1a37d081a87822dde6b`
- start commit message：`docs: add r3 a5 observation rehearsal task`
- end commit：未提交；本开发线按任务要求不执行 `git add` / `git commit`
- 当前变更等待主管线回收

## Read / Write Scope

### 读取

- R3-A5 任务包、当前入口文档、AGENTS / multi-agent collaboration 规则、R3 合同。
- R3-A1 / R3-A2 / R3-A3 / R3-A4 任务包与 supervisor checkpoint。
- 现有 R3 SQLite modules：schema / importer / apply / exporter / dual-write / read-cut。
- 现有 R3-A4 fixtures，用作 R3-A5 独立 fixture payload 来源。

### 写入

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_observation_period.rs`
- `prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a5/**`
- `evidence/2026-06-11-root-treatment-r3-a5-fixture-only-observation-export-and-rollback-verification-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-a5-fixture-only-observation-export-and-rollback-verification-v1-result.md`

未更新 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md` 或 Root Treatment 官方计划。

## Changed Files

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
  - 仅新增 `mod workbench_sqlite_observation_period;`。
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_observation_period.rs`
  - 新增 fixture-only observation period rehearsal API、two-sample stability verification、export verification、rollback recovery verification dry-run、failure injection 和 focused tests。
- `prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a5/**`
  - 新增 9 组 R3-A5 fixture 目录，共 63 个 JSON 输入文件。
- 本 evidence / handoff。

## Shape Metrics

- `lib.rs` before：13955 行。
- `lib.rs` after：13956 行。
- `lib.rs` delta：+1 行，仅 module declaration。
- `workbench_sqlite_observation_period.rs`：1047 行，低于 Rust 新文件 3000 行上限。
- Tauri command 总量：96。
- `lib.rs` 内 command 数量：0。
- sidecar JSON kinds：shape gate 检测 14 allowed / 0 unknown。

## Observation Rehearsal Summary

新增 `rehearse_fixture_observation_period(...)`，要求调用方显式传入：

- fixture source root。
- temp DB path。
- temp JSON projection root。
- observation report path。
- rollback manifest path。
- optional observation / export / rollback failure injection point。

实现路径：

1. 校验 fixture root 仅允许 temp 或 `src-tauri/fixtures/r3-a5/**`。
2. 校验 DB path 仅允许 temp path。
3. 校验 projection root / observation report / rollback manifest 仅允许 temp 或 R3-A5 fixture path，且 report / manifest 必须位于 projection root 下。
4. 复用 R3-A2 `apply_fixture_dir_to_temp_db` 写入 temp DB。
5. 连续两次复用 R3-A2 `export_temp_db_to_json_dry_run` 生成 deterministic observation samples。
6. 校验 sample 1 / sample 2 的 export hash、projection hash、projected files、DB row counts 和 redaction policy 稳定。
7. 只在稳定后写 projection files、completed rollback manifest 和 completed observation report。

成功报告明确 `observation_status=stable_verified`、`stable_verified=true`、`degraded=false`。blocked / degraded / rollback readiness 不会写 completed stable observation report。

## Export Verification Summary

- Observation success 来自 temp DB export dry-run、projection hash 和 two-sample stability，不复制 source fixture 作为成功证据。
- Export verification 记录：
  - `source_root_hash`
  - `db_export_hash`
  - `projection_hash`
  - `export_manifest_hash`
  - per-file `path` / `hash` / `record_count` / `redaction_status`
  - runtime log alias policy
- Projection 文件写入 temp projection root。
- Canonical runtime log alias policy：
  - 输出 `runtime-logs.v1.json`。
  - 不输出 legacy singular `runtime-log.v1.json`。
- 未新增 sidecar JSON kind。

## Rollback Recovery Verification Summary

Rollback verification 只生成 dry-run plan，不执行真实恢复。`rollback_recovery_verification` 明确记录：

- `would_disable_db_read_cut=true`
- `would_use_last_verified_json_projection=true`
- `would_preserve_db_for_audit=true`
- `would_require_supervisor_decision=true`
- `production_restore_performed=false`

Completed manifest：`rollback-manifest.json`。Manifest 记录 source root hash、DB path ref、projection root ref、projected file hashes/counts、DB row counts、redaction policy、canonical runtime log alias policy 和 rollback dry-run verification。

## Failure Injection Summary

| Failure point | 行为 | 证据 |
| --- | --- | --- |
| `BeforeObservationSample` | DB / projection / report 均不创建 | `sqlite_observation_failure_before_sample_creates_no_outputs` |
| `AfterFirstExportBeforeSecondSample` | sample 1 后中断，不写 completed report | `sqlite_observation_failure_after_first_export_before_second_sample_creates_no_report` |
| `ExportHashMismatch` | export verification blocked，不写 stable report | `sqlite_observation_export_hash_mismatch_blocks_without_stable_report` |
| `ProjectionFileMissing` | projection file missing blocked，不写 stable report | `sqlite_observation_projection_missing_blocks_without_stable_report` |
| `ProjectionFileCorrupt` | projection corrupt blocked，不写 stable report | `sqlite_observation_projection_corrupt_blocks_without_stable_report` |
| `RollbackManifestMissing` | manifest missing blocked，不写 stable report | `sqlite_observation_missing_manifest_blocks_without_stable_report` |
| `RollbackManifestIncomplete` | incomplete manifest blocked，completed manifest removed，不写 stable report | `sqlite_observation_incomplete_manifest_blocks_without_stable_report` |
| `DbIntegrityOrSchemaMismatch` | status degraded by error path，不写 stable report | `sqlite_observation_db_integrity_failure_is_degraded_and_has_no_stable_report` |
| `ObservationDriftBetweenSamples` | two-sample drift blocked，不写 stable report | `sqlite_observation_drift_between_samples_blocks_without_stable_report` |
| `AfterRollbackSelectedBeforeReportCommit` | rollback manifest 可见，但 stable report 不提交 | `sqlite_observation_failure_after_rollback_selected_before_report_commit_creates_no_report` |

## Fixture Coverage Matrix

| Fixture | Coverage |
| --- | --- |
| `observation-export-valid-core-chain` | temp DB apply + export dry-run + two-sample observation + stable report |
| `observation-export-idempotent-rerun` | same fixture / DB / projection root rerun keeps stable report text |
| `observation-export-hash-mismatch-blocked` | export hash mismatch blocked |
| `observation-projection-missing-blocked` | projection missing and corrupt blocked |
| `observation-manifest-missing-blocked` | rollback manifest missing blocked |
| `observation-manifest-incomplete-blocked` | incomplete rollback manifest blocked |
| `observation-db-integrity-failure-degraded` | corrupt DB / schema mismatch degraded, no stable report |
| `rollback-export-recovery-verification-dry-run` | rollback recovery verification dry-run only |
| `observation-sensitive-redaction` | report / projection / manifest omit forbidden sensitive body classes |

说明：R3-A5 fixtures 复用 R3-A4 legal payload shape under new `fixtures/r3-a5/**` dirs。该复用只用于 fixture-only rehearsal，不修改 A4 fixtures，不代表生产数据迁移。

## Forbidden Sensitive Field Handling

- A5 report / projection / manifest 不输出 prompt body、full transcript、secret/token/credential/keychain/OAuth/provider credential 或 rollout body。
- Exporter redaction policy 继续输出 `prompt_body:omitted`、`full_transcript:omitted`、secret/token/credential/keychain/OAuth/provider credential omitted、`rollout_body:omitted`。
- `observation-sensitive-redaction` 联合扫描 observation report、rollback manifest 和 projection files，确认不包含 provider credential value、full transcript body、rollout body payload 或 `"prompt_body"`。
- 敏感 / 真实执行扫描结果：命中均为 redaction policy / 测试断言 / `plan_authorization(s)` 合法表名 / `db_authoritative` 状态文本；未命中真实 `Command::new("codex")`、真实 `codex exec` / `codex exec resume`、`/Users/yoyi/.codex` 访问或 fixture 中的敏感 body。

## Runtime-Log Alias Handling

- A5 projection 只输出 canonical `runtime-logs.v1.json`。
- A5 helper 显式拒绝 `runtime-log.v1.json` export / projection 文件。
- `sqlite_observation_export_records_per_file_verification_fields` 和 `sqlite_observation_stable_verifies_two_samples_and_writes_report` 覆盖 canonical alias。

## Evidence / Checks Run

Required checks:

- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors / 0 warnings；`lib.rs` 13956 行；Tauri commands 96 total / 0 in `lib.rs`；sidecar JSON kinds 14 allowed / 0 unknown。
- `cargo test --lib sqlite_schema`：pass，3 passed。
- `cargo test --lib sqlite_apply_importer`：pass，6 passed。
- `cargo test --lib sqlite_export_dry_run`：pass，3 passed。
- `cargo test --lib sqlite_dual_write`：pass，10 passed。
- `cargo test --lib sqlite_read_cut`：pass，12 passed。
- `cargo test --lib sqlite_observation`：pass，15 passed。
- `cargo test --lib workflow_state`：pass，11 passed。
- `cargo test --lib`：pass，391 passed / 16 ignored。
- `cargo fmt -- --check`：pass。
- `git diff --check`：pass。
- Required sensitive / real-exec scan：ran; only expected redaction-policy/test/assertion, legal `plan_authorization(s)` naming, and `db_authoritative` status text matches.
- Required sidecar / observation / export / rollback scan：ran; only allowed workflow / memory / runtime / product / continuation names and A5 observation/export/rollback names in R3-A5 fixtures and SQLite rehearsal modules.

Known warning:

- Cargo tests still show the existing `JsonRpcError::invalid_params` dead_code warning from `src/mcp/protocol.rs`; this warning was already recorded in prior R3 checkpoints and is not introduced by R3-A5.

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：R3-A5 仍是 fixture-only rehearsal；不是生产 DB、production read-cut、JSON / sidecar stop-write、rollback production workflow 或 R3 SQLite 完成。
- P2：A5 fixtures 使用 R3-A4 legal payload shape under new R3-A5 fixture dirs；后续可增加更丰富 domain-specific payload，但本轮覆盖 required observation/export/rollback failure matrix。
- P2：生产 read path、JSON stop-write、SQLite production transaction boundary、rollback production workflow 和多 agent 并行真实执行解锁仍待后续 R3 task。

## Boundary Confirmation

- 未创建生产 DB。
- 未写用户真实数据目录。
- 未迁移真实 `workflow-state.v0.json` 或 sidecar。
- 未修改任何真实 JSON / sidecar。
- 未切任何产品读写路径到 DB。
- 未让真实 app read model 读 DB。
- 未停止 JSON / sidecar 写入。
- 未把 JSON 降为生产 fallback。
- 未在 app startup / Tauri command / UI 中接入 observation rehearsal。
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

## Do Not Claim

- 不声明 R3 SQLite 迁移开始或完成。
- 不声明生产 DB 创建完成。
- 不声明生产双写期开始。
- 不声明生产读切 DB 完成。
- 不声明 JSON / sidecar 停写。
- 不声明 rollback production workflow 完成。
- 不声明多 agent 并行真实执行解锁。
- 不声明 Stage L / K3-B1 / K3-B2 已恢复。
