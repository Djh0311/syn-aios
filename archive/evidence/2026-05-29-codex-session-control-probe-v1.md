# Codex 会话控制能力探针 v1 证据

## 薄弱点

- 本轮只做无副作用探针，没有真实创建会话、resume 会话或发送 prompt。依据：任务包要求没有明确批准时不能写 `/Users/yoyi/.codex`；本轮没有用户明确批准受控写入。
- 帮助里出现 `exec`、`resume`、`--json`、`--output-last-message` 只能证明存在候选入口，不能证明真实工作流可用。依据：未执行真实会话写入类命令。
- `app-server`、`remote-control` 标注为 experimental，不能作为阶段 3 默认控制路线。依据：本地帮助输出中这两个入口带 experimental 字样。
- `codex --version` 和帮助命令都出现 `could not update PATH` warning。依据：无副作用命令 stderr 中出现该 warning；探针结果归类为 `codex_path_update_warning`。
- 当前没有验证 Codex CLI 是否会写 state sqlite、session JSONL 或其它状态文件。依据：本轮没有执行会话创建、resume 或发送。

## 做了什么

- 新增无副作用能力探针脚本：`product-line/prototypes/index-kernel/codex_session_control_probe.py`。
- 新增测试：`product-line/prototypes/index-kernel/tests/test_codex_session_control_probe.py`。
- 运行本机 `codex` 版本和帮助探针。
- 输出 JSON 能力矩阵到 `/tmp/codex-session-control-probe-v1.json`。
- 没有执行真实会话创建、resume、fork、发送 prompt。
- 没有启动桌面 UI。
- 没有运行 harness。

## 无副作用命令

已运行：

```bash
command -v codex
codex --version
codex --help
codex exec --help
codex exec resume --help
codex resume --help
codex fork --help
codex mcp-server --help
codex app-server --help
codex remote-control --help
codex features --help
```

没有运行：

- `codex exec <prompt>`
- `codex resume <session> <prompt>`
- `codex exec resume <session> <prompt>`
- `codex fork <session> <prompt>`
- `codex app`
- `codex app-server daemon`
- `codex remote-control start`

## CLI 发现结果

```json
{
  "available": true,
  "path": "/opt/homebrew/Cellar/node/23.11.0/bin/codex",
  "version": "codex-cli 0.134.0",
  "help_checked": true,
  "commands_detected": [
    "exec",
    "review",
    "login",
    "logout",
    "mcp",
    "plugin",
    "mcp-server",
    "app-server",
    "remote-control",
    "app",
    "completion",
    "update",
    "doctor",
    "sandbox",
    "debug",
    "apply",
    "resume",
    "fork",
    "cloud",
    "exec-server",
    "features",
    "help"
  ]
}
```

## 能力矩阵

```json
{
  "discover_cli": {
    "status": "supported",
    "basis": [
      "codex executable found on PATH"
    ]
  },
  "inspect_help": {
    "status": "supported",
    "basis": [
      "codex --help completed"
    ]
  },
  "create_session": {
    "status": "blocked",
    "basis": [
      "help shows codex exec accepts an initial prompt",
      "codex exec help lists --ephemeral"
    ],
    "candidate_entrypoints": [
      "codex exec [PROMPT]"
    ],
    "blocked_reason": "real_session_probe_not_authorized"
  },
  "resume_session": {
    "status": "blocked",
    "basis": [
      "help shows codex resume accepts SESSION_ID",
      "help shows codex exec resume accepts SESSION_ID"
    ],
    "candidate_entrypoints": [
      "codex resume [SESSION_ID] [PROMPT]",
      "codex exec resume [SESSION_ID] [PROMPT]"
    ],
    "blocked_reason": "real_session_probe_not_authorized"
  },
  "send_prompt": {
    "status": "blocked",
    "basis": [
      "codex exec help includes [PROMPT]",
      "codex resume help includes [PROMPT]",
      "codex exec resume help includes [PROMPT]"
    ],
    "candidate_entrypoints": [
      "codex exec [PROMPT]",
      "codex resume [SESSION_ID] [PROMPT]",
      "codex exec resume [SESSION_ID] [PROMPT]"
    ],
    "blocked_reason": "real_session_probe_not_authorized"
  },
  "wait_for_result": {
    "status": "blocked",
    "basis": [
      "exec help lists --json machine-readable event output",
      "exec help lists --output-last-message",
      "exec help describes non-interactive execution"
    ],
    "candidate_entrypoints": [
      "codex exec --json",
      "codex exec resume --json"
    ],
    "blocked_reason": "real_session_probe_not_authorized"
  },
  "read_back_with_transcript": {
    "status": "blocked",
    "basis": [
      "transcript_reader.py exists",
      "CLI help has candidate session creation or resume entrypoint"
    ],
    "candidate_entrypoints": [
      "transcript_reader.py after a verified persisted session write"
    ],
    "blocked_reason": "real_session_probe_not_authorized"
  }
}
```

