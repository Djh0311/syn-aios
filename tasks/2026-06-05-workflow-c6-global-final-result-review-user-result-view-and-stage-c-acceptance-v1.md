# Task Package：Workflow C6 Global Final Result Review, User Result View And Stage C Acceptance v1

状态：已完成。  
用途：实现中间版本阶段 C 的第六步：全局主管最终结果复核、用户结果查看，以及阶段 C 自动化工作流产品化闭环验收入口。  
执行方式：一个中等批次完成；开发重点在复用 C1-C5 已落地的授权、方案、prepared dispatch、worker report、process fact observation、review / audit 和失败可见化能力，补齐“最终复核 -> 用户查看 / 拍板 -> 阶段 C 验收摘要”的受控链路，不执行真实 worker / Codex。

## 1. 先说薄弱点

- C1-C5 已经形成方案授权、用户确认、全局边界复核、项目主管拆任务、prepared dispatch、worker 结构化汇报、项目主管过程事实确认和失败 / readback / 权限最小可见化，但还没有全局主管最终结果复核。
- C5 的 `process_fact` observation 仍不是正式事实，也不是正式记忆；C6 不能把它越级解释成“成果已被系统认可”。
- 用户目前还没有一个中间版本范围内的“结果查看 / 必须拍板事项”入口，容易把项目工作流侧栏里的过程信息误当最终结果。
- 阶段 C 是否完成需要验收摘要和明确 gating，不能因为 C5 有 worker report 或 observation 就自动宣称自动化工作流产品化闭环完成。
- C6 仍会触及项目工作流 UI、确认弹层、读模型摘要和阶段验收文案，必须遵守前端显示边界规则，不新增一级入口、不把审计 / raw event / sidecar 路径铺进主界面。

## 2. 任务目标

新增“C1-C5 已确认工作流事实 -> 全局主管最终复核 -> 用户结果查看 / 决定 -> 阶段 C 验收摘要”的受控链路：

```text
C1 plan authorization
-> C2 confirmed project proposal
-> C3 global boundary approved / active authorization
-> C4 project director task plan + prepared dispatch
-> C5 worker report + project director process fact decisions
-> C6 global final result review
-> user result view / user decision
-> Stage C acceptance summary
```

C6 完成后可以说：

- 全局主管可以基于 C1-C5 的授权、方案、任务、worker report、process fact observation、review / audit、readback / permission / failure 摘要做最终结果复核。
- 全局主管最终复核可以记录为 `accepted` / `needs_changes` / `blocked` 或实际命名等价结果。
- 用户可以看到中间版本可读的结果摘要、已确认过程事实、开放问题、失败 / 权限 / readback 摘要和全局主管最终复核结论。
- 用户可以明确记录结果决定，例如 `accept_result` / `request_changes` / `reject_result` 或实际命名等价动作。
- 阶段 C 可以生成验收摘要，说明 C1-C6 哪些 gate 已满足、哪些仍缺真实 worker / Tauri 截图 / 自动重试 / 运维日志等后置项。
- 阶段 C 若 gates 全部满足，可以接受为“自动化工作流产品化闭环阶段 C 完成”。

C6 完成后仍不能说：

- 中间版本整体完成。
- 中间版本完整记忆系统完成。
- M7-M13 已完成。
- 真实 worker 已执行，除非另有明确授权和 evidence。
- 真实 Codex 已执行，除非另有明确授权和 evidence。
- `process_fact` observation 已成为正式事实或正式记忆。
- 用户已接受所有未来任务结果。
- 完整自动重试系统、运行日志体系、真实 Tauri 全面验收或运维诊断完成。

## 3. 前置条件

必须已完成：

- `tasks/2026-06-04-workflow-c1-plan-authorization-and-controlled-auto-dispatch-foundation-v1.md`
- `tasks/2026-06-04-workflow-c2-project-consultation-proposal-and-user-confirmation-entry-v1.md`
- `tasks/2026-06-04-workflow-c3-global-boundary-review-and-authorization-activation-v1.md`
- `tasks/2026-06-04-workflow-c4-project-director-task-decomposition-and-authorized-prepared-auto-dispatch-v1.md`
- `tasks/2026-06-04-workflow-c5-worker-structured-report-process-fact-confirmation-and-failure-visibility-v1.md`
- `evidence/2026-06-04-workflow-c5-worker-structured-report-process-fact-confirmation-and-failure-visibility-v1.md`
- `handoffs/2026-06-04-workflow-c5-worker-structured-report-process-fact-confirmation-and-failure-visibility-v1-result.md`

