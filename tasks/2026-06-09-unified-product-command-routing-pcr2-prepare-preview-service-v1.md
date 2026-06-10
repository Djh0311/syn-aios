# Unified Product Command Routing PCR2 Prepare Preview Service v1

日期：2026-06-09

状态：已完成。

后端线任务。本文用于在 PCR1 已完成的统一 Product Command 契约、`real-execution-product-commands.v1.json` sidecar skeleton 和 `WorkbenchSnapshot.real_execution_product_commands` 只读摘要之上，新增统一 prepare / preview 服务。PCR2 仍是 Level A：可以聚合 preview、guard、diagnostics、duplicate、memory packet readiness，并可在安全通过时写入工作台自有 product-command sidecar；但不授权真实 `codex exec` / `codex exec resume`，不发送 prompt，不读写 `/Users/yoyi/.codex`，不新增 UI 执行入口，不同步权威入口。

## 0. 全局主管理解

已知事实：

- PCR0 已冻结方向：新增 `real-execution-product-commands.v1.json` 作为产品层 command / preview / decision / attempt refs sidecar；旧 workflow / machine / canvas entrypoints 继续 legacy / sealed / blocked。
- PCR1 已完成：`RealExecutionProductCommandRequest / Preview / Decision / Attempt / Store / ReadModel`、store empty/load/validate/read model、默认非真实 preview / attempt、WorkbenchSnapshot 只读摘要、TS 类型和离线断言均已接入。
- PCR1 复核线已确认无 P0/P1/P2；`validate_pcr1_attempt_safety` 已拒绝 `writes_project_files=true`。
- 现有 `preview_h5_project_workflow_dispatch_at` 已能输出 H5 Level A preview，并已覆盖 missing memory packet、stale memory、diagnostics degraded、duplicate active、prompt summary/ref/hash missing、resume target missing 等 blocked reasons。
- 当前代码没有 `diagnostics_store.rs`；诊断输入主要来自 `H5DiagnosticSummaryInput`，runtime 诊断摘要在 `runtime_log_store.rs` 中派生。PCR2 不新建 diagnostics store。

本任务假设：

- PCR2 只把现有 H5 / session continuation / diagnostics / duplicate / memory readiness 统一映射到 `RealExecutionProductCommandPreview`。
- `preview_real_execution_product_command` 是只读命令，不写 sidecar。
- `prepare_real_execution_product_command` 可以写 `real-execution-product-commands.v1.json`，但只能写 product command request + preview snapshot，不能写 decision / real attempt / runner result。
- blocked preview 默认不写 sidecar；如开发线认为需要记录 blocked preview，必须先回交主管线，不得自行扩大写入范围。
- 入口文档仍按计划留到 PCR8 或 PCR10 checkpoint 同步；PCR2 不更新 `CURRENT.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`、`tasks/README.md`。

## 1. 目标

PCR2 目标：

1. 新增统一 product command prepare / preview 应用服务。
2. 将现有 `preview_h5_project_workflow_dispatch_at` 输出映射为 `RealExecutionProductCommandPreview`。
3. 将 session continuation preview / guard、diagnostics summary、duplicate guard、memory packet readiness 汇入统一 readiness / permission envelope / guard preview / runtime audit preview。
4. 暴露 Tauri commands：
   - `preview_real_execution_product_command`
   - `prepare_real_execution_product_command`
5. `preview` 永远不写 sidecar，不执行 Codex。
6. `prepare` 仅在 preview 安全通过、无 blocked reasons、store revision 未冲突时写 product command sidecar；仍不执行 Codex。
7. 验证以下 blocked cases：
   - missing memory packet
   - stale memory packet
   - diagnostics degraded / blocking
   - duplicate active attempt
8. PCR2 产物必须为 PCR3 decision / confirmation 服务提供稳定输入：prepared command、preview snapshot、store revision、permission envelope、blocked reasons。

## 2. 非目标

PCR2 不做：

- 不执行真实 `codex exec` / `codex exec resume`。
- 不发送真实 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 secret、token、`.env`、keychain、OAuth、provider credential、full transcript、rollout。
- 不调用 `run_controlled_session_continuation_real_resume_phase_a` / `run_controlled_session_continuation_real_resume_phase_b`。
- 不调用任何 `Command::new("codex")` 路径。
- 不新增真实 runner / retry / auto-retry。
- 不写正式 decision。
- 不写真实 attempt。
- 不写 runtime log 真实执行条目。
- 不把 preview result 包装成 worker running / Codex received task / execution readback。
- 不新增普通 UI 执行按钮。
- 不接 planned adapters 真实执行。
- 不做 PCR3 / PCR4 / PCR5 / PCR9。
- 不修改 `workflow-state.v0.json` 顶层结构。
- 不同步权威入口。

