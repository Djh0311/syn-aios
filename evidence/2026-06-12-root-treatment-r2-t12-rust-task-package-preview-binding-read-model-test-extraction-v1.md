# Evidence: Root Treatment / R2-T12 Rust Task Package Preview Binding Read Model Test Extraction v1

日期：2026-06-12

Supervisor：Claude（claude-fable-5，临时代班，依据 `handoffs/2026-06-12-supervisor-line-temporary-takeover-codex-to-claude-v1.md`）

状态：已完成并通过复核线 `STATUS: CLEAR`；用户已放行，commit 序列执行中，checkpoint 同步随后。

任务包：`tasks/2026-06-12-root-treatment-r2-t12-rust-task-package-preview-binding-read-model-test-extraction-v1.md`

Planning baseline commit：`435c21471ad056bd7ed1b44681ad52f285883b5c`

Takeover docs commit：`6fba75ef3facbaf03f81f19c3f235df5981b2875`

Task package commit：`7d5333936b46c2532c4811c70c99233e31125b9d`

Implementation commit：`a3fce1f7385616bae3d0b19a1ec0907b5943ea47`

Review result：`CLEAR`；复核结论文件 `evidence/2026-06-12-root-treatment-r2-t12-rust-task-package-preview-binding-read-model-test-extraction-v1-review-claude-v1.md`（Reviewer：claude-opus-4-8，复核线临时代班）；P0/P1/P2 无；详见第 9 节

## 1. 本轮目标

按 2026-06-12 执行策略继续 R2 后段 inline tests 迁移，只抽能降低 `lib.rs` 棘轮指标的低风险测试切片。本轮迁移 work item state update / workflow node session binding / task package preview / task package read model / project blackboard read model 相关 Rust inline tests（11 个，`lib.rs` 3411-3949 连续区段），并完成 checkpoint 要求的 R2-T12 候选切片评估（全量分类记录见任务包第 6 节）。

## 2. 改动范围

修改：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `scripts/harness/workbench-shape-gate.js`

新增：

- `prototypes/productized-desktop-shell/src-tauri/src/lib_task_package_preview_binding_read_model_tests.rs`
- `tasks/2026-06-12-root-treatment-r2-t12-rust-task-package-preview-binding-read-model-test-extraction-v1.md`
- `evidence/2026-06-12-root-treatment-r2-t12-rust-task-package-preview-binding-read-model-test-extraction-v1.md`
- `handoffs/2026-06-12-root-treatment-r2-t12-rust-task-package-preview-binding-read-model-test-extraction-v1-result.md`

本轮未修改：

- 产品函数签名、可见性或行为。
- task package / task memory injection / dispatch readiness 产品语义或断言口径。
- helper / fixture builder / stub runner / runner fake。
- K3-B runtime prompt guard、workflow node dispatch prepare/execute/readback、workflow execution runner、workflow machine、offline role dispatch、ignored real-state tests、cross-store memory adoption、memory candidate adoption、formal memory adoption、legacy dispatch execution 或真实 execution tests。
- Tauri command、DB schema、sidecar schema、workflow state JSON schema。
- 前端 UI / CSS / TS。

## 3. 实现说明

- 将任务包允许的 11 个 tests 原样迁入 `lib_task_package_preview_binding_read_model_tests.rs`（字节级提取，不改缩进、不改断言）。
- 在 `lib.rs` 原位置保留 `include!("lib_task_package_preview_binding_read_model_tests.rs");`，测试仍运行在 crate-root `tests` module 内，不扩大生产函数可见性。
- 共享 helper 继续留在 `lib.rs`，包括 `fixture_project`、`fixture_session`、`fixture_task_draft_request`、`fixture_work_item_state_update_request`、`fixture_node_session_bind_request`、`fixture_node_session_unbind_request`、`fixture_task_preview_request`、`fixture_task_file_generation_request`、`ready_fields_update_request`、`mark_task_package_fixture_ready`、`append_fixture_dispatch` 等。
- 将 shape gate `lib.rs` waterline 从 `6544` 更新为 `6006`。

## 4. 形状收益

- `lib.rs`：`6544 -> 6006`，下降 `538` 行。
- 新增 `lib_task_package_preview_binding_read_model_tests.rs`：`539` 行，低于 `.rs` 新文件上限 `3000`。
- shape gate 输出：`lib.rs: 6006/6006 (same)`，0 errors，0 warnings。
- 字节级对比：从 planning baseline（HEAD `435c214`）的旧 `lib.rs` 提取 3411-3949 区段后与新 include 文件 `cmp` 完全一致，`byte_exact_match: True`；旧块 `539` 行，新 include `539` 行；新文件 EOF 为单一换行、无尾部空白（T11 P2 同类问题已前置规避）。

## 5. 验证

已通过：

