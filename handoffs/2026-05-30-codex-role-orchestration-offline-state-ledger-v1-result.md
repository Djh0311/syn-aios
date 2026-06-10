# Handoff：Codex 角色编排离线状态账本 v1

## 薄弱点

- 本轮仍不是复杂自动化完成。
- 没有执行真实 `codex exec` 或 `codex exec resume`。
- 没有把真实 Codex 会话按多角色自动跑起来。
- 角色回传仍是桩结果，只证明工作台能记录和推进账本。

## 做了什么

- 增加离线角色派发后端命令，写 `workflow_node_dispatches[]` prepared 记录。
- 增加离线角色回传命令，写 handoff artifact，把 dispatch 置为 completed，把 work item 推到 `ready_for_review`。
- 增加离线总指导回收命令，写 `reviews[]`，把 work item 推到回收结论状态。
- 前端工作流页接入三个待确认动作：写入离线派发、写入角色回传、写入总指导回收。
- 权限弹层展示离线派发、回传、回收的边界和核心字段。
- 离线派发 readback 会带回 `offline_role_dispatch`，角色回传优先使用已落账派发块。

## 改了哪些文件

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/views/OfflineRoleOrchestrationPanel.tsx`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `prototypes/productized-desktop-shell/dist/index.html`
- `prototypes/productized-desktop-shell/dist/assets/index-DDJziFyU.css`
- `prototypes/productized-desktop-shell/dist/assets/index-DmWTx_Ms.js`

## 新增 Evidence

- `evidence/2026-05-30-codex-role-orchestration-offline-state-ledger-v1.md`

## 边界

- 是否执行 `codex exec`：否。
- 是否执行 `codex exec resume`：否。
- 是否写 `/Users/yoyi/.codex`：否。
- 是否发送 Codex 消息：否。
- 是否写真实 workflow state：否；只在临时测试状态里验证写入，真实 UI 仍需要用户点击确认。
- 是否修改业务项目文件：否。
- 是否读取敏感文件或完整 transcript：否。
- 是否运行 harness：否。

## 验证

- `cargo fmt`：通过。
- 指定共享 Cargo 缓存的 `cargo test --offline`：通过，66 passed，1 ignored。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 4`。
- `npm run build`：通过。
- 索引校验：`validation_ok`。
- `rg -F 'codex exec resume' tasks evidence handoffs CURRENT.md`：固定字符串搜索完成，未触发命令替换。

## 当前可回收口径

可接受为：

- 工作台自己的离线角色编排账本闭环已完成。
- prepared dispatch、role handoff、director review 三步都有后端命令和离线测试。
- UI 能从 ready_to_dispatch work item 作为账本锚点发起三步待确认动作。

不可接受为：

- 真实多 Codex 会话自动编排。
- 总指导自动制定阶段计划。
- 角色会话真实执行和回传。
- 复杂业务自动工作流完成。

## 下一步

建议下一步做“总指导计划块接入 v1”：把总指导回复解析成多个候选派发块，仍走用户确认，不直接真实执行。
