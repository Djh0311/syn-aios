# Kickoff：S1B-H2-R3B 安全内部诊断闭环 v1

- 日期：2026-07-22
- 状态：等待用户转发后执行
- 权威任务包：`tasks/2026-07-22-s1b-h2-r3b-safe-internal-diagnostic-package-v1.md`

## 可直接执行的 kickoff

执行 `S1B-H2-R3B 安全内部诊断闭环 v1`。

先完整阅读并严格遵守：

`/Users/yoyi/workspace/product-line/tasks/2026-07-22-s1b-h2-r3b-safe-internal-diagnostic-package-v1.md`

R3 的结论是 `NEEDS_SAFE_INTERNAL_DIAGNOSTIC`：三条用户消息已各自 canonical-recorded，但都止于 prepared 前；现有入口吞掉多类 consult 错误，不能猜定代码根因。只允许增加一个通过既有 Batch 2 canonical 写路的、message-scoped 稳定错误族诊断事实，并做离线先红后绿验证。

不得改 H2 单工具批准逻辑、read-only/approval/sandbox、watchdog、invalid-resume 单次轮转、进程组清理、M5 DB-primary/CAS/降级逻辑、真实 store 或测试项目。不得新增 command、sidecar、MCP server 或消息路。diagnostic 写失败必须不影响已完成的 recorded 业务事实；原始错误、stderr、用户正文、auth/token 与私有 `CODEX_HOME` 不得进 canonical/read model/仓库证据。

代码和离线闸通过即停：不启动真实 App、不发任何消息、不操作真实 store、不点卡、不启动 chain、不 stage/commit。真实 R4 另包、另授权。
