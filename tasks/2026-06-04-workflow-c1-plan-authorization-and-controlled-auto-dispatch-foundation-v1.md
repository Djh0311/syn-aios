# Task Package：Workflow C1 Plan Authorization And Controlled Auto Dispatch Foundation v1

状态：已完成。  
用途：实现中间版本阶段 C 的第一步：方案授权对象、授权范围 guard、自动推进前置检查和只读授权状态展示。  
执行方式：一个中等批次完成；开发重点在后端授权对象、控制核心检查、审计和读模型，UI 只做项目工作流详情里的最小只读状态，不启动真实 worker。

完成记录：

- `evidence/2026-06-04-workflow-c1-plan-authorization-and-controlled-auto-dispatch-foundation-v1.md`
- `handoffs/2026-06-04-workflow-c1-plan-authorization-and-controlled-auto-dispatch-foundation-v1-result.md`

回收结论：

- 接受为 C1 方案授权对象、授权范围 guard、自动推进前置检查和项目工作流详情只读授权摘要完成。
- 不接受为阶段 C 完成、自动化工作流产品化闭环完成、真实 worker 已执行或真实 Codex 已执行。
- 本轮未执行 `codex exec` / `codex exec resume`，未读写 `/Users/yoyi/.codex`，未新增 `workflow-state.v0.json` 顶层结构，未修改 workflow / work item / node 状态枚举。
- 真实 Tauri 窗口 / 截图验收未完成；本轮使用离线 SSR 测试、typecheck、build 和 Rust 单测验收。

## 1. 先说薄弱点

- M6 已完成任务包记忆注入和第一条真实记忆闭环，但仍没有中间版本要求的“方案授权制”产品对象。
- 现有工作流已有任务包、prepared dispatch、真实 `codex exec resume` 代码路径和工作流机器历史能力，但这些能力还没有被一个可审计的方案授权范围统一约束。
- 如果直接继续真实派发，会把“每次手动确认”或“历史 demo runner”误当成中间版本自动化工作流闭环。
- 自动化工作流必须先能判断：某个 work item、任务包、角色、agent、读写范围、工具和测试是否在用户确认的方案授权内。
- 本任务会影响前端读模型和项目工作流详情文案，必须遵守 `docs/workbench-frontend-display-boundary-v1.md` 和 `docs/plans/task-package-ui-display-boundary-rule-v1.md`。

## 2. 任务目标

新增一层“方案授权”基础能力：

```text
方案草案 / 已确认方案
-> PlanAuthorization
-> Global boundary review status
-> AuthorizedExecutionScope
-> AutoDispatchGuard
-> work item / task package scope inspection
-> 项目工作流详情只读显示 authorized / blocked / needs_review
```

C1 完成后可以说：

- 工作台有可持久化、可读取、可审计的方案授权对象。
- 后端可以检查某个任务包或准备派发请求是否在授权范围内。
- 超出授权范围时能给出 deterministic blocked reason。
- 项目工作流详情可以只读显示方案授权状态和阻断摘要。
- 后续 C2/C3/C4 可以在这个基础上接项目咨询、用户确认、全局主管复核和受控自动派发。

C1 完成后仍不能说：

- 自动化工作流产品化闭环完成。
- 项目咨询已经能自动生成完整方案。
- 用户已经确认真实方案，除非本任务 evidence 里有明确确认记录。
- 全局主管已经真实复核任意业务方案，除非本任务 evidence 里有明确记录。
- 项目主管已经自动派发 worker。
- 真实 worker 已执行。
- 可以绕过用户确认执行 `codex exec` / `codex exec resume`。

## 3. 前置条件

必须已完成：

- `tasks/2026-06-03-agent-adapter-backend-capability-read-model-v1.md`
- `tasks/2026-06-04-memory-layer-m6-workflow-task-package-injection-and-end-to-end-loop-v1.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/workflow-task-package-design-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`

开始前必须复核：

- M6 只接受为第一条真实记忆闭环完成，不接受为自动化工作流产品化闭环。
- 现有 `prepare_workflow_node_dispatch_at` / `prepare_offline_role_dispatch_at` 只是准备态；真实执行路径仍是 `execute_workflow_node_dispatch_at`。
- 本任务默认不调用真实执行路径。
- 当前工作流事实源仍是工作台自己的 workflow state 和 sidecar；不能让 UI、Markdown 或画布直接触发真实命令。

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
- `docs/plans/memory-layer-implementation-slice-v1.md`

