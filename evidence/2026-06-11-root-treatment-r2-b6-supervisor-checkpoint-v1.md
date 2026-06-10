# Root Treatment R2-B6 Supervisor Checkpoint v1

日期：2026-06-11

## 结论

R2-B6 已由主管线回收为 `accepted_with_p2`。

接受范围：

- 任务包点名的 workflow dispatch execution control、offline role dispatch、workflow machine run loop 和相邻 execution result helper 已从 `src-tauri/src/lib.rs` 抽出到 `src-tauri/src/workflow_execution_entrypoints.rs`。
- `lib.rs` 通过 `include!("workflow_execution_entrypoints.rs")` 在 crate root 展开 helper，保持函数可见性和行为不变。
- `lib.rs` 从 21,463 行降到 19,401 行，低于 R0 水位线。
- 新 helper 文件为 2,068 行，低于 Rust 3,000 行治理阈值。
- command registry 总量保持 96，`lib.rs` 内 `#[tauri::command]` 保持 0。

不接受范围：

- 不接受为 R2 全部完成。
- 不接受为 `lib.rs <= 15,000` 或 `lib.rs <= 3,000` 目标完成。
- 不接受为 memory、runtime diagnostics、provider adapter、tests 巨石、SQLite 迁移或 R3 完成。
- 不接受为 Stage L / K3-B1 / K3-B2 恢复。
- 不接受为新的真实 Codex 执行授权。

## Commit 记录

- R2-B6 start commit：`93c20bb5b515ced0f0306ec57d61b222a592a08a`
- R2-B6 completion commit：`2dd766be84e977d75e77f31ec2dbf9d463f45690`
- 本 supervisor checkpoint 提交：待提交后回填

## 复核文件

- `tasks/2026-06-11-root-treatment-r2-b6-workflow-execution-control-offline-role-and-machine-extraction-v1.md`
- `evidence/2026-06-11-root-treatment-r2-b6-workflow-execution-control-offline-role-and-machine-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r2-b6-workflow-execution-control-offline-role-and-machine-extraction-v1-result.md`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_execution_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`

## 主管 Fresh Verify

已重新运行：

```bash
node scripts/harness/workbench-shape-gate.js --mode baseline
node scripts/harness/workbench-shape-gate.js --mode check
cargo test --lib workflow_dispatch
cargo test --lib workflow_node_dispatch
cargo test --lib offline_role
cargo test --lib workflow_machine
cargo test --lib workflow_permission
cargo test --lib
cargo fmt -- --check
git diff --check
git status --short
```

结果：

- shape gate baseline：通过，0 errors / 0 warnings；`lib.rs` 19,401 lines；Tauri commands 96 total / 0 in `lib.rs`；Sidecar JSON kinds 14 detected / 0 unknown。
- shape gate check：通过，0 errors / 0 warnings；`lib.rs` ratchet `19,401 / 25,925`，status `decreased`。
- `cargo test --lib workflow_dispatch`：通过，2 passed / 0 failed / 1 ignored / 349 filtered out；保留既有 `JsonRpcError::invalid_params` dead_code warning。
- `cargo test --lib workflow_node_dispatch`：通过，11 passed / 0 failed / 341 filtered out；保留同一既有 warning。
- `cargo test --lib offline_role`：通过，3 passed / 0 failed / 349 filtered out；保留同一既有 warning。
- `cargo test --lib workflow_machine`：通过，2 passed / 0 failed / 350 filtered out；保留同一既有 warning。
- `cargo test --lib workflow_permission`：通过，1 passed / 0 failed / 351 filtered out；保留同一既有 warning。
- `cargo test --lib`：通过，336 passed / 0 failed / 16 ignored；保留同一既有 warning。
- `cargo fmt -- --check`：通过。
- `git diff --check`：通过，无输出。
- `git status --short`：通过，无输出。

## 主管边界复核

已核对：

- R2-B6 completion commit 只包含 4 个文件：`lib.rs`、`workflow_execution_entrypoints.rs`、R2-B6 evidence、R2-B6 handoff。
- helper 函数清单集中于 workflow dispatch authorization / execution control、offline role dispatch / handoff / review、workflow machine validation / step loop / result assembly。
- helper 内没有新增 `#[tauri::command]`、sidecar store、SQLite schema、UI、planned adapter 或 provider credential 读取。
- `render_workflow_machine_step_prompt` 和相关 prompt render helper 是既有 workflow machine 逻辑的物理搬移，不是本轮新增真实 Codex 执行入口。

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：R2-B6 仍使用 `include!` 作为保守过渡，后续 R2 可以继续收敛为正式模块边界。
- P2：R2-B6 只抽出 workflow dispatch execution control / offline role dispatch / workflow machine，不代表 memory、runtime diagnostics、provider adapter、storage migration、UI、tests 巨石拆分或 R2 水位线目标完成。
- P2：相关测试仍主要保留在 `lib.rs` inline tests 中，后续 R2 后段可按领域迁移 tests。

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

创建并执行 R2-B7。R2-B7 建议限定为 memory domain 相关逻辑的行为不变物理抽出；不顺手做 R3 SQLite、R4 UI/按页读模型、Stage L/K3 恢复或真实 Codex 执行。
