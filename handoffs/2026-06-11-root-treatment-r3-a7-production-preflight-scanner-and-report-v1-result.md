# Root Treatment R3-A7 Production Preflight Scanner And Report v1 Result

日期：2026-06-11

## STATUS

`DONE_WITH_CONCERNS`

R3-A7 production preflight scanner / report 已实现并通过 temp fixture 验证，implementation commit 为 `7949253c91c8e688dc48e03c47a952f00fcd6fda`；主管 checkpoint 已回收，复核线提出的 2 个 P1 已修补并加回归测试。

## IMPLEMENTED

- 新增 `workbench_sqlite_preflight.rs`，834 行。
- implementation commit：`7949253c91c8e688dc48e03c47a952f00fcd6fda`。
- `lib.rs` 仅新增 `mod workbench_sqlite_preflight;`。
- 默认 scanner 入口：`scan_workbench_state_root_preflight`。
- 显式 config scanner 入口：`scan_workbench_state_root_preflight_with_config`。
- Scanner 输出 metadata / hash / schema / revision / top-level keys / record count estimate / backup readiness / sidecar readiness。
- Scanner flags 固定保持 `production_db_created=false`、`production_root_written=false`、`read_cut_enabled=false`、`stop_write_json=false`、`codex_home_touched=false`。
- 自定义 config 不能移除默认硬拒绝 markers。
- denied marker check 先于 dotfile / directory ignore；`.env` 文件和 `.codex` 目录会 blocked / rejected 且不输出 body。
- `report_path` 位于 `source_root` 内会被硬拒绝。

## VERIFIED

- `cargo fmt`：pass。
- `cargo fmt -- --check`：pass。
- `cargo test --lib sqlite_preflight`：8 passed。
- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors / 0 warnings。
- `cargo test --lib sqlite_schema`：3 passed。
- `cargo test --lib sqlite_observation`：15 passed。
- `cargo test --lib workflow_state`：11 passed。
- `cargo test --lib`：399 passed / 16 ignored。
- `git diff --check`：pass。

Review fixes verified：

- `sqlite_preflight_denied_dotfile_and_directory_block_before_ignore` 覆盖 sensitive dotfile / directory 先于 ignore 被 blocked。
- `sqlite_preflight_denies_report_path_inside_source_root` 覆盖 report path inside source root 被拒绝。

Known warning：既有 `JsonRpcError::invalid_params` dead_code warning；非 R3-A7 引入。

## REAL PRODUCTION PREFLIGHT

未执行。

本轮只做 scanner module 和 temp fixture tests；没有声明 allowed production root，没有写 report path execution record，没有读取真实 state root。

## BOUNDARY CONFIRMATION

- 未创建 production DB。
- 未写 production root。
- 未扫描真实 production root。
- 未迁移或修改真实 JSON / sidecar。
- 未切产品读写路径到 DB。
- 未停写 JSON / sidecar。
- 未新增 Tauri command、startup hook、UI 或 sidecar kind。
- 未执行真实 `codex exec` / `codex exec resume`，未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 未启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 未启动 Stage L / K3-B1 / K3-B2，未解冻 backlog 功能。

## NEXT

建议主管 checkpoint 后进入 R3-A8：copied production snapshot temp DB apply and export verification。

R3-A8 仍不得直接 production apply / read-cut / stop-write；必须先使用复制快照和 temp DB 验证 importer / apply / export / rollback 边界。

## DO NOT CLAIM

- 不声明 R3 SQLite 迁移开始或完成。
- 不声明真实 production root preflight 已执行。
- 不声明生产 DB 创建完成。
- 不声明 production apply 已完成。
- 不声明生产读切 DB 完成。
- 不声明 JSON / sidecar 停写。
- 不声明 rollback production workflow 完成。
- 不声明多 agent 并行真实执行解锁。
- 不声明 Stage L / K3-B1 / K3-B2 已恢复。
