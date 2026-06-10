# Final Skeleton 05 Canvas Reference Research v1 Evidence

日期：2026-06-02

## 本轮结论

先说薄弱点：

- 当前项目页虽然已经标成“项目工作流主入口”，但仍把运行前检查、黑板、读模型、任务包、账本、审查、异常、状态机、接口边界、验收场景、派发和工作流机器入口大量平铺在主界面里。依据：`ProjectsView.tsx:1503-1601`、`ProjectsView.tsx:1768-1796`、`ProjectsView.tsx:1849-1903`。
- 独立 `CanvasView` 已降权为“实验 / 模板画布”，但仍有保存、启动实验运行、停止实验运行、轮询 run 状态能力。依据：`CanvasView.tsx:190-249`、`CanvasView.tsx:323-394`。
- 当前项目画布主界面没有真正使用 React Flow；React Flow 只在独立实验画布里使用。依据：`ProjectsView.tsx:1849-1903` 是手写节点展示；`CanvasView.tsx:1-17`、`CanvasView.tsx:382-394` 使用 `@xyflow/react`。

结论：

- 可以继续进入 `final-skeleton-06-project-canvas-node-schema-v1`。
- Skeleton-06 应只定义 schema / 计划，不改 UI，不改 workflow state JSON。
- 当前没有发现必须停止的权威冲突；但有明确风险：项目主画布和独立实验画布仍是两套模型，后续实现前必须先把节点 schema 和读模型边界写清楚。

## 本轮范围

执行总包中的：

- `final-skeleton-05-canvas-reference-research-v1`

任务要求：

- 复核画布参考源。
- 对照当前 `ProjectsView.tsx` 和独立 `CanvasView.tsx`。
- 输出能力分层：必须有、后置、明确不做。
- 输出风险清单。
- 输出后续节点 schema 任务建议。

禁止事项：

- 不接通用自动化平台。
- 不做复杂低代码编辑器。
- 不启动 MCP canvas run。
- 不改真实工作流事实。
- 不执行真实 Codex。

## 读过的依据

| 文件 | 用途 |
|---|---|
| `tasks/2026-06-01-final-workbench-skeleton-execution-package-v1.md:710-760` | Skeleton-05 的任务边界。 |
| `archive/decisions/2026-05-29-ui-reference-sources.md:120-288` | Langflow、n8n、React Flow、Storybook、shadcn/ui 等参考源的本地转译边界。 |
| `decisions/2026-05-31-editable-canvas-codex-as-director-v1.md:212-240` | 画布参考源后置研究目标和明确不做项。 |
| `decisions/2026-06-01-project-workflow-canvas-authority-v1.md:5-55` | 项目 workflow state 是项目工作流画布权威事实源，独立 `CanvasView` 是实验/模板/后置能力。 |
| `docs/plans/2026-06-01-workflow-task-package-design-v1-execution-plan.md:312-340` | 已确认画布当前只作为工作流可视化、节点详情和状态查看入口，不做通用节点执行器。 |
| `docs/plans/2026-06-01-workbench-architecture-implementation-plan-v1.md:514-526` | 项目页主界面应只保留项目列表 + 工作流画布，任务包/账本/审查/异常应进入节点详情或右侧展开。 |
| `decisions/2026-05-28-codex-workbench-ui-ia-direction.md:137-169` | 项目内工作流应以画布优先，右侧详情面板显示节点详情、任务包、handoff、evidence、回收意见。 |
| `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx` | 当前项目工作流画布和内部协议面板。 |
| `prototypes/productized-desktop-shell/src/views/CanvasView.tsx` | 当前独立实验/模板画布。 |
| `prototypes/productized-desktop-shell/src/App.tsx` | 当前导航和右侧运行入口。 |
| `prototypes/productized-desktop-shell/package.json` | 当前已依赖 `@xyflow/react`。 |

没有读取：

- `/Users/yoyi/.codex`
- auth、token、`.env`、密钥、完整 transcript 或 rollout JSONL 正文

## 参考源转译

