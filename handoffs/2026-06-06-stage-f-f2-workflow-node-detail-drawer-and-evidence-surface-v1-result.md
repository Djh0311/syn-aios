# Handoff：Stage F / F2 Workflow Node Detail Drawer And Evidence Surface v1

日期：2026-06-06

## 1. 结果

F2 已完成，可接受为：

```text
workflow_node_detail_drawer_and_evidence_surface_completed
```

接受范围：

- 项目工作流节点详情 / evidence surface 产品化完成。
- 节点详情按用户摘要、项目主管信息、技术详情三层展示。
- 任务包、任务记忆包、权限、readback、失败、audit、evidence、handoff 和 director review 已进入既有项目页节点详情面板。
- 所有信息都以摘要 / 引用展示，不铺全文。
- 高风险动作仍只显示边界和“需确认弹层”，不在详情中直接批准。
- 未新增一级入口、右侧顶级入口、项目页 tab、真实执行按钮、store、sidecar、数据库迁移或 workflow state schema。

不能接受为：

- F3 受控编辑 / 布局边界完成。
- 画布编辑器完成。
- 真实 worker / Codex 执行完成。
- 真实 send / resume 产品化完成。
- 自动派发、自动重试、runtime log、diagnostics 或阶段 G 真实 Tauri 验收完成。
- 阶段 F 完成。
- 中间版本最终验收完成。

## 2. 关键文件

产品改动：

- `prototypes/productized-desktop-shell/src/lib/projectCanvas.ts`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

记录：

- `evidence/2026-06-06-stage-f-f2-workflow-node-detail-drawer-and-evidence-surface-v1.md`
- `handoffs/2026-06-06-stage-f-f2-workflow-node-detail-drawer-and-evidence-surface-v1-result.md`

入口同步：

- `tasks/2026-06-06-stage-f-f2-workflow-node-detail-drawer-and-evidence-surface-v1.md`
- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`

## 3. 验证结果

通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`

扫描：

- 禁止完成态文案扫描无命中。
- 详情边界扫描命中均为否定边界、既有真实 0 计数分支或测试禁止词。
- 未发现把 readback unavailable / failed 显示成真实 0 条结果。

未完成：

- Vite dev server 在沙箱内 `listen EPERM`。
- escalated 本地端口启动申请被安全审查拒绝。
- 浏览器 / 真实窗口 smoke 未完成。
- 真实 Tauri / 截图验收未完成，仍交给 G3。

## 4. F3 是否可以开始

可以开始编写 / 执行 F3，但必须另开任务包或由用户明确指定：

```text
F3 Controlled Workflow Edit Proposal And Layout Boundary
```

F3 可以继承：

- `ProjectWorkflowCanvasReadModel`
- `ProjectCanvasDetailSection.layer`
- F2 三层节点详情结构
- task package / memory packet / permission / readback / audit / evidence / handoff 的摘要 / 引用展示边界
- React Flow 只读渲染边界

F3 仍不能默认做：

- 不把 React Flow 当事实源。
- 不直接保存拖拽布局、连线、新增节点或删除节点。
- 不执行真实 Codex。
- 不启动真实 worker。
- 不读写 `/Users/yoyi/.codex`，除非用户对具体任务重新明确授权。
- 不把详情面板变成治理后台。
- 不展示任务包全文、audit 全文、transcript / rollout 全文、raw workflow state、raw sidecar 或 raw log。

## 5. 遗留风险

- 浏览器 / Tauri 截图验收未完成，不能把 F2 说成真实窗口验收完成。
- F2 仍是前端读模型和 UI 层产品化；如果后续需要跨页面复用、导出或后端统一 snapshot，必须另拆后端 read model 任务。
- 详情层级已经收敛，但信息仍然密集；F3 做编辑提案边界时要避免把右侧详情扩成治理后台。
- runtime log、diagnostics、自动重试和真实 Tauri 验收仍属于 G 阶段，不应提前混进 F3。
