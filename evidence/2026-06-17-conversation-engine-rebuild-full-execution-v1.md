# 会话引擎重做 · 原生聊天范式 · 执行证据 v1

日期：2026-06-17 17:43 CST

任务包：`tasks/2026-06-17-conversation-engine-rebuild-full-execution-v1.md`

策略正本：`docs/plans/2026-06-17-conversation-engine-rebuild-native-chat-paradigm-plan-v1.md`

执行线：Codex

提交状态：未提交；等待咨询线审实物后由用户决定是否 `git add` / `git commit`。

## 拍板摘要

本包把会话 / 对话页从“分页地基 + 前端手写窗口 + 6 步发送面”收敛为更接近原生聊天范式的产品面：会话列表沿用当前未提交的后端分页，正文区改为虚拟消息窗口、选中会话保留已读历史、流式尾条分离、黏底 / 回到底部提示、撰写区 Enter 发送并立即冒泡 pending 用户消息。

关键澄清：这个“发送”只记录发送意图并追加本地 pending 气泡；没有调用 Codex、没有 `codex exec` / `codex exec resume`、没有放宽 K3-B1 / K3-B2 / real-resume 门。视觉只做结构性行为，不硬定气泡配色、水墨质感或最终 UI 细调。

一句话判据：若改动是为了会话流畅性（虚拟化 / 选中即显 / 黏底 / 流式尾条 / 单道发送意图）且不接真实执行、不碰记忆 / 编排 / 整体布局、不硬定视觉，则在本包内。

## 读写范围

读取：
- 任务包、策略正本、`CURRENT.md` 首条、`AGENTS.md`。
- 相关技能：`using-superpowers`、`executing-plans`、`test-driven-development`、`ui-browser-verification`、`verification-before-completion`、`evaluator-acceptance-review`、`requesting-code-review`、Browser skill。
- 当前 dirty worktree 中上一包“会话外壳分页 / 归档隔离”相关文件与证据。

写入：
- 前端会话引擎与相关测试 helper。
- 本证据文件与交接文件。

未写入：
- `CURRENT.md` 未更新。
- 未 `git add` / `git commit`。
- 未写 `/Users/yoyi/.codex`，未读取 auth/token/secret/full transcript/prompt body。

## M1 自检：数据与骨架

实现：
- `AgentView.tsx` 选中可读会话后自动触发 transcript load；刷新时不先把已读 transcript 清空。
- `useAgentTranscriptLoader.ts` 保存已读 thread 的内存 cache；切回已读 thread 时可直接显示缓存 transcript。首次读取尚无正文时显示 loading 态，并明确“这不是 0 条结果”。
- `TranscriptViews.tsx` 改为虚拟消息窗口，`data-conversation-engine="virtualized"`；默认窗口显示最新消息，不把大对话全量放进 DOM。
- 修正一个收口时发现的虚拟滚动边界：初始显示最新不再通过 `scrollTop <= 0` 伪装，真实滚到顶部时可以对应最早窗口。

离线断言：
- 180 条 fixture transcript 初始只渲染最新窗口，包含 `Message fixture 179`，不包含早期 `Message fixture 20`。
- loading 同一 thread 时仍显示已读历史，并展示“正在刷新这条对话，已读历史保持可见。”
- 首次读取新会话时显示“正在读取这条对话”，并明示“这不是 0 条结果”。

## M2 自检：流畅层

实现：
- `TranscriptViews.tsx` 将 metadata `conversation_engine_streaming=true` 的最后一条 assistant event 从稳定虚拟窗口分离到 `.chat-streaming-tail`。
- 默认近底状态通过 `data-stick-to-bottom="true"` 暴露；新稳定消息和流式文本在近底时用 `requestAnimationFrame` 按真实 DOM `scrollHeight` 滚到底部，流式尾条高度计入滚动目标。
- 用户不在底部或存在流式尾条时显示“回到底部”入口。

离线断言：
- streaming fixture 的稳定窗口与流式尾条分离，`data-streaming-separated="true"`。
- streaming 文本作为单条自然流显示。

浏览器 / 真机证据：
- Codex in-app Browser 访问 `http://127.0.0.1:5173/` 被非 Tauri runtime 阻断，页面进入 `BootErrorBoundary`，错误为 `getCurrentWindow().metadata` undefined；因此不能把普通浏览器截图当作 UI 通过。
- 直接 `cargo run --no-default-features --color always --` 启动 Tauri desktop shell 成功进入 `target/debug/codex-governance-workbench`；启动后 5 秒内终端无新增 runtime 错误。但当前工具无法读取原生 Tauri webview DOM 或捕捉 Claude Preview 截图，所以“真机流畅手感 / Claude Preview 截图”仍是待用户在场验证项。

