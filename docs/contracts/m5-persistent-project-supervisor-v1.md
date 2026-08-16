# M5 持久 Project Supervisor 合同 v1

- 版本：v1（2026-08-16）
- 状态：**FROZEN（M5R04 冻结）**
- 关系：补充 M3 RoleSession 与 M5R02 编排；**不改 M1–M4 正文与 hash**。

## 规则

- 身份真源是 M3 `RoleSessionId`，经 `ProjectSupervisorRoleSessionPort` 读取；禁止平行字符串 session。
- 默认 chat/read 零 Proposal、零 Grant、零 spawn、零业务副作用。
- 结构化动作先 `SubmitProposal`（DRAFT）；只有 APPROVED 后才能调用 M5R02 `prepare_and_dispatch`。
- Supervisor 不得跨项目 dispatch；两个项目 binding 不得串用。
- 重启同一 `project_id + role_session_id` 恢复同一 binding。
