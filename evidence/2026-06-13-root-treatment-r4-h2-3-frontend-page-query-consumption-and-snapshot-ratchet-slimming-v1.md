# Evidence: Root Treatment / R4-H2-3 Frontend Page Query Consumption And Snapshot Ratchet Slimming v1

日期：2026-06-13

状态：已完成，并通过独立复核。

## 1. 范围

本轮执行：

- `tasks/2026-06-13-root-treatment-r4-h2-3-frontend-page-query-consumption-and-snapshot-ratchet-slimming-v1.md`

本轮只做 H2-3 + 最小 H2-4：

- 前端读取入口从直接 `loadWorkbenchSnapshot()` 切到六个 page query。
- 后端六页 payload 增加只读 `snapshot_slice`，供前端合成等价只读数据。
- `types.rs` 继续瘦身，降低棘轮指标。

## 2. 实现内容

修改：

- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src-tauri/src/page_read_model.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_snapshot_types.rs`
- `prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs`
- `prototypes/productized-desktop-shell/tests/r4-page-read-model-query-contract.test.ts`
- `tasks/2026-06-13-root-treatment-r4-h2-3-frontend-page-query-consumption-and-snapshot-ratchet-slimming-v1.md`

新增：

- `prototypes/productized-desktop-shell/src/lib/emptySnapshot.ts`
- `prototypes/productized-desktop-shell/src/lib/pageReadModelRuntime.ts`
- `prototypes/productized-desktop-shell/tests/r4-page-read-model-runtime.test.ts`
- `evidence/2026-06-13-root-treatment-r4-h2-3-frontend-page-query-consumption-and-snapshot-ratchet-slimming-v1.md`

## 3. Page Query Consumption

前端读取：

- `App.tsx` 不再直接调用 `loadWorkbenchSnapshot()`。
- `reload()` 调用 `loadWorkbenchSnapshotFromPageQueries(queryWorkbenchPageReadModel)`。
- `run-project-workflow-automation-phase-a` 后的刷新同样走 page query。
- `tauri.ts` 中 `loadWorkbenchSnapshot()` wrapper 保留，作为后端兼容 wrapper；本轮未删除 command。
- `pageReadModelRuntime.ts` 对缺页 / 非 ready / payload 来源异常 / 缺少 `snapshot_slice` 返回 warning，同时使用 `emptySnapshot` 兜底，不把缺页冒充完整数据。

后端 payload：

- 六页 `page_payload.data.snapshot_slice` 按 H2-1 schema 的 `snapshot_fields` 输出。
- 目标页 query result 设置 `selector_plan.ui_consumption_status="page_query_payload_ready"`。
- 目标页 `target_schema.returns_business_data=true`、`target_schema.page_ui_migrated=true`。
- payload warnings 改为 `snapshot_slice_read_only`，不再输出 H2-2 的 `frontend_page_consumption_not_migrated`。

## 4. 形状结果

```text
5229 prototypes/productized-desktop-shell/src-tauri/src/types.rs
158  prototypes/productized-desktop-shell/src-tauri/src/workbench_snapshot_types.rs
1037 prototypes/productized-desktop-shell/src/App.tsx
90   prototypes/productized-desktop-shell/src/lib/pageReadModelRuntime.ts
142  prototypes/productized-desktop-shell/src/lib/emptySnapshot.ts
1377 prototypes/productized-desktop-shell/src-tauri/src/page_read_model.rs
```

对比：

- `types.rs`：5364 -> 5229，本包下降 135 行。
- `types.rs` 相对 shape gate waterline 5386：下降 157 行。
- `App.tsx`：1176 -> 1037，下降 139 行。
- `page_read_model.rs`：1377 行，低于 3,000 行。
- Tauri commands：97 total，0 in `lib.rs`。

shape gate：

```text
Status: pass
Errors: 0
Warnings: 0
types.rs: 5229/5386 (decreased)
Tauri commands: 97 total; 0 in lib.rs
```

## 5. 验证

已通过：

```text
cargo test --lib page_read_model
```

结果：7 passed。

```text
cargo test --lib
```

结果：475 passed，16 ignored；保留既有 `JsonRpcError::invalid_params` dead code warning。

```text
cargo fmt -- --check
```

结果：通过。

```text
npm run typecheck
```

结果：通过。

```text
npm run test:offline-interaction
```

结果：

```text
offline interaction tests passed: 14
r4 page read model settings test passed
r4 page read model query contract test passed
r4 page read model runtime test passed
r4 page selectors test passed
```

```text
npm run build
```

结果：通过；保留既有 Vite chunk size warning。

```text
node scripts/harness/workbench-shape-gate.js --mode check
```

结果：pass，0 errors，0 warnings。

```text
git diff --check
```

结果：通过。

## 6. 扫描

整包 snapshot 直接读取扫描：

```text
rg -n "loadWorkbenchSnapshot\\(|load_workbench_snapshot\\(" prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/tests
```

结果：仅命中 `prototypes/productized-desktop-shell/src/lib/tauri.ts:136` 的 `loadWorkbenchSnapshot()` wrapper；`App.tsx` 和 tests 无直接整包读取。

Page query / slice 扫描：

```text
rg -n "queryWorkbenchPageReadModel|snapshot_slice|workbench_page_query|page_query_payload_ready" prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/tests prototypes/productized-desktop-shell/src-tauri/src/page_read_model.rs
```

结果：命中 App、`pageReadModelRuntime.ts`、后端 `page_read_model.rs` 和 R4 page tests。

## 7. 边界确认

本轮没有：

- 修改 UI / CSS / 水墨风格 / 布局 / 文案 / 交互。
- 拆分 `ProjectsView` / `AgentView` 或其他 View。
- 新增 Tauri command 或把 command 放回 `lib.rs`。
- 修改 DB / sidecar schema / workflow state schema。
- 执行真实 `codex exec` / `codex exec resume`。
- 发送 prompt。
- 读写 `/Users/yoyi/.codex`。
- 启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 进入 R3 Level B、批二 View 拆分或 backlog 解冻。

## 8. 外部脏文件

本轮开始前已有外部脏文件，未纳入本包修改：

- `AGENTS.md`
- `backlog.md`
- `docs/own-agent-and-company-vision-v1.md`
- `handoffs/2026-06-13-r4-hard-targets-execution-plan-supervisor-v1.md`

## 9. 不接受为

本轮不接受为：

- 后端 `WorkbenchSnapshot` 结构已废弃。
- 批一全部完成。
- 批二 View 拆分开始。
- UI 重做或视觉验收完成。
- R3 Level B 执行。
- 真实 Codex 执行、`.codex` 读写或 backlog 功能解冻。

## 10. 复核结论

独立复核线 Faraday 返回 `STATUS: CLEAR`。

- P0/P1：无。
- 已确认 `git diff --cached --name-only` 仅包含 H2-3 实现、测试、任务包和 evidence；未 staged 的 `AGENTS.md`、`backlog.md`、`docs/own-agent-and-company-vision-v1.md` 已隔离，不计入本包。
- 已确认 `App.tsx` 通过六页 page query 合成 snapshot，直接 `loadWorkbenchSnapshot()` 只剩 `tauri.ts` wrapper。
- 已确认后端 `snapshot_slice` 按 schema `snapshot_fields` 拷贝。
- 已确认 cached diff 未触 UI/CSS/`ProjectsView`/`AgentView`，Tauri command 总数仍为 97，`git diff --cached --check` 通过。
- P2：runtime helper warning 测试未逐分支覆盖 non-ready / unexpected source / missing slice 三个分支；已有完整六页无 warning、缺页 warning 和 fallback 不冒充完整数据测试。本项不阻断。
