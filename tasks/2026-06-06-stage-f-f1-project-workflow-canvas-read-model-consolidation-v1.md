# Task Package：Stage F / F1 Project Workflow Canvas Read Model Consolidation v1

状态：已完成。  
用途：在阶段 E 完成、E5 Level B mario test 健康探针已回收后，开始阶段 F 项目工作流画布产品化第一刀：收敛项目工作流画布读模型，让画布节点、边、状态、badge、attention、来源和详情摘要都来自 workflow state 及其稳定派生读模型，而不是由 React Flow 或临时 UI 状态补编事实。  
执行方式：允许最小产品代码改动、测试、evidence 和 handoff；不得执行真实 Codex、不得读写 `/Users/yoyi/.codex`、不得写 workflow state 顶层结构、不得启动真实 worker。

完成记录：

- Evidence：`evidence/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1.md`
- Handoff：`handoffs/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1-result.md`
- 结论：`accepted_as_project_workflow_canvas_read_model_consolidation`
- 说明：已收敛项目画布读模型、状态原因、attention、任务包 / 记忆包 / 权限 / readback / audit / evidence / handoff 摘要；未完成真实窗口 / 截图验收，仍交给 G3。

## 0. 先说薄弱点

- 当前项目页已经有 `projectCanvas.ts`、`ProjectsView.tsx` 和 React Flow 渲染基础；F1 不是从零做画布，也不是重写界面。
- 现有画布读模型仍有前端派生和 UI 细节混在一起的风险：状态、badge、attention、权限、readback、task package、memory packet、audit 摘要需要更稳定、可测试、可解释。
- F1 名字里有 “canvas”，容易被误解成复杂 React Flow 编辑器；本任务只做读模型收敛和摘要展示，不做编辑、拖拽保存、连线、自动布局产品化或通用自动化平台。
- 项目工作流画布是项目主管主工作界面的中间版本入口，但不能把任务包全文、audit 全文、raw workflow state、raw transcript、路径大表或内部 schema 铺给普通用户。
- E5 Level B 已证明指定 session 的最小真实 resume 健康探针可用，但 F1 不能继承真实执行授权；后续任何新的 `codex exec resume`、真实 prompt、readback 或 `/Users/yoyi/.codex` 读写都必须另行授权。
- GEPA / Paseo / Odysseus 研究仍是蓝图参考和后置候选，不进入 F1。

## 1. 已知事实 / 未知 / 假设

已知事实：

- 阶段 C1-C6 已完成，接受为自动化工作流受控闭环完成，但不等于真实 worker 产品化完成。
- 记忆层 M1-M13 已完成，M13 结论为 `accepted_with_deferred_items`。
- 阶段 E / E1-E7 已完成，E7 结论为 `accepted_with_deferred_items`。
- E5 Level A 已完成受控 continuation store / stub / guard。
- E5 Level B mario test 健康探针已完成：指定 `/Users/yoyi/Documents/mario test` “总指导” session 真实 `codex exec resume` exit code `0`，last message 返回固定标记；该结论只接受为单 session 最小健康探针。
- `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md` 已把 F1 定义为 Project Workflow Canvas Read Model Consolidation。
- `docs/plans/2026-06-01-project-workflow-canvas-node-schema-v1.md` 已定义项目画布 schema 草案。
- 当前代码已有 `prototypes/productized-desktop-shell/src/lib/projectCanvas.ts`、`ProjectsView.tsx`、React Flow 项目画布和静态 fallback。

未知：

- 现有 `deriveProjectWorkflowCanvasReadModel` 是否已经完整覆盖 authorization、task package、memory packet、permission、readback、audit 和 runtime attention 的可见摘要。
- 现有画布状态文案是否已经覆盖 empty、blocked、needs_review、prepared、running、ready_for_review、accepted、failed、timed_out、readback_unavailable 等状态。
- F1 是否需要把部分前端派生读模型上移到 Rust `WorkbenchSnapshot`；默认优先保持最小改动，如果现有前端纯派生足够，可不新增后端 command / sidecar。
- 当前真实 Tauri / 浏览器截图工具是否可用；如果不可用，必须在 evidence / handoff 写清。

