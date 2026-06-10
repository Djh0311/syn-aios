# Final Skeleton 05 Canvas Reference Research v1 Result

日期：2026-06-02

## 本轮完成

完成 `final-skeleton-05-canvas-reference-research-v1`。

先说限制：

- 本轮只做研究和任务拆分，没有改代码。
- 没有启动 Tauri，没有截图。
- 没有运行代码测试，因为本切片没有代码改动。

已完成：

- 复核本地 UI 参考源决策。
- 对照项目页工作流画布和独立实验画布。
- 输出画布能力分层：必须有、后置、明确不做。
- 输出风险清单。
- 给出 Skeleton-06 节点 schema 建议。

## 新增产物

| 文件 | 内容 |
|---|---|
| `evidence/2026-06-01-final-skeleton-05-canvas-reference-research-v1.md` | Skeleton-05 研究证据和能力分层。 |
| `handoffs/2026-06-01-final-skeleton-05-canvas-reference-research-v1-result.md` | 本 handoff。 |

## 更新文件

| 文件 | 内容 |
|---|---|
| `CURRENT.md` | 同步 Skeleton-05 已完成和下一步 Skeleton-06。 |
| `tasks/README.md` | 同步当前任务队列。 |

## 关键判断

可以继续 Skeleton-06，但只能做 schema / 计划。

依据：

- 当前项目工作流画布事实源方向正确：项目页读取 `workflowState` / `derived_workflow`。
- 独立 `CanvasView` 已显示为“实验 / 模板画布”，不是项目主入口。
- 权威决策仍要求独立 canvas 文件层不能成为项目工作流事实源。

主要风险：

- 项目页仍平铺太多内部协议面板，后续必须收进节点详情或右侧展开。
- 项目主画布还不是 React Flow 实现，React Flow 目前只在独立实验画布中使用。
- 独立实验画布仍有启动运行能力，不能在没有迁移计划时并入项目事实。

## 下一步

下一步是：

- `final-skeleton-06-project-canvas-node-schema-v1`

边界：

- 只写 `docs/plans/2026-06-01-project-workflow-canvas-node-schema-v1.md`。
- 只写 evidence / handoff。
- 不写 UI 实现。
- 不改 workflow state JSON 结构。
- 不启动 MCP canvas run。
- 不执行真实 Codex。
- 不把任务包暴露成主 UI。

## 手动复核清单

如果要人工复核本轮结论，看这些点：

1. 打开 `ProjectsView.tsx`，确认 `WorkflowCanvas` 读取的是 `workflowState`，不是独立 `CanvasDefinition`。
2. 看 `WorkflowCanvas` 下是否仍平铺运行前检查、黑板、读模型、账本、状态机、派发和机器入口。
3. 打开 `CanvasView.tsx`，确认它是独立实验/模板画布，并且仍有保存、启动实验运行、停止实验运行。
4. 打开 `App.tsx`，确认全局入口叫“实验画布”，右侧运行入口叫“项目运行”。
5. 对照 `decisions/2026-06-01-project-workflow-canvas-authority-v1.md`，确认独立 canvas 文件层没有被当成项目事实源。

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
