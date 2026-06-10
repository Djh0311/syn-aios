# Review：工作流派发目标节点解析修正 v1

## 结论

接受。

接受为：

- 桌面壳 UI 派发目标节点解析已修正。
- `ready_to_dispatch` 的当前流程节点保留为 `director`。
- 实际派发节点可按 `assigned_role_id=codex-dev` 解析到 `:node:codex-dev`。
- current node 为 director、binding 在 codex-dev 的形态已由离线测试覆盖。

不接受为：

- 真实 README smoke 已执行。
- 真实 `codex exec resume` 已验证。
- 复杂业务自动编排已完成。

## 薄弱点

- 本轮没有执行真实 README smoke，所以不能证明下一轮真实派发一定成功。
- 本轮没有写真实 workflow state，只验证 UI 和离线 action payload。
- `product-line` 目录不是 git 仓库，不能用 `git status` 做变更基线。
- `npm run build` 重新生成了 `dist/`，本轮接受为构建产物变化，但后续仍应决定是否长期纳入交付口径。

## 回收依据

已复核：

- `evidence/2026-05-30-workflow-dispatch-target-node-resolution-fix-v1.md`
- `handoffs/2026-05-30-workflow-dispatch-target-node-resolution-fix-v1-result.md`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

## 关键复核结果

### 派发目标节点解析

接受。

依据：

- `dispatchNodeIdForWorkItem(workItem)` 优先读取 `assigned_role_id`。
- `assigned_role_id=codex-dev` 时返回 `${workflow_id}:node:codex-dev`。
- `assigned_role_id` 为空时才回退 `current_node_id`。

### Binding 查找

接受。

依据：

`currentBinding` 查找顺序已经改成：

1. 实际派发节点 + 当前 work item。
2. 实际派发节点通用绑定。
3. 当前流程节点 + 当前 work item。
4. 当前流程节点通用绑定。

### UI 文案

接受。

依据：

- UI 显示 `当前流程节点`。
- UI 显示 `实际派发节点`。
- 绑定区标题改为 `实际派发节点已有绑定`。

### 派发 action

接受。

依据：

- 绑定候选 action 使用 `dispatchNodeId`。
- safe probe action 使用 `dispatchNodeId`。
- 用户审核业务派发 action 使用 `dispatchNodeId`。
- 解绑仍使用当前 binding id，符合绑定对象实际来源。

### 离线测试

接受。

依据：

- 离线 work item 保留 `current_node_id=...:node:director`。
- 离线 active binding 改为 `node_id=...:node:codex-dev`。
- 离线断言 safe probe action 的 `node_id=...:node:codex-dev`。
- 离线断言用户审核业务派发 action 的 `node_id=...:node:codex-dev`。
- 搜索复核显示 director 节点残留只在 work item 的 `current_node_id`，符合流程节点保留规则。

## 验证结果

总指导复跑：

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 3`。
- `npm run build`：通过，产物为：
  - `dist/index.html`
  - `dist/assets/index-DEUb_c1t.js`
  - `dist/assets/index-BTACVauc.css`
- `/Users/yoyi/miniconda3/bin/python3 prototypes/index-kernel/build_index.py --check prototypes/index-kernel/codex-index.json`：通过，`validation_ok`。
- `rg -F 'codex exec resume' tasks evidence handoffs CURRENT.md`：固定字符串搜索完成；输出为历史文档和任务文本命中，没有触发命令替换。
- `rg -n "binding:offline:director|node_id: \"workflow:offline-fixture-projects-codex-workbench:default:node:director\"" ...`：只剩 work item 的 `current_node_id=...:node:director`。

## 边界

- 是否执行 `codex exec`：否。
- 是否执行 `codex exec resume`：否。
- 是否写 `/Users/yoyi/.codex`：否。
- 是否写真实 workflow state：否。
- 是否修改 `/Users/yoyi/codex-workflow-mario-test/README.md`：否。
- 是否读取敏感文件或完整 transcript：否。
- 是否运行 harness：否。

## 回收决定

本轮通过。

可以进入：

- `tasks/2026-05-30-workflow-user-reviewed-business-dispatch-real-readme-smoke-v1.md`

但执行前必须再次获得用户明确批准。

原因：

- 下一轮会执行真实 `codex exec resume`。
- 下一轮会写 `/Users/yoyi/.codex`。
- 下一轮会修改 `/Users/yoyi/codex-workflow-mario-test/README.md`。
- 下一轮会写真实 workflow state。
