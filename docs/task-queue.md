# 当前任务入口

Stage 3 归档后没有活动工程任务。

下一入口是 M3 只读指导：依据 `docs/plans/2026-08-01-syn-stage-3-role-session-and-explicit-handoff-plan-v1.md`，重新核对当前 main 的代码、M1/M2 exits、未知项、HOLD、任务切片和验收边界。

该入口只产生复核结论或待用户确认的 M3 计划，不授权：

- 修改产品代码；
- 激活 Harness Lite stage/leaf；
- 运行真实 provider、消息或账号；
- 迁移 live Workbench；
- push、部署或发布。

具体执行只看新的用户指令、Harness Lite 当前链和 `docs/harness/authorization.json`，不从 `tasks/**`、handoff 或历史 `CURRENT/AUTHORITY` 恢复权限。
