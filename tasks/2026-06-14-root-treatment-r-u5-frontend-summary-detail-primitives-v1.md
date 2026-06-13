# Root Treatment / R-U5 Frontend Summary Detail Primitives v1

日期：2026-06-14

状态：已完成。

性质：R-U 前端 util / component 去重。本包只把重复 `SummaryTile` / `DetailLine` 纯展示组件收敛到 `src/components/`；严格行为和视觉零变更。

Planning baseline：`df2ed51`。

## 0. 主管线理解

用户要求本夜目标先完成 U5：

- 前端包，不跑 cargo。
- 把 `DetailLine` / `SummaryTile` 合并到 `src/components/`。
- 验证必须包含 `npm run typecheck`、`npm run test:offline-interaction`、`npm run build`、shape gate、`git diff --check`。
- 独立复核 CLEAR 后提交 implementation commit，再写 checkpoint，停在 U5 复核点。
- 不改 UI 文案、布局、视觉风格或信息层级。

## 1. 当前扫描事实

重复定义：

- `SummaryTile`
  - `views/SkillsBoardView.tsx`
  - `views/HarnessBoardView.tsx`
  - `views/RunningWorkflowsView.tsx`
- `DetailLine`
  - `views/projects/ProjectOverviewPanels.tsx`
  - `views/projects/ProjectWorkflowCanvasView.tsx`
  - `views/projects/projectWorkflowLabels.ts`
  - `views/MemoryCenterView.tsx`
  - `views/KnowledgeBaseView.tsx`
  - `views/OfflineRoleOrchestrationPanel.tsx`

差异：

- 大多数 `DetailLine` 为 `<div className="detail-line"><span>{label}</span><strong>{value}</strong></div>`。
- `OfflineRoleOrchestrationPanel.tsx` 的 `DetailLine` 对空值显示 `value || "未登记"`；本包必须保留该行为。
- `projectWorkflowLabels.ts` 当前用 `createElement` 生成同样结构；可改为 re-export 公共组件，同时保留公开导出名。

## 2. 目标

完成后：

- 新增 `prototypes/productized-desktop-shell/src/components/WorkbenchPrimitives.tsx`。
- 导出：

```tsx
export function SummaryTile({ label, value, hint }: { label: string; value: string; hint: string }) { ... }
export function DetailLine({ label, value, emptyValue }: { label: string; value: string; emptyValue?: string }) { ... }
```

- `SummaryTile` 仍渲染 `summary-tile`、`span`、`strong`、`em`。
- `DetailLine` 仍渲染 `detail-line`、`span`、`strong`。
- `OfflineRoleOrchestrationPanel.tsx` 使用 `emptyValue="未登记"`，保持原空值展示。
- `projectWorkflowLabels.ts` 保留 `DetailLine` 导出，避免项目工作流组件 import 链大改。

## 3. 允许范围

允许修改：

- `prototypes/productized-desktop-shell/src/components/WorkbenchPrimitives.tsx`
- `prototypes/productized-desktop-shell/src/views/SkillsBoardView.tsx`
- `prototypes/productized-desktop-shell/src/views/HarnessBoardView.tsx`
- `prototypes/productized-desktop-shell/src/views/RunningWorkflowsView.tsx`
- `prototypes/productized-desktop-shell/src/views/MemoryCenterView.tsx`
- `prototypes/productized-desktop-shell/src/views/KnowledgeBaseView.tsx`
- `prototypes/productized-desktop-shell/src/views/OfflineRoleOrchestrationPanel.tsx`
- `prototypes/productized-desktop-shell/src/views/projects/ProjectOverviewPanels.tsx`
- `prototypes/productized-desktop-shell/src/views/projects/ProjectWorkflowCanvasView.tsx`
- `prototypes/productized-desktop-shell/src/views/projects/projectWorkflowLabels.ts`
- 本任务包、对应 evidence / handoff / review evidence、checkpoint 入口文档。

允许的代码变化仅限：

- 增加公共组件 import。
- 删除本地重复组件定义。
- 保留 `projectWorkflowLabels.ts` 的 `DetailLine` 兼容导出。
- 为 `OfflineRoleOrchestrationPanel.tsx` 传入 `emptyValue="未登记"`。

## 4. 禁止范围

禁止：

- 修改页面布局、文案、className、视觉风格。
- 修改数据读取、状态派生、导航行为。
- 修改 CSS。
- 修改 Rust / Tauri / DB / sidecar / workflow state schema。
- 启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 执行真实 `codex exec` / `codex exec resume`。
- 读写 `/Users/yoyi/.codex`。
- 解冻 backlog。

## 5. 停止线

若抽取公共组件导致以下任一情况，必须停止：

- 页面 DOM 结构或 className 需要变化才能通过。
- `OfflineRoleOrchestrationPanel` 的空值 fallback 不能保持。
- 项目工作流组件 import 链需要大规模重排。
- 需要改 CSS / UI / 数据语义。

发生停止时，保留不能安全合并的组件为 deferred，不硬合。

## 6. 验证

必须通过并在 evidence 粘贴原始尾部输出：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

必须扫描：

- `rg -n "function SummaryTile|const SummaryTile|function DetailLine|const DetailLine" prototypes/productized-desktop-shell/src/views prototypes/productized-desktop-shell/src/components`
- `rg -n "summary-tile|detail-line" prototypes/productized-desktop-shell/src/components/WorkbenchPrimitives.tsx prototypes/productized-desktop-shell/src/views`

## 7. 复核判据

独立复核线必须确认：

- 公共组件 DOM 结构和 className 与原本一致。
- `SummaryTile` 本地定义归零，只剩公共组件。
- 可合并 `DetailLine` 本地定义归零；`projectWorkflowLabels.ts` 只保留兼容 re-export。
- `OfflineRoleOrchestrationPanel` 空值显示仍为“未登记”。
- 未改 UI 文案、布局、CSS、数据语义、Rust / Tauri / DB。
- 验证记录可信。

## 8. 不接受为

本包不接受为：

- R-U 全部完成。
- U4 / U-Gate 完成。
- 页面 UI 重做或信息层级调整完成。
- 查重门实现。
- R3 Level B 执行。
- 真实 Codex 执行。
- backlog 解冻。

## 9. 停止点

任务包已完成；实现经独立复核 Hilbert `STATUS: CLEAR`，implementation commit 为 `c4335e1`，checkpoint commit 待写入。
