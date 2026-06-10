# Unified Product Command Routing PCR4 Execute Phase A Noop Fake Runner v1

日期：2026-06-09

状态：已完成。

后端线任务。本文用于在 PCR1 后端契约、PCR2 prepare / preview 服务、PCR3 decision / confirmation 服务之上，补齐统一 Product Command 的 Phase A 非真实执行链路。PCR4 仍是 Level A：允许通过 fake / no-op runner 写工作台自有 sidecar、continuation、runtime log、audit 和 readback 边界引用；不执行真实 `codex exec` / `codex exec resume`，不发送 prompt，不读写 `/Users/yoyi/.codex`，不新增 UI 执行按钮，不同步权威入口。

## 0. 全局主管理解

已知事实：

- PCR0 已冻结方向：真实执行必须归口到统一 Product Command Routing；旧 workflow / machine / canvas 入口保持 legacy / sealed / blocked。
- PCR1 已完成：`RealExecutionProductCommandRequest / Preview / Decision / Attempt / Store / ReadModel`、store skeleton、read model、TS 类型和安全校验已建立。
- PCR2 已完成：`preview_real_execution_product_command` 只读；`prepare_real_execution_product_command` 在安全通过、无 blocked reasons、store revision 匹配时写入 command + preview snapshot。
- PCR3 已完成：`record_real_execution_product_command_decision` / `confirm_real_execution_product_command` 只写 product command sidecar 内部 decision + audit ref；`approved` 只是用户许可记录，不等于执行。
- 现有 `session_continuation_store` 已有 controlled continuation、stub / Phase A no-real runner、runtime log、audit、readback boundary 的基础能力。PCR4 可以复用这些能力，但不得调用 Phase B real process runner。

本任务假设：

- PCR4 的执行输入必须来自已 prepare 的 product command 和已记录的 user approved decision。
- PCR4 成功表示“Phase A fake/no-op 产品链路打通”，不表示真实 Codex 收到 prompt。
- PCR4 可以写 `real-execution-product-commands.v1.json` 中的 product command attempt。
- PCR4 可以写 `session-continuations.v1.json`、`runtime-log.v1.json` 或现有 session continuation / runtime log sidecar，但只能通过 fake / no-op / Phase A no-real 路径写入。
- PCR4 不允许 product command attempt 中 `runner_call_allowed=true`。本任务包初稿曾把该字段解释为 fake/no-op runner 许可，但当前 store validator 和 PCR1-PCR8 安全边界统一把 `runner_call_allowed` 视为真实 runner gate；因此 PCR4 成功路径也必须保持 `runner_call_allowed=false`，并通过 `writes_continuation_sidecar=true` / `writes_runtime_log=true` / status / warnings 表达 Phase A no-op 链路已记录。
- 入口文档仍按计划留到 PCR8 或 PCR10 checkpoint 同步；PCR4 不更新 `CURRENT.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`、`tasks/README.md`。

## 1. 目标

PCR4 目标：

1. 新增统一 product command Phase A execute 应用服务。
2. 新增 Tauri command：`run_real_execution_product_command_phase_a`。
3. 支持从已 prepare + approved 的 product command 触发 fake / no-op Phase A runner。
4. 成功路径写入 product command attempt。
5. 成功路径写入或复用 controlled session continuation，并写入 continuation attempt。
6. 成功路径写入 runtime log ref / audit refs / readback boundary，使 product command -> continuation -> runtime log -> readback -> audit 可追溯。
7. 阻断路径不调用 fake runner、不写 continuation attempt、不写 runtime log；是否写 product command blocked attempt 需要按第 5.4 节处理。
8. readback unavailable / failed / timed_out 必须保持 `result_count=null`。
9. 为 PCR5 legacy migration、PCR6 UI readiness、PCR8 checkpoint、PCR9 Level B 真实执行提供可复核的 Level A 基线。

## 2. 非目标

PCR4 不做：

- 不执行真实 `codex exec` / `codex exec resume`。
- 不发送真实 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 secret、token、`.env`、keychain、OAuth、provider credential、full transcript、rollout。
- 不调用 `run_controlled_session_continuation_real_resume_phase_b`。
- 不调用 H3-B real new-session runner。
- 不新增 `Command::new("codex")`。
- 不新增真实 runner / retry / auto-retry。
- 不做 PCR5 legacy migration。
- 不做 PCR6 UI 接入。
- 不做 PCR9 Level B。
- 不接 planned adapters 真实执行。
- 不新增普通 UI 执行按钮。
- 不把 Phase A fake/no-op attempt 显示或记录为真实 sent / completed by Codex / readback completed。
- 不写正式 memory / observation。
- 不改 `workflow-state.v0.json` 顶层结构。
- 不同步权威入口。

