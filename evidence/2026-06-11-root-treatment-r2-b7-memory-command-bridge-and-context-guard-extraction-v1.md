# Root Treatment R2-B7 Memory Command Bridge And Context Guard Extraction v1

日期：2026-06-11

## 结论

R2-B7 本轮完成行为不变的第七批 `lib.rs` 形状治理：

- 新增 `prototypes/productized-desktop-shell/src-tauri/src/memory_context_entrypoints.rs`。
- 将 memory command bridge、observation bridge、task memory packet preview bridge 和上下文绑定 guard 从 `lib.rs` 物理移入新 helper 文件。
- `lib.rs` 原位置保留 `include!("memory_context_entrypoints.rs")`，继续在 crate root 展开，避免修改 helper 可见性。
- 未修改函数语义、返回值、错误文案、公开 Tauri command 契约或 workflow state schema。

R2-B7 可接受为：memory command bridge / observation bridge / task memory packet preview bridge / context guard 已从 `lib.rs` 抽出，`lib.rs` 行数继续下降，command 总量和 `lib.rs` 内 command 数量保持不变，并通过 shape gate、任务包指定 Rust 聚焦测试、全量库测试和格式检查。

R2-B7 不接受为：R2 全部完成、`lib.rs <= 15,000` 或 `<= 3,000` 目标完成、memory 系统产品功能新增、memory entity relation / formal memory lifecycle / memory store 内部实现重构完成、runtime diagnostics / provider adapter / tests 巨石拆分完成、SQLite 迁移完成、workflow state schema 迁移完成、Stage L / K3-B1 / K3-B2 恢复或新的真实 Codex 执行授权。

## Commit 记录

- Start commit：`174338153663d1d906933c6893165113f7edb376`
- End commit：本文件随 R2-B7 completion commit 一起提交；提交创建后由最终回交记录实际 hash。

## 改动文件

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_context_entrypoints.rs`
- `evidence/2026-06-11-root-treatment-r2-b7-memory-command-bridge-and-context-guard-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r2-b7-memory-command-bridge-and-context-guard-extraction-v1-result.md`

未修改：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `prototypes/productized-desktop-shell/src-tauri/src/formal_memory_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/formal_memory_lifecycle.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_candidate_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_lint_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/observation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/task_memory_packet_builder.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/task_memory_injection.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_lifecycle_task_package.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs`

## 抽出清单

以下函数已从 `lib.rs` 移入 `memory_context_entrypoints.rs`：

- `create_formal_memory_record_at`
- `adopt_memory_candidate_to_formal_memory_at`
- `run_memory_lint_at`
- `validate_memory_lint_context_binding`
- `create_observation_at`
- `create_memory_candidate_from_observation_at`
- `preview_task_memory_packet_at`
- `validate_task_memory_packet_context_binding`
- `validate_task_memory_packet_context_field`
- `validate_task_memory_packet_project_registered`
- `validate_observation_context_binding`
- `validate_observation_context_field`
- `validate_observation_project_registered`
- `validate_formal_memory_context_binding`
- `validate_formal_memory_context_field`
- `validate_formal_memory_project_registered`

本轮未迁移 Rust 类型定义、command wrapper、memory store 内部实现、memory entity relation 逻辑或 tests；相关类型仍按既有 `types.rs` crate-root include 暴露，测试仍保留在既有 inline tests 中。

## Shape 指标

| 指标 | R2-B7 前 | R2-B7 后 |
| --- | ---: | ---: |
| `lib.rs` 行数 | 19,401 | 18,932 |
| `memory_context_entrypoints.rs` 行数 | 0 | 475 |
| Tauri command registry 总量 | 96 | 96 |
| `lib.rs` 内 `#[tauri::command]` 数量 | 0 | 0 |
| Sidecar JSON kinds | 14 allowed / 0 unknown | 14 allowed / 0 unknown |

说明：

- `lib.rs` 较 R2-B6 checkpoint 继续减少 469 行。
- 新 helper 文件 475 行，低于 Rust 3,000 行治理阈值。
- 本轮选择 crate-root `include!`。原因是抽出块依赖 crate-root private helper 和既有 inline tests；正式 `mod` 会要求扩大可见性修改，不符合本任务的行为不变和小风险边界。
- 本轮只搬移 memory command bridge / observation bridge / task memory packet preview bridge / context guard；未拆 C5/C6 自动化、blackboard、runtime diagnostics、provider adapter、SQLite、UI 或 tests 巨石。

