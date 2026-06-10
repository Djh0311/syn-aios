# Task Package：Workflow C2 Project Consultation Proposal And User Confirmation Entry v1

状态：已完成。  
用途：实现中间版本阶段 C 的第二步：项目咨询方案草案、用户确认入口，以及与 C1 `PlanAuthorization` 的受控衔接。  
执行方式：一个中等批次完成；开发重点在后端方案草案 store、确认状态机、授权对象创建 / 确认联动、审计和读模型，UI 只做项目工作流侧栏 / 详情里的方案草案与确认动作，不启动真实 worker。

完成记录：

- `evidence/2026-06-04-workflow-c2-project-consultation-proposal-and-user-confirmation-entry-v1.md`
- `handoffs/2026-06-04-workflow-c2-project-consultation-proposal-and-user-confirmation-entry-v1-result.md`

回收结论：

- 接受为 C2 项目咨询方案草案 sidecar、用户决定入口、C1 `PlanAuthorization` 联动、审计和项目工作流侧栏确认 UI 完成。
- 不接受为全局主管边界复核完成、授权 active、真实 worker 已执行、真实 Codex 已执行、自动化工作流产品化闭环完成或真实项目咨询 agent 已接入。
- 用户确认 proposal 后，授权仍停在 `pending_global_boundary_review`，C1 guard 仍返回 `needs_review`，不能自动派发。
- 本轮未执行 `codex exec` / `codex exec resume`，未新增 `workflow-state.v0.json` 顶层结构，未修改 workflow / work item / node 状态枚举。
- 真实截图验收未完成：当前对话未暴露浏览器截图工具，项目未安装 Playwright；已做 Vite 本地 HTTP smoke。

## 1. 先说薄弱点

- C1 已有 `PlanAuthorization`、授权范围 guard 和自动推进前置检查，但还没有“用户看得懂、可确认的方案草案”作为授权来源。
- 现在如果直接创建或确认授权对象，用户看到的是偏内部的 scope / guard 信息，不是完整项目咨询方案。
- 中间版本要求先由项目咨询把用户目标整理成方案，再由用户确认方案；不能把 raw authorization JSON 当成方案。
- 用户确认方案后，只表示进入“待全局边界复核”或等价中间状态；C2 不能让授权直接变成可自动派发。
- 本任务会新增确认动作和 UI 文案，必须严格遵守 `docs/workbench-frontend-display-boundary-v1.md` 与 `docs/plans/task-package-ui-display-boundary-rule-v1.md`。

## 2. 任务目标

新增“项目咨询方案草案 -> 用户确认 -> C1 授权对象”的受控链路：

```text
用户目标 / 项目上下文
-> ProjectConsultationProposal draft
-> 方案草案只读 / 可确认展示
-> 用户确认 / 要求修改 / 拒绝
-> create PlanAuthorization with source_proposal_id
-> record PlanAuthorization user confirmation
-> 授权状态停在 user_confirmed / pending_global_boundary_review
-> 项目工作流详情显示“方案已由用户确认，待全局边界复核”
```

C2 完成后可以说：

- 工作台有可持久化、可读取、可审计的项目咨询方案草案。
- 方案草案能表达目标、范围、允许角色 / agent、读写范围、工具 / 检查、停止条件、验收方式和风险。
- 用户可以对方案草案做 `confirm` / `request_changes` / `reject` 之一的受控决定。
- 用户确认会创建或关联 C1 `PlanAuthorization`，并记录用户确认。
- 用户确认后授权仍需 C3 全局主管边界复核，不能自动派发。

C2 完成后仍不能说：

- 全局主管已经复核方案边界。
- 方案授权已经 active。
- 项目主管已经自动派发 worker。
- 真实 worker 已执行。
- 自动化工作流产品化闭环完成。
- C2 已经接入真实项目咨询 LLM / Codex 会话，除非另有明确实现和 evidence。
- 可以绕过 C1 guard 执行真实 `codex exec` / `codex exec resume`。

## 3. 前置条件

必须已完成：

- `tasks/2026-06-04-workflow-c1-plan-authorization-and-controlled-auto-dispatch-foundation-v1.md`
- `evidence/2026-06-04-workflow-c1-plan-authorization-and-controlled-auto-dispatch-foundation-v1.md`
- `handoffs/2026-06-04-workflow-c1-plan-authorization-and-controlled-auto-dispatch-foundation-v1-result.md`

开始前必须复核：

