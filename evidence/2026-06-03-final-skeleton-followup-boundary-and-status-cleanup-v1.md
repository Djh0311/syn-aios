# Evidence：最终骨架后续任务边界和文档状态清理 v1

## 这轮做了什么

只改文档，不改产品代码。

清理了 `final-skeleton-07` 到 `final-skeleton-09` 完成后的后续任务冲突：

- 总执行包尾部不再把“画布基础批次”写成当前下一批次。
- 当前下一批次统一为 `final-skeleton-10-blackboard-candidate-schema-design-v1`。
- `final-skeleton-11` 明确必须在用户确认 schema / 迁移计划并允许最小实现后才能开始。
- `final-skeleton-11` 明确只改变黑板候选状态和审计，不写正式事实、正式记忆或 workflow 状态。
- `final-skeleton-14` 明确只做记忆候选生命周期，不写正式长期记忆。
- `final-skeleton-12`、`14`、`15` 的完成后跳转从“进入最终统一验收”改为继续后续 Skeleton。
- `tasks/README.md` 的当前执行目标不再说先补齐画布批次。

## 改了哪些文件

- `tasks/2026-06-01-final-workbench-skeleton-execution-package-v1.md`
- `tasks/README.md`
- `CURRENT.md`

## 未做

- 未改产品代码。
- 未跑代码测试。
- 未写 workflow state。
- 未读写 `/Users/yoyi/.codex`。
- 未执行真实 Codex。

## 判断

现在可以把 `final-skeleton-10` 交给其他对话执行；执行范围只能是 schema / 迁移计划 / 后续实现任务包草案。写完必须停下来给用户确认，不能直接实现 `final-skeleton-11`。
