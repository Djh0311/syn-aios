# Review：准备 README smoke 测试 workflow state v2

## 结论

需要修改。

不是因为真实 workflow state 没写进去，而是因为下一轮如果通过桌面壳 UI 进入真实 README smoke，当前派发入口大概率会被“当前流程节点 / 实际派发节点不一致”卡住。

## 薄弱点

- 真实 workflow state 已写入，但 UI 派发路径还没闭合。
- work item 当前状态是 `ready_to_dispatch`，按现有状态规则当前节点是 `director`。
- active binding 写在 `codex-dev` 节点上。
- 前端当前按 `workItem.current_node_id` 找 binding 并派发；这会找 `director` 绑定，而不是 `codex-dev` 绑定。
- 后端如果直接传 `node_id=codex-dev`，结构上可以继续；但桌面壳按钮路径会先卡在 UI 绑定识别。

## 回收依据

开发线回传：

- 写真实 workflow state：是。
- 写 `/Users/yoyi/.codex`：否。
- 执行 `codex exec` 或 `codex exec resume`：否。
- README 未修改。
- 绑定 thread：`019e7738-5e29-74e0-a22f-5c2481b64c38`。
- rollout 存在。

只读复核：

- 新 project 存在：`project:users-yoyi-codex-workflow-mario-test`。
- 新 workflow 存在：`workflow:users-yoyi-codex-workflow-mario-test:default`。
- 新 work item 存在，状态为 `ready_to_dispatch`。
- 新 work item 的 `current_node_id=workflow:users-yoyi-codex-workflow-mario-test:default:node:director`。
- 新 binding 存在。
- 新 binding 的 `node_id=workflow:users-yoyi-codex-workflow-mario-test:default:node:codex-dev`。
- 新 binding 的 `native_thread_id=019e7738-5e29-74e0-a22f-5c2481b64c38`。
- 索引中目标 thread 存在，`project_root=/Users/yoyi/codex-workflow-mario-test`，`rollout_exists=true`。

## 后端路径判断

部分接受。

依据：

- `workflow_node_dispatch_context` 会按请求里的 `workflow_id + node_id + work_item_id` 查 work item 和 active binding。
- 当前 state 里存在 `codex-dev` 节点绑定，且绑定 work item id 匹配。
- 如果请求显式传 `node_id=workflow:users-yoyi-codex-workflow-mario-test:default:node:codex-dev`，后端前置条件大体满足。

不接受为“桌面壳派发路径已准备好”。

依据：

- `ProjectsView.tsx` 里 `currentBinding` 通过 `workItem.current_node_id` 查找。
- `ready_to_dispatch` 的 `current_node_id` 按状态规则是 `:node:director`。
- 派发按钮里的 `nodeDispatch.node_id` 也来自 `currentNodeId`。
- 因此 UI 会倾向使用 director 节点，而真实 Codex binding 在 codex-dev 节点。

## 回收决定

本轮接受为：

- README smoke 的真实 workflow state 已写入。
- 测试项目 project / workflow / node / work item / active binding 已存在。
- 绑定 thread 路径匹配，rollout 存在。
- 没有执行 README smoke。
- 没有写 `/Users/yoyi/.codex`。
- 没有改 README。

本轮不接受为：

- 可以直接通过桌面壳 UI 进入真实 README smoke。
- 用户审核业务派发闭环已经可验证。

## 下一步

先修正桌面壳派发目标节点解析。

目标：

- 保留 `ready_to_dispatch -> current_node_id=director` 的状态规则。
- 派发动作改用 work item 的执行节点，也就是 `assigned_role_id=codex-dev` 对应的 `:node:codex-dev`。
- UI 同时显示“当前流程节点”和“实际派发节点”，避免把总指导节点误当成 Codex 执行节点。
- 离线测试要覆盖：work item 当前节点是 director、binding 在 codex-dev 时，派发按钮仍可用，且派发请求传 `node_id=codex-dev`。

下一任务包：

- `tasks/2026-05-30-workflow-dispatch-target-node-resolution-fix-v1.md`
