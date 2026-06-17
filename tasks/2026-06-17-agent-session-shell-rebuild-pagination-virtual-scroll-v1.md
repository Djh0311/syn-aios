# 任务包：会话外壳重做（会话列表卡顿补缺：分页 + 虚拟滚动 + 归档隔离）v1

日期：2026-06-17

阶段：UI track · 会话外壳重做（落地 backlog「智能体会话读模型重做」；用户 2026-06-17 定为「卡真用的点、先做」）

执行：建议 **Codex 执行线**（在用户 Mac 上真测 Tauri + 真 `.codex` 只读数据量）→ 独立复核 → 咨询线审实物。

## 拍板摘要

- **要做的事**：治「智能体会话列表根本用不了」的卡顿——后端会话列表**全量加载（SELECT ALL）**改成**分页 + `WHERE archived=0`**；前端会话中心上**虚拟滚动 + 加载更多 + 归档隔离**。这是用户点名「卡真用」的点。
- **代价**：一次前端 + 后端的「补缺」改动（backlog 已确认无架构陷阱）；需在 Mac 上真测。
- **不做的后果**：一进会话页就卡，真用（含 Step 3 GUI 真用）走不动。

## 一句话判据

判某改动在不在本包内——问：**「是不是为了让会话列表『不卡、能用』（分页 / 虚拟滚动 / 归档隔离 / 聊天工程基线），且只读 `.codex` 会话元数据、不碰正文 / auth、不改记忆与执行语义？」** 是 → 做；否 → 停、另议。

## 背景：根因已在 backlog 定位

见 `backlog.md`「智能体会话读模型重做」：

- 卡顿根因：`index_host_app_entrypoints.rs:load_sessions` → `codex_db.rs:read_threads()` 对 `~/.codex/state_*.sqlite` 做 `SELECT ALL`，WHERE 仅 `has_user_event=1`、**未过滤 archived**；归档 / 侧栏 / subagent 全在同表一次捞出。前端 `AgentView.tsx` 只内存过滤（`filterAgentSessions`）不减加载量。
- 单条会话**消息**已按需加载（`loadTranscript`）——卡的是**列表全量**，不是消息。

## 范围（写）

- 后端：`codex_db.rs:read_threads()` / `index_host_app_entrypoints.rs:load_sessions` 加分页参数 + `WHERE archived=0`。
- 前端：会话中心（`AgentSessionCenter` / `AgentView.tsx`）虚拟滚动（react-window 类）+ 加载更多 + 归档隔离（SQL 条件 + 前端过滤配合）。
- 工程基线（backlog「实现规范」，一并）：稳定虚拟列表、降频滚动防抖、滚离底部显「回到底部」不强拉、空状态带快捷提示。
- **可复用、别重造**：`pageSelectors.ts`、`deriveAgentAdapterDescriptors`、`conversationTurns`、`softwareGroupsForSessions`、`SessionRecord` 类型、`AgentView` / `AgentSessionCenter` 容器结构。

## 不做 / 边界

- subagent 折叠：**本包只做发现**——查 Codex sqlite 有无 `parent_session_id` 字段；有 → 留下一包做，无 → 记下。不强做。
- `.codex`：仅**只读会话元数据**（已拍板 `decisions/2026-06-17-codex-state-readonly-session-metadata-access-for-real-use-v1.md`）；**不读 transcript 正文 / auth / token / secret**，不写 `.codex`。
- **dev agent 实现期用 fixtures 测**（仿 `codex_db.rs` 既有 `create_threads_db` 测试），开发中不读真 `.codex`。
- 不改记忆层 / 执行语义 / R3 切换 / 真 Codex 执行；**不动视觉风格（水墨）**——本包是「会话外壳能用」，不是布局重做（那是另一对话的 UI track）。

## 验收

- 后端：分页 + archived 过滤有**单测（fixtures）**；大列表只加载一页、不再全量。
- 前端：用 **`Claude Preview` MCP**（已定口径，不自建工具）截图 / 快照证明列表虚拟滚动、不卡、归档不混入。
- 真机：用户 Mac 上 `tauri dev` 真开，会话页**不再卡、能用**（真实数据量只在你机器上，必须真测）。
- shape gate 绿、`cargo test` / `npm test` 绿、`cargo fmt` / `git diff --check`。

## 验证（TDD 优先）

- 后端分页 / 过滤：test-first（fixtures）。
- 前端虚拟滚动 / 交互：Claude Preview 视觉 + offline fixture 渲染。
- 真机卡顿消除：用户 Mac 实测（收口前）。

## 回交

- 实现 + 证据（单测、Claude Preview 截图、真机卡顿前后对比）→ 独立复核 → 咨询线审实物。

## 不接受为

- 不接受为布局重做完成（那是另一对话）/ 视觉风格变更。
- 不接受为读了 transcript 正文 / auth / secret / 写了 `.codex` / 真跑 Codex。
- 不接受为 subagent 折叠已做（本包只发现）。
