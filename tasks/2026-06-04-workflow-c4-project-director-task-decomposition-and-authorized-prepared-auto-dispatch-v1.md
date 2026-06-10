# Task Package：Workflow C4 Project Director Task Decomposition And Authorized Prepared Auto Dispatch v1

状态：已完成。  
完成记录：`evidence/2026-06-04-workflow-c4-project-director-task-decomposition-and-authorized-prepared-auto-dispatch-v1.md`；`handoffs/2026-06-04-workflow-c4-project-director-task-decomposition-and-authorized-prepared-auto-dispatch-v1-result.md`。  
用途：实现中间版本阶段 C 的第四步：项目主管在 C3 active 授权范围内拆任务、生成子任务 / 任务包草案，并建立受控 prepared auto dispatch 链路。  
执行方式：一个中等批次完成；开发重点在授权范围校验、项目主管拆任务读写链路、任务包 / 记忆包冻结、prepared dispatch 落账和项目工作流侧栏摘要，不启动真实 worker。

## 1. 先说薄弱点

- C3 已让 authorization 进入 `active`，但 active 只表示授权有效，不等于 worker 已启动。
- 中间版本要求项目主管能在授权范围内自动推进；如果 C4 不做，C3 active 授权仍无法转成可派发的受控任务。
- 现有代码已有 task package、dispatch prepare、offline dispatch 和 C1 guard 等碎片能力，但还缺“C3 active authorization -> 项目主管拆任务 -> prepared dispatch”的产品化串联。
- C4 容易做过头：如果直接执行 `codex exec` 或把 prepared dispatch 显示成“已派发”，就会越过 C5 的 worker 汇报 / readback / 失败可见化边界。
- 本任务会改项目工作流侧栏 / 节点详情 UI，必须遵守前端显示边界规则，不能把任务包管理器铺成主界面。

## 2. 任务目标

新增“授权有效 -> 项目主管拆任务 -> 任务包准备 -> prepared auto dispatch”的受控链路：

```text
C3 active PlanAuthorization
-> C1 guard validates project director task scope
-> ProjectDirectorTaskPlan
-> worker work_items / workflow nodes / task package drafts
-> TaskMemoryPacket frozen snapshot
-> prepared dispatch record with authorization_check
-> 项目工作流详情显示“已准备派发；仍未执行 worker”
```

C4 完成后可以说：

- 项目主管可以基于 C3 active 授权拆出 worker 子任务。
- 每个子任务都有授权范围检查结果。
- 授权范围内的子任务可以生成任务包草案和任务记忆包冻结快照。
- 已绑定会话且检查通过的子任务可以建立 prepared dispatch 记录。
- prepared dispatch 会记录 authorization id、guard result、任务记忆包 snapshot 和 prompt preview。
- 项目工作流侧栏 / 节点详情可以显示“准备派发”状态和阻断原因。

C4 完成后仍不能说：

- 真实 worker 已执行。
- 真实 Codex 已执行。
- `codex exec` / `codex exec resume` 已执行。
- worker 已结构化汇报。
- 项目主管已确认过程事实。
- 自动化工作流产品化闭环完成。
- 全局主管已复核最终结果。

## 3. 前置条件

必须已完成：

- `tasks/2026-06-04-workflow-c1-plan-authorization-and-controlled-auto-dispatch-foundation-v1.md`
- `tasks/2026-06-04-workflow-c2-project-consultation-proposal-and-user-confirmation-entry-v1.md`
- `tasks/2026-06-04-workflow-c3-global-boundary-review-and-authorization-activation-v1.md`
- `evidence/2026-06-04-workflow-c3-global-boundary-review-and-authorization-activation-v1.md`
- `handoffs/2026-06-04-workflow-c3-global-boundary-review-and-authorization-activation-v1-result.md`

开始前必须复核：

