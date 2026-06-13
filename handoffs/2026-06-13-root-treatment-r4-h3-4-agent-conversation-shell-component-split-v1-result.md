# Handoff: Root Treatment / R4-H3-4 Agent Conversation Shell Component Split v1

日期：2026-06-13

状态：`completed_with_independent_review_clear`

## 1. 本轮结论

H3-4 已实现、通过主管线离线验证，并经独立复核线 Faraday 复核 `STATUS: CLEAR`。

本轮可接受为 H3-4 完成。仍未完成：

- H3-5 开发者边界面板拆分。
- H3 批二整体完成。
- ProjectsView 拆分。

## 2. 关键改动

- `AgentView.tsx` 从 3118 行降到 1974 行。
- 新增 `AgentConversationShell.tsx`，承接 `AgentSessionCenter`、普通会话布局、`SessionReader`、K2 普通输入状态和开发者详情插槽。
- 新增 `AgentSessionList.tsx`，承接搜索、读取状态过滤、分组折叠、会话卡片和会话过滤 helper。
- 新增 `AgentChatComposer.tsx`，承接任务输入、发送预览、准备 / 确认 / 预检 / 执行按钮。
- `AgentView.tsx` 继续 re-export `AgentSessionCenter`、`filterAgentSessions`、`softwareGroupsForSessions`、`ChatTranscript`、`TranscriptTimeline`。
- `AgentView.tsx` 仍保留开发者详情面板定义；H3-5 尚未开始。
- shape gate 中 `AgentView.tsx` waterline 更新为 1974。

## 3. 记录文件

- Task：`tasks/2026-06-13-root-treatment-r4-h3-4-agent-conversation-shell-component-split-v1.md`
- Evidence：`evidence/2026-06-13-root-treatment-r4-h3-4-agent-conversation-shell-component-split-v1.md`
- H3 design：`docs/plans/2026-06-13-root-treatment-r4-h3-project-agent-view-layout-block-split-design-v1.md`

## 4. 验证结果

已通过：

- `npm run typecheck`
- `npm run test:offline-interaction`，14 passed
- `npm run build`，通过，仅既有 Vite chunk size warning
- `node scripts/harness/workbench-shape-gate.js --mode check`，pass，0 errors，0 warnings
- `git diff --check`

未跑：

- `cargo test`：本包未改 Rust / Tauri。
- Browser / Tauri / screenshot：本包禁止启动。

## 5. 边界确认

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

## 6. 复核线输入

独立复核线 Faraday 已检查：

- `AgentView.tsx` 低于 2300 行。
- 新增文件均低于 2000 行。
- 兼容导出保持。
- `ProjectsView.tsx` 和现有测试仍通过 `AgentView` 入口消费 `AgentSessionCenter`。
- UI / CSS / 文案 / 交互无变更证据。
- 开发者详情面板未提前拆分或改语义。
- `AgentSessionList` 为后续分页 / 虚拟滚动 / 直读数据库留口。
- K2 / J1 真实执行 guard、provider / credential / model 边界未削弱。
- 未发现 `.codex` 接触、真实 Codex 执行或桌面 / 浏览器启动证据。

Review 文件：

- `evidence/2026-06-13-root-treatment-r4-h3-4-agent-conversation-shell-component-split-v1-review-faraday-v1.md`

## 7. 外部脏文件

以下为本轮开始前已存在的外部脏文件 / 未跟踪文件，本包未纳入：

- `AGENTS.md`
- `backlog.md`
- `docs/own-agent-and-company-vision-v1.md`
- `handoffs/2026-06-13-r4-hard-targets-execution-plan-supervisor-v1.md`

## 8. 下一步

下一步：

- 由主管线提交 H3-4 相关文件。
- 写 checkpoint。
- 然后准备 H3-5 任务包。

仍不得进入：

- ProjectsView H3-1 / H3-2 / H3-3。
- R3 Level B。
- 真实 Codex 执行。
- backlog 解冻。
