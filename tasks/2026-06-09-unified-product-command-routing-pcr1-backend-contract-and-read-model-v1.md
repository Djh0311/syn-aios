# Unified Product Command Routing PCR1 Backend Contract And Read Model v1

日期：2026-06-09

状态：已完成。

后端线任务。本文用于在 PCR0 入口矩阵和主管决策冻结后，建立统一 Product Command Routing 的后端契约、只读模型和 Level A 测试底座。本文不授权真实 `codex exec` / `codex exec resume`，不授权发送 prompt，不授权读写 `/Users/yoyi/.codex`，不接 UI 执行按钮，不同步权威入口。

## 0. 全局主管理解

已知事实：

- PCR0 任务包已经冻结主要入口矩阵，补充了 `inspect_controlled_session_continuation_real_resume_authorization`、`run_controlled_session_continuation_real_resume_phase_a`、H3-B internal / ignored probe path 的分类。
- 旧 `execute_workflow_node_dispatch`、`run_workflow_machine`、`read_workflow_node_dispatch_result`、`__run_workflow_machine_real`、`canvas_start_run`、`canvas_tick_run` 默认仍是 legacy / sealed / blocked，不是统一 product command。
- H2 Phase B 和 H3-B runner 是真实执行 adapter 路径；PCR1 不调用它们，只定义契约和读模型。
- H5 preview 仍必须保持 `prompt_sent=false`、`real_codex_executed=false`、`writes_codex_home=false`。

本任务假设：

- PCR0 最终收口无 P0/P1 后才能执行本任务。
- PCR0 默认主管决策采用新增 `real-execution-product-commands.v1.json` sidecar；如 PCR0 最终改变决策，PCR1 必须先更新本文再执行。
- PCR1 只做 Level A：类型、store skeleton、read model、测试，不做真实执行、不做 UI 主链路接入。

## 1. 目标

PCR1 目标：

1. 定义统一真实执行产品命令的 Rust / TS 类型。
2. 新增或准备 `real-execution-product-commands.v1.json` sidecar 的 store 契约，包含 revision、command、preview、decision、attempt refs。
3. 在 `WorkbenchSnapshot` 中输出 product command readiness / boundary summary，只读展示当前能力和阻断原因。
4. 保持旧入口 blocked，不新增任何真实 runner 调用。
5. 为 PCR2 prepare / preview 服务、PCR3 decision 服务、PCR4 fake execute 链路提供稳定输入。

## 2. 非目标

PCR1 不做：

- 不执行真实 `codex exec` / `codex exec resume`。
- 不发送 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 secret、token、`.env`、keychain、OAuth、provider credential、full transcript、rollout。
- 不迁移 `workflow-state.v0.json` 顶层结构。
- 不把旧入口改成可执行。
- 不接真实 UI 执行按钮。
- 不做 PCR2 prepare / preview 聚合服务。
- 不做 PCR3 用户确认写入服务。
- 不做 PCR4 fake execute。
- 不做 PCR9 Level B 真实探针。

## 3. 文件范围

允许修改：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`，仅用于注册只读 snapshot / boundary 所需命令或类型串联；不得改旧真实执行 blocked wrapper。
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`，仅同步类型 / 只读 wrapper；不得新增 execute UI wrapper。
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`，仅补类型 / read model 离线断言。

默认不修改：

- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/codex_local_runner.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/codex_runner.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/commands.rs`

如确需触碰默认不修改文件，后端线必须先回交原因，由全局主管决定是否拆 PCR1.1。

## 4. 后端契约要求

### 4.1 Command Request

新增或补齐 `RealExecutionProductCommandRequest`，最低字段：

- `product_command_id`
- `command_family`
- `operation_id`
- `project_id`
- `project_root`
- `workflow_id`
- `node_id`
- `work_item_id`
- `task_package_ref`
- `memory_packet_ref`
- `adapter_id`
- `session_mode`
- `target_session_id`
- `prompt_summary`
- `prompt_ref`
- `prompt_hash`
- `allowed_write_roots`
- `denied_paths`
- `readback_plan`
- `timeout_ms`
- `requested_by`
- `created_at`

要求：

- `prompt_summary` 可以进入 read model；完整 prompt body 不进入 PCR1 store。
- `prompt_hash` 是边界字段；PCR1 不计算真实 prompt body hash，除非已有输入可安全派生。
- `allowed_write_roots` 和 `denied_paths` 必须显示，不得默认为任意项目自由写。

