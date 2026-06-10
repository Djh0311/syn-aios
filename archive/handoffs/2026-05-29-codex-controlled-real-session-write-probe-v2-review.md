# Codex 受控真实会话写入探针 v2 总指导回收意见

## 结论

接受。

接受为“Codex 最小真实会话写入和读回闭环已打通”。

不接受为“Codex 工作流编排已完成”，不接受为“可以直接自动派发真实业务任务”，不接受为“resume / 多轮 / 并发 / 失败恢复已验证”。

## 薄弱点

- 第一次命令因 `/tmp` 目录信任检查失败，说明后续工作台调用 Codex CLI 时必须显式管理工作目录或参数。
- 本轮只创建新测试会话，没有验证恢复已有会话。
- 本轮没有验证总指导会话向开发线会话派发、开发线回传、总指导回收的协议。
- `--json` 当前只看到最小 4 类事件，不能直接当完整运行协议。
- Codex CLI stderr 有远程插件和 MCP shutdown warning，后续长流程要观察是否会造成噪音或阻塞。

## 接受依据

- 用户已批准一次受控真实写入。
- 第二次 `codex exec --skip-git-repo-check --json --output-last-message ...` 退出码为 0。
- 线程数从 318 增加到 319。
- session JSONL 文件数从 318 增加到 319。
- 新 `thread_id` 为 `019e7389-349a-7f02-aa31-a4a90b24e865`。
- 新 rollout 文件为 `/Users/yoyi/.codex/sessions/2026/05/29/rollout-2026-05-29T19-40-32-019e7389-349a-7f02-aa31-a4a90b24e865.jsonl`。
- `--output-last-message` 写出最终回复，并包含目标文本。
- 临时索引能发现新会话。
- transcript reader 能读回新会话，12 个事件，坏 JSON 行 0，未知事件 0。
- evidence / handoff 没有包含完整 transcript 或完整事件流。

## 本轮验证

已复跑：

```bash
python3 -m unittest /Users/yoyi/workspace/product-line/prototypes/index-kernel/tests/test_codex_session_control_probe.py
```

结果：6 个通过。

已复跑：

```bash
python3 -m unittest /Users/yoyi/workspace/product-line/prototypes/index-kernel/tests/test_transcript_reader.py
```

结果：12 个通过。

已复跑：

```bash
python3 /Users/yoyi/workspace/product-line/prototypes/index-kernel/build_index.py --check /Users/yoyi/workspace/product-line/prototypes/index-kernel/codex-index.json
```

结果：`validation_ok`。

## 写入边界

开发线回传写入：

- `product-line/evidence/2026-05-29-codex-controlled-real-session-write-probe-v2.md`
- `product-line/handoffs/2026-05-29-codex-controlled-real-session-write-probe-v2-result.md`

总指导本轮新增：

- `product-line/handoffs/2026-05-29-codex-controlled-real-session-write-probe-v2-review.md`

Codex CLI 因受控测试自然写入：

- `/Users/yoyi/.codex/state_5.sqlite`
- `/Users/yoyi/.codex/sessions/2026/05/29/rollout-2026-05-29T19-40-32-019e7389-349a-7f02-aa31-a4a90b24e865.jsonl`

临时输出：

- `/tmp/codex-control-probe-v2-events.jsonl`
- `/tmp/codex-control-probe-v2-last-message.txt`
- `/tmp/codex-control-probe-v2-exit-code.txt`
- `/tmp/codex-control-probe-v2-index.json`
- `/tmp/codex-control-probe-v2-transcript.json`

## 安全边界

接受原因：

- prompt 是无业务测试文本。
- 没有向现有真实业务会话发送测试消息。
- 没有运行 resume 或 fork。
- 没有读取 `auth.json`、`.env`、授权文件或密钥文件。
- 没有删除、迁移、归档、重命名 Codex 会话。

## 对当前阶段的影响

可以派发下一任务：“Codex 工作流编排运行模型 v1”。

建议目标不要再回到任务包管理器，而是验证最小编排链路：

- 总指导节点生成任务指令。
- 执行节点创建新 Codex 会话或选择已有测试会话。
- 工作台发送任务 prompt。
- 工作台等待返回。
- 工作台用 transcript reader 读回执行结果。
- 总指导根据回传决定下一步。

任务包文件仍可作为内部协议和审计产物，但不作为主界面和主流程中心。
