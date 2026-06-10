# 任务包：准备真实 workflow state 以便 safe probe v1 - 多线协作版

## 任务名

准备真实 workflow state 以便 safe probe v1 - 多线协作版。

## 所属开发线

总指导线牵头。

参与开发线：

- Codex 会话线。
- 桌面应用线 / 工作流运行线。
- 验证线。

信息架构线只在发现 UI 文案或入口误导时介入。

## 当前判断

下一步不能直接执行 safe probe 派发。

原因：

- 真实 workflow state 当前没有节点会话绑定。
- 当前唯一工作项状态是 `draft`，不是 `ready_to_dispatch`。
- 真实 workflow state 当前指向 `/Users/yoyi/gameai/agent world`，不是当前 `product-line`。必须确认目标项目。

因此本任务只负责把真实 workflow state 准备到可派发状态，不负责发送 safe probe。

依据：

- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-node-safe-probe-real-confirmed-dispatch-v1-result.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-node-safe-probe-real-confirmed-dispatch-v1-review.md`
- `/Users/yoyi/workspace/product-line/tasks/2026-05-30-prepare-real-workflow-state-for-safe-probe-v1.md`
- `/Users/yoyi/workspace/product-line/DEV_LINES.md`
- `/Users/yoyi/workspace/product-line/CURRENT.md`

## 薄弱点

- 目前“多线协作”仍主要靠任务包、handoff 和总指导回收，不是系统自动调度。依据：真实 workflow state 尚未准备好，safe probe 也尚未跑通。
- 如果没有明确测试会话，桌面应用线不能自行选一个业务会话替代。
- 如果目标项目不是 `/Users/yoyi/gameai/agent world`，当前真实 workflow state 需要先重新确认项目，不应直接写现有状态。
- 绑定会话和推进工作项都会写真实 workflow state，必须先获得用户明确批准。
- 本任务如果执行 `codex exec resume`，就是越界。

## 背景

上一轮真实 safe probe 派发任务停止在前置检查：

- `workflow_node_session_bindings[] = 0`
- `workflow_node_dispatches[] = 0`
- 当前 work item 状态是 `draft`
- 没有目标 thread id
- 没有写 `/Users/yoyi/.codex`
- 没有写真实 workflow state

总指导回收意见是：

- 不接受为真实 safe probe 派发完成。
- 接受“前置检查按规则停止”。
- 下一步先准备真实 workflow state。

## 总目标

在不发送任何 Codex 消息的前提下，把真实 workflow state 准备到下一轮 safe probe 可以执行：

1. 明确目标项目。
2. 明确目标 workflow。
3. 明确目标 work item。
4. 明确目标开发节点。
5. 明确一个测试 Codex 会话。
6. 用户确认后写入节点会话绑定。
7. 用户确认后把 work item 推进到 `ready_to_dispatch`。
8. 验证线复核前置条件满足。
9. 总指导回收后再决定是否重试 safe probe 派发。

## 分线任务

### Codex 会话线

目标：

- 只读列出可作为测试会话的候选 Codex 会话摘要。
- 不读取完整 transcript。
- 不发送消息。
- 不写 `/Users/yoyi/.codex`。

允许输出字段：

- thread id。
- 原 Codex 会话名。
- 项目路径。
- 最近更新时间。
- rollout 是否存在。
- 索引 warning。
- 是否看起来是测试会话。

禁止：

- 禁止读取 `auth.json`、`.env`、密钥、token 或授权文件内容。
- 禁止读取完整会话正文。
- 禁止执行 `codex exec`、`codex exec resume`、`codex fork`。
- 禁止把“看起来像测试会话”当作用户确认事实。

必须回传：

1. 候选测试会话列表。
2. 每个候选的依据。
3. 哪些候选不应使用，原因是什么。
4. 是否读取授权、密钥、`.env`。
5. 是否写 `/Users/yoyi/.codex`。
6. 是否发送任何消息。

### 桌面应用线 / 工作流运行线

目标：

- 在用户确认目标对象后，写真实 workflow state。
- 绑定指定测试会话到指定工作流节点。
- 把指定 work item 从 `draft` 推进到 `ready_to_dispatch`。
- 写审计事件和备份。

允许写入：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/`
- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-prepare-real-workflow-state-for-safe-probe-multiline-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-prepare-real-workflow-state-for-safe-probe-multiline-v1-result.md`

禁止：

- 禁止未获用户明确批准就写真实 workflow state。
- 禁止执行 `codex exec resume`。
- 禁止发送 safe probe。
- 禁止写 `/Users/yoyi/.codex`。
- 禁止绑定真实业务会话作为测试会话。
- 禁止读取完整 transcript。

必须回传：

1. 用户确认方式。
2. 目标项目路径。
3. 目标 workflow id。
4. 目标 node id。
5. 目标 work item id。
6. 目标 thread id。
7. 写入了哪些字段类型，不打印完整状态正文。
8. 是否写了真实 workflow state。
9. 是否写了 `/Users/yoyi/.codex`。
10. 是否执行 `codex exec resume`。
11. 新增 evidence / handoff。

### 验证线

目标：

- 只读验证真实 workflow state 是否满足下一轮 safe probe 前置条件。

必须检查：

- `workflow_node_session_bindings[]` 是否存在 active 绑定。
- 绑定 thread id 是否等于用户确认的测试会话。
- 绑定是否指向目标 node id。
- 绑定是否指向目标 work item id，或明确说明是节点级绑定。
- 目标 work item 是否为 `ready_to_dispatch`。
- `workflow_node_dispatches[]` 是否仍未新增真实派发记录。
- `/Users/yoyi/.codex` 是否未被本任务写入。

禁止：

- 禁止写真实 workflow state。
- 禁止执行 `codex exec resume`。
- 禁止读取完整 transcript。
- 禁止读取授权、密钥、`.env`。

必须回传：

1. 是否满足 safe probe 前置条件。
2. 如果不满足，缺哪一项。
3. 是否发现越界写入。
4. 是否建议进入下一轮 safe probe 派发。

### 总指导线

职责：

- 派发本任务包。
- 汇总三条线回传。
- 判断本任务接受、需要修改、暂停或废弃。
- 如果接受，再派 `2026-05-30-workflow-node-safe-probe-real-confirmed-dispatch-v1.md` 重试真实 safe probe。

禁止：

- 不把状态准备说成 safe probe 已派发。
- 不在没有验证线复核时进入下一派发任务。

## 目标对象初始事实

当前只读复核看到：

- 项目路径：`/Users/yoyi/gameai/agent world`
- workflow：`workflow:users-yoyi-gameai-agent-world:default`
- 开发节点：`workflow:users-yoyi-gameai-agent-world:default:node:codex-dev`
- work item：`work-item:workflow:users-yoyi-gameai-agent-world:default:1780032043420`
- work item 当前状态：`draft`

这些只是当前事实，不等于用户已确认目标。

## 用户确认要求

执行真实 workflow state 写入前，必须让用户明确确认：

- 目标项目路径。
- 目标 workflow id。
- 目标 node id。
- 目标 work item id。
- 目标测试 thread id。
- 允许写真实 workflow state。
- 知道本任务不会执行 `codex exec resume`。
- 知道本任务不会写 `/Users/yoyi/.codex`。

没有这个确认，桌面应用线只能输出阻塞原因。

## 统一禁止事项

- 禁止执行 `codex exec resume`。
- 禁止发送 safe probe。
- 禁止写 `/Users/yoyi/.codex`。
- 禁止读取 `auth.json`、`.env`、密钥、token 或授权文件内容。
- 禁止读取完整会话正文。
- 禁止运行 harness。
- 禁止创建新 Codex 会话。
- 禁止删除、移动、归档 Codex 会话。
- 禁止触碰真实业务会话。
- 禁止把索引推断当作用户确认事实。

## 验收标准

本任务接受的最低标准：

- Codex 会话线给出候选测试会话摘要，且没有越界读取或发送消息。
- 用户明确确认目标对象和写真实 workflow state。
- 桌面应用线写入 active 节点会话绑定。
- 桌面应用线把目标 work item 推进到 `ready_to_dispatch`。
- 验证线确认下一轮 safe probe 前置条件满足。
- 没有执行 `codex exec resume`。
- 没有写 `/Users/yoyi/.codex`。
- 没有读取授权、密钥、`.env`。
- evidence 和 handoff 不包含完整 transcript。

如果没有用户确认，最低可接受结果只能是：

- 暂停。
- 明确列出缺少的确认项。
- 不写真实 workflow state。

## 建议验证命令

如果没有改应用代码，可以只做只读结构验证，并说明未运行构建命令的原因。

如果改了应用代码，必须运行：

```bash
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --offline
```

不能把未运行命令写成已通过。

## 必须回传

最终 handoff 必须包含：

1. 薄弱点。
2. 三条线分别做了什么。
3. 是否获得用户确认。
4. 是否写真实 workflow state。
5. 是否写 `/Users/yoyi/.codex`。
6. 是否执行 `codex exec resume`。
7. 目标项目、workflow、node、work item、thread id。
8. 写入字段类型。
9. 验证线结论。
10. 新增 evidence / handoff。
11. 是否可以进入下一轮 safe probe 派发。

## 总指导回收动作

总指导必须判断：

- 接受。
- 需要修改。
- 暂停。
- 废弃。

只有在验证线确认前置条件满足后，才能进入：

- `tasks/2026-05-30-workflow-node-safe-probe-real-confirmed-dispatch-v1.md`