## M3 自检：撰写区

实现：
- `AgentChatComposer.tsx` 普通撰写区只保留主按钮“发送”，`data-send-mode="decision-only"`。
- Enter 提交，Shift+Enter 换行。
- `AgentConversationShell.tsx` 删除普通 shell 中旧 K2 preview / prepare / confirm / phase-a / phase-b 调用链；发送只构造 pending user message 并清空 draft。
- `conversationEngine.ts` 新增 `buildPendingUserMessage()` 与 `appendPendingUserMessage()`；pending event id 用 `createdAt + prompt` 参与 hash，避免同一 thread 连续发送相同 prompt 时 React key 重复；pending metadata 固定：
  - `conversation_engine_pending=true`
  - `conversation_engine_send_mode="decision_only"`
  - `real_codex_executed=false`
  - warning `pending_decision_only_no_codex_execution`

离线断言：
- 普通撰写区不再出现“生成发送预览”或“确认执行 Codex”。
- pending 消息立即出现在 transcript；metadata 证明不是 Codex 真实执行。
- 相同 prompt 在不同 createdAt 下会生成不同 pending event id。

## 边界核对

本包改了：
- 会话列表 / 会话页前端：分页地基消费、虚拟窗口、loading 保留历史、流式尾条、黏底、单道发送意图。
- 后端分页相关文件属于当前工作树已有未提交地基：`codex_db.rs`、`commands.rs`、`command_registry.rs`、`index_host_app_entrypoints.rs`、`types.rs`。
- 相关离线 fixture / helper，用于覆盖分页、虚拟化、撰写区边界。
- 新增 `useAgentTranscriptLoader.ts`，把 transcript 加载与已读缓存从 `AgentView.tsx` 拆出，避免 ratchet 文件增长。

本包未改：
- 记忆 store / 记忆 schema / FormalMemory 采纳路径。
- 编排 / 画布模块。
- 产品全局真实执行入口、real-resume 门、`run_real_resume_phase_b_with_runner()`。
- `.codex` 正文、auth、token、secret、prompt body 或完整 transcript 读取路径；未写 `.codex`。
- 气泡配色、水墨视觉方向、整体工作台布局。

Diff 级安全扫描：

```text
git diff --unified=0 -- prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/tests | rg -n "^\\+.*(previewRealExecutionProductCommand|prepareRealExecutionProductCommand|confirmRealExecutionProductCommand|runRealExecutionProductCommand|Command::new|codex exec|codex exec resume|/Users/yoyi/.codex|captureMemoryEvent|FormalMemory|memoryCandidate|projectWorkflowCanvas|canvas|orchestrat)"
exit 1, no matches

git diff --unified=0 -- prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/tests | rg -n "^\\+.*(生成发送预览|确认执行 Codex|写入准备|用户确认|记录预检|自动重试已启用|安全审查已绕过|已执行真实操作|K3-B2 可开始|result_count: 0)"
exit 1, no matches
```

## 验证原始输出

### capability-scan

任务包要求开工前跑 capability scan；本轮是中断恢复接手后补跑，作为 late pre-work scan 记录：

```text
Harness capability scan: /Users/yoyi/workspace/product-line

PASS (7)
  - Harness config readable: harness.config.json
  - Project type inference: mixed (harness config project.type)
  - Test files detected: 5
  - Harness rule file found: AGENTS.md
  - Harness rule file found: codex-multi-agent-safe-collaboration.md
  - Harness rule file found: skills/using-superpowers/SKILL.md
  - Runtime docs present: 11/11

WARN (10)
  - No package.json found; command detection is limited to files and PATH
  - No package manager field or lockfile found
  - No lint script detected
  - No typecheck script detected
  - No test script detected
  - No e2e script detected
  - No build script detected
  - No dev script detected
  - No E2E/browser test assets detected in shallow scan
  - No CI workflow detected

FAIL (0)
  None
```

warning 归类：仓库根无 `package.json`；实际前端脚本位于 `prototypes/productized-desktop-shell/`，本包已在该目录跑 `npm run typecheck` / `test:offline-interaction` / `build`。

### guard-state-files

受保护文件写入前已跑：

```text
Harness state-file guard: /Users/yoyi/workspace/product-line
PASS (19)
...
```

注：该命令输出巨大，列出受保护路径与 ignored target artifacts；此处只记录结论头部。命令 exit 0。

### npm run typecheck

```text
> codex-governance-workbench@0.1.0 typecheck
> tsc --noEmit
```

### npm run test:offline-interaction

