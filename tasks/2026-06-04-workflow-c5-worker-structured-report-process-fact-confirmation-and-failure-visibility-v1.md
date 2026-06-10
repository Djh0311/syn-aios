# Task Package：Workflow C5 Worker Structured Report, Process Fact Confirmation And Failure Visibility v1

状态：已完成。  
用途：实现中间版本阶段 C 的第五步：worker 结构化汇报、项目主管过程事实确认，以及失败 / readback / 权限 / 超时 / 取消的最小可见化。  
执行方式：一个中等批次完成；开发重点在复用 C4 prepared dispatch、现有 readback / permission / execution attempt / ObservationStore 能力，补齐“汇报 -> 主管确认 -> 过程事实 / observation / 候选”的受控链路，不做最终结果复核。

## 1. 先说薄弱点

- C4 已能准备 worker 任务和 prepared dispatch，但还没有把 worker 汇报产品化落账。
- 现有读模型里已经有 `SubagentReport`、`ReviewResult`、`WorkflowException`、permission request、execution attempt 和 readback stats，但它们还没有形成 C5 的明确入口、确认动作和失败可见化标准。
- 中间版本明确要求 worker 的话只是汇报，不能直接成为正式事实；必须由项目主管结合证据确认过程事实。
- readback 原生 parser 已迁移，但读取失败可见化仍是缺口，不能把“读取失败”显示成真实 0 条结果。
- C5 会触及项目工作流 UI、节点详情、通知 / 运行中 / 权限摘要，必须遵守前端显示边界规则，不把 raw transcript、长日志和审计流水铺进主界面。

## 2. 任务目标

新增“prepared dispatch / worker handoff -> 结构化汇报 -> 项目主管确认过程事实 -> observation / candidate 边界”的受控链路：

```text
C4 prepared dispatch
-> worker structured report / handoff record
-> readback / permission / failure visibility summary
-> project director process fact decision
-> confirmed process facts become recorded observations
-> optional candidate creation remains explicit and controlled
-> 项目工作流详情显示“过程事实已确认 / 待确认 / 需返工 / 已阻断”
```

C5 完成后可以说：

- worker 可以提交结构化汇报或 handoff。
- 工作台能展示 worker 汇报摘要、证据、open issue、权限请求、方向风险和验收状态。
- 项目主管可以确认 / 要求返工 / 阻断 worker 汇报中的过程事实。
- 被确认的过程事实可以写入 `ObservationStore`，并保留来源、证据、权限和审计引用。
- 失败、超时、取消、readback 读取失败、权限等待能在项目工作流侧栏 / 节点详情中以人话显示。
- readback 的“真实 0 条结果”和“读取失败 / 不可访问 / 解析失败”能被区分。

C5 完成后仍不能说：

- 全局主管已复核最终结果。
- 用户已接受最终结果。
- 自动化工作流产品化闭环完成。
- worker 汇报已直接成为正式事实。
- worker 汇报已直接成为正式记忆。
- 秘书确认了 worker 汇报或过程事实。
- C5 已完成完整失败重试系统或真实多 worker 并发调度。

## 3. 前置条件

必须已完成：

- `tasks/2026-06-04-workflow-c1-plan-authorization-and-controlled-auto-dispatch-foundation-v1.md`
- `tasks/2026-06-04-workflow-c2-project-consultation-proposal-and-user-confirmation-entry-v1.md`
- `tasks/2026-06-04-workflow-c3-global-boundary-review-and-authorization-activation-v1.md`
- `tasks/2026-06-04-workflow-c4-project-director-task-decomposition-and-authorized-prepared-auto-dispatch-v1.md`
- `evidence/2026-06-04-workflow-c4-project-director-task-decomposition-and-authorized-prepared-auto-dispatch-v1.md`
- `handoffs/2026-06-04-workflow-c4-project-director-task-decomposition-and-authorized-prepared-auto-dispatch-v1-result.md`

开始前必须复核：

- C4 prepared dispatch 记录必须存在，且仍关联 C3 active authorization。
- C1 guard 对关联任务范围不能被 C5 放宽。
- prepared dispatch 如果尚未真实执行，C5 只能支持手动 / 离线 handoff 测试入口，不能声称真实 worker 产出。
- `ObservationStore` 已存在且只接受明确来源的 workflow event。
- 过程事实确认必须由 `project_director` 或等价项目主管角色发起；秘书不能确认事实。

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

- `evidence/2026-06-04-workflow-c3-global-boundary-review-and-authorization-activation-v1.md`
- `handoffs/2026-06-04-workflow-c3-global-boundary-review-and-authorization-activation-v1-result.md`
- `evidence/2026-06-04-workflow-c4-project-director-task-decomposition-and-authorized-prepared-auto-dispatch-v1.md`
- `handoffs/2026-06-04-workflow-c4-project-director-task-decomposition-and-authorized-prepared-auto-dispatch-v1-result.md`

