# Kickoff：S1B-H2-R4 真实 App 可归因对话与单张 Pending 卡验收 v1

- 日期：2026-07-22
- 状态：等待用户在场转发后执行
- 权威任务包：`tasks/2026-07-22-s1b-h2-r4-real-app-diagnostic-and-pending-card-verification-package-v1.md`

## 可直接执行的 kickoff

执行 `S1B-H2-R4 真实 App 可归因对话与单张 Pending 卡验收 v1`。

完整阅读并严格遵守：

`/Users/yoyi/workspace/product-line/tasks/2026-07-22-s1b-h2-r4-real-app-diagnostic-and-pending-card-verification-package-v1.md`

这是新的真实现场授权：重新完成 Gate 0，重建并冻结当前源码对应的裸 debug binary，只启动该 binary。用户在场，在固定测试项目中把首句 `我想给这个游戏里的标题改成小马里奥` 只发送一次；不得双击、自动重发或复用 R2 的 client identity。

只有首句 canonical recorded、同 message injected、主管自然 reply 全部成功后，才把 `按这个出方案` 发送一次。只允许预批准的 `supervisor_orchestrator.submit_proposal`；要求恰好新增一张目标匹配的 `PendingUserConfirmation` 卡，refresh 一次不重复，chain、worker 和测试项目保持不变。看到一张卡立即停止，不批准卡、不启动 chain。

如果首句失败，绝不发送第二句：只读对账同一 `message_id` 的 `supervisor_resident_delivery_diagnostic_recorded`。存在时只回传安全 `stage/stable_error_family/generation/thread`；缺失、重复或 identity 不匹配都按任务包对应 blocker 止损，不从用户面或私有 stderr 猜根因。若第二句对话成功但工具落卡失败，也不得重发。

不得修改代码、直接写真实 store、扩大工具/approval/sandbox、kill 进程、stage 或 commit。正常关闭 App 后完成 registry/holder/process、DB/JSON、卡片/chain 和固定测试项目 manifest 对账，再写新 evidence/CURRENT；失败则按稳定 family 或明确边界另出下一包。

