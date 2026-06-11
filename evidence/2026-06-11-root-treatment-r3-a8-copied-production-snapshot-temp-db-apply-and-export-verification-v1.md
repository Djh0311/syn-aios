# Root Treatment / R3-A8 Copied Snapshot Temp DB Apply Evidence v1

日期：2026-06-11

## STATUS

DONE

本轮只完成 R3-A8 Level A：fixture / temp copied snapshot apply + temp DB + export + rollback boundary verification。Level B 未执行；未读取真实 workbench state root，未复制真实 production snapshot。

## READ / WRITE SCOPE

读取范围：

- R3-A8 任务包、当前权威入口、R3-A7 / R3-A6 相关上下文。
- 现有 R3 SQLite helper：`workbench_sqlite_preflight.rs`、`workbench_sqlite_apply.rs`、`workbench_sqlite_exporter.rs`、`workbench_sqlite_importer.rs`、`workbench_sqlite_schema.rs`、`workbench_sqlite_observation_period.rs`。
- 仓库内 R3 fixture。

写入范围：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_snapshot_apply.rs`
- `prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a8/**`
- `evidence/2026-06-11-root-treatment-r3-a8-copied-production-snapshot-temp-db-apply-and-export-verification-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-a8-copied-production-snapshot-temp-db-apply-and-export-verification-v1-result.md`

未写入：入口文档、生产 root、真实 JSON / sidecar、Tauri command、UI、app startup hook。

## LEVEL A SUMMARY

新增 `workbench_sqlite_snapshot_apply.rs`，实现 crate 内 helper：

- `rehearse_copied_snapshot_apply_level_a(...)`
- `SqliteSnapshotApplyConfig`
- `SqliteSnapshotApplyFailurePoint`
- `SqliteSnapshotApplyReport`
- copied file manifest / preflight summary / apply summary / export verification / rollback boundary / safety flags 结构

行为边界：

- 先对 source fixture snapshot 运行 preflight。
- 再复制 source fixture snapshot 到 temp copy root。
- 只对 temp copy root 执行 copy preflight、temp DB apply、DB -> JSON export dry-run projection、rollback boundary dry-run。
- 拒绝 copy root / report path / temp DB path / temp export root 位于 source root 内。
- 拒绝 denied marker path，并保持默认 denied markers 不能被空 custom config 移除。
- completed report 的 flags 全部保持 false。

`lib.rs` 只新增 `mod workbench_sqlite_snapshot_apply;` 一行。

行数：

- `lib.rs` before：13957 行。
- `lib.rs` after：13958 行。
- 新 helper：1257 行，低于 3000 行上限。

Shape metrics：

- Tauri commands：96 total。
- `lib.rs` 内 Tauri command：0。
- Sidecar JSON kinds：14 detected，0 unknown。

## LEVEL B STATUS

未执行。

- 未读取真实 workbench state root。
- 未复制真实 production snapshot。
- 未写 `product-line/tmp/root-treatment-r3-a8/**` 的 Level B staging。
- 未声明 production apply / production DB / production read-cut / JSON stop-write。

## COPIED SNAPSHOT MANIFEST

新增 fixture root：`prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a8/**`。

Fixture matrix：

| Fixture | Purpose |
| --- | --- |
| `snapshot-valid-core-chain` | success path：copy -> preflight -> temp DB apply -> export -> rollback boundary |
| `snapshot-idempotent-rerun` | same snapshot / same temp roots deterministic report text |
| `snapshot-preflight-blocked-denied-path` | source preflight blocked by denied `.env` path marker |
| `snapshot-apply-corrupt-blocked` | source legal，copy 后注入 corrupt primary，apply rejected |
| `snapshot-export-hash-mismatch-blocked` | export hash mismatch injection |
| `snapshot-rollback-manifest-missing-blocked` | rollback manifest missing injection |
| `snapshot-rollback-manifest-incomplete-blocked` | rollback manifest incomplete injection |
| `snapshot-cleanup-failure-boundary` | cleanup failure leaves only temp artifacts and source unchanged |

Fixture file count：57 files。

Copied manifest file name uses `copied-snapshot-manifest.json`，不是 `*.v1.json` sidecar kind；rollback boundary manifest uses `rollback-boundary-manifest.json`，不是新 sidecar kind。

## TEMP DB APPLY SUMMARY

Temp DB apply reuses `apply_fixture_dir_to_temp_db` against the copied snapshot root only.

Covered checks:

- temp DB path must be absolute and under temp.
- source/copy root hashes must match before apply.
- source fixture hash mismatch can be blocked via optional expected hash.
- corrupt copied snapshot is rejected before completed report is written.
- DB row counts are recorded from temp DB only.

No production DB path is accepted or created.

## EXPORT VERIFICATION SUMMARY

Export verification reuses `export_temp_db_to_json_dry_run` and writes projected files only under temp export root during rehearsal.

Verified:

- canonical `runtime-logs.v1.json` is required.
- legacy singular `runtime-log.v1.json` is rejected if emitted or present.
- projected file path / hash / record_count / redaction_status are recorded.
- export hash mismatch injection blocks completed report.
- redaction manifest is propagated from exporter.

## ROLLBACK BOUNDARY SUMMARY

Rollback boundary is dry-run only:

- `would_disable_db_read_cut=true`
- `would_use_snapshot_projection=true`
- `would_preserve_temp_db_for_audit=true`
- `would_require_supervisor_decision=true`
- `production_restore_performed=false`

No production restore is performed.

## FAILURE INJECTION SUMMARY

Covered by `cargo test --lib sqlite_snapshot`:

- source preflight blocked.
- copy destination inside source root rejected.
- report path inside source root rejected.
- temp DB path inside source root rejected.
- copy interrupted before manifest blocks report.
- apply rejected / corrupt copied snapshot blocks report.
- export hash mismatch blocks report.
- rollback manifest missing blocks report.
- rollback manifest incomplete blocks report.
- cleanup failure leaves only temp artifacts and source hash unchanged.
- default denied markers remain active even when custom config has an empty denied list.

## CHECKS RUN

All commands were run from the repo or `src-tauri` as appropriate.

| Command | Result |
| --- | --- |
| `node scripts/harness/workbench-shape-gate.js --mode check` | pass; 0 errors / 0 warnings |
| `cargo test --lib sqlite_schema` | pass; 3 passed |
| `cargo test --lib sqlite_snapshot` | pass; 13 passed |
| `cargo test --lib sqlite_preflight` | pass; 8 passed |
| `cargo test --lib sqlite_apply` | pass; 6 passed |
| `cargo test --lib sqlite_export` | pass; 3 passed |
| `cargo test --lib sqlite_observation` | pass; 15 passed |
| `cargo test --lib workflow_state` | pass; 11 passed |
| `cargo test --lib` | pass; 412 passed / 16 ignored |
| `cargo fmt -- --check` | pass |
| `git diff --check` | pass |
| `git status --short` | pass command run; expected R3-A8 files only |

Notes:

- Cargo emitted existing warning: `mcp::protocol::JsonRpcError::invalid_params` is unused.
- Initial shape gate failed because two temp manifest names used `*.v1.json` and looked like new sidecar kinds. Fixed by renaming them to non-sidecar manifest names; final shape gate passed.

## SCANS

Flag true-pattern scan before evidence/handoff:

- Result: no hits.

Sensitive marker scan before evidence/handoff:

- Hits only in helper denied-marker constants.
- Hidden fixture scan confirmed `.env` is a fixture-only denied marker file; it contains no secret material.

Final sensitive marker scan after evidence/handoff:

- Hits in helper denied-marker constants: guard / denied marker definitions only.
- Hits in evidence/handoff: forbidden-scan text and boundary confirmation only.
- No real secret, token, credential, keychain, OAuth, provider credential, full transcript, rollout body, or `.codex` data was read or written.

Sidecar / allowed JSON scan:

- A8 fixtures use existing allowed JSON kinds only: `workflow-state.v0.json`, `formal-memories.v1.json`, `memory-candidates.v1.json`, `observations.v1.json`, `runtime-logs.v1.json`, `real-execution-product-commands.v1.json`, `session-continuations.v1.json`.

## P0 / P1 / P2

P0：无。

P1：无。

P2：无新增。保留既有 Cargo dead_code warning，不属于本任务新增风险。

## BOUNDARY CONFIRMATION

确认未执行：

- Level B real copied snapshot rehearsal。
- 真实 workbench state root read。
- production DB create / apply。
- production root write。
- product read/write path cutover。
- JSON / sidecar stop-write。
- Tauri command / UI / startup hook 接入。
- 真实 `codex exec` / `codex exec resume`。
- prompt send。
- `/Users/yoyi/.codex` read/write。
- secret / token / env / keychain / OAuth / provider credential / full transcript / rollout 读取。
- Tauri / Browser / Chrome / Vite / screenshot 启动。
- Stage L / K3-B1 / K3-B2 / backlog 解冻。

## DO NOT CLAIM

本轮不得声明：

- R3 SQLite 迁移开始或完成。
- production DB 已创建。
- production apply 已完成。
- production read-cut 已完成。
- JSON / sidecar 停写。
- rollback production workflow 完成。
- Level B copied real state rehearsal 完成。
- 多 agent 并行真实执行解锁。
