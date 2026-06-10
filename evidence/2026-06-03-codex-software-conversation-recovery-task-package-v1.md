# Codex Software Conversation Recovery Task Package Evidence

时间：2026-06-03 21:24 CST

## 结论

本轮已新增任务包：

- `tasks/2026-06-03-codex-software-conversation-recovery-v1.md`

用途：处理用户反馈的“Codex 软件对话列表里有一些旧对话消失、不能被工作台识别”的问题。

## 任务包核心口径

- 先只读诊断，再按诊断结果做最小修复。
- 诊断需要对比 sqlite、兼容 index、sessions、archived_sessions、session_index、workflow bindings 等来源。
- 修复优先使用工作台自己的恢复 sidecar / 兼容 catalog，不写 Codex 原生状态。
- 不能把 `index.json` 重新变成 transcript 准入名单。

## 安全边界

任务包已明确禁止：

- 执行 `codex exec`。
- 执行 `codex exec resume`。
- 写 `/Users/yoyi/.codex`。
- 修改 Codex 原生 sqlite、session index、rollout JSONL 或内部状态库。
- 读取真实完整 transcript 正文作为诊断证据。
- 读取 auth、token、`.env`、密钥或授权文件。
- 把 rollout 正文复制进 evidence / handoff。

## 已更新入口

- `CURRENT.md`
- `tasks/README.md`

## 未做

- 未执行诊断。
- 未扫描真实 `/Users/yoyi/.codex`。
- 未读取真实 transcript。
- 未改产品代码。
- 未运行测试。
- 未修复任何真实会话列表。

原因：本轮只写任务包。

