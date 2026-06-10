# 总指导回收意见：准备真实 workflow state 以便 safe probe v1 - 多线协作版

## 回收对象

- 任务包：`/Users/yoyi/workspace/product-line/tasks/2026-05-30-prepare-real-workflow-state-for-safe-probe-multiline-v1.md`
- Evidence：`/Users/yoyi/workspace/product-line/evidence/2026-05-30-prepare-real-workflow-state-for-safe-probe-multiline-v1.md`
- Handoff：`/Users/yoyi/workspace/product-line/handoffs/2026-05-30-prepare-real-workflow-state-for-safe-probe-multiline-v1-result.md`

## 结论

需要修改。

接受：

- 真实 workflow state 已写入绑定。
- 目标 work item 已从 `draft` 推进到 `ready_to_dispatch`。
- 没有执行 `codex exec resume`。
- 没有发送 safe probe。
- 没有写 `/Users/yoyi/.codex`。

不接受：

- 不接受“下一轮 safe probe 前置条件已满足”这个结论。

原因：

- 绑定 thread id 当前不在桌面壳使用的 `codex-index.json`。
- 当前后端派发代码会在 `workflow_node_dispatch_context` 里调用 `find_index_thread`。
- 如果当前索引中找不到绑定 thread，会直接报错：`绑定会话不在当前索引内，已拒绝派发`。

## 薄弱点

- 当前 binding 记录存在，但绑定会话不在当前索引中。依据：handoff 明确写了 `session_not_found_in_current_index`，总指导复核 `codex-index.json` 未命中 thread id。
- 当前验证线结论只验证了 workflow state，没有验证桌面派发代码的索引前置条件。
- 测试会话 cwd 是 `/private/tmp/codex-control-probe-v2`，目标项目是 `/Users/yoyi/gameai/agent world`。该 warning 已写入，处理是正确的，但后续派发仍只能按无业务测试会话处理。
- 真实 safe probe 仍未执行。

## 接受内容

接受以下状态写入：

- 新增 active `workflow_node_session_bindings[]`。
- `work_items[].state` 从 `draft` 到 `ready_to_dispatch`。
- `work_items[].current_node_id` 指向 `codex-dev` 节点。
- 目标 `nodes[].state` 更新。
- 写入两个审计事件。
- 记录 warnings：
  - `session_not_found_in_current_index`
  - `session_cwd_differs_from_project_root`
  - `test_session_cwd:/private/tmp/codex-control-probe-v2`
  - `confirmed_test_session_not_business_session`

## 验证依据

总指导只读复核真实 workflow state：

- active binding 存在。
- binding thread id 是 `019e7389-349a-7f02-aa31-a4a90b24e865`。
- binding node id 指向 `workflow:users-yoyi-gameai-agent-world:default:node:codex-dev`。
- binding work item id 指向 `work-item:workflow:users-yoyi-gameai-agent-world:default:1780032043420`。
- work item 状态是 `ready_to_dispatch`。
- `workflow_node_dispatches[]` 数量仍为 `0`。

代码前置条件复核：

- `execute_workflow_node_dispatch_at` 会调用 `workflow_node_dispatch_context`。
- `workflow_node_dispatch_context` 会调用 `find_index_thread(index, &native_thread_id)`。
- 找不到时会拒绝派发。

## 当前不能说

- 不能说 safe probe 前置条件完全满足。
- 不能说可以直接进入真实 safe probe 派发。
- 不能说测试会话已进入当前桌面壳索引。
- 不能说真实业务自动工作流完成。

## 当前可以说

- workflow state 侧的绑定和工作项状态已经准备好。
- Codex 索引侧还没准备好。
- 下一步应先刷新或补齐桌面壳使用的 Codex 索引，让确认测试 thread 出现在 `codex-index.json` 中。

## 下一步

新派任务：

- `tasks/2026-05-30-refresh-codex-index-for-confirmed-safe-probe-thread-v1.md`

目标：

- 只读扫描 Codex 元数据。
- 刷新或生成桌面壳可用的 `codex-index.json`。
- 确认 thread `019e7389-349a-7f02-aa31-a4a90b24e865` 存在于当前索引。
- 确认 rollout 存在。
- 不读取完整 transcript。
- 不写 `/Users/yoyi/.codex`。
- 不执行 `codex exec resume`。

完成后再重试：

- `tasks/2026-05-30-workflow-node-safe-probe-real-confirmed-dispatch-v1.md`
