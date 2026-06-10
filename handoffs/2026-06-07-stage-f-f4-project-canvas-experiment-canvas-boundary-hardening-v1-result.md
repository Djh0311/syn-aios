# Handoff: Stage F / F4 Project Canvas / Experiment Canvas Boundary Hardening v1

日期：2026-06-07

## 回收结论

F4 可接受为：

```text
project_canvas_experiment_canvas_boundary_hardening_completed
```

接受范围：

- 项目工作流画布和一级实验 / 模板画布边界硬化完成。
- 一级画布明确是 experiment / template / canvas library 语境。
- 项目画布明确是 project / workflow / authorization / control core 语境。
- 实验画布不写正式项目事实、正式记忆或项目 workflow。
- 项目画布运行和变更仍必须经过 workflow state、控制核心、权限和审计。
- 未新增入口、tab、真实执行按钮、store、sidecar、数据库迁移或 workflow state schema。

## 关键文件

产品改动：

- `prototypes/productized-desktop-shell/src/lib/canvasSurfaceBoundaries.ts`
- `prototypes/productized-desktop-shell/src/views/CanvasView.tsx`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

记录：

- `evidence/2026-06-07-stage-f-f4-project-canvas-experiment-canvas-boundary-hardening-v1.md`
- `handoffs/2026-06-07-stage-f-f4-project-canvas-experiment-canvas-boundary-hardening-v1-result.md`

文档同步：

- `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md` 已修正 F4 推荐任务包状态残留；当前为“已完成”，不是“待执行”。

## 验证结果

通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`

收尾扫描：

- 当前入口 F4 待执行残留扫描无命中。
- 可见源码误导完成态扫描无命中，已排除仅用于测试的 `canvasSurfaceBoundaries.ts` 黑名单常量。

未完成：

- Vite dev server 在沙箱内 `listen EPERM`。
- 非沙箱本地端口启动申请被安全审查拒绝。
- 浏览器 / 真实窗口 smoke 未完成。
- 真实 Tauri / 截图验收未完成，仍交给 G3。

## F5 是否可以开始

可以开始 F5，但必须另开任务包：

```text
Stage F Acceptance
```

F5 可以继承：

- F1 `ProjectWorkflowCanvasReadModel`。
- F2 节点详情三层和 evidence surface。
- F3 `ProjectWorkflowEditBoundary`。
- F4 `CanvasSurfaceBoundary` 和项目 / 实验画布边界文案。

F5 仍不能默认做：

- 不新增功能。
- 不启动 MCP canvas run。
- 不执行真实 Codex。
- 不读写 `/Users/yoyi/.codex`。
- 不把阶段 F 验收冒领为阶段 G 真实 Tauri 验收或中间版本最终验收。

## 仍不能接受为

- 项目画布和实验画布已经合一。
- 独立画布可以写项目 workflow state。
- MCP canvas run 已成为正式 workflow。
- 模板库 / 节点市场完成。
- ComfyUI / n8n / Langflow 复刻完成。
- 真实 worker / Codex 执行完成。
- 真实 send / resume 产品化完成。
- 自动派发或自动重试完成。
- runtime log / diagnostics 完成。
- planned adapters 真实接入完成。
- provider credential / model verification 完成。
- 阶段 F 完成。
- 阶段 G 真实 Tauri 验收完成。
- 中间版本最终验收完成。

## 遗留风险

- 真实窗口 / 截图验收未完成；需要 G3 或后续获得可用浏览器 / Tauri 环境后补。
- 一级实验画布仍有实验运行按钮；F4 只硬化语境，不取消实验能力。后续如要让实验画布发布到项目 workflow，必须另拆迁移和授权任务。
- `src` 和 `src-tauri/src` 中仍有大量既有真实执行 / 敏感关键词边界文案和历史 guard；这不是 F4 新增，但后续真实执行相关任务仍必须逐项审核。
