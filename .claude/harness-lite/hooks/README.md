# Harness Lite 钩子接法

四个钩子共用项目内 runtime。前三个恢复上下文和写事实；`PreToolUse` 只在五类硬边界出现时给出决定，普通开发动作不打扰。

## Claude

把下面片段合并进项目自己的 `.claude/settings.json`，路径换成目标项目。安装器只生成片段，不覆盖用户设置。

```json
{
  "hooks": {
    "SessionStart": [{"hooks":[{"type":"command","command":"node \"/path/to/project/.claude/harness-lite/hooks/session-start.js\"","timeout":15}]}],
    "UserPromptSubmit": [{"hooks":[{"type":"command","command":"node \"/path/to/project/.claude/harness-lite/hooks/user-prompt.js\"","timeout":10}]}],
    "Stop": [{"hooks":[{"type":"command","command":"node \"/path/to/project/.claude/harness-lite/hooks/stop.js\"","timeout":30}]}],
    "PreToolUse": [{"hooks":[{"type":"command","command":"node \"/path/to/project/.claude/harness-lite/hooks/pre-push.js\" --surface claude","timeout":10}]}]
  }
}
```

Claude 的未授权硬门输出 `ask`，已经存在且匹配当前任务的用户授权输出 `allow`。

## Codex

Codex 使用同一核心，但适配语义不同：当前 Codex 不执行 `ask`，所以未授权必须输出 `deny`。安装器给出 `.codex/harness-lite/hooks-snippet.json`，由用户或项目维护者合并到受信任的 `.codex/hooks.json`。

```json
{
  "hooks": {
    "PreToolUse": [{"hooks":[{"type":"command","command":"node \"/path/to/project/.claude/harness-lite/hooks/pre-push.js\" --surface codex","timeout":10}]}]
  }
}
```

## 普通 CLI 与 Git

- 显式检查：`node .claude/harness-lite/bin/hl.js gate <类别> <动作> <目标> --target <项目> --write`。
- 原生 `pre-push` 应调用 `.claude/harness-lite/lib/git-pre-push.js`；它先检查当前用户授权，再做原有密钥扫描，任一失败都以非零退出。
- 安装器不会静默改 `.git/hooks`、`core.hooksPath`、Claude 或 Codex 的用户配置。

## 五类硬门

- `external`：进入远端、服务器、生产或真实世界。
- `destructive`：删除或难恢复动作。
- `context`：改变或结束当前工作。
- `control`：修改 Harness 的守门、授权或审计。
- `project-sensitive`：项目额外声明的真实凭据、设备等边界。

授权只从 `docs/harness/authorization.json` 读取，没有 `hl authorize` 自授权命令。break-glass 只引用当前用户授权里的专用 grant，并写 `docs/harness/audit/*.jsonl`；不记录原始命令或敏感参数。

## 可信边界

这是轻量协作门：Codex trusted hook 的 `deny`、Claude 的宿主决定、CLI 非零退出和 Git hook 会在各自注册入口阻断当次动作。同一 macOS 用户仍能停用 hook、改授权文件或直跑底层命令，因此不能宣传为防本机蓄意绕过的系统安全边界。

不要用 `PreCompact`；压缩后恢复由 `SessionStart` 完成。`Stop` 必须保持 30 秒，并通过 `hookSpecificOutput.additionalContext` 回传。
