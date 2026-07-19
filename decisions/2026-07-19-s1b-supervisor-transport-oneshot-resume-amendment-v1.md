# S1B 主管传输选型修正 v1

日期：2026-07-19  
权威任务包：`tasks/2026-07-18-s1b-supervisor-transport-oneshot-resume-package-v1.md`

## 决定

2026-07-19（选型落档；实现与离线验证见 S1B evidence）：P1-0b 原定主方案 `codex mcp-server` 改判为项目常驻私有 `CODEX_HOME` 上每回合一次一发的 `codex exec` / `codex exec resume`；原 shell-resume 备胎转正，依据是两次 420 秒整回合超时、超时后宿主/桥孤儿与陈账风险；私有 MCP 白名单仅 `supervisor_orchestrator`，canonical/audit 与用户确认闸不变。

仓内不存在可更新的独立 P1-0 决策文件；本 amendment 不改写散落在历史任务包与 `CURRENT.md` 中的当时事实。

本决定只确定传输选型。07-19 后续副本店 live 已实证 invalid-resume 自愈换代、事实注入和后两轮同新 thread 续接；完整 ignored 场景则在私有 MCP handler 前被 Codex 客户端以 `user cancelled MCP tool call` 取消。该批准可达性另列 S1B-H1 harness 欠账；修好并复跑前，仍不能把离线 handler 覆盖或部分 live 称为真实工具落卡验收。

## 不变项

- 主管仍只在固定测试项目、`read-only` 沙箱和私有白名单 MCP 下工作。
- 聊天只写既有 canonical 用户/注入/主管消息事件；终版方案仍只能经 `submit_proposal` 生成 `PendingUserConfirmation` 卡。
- 任何一次一发子进程退出后保留 thread 与项目私有 home；不会保留常驻 guardian。
