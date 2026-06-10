# Codex 绑定会话受控派发探针 v1 结果交接

## 薄弱点

- 真实 resume 第一次在沙箱内失败，提权后才成功。原因是需要写 `/Users/yoyi/.codex/state_5.sqlite`。
- 提权审批过程额外产生了一个 `codex-auto-review` / guardian 线程，所以线程数增加不能完全归因于目标 resume。
- stdout JSON 事件流没有保存成独立 `events.jsonl` 文件，只能依据工具输出记录事件类型。
- 这只是无业务短 prompt，不证明真实业务长任务、工具调用、权限确认、失败重试或并发调度。

## 做了什么

- 在用户精确批准后，执行真实无业务 `codex exec resume`。
- 使用 v2 的测试会话 `019e7389-349a-7f02-aa31-a4a90b24e865`。
- 发送第二轮无业务 prompt。
- 等待命令完成。
- 验证最终回复文件命中目标文本。
- 生成临时索引。
- 用 transcript reader 读回第二轮内容。
- 更新 evidence / handoff。

## 用户精确批准语句

```text
批准执行 Codex 绑定会话受控派发探针 v1 的真实无业务 resume
```

## 是否执行真实 resume

是。

命令：

```bash
codex exec resume --skip-git-repo-check --json --output-last-message /tmp/codex-bound-session-dispatch-v1/last-message.txt 019e7389-349a-7f02-aa31-a4a90b24e865 "请只回复这一句：BOUND_SESSION_DISPATCH_OK_2026_05_29"
```

第一次沙箱内执行失败：

- 退出码：1。
- 原因：`attempt to write a readonly database`。

提权执行成功：

- 退出码：0。
- 返回目标文本。

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

## supported 依据

`resume_command_shape=supported`：

- `codex exec resume [SESSION_ID] [PROMPT]` 真实执行成功。

`resume_json_events=supported`：

- 工具输出中看到 `thread.started`、`turn.started`、`item.completed`、`turn.completed`。

`resume_output_last_message=supported`：

- `/tmp/codex-bound-session-dispatch-v1/last-message.txt` 存在。
- 文件长度 36。
- 命中 `BOUND_SESSION_DISPATCH_OK_2026_05_29`。

`resume_wait_for_result=supported`：

- 命令退出码为 0。
- 输出 `turn.completed`。

`read_back_second_turn=supported`：

- transcript reader 读回目标测试会话。
- 读回 event 数 22。
- 目标文本命中 4 次。
- unknown event 数 0。
- bad JSON 行数 0。

`safe_for_workflow_dispatch_v1=yes`：

- 已证明绑定的无业务测试会话能接收第二轮 prompt、等待完成、写最终回复、被 transcript reader 读回。
- 仍只适合进入受控工作流派发原型，不适合直接跑真实业务长任务。

## Codex 状态变化

执行前：

```json
{
  "threads": 319,
  "jsonl_files": 319
}
```

执行后：

```json
{
  "threads": 320,
  "jsonl_files": 320
}
```

说明：

- 目标测试会话 rollout 被追加更新，`updated_at_ms=1780064100985`。
- 额外新增的 thread 是提权审批产生的 `codex-auto-review` / guardian 线程，不是目标测试会话。
- 本轮没有读取该 guardian 线程正文。

## 写了哪些文件

项目内：

- `product-line/evidence/2026-05-29-codex-bound-session-dispatch-probe-v1.md`
- `product-line/handoffs/2026-05-29-codex-bound-session-dispatch-probe-v1-result.md`

临时：

- `/tmp/codex-bound-session-dispatch-v1/last-message.txt`
- `/tmp/codex-bound-session-dispatch-v1/index.json`
- `/tmp/codex-bound-session-dispatch-v1/transcript.json`

Codex 自身：

- 目标测试会话 rollout 追加更新。
- 提权审批过程产生一个额外 guardian 线程。

## 是否写了 `/Users/yoyi/.codex`

是。

依据：

- 真实 `codex exec resume` 成功执行需要写 Codex 状态。
- 目标测试会话 rollout 更新。
- 提权审批过程额外产生 guardian 线程。

## 是否触碰真实业务会话

没有向真实业务会话发送测试消息。

使用的是 v2 无业务测试会话：

```text
019e7389-349a-7f02-aa31-a4a90b24e865
```

## 是否读取授权、密钥、`.env`

没有。

本轮没有读取：

- `/Users/yoyi/.codex/auth.json`
- `.env`
- 授权文件
- 密钥文件
- 业务会话正文

## 是否适合下一步进入“工作流节点派发 Codex 指令 v1”

适合进入受控原型。

依据：

- 已验证 `codex exec resume` 能向已有测试会话发送第二轮 prompt。
- 已验证能等待完成。
- 已验证能生成最终回复文件。
- 已验证 transcript reader 能读回第二轮内容。

限制：

- 下一步仍应只使用无业务或明确审核后的工作流指令。
- 真实业务自动执行前，还需要定义派发协议、权限确认、失败重试和回收判断。

## 回收建议

接受为“绑定会话真实无业务 resume 派发闭环通过”。

不要接受为“真实业务工作流自动执行完成”。