| 参考源 | 可以借鉴 | 当前不吸收 |
|---|---|---|
| React Flow / xyflow | 节点、边、背景、小地图、控制器、节点工具栏、节点尺寸调整；用明确 schema 承载项目工作流节点。 | 不把示例通用流程图当产品流程；不把画布做成通用自动化平台。依据：`archive/decisions/2026-05-29-ui-reference-sources.md:233-252`。 |
| n8n | 节点配置面板、模板组织。 | 不接 400+ 集成生态，不做 webhook / cron 自动化，不自动执行外部系统动作。依据：`archive/decisions/2026-05-29-ui-reference-sources.md:160-178`。 |
| Langflow | 可视化 builder、节点逐步测试、observability。 | 不做通用 LLM 应用构建平台，不把向量库/RAG/多模型编排作为主线。依据：`archive/decisions/2026-05-29-ui-reference-sources.md:120-139`。 |
| Storybook | 画布节点、节点详情、权限队列、运行检查条的组件状态样例。 | 不把 Storybook 当产品运行时，不第一轮引入复杂视觉测试流水线。依据：`archive/decisions/2026-05-29-ui-reference-sources.md:254-270`。 |
| shadcn/ui | open code 组件组织方式；按钮、标签、抽屉、表单、滚动区、确认弹层。 | 不强制迁移 Tailwind，不为了用组件库重写设计系统。依据：`archive/decisions/2026-05-29-ui-reference-sources.md:272-288`。 |
| ComfyUI | 可参考节点图、参数面板、运行队列、历史记录和模板复用。 | 不做插件节点生态，不做任意 Python/shell/API 节点执行器。依据：`decisions/2026-05-31-editable-canvas-codex-as-director-v1.md:212-236`。 |

## 当前代码对照

### 项目工作流画布

| 代码位置 | 当前表现 | 判断 |
|---|---|---|
| `ProjectsView.tsx:1503-1538` | `WorkflowCanvas` 读取 `workflowState`，显示“项目工作流主入口”。 | 权威方向正确。 |
| `ProjectsView.tsx:1542-1549` | 主界面直接展示运行前检查、项目黑板、派生读模型。 | 信息有用，但主界面过重。 |
| `ProjectsView.tsx:1550-1587` | 状态条和三类角色节点手写展示。 | 可作为 schema 输入，但不是最终 React Flow 画布。 |
| `ProjectsView.tsx:1588-1601` | 当前工作项派发、绑定、回收等入口直接挂在主画布下。 | 风险高；后续应收进节点详情或右侧面板。 |
| `ProjectsView.tsx:1768-1796` | `DerivedWorkflowSummary` 平铺任务包、蓝图、账本、审查、异常、状态机、接口边界、验收场景。 | 仍有任务包管理器/内部协议仪表盘倾向。 |
| `ProjectsView.tsx:1849-1903` | `WorkflowBlueprintCanvas` 手写主节点和节点详情字段。 | 可作为 Skeleton-06 schema 草稿素材。 |

### 独立实验画布

| 代码位置 | 当前表现 | 判断 |
|---|---|---|
| `CanvasView.tsx:1-17` | 引入 `@xyflow/react` 的 `ReactFlow`、`Background`、`Controls`、`MiniMap` 等能力。 | 已有画布底座。 |
| `CanvasView.tsx:52-103` | `CanvasDefinition` 与 React Flow nodes/edges 互转。 | 属独立 canvas 模型，不是项目 workflow state。 |
| `CanvasView.tsx:190-204` | 保存独立画布。 | 作为实验/模板能力可保留，但不能变成项目事实源。 |
| `CanvasView.tsx:206-249` | 启动/停止实验运行。 | 高风险能力，不能在本轮扩大；后续如接项目必须通过控制核心。 |
| `CanvasView.tsx:323-394` | 展示实验/模板画布，包含新增节点、节点编辑、保存、运行、React Flow 区域。 | 适合参考交互，不适合直接并入项目主画布。 |

### 导航与右侧入口

