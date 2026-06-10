# Evidence：codex-native-conversation-task-correction-v1

日期：2026-06-03

## 背景

用户原始问题是 Codex 原生软件自己的对话列表里有旧对话消失、不能被 Codex 识别。

此前 `tasks/2026-06-03-codex-software-conversation-recovery-v1.md` 错误地把问题解释为工作台智能体页不能识别旧 Codex 会话，并已被另一个对话执行完成。

该执行结果只能接受为工作台侧恢复读模型，不接受为 Codex 原生 app 修复。

## 本轮完成

新增真正针对 Codex 原生软件的任务包：

- `tasks/2026-06-03-codex-native-app-conversation-list-repair-v1.md`

该任务包明确：

- 验收对象是 Codex 原生 app 会话列表，不是工作台智能体页。
- 工作台 sidecar 不能作为成功标准。
- 诊断阶段必须先确认 Codex 原生 app 使用的数据源。
- 写 `.codex`、sqlite、session index 或缓存前必须有文件级用户批准、备份和回滚方案。
- 不读取真实完整 rollout / transcript 正文。
- 不改 rollout JSONL 正文。
- 修复后必须由 Codex 原生 app 重新显示旧对话来验收。

更新旧任务包：

- `tasks/2026-06-03-codex-software-conversation-recovery-v1.md`

已在顶部标明：

- 已完成但目标错误。
- 已被 `2026-06-03-codex-native-app-conversation-list-repair-v1.md` superseded。
- 实际完成内容是工作台侧旧 Codex 会话恢复读模型。

更新当前入口：

- `CURRENT.md`
- `tasks/README.md`

两处都已说明：

- 旧任务不接受为 Codex 原生 app 会话列表已修复。
- 真正待执行任务是 `tasks/2026-06-03-codex-native-app-conversation-list-repair-v1.md`。

## 本轮未做

- 未修改产品代码。
- 未读取 `/Users/yoyi/.codex`。
- 未写 `/Users/yoyi/.codex`。
- 未修改 Codex 原生 sqlite、session index、缓存或 rollout。
- 未执行 `codex exec`。
- 未执行 `codex exec resume`。
- 未启动或重启 Codex 原生 app。

## 结论

当前权威状态已经纠偏：

- `codex-software-conversation-recovery-v1` 是历史错误目标任务，只能作为工作台侧恢复实现记录。
- `codex-native-app-conversation-list-repair-v1` 是真正可交给其他对话执行的 Codex 原生 app 旧对话列表修复任务包。
