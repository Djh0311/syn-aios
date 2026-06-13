# Evidence: Root Treatment / R4-H2-2 Backend Page Query Business Payload v1

日期：2026-06-13

状态：已完成。

## 1. 范围

本轮执行：

- `tasks/2026-06-13-root-treatment-r4-h2-2-backend-page-query-business-payload-v1.md`

本轮只做 H2-2：

- 让现有 `query_workbench_page_read_model` 返回六个目标页的后端业务 payload。
- 不新增 Tauri command。
- 不切前端消费。
- 不改 UI / CSS / View。

伴随最小 H2-4：

- 移动 Rust `WorkbenchSnapshot` struct 到 `workbench_snapshot_types.rs`，降低 `types.rs` 棘轮。

## 2. 实现内容

修改：

- `prototypes/productized-desktop-shell/src-tauri/src/page_read_model.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src/lib/pageReadModel.ts`
- `prototypes/productized-desktop-shell/tests/r4-page-read-model-query-contract.test.ts`
- `tasks/2026-06-13-root-treatment-r4-h2-2-backend-page-query-business-payload-v1.md`

新增：

- `prototypes/productized-desktop-shell/src-tauri/src/workbench_snapshot_types.rs`

query 行为：

- `query_workbench_page_read_model` 复用 `build_snapshot`。
- snapshot / workflow state 序列化为 JSON 后进入 `query_page_read_model_with_snapshot_value`。
- 六个目标页返回 `page_data_ready` 和 `page_payload`。
- 非批一页面仍返回 contract-only。

六页 payload：

- `projects`
- `agents`
- `running_workflows`
- `memory`
- `knowledge`
- `settings`

## 3. Query Boundary

目标页：

```text
status = "page_data_ready"
source_boundary.returns_business_data = true
source_boundary.writes_stores = false
source_boundary.tauri_command_migrates_page = false
page_payload.generated_from = "workbench_page_query"
```

非批一页面：

```text
status = "selector_contract_only"
source_boundary.returns_business_data = false
page_payload = null
```

本轮仍不声明前端已切流：`selector_plan.ui_consumption_status` 仍为 `not_connected_to_pages`。

## 4. 形状结果

```text
5364 prototypes/productized-desktop-shell/src-tauri/src/types.rs
23   prototypes/productized-desktop-shell/src-tauri/src/workbench_snapshot_types.rs
1335 prototypes/productized-desktop-shell/src-tauri/src/page_read_model.rs
1293 prototypes/productized-desktop-shell/src-tauri/src/commands.rs
100  prototypes/productized-desktop-shell/src/lib/pageReadModel.ts
```

对比：

- `types.rs`：5386 -> 5364，下降 22 行。
- `workbench_snapshot_types.rs`：新增 23 行。
- `page_read_model.rs`：1335 行，低于 3,000 行。
- `commands.rs`：未新增 command。

shape gate：

```text
Status: pass
Errors: 0
Warnings: 0
types.rs: 5364/5386 (decreased)
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

结果：475 passed，16 ignored。保留既有 `JsonRpcError::invalid_params` dead code warning。

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

payload / query 关键词：

```text
rg -n "page_data_ready|workbench_page_query|returns_business_data|query_workbench_page_read_model" prototypes/productized-desktop-shell/src-tauri/src prototypes/productized-desktop-shell/src/lib/pageReadModel.ts
```

结果：命中 query command、page read model、TS type 和测试；未见 UI 消费切流。

command baseline：

```text
node scripts/harness/workbench-shape-gate.js --mode check
```

结果：Tauri commands 仍为 97 total，0 in `lib.rs`。

## 7. 边界确认

本轮没有：

- 新增 Tauri command。
- 修改 `load_workbench_snapshot` 返回结构或语义。
- 让前端页面消费按页 query。
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

- H2-3 前端已切到按页取数。
- `WorkbenchSnapshot` 已废弃。
- 批一完成。
- 批二开始。
- UI / CSS / 水墨风格修改完成。
- R3 Level B 执行。
- 真实 Codex 执行、`.codex` 读写或 backlog 功能解冻。

## 9. 独立复核

复核线：Volta。

结论：`STATUS: CLEAR`。

P0/P1/P2：无。

复核覆盖：

- H2-2 scope 清晰，相关改动集中在指定代码 / 任务包 / evidence / handoff。
- 六页返回 `page_data_ready` 和 `workbench_page_query` payload。
- 非批一页面仍保持 `selector_contract_only`、`returns_business_data=false`、无 payload。
- command baseline 仍为 97 total，0 in `lib.rs`。
- `WorkbenchSnapshot` struct move 语义等价。
- 未见 UI/CSS/View/DB/sidecar/workflow state schema 改动。
- H2-3 / 批二未执行。
