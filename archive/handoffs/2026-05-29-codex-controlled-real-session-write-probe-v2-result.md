# Codex 受控真实会话写入探针 v2 结果

## 薄弱点

- 第一次命令失败在 `/tmp` 目录信任检查，退出码 1，没有生成事件流或最终回复；第二次加 `--skip-git-repo-check` 后才跑通。
- 本轮只验证新建无业务测试会话，不验证 resume、fork、连续多轮、已有业务会话派发。
- `--json` 事件流目前只验证到 4 行最小事件，不足以直接当完整工作流协议。
- transcript reader 读回有 1 个加密内容省略 warning。
- Codex CLI 执行时出现远程插件 401 和 MCP shutdown warning，未阻塞本轮，但后续长任务要继续观察。

## 本轮完成

- 真实运行一次受控 `codex exec` 测试。
- 创建了一个新的无业务测试会话。
- `--json` 输出机器可读 JSONL 事件流。
- `--output-last-message` 写出最终回复。
- 临时索引能发现新会话。
- transcript reader 能按新 `thread_id` 读回会话，并命中目标文本。

## 关键结果

```text
thread_id: 019e7389-349a-7f02-aa31-a4a90b24e865
rollout_path: /Users/yoyi/.codex/sessions/2026/05/29/rollout-2026-05-29T19-40-32-019e7389-349a-7f02-aa31-a4a90b24e865.jsonl
cwd: /private/tmp/codex-control-probe-v2
```

状态变化：

- threads 数量：318 -> 319。
- session JSONL 文件数：318 -> 319。
- 最终回复文件存在，且包含 `CONTROL_PROBE_OK_2026_05_29`。
- transcript event 数：12。
- bad JSON 行：0。
- unknown event：0。
- sensitive-like event：0。
- encrypted content 省略：1。

## 临时产物

- `/tmp/codex-control-probe-v2/`
- `/tmp/codex-control-probe-v2-events.jsonl`
- `/tmp/codex-control-probe-v2-last-message.txt`
- `/tmp/codex-control-probe-v2-exit-code.txt`
- `/tmp/codex-control-probe-v2-index.json`
- `/tmp/codex-control-probe-v2-transcript.json`

这些没有复制到仓库。

## 写入文件

- `/Users/yoyi/workspace/product-line/evidence/2026-05-29-codex-controlled-real-session-write-probe-v2.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-29-codex-controlled-real-session-write-probe-v2-result.md`

## 验证

- `python3 -m unittest /Users/yoyi/workspace/product-line/prototypes/index-kernel/tests/test_codex_session_control_probe.py`：6 个通过。
- `python3 -m unittest /Users/yoyi/workspace/product-line/prototypes/index-kernel/tests/test_transcript_reader.py`：12 个通过。
- `python3 /Users/yoyi/workspace/product-line/prototypes/index-kernel/build_index.py --check /Users/yoyi/workspace/product-line/prototypes/index-kernel/codex-index.json`：`validation_ok`。

## 禁止事项复核

- 没有读取 `/Users/yoyi/.codex/auth.json`。
- 没有读取 `.env`、授权文件或密钥文件。
- 没有向已有业务会话发送测试消息。
- 没有运行 `codex resume` 或 `codex fork`。
- 没有删除、迁移、归档、重命名会话。
- 没有把完整 transcript 或完整事件流写入仓库。
- 没有改 Tauri / React 前端。
- 没有运行 harness。

## 下一步建议

可以进入“Codex 工作流编排运行模型 v1”。

范围建议收窄成：总指导生成任务、创建或选择执行会话、发送任务 prompt、等待执行结果、读回 transcript、形成回收摘要。先不做复杂 UI、并发调度和多 agent。
