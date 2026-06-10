# Root Treatment R2-B3 Supervisor Checkpoint v1

日期：2026-06-11

## 结论

R2-B3 已由主管线回收为 `accepted_with_p2`。

接受范围：

- 任务包点名的 workflow state 生命周期入口和 task package 写入链已从 `src-tauri/src/lib.rs` 抽出到 `src-tauri/src/workflow_state_lifecycle_task_package.rs`。
- `lib.rs` 通过 `include!("workflow_state_lifecycle_task_package.rs")` 在 crate root 展开 helper，保持函数可见性和行为不变。
- `lib.rs` 从 25,643 行降到 24,635 行，低于 R0 水位线。
- 新 helper 文件为 1,012 行，低于 Rust 3,000 行治理阈值。
- command registry 总量保持 96，`lib.rs` 内 `#[tauri::command]` 保持 0。

不接受范围：

- 不接受为 R2 全部完成。
- 不接受为 `lib.rs <= 15,000` 或 `lib.rs <= 3,000` 目标完成。
- 不接受为 workflow read model、memory、runtime / diagnostics、SQLite 迁移或 R3 完成。
- 不接受为 Stage L / K3-B1 / K3-B2 恢复。
- 不接受为新的真实 Codex 执行授权。

## Commit 记录

- R2-B3 start commit：`446c1832b3d4e63d7bed5667814d47919277d342`
- R2-B3 completion commit：`208fabaa4cae8aeda45cdce4c66cbe7f2cf8e6c3`
- 本 supervisor checkpoint 提交：`87b04ec55c8bdc57d20f1e81ab96f0f2eeccda8d`

## 复核文件

- `tasks/2026-06-11-root-treatment-r2-b3-workflow-state-lifecycle-and-task-package-chain-extraction-v1.md`
- `evidence/2026-06-11-root-treatment-r2-b3-workflow-state-lifecycle-and-task-package-chain-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r2-b3-workflow-state-lifecycle-and-task-package-chain-extraction-v1-result.md`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_lifecycle_task_package.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`

## 主管 Fresh Verify

已重新运行：

```bash
node scripts/harness/workbench-shape-gate.js --mode baseline
node scripts/harness/workbench-shape-gate.js --mode check
cargo test --lib workflow_state
cargo test --lib task_package
cargo test --lib workflow_run_check
cargo test --lib
cargo fmt -- --check
git diff --check
git status --short
```

结果：

- shape gate baseline：通过，0 errors / 0 warnings；`lib.rs` 24,635 lines；Tauri commands 96 total / 0 in `lib.rs`；Sidecar JSON kinds 14 detected / 0 unknown。
- shape gate check：通过，0 errors / 0 warnings；`lib.rs` ratchet `24,635 / 25,925`，status `decreased`。
- `cargo test --lib workflow_state`：通过，11 passed / 0 failed / 341 filtered out；保留既有 `JsonRpcError::invalid_params` dead_code warning。
- `cargo test --lib task_package`：通过，29 passed / 0 failed / 1 ignored / 322 filtered out；保留同一既有 warning。
- `cargo test --lib workflow_run_check`：通过，2 passed / 0 failed / 350 filtered out；保留同一既有 warning。
- `cargo test --lib`：通过，336 passed / 0 failed / 16 ignored；保留同一既有 warning。
- `cargo fmt -- --check`：通过。
- `git diff --check`：通过，无输出。
- `git status --short`：通过，无输出。

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：R2-B3 仍使用 `include!` 作为保守过渡，后续 R2 可以继续收敛为正式模块边界。
- P2：R2-B3 只抽出 workflow state 生命周期入口和 task package 写入链，不代表 workflow read model、storage migration、memory / runtime diagnostics 拆分或 R2 水位线目标完成。
- P2：task package 相关测试仍主要保留在 `lib.rs` inline tests 中，后续 R2 后段可按领域迁移 tests。

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

创建并执行 R2-B4。主管线决定把 R2-B4 限定为前段连续 workflow run / binding / legacy dispatch entrypoints 物理抽出，而不是立即跳到大型 workflow read model 汇合；原因是该连续块更小、测试落点清楚、风险低。workflow read model 顺延到后续 R2 批次。
