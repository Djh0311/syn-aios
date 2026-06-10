# Handoff：Codex 角色编排离线闭环 v1

## 结论

已完成第一版工作台内 Codex 角色编排入口。

它能把总指导派发块解析成离线派发确认动作，并展示角色桩结果回传总指导；但还没有持久化到 workflow state，也没有真实调度多个 Codex 会话。

## 薄弱点

- 这不是复杂自动化完成。
- `offline-role-dispatch` 当前只确认前端动作并显示 notice，不写真实 workflow state。
- 角色回传是桩结果，不是来自真实 Codex 会话。
- 派发块预览不是实时编辑器，提交时才按表单内容重新解析。

## 边界

- 是否执行 `codex exec`：否。
- 是否执行 `codex exec resume`：否。
- 是否写 `/Users/yoyi/.codex`：否。
- 是否写真实 workflow state：否。
- 是否修改业务项目文件：否。
- 是否读取敏感文件或完整 transcript：否。
- 是否运行 harness：否。

## 改动文件

- `prototypes/productized-desktop-shell/src/views/OfflineRoleOrchestrationPanel.tsx`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `prototypes/productized-desktop-shell/dist/index.html`
- `prototypes/productized-desktop-shell/dist/assets/index-DDJziFyU.css`
- `prototypes/productized-desktop-shell/dist/assets/index-BakAseJd.js`

## 新增 Evidence

- `evidence/2026-05-30-codex-role-orchestration-offline-closed-loop-v1.md`

## 验证

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 4`。
- `npm run build`：通过。

## 当前可回收口径

可接受为：

- Codex 角色编排离线入口已落到工作台 UI。
- 固定字段派发块解析、缺字段阻止、确认弹层、安全边界文案和桩回传已被离线测试覆盖。

不可接受为：

- 真实多会话自动编排。
- 工作流状态持久化编排。
- 总指导自动计划和连续调度。

## 下一步

建议下一步做“离线编排持久化 v1”：

- prepared role dispatch 写入工作台 workflow state。
- role result handoff 写入 artifact / dispatch completed。
- director review 写入 reviews[] 并推进工作项状态。

仍不接真实 `codex exec resume`，先把工作台自己的账本闭环打通。
