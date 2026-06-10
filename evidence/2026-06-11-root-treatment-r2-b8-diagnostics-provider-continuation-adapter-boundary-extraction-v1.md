# Root Treatment R2-B8 Diagnostics Provider Continuation Adapter Boundary Extraction v1

日期：2026-06-11

## 结论

R2-B8 本轮完成行为不变的第八批 `lib.rs` 形状治理：

- 新增 `prototypes/productized-desktop-shell/src-tauri/src/diagnostics_provider_session_entrypoints.rs`。
- 将 diagnostics、store integrity、provider availability、session continuation preview / guard、agent adapter descriptors 和 session operation descriptors 从 `lib.rs` 物理移入新 helper 文件。
- `lib.rs` 原位置保留 `include!("diagnostics_provider_session_entrypoints.rs")`，继续在 crate root 展开，避免修改 helper 可见性。
- 未修改函数语义、返回值、错误文案、公开 Tauri command 契约或 workflow state schema。

R2-B8 可接受为：diagnostics / provider / continuation / adapter / session operation descriptor 相关 helper 已从 `lib.rs` 抽出，`lib.rs` 行数继续下降，command 总量和 `lib.rs` 内 command 数量保持不变，并通过 shape gate、任务包指定 Rust 聚焦测试、全量库测试和格式检查。

R2-B8 不接受为：R2 全部完成、`lib.rs <= 15,000` 或 `<= 3,000` 目标完成、diagnostics 自动修复完成、provider credential / model verification 完成、planned adapters 真实接入、session continuation 真实 send / resume 新能力完成、runtime log / worker protocol / real execution command / project workflow automation 模块内部重构完成、SQLite 迁移完成、workflow state schema 迁移完成、Stage L / K3-B1 / K3-B2 恢复或新的真实 Codex 执行授权。

## Commit 记录

- Start commit：`385fe41d91cc0738b4e8c0d21c142a903c13ae0c`
- End commit：本文件随 R2-B8 completion commit 一起提交；提交创建后由最终回交记录实际 hash。

## 改动文件

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/diagnostics_provider_session_entrypoints.rs`
- `evidence/2026-06-11-root-treatment-r2-b8-diagnostics-provider-continuation-adapter-boundary-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r2-b8-diagnostics-provider-continuation-adapter-boundary-extraction-v1-result.md`

未修改：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `prototypes/productized-desktop-shell/src-tauri/src/runtime_session_attention.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/runtime_log_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/worker_protocol.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/project_workflow_automation.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs`

说明：任务包列出的 `diagnostic.rs` 当前不存在；diagnostics 目标代码实际位于 `lib.rs` 中，本轮只抽出该连续 helper 块，没有新增或修改 `diagnostic.rs`。

## 抽出清单

以下函数 / 类型已从 `lib.rs` 移入 `diagnostics_provider_session_entrypoints.rs`：

- `derive_diagnostic_summary`
- `workflow_state_integrity`
- `json_file_integrity`
- `text_file_integrity`
- `sidecar_integrity`
- `derived_store_integrity_findings`
- `derive_provider_availability_summaries`
- `provider_availability_for_adapter`
- `provider_kind_for_adapter`
- `derive_session_continuation_previews`
- `active_session_bindings_for_adapter`
- `session_continuation_preview_for_binding`
- `continuation_prompt_source_kind`
- `continuation_prompt_summary`
- `continuation_readback_expectation`
- `continuation_failure_boundary`
- `continuation_audit_impact`
- `inspect_session_continuation_guard`
- `sensitive_path_like`
- `path_within_scope`
- `derive_agent_adapter_descriptors`
- `planned_agent_adapter_descriptors`
- `planned_agent_adapter_descriptor`
- `adapter_capability`
- `SessionOperationSpec`
- `derive_session_operation_descriptors`
- `session_operation_specs`
- `session_operation_descriptor_for_adapter`

本轮未迁移 `build_snapshot` / `build_snapshot_with_session_source`、session loading、index parser、host OS helper、Tauri app assembly、worker_protocol / real_execution_command / project_workflow_automation 模块内部实现或 tests。

## Shape 指标

| 指标 | R2-B8 前 | R2-B8 后 |
| --- | ---: | ---: |
| `lib.rs` 行数 | 18,932 | 17,042 |
| `diagnostics_provider_session_entrypoints.rs` 行数 | 0 | 1,894 |
| Tauri command registry 总量 | 96 | 96 |
| `lib.rs` 内 `#[tauri::command]` 数量 | 0 | 0 |
| Sidecar JSON kinds | 14 allowed / 0 unknown | 14 allowed / 0 unknown |

说明：