## 3. 文件范围

允许修改：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs`，仅允许抽取或复用现有 Phase A no-real helper；不得改 Phase B real runner 安全边界。
- `prototypes/productized-desktop-shell/src-tauri/src/runtime_log_store.rs`，仅当需要补最小 runtime log helper / 测试时允许。
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`，仅允许补 wrapper / no UI execution 断言；不接按钮。
- 本任务包，开发线可把状态改为“待主管复核”并追加开发线执行结果草稿；不得自行标记“已完成”。

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

如确需触碰默认不修改文件，开发线必须先停止并回交理由，由主管线决定是否拆 PCR4.1。

## 4. 后端 API 要求

### 4.1 输入类型

建议新增：

- `RunRealExecutionProductCommandPhaseAInput`
- `RealExecutionProductCommandPhaseAOutput`

`RunRealExecutionProductCommandPhaseAInput` 最低字段：

- `product_command_id`
- `expected_product_command_store_revision`
- `actor_role`
- `execution_decision`：默认 `phase_a_noop`
- `timeout_ms` 可选
- `requested_at` 可选，缺省由后端生成

如复用 session continuation store 需要 revision，可补：

- `expected_session_continuation_store_revision`

但不能要求调用方传 prompt body；PCR4 不发送 prompt，最多使用 `prompt_ref` / `prompt_hash` / redacted preview。

### 4.2 输出类型

`RealExecutionProductCommandPhaseAOutput` 最低字段：

- `status`：`phase_a_completed` / `phase_a_blocked` / `store_conflict` / `blocked`
- `product_command_id`
- `product_command_attempt`
- `read_model`
- `product_command_store_revision`
- `product_command_sidecar_path`
- `continuation_id`
- `continuation_attempt_id`
- `session_continuation_store_revision`
- `runtime_log_ref`
- `audit_refs`
- `readback_summary`
- `runner_call_allowed`
- `prompt_sent=false`
- `real_codex_executed=false`
- `writes_codex_home=false`
- `writes_project_files=false`
- `writes_product_command_sidecar`
- `writes_continuation_sidecar`
- `writes_runtime_log`
- `blocked_reasons`
- `warnings`

输出里的 `runner_call_allowed=false` 必须保持成立；PCR4 不用该字段表示 fake/no-op runner 许可，避免与真实 Codex runner gate 混淆。

### 4.3 Tauri commands

新增 Tauri command：

- `run_real_execution_product_command_phase_a`

TS wrapper：

- `runRealExecutionProductCommandPhaseA`

wrapper 只用于后续 PCR6 或测试准备；PCR4 不接普通 UI 按钮。

## 5. 服务行为要求

### 5.1 通用执行前检查

`run_real_execution_product_command_phase_a`：

1. 读取 `real-execution-product-commands.v1.json`。
2. 如果 sidecar 不存在，返回 `product_command_sidecar_missing_for_phase_a`，不创建空 sidecar。
3. 如果 sidecar JSON 损坏，返回错误，不覆盖。
4. 如果 schema version 不匹配，返回错误，不迁移。
5. 校验 `expected_product_command_store_revision == current_revision`；不匹配返回 `store_conflict`，不写。
6. 查找 `product_command_id` 对应 command；不存在返回 `product_command_not_prepared`。
7. 查找对应 preview snapshot；不存在返回 `product_command_preview_missing`。
8. 校验 preview 无 blocked reasons，readiness / guard / diagnostics / duplicate 不阻断。
9. 查找 terminal decision；必须存在 `decision=approved` 且 `confirmed_by=user`。
10. 拒绝 `rejected` / `request_changes` / 无 decision / 非 user decision。
11. 拒绝已有 running product command attempt 的重复执行。
12. 拒绝已有 terminal successful Phase A attempt 的重复执行，除非后续任务包定义 retry；PCR4 默认不实现 retry。
13. 通过检查后才允许进入 fake/no-op Phase A runner。

### 5.2 成功路径

成功路径必须：

