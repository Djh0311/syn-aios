# Final Skeleton 06 Project Canvas Node Schema v1 Evidence

日期：2026-06-02

## 本轮结论

先说薄弱点：

- 当前只是 schema / 计划，没有实现项目 React Flow 画布。
- 当前项目页仍有面板平铺风险，schema 只能定义收敛方向，不能自动修 UI。
- 独立 `CanvasView` 的模型仍未与项目 workflow state 合一；本轮也不合一。

结论：

- 已完成项目工作流画布节点 schema v1。
- 该 schema 是读模型 schema，不是数据库 schema，不需要迁移。
- 可以继续 `final-skeleton-07-canvas-component-state-examples-v1`，但仍不能改 workflow state JSON、不能启动 MCP canvas run、不能执行真实 Codex。

## 任务边界

执行：

- `final-skeleton-06-project-canvas-node-schema-v1`

目标：

- 定义项目工作流画布的节点、边、状态、详情面板数据 schema。
- 定义从 workflow state / project blackboard 派生到画布读模型的规则。

禁止：

- 不写 UI 大改。
- 不改 workflow state JSON 结构。
- 不做通用自动化节点。
- 不把任务包暴露成主 UI。

## 读过的依据

| 文件 | 用途 |
|---|---|
| `tasks/2026-06-01-final-workbench-skeleton-execution-package-v1.md:760-825` | Skeleton-06 目标、禁止项、输出和继续条件。 |
| `evidence/2026-06-01-final-skeleton-05-canvas-reference-research-v1.md` | 上一轮能力分层、风险和 schema 建议。 |
| `handoffs/2026-06-01-final-skeleton-05-canvas-reference-research-v1-result.md` | 上一轮下一步边界。 |
| `prototypes/productized-desktop-shell/src/lib/types.ts` | 现有 `WorkflowStateSnapshot`、`ProjectWorkflowSummary`、`Workflow`、`WorkflowNode`、`TaskPackage`、`ProjectBlackboard` 等输入模型。 |
| `decisions/2026-06-01-project-workflow-canvas-authority-v1.md` | 项目 workflow state 是权威事实源，独立 canvas 是实验/模板/后置能力。 |

没有读取：

- `/Users/yoyi/.codex`
- auth、token、`.env`、密钥、完整 transcript 或 rollout JSONL 正文

## 新增文档

| 文件 | 内容 |
|---|---|
| `docs/plans/2026-06-01-project-workflow-canvas-node-schema-v1.md` | 项目画布读模型 schema、节点/边/状态/详情/动作/派生规则、React Flow 映射和验收场景。 |

## Schema 摘要

顶层读模型：

- `ProjectWorkflowCanvasReadModel`

核心对象：

- `ProjectCanvasNode`
- `ProjectCanvasEdge`
- `ProjectCanvasNodeDetail`
- `ProjectCanvasDetailSection`
- `ProjectCanvasAction`
- `ProjectCanvasSourceRef`
- `ProjectCanvasBadge`
- `ProjectCanvasNodeMetric`

节点类型：

- `project_goal`
- `director`
- `dev_line`
- `validation_line`
- `review_line`
- `permission_request`
- `blackboard_candidate`
- `evidence_ref`
- `audit_ref`

边类型：

- `responsibility_flow`
- `handoff_flow`
- `review_flow`
- `evidence_reference`
- `blocking_relation`

状态：

- `idle`
- `ready_to_dispatch`
- `running`
- `waiting_for_permission`
- `ready_for_review`
- `needs_changes`
- `accepted`
- `failed`
- `timed_out`
- `blocked`
- `unknown`

## 关键设计判断

| 判断 | 依据 |
|---|---|
| 不需要 workflow state JSON 迁移 | schema 只从现有 `WorkflowStateSnapshot`、`derived_workflow` 和 `project_blackboards` 派生。 |
| 任务包不做主节点 | 总包禁止“不把任务包暴露成主 UI”；任务包进入节点详情 section。 |
| harness 不做普通节点 | 当前规则要求 harness 是检查和完成闸门，不是普通节点执行器。 |
| 权限请求可做 sidecar 节点 | 权限请求会阻塞主线节点，需要可视化，但批准/拒绝仍走确认弹层和控制核心。 |
| 黑板候选可做 sidecar 节点 | 黑板承载中间态/候选，不能直接升级为正式事实或正式记忆。 |
| React Flow 只承载显示 | React Flow 节点/边映射本 schema，不直接读写项目事实。 |

## 后续实现边界

可继续：

- Skeleton-07 组件状态样例。

仍禁止：

- 不改 workflow state JSON。
- 不启动 MCP canvas run。
- 不执行真实 Codex。
- 不把独立 `CanvasView` 合并为项目事实源。
- 不接通用自动化节点。
- 不写真实业务项目目录。

## 禁止事项执行情况

| 禁止项 | 结果 |
|---|---|
| 不写 UI 大改 | 已遵守。 |
| 不改 workflow state JSON 结构 | 已遵守。 |
| 不做通用自动化节点 | 已遵守。 |
| 不把任务包暴露成主 UI | 已遵守；schema 明确任务包进入详情 section。 |
| 不启动 MCP canvas run | 已遵守。 |
| 不执行真实 Codex | 已遵守。 |
| 不读写 `/Users/yoyi/.codex` | 已遵守。 |
| 不读 auth/token/`.env`/完整 transcript | 已遵守。 |
| 不迁移数据库 | 已遵守。 |

## 验证

本切片只写 schema / 计划，没有改代码，未跑代码测试。

依据：

- Skeleton-06 验收要求是“只写 schema / 计划；不改状态文件结构；不写实现”。

文档一致性检查：

- `docs/plans/2026-06-01-project-workflow-canvas-node-schema-v1.md` 已新增。
- 本 evidence 已新增。
- handoff 已新增。

## 不接受为

不接受为：

- 项目 React Flow 画布已实现。
- 组件状态样例已完成。
- 工作流主界面已收敛完成。
- 独立 `CanvasView` 已并入项目 workflow state。
- MCP canvas run 已验证。
- 真实业务自动编排完成。
