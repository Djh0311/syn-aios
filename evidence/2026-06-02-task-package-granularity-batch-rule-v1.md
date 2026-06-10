# Evidence: task package granularity batch rule v1

日期：2026-06-02

## 触发原因

用户指出：任务包不应拆分成太多小步骤，否则 agent 很多精力会耗在治理流程、重复 evidence、重复 handoff 上。

## 判断

这个问题成立，但不能直接改成大任务随便跑。

合理调整是“批次化小步”：

- 能合并的微步骤合并成一个执行批次。
- 批次完成后统一 evidence / handoff。
- 碰到硬边界、事实结构变化、真实 Codex 执行、数据库迁移、黑板/记忆正式写入时仍然拆开并停下来。

## 本轮改动

- `tasks/2026-06-01-final-workbench-skeleton-execution-package-v1.md`
  - 增加“执行粒度采用批次化小步”。
  - 增加“任务粒度规则”。
  - 明确哪些情况可以合并，哪些必须拆开。
  - 把当前下一步从单个 `final-skeleton-07` 调整为“画布基础批次”，合并执行 `final-skeleton-07` 到 `final-skeleton-09`。
- `tasks/README.md`
  - 当前任务改为“当前待派发批次”。
  - 当前下一批次改为画布基础批次。
- `CURRENT.md`
  - 当前任务和下一步建议改为画布基础批次。
  - 明确普通微步骤不再强制逐个单独 evidence / handoff。
- `README.md`
  - 下一步改为画布基础批次。
- `AUTHORITY.md`
  - 当前阶段计划入口改为画布基础批次。
- `STAGE_PLAN.md`
  - 阶段 4 当前进度改为画布基础批次。

## 当前批次口径

画布基础批次合并：

1. `final-skeleton-07-canvas-component-state-examples-v1`
2. `final-skeleton-08-react-flow-project-canvas-v1`
3. `final-skeleton-09-canvas-node-detail-panel-v1`

批次范围：

- 画布组件状态样例。
- React Flow 项目画布最小实现。
- 节点详情 / 右侧面板收敛。

批次禁止：

- 不写真实 workflow state。
- 不改工作流状态机。
- 不做完整低代码编辑器。
- 不引入复杂视觉测试流水线。
- 不启动 MCP canvas run。
- 不执行真实 Codex。

## 未做

- 未改代码。
- 未跑测试。
- 未启动 Tauri。
- 未执行真实 Codex。
- 未改 workflow state JSON。