开始前必须复核：

- C2 proposal 已由用户确认。
- C3 global boundary review 已 approved，关联 authorization 为 active。
- C4 prepared dispatch 仍在 C1 guard 授权范围内。
- C5 worker report / process fact decision 已可被读模型读取。
- C5 confirmed process fact 只是 observation，不能被 C6 当正式记忆。
- 当前没有真实 worker / Codex 执行授权；C6 默认只做受控记录、读模型和 UI。

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
- `docs/memory-layer-design-v1.md`
- `docs/plans/memory-layer-implementation-slice-v1.md`

前置记录：

- `evidence/2026-06-04-workflow-c3-global-boundary-review-and-authorization-activation-v1.md`
- `handoffs/2026-06-04-workflow-c3-global-boundary-review-and-authorization-activation-v1-result.md`
- `evidence/2026-06-04-workflow-c4-project-director-task-decomposition-and-authorized-prepared-auto-dispatch-v1.md`
- `handoffs/2026-06-04-workflow-c4-project-director-task-decomposition-and-authorized-prepared-auto-dispatch-v1-result.md`
- `evidence/2026-06-04-workflow-c5-worker-structured-report-process-fact-confirmation-and-failure-visibility-v1.md`
- `handoffs/2026-06-04-workflow-c5-worker-structured-report-process-fact-confirmation-and-failure-visibility-v1-result.md`

当前实现重点：

