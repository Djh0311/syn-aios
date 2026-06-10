# Stage J / J4 Run Queue, Failure Control, And User Confirmation Queue v1

日期：2026-06-09

状态：已完成，结论为 `accepted_with_deferred_items`。本任务包接在 J3 `accepted_with_deferred_items` 之后，推进 Stage J 的“自由操控 Codex + 自动化工作流编排 + 记忆层记录 / 分析 / 候选化”产品闭环。J4 不默认授权新的真实 `codex exec` / `codex exec resume`，不发送 prompt，不读写 `/Users/yoyi/.codex`，不做无确认自动 retry / stop / restart，不自动写 FormalMemory。复核线最终确认无 P0/P1/P2，允许主管线收口；下一步进入 J5 UI 信息层级和真实 Tauri 产品验收。

全局主管任务。J4 的目标是把 J1 / J2 / J2-B / J3 已经形成的 Product Command、run unit、runtime log、readback、capture event、observation / candidate refs 组织成用户能理解和处理的“运行队列 + 失败控制 + 用户确认队列”。J4 要让工作台从“能执行和记录”进一步变成“能持续管理运行中的工作”，但仍然不能绕过统一 Product Command、用户确认、audit 和记忆确认链路。

## 0. 先说薄弱点

- J1/J2 已能把自由操控和项目工作流编排接入统一 Product Command，但用户还缺一个稳定的运行队列视角来理解“哪些在运行、哪些等待我确认、哪些失败、哪些被 guard 阻断”。
- H4 已完成 readback / failure / timeout / duplicate guard 的底座，但 Stage J 尚未把这些状态产品化到用户确认队列。
- J3 已把运行事件接入 capture / observation / candidate，但 `candidate_allowed` 仍存在跨 store 原子性 P2：先写 observation/candidate，再 append capture event，晚期失败可能留下半完成状态。J4 需要显式展示这种补偿 / 需要人工确认的状态，而不是静默吞掉。
- 如果 J4 做成自动重试系统，会绕过用户确认和风险边界；如果只做只读列表，又不能支撑“自由操控 Codex”的持续产品化。

## 1. 权威依据

