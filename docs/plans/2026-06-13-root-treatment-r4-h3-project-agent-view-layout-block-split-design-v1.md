# Root Treatment / R4-H3 ProjectsView 与 AgentView 布局区块拆分设计 v1

日期：2026-06-13

状态：`draft_pending_user_confirmation`

性质：批二执行前设计稿。本文只定义 `ProjectsView.tsx` / `AgentView.tsx` 后续拆分边界，不授权实现，不修改 UI / CSS / 水墨风格。

## 1. 当前结论

R4-H2 批一已经收口，停止线仍然生效：前端已从整包 `WorkbenchSnapshot` 切到六页 page query，但不得直接进入 `ProjectsView` / `AgentView` 拆分。批二必须先确认“布局区块 -> 组件边界”映射，再另行写任务包执行。

本文建议将批二命名为 R4-H3，目标是：

- 按最终蓝图的信息层级拆分 `ProjectsView.tsx` 和 `AgentView.tsx`。
- 保持当前用户可见行为和视觉零变更。
- 降低两个大文件的棘轮水位。
- 为后续 UI 真实优化留下清晰组件边界，但本批不做 UI 重做。

## 2. 必读依据

- `CURRENT.md`：当前权威入口，确认 R4-H2 批一完成并停在批二前。
- `handoffs/2026-06-13-root-treatment-r4-h2-batch-one-page-query-completion-checklist-v1.md`：批一完成清单和停止线。
- `handoffs/2026-06-13-r4-hard-targets-execution-plan-supervisor-v1.md`：R4 硬目标批一 / 批二规划。
- `docs/research/xuanji-ui-design-extraction-report.md`：只吸收信息结构和布局方法，不复制源码、风格、命名或资产。
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`：Root Treatment 治理期总边界。

## 3. 当前代码事实

当前棘轮文件：

- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`：5897 行。
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`：3118 行。
- `prototypes/productized-desktop-shell/src/styles.css`：8464 行，本批不改。

已有可复用目录：

- `src/views/projects/ProjectGallery.tsx`
- `src/views/projects/ProjectReferencePanels.tsx`
- `src/views/agent/TranscriptViews.tsx`

现有结构风险：

- `ProjectsView.tsx` 同时承载项目壳、项目概览、项目工作流画布、React Flow 渲染、右侧节点详情、执行状态、方案授权、边界复核、项目主管拆任务、工作项编排、候选治理、记忆包预览和大量 label helper。
- `AgentView.tsx` 同时承载会话选择、会话列表、对话正文、任务输入、K2 产品命令流程、统一执行链路、适配器、provider、session continuation、真实 resume 授权、runtime attention 和 SDK/diagnostic 边界。
- 当前 `AgentView` 已经向“对话界面”收敛，但开发者 / 边界面板仍与普通会话界面同文件耦合。

## 4. 参考布局原则

从 Xuanji 研究报告吸收的原则：

- 中央界面永远展示当前工作对象。
- 右侧面板展示当前选择的详情。
- 聊天是入口，不是所有结构化状态的容器。
- 状态条 / 运行状态应常驻可见，但不应压进普通对话内容。

本工作台自己的约束：

- 项目页中央主对象是项目与项目工作流。
- 智能体页中央主对象是项目下的会话对话。
- 开发 / 内部 / 边界详情进入开发者详情或设置，不占普通主路径。
- 本批只做组件边界和文件瘦身，不改视觉风格，不改布局，不改交互语义。

## 5. 目标布局区块

### 5.1 项目页区块

项目页后续按以下区块拆：

1. 项目入口与项目壳：空状态、项目方块、返回项目、项目标题、项目元信息、项目 tab。
2. 项目概览：项目概览卡、智能体入口摘要、当前工作流摘要、交接 / 证据 / 权威摘要。
3. 中央工作流画布：React Flow / 静态画布、节点、边、状态条、关注条。
4. 右侧节点详情：选中节点详情、用户摘要、项目主管信息、技术详情、允许动作。
5. 右侧运行与治理摘要：统一执行状态、运行前检查、方案草案、全局边界复核、项目主管拆任务、授权摘要。
6. 右侧工作项与记忆治理：工作项编排、派发 / review / 过程事实、候选治理、任务记忆包预览。
7. 资源 / 证据 / 占位面板：项目资料、运行器资源、项目设置占位。

### 5.2 智能体页区块

智能体页后续按以下区块拆：

1. 智能体页壳：项目选择、对话选择、状态说明、软件筛选。
2. 会话列表：搜索、读取状态过滤、项目 / 软件分组、会话卡片。
3. 对话正文：Transcript reader、聊天气泡、代码块、内部过程事件折叠。
4. 任务输入与发送确认：composer、K2 preview / prepare / confirm / Phase A / Phase B 状态展示。
5. 开发者执行详情：Codex 控制入口、统一执行链路、真实执行边界。
6. 开发者适配器与 provider 详情：adapter capability、provider availability、session operation boundary。
7. 开发者 continuation / readback / diagnostics 详情：continuation preview、controlled continuation、H2 authorization、runtime attention、I5 diagnostics。

## 6. 组件边界建议

### 6.1 项目页目标文件

建议保留：

- `src/views/ProjectsView.tsx`：只保留 `ProjectsView`、最小 props 编排、selected project / selected tool 状态、顶层路由。

建议新增 / 扩展：

- `src/views/projects/ProjectWorkspaceShell.tsx`：`ProjectDetail`、项目头部、项目 tab、项目工具路由。
- `src/views/projects/ProjectOverviewPanels.tsx`：`ProjectOverview`、`ProjectAgentMovedPanel`、`ProjectToolPlaceholder`。
- `src/views/projects/ProjectWorkflowCanvasView.tsx`：`WorkflowCanvas`、`ProjectWorkflowReactFlowCanvas`、`ProjectCanvasStaticStage`、`ProjectCanvasFlowNodeView`、画布关注 / 编辑 / surface boundary 面板。
- `src/views/projects/ProjectWorkflowSidePanel.tsx`：`ProjectCanvasSidePanel`、`ProjectCanvasNodeDetailView`、`ProjectCanvasDerivedSummary`、`WorkflowRunCheckPanel`。
- `src/views/projects/ProjectWorkflowGovernancePanels.tsx`：方案草案、全局边界复核、项目主管拆任务、授权摘要。
- `src/views/projects/ProjectWorkflowMemoryPanels.tsx`：`CandidateGovernanceStrip`、`ProjectBlackboardPanel`、任务记忆包预览相关展示。
- `src/views/projects/ProjectWorkflowExecutionPanels.tsx`：`ProjectUnifiedExecutionStateCard`、`ExecutionControlPanel`、工作项编排相关执行状态展示。
- `src/views/projects/projectWorkflowLabels.ts`：项目页专用 label / tone / helper。只迁移纯展示 helper，不迁移状态写入逻辑。

### 6.2 智能体页目标文件

建议保留：

- `src/views/AgentView.tsx`：只保留 `AgentView` 顶层 props、page read model fallback、会话选择状态、数据派发。

建议新增 / 扩展：

- `src/views/agent/AgentConversationShell.tsx`：`AgentSessionCenter` 的普通会话布局、项目 / 对话选择、会话列表、正文区域。
- `src/views/agent/AgentSessionList.tsx`：搜索、读取状态过滤、会话分组、会话卡片。
- `src/views/agent/AgentChatComposer.tsx`：任务输入、发送前确认材料、K2 状态按钮。
- `src/views/agent/AgentDeveloperPanels.tsx`：开发者 details 总容器，只在主动展开后显示。
- `src/views/agent/AgentExecutionPanels.tsx`：`CodexControlEntryPanel`、`UnifiedExecutionStatusPanel`。
- `src/views/agent/AgentAdapterBoundaryPanels.tsx`：adapter、provider、operation boundary。
- `src/views/agent/AgentContinuationBoundaryPanels.tsx`：session continuation、controlled continuation、H2 readiness、runtime attention、I5 diagnostics。
- `src/views/agent/agentLabels.ts`：智能体页专用 label / tone / grouping helper。

H3-4 / H3-5 拆 `AgentSessionList` 与对话正文时，组件接口必须给后续会话外壳重做预留空间：分页、分组、归档隔离、subagent 折叠、虚拟滚动和直读数据库常驻都应能在外层接入。H3 本批不实现这些能力，但不得把“全量数组输入 / 全量 DOM 渲染”焊死成长期接口；若当前实现仍传数组，必须保持为当前适配层细节，而不是未来契约。

已有 `src/views/agent/TranscriptViews.tsx` 继续承接对话正文，不迁回 `AgentView.tsx`。

## 7. 分包计划

每包必须独立任务包、独立实现、独立验证、独立复核、独立 commit。每包都必须降低至少一个棘轮文件水位；不降水位的准备工作并入能降水位的包。

### H3-1：项目壳与项目概览拆分

目标：

- 抽出 `ProjectWorkspaceShell.tsx` 和 `ProjectOverviewPanels.tsx`。
- `ProjectsView.tsx` 只保留项目选择和顶层路由。

预计棘轮下降：

- `ProjectsView.tsx`：5897 -> 5200 以下，至少下降 500 行。

禁止：

- 不改项目 tab 数量。
- 不改默认 tab。
- 不改项目页视觉和文案。

验证：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

### H3-2：项目中央工作流画布拆分

目标：

- 抽出 `ProjectWorkflowCanvasView.tsx`。
- React Flow / static fallback / node view / attention strip 进入画布组件文件。
- `ProjectsView.tsx` 不再直接承载画布渲染细节。

预计棘轮下降：

- `ProjectsView.tsx`：在 H3-1 基线上再下降 600 行以上。

禁止：

- 不改 React Flow 行为。
- 不改节点、边、状态条、关注条视觉。
- 不新增画布编辑能力。

验证同 H3-1。

### H3-3：项目右侧详情、治理和记忆面板拆分

目标：

- 抽出 `ProjectWorkflowSidePanel.tsx`、`ProjectWorkflowGovernancePanels.tsx`、`ProjectWorkflowMemoryPanels.tsx`、`ProjectWorkflowExecutionPanels.tsx`。
- 右侧面板仍按当前顺序渲染，不做信息层级重排。
- helper 可迁入 `projectWorkflowLabels.ts`，但只迁纯展示 helper。

预计棘轮下降：

- `ProjectsView.tsx`：在 H3-2 基线上再下降 1200 行以上。
- 新增单文件不得超过 2000 行。

禁止：

- 不修改 action proposal。
- 不改权限弹层。
- 不改 workflow state / sidecar / DB / Rust。
- 不把开发者细节提升到主视觉。

验证同 H3-1。

### H3-4：智能体普通对话区拆分

目标：

- 抽出 `AgentConversationShell.tsx`、`AgentSessionList.tsx`、`AgentChatComposer.tsx`。
- `AgentView.tsx` 只负责装配数据和顶层状态，不再承载列表和 composer DOM。
- 保持“选择项目、选择对话、显示对话框、输入任务”的当前普通路径。

预计棘轮下降：

- `AgentView.tsx`：3118 -> 2300 以下，至少下降 700 行。

禁止：

- 不改变智能体页滚动策略。
- 不改 K2 真实执行权限逻辑。
- 不新增裸控制台。
- 不改 `TranscriptViews.tsx` 对话正文语义。

验证同 H3-1。

### H3-5：智能体开发者边界面板拆分

目标：

- 抽出 `AgentDeveloperPanels.tsx`、`AgentExecutionPanels.tsx`、`AgentAdapterBoundaryPanels.tsx`、`AgentContinuationBoundaryPanels.tsx`、`agentLabels.ts`。
- `AgentView.tsx` 不再直接承载 adapter / provider / continuation / runtime / diagnostic 面板。
- 开发者详情仍默认不抢普通对话主路径。

预计棘轮下降：

- `AgentView.tsx`：在 H3-4 基线上再下降 1000 行以上，目标收敛到 1300 行以下。
- 新增单文件不得超过 2000 行。

禁止：

- 不改真实 Codex 执行 guard。
- 不读写 `/Users/yoyi/.codex`。
- 不修改 provider / credential / model 验证边界。
- 不把 planned adapters 显示成可执行。

验证同 H3-1。

## 8. Shape Gate 与水位策略

每包实现后必须更新 shape gate 的对应历史最低水位：

- `ProjectsView.tsx` 的 waterline 随 H3-1 / H3-2 / H3-3 下降。
- `AgentView.tsx` 的 waterline 随 H3-4 / H3-5 下降。

新增文件约束：

- 新增 `.tsx` 文件原则上小于 2000 行。
- 如果单个新增文件接近 1500 行，任务包必须说明为什么不继续拆。
- 不得用一个新巨型文件替代旧巨型文件。

## 9. 验收口径

H3 批二全部完成后可声明：

- `ProjectsView.tsx` 和 `AgentView.tsx` 已按布局区块拆分。
- 普通 UI 行为和视觉保持不变。
- 大文件棘轮水位下降。
- 后续 UI 重做具备更清晰组件边界。

不得声明：

- UI 重做完成。
- 项目页 / 智能体页视觉已经达到最终蓝图。
- 真实 Tauri / 截图验收完成。
- 真实 Codex 执行产品化完成。
- backlog 解冻。
- R3 Level B 执行完成。

## 10. 风险与缓解

风险一：抽组件时顺手改 UI。

缓解：H3 只允许文件移动和 props 传递调整；CSS、布局类名、文案、按钮行为默认不改。

风险二：新文件变成新的巨型文件。

缓解：每包约束新增文件小于 2000 行，并在 shape gate 中同步观察关键文件。

风险三：helper 迁移破坏测试引用。

缓解：纯展示 helper 优先迁移到局部 labels 文件；被测试直接引用的 helper 保留 export 或通过 barrel 兼容。

风险四：循环依赖。

缓解：组件文件只向下依赖 `lib/*`、`components/*` 和同目录 helper；`ProjectsView.tsx` / `AgentView.tsx` 不被子组件反向 import。

风险五：与后续 UI 设计混线。

缓解：H3 只做结构拆分；真正的信息层级和视觉优化另立任务，基于拆分后的组件进行。

## 11. 停止线

本文写成后，主管线必须停下等待用户确认。用户确认前不得创建 H3-1 实现任务包，不得拆 `ProjectsView.tsx` / `AgentView.tsx`，不得改 UI / CSS / 视觉。

用户确认后，主管线再按 H3-1 到 H3-5 顺序写任务包并组织实现线 / 复核线执行。