- C3 active authorization 必须存在，且 `global_boundary_review.status = approved`。
- C1 guard 对目标 project / workflow / task package kind / role / agent / 读写范围 / 工具 / 检查仍返回 `authorized`。
- C2 confirmed proposal 和 authorization 回链仍匹配。
- prepared dispatch 不是执行态，不得写 started / ended / exit code / transcript readback。
- 如果目标 worker 节点没有 active session binding，只能记录待绑定 / 阻断摘要，不能伪造 prepared dispatch 已可执行。

## 4. 必须先读

当前入口：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `docs/middleware-version-development-plan-v1.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/workflow-task-package-design-v1.md`
- `docs/workbench-system-architecture-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `docs/plans/2026-06-01-project-workflow-canvas-node-schema-v1.md`

前置记录：

- `evidence/2026-06-04-workflow-c1-plan-authorization-and-controlled-auto-dispatch-foundation-v1.md`
- `handoffs/2026-06-04-workflow-c1-plan-authorization-and-controlled-auto-dispatch-foundation-v1-result.md`
- `evidence/2026-06-04-workflow-c2-project-consultation-proposal-and-user-confirmation-entry-v1.md`
- `handoffs/2026-06-04-workflow-c2-project-consultation-proposal-and-user-confirmation-entry-v1-result.md`
- `evidence/2026-06-04-workflow-c3-global-boundary-review-and-authorization-activation-v1.md`
- `handoffs/2026-06-04-workflow-c3-global-boundary-review-and-authorization-activation-v1-result.md`

当前实现重点：

- `prototypes/productized-desktop-shell/src-tauri/src/plan_authorization_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/project_consultation_proposal_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/task_memory_injection.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/task_memory_packet_builder.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/control_core.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/planAuthorization.ts`
- `prototypes/productized-desktop-shell/src/lib/projectConsultationProposal.ts`
- `prototypes/productized-desktop-shell/src/lib/candidateGovernance.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

## 5. 范围

允许：

- 新增或扩展类型：
  - `ProjectDirectorTaskPlan`
  - `ProjectDirectorPlannedTask`
  - `ProjectDirectorTaskScope`
  - `AuthorizedPreparedDispatchPlan`
  - `AuthorizedPreparedDispatchResult`
  - `PreparedAutoDispatchReadModel`
- 新增后端命令或 wrapper，例如：
  - `preview_project_director_task_plan`
  - `prepare_authorized_auto_dispatch`
  - 或按现有命名约定拆为更小命令。
- 复用 C1 `inspect_auto_dispatch_authorization` / guard，不新增绕过 guard 的本地判断。
- 复用 C2 confirmed proposal 和 C3 active authorization。
- 从 C2 proposal / authorization scope / 当前 workflow state 生成 deterministic 项目主管任务拆解草案。
- 在已有 `workflow-state.v0.json` 结构中追加或更新现有数组项：
  - `work_items[]`
  - `nodes[]`
  - `edges[]`
  - `artifacts[]`
  - `node_dispatches[]`
  - `audit_events[]`
  - 如现有结构已有 task package / execution control / permission request 对应数组，也可按既有 schema 使用。
- 为每个 in-scope planned task 生成 task package draft / artifact。
- 调用 M6 任务记忆包注入能力，把正式记忆冻结快照挂到任务包 artifact / prompt preview。
- 对有 active binding 且 guard authorized 的子任务创建 prepared dispatch 记录。
- 对缺 binding、scope 越界、lint 阻断、任务包不完整等情况生成 blocked / needs_setup 摘要。
- 在项目工作流侧栏 / 节点详情显示“项目主管拆任务”和“准备派发”摘要。
- 新增 Rust 单测和前端离线测试。
- 更新 evidence / handoff / `CURRENT.md` / `tasks/README.md` / `AUTHORITY.md` / `STAGE_PLAN.md` / 阶段计划。

本任务显式授权的数据变更：

