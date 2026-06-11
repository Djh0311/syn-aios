# Root Treatment R3-A5 Supervisor Checkpoint v1

日期：2026-06-11

## STATUS

`accepted_with_p2`

全局主管已复核并回收 R3-A5：fixture-only observation period、DB -> JSON export verification、two-sample stability、rollback recovery verification dry-run、degraded / blocked failure injection 和 sensitive redaction 覆盖已完成并提交。

R3-A5 只接受为临时 DB + 临时 JSON projection root + R3-A5 fixture root 内的 observation / export / rollback verification rehearsal；不接受为生产 DB、生产 read-cut、生产 rollback workflow、真实 JSON / sidecar 迁移、JSON / sidecar stop-write、产品读写路径切 DB 或多 agent 并行真实执行解锁。

## Commits

- start commit：`6a9b5b7433f2bd50fc80e1a37d081a87822dde6b`
- implementation commit：`0e8255a8248601caf7b1d513131f43e4bb157589`
- commit message：`chore: add r3 sqlite observation rehearsal`
- checkpoint commit：本文随主管 checkpoint commit 提交；实际 hash 以 git log / 主管最终回交为准。

## Accepted Scope

- `lib.rs` 仅新增 `mod workbench_sqlite_observation_period;`。
- 新增 `workbench_sqlite_observation_period.rs`，1047 行，低于 Rust 新文件 3,000 行上限。
- 新增 `src-tauri/fixtures/r3-a5/**`，9 组 fixture、63 个 JSON 输入文件。
- 新增开发线 evidence / handoff。
- 未新增 Tauri command、startup hook、UI、sidecar store、sidecar JSON kind 或产品读写路径接入。

## Fresh Verification

- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors / 0 warnings；HEAD `0e8255a8248601caf7b1d513131f43e4bb157589`；`lib.rs` 13956 行；Tauri commands 96 total / 0 in `lib.rs`；sidecar JSON kinds 14 allowed / 0 unknown。
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

Known warning:

- Rust tests still show the existing `JsonRpcError::invalid_params` dead_code warning from `src/mcp/protocol.rs`; it is pre-existing and not introduced by R3-A5.

## Acceptance Review

R3-A5 requirements checked:

- Observation success is derived from temp DB apply, DB export dry-run, projection hash verification and two-sample stability, not from direct source fixture copy.
- Writes are restricted to temp paths or R3-A5 fixture paths.
- Stable observation report is only committed when `observation_status=stable_verified`, `stable_verified=true` and `degraded=false`.
- Export verification records source root hash, DB export hash, projection hash, export manifest hash, per-file hash / record count / redaction status and canonical runtime log alias policy.
- Projection writes emit canonical `runtime-logs.v1.json` and reject / omit legacy singular `runtime-log.v1.json`.
- Rollback verification remains dry-run only and records production restore as not performed.
- Failure injection covers before sample, after first export, export hash mismatch, projection missing/corrupt, manifest missing/incomplete, DB integrity/schema mismatch, observation drift and after rollback selected before report commit.
- Prompt body / full transcript / rollout body / provider credential body are omitted from report, projection and manifest policy.

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

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：R3-A5 仍是 fixture-only rehearsal，不是生产 DB、production read-cut、JSON / sidecar stop-write 或 rollback production workflow。
- P2：A5 fixtures 复用 R3-A4 legal payload shape under new R3-A5 dirs；后续可增加更丰富 domain-specific payload。
- P2：生产 DB 创建、生产 transaction boundary、真实 read path、JSON stop-write、rollback production workflow 和多 agent 并行真实执行解锁仍待后续 R3 task。

## Do Not Claim

- 不得声明 R3 SQLite 迁移开始或完成。
- 不得声明生产 DB 创建完成。
- 不得声明生产双写期开始。
- 不得声明生产读切 DB 完成。
- 不得声明 JSON / sidecar 停写。
- 不得声明 rollback production workflow 完成。
- 不得声明多 agent 并行真实执行解锁。
- 不得声明 Stage L / K3-B1 / K3-B2 已恢复。

## Next

建议进入 R3-A6 任务包准备：先冻结生产路径前置门槛 / cutover contract / rollback operator contract，明确哪些生产路径可以开始接 SQLite、哪些仍必须停留在 fixture-only；不得从 R3-A5 直接跳到生产 DB read-cut、JSON / sidecar stop-write 或多 agent 并行真实执行解锁。