1. 构建或绑定 controlled session continuation。
2. 通过现有 `session_continuation_store` 的 confirm / Phase A no-real 能力写入 continuation 与 attempt；如需要抽取 helper，必须保持 Phase B real runner 仍由独立授权路径控制。
3. 写入 runtime log ref。
4. 写入 session continuation audit event。
5. append `RealExecutionProductCommandAttempt` 到 product command sidecar。
6. product command attempt 必须引用 continuation id、runtime log ref、audit refs 和 readback summary。
7. product command sidecar revision +1，更新 `updated_at`、`last_write_id`。
8. 返回新的 product command read model。

成功路径 flags 必须：

- `runner_call_allowed=false`
- `prompt_sent=false`
- `real_codex_executed=false`
- `writes_codex_home=false`
- `writes_project_files=false`
- `writes_product_command_sidecar=true`
- `writes_continuation_sidecar=true`
- `writes_runtime_log=true`

成功路径 status 建议：

- product command attempt：`phase_a_noop_completed`
- continuation attempt：沿用现有 Phase A no-real / stub 成功状态，但不得叫 `real_completed`。

### 5.3 Readback 边界

PCR4 不做真实 readback。readback summary 必须满足：

- `attempted=false` 或 `real_readback_performed=false`
- `status` 可为 `unavailable` / `not_attempted` / `failed` / `timed_out`
- `result_count=null`
- warnings 必须说明 fake/no-op Phase A 未做真实 transcript readback

不得把 readback unavailable 显示或记录成 0 条结果。

### 5.4 阻断路径

阻断路径默认不写任何 sidecar，除非任务内已有清晰理由需要记录 product command blocked attempt。若写 blocked attempt，必须满足：

- 不写 continuation attempt。
- 不写 runtime log。
- `runner_call_allowed=false`
- `prompt_sent=false`
- `real_codex_executed=false`
- `writes_codex_home=false`
- `writes_project_files=false`
- `writes_continuation_sidecar=false`
- `writes_runtime_log=false`
- blocked attempt 的 status 必须是 `phase_a_blocked`，不能是 `running` / `sent` / `completed`。

优先建议：revision conflict / damaged JSON / unknown command / missing preview / no approved decision / duplicate running / preview blocked 全部不写 sidecar。

## 6. Store 写入要求

写入 `real-execution-product-commands.v1.json` 时：

- 如果 sidecar JSON 损坏，返回错误，不覆盖。
- 如果 schema version 不匹配，返回错误，不迁移。
- 如果 product command store revision conflict，返回 `store_conflict`，不写。
- revision 只能 +1。
- 只 append product command attempt；不得修改已有 command / preview / decision。
- audit refs 可以 append 到 store `audit_refs`，但必须可追溯到 attempt。
- 不复制完整 runtime log、完整 transcript、prompt body、secret 或 provider credential。
- 不写 `/Users/yoyi/.codex`。
- 不改 `workflow-state.v0.json` 顶层结构。

写入 `session-continuations.v1.json` / runtime log sidecar 时：

- 必须沿用现有 atomic write / lock / revision 规则。
- 损坏 JSON 不覆盖。
- revision conflict 不覆盖。
- 只写 fake/no-op Phase A 所需 continuation / attempt / audit / runtime log。
- 不调用 Phase B real runner。

## 7. Read Model 要求

PCR4 后 read model 至少应体现：

- `store_revision` 增加。
- `command_count` 不变。
- `pending_decision_count` 不增加。
- `running_attempt_count` 不应因完成的 Phase A no-op attempt 增加。
- `blocked_attempt_count` 只在明确写 blocked attempt 时增加；默认阻断不写则不增加。
- `last_attempt_status=phase_a_noop_completed` 或等价非真实状态。
- warnings 必须继续提示 PCR4 不是真实 Codex 执行，Level B 仍需 PCR9 单独授权。

如现有 read model 无法表达 Phase A no-op 与真实执行差异，优先通过 status / warnings 表达；不要为 PCR4 过度扩张 UI read model。

## 8. 实现建议

建议开发线按以下顺序实现：

1. 补 `RunRealExecutionProductCommandPhaseAInput` / `RealExecutionProductCommandPhaseAOutput`。
2. 补测试 fixture：prepared command + approved user decision + safe preview。
3. 先写红测覆盖成功 fake/no-op path。
4. 写执行前检查 helper：
   - store exists
   - revision match
   - command exists
   - preview exists
   - preview ready
   - approved user decision exists
   - no duplicate running / terminal phase_a attempt
