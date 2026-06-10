# Unified Product Command Routing PCR3 Decision Confirmation Service v1

日期：2026-06-09

状态：已完成。

后端线任务。本文用于在 PCR1 后端契约和 PCR2 prepare / preview 服务之上，补齐统一 Product Command 的用户决定 / 确认写入服务。PCR3 仍是 Level A：可以写工作台自有 `real-execution-product-commands.v1.json` 中的 decision 和 audit refs，但不授权真实 `codex exec` / `codex exec resume`，不发送 prompt，不读写 `/Users/yoyi/.codex`，不新增 UI 执行按钮，不同步权威入口。

## 0. 全局主管理解

已知事实：

- PCR0 已冻结方向：真实执行必须归口到统一 Product Command Routing；旧 workflow / machine / canvas 入口保持 legacy / sealed / blocked。
- PCR1 已完成：`RealExecutionProductCommandRequest / Preview / Decision / Attempt / Store / ReadModel`、store skeleton、read model、TS 类型和安全校验已建立。
- PCR2 已完成：`preview_real_execution_product_command` 只读；`prepare_real_execution_product_command` 在安全通过、无 blocked reasons、store revision 匹配时写入 command + preview snapshot。
- PCR2 复核线确认无 P0/P1/P2 required fixes；可选后续 hardening 是损坏 JSON 不覆盖测试。
- 现有 `validate_real_execution_product_command_decision` 已具备基础规则：decision command id 必须匹配，高影响 approved 必须 `confirmed_by == "user"`、`allowed_once == true`、risk acknowledgement 非空。

本任务假设：

- PCR3 的 decision 只针对已 prepare 成功并已写入 sidecar 的 product command。
- `approved` 决定只是用户许可记录，不等于执行；真正 runner 调度最早到 PCR4 fake runner / PCR9 Level B。
- `rejected` / `request_changes` 必须可追溯，但绝不能调用 runner。
- audit refs 是 product command store 内的轻量可追溯引用，不写正式 runtime log、不写正式 memory audit、不写 `.codex`。
- 入口文档仍按计划留到 PCR8 或 PCR10 checkpoint 同步；PCR3 不更新 `CURRENT.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`、`tasks/README.md`。

## 1. 目标

PCR3 目标：

1. 新增统一 product command decision / confirmation 应用服务。
2. 支持记录用户决定：
   - `approved`
   - `rejected`
   - `request_changes`
3. 支持高影响真实执行确认快捷命令：`confirm_real_execution_product_command`。
4. 写入 `RealExecutionProductCommandDecision` 到 `real-execution-product-commands.v1.json`。
5. 为每次 decision 追加轻量 audit ref 到 store `audit_refs`，并把该 audit ref 可追溯地关联到 decision。
6. 校验 sidecar revision conflict。
7. 校验 command / preview 存在且未损坏。
8. 校验高影响 approved 必须由用户确认；`project_director`、`global_supervisor` 或其他角色不能替代用户。
9. `rejected` / `request_changes` 可记录决定，但不调用 runner、不写真实 attempt、不写 runtime log。
10. 为 PCR4 fake execute 提供稳定输入：prepared command + approved user decision + audit ref + read model。

## 2. 非目标

PCR3 不做：

- 不执行真实 `codex exec` / `codex exec resume`。
- 不发送真实 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 secret、token、`.env`、keychain、OAuth、provider credential、full transcript、rollout。
- 不调用 `run_controlled_session_continuation_real_resume_phase_a` / `run_controlled_session_continuation_real_resume_phase_b`。
- 不调用 `Command::new("codex")`。
- 不新增真实 runner / retry / auto-retry。
- 不写真实 execution attempt。
- 不写 runtime log 正式执行条目。
- 不写 formal memory / observation / workflow state 顶层结构。
- 不新增普通 UI 执行按钮。
- 不把 approved 决定显示或记录为 running / sent / completed / readback completed。
- 不接 planned adapters 真实执行。
- 不做 PCR4 / PCR5 / PCR6 / PCR9。
- 不同步权威入口。

## 3. 文件范围

