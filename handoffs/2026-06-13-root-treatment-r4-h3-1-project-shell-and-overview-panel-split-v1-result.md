# Handoff: Root Treatment / R4-H3-1 Project Shell And Overview Panel Split v1 Result

日期：2026-06-13

任务包：`tasks/2026-06-13-root-treatment-r4-h3-1-project-shell-and-overview-panel-split-v1.md`

## 1. Result

H3-1 已实现并通过独立复核线 `STATUS: CLEAR`。

本轮只拆项目壳与项目概览：

- `ProjectsView.tsx` 从 5897 行降到 4867 行。
- 新增 `ProjectWorkspaceShell.tsx`，承接项目详情壳、项目 header、tab nav、项目工具路由、旧 `task-packages` 草稿面板和任务草稿控制器。
- 新增 `ProjectOverviewPanels.tsx`，承接项目概览、智能体承接提示、占位面板和概览 helper。
- `ProjectDetail` 仍从 `ProjectsView.tsx` 导出，作为兼容包装并把现有 `WorkflowCanvas` 通过 slot 传入壳层。
- 任务草稿相关测试 import 仍从 `ProjectsView.tsx` re-export。
- `ProjectsView.tsx` 仍保留 H3-2 / H3-3 的中央画布、右侧详情、治理、记忆和执行面板。

## 2. Validation

已通过：

- `npm run typecheck`
- `npm run test:offline-interaction`，14 passed
- `npm run build`，通过，仅既有 Vite chunk-size warning
- `node scripts/harness/workbench-shape-gate.js --mode check`，pass，0 errors，0 warnings
- `git diff --check`

## 3. Review Input

复核线重点看：

- `ProjectsView.tsx` 是否真实降到 5200 以下，当前为 4867。
- 新增文件是否低于 2000 行，当前 `ProjectWorkspaceShell.tsx` 958，`ProjectOverviewPanels.tsx` 221。
- 是否只搬项目壳、项目概览和旧 task-packages 草稿路由，没有改 UI / CSS / 文案 / 交互。
- 项目 tab 数量和默认 tab 是否保持不变，默认仍是 `"workflow"`。
- `WorkflowCanvas` 是否仍留在 `ProjectsView.tsx`，未提前做 H3-2。
- 右侧治理 / 记忆 / 执行面板是否仍留在 `ProjectsView.tsx`，未提前做 H3-3。
- `ProjectDetail` 和任务草稿测试 import 兼容性是否保持。
- 是否未修改 `AgentView.tsx`、`styles.css`、Rust / Tauri / DB / sidecar / workflow state schema。

## 4. Boundary

- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 未进入 R-U / R3 Level B。
- 未解冻 backlog。

## 5. Not Accepted As

H3-1 不接受为：

- H3 全部完成。
- ProjectsView 拆分全部完成。
- H3-2 中央工作流画布拆分完成。
- H3-3 项目右侧详情 / 治理 / 记忆 / 执行面板拆分完成。
- 项目页 UI 重做完成。
- 真实 Codex 执行产品化完成。
- R-U 后端 util 去重开始或完成。
- R3 Level B 执行、`.codex` 读写或 backlog 解冻。

## 6. Next

主管线可提交实现并写 checkpoint；之后停在 H3-1 复核点，不顺手进入 H3-2。