必须服从：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/2026-06-09-stage-j-codex-control-plane-workflow-memory-productization-plan-v1.md`
- `tasks/2026-06-09-stage-j-j0-permission-product-scope-and-acceptance-matrix-freeze-v1.md`
- `tasks/2026-06-09-stage-j-j1-codex-control-plane-free-control-entry-v1.md`
- `tasks/2026-06-09-stage-j-j2-project-workflow-automation-run-units-and-controlled-closed-loop-v1.md`
- `tasks/2026-06-09-stage-j-j2-b-controlled-real-workflow-automation-execution-point-freeze-v1.md`
- `tasks/2026-06-09-stage-j-j3-memory-capture-bus-and-candidate-generation-v1.md`
- `evidence/2026-06-09-stage-j-j3-memory-capture-bus-and-candidate-generation-v1.md`
- `handoffs/2026-06-09-stage-j-j3-memory-capture-bus-and-candidate-generation-v1-result.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`

必须复用：

- PCR0-PCR10：真实执行必须归口统一 Product Command。
- J1：Codex Control Plane 操作入口和 `codex_control` source。
- J2：ProjectWorkflowAutomationPlan / run units / controlled closed loop。
- J3：MemoryCaptureEvent / observation / MemoryCandidate refs。
- G1/G2/H4：runtime log、diagnostics、readback unknown-result、duplicate guard、failure / timeout classification。
- M2/M9/M12：正式记忆采纳和生命周期确认链路。

## 2. J4 目标

J4 要交付：

1. 新增或等价实现运行队列读模型，统一展示：
   - `running`
   - `waiting_user`
   - `blocked_by_guard`
   - `failed`
   - `readback_unavailable`
   - `readback_failed`
   - `timed_out`
   - `duplicate_blocked`
   - `completed_needs_review`
2. 新增或等价实现用户确认队列读模型，覆盖：
   - 执行确认
   - retry 确认
   - stop / cancel 确认
   - 结果确认
   - process fact 确认
   - memory candidate / formalization 确认
   - capture 补偿 / 半完成状态确认
3. 新增或等价实现 failure control 摘要，说明失败原因、下一步、是否可 retry、是否需要用户确认、是否会写项目文件或 `.codex`。
4. 所有 retry / stop / restart / cancel 必须先变成 preview / proposal / confirmation envelope；J4 默认不做真实动作。
5. `result_count=null` 的 unknown-result 状态必须继续显示为“未知 / 不可用”，不能显示为 0。
6. 运行中工作流入口、右侧待办 / 运行中、项目工作流节点详情和智能体会话中心要能看到同一套状态摘要，不能各说各话。
7. J3 capture / observation / candidate refs 要进入运行详情，但不能把 candidate / observation 冒充 FormalMemory。

## 3. J4 非目标

J4 不做：

- 不执行新的真实 Codex。
- 不发送 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 auth/token/secret/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 不做无确认自动 retry / stop / restart。
- 不 kill Codex 进程。
- 不自动写 FormalMemory。
- 不接 planned adapters 真实执行。
- 不做 provider credential store 或 model verification。
- 不把 J4 完成说成 J5 / J6 或 Stage J 完成。

## 4. 数据契约

实现可以在 Rust 后端派生读模型，也可以复用已有 stores 派生前端读模型；但必须有稳定类型，不能只靠散落 UI 文案。

建议新增或等价字段：

### 4.1 RunQueueItem

```text
queue_item_id
project_id
project_root
workflow_id
workflow_node_id
run_unit_id
product_command_id
product_attempt_id
adapter_id
session_id
status
status_reason
user_visible_summary
next_step_label
requires_user_action
can_retry
can_stop
can_restart
can_resume
readback_status
readback_result_count
runtime_log_refs[]
audit_refs[]
capture_event_refs[]
observation_refs[]
memory_candidate_refs[]
created_at
updated_at
```

### 4.2 UserConfirmationQueueItem

```text
confirmation_item_id
kind
project_id
workflow_id
run_unit_id
product_command_id
source_ref
title
summary
risk_level
requested_by
requires_user
allowed_once_required
writes_project_files
writes_codex_home
writes_workbench_sidecars
confirmation_command_kind
blocked_reason
created_at
```

### 4.3 FailureControlSummary

```text
failure_id
source_kind
status
classification
user_message
developer_detail_ref
recoverability
recommended_next_step
retry_requires_user_confirmation
stop_requires_user_confirmation
memory_capture_compensation_needed
readback_result_count
audit_refs[]
runtime_log_refs[]
```

## 5. UI 显示边界确认

本任务会改前端：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [x] 改前端类型 / Tauri wrapper。
- [x] 改读模型摘要或状态显示。
- [x] 改已有页面局部 UI。
- [ ] 新增普通用户主导航入口。

普通 UI 应显示：

- 当前运行项：在做什么、卡在哪里、是否需要用户确认、下一步是什么。
- 待确认项：确认什么、为什么、风险、确认后会写哪里。
- 失败项：失败原因、readback 状态、是否可重试、为什么不能自动重试。
- J3 记忆捕获状态：是否已形成 capture / observation / candidate，是否需要用户或授权角色确认。

普通 UI 禁止显示：

- raw enum 长列表。
- sidecar 绝对路径。
- internal id 长列表。
- prompt body。
- full transcript。
- raw stdout / stderr。
- secret / credential。
- “自动重试中”“已自动修复”“已自动记住”“已写正式记忆”等越界文案。

显示位置：

- `运行中工作流`：作为 J4 主显示面。
- 右侧 `运行中`：显示最重要的当前运行摘要。
- 右侧 `待办`：显示用户确认队列。
- `项目` 工作流节点详情：显示该节点相关 queue / failure / confirmation 摘要。
- `智能体`：显示 session 相关 queue / failure / readback 状态。
- `记忆`：继续显示 capture / observation / candidate；不承接 retry / stop 动作。
- `设置 / 开发者`：raw refs、diagnostics、sidecar 名称和内部边界。

本任务不做手机端 UI，不新增 mobile responsive 规则。

## 6. 后端改动范围

允许改：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/runtime_session_attention.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/runtime_log_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/project_workflow_automation.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_capture_bus.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- 必要时新增小型模块，例如 `run_queue_read_model.rs` 或 `user_confirmation_queue.rs`。

默认不改：

- FormalMemory schema。
- provider / credential store。
- planned adapter 真实执行逻辑。
- legacy / H5 真实 runner 产品入口。
- workflow state 顶层结构，除非只派生读模型或向既有数组追加受控 refs 且有测试覆盖。

## 7. 前端改动范围

允许改：

- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/lib/runningWorkflows.ts`
- `prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts`
- `prototypes/productized-desktop-shell/src/lib/projectCanvas.ts`
- `prototypes/productized-desktop-shell/src/views/RunningWorkflowsView.tsx`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

