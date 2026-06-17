# 交接：会话外壳重做 · 咨询线 → Codex 执行 · kickoff v1

日期：2026-06-17

出自：咨询线（Claude）。性质：把「会话外壳重做」交给执行线 Codex 执行的 kickoff。**任务正本 = `tasks/2026-06-17-agent-session-shell-rebuild-pagination-virtual-scroll-v1.md`**（范围 / 验收 / 边界以它为准）；本文只加执行框架 + 安全口径。

## 0. 接手须知

- 你是**执行线**。流水线：**你执行 → 独立复核线只读复核（STATUS）→ 咨询线（Claude）审实物 → 用户拍板**。
- 先读任务正本 + `CURRENT.md` 首条 + `AGENTS.md`。**全程中文、术语标中文注释**。
- **子线不 `git add` / `git commit`**；提交只由用户在咨询线询问后做。

## 1. 一句话目标

治「会话列表根本用不了」的卡顿：**后端分页 + `WHERE archived=0`；前端虚拟滚动 + 加载更多 + 归档隔离 + 聊天工程基线**。根因与范围详见任务正本。

## 2. 硬封印

- 只读 `.codex` 会话元数据（已拍板 `decisions/2026-06-17-codex-state-readonly-session-metadata-access-for-real-use-v1.md`）；**不读 transcript 正文 / auth / token / secret**，不写 `.codex`。
- **开发期用 fixtures 测**（仿 `codex_db.rs` 既有 `create_threads_db`），开发中别读真 `.codex`；真机卡顿验证由用户在场时做。
- **不动视觉风格（水墨）**——本包是「会话外壳能用」，不是布局重做（那是另一对话的 UI track）。
- 不改记忆层 / 执行语义 / R3 切换；不真跑 Codex（K3-B1 / B2 维持封死）。

## 3. UI 看图口径（已定）

- 前端验证用 **`Claude Preview` MCP** 当 agent 的「眼睛」（截图 / 页面结构快照），**不自建 MCP 工具**（见 `backlog.md`「UI 视觉反馈 MCP 工具」口径）。

## 4. 验收 / 回交（详见任务正本）

- 后端分页 / 过滤**单测（fixtures）**绿；前端 Claude Preview 证明虚拟滚动、不卡、归档不混入；真机 `tauri dev` 用户实测「不卡了、能用」；shape gate / `cargo test` / `npm test` / `cargo fmt` / `git diff --check` 绿。
- **subagent 折叠本包只做发现**（查 sqlite 有无 `parent_session_id`），不强做。
- 回交：实现 + 证据（单测 / Claude Preview 截图 / 真机前后对比）→ 独立复核 → 咨询线审实物。

## 5. 不接受为

- 布局重做完成（另一对话）/ 视觉风格变更 / 读了 transcript 正文或 auth / 写了 `.codex` / 真跑 Codex / subagent 折叠已做。

---

*本文是执行 kickoff，不替代任务正本。需改任务范围先回咨询线，不擅自扩。*
