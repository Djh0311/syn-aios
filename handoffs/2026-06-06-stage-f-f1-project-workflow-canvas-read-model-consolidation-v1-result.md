# Handoff：Stage F / F1 Project Workflow Canvas Read Model Consolidation v1

日期：2026-06-06

## 1. 结果

F1 已完成，可接受为：

```text
project_workflow_canvas_read_model_consolidation_completed
```

接受范围：

- 项目工作流画布读模型收敛完成。
- 主画布使用统一 `ProjectWorkflowCanvasReadModel` 展示节点、边、状态、badge、attention 和状态原因。
- 节点详情侧栏显示任务包、任务记忆包、权限、readback、audit、evidence、handoff 和 director review 的摘要 / 引用。
- React Flow 仍只是渲染映射，不是事实源。
- 没有新增入口、确认动作、真实执行按钮、workflow state schema、store、sidecar 或数据库迁移。

不能接受为：

- F2 节点详情完整抽屉完成。
- 画布编辑器完成。
- 真实 worker / Codex 执行完成。
- 真实 send / resume 产品化完成。
- 自动派发产品化完成。
- 自动重试、runtime log、diagnostics、真实 Tauri 验收或阶段 F 完成。

## 2. 关键文件

产品改动：

- `prototypes/productized-desktop-shell/src/lib/projectCanvas.ts`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

记录：

- `evidence/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1.md`
- `handoffs/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1-result.md`

## 3. 验证结果

通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`

扫描：

- 禁止完成态文案扫描无命中。
- readback 0 条结果相关命中均为否定边界、guard 或测试断言，未发现误把 unavailable 写成真实 0。
- `codex exec resume` 命中均为既有权限边界、fixture 或禁止 / 需确认文案；F1 未新增真实执行路径。

未完成：

- Vite dev server 在沙箱内 `listen EPERM`。
- escalated 本地端口启动申请被安全审查拒绝。
- 普通浏览器 / 截图 smoke 未完成。
- 真实 Tauri / 截图验收未完成，仍交给 G3。

## 4. F2 是否可以开始

可以开始 F2，但必须另开任务包。

F2 可以继承：

- `ProjectWorkflowCanvasReadModel.status_reason`
- `ProjectWorkflowCanvasReadModel.attention_items[]`
- 节点详情中的 task package / memory packet / permission / readback / audit / evidence / handoff 摘要结构
- React Flow 只读渲染边界

F2 仍不能做：

- 不执行真实 Codex。
- 不启动真实 worker。
- 不把详情抽屉做成可直接批准高风险动作的后台。
- 不展示任务包全文、audit 全文、transcript / rollout 全文、raw workflow state 或 raw log。
- 不把 readback unavailable 显示成真实 0 条结果。

## 5. 遗留风险

- 项目画布现在已有 status reason 和 attention 摘要，但 F2 仍需要把节点详情层级继续产品化，避免侧栏信息密度过高。
- 本轮未改 Rust，因此 F1 的画布模型仍是前端纯派生；当前够用，但后续如要跨页面复用或导出，需要评估是否上移为后端正式 read model。
- 浏览器 / Tauri 截图验收未完成，不应把本轮说成真实窗口验收完成。
- runtime log、diagnostics 和真实 Tauri 验收仍属于 G 阶段，不能挤进 F1/F2 口径。
