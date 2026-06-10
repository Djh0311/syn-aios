# Evidence：Memory Layer M5 Conflict And Memory Lint Minimal Blocking v1

日期：2026-06-04

## 结论

接受为 M5 冲突和记忆 lint 最小阻断完成。

不接受为任务包注入完成；不接受为完整维护任务系统完成；不接受为正式记忆生命周期完成；不接受为中间版本记忆层完成。

## 实现范围

- 新增 `memory-lint.v1.json` sidecar 的后端 store / lock / atomic write / backup / damaged JSON refusal / revision conflict guard。
- 新增 deterministic `MemoryLintEngine`：duplicate claim、authority superseded、revoked source、candidate conflicts with active memory。
- `adopt_memory_candidate_to_formal_memory` 采纳前执行 lint guard；本次 run 产生 open blocking finding 时拒绝采纳，不写正式 record / version / adoption audit。
- `TaskMemoryPacketBuilder` 读取 lint store；open blocking finding 命中正式记忆时排除为 `conflicted`，并输出 `memory_lint_blocking_findings_excluded` warning。
- 前端新增 Tauri wrapper、lint 摘要 helper、项目工作流候选治理侧栏的只读“记忆 lint 阻断摘要”。
- 离线测试覆盖 open / blocking / needs_review 计数、required UI 文案和 forbidden UI 文案。

## 改动文件

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_lint_engine.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_lint_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/control_core.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/task_memory_packet_builder.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/lib/candidateGovernance.ts`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

## 验证

在 `prototypes/productized-desktop-shell`：

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，9 scenarios。
- `npm run build`：通过；Vite 保留既有 chunk size warning。

在 `prototypes/productized-desktop-shell/src-tauri`：

- `cargo test --lib memory_lint`：通过，9 passed。
- `cargo test --lib task_memory_packet`：通过，10 passed。
- `cargo test --lib memory_candidate_adoption`：通过，7 passed。
- `cargo test --lib formal_memory`：通过，18 passed。
- `cargo test --lib`：通过，164 passed，1 ignored。
- `rustfmt --check src/memory_lint_store.rs src/memory_lint_engine.rs src/formal_memory_store.rs src/memory_candidate_store.rs src/task_memory_packet_builder.rs src/control_core.rs src/commands.rs src/types.rs`：通过。

Rust 测试仍有既有 warning：`JsonRpcError::invalid_params` unused。

## UI 验收状态

项目页 UI 已通过 React SSR 离线测试覆盖 required / forbidden 文案。

补充浏览器 smoke：

- 启动 Vite dev server：`npm run dev -- --port 4177`。
- 用 Codex in-app Browser 打开 `http://127.0.0.1:4177/`。
- 切换到“项目”主导航。
- 结果：项目页占位可见，普通浏览器明确显示没有 Tauri 数据桥；console error / warning 为空。
- 局限：普通浏览器没有 Tauri 数据桥，未显示真实项目数据，也未显示 `记忆 lint 阻断摘要`。

真实 Tauri 数据态 / 真实项目页 lint 摘要截图验收未完成：本轮未启动真实 Tauri 窗口。

## 边界确认

- 未执行真实 worker。
- 未执行真实 Codex。
- 未执行 `codex exec` / `codex exec resume`。
- 未读写 `/Users/yoyi/.codex`。
- 未迁移数据库。
- 未改 `workflow-state.v0.json` 结构。
- 未自动修改正式记忆状态。
- 未自动新增正式记忆版本。
- 未自动写 `MemoryRecord.conflict_refs[]`。
- 未把 candidate / observation / knowledge hit / LLM summary 当正式记忆。
