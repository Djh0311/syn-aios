# Root Treatment R2-B4 Workflow Run Binding And Legacy Dispatch Entrypoints Extraction v1 Result

日期：2026-06-11

## 结论

R2-B4 本轮可接受为：任务包点名的 workflow run check、work item state、session binding 和 legacy workflow node dispatch entrypoints 已从 `lib.rs` 物理抽出到 `workflow_run_dispatch_entrypoints.rs`，并通过必需验证。行为保持不变，command 总量仍为 96，`lib.rs` 内 `#[tauri::command]` 仍为 0。

不接受为：R2 全部完成、`lib.rs <= 15,000` 水位线完成、workflow read model / C4-C6 自动化 / memory / runtime diagnostics / tests 巨石拆分完成、SQLite 迁移完成、workflow state schema 迁移或真实 Codex 执行恢复。

## 已完成

- 新增 `prototypes/productized-desktop-shell/src-tauri/src/workflow_run_dispatch_entrypoints.rs`。
- 将 13 个 workflow run / binding / legacy dispatch helper 函数和 1 个 context 类型从 `lib.rs` 搬到新 helper 文件。
- `lib.rs` 原位置改为 `include!("workflow_run_dispatch_entrypoints.rs")`。
- 新增 R2-B4 evidence：`evidence/2026-06-11-root-treatment-r2-b4-workflow-run-binding-and-legacy-dispatch-entrypoints-extraction-v1.md`。
- 未同步入口文档。

## 改动文件

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_run_dispatch_entrypoints.rs`
- `evidence/2026-06-11-root-treatment-r2-b4-workflow-run-binding-and-legacy-dispatch-entrypoints-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r2-b4-workflow-run-binding-and-legacy-dispatch-entrypoints-extraction-v1-result.md`

## 抽出函数 / 类型

- `inspect_workflow_run_check_at`
- `inspect_workflow_run_check_from_value`
- `blocked_workflow_run_check`
- `push_check`
- `find_task_package_artifact_for_work_item`
- `update_work_item_state_at`
- `bind_workflow_node_codex_session_at`
- `unbind_workflow_node_codex_session_at`
- `prepare_workflow_node_dispatch_at`
- `execute_workflow_node_dispatch_at`
- `read_workflow_node_dispatch_result_at`
- `WorkflowNodeDispatchContext`
- `workflow_node_dispatch_context`
- `inspect_task_package_authorization_at`

## Shape 指标

- `lib.rs`：24,635 lines -> 23,524 lines。
- `workflow_run_dispatch_entrypoints.rs`：新增 1,115 lines。
- Tauri command registry：96 total -> 96 total。
- `lib.rs` 内 `#[tauri::command]`：0 -> 0。
- Sidecar JSON kinds：14 detected / 0 unknown。

## 验证

已通过：

- `node scripts/harness/workbench-shape-gate.js --mode baseline`：pass，0 errors / 0 warnings；`lib.rs` 23,524 / 25,925，status `decreased`；Tauri commands 96 total / 0 in `lib.rs`。
- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors / 0 warnings；`lib.rs` 23,524 / 25,925，status `decreased`；Tauri commands 96 total / 0 in `lib.rs`。
- `cargo test --lib workflow_run_check`：2 passed / 0 failed / 350 filtered out。
- `cargo test --lib workflow_node_dispatch`：11 passed / 0 failed / 341 filtered out。
- `cargo test --lib workflow_node_session_binding`：2 passed / 0 failed / 350 filtered out。
- `cargo test --lib work_item_state_update`：3 passed / 0 failed / 349 filtered out。
- `cargo test --lib task_package`：29 passed / 0 failed / 1 ignored / 322 filtered out。
- `cargo test --lib`：336 passed / 0 failed / 16 ignored。
- `cargo fmt -- --check`：通过。
- `git diff --check`：通过。
- `git status --short`：提交前仅包含 R2-B4 范围文件。

已知 warning：

- Rust 测试仍有既有 dead_code warning：`JsonRpcError::invalid_params` 未使用。

## Commit 记录

- Start commit：`83b9219e464d51549ae470bde480fb7e81cff19b`
- End commit：`66a0cff5a4fb94101c1830a174dc908448ec8dba`

## 边界

- 未同步 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`。
- 未改产品业务逻辑。
- 未改函数语义、返回值、错误文案、公开 Tauri command 契约或 workflow state schema。
- 未新增 Tauri command。
- 未新增 sidecar / sidecar JSON kind。
- 未迁移 SQLite。
- 未做 UI。
- 未做 C4-C6 自动化 / workflow read model / memory / runtime diagnostics / tests 巨石其他拆分。
- 未执行真实 Codex。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript 或 rollout。
- 未启动 Tauri / Browser / Chrome / Vite / 截图工具。

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：`include!` 是保守过渡，后续 R2 可再收敛为正式模块边界。
- P2：本轮只抽出 workflow run / binding / legacy dispatch entrypoints，不代表 R2 水位线、workflow read model、C4-C6 自动化、storage migration、memory / runtime diagnostics 或 tests 巨石拆分完成。
- P2：dispatch / binding 相关测试仍主要留在 `lib.rs` inline tests 中，后续 R2 后段再迁移 tests。
