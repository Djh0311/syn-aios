# Harness Lite 0.8 Hooks

四个 Codex 事件共用 `hooks/dispatcher.js`：`SessionStart` 恢复短链，`UserPromptSubmit` 记录 user-role 来源 receipt 和 Git baseline，`Stop` 只有在短期授权文件与当前 project/session/turn/leaf/stage 精确绑定且有新产品进展时才返回顶层 `decision: "block"`；固定 continuation 先判为 internal，不能自签授权或扩大范围。`PreToolUse` 只匹配 `^Bash$` 并硬拦未确认的精确 push。

`project` profile 把 definition 精确合并到权威 root checkout 的 `.codex/hooks.json` 并保留外来组。`managed` profile 不写项目 Harness Hook，由管理员配置的 `managed_dir` 统一加载 global gateway；gateway 校验受保护 registry、allowlist 和 global runtime 后才转发。非法 JSON、未知 Harness 冲突、未知 digest 或不安全路径进入 HOLD。

官方 wire 字段使用 `hook_event_name`、`session_id`、turn 事件的 `turn_id`，Bash 命令使用 `tool_input.command`。`transcript_path` 不是稳定接口。PreToolUse 拒绝使用 `hookSpecificOutput.permissionDecision: "deny"`；非 push 输出为空，绝不返回尚未支持的 `ask`。

健康状态必须分别证明 installed、configured、trusted/policyTrusted、四事件 observed，以及 native pre-push configured/probed。证据 bundle 保存精简 hooks/list 前后快照、started/completed run id 和无秘密 receipt，再做 exact/ambiguous/missing join。Desktop、CLI、unknown-live 和 synthetic 分开计算，离线直接执行 dispatcher 不能补真实宿主或 Desktop。

原生 Git pre-push 是单独、repo-wide 的维护动作；需要精确授权后才链式保留外来 hook，并用 disposable local bare remote probe。它与 Codex PreToolUse 共用同一 pending + user-role event + 模型 push-assert 的一次性绑定和密钥扫描逻辑。wrapper 在已管理 worktree 发现 runtime identity 或 marker 损坏时 fail closed；未安装 0.8 runtime 的其它 worktree不介入，只链外来 hook。
