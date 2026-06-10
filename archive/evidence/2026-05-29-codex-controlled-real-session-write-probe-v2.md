# Codex 受控真实会话写入探针 v2 证据

## 薄弱点

- 第一次真实命令没有进入会话写入阶段。依据：在 `/tmp/codex-control-probe-v2` 下未加 `--skip-git-repo-check`，Codex CLI 返回 `Not inside a trusted directory`，退出码为 1，事件流 0 字节，最终回复文件未生成，线程数和 session 文件数仍为 318。
- 第二次命令加了 `--skip-git-repo-check` 才跑通。依据：`codex exec --help` 明确存在该参数；第二次退出码为 0。
- 这次只验证新建无业务测试会话，不验证 resume、fork、连续多轮对话、已有业务会话派发。依据：本轮只运行 `codex exec` 新会话命令，没有运行 `codex resume` 或 `codex fork`。
- `--json` 事件流只有 4 行，适合做最小运行状态判断，但还不够定义完整编排协议。依据：事件类型只有 `thread.started`、`turn.started`、`item.completed`、`turn.completed`。
- transcript reader 读回时出现 1 个加密内容省略 warning。依据：读回 summary 显示 `encrypted_content_event_count=1`，事件 warning 为 `encrypted_content_omitted`。
- 执行过程中 Codex CLI stderr 出现远程插件 catalog 401 和 MCP shutdown warning。依据：命令输出包含 remote plugin sync unauthorized 和 MCP shutdown handshake warning；这些 warning 未阻止本轮闭环，但需要后续诊断是否影响长时间编排。

## 做了什么

- 在用户批准的一次受控测试范围内，真实运行了无业务 `codex exec`。
- 验证 `--json` 能输出机器可读事件流。
- 验证 `--output-last-message` 能写出最终回复。
- 验证 Codex 本地线程数和 session JSONL 文件数增加。
- 生成临时索引，并用 transcript reader 按新 `thread_id` 读回本次测试会话。
- 只记录统计、路径和目标文本命中情况，不写完整事件流或完整 transcript 到仓库。

## 运行命令

第一次命令：

```bash
codex exec --json --output-last-message /tmp/codex-control-probe-v2-last-message.txt "请只回复这一句：CONTROL_PROBE_OK_2026_05_29"
```

结果：失败，退出码 1。原因是 `/tmp/codex-control-probe-v2` 不是可信目录，且未指定 `--skip-git-repo-check`。

第二次命令：

```bash
codex exec --skip-git-repo-check --json --output-last-message /tmp/codex-control-probe-v2-last-message.txt "请只回复这一句：CONTROL_PROBE_OK_2026_05_29"
```

stdout 事件流保存到：

```text
/tmp/codex-control-probe-v2-events.jsonl
```

退出码保存到：

```text
/tmp/codex-control-probe-v2-exit-code.txt
```

## 真实执行结果

- 第二次命令退出码：0。
- `/tmp/codex-control-probe-v2-events.jsonl`：343 字节。
- `/tmp/codex-control-probe-v2-last-message.txt`：27 字节。
- 最终回复文件命中目标文本：是。
- 事件流行数：4。
- 事件类型统计：
  - `thread.started`: 1
  - `turn.started`: 1
  - `item.completed`: 1
  - `turn.completed`: 1

新会话：

```text
thread_id: 019e7389-349a-7f02-aa31-a4a90b24e865
rollout_path: /Users/yoyi/.codex/sessions/2026/05/29/rollout-2026-05-29T19-40-32-019e7389-349a-7f02-aa31-a4a90b24e865.jsonl
cwd: /private/tmp/codex-control-probe-v2
source: exec
model_provider: ai
```

## Codex 本地状态变化

执行前基线：

- `state_5.sqlite` threads 数量：318。
- sessions / archived_sessions JSONL 文件数：318。

执行后：

- `state_5.sqlite` threads 数量：319。
- sessions / archived_sessions JSONL 文件数：319。
- 新增 rollout 文件：`/Users/yoyi/.codex/sessions/2026/05/29/rollout-2026-05-29T19-40-32-019e7389-349a-7f02-aa31-a4a90b24e865.jsonl`。

说明：不把 `max(updated_at_ms)` 当唯一依据。当前总指导会话也可能更新 Codex 状态，可靠依据是线程数、文件数、新 `thread_id` 和新 rollout 路径。

## 临时索引和 transcript 读回

生成临时索引：

```bash
python3 /Users/yoyi/workspace/product-line/prototypes/index-kernel/build_index.py --codex-home /Users/yoyi/.codex --output /tmp/codex-control-probe-v2-index.json
```

结果：

- `thread_count`: 319
- `rollout_checked`: 319
- `rollout_existing`: 319
- `project_count`: 33
- `skill_count`: 51
- `plugin_count`: 11
- `warning_count`: 0

读取新 transcript：

```bash
python3 /Users/yoyi/workspace/product-line/prototypes/index-kernel/transcript_reader.py --index /tmp/codex-control-probe-v2-index.json --thread-id 019e7389-349a-7f02-aa31-a4a90b24e865 --output /tmp/codex-control-probe-v2-transcript.json
```

结果：

- transcript event 数：12。
- JSONL 总行数：12。
- parsed 行数：12。
- bad JSON 行数：0。
- unknown event 数：0。
- sensitive-like event 数：0。
- encrypted content 省略数：1。
- warning：`encrypted_content_event_count:1`。
- 目标文本在 transcript 文本字段中命中：4 次。

## 回归检查

已运行：

```bash
python3 -m unittest /Users/yoyi/workspace/product-line/prototypes/index-kernel/tests/test_codex_session_control_probe.py
python3 -m unittest /Users/yoyi/workspace/product-line/prototypes/index-kernel/tests/test_transcript_reader.py
python3 /Users/yoyi/workspace/product-line/prototypes/index-kernel/build_index.py --check /Users/yoyi/workspace/product-line/prototypes/index-kernel/codex-index.json
```

结果：

- 控制探针测试：6 个通过。
- transcript reader 测试：12 个通过。
- 默认索引 check：`validation_ok`。

## 安全边界

- 没有运行 `codex resume`。
- 没有运行 `codex fork`。
- 没有向现有真实业务会话发送测试消息。
- 没有删除、迁移、归档、重命名 Codex 会话。
- 没有读取 `/Users/yoyi/.codex/auth.json`。
- 没有读取 `.env`。
- 没有读取授权文件或密钥文件。
- 没有把完整 transcript、完整事件流或完整 session JSONL 写入仓库。
- 没有修改 Tauri / React 前端。
- 没有运行 harness。

## 结论

本轮可以接受为“Codex 新建测试会话、发送 prompt、等待结果、最终回复文件、临时索引发现、transcript reader 读回”的最小真实闭环已打通。

不能接受为“完整工作流编排已完成”。还没有验证 resume、多轮对话、任务派发协议、开发线回传、总指导回收、失败重试、并发会话和权限隔离。
