# Memory Layer M1.1 and M2 Task Packages Evidence

时间：2026-06-03 21:14 CST

## 结论

本轮完成两个记忆层后续任务包的编写，并更新当前入口。

新增任务包：

- `tasks/2026-06-03-memory-layer-m1-1-formal-memory-context-binding-guard-v1.md`
- `tasks/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1.md`

已修正：

- `tasks/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1.md` 的状态从“待执行”改为“已完成”。

## 为什么先有 M1.1

M1 复核发现：正式记忆创建已经有 store / version / audit 骨架，但项目上下文绑定校验还不够实。

风险是：

- 请求里 `project_id == scope.project_id` 不代表它们真的属于 `project_root`。
- M2 采纳候选为正式记忆时，如果不先补绑定校验，候选可能被写到错误项目或错误 workflow 范围。

因此拆出 M1.1，作为 M2 前置。

## M1.1 任务目标

补正式记忆创建的上下文绑定校验：

- 后端从 `project_root` 推导 `expected_project_id`。
- 后端从 `project_root` 推导 `expected_workflow_id`。
- 校验 `project_id`、`workflow_id`、`scope.project_id`、`scope.workflow_id` 与后端推导结果一致。
- 校验 `project_director` 只能写本项目 / 本 workflow / 本 session。
- 建议只读校验 `project_root` 是否存在于当前 workflow state projects[]。

M1.1 禁止实现候选采纳。

## M2 任务目标

实现候选到正式记忆的受控采纳：

- 新增采纳命令。
- 校验角色、风险、作用域、来源、冲突和上下文绑定。
- 从 `MemoryCandidate` 生成正式 `MemoryRecord`、`MemoryVersion`、`MemoryAuditEvent`。
- 候选 store 保留历史，并能反查正式记忆 ID。
- UI / 读模型显示候选已受控采纳为正式记忆。

M2 必须等 M1.1 完成后执行。

## 已更新入口

- `CURRENT.md`
- `tasks/README.md`
- `STAGE_PLAN.md`
- `docs/plans/middleware-version-stage-plan-v1.md`

## 未做

- 未实现 M1.1。
- 未实现 M2。
- 未改产品代码。
- 未跑 npm / cargo。
- 未执行真实 Codex。
- 未读写 `/Users/yoyi/.codex`。

## 边界

- M1.1 和 M2 都不能改 `workflow-state.v0.json` 结构。
- M2 不能自动采纳所有候选。
- 秘书、worker、黑板候选、知识库命中、LLM 摘要都不能直接写正式记忆。
- M2 完成后仍不能宣称任务包召回、任务包注入、正式记忆生命周期或中间版本记忆层完成。

