# Codex 工作流编排运行模型 v1 证据

## 薄弱点

- 本轮只执行了 dry-run，没有执行真实无业务 Codex 探针。依据：用户没有在本任务中给出精确批准语句 `批准执行 Codex 工作流运行模型 v1 的真实无业务探针`。
- dry-run 能证明运行模型结构和状态流，不证明 Codex CLI 在本轮真实创建了会话。依据：`worker-session.thread_id=null`，`created_new=false`。
- 真实路径虽然已实现门禁和命令流程，但没有在本轮执行，所以不能把它回收为已验证。依据：`real_codex_probe_not_executed` warning。
- 本轮不验证 resume、fork、并发、长任务或失败恢复。依据：任务包非目标明确排除这些项。

## 做了什么

- 新增独立原型目录：`product-line/prototypes/codex-workflow-runtime/`。
- 新增运行模型脚本：`workflow_runtime.py`。
- 新增 README：`README.md`。
- 新增测试：`tests/test_workflow_runtime.py`。
- 实现 `--dry-run`：不运行 Codex CLI，只生成结构化 run、节点、边、事件、会话占位和产物引用。
- 实现 `--real-codex-probe` 的严格门禁：没有精确批准语句时直接失败，不执行 Codex。
- dry-run 输出到 `/tmp/codex-workflow-runtime-v1/`。

## 运行命令

已运行 dry-run：

```bash
python3 /Users/yoyi/workspace/product-line/prototypes/codex-workflow-runtime/workflow_runtime.py --dry-run --output-dir /tmp/codex-workflow-runtime-v1
```

结果摘要：

```json
{
  "artifact_count": 7,
  "mode": "dry-run",
  "node_statuses": {
    "director_plan": "completed",
    "director_review": "completed",
    "runtime_summary": "accepted",
    "worker_readback": "completed",
    "worker_run": "completed"
  },
  "state_flow": [
    "planned",
    "dispatched",
    "running",
    "reported",
    "recovered",
    "accepted"
  ],
  "status": "accepted",
  "warning_count": 2,
  "worker_thread_id": null
}
```

已运行真实探针门禁负例：

```bash
python3 /Users/yoyi/workspace/product-line/prototypes/codex-workflow-runtime/workflow_runtime.py --real-codex-probe --approval-text 批准 --output-dir /tmp/codex-workflow-runtime-v1-denied
```

结果：

```text
workflow_runtime_failed:real_probe_not_approved:real probe requires exact approval text: 批准执行 Codex 工作流运行模型 v1 的真实无业务探针
```

没有运行真实 Codex 探针。

## 测试

已运行：

```bash
python3 -m unittest /Users/yoyi/workspace/product-line/prototypes/codex-workflow-runtime/tests/test_workflow_runtime.py
```

结果：

- 6 个测试通过。

测试覆盖：

- dry-run 能生成 accepted 状态流和产物文件。
- `run.json` 包含必需运行区段。
- dry-run 不创建真实 session 引用。
- `--real-codex-probe` 需要精确批准语句。
- 总指导回收意见能识别完成标记。
- worker prompt 是无业务测试 prompt，包含要求完成标记。

## run.json 结构

`run.json` 顶层包含：

- `run_id`
- `created_at_ms`
- `workflow_version`
- `mode`
- `goal`
- `status`
- `state_flow`
- `nodes`
- `edges`
- `events`
- `sessions`
- `artifacts`
- `warnings`

节点包含：

- `node_id`
- `node_type`
- `role`
- `status`
- `started_at_ms`
- `ended_at_ms`
- `input_ref`
- `output_ref`
- `session_ref`
- `warnings`

事件包含：

- `event_id`
- `event_type`
- `node_id`
- `timestamp_ms`
- `summary`
- `artifact_refs`

## dry-run 状态流

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

事件类型：

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

## 产物

dry-run 生成：

- `/tmp/codex-workflow-runtime-v1/run.json`
- `/tmp/codex-workflow-runtime-v1/director-task.json`
- `/tmp/codex-workflow-runtime-v1/worker-events.jsonl`
- `/tmp/codex-workflow-runtime-v1/worker-last-message.txt`
- `/tmp/codex-workflow-runtime-v1/index.json`
- `/tmp/codex-workflow-runtime-v1/worker-transcript.json`
- `/tmp/codex-workflow-runtime-v1/director-review.json`

项目内写入：

- `product-line/prototypes/codex-workflow-runtime/README.md`
- `product-line/prototypes/codex-workflow-runtime/workflow_runtime.py`
- `product-line/prototypes/codex-workflow-runtime/tests/test_workflow_runtime.py`
- `product-line/evidence/2026-05-29-codex-workflow-orchestration-runtime-model-v1.md`
- `product-line/handoffs/2026-05-29-codex-workflow-orchestration-runtime-model-v1-result.md`

## 安全边界

本轮没有：

- 运行真实 Codex CLI 会话创建。
- 运行 `codex resume`。
- 运行 `codex fork`.
- 向现有真实业务会话发送 prompt。
- 读取 `auth.json`。
- 读取 `.env`。
- 读取授权文件或密钥文件。
- 读取业务会话正文。
- 写 `/Users/yoyi/.codex`。
- 改 Codex 状态库。
- 删除、迁移、归档、重命名任何会话。
- 运行 harness。
- 修改 Tauri / React 前端。

## 回收判断

建议接受为 dry-run 运行模型 v1。

不建议接受为真实 Codex 工作流闭环已验证。

是否适合进入桌面壳会话管理 UI：

- 适合进入“基于 dry-run run.json 的只读 UI 映射设计或展示”。
- 不适合直接进入“真实自动派发和执行 UI”。

下一步建议：

- 若要验证真实闭环，需要用户在新任务中明确写出精确批准语句。
- 若继续 UI，可以先把 `run.json` 映射到项目工作流节点、事件列表、session 引用和 artifact 面板。
