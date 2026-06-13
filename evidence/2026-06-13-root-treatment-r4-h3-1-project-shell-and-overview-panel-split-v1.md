# Evidence: Root Treatment / R4-H3-1 Project Shell And Overview Panel Split v1

日期：2026-06-13

任务包：`tasks/2026-06-13-root-treatment-r4-h3-1-project-shell-and-overview-panel-split-v1.md`

## 1. Scope

本轮只做 H3-1 项目壳与项目概览拆分：

- 新增 `ProjectOverviewPanels.tsx`，承接项目概览、智能体承接提示、占位面板和概览区 helper。
- 新增 `ProjectWorkspaceShell.tsx`，承接项目详情壳、项目 header、tab nav、项目工具路由、旧 `task-packages` 草稿面板和任务草稿控制器。
- `ProjectsView.tsx` 保留顶层项目选择 / selected tool 状态、`ProjectDetail` 兼容包装、`WorkflowCanvas` 及后续 H3-2 / H3-3 的画布 / 右侧面板实现。
- `ProjectDetail` 和任务草稿相关测试 import 继续从 `ProjectsView.tsx` 导出。

本轮不改 UI / CSS / 文案 / 交互，不改默认 tab，不改项目 tab 数量，不进入 H3-2 / H3-3。

## 2. Shape

`wc -l prototypes/productized-desktop-shell/src/views/ProjectsView.tsx prototypes/productized-desktop-shell/src/views/projects/ProjectWorkspaceShell.tsx prototypes/productized-desktop-shell/src/views/projects/ProjectOverviewPanels.tsx prototypes/productized-desktop-shell/src/views/AgentView.tsx prototypes/productized-desktop-shell/src/styles.css`

结果：

- `ProjectsView.tsx`：4867
- `ProjectWorkspaceShell.tsx`：958
- `ProjectOverviewPanels.tsx`：221
- `AgentView.tsx`：285
- `styles.css`：8464

H3-1 目标是 `ProjectsView.tsx` 5897 -> 5200 以下；本轮实际下降 1030 行。

Shape gate：

- `node scripts/harness/workbench-shape-gate.js --mode check`
- 结果：pass，0 errors，0 warnings
- `ProjectsView.tsx: 4867/4867`

## 3. Validation

已通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
  - `offline interaction tests passed: 14`
  - `r4 page read model settings test passed`
  - `r4 page read model query contract test passed`
  - `r4 page read model runtime test passed`
  - `r4 page selectors test passed`
- `npm run build`
  - 通过
  - 仅既有 Vite chunk-size warning
- `node scripts/harness/workbench-shape-gate.js --mode check`
  - pass，0 errors，0 warnings
- `git diff --check`

## 4. Boundary Scans

`git diff -- prototypes/productized-desktop-shell/src/styles.css prototypes/productized-desktop-shell/src/views/AgentView.tsx`

- 无输出，确认未修改 `styles.css` 和 `AgentView.tsx`。

`rg -n "codex exec|exec resume|/Users/yoyi/.codex|provider credential|full transcript" prototypes/productized-desktop-shell/src/views/ProjectsView.tsx prototypes/productized-desktop-shell/src/views/projects`

- 命中均为既有产品边界文案，例如“不执行真实 Codex / 不写 `/Users/yoyi/.codex` / 不读取完整会话记录”。
- 本轮未新增真实 `codex exec` / `codex exec resume` 调用路径，未读取 credential / full transcript。

## 5. Files Changed

代码：

- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/views/projects/ProjectWorkspaceShell.tsx`
- `prototypes/productized-desktop-shell/src/views/projects/ProjectOverviewPanels.tsx`
- `scripts/harness/workbench-shape-gate.js`

文档：

- `tasks/2026-06-13-root-treatment-r4-h3-1-project-shell-and-overview-panel-split-v1.md`
- `evidence/2026-06-13-root-treatment-r4-h3-1-project-shell-and-overview-panel-split-v1.md`
- `handoffs/2026-06-13-root-treatment-r4-h3-1-project-shell-and-overview-panel-split-v1-result.md`

## 6. Boundary Confirmation

- 未修改 `styles.css`。
- 未修改 `AgentView.tsx`。
- 未修改 Rust / Tauri / DB / sidecar schema / workflow state schema。
- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 未进入 R-U / R3 Level B。
- 未解冻 backlog。

## 7. Review

独立复核线 `019ebf65-d9c0-7410-8cad-820fcf57cdab` 已回交 `STATUS: CLEAR`。

- Review：`evidence/2026-06-13-root-treatment-r4-h3-1-project-shell-and-overview-panel-split-v1-review-v1.md`
- P0 / P1 / P2：无。
