# Evidence：Codex 角色编排离线闭环 v1

## 结论

已在桌面壳项目工作流页新增 Codex 角色编排入口。

本轮接受为：

- 工作台能展示总指导、开发线、验证线、回收线四个角色。
- 工作台能解析固定字段的总指导派发块。
- 工作台能生成 `offline-role-dispatch` 确认动作。
- 确认动作不会进入真实 `codex exec resume` 路径。
- 页面能展示角色桩结果和回传总指导摘要。

本轮不接受为：

- 真实多 Codex 会话自动编排已经完成。
- 派发结果已持久化到真实 workflow state。
- 总指导能自动制定计划并连续调度各角色。

## 薄弱点

- 当前是离线闭环入口，不是持久化编排。`offline-role-dispatch` 只在前端确认后设置 notice，不写 `workflow_node_dispatches[]`、handoff、review 或 audit。
- 派发块预览默认使用内置示例；提交时会按表单里的派发块解析，但当前页面没有实时预览更新。
- “角色回传”是桩结果，说明角色接收和回传格式，不代表真实 Codex 会话执行过。
- 当前没有写后端离线派发 / 离线交接 / 离线回收命令。

## 改动内容

新增：

- `src/views/OfflineRoleOrchestrationPanel.tsx`

修改：

- `src/views/ProjectsView.tsx`
- `src/components/PermissionDialog.tsx`
- `src/App.tsx`
- `src/lib/types.ts`
- `src/styles.css`
- `tests/offline-permission-dialog.test.tsx`
- `dist/index.html`
- `dist/assets/index-DDJziFyU.css`
- `dist/assets/index-BakAseJd.js`

## 实现说明

新增的固定派发块字段：

- `派发给`
- `任务名`
- `目标`
- `执行目录`
- `允许读取`
- `允许写入`
- `禁止事项`
- `验收标准`
- `超时`
- `回传要求`

角色映射：

- `总指导` / `director` -> `director`
- `开发线` / `developer` / `codex-dev` / `dev` -> `codex-dev`
- `验证线` / `validation` / `verifier` -> `validation`
- `回收线` / `review` / `reviewer` -> `review`

安全边界：

- 不启动 Codex。
- 不执行 `codex exec resume`。
- 不发送消息。
- 不写 `/Users/yoyi/.codex`。
- 不运行 harness。
- 不写真实 workflow state。

## 边界复核

- 是否执行 `codex exec`：否。
- 是否执行 `codex exec resume`：否。
- 是否写 `/Users/yoyi/.codex`：否。
- 是否写真实 workflow state：否。
- 是否修改业务项目文件：否。
- 是否读取敏感文件或完整 transcript：否。
- 是否运行 harness：否。

## 验证结果

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 4`。
- `npm run build`：通过。

## Subagent 复核

使用 subagent 做了只读复核。

复核结论：

- 未发现离线编排路径会误触真实 `codex exec` / `codex exec resume`。
- 未发现写 `/Users/yoyi/.codex` 或真实 workflow state 的路径。
- 指出早期版本有假按钮和测试未覆盖问题；本轮已删除无实际动作按钮，并补了离线解析 / 确认弹层 / 缺字段阻止 / 桩回传断言。

## 下一步建议

下一步如果要把它从“离线闭环入口”推进到“可用工作流编排”，应新增后端状态命令：

- `prepare_offline_role_dispatch`
- `record_offline_role_result_handoff`
- `record_offline_director_review`

这些命令仍然只写工作台自己的 workflow state，不接真实 `codex exec resume`。
