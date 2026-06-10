# 任务包：工作流节点 safe probe 真实确认派发 v1

## 任务名

工作流节点 safe probe 真实确认派发 v1。

## 所属开发线

桌面应用线 / 工作流运行线。

## 当前判断

本任务只验证“真实工作台工作流节点能向已绑定 Codex 测试会话派发一条无业务探针，并写回真实工作流状态”。

本任务不等于真实业务自动工作流完成。

依据：

- `product-line/CURRENT.md`
- `product-line/STAGE_PLAN.md`
- `product-line/tasks/README.md`
- `product-line/archive/handoffs/2026-05-29-desktop-shell-workflow-node-dispatch-codex-instruction-v1-review.md`

## 薄弱点

- 之前只完成了桌面派发代码路径，没有执行真实桌面 safe probe。依据：回收意见明确写了未执行真实 `codex exec resume`。
- 之前没有写真实 `/Users/yoyi/.codex`。依据：回收意见明确写了 `Wrote /Users/yoyi/.codex: no`。
- 之前没有写真实 workflow state。依据：回收意见明确写了只用测试临时状态。
- 当前还没有长任务、权限确认、失败重试、超时取消和总指导自动回收。依据：`CURRENT.md` 当前未完成列表。
- 本任务会触碰真实 Codex 会话存储，所以必须先获得用户对本次无业务探针的明确批准。

## 背景

当前已经完成：

- 单会话 transcript 读取 v1。
- 无业务 `codex exec` 新建测试会话并读回。
- 无业务 `codex exec resume` 向绑定测试会话派发第二轮 prompt 并读回。
- 工作流最小状态流转。
- 工作流节点绑定已有 Codex 会话。
- 桌面工作流节点派发 Codex 指令 v1 代码路径。
- 前端 safe probe 预览、确认弹层和最近派发摘要。

但这些还没有从真实工作台状态执行一次完整 safe probe 派发。因此下一步必须先补这个缺口。

## 目标

在用户明确批准后，从真实工作台状态执行一次无业务 safe probe 派发：

1. 使用真实工作台状态中的已绑定测试会话。
2. 通过工作流节点派发入口生成 safe probe 预览。
3. 弹出确认信息，明确说明会写 `/Users/yoyi/.codex` 和真实 workflow state。
4. 用户确认后调用绑定会话的 `codex exec resume`。
5. 等待命令完成，保存最终回复摘要。
6. 用 transcript reader 读回目标会话统计。
7. 把工作项状态推进到 `ready_for_review`。
8. 写入派发记录和审计事件。
9. 输出 evidence 和 handoff，供总指导回收。

大白话目标：

证明“工作流节点真的能派出去、收回来、写账本”。

## 非目标

- 不执行真实业务任务。
- 不运行 harness。
- 不做并发调度。
- 不做失败重试。
- 不做超时取消。
- 不做权限确认队列。
- 不做总指导自动回收。
- 不创建新的业务会话。
- 不触碰真实业务会话。
- 不修改 Codex 原始会话名。
- 不删除、移动、归档 Codex 会话。
- 不读取 `auth.json`、`.env`、密钥、token 或授权文件内容。
- 不保存完整 transcript 到 workflow state、evidence 或 handoff。
- 不把 safe probe 说成真实业务自动编排。

## 已知、未知和假设

已知：

- 桌面派发代码路径已被总指导接受为“代码路径已实现”。
- safe probe prompt 已固定为无业务内容。
- 当前任务队列建议下一步就是工作流节点 safe probe 真实确认派发 v1。
- 真实派发会写 `/Users/yoyi/.codex`。

未知：

- 当前真实工作台状态里是否仍有可用的已绑定测试会话。
- 目标测试会话是否仍可被 `codex exec resume` 正常追加消息。
- 执行时是否会产生 guardian / auto-review 额外线程。
- 执行过程中是否会遇到桌面权限、沙箱或命令权限问题。

假设：

- 只使用已绑定的测试会话。
- 如果找不到已绑定测试会话，本任务应停止并回传原因，不临时改用业务会话。
- 如果需要额外批准，先请求批准，不绕过。
- 如果 safe probe 最终回复不完全匹配预期文本，视为失败或待回收，不自行包装成功。

## 派发内容

safe probe prompt 固定为：

```text
请只回复这一句：WORKFLOW_NODE_DISPATCH_OK_2026_05_29
```

期望最终回复：

```text
WORKFLOW_NODE_DISPATCH_OK_2026_05_29
```

任何额外解释、改写、缺失或命令失败，都必须记录为风险。

## 执行前确认

本任务包本身不构成真实派发授权。

执行真实 safe probe 前，必须再次取得用户明确批准。确认内容必须包含：

- 将向已绑定 Codex 测试会话发送 safe probe。
- 会写 `/Users/yoyi/.codex`。
- 会写真实 workflow state。
- 不读取授权、密钥或 `.env`。
- 不运行 harness。
- 不触碰真实业务会话。
- 不删除、移动、归档任何 Codex 会话。

如果用户没有明确批准，只允许做预览、检查和阻止态验证，不允许执行 `codex exec resume`。

## 允许读取

允许读取项目内：