当前实现重点：

- `prototypes/productized-desktop-shell/src-tauri/src/control_core.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/observation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/task_memory_injection.rs`
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
  - `WorkerStructuredReport`
  - `WorkerReportEvidenceRef`
  - `ProcessFactCandidate`
  - `ProjectDirectorProcessFactDecision`
  - `WorkflowReadbackVisibility`
  - `WorkflowFailureVisibility`
  - `WorkflowPermissionVisibility`
  - `ProcessFactConfirmationReadModel`
- 新增后端命令或 wrapper，例如：
  - `record_worker_structured_report`
  - `record_project_director_process_fact_decision`
  - `load_workflow_failure_visibility`
  - `classify_dispatch_readback_visibility`
  - 或按现有命名约定拆为更小命令。
- 复用已有 `WorkflowNodeDispatchRecord`、`WorkflowExecutionAttemptRecord`、`WorkflowPermissionRequestRecord`、`SubagentReport`、`ReviewResult`、`WorkflowException` 读模型。
- 复用 M3 `ObservationStore`，把项目主管确认后的过程事实写为 `observation_type = "process_fact"` 或现有等价类型；如果现有枚举不支持，允许兼容追加，但必须测试。
- 允许从 confirmed process fact 显式生成 MemoryCandidate，但必须复用 M3/M2 边界，不能自动采纳为正式记忆。
- 允许把 worker structured report / handoff 保存为 workflow state 现有数组项或 artifact。
- 允许更新已有 `workflow-state.v0.json` 的既有数组项：
  - `node_dispatches[]`
  - `execution_attempts[]`
  - `permission_requests[]`
  - `artifacts[]`
  - `audit_events[]`
  - `reviews[]`
  - 如现有结构已有 reports / exceptions 对应数组，也可按既有 schema 使用。
- 允许在项目工作流侧栏 / 节点详情显示汇报摘要、过程事实确认状态、失败摘要、readback 摘要和权限等待摘要。
- 新增 Rust 单测和前端离线测试。
- 更新 evidence / handoff / `CURRENT.md` / `tasks/README.md` / `AUTHORITY.md` / `STAGE_PLAN.md` / 阶段计划。

本任务显式授权的数据变更：

- 允许读取 `plan-authorizations.v1.json`、`project-proposals.v1.json`、`observations.v1.json`、`memory-candidates.v1.json`、`formal-memories.v1.json`、`memory-lint.v1.json`。
- 允许更新 `observations.v1.json`，但只用于项目主管确认后的过程事实 observation。
- 允许更新已有 `workflow-state.v0.json` 的既有数组项，用于记录 worker report、process fact decision、readback classification、permission visibility、failure visibility 和 audit。
- 不允许新增 `workflow-state.v0.json` 顶层数组。
- 不允许修改 workflow / work item / node / dispatch 既有状态枚举；如果现有状态无法表达待确认 / 返工 / 阻断，必须先停下并回报。
- 不允许迁移数据库。

禁止：

- 不执行真实 worker，除非用户另行明确授权。
- 不执行真实 Codex，除非用户另行明确授权。
- 不执行 `codex exec` / `codex exec resume`，除非用户另行明确授权。
- 不读写 `/Users/yoyi/.codex`，除非用户另行明确授权。
- 不创建新的 Codex session。
- 不把 worker 汇报直接写正式事实。
- 不把 worker 汇报直接写正式记忆。
- 不把 readback 失败显示成真实 0 条读回。
- 不确认最终结果。
- 不让秘书确认 worker 汇报、过程事实或成果。
- 不把权限等待、超时、取消、读取失败隐藏在 warning 里不显示。
- 不把 traceback、raw transcript、完整日志铺进普通 UI。

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

本任务允许显示：

- “worker 汇报”摘要卡。
- “过程事实确认”卡片。
- “失败 / readback / 权限可见化”摘要卡。
- 汇报状态：`待主管确认`、`过程事实已确认`、`要求返工`、`已阻断`。
- readback 状态：`读取成功`、`真实 0 条结果`、`读取失败`、`rollout 不可访问`、`解析失败`。
- 权限状态：`等待权限`、`已批准`、`已拒绝`、`超出项目主管权限，需要用户确认`。
- 失败状态：`超时`、`取消`、`执行失败`、`方向风险`、`harness 未通过`。
- 最多 3 条 open issue / blocked reason / permission reason。
- 动作：`确认为过程事实`、`要求返工`、`阻断并上报`、`从确认事实生成候选`。

