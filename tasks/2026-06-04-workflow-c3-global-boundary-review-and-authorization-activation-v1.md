# Task Package：Workflow C3 Global Boundary Review And Authorization Activation v1

状态：已完成。  
用途：实现中间版本阶段 C 的第三步：全局主管方案边界复核、复核结论落账，以及 C1 `PlanAuthorization` 的受控生效。  
执行方式：一个中等批次完成；开发重点在复用 C1/C2 的 proposal 和 authorization 回链、补全全局边界复核读模型 / UI / 审计 / guard 验证，不启动真实 worker。

完成记录：

- `evidence/2026-06-04-workflow-c3-global-boundary-review-and-authorization-activation-v1.md`
- `handoffs/2026-06-04-workflow-c3-global-boundary-review-and-authorization-activation-v1-result.md`

回收结论：

- 接受为 C3 全局主管方案边界复核、approved 授权 active、needs_changes / blocked 授权 paused、C2 proposal 与 C1 authorization 回链校验、guard 验证摘要和项目工作流侧栏确认 UI 完成。
- 不接受为项目主管已拆任务、项目主管已自动派发 worker、真实 worker 已执行、真实 Codex 已执行、自动化工作流产品化闭环完成或最终结果复核完成。
- approved 路径会让 C1 authorization 进入 `active`；这只表示授权有效，仍未派发 worker。
- 本轮未执行 `codex exec` / `codex exec resume`，未读写 `/Users/yoyi/.codex`，未新增 `workflow-state.v0.json` 顶层结构，未修改 workflow / work item / node 状态枚举。
- 真实截图验收未完成：本轮没有暴露可用 browser / screenshot 工具，项目未安装 Playwright；已做 Vite HTTP smoke。

## 1. 先说薄弱点

- C1 已有 `record_plan_authorization_global_boundary_review`，但它仍偏底层命令；C2 用户确认后授权停在 `pending_global_boundary_review`。
- 中间版本要求全局主管只复核方案边界和最终结果，不逐条确认 worker 日常汇报。
- 如果没有 C3，用户确认的方案不能受控进入 active 授权，C4 项目主管也不能在授权范围内准备自动派发。
- 如果 C3 做过头，把 active 授权直接解释为“worker 已启动”，就会越过 C4/C5。
- 本任务会新增全局主管复核动作、状态摘要和 UI 文案，必须遵守前端显示边界规则。

## 2. 任务目标

新增“已确认方案 -> 全局主管边界复核 -> 授权生效”的产品化链路：

```text
C2 confirmed ProjectConsultationProposal
-> linked PlanAuthorization pending_global_boundary_review
-> GlobalBoundaryReview checklist / decision
-> approved: PlanAuthorization active
-> blocked / needs_changes: PlanAuthorization paused
-> C1 guard 对匹配输入可返回 authorized
-> 项目工作流详情显示“授权有效，但未派发 worker”
```

C3 完成后可以说：

- 全局主管可以对用户已确认方案做边界复核。
- 复核结果能落到账审计，并回写 C1 `PlanAuthorization.global_boundary_review`。
- approved 复核会让授权进入 `active`。
- blocked / needs_changes 会让授权进入受控暂停或待修改状态，不允许自动推进。
- C1 guard 能证明 active 授权对匹配范围返回 `authorized`，对越界范围仍然阻断。

C3 完成后仍不能说：

- 项目主管已经拆任务。
- 项目主管已经自动派发 worker。
- 真实 worker 已执行。
- 真实 Codex 已执行。
- 自动化工作流产品化闭环完成。
- 全局主管会逐条确认 worker 日报。
- C3 已经完成最终结果复核。

## 3. 前置条件

必须已完成：

- `tasks/2026-06-04-workflow-c1-plan-authorization-and-controlled-auto-dispatch-foundation-v1.md`
- `tasks/2026-06-04-workflow-c2-project-consultation-proposal-and-user-confirmation-entry-v1.md`
- `evidence/2026-06-04-workflow-c2-project-consultation-proposal-and-user-confirmation-entry-v1.md`
- `handoffs/2026-06-04-workflow-c2-project-consultation-proposal-and-user-confirmation-entry-v1-result.md`

开始前必须复核：

- C2 confirmed proposal 必须有 `plan_authorization_id` 回链。
- C1 authorization 必须有 `user_confirmation`，否则不得进入 active。
- C1 `record_plan_authorization_global_boundary_review` 当前 approved 会变 active，blocked / needs_changes 会变 paused；如调整状态语义，必须写清迁移和测试。
- C1 guard 对 active 授权仍必须检查角色、agent、读写范围、工具、检查、任务包类型和停止条件。

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

当前实现重点：