| 代码位置 | 当前表现 | 判断 |
|---|---|---|
| `App.tsx:54-67` | 全局 `workflow` 入口显示为“实验画布”。 | 与画布权威决策一致。 |
| `App.tsx:70-75` | 右侧栏 `running` 显示为“项目运行”。 | 与 Task C 收敛一致。 |
| `App.tsx:608-625` | 项目运行入口点击回到项目页。 | 不再像第二个权威画布。 |

## 能力分层

### 必须有

| 能力 | 说明 | 依据 |
|---|---|---|
| 项目 workflow state 驱动的项目画布 | 项目画布的事实源必须是项目 workflow state，后续叠加项目黑板和控制核心确认事件。 | `decisions/2026-06-01-project-workflow-canvas-authority-v1.md:5-15`。 |
| 节点 / 边 schema | 先定义节点类型、边类型、状态、详情数据，再写 UI。 | 总包 Skeleton-06 要求。 |
| 角色节点 | 至少覆盖项目目标、总指导、开发线、验证线、回收线。 | 总包 Skeleton-06 要求；当前 `WorkflowBlueprintCanvas` 已有 director/subagent/review/report 草稿。 |
| 状态节点或节点状态 | 覆盖 `idle`、`ready_to_dispatch`、`running`、`waiting_for_permission`、`ready_for_review`、`needs_changes`、`accepted`、`failed`、`timed_out`。 | 总包 Skeleton-06 要求。 |
| 右侧节点详情 | 任务包、handoff、evidence、回收意见、权限、模型、工具、harness、账本引用应进节点详情或右侧展开。 | `docs/plans/2026-06-01-workbench-architecture-implementation-plan-v1.md:514-526`；`decisions/2026-05-28-codex-workbench-ui-ia-direction.md:152-156`。 |
| 项目黑板候选展示 | 黑板候选、风险、权限请求、工具摘要、记忆候选、知识引用可以作为节点详情或辅助节点。 | `decisions/2026-06-01-project-workflow-canvas-authority-v1.md:26-32`。 |
| 权限和阻塞可视化 | 权限请求、运行前检查、阻塞原因必须可见，但不应铺满主画布。 | 当前运行前检查和权限队列已有数据源；架构计划要求右侧展开。 |
| 审计和证据引用 | 画布显示引用，不显示全文。 | workflow task package 计划中确认账本和 evidence 不应全文铺进画布。 |

### 后置

| 能力 | 后置原因 |
|---|---|
| 独立 `CanvasView` 与项目 workflow state 合一 | 当前权威决策要求另开迁移计划；不能默认合并。 |
| 模板库 | 需要先完成节点 schema 和模板边界，避免变成通用自动化模板市场。 |
| 运行队列和运行历史 | 参考 n8n / ComfyUI / Langflow，但当前不能把画布变成执行队列。 |
| 节点逐步测试 | 可参考 Langflow，但需要定义哪些节点可测试、谁能触发、结果写入何处。 |
| Storybook / 组件状态样例 | 应在 Skeleton-07 做；本轮只研究。 |
| 自动化视觉回归 | 当前没有引入复杂视觉测试流水线，后置。 |
| 节点工具栏、节点尺寸调整、小地图深度定制 | React Flow 支持，但需要等项目节点 schema 固定。 |
| 复杂低代码编辑器 | 明确不作为当前阶段目标。 |

### 明确不做

| 不做项 | 依据 |
|---|---|
| 通用自动化平台 | n8n、Langflow 等只作参考；本地决策明确不吸收通用平台。 |
| ComfyUI 式插件节点生态 | 本地决策明确不做。 |
| 任意 Python / shell / API 节点执行器 | 本地决策明确不做。 |
| MCP canvas run 绕过项目规则 | 项目画布权威决策明确不接受。 |
| 独立 canvas 文件成为项目工作流事实源 | 项目画布权威决策明确不接受。 |
| 任务包暴露成主 UI | 架构计划要求任务包进入节点详情或右侧展开。 |
| harness 作为普通画布节点 | 现有计划要求 harness 是检查和完成闸门，不是普通节点。 |
| 知识库引用直接变记忆 | 继承项目黑板和记忆治理边界，不能把知识引用当正式记忆。 |

## 风险清单

