# Kickoff：S1B-H2-R4F 一次真实工具归因与 Pending 卡收口 v1

S1B-H2-R4F 开工；用户在场。

完整阅读并严格执行：

`tasks/2026-07-22-s1b-h2-r4f-one-pass-real-app-tool-attribution-and-pending-card-package-v1.md`

授权从全新 Gate 0 重新核空 holder/registry，冻结当前源码、真实 store 与固定测试项目，重新构建并冻结当前裸 debug binary，只启动该 binary。

首句只发送一次：

`我想给这个游戏里的标题改成小马里奥`

只有同 message 完成 canonical recorded、injected 和主管自然回复后，第二句才只发送一次：

`按这个出方案`

第二句必须使用 R4E 的 message-scoped `tools_list_served → tools_call_received → submit_handler_entered → submit_handler_finished → tool_audit_boundary` 事实，当场裁决 PASS、A、B、C、D1、D2、D3 或 LIVE-DIAG。不得依赖 R4D 私有 trace，不得重发、现场修码或再出中间诊断包。

只允许既有唯一预批准 `supervisor_orchestrator.submit_proposal`。恰好新增一张目标 `PendingUserConfirmation` 卡后停止，只 refresh 一次；不点卡、不批准、不启动 chain/worker、不修改固定测试项目。

失败时只出一个最小修复任务包与 kickoff，不执行修复。成功时 H2 收口，下一阶段回到交办页方案列表 UI。

正常 Quit 后，如仅残留本轮已核验裸 binary，且 registry/store holder 均为 0，授权只向该精确单一 PID 发送一次 `TERM` 并复核清零；禁止 `pkill`、进程组 kill、`KILL/-9` 或触碰其他进程。

不得修改代码、配置、审批或安全闸，不得直接写/恢复/reseed/migrate 真实 store，不得 stage/commit/reset/clean/stash。