- C1 的 `PlanAuthorization` status 和 `record_plan_authorization_user_confirmation` 行为。
- C1 guard 对 `user_confirmed` / `pending_global_boundary_review` 的结果必须仍是 `needs_review` 或 blocked，不允许自动派发。
- 当前是否已有 proposal / consultation 类型、字段或 UI 草案；如有，优先复用，不要重复造一套相近概念。
- 当前工作流事实源仍是工作台自己的 workflow state、sidecar 和控制核心；UI 不能直接触发真实命令。

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
- `evidence/2026-06-04-memory-layer-m6-workflow-task-package-injection-and-end-to-end-loop-v1.md`

当前实现重点：

- `prototypes/productized-desktop-shell/src-tauri/src/plan_authorization_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/control_core.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_audit.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_read_model.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/lib/planAuthorization.ts`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

## 5. 范围

允许：

- 新增后端模块，例如 `project_consultation_proposal_store.rs`。
- 新增独立 sidecar：`project-proposals.v1.json`，放在 workflow state 同级目录。
- 新增后端 / 前端类型：
  - `ProjectConsultationProposal`
  - `ProjectConsultationProposalStoreV1`
  - `ProjectConsultationProposalSection`
  - `ProjectConsultationProposalScopeDraft`
  - `ProjectConsultationProposalRisk`
  - `ProjectConsultationProposalDecision`
  - `ProjectConsultationProposalReadModel`
  - `CreateProjectConsultationProposalInput`
  - `RecordProjectConsultationProposalDecisionInput`
- 新增 Tauri 命令：
  - `load_project_consultation_proposal_store`
  - `create_project_consultation_proposal`
  - `render_project_consultation_proposal_markdown`
  - `record_project_consultation_proposal_decision`
- 在用户确认方案时，调用或复用 C1 `create_plan_authorization` / `record_plan_authorization_user_confirmation` 对应后端逻辑，保证 `source_proposal_id` 回链。
- 在项目工作流侧栏 / 节点详情显示最小“项目咨询方案”卡片和用户确认状态。
- 新增确认动作，但必须弹出明确确认摘要，显示本轮不执行真实 worker。
- 写 proposal audit，例如 `project_consultation_proposal_created`、`project_consultation_proposal_confirmed_by_user`、`project_consultation_proposal_changes_requested`、`project_consultation_proposal_rejected`。
- 新增 Rust 单测和前端离线测试，覆盖草案创建、确认生成授权、拒绝不生成授权、要求修改不生成授权、UI 文案边界。
- 更新 evidence / handoff / `CURRENT.md` / `tasks/README.md` / `AUTHORITY.md` / `STAGE_PLAN.md` / 阶段计划。

本任务显式授权的数据变更：

- 允许新增 `project-proposals.v1.json` sidecar。
- 允许通过 C1 现有 sidecar 写入或更新 `plan-authorizations.v1.json` 中与 confirmed proposal 关联的授权对象。
- 允许在 proposal store 内保存 `plan_authorization_id` 回链。
- 不允许新增 `workflow-state.v0.json` 顶层数组。
- 不允许修改 workflow / work item / node 既有状态枚举。
- 不允许迁移数据库。

禁止：

- 不执行真实 worker。
- 不执行真实 Codex。
- 不执行 `codex exec` / `codex exec resume`。
- 不读写 `/Users/yoyi/.codex`。
- 不创建新的 Codex session。
- 不调用外部 LLM 或本地 agent 生成方案，除非另有单独任务包和用户明确授权。
- 不把“模板生成方案草案”说成真实项目咨询智能体已完成。
- 不让用户确认后直接把授权置为 active；C3 才处理全局主管边界复核和授权生效。
- 不绕过 C1 guard 自动派发。
- 不让秘书确认方案、判断方案正确或批准授权；秘书最多后续提醒有待确认事项。
- 不把 proposal 草案写成正式事实或正式记忆。
- 不把 proposal store 当知识库或记忆层权威。
- 不把项目页变成任务包管理器或方案后台。

如果执行者认为必须让真实 Codex / 项目咨询 agent 生成方案，必须先停止并向用户申请明确授权，写清：

- 目标项目路径。
- 目标 agent / session。
- 是否会写 `/Users/yoyi/.codex`。
- 会读取哪些上下文。
- 会写入哪些文件或 sidecar。
- 超时、取消和失败处理。

## 5.1 UI 显示边界确认