- `/Users/yoyi/workspace/product-line/CURRENT.md`
- `/Users/yoyi/workspace/product-line/STAGE_PLAN.md`
- `/Users/yoyi/workspace/product-line/tasks/README.md`
- `/Users/yoyi/workspace/product-line/decisions/2026-05-29-codex-session-workflow-route-correction.md`
- `/Users/yoyi/workspace/product-line/decisions/2026-05-29-codex-agent-session-center-project-binding-v1.md`
- `/Users/yoyi/workspace/product-line/decisions/2026-05-30-workflow-first-before-workbench-iteration.md`
- `/Users/yoyi/workspace/product-line/archive/handoffs/2026-05-29-desktop-shell-workflow-node-dispatch-codex-instruction-v1-review.md`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/tests/`
- `/Users/yoyi/workspace/product-line/prototypes/index-kernel/transcript_reader.py`
- `/Users/yoyi/workspace/product-line/prototypes/index-kernel/codex-index.json`

允许读取真实工作台状态的必要结构：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`

用户批准真实派发后，允许只读：

- `/Users/yoyi/.codex/state_5.sqlite` 中目标线程的元数据和统计。
- `/Users/yoyi/.codex/sessions/` 中目标绑定测试会话的 JSONL。
- `/Users/yoyi/.codex/archived_sessions/` 的必要文件清单。

## 允许写入

允许写入项目内交付物：

- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-workflow-node-safe-probe-real-confirmed-dispatch-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-node-safe-probe-real-confirmed-dispatch-v1-result.md`

如发现代码路径存在小缺口，允许写入：

- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/tests/`

用户明确批准真实派发后，允许写入：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/`
- 目标绑定 Codex 测试会话对应的 `/Users/yoyi/.codex` 会话追加记录。

允许写临时文件：

- `/private/tmp/codex-workflow-node-safe-probe-real-dispatch-v1/`

## 禁止事项

- 禁止未获用户明确批准就执行真实 `codex exec resume`。
- 禁止向真实业务会话派发。
- 禁止用未绑定会话临时代替目标测试会话。
- 禁止读取 `/Users/yoyi/.codex/auth.json`。
- 禁止读取 `.env`、密钥、token、授权文件内容。
- 禁止保存完整 transcript。
- 禁止把完整 JSONL 事件流写入 evidence 或 handoff。
- 禁止运行 harness。
- 禁止执行真实业务开发任务。
- 禁止执行 `codex fork`。
- 禁止删除、移动、归档 Codex 会话。
- 禁止绕过工作台确认弹层和审计事件。
- 禁止把 safe probe 成功包装成“真实业务自动工作流完成”。

## 实施要求

执行时按以下顺序：

1. 读取当前 workflow state，确认存在项目、工作流、节点、工作项和绑定测试会话。
2. 生成 safe probe 派发预览。
3. 确认 UI 或后端确认逻辑会明确提示写入边界。
4. 请求用户明确批准真实派发。
5. 批准后执行 `codex exec resume`。
6. 保存最终回复摘要，不保存完整 transcript。
7. 用 transcript reader 读取目标线程统计。
8. 写入 `workflow_node_dispatches[]`。
9. 写入审计事件。
10. 推进工作项状态。
11. 输出 evidence。
12. 输出 handoff。

如果任一步失败：

- 不继续假装后续成功。
- 写明失败发生在哪一步。
- 写明是否已写 `/Users/yoyi/.codex`。
- 写明是否已写真实 workflow state。
- 写明是否需要回滚或人工处理。

## 验收标准

必须满足：

- 找不到绑定测试会话时停止，不派发。
- 用户未明确批准时停止，不派发。
- safe probe prompt 与任务包文本一致。
- 真实派发只发给绑定测试会话。
- 执行结果能拿到最终回复摘要。
- transcript reader 能读回目标线程统计。
- workflow state 中能看到本次派发记录。
- audit events 中能看到准备、开始、完成或失败事件。
- 工作项状态变化有依据。
- evidence 和 handoff 不包含完整 transcript、密钥、授权内容。

验证命令：

```bash
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --offline
```

如果某条验证命令因为环境或权限不能运行，必须写明具体原因，不能写“已验证”。

## 必须回传

开发线回传必须包含：

1. 薄弱点。
2. 做了什么。
3. 改了哪些文件。
4. 新增了哪些 evidence / handoff。
5. 是否执行真实派发。
6. 是否获得用户明确批准。
7. 使用的目标 thread id。
8. 是否写了 `/Users/yoyi/.codex`。
9. 是否写了真实 workflow state。
10. 写入了哪些 workflow state 字段，不打印完整状态正文。
11. 是否读取授权、密钥、`.env`。
12. 是否触碰真实业务会话。
13. 最终回复摘要。
14. transcript 统计。
15. 测试命令和结果。
16. 当前距离真实业务自动工作流还缺什么。

## 总指导回收动作

总指导回收时必须判断：

- 接受。
- 需要修改。
- 暂停。
- 废弃。

回收重点：

- 是否真的从真实工作台状态触发派发。
- 是否只向绑定测试会话派发。
- 是否有明确批准。
- 是否写了真实 workflow state。
- 是否读回最终回复和 transcript 统计。
- 是否没有触碰真实业务会话。
- 是否没有读取授权或密钥。
- 是否没有把 safe probe 夸大成真实业务自动工作流。

## 完成后的下一步候选

如果本任务通过，再派：

- 派发结果读回 UI / 总指导回收意见 v1。
- 长任务、权限确认、失败重试、超时取消最小协议 v1。
- 第一个极小真实工作台自迭代任务。