## 3. 文件范围

允许修改：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/h5_project_dispatch_bridge.rs`，仅限抽取或复用安全 preview 映射 helper；不得改变 H5 preview 非真实边界。
- `prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs`，仅限暴露只读 preview / duplicate / guard helper；如无需修改，优先不碰。
- `prototypes/productized-desktop-shell/src-tauri/src/runtime_log_store.rs`，仅限复用或轻量派生 diagnostics summary helper；不得写 runtime execution entry。
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`，仅限 command registration / tests / snapshot warning 轻量串联。
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`，仅补类型 / wrapper / no UI execution 断言。

默认不修改：

- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/views/*`
- `prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `prototypes/productized-desktop-shell/src-tauri/src/codex_local_runner.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/codex_runner.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/commands.rs`
- `CURRENT.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `tasks/README.md`

如确需触碰默认不修改文件，开发线必须先停止并回交理由，由主管线决定是否拆 PCR2.1。

## 4. 后端 API 要求

### 4.1 输入类型

建议新增或补齐：

- `PreviewRealExecutionProductCommandInput`
- `PrepareRealExecutionProductCommandInput`
- `RealExecutionProductCommandPrepareOutput`

最低字段建议：

- `source_kind`
- `h5_dispatch_preview`，类型可复用 `H5ProjectWorkflowDispatchPreviewInput`
- `expected_store_revision`，仅 prepare 必需或可选
- `requested_by`
- `created_at` 可由后端生成，不要求前端传入

`source_kind` PCR2 最低支持：

- `h5_project_workflow_dispatch`

PCR2 可以为未来扩展保留 enum / string，但不得假装非 H5 source 已实现。非支持 source 必须返回明确错误或 blocked preview：

- `unsupported_product_command_source`

### 4.2 Preview command

新增 Tauri command：

- `preview_real_execution_product_command`

行为：

- 读取 workflow state。
- 调用或复用 `preview_h5_project_workflow_dispatch_at`。
- 将 H5 preview 映射为 `RealExecutionProductCommandPreview`。
- 不写 `real-execution-product-commands.v1.json`。
- 不写 `session-continuations.v1.json`。
- 不写 runtime log / audit 正式条目。
- 不调用真实 runner。

必须保持：

- `prompt_sent=false`
- `real_codex_executed=false`
- `writes_codex_home=false`
- `writes_project_files=false`
- `writes_workbench_state=false`
- `readback_boundary.result_count=null` when status is `readback_unavailable` / `readback_failed` / `readback_timed_out` / `timed_out` / equivalent unknown state.

### 4.3 Prepare command

新增 Tauri command：

- `prepare_real_execution_product_command`

行为：

- 先执行与 `preview_real_execution_product_command` 相同的 preview 构造。
- 如果 preview 有 `blocked_reasons`，返回 blocked output，不写 sidecar。
- 如果 preview 安全通过，校验 `expected_store_revision`。
- 将 `RealExecutionProductCommandRequest` 和 `RealExecutionProductCommandPreview` append 到 `real-execution-product-commands.v1.json`。
- 更新 sidecar `revision`、`updated_at`、`last_write_id`。
- 返回更新后的 `RealExecutionProductCommandReadModel`。

必须保持：

- 不写 decision。
- 不写 attempt，除非是明确的 `prepare_blocked_no_runner` fixture 并且主管线批准；默认 PCR2 不写 attempt。
- 不写 runtime log 正式条目。
- 不写 audit 正式条目。
- `preview.prompt_sent=false`
- `preview.real_codex_executed=false`
- `preview.writes_codex_home=false`
- `preview.writes_project_files=false`
- `preview.writes_workbench_state=true` 只允许表示 product command sidecar prepare write；不得表示 workflow-state、project files、Codex home 或 runtime log 写入。

若开发线认为当前 `RealExecutionProductCommandPreview.writes_workbench_state` 不适合区分 preview write 和 prepare write，应优先在 output 层新增 `writes_product_command_sidecar`，不要篡改 preview 本身的语义。

## 5. 映射要求

### 5.1 Request 映射

