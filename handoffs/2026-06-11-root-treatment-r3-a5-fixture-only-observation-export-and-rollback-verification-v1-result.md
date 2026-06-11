# Root Treatment R3-A5 Fixture Only Observation Export And Rollback Verification v1 Result

日期：2026-06-11

## STATUS

`DONE_WITH_CONCERNS`

R3-A5 fixture-only observation / export / rollback verification rehearsal 已完成并等待主管线回收；本开发线未执行 `git add` / `git commit`。

## CHANGED_FILES

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_observation_period.rs`
- `prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a5/**`
- `evidence/2026-06-11-root-treatment-r3-a5-fixture-only-observation-export-and-rollback-verification-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-a5-fixture-only-observation-export-and-rollback-verification-v1-result.md`

## OBSERVATION REHEARSAL SUMMARY

- 新增 `workbench_sqlite_observation_period` fixture-only rehearsal 模块。
- `lib.rs` 仅新增 1 行 `mod workbench_sqlite_observation_period;`。
- Rehearsal 显式传入 fixture root、temp DB path、temp projection root、observation report path、rollback manifest path 和 optional failure injection。
- Success path：fixture apply 到 temp DB -> two-sample DB export dry-run -> export/projection/hash/count/redaction stability verification -> write projection -> commit rollback manifest -> write `stable_verified` observation report。
- Blocked / degraded / rollback readiness 路径不写 completed stable observation report。

## EXPORT VERIFICATION SUMMARY

- Export verification 来自 temp DB export dry-run，不复制 source fixture 作为 success evidence。
- Report 记录 source root hash、DB export hash、projection hash、export manifest hash 和 per-file path/hash/record_count/redaction_status。
- Runtime log canonical alias：输出 `runtime-logs.v1.json`，不输出 `runtime-log.v1.json`。

## ROLLBACK RECOVERY VERIFICATION SUMMARY

- Rollback recovery verification 是 dry-run only。
- Manifest / report 明确：would-disable DB read-cut、would-use last verified JSON projection、would-preserve DB for audit、would-require supervisor decision、`production_restore_performed=false`。
- 不执行真实恢复，不写 production JSON，不切产品路径。

## FAILURE INJECTION SUMMARY

- `BeforeObservationSample`：无 DB / projection / report。
- `AfterFirstExportBeforeSecondSample`：sample 1 后中断，无 report。
- `ExportHashMismatch`：blocked，无 stable report。
- `ProjectionFileMissing` / `ProjectionFileCorrupt`：blocked，无 stable report。
- `RollbackManifestMissing` / `RollbackManifestIncomplete`：blocked，无 stable report。
- `DbIntegrityOrSchemaMismatch`：degraded error path，无 stable report。
- `ObservationDriftBetweenSamples`：blocked，无 stable report。
- `AfterRollbackSelectedBeforeReportCommit`：rollback manifest 可见，stable report 不提交。

## FIXTURE COVERAGE MATRIX

- `observation-export-valid-core-chain`：stable observation。
- `observation-export-idempotent-rerun`：idempotent rerun。
- `observation-export-hash-mismatch-blocked`：export hash mismatch。
- `observation-projection-missing-blocked`：projection missing / corrupt。
- `observation-manifest-missing-blocked`：missing rollback manifest。
- `observation-manifest-incomplete-blocked`：incomplete rollback manifest。
- `observation-db-integrity-failure-degraded`：DB integrity / schema mismatch degraded。
- `rollback-export-recovery-verification-dry-run`：rollback verification dry-run only。
- `observation-sensitive-redaction`：sensitive body omission across report / projection / manifest。

## FORBIDDEN SENSITIVE FIELD HANDLING

- Report / projection / manifest 不包含 prompt body、full transcript、secret/token/credential/keychain/OAuth/provider credential 或 rollout body。
- Required sensitive scan 已运行；命中仅为 redaction policy / 测试断言 / 合法 `plan_authorization(s)` 表名 / `db_authoritative` 状态文本。
- 未命中真实 Codex execution、`.codex` 访问或 fixture 敏感 body。

## EVIDENCE / CHECKS RUN

- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors / 0 warnings。
- `cargo test --lib sqlite_schema`：3 passed。
- `cargo test --lib sqlite_apply_importer`：6 passed。
- `cargo test --lib sqlite_export_dry_run`：3 passed。
- `cargo test --lib sqlite_dual_write`：10 passed。
- `cargo test --lib sqlite_read_cut`：12 passed。
- `cargo test --lib sqlite_observation`：15 passed。
- `cargo test --lib workflow_state`：11 passed。
- `cargo test --lib`：391 passed / 16 ignored。
- `cargo fmt -- --check`：pass。
- `git diff --check`：pass。
- Required sensitive / sidecar scans：ran, expected-only matches.

Known warning：cargo test 仍显示既有 `JsonRpcError::invalid_params` dead_code warning；非 R3-A5 引入。

## COMMITS / METRICS

- start commit：`6a9b5b7433f2bd50fc80e1a37d081a87822dde6b`
- end commit：未提交，待主管线回收。
- `lib.rs` before / after：13955 -> 13956 行。
- `workbench_sqlite_observation_period.rs`：1047 行。
- R3-A5 fixtures：9 组 / 63 个 JSON 输入文件。
- Tauri commands：96 total / 0 in `lib.rs`。

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：本轮仅是 fixture-only observation/export/rollback rehearsal，不是生产 DB、生产 read-cut、JSON / sidecar stop-write 或 R3 完成。
- P2：R3-A5 fixtures 复用 R3-A4 legal payload shape under new R3-A5 dirs；后续可继续扩展更丰富 payload。
- P2：生产 read path、rollback production workflow、JSON stop-write、多 agent 并行真实执行解锁仍待后续 R3 task。

## BOUNDARY CONFIRMATION

- 未创建生产 DB。
- 未写用户真实数据目录。
- 未迁移或修改真实 JSON / sidecar。
- 未切产品读写路径到 DB。
- 未停写 JSON / sidecar，未把 JSON 降为生产 fallback。
- 未新增 Tauri command。
- 未接入 app startup / UI / 产品路径。
- 未读写 `/Users/yoyi/.codex`。
- 未执行真实 `codex exec` / `codex exec resume`，未发送 prompt。
- 未读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 未启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 未启动 Stage L / K3-B1 / K3-B2，未解冻 backlog 功能。

## REQUESTS

- 主管线回收时请 fresh rerun required checks if needed，并决定是否提交。
- 不要把本结果声明为 R3 SQLite 迁移完成、生产 read-cut 完成、JSON / sidecar stop-write 或多 agent 并行真实执行解锁。
