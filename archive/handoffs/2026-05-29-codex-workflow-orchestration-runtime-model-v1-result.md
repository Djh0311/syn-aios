# Codex 工作流编排运行模型 v1 结果交接

## 薄弱点先说

- 本轮只做了 dry-run，没有执行真实无业务 Codex 探针。依据：用户没有给出精确批准语句。
- dry-run 证明结构和状态流，不证明本轮真实创建、发送、等待、读回成功。依据：`worker_thread_id=null`。
- 真实路径有代码和门禁，但未执行，不能回收为真实闭环验证。
- 没有验证 resume、fork、并发、长任务和失败恢复。

## 是否只做 dry-run

只做了 dry-run。

没有执行真实无业务探针。

没有用户明确批准语句。

## 运行了哪些命令

```bash
python3 -m unittest /Users/yoyi/workspace/product-line/prototypes/codex-workflow-runtime/tests/test_workflow_runtime.py
```

结果：6 个测试通过。

```bash
python3 /Users/yoyi/workspace/product-line/prototypes/codex-workflow-runtime/workflow_runtime.py --dry-run --output-dir /tmp/codex-workflow-runtime-v1
```

结果：状态流走到 `accepted`。

```bash
python3 /Users/yoyi/workspace/product-line/prototypes/codex-workflow-runtime/workflow_runtime.py --real-codex-probe --approval-text 批准 --output-dir /tmp/codex-workflow-runtime-v1-denied
```

结果：被门禁拒绝，未执行 Codex。

## 创建了哪些新测试会话

没有创建新测试会话。

dry-run session 占位：

- `worker-session.thread_id=null`
- `worker-session.created_new=false`
- `worker-session.existing_business_session_touched=false`

## 工作流状态流是否完整走到 accepted

dry-run 完整走到 `accepted`。

状态流：

```json
[
  "planned",
  "dispatched",
  "running",
  "reported",
  "recovered",
  "accepted"
]
```

节点状态：

```json
{
  "director_plan": "completed",
  "worker_run": "completed",
  "worker_readback": "completed",
  "director_review": "completed",
  "runtime_summary": "accepted"
}
```

## transcript reader 是否读回执行线结果

没有真实读回。

dry-run 生成了 `worker-transcript.json` 占位，里面标记 `dry_run_no_transcript`。

## 总指导回收意见是否生成

生成了 dry-run 回收意见：

- `/tmp/codex-workflow-runtime-v1/director-review.json`

dry-run 的回收意见用于验证模型结构，不代表真实执行线结果已被读回。

## 写了哪些文件

项目内：

- `product-line/prototypes/codex-workflow-runtime/README.md`
- `product-line/prototypes/codex-workflow-runtime/workflow_runtime.py`
- `product-line/prototypes/codex-workflow-runtime/tests/test_workflow_runtime.py`
- `product-line/evidence/2026-05-29-codex-workflow-orchestration-runtime-model-v1.md`
- `product-line/handoffs/2026-05-29-codex-workflow-orchestration-runtime-model-v1-result.md`

临时目录：

- `/tmp/codex-workflow-runtime-v1/run.json`
- `/tmp/codex-workflow-runtime-v1/director-task.json`
- `/tmp/codex-workflow-runtime-v1/worker-events.jsonl`
- `/tmp/codex-workflow-runtime-v1/worker-last-message.txt`
- `/tmp/codex-workflow-runtime-v1/index.json`
- `/tmp/codex-workflow-runtime-v1/worker-transcript.json`
- `/tmp/codex-workflow-runtime-v1/director-review.json`

## 是否读取授权、密钥或业务会话正文

没有。

本轮没有读取：

- `/Users/yoyi/.codex/auth.json`
- `.env`
- 授权文件
- 密钥文件
- 业务会话正文

本轮没有写：

- `/Users/yoyi/.codex`
- Codex 状态库

## 是否适合进入桌面壳会话管理 UI

适合进入“dry-run run.json 只读展示和 UI 映射”。

不适合直接进入“真实自动派发执行 UI”。

原因：

- 运行模型结构已有。
- 真实执行路径需要另一次明确批准后验证。
- 还没有 resume、多轮、失败恢复和并发能力。

## 回收建议

建议：接受为 dry-run 运行模型 v1。

不要接受为真实 Codex 工作流闭环已完成。

下一步如果继续开发 UI，建议先展示：

- 节点状态。
- 状态流。
- 事件列表。
- session 引用。
- artifact 引用。
- warning。

下一步如果继续能力验证，必须另起真实无业务探针，并要求用户明确写出：

```text
批准执行 Codex 工作流运行模型 v1 的真实无业务探针
```
