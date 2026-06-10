# Handoff: task package granularity batch rule v1

日期：2026-06-02

## 结果

已把任务包粒度从“过度微步骤”调整为“批次化小步”。

当前下一步不再是单独 `final-skeleton-07`，而是“画布基础批次”：

- `final-skeleton-07`
- `final-skeleton-08`
- `final-skeleton-09`

批次完成后统一输出 evidence / handoff。执行中如果触及硬边界，必须拆开并停下来。

## 改动文件

- `tasks/2026-06-01-final-workbench-skeleton-execution-package-v1.md`
- `tasks/README.md`
- `CURRENT.md`
- `README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `evidence/2026-06-02-task-package-granularity-batch-rule-v1.md`
- `handoffs/2026-06-02-task-package-granularity-batch-rule-v1-result.md`

## 当前入口

先读：

1. `CURRENT.md`
2. `tasks/README.md`
3. `tasks/2026-06-01-final-workbench-skeleton-execution-package-v1.md`

当前执行批次：

- 画布基础批次，合并执行 `final-skeleton-07` 到 `final-skeleton-09`。

## 验证

只改文档，没有跑代码测试。

## 边界

本轮未改代码、未启动 Tauri、未执行真实 Codex、未改 workflow state JSON。