## 候选入口

- `codex exec [PROMPT]`：候选用于创建非交互会话和发送初始 prompt，但未真实执行。
- `codex resume [SESSION_ID] [PROMPT]`：候选用于恢复交互会话并发送 prompt，但未真实执行。
- `codex exec resume [SESSION_ID] [PROMPT]`：候选用于非交互恢复会话并发送 prompt，但未真实执行。
- `codex exec --json`：候选用于机器读取事件输出，但未真实执行。
- `codex exec resume --json`：候选用于机器读取 resume 后事件输出，但未真实执行。
- `transcript_reader.py after a verified persisted session write`：候选用于读回持久化会话文件，但本轮没有创建新会话文件。

## 测试

已运行：

```bash
python3 -m unittest product-line/prototypes/index-kernel/tests/test_codex_session_control_probe.py
python3 -m unittest discover product-line/prototypes/index-kernel/tests
```

结果：

- 新增探针测试：6 个通过。
- index-kernel 全量测试：44 个通过。

测试覆盖：

- 没有 `codex` 命令时 `codex_cli.available=false`。
- help 不含会话控制线索时能力保持 unknown。
- help 含 `resume` / `exec` / `prompt` / `--json` 线索时只生成候选能力，不标记真实 supported。
- 未获真实执行授权时写入类能力标记 blocked。
- 即使传入授权标记，v1 脚本仍不执行真实写入，也不把控制能力标记 supported。
- 输出 JSON schema 稳定。
- help 中疑似敏感样式内容会在 evidence lines 里脱敏。

## 写入边界

实际写入仓库文件：

- `product-line/prototypes/index-kernel/codex_session_control_probe.py`
- `product-line/prototypes/index-kernel/tests/test_codex_session_control_probe.py`
- `product-line/evidence/2026-05-29-codex-session-control-probe-v1.md`
- `product-line/handoffs/2026-05-29-codex-session-control-probe-v1-result.md`

临时输出：

- `/tmp/codex-session-control-probe-v1.json`

没有写 `/Users/yoyi/.codex` 或 Codex 状态库。

依据：

- 没有运行 `codex exec`、`codex resume`、`codex exec resume`、`codex fork` 等会话写入类命令。
- 探针脚本只运行版本和帮助类命令。
- 输出只写到 `/tmp` 和允许的 evidence / handoff 文件。

## 是否执行真实会话创建、resume 或发送

没有。

原因：

- 任务包要求没有用户明确批准时只能做无副作用探针。
- 本轮没有用户明确批准受控测试写入。

## 结论

可接受为“无副作用能力探针结果”，但不能接受为“Codex 会话控制能力已经打通”。

当前 supported：

- 发现本机 Codex CLI。
- 读取本地版本和帮助信息。

当前 blocked：

- 新建会话。
- resume 既有会话。
- 发送 prompt。
- 等待执行完成并拿到回复。
- 用 transcript 读取器读回新回复。

是否适合进入“Codex 工作流编排运行模型 v1”：

- 不适合直接进入自动编排运行模型。
- 适合先进入“受控真实会话写入探针 v2”，前提是用户明确批准一个临时、无业务内容、可审计的测试会话写入。