- 允许读 `plan-authorizations.v1.json`、`project-proposals.v1.json`、`formal-memories.v1.json`、`memory-candidates.v1.json`、`observations.v1.json`、`memory-lint.v1.json`。
- 允许更新已有 `workflow-state.v0.json` 的既有数组项，用于记录子任务、worker 节点、任务包 artifact、prepared dispatch 和 audit。
- 允许更新任务包 artifact 的 memory packet snapshot / prompt preview / authorization check 引用。
- 不允许新增 `workflow-state.v0.json` 顶层数组。
- 不允许修改 workflow / work item / node / dispatch 既有状态枚举；如果现有状态无法表达 prepared / blocked / needs_setup，必须先停下并回报。
- 不允许迁移数据库。

禁止：

- 不执行真实 worker。
- 不执行真实 Codex。
- 不执行 `codex exec` / `codex exec resume`。
- 不读写 `/Users/yoyi/.codex`。
- 不创建新的 Codex session。
- 不调用外部 LLM 做项目主管拆任务。
- 不把项目主管拆任务结果直接当 worker 已完成事实。
- 不把 prepared dispatch 显示成“已启动”“已派发到 Codex”“worker 执行中”。
- 不写 execution attempt started / running / completed。
- 不做 dispatch readback。
- 不确认 worker 汇报。
- 不把任务包内容、prepared prompt 或 worker 计划写成正式记忆。
- 不让秘书拆任务、派发任务或裁判结果。

如果执行者认为必须做真实 Codex / worker 端到端验证，必须先停止并向用户申请明确授权，写清：

- 目标项目路径。
- 目标 agent / session。
- 是否会写 `/Users/yoyi/.codex`。
- 会读取哪些上下文。
- 会写入哪些文件、workflow state 或 sidecar。
- 超时、取消和失败处理。

## 5.1 UI 显示边界确认

本任务会改前端：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [x] 改前端类型 / Tauri wrapper，但不新增可见 UI。
- [x] 改读模型摘要或状态显示。
- [x] 改已有页面局部 UI。
- [x] 新增准备动作。

已读取：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `CURRENT.md`
- `tasks/README.md`

本任务允许显示：

- “项目主管拆任务”卡片。
- “准备派发”卡片或节点详情摘要。
- 子任务数量、可准备数量、阻断数量。
- 授权检查摘要：`授权范围检查通过` 或最多 3 条 blocked reason。
- 任务包准备状态：`任务包草案已生成`、`记忆快照已冻结`、`等待绑定会话`、`准备派发已记录`。
- prepared dispatch 状态：`已准备；仍未执行 worker`。
- 任务记忆包轻提示：`使用了 N 条正式记忆`、`排除了 N 条候选 / 观察 / lint 阻断项`。
- 动作：`生成拆任务草案`、`准备授权范围内派发`、`刷新授权检查`。

本任务禁止显示：

- 不新增一级入口。
- 不新增右侧顶级入口。
- 不新增项目页 tab。
- 不在画布主区域铺任务包正文、完整 prompt、raw JSON、审计日志、sidecar 路径或内部 schema。
- 不显示“worker 已启动”“Codex 已收到任务”“自动派发已开始”“worker 执行中”“项目已自动完成”。
- 不把 prepared dispatch 显示成真实 dispatch。
- 不把秘书显示为拆任务者、派发者或成果裁判。
- 不显示未实现的“一键真实执行 worker”按钮。

显示位置：

- 一级入口：不改。
- 右侧入口：不改。
- 项目页：允许在项目工作流侧栏、节点详情或任务包详情卡显示摘要和动作。
- 画布：主区域仍只显示项目工作流画布；拆任务和 prepared dispatch 信息只能在详情侧栏。
- 记忆入口：不改。
- 知识库入口：不改。
- 智能体入口：不改。
- 管理入口：不改。

中间版本范围：

- 本轮必须落地：项目主管拆任务草案、授权 guard 校验、任务包 / 记忆包准备、prepared dispatch 记录和只读摘要。
- 本轮只做读模型 / 摘要：planned task count、prepared dispatch count、blocked reason、authorization id、memory snapshot summary、binding status。
- 本轮后置：真实 worker 执行、readback、结构化汇报、项目主管过程事实确认、失败重试、最终结果复核。