- `prototypes/productized-desktop-shell/src-tauri/src/control_core.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/observation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/task_memory_injection.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/plan_authorization_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/project_consultation_proposal_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/codex_transcript.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/codex_db.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/candidateGovernance.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

## 5. 范围

允许：

- 新增或扩展类型：
  - `GlobalFinalResultReview`
  - `GlobalFinalResultReviewInput`
  - `UserResultDecision`
  - `UserResultDecisionInput`
  - `WorkflowResultSummaryReadModel`
  - `StageCAcceptanceGate`
  - `StageCAcceptanceSummary`
  - `WorkflowResultEvidenceRef`
- 新增后端命令或 wrapper，例如：
  - `record_global_final_result_review`
  - `record_user_result_decision`
  - `load_workflow_result_summary`
  - `load_stage_c_acceptance_summary`
  - 或按现有命名约定拆为更小命令。
- 复用已有 `ReviewResult`、`SubagentReport`、`WorkflowException`、`WorkflowNodeDispatchRecord`、`WorkflowExecutionAttemptRecord`、`WorkflowPermissionRequestRecord`、C1-C5 audit / artifact / observation 读模型。
- 允许读取 C1-C5 sidecar 和 workflow state，用于生成最终结果复核上下文和阶段 C 验收摘要。
- 允许把全局主管最终复核、用户结果决定和阶段 C 验收摘要保存到现有 `workflow-state.v0.json` 的既有数组项：
  - `reviews[]`
  - `artifacts[]`
  - `audit_events[]`
  - 如现有结构已有 result / exception / review 对应数组，也可按既有 schema 使用。
- 允许在项目工作流侧栏 / 节点详情 / 结果摘要卡显示最终复核、用户结果查看和阶段 C gate 摘要。
- 新增 Rust 单测和前端离线测试。
- 更新 evidence / handoff / `CURRENT.md` / `tasks/README.md` / `AUTHORITY.md` / `STAGE_PLAN.md` / 阶段计划。

本任务显式授权的数据变更：

- 允许读取 `plan-authorizations.v1.json`、`project-proposals.v1.json`、`observations.v1.json`、`memory-candidates.v1.json`、`formal-memories.v1.json`、`memory-lint.v1.json`。
- 允许读取已有 `workflow-state.v0.json` 的既有数组项，用于结果摘要和阶段 C gate 计算。
- 允许更新已有 `workflow-state.v0.json` 的既有 `reviews[]`、`artifacts[]`、`audit_events[]`，用于记录全局主管最终复核、用户结果决定和阶段 C 验收摘要。
- 不允许新增 `workflow-state.v0.json` 顶层数组。
- 不允许修改 workflow / work item / node / dispatch 既有状态枚举；如果现有状态无法表达最终复核、用户决定或阶段 C 验收，必须先停下并回报。
- 不允许写 `observations.v1.json`，除非只是读取 C5 已确认的 process fact observation；C6 不新增 observation。
- 不允许自动生成 MemoryCandidate。
- 不允许写正式记忆。
- 不允许迁移数据库。

禁止：

- 不执行真实 worker，除非用户另行明确授权。
- 不执行真实 Codex，除非用户另行明确授权。
- 不执行 `codex exec` / `codex exec resume`，除非用户另行明确授权。
- 不读写 `/Users/yoyi/.codex`，除非用户另行明确授权。
- 不创建新的 Codex session。
- 不把 C5 worker report 直接写正式事实。
- 不把 C5 `process_fact` observation 写成正式记忆。
- 不把 readback 失败显示成真实 0 条读回。
- 不让秘书确认 worker 汇报、过程事实、最终成果或用户接受。
- 不把全局主管最终复核写成用户已接受。
- 不把用户结果查看写成阶段 C 已完成；必须有 gate 摘要和明确决定。
- 不显示“中间版本已完成”“完整记忆系统已完成”“真实 worker 已执行”“系统已记住”。
- 不把 traceback、raw transcript、完整日志、完整审计、sidecar 路径或内部 schema 铺进普通 UI。

如果执行者认为必须做真实 Codex / worker 端到端验证，必须先停止并向用户申请明确授权，写清：

- 目标项目路径。
- 目标 agent / session。
- 是否会写 `/Users/yoyi/.codex`。
- 会读取哪些上下文。
- 会写入哪些文件、workflow state 或 sidecar。
- 超时、取消、失败和回滚处理。

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
- `docs/plans/2026-06-01-project-workflow-canvas-node-schema-v1.md`

本任务允许显示：

- “全局最终复核”摘要卡。
- “用户结果查看”摘要卡。
- “阶段 C 验收”gate 摘要卡。
- 最终复核状态：`待全局主管复核`、`最终复核通过`、`需要修改`、`已阻断`。
- 用户结果决定：`待用户查看`、`用户已接受`、`用户要求修改`、`用户拒绝结果`。
- 阶段 C gate：`通过`、`缺少证据`、`需修改`、`阻断`、`后置项`。
- 最多 5 条结果摘要 / 开放问题 / 阻断原因 / 后置项。
- 动作：`记录全局最终复核`、`记录用户结果决定`、`生成阶段 C 验收摘要`。

本任务禁止显示：

- 不新增一级入口。
- 不新增右侧顶级入口。
- 不新增项目页 tab。
- 不在画布主区域铺完整结果包、raw transcript、完整日志、完整审计、sidecar 路径或内部 schema。
- 不显示“中间版本已完成”“完整记忆系统已完成”“真实 worker 已执行”“worker 汇报已成为正式事实”“系统已记住”。
- 不把 C5 observation / candidate 显示为正式记忆。
- 不把秘书显示为最终成果裁判、用户代理或复核人。
- 不显示未实现的“一键真实执行 / 一键真实重试 worker”按钮。
- 不显示未实际存在的 C6 完成状态。

显示位置：

- 一级入口：不改。
- 右侧入口：不改；通知 / 待办 / 运行中可显示摘要，但不新增入口。
- 项目页：允许在项目工作流侧栏、节点详情或运行详情卡显示最终复核、用户结果和阶段 C gate 摘要。
- 画布：主区域仍只显示项目工作流画布；最终复核、用户结果、stage gate 信息只能在详情侧栏。
- 记忆入口：不改。
- 知识库入口：不改。
- 智能体入口：不改。
- 管理入口：不改；完整审计和日志仍进入管理。

中间版本范围：

- 本轮必须落地：全局主管最终复核记录、用户结果查看 / 决定记录、阶段 C 验收摘要读模型和项目工作流侧栏显示。
- 本轮只做读模型 / 摘要：C1-C6 gate status、final review status、user decision status、open blockers、deferred items。
- 本轮后置：真实 worker / Codex 执行、自动重试系统、完整运行日志体系、真实 Tauri 全面验收、中间版本整体验收、M7-M13 完整记忆系统。

后端和数据依赖：

- 最终复核、用户决定和阶段 C gate 必须来自后端命令 / 读模型。
- 前端不能 mock “全局主管已复核”“用户已接受”“阶段 C 已完成”。
- C5 process fact observation 只能作为复核证据，不能当正式事实或正式记忆。
- candidate / formal memory 必须继续走现有 M3 / M2 / M1 边界。

UI 文案边界：

- 禁止说：“中间版本已完成”“完整记忆系统已完成”“系统已记住”“worker 汇报已成为正式事实”“真实 worker 已执行”“用户已接受所有结果”。
- 允许说：“全局主管已完成最终复核”“用户已查看结果并作出决定”“阶段 C 验收 gate 已通过 / 仍有后置项”“C5 observation 仅作为过程事实证据，仍不是正式记忆”。

验收：

- 必须跑 `npm run typecheck`。
- 必须跑 `npm run test:offline-interaction`。
- 必须跑 `npm run build`。
- 因新增确认动作和项目页局部 UI，必须做真实窗口或浏览器截图验收。
- 如果当前对话没有可用截图工具，必须在 evidence / handoff 写明“真实窗口 / 截图验收未完成”。

## 6. 建议数据对象

```ts
type GlobalFinalReviewDecision = "accepted" | "needs_changes" | "blocked";
```

```ts
type UserResultDecisionKind = "accept_result" | "request_changes" | "reject_result";
```

```ts
type StageCAcceptanceGateStatus =
  | "passed"
  | "missing_evidence"
  | "needs_changes"
  | "blocked"
  | "deferred";