前置记录：

- `evidence/2026-06-03-agent-adapter-backend-capability-read-model-v1.md`
- `evidence/2026-06-04-memory-layer-m6-workflow-task-package-injection-and-end-to-end-loop-v1.md`
- `handoffs/2026-06-04-memory-layer-m6-workflow-task-package-injection-and-end-to-end-loop-v1-result.md`

当前实现重点：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/control_core.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_audit.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_read_model.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_store.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

## 5. 范围

允许：

- 新增后端模块，例如 `plan_authorization_store.rs` 或 `workflow_authorization.rs`。
- 新增独立 sidecar：`plan-authorizations.v1.json`，放在 workflow state 同级目录。
- 新增后端类型：
  - `PlanAuthorization`
  - `AuthorizedExecutionScope`
  - `PlanAuthorizationActorScope`
  - `PlanAuthorizationResourceScope`
  - `PlanAuthorizationStopCondition`
  - `AutoDispatchGuardInput`
  - `AutoDispatchGuardResult`
  - `PlanAuthorizationReadModel`
  - `PlanAuthorizationAuditEvent`
- 新增 Tauri 命令：
  - `load_plan_authorization_store`
  - `create_plan_authorization`
  - `record_plan_authorization_user_confirmation`
  - `record_plan_authorization_global_boundary_review`
  - `revoke_plan_authorization`
  - `inspect_auto_dispatch_authorization`
- 新增控制核心 helper，用于检查 work item / task package / prepared dispatch 是否在授权范围内。
- 给 prepared dispatch / offline dispatch 的准备态检查增加“授权检查摘要”，前提是只做 prepare / inspect，不调用真实执行。
- 在项目工作流详情侧栏显示最小只读“方案授权摘要”。
- 写工作台自己的审计事件，例如 `plan_authorization_created`、`plan_authorization_confirmed_by_user`、`plan_authorization_boundary_reviewed`、`auto_dispatch_scope_checked`。
- 新增 Rust 单测和前端离线测试，覆盖授权通过、缺授权阻断、越界阻断、撤销阻断和 UI 摘要。
- 更新 evidence / handoff / `CURRENT.md` / `tasks/README.md` / 阶段计划。

本任务显式授权的数据变更：

- 允许新增 `plan-authorizations.v1.json` sidecar。
- 允许在 prepared dispatch 记录中保存只读授权检查摘要或 `plan_authorization_id` 引用。
- 不允许新增 `workflow-state.v0.json` 顶层数组。
- 不允许修改 workflow / work item / node 既有状态枚举。
- 不允许迁移数据库。

禁止：

- 不执行真实 worker。
- 不执行真实 Codex。
- 不执行 `codex exec` / `codex exec resume`。
- 不读写 `/Users/yoyi/.codex`。
- 不创建新的 Codex session。
- 不把历史 workflow machine demo 当成中间版本产品化自动派发。
- 不让项目主管绕过方案授权派发。
- 不让全局主管逐条确认 worker 日常汇报。
- 不让秘书确认事实、复核成果、批准权限或派发任务。
- 不把 worker 汇报直接写正式事实或正式记忆。
- 不把 authorization UI 做成任务包管理器或治理后台。
- 不扫描完整 transcript。
- 不接外部模型、凭据或多 agent 真实执行。
- 不把 C1 说成阶段 C 完成。

如果执行者认为必须做真实 `codex exec` / `codex exec resume` 端到端验收，必须先停止并向用户申请明确授权，写清：

- 目标项目路径。
- 目标 Codex thread / session。
- 会写入哪些文件。
- 是否会写 `/Users/yoyi/.codex`。
- 备份和回滚方案。
- 超时、取消和失败处理。

## 5.1 UI 显示边界确认

本任务会改前端：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [x] 改前端类型 / Tauri wrapper，但不新增可见 UI。
- [x] 改读模型摘要或状态显示。
- [x] 改已有页面局部 UI。
- [ ] 新增入口、面板、tab、按钮或确认动作。

已读取：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `CURRENT.md`
- `tasks/README.md`

本任务允许显示：

