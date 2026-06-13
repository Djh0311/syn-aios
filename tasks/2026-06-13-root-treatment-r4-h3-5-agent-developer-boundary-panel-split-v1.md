# Root Treatment / R4-H3-5 Agent Developer Boundary Panel Split v1

日期：2026-06-13

状态：已完成。

性质：R4 硬目标 / 批二 `AgentView` 优先拆分的第 2 包。本包只拆智能体页开发者边界面板和相关展示 helper，目标是在 H3-4 已完成的普通对话区拆分基础上继续降低 `AgentView.tsx` 棘轮水位；不做 UI 重做，不改真实执行逻辑。

Planning baseline：`8b39f0d0472fbb07774e2d4f36a48988ad9c5624`。

## 0. 全局主管理解

用户已确认 H3 设计稿，并指定执行顺序：

- 先 AgentView：H3-4、H3-5。
- 后 ProjectsView：H3-1、H3-2、H3-3。

H3-4 已完成普通对话区拆分，`AgentView.tsx` 当前为 1974 行，新增 `AgentConversationShell.tsx`、`AgentSessionList.tsx`、`AgentChatComposer.tsx`。H3-5 接在 H3-4 后面，只处理仍留在 `AgentView.tsx` 的开发者详情面板：Codex 控制入口、统一执行链路、adapter / provider / session operation、continuation / H2 authorization、runtime attention、I5 diagnostics 以及这些面板使用的纯展示 label / tone / grouping helper。

本包不处理普通会话外壳，不改变“选择项目、选择对话、显示对话、输入任务”的普通路径；不进入 ProjectsView 拆分。

## 1. 目标

完成后：

- `AgentView.tsx` 从 1974 行下降到 1000 行以下，至少下降 900 行；若未低于 1000 行，不得收口为完成。
- 新增 `src/views/agent/AgentDeveloperPanels.tsx`，承接开发者 details 总容器和面板编排。
- 新增 `src/views/agent/AgentExecutionPanels.tsx`，承接 `CodexControlEntryPanel`、`UnifiedExecutionStatusPanel` 以及真实执行产品命令相关展示。
- 新增 `src/views/agent/AgentAdapterBoundaryPanels.tsx`，承接 `AgentAdapterCapabilityPanel`、`ProviderAvailabilityPanel`、`SessionOperationBoundaryPanel`。
- 新增 `src/views/agent/AgentContinuationBoundaryPanels.tsx`，承接 `SessionContinuationPreviewPanel`、`ControlledSessionContinuationPanel`、`H2RealResumeAuthorizationPanel`、`H2RealResumeExecutionDecisionPanel`、`RuntimeSessionAttentionPanel`、`AdapterSdkCliDiagnosticsPanel`。
- 新增 `src/views/agent/agentLabels.ts`，承接智能体页开发者面板使用的纯展示 label / tone / grouping helper。
- `AgentView.tsx` 保留顶层数据派生、会话选择状态、`AgentSessionCenter` 装配、兼容导出和 `developerDetails` 插槽输入。
- 视觉、DOM class、按钮文案、交互顺序、权限 guard 和当前测试断言保持不变。

## 2. 当前代码事实

H3-4 后当前结构：

- `AgentView.tsx`：1974 行。
- `AgentConversationShell.tsx`：925 行。
- `AgentSessionList.tsx`：253 行。
- `AgentChatComposer.tsx`：185 行。

`AgentView.tsx` 当前仍直接包含：

- `CodexControlEntryPanel`
- `UnifiedExecutionStatusPanel`
- `AgentAdapterCapabilityPanel`
- `ProviderAvailabilityPanel`
- `SessionContinuationPreviewPanel`
- `ControlledSessionContinuationPanel`
- `H2RealResumeAuthorizationPanel`
- `H2RealResumeExecutionDecisionPanel`
- `RuntimeSessionAttentionPanel`
- `AdapterSdkCliDiagnosticsPanel`
- `SessionOperationBoundaryPanel`
- 大量 `*Label` / `*Tone` / grouping helper

这些内容都属于开发者边界详情，不应继续占用 `AgentView.tsx` 主文件。

## 3. 形状影响

预期：

- `AgentView.tsx`：1974 -> 1000 以下。
- 新增 `.tsx` / `.ts` 文件均低于 2000 行。
- `AgentConversationShell.tsx` 不增长为新巨型文件；原则上不改或只做必要 import / prop 对接。
- `AgentSessionList.tsx` 和 `AgentChatComposer.tsx` 不改。
- `styles.css` 不变。
- `ProjectsView.tsx` 不变。
- Rust / Tauri / DB / sidecar / workflow state schema 不变。
- shape gate 的 `AgentView.tsx` waterline 随本包下降。

