# Root Treatment / R2-T11 Rust Task Package Dispatch Preparation Test Extraction v1

日期：2026-06-12

状态：已完成并复核通过，checkpoint 已同步。

Planning baseline commit：`d45fd0adaf437eecfbbb1258ff0f095b502b45f1`

Task package commit：`9e1b6abbf46c7203cb90b906900226d9e3a592fe`

Task package hash backfill commit：`5ea9fba80b609b49fee110e32363e44f137c9d18`

Implementation commit：`6a0640a9088445f196282f8c0b657c4ec079872b`

P2 fix commit：`e07fe3411b1a18e01ffff43c9b9443c39dcbdb9a`

Authority sync commit：`fe7f40b3a94fce0227bcc28c4380942e9ad2aadc`

Review result：`CLEAR`；复核线程 `019ebb31-ccb7-7072-b105-6b80f37b997f`；P0/P1/P2 无。

本文是 Root Treatment / Stage R 的 R2-T11 任务包，承接 R2-T10 和 2026-06-12 新策略，只迁移能实际降低 `lib.rs` 棘轮指标的低风险 Rust inline tests。

## 1. 目标

把 `prototypes/productized-desktop-shell/src-tauri/src/lib.rs` 中任务包生成 / 任务记忆注入 / dispatch readiness / field correction 相关 inline tests 迁出到 crate-root test include 文件，继续降低 `lib.rs` 历史最低水位线。

目标文件：

- 新增：`prototypes/productized-desktop-shell/src-tauri/src/lib_task_package_dispatch_preparation_tests.rs`
- 修改：`prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- 修改：`scripts/harness/workbench-shape-gate.js`

预计收益：

- `lib.rs` 预计下降约 1,400-1,500 行。
- 新增 `.rs` test include 预计低于 3,000 行。
- 该切片降低 `lib.rs` 棘轮指标，符合新策略“不得立项不降低棘轮指标的拆分包”。

## 2. 允许范围

允许迁移以下 tests，保持测试体和断言语义不变：

- `workflow_run_check_blocks_missing_workflow_and_missing_required_fields`
- `workflow_run_check_allows_runnable_fixture_without_auto_filling_optional_refs`
- `task_package_blocks_missing_report_model_and_stale_after_edit`
- `task_memory_injection_writes_snapshot_to_task_package_artifact`
- `task_memory_injection_markdown_and_dispatch_prompt_use_same_snapshot`
- `task_memory_injection_marks_snapshot_stale_on_store_revision_change`
- `task_memory_injection_blocks_readiness_when_required_snapshot_missing`
- `task_memory_injection_excludes_lint_blocked_formal_memory`
- `task_package_fields_update_rejects_non_index_project`
- `task_package_fields_update_rejects_missing_state_file`
- `task_package_fields_update_rejects_missing_workflow`
- `task_package_fields_update_rejects_missing_work_item`
- `task_package_fields_update_rejects_missing_task_package_artifact`
- `task_package_fields_update_writes_structured_fields_backup_and_audit`
- `task_package_fields_update_keeps_empty_fields_as_missing_facts`
- `task_package_file_generation_rejects_non_index_project`
- `task_package_file_generation_rejects_missing_state_file`
- `task_package_file_generation_rejects_missing_workflow`
- `task_package_file_generation_rejects_missing_work_item`
- `task_package_file_generation_rejects_missing_task_package_artifact`
- `task_package_file_generation_writes_file_updates_artifact_and_audit`
- `task_package_file_generation_uses_suffix_without_overwriting_existing_file`
- `task_package_file_generation_keeps_missing_fields_as_placeholders`
- `task_package_dispatch_readiness_flags_polluted_generated_draft_as_not_ready`
- `task_package_dispatch_readiness_rejects_missing_fields`
- `task_package_dispatch_readiness_rejects_conflicting_generation_ban`
- `task_package_dispatch_readiness_can_be_ready_after_field_fix_and_file_generation`
- `dispatch_field_correction_rejects_non_index_project`
- `dispatch_field_correction_rejects_missing_state_file`
- `dispatch_field_correction_rejects_missing_workflow_work_item_and_artifact`
- `dispatch_field_correction_backs_up_writes_audit_keeps_path_and_rechecks_ready`
- `dispatch_field_correction_keeps_empty_fields_missing`

允许：

- 在 `lib.rs` 原位置插入 `include!("lib_task_package_dispatch_preparation_tests.rs");`
- 让共享 helper 继续留在 `lib.rs`，包括 `fixture_project`、`fixture_task_draft_request`、`fixture_task_preview_request`、`fixture_fields_update_request`、`ready_fields_update_request`、`fixture_task_file_generation_request`、`fixture_dispatch_readiness_request`、`setup_task_memory_injection_fixture`、`mark_task_package_fixture_ready` 和 `fixture_dispatch_correction_request` 等。
- 更新 shape gate 中 `lib.rs` waterline 为本轮完成后的历史最低收口值。
- 新增 evidence / handoff。

## 3. 禁止范围

禁止：

- 修改产品函数签名、可见性、语义或断言口径。
- 迁移 helper / fixture builder / stub runner / runner fake。
- 迁移 K3-B runtime prompt guard、workflow node dispatch execute/readback、workflow execution runner、workflow machine、offline role dispatch、ignored real-state tests、cross-store memory adoption、memory candidate adoption、formal memory adoption、legacy dispatch execution 或真实 execution tests。
- 修改 task package / task memory injection / dispatch readiness 产品语义。
- 修改 workflow state JSON schema、Tauri command、DB schema 或 sidecar schema。
- 修改前端 UI / CSS / TS。
- 执行真实 `codex exec` / `codex exec resume`、发送 prompt、读写 `/Users/yoyi/.codex`。
- 启动 Tauri / Browser / Chrome / Vite / 截图工具。

## 4. 验收

必须通过：

- `cargo test --lib workflow_run_check`
- `cargo test --lib task_package_blocks`
- `cargo test --lib task_memory_injection`
- `cargo test --lib task_package_fields_update`
- `cargo test --lib task_package_file_generation`
- `cargo test --lib task_package_dispatch_readiness`
- `cargo test --lib dispatch_field_correction`
- `cargo test --lib`
- `cargo fmt -- --check`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`
- 复核线只读审查，结论不得有 P0/P1。

