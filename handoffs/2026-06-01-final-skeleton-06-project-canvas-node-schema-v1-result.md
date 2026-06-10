# Final Skeleton 06 Project Canvas Node Schema v1 Result

日期：2026-06-02

## 本轮完成

完成 `final-skeleton-06-project-canvas-node-schema-v1`。

先说限制：

- 本轮只写 schema / 计划，没有写 UI 实现。
- 没有改 workflow state JSON。
- 没有启动 MCP canvas run。
- 没有执行真实 Codex。

已完成：

- 定义项目工作流画布顶层读模型。
- 定义节点类型、边类型、状态、详情面板、允许动作和派生规则。
- 明确 React Flow 只作为渲染层映射，不作为事实源。
- 明确不需要数据库或 JSON 迁移。

## 新增产物

| 文件 | 内容 |
|---|---|
| `docs/plans/2026-06-01-project-workflow-canvas-node-schema-v1.md` | 项目工作流画布节点 schema v1。 |
| `evidence/2026-06-01-final-skeleton-06-project-canvas-node-schema-v1.md` | 本轮 evidence。 |
| `handoffs/2026-06-01-final-skeleton-06-project-canvas-node-schema-v1-result.md` | 本 handoff。 |

## 更新文件

| 文件 | 内容 |
|---|---|
| `CURRENT.md` | 同步 Skeleton-06 已完成和下一步 Skeleton-07。 |
| `tasks/README.md` | 同步当前任务队列。 |

## 关键判断

可以继续 Skeleton-07。

依据：

- 本 schema 只从现有 `WorkflowStateSnapshot`、`ProjectWorkflowSummary.derived_workflow` 和 `ProjectBlackboard` 派生，不要求改 workflow state JSON。
- 权威事实源仍是项目 workflow state + 项目黑板 + 控制核心确认后的事件和审计。
- 独立 `CanvasView` / `CanvasDefinition` 没有被纳入项目事实源。

主要风险：

- 下一步如果直接写 React Flow 实现，会跳过组件状态样例，容易把复杂状态写散。
- 项目页现有面板仍需后续收进节点详情或右侧展开。
- 如果要保存用户拖拽布局，必须另开 schema / 迁移计划。

## 下一步

下一步是：

- `final-skeleton-07-canvas-component-state-examples-v1`

边界：

- 建立组件状态样例。
- 不改 workflow state JSON。
- 不启动 MCP canvas run。
- 不执行真实 Codex。
- 不实现完整 React Flow 项目画布。

## 手动复核清单

1. 打开 `docs/plans/2026-06-01-project-workflow-canvas-node-schema-v1.md`。
2. 确认它定义的是 `ProjectWorkflowCanvasReadModel` 读模型，不是数据库 schema。
3. 确认节点类型包含项目目标、总指导、开发线、验证线、回收线、权限请求、黑板候选、证据引用、审计引用。
4. 确认边类型包含责任流转、handoff、review、证据引用、阻塞关系。
5. 确认任务包只进入详情面板 section，没有被定义成主节点。
6. 确认 React Flow 映射只是渲染映射，不保存项目事实。
7. 确认文档写明不需要 workflow state JSON 迁移。

## 明确未做

- 未改产品代码。
- 未执行真实 `codex exec`。
- 未执行真实 `codex exec resume`。
- 未读取或写入 `/Users/yoyi/.codex`。
- 未读取 auth、token、`.env`、密钥、完整 transcript 或 rollout JSONL 正文。
- 未启动 MCP canvas run。
- 未写真实业务项目目录。
- 未改 workflow state JSON。
- 未迁移数据库。
