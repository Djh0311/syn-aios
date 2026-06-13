# Evidence: Root Treatment / R4-H3-4 Agent Conversation Shell Component Split v1

日期：2026-06-13

状态：`completed_with_independent_review_clear`

任务包：`tasks/2026-06-13-root-treatment-r4-h3-4-agent-conversation-shell-component-split-v1.md`

## 1. 目标

本包执行 R4-H3 批二的 AgentView 优先拆分第一包：

- 先拆智能体普通对话区。
- 保持 UI / CSS / 文案 / 交互零变更。
- 不拆开发者详情面板。
- 不改真实 Codex 执行、权限、provider、credential、model 边界。
- 将 `AgentView.tsx` 降到 2300 行以下。

## 2. 设计稿补充

已补充：

- `docs/plans/2026-06-13-root-treatment-r4-h3-project-agent-view-layout-block-split-design-v1.md`

补充内容：

- H3-4 / H3-5 拆 `AgentSessionList` 与对话正文时，接口必须给后续分页、分组、归档隔离、subagent 折叠、虚拟滚动、直读数据库常驻预留空间。
- H3 本批不实现这些功能。
- 不得把“全量数组输入 / 全量 DOM 渲染”焊死成长期接口。

## 3. 实现改动

改动文件：

- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/views/agent/AgentConversationShell.tsx`
- `prototypes/productized-desktop-shell/src/views/agent/AgentSessionList.tsx`
- `prototypes/productized-desktop-shell/src/views/agent/AgentChatComposer.tsx`
- `scripts/harness/workbench-shape-gate.js`
- `docs/plans/2026-06-13-root-treatment-r4-h3-project-agent-view-layout-block-split-design-v1.md`
- `tasks/2026-06-13-root-treatment-r4-h3-4-agent-conversation-shell-component-split-v1.md`

未改：

- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- Rust / Tauri / DB / sidecar schema / workflow state schema

## 4. 形状结果

最终行数：

- `AgentView.tsx`：1974 行。
- `AgentConversationShell.tsx`：925 行。
- `AgentSessionList.tsx`：253 行。
- `AgentChatComposer.tsx`：187 行。
- `TranscriptViews.tsx`：246 行，未修改。

水位：

- `scripts/harness/workbench-shape-gate.js` 中 `AgentView.tsx` waterline 更新为 1974。

结论：

- `AgentView.tsx` 从 3118 降到 1974，下降 1144 行。
- 新增文件均低于 2000 行。
- 未用一个新巨型文件替代旧巨型文件。

## 5. 行为与兼容

保持：

- `AgentView` 主路径仍显示项目 / 对话选择、会话列表、会话正文、任务输入、发送预览。
- `AgentSessionCenter` 仍从 `src/views/AgentView.tsx` re-export。
- `filterAgentSessions` / `softwareGroupsForSessions` 仍从 `src/views/AgentView.tsx` re-export。
- `ProjectsView.tsx` 当前 `import { AgentSessionCenter } from "./AgentView"` 不需要修改。
- 离线测试当前 `import { AgentSessionCenter, AgentView, ChatTranscript, filterAgentSessions } from "../src/views/AgentView"` 不需要修改。

结构变化：

- `AgentConversationShell.tsx` 承接 `AgentSessionCenter`、普通对话布局、`SessionReader`、K2 普通输入状态和开发者详情插槽。
- `AgentSessionList.tsx` 承接搜索、读取状态过滤、分组折叠、会话卡片和会话过滤 helper。
- `AgentChatComposer.tsx` 承接任务输入、发送预览、准备 / 确认 / 预检 / 执行按钮。
- `AgentView.tsx` 保留顶层数据派生、transcript load 状态、开发者详情面板定义和执行边界面板。

## 6. 验证

已通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
  - 输出：`offline interaction tests passed: 14`
  - 伴随输出：R4 page read model settings / query contract / runtime / selectors tests passed
- `npm run build`
  - 通过
  - 仅既有 Vite chunk size warning
- `node scripts/harness/workbench-shape-gate.js --mode check`
  - pass
  - 0 errors
  - 0 warnings
  - `AgentView.tsx: 1974/1974 (same)`
- `git diff --check`

辅助扫描：

- `git diff -- prototypes/productized-desktop-shell/src/styles.css` 无输出。
- `rg -n "AgentSessionCenter|AgentChatComposer|filterAgentSessions|agent-session-shell|agent-chat-composer" prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/tests` 命中兼容 re-export、新组件、现有 CSS class 和现有测试断言。

## 7. 边界确认

本包未执行：

- 真实 `codex exec` / `codex exec resume`。
- prompt 发送。
- `/Users/yoyi/.codex` 读写。
- Tauri / Browser / Chrome / Vite dev / screenshot。
- R3 Level B。
- DB / sidecar schema / workflow state schema 修改。
- CSS / 水墨风格 / UI 视觉修改。
- `ProjectsView.tsx` 拆分。
- H3-5 开发者详情拆分。
- backlog 解冻。

## 8. 复核输入

复核线重点看：

- `AgentView.tsx` 是否真实降到 2300 以下。
- `AgentSessionCenter` / `filterAgentSessions` 兼容导出是否保持。
- 是否只拆普通对话区，未改开发者面板语义。
- 是否误改 UI / CSS / 文案 / 交互。
- `AgentSessionList` 是否把当前数组输入限定为适配层细节，没有把全量 DOM 渲染写成长期契约。
- K2 / J1 / 真实执行 guard 是否未削弱。
- 是否接触 `.codex`、执行真实 Codex 或启动桌面 / 浏览器。

## 9. 复核结论

独立复核线 Faraday 已回交：

- `STATUS: CLEAR`
- P0：无
- P1：无
- P2：无

Review 文件：

- `evidence/2026-06-13-root-treatment-r4-h3-4-agent-conversation-shell-component-split-v1-review-faraday-v1.md`

当前结论：

- H3-4 可接受为 Agent 普通对话区组件拆分完成。
- H3-5 尚未开始。
