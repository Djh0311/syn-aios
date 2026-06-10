# Codex 会话全文读取 v1 证据

## 薄弱点

- 真实样本只验证了 1 个索引内线程，不能证明所有历史 JSONL 形态都覆盖。依据：真实验证只读取 `019e66fb-d5b4-7023-b02a-e3b586e60ad1`。
- 未知事件策略是保留 metadata 并 warning，不是语义完备解析。依据：`transcript_reader.py` 对未识别 `type` / `payload.type` 进入 `unknown`。
- 敏感内容策略当前是检测并打 `sensitive_like_content` warning，不会自动改写用户选择输出文件里的普通正文。依据：任务包最低要求是对疑似密钥、token、Authorization header 打 warning；只有 `payload.encrypted_content` 被强制省略。
- 权限拒绝测试在当前系统可能被跳过。依据：测试里对 `chmod 000` 不触发 `PermissionError` 的环境保留 `skipTest` 分支。
- 当前 `/Users/yoyi/workspace` 不是 git 仓库，不能用 `git diff` 证明写入范围。依据：`git -C product-line status --short` 返回 `fatal: not a git repository`。

## 做了什么

- 新增单会话 transcript 读取入口：`product-line/prototypes/index-kernel/transcript_reader.py`。
- 新增人工夹具测试：`product-line/prototypes/index-kernel/tests/test_transcript_reader.py`。
- 入口按 `codex-index.json` 中的 `thread_id` 找到 `rollout_path`，只读取该单个 JSONL。
- 输出结构化 JSON，不修改默认 `codex-index.json`。
- 保留原始事件顺序，按 JSONL 行号生成稳定 `event_id`。
- 区分用户消息、assistant 消息、工具调用、工具结果、命令输出、turn context、session meta、compacted、unknown。
- 损坏 JSONL 行不会中断读取，会进入顶层 warning。
- `payload.encrypted_content` 不展示原值，只输出存在且已省略的标记。

## 输出结构

顶层字段：

- `thread_id`
- `rollout_path`
- `project_path`
- `title`
- `created_at_ms`
- `updated_at_ms`
- `events`
- `summary`
- `warnings`
- `source_stats`

`events[]` 字段：

- `event_id`
- `timestamp`
- `event_type`
- `actor`
- `role`
- `turn_id`
- `call_id`
- `tool_name`
- `text`
- `arguments`
- `output`
- `stdout`
- `stderr`
- `exit_code`
- `metadata`
- `warnings`

字段调整依据：

- 保留任务包建议字段，并新增 `stdout` / `stderr`，因为命令输出验收要求明确区分 stdout、stderr、exit code。
- `metadata.payload_keys` 保留真实 payload 字段名，便于诊断格式变化，但不把未知事件丢弃。
- `source_stats.raw_type_counts` 和 `source_stats.payload_type_counts` 只保留计数，便于验证覆盖，不暴露正文。

## 测试

已运行：

```bash
python3 -m unittest product-line/prototypes/index-kernel/tests/test_transcript_reader.py
```

结果：

- 12 个测试通过。

已运行：

```bash
python3 -m unittest discover product-line/prototypes/index-kernel/tests
```

结果：

- 38 个测试通过。

覆盖点：

- 正常 transcript 夹具能解析用户消息、assistant 消息、工具调用和工具结果。
- 命令输出能区分 stdout、stderr、exit_code。
- `payload.encrypted_content` 只标记 warning，不输出内容。
- 损坏 JSONL 行产生 warning，其余行继续解析。
- 未知事件类型保留 metadata 并产生 warning。
- 非索引 thread 拒绝。
- `rollout_path` 越界拒绝。
- 缺失 rollout 文件给出明确错误。
- 权限拒绝有测试分支；当前系统不稳定触发时会 skip。
- 默认索引生成不包含全文正文相关字段。

## 真实会话验证

读取的线程：

- `019e66fb-d5b4-7023-b02a-e3b586e60ad1`

输出位置：

- `/tmp/codex-transcript-019e66fb.json`

注意：

- 该输出在 `/tmp`，未写入仓库。
- 本证据不贴完整正文、工具输出正文或命令输出正文。

只记录统计：