本任务禁止显示：

- 不新增一级入口。
- 不新增右侧顶级入口。
- 不新增项目页 tab。
- 不在画布主区域铺完整 worker 汇报、raw transcript、完整日志、完整审计、sidecar 路径或内部 schema。
- 不显示“最终结果已通过”“用户已接受结果”“自动化工作流已完成”。
- 不把 worker 汇报显示为正式事实。
- 不把 observation / candidate 显示为正式记忆。
- 不把秘书显示为汇报确认者、事实裁判或成果裁判。
- 不显示未实现的“一键真实重试 worker”按钮。

显示位置：

- 一级入口：不改。
- 右侧入口：不改；权限 / 运行中 / 通知可显示摘要，但不新增入口。
- 项目页：允许在项目工作流侧栏、节点详情或运行详情卡显示摘要和动作。
- 画布：主区域仍只显示项目工作流画布；汇报、失败、权限和 readback 信息只能在详情侧栏。
- 记忆入口：不改。
- 知识库入口：不改。
- 智能体入口：不改。
- 管理入口：不改；完整审计仍进入管理。

中间版本范围：

- 本轮必须落地：worker 结构化汇报记录、项目主管过程事实确认、readback 成功 / 0 条 / 失败分类、权限等待摘要、失败 / 超时 / 取消摘要。
- 本轮只做读模型 / 摘要：report count、confirmed fact count、pending fact count、readback status、permission status、failure status、blocked reason。
- 本轮后置：全局主管最终结果复核、用户最终接受、完整自动重试系统、真实多 worker 并发调度。

后端和数据依赖：

- 汇报、确认和失败分类必须来自后端命令 / 读模型。
- 前端不能 mock “过程事实已确认”。
- process fact observation 必须来自项目主管确认动作。
- candidate / formal memory 必须继续走现有 M3 / M2 / M1 边界。

UI 文案边界：

- 禁止说：“worker 汇报已成为正式事实”“系统已记住”“最终结果已通过”“自动化工作流已完成”。
- 允许说：“项目主管已确认过程事实”“已记录为观察，仍不是正式记忆”“readback 读取失败，需要处理”“权限等待中”。

验收：

- 必须跑 `npm run typecheck`。
- 必须跑 `npm run test:offline-interaction`。
- 必须跑 `npm run build`。
- 因新增确认动作和项目页局部 UI，必须做真实窗口或浏览器截图验收。
- 如果当前对话没有可用截图工具，必须在 evidence / handoff 写明“真实窗口 / 截图验收未完成”。

## 6. 建议数据对象

```ts
type WorkerReportDecision = "confirm_process_fact" | "request_rework" | "block_and_escalate";
```

```ts
type ReadbackVisibilityStatus =
  | "success"
  | "empty_result"
  | "read_failed"
  | "rollout_unavailable"
  | "parse_failed";
```

```ts
type WorkerStructuredReport = {
  report_id: string;
  project_id: string;
  workflow_id: string;
  workflow_node_id: string;
  work_item_id: string;
  dispatch_id?: string | null;
  actor_role: string;
  executed_what: string;
  changed_what: string;
  summary: string;
  evidence_refs: string[];
  open_issues: string[];
  permission_requests: string[];
  direction_risks: string[];
  follow_up_suggestions: string[];
  acceptance_status: "reported_completed" | "reported_not_completed" | "blocked" | "needs_rework";
  source_refs: string[];
};
```

```ts
type ProcessFactCandidate = {
  process_fact_id: string;
  summary: string;
  source_report_id: string;
  source_dispatch_id?: string | null;
  evidence_refs: string[];
  risk_level: "low" | "medium" | "high";
  sensitive_level: "public" | "internal" | "secret";
  proposed_observation_type: "process_fact" | "worker_report";
};
```

```ts
type ProjectDirectorProcessFactDecision = {
  project_id: string;
  workflow_id: string;
  report_id: string;
  actor_id: string;
  decision: WorkerReportDecision;
  accepted_facts: ProcessFactCandidate[];
  rejected_fact_ids: string[];
  summary: string;
  expected_workflow_revision?: number | null;
  expected_observation_store_revision?: number | null;
};
```

## 7. 后端要求

- worker report 记录前必须要求：
  - project / workflow / work item 存在。
  - 如果关联 dispatch，则 dispatch 必须来自 C4 prepared 或后续受控执行路径。
  - report 必须包含 executed_what、changed_what、summary、evidence_refs。
  - source_refs 必须存在，不能从普通聊天自动捕获。
  - report 格式长度受控，避免存入长日志或 raw transcript。
