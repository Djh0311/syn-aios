# Codex 会话控制能力探针 v1 结果交接

## 薄弱点先说

- 本轮没有真实创建会话、resume 会话或发送 prompt。依据：任务包没有给出本轮明确写入批准。
- 帮助里的 `exec`、`resume`、`--json`、`--output-last-message` 只能算候选入口，不能算会话控制已支持。依据：未执行真实写入类命令。
- 当前不能证明 Codex CLI 写入哪些状态文件。依据：没有执行会话创建或 resume。
- `app-server` 和 `remote-control` 是 experimental，不能作为默认路线。依据：本地 help 标注 experimental。
- 本地 CLI 帮助命令都会出现 PATH 更新 warning。依据：探针记录 `codex_path_update_warning`。

## 做了什么

- 新增无副作用探针脚本。
- 新增探针测试。
- 读取本机 `codex` 路径、版本、顶层帮助和会话相关子命令帮助。
- 生成能力矩阵到 `/tmp/codex-session-control-probe-v1.json`。
- 只记录帮助线索摘要和 hash，不保存完整 help 正文到 evidence。

## 改了哪些文件

- `product-line/prototypes/index-kernel/codex_session_control_probe.py`
- `product-line/prototypes/index-kernel/tests/test_codex_session_control_probe.py`
- `product-line/evidence/2026-05-29-codex-session-control-probe-v1.md`
- `product-line/handoffs/2026-05-29-codex-session-control-probe-v1-result.md`

## 新增测试

新增 6 个测试，覆盖：

- 没有 `codex` 命令。
- help 无会话控制线索。
- help 有 `resume` / `exec` / `prompt` / `--json` 线索但未授权真实执行。
- 未授权时写入类能力为 blocked。
- 授权标记不等于真实 supported。
- JSON schema 稳定。
- help 中疑似敏感样式内容脱敏。

已运行：

```bash
python3 -m unittest product-line/prototypes/index-kernel/tests/test_codex_session_control_probe.py
python3 -m unittest discover product-line/prototypes/index-kernel/tests
```

结果：

- 新增测试 6 个通过。
- 全量 index-kernel 测试 44 个通过。

## 能力矩阵

```json
{
  "discover_cli": "supported",
  "inspect_help": "supported",
  "create_session": "blocked",
  "resume_session": "blocked",
  "send_prompt": "blocked",
  "wait_for_result": "blocked",
  "read_back_with_transcript": "blocked"
}
```

## 有依据支持的能力

- `discover_cli=supported`：`command -v codex` 找到 `/opt/homebrew/Cellar/node/23.11.0/bin/codex`。
- `inspect_help=supported`：`codex --help` 和相关子命令 help 成功返回。

CLI 版本：

- `codex-cli 0.134.0`

## unknown / blocked 能力

- `create_session=blocked`：帮助显示 `codex exec [PROMPT]` 和 `--ephemeral`，但未获真实写入授权，未执行。
- `resume_session=blocked`：帮助显示 `codex resume [SESSION_ID] [PROMPT]` 和 `codex exec resume [SESSION_ID] [PROMPT]`，但未获真实写入授权，未执行。
- `send_prompt=blocked`：帮助显示多个 `[PROMPT]` 入口，但未获真实写入授权，未执行。
- `wait_for_result=blocked`：帮助显示 `--json` 和 `--output-last-message`，但未真实执行，不能证明可等待和可机器读取结果。
- `read_back_with_transcript=blocked`：`transcript_reader.py` 已存在，但本轮没有新写入会话文件，所以无法读回新回复。

## 是否执行真实会话创建、resume 或发送

没有执行。

没有用户明确批准真实会话写入。

## 是否写了 `/Users/yoyi/.codex` 或 Codex 状态库

没有。

依据：

- 未运行会话写入类命令。
- 探针脚本只运行版本和 help 类命令。
- 临时结果写在 `/tmp/codex-session-control-probe-v1.json`。

## 是否适合进入下一步

不适合直接进入“Codex 工作流编排运行模型 v1”。

建议先做“受控真实会话写入探针 v2”。

原因：

- 当前只证明 CLI 存在和候选入口存在。
- 还没证明创建、恢复、发送、等待、读回能形成闭环。
- 也没证明真实写入会影响哪些 Codex 状态文件。

## 回收建议

建议：接受为无副作用探针结果。

但不要把它回收成“会话控制能力已完成”。

下一步如果继续，需要用户明确批准一次受控测试写入，测试 prompt 必须无业务内容，输出和统计必须可审计，不碰现有真实业务会话。
