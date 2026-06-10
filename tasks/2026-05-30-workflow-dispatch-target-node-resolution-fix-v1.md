# 任务包：工作流派发目标节点解析修正 v1

## 任务名

工作流派发目标节点解析修正 v1。

## 所属开发线

桌面应用线。

总指导线回收。

## 当前判断

README smoke workflow state 已写入，但还不能直接进入真实 README smoke。

依据：

- `work_item.state=ready_to_dispatch`。
- `work_item.current_node_id=...:node:director`。
- active binding 在 `...:node:codex-dev`。
- 前端 `ProjectsView.tsx` 当前按 `current_node_id` 找 binding，并把 `currentNodeId` 作为派发请求的 `node_id`。
- 后端派发需要请求里的 `node_id` 能找到 active binding。

大白话：

工作项现在在“总指导准备派发”这个流程节点上，但真正要派给 Codex 开发线。UI 现在把这两个节点混在一起了。

## 薄弱点

- 这是代码修正任务，不执行真实 README smoke。
- 不验证真实 `codex exec resume`。
- 如果只改 state，把 current node 改成 codex-dev，会破坏现有状态规则；应优先修 UI 的目标节点解析。
- 离线测试原夹具把 binding 放在 director 节点，没覆盖这次真实状态暴露出来的情况。

## 目标

修正桌面壳派发目标节点解析：

1. 保留 `ready_to_dispatch` 的 `current_node_id=director` 状态规则。
2. 为 work item 计算实际派发节点：
   - 优先使用 `assigned_role_id` 映射的节点。
   - `assigned_role_id=codex-dev` 时，目标节点是 `${workflow_id}:node:codex-dev`。
   - 找不到时再回退到 `current_node_id`。
3. `currentBinding` 查找应优先找实际派发节点上的 work-item 绑定。
4. 绑定候选动作应把会话绑定到实际派发节点，而不是 director。
5. safe probe 和用户审核业务派发请求里的 `node_id` 应使用实际派发节点。
6. UI 文案区分：
   - 当前流程节点。
   - 实际派发节点。
7. 离线测试覆盖真实形态：
   - work item 当前节点是 director。
   - binding 在 codex-dev。
   - 派发按钮可用。
   - 派发请求 `node_id=...:node:codex-dev`。

## 非目标

- 不执行真实 `codex exec resume`。
- 不发送 README smoke 指令。
- 不执行新的 `codex exec`。
- 不写 `/Users/yoyi/.codex`。
- 不写真实 workflow state。
- 不修改 `/Users/yoyi/codex-workflow-mario-test/README.md`。
- 不读取完整 transcript。
- 不读取 `auth.json`、`.env`、密钥、token、授权文件。
- 不运行 harness。
- 不改变 workflow state 的状态机规则。

## 允许读取

允许读取：

- `/Users/yoyi/workspace/product-line/CURRENT.md`
- `/Users/yoyi/workspace/product-line/tasks/README.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-prepare-workflow-state-for-readme-smoke-v2-review.md`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`，只用于确认状态规则和后端派发字段

允许只读复核真实 workflow state 必要摘要。

禁止读取：

- `/Users/yoyi/.codex/auth.json`
- `.env`
- 密钥、token、授权文件内容
- 完整 transcript 正文
- rollout JSONL 正文

## 允许写入

允许修改：

- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- 如类型需要，允许修改 `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- 如构建产物由 `npm run build` 更新，必须在 evidence / handoff 记录 `dist/` 变化

允许写 evidence / handoff：

- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-workflow-dispatch-target-node-resolution-fix-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-dispatch-target-node-resolution-fix-v1-result.md`

允许更新：

- `/Users/yoyi/workspace/product-line/CURRENT.md`
- `/Users/yoyi/workspace/product-line/tasks/README.md`

## 禁止事项

- 禁止执行 `codex exec resume`。
- 禁止发送 README smoke 指令。
- 禁止执行新的 `codex exec`。
- 禁止写 `/Users/yoyi/.codex`。
- 禁止写真实 workflow state。
- 禁止修改 README。
- 禁止读取完整 transcript。
- 禁止读取敏感文件。
- 禁止运行 harness。
- 禁止把修 UI 说成真实 README smoke 已完成。
- 禁止在 shell 双引号里写未转义反引号模式；搜索包含反引号文本时必须使用单引号或 `rg -F`。

## 建议实现口径

在 `ProjectsView.tsx` 中增加本地解析逻辑：

```text
dispatchNodeId = dispatchNodeIdForWorkItem(workItem)
```

建议规则：

- 如果 `workItem.assigned_role_id` 存在，使用 `${workItem.workflow_id}:node:${assigned_role_id}`。
- 否则使用 `workItem.current_node_id`。

`currentBinding` 改为：

1. 先找 `binding.node_id === dispatchNodeId && binding.work_item_id === workItem.work_item_id`。
2. 再找 `binding.node_id === dispatchNodeId && !binding.work_item_id`。
3. 最后才回退到当前流程节点绑定。

绑定按钮、解绑按钮、safe probe、审核后派发都应明确使用实际派发节点。

## 验收标准

必须满足：

- work item 当前节点是 director、binding 在 codex-dev 时，UI 显示已绑定。
- safe probe 按钮可用。
- 审核后派发在完整用户审核指令下可用。
- 派发请求的 `node_id` 是 `...:node:codex-dev`。
- 绑定新会话时写入的 `node_id` 是实际派发节点。
- 离线测试覆盖上述行为。
- 未执行 `codex exec` 或 `codex exec resume`。
- 未写 `/Users/yoyi/.codex`。
- 未写真实 workflow state。
- 未修改 README。

## 验证命令

至少运行：

```bash
npm run typecheck
npm run test:offline-interaction
npm run build
```

如 Rust 代码未改，可说明未运行 Cargo；如 Rust 代码有改，必须运行：

```bash
cargo fmt
cargo test --offline
```

索引校验：

```bash
/Users/yoyi/miniconda3/bin/python3 prototypes/index-kernel/build_index.py --check prototypes/index-kernel/codex-index.json
```

固定字符串搜索：

```bash
rg -F 'codex exec resume' tasks evidence handoffs CURRENT.md
```

## 必须回传

回传必须包含：

1. 薄弱点。
2. 改了哪些文件。
3. 是否执行 `codex exec` 或 `codex exec resume`。
4. 是否写 `/Users/yoyi/.codex`。
5. 是否写真实 workflow state。
6. 是否修改 README。
7. 派发目标节点解析规则。
8. 离线测试覆盖点。
9. 验证命令和结果。
10. 新增 evidence / handoff。

## 总指导回收重点

总指导回收时必须判断：

- 是否解决 director 当前节点与 codex-dev 执行节点分离的问题。
- 是否没有通过改真实 state 绕过 UI 问题。
- 是否可以进入真实 README smoke 派发任务。