本任务采用的假设：

- F1 默认不新增持久 sidecar，不迁移数据库，不改 workflow state JSON 顶层结构或状态枚举。
- F1 可以扩展 TypeScript 读模型、前端纯派生 helper、离线测试和项目页局部 UI；只有在必要时才扩展 Rust snapshot 类型。
- F1 可以复用 E6 runtime attention / readback boundary 的摘要语义，但不新增 G1 runtime log store。
- F1 只在既有 `项目` 页工作流画布区域和既有节点详情侧栏内展示，不新增一级入口、右侧顶级入口或项目页 tab。

## 2. 任务目标

完成阶段 F 第一刀：

```text
workflow state / project workflow summary
+ plan authorization / proposal / prepared dispatch
+ task package / task memory packet preview
+ permission / readback / runtime attention
+ audit / evidence / handoff refs
-> ProjectWorkflowCanvasReadModel v1 收敛
-> nodes / edges / global badges / attention / detail summaries
-> ProjectsView 项目画布局部展示
-> tests + evidence + handoff
```

F1 完成后可以说：

- 项目工作流画布主区域使用统一 `ProjectWorkflowCanvasReadModel` 展示项目工作流摘要。
- 画布节点、边、状态、badge、attention、warnings 和 detail panel 数据都有明确事实来源或派生来源。
- React Flow 只承载渲染、选择和查看，不承载事实、不写状态。
- 用户能区分空态、阻断、待复核、准备派发、执行中、待回收、已接受、失败 / 超时、readback 不可用等状态。
- 主画布只展示摘要和可理解提示；任务包、审计、证据、handoff、记忆包和 readback 只显示摘要 / 引用 / 状态，不铺全文。

F1 完成后仍不能说：

- 画布编辑器完成。
- 节点详情抽屉完整完成。
- 项目工作流自动派发产品化完成。
- 真实 worker / Codex 已执行。
- 新的真实 `codex exec resume` 已执行。
- 通用 send / resume 产品化完成。
- runtime log / diagnostics 完成。
- 阶段 F 完成。
- 阶段 G 真实 Tauri 验收完成。
- 中间版本最终验收完成。

## 3. 必须先读

当前入口：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`

UI / 画布边界：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `docs/plans/2026-06-01-project-workflow-canvas-node-schema-v1.md`

阶段 E / F 前置证据：

- `tasks/2026-06-06-stage-e-e7-session-adapter-model-boundary-acceptance-v1.md`
- `evidence/2026-06-06-stage-e-e7-session-adapter-model-boundary-acceptance-v1.md`
- `handoffs/2026-06-06-stage-e-e7-session-adapter-model-boundary-acceptance-v1-result.md`
- `tasks/2026-06-06-stage-e-e5-level-b-mario-test-controlled-real-resume-health-probe-v1.md`
- `evidence/2026-06-06-stage-e-e5-level-b-mario-test-controlled-real-resume-health-probe-v1.md`
- `handoffs/2026-06-06-stage-e-e5-level-b-mario-test-controlled-real-resume-health-probe-v1-result.md`
- `tasks/2026-06-06-stage-e-e6-runtime-session-attention-and-readback-failure-boundary-v1.md`

主要代码入口：

- `prototypes/productized-desktop-shell/src/lib/projectCanvas.ts`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

如果改后端 snapshot / command，还必须读：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`

搜索固定文本必须用 `rg -F '...'` 或单引号，禁止用 shell 双引号包住未转义反引号。

## 4. 范围

允许：

