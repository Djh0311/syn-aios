# Root Treatment / R4-H2-3 Frontend Page Query Consumption And Snapshot Ratchet Slimming v1

日期：2026-06-13

状态：已完成。

性质：R4 硬目标 / 批一 snapshot 按页查询的第 3 包。本包在 H2-1 / H2-2 基础上，把前端读取入口从一次性 `loadWorkbenchSnapshot()` 切到六个 page query，并伴随最小 H2-4 `types.rs` 瘦身，确保本包降低棘轮指标。

Planning baseline：`a5cd7b3`。

## 0. 全局主管理解

H2-1 / H2-2 已完成：

- 六页 schema 和 coverage 已定义。
- `query_workbench_page_read_model` 返回六页 `page_data_ready` payload。
- 前端仍在 `App.tsx` 通过 `loadWorkbenchSnapshot()` 一次性拉整包。

本包目标：

- 前端 reload 改为并行查询六个目标页：`projects`、`agents`、`running_workflows`、`memory`、`knowledge`、`settings`。
- 页面仍收到与原来等价的只读数据，视觉 / 交互零变更。
- 不拆 `ProjectsView` / `AgentView`。
- 不进入批二。

## 1. 目标

完成后：

- `App.tsx` 不再调用 `loadWorkbenchSnapshot()`。
- `App.tsx` 通过六个 `queryWorkbenchPageReadModel({ page_id })` 结果合成当前前端只读 snapshot。
- 六个 page payload 都提供 `snapshot_slice`，前端只从 page query payload 合成所需字段。
- 非批一页面不冒充独立完成；它们只是消费六页 payload 合成后的只读数据。
- 页面视觉、布局、CSS、文案和交互保持不变。

## 2. 形状影响

预期：

- `types.rs` 通过移动 WorkbenchSnapshot 相邻索引 / 项目 / 会话 / 诊断结构继续下降，预计下降 150 行以上。
- `App.tsx` 通过移出 `emptySnapshot` 降低行数，但 `App.tsx` 不在 shape gate ratchet 中，仅作为辅助简化。
- 不新增 Tauri command；command baseline 保持 97 total，0 in `lib.rs`。
- 新增 TS helper 文件低于 2,000 行。
- `page_read_model.rs` 增长但保持低于 3,000 行。

## 3. 允许范围

允许修改：

- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/lib/pageReadModel.ts`
- `prototypes/productized-desktop-shell/src/lib/pageReadModelRuntime.ts`
- `prototypes/productized-desktop-shell/src/lib/emptySnapshot.ts`
- `prototypes/productized-desktop-shell/src-tauri/src/page_read_model.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_snapshot_types.rs`
- 相关离线测试 / test runner。
- 当前任务包、evidence、handoff。

允许新增：

- `prototypes/productized-desktop-shell/src/lib/pageReadModelRuntime.ts`
- `prototypes/productized-desktop-shell/src/lib/emptySnapshot.ts`
- 必要的离线测试文件。

## 4. 禁止范围

禁止：

- 修改 UI / CSS / 水墨风格 / 布局 / 文案 / 交互。
- 拆分 `ProjectsView` / `AgentView` 或其他 View。
- 新增 Tauri command 或把 command 放回 `lib.rs`。
- 修改 DB / sidecar schema / workflow state schema。
- 执行真实 `codex exec` / `codex exec resume`，发送 prompt，读写 `/Users/yoyi/.codex`。
- 启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 进入 R3 Level B、批二 View 拆分或 backlog 解冻。

## 5. 实现步骤

1. H2-3 schema 修正：在六页后端 payload 中新增 `snapshot_slice`，包含该页覆盖的 `WorkbenchSnapshot` 顶层字段；必要时补齐项目页 / 智能体页实际消费字段。
2. 新增前端 runtime helper：定义六页 page id、合并 page payload snapshot slice、缺页 warning 和 fallback。
3. 移出 `emptySnapshot`，保持前端兜底语义不变。
4. `App.tsx` reload 改为 page query；删除 `loadWorkbenchSnapshot()` 直接调用。
5. 补离线测试：确认六页 payload 可合成完整 snapshot，且缺失 / 非 ready page 不会伪装为完整数据。
6. 最小 H2-4：移动 WorkbenchSnapshot 相邻基础结构到 `workbench_snapshot_types.rs`，降低 `types.rs`。

## 6. 验证

必须通过：

- `cargo test --lib page_read_model`
- `cargo test --lib`
- `cargo fmt -- --check`
- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

建议扫描：

- `rg -n "loadWorkbenchSnapshot" prototypes/productized-desktop-shell/src`
- `rg -n "queryWorkbenchPageReadModel|snapshot_slice|workbench_page_query" prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/tests`
- `wc -l prototypes/productized-desktop-shell/src-tauri/src/types.rs prototypes/productized-desktop-shell/src-tauri/src/workbench_snapshot_types.rs prototypes/productized-desktop-shell/src/App.tsx`

## 7. 复核要求

复核线重点检查：

- 前端是否真的不再调用 `loadWorkbenchSnapshot()`。
- 六页是否通过 page query payload 合成前端只读数据。
- 视觉 / 文案 / CSS / View 拆分是否为零变更。
- `types.rs` 是否下降，且移动类型字段 / derive / 可见性是否等价。
- 是否新增 command、改 schema、触发真实执行、接触 `.codex`、进入批二或解冻 backlog。

## 8. 不接受为

本包不接受为：

- `WorkbenchSnapshot` 后端结构已废弃。
- 批一全部完成。
- 批二 View 拆分开始。
- UI 重做或视觉验收完成。
- R3 Level B 执行。
- 真实 Codex 执行、`.codex` 读写或 backlog 功能解冻。

## 9. 实现记录

实现完成时间：2026-06-13。

实现结果：

- `App.tsx` 删除直接 `loadWorkbenchSnapshot()` 调用，改为 `loadWorkbenchSnapshotFromPageQueries(queryWorkbenchPageReadModel)`。
- 新增 `pageReadModelRuntime.ts`，固定六个 batch-one page id，并从 page query payload 的 `snapshot_slice` 合成前端只读 snapshot；缺页 / 非 ready / payload 来源异常 / 缺 slice 均返回 warning，同时保持兜底数据。
- `page_read_model.rs` 的六页 payload 新增 `snapshot_slice`，每页只包含 H2-1 schema 声明覆盖的 snapshot 顶层字段。
- `query_page_read_model_with_snapshot_value` 的目标页 query result 更新为 `ui_consumption_status="page_query_payload_ready"`，不再输出 H2-2 的“frontend not migrated”误导 warning。
- `emptySnapshot` 从 `App.tsx` 移到 `emptySnapshot.ts`，兜底内容不变。
- `FileCandidate` / `ProjectRecord` / `SessionRecord` / `SkillRecord` / `PluginRecord` / `TaskEntry` / `Diagnostics` / `IndexSummary` 等 WorkbenchSnapshot 相邻基础类型移入 `workbench_snapshot_types.rs`。
- 新增 `r4-page-read-model-runtime.test.ts` 并纳入离线测试入口。

形状结果：

- `types.rs`：5364 -> 5229，下降 135 行；相对 H2-1 waterline 5386 下降 157 行。
- `workbench_snapshot_types.rs`：23 -> 158 行。
- `App.tsx`：1176 -> 1037 行。
- `page_read_model.rs`：1335 -> 1377 行，仍低于 3,000 行。
- Tauri commands：97 total，0 in `lib.rs`。

验证已通过：

- `cargo test --lib page_read_model`：7 passed。
- `cargo test --lib`：475 passed，16 ignored。
- `cargo fmt -- --check`：通过。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过；offline interaction 14 passed，R4 page settings/query/runtime/selectors tests 通过。
- `npm run build`：通过，仅既有 Vite chunk size warning。
- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors，0 warnings；`types.rs: 5229/5386 (decreased)`。
- `git diff --check`：通过。

扫描：

- `rg -n "loadWorkbenchSnapshot\\(|load_workbench_snapshot\\(" prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/tests` 仅命中 `src/lib/tauri.ts` wrapper；`App.tsx` 无直接整包读取。
- `rg -n "queryWorkbenchPageReadModel|snapshot_slice|workbench_page_query|page_query_payload_ready" ...` 命中 App、runtime helper、page read model 和测试。

边界确认：

- 未改 UI / CSS / 水墨风格 / 布局 / 文案 / 交互。
- 未拆 `ProjectsView` / `AgentView` 或其他 View。
- 未新增 Tauri command，未把 command 放入 `lib.rs`。
- 未改 DB / sidecar schema / workflow state schema。
- 未执行真实 `codex exec` / `codex exec resume`，未发送 prompt，未读写 `/Users/yoyi/.codex`。
- 未启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 未进入 R3 Level B、批二 View 拆分或 backlog 解冻。

复核结论：

- 独立复核线 Faraday 返回 `STATUS: CLEAR`。
- P0/P1：无。
- P2：runtime helper warning 测试未逐分支覆盖 non-ready / unexpected source / missing slice 三个分支；已有完整六页无 warning、缺页 warning 和 fallback 不冒充完整数据测试。本项不阻断。