```json
{
  "event_count": 867,
  "jsonl_stats": {
    "bad_json_line_count": 0,
    "line_count": 867,
    "parsed_line_count": 867
  },
  "raw_type_counts": {
    "event_msg": 260,
    "response_item": 601,
    "session_meta": 1,
    "turn_context": 5
  },
  "payload_type_counts": {
    "agent_message": 78,
    "custom_tool_call": 39,
    "custom_tool_call_output": 39,
    "function_call": 197,
    "function_call_output": 197,
    "message": 87,
    "null": 6,
    "patch_apply_end": 38,
    "reasoning": 42,
    "task_complete": 5,
    "task_started": 5,
    "token_count": 129,
    "user_message": 5
  },
  "summary": {
    "encrypted_content_event_count": 42,
    "event_type_counts": {
      "assistant_message": 158,
      "command_output": 38,
      "session_meta": 1,
      "system_context": 181,
      "tool_call": 236,
      "tool_result": 236,
      "turn_context": 5,
      "user_message": 12
    },
    "sensitive_like_event_count": 0,
    "total_events": 867,
    "unknown_event_count": 0,
    "warning_count": 43
  },
  "warning_codes": {
    "encrypted_content_event_count": 1,
    "encrypted_content_omitted": 42
  }
}
```

## 默认索引未加入全文正文

验证命令：

```bash
python3 product-line/prototypes/index-kernel/build_index.py --check product-line/prototypes/index-kernel/codex-index.json
```

结果：

- `validation_ok`

额外字段检查结果：

```json
{
  "forbidden_project_keys_present": [],
  "forbidden_thread_keys_present": [],
  "thread_count": 310,
  "thread_keys": [
    "archived",
    "created_at_ms",
    "has_user_event",
    "model",
    "model_provider",
    "project_root",
    "reasoning_effort",
    "rollout_exists",
    "rollout_path",
    "thread_id",
    "thread_source",
    "title",
    "tokens_used",
    "updated_at_ms",
    "warnings"
  ]
}
```

结论：默认 `codex-index.json` 没有 `events`、`transcript`、`text`、`arguments`、`output`、`stdout`、`stderr`、`exit_code` 等全文或工具输出字段。依据是上面的字段检查。

## 写入边界

实际写入仓库文件：

- `product-line/prototypes/index-kernel/transcript_reader.py`
- `product-line/prototypes/index-kernel/tests/test_transcript_reader.py`
- `product-line/evidence/2026-05-29-codex-session-full-transcript-v1.md`
- `product-line/handoffs/2026-05-29-codex-session-full-transcript-v1-result.md`

真实 transcript 只写到 `/tmp/codex-transcript-019e66fb.json` 用于验证，没有写入 evidence、handoff 或 `codex-index.json`。

没有写 `/Users/yoyi/.codex` 或 Codex 状态库的依据：

- 实现代码没有任何写 `/Users/yoyi/.codex` 的路径。
- 读取入口只打开 `--index` 和索引内单个 `rollout_path`，输出只写 `--output`。
- 本轮没有运行会写 Codex 状态库的命令。
- `build_index.py --check` 只校验现有索引，不生成或写入 Codex 状态。

补充说明：

- 本轮曾列出过 `/Users/yoyi/.codex/auth.json` 的路径，但没有读取其内容；这个动作不作为安全验证依据。

## 仍不确定的事件类型

- 未在本真实样本中出现的历史或新版本事件类型仍不确定。
- `compacted` 在真实样本中未出现，只由人工夹具覆盖。
- 损坏 JSONL 行由人工夹具覆盖，真实样本没有坏行。
- 权限拒绝在当前文件系统上可能无法稳定复现。

## 下一步判断

适合进入下一步“Codex 会话控制能力探针 v1”。

依据：

- 已能按索引内 `thread_id` 读取单条会话完整 JSONL。
- 已能生成结构化 transcript，且不把全文塞进默认索引。
- 已有 unknown、坏行、越界、缺文件、加密内容省略、敏感疑似 warning 的基本防线。

保留风险：

- 事件类型覆盖率仍依赖更多真实样本验证。
- 输出文件本身可能包含用户选择会话正文，因此后续 UI 展示必须加权限提示、折叠和敏感 warning 展示。