- 收敛或扩展 `ProjectWorkflowCanvasReadModel`、`ProjectCanvasNode`、`ProjectCanvasEdge`、`ProjectCanvasBadge`、`ProjectCanvasNodeDetail` 等前端读模型。
- 新增 `ProjectCanvasAttention` / `ProjectCanvasStatusReason` / `ProjectCanvasSummaryRef` 等纯读类型，前提是它们只从已有 state / store / read model 派生。
- 把 authorization、proposal、prepared dispatch、task package、task memory packet preview、permission、readback、runtime attention、audit、evidence、handoff 等摘要合入画布读模型。
- 在项目画布主区域显示少量 global badges、warnings、attention 和状态说明。
- 在既有右侧节点详情卡里显示摘要、引用、状态和下一步查看建议。
- 补齐状态文案和映射：
  - `empty`
  - `blocked`
  - `needs_review`
  - `prepared`
  - `running`
  - `ready_for_review`
  - `accepted`
  - `failed`
  - `timed_out`
  - `waiting_for_permission`
  - `readback_unavailable`
  - `unknown`
- 增加离线测试覆盖读模型状态、React Flow fallback / static fallback、主画布摘要、节点详情摘要和禁止文案。
- 如果现有数据不足，可新增纯派生 helper；只有必要时才扩展 `WorkbenchSnapshot`，且不得新增持久化 store。
- 更新 evidence / handoff 和当前入口。

禁止：

- 不执行真实 `codex exec`。
- 不执行真实 `codex exec resume`。
- 不发送真实 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取完整 transcript / rollout。
- 不读取 auth、token、`.env`、secret、keychain、OAuth、provider credential 或密钥文件内容。
- 不调用 Claude Code / OpenClaw / OpenCode / OpenCode-like。
- 不调用外部模型 provider。
- 不写 workflow state 新顶层结构。
- 不迁移数据库。
- 不新增持久 sidecar。
- 不新增真实 worker dispatch。
- 不启动四角色完整工作流。
- 不把 E5 Level B 单 session 健康探针扩大为通用会话控制能力。
- 不做 React Flow 编辑器、拖拽保存、连线保存、节点新增 / 删除、布局保存或复杂编辑器。
- 不把独立实验 `CanvasView` / `CanvasDefinition` 当项目 workflow 事实源。
- 不新增一级入口、右侧顶级入口或项目页 tab。
- 不把任务包全文、audit 全文、transcript 全文、raw workflow state、raw sidecar、raw log、数据库路径大表或内部 schema 展示给普通用户。
- 不把 readback unavailable 显示成真实 0 条结果。
- 不把 observation、candidate、knowledge hit、runtime attention 或 LLM 摘要写成正式事实 / 正式记忆。
- 不把 GEPA / Paseo / Odysseus 研究项并入本任务实现。

## 5. 读模型收敛要求

F1 的读模型至少要回答这些问题：

- 当前项目是否有 workflow；没有时显示空态，不补编工作项。
- 当前选中 work item / task draft 是什么；没有时显示 idle / empty。
- 当前 workflow 的主状态是什么：blocked、prepared、running、ready_for_review、accepted、failed、timed_out、unknown 等。
- 哪些节点需要用户注意：权限待处理、授权未通过、readback unavailable、lint blocking、记忆包不可用、dispatch 失败、验证失败、等待回收。
- 每个节点的事实来源是什么：workflow、workflow_node、work_item、task_package、dispatch、permission、execution_attempt、audit、evidence、handoff、memory_packet、authorization。
- 哪些信息只应作为引用显示，不在主画布展开。

建议结构可以保持前端纯读：

```text
ProjectWorkflowCanvasReadModel {
  schema_version,
  project_id,
  project_root,
  workflow_id,
  title,
  status,
  source,
  viewport_hint,
  nodes[],
  edges[],
  detail_panels{},
  global_badges[],
  attention_items[],
  warnings[]
}
```

如果不新增 `attention_items[]`，必须能用 `global_badges[]` / `warnings[]` / node `badges[]` 等现有字段表达同等信息，并在 evidence 写清原因。