```

```ts
type GlobalFinalResultReviewInput = {
  project_id: string;
  workflow_id: string;
  authorization_id: string;
  proposal_id: string;
  actor_id: string;
  actor_role: "global_director";
  decision: GlobalFinalReviewDecision;
  summary: string;
  evidence_refs: string[];
  accepted_process_fact_ids: string[];
  open_issues: string[];
  deferred_items: string[];
  expected_workflow_revision?: number | null;
};
```

```ts
type UserResultDecisionInput = {
  project_id: string;
  workflow_id: string;
  actor_id: string;
  actor_role: "user";
  decision: UserResultDecisionKind;
  summary: string;
  requested_changes: string[];
  accepted_review_id?: string | null;
  expected_workflow_revision?: number | null;
};
```

```ts
type StageCAcceptanceSummary = {
  project_id: string;
  workflow_id: string;
  gates: Array<{
    gate_id: string;
    label: string;
    status: StageCAcceptanceGateStatus;
    reason: string;
    evidence_refs: string[];
  }>;
  final_review_status: GlobalFinalReviewDecision | "pending";
  user_decision_status: UserResultDecisionKind | "pending";
  accepted_as_stage_c_complete: boolean;
  deferred_items: string[];
};
```

## 7. 后端要求

- 全局最终复核前必须要求：
  - C2 confirmed proposal 存在。
  - C3 approved global boundary review 存在，authorization 为 active。
  - C4 project director task plan / prepared dispatch 记录存在。
  - C5 worker report 或等价 handoff 记录存在。
  - C5 process fact decisions 已处理；不能有未处理的 blocking / request_rework 被忽略。
  - readback / permission / failure 摘要可被分类；不能把失败读回当真实空结果。
- 全局最终复核 actor 必须是 `global_director` 或等价全局主管角色。
- 用户结果决定 actor 必须是 `user` 或等价用户角色；秘书、项目主管、worker、system 不能代替用户接受结果。
- `accepted` final review 不能自动写正式记忆，也不能自动生成 MemoryCandidate。
- `accepted` final review 不能自动表示用户已接受；必须有单独 user decision。
- `request_changes` / `blocked` 必须写明原因，并进入 stage C gate 摘要。
- 所有复核 / 用户决定 / stage gate 生成必须写 audit。
- 如果复用现有 `ReviewResult`，必须保留 reviewer role、review target、evidence refs 和 source refs。
- 如果需要新增 result artifact，必须写在现有 `artifacts[]`，不能新增顶层数组。

## 8. 前端 / 读模型要求

- 新增 TS 类型和 Tauri wrapper。
- 新增或扩展纯函数读模型，把 C1-C5 的 authorization / proposal / dispatch / report / observation / review / exception / permission 派生成 C6 结果摘要。
- 项目工作流侧栏显示：
  - final review status。
  - user decision status。
  - stage C gate count。
  - passed / blocked / needs_changes / deferred counts。
  - 最多 5 条 open issue / deferred item / missing evidence。
- `记录全局最终复核` 必须有确认弹层，明确“这只是全局主管最终复核；不代表用户已接受，不写正式记忆，不代表中间版本完成”。
- `记录用户结果决定` 必须有确认弹层，明确“只记录本次结果决定；不代表未来任务默认接受”。
- `生成阶段 C 验收摘要` 必须明确 gate 来源和后置项。
- 不显示完整 raw transcript、长日志、完整 audit、内部 schema；只显示摘要和必要明细。

## 9. 验收

必须新增或更新测试，至少覆盖：

- 缺 C2 confirmed proposal 时，全局最终复核被拒绝。
- 缺 C3 approved boundary / active authorization 时，全局最终复核被拒绝。
- 缺 C4 prepared dispatch 或任务包 artifact 时，全局最终复核被拒绝或进入 missing evidence。
- 缺 C5 worker report / process fact decision 时，全局最终复核被拒绝或进入 missing evidence。
- global_director 可以记录 final result review。
- project_director / secretary / worker / system 不能记录 final result review。
- user 可以记录 user result decision。
- secretary / global_director / project_director / worker / system 不能代替 user 接受结果。
- final result review accepted 不会自动写正式记忆，不会自动生成 MemoryCandidate。
- user result accepted 不会自动写正式记忆。
- stage C acceptance summary 正确区分 passed / missing_evidence / needs_changes / blocked / deferred。
- readback failed / rollout unavailable / parse failed 不会显示成真实 0 条结果。
- UI 显示“全局主管已完成最终复核”和“用户已查看结果并作出决定”。
- UI 不显示“中间版本已完成 / 完整记忆系统已完成 / 系统已记住 / worker 汇报已成为正式事实 / 真实 worker 已执行”。

建议命令：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib global_final_result_review
cargo test --lib user_result_decision
cargo test --lib stage_c_acceptance
cargo test --lib process_fact
cargo test --lib dispatch_readback_stats
cargo test --lib workflow_authorization
cargo test --lib plan_authorization
cargo test --lib
rustfmt --check src/control_core.rs src/commands.rs src/types.rs src/observation_store.rs src/codex_transcript.rs src/codex_db.rs src/lib.rs
```

