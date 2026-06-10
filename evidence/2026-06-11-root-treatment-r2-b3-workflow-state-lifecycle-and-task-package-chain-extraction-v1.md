# Root Treatment R2-B3 Workflow State Lifecycle And Task Package Chain Extraction v1

日期：2026-06-11

## 结论

R2-B3 本轮完成行为不变的第三批 `lib.rs` 形状治理：

- 新增 `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_lifecycle_task_package.rs`。
- 将任务包点名的 workflow state 生命周期入口和 task package 写入链函数从 `lib.rs` 物理移入新 helper 文件。
- `lib.rs` 原位置保留 `include!("workflow_state_lifecycle_task_package.rs")`，继续在 crate root 展开，避免修改 helper 可见性。
- 未修改函数语义、返回值、错误文案、公开 Tauri command 契约或 workflow state schema。

R2-B3 可接受为：workflow state 生命周期入口和 task package 写入链已从 `lib.rs` 抽出，`lib.rs` 行数继续下降，command 总量和 `lib.rs` 内 command 数量保持不变，并通过 shape gate、workflow state / task package / workflow run check 聚焦测试、全量库测试和格式检查。

R2-B3 不接受为：R2 全部完成、`lib.rs <= 15,000` 或 `<= 3,000` 目标完成、workflow read model 拆分完成、memory / runtime diagnostics / tests 巨石拆分完成、SQLite 迁移完成、workflow state schema 迁移完成、Stage L / K3-B1 / K3-B2 恢复或新的真实 Codex 执行授权。

## Commit 记录

- Start commit：`446c1832b3d4e63d7bed5667814d47919277d342`
- End commit：本 R2-B3 提交承载；提交后真实 hash 由回交记录。

## 改动文件

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_lifecycle_task_package.rs`
- `evidence/2026-06-11-root-treatment-r2-b3-workflow-state-lifecycle-and-task-package-chain-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r2-b3-workflow-state-lifecycle-and-task-package-chain-extraction-v1-result.md`

未修改：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_json_helpers.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs`

## 抽出清单

以下 helper 已从 `lib.rs` 移入 `workflow_state_lifecycle_task_package.rs`：

- `read_workflow_state_snapshot`
- `initialize_workflow_state_at`
- `bootstrap_project_workflow_at`
- `append_default_project_workflow`
- `create_task_draft_at`
- `render_task_package_preview_at`
- `update_task_package_draft_fields_at`
- `update_task_package_fields_at`
- `generate_task_package_file_at`
- `inspect_task_package_dispatch_readiness_at`

仍保留在 `lib.rs` 的相邻入口：

- `inspect_workflow_run_check_at`

## Shape 指标

| 指标 | R2-B3 前 | R2-B3 后 |
| --- | ---: | ---: |
| `lib.rs` 行数 | 25,643 | 24,635 |
| `workflow_state_lifecycle_task_package.rs` 行数 | 0 | 1,012 |
| Tauri command registry 总量 | 96 | 96 |
| `lib.rs` 内 `#[tauri::command]` 数量 | 0 | 0 |
| Sidecar JSON kinds | 14 allowed / 0 unknown | 14 allowed / 0 unknown |

说明：

- `lib.rs` 较 R2-B2 checkpoint 继续减少 1,008 行。
- 新 helper 文件 1,012 行，低于 Rust 3,000 行治理阈值。
- 本轮只搬移 `lib.rs` 中 244-1253 附近的连续函数块；未拆 workflow read model、memory、runtime diagnostics、SQLite 或 tests 巨石。

## 代码地图摘要

R2-B2 代码地图中：

- `244-546`：workflow state 生命周期入口。
- `547-1253`：task package 写入链。

R2-B3 已将上述两段对应函数抽入 `workflow_state_lifecycle_task_package.rs`，`lib.rs` 原位置仅保留 crate-root `include!`。后续 `inspect_workflow_run_check_at` 及其后的 workflow run / binding / dispatch 入口仍留在 `lib.rs`，等待后续 R2 批次处理。

## 验证记录

已运行：

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

- `node scripts/harness/workbench-shape-gate.js --mode baseline`：通过，Status `pass`，0 errors / 0 warnings / 12 info。
- baseline key metrics：`lib.rs` 24,635 lines；Tauri commands 96 total / 0 in `lib.rs`；Sidecar JSON kinds 14 detected / 0 unknown；`lib.rs` ratchet 24,635 / 25,925，status `decreased`。
- `node scripts/harness/workbench-shape-gate.js --mode check`：通过，Status `pass`，0 errors / 0 warnings / 12 info。
- check key metrics：`lib.rs` 24,635 lines；Tauri commands 96 total / 0 in `lib.rs`；Sidecar JSON kinds 14 detected / 0 unknown；`lib.rs` ratchet 24,635 / 25,925，status `decreased`。
- `cargo test --lib workflow_state`：通过，11 passed / 0 failed / 341 filtered out；保留既有 warning：`JsonRpcError::invalid_params` dead_code。
- `cargo test --lib task_package`：通过，29 passed / 0 failed / 1 ignored / 322 filtered out；保留同一既有 warning。
- `cargo test --lib workflow_run_check`：通过，2 passed / 0 failed / 350 filtered out；保留同一既有 warning。
- `cargo test --lib`：通过，336 passed / 0 failed / 16 ignored；保留同一既有 warning。
- `cargo fmt -- --check`：通过，无输出。
- `git diff --check`：通过，无输出。
- `git status --short`：提交前仅包含 R2-B3 范围文件。

`task_package` 和 `workflow_run_check` filters 均有匹配测试并通过；没有将无匹配或环境失败冒充通过。

## 边界确认

- 未改产品业务逻辑。
- 未改函数语义、返回值、错误文案或公开 Tauri command 契约。
- 未改 workflow state 顶层 schema。
- 未新增 `#[tauri::command]`。
- 未新增 sidecar store 或 sidecar JSON 种类。
- 未迁移 SQLite。
- 未做 UI。
- 未做 workflow read model / memory / runtime diagnostics / tests 巨石其他拆分。
- 未改真实 Codex runner。
- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript 或 rollout。
- 未启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 未启动 Stage L / K3-B1 retry / K3-B2。
- 未解冻 backlog 功能。

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：继续使用 `include!` 作为保守过渡，后续 R2 可再收敛为正式模块边界。
- P2：本轮只抽出 workflow state 生命周期入口和 task package 写入链，不代表 workflow read model、storage migration、memory / runtime diagnostics / tests 巨石拆分或 R2 水位线目标完成。
- P2：task package 相关测试仍主要保留在 `lib.rs` inline tests 中，后续 R2 后段可按领域迁移 tests。
