# Agent Session Shell Rebuild Pagination Virtual Scroll v1

日期：2026-06-17

阶段：UI track · 会话外壳重做

状态：`completed_pending_consultation_review`

## 拍板摘要

本包把会话中心从“前端一次吃完整列表”推进到“Agent 页显式分页读取会话元数据 + 默认隔离归档 + 列表 DOM 窗口化 + 加载更多”。代价是新增一个窄 Tauri 命令 `load_codex_session_page`（只读会话元数据），以及前端少量结构拆分；不做会继续卡在大列表。普通浏览器验证因当前 app 依赖 Tauri runtime 被阻断，真机 Tauri / Claude Preview 验证仍需用户在场补。

一句话判据：本包可接受为“会话中心具备分页/窗口化/归档隔离的产品路径和 fixture 验证”，不可接受为“真实 `.codex` 数据量下已由用户验不卡”或“subagent 折叠已实现”。

## 改动范围

- `prototypes/productized-desktop-shell/src-tauri/src/codex_db.rs`
  - 新增 `read_threads_page()`：默认 `archived = 0`，支持 `archived_only` 显式归档页，`LIMIT page_size + 1 / OFFSET` 判断 `has_more`。
  - `read_threads()` 保持旧兼容语义：循环分页读完所有 rows，避免旧 transcript lookup / rollout allowlist 被 250 上限截断。
  - 新增 fixture-only `has_parent_session_id_column()` schema 探测；本包只发现，不实现 subagent 折叠。
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs` / `command_registry.rs` / `types.rs`
  - 新增只读 Tauri command `load_codex_session_page` 与 DTO。
  - SQLite 不可用时回落到 index 分页并返回 warning。
- `prototypes/productized-desktop-shell/src/views/agent/useAgentSessionPage.ts`
  - 新增 Agent 页分页 hook；Tauri 可用时请求分页，非 Tauri/offline 下回落到 snapshot/fixture。
- `prototypes/productized-desktop-shell/src/views/agent/AgentSessionList.tsx`
  - 过滤语义收紧：`readable` / `all` / `missing` 均排除 archived；只有 `archived` 显式归档视图显示 archived。
  - 默认只渲染 40 条可见会话卡，提供“显示更多会话”；显示当前渲染数、分页来源和归档隔离说明。
- `prototypes/productized-desktop-shell/src/views/agent/AgentSoftwareFilterBar.tsx`
  - 抽出软件筛选条，避免 `AgentView.tsx` 撞 ratchet 水线。
- `prototypes/productized-desktop-shell/src/views/agent/TranscriptViews.tsx`
  - 长 transcript 收纳提示新增“回到最新消息”按钮，不强拉、不新增正文读取。
- `prototypes/productized-desktop-shell/tests/helpers/offlineAgentSessionShellPaginationScenario.tsx`
  - 新增离线分页/窗口化/归档隔离场景，主测试文件保持水线。

## TDD / 测试说明

- 新增前端离线断言先红：大列表默认窗口化断言失败，随后实现通过。
- 后端新增分页/归档 fixture 测试；上一执行线已先写部分分页代码，因此本包不能声称全程严格 TDD，只能声称新增行为按 red/green 和 verify-after 收口。

## Subagent Folding 发现

- 未读取真实 `.codex`。
- 仅在 fixture sqlite schema 上测试 `parent_session_id` 探测。
- 当前 fixture baseline 不含 `parent_session_id`；当 fixture `ALTER TABLE threads ADD COLUMN parent_session_id TEXT` 后 helper 能识别。
- 本包未实现折叠 UI / 数据模型。

## 浏览器验证

工具口径：

- Claude Preview MCP 未在本会话暴露可调用工具。
- 按 `ui-browser-verification` fallback 使用 in-app Browser。

实际结果：

```text
URL: http://127.0.0.1:5173/
Title: Codex 治理工作台
Result: 普通浏览器被现有 Tauri runtime 依赖阻断。
Error: getCurrentWindow() 读取 undefined metadata，BootErrorBoundary 接管。
```

分类：

- 这不是本包新增错误；当前 `App.tsx` 普通浏览器路径本来依赖 Tauri runtime。
- 因任务边界要求开发期不读真实 `.codex`，本包未擅自启动 `tauri:dev` 做真实数据验证。
- 真机 Tauri / Claude Preview 截图和真实数据量“不卡”验证仍是未完成项，应由用户在场窗口补。

## 验证原始输出

`node scripts/harness/capability-scan.js --target .`

```text
PASS (7)
WARN (10)
FAIL (0)
```

前端 red test（实现前）

```text
Error: 大列表应默认只渲染首个虚拟窗口
```

`cargo test --lib codex_db -- --nocapture`

```text
running 7 tests
test codex_db::tests::read_threads_falls_back_to_sqlite_title_without_session_index ... ok
test codex_db::tests::read_threads_prefers_session_index_thread_name ... ok
test codex_db::tests::detects_parent_session_id_column_from_fixture_schema_only ... ok
test codex_db::tests::read_threads_page_can_target_archived_only ... ok
test codex_db::tests::read_threads_page_can_include_archived_for_explicit_archive_view ... ok
test codex_db::tests::read_threads_page_filters_archived_and_limits_rows ... ok
test codex_db::tests::read_threads_legacy_helper_still_reads_all_pages ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 541 filtered out; finished in 0.13s
```

`npm run typecheck`

```text
> codex-governance-workbench@0.1.0 typecheck
> tsc --noEmit
```

`npm run test:offline-interaction`

```text
offline interaction tests passed: 15
r4 page read model settings test passed
r4 page read model query contract test passed
r4 page read model runtime test passed
r4 page selectors test passed
```

`npm run build`

```text
vite v7.3.3 building client environment for production...
✓ 256 modules transformed.
✓ built in 3.63s
```

备注：Vite 仍报告既有 chunk size warning（> 500 kB），本包未处理打包拆分。

`cargo test --lib`

```text
running 548 tests
test result: ok. 526 passed; 0 failed; 22 ignored; 0 measured; 0 filtered out; finished in 11.48s
```

`cargo fmt -- --check`

```text
<no output; exit 0>
```

`node scripts/harness/workbench-shape-gate.js --mode check`

```text
Status: pass
Errors: 0
Warnings: 1
Tauri commands: 99 total; 0 in lib.rs
warning: tauri_command_total_increased {"current":99,"baseline":97}
AgentView.tsx: 281/285 (decreased)
offline-permission-dialog.test.tsx: 3404/3404 (same)
```

`git diff --check`

```text
<no output; exit 0>
```

## 边界与不可声称

- 未读取真实 `/Users/yoyi/.codex`，未写 `.codex`。
- 未读 transcript 正文、auth、token、secret、`.env`、keychain、OAuth/provider credential。
- 未执行 `codex exec` / `codex exec resume`，未启动 K3-B1 / K3-B2。
- 未改 R3 DB 切换、记忆层、执行语义。
- 未实现 subagent 折叠。
- 未完成真实 Tauri / 用户 Mac 大数据量不卡验收。
