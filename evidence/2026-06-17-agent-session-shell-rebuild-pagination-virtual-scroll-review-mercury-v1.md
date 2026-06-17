# Review: Agent Session Shell Rebuild Pagination Virtual Scroll · Mercury v1

日期：2026-06-17

复核线：Mercury（同源只读复核）

Agent id：Codex main session, local review pass

STATUS: CLEAR_WITH_NOTE

## 复核范围

- 任务包：`tasks/2026-06-17-agent-session-shell-rebuild-pagination-virtual-scroll-v1.md`
- Kickoff：`handoffs/2026-06-17-agent-session-shell-rebuild-claude-to-codex-kickoff-v1.md`
- 代码 diff：当前未提交工作区全部会话外壳相关改动。
- 验证输出：typecheck / offline interaction / build / cargo tests / fmt / shape gate / diff check / browser fallback。

## 结论

无 P0 / P1 / P2。

Note：普通浏览器验证被现有 Tauri runtime 依赖阻断，未完成真实 Tauri / Claude Preview / 用户 Mac 真实数据量不卡验收。本包不能据此宣称“真机不卡已验”；只能宣称分页/窗口化/归档隔离代码路径与 fixture/offline 验证通过。

## 逐项核验

- 后端分页：`read_threads_page()` 使用 `LIMIT page_size + 1 OFFSET`，默认 `archived = 0`；`archived_only` 使用 `archived = 1`。fixture 测试覆盖默认过滤、显式 include、archived-only、has_more。
- 旧 helper 兼容：`read_threads()` 循环调用分页直到 `has_more=false`，测试 `read_threads_legacy_helper_still_reads_all_pages` 覆盖超过 250 行场景，避免旧 transcript lookup 被截断。
- Tauri 命令：新增 `load_codex_session_page`，返回 `CodexSessionPage`，SQLite 失败回落 index 分页并带 warning；命令注册在 `command_registry.rs`，未塞入 `lib.rs`。
- 前端分页：`useAgentSessionPage()` 仅在 Agent 页接分页命令；非 Tauri/offline 路径回落 `sessions` fixture/snapshot。
- 归档隔离：`sessionMatchesReadFilter()` 中 `readable` / `all` / `missing` 均排除 archived，只有 `archived` 显式视图显示归档；离线测试已钉住。
- 虚拟/窗口化：`AgentSessionList` 默认渲染 40 条可见会话，显示“已渲染 X / Y”和“显示更多会话”；离线场景确认第 41 条和远端第 90 条在加载更多前不进入 DOM。
- 聊天工程基线：既有 transcript 较早消息收纳保留，本包新增“回到最新消息”按钮；未做大规模聊天 UI 改造。
- Subagent folding：只新增 fixture schema 探测 `parent_session_id`，无折叠实现、无真实 `.codex` schema 读取。
- Shape gate：`AgentView.tsx` 降到 281/285，offline test 3404/3404，新增逻辑拆到新文件；0 errors。唯一 warning 是 Tauri command 总数增加，符合新增窄命令事实。
- 安全边界：扫描未发现本包新增 `codex exec` / `codex exec resume` / `Command::new("codex")` / `.codex` 读写。命中为既有 `pbcopy/open`、既有 K2 prompt 字段、既有 deny 文案和历史测试。

## 证据检查

- `cargo test --lib codex_db -- --nocapture`：7 passed。
- `cargo test --lib`：526 passed / 22 ignored。
- `npm run typecheck`：exit 0。
- `npm run test:offline-interaction`：15 passed + R4 page tests passed。
- `npm run build`：built；Vite chunk warning 为既有打包体积提醒。
- `cargo fmt -- --check`：exit 0。
- `node scripts/harness/workbench-shape-gate.js --mode check`：Status pass，Errors 0，Warnings 1。
- `git diff --check`：exit 0。

## 残余风险

- 真实 `.codex` 数据量下的 Tauri 会话列表不卡尚未由用户在场验证。
- 普通 browser fallback 不能替代 Tauri 验证，因为现有 app 在非 Tauri 环境会因 `getCurrentWindow()` 崩溃。
- Snapshot 兼容路径仍存在全量 `read_threads()`，本包的“不卡”主要落实在 Agent 会话中心分页命令路径；项目摘要/旧 snapshot 全局重构不在本包内。

## 不可声称

- 不可声称 Claude Preview / Tauri 真机视觉验收完成。
- 不可声称真实 `.codex` 数据已由本执行线读取验证。
- 不可声称 subagent 折叠完成。
- 不可声称产品执行语义、R3 切换、K3-B1/K3-B2 有任何解锁。
