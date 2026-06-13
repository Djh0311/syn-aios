# Root Treatment / R4-H3-1 Project Shell And Overview Panel Split v1

日期：2026-06-13

状态：已完成。

性质：R4 硬目标 / H3 View 按目标布局区块拆分的 ProjectsView 第 1 包。本包只拆项目壳与项目概览区块，目标是在 H3-4 / H3-5 已完成 AgentView 拆分后，开始降低 `ProjectsView.tsx` 棘轮水位；不做 UI 重做，不改产品行为，不进入画布 / 右侧治理面板拆分。

Planning baseline：`2880ef1`。

## 0. 全局主管理解

新执行正本 `docs/plans/2026-06-13-stage-r-remaining-execution-plan-v1.md` 定义 Stage R 剩余执行顺序：

1. 先完成 R4-H 前端硬目标。
2. 再进入 R-U 后端 util 去重。
3. 再进入 R3 Level B SQLite 真实切换。
4. 最后 R5 文档与蓝图对齐。

正本 §2 仍将 H3-5 写为“已确认执行”，但 `CURRENT.md` 和 git 最新事实显示 H3-5 已完成：implementation commit `a943cee`，checkpoint commit `2880ef1`，独立复核线 Singer `STATUS: CLEAR`。本包按最新事实推进，不重做 H3-5；H3 下一步进入 ProjectsView 拆分序列：H3-1、H3-2、H3-3。

H3 设计稿 `docs/plans/2026-06-13-root-treatment-r4-h3-project-agent-view-layout-block-split-design-v1.md` 对 H3-1 的定义是：

- 抽出 `ProjectWorkspaceShell.tsx`。
- 抽出 `ProjectOverviewPanels.tsx`。
- `ProjectsView.tsx` 只保留项目选择和顶层路由。
- 行为与视觉零变更。

## 1. 目标

完成后：

- `ProjectsView.tsx` 从 5897 行下降到 5200 行以下，至少下降 500 行；若未低于 5200 行，不得收口为完成。
- 新增 `src/views/projects/ProjectWorkspaceShell.tsx`，承接 `ProjectDetail`、项目头部、项目 tab、项目工具路由，以及空状态壳层。
- 新增 `src/views/projects/ProjectOverviewPanels.tsx`，承接 `ProjectOverview`、`ProjectAgentMovedPanel`、`ProjectToolPlaceholder` 和只服务概览 / 壳层的纯展示 helper。
- `ProjectWorkspaceShell.tsx` 同时承接旧 `task-packages` 路由下的 `ProjectWorkflowDraftPanel` 与任务草稿控制器。该路由属于项目壳内的历史 tab 分支，不属于 H3-2 中央画布，也不属于 H3-3 右侧治理 / 记忆 / 执行面板；迁入它是为了达成 H3-1 水位目标，同时保持 UI / 行为不变。
- `ProjectsView.tsx` 保留 `ProjectsView` 顶层状态：selected project root、selected tool、project sessions 派生、空项目 / 项目列表 / 项目详情路由装配。
- 既有 `ProjectGallery`、`ProjectHandoffEvidencePanel`、`ProjectResourcesPanel` 继续复用，不迁回 `ProjectsView.tsx`。
- 视觉、DOM class、项目 tab 数量、默认 tab、按钮文案、交互顺序和测试断言保持不变。

## 2. 当前代码事实

当前结构：

- `ProjectsView.tsx`：5897 行。
- `AgentView.tsx`：285 行，H3-4 / H3-5 已完成。
- `src/views/projects/ProjectGallery.tsx`：项目方块入口已独立。
- `src/views/projects/ProjectReferencePanels.tsx`：交接 / 证据 / 资源面板已独立。

`ProjectsView.tsx` 当前直接包含：

- `ProjectsView`
- `ProjectDetail`
- `ProjectOverview`
- `ProjectAgentMovedPanel`
- `ProjectToolPlaceholder`
- `ProjectWorkflowDraftPanel`
- `WorkflowCanvas`
- 项目画布、右侧详情、治理、记忆、执行和 label helper 等大量后续 H3-2 / H3-3 内容

本包只处理 `ProjectDetail` / `ProjectOverview` / `ProjectAgentMovedPanel` / `ProjectToolPlaceholder` / 旧 `task-packages` 草稿面板及其最小依赖，后续中央画布和右侧面板留给 H3-2 / H3-3。

