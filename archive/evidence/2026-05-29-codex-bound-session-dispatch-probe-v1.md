# Codex 绑定会话受控派发探针 v1 证据

## 薄弱点

- 真实 resume 需要提权才跑通。依据：第一次在沙箱内执行时报 `attempt to write a readonly database` 和 `Operation not permitted`，未写入成功。
- 提权审批过程额外产生了一个 `codex-auto-review` / guardian 线程，所以 `.codex` 线程数从 319 到 320 不能直接归因给目标 resume。依据：临时索引显示新增线程 `019e7416-7d30-7553-a077-3c61b6a829d8`，`model=codex-auto-review`，`source` 为 subagent guardian。
- 目标测试会话没有新建第二个 thread，而是在 v2 测试会话原 rollout 上追加更新。依据：目标 `thread_id=019e7389-349a-7f02-aa31-a4a90b24e865` 的 rollout 路径仍是 `rollout-2026-05-29T19-40-32-019e7389-349a-7f02-aa31-a4a90b24e865.jsonl`，`updated_at_ms` 更新为 `1780064100985`。
- 这轮 stdout 事件流没有重定向保存成 `/tmp/.../events.jsonl`。依据：命令直接通过工具输出返回；因此本证据只能记录工具输出中看到 `thread.started`、`turn.started`、`item.completed`、`turn.completed`，不能声称已保存事件流文件。
- 这只是无业务短 prompt resume，不能证明长任务、工具调用、权限确认、失败重试或并发调度稳定。依据：测试 prompt 只要求回复固定文本。

## 做了什么

- 在用户精确批准后执行真实无业务 `codex exec resume`。
- 先在沙箱内执行一次，确认会被 `.codex` 写入权限挡住。
- 使用提权执行同一条无业务 resume 命令。
- 验证 `--output-last-message` 写出最终回复。
- 生成临时索引。
- 用 transcript reader 读回目标测试会话。
- 只记录统计、路径和目标文本命中情况，不把完整 transcript 或完整 session JSONL 写入仓库。

## 用户精确批准语句

```text
批准执行 Codex 绑定会话受控派发探针 v1 的真实无业务 resume
```

## 前置测试会话

使用 v2 生成的无业务测试会话：

```text
thread_id: 019e7389-349a-7f02-aa31-a4a90b24e865
rollout_path: /Users/yoyi/.codex/sessions/2026/05/29/rollout-2026-05-29T19-40-32-019e7389-349a-7f02-aa31-a4a90b24e865.jsonl
project_root: /private/tmp/codex-control-probe-v2
```

没有向真实业务会话发送测试消息。

## 执行前统计

```json
{
  "threads": 319,
  "jsonl_files": 319
}
```

## 第一次真实命令

在沙箱内执行：

```bash
codex exec resume --skip-git-repo-check --json --output-last-message /tmp/codex-bound-session-dispatch-v1/last-message.txt 019e7389-349a-7f02-aa31-a4a90b24e865 "请只回复这一句：BOUND_SESSION_DISPATCH_OK_2026_05_29"
```

结果：

- 退出码：1。
- 未完成真实 resume。
- 报错：`attempt to write a readonly database`。
- 报错：`failed to initialize in-process app-server client: Operation not permitted`。

判断：

- 真实 resume 确实需要写 `/Users/yoyi/.codex`。
- 沙箱内不能完成该探针。

## 提权真实命令

在用户已经给出精确批准后，使用提权执行同一命令：

```bash
codex exec resume --skip-git-repo-check --json --output-last-message /tmp/codex-bound-session-dispatch-v1/last-message.txt 019e7389-349a-7f02-aa31-a4a90b24e865 "请只回复这一句：BOUND_SESSION_DISPATCH_OK_2026_05_29"
```

结果：

- 退出码：0。
- 工具输出中出现 `thread.started`，thread id 为 `019e7389-349a-7f02-aa31-a4a90b24e865`。
- 工具输出中出现 `turn.started`。
- 工具输出中出现 `item.completed`，文本为目标固定回复。
- 工具输出中出现 `turn.completed`。
- stderr 有 remote plugin catalog 401 和 MCP shutdown warning，未阻止本轮完成。

## 最终回复文件

路径：

```text
/tmp/codex-bound-session-dispatch-v1/last-message.txt
```

检查结果：

```json
{
  "exists": true,
  "contains_target": true,
  "length": 36
}
```

`wc -c` 结果：

```text
36 /tmp/codex-bound-session-dispatch-v1/last-message.txt
```

## 执行后统计

```json
{
  "threads": 320,
  "jsonl_files": 320
}
```

注意：

- 线程数增加 1 不能归因给目标 resume。
- 额外线程来自提权审批的 `codex-auto-review` / guardian 线程。
- 目标测试会话本身是原 rollout 被追加更新。

## 目标测试会话元数据

临时索引中目标测试会话：

