# 任务包：派发结果读回 UI 与总指导回收记录 v1

## 任务名

派发结果读回 UI 与总指导回收记录 v1。

## 所属开发线

桌面应用线 / 总指导线。

验证线按需复核。

## 当前判断

真实工作流节点 safe probe 已跑通一次。

下一步不应急着做真实业务派发，而应先把“派发结果如何被用户看见、如何被总指导回收、如何落 review 账”补上。

依据：

- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-node-safe-probe-real-confirmed-dispatch-v1-final-review.md`
- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-workflow-node-safe-probe-real-confirmed-dispatch-v1.md`
- 真实 workflow state 中 work item 已进入 `ready_for_review`

## 薄弱点

- 现在 workflow state 里有派发记录，但 UI 和总指导回收记录还没有成为明确的一等流程。
- 如果不补回收记录，下一步很容易只看 handoff，人为判断，不能形成后续自动工作流依据。
- 本任务不能再派发新 Codex 指令，否则会和结果回收混在一起。

## 目标

让用户能在工作台中看到这次派发结果，并让总指导回收意见落到 workflow state：

1. UI 显示最近一次 dispatch 的状态、最终回复摘要、transcript 统计、warnings。
2. UI 显示 work item 当前为 `ready_for_review`。
3. 支持写入一条总指导 review 记录，结论为接受 / 需要修改 / 暂停 / 废弃。
4. 总指导 review 记录引用 dispatch id 和 work item id。
5. 写入 audit event。
6. 不发送任何 Codex 消息。

大白话目标：

派出去、收回来以后，工作台要能看见，也要能把总指导判断记到账本里。

## 非目标

- 不执行 `codex exec resume`。
- 不发送 safe probe。
- 不执行真实业务任务。
- 不写 `/Users/yoyi/.codex`。
- 不读取完整 transcript。
- 不读取授权、密钥、`.env`。
- 不运行 harness。
- 不做长任务、权限确认、失败重试、超时取消。
- 不做用户审核业务指令 prompt/schema。

## 允许读取

允许读取项目内：

- `/Users/yoyi/workspace/product-line/CURRENT.md`
- `/Users/yoyi/workspace/product-line/tasks/README.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-node-safe-probe-real-confirmed-dispatch-v1-final-review.md`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/tests/`

允许读取真实 workflow state 必要结构：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`

禁止读取：

- `/Users/yoyi/.codex/auth.json`
- `.env`
- 密钥、token、授权文件内容
- 完整 transcript 正文

## 允许写入

允许写入项目内：

- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/tests/`
- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-dispatch-result-readback-ui-and-director-review-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-dispatch-result-readback-ui-and-director-review-v1-result.md`

用户确认后，允许写真实 workflow state：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/`

## 禁止事项

- 禁止执行 `codex exec resume`。
- 禁止发送任何消息到 Codex 会话。
- 禁止写 `/Users/yoyi/.codex`。
- 禁止读取完整 transcript。
- 禁止读取授权、密钥或 `.env`。
- 禁止运行 harness。
- 禁止把 safe probe 结果说成真实业务自动工作流完成。

## 建议数据模型

如现有 `reviews[]` 足够，复用它。

建议 review 字段：

- `review_id`
- `project_id`
- `workflow_id`
- `work_item_id`
- `dispatch_id`
- `reviewer_role`
- `decision`
- `summary`
- `evidence_refs`
- `handoff_refs`
- `created_at`
- `updated_at`
- `warnings`

建议 decision：

- `accepted`
- `needs_changes`
- `paused`
- `discarded`

建议 audit event：

- `workflow_dispatch_director_review_recorded`

## 验收标准

必须满足：

- UI 能显示 completed dispatch 摘要。
- UI 能显示 transcript 统计和 warnings。
- UI 能显示 work item 为 `ready_for_review`。
- 总指导 review 可以写入 workflow state。
- review 记录引用 dispatch id 和 work item id。
- 写入 audit event。
- 不执行 `codex exec resume`。
- 不写 `/Users/yoyi/.codex`。
- 不读取完整 transcript。

建议验证命令：

```bash
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --offline
```

## 必须回传

回传必须包含：

1. 薄弱点。
2. 做了什么。
3. 改了哪些文件。
4. 是否写真实 workflow state。
5. 是否写 `/Users/yoyi/.codex`。
6. 是否执行 `codex exec resume`。
7. 写入了哪些字段类型，不打印完整状态正文。
8. 新增 evidence / handoff。
9. 测试命令和结果。
10. 下一步建议。

## 总指导回收动作

总指导回收时必须判断：

- 接受。
- 需要修改。
- 暂停。
- 废弃。

通过后再考虑：

- 长任务、权限确认、失败重试、超时取消最小协议。
- 用户审核业务指令 prompt/schema。
