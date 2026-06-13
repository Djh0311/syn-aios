# Review: Root Treatment / R4-H3-4 Agent Conversation Shell Component Split v1

日期：2026-06-13

复核线：Faraday

状态：`STATUS: CLEAR`

## 1. 结论

H3-4 可放行。主管线可以进入 commit / checkpoint。

P0：无。

P1：无。

P2：无。

## 2. 关键证据

- `AgentView.tsx` 当前 1974 行，低于 2300 目标。
- 新增文件均低于 2000 行：
  - `AgentConversationShell.tsx`：925 行。
  - `AgentSessionList.tsx`：253 行。
  - `AgentChatComposer.tsx`：187 行。
- `AgentView.tsx` 兼容导出仍保留，`ProjectsView.tsx` 仍从旧入口消费 `AgentSessionCenter`。
- 开发者详情面板函数仍留在 `AgentView.tsx`，未提前做 H3-5。
- `AgentSessionList` 明确把数组作为当前适配层输入，并给分页 / 虚拟滚动 / 直读数据库留口。
- `styles.css`、`ProjectsView.tsx`、tests 无 diff。
- shape gate waterline 已更新为 `AgentView.tsx: 1974`。

## 3. 实际检查

复核线读取：

- `docs/plans/2026-06-13-root-treatment-r4-h3-project-agent-view-layout-block-split-design-v1.md`
- `tasks/2026-06-13-root-treatment-r4-h3-4-agent-conversation-shell-component-split-v1.md`
- `evidence/2026-06-13-root-treatment-r4-h3-4-agent-conversation-shell-component-split-v1.md`
- `handoffs/2026-06-13-root-treatment-r4-h3-4-agent-conversation-shell-component-split-v1-result.md`
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/views/agent/AgentConversationShell.tsx`
- `prototypes/productized-desktop-shell/src/views/agent/AgentSessionList.tsx`
- `prototypes/productized-desktop-shell/src/views/agent/AgentChatComposer.tsx`
- `scripts/harness/workbench-shape-gate.js`

复核线运行并通过：

- `git diff --check`
- `npm run typecheck`
- `npm run test:offline-interaction`
- `node scripts/harness/workbench-shape-gate.js --mode check`

## 4. 越界检查

未发现 H3-4 越界。

复核线未启动：

- Tauri
- Browser
- Chrome
- Vite dev
- screenshot

复核线未执行：

- 真实 `codex exec`
- 真实 `codex exec resume`

复核线未读写：

- `/Users/yoyi/.codex`

## 5. 主管线提交提醒

提交时只应纳入 H3-4 文件，排除当前外部脏文件：

- `AGENTS.md`
- `backlog.md`
- `docs/own-agent-and-company-vision-v1.md`
- `handoffs/2026-06-13-r4-hard-targets-execution-plan-supervisor-v1.md`
