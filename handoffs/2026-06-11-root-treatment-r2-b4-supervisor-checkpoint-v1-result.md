# Root Treatment R2-B4 Supervisor Checkpoint v1 Result

日期：2026-06-11

## 结论

R2-B4 已回收为 `accepted_with_p2`。本轮接受为 workflow run check、work item state、session binding 和 legacy workflow node dispatch entrypoints 从 `lib.rs` 物理抽出完成；不接受为 R2 全部完成、R3 SQLite、Stage L 恢复或新的真实 Codex 执行授权。

## 主管复核结果

- HEAD 复核时为 `66a0cff5a4fb94101c1830a174dc908448ec8dba`。
- 本 supervisor checkpoint 提交：`1a9dcae521777f08e97ebe866a6d5563d1d902a8`
- `lib.rs`：24,635 lines -> 23,524 lines。
- `workflow_run_dispatch_entrypoints.rs`：新增 1,115 lines。
- Tauri command registry：96 total，`lib.rs` 内 `#[tauri::command]` 为 0。
- shape gate baseline / check 均通过。
- 聚焦测试和全量 `cargo test --lib` 均通过。

## 验证

已通过：

- `node scripts/harness/workbench-shape-gate.js --mode baseline`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `cargo test --lib workflow_run_check`
- `cargo test --lib workflow_node_dispatch`
- `cargo test --lib workflow_node_session_binding`
- `cargo test --lib work_item_state_update`
- `cargo test --lib task_package`
- `cargo test --lib`，336 passed / 16 ignored
- `cargo fmt -- --check`
- `git diff --check`
- `git status --short`

已知 warning：

- Rust 测试仍有既有 dead_code warning：`JsonRpcError::invalid_params` 未使用。

## 边界

- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript 或 rollout。
- 未启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 未迁移 SQLite，未改 workflow state schema，未新增 sidecar JSON 种类，未新增 Tauri command。

## P2

- `include!` 仍是保守过渡。
- R2-B4 不是 R2 水位线完成。
- 测试仍主要留在 `lib.rs` inline tests，后续 R2 后段再迁移。

## 下一步

R2-B5 建议执行 workflow read model / dispatch summary / readback stats extraction。它是行为不变治理任务，不启动 Stage L / K3-B1 / K3-B2，不执行真实 Codex，不读写 `/Users/yoyi/.codex`。