5. 复用或抽取 session continuation Phase A no-real helper。
6. append product command attempt。
7. 补 Tauri command / invoke handler / TS wrapper。
8. 补 no UI wrapper usage 扫描。
9. 更新本任务包为“待主管复核”并追加开发线结果草稿。

如果在第 5 步发现现有 Phase A helper 强依赖 H2 authorization matrix 且会引入过宽耦合，停止并回交主管线，不要临时绕过授权或直接伪造 runtime log。

## 9. 测试要求

必须新增 / 补齐测试：

1. `phase_a_success_writes_product_attempt_continuation_runtime_log_audit_readback_refs`
   - 输入为 prepared command + approved user decision。
   - product command attempts len +1。
   - continuation store attempts len +1。
   - runtime log store 有新增 ref / event。
   - output `runner_call_allowed=false`。
   - output `prompt_sent=false`。
   - output `real_codex_executed=false`。
   - output `writes_codex_home=false`。
   - output `writes_project_files=false`。
   - readback `result_count=None`。
2. `phase_a_requires_user_approved_decision`
   - 无 decision / rejected / request_changes / non-user approved 全部 blocked，不调用 runner。
3. `phase_a_blocks_preview_blocked_without_writing`
   - preview blocked reasons 非空，不写 product command attempt / continuation attempt / runtime log。
4. `phase_a_revision_conflict_does_not_write`
   - product command store revision conflict，不覆盖 sidecar。
5. `phase_a_refuses_corrupt_product_command_sidecar_without_overwrite`
   - 损坏 JSON 不覆盖。
6. `phase_a_duplicate_running_or_completed_attempt_blocked`
   - 既有 running 或 completed Phase A attempt 时阻断。
7. `phase_a_readback_unavailable_keeps_result_count_null`
   - unavailable / failed / timed_out 均不能显示为 0。
8. `phase_a_does_not_call_real_codex_runner`
   - 不新增 `Command::new("codex")`。
   - 不调用 Phase B real runner。
   - 不读写 `.codex`。

可复用现有测试命名风格；不要求完全使用上述函数名。

## 10. 验证命令

开发线至少运行：

```bash
cd /Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri
cargo test --lib real_execution_command
cargo test --lib session_continuation
cargo test --lib runtime_log
cargo test --lib h5_project_dispatch_bridge
cargo test --lib
cargo fmt -- --check
```

```bash
cd /Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell
npm run typecheck
npm run test:offline-interaction
npm run build
```

如某个命令失败，必须先判断是否为本轮引入；不能绕过失败直接回交。

## 11. 扫描要求

开发线必须执行并分类：

```bash
cd /Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri
rg -n 'Command::new\("codex"\)|codex exec|codex exec resume|run_real_resume_phase_b|run_real_new_session_h3_b' src
```

```bash
cd /Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell
rg -n 'runRealExecutionProductCommandPhaseA|run_real_execution_product_command_phase_a' src/App.tsx src/views src/components tests
```

```bash
cd /Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell
rg -n 'phase_a.*已发送|fake.*真实|no-op.*真实|Codex 已收到|真实 Codex 已执行|result_count.*0' src tests
```

```bash
cd /Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell
rg -n 'prompt_sent:\s*true|real_codex_executed:\s*true|writes_codex_home:\s*true|writes_project_files:\s*true' src-tauri/src/real_execution_command.rs src/lib/tauri.ts src/lib/types.ts tests
```

说明：

- 第一组可能命中既有 Phase B / H3-B real runner、guard、任务文案、测试 fixture；必须分类为既有路径还是 PCR4 新增。
- 第二组 wrapper 允许在 `src/lib/tauri.ts` 和 tests 命中；不得在 `App.tsx` / views / components 命中。
- 第四组 PCR4 成功路径不允许 `runner_call_allowed=true`，也不得出现真实执行 flags 为 true。

## 12. 验收标准

PCR4 可接受为完成，当且仅当：

- `run_real_execution_product_command_phase_a` 存在并注册。
- prepared + user approved command 可以通过 fake/no-op Phase A 写入 product command attempt。
- product command attempt 可追溯到 continuation id、runtime log ref、audit refs、readback boundary。
- continuation / runtime log / audit 侧写入是现有受控 sidecar 路径，不写 `.codex`。
- blocked cases 不调用 runner。
- `prompt_sent=false`、`real_codex_executed=false`、`writes_codex_home=false`、`writes_project_files=false` 在 PCR4 所有输出和 attempts 中保持成立。
- readback unavailable / failed / timed_out 均保持 `result_count=null`。
- wrapper 未接普通 UI。
- 验证命令通过。
- 扫描完成并分类。
- 任务包状态为“待主管复核”，等待主管线 fresh verify 和复核线审查。

