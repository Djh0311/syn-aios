# 绑定 agent world 业务 Codex 会话 v1 evidence

## 范围

- 目标项目：`/Users/yoyi/gameai/agent world`
- 目标 workflow：`workflow:users-yoyi-gameai-agent-world:default`
- 目标节点：`workflow:users-yoyi-gameai-agent-world:default:node:codex-dev`
- 目标 work item：`work-item:workflow:users-yoyi-gameai-agent-world:default:1780032043420`
- 业务 thread：`019e76d9-0f67-7433-81eb-72da585d28a4`

## 薄弱点

- 这一步只重新绑定业务会话，不执行真实业务任务。
- 业务会话是 no-op 创建的会话，最后回复为 `BUSINESS_SESSION_READY_2026_05_30`。
- 目标项目目录当前顶层条目数为 0，后续只读体检可能只能回传目录为空。

## 前置复核

- 当前索引字段是 `threads[]`，不是 `sessions[]`。
- 业务 thread 在 `threads[]` 中 count = 1。
- project_root：`/Users/yoyi/gameai/agent world`
- rollout_exists：true。
- rollout_path：`/Users/yoyi/.codex/sessions/2026/05/30/rollout-2026-05-30T11-06-37-019e76d9-0f67-7433-81eb-72da585d28a4.jsonl`

## 写入结果

- 是否写真实 workflow state：是。
- 是否写 `/Users/yoyi/.codex`：否。
- 是否执行 `codex exec` 或 `codex exec resume`：否。
- 是否发送 Codex 消息：否。
- 是否读取完整 transcript：否。
- 是否读取授权、密钥、`.env`、token：否。

## 绑定结果

- binding id：`binding:workflow-users-yoyi-gameai-agent-world-default:workflow-users-yoyi-gameai-agent-world-default-node-codex-dev:work-item-workflow-users-yoyi-gameai-agent-world-default-1780032043420`
- before thread：`019e7389-349a-7f02-aa31-a4a90b24e865`
- after thread：`019e76d9-0f67-7433-81eb-72da585d28a4`
- warnings_count：0
- rollout_exists：true

## 审计和备份

- audit event id：`audit:workflow-node-session:workflow-users-yoyi-gameai-agent-world-default-node-codex-dev:1780111013102`
- event type：`workflow_node_session_rebound`
- backup：`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780111013102.json`

## 验证

- 只读复核 active binding count = 1。
- active binding thread id = `019e76d9-0f67-7433-81eb-72da585d28a4`。
- 审计事件存在。
- 备份文件存在。
- `python3 prototypes/index-kernel/build_index.py --check prototypes/index-kernel/codex-index.json`：通过，`validation_ok`。

## 下一步

回到第一条用户审核极小业务试跑的确认问题。

绑定已满足业务会话前置条件，但仍需要用户确认是否执行“只读项目目录体检”。