- `prototypes/productized-desktop-shell/src-tauri/src/plan_authorization_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/project_consultation_proposal_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/control_core.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/planAuthorization.ts`
- `prototypes/productized-desktop-shell/src/lib/projectConsultationProposal.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

## 5. 范围

允许：

- 复用并扩展 C1 `record_plan_authorization_global_boundary_review`。
- 新增或扩展类型：
  - `GlobalBoundaryReviewChecklist`
  - `GlobalBoundaryReviewFinding`
  - `GlobalBoundaryReviewReadModel`
  - `RecordGlobalBoundaryReviewInput`
  - `RecordGlobalBoundaryReviewOutput`
- 如果现有 `PlanAuthorizationGlobalBoundaryReview` 字段不足，允许向其中追加 checklist / findings / source_proposal_id / reviewed_scope_fingerprint 等兼容字段。
- 在后端增加 C2 proposal 和 C1 authorization 的一致性检查：proposal 已确认、authorization 回链匹配、project/workflow 匹配、用户确认存在。
- 在项目工作流侧栏 / 节点详情显示“全局边界复核”最小卡片。
- 新增全局主管复核动作：`批准并生效`、`要求修改`、`阻断方案`。
- approved 后允许 authorization 进入 `active`。
- blocked / needs_changes 后允许 authorization 进入 paused / needs_changes 等受控不可自动推进状态；如果沿用 C1 paused 语义，UI 必须写清“待修改 / 已阻断”。
- 新增 guard 验证摘要：active 且范围匹配时显示 `authorized`，越界仍显示 blocked。
- 新增 Rust 单测和前端离线测试。
- 更新 evidence / handoff / `CURRENT.md` / `tasks/README.md` / `AUTHORITY.md` / `STAGE_PLAN.md` / 阶段计划。

本任务显式授权的数据变更：

- 允许更新 `plan-authorizations.v1.json` 中已存在 authorization 的 `global_boundary_review`、status、audit_refs 和相关兼容字段。
- 允许读取 `project-proposals.v1.json` 来校验 confirmed proposal。
- 不允许新增 `workflow-state.v0.json` 顶层数组。
- 不允许修改 workflow / work item / node 既有状态枚举。
- 不允许迁移数据库。

禁止：

- 不执行真实 worker。
- 不执行真实 Codex。
- 不执行 `codex exec` / `codex exec resume`。
- 不读写 `/Users/yoyi/.codex`。
- 不创建新的 Codex session。
- 不调用外部 LLM 做复核。
- 不让全局主管逐条确认 worker 日常汇报。
- 不把 active 授权显示成“worker 已启动”。
- 不在 C3 自动创建 task package、prepared dispatch 或 workflow machine run。
- 不绕过 C1 guard。
- 不把 proposal、authorization 或 review 写成正式记忆。
- 不把秘书当复核者或成果裁判。

如果执行者认为必须做真实 Codex / worker 端到端验证，必须先停止并向用户申请明确授权，写清：

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

- “全局边界复核”卡片。
- 复核状态：`待全局复核`、`复核通过，授权有效`、`要求修改`、`已阻断`。
- 复核摘要：架构边界、跨项目影响、权限范围、读写范围、工具 / 检查、停止条件、验收方式、风险数量。
- 动作：`批准并生效`、`要求修改`、`阻断方案`。
- active 后显示：`授权有效；仍未派发 worker`。
- guard 验证摘要：`授权范围检查通过` 或最多 3 条 blocked reason。

本任务禁止显示：

- 不新增一级入口。
- 不新增右侧顶级入口。
- 不新增项目页 tab。
- 不在画布主区域铺完整复核 checklist、raw JSON、审计日志或内部 schema。
- 不显示“worker 已启动”“Codex 已收到任务”“项目主管已自动派发”“自动化工作流已完成”。
- 不把全局主管复核显示成 worker 日报逐条审批。
- 不把秘书显示为复核者。
- 不显示未实现的“一键真实执行 worker”按钮。

显示位置：

- 一级入口：不改。
- 右侧入口：不改。
- 项目页：允许在项目工作流侧栏、节点详情或方案详情卡显示复核摘要和动作。
- 画布：主区域仍只显示项目工作流画布；复核信息只能在详情侧栏。
- 记忆入口：不改。
- 知识库入口：不改。
- 智能体入口：不改。
- 管理入口：不改。

中间版本范围：

- 本轮必须落地：全局边界复核动作、authorization active / paused 受控状态变化、审计、guard 验证摘要。
- 本轮只做读模型 / 摘要：复核状态、复核摘要、active authorization id、guard result、blocked reason。
- 本轮后置：项目主管拆任务、自动派发队列、真实 worker 执行、失败重试、最终结果复核。

后端和数据依赖：

- 复核必须来自后端命令和 `plan-authorizations.v1.json`。
- 必须校验 C2 proposal 和 C1 authorization 回链。
- active 状态必须有用户确认和全局主管 approved 复核记录。
- 前端不能 mock 授权已生效。

UI 文案边界：

- 禁止说：“worker 已执行”“自动派发已开始”“项目已自动完成”“全局主管已复核最终结果”。
- 允许说：“复核通过，授权有效”“授权有效；仍未派发 worker”“要求修改后不能自动推进”“阻断方案后不能自动推进”。

验收：

- 必须跑 `npm run typecheck`。
- 必须跑 `npm run test:offline-interaction`。
- 必须跑 `npm run build`。
- 因新增确认动作和项目页局部 UI，必须做真实窗口或浏览器截图验收。
- 如果当前对话没有可用截图工具，必须在 evidence / handoff 写明“真实窗口 / 截图验收未完成”。

## 6. 建议数据对象

```ts
type GlobalBoundaryReviewStatus = "approved" | "needs_changes" | "blocked";
```

```ts
type GlobalBoundaryReviewChecklist = {
  architecture_boundary_checked: boolean;
  cross_project_impact_checked: boolean;
  permission_scope_checked: boolean;
  read_write_scope_checked: boolean;
  tool_and_check_scope_checked: boolean;
  memory_boundary_checked: boolean;
  stop_conditions_checked: boolean;
  acceptance_criteria_checked: boolean;
};
```

```ts
type GlobalBoundaryReviewFinding = {
  finding_id: string;
  severity: "info" | "warning" | "blocking";
  summary: string;
  recommendation?: string | null;
};
```

```ts
type RecordGlobalBoundaryReviewInput = {
  project_id: string;
  workflow_id: string;
  proposal_id: string;
  authorization_id: string;
  actor_id: string;
  review_status: GlobalBoundaryReviewStatus;
  summary: string;
  checklist: GlobalBoundaryReviewChecklist;
  findings: GlobalBoundaryReviewFinding[];
  expected_authorization_revision: number;
};
```

## 7. 后端要求

- `approved` 复核必须要求：
  - proposal 存在且 status 为 `user_confirmed`。
  - proposal `plan_authorization_id` 匹配输入 authorization。
  - authorization 存在且 `source_proposal_id` 匹配 proposal。
  - authorization 有 `user_confirmation`。
  - project_id / workflow_id 全部一致。
  - checklist 必填项为 true。
  - findings 中不能存在 `severity = "blocking"`。
- `approved` 成功后 authorization 进入 `active`，并写 `plan_authorization_boundary_reviewed` audit。
- `needs_changes` / `blocked` 不得进入 active，必须让 C1 guard 继续不通过。
- C1 guard active 成功路径必须保留所有范围检查，不因为 C3 复核通过而放宽。
- 复核摘要和 findings 必须长度受控，避免把长日志或 raw dump 写入 sidecar。
- 复核命令必须幂等或明确拒绝重复 approved，防止重复 active audit。
- 如果 C1 现有命令无法表达 checklist / findings，应新增 C3 wrapper，内部调用 C1 命令并保存扩展字段。

## 8. 前端 / 读模型要求

- 新增 TS 类型和 Tauri wrapper。
- 新增或扩展纯函数读模型，把 proposal store + authorization store 派生成“全局边界复核摘要”。
- 复核卡片必须显示用户确认状态、全局复核状态、active authorization id、guard 验证状态。
- `批准并生效` 必须有确认弹层，明确“只让授权生效，不启动 worker”。
- `要求修改` 必须允许填写简短原因。
- `阻断方案` 必须允许填写阻断原因。
- 没有 confirmed proposal 或 authorization 回链时，必须显示阻断原因。
- 不显示 raw sidecar、完整 audit、完整 checklist 大表；只显示摘要和最多 3 条 finding。

## 9. 验收

必须新增或更新测试，至少覆盖：

- 没有用户确认时，全局 approved 被拒绝。
- proposal / authorization 回链不匹配时被拒绝。
- checklist 缺必填项时 approved 被拒绝。
- blocking finding 存在时 approved 被拒绝。
- approved 后 authorization 进入 `active`，且 C1 guard 对匹配输入返回 `authorized`。
- active 后越界输入仍被 guard 阻断。
- needs_changes / blocked 后 authorization 不进入 active，guard 不允许自动推进。
- UI 显示“授权有效；仍未派发 worker”。
- UI 不显示“worker 已执行 / 自动派发已开始”。

建议命令：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib global_boundary_review
cargo test --lib plan_authorization
cargo test --lib project_consultation_proposal
cargo test --lib workflow_authorization
cargo test --lib
rustfmt --check src/plan_authorization_store.rs src/project_consultation_proposal_store.rs src/control_core.rs src/commands.rs src/types.rs
```

如果模块命名不同，按实际文件调整 `cargo test` 和 `rustfmt --check`，但 evidence 必须写明。

自检搜索：

```text
rg -F 'codex exec' evidence handoffs tasks docs CURRENT.md STAGE_PLAN.md
rg -F 'worker 已执行' prototypes/productized-desktop-shell/src
rg -F '自动派发已开始' prototypes/productized-desktop-shell/src
```

搜索结果里如果出现相关文案，必须确认它们是否是禁止边界或历史记录，不能把 C3 写成已执行。

## 10. 回收要求

完成后新增：

- `evidence/2026-06-04-workflow-c3-global-boundary-review-and-authorization-activation-v1.md`
- `handoffs/2026-06-04-workflow-c3-global-boundary-review-and-authorization-activation-v1-result.md`

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
- 是否让 authorization 进入 active。
- 是否启动任何 worker。
- 是否完成真实窗口 / 截图验收。
- 下一步是 C4：项目主管拆任务和授权范围内 prepared auto dispatch，还是先修复 C3 发现的阻断。