## 3. 形状影响

预期：

- `ProjectsView.tsx`：5897 -> 5200 以下。
- `ProjectWorkspaceShell.tsx`：低于 2000 行。
- `ProjectOverviewPanels.tsx`：低于 2000 行。
- `styles.css` 不变。
- `AgentView.tsx` 不变。
- Rust / Tauri / DB / sidecar / workflow state schema 不变。
- shape gate 的 `ProjectsView.tsx` waterline 随本包下降。

若新增文件超过 2000 行，本包不得收口；若新增文件接近 1500 行，implementation evidence 必须说明为什么不继续拆。

## 4. 允许范围

允许修改：

- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/views/projects/ProjectWorkspaceShell.tsx`
- `prototypes/productized-desktop-shell/src/views/projects/ProjectOverviewPanels.tsx`
- `scripts/harness/workbench-shape-gate.js`
- 必要的前端离线测试 import / 断言兼容修正。
- 当前任务包、evidence、handoff。

允许新增：

- `src/views/projects/ProjectWorkspaceShell.tsx`
- `src/views/projects/ProjectOverviewPanels.tsx`

## 5. 禁止范围

禁止：

- 修改 UI / CSS / 水墨风格 / 布局 / 文案 / 交互。
- 修改项目 tab 数量。
- 修改默认 tab；当前 `ProjectsView` 默认仍为 `"workflow"`。
- 提前拆 H3-2 画布：`WorkflowCanvas`、React Flow / static fallback / node view / attention strip 不进入本包目标。
- 提前拆 H3-3 右侧详情 / 治理 / 记忆 / 执行面板。
- 修改 `AgentView.tsx` 或 H3-4 / H3-5 已拆文件。
- 修改 Rust / Tauri / DB / sidecar schema / workflow state schema。
- 执行真实 `codex exec` / `codex exec resume`，发送 prompt，读写 `/Users/yoyi/.codex`。
- 启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 进入 R-U、R3 Level B 或解冻 backlog。

## 6. 实现步骤

1. 新建 `ProjectOverviewPanels.tsx`，迁出 `ProjectOverview`、`ProjectAgentMovedPanel`、`ProjectToolPlaceholder`。
2. 将概览区需要的最小纯展示 helper 一并迁出；若 helper 被 H3-2 / H3-3 大量依赖，暂留 `ProjectsView.tsx`，避免反向耦合。
3. 新建 `ProjectWorkspaceShell.tsx`，迁出 `ProjectDetail`，保持项目 header、tab nav 和 selected tool 路由顺序不变。
4. `ProjectWorkspaceShell.tsx` 通过 `workflowPanel` slot 继续渲染现有 `WorkflowCanvas`，但 `WorkflowCanvas` 定义仍保留在 `ProjectsView.tsx`，等待 H3-2。
5. 将旧 `task-packages` 路由下的 `ProjectWorkflowDraftPanel` 与任务草稿控制器迁入 `ProjectWorkspaceShell.tsx`；保持 action payload、权限边界文案、测试 import 兼容 re-export 不变。
6. `ProjectsView.tsx` 改为 import `ProjectWorkspaceShell`，并保留顶层 selected project / selected tool / project sessions 装配。
7. 更新 shape gate `ProjectsView.tsx` waterline 到本包完成后的新低水位。

## 7. 兼容要求

必须保持：

- `ProjectsView` 默认导出 / 命名导出行为不变。
- `filterProjectSessionsForProject` 兼容导出不变。
- `ProjectDetail` 若有外部测试或 import 依赖，必须通过 `ProjectsView.tsx` 兼容 re-export 保持。
- 项目详情 header、tab nav、tab 内容分支顺序不变。
- `ProjectOverview` 内三个动作按钮仍只切换 tab，不触发真实执行。
- `ProjectAgentMovedPanel` 仍只提示会话入口迁到智能体页，不把会话中心搬回项目页。
- `ProjectToolPlaceholder` 仍保持占位语义。

## 8. 验证

必须通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

建议扫描：

- `wc -l prototypes/productized-desktop-shell/src/views/ProjectsView.tsx prototypes/productized-desktop-shell/src/views/projects/*.tsx`
- `git diff -- prototypes/productized-desktop-shell/src/styles.css prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `rg -n "ProjectDetail|ProjectOverview|ProjectAgentMovedPanel|ProjectToolPlaceholder" prototypes/productized-desktop-shell/src/views/ProjectsView.tsx prototypes/productized-desktop-shell/src/views/projects`
- `rg -n "codex exec|exec resume|/Users/yoyi/.codex|provider credential|full transcript" prototypes/productized-desktop-shell/src/views/ProjectsView.tsx prototypes/productized-desktop-shell/src/views/projects`

## 9. 复核要求

复核线重点检查：

- `ProjectsView.tsx` 是否真实下降到 5200 行以下。
- 新增文件是否都低于 2000 行，没有制造新巨型文件。
- 是否只是迁移项目壳 / 项目概览代码，不改 UI / CSS / 文案 / 交互。
- 项目 tab 数量和默认 tab 是否保持不变。
- H3-2 / H3-3 的画布、右侧治理、记忆和执行面板是否未被提前迁移或改语义。
- 旧 `task-packages` 草稿面板是否只是迁入壳层，action payload 和权限边界是否保持不变。
- 是否未修改 `AgentView.tsx`、Rust / Tauri / DB / sidecar / workflow state schema。
- 是否未接触 `.codex`、未执行真实 Codex、未启动 Tauri / Browser / Chrome / Vite dev。

## 10. 不接受为

本包不接受为：

- H3 全部完成。
- ProjectsView 拆分全部完成。
- H3-2 中央工作流画布拆分完成。
- H3-3 项目右侧详情 / 治理 / 记忆 / 执行面板拆分完成。
- 项目页 UI 重做完成。
- 项目 tab 或默认入口调整完成。
- 真实 Codex 执行产品化完成。
- R-U 后端 util 去重开始或完成。
- R3 Level B 执行、`.codex` 读写或 backlog 解冻。

## 11. 停止线

实现完成后必须交给独立复核线复核；主管线不得自审替代复核线结论。

H3-1 完成并 checkpoint 后，下一步才允许进入 H3-2 项目中央工作流画布拆分；不得顺手进入 H3-2 / H3-3 或 R-U。

## 12. 实现记录

实现日期：2026-06-13。

实现结果：

- `ProjectsView.tsx` 从 5897 行降到 4867 行，低于本包目标 5200 行。
- 新增 `src/views/projects/ProjectOverviewPanels.tsx`，承接 `ProjectOverview`、`ProjectAgentMovedPanel`、`ProjectToolPlaceholder` 和概览区纯展示 helper。
- 新增 `src/views/projects/ProjectWorkspaceShell.tsx`，承接项目详情壳、项目 header、tab nav、项目工具路由、旧 `task-packages` 草稿面板和任务草稿控制器。
- `ProjectsView.tsx` 保留 `ProjectsView` 顶层状态、`ProjectDetail` 兼容包装、`WorkflowCanvas` 及后续 H3-2 / H3-3 的画布 / 右侧面板实现。
- `ProjectDetail` 仍从 `ProjectsView.tsx` 导出；任务草稿相关测试 import 通过 `ProjectsView.tsx` re-export 保持兼容。
- `workbench-shape-gate.js` 的 `ProjectsView.tsx` waterline 更新为 4867。

验证已通过：

- `npm run typecheck`
- `npm run test:offline-interaction`，14 passed，含 R4 page read model settings / query contract / runtime / selectors 测试
- `npm run build`，通过，仅既有 Vite chunk size warning
- `node scripts/harness/workbench-shape-gate.js --mode check`，pass，0 errors，0 warnings
- `git diff --check`

边界确认：

- 未修改 `styles.css`。
- 未修改 `AgentView.tsx`。
- 未修改 Rust / Tauri / DB / sidecar schema / workflow state schema。
- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 未进入 R-U / R3 Level B，未解冻 backlog。

复核结论：

- 独立复核线 `019ebf65-d9c0-7410-8cad-820fcf57cdab` 已回交 `STATUS: CLEAR`。
- P0 / P1 / P2：无。
- Review：`evidence/2026-06-13-root-treatment-r4-h3-1-project-shell-and-overview-panel-split-v1-review-v1.md`
- Evidence：`evidence/2026-06-13-root-treatment-r4-h3-1-project-shell-and-overview-panel-split-v1.md`
- Handoff：`handoffs/2026-06-13-root-treatment-r4-h3-1-project-shell-and-overview-panel-split-v1-result.md`