```text
> codex-governance-workbench@0.1.0 test:offline-interaction
> node scripts/run-offline-interaction-test.mjs

offline interaction tests passed: 15
r4 page read model settings test passed
r4 page read model query contract test passed
r4 page read model runtime test passed
r4 page selectors test passed
```

### npm run build

```text
> codex-governance-workbench@0.1.0 build
> tsc --noEmit && vite build

vite v7.3.3 building client environment for production...
transforming...
✓ 258 modules transformed.
rendering chunks...
computing gzip size...
dist/index.html                   0.59 kB │ gzip:   0.42 kB
dist/assets/index-Cq18P1uG.css  145.61 kB │ gzip:  24.83 kB
dist/assets/index-BmgM9IwM.js   998.38 kB │ gzip: 273.58 kB

(!) Some chunks are larger than 500 kB after minification. Consider:
- Using dynamic import() to code-split the application
- Use build.rollupOptions.output.manualChunks to improve chunking: https://rollupjs.org/configuration-options/#output-manualchunks
- Adjust chunk size limit for this warning via build.chunkSizeWarningLimit.
✓ built in 1.18s
```

### cargo fmt -- --check

```text
<no output; exit 0>
```

### cargo test --lib codex_db -- --nocapture

```text
running 7 tests
test codex_db::tests::read_threads_falls_back_to_sqlite_title_without_session_index ... ok
test codex_db::tests::read_threads_page_can_target_archived_only ... ok
test codex_db::tests::read_threads_page_can_include_archived_for_explicit_archive_view ... ok
test codex_db::tests::read_threads_prefers_session_index_thread_name ... ok
test codex_db::tests::detects_parent_session_id_column_from_fixture_schema_only ... ok
test codex_db::tests::read_threads_page_filters_archived_and_limits_rows ... ok
test codex_db::tests::read_threads_legacy_helper_still_reads_all_pages ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 541 filtered out; finished in 1.44s
```

### cargo test --lib

```text
test result: ok. 526 passed; 0 failed; 22 ignored; 0 measured; 0 filtered out; finished in 8.77s
```

### shape gate

第一次 shape gate 发现 `styles.css` ratchet 文件增加 15 行，已修：撤回 CSS 文件新增，结构样式改为组件内联。最终输出：

第二次修复复核 P2 时 `AgentView.tsx` 因 transcript cache 增至 297/285，shape gate 再次 fail；已把加载/cache 逻辑拆到新文件 `useAgentTranscriptLoader.ts`，`AgentView.tsx` 降至 254/285。最终输出：

```text
Workbench shape gate: /Users/yoyi/workspace/product-line
Mode: check
Status: pass
Errors: 0
Warnings: 1
...
- prototypes/productized-desktop-shell/src/styles.css: 8464/8464 (same)
- prototypes/productized-desktop-shell/src/views/AgentView.tsx: 254/285 (decreased)
...
- [warn] tauri_command_total_increased: Tauri command total increased; confirm task package shape impact and non-lib.rs placement. {"current":99,"baseline":97}
```

warning 归类：来自当前工作树上一包分页地基新增窄 Tauri command，`lib.rs` 中 Tauri commands 为 0；本会话引擎 M1-M3 未新增 Tauri command。

### git diff --check

```text
<no output; exit 0>
```

### Browser fallback

```json
{
  "title": "Codex 治理工作台",
  "url": "http://127.0.0.1:5173/",
  "snapshotStart": "- alert:\n  - strong: 工作台启动失败\n  - generic: \"Uncaught TypeError: Cannot read properties of undefined (reading 'metadata')\"",
  "logs": [
    {
      "level": "error",
      "message": "TypeError: Cannot read properties of undefined (reading 'metadata') ... at getCurrentWindow ... at src/App.tsx ... BootErrorBoundary"
    }
  ]
}
```

结论：普通 browser fallback 不能作为 UI 通过证据。

