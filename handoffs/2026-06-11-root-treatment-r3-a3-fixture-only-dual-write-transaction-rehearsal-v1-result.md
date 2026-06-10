# Root Treatment R3-A3 Fixture Only Dual Write Transaction Rehearsal v1 Result

日期：2026-06-11

## STATUS

`DONE_WITH_CONCERNS`

R3-A3 fixture-only dual-write transaction rehearsal 已完成并等待主管线回收；本开发线未执行 `git add` / `git commit`。

## CHANGED_FILES

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_dual_write.rs`
- `prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a3/**`
- `evidence/2026-06-11-root-treatment-r3-a3-fixture-only-dual-write-transaction-rehearsal-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-a3-fixture-only-dual-write-transaction-rehearsal-v1-result.md`

## SUMMARY

- 新增 `workbench_sqlite_dual_write` fixture-only rehearsal 模块。
- `lib.rs` 仅新增 1 行 module declaration。
- 新增 8 组 R3-A3 fixtures，共 56 个 JSON 输入文件。
- Rehearsal 明确限制在 temp DB + temp projection root + R3-A3 fixture root；不接入产品路径。

## DUAL-WRITE REHEARSAL SUMMARY

- 成功路径：fixture apply 到 temp DB -> DB export dry-run -> projection files 写入 temp projection root -> completed rollback manifest 原子提交。
- 幂等路径：同一 fixture / DB / projection root 重跑，manifest 文本稳定，DB rows 不重复。
- DB committed but projection failed：保留 DB committed rows，并返回 `projection_failed_after_db_commit`，不冒充 transaction rollback。

## PROJECTION / EXPORT DRY-RUN SUMMARY

- Projection 来自 DB export dry-run，不直接复制 source fixture。
- Projection 文件只写 temp projection root。
- 输出仅使用既有 allowed file names：`workflow-state.v0.json`、`formal-memories.v1.json`、`runtime-logs.v1.json`、`real-execution-product-commands.v1.json`、`session-continuations.v1.json`。
- Runtime log export 仍只输出 canonical `runtime-logs.v1.json`。

## ROLLBACK MANIFEST / RECOVERY DRY-RUN SUMMARY

- Completed manifest：`rollback-manifest.json`。
- Manifest 内容包含 source root hash、DB path ref、projection root ref、projected file hashes、row counts、redaction policy 和 recovery dry-run instructions。
- Recovery dry-run 只记录 would-remove / would-preserve，不执行真实恢复，不写 production JSON。

## FAILURE INJECTION SUMMARY

- `BeforeDbApply`：无 DB / projection / manifest 输出。
- `AfterDbApplyBeforeProjectionWrite`：DB committed rows 保留，projection 不写。
- `AfterFirstProjectionFileBeforeManifest`：partial projection 清理，写 cleanup incomplete marker。
- `BeforeManifestCommit`：写 incomplete manifest，不写 completed manifest。
- `AfterManifestCommit`：completed manifest 保留，返回 injected failure。

## FIXTURE COVERAGE MATRIX

- `dual-write-valid-core-chain`：success。
- `dual-write-idempotent-rerun`：idempotency。
- `dual-write-after-db-before-projection-failure`：DB committed / projection failed。
- `dual-write-after-first-projection-before-manifest-failure`：partial projection cleanup。
- `dual-write-before-manifest-commit-failure`：incomplete manifest。
- `dual-write-after-manifest-commit`：completed manifest remains after injected failure。
- `dual-write-sensitive-redaction`：safe summary/hash fields + redaction policy。
- `rollback-manifest-recovery-dry-run`：recovery dry-run only。

## FORBIDDEN SENSITIVE FIELD HANDLING

- 未写 prompt body、full transcript、secret/token/credential/keychain/OAuth/provider credential 或 rollout body。
- 敏感扫描命中仅为 redaction policy / 测试断言 / 合法 `plan_authorization(s)` 命名；无真实 Codex 执行命中、无 `/Users/yoyi/.codex` 访问命中。

## EVIDENCE / CHECKS RUN

- `node scripts/harness/workbench-shape-gate.js --mode check`：pass。
- `cargo test --lib sqlite_schema`：3 passed。
- `cargo test --lib sqlite_apply_importer`：6 passed。
- `cargo test --lib sqlite_export_dry_run`：3 passed。
- `cargo test --lib sqlite_dual_write`：10 passed。
- `cargo test --lib workflow_state`：11 passed。
- `cargo test --lib`：364 passed / 16 ignored。
- `cargo fmt -- --check`：pass。
- `git diff --check`：pass。
- `git status --short`：最终回交前仅含 R3-A3 范围文件。

## COMMITS / METRICS

- start commit：`c729ecab14df32076c5436d048aa7d4b69efdeea`
- end commit：未提交，待主管线回收。
- `lib.rs` before / after：13953 -> 13954 行。
- `workbench_sqlite_dual_write.rs`：574 行。
- Tauri commands：96 total / 0 in `lib.rs`。

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：本轮仅是 fixture-only rehearsal，不是生产双写期 / 生产迁移 / read-cutover。
- P2：R3-A3 fixtures 复用 R3-A2 legal core chain shape under new R3-A3 dirs；后续可继续扩展更丰富 payload。
- P2：rollback production workflow、SQLite production transaction boundary、read DB path 仍待后续 R3 task。

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
- 不要把本结果声明为 R3 SQLite 迁移完成、生产双写开始或多 agent 并行真实执行解锁。