### 4.2 Preview

新增或补齐 `RealExecutionProductCommandPreview`，最低字段：

- `request`
- `permission_envelope`
- `readiness`
- `guard_preview`
- `diagnostics_summary`
- `duplicate_scope`
- `runtime_log_preview`
- `audit_preview`
- `readback_boundary`
- `warnings`
- `blocked_reasons`
- `prompt_sent=false`
- `real_codex_executed=false`
- `writes_codex_home=false`
- `writes_project_files=false`
- `writes_workbench_state=false`

要求：

- PCR1 preview 只是契约 / read model 占位，不执行 Codex。
- `readback_boundary.result_count` 对 unavailable / failed / timed_out 必须是 `null`，不能写 0。

### 4.3 Decision

新增或补齐 `RealExecutionProductCommandDecision`，最低字段：

- `decision_id`
- `product_command_id`
- `decision`
- `confirmed_by`
- `confirmed_at`
- `store_revision`
- `risk_acknowledgement`
- `allowed_once`
- `reason`

要求：

- PCR1 只定义类型和可测试校验函数，不新增真实 confirmation command。
- 高影响真实执行必须 `confirmed_by=user` 的规则要进入契约测试。
- `confirmed_by=project_director` 不能替代用户确认。

### 4.4 Attempt

新增或补齐 `RealExecutionProductCommandAttempt`，最低字段：

- `attempt_id`
- `product_command_id`
- `continuation_id`
- `adapter_id`
- `operation_id`
- `status`
- `started_at`
- `completed_at`
- `runner_call_allowed`
- `prompt_sent`
- `real_codex_executed`
- `writes_codex_home`
- `writes_project_files`
- `runtime_log_ref`
- `audit_refs`
- `readback_summary`
- `failure_reason`
- `warnings`

要求：

- PCR1 attempts 可以是 fixture / blocked / no-op contract test；不得写真实 runner result。
- `runner_call_allowed=false` 是 PCR1 默认安全态。

### 4.5 Store

如沿用 PCR0 默认决策，新增 `RealExecutionProductCommandStore` 契约：

- `schema_version`
- `revision`
- `created_at`
- `updated_at`
- `last_write_id`
- `commands`
- `previews`
- `decisions`
- `attempts`
- `audit_refs`
- `warnings`

Sidecar 文件名：

- `real-execution-product-commands.v1.json`

要求：

- PCR1 可以新增 store load / empty / validate / summary 函数。
- PCR1 不要求写入真实 sidecar；如实现写入测试，只能在临时 fixture 路径。
- 不修改 `workflow-state.v0.json` 顶层结构。

## 5. WorkbenchSnapshot 读模型

新增 `WorkbenchSnapshot` 字段建议：

- `real_execution_product_commands`

最低 summary 字段：

- `schema_version`
- `store_available`
- `store_revision`
- `command_count`
- `pending_decision_count`
- `running_attempt_count`
- `blocked_attempt_count`
- `last_attempt_status`
- `ordinary_product_entry_status`
- `legacy_entry_status`
- `runner_entry_status`
- `level_b_authorization_required`
- `warnings`

要求：

- 普通 UI 看到的是 boundary summary，不是完整 prompt、secret、raw transcript、raw log。
- planned adapters 仍显示 planned / unavailable / credential not configured / model unverified。
- `legacy_entry_status` 必须说明旧入口仍 blocked / legacy，不是统一 product command。

## 6. 入口分类冻结

PCR1 必须按 PCR0 分类处理：

- `prepare / preview`：可进入 preview contract，但不得执行。
- `inspect / preflight`：可进入 guard / authorization summary，但不得执行。
- `phase_a_no_real`：只记录结构化 runner path / no-real boundary，不得执行。
- `execute`：PCR1 不实现真实 execute；默认 blocked / not implemented。
- H3-B：按 internal / ignored probe path 处理，不暴露为普通 Tauri / UI entry。
- legacy Tauri / CLI / MCP：保持 blocked / sealed / deprecated。

## 7. 测试要求

Rust 必跑：

```text
cargo test --lib real_execution_command
cargo test --lib adapter_descriptor
cargo test --lib
cargo fmt -- --check
```