## 5. 不接受为

本任务不接受为：

- `lib.rs <= 3,000`
- R2 全部完成
- R3 Level B 执行
- 生产 SQLite 迁移 / read-cut / stop-write
- 多 agent 并行真实执行解锁
- 真实 Codex 执行
- task package / task memory injection / dispatch readiness 产品语义变更或新增能力
- workflow node dispatch execute/readback 迁移完成
- workflow execution runner / workflow machine / K3-B guard 迁移完成
- memory candidate adoption 迁移完成
- formal memory adoption 迁移完成
- UI / 产品行为修改
- backlog 功能解冻

## 6. 执行记录

本轮已完成实现、本地验证、P2 whitespace 修补和复核线只读审查。

实际改动：

- 新增 `prototypes/productized-desktop-shell/src-tauri/src/lib_task_package_dispatch_preparation_tests.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs` 原测试块替换为 `include!("lib_task_package_dispatch_preparation_tests.rs");`
- `scripts/harness/workbench-shape-gate.js` 的 `lib.rs` waterline 更新为 `6544`

实际形状收益：

- `lib.rs`：`8045 -> 6544`，下降 `1501` 行。
- 新增 include 文件：`1503` 行，低于 `.rs` 新文件上限 `3000`。
- 旧测试块与新 include 字节级一致；P2 fix 只删除末尾空白。

验证已通过：

- `cargo test --lib workflow_run_check`：2 passed。
- `cargo test --lib task_package`：29 passed，1 ignored。
- `cargo test --lib task_memory_injection`：5 passed。
- `cargo test --lib dispatch_field_correction`：5 passed。
- `cargo test --lib`：471 passed，16 ignored。
- `cargo fmt -- --check`：通过。
- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors，0 warnings。
- `git diff --check`：通过。
- `git diff --check 5ea9fba80b609b49fee110e32363e44f137c9d18..HEAD`：通过。

复核结论：

- `STATUS: CLEAR`
- 复核线程：`019ebb31-ccb7-7072-b105-6b80f37b997f`
- P0/P1/P2：无。
- 复核确认新 include 只包含任务包允许的 32 个 tests；禁止关键词扫描无命中；shape gate waterline `6544` 与当前 `wc -l lib.rs` 一致。
