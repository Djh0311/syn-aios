# Evidence: Root Treatment / R4-H2-1 Page Read Model Schema And Snapshot Field Coverage v1

日期：2026-06-13

状态：已完成。

## 1. 范围

本轮执行：

- `tasks/2026-06-13-root-treatment-r4-h2-1-page-read-model-schema-and-snapshot-field-coverage-v1.md`

依据：

- `CURRENT.md` 当前结论。
- `decisions/2026-06-13-root-treatment-r2-late-stage-closure-track-v1.md` §5。
- `handoffs/2026-06-13-r4-hard-targets-execution-plan-supervisor-v1.md`。

本轮只做 H2-1：

- 定义六页 read model schema。
- 对 `WorkbenchSnapshot` 顶层字段做覆盖核对。
- 降低 `types.ts` 棘轮。

本轮不做：

- H2-2 后端按页业务 payload。
- H2-3 前端消费切流。
- H2-4 `types.rs` 瘦身。
- 批二 View 拆分。

## 2. 实现内容

修改：

- `prototypes/productized-desktop-shell/src-tauri/src/page_read_model.rs`
- `prototypes/productized-desktop-shell/src/lib/pageReadModel.ts`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/types/workbenchSnapshot.ts`
- `prototypes/productized-desktop-shell/tests/r4-page-read-model-query-contract.test.ts`
- `tasks/2026-06-13-root-treatment-r4-h2-1-page-read-model-schema-and-snapshot-field-coverage-v1.md`

新增 schema catalog：

- `projects` -> `ProjectsPageReadModel`
- `agents` -> `AgentsPageReadModel`
- `running_workflows` -> `RunningWorkflowsPageReadModel`
- `memory` -> `MemoryCenterPageReadModel`
- `knowledge` -> `KnowledgeBasePageReadModel`
- `settings` -> `SettingsPageReadModel`

query 边界：

- `query_page_read_model` 现在返回 `target_schema`、`snapshot_field_coverage`、`uncovered_snapshot_fields`。
- `status` 仍为 `selector_contract_only`。
- `source_boundary.returns_business_data=false`。
- `source_boundary.writes_stores=false`。
- `source_boundary.tauri_command_migrates_page=false`。
- warnings 包含 `h2_1_schema_defined_no_query_migration` 和 `do_not_claim_workbench_snapshot_deprecated`。

## 3. WorkbenchSnapshot 字段覆盖

当前 `WorkbenchSnapshot` 顶层字段共 20 个，coverage matrix 已覆盖：

```text
summary
projects
sessions
skills
plugins
tasks
agent_adapters
session_operations
provider_availability
session_continuation_previews
session_continuation_store
runtime_session_attention
session_run_status_summaries
runtime_log_store
worker_protocol
real_execution_product_commands
project_workflow_automation
page_read_model_inventory
diagnostic_summary
diagnostics
```

验证断言：

- `catalog.snapshot_field_coverage.len() == 20`
- `catalog.uncovered_snapshot_fields.is_empty()`
- 每个字段至少有一个 `covered_by_pages`

## 4. 形状结果

```text
43  prototypes/productized-desktop-shell/src/lib/types.ts
49  prototypes/productized-desktop-shell/src/lib/types/workbenchSnapshot.ts
737 prototypes/productized-desktop-shell/src-tauri/src/page_read_model.rs
91  prototypes/productized-desktop-shell/src/lib/pageReadModel.ts
```

对比：

- `src/lib/types.ts`：93 -> 43，下降 50 行。
- 新增 `types/workbenchSnapshot.ts`：49 行，低于 2,000 行。
- `page_read_model.rs`：262 -> 737，低于 Rust 3,000 行模块上限；该文件不在 ratchet 水位。

shape gate：

```text
Status: pass
Errors: 0
Warnings: 0
types.ts: 43/4998 (decreased)
Tauri commands: 97 total; 0 in lib.rs
```

## 5. 验证

已通过：

```text
cargo test --lib page_read_model
```

结果：5 passed。

```text
cargo test --lib
```

结果：473 passed，16 ignored。保留既有 `JsonRpcError::invalid_params` dead code warning。

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

结果：`offline interaction tests passed: 14`，R4 page read model / selectors tests 通过。

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

schema / 边界关键词：

```text
rg -n "uncovered_snapshot_fields|returns_business_data|workbench_snapshot_active|schema_defined_no_query_migration" prototypes/productized-desktop-shell/src-tauri/src/page_read_model.rs prototypes/productized-desktop-shell/src/lib/pageReadModel.ts
```

结果：仅 schema / source boundary / tests 命中。

import 兼容入口：

```text
rg -n "from \"\\.\\/lib\\/types\"|from \"\\.\\/types\"" prototypes/productized-desktop-shell/src
```

结果：现有 `./lib/types` / `./types` import 仍存在，`types.ts` re-export 兼容入口保留；`npm run typecheck` 已通过。

snapshot / page query 调用：

```text
rg -n "loadWorkbenchSnapshot\\(|load_workbench_snapshot|queryWorkbenchPageReadModel\\(|query_workbench_page_read_model" prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/src-tauri/src
```

结果：`App.tsx` 仍通过 `loadWorkbenchSnapshot()` 取整包；`queryWorkbenchPageReadModel` 仅 wrapper 存在。本轮未执行 H2-3 前端切流。

## 7. 边界确认

本轮没有：

- 新增 Tauri command。
- 修改 `load_workbench_snapshot` 行为。
- 实现后端按页业务 payload。
- 让前端页面改为消费按页 query。
- 拆 `ProjectsView` / `AgentView` 或任何 View。
- 修改 UI、CSS、水墨风格、布局、文案或交互。
- 修改 DB / sidecar schema / workflow state schema。
- 执行真实 `codex exec` / `codex exec resume`。
- 发送 prompt。
- 读写 `/Users/yoyi/.codex`。
- 启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 触碰 R3 Level B、解冻 backlog 或进入批二。

## 8. 不接受为

本轮不接受为：

- H2-2 后端按页 query 完成。
- H2-3 前端已切到按页取数。
- `WorkbenchSnapshot` 已废弃。
- 批一完成。
- 批二开始。
- UI / CSS / 水墨风格修改完成。
- R3 Level B 执行。
- 真实 Codex 执行、`.codex` 读写或 backlog 功能解冻。

## 9. 独立复核

复核线：Tesla。

结论：`STATUS: CLEAR`。

P0/P1/P2：无。

复核覆盖：

- H2-1 scope 清晰，相关改动集中在指定代码 / 测试 / 任务包 / evidence / handoff。
- 六页 schema 已定义。
- `WorkbenchSnapshot` 20 个顶层字段 coverage 完整，`uncovered_snapshot_fields=[]`。
- `query_page_read_model` 仍保持 `selector_contract_only`、`returns_business_data=false`、`writes_stores=false`、`tauri_command_migrates_page=false`。
- `types.ts` 为 43 行，并通过 `export * from "./types/workbenchSnapshot"` 保持旧 import 兼容。
- 未见 UI/CSS/View/DB/sidecar/workflow state schema 改动。
- 未执行 H2-2/H2-3/批二。
