# Root Treatment / R4-H2-2 Backend Page Query Business Payload v1

日期：2026-06-13

状态：已完成。

性质：R4 硬目标 / 批一 snapshot 按页查询的第 2 包。本包在 H2-1 schema / coverage 基础上，让现有 `query_workbench_page_read_model` 返回六个目标页的后端业务 payload；不新增 Tauri command，不切前端消费。

Planning baseline：`6c4a050`。

## 0. 全局主管理解

H2-1 已完成：

- 六页 schema catalog 已定义。
- `WorkbenchSnapshot` 20 个顶层字段 coverage 已覆盖。
- query 仍是 schema-only，`returns_business_data=false`。

H2-2 的目标是把 existing query 从 schema-only 推进到 backend page data ready：

- 复用现有 `build_snapshot`，避免重写 index/session/sidecar 读取逻辑。
- 现有 command `query_workbench_page_read_model` 继续留在 `commands.rs`，不新增 command，不进 `lib.rs`。
- 返回 payload 后仍不改 UI，不让前端页面切流。

## 1. 目标

对六个目标页返回后端业务 payload：

- `projects`
- `agents`
- `running_workflows`
- `memory`
- `knowledge`
- `settings`

完成后：

- 这六页 `PageReadModelQueryResult.status` 应为 `page_data_ready`。
- `source_boundary.returns_business_data=true`。
- `source_boundary.writes_stores=false`。
- `source_boundary.tauri_command_migrates_page=false`。
- `page_payload` 包含 `page_id`、`schema_version`、`generated_from="workbench_page_query"`、`data`、`warnings`。
- `home` / `skill` / `harness` 等非批一页面仍保持 schema / contract only，不冒充完成。

## 2. 形状影响

预期：

- 不新增 Tauri command；command baseline 保持 97 total，0 in `lib.rs`。
- `types.rs` 通过最小 H2-4 伴随项下降：将 `WorkbenchSnapshot` struct 移入 `workbench_snapshot_types.rs` 并在 `types.rs` include，预计下降约 15 行。
- `page_read_model.rs` 增长但保持低于 3,000 行。
- 不增长 `lib.rs`、`commands.rs` ratchet。

## 3. 允许范围

允许修改：

- `prototypes/productized-desktop-shell/src-tauri/src/page_read_model.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_snapshot_types.rs`
- `prototypes/productized-desktop-shell/src/lib/pageReadModel.ts`
- 相关离线测试 / fixture。
- 当前任务包、evidence、handoff。

允许新增：

- `prototypes/productized-desktop-shell/src-tauri/src/workbench_snapshot_types.rs`

## 4. 禁止范围

禁止：

- 新增 Tauri command。
- 修改 `load_workbench_snapshot` 返回结构或语义。
- 让前端页面消费按页 query。
- 拆 `ProjectsView` / `AgentView` 或任何 View。
- 修改 UI、CSS、水墨风格、布局、文案或交互。
- 修改 DB / sidecar schema / workflow state schema。
- 执行真实 `codex exec` / `codex exec resume`，发送 prompt，读写 `/Users/yoyi/.codex`。
- 启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 触碰 R3 Level B、解冻 backlog 或进入批二。

## 5. 实现步骤

1. 新增 `PageReadModelPayload`，以 `serde_json::Value` 承载后端页 payload，避免在 H2-2 大规模复制前端 UI 类型。
2. 新增 `query_page_read_model_with_snapshot_value`，接收 snapshot JSON 和 workflow state JSON，派生六页 payload。
3. `commands.rs` 中现有 `query_workbench_page_read_model` 复用 `build_snapshot`，序列化 snapshot 后调用 H2-2 query。
4. 保持非批一页面 contract-only。
5. TS `pageReadModel.ts` 同步 `PageReadModelPayload`。
6. 最小 H2-4 伴随项：移动 Rust `WorkbenchSnapshot` struct，降低 `types.rs` 行数。
7. 补 Rust/TS 契约测试，确认六页 payload、非批一不冒领、无新增 command。

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

- `rg -n "page_data_ready|workbench_page_query|returns_business_data|query_workbench_page_read_model" prototypes/productized-desktop-shell/src-tauri/src prototypes/productized-desktop-shell/src/lib/pageReadModel.ts`
- `rg -n "#\\[tauri::command\\]" prototypes/productized-desktop-shell/src-tauri/src | wc -l`
- `wc -l prototypes/productized-desktop-shell/src-tauri/src/types.rs prototypes/productized-desktop-shell/src-tauri/src/workbench_snapshot_types.rs prototypes/productized-desktop-shell/src-tauri/src/page_read_model.rs`

## 7. 复核要求

复核线重点检查：

- 六个目标页是否返回 `page_data_ready` 和 payload。
- 非批一页面是否仍 contract-only。
- 是否新增 command 或改 `load_workbench_snapshot` 语义。
- `types.rs` 是否下降，`WorkbenchSnapshot` include 是否保持结构不变。
- 是否误触 UI、DB、schema、真实执行、`.codex` 或批二。

## 8. 不接受为

本包不接受为：

- H2-3 前端已切到按页取数。
- `WorkbenchSnapshot` 已废弃。
- 批一完成。
- 批二开始。
- UI / CSS / 水墨风格修改完成。
- R3 Level B 执行。
- 真实 Codex 执行、`.codex` 读写或 backlog 功能解冻。

## 9. 实现记录

实现完成时间：2026-06-13。

实现结果：

- `query_workbench_page_read_model` 仍使用既有 Tauri command，不新增 command。
- command 复用 `build_snapshot`，序列化 snapshot / workflow state 后调用 `query_page_read_model_with_snapshot_value`。
- `projects` / `agents` / `running_workflows` / `memory` / `knowledge` / `settings` 六页返回 `status="page_data_ready"`。
- 六页返回 `page_payload.generated_from="workbench_page_query"`。
- 六页 `source_boundary.returns_business_data=true`、`writes_stores=false`、`tauri_command_migrates_page=false`。
- `home` 等非批一页面仍保持 `selector_contract_only`，不返回 payload。
- `types.rs` 最小瘦身：`WorkbenchSnapshot` struct 移到 `workbench_snapshot_types.rs` 并由 `types.rs` include。

形状结果：

- `types.rs`：5386 -> 5364，下降 22 行。
- `workbench_snapshot_types.rs`：新增 23 行。
- `page_read_model.rs`：1335 行，低于 3,000 行。
- `commands.rs`：1293 行，不在 ratchet，未新增 command。
- Tauri command baseline：97 total，0 in `lib.rs`。

验证已通过：

- `cargo test --lib page_read_model`：7 passed。
- `cargo test --lib`：475 passed，16 ignored。
- `cargo fmt -- --check`：通过。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：14 passed；R4 page tests 通过。
- `npm run build`：通过，仅既有 Vite chunk size warning。
- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors，0 warnings；`types.rs: 5364/5386 (decreased)`，Tauri commands `97 total; 0 in lib.rs`。
- `git diff --check`：通过。

复核结论：

- 独立复核线 Volta 返回 `STATUS: CLEAR`。
- P0/P1/P2 均无。
- 复核确认六页 payload、非批一页诚实状态、command baseline、`WorkbenchSnapshot` struct move、验证记录和边界声明均可接受。