- “方案授权摘要”最小只读状态。
- 授权状态：`未建立`、`待用户确认`、`待全局复核`、`授权有效`、`已暂停`、`已撤销`、`已过期`、`阻断`。
- 授权范围摘要：允许角色数量、允许 agent 数量、读写范围数量、允许工具 / 检查数量、停止条件数量。
- 当前 work item / task package 的授权检查结果：`authorized`、`blocked`、`needs_review`。
- blocked reason 的人话摘要，例如 `缺少有效方案授权`、`写入范围超出方案授权`、`目标 agent 不在授权范围内`、`触发必须请用户确认的停止条件`。

本任务禁止显示：

- 不新增一级入口。
- 不新增右侧顶级入口。
- 不新增项目页 tab。
- 不把授权对象 raw JSON、完整 sidecar、完整审计日志或内部 schema 铺进普通 UI。
- 不把项目工作流页变成授权后台或任务包管理器。
- 不显示“一键自动执行真实 worker”“自动调用 Codex”“自动完成项目”按钮。
- 不显示未实现的 Claude Code / OpenClaw / OpenCode 真实执行按钮。
- 不显示“用户已确认方案”“全局主管已复核”“worker 已自动执行”，除非有真实 evidence。

显示位置：

- 一级入口：不改。
- 右侧入口：不改。
- 项目页：允许在项目工作流侧栏、节点详情或任务包详情显示最小只读摘要。
- 画布：不在画布主区域新增授权面板；画布主区域仍只显示项目工作流。
- 记忆入口：不改。
- 知识库入口：不改。
- 智能体入口：不改。
- 管理入口：不改。

中间版本范围：

- 本轮必须落地：方案授权 sidecar、控制核心 guard、inspect 命令、审计、只读摘要。
- 本轮只做读模型 / 摘要：授权范围计数、授权状态、blocked reason、最近审计 id。
- 本轮后置：项目咨询自动生成完整方案、完整用户确认流、真实自动派发 worker、自动调度队列、失败重试、权限确认队列、最终结果复核页面。

后端和数据依赖：

- 授权状态必须来自后端正式读模型或 sidecar。
- 授权检查必须由控制核心执行，前端不能自己判定通过。
- 审计必须来自工作台正式 audit / authorization audit，不从日志伪造。
- 前端不能用假数据显示授权已通过。

UI 文案边界：

- 禁止说：“已自动执行”“worker 已启动”“Codex 已收到任务”“用户已确认真实方案”“自动化工作流闭环已完成”。
- 允许说：“方案授权摘要”“授权范围检查”“当前任务包在授权范围内”“当前任务包超出授权范围，需人工确认”“本轮未执行真实 worker”。

验收：

- 必须跑 `npm run typecheck`。
- 必须跑 `npm run test:offline-interaction`。
- 必须跑 `npm run build`。
- 如果改变项目页布局或新增确认动作，必须做真实窗口或浏览器截图验收。
- 如果当前对话没有可用截图工具，必须在 evidence / handoff 写明“真实窗口 / 截图验收未完成”。

## 6. 建议数据对象

```ts
type PlanAuthorizationStatus =
  | "draft"
  | "pending_user_confirmation"
  | "user_confirmed"
  | "pending_global_boundary_review"
  | "active"
  | "paused"
  | "revoked"
  | "expired"
  | "completed";
```

```ts
type AuthorizedExecutionScope = {
  project_id: string;
  workflow_id: string;
  allowed_role_ids: string[];
  allowed_agent_ids: string[];
  allowed_read_roots: string[];
  allowed_write_roots: string[];
  allowed_tools: string[];
  allowed_checks: string[];
  allowed_task_package_kinds: string[];
  max_worker_dispatches?: number;
  max_runtime_minutes?: number;
  stop_conditions: string[];
};
```

```ts
type PlanAuthorization = {
  authorization_id: string;
  schema_version: "plan_authorization.v1";
  project_id: string;
  workflow_id: string;
  source_proposal_id?: string | null;
  title: string;
  goal_summary: string;
  status: PlanAuthorizationStatus;
  scope: AuthorizedExecutionScope;
  user_confirmation?: {
    confirmed_by: "user";
    confirmed_at_ms: number;
    confirmation_summary: string;
  } | null;
  global_boundary_review?: {
    reviewed_by: "global_director";
    reviewed_at_ms: number;
    status: "approved" | "blocked" | "needs_changes";
    summary: string;
  } | null;
  audit_refs: string[];
  created_at_ms: number;
  updated_at_ms: number;
  expires_at_ms?: number | null;
};
```

