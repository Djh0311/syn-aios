# Root Treatment R2-B2 Lib Map And Workflow State Helper Extraction v1

日期：2026-06-11

## 结论

R2-B2 本轮完成行为不变的第二批 `lib.rs` 形状治理：

- 新增 R2 `lib.rs` 静态代码地图：`docs/plans/2026-06-11-root-treatment-r2-lib-rs-code-map-v1.md`。
- 新增 `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_json_helpers.rs`。
- 将任务包点名的 15 个 workflow state JSON helper 从 `lib.rs` 物理移入新 helper 文件。
- `lib.rs` 原位置保留 `include!("workflow_state_json_helpers.rs")`，继续在 crate root 展开，避免修改 helper 可见性。
- 未修改函数语义、返回值、错误文案、公开 Tauri command 契约或 workflow state schema。

R2-B2 可接受为：R2 代码地图已建立，workflow state JSON helper 已从 `lib.rs` 抽出，`lib.rs` 行数继续下降，command 总量和 `lib.rs` 内 command 数量保持不变，并通过 shape gate、workflow state 聚焦测试、全量库测试和格式检查。

R2-B2 不接受为：R2 全部完成、`lib.rs <= 15,000` 或 `<= 3,000` 目标完成、workflow read model 拆分完成、memory / runtime diagnostics 拆分完成、SQLite 迁移完成、workflow state schema 迁移完成、Stage L / K3-B1 / K3-B2 恢复或新的真实 Codex 执行授权。

## Commit 记录

- Start commit：`d737c78eb9e9ce1e1f8e620390d595c498a70e0f`
- End commit：`76ed0ef46d9b0a2a83f6e77ce533d6c8741c93cf`

## 改动文件

- `docs/plans/2026-06-11-root-treatment-r2-lib-rs-code-map-v1.md`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_json_helpers.rs`
- `evidence/2026-06-11-root-treatment-r2-b2-lib-map-and-workflow-state-helper-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r2-b2-lib-map-and-workflow-state-helper-extraction-v1-result.md`

未修改：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs`

## 抽出清单

以下 helper 已从 `lib.rs` 移入 `workflow_state_json_helpers.rs`：

- `initial_workflow_state_json`
- `read_workflow_state_value`
- `validate_workflow_state`
- `write_validated_workflow_state`
- `backup_workflow_state_file`
- `ensure_workflow_node_session_bindings_array`
- `ensure_workflow_node_dispatches_array`
- `array_mut`
- `ensure_array_mut`
- `find_workflow_node_dispatch`
- `find_workflow_node_dispatch_index`
- `node_exists`
- `workflow_node_session_binding_index`
- `project_exists`
- `workflow_exists`

## Shape 指标

| 指标 | R2-B2 前 | R2-B2 后 |
| --- | ---: | ---: |
| `lib.rs` 行数 | 25,829 | 25,643 |
| `workflow_state_json_helpers.rs` 行数 | 0 | 190 |
| Tauri command registry 总量 | 96 | 96 |
| `lib.rs` 内 `#[tauri::command]` 数量 | 0 | 0 |
| Sidecar JSON kinds | 14 allowed / 0 unknown | 14 allowed / 0 unknown |

说明：

- `lib.rs` 较 R2-B1 checkpoint 继续减少 186 行。
- 新 helper 文件 190 行，低于 Rust 3,000 行治理阈值。
- `workflow_state_store.rs` 未改；本轮只是把 `lib.rs` 内 wrapper / JSON helper 物理搬出。

## 代码地图摘要

新增代码地图记录了 R2-B2 后 `lib.rs` 主要领域块：

- crate 装配与保守 include。
- index / transcript 读取。
- workflow state 生命周期入口。
- task package 写入链。
- workflow run check / binding / dispatch 入口。
- C4-C6 自动化工作流治理。
- workflow dispatch 执行控制。
- offline role dispatch 与 workflow machine。
- task package 渲染 / finder helper。
- memory command bridge / context guard。
- workflow snapshot / read model / dispatch summaries。
- snapshot assembly、diagnostics、provider / continuation / adapter descriptors。
- session / index parsing、allowed paths、host OS helper、Tauri app assembly。
- inline tests。

地图建议后续 R2 批次按 workflow state 生命周期、workflow read model、dispatch / workflow machine、memory context guard、diagnostics / continuation / adapter、index parsing / app assembly、tests 迁移逐步拆分。

## 验证记录

已运行：

```bash
node scripts/harness/workbench-shape-gate.js --mode baseline
node scripts/harness/workbench-shape-gate.js --mode check
cargo test --lib workflow_state
cargo test --lib
cargo fmt -- --check
git diff --check
git status --short
```

结果：

- `node scripts/harness/workbench-shape-gate.js --mode baseline`：通过，Status `pass`，0 errors / 0 warnings / 12 info。
- baseline key metrics：`lib.rs` 25,643 lines；Tauri commands 96 total / 0 in `lib.rs`；Sidecar JSON kinds 14 detected / 0 unknown；`lib.rs` ratchet 25,643 / 25,925，status `decreased`。
- `node scripts/harness/workbench-shape-gate.js --mode check`：通过，Status `pass`，0 errors / 0 warnings / 12 info。
- check key metrics：`lib.rs` 25,643 lines；Tauri commands 96 total / 0 in `lib.rs`；Sidecar JSON kinds 14 detected / 0 unknown；`lib.rs` ratchet 25,643 / 25,925，status `decreased`。
- `cargo test --lib workflow_state`：通过，11 passed / 0 failed / 341 filtered out；保留既有 warning：`JsonRpcError::invalid_params` dead_code。
- `cargo test --lib`：通过，336 passed / 0 failed / 16 ignored；保留同一既有 warning。
- `cargo fmt -- --check`：通过，无输出。
- `git diff --check`：通过，无输出。
- `git status --short`：提交前仅包含 R2-B2 范围文件。

## 边界确认

- 未改产品业务逻辑。
- 未改函数语义、返回值、错误文案或公开 Tauri command 契约。
- 未改 workflow state 顶层 schema。
- 未新增 `#[tauri::command]`。
- 未新增 sidecar store 或 sidecar JSON 种类。
- 未迁移 SQLite。
- 未做 UI。
- 未做 workflow read model / memory / runtime diagnostics 其他拆分。
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
- P2：R2 代码地图是人工静态地图，后续行号会随拆分漂移，可在后续治理批次考虑脚本化。
- P2：本轮只抽出 workflow state JSON helper，不代表 workflow read model、storage migration、memory / runtime diagnostics 拆分或 R2 水位线目标完成。
