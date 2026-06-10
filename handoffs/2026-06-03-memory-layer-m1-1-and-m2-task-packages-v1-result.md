# Memory Layer M1.1 and M2 Task Packages Result

时间：2026-06-03 21:14 CST

## 本轮做了什么

已新增两个可交给其他对话执行的任务包：

- `tasks/2026-06-03-memory-layer-m1-1-formal-memory-context-binding-guard-v1.md`
- `tasks/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1.md`

并更新：

- `CURRENT.md`
- `tasks/README.md`
- `STAGE_PLAN.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `tasks/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1.md`

## 当前执行顺序

1. 先执行 M1.1：正式记忆上下文绑定校验。
2. M1.1 完成后执行 M2：候选到正式记忆受控采纳。

M2 不能跳过 M1.1。

## M1.1 重点

- 校验 `project_root` 和 `project_id` / `workflow_id` / scope 的真实绑定。
- 不只检查请求字段内部一致性。
- 不实现候选采纳。

## M2 重点

- 受控采纳 `MemoryCandidate` 为正式记忆。
- 生成 `MemoryRecord`、`MemoryVersion`、`MemoryAuditEvent`。
- 候选 store 保留历史并关联正式记忆 ID。
- 用户偏好、全局蓝图、成熟模式、高风险、跨项目和敏感候选必须用户确认。
- project_director 只可采纳低风险本项目记忆。

## 未做

- 未改产品代码。
- 未实现任何命令。
- 未运行测试。
- 未执行真实 Codex。
- 未读写 `/Users/yoyi/.codex`。

原因：本轮任务是写任务包和入口，不是实现。

## 下一步

把 `tasks/2026-06-03-memory-layer-m1-1-formal-memory-context-binding-guard-v1.md` 交给其他对话执行。M1.1 回收通过后，再执行 `tasks/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1.md`。
