# Root Treatment / R3-A12 Stop-Write JSON Decision And Rollback Drill v1 Result

日期：2026-06-11

## STATUS

`DONE_LEVEL_A`

R3-A12 Level A 已实现。Level B 未执行。

Implementation commit：`eacfad7c4a916f1307e633a37a6084a9fc2927e6`

## Summary

本轮新增 `workbench_sqlite_stop_write.rs`，把 stop-write JSON / sidecar 从“可能被误解为可以直接执行的动作”收敛为显式 supervisor decision contract 和 fixture / temp rollback drill。

核心结果：

- `prepare_only`：记录 `not_ready`，不真实 stop-write。
- `reject_stop_write`：记录 `rejected_by_supervisor`，不真实 stop-write。
- `approve_stop_write`：缺少 A9/A10/A11 Level B evidence 时 blocked，不写 completed report。
- fixture 模拟全部前置 evidence 齐全时，只输出 `ready_but_not_executed`，仍不真实 stop-write。
- safety flags 保持真实 stop-write、source JSON 写入、sidecar 写入、产品全局读写路径、startup、Tauri command、UI、production restore、Codex home touched 为 false。

`lib.rs` 只新增 module declaration；没有新增 Tauri command、startup hook、UI 或产品真实读写路径。

## Verification

通过：

- `node scripts/harness/workbench-shape-gate.js --mode check`：0 errors / 0 warnings
- `cargo test --lib sqlite_stop_write`：16 passed
- `cargo test --lib sqlite_observation`：24 passed
- `cargo test --lib sqlite_read_cut`：26 passed
- `cargo test --lib sqlite_production`：21 passed
- `cargo test --lib sqlite_export`：3 passed
- `cargo test --lib sqlite_apply`：6 passed
- `cargo test --lib workflow_state`：11 passed
- `cargo test --lib`：463 passed / 16 ignored
- `cargo fmt -- --check`
- `git diff --check`

仅保留既有 warning：`JsonRpcError::invalid_params` unused。

## Boundary

未执行真实 Codex，未发送 prompt，未读写 `/Users/yoyi/.codex`，未读取 secret/token/full transcript/rollout，未启动 Tauri/Browser/Chrome/Vite/screenshot，未读取真实 workbench state root，未创建真实 production DB，未切产品读写路径，未停写 JSON / sidecar，未新增 Tauri command、UI 或 startup hook。

## Do Not Claim

不能声明 JSON / sidecar stop-write 完成、production read-cut 完成、app 真实 SQLite 读写路径已启用、production observation Level B 完成、rollback production workflow 完成、R3 完成或多 agent 并行真实执行解锁。

## Review Result

复核线结论：`CLEAR_WITH_P2`。

P0：无。

P1：无。

P2：

- `verify_rollback_manifest` 当前只以 `status=completed` 且 `production_restore_performed=false` 判定 complete；Level A fixture / temp 可接受。Level B 前建议补 schema/version、rollback boundary 字段完整性、dry-run/source/projection/decision 绑定校验。
- denied marker 主要校验传入 path；当前 R3-A12 fixture 干净且无真实入口，非提交阻断。后续若扩大 caller，建议对子文件名也套 denied marker，避免 `.env`/secret/token 文件被 hash。

复核线已确认：

- A12 是否仍只是 Level A fixture / temp。
- 是否没有新增 Tauri command、startup hook、UI 或真实 product read/write path。
- approve stop-write 在缺少真实 Level B evidence 时是否 blocked。
- ready_but_not_executed 是否仍不真实 stop-write。
- rollback drill 是否没有执行 restore。
- safety flags 是否没有越界 true。
- source hashes 是否证明 source JSON / sidecar 未改。
- evidence / handoff 是否没有夸大为 stop-write、production read-cut、rollback production workflow 或 R3 完成。
- 是否没有 `.codex`、secret、token、full transcript、rollout 越界。

复核线建议主管线提交。

## Next

R3-A12 checkpoint 具备复核条件；复核 clear 后可提交 implementation commit 并同步当前入口。

下一步不要直接真实 stop-write。建议进入 R3-A13 final acceptance / cutover gap matrix，或先单独决策是否需要 A9/A10/A11/A12 Level B。
