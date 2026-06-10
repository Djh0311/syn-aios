# Root Treatment R2-B4 Workflow Run Binding And Legacy Dispatch Entrypoints Extraction v1

日期：2026-06-11

## 结论

R2-B4 本轮完成行为不变的第四批 `lib.rs` 形状治理：

- 新增 `prototypes/productized-desktop-shell/src-tauri/src/workflow_run_dispatch_entrypoints.rs`。
- 将任务包点名的 workflow run check、work item state、session binding 和 legacy workflow node dispatch entrypoints 从 `lib.rs` 物理移入新 helper 文件。
- `lib.rs` 原位置保留 `include!("workflow_run_dispatch_entrypoints.rs")`，继续在 crate root 展开，避免修改 helper 可见性。
- 未修改函数语义、返回值、错误文案、公开 Tauri command 契约或 workflow state schema。

R2-B4 可接受为：workflow run / binding / legacy dispatch entrypoints 已从 `lib.rs` 抽出，`lib.rs` 行数继续下降，command 总量和 `lib.rs` 内 command 数量保持不变，并通过 shape gate、workflow run / dispatch / binding / work item / task package 聚焦测试、全量库测试和格式检查。

R2-B4 不接受为：R2 全部完成、`lib.rs <= 15,000` 或 `<= 3,000` 目标完成、workflow read model 拆分完成、C4-C6 自动化拆分完成、memory / runtime diagnostics / tests 巨石拆分完成、SQLite 迁移完成、workflow state schema 迁移完成、Stage L / K3-B1 / K3-B2 恢复或新的真实 Codex 执行授权。

## Commit 记录

- Start commit：`83b9219e464d51549ae470bde480fb7e81cff19b`
- End commit：本 R2-B4 提交承载；提交后真实 hash 由回交记录。

## 改动文件

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_run_dispatch_entrypoints.rs`
- `evidence/2026-06-11-root-treatment-r2-b4-workflow-run-binding-and-legacy-dispatch-entrypoints-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r2-b4-workflow-run-binding-and-legacy-dispatch-entrypoints-extraction-v1-result.md`

未修改：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_json_helpers.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_lifecycle_task_package.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs`

## 抽出清单

以下函数 / 类型已从 `lib.rs` 移入 `workflow_run_dispatch_entrypoints.rs`：

- `inspect_workflow_run_check_at`
- `inspect_workflow_run_check_from_value`
- `blocked_workflow_run_check`
- `push_check`
- `find_task_package_artifact_for_work_item`
- `update_work_item_state_at`
- `bind_workflow_node_codex_session_at`
- `unbind_workflow_node_codex_session_at`
- `prepare_workflow_node_dispatch_at`
- `execute_workflow_node_dispatch_at`
- `read_workflow_node_dispatch_result_at`
- `WorkflowNodeDispatchContext`
- `workflow_node_dispatch_context`
- `inspect_task_package_authorization_at`

仍保留在 `lib.rs` 的相邻 C4 起点：

- `ProjectDirectorAuthorizationContext`
- `ActiveBindingInfo`
- `preview_project_director_task_plan_for_index_at`

## Shape 指标

| 指标 | R2-B4 前 | R2-B4 后 |
| --- | ---: | ---: |
| `lib.rs` 行数 | 24,635 | 23,524 |
| `workflow_run_dispatch_entrypoints.rs` 行数 | 0 | 1,115 |
| Tauri command registry 总量 | 96 | 96 |
| `lib.rs` 内 `#[tauri::command]` 数量 | 0 | 0 |
| Sidecar JSON kinds | 14 allowed / 0 unknown | 14 allowed / 0 unknown |

说明：

- `lib.rs` 较 R2-B3 checkpoint 继续减少 1,111 行。
- 新 helper 文件 1,115 行，低于 Rust 3,000 行治理阈值。
- 本轮只搬移 workflow run check、work item state、session binding 和 legacy workflow node dispatch entrypoints；未拆 C4-C6 自动化、workflow read model、memory、runtime diagnostics、SQLite、UI 或 tests 巨石。

## 代码地图摘要

R2 代码地图中 R2-B4 对应前段连续块：

- workflow run check。
- work item state update。
- workflow node session binding / unbinding。
- legacy workflow node dispatch prepare / execute / readback context。
- task package authorization inspection helper。

R2-B4 已将上述函数 / 类型抽入 `workflow_run_dispatch_entrypoints.rs`，`lib.rs` 原位置仅保留 crate-root `include!`。C4-C6 自动化相关入口和类型仍从 `ProjectDirectorAuthorizationContext` 开始留在 `lib.rs`，等待后续 R2 批次处理。

## 验证记录

已运行：

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

- `node scripts/harness/workbench-shape-gate.js --mode baseline`：通过，Status `pass`，0 errors / 0 warnings / 12 info。
- baseline key metrics：`lib.rs` 23,524 lines；Tauri commands 96 total / 0 in `lib.rs`；Sidecar JSON kinds 14 detected / 0 unknown；`lib.rs` ratchet 23,524 / 25,925，status `decreased`。
- `node scripts/harness/workbench-shape-gate.js --mode check`：通过，Status `pass`，0 errors / 0 warnings / 12 info。
- check key metrics：`lib.rs` 23,524 lines；Tauri commands 96 total / 0 in `lib.rs`；Sidecar JSON kinds 14 detected / 0 unknown；`lib.rs` ratchet 23,524 / 25,925，status `decreased`。
- `cargo test --lib workflow_run_check`：通过，2 passed / 0 failed / 350 filtered out；保留既有 warning：`JsonRpcError::invalid_params` dead_code。
- `cargo test --lib workflow_node_dispatch`：通过，11 passed / 0 failed / 341 filtered out；保留同一既有 warning。该命令在并行启动时曾正常等待 cargo artifact directory file lock，随后通过，不是测试失败或环境失败。
- `cargo test --lib workflow_node_session_binding`：通过，2 passed / 0 failed / 350 filtered out；保留同一既有 warning。
- `cargo test --lib work_item_state_update`：通过，3 passed / 0 failed / 349 filtered out；保留同一既有 warning。
- `cargo test --lib task_package`：通过，29 passed / 0 failed / 1 ignored / 322 filtered out；保留同一既有 warning。
- `cargo test --lib`：通过，336 passed / 0 failed / 16 ignored；保留同一既有 warning。
- `cargo fmt -- --check`：通过，无输出。
- `git diff --check`：通过，无输出。
- `git status --short`：提交前仅包含 R2-B4 范围文件。

所有任务包点名 filters 均有匹配测试并通过；没有将无匹配或环境失败冒充通过。

## 边界确认

- 未改产品业务逻辑。
- 未改函数语义、返回值、错误文案或公开 Tauri command 契约。
- 未改 workflow state 顶层 schema。
- 未新增 `#[tauri::command]`。
- 未新增 sidecar store 或 sidecar JSON 种类。
- 未迁移 SQLite。
- 未做 UI。
- 未做 C4-C6 自动化 / workflow read model / memory / runtime diagnostics / tests 巨石其他拆分。
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
- P2：本轮只抽出 workflow run / binding / legacy dispatch entrypoints，不代表 workflow read model、C4-C6 自动化、storage migration、memory / runtime diagnostics / tests 巨石拆分或 R2 水位线目标完成。
- P2：dispatch / binding 相关测试仍主要保留在 `lib.rs` inline tests 中，后续 R2 后段可按领域迁移 tests。
