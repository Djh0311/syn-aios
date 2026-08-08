@AGENTS.md

Claude 使用 `.claude/harness-lite/settings-snippet.json`：SessionStart 恢复当前链，UserPromptSubmit 提醒范围，Stop 写机器事实，PreToolUse 调用共享 gate。未授权硬门映射为 `ask`；已有匹配用户授权映射为 `allow`。安装器不会覆盖项目自己的 `.claude/settings.json`。
