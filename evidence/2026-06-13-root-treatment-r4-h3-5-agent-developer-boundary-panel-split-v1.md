# Evidence: Root Treatment / R4-H3-5 Agent Developer Boundary Panel Split v1

日期：2026-06-13

状态：已完成。

任务包：`tasks/2026-06-13-root-treatment-r4-h3-5-agent-developer-boundary-panel-split-v1.md`

## 1. 范围

本轮只拆 `AgentView.tsx` 中的开发者边界面板和相关展示 helper，不改普通会话外壳、不改 UI/CSS、不改真实执行后端、不进入 ProjectsView 拆分。

## 2. 改动摘要

- `AgentView.tsx` 从 1974 行降到 285 行，只保留顶层数据派生、会话选择状态、`AgentSessionCenter` 装配、兼容导出和 `developerDetails` 插槽输入。
- 新增 `src/views/agent/AgentDeveloperPanels.tsx`：开发者 details 总容器，保持原面板渲染顺序。
- 新增 `src/views/agent/AgentExecutionPanels.tsx`：`CodexControlEntryPanel`、`UnifiedExecutionStatusPanel`。
- 新增 `src/views/agent/AgentAdapterBoundaryPanels.tsx`：adapter capability、provider availability、session operation boundary。
- 新增 `src/views/agent/AgentContinuationBoundaryPanels.tsx`：session continuation、controlled continuation、H2 readiness / decision、runtime attention、I5 diagnostics。
- 新增 `src/views/agent/agentLabels.ts`：智能体页开发者面板 label / tone / grouping helper。
- `scripts/harness/workbench-shape-gate.js` 的 `AgentView.tsx` waterline 更新为 285。

## 3. 行数证据

`wc -l prototypes/productized-desktop-shell/src/views/AgentView.tsx prototypes/productized-desktop-shell/src/views/agent/AgentDeveloperPanels.tsx prototypes/productized-desktop-shell/src/views/agent/AgentExecutionPanels.tsx prototypes/productized-desktop-shell/src/views/agent/AgentAdapterBoundaryPanels.tsx prototypes/productized-desktop-shell/src/views/agent/AgentContinuationBoundaryPanels.tsx prototypes/productized-desktop-shell/src/views/agent/agentLabels.ts`

结果：

- `AgentView.tsx`：285
- `AgentDeveloperPanels.tsx`：102
- `AgentExecutionPanels.tsx`：518
- `AgentAdapterBoundaryPanels.tsx`：185
- `AgentContinuationBoundaryPanels.tsx`：523
- `agentLabels.ts`：559

新增文件均低于 2000 行。

## 4. 验证

已通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
  - `offline interaction tests passed: 14`
  - `r4 page read model settings test passed`
  - `r4 page read model query contract test passed`
  - `r4 page read model runtime test passed`
  - `r4 page selectors test passed`
- `npm run build`
  - 通过，仅既有 Vite chunk size warning
- `node scripts/harness/workbench-shape-gate.js --mode check`
  - `Status: pass`
  - `Errors: 0`
  - `Warnings: 0`
  - `AgentView.tsx: 285/285`
- `git diff --check`

## 5. 边界扫描

- `git diff -- prototypes/productized-desktop-shell/src/styles.css prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`：无输出，未改 UI/CSS 和 ProjectsView。
- `rg -n "function .*Panel|CodexControlEntryPanel|UnifiedExecutionStatusPanel|ProviderAvailabilityPanel|SessionContinuationPreviewPanel|RuntimeSessionAttentionPanel|AdapterSdkCliDiagnosticsPanel|SessionOperationBoundaryPanel" prototypes/productized-desktop-shell/src/views/AgentView.tsx prototypes/productized-desktop-shell/src/views/agent`：目标面板已迁移到 `src/views/agent/*`，`AgentView.tsx` 不再直接承载这些函数。

## 6. 边界确认

- 未修改 `styles.css`。
- 未修改 `ProjectsView.tsx`。
- 未修改 Rust / Tauri / DB / sidecar schema / workflow state schema。
- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript / rollout。
- 未启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 未进入 R3 Level B。
- 未解冻 backlog。

## 7. 外部脏文件

本轮未触碰以下既有外部脏文件：

- `AGENTS.md`
- `backlog.md`
- `docs/own-agent-and-company-vision-v1.md`
- `docs/workbench-architecture-principles-v1.md`
- `handoffs/2026-06-13-r4-hard-targets-execution-plan-supervisor-v1.md`

## 8. 不接受为

本轮不接受为：

- H3 全部完成。
- ProjectsView 拆分完成。
- AgentView UI 重做完成。
- 普通对话外壳重做、分页、虚拟滚动、归档隔离、subagent 折叠或直读数据库常驻完成。
- 真实 Codex 执行产品化完成。
- planned adapters 真实接入。
- provider credential / model verification 完成。
- 自动重试、stop、restart 或 diagnostics 执行能力完成。
- R3 Level B 执行、`.codex` 读写或 backlog 解冻。

## 9. 复核状态

独立复核线 Singer 已回交 `STATUS: CLEAR`。

- P0 / P1 / P2：无。
- 复核记录：`evidence/2026-06-13-root-treatment-r4-h3-5-agent-developer-boundary-panel-split-v1-review-singer-v1.md`
- 复核线验证通过 `git diff --check`、`node scripts/harness/workbench-shape-gate.js --mode check`、`npm run typecheck` 和 `npm run test:offline-interaction`。

复核线备注：工作树另有未声明未跟踪文件 `docs/workbench-architecture-principles-v1.md`；主管线提交时排除，未计入本包。