```json
{
  "thread_id": "019e7389-349a-7f02-aa31-a4a90b24e865",
  "title": "请只回复这一句：CONTROL_PROBE_OK_2026_05_29",
  "project_root": "/private/tmp/codex-control-probe-v2",
  "rollout_path": "/Users/yoyi/.codex/sessions/2026/05/29/rollout-2026-05-29T19-40-32-019e7389-349a-7f02-aa31-a4a90b24e865.jsonl",
  "created_at_ms": 1780054832282,
  "updated_at_ms": 1780064100985,
  "model_provider": "ai",
  "model": "gpt-5.5"
}
```

## 额外 guardian 线程

临时索引显示本轮还出现一个额外线程：

```text
thread_id: 019e7416-7d30-7553-a077-3c61b6a829d8
model: codex-auto-review
project_root: /Users/yoyi/workspace
```

判断：

- 这是提权审批 / auto-review 产生的线程，不是目标 resume 会话。
- 本轮没有读取该线程正文。
- 本证据不记录该线程 title 或 transcript 内容。

## 临时索引

命令：

```bash
python3 product-line/prototypes/index-kernel/build_index.py --codex-home /Users/yoyi/.codex --output /tmp/codex-bound-session-dispatch-v1/index.json
```

结果：

```json
{
  "thread_count": 320,
  "rollout_checked": 320,
  "rollout_existing": 320,
  "project_count": 33,
  "skill_count": 51,
  "plugin_count": 11,
  "warning_count": 0
}
```

## transcript 读回

命令：

```bash
python3 product-line/prototypes/index-kernel/transcript_reader.py --index /tmp/codex-bound-session-dispatch-v1/index.json --thread-id 019e7389-349a-7f02-aa31-a4a90b24e865 --output /tmp/codex-bound-session-dispatch-v1/transcript.json
```

结果：

```json
{
  "thread_id": "019e7389-349a-7f02-aa31-a4a90b24e865",
  "event_count": 22,
  "bad_json_line_count": 0,
  "unknown_event_count": 0,
  "warning_count": 3
}
```

统计摘要：

```json
{
  "target_hits": 4,
  "encrypted_content_event_count": 2,
  "sensitive_like_event_count": 0,
  "total_events": 22,
  "payload_type_counts": {
    "agent_message": 2,
    "message": 7,
    "null": 3,
    "reasoning": 2,
    "task_complete": 2,
    "task_started": 2,
    "token_count": 2,
    "user_message": 2
  }
}
```

判断：

- 第二轮目标文本已被 transcript reader 读回。
- 目标文本命中 4 次。
- 加密内容仍按既有规则省略。
- 没有未知事件。
- 没有疑似敏感事件。

## 能力矩阵

```json
{
  "resume_command_shape": "supported",
  "resume_json_events": "supported",
  "resume_output_last_message": "supported",
  "resume_wait_for_result": "supported",
  "read_back_second_turn": "supported",
  "safe_for_workflow_dispatch_v1": "yes"
}
```

依据：

- `resume_command_shape=supported`：`codex exec resume [SESSION_ID] [PROMPT]` 真实执行成功。
- `resume_json_events=supported`：工具输出中看到 JSON 事件 `thread.started`、`turn.started`、`item.completed`、`turn.completed`。
- `resume_output_last_message=supported`：`last-message.txt` 存在并包含目标文本。
- `resume_wait_for_result=supported`：命令退出码为 0，并返回 `turn.completed`。
- `read_back_second_turn=supported`：transcript reader 读回目标文本。
- `safe_for_workflow_dispatch_v1=yes`：可进入下一步无业务或受控工作流派发原型，但仍不代表可直接跑真实业务长任务。

## 写入边界

写入了：

- 目标测试会话 rollout 追加更新。
- `/tmp/codex-bound-session-dispatch-v1/last-message.txt`
- `/tmp/codex-bound-session-dispatch-v1/index.json`
- `/tmp/codex-bound-session-dispatch-v1/transcript.json`
- `product-line/evidence/2026-05-29-codex-bound-session-dispatch-probe-v1.md`
- `product-line/handoffs/2026-05-29-codex-bound-session-dispatch-probe-v1-result.md`

没有写：

- 工作台真实 workflow state。
- Tauri / React 前端。
- 项目业务目录。

## 安全边界

本轮没有：

- 向真实业务会话发送测试消息。
- 运行 `codex fork`。
- 删除、迁移、归档、重命名 Codex 会话。
- 读取 `/Users/yoyi/.codex/auth.json`。
- 读取 `.env`。
- 读取授权文件或密钥文件。
- 读取业务会话正文。
- 写完整 transcript 到仓库。
- 写完整事件流到仓库。
- 运行 harness。

## 结论

可以接受为“Codex 绑定测试会话真实 resume 派发、等待、最终回复、transcript 读回”的最小闭环已打通。

不能接受为“真实业务工作流自动执行已完成”。还没有验证：

- 真实业务会话派发。
- 总指导生成实际任务。
- 执行线执行开发。
- 执行结果回收。
- 权限确认。
- 长任务稳定性。
- 失败重试。
- 并发调度。