## UI 显示边界确认

本任务是否改前端：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [ ] 改前端类型 / Tauri wrapper，但不新增可见 UI。
- [x] 改读模型摘要或状态显示。
- [x] 改已有页面局部 UI。
- [ ] 新增入口、面板、tab、按钮或确认动作。

已读取：

- `docs/workbench-frontend-display-boundary-v1.md`
- `CURRENT.md`
- `tasks/README.md`
- `docs/plans/2026-06-01-project-workflow-canvas-node-schema-v1.md`

本任务允许显示：

- 项目工作流画布主区域的节点、边、状态、少量 badge 和 attention 摘要。
- 节点详情侧栏中的任务包摘要、任务记忆包摘要、权限状态、readback 状态、audit / evidence / handoff 引用。
- 空态、blocked、needs_review、prepared、running、ready_for_review、accepted、failed、timed_out、readback_unavailable 的用户可理解文案。

本任务禁止显示：

- 任务包全文。
- audit 全文。
- transcript / rollout 全文。
- raw workflow state / raw sidecar / raw log。
- `/Users/yoyi/.codex` 路径内容、token、secret、`.env`、provider credential。
- “worker 已执行”“Codex 已收到任务”“自动派发已开始”“自动重试已完成”“runtime log 已完成”“阶段 G 已验收”等误导文案。

显示位置：

- 一级入口：不新增；继续使用 `项目`。
- 右侧入口：不新增；不改秘书 / 通知 / 待办 / 运行中 / 管理入口。
- 项目页：只改既有项目工作流画布区域和既有节点详情侧栏。
- 画布：使用项目工作流画布，不使用独立实验画布作为事实源。
- 记忆入口：不改。
- 知识库入口：不改。
- 智能体入口：不改。
- 管理入口：不改。

中间版本范围：

- 本轮必须落地：项目工作流画布读模型状态、badge、attention、来源和摘要收敛。
- 本轮只做读模型 / 摘要：task package、task memory packet、permission、readback、audit、evidence、handoff。
- 本轮后置：节点详情完整抽屉、画布编辑、runtime log、diagnostics、真实 Tauri 全面验收。

后端和数据依赖：

- 需要后端正式读模型：优先复用 `WorkflowStateSnapshot.project_workflows[]`、authorization / proposal / memory packet / lint / runtime attention 等既有读模型；如不足，新增纯派生字段，不新增 store。
- 需要审计 / 日志 / 权限 / 状态机：只显示已有 audit / permission / control_core / workflow state 摘要；不伪造日志。
- 不能用假数据伪装：不能 hardcode 成功态，不能把 unavailable 写成 0，不能把计划中 adapter 写成可执行。

UI 文案边界：

- 禁止说：`worker 已执行`、`Codex 已收到任务`、`自动派发已开始`、`自动重试已完成`、`runtime log 已完成`、`阶段 G 已验收`、`通用 send/resume 已完成`。
- 允许说：`准备派发`、`等待权限`、`等待回收`、`readback 不可用`、`仅显示摘要`、`引用 evidence / handoff`、`React Flow 仅负责渲染`。

验收：

- 类型检查：必须跑 `npm run typecheck`。
- 离线交互测试：必须跑 `npm run test:offline-interaction`，新增或更新 F1 覆盖。
- 构建：必须跑 `npm run build`。
- Rust：如果改 Rust，必须跑相关 `cargo test --lib ...` 和 `cargo test --lib`，并跑对应 `rustfmt --check ...`。
- 真实窗口 / 截图验收：如可用，做项目页画布 smoke 和截图；如不可用，必须写入 evidence / handoff。
- 未验收项必须写入 evidence / handoff。

## 6. 建议实现顺序

