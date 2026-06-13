# Root Treatment / R4-H3-4 Agent Conversation Shell Component Split v1

日期：2026-06-13

状态：已完成。

性质：R4 硬目标 / 批二 `AgentView` 优先拆分的第 1 包。本包只拆智能体普通对话区，目标是降低 `AgentView.tsx` 棘轮水位，并为后续会话外壳重做预留接口形状；不做 UI 重做，不改真实执行逻辑。

Planning baseline：`d6694ec9c9f799947e13c74a996249da8926af52`。

## 0. 全局主管理解

用户已确认 H3 设计稿，并调整实现顺序：

- 先 AgentView：H3-4、H3-5。
- 后 ProjectsView：H3-1、H3-2、H3-3。

本包是 H3-4，目标是把智能体普通对话区从 `AgentView.tsx` 抽到 `src/views/agent/` 下。普通对话区包括项目 / 对话选择、会话搜索和过滤、会话分组列表、正文容器、任务输入与发送前确认材料。

本包不处理开发者详情面板；adapter / provider / continuation / runtime / diagnostics 留给 H3-5。

## 1. 目标

完成后：

- `AgentView.tsx` 从 3118 行下降到 2300 行以下，至少下降 700 行。
- 新增 `src/views/agent/AgentConversationShell.tsx`，承接 `AgentSessionCenter` 的普通会话布局。
- 新增 `src/views/agent/AgentSessionList.tsx`，承接搜索、读取状态过滤、分组和会话卡片。
- 新增 `src/views/agent/AgentChatComposer.tsx`，承接任务输入、K2 发送预览和确认状态展示。
- `AgentView.tsx` 保留兼容导出：`AgentView`、`AgentSessionCenter`、`filterAgentSessions`、`softwareGroupsForSessions`、`ChatTranscript`、`TranscriptTimeline`。
- `ProjectsView.tsx` 和离线测试仍可从 `./AgentView` / `../src/views/AgentView` 引入 `AgentSessionCenter`。
- 视觉、DOM class、按钮文案、交互顺序和当前测试断言保持不变。

## 2. 形状影响

预期：

- `AgentView.tsx`：3118 -> 2300 以下。
- 新增 `.tsx` 文件均低于 2000 行。
- `styles.css` 不变。
- Rust / Tauri / DB / sidecar / workflow state schema 不变。
- shape gate 的 `AgentView.tsx` waterline 随本包下降。

若 `AgentView.tsx` 下降低于 700 行，本包不得收口为完成，除非全局主管在 evidence 中说明实际边界和后续包承接原因，并重新取得复核线认可。

## 3. 接口预留要求

H3-4 抽 `AgentSessionList` 与对话正文时，组件接口必须给后续会话外壳重做预留空间：

- 分页。
- 分组。
- 归档隔离。
- subagent 折叠。
- 虚拟滚动。
- 直读数据库常驻。

本包不实现这些能力，但不得把“全量数组输入 / 全量 DOM 渲染”焊死成长期接口。若当前仍传 `sessions` 数组，必须把它视为当前适配层输入，并在组件 props 命名和注释中保留未来替换为 page / datasource / adapter 的空间。

## 4. 允许范围

允许修改：

- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/views/agent/AgentConversationShell.tsx`
- `prototypes/productized-desktop-shell/src/views/agent/AgentSessionList.tsx`
- `prototypes/productized-desktop-shell/src/views/agent/AgentChatComposer.tsx`
- `scripts/harness/workbench-shape-gate.js`
- 必要的前端离线测试 import / 断言兼容修正。
- 当前任务包、evidence、handoff。

允许新增：

- `src/views/agent/AgentConversationShell.tsx`
- `src/views/agent/AgentSessionList.tsx`
- `src/views/agent/AgentChatComposer.tsx`

## 5. 禁止范围

禁止：

- 修改 UI / CSS / 水墨风格 / 布局 / 文案 / 交互。
- 改变智能体页滚动策略。
- 改 K2 / J1 真实执行权限逻辑。
- 新增裸控制台或绕过 Product Command / 权限 / 审计。
- 修改 adapter / provider / credential / model 验证边界。
- 拆 H3-5 的开发者详情面板。
- 修改 ProjectsView 结构。
- 修改 Rust / Tauri / DB / sidecar schema / workflow state schema。
- 执行真实 `codex exec` / `codex exec resume`，发送 prompt，读写 `/Users/yoyi/.codex`。
- 启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 进入 R3 Level B 或解冻 backlog。

## 6. 实现步骤

1. 把 `AgentSessionCenterProps` 和普通会话布局迁到 `AgentConversationShell.tsx`，保留 `AgentView.tsx` re-export。
2. 把会话列表渲染迁到 `AgentSessionList.tsx`，保留搜索、读取状态过滤、分组折叠和会话卡片 DOM class 不变。
3. 把 `AgentChatComposer` 迁到 `AgentChatComposer.tsx`，保持 K2 preview / prepare / confirm / Phase A / Phase B 按钮和状态展示不变。
4. 将普通区需要的纯展示 helper 迁到对应文件；H3-5 需要的开发者 helper 暂留 `AgentView.tsx`。
5. 在 `AgentView.tsx` 中保留顶层数据派生、K2 action handler、开发者详情面板和兼容导出。
6. 更新 shape gate `AgentView.tsx` waterline 到本包完成后的新低水位。

## 7. 验证

必须通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

建议扫描：

- `wc -l prototypes/productized-desktop-shell/src/views/AgentView.tsx prototypes/productized-desktop-shell/src/views/agent/*.tsx`
- `rg -n "AgentSessionCenter|AgentChatComposer|filterAgentSessions|agent-session-shell|agent-chat-composer" prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/tests`
- `git diff -- prototypes/productized-desktop-shell/src/styles.css`

## 8. 复核要求

复核线重点检查：

- `AgentView.tsx` 是否真实下降到 2300 行以下。
- 新增组件是否只是拆分普通对话区，不改 UI / CSS / 文案 / 交互。
- `AgentSessionCenter` 兼容导出是否保持。
- `ProjectsView.tsx` 和离线测试是否仍能正常引用。
- 是否把全量数组 / 全量 DOM 渲染写成长期不可替换契约。
- 是否误改 K2 / J1 真实执行 guard、provider / credential / model 边界或开发者面板。
- 是否接触 `.codex`、执行真实 Codex、启动 Tauri / Browser / Chrome / Vite dev。

## 9. 不接受为

本包不接受为：

- H3 全部完成。
- H3-5 开发者边界面板拆分完成。
- AgentView UI 重做完成。
- 会话分页、虚拟滚动、归档隔离、subagent 折叠或直读数据库常驻完成。
- 真实 Codex 执行产品化完成。
- R3 Level B 执行、`.codex` 读写或 backlog 解冻。

## 10. 实现记录

实现日期：2026-06-13。

实现结果：

- `AgentView.tsx` 从 3118 行降到 1974 行，低于本包目标 2300 行。
- 新增 `src/views/agent/AgentConversationShell.tsx`，承接 `AgentSessionCenter` 普通会话布局、K2 对话输入状态、`SessionReader` 和开发者详情插槽。
- 新增 `src/views/agent/AgentSessionList.tsx`，承接搜索、读取状态过滤、分组折叠、会话卡片和 `filterAgentSessions` / `softwareGroupsForSessions` 兼容导出。
- 新增 `src/views/agent/AgentChatComposer.tsx`，承接任务输入、发送预览、准备 / 确认 / 预检 / 执行按钮和读回结果展示。
- `AgentView.tsx` 继续 re-export `AgentSessionCenter`、`filterAgentSessions`、`softwareGroupsForSessions`、`ChatTranscript`、`TranscriptTimeline`，保持 `ProjectsView.tsx` 和离线测试兼容。
- 开发者详情面板定义仍留在 `AgentView.tsx`，通过 `developerDetails` 插槽传入 `AgentConversationShell`；未提前做 H3-5。
- `AgentSessionList` props 明确当前数组是适配层输入，后续可替换成分页、虚拟滚动或直读数据库数据源；本包未实现这些后续能力。
- `workbench-shape-gate.js` 的 `AgentView.tsx` waterline 更新为 1974。

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

- 独立复核线 Faraday 已回交 `STATUS: CLEAR`。
- P0 / P1 / P2：无。
- Review：`evidence/2026-06-13-root-treatment-r4-h3-4-agent-conversation-shell-component-split-v1-review-faraday-v1.md`
- Evidence：`evidence/2026-06-13-root-treatment-r4-h3-4-agent-conversation-shell-component-split-v1.md`
- Handoff：`handoffs/2026-06-13-root-treatment-r4-h3-4-agent-conversation-shell-component-split-v1-result.md`
