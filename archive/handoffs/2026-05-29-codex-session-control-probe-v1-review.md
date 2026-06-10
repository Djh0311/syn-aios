# Codex 会话控制能力探针 v1 总指导回收意见

## 结论

接受。

接受为“Codex 会话控制无副作用探针 v1 已完成”。

不接受为“Codex 会话控制能力已打通”，不接受为“可以直接进入自动编排运行”，不接受为“工作台内对话 UI 可以开始实现”。

## 薄弱点

- 本轮没有真实创建会话、resume 会话或发送 prompt。
- 帮助里的 `exec`、`resume`、`--json`、`--output-last-message` 只能算候选入口，不能证明真实闭环。
- 不能证明 Codex CLI 会写哪些状态文件，因为没有执行会话写入类命令。
- `app-server` 和 `remote-control` 是 experimental，不适合作为阶段 3 默认路线。
- 本地 CLI help / version 命令出现 `could not update PATH` warning，需要后续诊断，但不阻塞本轮无副作用探针。

## 接受依据

- 新增无副作用探针脚本 `product-line/prototypes/index-kernel/codex_session_control_probe.py`。
- 新增测试 `product-line/prototypes/index-kernel/tests/test_codex_session_control_probe.py`。
- 能发现本机 Codex CLI：`/opt/homebrew/Cellar/node/23.11.0/bin/codex`。
- 能读取 CLI 版本：`codex-cli 0.134.0`。
- 能读取顶层 help 和会话相关子命令 help。
- 能力矩阵没有把候选入口误标为 supported；写入类能力均为 blocked。
- help 中疑似敏感样式内容会脱敏。
- evidence / handoff 没有包含密钥、授权内容或完整会话正文。

## 能力矩阵回收口径

接受的 supported：

- `discover_cli`
- `inspect_help`

接受的 blocked：

- `create_session`
- `resume_session`
- `send_prompt`
- `wait_for_result`
- `read_back_with_transcript`

blocked 的原因是没有用户明确批准真实写入探针，不是能力不存在。

## 本轮验证

已复跑：

```bash
python3 -m unittest product-line/prototypes/index-kernel/tests/test_codex_session_control_probe.py
```

结果：6 tests OK。

已复跑：

```bash
python3 product-line/prototypes/index-kernel/build_index.py --check product-line/prototypes/index-kernel/codex-index.json
```

结果：`validation_ok`。

没有复跑完整 discover；回传记录为 44 个通过。

## 写入边界

开发线回传的写入文件：

- `product-line/prototypes/index-kernel/codex_session_control_probe.py`
- `product-line/prototypes/index-kernel/tests/test_codex_session_control_probe.py`
- `product-line/evidence/2026-05-29-codex-session-control-probe-v1.md`
- `product-line/handoffs/2026-05-29-codex-session-control-probe-v1-result.md`

总指导本轮新增：

- `product-line/handoffs/2026-05-29-codex-session-control-probe-v1-review.md`

临时输出：

- `/tmp/codex-session-control-probe-v1.json`

## 安全边界

接受原因：

- 没有运行 `codex exec <prompt>`。
- 没有运行 `codex resume <session> <prompt>`。
- 没有运行 `codex exec resume <session> <prompt>`。
- 没有运行 `codex fork <session> <prompt>`。
- 没有写 `/Users/yoyi/.codex`。
- 没有改 Codex 状态库。
- 没有向现有真实业务会话发送测试消息。

## 对当前阶段的影响

下一步不能直接进入“Codex 工作流编排运行模型 v1”。

下一步应先做“受控真实会话写入探针 v2”，并需要用户明确批准一次受控测试写入。v2 必须回答：

- `codex exec [PROMPT]` 是否会创建可读回的会话。
- `codex exec --json` 是否能给出稳定机器可读事件。
- `--output-last-message` 是否能拿到最终回复。
- 新会话是否能被当前索引或 transcript reader 发现。
- 写入了哪些 Codex 文件或状态。
- 能否在不碰现有业务会话的前提下清理或标记测试产物。
