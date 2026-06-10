# Root Treatment R2-B4 Supervisor Checkpoint v1

日期：2026-06-11

## 结论

R2-B4 已由主管线回收为 `accepted_with_p2`。

接受范围：

- 任务包点名的 workflow run check、work item state、session binding 和 legacy workflow node dispatch entrypoints 已从 `src-tauri/src/lib.rs` 抽出到 `src-tauri/src/workflow_run_dispatch_entrypoints.rs`。
- `lib.rs` 通过 `include!("workflow_run_dispatch_entrypoints.rs")` 在 crate root 展开 helper，保持函数可见性和行为不变。
- `lib.rs` 从 24,635 行降到 23,524 行，低于 R0 水位线。
- 新 helper 文件为 1,115 行，低于 Rust 3,000 行治理阈值。
- command registry 总量保持 96，`lib.rs` 内 `#[tauri::command]` 保持 0。

不接受范围：

- 不接受为 R2 全部完成。
- 不接受为 `lib.rs <= 15,000` 或 `lib.rs <= 3,000` 目标完成。
- 不接受为 workflow read model、C4-C6 自动化、memory、runtime diagnostics、SQLite 迁移或 R3 完成。
- 不接受为 Stage L / K3-B1 / K3-B2 恢复。
- 不接受为新的真实 Codex 执行授权。

## Commit 记录

- R2-B4 start commit：`83b9219e464d51549ae470bde480fb7e81cff19b`
- R2-B4 completion commit：`66a0cff5a4fb94101c1830a174dc908448ec8dba`
- 本 supervisor checkpoint 提交：`1a9dcae521777f08e97ebe866a6d5563d1d902a8`

## 复核文件

- `tasks/2026-06-11-root-treatment-r2-b4-workflow-run-binding-and-legacy-dispatch-entrypoints-extraction-v1.md`
- `evidence/2026-06-11-root-treatment-r2-b4-workflow-run-binding-and-legacy-dispatch-entrypoints-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r2-b4-workflow-run-binding-and-legacy-dispatch-entrypoints-extraction-v1-result.md`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_run_dispatch_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`

## 主管 Fresh Verify

已重新运行：

```bash
node scripts/harness/workbench-shape-gate.js --mode baseline
node scripts/harness/workbench-shape-gate.js --mode check
cargo test --lib workflow_run_check
cargo test --lib workflow_node_dispatch
cargo test --lib workflow_node_session_binding
cargo test --lib work_item_state_update
cargo test --lib task_package
cargo test --lib
cargo fmt -- --check
git diff --check
git status --short
```

结果：

- shape gate baseline：通过，0 errors / 0 warnings；`lib.rs` 23,524 lines；Tauri commands 96 total / 0 in `lib.rs`；Sidecar JSON kinds 14 detected / 0 unknown。
- shape gate check：通过，0 errors / 0 warnings；`lib.rs` ratchet `23,524 / 25,925`，status `decreased`。
- `cargo test --lib workflow_run_check`：通过，2 passed / 0 failed / 350 filtered out；保留既有 `JsonRpcError::invalid_params` dead_code warning。
- `cargo test --lib workflow_node_dispatch`：通过，11 passed / 0 failed / 341 filtered out；保留同一既有 warning。
- `cargo test --lib workflow_node_session_binding`：通过，2 passed / 0 failed / 350 filtered out；保留同一既有 warning。
- `cargo test --lib work_item_state_update`：通过，3 passed / 0 failed / 349 filtered out；保留同一既有 warning。
- `cargo test --lib task_package`：通过，29 passed / 0 failed / 1 ignored / 322 filtered out；保留同一既有 warning。
- `cargo test --lib`：通过，336 passed / 0 failed / 16 ignored；保留同一既有 warning。
- `cargo fmt -- --check`：通过。
- `git diff --check`：通过，无输出。
- `git status --short`：通过，无输出。

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：R2-B4 仍使用 `include!` 作为保守过渡，后续 R2 可以继续收敛为正式模块边界。
- P2：R2-B4 只抽出 workflow run / binding / legacy dispatch 入口，不代表 workflow read model、C4-C6 自动化、storage migration、memory / runtime diagnostics 拆分或 R2 水位线目标完成。
- P2：dispatch / binding 相关测试仍主要保留在 `lib.rs` inline tests 中，后续 R2 后段可按领域迁移 tests。

## 边界确认

- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript 或 rollout。
- 未启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 未新增 sidecar store 或 sidecar JSON 种类。
- 未迁移 SQLite。
- 未改 workflow state 顶层 schema。
- 未新增 Tauri command。
- 未改真实 Codex runner。
- 未启动 Stage L / K3-B1 retry / K3-B2。
- 未解冻 backlog 功能。

## 下一步

创建并执行 R2-B5。R2-B5 应限定为 workflow read model / dispatch summary / readback stats 的物理抽出和既有 `workflow_read_model.rs` 边界收敛；不顺手做 R3 SQLite、R4 UI/按页读模型、Stage L/K3 恢复或真实 Codex 执行。
