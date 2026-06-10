# Root Treatment R2-B8 Diagnostics Provider Continuation Adapter Boundary Extraction v1 Result

日期：2026-06-11

## 结论

R2-B8 本轮可接受为：任务包点名的 diagnostics、provider availability、session continuation preview / guard、agent adapter descriptors 和 session operation descriptors 已从 `lib.rs` 物理抽出到 `diagnostics_provider_session_entrypoints.rs`，并通过必需验证。行为保持不变，command 总量仍为 96，`lib.rs` 内 `#[tauri::command]` 仍为 0。

不接受为：R2 全部完成、`lib.rs <= 15,000` 水位线完成、diagnostics 自动修复、provider credential / model verification、planned adapters 真实接入、session continuation 真实 send / resume 新能力、runtime log / worker protocol / real execution command / project workflow automation 模块内部重构、SQLite 迁移、workflow state schema 迁移或真实 Codex 执行恢复。

## 已完成

- 新增 `prototypes/productized-desktop-shell/src-tauri/src/diagnostics_provider_session_entrypoints.rs`。
- 将 diagnostics / store integrity / provider availability / session continuation preview and guard / adapter descriptor / session operation descriptor helper 从 `lib.rs` 搬到新 helper 文件。
- `lib.rs` 原位置改为 `include!("diagnostics_provider_session_entrypoints.rs")`。
- 新增 R2-B8 evidence：`evidence/2026-06-11-root-treatment-r2-b8-diagnostics-provider-continuation-adapter-boundary-extraction-v1.md`。
- 未同步入口文档。

## 改动文件

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/diagnostics_provider_session_entrypoints.rs`
- `evidence/2026-06-11-root-treatment-r2-b8-diagnostics-provider-continuation-adapter-boundary-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r2-b8-diagnostics-provider-continuation-adapter-boundary-extraction-v1-result.md`

## 抽出函数 / 类型

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

- `lib.rs`：18,932 lines -> 17,042 lines。
- `diagnostics_provider_session_entrypoints.rs`：新增 1,894 lines。
- Tauri command registry：96 total -> 96 total。
- `lib.rs` 内 `#[tauri::command]`：0 -> 0。
- Sidecar JSON kinds：14 detected / 0 unknown。

## 验证

已通过：

- `node scripts/harness/workbench-shape-gate.js --mode baseline`：pass，0 errors / 0 warnings；`lib.rs` 17,042 / 25,925，status `decreased`；Tauri commands 96 total / 0 in `lib.rs`。
- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors / 0 warnings；`lib.rs` 17,042 / 25,925，status `decreased`；Tauri commands 96 total / 0 in `lib.rs`。
- `cargo test --lib diagnostic`：4 passed / 0 failed / 348 filtered out。
- `cargo test --lib provider_availability`：1 passed / 0 failed / 351 filtered out。
- `cargo test --lib session_continuation`：17 passed / 0 failed / 4 ignored / 331 filtered out。
- `cargo test --lib agent_adapter`：2 passed / 0 failed / 350 filtered out。
- `cargo test --lib session_operation`：1 passed / 0 failed / 351 filtered out。
- `cargo test --lib workbench_snapshot`：1 passed / 0 failed / 351 filtered out。
- `cargo test --lib`：336 passed / 0 failed / 16 ignored。
- `cargo fmt -- --check`：通过。
- `git diff --check`：通过。
- `git status --short`：提交前仅包含 R2-B8 范围文件。

已知 warning：

- Rust 测试仍有既有 dead_code warning：`JsonRpcError::invalid_params` 未使用。

## Commit 记录

- Start commit：`385fe41d91cc0738b4e8c0d21c142a903c13ae0c`
- End commit：本文件随 R2-B8 completion commit 一起提交；提交创建后由最终回交记录实际 hash。

## 边界

- 未同步 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`。
- 未改产品业务逻辑。
- 未改函数语义、返回值、错误文案、公开 Tauri command 契约或 workflow state schema。
- 未新增 Tauri command。
- 未新增 sidecar / sidecar JSON kind。
- 未迁移 SQLite。
- 未做 UI。
- 未做 snapshot assembly / index parser / host OS helper / app assembly / worker_protocol / real_execution_command / project_workflow_automation / tests 巨石其他拆分。
- 未执行真实 Codex。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript 或 rollout。
- 未启动 Tauri / Browser / Chrome / Vite / 截图工具。

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：`include!` 是保守过渡，后续 R2 可再收敛为正式模块边界。
- P2：本轮只抽出 diagnostics / provider / continuation / adapter / session operation descriptor helper，不代表 diagnostics 自动修复、provider credential / model verification、planned adapters 真实接入、session continuation 真实 send / resume 新能力、SQLite、UI 或 tests 巨石拆分完成。
- P2：相关测试仍主要留在 `lib.rs` inline tests，后续 R2 后段再迁移 tests。
