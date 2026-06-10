# Handoff: Project Workflow Canvas Task C Receive And Next v1

日期：2026-06-01

## 结果

已接收 Task C 项目工作流画布权威收敛保守切片。

当前状态：

- Task A 架构只读审计：已完成。
- Task B 保守拆模块切片：已完成。
- Task C 项目工作流画布权威收敛：已完成一个保守切片。
- 下一步建议：Task D 项目黑板最小实现。

## Task C 接收判断

接受为：

- 项目页工作流入口和文案已标明项目 workflow 主入口。
- 独立 `CanvasView` 已降权为实验/模板画布。
- 右侧运行入口已收敛为项目运行。

不接受为：

- 独立 `CanvasView` 已冻结。
- 项目页内部任务包、账本、状态机面板已完成收纳。
- 独立 canvas 文件层已经并入项目 workflow state。
- 画布双模型风险已经完全消除。

## 下一轮建议

派发 Task D：项目黑板最小实现。

任务口径：

- 建立 `ProjectBlackboard` / `BlackboardEntry` / `BlackboardEntryKind` / `BlackboardSourceRef` / `BlackboardPromotionDecision` 的最小模型或读模型。
- 先承载子智能体汇报、风险、权限请求、工具摘要、记忆候选、知识引用。
- 黑板内容默认只是中间态或候选。
- 只有控制核心确认后，黑板内容才能升级为正式事实、正式记忆、审计事件或状态变化。

禁止：

- 不执行真实 Codex。
- 不改 workflow state JSON 结构，除非另开迁移计划。
- 不让黑板直接推进工作流状态。
- 不让黑板直接写正式记忆。
- 不把知识库引用直接当记忆。

## 已改文件

- `AUTHORITY.md`
- `README.md`
- `tasks/README.md`
- `docs/plans/2026-06-01-workbench-architecture-implementation-plan-v1.md`

## 验证

- 本轮只改文档，未运行代码测试。
- 已抽查 Task C evidence、handoff 和关键 UI 文案入口。
