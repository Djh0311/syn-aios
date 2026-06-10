# Evidence：工作流派发目标节点解析修正 v1

## 结论

已修正桌面壳工作流派发目标节点解析。

本轮只改代码和离线测试，没有执行真实 README smoke，没有执行 `codex exec` 或 `codex exec resume`，没有写真实 workflow state。

## 薄弱点

- 本轮没有执行真实派发，所以不能证明下一轮真实 README smoke 一定成功。
- 本轮没有读完整 transcript，也没有读取 rollout 正文；只覆盖 UI 解析和离线 action payload。
- `/Users/yoyi/workspace/product-line` 不是 git 仓库，无法用 `git status` 作为变更基线；本 evidence 依据命令输出、定向搜索和文件路径记录。
- `npm run build` 重新生成了 `prototypes/productized-desktop-shell/dist/`，需要总指导确认构建产物是否纳入交付口径。

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

## 派发目标节点解析规则

- `ready_to_dispatch` 的 `current_node_id` 保持为流程节点，例如 `...:node:director`。
- UI 计算实际派发节点时优先使用 `assigned_role_id`：
  - `assigned_role_id=codex-dev` 时使用 `${workflow_id}:node:codex-dev`。
  - 没有 `assigned_role_id` 时回退到 `current_node_id`。
- active binding 查找顺序：
  - 实际派发节点 + 当前 work item。
  - 实际派发节点通用绑定。
  - 当前流程节点 + 当前 work item。
  - 当前流程节点通用绑定。
- 新绑定候选、safe probe 派发、用户审核业务派发的 `node_id` 均使用实际派发节点。
- UI 文案区分 `当前流程节点` 和 `实际派发节点`。

## 离线覆盖

- 离线夹具中 work item 保持：
  - `current_node_id=workflow:offline-fixture-projects-codex-workbench:default:node:director`
  - `assigned_role_id=codex-dev`
- 离线夹具中 active binding 改为：
  - `node_id=workflow:offline-fixture-projects-codex-workbench:default:node:codex-dev`
- 覆盖点：
  - UI 显示当前流程节点和实际派发节点。
  - director 流程节点下仍能识别 codex-dev 绑定。
  - 绑定候选动作写入 `node_id=...:node:codex-dev`。
  - 解绑动作使用 codex-dev binding id。
  - safe probe action 写入 `node_id=...:node:codex-dev`。
  - 用户审核业务派发 action 写入 `node_id=...:node:codex-dev`。

## 边界

- 是否执行 `codex exec`：否。
- 是否执行 `codex exec resume`：否。
- 是否写 `/Users/yoyi/.codex`：否。
- 是否写真实 workflow state：否。
- 是否修改 `/Users/yoyi/codex-workflow-mario-test/README.md`：否。
- 是否读取授权、密钥、`.env` 或完整 transcript：否。
- 是否运行 harness：否。

## 验证命令和结果

- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 3`。
- `npm run typecheck`：通过。
- `npm run build`：通过，Vite 构建成功并重新生成 `dist/`。
- `/Users/yoyi/miniconda3/bin/python3 prototypes/index-kernel/build_index.py --check prototypes/index-kernel/codex-index.json`：通过，`validation_ok`。
- `rg -F 'codex exec resume' tasks evidence handoffs CURRENT.md`：固定字符串搜索完成；输出为历史任务、evidence、handoff 和当前任务包文本命中，没有触发命令替换。
- `rg -n "binding:offline:director|当前节点已有绑定|node_id: \"workflow:offline-fixture-projects-codex-workbench:default:node:director\"" prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`：只剩 work item 的 `current_node_id=...:node:director`，符合流程节点保留规则。

## Handoff

- `handoffs/2026-05-30-workflow-dispatch-target-node-resolution-fix-v1-result.md`
