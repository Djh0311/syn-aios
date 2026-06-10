# Root Treatment R2-B1 Command Registry Extraction v1

日期：2026-06-10

## 结论

R2-B1 本轮完成行为不变的 Tauri command registry 物理拆分：

- 新增 `prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs`。
- 将原 `lib.rs` / `run()` 内的 `tauri::generate_handler![...]` 命令清单移入 `workbench_command_handler!()` 宏。
- `lib.rs` 继续在 crate root `include!("commands.rs")` 后 `include!("command_registry.rs")`，以保守保持 command wrapper 的可见性和名称解析。
- `run()` 内 `.invoke_handler(...)` 现在只调用 `.invoke_handler(workbench_command_handler!())`。
- 未修改任何 command wrapper 函数签名、名称、参数或返回值。
- 未新增 `#[tauri::command]`。

R2-B1 可接受为：command registry 已从 `lib.rs` 的 `run()` 中拆出，command 总量保持 96，`lib.rs` 内 command 数量保持 0，并通过 shape gate、Rust 测试和格式检查。

R2-B1 不接受为：R2 `lib.rs <= 15,000` 水位线完成、command surface 重构完成、workflow / memory / runtime diagnostics 拆分完成、R3 SQLite 迁移完成、真实 Codex 执行恢复或 Stage L / K3 恢复。

## Commit 记录

- R2-B1 supervisor 指定 start commit：`c9b99632beb91255e05a4facf8a6e337a23a3d77`
- 任务包内历史基线说明 commit：`b0a6447`
- R2-B1 completion commit：`13016917442070fc2f59a130b2748eb0cba06a34`

## 改动文件

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs`
- `evidence/2026-06-10-root-treatment-r2-b1-command-registry-extraction-v1.md`
- `handoffs/2026-06-10-root-treatment-r2-b1-command-registry-extraction-v1-result.md`

未修改：

- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`

## Shape 指标

| 指标 | R2-B1 前 | R2-B1 后 |
| --- | ---: | ---: |
| `lib.rs` 行数 | 25,925 | 25,829 |
| `command_registry.rs` 行数 | 0 | 105 |
| `commands.rs` 行数 | 1,267 | 1,267 |
| Tauri command registry 总量 | 96 | 96 |
| `lib.rs` 内 `#[tauri::command]` 数量 | 0 | 0 |
| `commands.rs` 内 `#[tauri::command]` 数量 | 90 | 90 |

说明：

- `lib.rs` 较 R0 水位线 25,925 行减少 96 行。
- 新增 `command_registry.rs` 为 105 行，低于 Rust 3,000 行治理阈值。
- registry 内 96 项包含 `commands.rs` 的 90 个 wrapper 以及 6 个 `mcp::commands::*` command。

## 验证记录

已运行：

```bash
node scripts/harness/workbench-shape-gate.js --mode baseline
node scripts/harness/workbench-shape-gate.js --mode check
cargo test --lib
cargo fmt -- --check
git diff --check
git status --short
```

结果：

- `node scripts/harness/workbench-shape-gate.js --mode baseline`：通过，Status `pass`，0 errors / 0 warnings / 12 info。
- baseline key metrics：`lib.rs` 25,829 lines；Tauri commands 96 total / 0 in `lib.rs`；Sidecar JSON kinds 14 detected / 0 unknown；`lib.rs` ratchet 25,829 / 25,925，status `decreased`。
- `node scripts/harness/workbench-shape-gate.js --mode check`：通过，Status `pass`，0 errors / 0 warnings / 12 info。
- check key metrics：`lib.rs` 25,829 lines；Tauri commands 96 total / 0 in `lib.rs`；Sidecar JSON kinds 14 detected / 0 unknown；`lib.rs` ratchet 25,829 / 25,925，status `decreased`。
- `cargo test --lib`：通过，336 passed / 0 failed / 16 ignored；保留既有 warning：`JsonRpcError::invalid_params` dead_code。
- `cargo fmt -- --check`：通过，无输出。
- `git diff --check`：通过，无输出。
- `git status --short`：提交前仅包含 R2-B1 范围文件。

## 边界确认

- 未改产品业务逻辑。
- 未改 command wrapper 函数签名、名称、参数或返回值。
- 未新增 `#[tauri::command]`。
- 未新增 sidecar store 或 sidecar JSON 种类。
- 未迁移 SQLite。
- 未改 workflow state 顶层 schema。
- 未改真实 Codex runner。
- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript 或 rollout。
- 未启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 未启动 Stage L / K3-B1 retry / K3-B2。
- 未解冻 backlog 功能。
- 未拆 workflow 读模型、记忆领域、runtime diagnostics 或 SQLite。

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：本轮仍使用 `include!` + `macro_rules!` 作为保守过渡，避免在 R2-B1 一次性改动 96 个 command wrapper 的可见性；后续 R2 批次可再收敛为正式模块边界。
- P2：本轮只减少约百行，不代表 R2 第一阶段 `lib.rs <= 15,000` 水位线目标完成。
- P2：command surface 只做 registry 物理拆分，未做命令分域、权限模型或 command 增量 gate 收紧。
