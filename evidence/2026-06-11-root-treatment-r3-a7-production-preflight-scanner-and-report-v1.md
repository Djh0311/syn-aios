# Root Treatment R3-A7 Production Preflight Scanner And Report v1

日期：2026-06-11

## STATUS

`DONE_WITH_CONCERNS`

R3-A7 已实现 production preflight scanner / report 的只读模块和 fixture 验证。Scanner 默认只扫描显式传入的 state root，输出 metadata / hash / schema / revision / top-level keys / record count estimate / backup readiness / sidecar readiness；不会创建 production DB，不写 production root，不迁移 JSON / sidecar，不接 Tauri command、startup hook、UI 或产品读写路径。

本轮未执行真实 production root scan；只使用 Rust 单测创建的 temp fixture roots。

## READ / WRITE SCOPE

### 读取

- 当前任务包：`tasks/2026-06-11-root-treatment-r3-a7-production-preflight-scanner-and-report-v1.md`。
- R3-A6 contract evidence / handoff。
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`。
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`。
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_importer.rs`。
- `scripts/harness/workbench-shape-gate.js`。

### 写入

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`：仅新增 `mod workbench_sqlite_preflight;`。
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_preflight.rs`。
- 本 evidence。
- `handoffs/2026-06-11-root-treatment-r3-a7-production-preflight-scanner-and-report-v1-result.md`。
- `tasks/2026-06-11-root-treatment-r3-a7-production-preflight-scanner-and-report-v1.md` 状态。

## SCANNER SUMMARY

新增模块：

- `workbench_sqlite_preflight.rs`，763 行，低于 Rust 新文件 3,000 行上限。

核心入口：

- `scan_workbench_state_root_preflight(source_root, report_path)`：默认配置入口。
- `scan_workbench_state_root_preflight_with_config(source_root, report_path, config)`：显式配置入口。
- `SqliteProductionPreflightConfig`：显式接收 `primary_workflow_state`、`allowed_sidecars`、`denied_path_markers`。

输出报告：

- `source_root_ref`、`source_root_hash`。
- per-file `path_ref`、`path_hash`、`file_hash`、`size_bytes`。
- per-file `schema_version`、`revision`、`top_level_keys`、`record_count_estimate`。
- `backup_readiness`：backups dir、workflow state backup count、latest backup ref/hash/timestamp。
- `sidecar_readiness`：allowed sidecar present / missing optional / canonical。
- `production_db_created=false`。
- `production_root_written=false`。
- `read_cut_enabled=false`。
- `stop_write_json=false`。
- `codex_home_touched=false`。

边界实现：

- 默认硬拒绝 markers 包含 `/Users/yoyi/.codex`、`.codex`、`.env`、token、secret、credential、keychain、oauth、provider credential、full transcript、rollout。
- 自定义 config 不能移除硬拒绝 markers；代码会把默认硬拒绝项和调用方 markers 合并。
- 未知 JSON 被标记 blocked，不读取 body 输出，`file_hash=None`。
- denied path / denied file name 被 rejected，不读取 body 输出。
- forbidden top-level key 被 rejected，只输出 metadata / top-level key，不输出 body。
- non-JSON 文件被 rejected；`.tmp` / `.lock` / dotfile 被忽略为支持文件。

## FIXTURE COVERAGE

单测覆盖：

- valid root with workflow state + canonical runtime log + backup + report path。
- missing optional sidecars are warnings, not blockers。
- unknown JSON blocks without body output。
- denied sensitive file name blocks without reading body。
- `/Users/yoyi/.codex` source root denied。
- explicit sidecar list + explicit denied markers：允许自定义 allowed sidecar，同时自定义 forbidden key 仍阻断且不输出 body。

## REAL PRODUCTION PREFLIGHT STATUS

真实 production root scan：未执行。

原因：

- R3-A7 默认任务目标是实现 scanner module 和 temp fixture 验证。
- 本轮未显式声明真实 production root allowed path、report path 或 execution record。
- 未读取工作台真实 state root 内容。

后续如要执行真实 production preflight，必须单独记录 allowed root、report path、读取字段、不读取项、source root hash，并确认 report 不含 forbidden body。

## CHECKS RUN

- `cargo fmt`：pass。
- `cargo fmt -- --check`：pass。
- `cargo test --lib sqlite_preflight`：pass，6 passed。
- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors / 0 warnings；sidecar JSON kinds 14 allowed / 0 unknown；Tauri commands 96 total / 0 in `lib.rs`。
- `cargo test --lib sqlite_schema`：pass，3 passed。
- `cargo test --lib sqlite_observation`：pass，15 passed。
- `cargo test --lib workflow_state`：pass，11 passed。
- `cargo test --lib`：pass，397 passed / 16 ignored。
- `git diff --check`：pass。

Known warning：

- Rust tests still show existing `JsonRpcError::invalid_params` dead_code warning from `src/mcp/protocol.rs`; not introduced by R3-A7。

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：R3-A7 只是 scanner module + temp fixture validation，不是 production root scan。
- P2：Scanner 目前没有 Tauri command / CLI command / UI；这是本任务的安全边界，不是缺陷。
- P2：后续 R3-A8 copied production snapshot temp DB apply 必须先消费 A7 report 或重新运行显式 production preflight，不能跳过。

## BOUNDARY CONFIRMATION

- 未创建生产 DB。
- 未写用户真实数据目录。
- 未扫描真实 production root。
- 未迁移真实 `workflow-state.v0.json` 或 sidecar。
- 未修改任何真实 JSON / sidecar。
- 未切任何产品读写路径到 DB。
- 未让真实 app read model 读 DB。
- 未停止 JSON / sidecar 写入。
- 未把 JSON 降为生产 fallback。
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

## DO NOT CLAIM

- 不声明 R3 SQLite 迁移开始或完成。
- 不声明真实 production root preflight 已执行。
- 不声明生产 DB 创建完成。
- 不声明 production apply 已完成。
- 不声明生产双写期开始。
- 不声明生产读切 DB 完成。
- 不声明 JSON / sidecar 停写。
- 不声明 rollback production workflow 完成。
- 不声明多 agent 并行真实执行解锁。
- 不声明 Stage L / K3-B1 / K3-B2 已恢复。