前端必跑：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
```

扫描必须覆盖：

```text
rg -n 'prompt_sent: true|real_codex_executed: true|writes_codex_home: true|Command::new\("codex"\)' prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs prototypes/productized-desktop-shell/src-tauri/src/types.rs
rg -n 'execute_workflow_node_dispatch|run_workflow_machine|canvas_start_run|canvas_tick_run' prototypes/productized-desktop-shell/src/App.tsx prototypes/productized-desktop-shell/src/views prototypes/productized-desktop-shell/src/lib/tauri.ts
rg -n '结果数：空|允许一次|H5 命令|启动实验画布运行|已启动实验画布运行' prototypes/productized-desktop-shell/src
```

验收说明：

- 第一条扫描如果命中 `true` 或 `Command::new("codex")`，必须解释为测试 fixture / runner adapter 之外的命中；PCR1 新增契约代码不得出现真实执行默认值。
- 第二条扫描允许 deprecated wrapper 定义命中，但不允许普通 UI 新增调用旧入口。
- 第三条扫描普通源码不得命中误导 UI 文案。

## 8. 验收标准

PCR1 完成必须满足：

- Rust / TS 类型同步。
- Product command store / sidecar 契约清楚。
- WorkbenchSnapshot 输出只读 boundary summary。
- 所有默认 preview / attempt 均为 `prompt_sent=false`、`real_codex_executed=false`、`writes_codex_home=false`。
- 高影响确认权规则有测试或明确校验函数。
- 不新增 runner 调用。
- 旧入口仍 blocked。
- 复核线只读回交无 P0/P1。

## 9. 回交格式

后端线回交必须包含：

1. 修改文件列表。
2. 新增类型 / store / read model 摘要。
3. 明确说明没有真实执行、没有 prompt 发送、没有 `.codex` 读写。
4. 验证命令结果。
5. 扫描结果和命中分类。
6. PCR2 可以接续的输入。
7. 不能声明完成的事项。

## 10. 不得声明

PCR1 完成后仍不得声明：

- 统一 Product Command Routing 已完整实现。
- 通用真实 send / resume 产品化完成。
- H5 通用真实派发已开放。
- 真实 `execute` 已可用。
- PCR2 / PCR3 / PCR4 已完成。
- PCR9 Level B 已授权。
- 任意项目自由执行。

## 11. 执行结果

结论：PCR1 已完成，复核线无 P0/P1/P2，允许进入 PCR2 准备；仍不得声明真实执行产品化完成。

实际落点：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`：新增 `RealExecutionProductCommandRequest / Preview / Decision / Attempt / Store / ReadModel`，并把 `real_execution_product_commands` 接入 `WorkbenchSnapshot`。
- `prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs`：新增 `real-execution-product-commands.v1.json` sidecar 契约、empty/load/validate/read model、PCR1 preview/attempt/readback/decision 校验函数和测试。
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`：snapshot 组装接入 product command read model，并纳入 warning count。
- `prototypes/productized-desktop-shell/src/lib/types.ts`：同步 TS 类型；`WorkbenchSnapshot.real_execution_product_commands` 保持可选以兼容旧前端 fallback。
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`：补 PCR1 只读 fixture 和离线断言。

主管 fresh 验证：

- `cargo test --lib real_execution_command`：7 passed。
- `cargo test --lib adapter_descriptor`：2 passed。
- `cargo test --lib`：276 passed / 5 ignored。
- `cargo fmt -- --check`：通过。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：13 passed。
- `npm run build`：通过，仅保留既有 Vite chunk size warning。

扫描结果：

- `real_execution_command.rs` / `types.rs` 中 `prompt_sent: true|real_codex_executed: true|writes_codex_home: true|Command::new("codex")` 无命中。
- `App.tsx` / `src/views` 中 PCR1 语义无命中；旧入口扫描只命中 `src/lib/tauri.ts` 既有 wrapper 定义。
- `结果数：空|允许一次|H5 命令|启动实验画布运行|已启动实验画布运行` 无命中。

复核线结论：

- 初次只读复核无 P0/P1，提出 P2：`validate_pcr1_attempt_safety` 未拒绝 `writes_project_files=true`。
- 主管线已补硬：`validate_pcr1_attempt_safety` 将 `writes_project_files` 纳入拒绝条件，单测覆盖 `writes_project_files=true` 必须被拒绝。
- 复核线复查确认 P2 已关闭，未发现新的 P0/P1/P2，建议主管线收口 PCR1。

边界确认：

- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送真实 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 未启动 Browser/Chrome/Tauri/Vite dev server/screenshot。
- 未同步 `CURRENT.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`、`tasks/README.md`；入口文档按计划留到 PCR8 或 PCR10 checkpoint 集中同步。
