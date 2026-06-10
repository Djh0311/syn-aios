# Root Treatment R3-A3 Supervisor Checkpoint v1

日期：2026-06-11

## STATUS

`accepted_with_p2`

全局主管已复核并回收 R3-A3：fixture-only dual-write transaction rehearsal、DB export dry-run projection 写盘、rollback manifest、failure injection、projection cleanup 和 recovery dry-run 已完成并提交。

R3-A3 只接受为临时 DB + 临时 JSON projection root + R3-A3 fixture root 内的演练完成；不接受为生产双写期开始、生产 DB 创建、真实 JSON / sidecar 迁移、读切 DB、JSON / sidecar 停写、rollback production workflow 或多 agent 并行真实执行解锁。

## Commits

- start commit：`c729ecab14df32076c5436d048aa7d4b69efdeea`
- implementation commit：`d9e5f0fd637daf7cbb6b117d7a8bac15448c9d8f`
- commit message：`chore: add r3 sqlite dual write rehearsal`

## Accepted Scope

- `lib.rs` 仅新增 `mod workbench_sqlite_dual_write;`。
- 新增 `workbench_sqlite_dual_write.rs`，574 行，低于 Rust 新文件 3,000 行上限。
- 新增 `src-tauri/fixtures/r3-a3/**`，8 组 fixture、56 个 JSON 输入文件。
- 新增开发线 evidence / handoff。
- 未新增 Tauri command、startup hook、UI、sidecar store 或产品读写路径接入。

## Fresh Verification

- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors / 0 warnings；`lib.rs` 13954 行；Tauri commands 96 total / 0 in `lib.rs`；sidecar JSON kinds 14 allowed / 0 unknown。
- `cargo test --lib sqlite_schema`：pass，3 passed。
- `cargo test --lib sqlite_apply_importer`：pass，6 passed。
- `cargo test --lib sqlite_export_dry_run`：pass，3 passed。
- `cargo test --lib sqlite_dual_write`：pass，10 passed。
- `cargo test --lib workflow_state`：pass，11 passed。
- `cargo test --lib`：pass，364 passed / 16 ignored。
- `cargo fmt -- --check`：pass。
- `git diff --check`：pass。
- `git diff --cached --check`：pass after removing two EOF blank lines in R3-A3 evidence / handoff.
- Sensitive / real-exec scan：no R3-A3 real Codex execution or `.codex` access hit; matches are redaction policy / test assertions / legal `plan_authorization(s)` names.
- Sidecar / projection scan：only allowed workflow / memory / runtime / product / continuation names in R3-A3 fixtures and exporter / dual-write code.
- Post-commit `git status --short`：clean.

Known warning:

- Rust tests still show the existing `JsonRpcError::invalid_params` dead_code warning; it is pre-existing and not introduced by R3-A3.

## Acceptance Review

R3-A3 requirements checked:

- Projection files are produced from DB export dry-run, not copied from source fixture at runtime.
- Writes are restricted by explicit path guards to temp paths or R3-A3 fixture paths.
- Failure injection covers before DB apply, after DB apply before projection, after first projection file before manifest, before manifest commit, and after manifest commit.
- Pre-manifest failure does not create completed rollback manifest.
- DB-committed / projection-failed case is explicit and does not pretend DB rollback happened.
- Recovery stays dry-run only and does not restore production JSON.
- Prompt body / full transcript / rollout body / provider credential body are omitted from projection and manifest policy.

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

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：R3-A3 仍是 fixture-only rehearsal，不是生产双写期、生产 migration、read-cutover 或 rollback production workflow。
- P2：A3 fixture payload 复用 R3-A2 legal core-chain shape，后续可增加更丰富 domain-specific payload。
- P2：R3 合同里的 A4 方向需要在下一任务包先冻结：不能从 A3 fixture-only 直接冒进为生产读切。
- P2：SQLite schema / importer 仍是 v0 prep；FK / typed columns / production transaction boundary 仍待后续 R3 task。

## Do Not Claim

- 不得声明 R3 SQLite 迁移开始或完成。
- 不得声明生产 DB 创建完成。
- 不得声明生产双写期开始。
- 不得声明读切 DB 完成。
- 不得声明 JSON / sidecar 停写。
- 不得声明 rollback production workflow 完成。
- 不得声明多 agent 并行真实执行解锁。
- 不得声明 Stage L / K3-B1 / K3-B2 已恢复。

## Next

建议进入 R3-A4 任务包准备 / 合同冻结：在正式进入产品路径前，先明确 A4 是生产双写前置、读切 DB rehearsal，还是两段拆分；必须写清 production path、备份、rollback、read fallback、JSON export 和 no-real-Codex 边界。R3-A4 未冻结前，不得开始生产 DB、生产双写或读切。