后端和数据依赖：

- 拆任务和 prepared dispatch 必须来自后端命令。
- 前端不能 mock “已准备派发”。
- prepared dispatch 必须带 C1 guard result。
- task package memory snapshot 必须来自 M6 已实现能力。

UI 文案边界：

- 禁止说：“worker 已执行”“自动派发已开始”“Codex 已收到任务”“项目已自动完成”。
- 允许说：“准备派发已记录”“已准备；仍未执行 worker”“等待会话绑定后才能准备派发”“越界任务已阻断”。

验收：

- 必须跑 `npm run typecheck`。
- 必须跑 `npm run test:offline-interaction`。
- 必须跑 `npm run build`。
- 因新增准备动作和项目页局部 UI，必须做真实窗口或浏览器截图验收。
- 如果当前对话没有可用截图工具，必须在 evidence / handoff 写明“真实窗口 / 截图验收未完成”。

## 6. 建议数据对象

```ts
type ProjectDirectorPlannedTaskStatus =
  | "draft"
  | "authorized"
  | "blocked"
  | "needs_binding"
  | "prepared";
```

```ts
type ProjectDirectorTaskScope = {
  project_id: string;
  workflow_id: string;
  target_role: "worker" | "project_director" | string;
  task_package_kind: string;
  allowed_read_scope: string[];
  allowed_write_scope: string[];
  callable_tool_capabilities: string[];
  required_checks: string[];
  stop_conditions: string[];
};
```

```ts
type ProjectDirectorPlannedTask = {
  planned_task_id: string;
  title: string;
  objective: string;
  scope: ProjectDirectorTaskScope;
  depends_on: string[];
  acceptance_criteria: string[];
  report_format: string[];
  status: ProjectDirectorPlannedTaskStatus;
  guard_result?: AutoDispatchGuardResult | null;
  work_item_id?: string | null;
  workflow_node_id?: string | null;
  task_package_id?: string | null;
  memory_packet_snapshot_id?: string | null;
  prepared_dispatch_id?: string | null;
  blocked_reasons: string[];
};
```

```ts
type PrepareAuthorizedAutoDispatchInput = {
  project_id: string;
  workflow_id: string;
  proposal_id: string;
  authorization_id: string;
  actor_id: "project_director" | string;
  planned_tasks: ProjectDirectorPlannedTask[];
  expected_workflow_revision?: number | null;
  expected_authorization_revision?: number | null;
};
```

## 7. 后端要求

- prepared dispatch 前必须要求：
  - proposal 存在且 status 为 `user_confirmed`。
  - authorization 存在且 status 为 `active`。
  - authorization 有 C3 approved `global_boundary_review`。
  - proposal `plan_authorization_id` 匹配 authorization。
  - project_id / workflow_id 全部一致。
  - planned task 的 role / agent / task package kind / 读写范围 / 工具 / 检查 / stop conditions 都在 authorization scope 内。
  - C1 guard 返回 `authorized`。
  - task package 必填字段完整。
  - memory packet snapshot 成功生成或明确记录缺失 / stale 阻断。
- 如果目标 worker 节点没有 active binding，不得创建可执行 prepared dispatch；应返回 `needs_binding`。
- prepared dispatch record 必须包含：
  - `plan_authorization_id`
  - `authorization_check`
  - `memory_packet_snapshot_id`
  - `memory_packet_fingerprint`
  - `prompt_preview`
  - `state = prepared` 或现有等价准备态
  - `started_at = null`
  - `ended_at = null`
  - `exit_code = null`
- 需要写 audit，例如：
  - `project_director_task_plan_created`
  - `authorized_prepared_dispatch_created`
  - `authorized_prepared_dispatch_blocked`
