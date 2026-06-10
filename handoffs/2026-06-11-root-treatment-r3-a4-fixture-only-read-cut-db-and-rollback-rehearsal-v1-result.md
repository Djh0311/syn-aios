# Root Treatment R3-A4 Fixture Only Read-Cut DB And Rollback Rehearsal v1 Result

日期：2026-06-11

## STATUS

`DONE_WITH_CONCERNS`

R3-A4 fixture-only read-cut DB and rollback rehearsal 已完成并等待主管线回收；本开发线未执行 `git add` / `git commit`。

## CHANGED_FILES

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_read_cut.rs`
- `prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a4/**`
- `evidence/2026-06-11-root-treatment-r3-a4-fixture-only-read-cut-db-and-rollback-rehearsal-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-a4-fixture-only-read-cut-db-and-rollback-rehearsal-v1-result.md`

## READ-CUT REHEARSAL SUMMARY

- 新增 `workbench_sqlite_read_cut` fixture-only rehearsal 模块。
- `lib.rs` 仅新增 1 行 `mod workbench_sqlite_read_cut;`。
- DB authoritative success 由 temp DB apply + DB export dry-run + projection hash verification 得出，不复制 source fixture 当成功证据。
- 成功 report 写 `read-cut-report.json`，记录 DB hash、projection hash、manifest hash、row counts、fallback decision 和 recovery dry-run。

## JSON FALLBACK / DEGRADE SUMMARY

- DB unavailable fallback 和 corrupt DB / schema mismatch fallback 均输出 `fallback_degraded`。
- fallback 只读取已验证 projection：校验 completed manifest、source root hash 和 projection file canonical hash。
- fallback 不显示 DB success，`db_read_hash=None` 且 `fallback_decision=selected:<reason>`。

## ROLLBACK RECOVERY DRY-RUN SUMMARY

- Recovery block 只记录 dry-run plan。
- DB success would-use DB；fallback would-use JSON projection and would-disable DB read-cut。
- 始终 would-preserve DB for audit，`production_restore_performed=false`。

## FAILURE INJECTION SUMMARY

- Covered: before DB read。
- Covered: after DB read before projection verification。
- Covered: projection hash mismatch。
- Covered: missing rollback manifest。
- Covered: incomplete rollback manifest。
- Covered: DB unavailable。
- Covered: corrupt DB path / schema mismatch。
- Covered: after fallback selected before report commit。
- Blocked / injected failure paths do not write completed read-cut report.

## FIXTURE COVERAGE MATRIX

- `read-cut-valid-core-chain`：DB authoritative success。
- `read-cut-idempotent-rerun`：report/hash stability。
- `read-cut-db-unavailable-json-fallback`：DB unavailable fallback degraded。
- `read-cut-db-schema-mismatch-fallback`：schema/corrupt DB fallback degraded。
- `read-cut-projection-hash-mismatch-blocked`：hash mismatch blocked。
- `read-cut-missing-manifest-blocked`：missing manifest blocked。
- `read-cut-incomplete-manifest-blocked`：incomplete manifest blocked。
- `read-cut-sensitive-redaction`：report/projection sensitive body omission。
- `rollback-read-cut-recovery-dry-run`：recovery dry-run only。

## FORBIDDEN SENSITIVE FIELD HANDLING

- Report/projection 不输出 prompt body、full transcript、secret/token/credential/keychain/OAuth/provider credential 或 rollout body。
- Required sensitive scan only matched redaction policy, assertions, legal `plan_authorization(s)` naming, and `db_authoritative` status text.
- No real Codex command, `.codex` access, prompt send, full transcript, credential body, or rollout body was added.

## EVIDENCE / CHECKS RUN

- `node scripts/harness/workbench-shape-gate.js --mode check`：pass。
- `cargo test --lib sqlite_schema`：3 passed。
- `cargo test --lib sqlite_apply_importer`：6 passed。
- `cargo test --lib sqlite_export_dry_run`：3 passed。
- `cargo test --lib sqlite_dual_write`：10 passed。
- `cargo test --lib sqlite_read_cut`：12 passed。
- `cargo test --lib workflow_state`：11 passed。
- `cargo test --lib`：376 passed / 16 ignored。
- `cargo fmt -- --check`：pass。
- `git diff --check`：pass before evidence / handoff write。
- `git status --short`：before evidence / handoff write only R3-A4 source / fixture scope.
- Final `git diff --check` after evidence / handoff write：pass。
- Final `git status --short` after evidence / handoff write：only R3-A4 scope plus this evidence / handoff.
- Required sensitive / real-exec scan：ran; expected matches only.
- Required sidecar / projection scan：ran; allowed sidecar names and R3-A4 read-cut / rollback names only.

## COMMITS / METRICS

- start commit：`221232cedc8e7cd2dc326005820eb575c1a40544`
- end commit：未提交，待主管线回收。
- `lib.rs` before / after：13954 -> 13955 行。
- `workbench_sqlite_read_cut.rs`：966 行。
- R3-A4 fixtures：9 dirs / 63 JSON files。
- Tauri commands：96 total / 0 in `lib.rs`。

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：本轮仅是 fixture-only rehearsal，不是生产 DB / production read-cut / JSON sidecar stop-write。
- P2：fixtures 复用 R3-A3 legal payload shape under new R3-A4 dirs；后续可增加更丰富 domain fixtures。
- P2：生产 transaction boundary、production rollback workflow、read path cutover 和 observation period 仍待后续 R3 task。

## BOUNDARY CONFIRMATION

- 未创建生产 DB。
- 未写用户真实数据目录。
- 未迁移真实 JSON / sidecar。
- 未修改真实 `workflow-state.v0.json` 或 sidecar。
- 未切产品读写路径到 DB。
- 未新增 Tauri command。
- 未接入 app startup / UI / 产品路径。
- 未读写 `/Users/yoyi/.codex`。
- 未执行真实 `codex exec` / `codex exec resume`，未发送 prompt。
- 未读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 未启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 未启动 Stage L / K3-B1 / K3-B2，未解冻 backlog 功能。

## REQUESTS

- 主管线回收时请 fresh rerun required checks if needed，并决定是否提交。
- 不要把本结果声明为 R3 SQLite 迁移完成、生产 read-cut 完成、JSON / sidecar 停写或多 agent 并行真实执行解锁。