- process fact 确认前必须要求：
  - actor role 是 `project_director` 或等价项目主管角色。
  - report 存在且未被重复确认为同一 process fact。
  - accepted facts 有 evidence_refs 和 source refs。
  - high risk / secret / cross-project fact 必须阻断或转用户确认，不得项目主管单独确认。
  - 确认后写 ObservationStore，状态为 recorded；普通 report 仍不等于正式记忆。
- readback visibility 必须区分：
  - transcript 读到且命中 0 条。
  - transcript / rollout 不可访问。
  - JSONL 解析失败。
  - index / sqlite catalog 缺失。
  - runner 没有返回 stats。
- permission visibility 必须汇总 pending / approved / rejected / requires_user_confirmation。
- failure visibility 必须汇总 failed / timed_out / cancelled / long_permission_wait / harness_failed / direction_risk。
- 所有确认 / 返工 / 阻断必须写 audit。
- 如果复用 `record_offline_role_result_handoff`，必须明确它是测试 / 离线 handoff，不能显示成真实 worker 自动执行。

## 8. 前端 / 读模型要求

- 新增 TS 类型和 Tauri wrapper。
- 新增或扩展纯函数读模型，把 dispatch / report / review / exception / permission / observation 派生成 C5 摘要。
- 项目工作流侧栏显示：
  - report count。
  - pending confirmation count。
  - confirmed process fact count。
  - readback visibility status。
  - permission visibility status。
  - failure visibility status。
  - 最多 3 条 blocked reason / open issue。
- `确认为过程事实` 必须有确认弹层，明确“只记录过程事实 observation，不写正式记忆，不完成最终验收”。
- `要求返工` 必须允许填写原因。
- `阻断并上报` 必须允许填写阻断原因。
- `从确认事实生成候选` 必须复用现有 candidate 创建边界，并明确候选仍需确认 / 采纳。
- 不显示完整 raw transcript、长日志、完整 audit、内部 schema；只显示摘要和必要明细。

## 9. 验收

必须新增或更新测试，至少覆盖：

- worker report 缺 evidence_refs 时被拒绝。
- worker report 只能来自 workflow event / handoff / dispatch source，普通聊天来源被拒绝。
- worker report 记录后不自动生成正式事实或正式记忆。
- project_director 可以确认低风险本项目 process fact，写入 recorded observation。
- secretary / worker / system 不能确认 process fact。
- high risk / secret / cross-project process fact 被阻断或需要用户确认。
- 同一 process fact 重复确认被拒绝或幂等。
- confirmed process fact 生成 observation 后，candidate creation 仍需显式动作。
- readback empty result 和 readback failed 被区分。
- rollout unavailable / parse failed 有可见 reason。
- permission pending / rejected / requires_user_confirmation 可见。
- timed_out / cancelled / failed / direction_risk 可见。
- UI 显示“已记录为观察，仍不是正式记忆”。
- UI 不显示“最终结果已通过 / 自动化工作流已完成 / worker 汇报已成为正式事实”。

建议命令：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib worker_structured_report
cargo test --lib process_fact_confirmation
cargo test --lib workflow_failure_visibility
cargo test --lib workflow_readback_visibility
cargo test --lib observation
cargo test --lib workflow_authorization
cargo test --lib
rustfmt --check src/control_core.rs src/commands.rs src/types.rs src/observation_store.rs src/codex_transcript.rs src/codex_db.rs
```

如果模块命名不同，按实际文件调整 `cargo test` 和 `rustfmt --check`，但 evidence 必须写明。

自检搜索：

```text
rg -F 'codex exec' evidence handoffs tasks docs CURRENT.md STAGE_PLAN.md
rg -F 'worker 汇报已成为正式事实' prototypes/productized-desktop-shell/src
rg -F '自动化工作流已完成' prototypes/productized-desktop-shell/src
rg -F '最终结果已通过' prototypes/productized-desktop-shell/src
rg -F '系统已记住' prototypes/productized-desktop-shell/src
```

搜索结果里如果出现相关文案，必须确认它们是否是禁止边界或历史记录，不能把 C5 写成最终验收或正式记忆完成。

## 10. 回收要求

完成后新增：

- `evidence/2026-06-04-workflow-c5-worker-structured-report-process-fact-confirmation-and-failure-visibility-v1.md`
- `handoffs/2026-06-04-workflow-c5-worker-structured-report-process-fact-confirmation-and-failure-visibility-v1-result.md`

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
- 是否写 ObservationStore。
- 是否生成 MemoryCandidate。
- 是否写正式记忆。
- 是否完成 readback 失败可见化。
- 是否完成真实窗口 / 截图验收。
- 下一步是 C6：全局主管最终结果复核、用户结果查看和阶段 C 验收，还是先修复 C5 发现的阻断。