1. 复核现有 `projectCanvas.ts` 和 `ProjectsView.tsx`，列出现有字段已经覆盖和缺口。
2. 收敛 `ProjectWorkflowCanvasReadModel`：补齐 status、badges、warnings、attention / source refs 和 detail summaries。
3. 保持 React Flow 只读：确认 `nodesDraggable=false`、`nodesConnectable=false`，不新增保存动作。
4. 收敛空态和失败态：无 workflow、无 task、blocked、failed、timed_out、readback_unavailable 都必须有文案和测试。
5. 项目页只做局部 UI：主画布摘要 + 侧栏摘要，不铺内部全文。
6. 补离线测试和文案扫描。
7. 更新 evidence / handoff 和入口文档。

## 7. 验收

必须通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- 如果改 Rust：相关 `cargo test --lib ...`
- 如果改 Rust：`cargo test --lib`
- 如果改 Rust：对应 `rustfmt --check ...`
- 禁止误导文案扫描，无新增误导完成态。

建议扫描：

```text
rg -n "worker 已执行|Codex 已收到任务|自动派发已开始|自动重试已完成|runtime log 已完成|阶段 G 已验收|通用 send/resume 已完成" prototypes/productized-desktop-shell/src
```

浏览器 / Tauri：

- 如果 Browser / Playwright / Tauri 工具可用，打开项目页，确认项目工作流画布可见、节点可选中、侧栏摘要可读、console 无新增 error。
- 如果不可用，至少做 Vite HTTP smoke，并在 evidence / handoff 写清不接受为真实窗口验收。

## 8. Evidence / Handoff 要求

完成后新增：

- `evidence/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1.md`
- `handoffs/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1-result.md`

Evidence 必须写清：

- F1 实际改了哪些读模型 / UI / 类型 / 测试。
- 是否改 Rust；如果改了，改动边界和测试结果。
- 画布事实源来自哪里。
- React Flow 是否仍只负责渲染。
- 空态、blocked、prepared、running、ready_for_review、accepted、failed、timed_out、readback_unavailable 等状态覆盖情况。
- 是否新增入口 / 按钮 / 确认动作；如果没有，要明确写没有。
- 禁止文案扫描结果。
- 是否做真实窗口 / 截图验收。
- 本轮不接受为什么。

Handoff 必须写清：

- F1 是否可接受为“项目工作流画布读模型收敛完成”。
- F2 是否可以开始。
- F1 仍不能接受为哪些能力完成。
- 遗留风险和建议。

## 9. 回收口径

完成后可接受为：

- F1 项目工作流画布读模型收敛完成。
- 项目画布主区域以统一读模型展示节点、边、状态、badge、attention 和摘要。
- React Flow 仍只是渲染映射，不是事实源。
- 任务包、记忆包、权限、readback、audit、evidence、handoff 以摘要 / 引用方式进入项目画布，不铺全文。

完成后不接受为：

- F2 节点详情完整抽屉完成。
- 画布编辑能力完成。
- 真实 worker / Codex 执行完成。
- 真实 send / resume 产品化完成。
- 自动派发产品化完成。
- 自动重试完成。
- runtime log / diagnostics 完成。
- planned adapters 真实接入完成。
- provider credential / model verification 完成。
- 阶段 F 完成。
- 阶段 G 真实 Tauri 验收完成。
- 中间版本最终验收完成。

## 10. Stop 条件

遇到以下情况必须停下：

- 需要执行真实 `codex exec` / `codex exec resume`。
- 需要发送真实 prompt。
- 需要读写 `/Users/yoyi/.codex`。
- 需要读取完整 transcript / rollout。
- 需要读取 secret、token、`.env`、keychain、OAuth、provider credential。
- 需要改 workflow state 顶层结构或状态枚举。
- 需要新增持久 sidecar 或数据库迁移。
- 需要新增真实派发、自动重试或 runtime log store。
- 需要把独立实验画布变成项目 workflow 事实源。
- 需要新增一级入口、右侧顶级入口或项目页 tab。
- 需要把任务包 / audit / transcript / raw state 全文铺到主画布。