如果模块命名不同，按实际文件调整 `cargo test` 和 `rustfmt --check`，但 evidence 必须写明。

自检搜索：

```text
rg -F 'codex exec' evidence handoffs tasks docs CURRENT.md STAGE_PLAN.md
rg -F '中间版本已完成' prototypes/productized-desktop-shell/src
rg -F '完整记忆系统已完成' prototypes/productized-desktop-shell/src
rg -F 'worker 汇报已成为正式事实' prototypes/productized-desktop-shell/src
rg -F '系统已记住' prototypes/productized-desktop-shell/src
rg -F '真实 worker 已执行' prototypes/productized-desktop-shell/src
```

搜索结果里如果出现相关文案，必须确认它们是否是禁止边界或历史记录，不能把 C6 写成中间版本整体完成、正式记忆完成或真实 worker 执行完成。

## 10. 回收要求

完成后新增：

- `evidence/2026-06-05-workflow-c6-global-final-result-review-user-result-view-and-stage-c-acceptance-v1.md`
- `handoffs/2026-06-05-workflow-c6-global-final-result-review-user-result-view-and-stage-c-acceptance-v1-result.md`

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
- 是否把 worker report 写成正式事实。
- 是否把 process fact observation 写成正式记忆。
- 是否生成 MemoryCandidate。
- 是否写正式记忆。
- 是否完成全局主管最终结果复核。
- 是否记录用户结果决定。
- 是否接受为阶段 C 完成。
- 是否接受为中间版本整体完成。
- 是否完成真实窗口 / 截图验收。
- 下一步是阶段 D / M7-M13，还是先修复 C6 发现的阻断。
