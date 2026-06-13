# Root Treatment / R-U5 Frontend Summary Detail Primitives v1 Result

日期：2026-06-14

状态：完成，独立复核 `STATUS: CLEAR`。

Planning baseline：`df2ed51`

Task package commit：`8abd89a docs: add r-u5 frontend primitives package`

复核线：Hilbert (`019ec1e4-f49c-7d90-9166-ffd8b1bb1a42`)

## 1. 完成内容

本包只做前端纯展示组件去重：

- 新增 `prototypes/productized-desktop-shell/src/components/WorkbenchPrimitives.tsx`。
- 抽出公共 `SummaryTile`，保留 `summary-tile`、`span`、`strong`、`em` 结构。
- 抽出公共 `DetailLine`，保留 `detail-line`、`span`、`strong` 结构。
- `OfflineRoleOrchestrationPanel.tsx` 通过 `emptyValue="未登记"` 保持原空值 fallback。
- `ProjectOverviewPanels.tsx` 和 `projectWorkflowLabels.ts` 保留兼容 re-export，避免项目工作流面板 import 链大改。

## 2. 修改文件

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

## 3. 验证

主管线通过：

- `npm run typecheck`
- `npm run test:offline-interaction`：`offline interaction tests passed: 14`
- `npm run build`：通过，仅既有 Vite chunk size warning
- `node scripts/harness/workbench-shape-gate.js --mode check`：`Status: pass`，`Errors: 0`，`Warnings: 0`
- `git diff --check`

复核线 Hilbert 通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

## 4. 复核结论

Hilbert 结论：`STATUS: CLEAR`

P0：无。

P1：无。

P2：无。

复核确认公共组件 DOM/className 与原本一致，重复定义归零，`OfflineRoleOrchestrationPanel` 的“未登记”空值 fallback 保持，未触及 CSS、Rust/Tauri、DB、sidecar、workflow state schema 或数据语义文件。

## 5. 边界确认

本包没有：

- 修改 UI 文案、布局、视觉风格或信息层级。
- 修改 CSS。
- 修改 Rust / Tauri / DB / sidecar / workflow state schema。
- 启动 Tauri / Browser / Chrome / Vite dev / 截图工具。
- 执行真实 `codex exec` / `codex exec resume`。
- 读写 `/Users/yoyi/.codex`。
- 解冻 backlog。

## 6. 不接受为

本包不接受为 R-U 全部完成、U4 / U-Gate 完成、页面 UI 重做或信息层级调整完成、查重门实现、R3 Level B 执行、真实 Codex 执行或 backlog 解冻。

## 7. 下一步

按用户夜间目标，U5 收口后继续 U4：扫描 normalize / normalization 重复实现，只合并规则相同且不触及 store 业务语义的 helper；规则不同或会改变业务含义的项记录 deferred，不硬合。