- `cargo test --lib work_item_state_update`：3 passed，0 failed（含本切片 1 个；另 2 个为 R2-T5 已迁的同前缀既有测试）。
- `cargo test --lib workflow_node_session_binding`：2 passed，0 failed。
- `cargo test --lib task_package_preview`：6 passed，0 failed。
- `cargo test --lib workflow_task_package_read_model`：1 passed，0 failed。
- `cargo test --lib project_blackboard_read_model`：1 passed，0 failed。
- `cargo test --lib`：471 passed，16 ignored，0 failed（与 R2-T11 收口基线一致）。
- `cargo fmt -- --check`：通过。
- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors，0 warnings。
- `git diff --check`：通过。

保留既有 warning：

- `src/mcp/protocol.rs` 中 `JsonRpcError::invalid_params` dead_code warning。本轮未触碰该文件。

## 6. 范围扫描

已扫描：

- `grep -nE "workflow_node_dispatch_execute_|workflow_machine|memory_candidate_adoption_|formal_memory_adoption_|k3_b_|stub|runner|real_state|prepare_offline_role_dispatch|record_offline_role|CodexResumeRunner" lib_task_package_preview_binding_read_model_tests.rs`：无命中。
- 新 include 没有迁移 helper / fixture builder；helper 仍留在 `lib.rs`。
- 禁迁区段确认未触碰：K3-B guard（1757、1770）、`reads_real_static_index_summary`（1782）、memory adoption / formal memory store 带（2943-3409）、dispatch prepare/execute/readback 带（迁移后紧随新 include 之后）、workflow machine、offline role、ignored real-state test 全部原位保留在 `lib.rs`。

## 7. 边界确认

本轮没有：

- 执行真实 `codex exec` / `codex exec resume`。
- 发送 prompt。
- 读写 `/Users/yoyi/.codex`。
- 读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript。
- 启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 修改 UI / CSS / TS。
- 修改 Tauri command、DB/schema、sidecar schema、workflow state JSON schema。

过程备注：

- 本包为主管线临时代班（Codex → Claude）首包。代班会话开机按 takeover handoff 顺序完成权威文档收口阅读后才开工。
- 一次 `git commit`（接管文档入库）被 harness 权限层按 AGENTS.md commit 确认规则拦截；据此调整为本回合完成全部离线工作，commit 序列在收口报告中一次性向用户申请确认后执行。该拦截没有产生任何仓库变更。
- 实现期间用户更新主管线接管档案 §5 并新增复核线职位档案：复核改由独立复核线会话（claude-opus-4-8，跨模型）承担，触发权在用户。主管线在档案更新前按旧版 §5 自派的同模型只读审查降级记录为主管线自查（其结论 `CLEAR_WITH_P2`：P2-1 计数笔误 32→33 已修正；P2-2 接管文档申报完整性已通过把复核线职位档案列入 commit 计划解决），不充当正式复核。主管线曾因旧版档案上下文短暂误判复核线职位档案不存在，经重读文件系统纠正；预防：收口各步骤前重读权威档案最新状态。

## 8. 不接受为

本轮不接受为：

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

## 9. 复核结论

复核线只读复核已通过，用户已放行：

- 复核结论文件：`evidence/2026-06-12-root-treatment-r2-t12-rust-task-package-preview-binding-read-model-test-extraction-v1-review-claude-v1.md`（Reviewer：claude-opus-4-8，复核线临时代班，依据 `handoffs/2026-06-12-review-line-temporary-takeover-claude-v1.md`）。
- 最终结论：`STATUS: CLEAR`；P0/P1/P2：无。
- 复核线独立重跑脚本门（非转述本 evidence）：`cargo test --lib` 471 passed / 0 failed / 16 ignored 与基线一致；定向过滤 3/2/6/1/1 全过；`cargo fmt -- --check`、shape gate（0 errors，`lib.rs: 6006/6006 (same)`）、`git diff --check` 全过。
- 复核确认：删除块与新 include 字节级一致（BYTE_EXACT_MATCH）；迁移名单 = 任务包 §2 白名单 11 个，无多无少；`#[test]` 守恒 55 = 44 + 11；禁迁关键词扫描零命中；`lib.rs` 仅"删测试块 + 插 include"，产品代码零改动；waterline 正确锁到历史新低 6006。
- 复核确认任务包 §6 计数笔误（32→33）已修正且三处一致，无 P 级问题。

主管线自查记录（不充当正式复核）：自查 agent 独立重跑全部脚本门通过（cargo test --lib 471/0/16、fmt、shape gate 0 errors、git diff --check），核对 11 个 tests 无多无少、字节级一致、禁迁关键词零命中、lib.rs diff 仅测试块删除 + include 插入、waterline 6006 与实测一致；自查结论 `CLEAR_WITH_P2`，两条 P2（计数笔误、接管文档申报完整性）均已处理。
