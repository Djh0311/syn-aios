# Root Treatment R3-A7 Supervisor Checkpoint v1

日期：2026-06-11

## STATUS

`accepted_with_p2`

全局主管已复核并回收 R3-A7：production preflight scanner / report 模块、显式 config、硬拒绝 marker 合并、unknown JSON 阻断、sensitive body 不输出、backup readiness、sidecar readiness 和 temp fixture validation 已完成并提交。

R3-A7 implementation commit 后，复核线发现 2 个 P1：denied marker check 可能被 dotfile / directory ignore 绕过、`report_path` 可写到 `source_root` 内。主管线已修补：denied marker check 先于 ignore 执行，`.env` 文件和 `.codex` 目录会 blocked / rejected 且不输出 body，`report_path` 位于 `source_root` 内会被硬拒绝，并新增两个回归测试覆盖。

R3-A7 只接受为 scanner module + temp fixture validation；本轮未执行真实 production root scan。不接受为 R3 SQLite 迁移开始或完成、生产 DB、真实数据迁移、production apply、生产 read-cut、JSON / sidecar stop-write、rollback production workflow 或多 agent 并行真实执行解锁。

## Commits

- start commit：`15ad8411141cd17a1811e493f602e4377e19eef4`
- implementation commit：`7949253c91c8e688dc48e03c47a952f00fcd6fda`
- commit message：`chore: add r3 production preflight scanner`
- checkpoint / review-fix commit：本文随主管 checkpoint commit 提交；实际 hash 以 git log / 主管最终回交为准。

## Accepted Scope

- `lib.rs` 仅新增 `mod workbench_sqlite_preflight;`。
- 新增 `workbench_sqlite_preflight.rs`，834 行，低于 Rust 新文件 3,000 行上限。
- 新增 R3-A7 evidence / handoff。
- 任务包状态改为已完成并回填 implementation commit。
- 当前入口同步到：R3-A7 已完成，下一步准备 R3-A8 copied production snapshot temp DB apply and export verification。
- 未新增 Tauri command、startup hook、UI、sidecar store、sidecar JSON kind 或产品读写路径接入。
- 复核线 2 个 P1 已修补并加回归测试，不再阻断 R3-A7 checkpoint。

## Fresh Verification

- `cargo fmt`：pass。
- `cargo fmt -- --check`：pass。
- `cargo test --lib sqlite_preflight`：pass，8 passed。
- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors / 0 warnings；sidecar JSON kinds 14 allowed / 0 unknown；Tauri commands 96 total / 0 in `lib.rs`。
- `cargo test --lib sqlite_schema`：pass，3 passed。
- `cargo test --lib sqlite_observation`：pass，15 passed。
- `cargo test --lib workflow_state`：pass，11 passed。
- `cargo test --lib`：pass，399 passed / 16 ignored。
- `git diff --check`：pass。
- 入口旧口径扫描：`当前下一步是准备 R3-A7` / `准备 R3-A7` / `R3-A7 待执行` 无命中。

Known warning：

- Rust tests still show the existing `JsonRpcError::invalid_params` dead_code warning from `src/mcp/protocol.rs`; it is pre-existing and not introduced by R3-A7.

## Acceptance Review

R3-A7 requirements checked:

- Scanner is implemented as internal Rust module only.
- `lib.rs` only adds one module declaration.
- No Tauri command, startup hook, UI, app read path, production write path, production DB creation, production apply, read-cut or stop-write was added.
- Scanner accepts explicit config for primary workflow state, allowed sidecars and denied path markers.
- Default hard denied markers cannot be removed by custom config.
- Scanner outputs metadata / hash / schema / revision / top-level keys / count estimates only.
- Unknown JSON and denied names block without body output.
- `/Users/yoyi/.codex` root is denied.
- `.env` dotfile and `.codex` directory block before dotfile / directory ignore and do not output body.
- Report path inside source root is denied.
- Shape gate confirms no unknown sidecar JSON kind.
- Evidence / handoff do not claim real production root preflight or R3 completion.

## Review Line

Supervisor delegated a read-only A7 review to the reusable Stage R review line `019eb263-54a9-7081-9725-25057df15d1c` after implementation commit. The line reported 2 P1 findings, and the supervisor applied local fixes before this checkpoint:

- P1 fixed：denied marker check now runs before dotfile / directory ignore.
- P1 fixed：`report_path` under `source_root` is rejected.
- Added tests：`sqlite_preflight_denied_dotfile_and_directory_block_before_ignore` and `sqlite_preflight_denies_report_path_inside_source_root`.

The same review line rechecked the P1 fixes and returned: P0 none, P1 none, submission allowed. Its only P2 was stale checkpoint handoff verification counts, fixed in this checkpoint to `sqlite_preflight` 8 passed and `cargo test --lib` 399 passed / 16 ignored.

## Boundary Confirmation

- 未创建生产 DB。
- 未写用户真实数据目录。
- 未扫描真实 production root。
- 未迁移真实 `workflow-state.v0.json` 或 sidecar。
- 未修改任何真实 JSON / sidecar。
- 未切任何产品读写路径到 DB。
- 未让真实 app read model 读 DB。
- 未停止 JSON / sidecar 写入。
- 未把 JSON 降为 production fallback。
- 未在 app startup / Tauri command / UI 中接入 scanner。
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
- P1 review fixes：复核线提出的 2 个 P1 已修补，fresh verification 通过。
- P2：R3-A7 未执行真实 production root scan；后续若需要真实 production preflight，必须单独写 execution record / allowed root / report path。
- P2：R3-A8 不能跳过复制快照和临时 DB，不能直接 production apply / read-cut / stop-write。
- P2 resolved：复核线二次回交允许提交；其 stale verification count P2 已修补。

## Do Not Claim

- 不得声明 R3 SQLite 迁移开始或完成。
- 不得声明真实 production root preflight 已执行。
- 不得声明生产 DB 创建完成。
- 不得声明 production apply 已完成。
- 不得声明生产双写期开始。
- 不得声明生产读切 DB 完成。
- 不得声明 JSON / sidecar 停写。
- 不得声明 rollback production workflow 完成。
- 不得声明多 agent 并行真实执行解锁。
- 不得声明 Stage L / K3-B1 / K3-B2 已恢复。

## Next

建议进入 R3-A8 任务包准备：copied production snapshot temp DB apply and export verification。R3-A8 应先定义 snapshot source、copy destination、temp DB path、report path、hash manifest、rollback / cleanup boundary 和 denied paths；不得直接 production apply、read-cut 或 stop-write。
