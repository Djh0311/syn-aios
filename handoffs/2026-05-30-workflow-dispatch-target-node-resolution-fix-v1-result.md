# Handoff：工作流派发目标节点解析修正 v1

## 结论

桌面壳派发目标节点解析已修正，当前流程节点为 director、实际 binding 在 codex-dev 的形态已由离线测试覆盖。

这不代表真实 README smoke 已执行。

## 薄弱点

- 本轮没有真实执行 README smoke，也没有真实发送 safe probe。
- 本轮不写真实 workflow state，所以不能证明后端真实 state 写入链路在下一轮一定成功。
- `product-line` 目录当前不是 git 仓库，缺少 git diff / status 级别的变更审计。
- 构建重新生成了 `prototypes/productized-desktop-shell/dist/`，总指导需要确认构建产物是否作为本轮交付物保留。

## 修改文件

- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `prototypes/productized-desktop-shell/dist/index.html`
- `prototypes/productized-desktop-shell/dist/assets/index-DEUb_c1t.js`
- `prototypes/productized-desktop-shell/dist/assets/index-BTACVauc.css`
- `CURRENT.md`
- `tasks/README.md`
- `evidence/2026-05-30-workflow-dispatch-target-node-resolution-fix-v1.md`
- `handoffs/2026-05-30-workflow-dispatch-target-node-resolution-fix-v1-result.md`

## 关键行为

- `dispatchNodeIdForWorkItem(workItem)` 优先把 `assigned_role_id` 映射成 `${workflow_id}:node:${assigned_role_id}`。
- `assigned_role_id` 为空时才回退 `current_node_id`。
- binding 查找优先实际派发节点，再回退当前流程节点。
- 新绑定、safe probe、用户审核业务派发 action 均使用实际派发节点。
- UI 展示区分 `当前流程节点` 和 `实际派发节点`，绑定标题改为 `实际派发节点已有绑定`。

## 离线测试覆盖

- work item 当前流程节点保持 `...:node:director`。
- active binding 和派发历史放在 `...:node:codex-dev`。
- 断言绑定候选、解绑、安全测试派发、审核后派发 action 的节点或 binding 都指向 codex-dev。
- 断言 safe probe 和审核后派发按钮在完整绑定和审核指令下可用。

## 禁止项复核

- 是否执行 `codex exec` 或 `codex exec resume`：否。
- 是否写 `/Users/yoyi/.codex`：否。
- 是否写真实 workflow state：否。
- 是否修改 README：否。
- 是否读取授权、密钥、`.env`：否。
- 是否读取完整 transcript：否。
- 是否运行 harness：否。

## 验证结果

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 3`。
- `npm run build`：通过，重新生成 `dist/`。
- `/Users/yoyi/miniconda3/bin/python3 prototypes/index-kernel/build_index.py --check prototypes/index-kernel/codex-index.json`：通过，`validation_ok`。
- `rg -F 'codex exec resume' tasks evidence handoffs CURRENT.md`：通过固定字符串搜索；输出包含历史记录和任务文本命中。
- director 节点残留复核：测试中只保留 work item 的 `current_node_id=...:node:director`，派发 action 不再使用 director。

## 下一步

总指导回收通过后，才能进入真实 README smoke 派发。下一轮会执行真实 `codex exec resume` 并写 `/Users/yoyi/.codex`，需要用户再次明确批准。