- 重复准备同一 planned task 必须幂等或明确拒绝，防止重复 dispatch。
- 所有 ID 必须 deterministic 或可追溯，避免同一输入反复生成一堆幽灵任务。
- 如果使用现有 `prepare_workflow_node_dispatch` 或 `prepare_offline_role_dispatch`，必须在外层补 C3 authorization / C1 guard / task memory snapshot 校验。

## 8. 前端 / 读模型要求

- 新增 TS 类型和 Tauri wrapper。
- 新增或扩展纯函数读模型，把 active authorization + project workflow + task package + dispatch records 派生成 C4 摘要。
- 项目工作流侧栏显示：
  - active authorization id。
  - planned task count。
  - prepared dispatch count。
  - blocked / needs_binding count。
  - memory snapshot summary。
  - 最多 3 条 blocked reason。
- `准备授权范围内派发` 必须有确认弹层，明确“只创建准备记录，不启动 worker”。
- 缺 active authorization 时禁用准备动作并显示原因。
- 越界任务必须显示阻断原因，不能静默丢弃。
- 不显示完整 prompt、raw sidecar、完整 audit、内部 schema；只显示摘要和必要明细。

## 9. 验收

必须新增或更新测试，至少覆盖：

- 没有 active authorization 时，C4 准备派发被拒绝。
- C3 approved 不存在时，C4 准备派发被拒绝。
- proposal / authorization 回链不匹配时被拒绝。
- planned task 读写范围越界时被 C1 guard 阻断。
- planned task 工具 / 检查 / task package kind 越界时被阻断。
- in-scope planned task 生成 work item / worker node / task package draft。
- task package 包含 M6 task memory packet snapshot。
- 已绑定会话且 guard authorized 时创建 prepared dispatch。
- 缺 binding 时返回 `needs_binding`，不创建可执行 prepared dispatch。
- 重复准备不会生成重复 dispatch。
- prepared dispatch 不写 started / ended / exit code / transcript readback。
- UI 显示“已准备；仍未执行 worker”。
- UI 不显示“worker 已执行 / 自动派发已开始 / Codex 已收到任务”。

建议命令：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib authorized_prepared_dispatch
cargo test --lib project_director_task_plan
cargo test --lib task_memory_injection
cargo test --lib plan_authorization
cargo test --lib workflow_authorization
cargo test --lib
rustfmt --check src/plan_authorization_store.rs src/project_consultation_proposal_store.rs src/task_memory_injection.rs src/control_core.rs src/commands.rs src/types.rs
```

如果模块命名不同，按实际文件调整 `cargo test` 和 `rustfmt --check`，但 evidence 必须写明。

自检搜索：

```text
rg -F 'codex exec' evidence handoffs tasks docs CURRENT.md STAGE_PLAN.md
rg -F 'worker 已执行' prototypes/productized-desktop-shell/src
rg -F '自动派发已开始' prototypes/productized-desktop-shell/src
rg -F 'Codex 已收到任务' prototypes/productized-desktop-shell/src
```

搜索结果里如果出现相关文案，必须确认它们是否是禁止边界或历史记录，不能把 C4 写成已执行。

## 10. 回收要求

完成后新增：

- `evidence/2026-06-04-workflow-c4-project-director-task-decomposition-and-authorized-prepared-auto-dispatch-v1.md`
- `handoffs/2026-06-04-workflow-c4-project-director-task-decomposition-and-authorized-prepared-auto-dispatch-v1-result.md`

必须更新：

- 本任务包状态。
- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `docs/plans/middleware-version-stage-plan-v1.md`

回收结论必须明确：

- 接受为什么。
- 不接受为什么。
- 是否执行真实 Codex。
- 是否读写 `/Users/yoyi/.codex`。
- 是否改 `workflow-state.v0.json` 顶层结构。
- 是否新增或修改状态枚举。
- 是否创建 prepared dispatch。
- 是否启动任何 worker。
- 是否写 execution attempt / readback。
- 是否完成真实窗口 / 截图验收。
- 下一步是 C5：worker 结构化汇报、项目主管过程事实确认和失败 / readback / 权限可见化，还是先修复 C4 发现的阻断。
