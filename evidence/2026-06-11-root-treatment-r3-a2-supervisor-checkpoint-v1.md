# Root Treatment R3-A2 Supervisor Checkpoint v1

日期：2026-06-11

## STATUS

`accepted_with_p2`

全局主管已复核并回收 R3-A2：临时 DB apply importer、schema constraint / index hardening、transaction failure injection、DB -> JSON export dry-run 和 R3-A2 fixture 矩阵已完成并提交。

## Commits

- start commit：`556efb023601cb7f59b0cb44aad1d563e02bad5d`
- implementation commit：`ea982932cd3510487187e710991f20fb9d7467db`
- commit message：`chore: add r3 sqlite apply dry run`

## Accepted Scope

- `lib.rs` 只新增 `mod workbench_sqlite_apply;` / `mod workbench_sqlite_exporter;`。
- `workbench_sqlite_schema.rs` 增加 core CHECK / index hardening。
- `workbench_sqlite_importer.rs` 只放宽 R3-A1 dry-run 常量 / hash helper 到 `pub(crate)`，供 apply/export 复用。
- `workbench_sqlite_apply.rs` 新增显式 fixture root + temp DB path 的 apply importer，限制 temp / R3 fixture DB path，使用 SQLite transaction。
- `workbench_sqlite_exporter.rs` 新增内存 export dry-run manifest / projection hash / redaction manifest，不写 JSON / sidecar 文件。
- `src-tauri/fixtures/r3-a2/**` 新增 apply success、idempotent reapply、conflict rollback、revision conflict、corrupt primary、sensitive reject、crash injection、export dry-run 和 runtime log alias policy fixture。

## Fresh Verification

- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors / 0 warnings。
- `cargo test --lib sqlite_schema`：pass，3 passed。
- `cargo test --lib sqlite_importer_dry_run`：pass，6 passed。
- `cargo test --lib sqlite_apply_importer`：pass，6 passed。
- `cargo test --lib sqlite_export_dry_run`：pass，3 passed。
- `cargo test --lib workflow_state`：pass，11 passed。
- `cargo test --lib`：pass，354 passed / 0 failed / 16 ignored。
- `cargo fmt -- --check`：pass。
- `git diff --check` / `git diff --cached --check`：pass。
- 敏感 / 真实执行扫描：`Command::new("codex")`、`codex exec`、`codex exec resume`、`/Users/yoyi/.codex` 无 R3-A2 产品代码命中；命中项为 importer/exporter 拒绝 / redaction 清单、合法 `plan_authorization(s)` 命名和 R3-A2 forbidden fixture 的 `prompt_body` 测试字段。
- 提交后 `git status --short`：干净。

## Supervisor Fixup

主管线在回收时补充 `sqlite_apply_importer_rejects_before_db_begin_without_creating_db`，覆盖任务包要求的 `BeforeDbBegin` failure injection：begin 前失败不得创建 temp DB。

## Boundary Confirmation

- 未创建生产 DB。
- 未写用户真实数据目录。
- 未迁移真实 `workflow-state.v0.json` 或 sidecar。
- 未修改任何真实 JSON / sidecar。
- 未双写 DB + JSON。
- 未切任何产品读写路径到 DB。
- 未在 app startup / Tauri command / UI 中接入 DB initializer、apply importer 或 exporter。
- 未改 workflow state 顶层 schema。
- 未新增 sidecar store 或 sidecar JSON 种类。
- 未新增 Tauri command。
- 未改真实 Codex runner。
- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 未启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 未解冻 Stage L / K3-B1 / K3-B2 / backlog 功能。

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：R3-A2 只有 temp / fixture apply importer 和 export dry-run，不是生产 migration。
- P2：schema v0 仍可继续细化 FK / typed columns / table normalization。
- P2：Product Command / continuation / runtime log 的真实单事务产品写路径仍待后续 R3 task。
- P2：DB -> JSON export 仍不写盘；rollback/export production workflow 尚未实现。

## Next

建议进入 R3-A3：fixture-only dual-write transaction rehearsal。R3-A3 仍不得创建生产 DB、迁移真实 JSON / sidecar、切读写路径、执行真实 Codex 或访问 `/Users/yoyi/.codex`。
