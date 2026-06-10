# Root Treatment R2-B1 Command Registry Extraction v1 Result

日期：2026-06-10

## 结论

R2-B1 本轮可接受为：`tauri::generate_handler![...]` command registry 已从 `lib.rs` 的 `run()` 中拆出到独立 `command_registry.rs`，行为保持不变，command 总量仍为 96，`lib.rs` 内 `#[tauri::command]` 仍为 0。

不接受为：R2 `lib.rs <= 15,000` 水位线完成、command surface 重构完成、workflow / memory / runtime diagnostics 拆分完成、SQLite 迁移完成或真实 Codex 执行恢复。

## 已完成

- 新增 `prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs`。
- 在新文件中定义 `workbench_command_handler!()`，承载原 `tauri::generate_handler![...]` 96 项清单。
- `lib.rs` 在 `include!("commands.rs")` 后增加 `include!("command_registry.rs")`。
- `lib.rs` 的 `run()` 改为 `.invoke_handler(workbench_command_handler!())`。
- 未修改 `commands.rs`。
- 新增 R2-B1 evidence：`evidence/2026-06-10-root-treatment-r2-b1-command-registry-extraction-v1.md`。

## Shape 指标

- `lib.rs`：25,925 lines -> 25,829 lines。
- `command_registry.rs`：新增 105 lines。
- `commands.rs`：1,267 lines -> 1,267 lines。
- Tauri command registry：96 total -> 96 total。
- `lib.rs` 内 `#[tauri::command]`：0 -> 0。
- `commands.rs` 内 `#[tauri::command]`：90 -> 90。
- sidecar JSON kinds：14 detected / 0 unknown。

## 验证

已通过：

- `node scripts/harness/workbench-shape-gate.js --mode baseline`：pass，0 errors / 0 warnings；`lib.rs` 25,829 / 25,925，status `decreased`；Tauri commands 96 total / 0 in `lib.rs`。
- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors / 0 warnings；`lib.rs` 25,829 / 25,925，status `decreased`；Tauri commands 96 total / 0 in `lib.rs`。
- `cargo test --lib`：336 passed / 0 failed / 16 ignored。
- `cargo fmt -- --check`：通过。
- `git diff --check`：通过。
- `git status --short`：提交前仅包含 R2-B1 范围文件。

已知 warning：

- Rust 测试仍有既有 dead_code warning：`JsonRpcError::invalid_params` 未使用。

## Commit 记录

- Start commit：`c9b99632beb91255e05a4facf8a6e337a23a3d77`
- End commit：本 R2-B1 提交承载；提交后真实 hash 由回交记录。

## 边界

- 未同步入口文档：`CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`。
- 未改 command wrapper 签名、名称、参数或返回值。
- 未新增 Tauri command。
- 未新增 sidecar / sidecar JSON kind。
- 未迁移 SQLite。
- 未改 workflow state 顶层 schema。
- 未改真实 Codex runner。
- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript 或 rollout。
- 未启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 未做 UI、workflow 读模型、记忆领域、runtime diagnostics 或 backlog 功能。

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：`include!` + `macro_rules!` 是保守过渡，后续 R2 可再收敛为正式模块边界。
- P2：本轮只降低 `lib.rs` 96 行，不代表 R2 第一阶段水位线完成。
- P2：command surface 未做分域或增量 gate 收紧。