- `lib.rs` 较 R2-B7 checkpoint 继续减少 1,890 行。
- 新 helper 文件 1,894 行，低于 Rust 3,000 行治理阈值。
- 本轮选择 crate-root `include!`。原因是抽出块依赖 crate-root private helper、snapshot assembly 调用点和既有 inline tests；正式 `mod` 会要求扩大可见性修改，不符合本任务的行为不变和小风险边界。
- 本轮只搬移 diagnostics / provider / continuation / adapter descriptor / session operation descriptor helper；未拆 snapshot assembly、index parser、host OS helper、Tauri app assembly、SQLite、UI 或 tests 巨石。

## 代码地图摘要

R2-B8 对应当前 `lib.rs` 中 `build_snapshot_with_session_source` 之后、`software_key_of_session` 之前的连续 helper block：

- diagnostics summary 与 store integrity read-model helper。
- provider availability 只读摘要。
- session continuation preview、prompt/readback/failure/audit boundary 和 guard。
- agent adapter descriptor 与 planned adapter descriptor。
- session operation spec / descriptor 派生。

R2-B8 已将上述函数抽入 `diagnostics_provider_session_entrypoints.rs`，`lib.rs` 原位置仅保留 crate-root `include!`。`build_snapshot` / `build_snapshot_with_session_source`、`load_sessions`、`parse_projects` / `parse_sessions` / `parse_tasks`、`copy_to_clipboard`、`run_open` 和 Tauri app assembly 均留在本批次外。

## 验证记录

已运行：

```bash
node scripts/harness/workbench-shape-gate.js --mode baseline
node scripts/harness/workbench-shape-gate.js --mode check
cargo test --lib diagnostic
cargo test --lib provider_availability
cargo test --lib session_continuation
cargo test --lib agent_adapter
cargo test --lib session_operation
cargo test --lib workbench_snapshot
cargo test --lib
cargo fmt -- --check
git diff --check
git status --short
```

结果：

- `node scripts/harness/workbench-shape-gate.js --mode baseline`：通过，Status `pass`，0 errors / 0 warnings / 12 info。
- baseline key metrics：`lib.rs` 17,042 lines；Tauri commands 96 total / 0 in `lib.rs`；Sidecar JSON kinds 14 detected / 0 unknown；`lib.rs` ratchet 17,042 / 25,925，status `decreased`。
- `node scripts/harness/workbench-shape-gate.js --mode check`：通过，Status `pass`，0 errors / 0 warnings / 12 info。
- check key metrics：`lib.rs` 17,042 lines；Tauri commands 96 total / 0 in `lib.rs`；Sidecar JSON kinds 14 detected / 0 unknown；`lib.rs` ratchet 17,042 / 25,925，status `decreased`。
- `cargo test --lib diagnostic`：通过，4 passed / 0 failed / 348 filtered out；保留既有 warning：`JsonRpcError::invalid_params` dead_code。
- `cargo test --lib provider_availability`：通过，1 passed / 0 failed / 351 filtered out；保留同一既有 warning。
- `cargo test --lib session_continuation`：通过，17 passed / 0 failed / 4 ignored / 331 filtered out；ignored 均为显式真实执行授权测试。
- `cargo test --lib agent_adapter`：通过，2 passed / 0 failed / 350 filtered out；保留同一既有 warning。
- `cargo test --lib session_operation`：通过，1 passed / 0 failed / 351 filtered out；保留同一既有 warning。
- `cargo test --lib workbench_snapshot`：通过，1 passed / 0 failed / 351 filtered out；保留同一既有 warning。
- `cargo test --lib`：通过，336 passed / 0 failed / 16 ignored；保留同一既有 warning。
- `cargo fmt -- --check`：通过，无输出。
- `git diff --check`：通过，无输出。
- `git status --short`：提交前仅包含 R2-B8 范围文件。

所有任务包点名 filters 均有匹配测试并通过；没有将无匹配或环境失败冒充通过。

## 边界确认

- 未改产品业务逻辑。
- 未改函数语义、返回值、错误文案或公开 Tauri command 契约。
- 未改 workflow state 顶层 schema。
- 未新增 `#[tauri::command]`。
- 未新增 sidecar store 或 sidecar JSON 种类。
- 未迁移 SQLite。
- 未做 UI。
- 未做 snapshot assembly / index parser / host OS helper / app assembly / worker_protocol / real_execution_command / project_workflow_automation / tests 巨石其他拆分。
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
- P2：本轮只抽出 diagnostics / provider / continuation / adapter / session operation descriptor helper，不代表 diagnostics 自动修复、provider credential / model verification、planned adapters 真实接入、session continuation 真实 send / resume 新能力、SQLite、UI 或 tests 巨石拆分完成。
- P2：相关测试仍主要保留在 `lib.rs` inline tests 中，后续 R2 后段可按领域迁移 tests。
