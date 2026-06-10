# Root Treatment R2-B5 Supervisor Checkpoint v1

日期：2026-06-11

## 结论

R2-B5 已由主管线回收为 `accepted_with_p2`。

接受范围：

- 任务包点名的 workflow read model、dispatch summary、readback stats 和相邻 workflow read surface 派生逻辑已从 `src-tauri/src/lib.rs` 抽出到 `src-tauri/src/workflow_read_model_entrypoints.rs`。
- `lib.rs` 通过 `include!("workflow_read_model_entrypoints.rs")` 在 crate root 展开 helper，保持函数可见性和行为不变。
- `lib.rs` 从 23,524 行降到 21,463 行，低于 R0 水位线。
- 新 helper 文件为 2,066 行，低于 Rust 3,000 行治理阈值。
- command registry 总量保持 96，`lib.rs` 内 `#[tauri::command]` 保持 0。

不接受范围：

- 不接受为 R2 全部完成。
- 不接受为 `lib.rs <= 15,000` 或 `lib.rs <= 3,000` 目标完成。
- 不接受为 C4-C6 自动化执行逻辑、workflow machine、memory、runtime diagnostics、provider adapter、tests 巨石、SQLite 迁移或 R3 完成。
- 不接受为 Stage L / K3-B1 / K3-B2 恢复。
- 不接受为新的真实 Codex 执行授权。

## Commit 记录

- R2-B5 start commit：`86ce04032cce9ec1b1bd2970c78cd6be587b3cd9`
- R2-B5 completion commit：`35cacc22ec813152e9357a42bc82e7ef581d2509`
- 本 supervisor checkpoint 提交：待本提交生成后回填。

## 复核文件

- `tasks/2026-06-11-root-treatment-r2-b5-workflow-read-model-dispatch-summary-and-readback-stats-extraction-v1.md`
- `evidence/2026-06-11-root-treatment-r2-b5-workflow-read-model-dispatch-summary-and-readback-stats-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r2-b5-workflow-read-model-dispatch-summary-and-readback-stats-extraction-v1-result.md`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_read_model_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`

## 主管 Fresh Verify

已重新运行：

```bash
node scripts/harness/workbench-shape-gate.js --mode baseline
node scripts/harness/workbench-shape-gate.js --mode check
cargo test --lib workflow_task_package_read_model
cargo test --lib workflow_ledger
cargo test --lib workflow_exception
cargo test --lib workflow_interfaces
cargo test --lib dispatch_readback_stats
cargo test --lib workbench_snapshot
cargo test --lib
cargo fmt -- --check
git diff --check
git status --short
```

结果：

- shape gate baseline：通过，0 errors / 0 warnings；`lib.rs` 21,463 lines；Tauri commands 96 total / 0 in `lib.rs`；Sidecar JSON kinds 14 detected / 0 unknown。
- shape gate check：通过，0 errors / 0 warnings；`lib.rs` ratchet `21,463 / 25,925`，status `decreased`。
- `cargo test --lib workflow_task_package_read_model`：通过，1 passed / 0 failed / 351 filtered out；保留既有 `JsonRpcError::invalid_params` dead_code warning。
- `cargo test --lib workflow_ledger`：通过，1 passed / 0 failed / 351 filtered out；保留同一既有 warning。
- `cargo test --lib workflow_exception`：通过，1 passed / 0 failed / 351 filtered out；保留同一既有 warning。
- `cargo test --lib workflow_interfaces`：通过，1 passed / 0 failed / 351 filtered out；保留同一既有 warning。
- `cargo test --lib dispatch_readback_stats`：通过，6 passed / 0 failed / 346 filtered out；保留同一既有 warning。
- `cargo test --lib workbench_snapshot`：通过，1 passed / 0 failed / 351 filtered out；保留同一既有 warning。
- `cargo test --lib`：通过，336 passed / 0 failed / 16 ignored；保留同一既有 warning。
- `cargo fmt -- --check`：通过。
- `git diff --check`：通过，无输出。
- `git status --short`：通过，无输出。

## 主管边界复核

已核对：

- R2-B5 completion commit 只包含 4 个文件：`lib.rs`、`workflow_read_model_entrypoints.rs`、R2-B5 evidence、R2-B5 handoff。
- helper 函数清单集中于 workflow snapshot/read model、project blackboard、ledger/result/exception/interface/state-machine/acceptance scenarios、dispatch summaries、readback stats。
- helper 内没有 `Command::new("codex")`、`safe_probe_prompt`、workflow machine runner、provider adapter、memory 写入、SQLite 迁移、Tauri command 或 sidecar 新增。
- helper 中出现的 `safe_probe_target()` 仅用于 dispatch readback stats 匹配目标文本，不是 prompt 发送或真实执行入口。

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：R2-B5 仍使用 `include!` 作为保守过渡，后续 R2 可以继续收敛为正式模块边界。
- P2：R2-B5 只抽出 workflow read model / dispatch summary / readback stats，不代表 C4-C6 自动化执行逻辑、workflow machine、storage migration、memory、runtime diagnostics、provider adapter、tests 巨石拆分或 R2 水位线目标完成。
- P2：read-model / dispatch readback 相关测试仍主要保留在 `lib.rs` inline tests 中，后续 R2 后段可按领域迁移 tests。

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

创建并执行 R2-B6。R2-B6 应限定为 workflow dispatch execution control / offline role dispatch / workflow machine 的行为不变物理抽出；不顺手做 R3 SQLite、R4 UI/按页读模型、Stage L/K3 恢复或真实 Codex 执行。
