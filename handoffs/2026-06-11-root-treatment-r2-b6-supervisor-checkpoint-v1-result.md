# Root Treatment R2-B6 Supervisor Checkpoint v1 Result

日期：2026-06-11

## 结论

R2-B6 已回收为 `accepted_with_p2`。本轮接受为 workflow dispatch execution control、offline role dispatch、workflow machine run loop 和相邻 execution result helper 从 `lib.rs` 物理抽出完成；不接受为 R2 全部完成、R3 SQLite、Stage L 恢复或新的真实 Codex 执行授权。

## 主管复核结果

- HEAD 复核时为 `2dd766be84e977d75e77f31ec2dbf9d463f45690`。
- 本 supervisor checkpoint 提交：`7e77fffe8339d553cfa4fcac3f09f503da43f8d5`。
- `lib.rs`：21,463 lines -> 19,401 lines。
- `workflow_execution_entrypoints.rs`：新增 2,068 lines。
- Tauri command registry：96 total，`lib.rs` 内 `#[tauri::command]` 为 0。
- shape gate baseline / check 均通过。
- 聚焦测试和全量 `cargo test --lib` 均通过。

## 验证

已通过：

- `node scripts/harness/workbench-shape-gate.js --mode baseline`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `cargo test --lib workflow_dispatch`
- `cargo test --lib workflow_node_dispatch`
- `cargo test --lib offline_role`
- `cargo test --lib workflow_machine`
- `cargo test --lib workflow_permission`
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
- R2-B6 不是 R2 水位线完成。
- 测试仍主要留在 `lib.rs` inline tests，后续 R2 后段再迁移。

## 下一步

R2-B7 建议执行 memory domain extraction。它是行为不变治理任务，不启动 Stage L / K3-B1 / K3-B2，不执行真实 Codex，不读写 `/Users/yoyi/.codex`。
