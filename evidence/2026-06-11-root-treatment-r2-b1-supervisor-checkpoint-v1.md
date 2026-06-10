# Root Treatment R2-B1 Supervisor Checkpoint v1

日期：2026-06-11

## 结论

R2-B1 已由主管线回收为 `accepted_with_p2`。

接受范围：

- `tauri::generate_handler![...]` command registry 已从 `src-tauri/src/lib.rs` 的 `run()` 中拆出到 `src-tauri/src/command_registry.rs`。
- `lib.rs` 通过 `include!("command_registry.rs")` 在 crate root 展开 `workbench_command_handler!()`，保持 command wrapper 可见性、名称和行为不变。
- command registry 总量保持 96，`lib.rs` 内 `#[tauri::command]` 保持 0。
- `lib.rs` 从 25,925 行降到 25,829 行，低于 R0 水位线。

不接受范围：

- 不接受为 R2 全部完成。
- 不接受为 `lib.rs <= 15,000` 或 `lib.rs <= 3,000` 目标完成。
- 不接受为 workflow 读模型、记忆领域、runtime / diagnostics、SQLite 迁移或 R3 完成。
- 不接受为 Stage L / K3-B1 / K3-B2 恢复。
- 不接受为新的真实 Codex 执行授权。

## Commit 记录

- R2-B1 start commit：`c9b99632beb91255e05a4facf8a6e337a23a3d77`
- R2-B1 completion commit：`13016917442070fc2f59a130b2748eb0cba06a34`
- 本 supervisor checkpoint 提交：由本 evidence 所在提交承载；提交后真实 hash 由回交记录。

## 复核文件

- `tasks/2026-06-10-root-treatment-r2-b1-command-registry-extraction-v1.md`
- `evidence/2026-06-10-root-treatment-r2-b1-command-registry-extraction-v1.md`
- `handoffs/2026-06-10-root-treatment-r2-b1-command-registry-extraction-v1-result.md`
- `prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`

## 主管 Fresh Verify

已重新运行：

```bash
node scripts/harness/workbench-shape-gate.js --mode baseline
node scripts/harness/workbench-shape-gate.js --mode check
cargo test --lib
cargo fmt -- --check
git diff --check
git status --short
```

结果：

- shape gate baseline：通过，0 errors / 0 warnings；`lib.rs` 25,829 lines；Tauri commands 96 total / 0 in `lib.rs`；Sidecar JSON kinds 14 detected / 0 unknown。
- shape gate check：通过，0 errors / 0 warnings；`lib.rs` ratchet `25,829 / 25,925`，status `decreased`。
- `cargo test --lib`：通过，336 passed / 0 failed / 16 ignored；保留既有 `JsonRpcError::invalid_params` dead_code warning。
- `cargo fmt -- --check`：通过。
- `git diff --check`：通过，无输出。
- `git status --short`：通过，无输出。

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：R2-B1 仍使用 `include!` + `macro_rules!` 作为保守过渡，后续 R2 可以继续收敛为正式模块边界。
- P2：R2-B1 只减少 96 行，不代表 R2 第一阶段水位线目标完成。
- P2：command surface 只做 registry 物理拆分，未做命令分域、权限模型或 command 增量 gate 收紧。

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

创建并执行 R2-B2。建议范围限定为“R2 代码地图 + workflow state JSON helper 物理抽出”，继续采用行为不变的小切片，避免同时拆 workflow 读模型、记忆领域、runtime diagnostics 或 SQLite。
