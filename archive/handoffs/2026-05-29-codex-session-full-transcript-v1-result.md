# Codex 会话全文读取 v1 结果交接

## 薄弱点先说

- 只用 1 个真实线程做了受限验证，不能断言覆盖所有 Codex JSONL 事件形态。依据：真实验证线程是 `019e66fb-d5b4-7023-b02a-e3b586e60ad1`。
- 未知事件不会被语义理解，只会保留诊断 metadata 并打 warning。依据：`transcript_reader.py` 的 `set_unknown_event`。
- 疑似密钥、token、Authorization header 目前只打 warning，不自动清洗普通正文。依据：任务包最低要求是 warning；加密字段另行省略。
- 当前没有 git 仓库，不能给出 git diff 级别的写入证明。依据：`git -C product-line status --short` 返回不是 git 仓库。

## 做了什么

- 新增单会话 transcript 读取入口。
- 新增 transcript 夹具测试。
- 按索引里的 `thread_id` 找到 `rollout_path`，只读取单个 JSONL。
- 输出结构化 transcript，保留事件顺序。
- 区分用户消息、assistant 消息、工具调用、工具结果、命令输出、turn context、session meta、compacted、unknown。
- 损坏 JSONL 行不会中断读取。
- `payload.encrypted_content` 不解析、不展示，只标记存在并省略。

## 改了哪些文件

- `product-line/prototypes/index-kernel/transcript_reader.py`
- `product-line/prototypes/index-kernel/tests/test_transcript_reader.py`
- `product-line/evidence/2026-05-29-codex-session-full-transcript-v1.md`
- `product-line/handoffs/2026-05-29-codex-session-full-transcript-v1-result.md`

## 新增测试

新增 12 个 transcript 测试，覆盖：

- 正常用户消息、assistant 消息、工具调用、工具结果。
- stdout、stderr、exit_code 拆分。
- `payload.encrypted_content` 省略。
- 损坏 JSONL 行 warning。
- 未知事件 metadata 保留。
- 非索引 thread 拒绝。
- rollout 路径越界拒绝。
- 缺失 rollout 文件报错。
- 权限拒绝分支。
- CLI stdout 只输出统计，不打印正文。
- 默认 `codex-index.json` 不含全文字段。

验证命令：

```bash
python3 -m unittest product-line/prototypes/index-kernel/tests/test_transcript_reader.py
python3 -m unittest discover product-line/prototypes/index-kernel/tests
python3 product-line/prototypes/index-kernel/build_index.py --check product-line/prototypes/index-kernel/codex-index.json
```

结果：

- transcript 测试：12 个通过。
- index-kernel 测试：38 个通过。
- index check：`validation_ok`。

## transcript 输出结构

顶层：

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

事件：

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

## 真实会话验证

做了真实会话验证。

读取线程：

- `019e66fb-d5b4-7023-b02a-e3b586e60ad1`

只记录统计：

- 总事件数：867。
- JSONL 行数：867。
- 坏 JSON 行：0。
- 顶层类型计数：`response_item` 601，`event_msg` 260，`turn_context` 5，`session_meta` 1。
- 输出事件类型计数：assistant 消息 158，用户消息 12，工具调用 236，工具结果 236，命令输出 38，系统上下文 181，turn context 5，session meta 1。
- 未知事件：0。
- 加密内容省略事件：42。
- 疑似敏感内容事件：0。

没有贴完整真实正文，没有贴工具输出正文。

## 写入和安全证明

- 没有写 `/Users/yoyi/.codex`。依据：实现只读索引和 rollout，输出只写 `--output` 指定路径；本轮写入仓库文件只在 `product-line/`。
- 没有写 Codex 状态库。依据：没有运行写 SQLite 或 Codex CLI 的命令；`build_index.py --check` 只是校验索引。
- 默认 `codex-index.json` 没有加入全文字段。依据：字段检查显示 thread/project 中没有 `events`、`transcript`、`text`、`arguments`、`output`、`stdout`、`stderr`、`exit_code`。
- 真实 transcript 输出在 `/tmp/codex-transcript-019e66fb.json`，没有写进 evidence、handoff 或 `codex-index.json`。

## 仍不确定

- 全部真实 JSONL 事件类型是否覆盖。
- Codex 后续版本是否新增 payload 结构。
- 长会话性能上限。
- 权限拒绝测试在当前系统是否稳定触发。

## 回收建议

建议：接受。

依据：

- 满足按 `thread_id` 读取单个会话全文的最小能力。
- 满足不把全文塞进默认索引。
- 满足坏行、未知事件、越界、缺文件、加密字段省略的基础安全和容错要求。
- 测试通过。

适合进入下一步“Codex 会话控制能力探针 v1”。

前置提醒：

- 下一步不要直接做假对话 UI。
- 先探针确认创建、恢复、发送、等待和读取回复的可行入口。
- UI 展示 transcript 时必须显示 sensitive warning，并对长正文和工具输出做折叠。
