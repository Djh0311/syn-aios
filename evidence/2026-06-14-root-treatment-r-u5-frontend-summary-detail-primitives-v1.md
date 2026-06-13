# Root Treatment / R-U5 Frontend Summary Detail Primitives v1 Evidence

日期：2026-06-14

状态：完成，独立复核 Hilbert `STATUS: CLEAR`。

Planning baseline：`df2ed51`

Task package commit：`8abd89a docs: add r-u5 frontend primitives package`

Implementation commit：`c4335e1 refactor: deduplicate frontend detail primitives`

## 1. 本包目标

本包只把前端重复的 `SummaryTile` / `DetailLine` 展示组件收敛到 `src/components/WorkbenchPrimitives.tsx`，保持行为与视觉零变更。

完成内容：

- 新增 `prototypes/productized-desktop-shell/src/components/WorkbenchPrimitives.tsx`。
- `SummaryTile` 统一渲染 `summary-tile`、`span`、`strong`、`em`。
- `DetailLine` 统一渲染 `detail-line`、`span`、`strong`。
- `OfflineRoleOrchestrationPanel.tsx` 通过 `emptyValue="未登记"` 保持原先 `value || "未登记"` 空值行为。
- `ProjectOverviewPanels.tsx` 和 `projectWorkflowLabels.ts` 保留原导出路径的兼容 re-export。

## 2. 修改范围

代码文件：

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

未修改：

- 未修改 CSS / UI 布局 / 文案 / 信息层级。
- 未修改 Rust / Tauri / DB / sidecar / workflow state schema。
- 未启动 Tauri / Browser / Chrome / Vite dev / 截图工具。
- 未执行真实 `codex exec` / `codex exec resume`。
- 未读写 `/Users/yoyi/.codex`。
- 未解冻 backlog。

## 3. 扫描记录

### 3.1 重复定义扫描

命令：

```bash
rg -n "function SummaryTile|const SummaryTile|function DetailLine|const DetailLine" prototypes/productized-desktop-shell/src/views prototypes/productized-desktop-shell/src/components
```

输出：

```text
prototypes/productized-desktop-shell/src/components/WorkbenchPrimitives.tsx:1:export function SummaryTile({ label, value, hint }: { label: string; value: string; hint: string }) {
prototypes/productized-desktop-shell/src/components/WorkbenchPrimitives.tsx:11:export function DetailLine({ label, value, emptyValue }: { label: string; value: string; emptyValue?: string }) {
```

### 3.2 className 扫描

命令：

```bash
rg -n "summary-tile|detail-line" prototypes/productized-desktop-shell/src/components/WorkbenchPrimitives.tsx prototypes/productized-desktop-shell/src/views
```

输出：

```text
prototypes/productized-desktop-shell/src/components/WorkbenchPrimitives.tsx:3:    <div className="summary-tile">
prototypes/productized-desktop-shell/src/components/WorkbenchPrimitives.tsx:14:    <div className="detail-line">
prototypes/productized-desktop-shell/src/views/projects/ProjectWorkflowSidePanel.tsx:306:    <div className={`project-canvas-detail-line ${item.value_kind ?? "text"}`}>
```

说明：`project-canvas-detail-line` 是不同组件的既有 className，不属于本包 `DetailLine` 重复定义。

## 4. 验证记录

### 4.1 `npm run typecheck`

执行目录：`prototypes/productized-desktop-shell`

```text
> codex-governance-workbench@0.1.0 typecheck
> tsc --noEmit
```

结果：通过。

### 4.2 `npm run test:offline-interaction`

执行目录：`prototypes/productized-desktop-shell`

```text
> codex-governance-workbench@0.1.0 test:offline-interaction
> node scripts/run-offline-interaction-test.mjs

offline interaction tests passed: 14
r4 page read model settings test passed
r4 page read model query contract test passed
r4 page read model runtime test passed
r4 page selectors test passed
```

结果：通过。

### 4.3 `npm run build`

执行目录：`prototypes/productized-desktop-shell`

```text
> codex-governance-workbench@0.1.0 build
> tsc --noEmit && vite build

vite v7.3.3 building client environment for production...
transforming...
✓ 252 modules transformed.
rendering chunks...
computing gzip size...
dist/index.html                   0.59 kB │ gzip:   0.42 kB
dist/assets/index-Cq18P1uG.css  145.61 kB │ gzip:  24.83 kB
dist/assets/index-FoD9VcZg.js   977.17 kB │ gzip: 266.95 kB

(!) Some chunks are larger than 500 kB after minification. Consider:
- Using dynamic import() to code-split the application
- Use build.rollupOptions.output.manualChunks to improve chunking: https://rollupjs.org/configuration-options/#output-manualchunks
- Adjust chunk size limit for this warning via build.chunkSizeWarningLimit.
✓ built in 1.46s
```