## 13. 不接受条件

出现以下任一情况，PCR4 不接受：

- 执行了真实 `codex exec` / `codex exec resume`。
- 发送了真实 prompt。
- 读写了 `/Users/yoyi/.codex`。
- 新增或调用了 `Command::new("codex")`。
- 调用了 Phase B real process runner 或 H3-B real runner。
- UI 出现执行按钮并可触发 PCR4。
- readback unavailable 被写成 0 条结果。
- `approved` 或 Phase A no-op 被写成真实 sent / completed by Codex。
- 阻断路径覆盖损坏 JSON 或 revision conflict sidecar。
- 同步了 `CURRENT.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`、`tasks/README.md`。

## 14. 开发线回交格式

开发线完成后中文回交：

1. 修改文件。
2. 新增 input/output/command/helper 摘要。
3. product command attempt 写入边界。
4. continuation / runtime log / audit / readback 链路如何建立。
5. 成功路径 flags。
6. 关键拒绝场景测试。
7. 验证命令结果。
8. 扫描分类。
9. PCR5 / PCR6 可接续输入。
10. 不能声明完成事项。

开发线不得自行标记“已完成”；只能改为“待主管复核”。

## 15. 全局主管复核要求

主管线收到开发线回交后必须 fresh verify：

- 重跑第 10 节关键命令。
- 复扫第 11 节。
- 核对 product command sidecar 只写 attempt，不修改 command / preview / decision。
- 核对 continuation / runtime log / audit 只走 fake/no-op Phase A。
- 核对 UI 未接 wrapper。
- 核对未同步权威入口。
- 然后交复核线只读审查。

复核线无 P0/P1 后，主管线才可把本文状态改为“已完成”。PCR4 完成后下一步是 PCR5：旧入口迁移 / 封存；仍不是 Level B 真实执行。

## 16. 本线执行结果草稿，待复核

执行时间：2026-06-09

执行人：全局主管线兜底实现。

状态：已通过复核线只读审查；主管线已标记完成。

### 16.1 修改文件

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `tasks/2026-06-09-unified-product-command-routing-pcr4-execute-phase-a-noop-fake-runner-v1.md`

未改 `App.tsx`、`src/views/*`、`src/components/*`、`CURRENT.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`、`tasks/README.md`。

### 16.2 实现摘要

- 新增 `RunRealExecutionProductCommandPhaseAInput` / `RealExecutionProductCommandPhaseAOutput`。
- 新增后端服务 `run_real_execution_product_command_phase_a_at`。
- 新增 Tauri command `run_real_execution_product_command_phase_a`。
- 新增 TS wrapper `runRealExecutionProductCommandPhaseA`。
- 成功路径从 prepared product command + user approved decision 出发，复用 `confirm_continuation` 和 `run_real_resume_phase_a`，写入 product command attempt、session continuation attempt、runtime log ref、audit refs、readback boundary。
- 在写 continuation 之前增加 `runtime_log_store::ensure_appendable` preflight，避免 runtime log 损坏时出现 continuation 已写但 product command 未回写的半提交。
- PCR4 成功路径保持 `runner_call_allowed=false`、`prompt_sent=false`、`real_codex_executed=false`、`writes_codex_home=false`、`writes_project_files=false`。其中 `runner_call_allowed=false` 是对本任务包初稿的安全修正，原因是现有 store validator 将该字段视为真实 runner gate。

### 16.3 测试覆盖

新增 / 补齐 PCR4 测试：

- `pcr4_phase_a_noop_writes_trace_refs_without_real_codex`
- `pcr4_phase_a_blocks_without_approved_user_decision_and_does_not_write`
- `pcr4_phase_a_revision_conflict_blocked_preview_duplicate_and_corrupt_json_do_not_write`
- `pcr4_phase_a_corrupt_runtime_log_preflight_does_not_write_partial_continuation`

覆盖点：

- prepared + user approved 成功写 product command attempt、continuation attempt、runtime log dispatch/readback refs、audit refs。
- 无 decision / rejected / request_changes 阻断且不写 product / continuation / runtime。
- revision conflict 不覆盖 sidecar。
- preview blocked 不写 product / continuation / runtime。
- duplicate completed Phase A 阻断。
- damaged product command JSON 不覆盖。
- damaged runtime log preflight 阻断，且不写 partial continuation。
- readback unavailable 保持 `result_count=null`。
- PCR4 全路径不发送 prompt、不执行真实 Codex、不写 `.codex`、不写项目文件。

