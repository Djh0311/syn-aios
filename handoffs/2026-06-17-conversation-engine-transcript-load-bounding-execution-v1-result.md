# 会话引擎补刀：Transcript Load Bounding 交接 v1

日期：2026-06-17  
任务包：`tasks/2026-06-17-conversation-engine-transcript-load-bounding-execution-v1.md`  
执行线状态：实现 + 自动验证完成，独立复核 `STATUS: CLEAR`，等待咨询线红队；未提交。

## 交接摘要

本轮补齐上一轮会话引擎重做漏掉的“加载界定”：点开 Agent 会话不再先拉整条 transcript，而是优先通过新 Tauri command 取最近 80 条；如果还有更早内容，UI 顶部显示“加载更早对话”，上滚到顶或手点后按 `older_before_line` 取 older page 并前插。

后端 page 路径仍顺序读取 JSONL 行文本以定位窗口，但只解析 selected window，不再全量解析 / 构建 / IPC 返回。若真实超大 rollout 仍卡在读盘，下一步要做文件尾反向 reader。

独立复核线 Noether（agent id `019ed568-0c43-7a22-8767-5709435ee752`，系统昵称 Beauvoir）最终 `STATUS: CLEAR`。初审 P3“空态缺 older 入口”已修复并经 follow-up 复核确认，无新 P0 / P1 / P2。

## 主要落点

- `prototypes/productized-desktop-shell/src-tauri/src/codex_transcript.rs`：新增 page request / page reader / pagination；旧 full reader 保留。
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`、`command_registry.rs`：新增并注册 `load_codex_session_transcript_page`。
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`、`src/lib/workbenchCoreTypes.ts`、`src/lib/types.ts`、`src/lib/tauri.ts`：补齐跨端 page 类型与 invoke。
- `prototypes/productized-desktop-shell/src/App.tsx`、`src/views/AgentView.tsx`、`src/views/agent/useAgentTranscriptLoader.ts`：Agent view 优先 page loader；Projects view 仍旧 full loader。
- `prototypes/productized-desktop-shell/src/lib/conversationEngine.ts`：新增 `mergeOlderTranscriptPage()`。
- `prototypes/productized-desktop-shell/src/views/agent/TranscriptViews.tsx`：bounded/full 标记、older 按钮、顶端自动 older、前插滚动补偿；空态也保留 older 入口。
- `prototypes/productized-desktop-shell/src-tauri/src/lib_transcript_readback_tests.rs`、`tests/helpers/offlineConversationEngineScenario.tsx`：新增 tail / older / bounded UI 断言；覆盖 internal-only tail 空态仍能加载更早。

## 验证

- `cargo test --lib codex_transcript -- --nocapture`：8 passed。
- `cargo test --lib transcript_catalog -- --nocapture`：8 passed。
- `npm run typecheck`：pass。
- `npm run test:offline-interaction`：offline interaction tests passed: 15。
- `npm run build`：pass，Vite large chunk warning only。
- `cargo test --lib`：528 passed / 22 ignored。
- `cargo fmt -- --check`：exit 0。
- `node scripts/harness/workbench-shape-gate.js --mode check`：Status pass，Errors 0，Warnings 1。
- `git diff --check`：exit 0。
- 安全扫描：新增 diff 未命中真执行、`.codex`、记忆/编排扩面或禁用文案。

## 已知限制

- In-app Browser / Claude Preview 打开 `127.0.0.1:5173` 只能看到 BootError：普通浏览器缺 Tauri runtime，`getCurrentWindow().metadata` undefined。已内联截图，未形成仓库图片文件。
- 执行线不能声明“真机大对话点开不卡”已通过；咨询线需要在真实 Tauri 窗口里红队验证。
- shape warning 是命令数 ratchet：Tauri commands 100 vs baseline 97；本包新增 1 个窄命令，且 0 个命令在 `lib.rs`。

## 不接受为

不接受为真实 Codex 发送/执行已接入，不接受为 K3-B1/K3-B2/real-resume 已解锁，不接受为 `.codex` 访问面扩大，不接受为记忆/编排/画布/整体布局改造，不接受为视觉已定稿，不接受为真机流畅度已用户验收。

## 咨询线红队建议

1. 查 `codex_transcript.rs`：page reader 是否只解析 selected lines，旧 full reader 是否未变。
2. 查 `commands.rs`：SQLite / index fallback 是否同源调用 page reader，路径 guard 是否沿用既有 rollout allowed dirs。
3. 查前端：Agent view 是否走 page loader，Projects view 是否没被扩大。
4. 真机 Tauri：选一条几百+消息 / 数 MB rollout，点开测首屏时间，上滚测 older page 和滚动补偿。
5. 若点开仍卡：优先做后端从文件尾反向读最后 N 行，避免顺序读完整 JSONL。
