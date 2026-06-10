# Root Treatment R2-B7 Memory Command Bridge And Context Guard Extraction v1 Result

日期：2026-06-11

## 结论

R2-B7 本轮可接受为：任务包点名的 memory command bridge、observation bridge、task memory packet preview bridge 和 context binding guard 已从 `lib.rs` 物理抽出到 `memory_context_entrypoints.rs`，并通过必需验证。行为保持不变，command 总量仍为 96，`lib.rs` 内 `#[tauri::command]` 仍为 0。

不接受为：R2 全部完成、`lib.rs <= 15,000` 水位线完成、memory 系统产品功能新增、memory entity relation / formal memory lifecycle / memory store 内部实现重构完成、runtime diagnostics / provider adapter / tests 巨石拆分完成、SQLite 迁移完成、workflow state schema 迁移或真实 Codex 执行恢复。

## 已完成

- 新增 `prototypes/productized-desktop-shell/src-tauri/src/memory_context_entrypoints.rs`。
- 将 formal memory / memory candidate adoption / memory lint / observation / task memory packet preview 的 bridge 和 context guard 从 `lib.rs` 搬到新 helper 文件。
- `lib.rs` 原位置改为 `include!("memory_context_entrypoints.rs")`。
- 新增 R2-B7 evidence：`evidence/2026-06-11-root-treatment-r2-b7-memory-command-bridge-and-context-guard-extraction-v1.md`。
- 未同步入口文档。

## 改动文件

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_context_entrypoints.rs`
- `evidence/2026-06-11-root-treatment-r2-b7-memory-command-bridge-and-context-guard-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r2-b7-memory-command-bridge-and-context-guard-extraction-v1-result.md`

## 抽出函数

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

本轮未迁移 Rust 类型定义、command wrapper、memory store 内部实现、memory entity relation 逻辑或 tests。

## Shape 指标

- `lib.rs`：19,401 lines -> 18,932 lines。
- `memory_context_entrypoints.rs`：新增 475 lines。
- Tauri command registry：96 total -> 96 total。
- `lib.rs` 内 `#[tauri::command]`：0 -> 0。
- Sidecar JSON kinds：14 detected / 0 unknown。

## 验证

已通过：

- `node scripts/harness/workbench-shape-gate.js --mode baseline`：pass，0 errors / 0 warnings；`lib.rs` 18,932 / 25,925，status `decreased`；Tauri commands 96 total / 0 in `lib.rs`。
- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors / 0 warnings；`lib.rs` 18,932 / 25,925，status `decreased`；Tauri commands 96 total / 0 in `lib.rs`。
- `cargo test --lib formal_memory`：29 passed / 0 failed / 323 filtered out。
- `cargo test --lib memory_candidate`：9 passed / 0 failed / 343 filtered out。
- `cargo test --lib memory_lint`：9 passed / 0 failed / 343 filtered out。
- `cargo test --lib observation`：15 passed / 0 failed / 337 filtered out。
- `cargo test --lib task_memory`：15 passed / 0 failed / 337 filtered out。
- `cargo test --lib memory_entity_relation`：5 passed / 0 failed / 347 filtered out。
- `cargo test --lib`：336 passed / 0 failed / 16 ignored。
- `cargo fmt -- --check`：通过。
- `git diff --check`：通过。
- `git status --short`：提交前仅包含 R2-B7 范围文件。

已知 warning：

- Rust 测试仍有既有 dead_code warning：`JsonRpcError::invalid_params` 未使用。

## Commit 记录

- Start commit：`174338153663d1d906933c6893165113f7edb376`
- End commit：本文件随 R2-B7 completion commit 一起提交；提交创建后由最终回交记录实际 hash。

## 边界

- 未同步 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`。
- 未改产品业务逻辑。
- 未改函数语义、返回值、错误文案、公开 Tauri command 契约或 workflow state schema。
- 未新增 Tauri command。
- 未新增 sidecar / sidecar JSON kind。
- 未迁移 SQLite。
- 未做 UI。
- 未做 C5/C6 自动化、blackboard、runtime diagnostics、provider adapter 或 tests 巨石其他拆分。
- 未执行真实 Codex。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript 或 rollout。
- 未启动 Tauri / Browser / Chrome / Vite / 截图工具。

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：`include!` 是保守过渡，后续 R2 可再收敛为正式模块边界。
- P2：本轮只抽出 memory command bridge / context guard，不代表 memory 模块内部重构、runtime diagnostics、provider adapter、SQLite、UI 或 tests 巨石拆分完成。
- P2：相关测试仍主要留在 `lib.rs` inline tests，后续 R2 后段再迁移 tests。