### 16.4 验证结果

- `cargo test --lib real_execution_command`：通过，27 passed。
- `cargo test --lib session_continuation`：通过，17 passed / 4 ignored。
- `cargo test --lib runtime_log`：通过，6 passed。
- `cargo test --lib h5_project_dispatch_bridge`：通过，4 passed。
- `cargo test --lib`：通过，296 passed / 5 ignored。
- `cargo fmt -- --check`：通过。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，13 passed。
- `npm run build`：通过，仅保留既有 Vite chunk size warning。

### 16.5 扫描分类

- `Command::new("codex")` / `codex exec` / `codex exec resume` / Phase B runner 扫描：命中既有 `src/lib.rs` 真实 runner、`src/mcp/codex_runner.rs`、`session_continuation_store.rs` Phase B/H3-B 授权路径、worker protocol command preview 和测试 fixture；PCR4 没有新增 `Command::new("codex")`，PCR4 服务只调用 Phase A no-op runner。
- PCR4 wrapper UI 接入扫描：`App.tsx`、`src/views`、`src/components`、`tests` 无 `runRealExecutionProductCommandPhaseA` / `run_real_execution_product_command_phase_a` 命中。wrapper 仅在 `src/lib/tauri.ts`，Tauri command 仅在 `src-tauri/src/commands.rs` / `src-tauri/src/lib.rs` 注册。
- 误导文案扫描：命中既有 tests 黑名单 fixture、`canvasSurfaceBoundaries.ts` 黑名单常量和 `sessionOperations.ts` 的 H3.1 边界文案；未发现 PCR4 新增 UI 文案把 Phase A no-op 说成真实 Codex 执行。
- PCR4 限定文件真实 flag 扫描：`prompt_sent:true`、`real_codex_executed:true`、`writes_codex_home:true`、`writes_project_files:true`、`runner_call_allowed:true` 无命中。

### 16.6 边界确认

- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送真实 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 未启动 Browser/Chrome/Tauri/Vite dev/screenshot。
- 未调用 `run_real_resume_phase_b` 或 `run_real_new_session_h3_b`。
- 未新增 UI 执行按钮。
- 未同步权威入口。

### 16.7 PCR5 / PCR6 可接续输入

- PCR5 可以基于 `run_real_execution_product_command_phase_a` 已存在且旧入口仍 sealed 的事实，继续做旧入口迁移 / 封存。
- PCR6 可以只读接入 `RealExecutionProductCommandPhaseAOutput` / read model，展示 Phase A no-op attempt、continuation/runtime/audit/readback refs，但不得提供真实执行按钮。
- PCR9 Level B 仍需单独任务包、单独授权、单独 `.codex` 读写边界。

## 17. 复核线结论

复核线：`019ea33a-23c4-7c10-8db3-95b8cf910fe7`

结论：无 P0 / P1，PCR4 可由主管线标记完成。`runner_call_allowed=false` 的修正合理，因为当前契约中该字段代表真实 runner gate，不应被 fake/no-op Phase A 复用。

P2 后续建议：

- PCR4 成功路径已通过 `runtime_log_store::ensure_appendable` 在写 continuation 前阻断损坏 runtime log，但跨 product command / continuation / runtime 三个 sidecar 仍不是完整事务。如果最终 product command sidecar 原子写因磁盘或权限异常失败，理论上可能留下 continuation/runtime 已写的半提交痕迹。建议 PCR5/PCR6 前后补“最终 product sidecar 写失败”的恢复/审计策略或单测。

复核确认：

- PCR4 只走 `NoopCodexLocalPhaseAProcessRunner`，不调用 Phase B / H3-B real runner。
- 成功路径只在 prepared + user approved decision + revision match + preview ready + no duplicate 后写入。
- product command attempt 和 output 都保持 `runner_call_allowed=false`、`prompt_sent=false`、`real_codex_executed=false`、`writes_codex_home=false`、`writes_project_files=false`。
- runtime log 损坏 preflight 在写 continuation 前执行，避免该场景的 partial continuation write。
- readback unavailable / failed / timed_out 保持 `result_count=null`。
- wrapper 未接入 `App.tsx` / views / components，普通 UI 未新增执行按钮。
- 第 16 节验证和扫描分类与代码事实一致，未夸大为真实执行。
