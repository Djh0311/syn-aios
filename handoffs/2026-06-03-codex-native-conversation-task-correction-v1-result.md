# Handoff：codex-native-conversation-task-correction-v1

日期：2026-06-03

## 结论

已完成任务包纠偏。

用户要修的是 Codex 原生软件自己的旧对话列表，不是工作台智能体页。此前 `2026-06-03-codex-software-conversation-recovery-v1.md` 已被执行，但目标错误，只能算工作台侧旧 Codex 会话恢复读模型完成。

真正待执行任务包：

- `tasks/2026-06-03-codex-native-app-conversation-list-repair-v1.md`

## 已改文件

- `tasks/2026-06-03-codex-native-app-conversation-list-repair-v1.md`
- `tasks/2026-06-03-codex-software-conversation-recovery-v1.md`
- `CURRENT.md`
- `tasks/README.md`
- `evidence/2026-06-03-codex-native-conversation-task-correction-v1.md`
- `handoffs/2026-06-03-codex-native-conversation-task-correction-v1-result.md`

## 新任务包边界

接受为目标：

- Codex 原生 app 会话列表旧对话消失的诊断和修复。
- 修复后旧对话必须重新出现在 Codex 原生 app 列表。
- 写 Codex 原生数据前必须有用户文件级批准、备份和回滚方案。

不接受为目标：

- 工作台智能体页能看到旧会话。
- 工作台 `codex-conversation-recovery.v1.json` sidecar 写入。
- 只读证明 rollout 或 sqlite 里有记录。

## 本轮边界

- 未改产品代码。
- 未读取 `/Users/yoyi/.codex`。
- 未写 `/Users/yoyi/.codex`。
- 未改 Codex 原生 sqlite、session index、缓存或 rollout。
- 未执行真实 Codex。

## 交给其他对话时的入口

先读：

- `CURRENT.md`
- `tasks/README.md`
- `tasks/2026-06-03-codex-native-app-conversation-list-repair-v1.md`

不要继续执行：

- `tasks/2026-06-03-codex-software-conversation-recovery-v1.md`

那份已经被 superseded，且方向是工作台侧恢复。