允许修改：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`，仅允许补 wrapper / no UI execution 断言；不接按钮。

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

如确需触碰默认不修改文件，开发线必须先停止并回交理由，由主管线决定是否拆 PCR3.1。

## 4. 后端 API 要求

### 4.1 输入类型

建议新增：

- `RecordRealExecutionProductCommandDecisionInput`
- `ConfirmRealExecutionProductCommandInput`
- `RealExecutionProductCommandDecisionOutput`

`RecordRealExecutionProductCommandDecisionInput` 最低字段：

- `product_command_id`
- `decision`：`approved` / `rejected` / `request_changes`
- `expected_store_revision`
- `confirmed_by`
- `risk_acknowledgement`
- `allowed_once`
- `reason`
- `requested_by`
- `confirmed_at` 可选，缺省由后端生成

`ConfirmRealExecutionProductCommandInput` 最低字段：

- `product_command_id`
- `expected_store_revision`
- `confirmed_by`
- `risk_acknowledgement`
- `allowed_once`
- `reason`
- `requested_by`
- `confirmed_at` 可选

`confirm_real_execution_product_command` 语义等价于记录 `decision=approved`，但必须保留更强的高影响确认校验和更明确的 blocked reason。

`RealExecutionProductCommandDecisionOutput` 最低字段：

- `status`：`decision_recorded` / `decision_rejected` / `store_conflict` / `blocked`
- `decision`
- `read_model`
- `store_revision`
- `sidecar_path`
- `audit_ref`
- `runner_call_allowed=false`
- `prompt_sent=false`
- `real_codex_executed=false`
- `writes_codex_home=false`
- `writes_project_files=false`
- `writes_product_command_sidecar`
- `blocked_reasons`
- `warnings`

### 4.2 Tauri commands

新增 Tauri commands：

- `record_real_execution_product_command_decision`
- `confirm_real_execution_product_command`

TS wrapper：

- `recordRealExecutionProductCommandDecision`
- `confirmRealExecutionProductCommand`

wrapper 只用于后续 PCR4/PCR6 或测试准备；PCR3 不接普通 UI 按钮。

## 5. 服务行为要求

### 5.1 通用写入流程

`record_real_execution_product_command_decision`：

1. 读取 `real-execution-product-commands.v1.json`。
2. 如果 sidecar 不存在，返回 `product_command_sidecar_missing_for_decision`，不创建空 sidecar。
3. 如果 sidecar JSON 损坏，返回错误，不覆盖。
4. 如果 schema version 不匹配，返回错误，不迁移。
5. 校验 `expected_store_revision == current_revision`；不匹配返回 `store_conflict`，不写。
6. 查找 `product_command_id` 对应 command；不存在返回 `product_command_not_prepared`。
7. 查找对应 preview snapshot；不存在返回 `product_command_preview_missing`。
8. 对 `approved`，如果 preview 有 blocked reasons 或 readiness 不是可进入 decision 状态，必须拒绝，返回 `product_command_preview_not_ready_for_approval`。
9. 构建 `RealExecutionProductCommandDecision`。
10. 调用 `validate_real_execution_product_command_decision`。
11. 拒绝重复 terminal decision：同一 command 已有 `approved` / `rejected` / `request_changes` 时不得覆盖；返回 `product_command_decision_already_recorded`。
12. append decision。
13. append audit ref。
14. revision +1，更新 `updated_at`、`last_write_id`。
15. 原子写回 sidecar。
16. 返回新的 read model。

### 5.2 Confirm 快捷命令

`confirm_real_execution_product_command`：

- 只能记录 `approved`。
- 必须强制高影响真实执行确认规则：
  - `confirmed_by == "user"`
  - `allowed_once == true`
  - `risk_acknowledgement` 非空
  - `reason` 非空
- `project_director` / `global_supervisor` / `assistant` / 空字符串都不能替代 user。
- 不调用 runner。
- 不写 attempt。
- 不写 runtime log。

### 5.3 Rejected / Request Changes

`rejected`：

- 可以记录在已 prepare 的 command 上。
- 必须 append audit ref。
- 不调用 runner。
- 不写 attempt，除非开发线先回交主管线说明为什么需要 `decision_rejected_no_runner` attempt；默认 PCR3 不写 attempt。
- read model 中该 command 不应继续计入 pending decision。

`request_changes`：

- 可以记录在已 prepare 的 command 上。
- 必须 append audit ref。
- 不调用 runner。
- 不写 attempt。
- 后续若需要基于 request_changes 重新 prepare，应由 PCR4+ 或单独任务包处理；PCR3 不自动生成新 command。

## 6. Audit Ref 要求

PCR3 的 audit ref 是 product command sidecar 内部轻量引用，建议格式：

```text
real-exec-command-audit:{product_command_id}:{decision_id}
```

要求：

- 每次 decision 必须产生 1 条 audit ref。
- `store.audit_refs` 必须包含该 ref。
- `RealExecutionProductCommandDecisionOutput.audit_ref` 必须返回该 ref。
- 不写正式 runtime audit store。
- 不写 memory audit store。
- 不写 `/Users/yoyi/.codex`。
- 不包含 prompt body、secret、full transcript。

## 7. Store 写入要求

写入 `real-execution-product-commands.v1.json` 时：

- 如果 sidecar JSON 损坏，返回错误，不覆盖。
- 如果 schema version 不匹配，返回错误，不迁移。
- 如果 revision conflict，返回 `store_conflict`，不写。
- revision 只能 +1。
- 只 append decision 和 audit ref。
- 不修改已有 command / preview。
- 不写 attempt。
- 不写 workflow state。
- 不写 runtime log。
- 不写 project files。
- 不写 `.codex`。

## 8. Read Model 要求

PCR3 后 read model 至少应体现：

- `store_revision` 增加。
- `pending_decision_count` 随 terminal decision 下降。
- `command_count` 不变。
- `running_attempt_count` 不变。
- `blocked_attempt_count` 不因 decision 写入增加。
- warnings 可以继续提示 runner 仍需 PCR4/PCR9，但不能说 approved 已执行。

如现有 read model 无法表达 “approved but not executed”，可以仅通过 warnings 表达；不要为 PCR3 过度扩张 UI read model。

## 9. 测试要求

Rust 最小测试：

1. `record_real_execution_product_command_decision` 写入 approved decision + audit ref，revision +1，read model pending 下降。
2. `confirm_real_execution_product_command` 成功路径必须 `confirmed_by=user`、`allowed_once=true`、risk acknowledgement 非空。
3. 高影响 approved 且 `confirmed_by=project_director` 被拒绝，不写 sidecar。
4. 高影响 approved 且 `allowed_once=false` 被拒绝，不写 sidecar。
5. 高影响 approved 且 risk acknowledgement 为空被拒绝，不写 sidecar。
6. `rejected` 写 decision + audit ref，不调用 runner，不写 attempt。
7. `request_changes` 写 decision + audit ref，不调用 runner，不写 attempt。
8. revision conflict 返回 `store_conflict`，不写 sidecar。
9. unknown command 返回 blocked / error，不创建 sidecar。
10. damaged JSON 返回 parse error，不覆盖原文件。
11. duplicate terminal decision 被拒绝，不覆盖已有 decision。
12. blocked preview 不允许 approved。

TypeScript / offline tests：

- 类型导出和 wrapper 可编译。
- 普通 UI 不调用 PCR3 wrappers。
- 搜索确认 `recordRealExecutionProductCommandDecision` / `confirmRealExecutionProductCommand` 不出现在 `src/views/*`、`src/components/*` 的普通按钮路径中。

## 10. 验证命令

开发线完成后必须运行：

```bash
cargo test --lib real_execution_command
cargo test --lib h5_project_dispatch_bridge
cargo test --lib session_continuation
cargo test --lib runtime_log
cargo test --lib
cargo fmt -- --check
npm run typecheck
npm run test:offline-interaction
npm run build
```

并执行搜索：

```bash
rg -n "Command::new\\(\"codex\"\\)|codex exec|codex exec resume|run_controlled_session_continuation_real_resume_phase|real_codex_executed:\\s*true|prompt_sent:\\s*true|writes_codex_home:\\s*true" prototypes/productized-desktop-shell/src-tauri/src prototypes/productized-desktop-shell/src tests
rg -n "recordRealExecutionProductCommandDecision|confirmRealExecutionProductCommand" prototypes/productized-desktop-shell/src/views prototypes/productized-desktop-shell/src/components prototypes/productized-desktop-shell/src/App.tsx
rg -n "approved.*running|approved.*sent|approved.*completed|用户确认.*已执行|已发送给 Codex" prototypes/productized-desktop-shell/src
```

搜索命中必须分类说明；只允许既有 runner adapter / guard / 测试 fixture / wrapper 命中，不允许 PCR3 新增真实执行路径。

## 11. 回交要求

开发线回交必须包含：

- 改动文件清单。
- 新增 commands / types / wrapper 名称。
- decision 写入行为说明。
- revision conflict、project director 不能替代 user、rejected / request_changes 不调用 runner的验证证据。
- 是否触碰默认不修改文件；若触碰，说明理由。
- 测试命令和结果。
- 搜索命中分类。
- 明确边界确认：
  - 未执行真实 `codex exec` / `codex exec resume`
  - 未发送 prompt
  - 未读写 `/Users/yoyi/.codex`
  - 未读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript / rollout
  - 未启动 Browser / Chrome / Tauri / Vite preview / 截图工具
  - 未同步权威入口

## 12. 复核线要求

复核线只读检查：

- 对照本任务包逐条核对。
- 核对 PCR3 没有绕过 PCR4/PCR9 调 runner。
- 核对 approved decision 不被冒领成 execution。
- 核对 high-impact confirmation 只能由 user 完成。
- 核对 damaged JSON / revision conflict 不覆盖 sidecar。
- 核对 UI 普通路径没有新增执行按钮或直接 decision wrapper。
- 输出 P0 / P1 / P2 分级结论。

P0：

- 任何真实 Codex 执行、prompt 发送、`.codex` 读写、secret 读取。
- approved decision 触发 runner。
- `project_director` 可替代 user 进行高影响 approved。
- revision conflict 或 damaged JSON 覆盖 sidecar。

P1：

- rejected / request_changes 写成 running / sent / completed。
- duplicate terminal decision 覆盖已有决定。
- 普通 UI 出现未验收的执行按钮。
- read model 把 approved 误显示为已执行。

P2：

- audit ref 命名不够稳定。
- warnings 文案仍偏内部，但不造成误导执行。
- 缺少非关键 fixture 覆盖。

## 13. 完成定义

PCR3 可接受为完成，当且仅当：

- 任务包状态改为 `已完成`。
- development handoff 返回完整证据。
- 主管线 fresh verify 通过。
- 复核线无 P0/P1。
- 所有 required tests 通过，或未跑项有明确主管接受的原因。
- 没有真实 Codex、`.codex`、secret、UI 执行入口、权威入口同步越界。

PCR3 完成后下一步是 PCR4：execute Phase A no-op / fake runner；仍不是 Level B 真实执行。

## 14. 开发线执行结果草稿

状态说明：PCR3 开发线实现和自验证已完成，等待主管线 fresh verify 与复核线审查；本文不自标 `已完成`。

改动文件：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `tasks/2026-06-09-unified-product-command-routing-pcr3-decision-confirmation-service-v1.md`

实现摘要：

- 新增 `RecordRealExecutionProductCommandDecisionInput`、`ConfirmRealExecutionProductCommandInput`、`RealExecutionProductCommandDecisionOutput`。
- 新增后端服务 `record_real_execution_product_command_decision_at`、`confirm_real_execution_product_command_at`。
- 新增 Tauri command `record_real_execution_product_command_decision`、`confirm_real_execution_product_command`。
- 新增 TS wrapper `recordRealExecutionProductCommandDecision`、`confirmRealExecutionProductCommand`。
- 成功写入只 append `decision` 和 `audit_ref`，revision +1，并更新 `updated_at` / `last_write_id` / store warning；不写 attempt、runtime log、formal audit、workflow-state 顶层结构或项目文件。
- audit ref 格式为 `real-exec-command-audit:{product_command_id}:{decision_id}`。

关键边界：

- `approved` 只记录 permission decision，不代表 running / sent / completed。
- 高影响 `approved` 仍要求 `confirmed_by=user`、`allowed_once=true`、risk acknowledgement 非空、reason 非空。
- `project_director` / `global_supervisor` / `assistant` 不能替代 user。
- `rejected` / `request_changes` 可写 decision + audit ref，但不写 attempt、不调用 runner。
- revision conflict、unknown command、damaged JSON、duplicate terminal decision、blocked preview approval 均不写 sidecar；damaged JSON 返回 parse error 且不覆盖。

验证结果：

- `cargo test --lib real_execution_command`：23 passed。
- `cargo test --lib h5_project_dispatch_bridge`：4 passed。
- `cargo test --lib session_continuation`：17 passed，4 ignored，ignored 项为既有需要显式真实授权的 probe。
- `cargo test --lib runtime_log`：5 passed。
- `cargo test --lib`：292 passed，5 ignored。
- `cargo fmt -- --check`：通过。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：offline interaction tests passed: 13。
- `npm run build`：通过；保留既有 Vite chunk size warning。

扫描结果摘要：

- 任务包原第一组 `rg` 命令因仓库根下不存在 `tests` 目录返回路径错误；已补扫实际 `prototypes/productized-desktop-shell/tests`。
- 真实执行关键词命中均为既有 H2/H5/session continuation/codex runner/测试 fixture/边界文案，不是 PCR3 新增执行路径。
- `recordRealExecutionProductCommandDecision|confirmRealExecutionProductCommand` 在 `src/views`、`src/components`、`src/App.tsx` 无命中；仅在 `src/lib/tauri.ts` wrapper 命中。
- `approved.*running|approved.*sent|approved.*completed|用户确认.*已执行|已发送给 Codex` 无命中。

边界确认：

- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript / rollout。
- 未启动 Browser / Chrome / Tauri / Vite dev / screenshot。
- 未同步 `CURRENT.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`、`tasks/README.md`。

PCR4 可接续输入：

- prepared command + safe preview snapshot。
- approved user decision。
- sidecar audit ref。
- read model 中 pending decision 下降且 runner 仍 blocked 的状态。

不能声明事项：

- 不能声明 PCR3 主管接受或阶段完成。
- 不能声明真实 Codex 已授权或已执行。
- 不能声明 PCR4 fake execute、PCR5 legacy migration、PCR6 UI、PCR9 Level B 已实现。

## 15. 主管复核与接受结论

状态说明：主管线 fresh verify 已完成；复核线只读审查无 P0/P1/P2 必修项。PCR3 接受为“待 checkpoint 文档同步前的代码完成”。本文只更新任务包状态，不同步 `CURRENT.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`、`tasks/README.md`。

主管 fresh verify：

- `cargo test --lib real_execution_command`：23 passed。
- `cargo test --lib h5_project_dispatch_bridge`：4 passed。
- `cargo test --lib session_continuation`：17 passed，4 ignored；ignored 为既有显式真实授权 probe。
- `cargo test --lib runtime_log`：5 passed。
- `cargo test --lib`：292 passed，5 ignored。
- `cargo fmt -- --check`：通过。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：offline interaction tests passed: 13。
- `npm run build`：通过；保留既有 Vite chunk size warning。

主管 fresh scan：

- `recordRealExecutionProductCommandDecision|confirmRealExecutionProductCommand` 在 `src/views`、`src/components`、`src/App.tsx` 无命中。
- `approved.*running|approved.*sent|approved.*completed|用户确认.*已执行|已发送给 Codex` 在 `src` 无命中。
- PCR3 改动文件中 `real_codex_executed:true|prompt_sent:true|writes_codex_home:true|writes_project_files:true|runner_call_allowed:true` 无命中。
- 广域真实执行关键词命中均为既有 H2/H5/session continuation/codex runner/worker protocol/测试 fixture/边界文案；不是 PCR3 新增执行路径。
- `CURRENT.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`、`tasks/README.md` 中无 PCR3 新命令口径命中。

复核线结论：

- P0：无。
- P1：无。
- P2：无必须修补项。
- 可选补强：后续可参数化补 `global_supervisor` / `assistant` 等非 user 角色的显式测试；当前代码已用 `confirmed_by != "user"` 覆盖，非阻断。

最终接受边界：

- PCR3 只接受为 unified product command 的 permission decision / confirmation 写入服务完成。
- `approved` 仍只是 permission decision，不代表 running / sent / completed。
- PCR3 不接受为真实 Codex 执行、PCR4 fake execute、PCR5 legacy migration、PCR6 UI 接入或 PCR9 Level B 完成。
- 本轮未执行真实 `codex exec` / `codex exec resume`，未发送 prompt，未读写 `/Users/yoyi/.codex`，未启动 Browser / Chrome / Tauri / Vite preview / 截图工具，未同步权威入口。