```ts
type AutoDispatchGuardInput = {
  project_id: string;
  workflow_id: string;
  work_item_id: string;
  task_package_id?: string | null;
  target_role_id: string;
  target_agent_id?: string | null;
  requested_read_roots: string[];
  requested_write_roots: string[];
  requested_tools: string[];
  requested_checks: string[];
  dispatch_kind: "inspect_only" | "prepare_offline" | "prepare_real";
};
```

```ts
type AutoDispatchGuardResult = {
  status: "authorized" | "blocked" | "needs_review";
  authorization_id?: string | null;
  reasons: string[];
  required_user_confirmation: boolean;
  required_global_review: boolean;
  checked_at_ms: number;
};
```

## 7. 后端要求

- `plan-authorizations.v1.json` 必须包含 `schema_version`、`revision`、`authorizations[]`、`audit_events[]`。
- sidecar 写入必须有 lock、revision、备份、原子写和损坏 JSON 拒绝覆盖。
- 创建授权对象时必须校验 `project_id` / `workflow_id` 存在于当前 workflow state。
- `active` 必须同时满足用户确认和全局边界复核通过；C1 可以允许 `user_confirmed` / `pending_global_boundary_review` 作为中间状态，但 guard 不能把它当完全授权。
- `revoke` / `paused` / `expired` 状态必须让 guard 阻断。
- guard 必须 deterministic：同一 store、同一输入、同一时间条件下返回相同结果。
- guard 必须检查角色、agent、读范围、写范围、工具、检查、任务包类型和停止条件。
- 路径范围检查必须采用规范化路径，不能用字符串前缀假判断。
- `prepare_offline_role_dispatch_at` / prepared dispatch 相关路径只允许保存检查摘要，不允许因此调用真实执行。
- 真实 `execute_workflow_node_dispatch_at` 本轮不扩展为自动执行；如为了安全加 guard，只能让已有手动执行更保守，不能放宽。
- 所有状态变化必须写 authorization audit；如果写 workflow audit，也只能写摘要和引用。

## 8. 前端 / 读模型要求

- 新增 TS 类型和 Tauri wrapper。
- 新增纯函数读模型，把后端授权 store / guard result 派生成项目工作流详情摘要。
- 项目工作流侧栏只显示最小摘要，不显示 raw JSON。
- 没有授权对象时显示“未建立方案授权；不能自动推进”，不能显示 0 条授权造成误解。
- blocked 时显示最多 3 条人话 reason；完整审计后置到管理。
- 不新增一级入口、右侧入口、项目 tab、画布主区域面板。
- 不改秘书职责；秘书最多后续提醒“有待确认事项”，本任务不把秘书接进确认动作。

## 9. 验收

必须新增或更新测试，至少覆盖：

- 无授权对象时，auto dispatch guard 返回 `blocked`。
- `pending_user_confirmation` / `pending_global_boundary_review` 不允许自动推进。
- `active` 且范围匹配时返回 `authorized`。
- 写入范围越界时返回 `blocked` 且包含人话 reason。
- 角色或 agent 不在授权范围内时返回 `blocked`。
- revoked / paused / expired 授权返回 `blocked`。
- 项目工作流详情能显示授权摘要和 blocked reason。
- UI 文案不宣称真实 worker 已执行。

建议命令：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib plan_authorization
cargo test --lib workflow_authorization
cargo test --lib
rustfmt --check src/plan_authorization_store.rs src/control_core.rs src/commands.rs src/types.rs
```

如果模块命名不同，按实际文件调整 `cargo test` 和 `rustfmt --check`，但 evidence 必须写明。

自检搜索：

```text
rg -F 'codex exec' evidence handoffs tasks docs CURRENT.md STAGE_PLAN.md
rg -F '自动化工作流产品化闭环完成' CURRENT.md tasks README.md docs
rg -F 'worker 已执行' prototypes/productized-desktop-shell/src
```

搜索结果里如果出现相关文案，必须确认它们是否是禁止边界或历史记录，不能把 C1 写成已执行。

## 10. 回收要求

完成后新增：

- `evidence/2026-06-04-workflow-c1-plan-authorization-and-controlled-auto-dispatch-foundation-v1.md`
- `handoffs/2026-06-04-workflow-c1-plan-authorization-and-controlled-auto-dispatch-foundation-v1-result.md`

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
- 是否完成真实窗口 / 截图验收。
- 下一步是 C2：项目咨询方案生成和用户确认入口，还是先修复 C1 发现的阻断。
