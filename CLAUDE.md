# CLAUDE.md — 入口桥接

本仓库 agent 规则唯一正本是 `AGENTS.md`；本文件只保留 Claude 的入口适配，不复制另一套宪法。

@AGENTS.md

Claude 专用补充：

- `AGENTS.md` 中对 Codex 或 agent 的约束，对 Claude 同样生效。
- 开工和上下文压缩后运行 Lite 的 `chain`、`progress`、`auth`，按唯一 current leaf 工作。
- `handoffs/`、旧 `CURRENT/AUTHORITY` 和历史任务包只供背景，不改变当前 leaf 或授权。
- 主导线负责统筹、用户沟通和核实实物；执行线负责范围内实现。
- 跨会话记忆只存稳定项目事实与协作经验；规则以磁盘 `AGENTS.md` 为准，动态工作以 Lite 当前链为准。发现记忆与磁盘冲突时，以新鲜磁盘和直接验证为准。