本任务会改前端：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [x] 改前端类型 / Tauri wrapper，但不新增可见 UI。
- [x] 改读模型摘要或状态显示。
- [x] 改已有页面局部 UI。
- [x] 新增确认动作。

已读取：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `CURRENT.md`
- `tasks/README.md`

本任务允许显示：

- “项目咨询方案草案”或“方案确认”卡片。
- 方案状态：`草案`、`待用户确认`、`用户已确认，待全局复核`、`用户要求修改`、`用户已拒绝`。
- 方案摘要：目标、范围、主要步骤、允许角色 / agent、读写范围、工具 / 检查、必须停下来的条件、验收方式、风险。
- 用户确认动作：`确认方案范围`、`要求修改`、`拒绝方案`。
- 确认后显示：`已记录用户确认；仍需全局主管复核后才可自动推进`。

本任务禁止显示：

- 不新增一级入口。
- 不新增右侧顶级入口。
- 不新增项目页 tab。
- 不在画布主区域铺完整方案正文、raw JSON、审计日志或内部 schema。
- 不显示“已开始自动执行”“worker 已启动”“Codex 已收到任务”“授权已生效可自动派发”，除非后续 C3/C4 有真实 evidence。
- 不显示未实现的“AI 自动生成完整方案并派发”按钮。
- 不把秘书显示为方案确认者。
- 不把 proposal / authorization / audit 路径大表放入普通 UI。

显示位置：

- 一级入口：不改。
- 右侧入口：不改。
- 项目页：允许在项目工作流侧栏、节点详情或方案详情卡显示草案和确认动作。
- 画布：主区域仍只显示项目工作流画布；方案摘要只能作为详情侧栏内容。
- 记忆入口：不改。
- 知识库入口：不改。
- 智能体入口：不改。
- 管理入口：不改。

中间版本范围：

- 本轮必须落地：proposal sidecar、草案创建 / 渲染、用户决定、C1 授权对象联动、审计和只读摘要。
- 本轮只做读模型 / 摘要：方案状态、用户决定状态、授权回链状态、风险数量、停止条件数量。
- 本轮后置：真实项目咨询 agent、全局主管边界复核、授权 active、项目主管拆任务、真实 worker 派发、失败重试、完整结果报告。

后端和数据依赖：

- 方案草案必须来自后端 proposal store 或正式读模型。
- 用户确认必须写 proposal audit，并通过后端联动 C1 authorization store。
- 前端不能 mock 用户已确认。
- 用户确认后 guard 仍不能通过真实自动派发，直到 C3 全局边界复核通过。

UI 文案边界：

- 禁止说：“自动化工作流已开始”“方案授权已生效”“项目主管已开始派发”“worker 已执行”“系统已自动完成方案”。
- 允许说：“项目咨询方案草案”“确认方案范围”“已记录用户确认，待全局主管复核”“本轮不会启动真实 worker”。

验收：

- 必须跑 `npm run typecheck`。
- 必须跑 `npm run test:offline-interaction`。
- 必须跑 `npm run build`。
- 因新增确认动作和项目页局部 UI，必须做真实窗口或浏览器截图验收。
- 如果当前对话没有可用截图工具，必须在 evidence / handoff 写明“真实窗口 / 截图验收未完成”。

## 6. 建议数据对象

```ts
type ProjectConsultationProposalStatus =
  | "draft"
  | "pending_user_confirmation"
  | "user_confirmed"
  | "changes_requested"
  | "rejected"
  | "superseded";
```

```ts
type ProjectConsultationProposalScopeDraft = {
  allowed_role_ids: string[];
  allowed_agent_ids: string[];
  allowed_read_roots: string[];
  allowed_write_roots: string[];
  allowed_tools: string[];
  allowed_checks: string[];
  allowed_task_package_kinds: string[];
  stop_conditions: string[];
  max_worker_dispatches?: number;
  max_runtime_minutes?: number;
};
```

```ts
type ProjectConsultationProposal = {
  proposal_id: string;
  schema_version: "project_consultation_proposal.v1";
  project_id: string;
  workflow_id: string;
  title: string;
  user_goal: string;
  goal_summary: string;
  proposed_steps: string[];
  scope_draft: ProjectConsultationProposalScopeDraft;
  risks: ProjectConsultationProposalRisk[];
  acceptance_criteria: string[];
  status: ProjectConsultationProposalStatus;
  plan_authorization_id?: string | null;
  created_by_role: "project_consultant" | "project_director" | "user";
  created_at_ms: number;
  updated_at_ms: number;
};
```