结果：通过，仅保留既有 Vite chunk size warning。

### 4.4 Shape gate

命令：

```bash
node scripts/harness/workbench-shape-gate.js --mode check
```

执行目录：`/Users/yoyi/workspace/product-line`

```text
Status: pass
Errors: 0
Warnings: 0
Info: 9
Git HEAD: 8abd89a71fb201ea69d7b0c108c9efc6ee907735

Key metrics:
- lib.rs: 5567 lines (prototypes/productized-desktop-shell/src-tauri/src/lib.rs)
- real_execution_command.rs: 8754 lines (prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs)
- ProjectsView.tsx: 378 lines (prototypes/productized-desktop-shell/src/views/ProjectsView.tsx)
- AgentView.tsx: 285 lines (prototypes/productized-desktop-shell/src/views/AgentView.tsx)
- types.rs: 5229 lines (prototypes/productized-desktop-shell/src-tauri/src/types.rs)
- types.ts: 43 lines (prototypes/productized-desktop-shell/src/lib/types.ts)
- styles.css: 8464 lines (prototypes/productized-desktop-shell/src/styles.css)
- offline-permission-dialog.test.tsx: 3404 lines (prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx)
- Tauri commands: 97 total; 0 in lib.rs
- Sidecar JSON kinds: 14 detected; 0 unknown
```

结果：通过。

### 4.5 `git diff --check`

命令：

```bash
git diff --check
```

输出为空。

结果：通过。

## 5. 当前 git 实物

### 5.1 `git status --short`

```text
 M prototypes/productized-desktop-shell/src/views/HarnessBoardView.tsx
 M prototypes/productized-desktop-shell/src/views/KnowledgeBaseView.tsx
 M prototypes/productized-desktop-shell/src/views/MemoryCenterView.tsx
 M prototypes/productized-desktop-shell/src/views/OfflineRoleOrchestrationPanel.tsx
 M prototypes/productized-desktop-shell/src/views/RunningWorkflowsView.tsx
 M prototypes/productized-desktop-shell/src/views/SkillsBoardView.tsx
 M prototypes/productized-desktop-shell/src/views/projects/ProjectOverviewPanels.tsx
 M prototypes/productized-desktop-shell/src/views/projects/ProjectWorkflowCanvasView.tsx
 M prototypes/productized-desktop-shell/src/views/projects/projectWorkflowLabels.ts
?? prototypes/productized-desktop-shell/src/components/WorkbenchPrimitives.tsx
```

### 5.2 `git diff --stat`

```text
 .../src/views/HarnessBoardView.tsx                   | 11 +----------
 .../src/views/KnowledgeBaseView.tsx                  | 10 +---------
 .../src/views/MemoryCenterView.tsx                   | 10 +---------
 .../src/views/OfflineRoleOrchestrationPanel.tsx      | 20 ++++++--------------
 .../src/views/RunningWorkflowsView.tsx               | 11 +----------
 .../src/views/SkillsBoardView.tsx                    | 11 +----------
 .../src/views/projects/ProjectOverviewPanels.tsx     | 12 +++---------
 .../src/views/projects/ProjectWorkflowCanvasView.tsx | 10 +---------
 .../src/views/projects/projectWorkflowLabels.ts      | 10 +---------
 9 files changed, 16 insertions(+), 89 deletions(-)
```

注：`git diff --stat` 不显示未跟踪的新文件；新文件见 `git status --short`。

## 6. 独立复核

复核线：Hilbert (`019ec1e4-f49c-7d90-9166-ffd8b1bb1a42`)

结论：`STATUS: CLEAR`

复核线确认：

- 公共组件 DOM 结构和 className 与原本一致。
- `SummaryTile` 本地定义归零，只剩公共组件。
- 可合并 `DetailLine` 本地定义归零；兼容 re-export 保持。
- `OfflineRoleOrchestrationPanel` 空值显示仍为“未登记”。
- 未改 UI 文案、布局、CSS、数据语义、Rust / Tauri / DB。
- 验证记录可信。

## 7. 不接受为

本包不接受为 R-U 全部完成、U4 / U-Gate 完成、页面 UI 重做或信息层级调整完成、查重门实现、R3 Level B 执行、真实 Codex 执行或 backlog 解冻。