前端原则：

- 运行中工作流不要做成开发者控制中心；用人话聚合状态。
- 待办必须是“用户需要处理的事”，不能混成通知。
- 运行中必须是“正在做 / 卡住 / 等待确认”，不能混成审计日志。
- 失败详情可以折叠到详情，不要把 raw refs 铺满普通屏。

## 8. 实现建议

建议实现顺序：

1. 后端或前端先派生 `RunQueueItem[]`，从 `real_execution_product_commands`、`project_workflow_automation`、`runtime_session_attention`、`session_run_status_summaries`、`memory_capture_store` 汇总。
2. 派生 `UserConfirmationQueueItem[]`，先只做 preview / confirmation envelope，不触发真实动作。
3. 派生 `FailureControlSummary[]`，把 unknown-result 保持为 `result_count=null`。
4. 接入 `RunningWorkflowsView` 和右侧 `运行中 / 待办`。
5. 在项目节点详情、智能体会话中心补最小摘要。
6. 更新秘书只读模型，让秘书解释“需要你确认什么 / 为什么不能自动重试”，但不替用户确认。
7. 补测试和禁止文案扫描。

## 9. 验收标准

必须通过：

- 有测试证明 `readback_unavailable` / `readback_failed` / `timed_out` 的 `result_count` 保持 null，不显示 0。
- 有测试证明 duplicate blocked 不会生成新的真实执行动作。
- 有测试证明 retry / stop / restart 默认只生成 confirmation / proposal，不调用 runner、不发送 prompt。
- 有测试证明用户确认队列能区分执行确认、retry 确认、结果确认、记忆候选 / formalization 确认。
- 有测试证明 J3 capture 半完成 / compensation-needed 状态能出现在 failure / confirmation 摘要里。
- UI 测试覆盖运行中工作流和右侧待办 / 运行中不会出现“自动重试中”“已自动修复”“已写正式记忆”等误导文案。
- prompt body / full transcript / secret / rollout 扫描无新增产品路径命中，或命中被分类为禁止项 / 历史文案 / 测试 fixture。

推荐命令：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib run_queue
cargo test --lib runtime_session_attention
cargo test --lib real_execution_command
cargo test --lib memory_capture
cargo test --lib project_workflow_automation
cargo test --lib
cargo fmt -- --check
```

如果没有新增 Rust 模块名 `run_queue`，对应命令应替换为实际模块测试名。

## 10. Evidence / Handoff

完成后新增：

- `evidence/2026-06-09-stage-j-j4-run-queue-failure-control-and-user-confirmation-queue-v1.md`
- `handoffs/2026-06-09-stage-j-j4-run-queue-failure-control-and-user-confirmation-queue-v1-result.md`

Evidence 必须记录：

- 改动范围。
- 运行队列数据契约。
- 用户确认队列数据契约。
- failure / readback / duplicate / timeout 分类。
- J3 capture compensation / 半完成状态如何显示。
- retry / stop / restart 默认不执行真实动作的证据。
- 测试结果。
- 真实执行边界：本任务默认未执行新的真实 Codex。

## 11. 复核线要求

完成实现和自测后，交给长期只读复核线审查：

- 是否存在 P0/P1 越界。
- 是否绕过统一 Product Command。
- 是否把 retry / stop / restart 做成无确认真实动作。
- 是否把 unknown result 显示成 0。
- 是否把 observation / candidate 冒充 FormalMemory。
- 是否把 J4 冒领为 J5 / J6 或 Stage J 完成。

主管线只在复核线无 P0/P1 且 fresh verify 通过后，才能把 J4 收口为 `accepted_with_deferred_items` 或 `accepted`。

## 12. 完成后不得声明

- 不得声明 Stage J 完成。
- 不得声明任意项目无限制自由执行完成。
- 不得声明自动 retry / stop / restart 已无条件可用。
- 不得声明 planned adapters 真实接入。
- 不得声明 provider credential / model verification 完成。
- 不得声明 FormalMemory 自动写入完成。
- 不得声明真实 Tauri J5 验收完成。