```ts
type ProjectConsultationProposalDecision = {
  decision_id: string;
  proposal_id: string;
  decided_by: "user";
  decision: "confirm" | "request_changes" | "reject";
  summary: string;
  created_at_ms: number;
};
```

## 7. 后端要求

- `project-proposals.v1.json` 必须包含 `schema_version`、`revision`、`proposals[]`、`decisions[]`、`audit_events[]`。
- sidecar 写入必须有 lock、revision、备份、原子写和损坏 JSON 拒绝覆盖。
- 创建 proposal 必须校验 `project_id` / `workflow_id` 存在于当前 workflow state。
- proposal 草案必须有 `user_goal`、`goal_summary`、至少一个 proposed step、至少一个 acceptance criterion。
- proposal 的 scope draft 必须能转换成 C1 `AuthorizedExecutionScope`，且转换过程必须可测试。
- `confirm` 决定必须：
  - 只允许 `pending_user_confirmation` / `draft` 等未终结状态。
  - 创建或关联 `PlanAuthorization`。
  - 写入 `source_proposal_id`。
  - 记录 C1 用户确认。
  - 保持授权为 `user_confirmed` 或 `pending_global_boundary_review`，不能变为 `active`。
  - 写 proposal decision / audit。
- `request_changes` / `reject` 不得创建 active authorization，不得允许自动派发。
- 同一 proposal 已确认后不能重复确认生成多个 active candidate；如允许重新确认，必须先 supersede 旧 proposal。
- 所有后端错误必须返回人话原因，不能只返回 raw JSON parse error。

## 8. 前端 / 读模型要求

- 新增 TS 类型和 Tauri wrapper。
- 新增纯函数读模型，把 proposal store + C1 authorization store 派生成项目工作流侧栏摘要。
- 方案草案卡片必须显示“草案 / 需要用户确认 / 用户已确认但待全局复核”等状态。
- 用户确认动作必须有确认弹层，摘要包括目标、读写范围、允许工具、停止条件和“不会启动真实 worker”。
- `request_changes` 必须允许用户填写简短原因；不需要实现完整编辑器。
- 没有 proposal 时显示“还没有项目咨询方案草案”，不能显示成 0 条方案已完成。
- 如果 proposal 已确认但 authorization 缺失，必须显示阻断或修复提示，不能显示可自动推进。
- UI 不显示 raw sidecar、完整 audit、内部 id 大表；id 可以在小字或开发者详情中后置，本轮不做开发者详情。

## 9. 验收

必须新增或更新测试，至少覆盖：

- 创建 proposal 草案成功，proposal store revision 增加。
- 缺少 `user_goal` / `goal_summary` / acceptance criteria 会被拒绝。
- 用户确认 proposal 会创建 / 关联 `PlanAuthorization`，并写 `source_proposal_id`。
- 用户确认后授权不是 `active`，guard 仍不能自动派发。
- 用户 request changes 不创建 authorization。
- 用户 reject 不创建 authorization。
- 重复确认被拒绝或受控 supersede。
- UI 显示方案草案、确认动作和“待全局复核”文案。
- UI 不显示“worker 已执行 / 自动派发已开始”。

建议命令：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib project_consultation_proposal
cargo test --lib plan_authorization
cargo test --lib workflow_authorization
cargo test --lib
rustfmt --check src/project_consultation_proposal_store.rs src/plan_authorization_store.rs src/control_core.rs src/commands.rs src/types.rs
```

如果模块命名不同，按实际文件调整 `cargo test` 和 `rustfmt --check`，但 evidence 必须写明。

自检搜索：

```text
rg -F 'codex exec' evidence handoffs tasks docs CURRENT.md STAGE_PLAN.md
rg -F 'worker 已执行' prototypes/productized-desktop-shell/src
rg -F '授权已生效' prototypes/productized-desktop-shell/src
```

搜索结果里如果出现相关文案，必须确认它们是否是禁止边界或历史记录，不能把 C2 写成已执行。

## 10. 回收要求

完成后新增：

- `evidence/2026-06-04-workflow-c2-project-consultation-proposal-and-user-confirmation-entry-v1.md`
- `handoffs/2026-06-04-workflow-c2-project-consultation-proposal-and-user-confirmation-entry-v1-result.md`

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
- 是否让 authorization 变成 active。
- 是否完成真实窗口 / 截图验收。
- 下一步是 C3：全局主管方案边界复核和授权生效，还是先修复 C2 发现的阻断。