从 H5 preview input / output 映射到 `RealExecutionProductCommandRequest`：

- `product_command_id`：稳定派生，建议 `real-exec-command:{dispatch_id}` 或带 timestamp 的唯一 id；prepare 重复时要能被 duplicate guard / revision 检测。
- `command_family`：`real_execution_product_command`
- `operation_id`：来自 H5 preview output `operation_id`
- `project_id`、`project_root`、`workflow_id`、`node_id`、`work_item_id`：来自 H5 preview input / output
- `task_package_ref`：来自 H5 preview output `task_package_id`
- `memory_packet_ref`：来自 H5 `memory_packet.snapshot_id` / `fingerprint`
- `adapter_id`：PCR2 仅 `codex-local`
- `session_mode`：由 `operation_id` / `target_session_id` 派生，建议 `resume_existing_session` 或 `new_session_preview_only`
- `target_session_id`：来自 H5 preview output
- `prompt_summary` / `prompt_ref` / `prompt_hash`：来自 H5 preview input
- `allowed_write_roots` / `denied_paths`：来自 H5 permission envelope
- `readback_plan`：来自 H5 readback boundary 或 codex local request readback plan
- `timeout_ms`：如 H5 输入无明确 timeout，可为 `None`
- `requested_by`：来自 input `requested_by` 或 H5 actor id
- `created_at`：后端生成

完整 prompt body 不得进入 product command store。

### 5.2 Preview 映射

从 H5 preview 映射到 `RealExecutionProductCommandPreview`：

- `permission_envelope`：由 H5 permission envelope 派生；必须保留 explicit user confirmation required / approved false / allowed roots / denied paths / risk summary。
- `readiness`：由 H5 status、blocked reasons、guard、diagnostics、memory packet、duplicate 派生。
- `guard_preview`：由 H5 codex local guard 派生；只允许 inspect guard，不允许 run。
- `diagnostics_summary`：由 H5 runtime audit preview diagnostic fields 或 H5 diagnostic input 派生。
- `duplicate_scope`：由 H5 active attempts / duplicate blocked 派生。
- `runtime_log_preview`：仅 preview refs，不写 runtime log。
- `audit_preview`：仅 preview refs，不写 audit。
- `readback_boundary`：unknown / unavailable / failed / timed_out 均保持 `result_count=null`。
- `warnings` / `blocked_reasons`：保留 H5 原始 blocked reason，并追加 PCR2 语义 warning。

### 5.3 Blocked reason 映射

必须覆盖并测试：

- `task_memory_packet_snapshot_missing` -> blocked
- `task_memory_packet_stale` -> blocked
- `diagnostics_blocking_degraded` -> blocked
- `duplicate_dispatch_blocked` -> blocked

建议保留原始 reason，同时可增加归一化 reason：

- `memory_packet_missing`
- `memory_packet_stale`
- `diagnostics_degraded`
- `duplicate_active`

不得把 blocked preview 显示或记录为 ready / running / sent。

## 6. Store 写入要求

`prepare_real_execution_product_command` 写入 sidecar 时：

- 如果 sidecar 不存在，创建 empty store。
- 如果 sidecar JSON 损坏，返回错误，不覆盖。
- 如果 schema_version 不匹配，返回错误，不迁移。
- 如果 `expected_store_revision` 与当前 revision 不一致，返回 conflict。
- append command + preview。
- revision +1。
- 写入必须只落在 `real-execution-product-commands.v1.json`。
- 不修改 `workflow-state.v0.json` 顶层结构。
- 不写 `/Users/yoyi/.codex`。
- 不写 project files。
- 不写 secret / credential 相关路径。

可接受的 prepare output：

- `status=prepared`
- `status=blocked_not_prepared`
- `status=store_conflict`

## 7. Tauri / TS 要求

Rust commands：

- `preview_real_execution_product_command`
- `prepare_real_execution_product_command`

TS wrapper 建议：

- `previewRealExecutionProductCommand`
- `prepareRealExecutionProductCommand`

要求：

- 不在 App / views 中调用。
- 不新增按钮。
- wrapper 只用于后续 PCR3/PCR4 或测试准备。
- 命名必须避免 `run` / `execute` 误导。

## 8. 测试要求

至少新增 / 补齐 Rust 测试：

1. `pcr2_preview_maps_h5_preview_without_real_execution`
   - 输出 `prompt_sent=false`
   - 输出 `real_codex_executed=false`
   - 输出 `writes_codex_home=false`
   - 输出 `writes_project_files=false`
   - preview 不写 sidecar

