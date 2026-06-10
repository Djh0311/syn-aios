# Handoff: Stage F / F3 Controlled Workflow Edit Proposal And Layout Boundary v1

日期：2026-06-07

## 回收结论

F3 可接受为“受控工作流编辑提案和布局边界完成”。

本轮只做前端纯读模型、既有项目工作流画布侧栏 UI、样式和离线测试。React Flow 仍只负责渲染；layout 不持久化；workflow mutation 不直接写 workflow state。

## 本轮完成

- `ProjectWorkflowCanvasReadModel.edit_boundary` 已新增。
- 编辑能力矩阵已覆盖：
  - `view_only`
  - `local_layout_preview`
  - `personal_layout_preference`
  - `workflow_node_mutation`
  - `workflow_edge_mutation`
  - `permission_or_model_mutation`
  - `execution_mutation`
- 项目画布侧栏新增“编辑 / 布局边界”卡片。
- 节点变更、边变更、高风险权限 / 模型变更和执行变更都只显示 proposal / preview / blocked boundary。
- 离线测试覆盖 read model、UI 文案和误导完成态黑名单。
- 当前入口文档已更新：`CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`、`docs/plans/middleware-version-stage-plan-v1.md`、`docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`。

## 验证结果

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过。
- `npm run build`：通过，只有 Vite chunk size warning。
- Vite smoke：沙箱内 `listen EPERM 127.0.0.1:5173`；非沙箱申请被自动拒绝，未绕过。

## 仍不能接受为

- 画布编辑器完成。
- 布局持久化完成。
- workflow edit proposal 持久 store 完成。
- 节点新增 / 删除 / 连线保存 / 拖拽保存完成。
- 真实 worker / Codex 执行完成。
- 真实 send / resume 产品化完成。
- runtime log / diagnostics 完成。
- F4 项目画布 / 实验画布边界硬化完成。
- F5 阶段 F 验收完成。
- 阶段 G 真实 Tauri 验收完成。
- 中间版本最终验收完成。

## 下一步

可以进入 F4：Project Canvas / Experiment Canvas Boundary Hardening。

建议 F4 继续保持窄范围：

- 区分项目工作流画布和一级实验画布。
- 不启动 MCP canvas run。
- 不把实验画布写成正式项目事实、正式记忆或正式 workflow。
- 不新增真实执行。
- 不读写 `/Users/yoyi/.codex`。

## 风险

- 本轮没有真实窗口 / 截图验收；需要留给 G 阶段或后续获得可用浏览器 / Tauri 环境后补。
- `src` 和 `src-tauri/src` 中仍有大量既有 `codex exec`、`/Users/yoyi/.codex`、secret / token 边界文案和历史执行代码路径命中；这些不是本轮新增，但后续真实执行相关任务必须继续逐项审核。