| 风险 | 证据 | 判断 |
|---|---|---|
| 项目主界面仍像内部协议后台 | `ProjectsView.tsx:1768-1796` 平铺任务包、账本、审查、异常、状态机、接口边界、验收场景。 | 后续必须收进节点详情或右侧展开。 |
| 项目画布不是 React Flow 实现 | `ProjectsView.tsx:1849-1903` 是手写节点展示；`@xyflow/react` 只在 `CanvasView.tsx`。 | Skeleton-06 先 schema，Skeleton-08 再实现 React Flow 项目画布。 |
| 独立实验画布仍可运行 | `CanvasView.tsx:206-249` 可启动/停止实验运行。 | 不能扩大；如果并入项目必须另开迁移和控制核心接入计划。 |
| 任务包细节太靠前 | `TaskPackageReadModelPreview` 在主读模型摘要内直接展示大量字段。 | 后续应成为节点详情的一个 tab/section，不应主界面默认展开。 |
| 权限/派发/工作流机器按钮离主画布太近 | `WorkItemOrchestrationCard` 直接挂在 `WorkflowCanvas` 下。 | 容易让主画布像操作台而非状态视图；高风险动作需要右侧详情 + 明确确认。 |
| 参考源可能拉偏产品路线 | n8n/Langflow/ComfyUI 都有通用节点/执行生态。 | 本轮明确只转译，不吸收平台路线。 |

## Skeleton-06 建议

建议 Skeleton-06 只写：

- `docs/plans/2026-06-01-project-workflow-canvas-node-schema-v1.md`
- `evidence/2026-06-01-final-skeleton-06-project-canvas-node-schema-v1.md`
- `handoffs/2026-06-01-final-skeleton-06-project-canvas-node-schema-v1-result.md`

Skeleton-06 不应写：

- UI 实现。
- workflow state JSON 结构。
- 独立 `CanvasView` 与项目 workflow state 合并逻辑。
- MCP canvas run。
- 真实 Codex 执行。

Schema 建议包含：

| 类别 | 建议字段 |
|---|---|
| 节点基础字段 | `node_id`、`node_type`、`title`、`subtitle`、`status`、`role_id`、`source_refs`、`warning_count`、`blocked_count`。 |
| 节点类型 | `project_goal`、`director`、`dev_line`、`validation_line`、`review_line`、`permission_request`、`blackboard_candidate`、`evidence_ref`、`audit_ref`。 |
| 边基础字段 | `edge_id`、`edge_type`、`source_node_id`、`target_node_id`、`status`、`source_refs`。 |
| 边类型 | `responsibility_flow`、`evidence_reference`、`blocking_relation`、`handoff_flow`、`review_flow`。 |
| 详情面板 | `summary`、`task_package_refs`、`handoff_refs`、`evidence_refs`、`audit_refs`、`permission_requests`、`blackboard_entries`、`run_check`、`completion_gate`、`allowed_actions`。 |
| 派生规则 | 从 `workflowState.project_workflows[].derived_workflow`、`task_drafts`、`node_session_bindings`、`permission_requests`、`project_blackboards` 派生，不反写事实。 |

继续条件：

- 可以继续 Skeleton-06，因为本轮没有发现必须停止的权威冲突。
- Skeleton-06 如果发现 schema 必须修改 workflow state JSON，必须停下来，不进入实现。

## 禁止事项执行情况

| 禁止项 | 结果 |
|---|---|
| 不接通用自动化平台 | 已遵守。 |
| 不做复杂低代码编辑器 | 已遵守。 |
| 不启动 MCP canvas run | 已遵守。 |
| 不改真实工作流事实 | 已遵守。 |
| 不执行真实 Codex | 已遵守。 |
| 不读写 `/Users/yoyi/.codex` | 已遵守。 |
| 不读 auth/token/`.env`/完整 transcript | 已遵守。 |
| 不迁移数据库 | 已遵守。 |

## 验证

本切片只做研究和任务拆分，未改代码，未跑代码测试。

依据：

- Skeleton-05 验收要求是“只做研究和任务拆分；不改代码”。