### Tauri desktop startup probe

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 10s
Running `target/debug/codex-governance-workbench`
```

启动后短暂观察无新增终端 runtime error，随后由执行线 Ctrl+C 停止临时进程。未捕获原生窗口截图，未声明真机流畅度通过。

## 残余与交咨询线事项

- Claude Preview MCP 未暴露；没有 Claude Preview 截图证据。
- in-app Browser fallback 被非 Tauri runtime 阻断；不能验证真实 DOM / 交互。
- Tauri desktop shell 可启动，但执行线无法读取原生 webview DOM，也未做用户在场手感确认；大对话真机流畅度需要咨询线 / 用户在场继续验。
- TDD 严格红绿无法完整回放：当前工作是在已有 dirty worktree 与上一执行摘要基础上恢复；离线行为测试已覆盖 M1/M2/M3，但不能声称所有测试均先红后绿。

## 独立复核反馈与处理

复核线 Kepler（agent `019ed4fa-17a8-7101-8d17-23f126414b7c`）初审 `STATUS: CLEAR_WITH_P2`：

- P2：切到新 thread 首次读取时仍会短暂显示 loading 空态。处理：新增 `useAgentTranscriptLoader.ts` 做已读 transcript cache；首次未读 thread 改为明确 loading 态且提示“这不是 0 条结果”；离线测试补断言。
- P2：流式尾条分离后 `scrollToLatest()` 未计入尾条高度。处理：改用真实 DOM `scrollHeight`。
- P3：同一 thread 连续发送相同 prompt 的 pending event id 可能重复。处理：event id hash 纳入 `createdAt`；离线测试补断言。

修后复核结果见 `evidence/2026-06-17-conversation-engine-rebuild-full-execution-review-kepler-v1.md`：`STATUS: CLEAR_WITH_NOTE`，无 P0/P1/P2/P3；原 P2/P3 均已关闭。

## 最终收口复跑

### checkpoint-audit

第一次未传 `--review`，checkpoint audit 按 commit-only 默认未找到复核文件，失败在 `review_status`。随后带复核文件重跑通过：

```text
checkpoint-audit: /Users/yoyi/workspace/product-line
Package: (commit-only)
...
Resolved claims:
- impl commit:   HEAD
- task commit:   (none)
- review file:   evidence/2026-06-17-conversation-engine-rebuild-full-execution-review-kepler-v1.md
- review STATUS: (parsed at check)
...
Checks:
- [PASS] commits_reachable: [{"role":"impl","sha":"HEAD","verdict":"ok"}]
- [WARN] tree_clean: {"declared_dirty":true,"entries":[" M prototypes/productized-desktop-shell/src-tauri/src/codex_db.rs", "..."]}
- [PASS] review_status: {"file":"evidence/2026-06-17-conversation-engine-rebuild-full-execution-review-kepler-v1.md","status":"CLEAR"}
- [NA] current_md_refs: no --package slug; CURRENT.md cross-ref skipped
- [NA] files_within_allow: allow-list not provided/parseable; pass --allow to verify boundary
- [NA] gates_green: skipped (--skip-gates)
- [NA] evidence_hash_format: no --record; evidence JSON hash fields not inspected

VERDICT: PASS
```

注：复核文件的精确状态是 `STATUS: CLEAR_WITH_NOTE`；audit 机械解析为 `CLEAR`。

### final shape-gate / diff-check

```text
Workbench shape gate: /Users/yoyi/workspace/product-line
Mode: check
Status: pass
Errors: 0
Warnings: 1
...
- [warn] tauri_command_total_increased: Tauri command total increased; confirm task package shape impact and non-lib.rs placement. {"current":99,"baseline":97}
```

```text
git diff --check
<no output; exit 0>
```

### final safety scan

```text
git diff --unified=0 -- prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/tests | rg -n "^\\+.*(previewRealExecutionProductCommand|prepareRealExecutionProductCommand|confirmRealExecutionProductCommand|runRealExecutionProductCommand|Command::new|codex exec|codex exec resume|/Users/yoyi/.codex|captureMemoryEvent|FormalMemory|memoryCandidate|projectWorkflowCanvas|canvas|orchestrat)"
<no output; exit 1, no matches>

git diff --unified=0 -- prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/tests | rg -n "^\\+.*(生成发送预览|确认执行 Codex|写入准备|用户确认|记录预检|自动重试已启用|安全审查已绕过|已执行真实操作|K3-B2 可开始|result_count: 0)"
<no output; exit 1, no matches>
```

### local preview port note

```text
lsof -nP -iTCP:5173 -sTCP:LISTEN
COMMAND   PID USER   FD   TYPE             DEVICE SIZE/OFF NODE NAME
node    51147 yoyi   16u  IPv4 0xd775b5e70ad17027      0t0  TCP 127.0.0.1:5173 (LISTEN)
```

结论：`5173` 当前仍有既存 Vite / node 进程占用；执行线未擅自 kill。

## 不接受为

- 不接受为真实 Codex 执行已接通。
- 不接受为 K3-B1 / K3-B2 / real-resume 门已解锁。
- 不接受为 `.codex` 正文 / auth / secret / prompt body 可读。
- 不接受为记忆层、编排、画布或整体工作台布局重做。
- 不接受为视觉最终定稿。
- 不接受为真机流畅度已由执行线完整验收。
