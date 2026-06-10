# 任务包：工作流协议空队列真实落账 v1

## 任务名

工作流协议空队列真实落账 v1。

## 所属开发线

桌面应用线 / 总指导线。

验证线按需复核。

## 当前判断

工作流可控执行协议 v1 已接受为协议能力完成，但真实 workflow state 里还没有落 `workflow_execution_controls[]`、`permission_requests[]`、`execution_attempts[]`。

下一步先把协议空队列写入真实 workflow state，比直接做真实业务试跑更稳。

依据：

- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-controlled-execution-protocol-v1-incident-guardrail-review.md`
- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-workflow-controlled-execution-protocol-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-controlled-execution-protocol-v1-result.md`

## 薄弱点

- 这一步只写协议空队列，不证明真实业务自动编排。
- 空队列只验证真实状态承载方式，不验证长任务稳定性。
- 写真实 workflow state 需要用户明确确认、备份和审计。

## 目标

在真实 workflow state 中补齐协议空队列和审计口径：

1. 写入或初始化 `workflow_execution_controls[]`。
2. 写入或初始化 `permission_requests[]`。
3. 写入或初始化 `execution_attempts[]`。
4. 写入审计事件，说明这是协议空队列初始化。
5. 写入前备份真实 workflow state。
6. 写入后只读复核字段存在且为空队列或安全初始值。

大白话目标：

让真实账本先有地方放长任务、权限、失败、重试、超时、取消记录，但不跑任何真实任务。

## 非目标

- 不执行真实业务任务。
- 不执行 `codex exec resume`。
- 不执行任何 `codex exec`。
- 不发送 Codex 消息。
- 不发送新的 safe probe。
- 不写 `/Users/yoyi/.codex`。
- 不读取完整 transcript。
- 不读取授权、密钥、`.env`、token。
- 不运行 harness。
- 不把空队列初始化说成真实业务自动工作流完成。

## 允许读取

允许读取项目内：

- `/Users/yoyi/workspace/product-line/CURRENT.md`
- `/Users/yoyi/workspace/product-line/tasks/README.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-controlled-execution-protocol-v1-incident-guardrail-review.md`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/lib/types.ts`

允许只读真实 workflow state 的必要结构：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`

禁止读取：

- `/Users/yoyi/.codex/auth.json`
- `.env`
- 密钥、token、授权文件内容
- 完整 transcript 正文

## 允许写入

用户明确确认后，允许写真实 workflow state：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/`

允许写项目内 evidence / handoff：

- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-workflow-protocol-empty-queues-real-state-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-protocol-empty-queues-real-state-v1-result.md`

如实现需要补桌面壳命令，允许写：

- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/tests/`

## 禁止事项

- 禁止执行 `codex exec resume`。
- 禁止执行任何 `codex exec`。
- 禁止发送任何消息到 Codex 会话。
- 禁止写 `/Users/yoyi/.codex`。
- 禁止读取完整 transcript。
- 禁止读取授权、密钥、`.env`、token。
- 禁止运行 harness。
- 禁止在 shell 双引号里写未转义反引号模式；搜索包含反引号的文本时必须使用单引号或 `rg -F`。

## 建议写入结构

如果字段缺失，初始化为：

```json
{
  "workflow_execution_controls": [],
  "permission_requests": [],
  "execution_attempts": []
}
```

建议审计事件：

- `event_type`: `workflow_protocol_empty_queues_initialized`
- `actor_ref`: `director_confirmed_desktop_shell`
- `source_kind`: `workspace_state`
- `permission_level`: `user_confirmed_write`
- `reason`: `用户确认初始化工作流可控执行协议空队列；没有发送 Codex 消息。`

## 验收标准

必须满足：

- 写入前备份真实 workflow state。
- `workflow_execution_controls[]` 存在。
- `permission_requests[]` 存在。
- `execution_attempts[]` 存在。
- 写入审计事件。
- 不执行 `codex exec resume`。
- 不执行任何 `codex exec`。
- 不写 `/Users/yoyi/.codex`。
- 不读取完整 transcript。

建议验证命令：

```bash
python3 prototypes/index-kernel/build_index.py --check prototypes/index-kernel/codex-index.json
rg -F 'codex exec resume' tasks evidence handoffs CURRENT.md
```

安全搜索要求：

- 搜索固定文本使用 `rg -F '固定文本' ...`。
- 搜索包含反引号的文本必须使用单引号或 `rg -F`。
- 禁止用 shell 双引号包住未转义反引号。

## 必须回传

回传必须包含：

1. 薄弱点。
2. 做了什么。
3. 是否写真实 workflow state。
4. 是否写 `/Users/yoyi/.codex`。
5. 是否执行 `codex exec resume` 或任何 `codex exec`。
6. 是否发送 Codex 消息。
7. 写入字段类型，不打印完整状态。
8. 审计事件 id。
9. 备份路径。
10. 新增 evidence / handoff。
11. 验证命令和结果。
12. 下一步建议。

## 总指导回收重点

总指导回收时必须判断：

- 是否只写协议空队列，没有执行真实业务任务。
- 是否没有触碰 `/Users/yoyi/.codex`。
- 是否没有执行 `codex exec resume`。
- 是否字段和审计事件可被后续真实业务小步试跑复用。
