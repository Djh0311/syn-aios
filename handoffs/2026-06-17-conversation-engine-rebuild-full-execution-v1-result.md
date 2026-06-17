# 会话引擎重做 · 原生聊天范式 · 执行线回交 v1

日期：2026-06-17

执行线：Codex

状态：实现与自动化验证已完成；独立复核 `STATUS: CLEAR_WITH_NOTE`；等待咨询线审实物。未提交。

## 结论

本包坐在当前未提交的会话外壳分页地基上，完成 M1 → M2 → M3：

- M1：会话正文区改为虚拟消息窗口；选中会话刷新时保留已读历史，不再先空窗。
- M2：近底黏底、流式尾条从稳定虚拟窗口分离、提供“回到底部”入口。
- M3：普通撰写区收为单道“发送”，Enter 发送 / Shift+Enter 换行；发送只追加 pending 用户气泡并记录 decision-only metadata，不调用真实执行链。

## 改动范围

会话引擎新增 / 修改：
- `prototypes/productized-desktop-shell/src/lib/conversationEngine.ts`
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/views/agent/AgentChatComposer.tsx`
- `prototypes/productized-desktop-shell/src/views/agent/AgentConversationShell.tsx`
- `prototypes/productized-desktop-shell/src/views/agent/TranscriptViews.tsx`
- `prototypes/productized-desktop-shell/tests/helpers/offlineConversationEngineScenario.tsx`
- `prototypes/productized-desktop-shell/tests/helpers/offlineAgentSessionShellPaginationScenario.tsx`
- 相关 offline fixture 文案调整。

当前工作树仍包含上一包分页 / 归档隔离地基：
- `codex_db.rs` / `commands.rs` / `command_registry.rs` / `index_host_app_entrypoints.rs` / `types.rs`
- `useAgentSessionPage.ts` / `AgentSoftwareFilterBar.tsx` / `AgentSessionList.tsx`
- pagination evidence / handoff 文件。

## 边界

未做：
- 未 `git add` / `git commit`。
- 未更新 `CURRENT.md`。
- 未接真实 Codex 执行；普通撰写区已移除旧 K2 preview / prepare / confirm / phase-a / phase-b 调用链。
- 未改记忆 store / FormalMemory / memory schema。
- 未改编排 / 画布 / 整体工作台布局。
- 未读 `.codex` 正文 / auth / token / secret / prompt body；未写 `.codex`。
- 未硬定气泡视觉、水墨配色或整体 UI 风格。

## 验证

已过：
- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `cargo fmt -- --check`
- `cargo test --lib codex_db -- --nocapture`
- `cargo test --lib`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`
- `node scripts/harness/checkpoint-audit.js --commit HEAD --allow-dirty --skip-gates --review evidence/2026-06-17-conversation-engine-rebuild-full-execution-review-kepler-v1.md`

关键结果：
- offline interaction: `15` passed，R4 selector/runtime checks passed。
- `cargo test --lib`: `526 passed / 22 ignored`。
- shape gate: `Status: pass`，`Errors: 0`，`Warnings: 1`。
- shape warning：`tauri_command_total_increased 99 vs 97`，归因于当前工作树上一包分页地基；`lib.rs` 中 Tauri commands 为 `0`。
- checkpoint audit：`VERDICT: PASS`；工作树 dirty 为预期，因为子线不提交且当前坐在未提交分页地基上。

UI / 真机证据：
- in-app Browser 打开 `http://127.0.0.1:5173/`，被非 Tauri runtime 阻断，错误为 `getCurrentWindow().metadata` undefined，不能作为通过截图。
- 直接 `cargo run --no-default-features --color always --` 启动 Tauri desktop shell 成功进入 `target/debug/codex-governance-workbench`，短暂观察无新增终端 runtime error，随后停止临时进程。
- Claude Preview MCP 未暴露；未捕获 Claude Preview 截图，未声明真机流畅度完成。
- 当前 `127.0.0.1:5173` 被既存 `node` PID `51147` 占用；执行线未擅自 kill。

## 已修问题

收口中 shape gate 先报 `styles.css` ratchet 增长 15 行。已撤回 CSS 文件新增，把虚拟层结构样式收回 `TranscriptViews.tsx` 内联，最终 shape gate 通过且 `styles.css` 回到 `8464/8464 same`。

## 证据

主证据：`evidence/2026-06-17-conversation-engine-rebuild-full-execution-v1.md`

独立复核：`evidence/2026-06-17-conversation-engine-rebuild-full-execution-review-kepler-v1.md`

复核结论：Kepler（agent `019ed4fa-17a8-7101-8d17-23f126414b7c`）修后复核 `STATUS: CLEAR_WITH_NOTE`，无 P0/P1/P2/P3；原 P2/P3 均已关闭。残余为未捕获 Claude Preview / 真机手感证据，以及同毫秒同 prompt 的 pending id 极端碰撞理论风险。

## 建议咨询线重点核

- 普通撰写区是否确实只剩 decision-only “发送”，且没有残留可触发真实 Codex 的按钮或调用链。
- `TranscriptViews.tsx` 的虚拟窗口是否满足初始最新、可滚早期、近底黏底、流式尾条分离。
- 当前 diff 是否只在会话引擎 / 分页地基 / 测试证据范围内，没有记忆 / 编排 / 画布漂移。
- UI 真机手感需用户或 Claude Preview 补验；本回交不得按“真机流畅度已完成”验收。