若某个新增文件接近 1500 行，implementation evidence 必须说明为什么不继续拆；若新增文件超过 2000 行，本包不得收口。

## 4. 允许范围

允许修改：

- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/views/agent/AgentDeveloperPanels.tsx`
- `prototypes/productized-desktop-shell/src/views/agent/AgentExecutionPanels.tsx`
- `prototypes/productized-desktop-shell/src/views/agent/AgentAdapterBoundaryPanels.tsx`
- `prototypes/productized-desktop-shell/src/views/agent/AgentContinuationBoundaryPanels.tsx`
- `prototypes/productized-desktop-shell/src/views/agent/agentLabels.ts`
- `scripts/harness/workbench-shape-gate.js`
- 必要的前端离线测试 import / 断言兼容修正。
- 当前任务包、evidence、handoff。

允许新增：

- `src/views/agent/AgentDeveloperPanels.tsx`
- `src/views/agent/AgentExecutionPanels.tsx`
- `src/views/agent/AgentAdapterBoundaryPanels.tsx`
- `src/views/agent/AgentContinuationBoundaryPanels.tsx`
- `src/views/agent/agentLabels.ts`

## 5. 禁止范围

禁止：

- 修改 UI / CSS / 水墨风格 / 布局 / 文案 / 交互。
- 改变智能体页滚动策略。
- 改 H3-4 已拆出的普通对话路径语义。
- 改 K2 / J1 真实执行权限逻辑。
- 新增裸控制台或绕过 Product Command / 权限 / 审计。
- 修改 adapter / provider / credential / model 验证边界。
- 把 planned adapters 显示成可执行。
- 修改 `ProjectsView.tsx` 结构。
- 修改 Rust / Tauri / DB / sidecar schema / workflow state schema。
- 执行真实 `codex exec` / `codex exec resume`，发送 prompt，读写 `/Users/yoyi/.codex`。
- 启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 进入 R3 Level B 或解冻 backlog。

## 6. 实现步骤

1. 新建 `AgentDeveloperPanels.tsx`，定义 `AgentDeveloperPanels` 总容器，接收 H3-5 所需 props，并按当前顺序渲染所有开发者详情面板。
2. 将 `CodexControlEntryPanel`、`UnifiedExecutionStatusPanel` 迁到 `AgentExecutionPanels.tsx`，保持 Product Command preview / prepare / confirm / Phase A 行为和按钮状态不变。
3. 将 adapter / provider / operation 三类面板迁到 `AgentAdapterBoundaryPanels.tsx`，保持 planned / unavailable / no credential / model unverified 文案和 tone 不变。
4. 将 continuation / H2 readiness / runtime attention / diagnostics 面板迁到 `AgentContinuationBoundaryPanels.tsx`，保持 `result_count=null` 展示语义和 readback unavailable / failed / timed out 文案不变。
5. 将纯展示 helper 迁到 `agentLabels.ts`；如果 helper 只被一个面板使用，可以留在对应 panel 文件，避免反向依赖。
6. `AgentView.tsx` 改为 import `AgentDeveloperPanels`，并在 `developerDetails` 插槽中只渲染该总容器。
7. 保留 `AgentView.tsx` 顶层数据派生：adapter descriptors、session operations、provider availability、session continuation previews、H2 readiness / decision surface、project dispatch / attempt count。
8. 更新 shape gate `AgentView.tsx` waterline 到本包完成后的新低水位。

## 7. 兼容要求

必须保持：

- `AgentView` 默认导出 / 命名导出行为不变。
- `AgentSessionCenter`、`filterAgentSessions`、`softwareGroupsForSessions`、`ChatTranscript`、`TranscriptTimeline` 兼容导出不变。
- `AgentConversationShell` 的 `developerDetails` 插槽使用方式不变。
- `CodexControlEntryPanel` 内部 prompt body 保存策略不变：任务正文不进入 sidecar、runtime log、audit 或记忆。
- Phase B 按钮和 guard 语义不变：本包只搬代码，不放宽真实执行条件。
- planned adapters 仍只展示 planned / unavailable / no credential / model unverified 等边界。
- readback unavailable / failed / timed out 等 unknown-result 状态继续显示为未知 / 不可用，不显示成真实 0 条结果。

## 8. 验证

必须通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

建议扫描：

- `wc -l prototypes/productized-desktop-shell/src/views/AgentView.tsx prototypes/productized-desktop-shell/src/views/agent/*.tsx prototypes/productized-desktop-shell/src/views/agent/*.ts`
- `rg -n "function .*Panel|CodexControlEntryPanel|UnifiedExecutionStatusPanel|ProviderAvailabilityPanel|SessionContinuationPreviewPanel|RuntimeSessionAttentionPanel|AdapterSdkCliDiagnosticsPanel|SessionOperationBoundaryPanel" prototypes/productized-desktop-shell/src/views/AgentView.tsx prototypes/productized-desktop-shell/src/views/agent`
- `git diff -- prototypes/productized-desktop-shell/src/styles.css prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `rg -n "codex exec|exec resume|/Users/yoyi/.codex|provider credential|full transcript" prototypes/productized-desktop-shell/src/views/AgentView.tsx prototypes/productized-desktop-shell/src/views/agent`

## 9. 复核要求

复核线重点检查：

- `AgentView.tsx` 是否真实下降到 1000 行以下。
- 新增文件是否都低于 2000 行，且没有把旧巨型文件简单转移成新巨型文件。
- 开发者面板是否只是迁移，不改 UI / CSS / 文案 / 交互。
- `AgentDeveloperPanels` 是否保持当前面板顺序。
- `CodexControlEntryPanel` 是否仍不持久化 prompt body，且没有绕过 Product Command / 权限 / 审计。
- Phase B / real Codex guard 是否未被放宽。
- adapter / provider / credential / model 边界是否未被改成可执行或已验证。
- readback unknown-result 是否没有被渲染为 0。
- 是否未修改 `ProjectsView.tsx`、Rust / Tauri / DB / sidecar / workflow state schema。
- 是否未接触 `.codex`、未执行真实 Codex、未启动 Tauri / Browser / Chrome / Vite dev。

## 10. 不接受为

本包不接受为：

- H3 全部完成。
- ProjectsView 拆分完成。
- AgentView UI 重做完成。
- 普通对话外壳重做、分页、虚拟滚动、归档隔离、subagent 折叠或直读数据库常驻完成。
- 真实 Codex 执行产品化完成。
- planned adapters 真实接入。
- provider credential / model verification 完成。
- 自动重试、stop、restart 或 diagnostics 执行能力完成。
- R3 Level B 执行、`.codex` 读写或 backlog 解冻。

## 11. 停止线

任务包写成后，主管线必须停在实现前，等待用户确认或明确“开始做 H3-5”。实现完成后必须交给独立复核线复核；主管线不得自审替代复核线结论。

H3-5 完成并 checkpoint 后，下一步才允许进入 ProjectsView 拆分序列；按用户已确认的顺序是 H3-1、H3-2、H3-3。

## 12. 实现记录

实现日期：2026-06-13。

实现结果：

- `AgentView.tsx` 从 1974 行降到 285 行，低于本包目标 1000 行。
- 新增 `src/views/agent/AgentDeveloperPanels.tsx`，承接开发者 details 总容器和面板顺序编排。
- 新增 `src/views/agent/AgentExecutionPanels.tsx`，承接 `CodexControlEntryPanel`、`UnifiedExecutionStatusPanel` 和真实执行产品命令相关展示。
- 新增 `src/views/agent/AgentAdapterBoundaryPanels.tsx`，承接 adapter / provider / session operation 只读边界面板。
- 新增 `src/views/agent/AgentContinuationBoundaryPanels.tsx`，承接 continuation / H2 readiness / runtime attention / I5 diagnostics 面板。
- 新增 `src/views/agent/agentLabels.ts`，承接开发者面板使用的 label / tone / grouping helper。
- `AgentView.tsx` 保留顶层数据派生、会话选择状态、`AgentSessionCenter` 装配、兼容导出和 `developerDetails` 插槽输入。
- `workbench-shape-gate.js` 的 `AgentView.tsx` waterline 更新为 285。

验证已通过：

- `npm run typecheck`
- `npm run test:offline-interaction`，14 passed
- `npm run build`，通过，仅既有 Vite chunk size warning
- `node scripts/harness/workbench-shape-gate.js --mode check`，pass，0 errors，0 warnings
- `git diff --check`

边界确认：

- 未修改 `styles.css`。
- 未修改 `ProjectsView.tsx`。
- 未修改 Rust / Tauri / DB / sidecar schema / workflow state schema。
- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 未进入 R3 Level B，未解冻 backlog。

复核结论：

- 独立复核线 Singer 已回交 `STATUS: CLEAR`。
- P0 / P1 / P2：无。
- Review：`evidence/2026-06-13-root-treatment-r4-h3-5-agent-developer-boundary-panel-split-v1-review-singer-v1.md`
- Evidence：`evidence/2026-06-13-root-treatment-r4-h3-5-agent-developer-boundary-panel-split-v1.md`
- Handoff：`handoffs/2026-06-13-root-treatment-r4-h3-5-agent-developer-boundary-panel-split-v1-result.md`