## 代码地图摘要

R2-B7 对应当前 `lib.rs` 中 `include!("workflow_state_json_helpers.rs")` 后的连续 memory context block：

- formal memory 创建入口与 context binding guard。
- memory candidate adoption bridge 与 memory lint guard。
- observation 创建 / observation-to-candidate bridge 与 context binding guard。
- task memory packet preview bridge 与 project registration / context field guard。

R2-B7 已将上述函数抽入 `memory_context_entrypoints.rs`，`lib.rs` 原位置仅保留 crate-root `include!`。`option_trimmed_is_empty` 及其后的 shared workflow utilities、blackboard helper、diagnostics、provider、continuation、adapter 和 inline tests 均留在本批次外。

## 验证记录

已运行：

```bash
node scripts/harness/workbench-shape-gate.js --mode baseline
node scripts/harness/workbench-shape-gate.js --mode check
cargo test --lib formal_memory
cargo test --lib memory_candidate
cargo test --lib memory_lint
cargo test --lib observation
cargo test --lib task_memory
cargo test --lib memory_entity_relation
cargo test --lib
cargo fmt -- --check
git diff --check
git status --short
```

结果：

- `node scripts/harness/workbench-shape-gate.js --mode baseline`：通过，Status `pass`，0 errors / 0 warnings / 12 info。
- baseline key metrics：`lib.rs` 18,932 lines；Tauri commands 96 total / 0 in `lib.rs`；Sidecar JSON kinds 14 detected / 0 unknown；`lib.rs` ratchet 18,932 / 25,925，status `decreased`。
- `node scripts/harness/workbench-shape-gate.js --mode check`：通过，Status `pass`，0 errors / 0 warnings / 12 info。
- check key metrics：`lib.rs` 18,932 lines；Tauri commands 96 total / 0 in `lib.rs`；Sidecar JSON kinds 14 detected / 0 unknown；`lib.rs` ratchet 18,932 / 25,925，status `decreased`。
- `cargo test --lib formal_memory`：通过，29 passed / 0 failed / 323 filtered out；保留既有 warning：`JsonRpcError::invalid_params` dead_code。
- `cargo test --lib memory_candidate`：通过，9 passed / 0 failed / 343 filtered out；保留同一既有 warning。
- `cargo test --lib memory_lint`：通过，9 passed / 0 failed / 343 filtered out；保留同一既有 warning。
- `cargo test --lib observation`：通过，15 passed / 0 failed / 337 filtered out；保留同一既有 warning。
- `cargo test --lib task_memory`：通过，15 passed / 0 failed / 337 filtered out；保留同一既有 warning。
- `cargo test --lib memory_entity_relation`：通过，5 passed / 0 failed / 347 filtered out；保留同一既有 warning。
- `cargo test --lib`：通过，336 passed / 0 failed / 16 ignored；保留同一既有 warning。
- `cargo fmt -- --check`：通过，无输出。
- `git diff --check`：通过，无输出。
- `git status --short`：提交前仅包含 R2-B7 范围文件。

所有任务包点名 filters 均有匹配测试并通过；没有将无匹配或环境失败冒充通过。

## 边界确认

- 未改产品业务逻辑。
- 未改函数语义、返回值、错误文案或公开 Tauri command 契约。
- 未改 workflow state 顶层 schema。
- 未新增 `#[tauri::command]`。
- 未新增 sidecar store 或 sidecar JSON 种类。
- 未迁移 SQLite。
- 未做 UI。
- 未做 C5/C6 自动化、blackboard、runtime diagnostics、provider adapter 或 tests 巨石其他拆分。
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
- P2：本轮只抽出 memory command bridge / context guard，不代表 memory 模块内部重构、runtime diagnostics、provider adapter、SQLite、UI 或 tests 巨石拆分完成。
- P2：相关测试仍主要保留在 `lib.rs` inline tests 中，后续 R2 后段可按领域迁移 tests。