2. `pcr2_prepare_writes_product_command_sidecar_only_when_ready`
   - ready preview 后 sidecar revision +1
   - commands +1
   - previews +1
   - no decisions / no attempts
   - no runtime log / audit formal write

3. `pcr2_preview_blocks_missing_memory_packet`
   - blocked reasons 包含 missing memory packet
   - 不写 sidecar

4. `pcr2_preview_blocks_stale_memory_packet`
   - blocked reasons 包含 stale memory
   - 不写 sidecar

5. `pcr2_preview_blocks_diagnostics_degraded`
   - blocked reasons 包含 diagnostics degraded
   - 不写 sidecar

6. `pcr2_preview_blocks_duplicate_active`
   - blocked reasons 包含 duplicate active
   - 不写 sidecar

7. `pcr2_prepare_rejects_store_revision_conflict`
   - expected revision 不匹配时返回 conflict
   - 不覆盖 sidecar

8. `pcr2_readback_unknown_keeps_result_count_null`
   - unavailable / failed / timed_out 保持 null

至少补齐前端 / TS 验证：

- `npm run typecheck` 通过。
- 如新增 TS wrapper，离线测试 fixture 不应新增 UI 可执行入口。

## 9. 必跑验证

开发线完成后至少运行：

- `cargo test --lib real_execution_command`
- `cargo test --lib h5_project_dispatch_bridge`
- `cargo test --lib session_continuation`
- `cargo test --lib runtime_log`
- `cargo test --lib`
- `cargo fmt -- --check`
- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`

如某项因环境失败，必须回交失败命令、失败输出摘要和判断，不得静默跳过。

## 10. 必跑扫描

开发线完成后必须扫描并分类：

```bash
rg -n 'Command::new\("codex"\)|codex exec|codex exec resume' prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs prototypes/productized-desktop-shell/src-tauri/src/commands.rs prototypes/productized-desktop-shell/src-tauri/src/h5_project_dispatch_bridge.rs prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs prototypes/productized-desktop-shell/src/lib/tauri.ts
```

```bash
rg -n 'prompt_sent:\s*true|real_codex_executed:\s*true|writes_codex_home:\s*true|writes_project_files:\s*true' prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs prototypes/productized-desktop-shell/src-tauri/src/h5_project_dispatch_bridge.rs prototypes/productized-desktop-shell/src-tauri/src/types.rs
```

```bash
rg -n 'previewRealExecutionProductCommand|prepareRealExecutionProductCommand|real_execution_product_command' prototypes/productized-desktop-shell/src/App.tsx prototypes/productized-desktop-shell/src/views prototypes/productized-desktop-shell/src/components
```

```bash
rg -n '已发送|正在执行|Codex 已收到|执行完成|结果数：0|结果数：空|允许一次' prototypes/productized-desktop-shell/src
```

预期：

- 新增 PCR2 文件不得新增真实 runner 调用。
- App / views / components 不应出现 PCR2 wrapper 调用。
- 误导文案无新增命中；若命中测试 fixture 或边界说明，必须分类。

## 11. 回交要求

开发线完成后中文回交：

1. 修改文件列表。
2. 新增 input/output/command/helper 摘要。
3. `preview` 和 `prepare` 的写入边界。
4. 四类 blocked case 的测试结果。
5. 明确没有真实执行、没有 prompt 发送、没有 `.codex` 读写、没有 UI 执行入口。
6. 必跑验证结果。
7. 扫描结果和命中分类。
8. PCR3 可以接续的输入。
9. 不能声明完成的事项。

开发线不得自行把本文状态改为“已完成”。开发线可将状态改为“待主管复核”，并在文末追加“开发线执行结果草稿”。最终完成状态由主管线 fresh verify + 复核线只读审查后再写。

## 12. 不得声明

PCR2 完成后仍不得声明：

- 统一 Product Command Routing 已完整实现。
- 通用真实 send / resume 产品化完成。
- 真实 execute 已可用。
- H5 通用真实派发已开放。
- PCR3 decision / confirmation 已完成。
- PCR4 fake execute 已完成。
- PCR5 legacy 入口迁移已完成。
- PCR9 Level B 已授权。
- planned adapters 已真实接入。
- provider credential / model verification 完成。
- 任意项目自由执行。

## 13. 开发线执行结果草稿

执行线：product-line 长期开发线。

状态：主管 fresh verify 与复核线只读审查通过。

本轮实现范围：

- 新增 `PreviewRealExecutionProductCommandInput`、`PrepareRealExecutionProductCommandInput`、`RealExecutionProductCommandPrepareOutput`。
- 新增 `preview_real_execution_product_command` / `prepare_real_execution_product_command` Tauri command。
- 新增 `previewRealExecutionProductCommand` / `prepareRealExecutionProductCommand` TS wrapper；未接入 `App.tsx`、views 或 components。
- 将 H5 Level A preview 映射为 `RealExecutionProductCommandPreview`。
- `preview` 只读，不写 `real-execution-product-commands.v1.json`。
- `prepare` 仅在 preview 无 blocked reasons 且 store revision 不冲突时 append command + preview snapshot。
- blocked preview 不写 sidecar；revision conflict 不写 sidecar。
- prepare 不写 decision、attempt、runtime execution log 或 formal audit。

边界确认：

- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript / rollout。
- 未新增 UI 执行入口。
- 未同步 `CURRENT.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`、`tasks/README.md`。

开发线验证摘要：

- `cargo test --lib real_execution_command`：15 passed。
- `cargo test --lib h5_project_dispatch_bridge`：4 passed。
- `cargo test --lib session_continuation`：17 passed，4 ignored。
- `cargo test --lib runtime_log`：5 passed。
- `cargo test --lib`：284 passed，5 ignored。
- `cargo fmt -- --check`：passed。
- `npm run typecheck`：passed。
- `npm run test:offline-interaction`：offline interaction tests passed: 13。
- `npm run build`：passed；仅 Vite chunk size warning。

扫描分类摘要：

- PCR2 新增 `real_execution_command.rs` / `commands.rs` / `h5_project_dispatch_bridge.rs` / `src/lib/tauri.ts` 未新增真实 runner 调用。
- `prompt_sent:true` / `real_codex_executed:true` / `writes_codex_home:true` / `writes_project_files:true` 在 PCR2 限定新增契约文件扫描无命中。
- `App.tsx` / views / components 无 `previewRealExecutionProductCommand` / `prepareRealExecutionProductCommand` / `real_execution_product_command` 命中。
- 文案扫描命中为既有 `AgentView.tsx` stub 状态文案和 `canvasSurfaceBoundaries.ts` 禁用词列表，不是 PCR2 新增执行入口或新增误导文案。

## 14. 主管收口结论

结论：PCR2 已完成，可接受为“统一 Product Command Routing prepare / preview 服务代码完成，等待 PCR8 或 PCR10 checkpoint 再统一同步入口文档”。

主管 fresh verify 已重跑：

- `cargo test --lib real_execution_command`：15 passed。
- `cargo test --lib h5_project_dispatch_bridge`：4 passed。
- `cargo test --lib session_continuation`：17 passed / 4 ignored，ignored 仍为显式真实授权探针。
- `cargo test --lib runtime_log`：5 passed。
- `cargo test --lib`：284 passed / 5 ignored。
- `cargo fmt -- --check`：通过。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：13 passed。
- `npm run build`：通过，仅保留既有 Vite chunk size warning。

主管 fresh scan 结论：

- `Command::new("codex")|codex exec|codex exec resume`：无 PCR2 新增 runner 调用；命中均为既有 `session_continuation_store.rs` preview 文案、测试断言或历史真实授权说明。
- `prompt_sent:true|real_codex_executed:true|writes_codex_home:true|writes_project_files:true`：PCR2 限定契约文件无命中。
- `App.tsx` / views / components 中 `previewRealExecutionProductCommand|prepareRealExecutionProductCommand|real_execution_product_command`：无命中。
- 误导文案扫描命中既有 `AgentView.tsx` 的“桩执行完成”和 `canvasSurfaceBoundaries.ts` 禁用词列表“Codex 已收到任务”，不是 PCR2 新增入口或新增文案。

复核线只读结论：

- P0：无。
- P1：无。
- P2：无必须修补项。
- 可选后续建议：后续可补一个显式“损坏 JSON 不覆盖”的单测；当前实现已通过 sidecar load 阶段 `serde_json::from_str` 失败直接返回错误支撑“不覆盖”，但测试名中未单独覆盖。

最终边界：

- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript / rollout。
- 未启动 Browser / Chrome / Tauri / Vite dev / screenshot。
- 未新增 UI 执行入口。
- 未同步 `CURRENT.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`、`tasks/README.md`。
