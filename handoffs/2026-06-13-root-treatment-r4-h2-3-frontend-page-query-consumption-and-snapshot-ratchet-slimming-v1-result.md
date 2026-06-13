# Handoff: Root Treatment / R4-H2-3 Frontend Page Query Consumption And Snapshot Ratchet Slimming v1

日期：2026-06-13

状态：已完成，并通过独立复核。

## 1. 完成内容

H2-3 已完成：

- 前端 `App.tsx` 不再直接调用 `loadWorkbenchSnapshot()`。
- 前端 reload 改为通过六个 page query 合成只读 snapshot。
- 六页后端 page payload 新增只读 `snapshot_slice`。
- Runtime helper 对缺页 / 非 ready / payload 来源异常 / 缺 slice 返回 warning，并保留 `emptySnapshot` 兜底。
- `types.rs` 通过 WorkbenchSnapshot 相邻基础类型迁移继续下降。

## 2. 文件

修改：

- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src-tauri/src/page_read_model.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_snapshot_types.rs`
- `prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs`
- `prototypes/productized-desktop-shell/tests/r4-page-read-model-query-contract.test.ts`
- `tasks/2026-06-13-root-treatment-r4-h2-3-frontend-page-query-consumption-and-snapshot-ratchet-slimming-v1.md`
- `evidence/2026-06-13-root-treatment-r4-h2-3-frontend-page-query-consumption-and-snapshot-ratchet-slimming-v1.md`

新增：

- `prototypes/productized-desktop-shell/src/lib/emptySnapshot.ts`
- `prototypes/productized-desktop-shell/src/lib/pageReadModelRuntime.ts`
- `prototypes/productized-desktop-shell/tests/r4-page-read-model-runtime.test.ts`
- `handoffs/2026-06-13-root-treatment-r4-h2-3-frontend-page-query-consumption-and-snapshot-ratchet-slimming-v1-result.md`

## 3. 关键结果

前端消费：

- `App.tsx` 通过 `loadWorkbenchSnapshotFromPageQueries(queryWorkbenchPageReadModel)` 读取六页。
- 直接 `loadWorkbenchSnapshot()` 只保留在 `src/lib/tauri.ts` wrapper 中，当前 App 不消费。

后端 payload：

- `page_payload.data.snapshot_slice` 按 H2-1 schema 的 `snapshot_fields` 输出。
- 目标页 `ui_consumption_status="page_query_payload_ready"`。
- `writes_stores=false`，不改 schema，不执行真实 runner。

形状：

- `types.rs`：5364 -> 5229。
- `workbench_snapshot_types.rs`：23 -> 158。
- `App.tsx`：1176 -> 1037。
- `page_read_model.rs`：1335 -> 1377，低于 3,000。
- Tauri commands：97 total，0 in `lib.rs`。

## 4. 验证

已通过：

- `cargo test --lib page_read_model`：7 passed。
- `cargo test --lib`：475 passed，16 ignored。
- `cargo fmt -- --check`：通过。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：14 passed，R4 page settings/query/runtime/selectors tests 通过。
- `npm run build`：通过，仅既有 Vite chunk size warning。
- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors，0 warnings；`types.rs: 5229/5386 (decreased)`。
- `git diff --check`：通过。

## 5. 复核结论

独立复核线 Faraday 返回 `STATUS: CLEAR`。

- P0/P1：无。
- P2：runtime helper warning 测试未逐分支覆盖 non-ready / unexpected source / missing slice 三个分支；已有完整六页无 warning、缺页 warning 和 fallback 不冒充完整数据测试。本项不阻断。

## 6. 边界确认

本轮未改 UI / CSS / 水墨风格 / 布局 / 文案 / 交互，未拆 `ProjectsView` / `AgentView`，未新增 Tauri command，未改 DB / sidecar schema / workflow state schema，未执行真实 Codex，未发送 prompt，未读写 `/Users/yoyi/.codex`，未启动 Tauri / Browser / Chrome / Vite dev / screenshot，未进入 R3 Level B、批二 View 拆分或 backlog 解冻。

## 7. 外部脏文件

以下文件是本包前已存在或外部更新，未纳入 H2-3 staged diff / commit 范围：

- `AGENTS.md`
- `backlog.md`
- `docs/own-agent-and-company-vision-v1.md`
- `handoffs/2026-06-13-r4-hard-targets-execution-plan-supervisor-v1.md`

## 8. 不接受为

本轮不接受为后端 `WorkbenchSnapshot` 结构已废弃、批一全部完成、批二 View 拆分开始、UI 重做、R3 Level B 执行、真实 Codex 执行、`.codex` 读写或 backlog 功能解冻。
