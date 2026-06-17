# 交接：会话外壳重做结果 v1

日期：2026-06-17

出自：Codex 执行线。性质：会话外壳分页 / 虚拟窗口 / 归档隔离实现回交，供独立复核线、咨询线和用户拍板使用。

## 结果摘要

- 后端：新增 `read_threads_page()` + `load_codex_session_page`，默认只读非归档会话元数据，显式归档视图走 `archived_only`。
- 前端：Agent 页接入分页 hook；会话列表默认只渲染 40 条卡片，提供“显示更多会话”；`all/missing/readable` 均不混入 archived。
- 兼容：旧 `read_threads()` 改为循环分页读完所有 rows，保留 transcript lookup / allowlist 等旧调用者语义。
- 发现：fixture schema 下可探测 `parent_session_id`，但本包不做 subagent 折叠。
- 未完成：普通浏览器验证被现有 Tauri runtime 依赖阻断；未做真实 Tauri / 真实 `.codex` 数据量不卡验收。

## 关键实物

- 证据：`evidence/2026-06-17-agent-session-shell-rebuild-pagination-virtual-scroll-v1.md`
- 复核：`evidence/2026-06-17-agent-session-shell-rebuild-pagination-virtual-scroll-review-mercury-v1.md`
- 后端入口：`prototypes/productized-desktop-shell/src-tauri/src/codex_db.rs`
- Tauri 命令：`prototypes/productized-desktop-shell/src-tauri/src/commands.rs` / `command_registry.rs`
- 前端 hook：`prototypes/productized-desktop-shell/src/views/agent/useAgentSessionPage.ts`
- 列表窗口：`prototypes/productized-desktop-shell/src/views/agent/AgentSessionList.tsx`
- 离线场景：`prototypes/productized-desktop-shell/tests/helpers/offlineAgentSessionShellPaginationScenario.tsx`

## 验证

- `cargo test --lib codex_db -- --nocapture`：7 passed。
- `cargo test --lib`：526 passed / 22 ignored。
- `npm run typecheck`：exit 0。
- `npm run test:offline-interaction`：15 passed + R4 page tests passed。
- `npm run build`：built，保留 Vite chunk size warning。
- `cargo fmt -- --check`：exit 0。
- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors / 1 warning（Tauri command total 99 vs baseline 97，本包新增窄命令）。
- `git diff --check`：exit 0。

## 边界

- 未读真实 `/Users/yoyi/.codex`，未写 `.codex`。
- 未读 transcript 正文、auth/token/secret。
- 未执行真实 Codex / K3-B1 / K3-B2。
- 未改记忆层、R3 切换或执行语义。
- 未 claim 真机不卡；需用户在场 Tauri/Claude Preview 补验。

## 给咨询线的核验建议

1. 核 `codex_db.rs`：默认分页 SQL 带 `archived = 0`，归档视图 `archived = 1`，旧 `read_threads()` 不被 250 页大小截断。
2. 核 `AgentSessionList.tsx`：`readable/all/missing` 不混 archived；只 `archived` 显示归档。
3. 核 `useAgentSessionPage.ts`：只调用 `loadCodexSessionPage`，非 Tauri/offline 回落 snapshot；无真实执行。
4. 核 shape gate warning：新增命令在 `commands.rs` / `command_registry.rs`，不是塞进 `lib.rs`。
5. 核证据未 overclaim：普通浏览器/Tauri 真机验证仍列为未完成。
