# Root Treatment / R4-H2-1 Page Read Model Schema And Snapshot Field Coverage v1

日期：2026-06-13

状态：已完成。

性质：R4 硬目标 / 批一 snapshot 按页查询的第 1 包。本包只定义六个目标页的读模型 schema 和 `WorkbenchSnapshot` 字段覆盖核对，为 H2-2 后端按页 query 做契约准备。

Planning baseline：`2f0c365b16cd5caa07044ed327933c8cb1559441`。

## 0. 全局主管理解

批一目标是把整包 `WorkbenchSnapshot` 读模型拆成按页查询：项目页 / 智能体页 / 运行中页 / 记忆页 / 知识库页 / 设置页。当前状态：

- `load_workbench_snapshot` 仍返回完整 `WorkbenchSnapshot`。
- `query_workbench_page_read_model` 已存在，但仍是 selector contract skeleton。
- `page_read_model.rs` 有 R4-A1 inventory/query 骨架。
- 前端 `pageSelectors.ts` 已有六页 selector 类型和纯函数，但仍从整包 `WorkbenchSnapshot` 派生。
- 旧 `tasks/2026-06-13-root-treatment-r4-h2-workbench-snapshot-page-query-first-slice-v1.md` 是 home/projects 第一切片草案，不按本轮新批一拆法执行。

本包不做 H2-2 的真实 page query payload，不做 H2-3 前端切流，只把后续 query 必须返回的六页 schema 和字段覆盖矩阵冻结下来。

## 1. 目标

新增或更新只读契约：

- 项目页 schema：覆盖 `summary`、`projects`、`sessions`、`tasks`、`project_workflow_automation` 等页面所需字段。
- 智能体页 schema：覆盖 `projects`、`sessions`、adapter、operation、provider、continuation、worker protocol 等会话相关字段。
- 运行中页 schema：覆盖 runtime attention、run status、runtime log、real execution product command、automation、diagnostic 等运行相关字段。
- 记忆页 schema：覆盖页面所需的项目 / 任务包上下文，并明确正式记忆、候选、观察、lint 等来自独立 store，不从 `WorkbenchSnapshot` 冒充。
- 知识库页 schema：覆盖项目 / 任务引用上下文，并明确资料、引用、候选入口来自知识库 / 记忆 store，不从 snapshot 冒充。
- 设置页 schema：覆盖 summary、skills、plugins、diagnostics、page inventory、adapter/provider 数量、runtime log 健康等开发者信息。

字段覆盖要求：

- 对 `WorkbenchSnapshot` 当前全部顶层字段做 coverage matrix。
- `uncovered_snapshot_fields` 必须为空。
- 不把 `WorkbenchSnapshot` 说成已废弃。
- 不把 schema 定义说成页面已经切换数据源。

## 2. 形状影响

预期：

- `src/lib/types.ts` 继续下降：从 93 行下降到约 70 行以内，降低至少 15 行。
- 新增 `src/lib/types/workbenchSnapshot.ts` 低于 2,000 行，仅承载 `WorkbenchSnapshot` 聚合类型。
- `page_read_model.rs` 增加 schema / coverage 定义和单测，但该文件保持低于 Rust 3,000 行，不进入 ratchet 水位。
- 不新增 Tauri command；command baseline 不变。
- 不修改 `types.rs`、`lib.rs`、`commands.rs` 行为。

如果 `types.ts` 未下降，本包不得以“单独准备工作”收口，必须并入能降低棘轮指标的后续包。

## 3. 允许范围

允许修改：

- `prototypes/productized-desktop-shell/src-tauri/src/page_read_model.rs`
- `prototypes/productized-desktop-shell/src/lib/pageReadModel.ts`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/types/workbenchSnapshot.ts`
- 相关离线测试 / fixture，限契约断言。
- 当前任务包、evidence、handoff。

允许新增：

- `prototypes/productized-desktop-shell/src/lib/types/workbenchSnapshot.ts`

## 4. 禁止范围

禁止：

- 新增 Tauri command。
- 修改 `load_workbench_snapshot` 行为。
- 实现 H2-2 后端按页业务 payload。
- 实现 H2-3 前端消费切流。
- 拆 `ProjectsView` / `AgentView` 或任何 View 组件。
- 修改 UI、CSS、水墨风格、布局、文案或交互。
- 修改 DB / sidecar schema / workflow state schema。
- 执行真实 `codex exec` / `codex exec resume`，发送 prompt，读写 `/Users/yoyi/.codex`。
- 启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 触碰 R3 Level B、解冻 backlog 或进入批二。

## 5. 实现步骤

1. 在 `page_read_model.rs` 增加六页 `PageReadModelSchemaContract` 和 `WorkbenchPageReadModelSchemaCatalog`。
2. 增加 `PageSnapshotFieldCoverage`，列出当前 `WorkbenchSnapshot` 全部顶层字段的归属页。
3. `query_page_read_model` 返回对应页 schema / coverage，但保持 `returns_business_data=false`。
4. TS `pageReadModel.ts` 同步 schema / coverage 类型。
5. 把 `WorkbenchSnapshot` 聚合类型从 `types.ts` 移到 `types/workbenchSnapshot.ts`，`types.ts` 继续 re-export，保持现有 import 兼容。
6. 补 Rust / TS 契约测试，确保六页 schema 存在、字段覆盖无遗漏、query 不冒充业务数据。

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

- `wc -l prototypes/productized-desktop-shell/src/lib/types.ts prototypes/productized-desktop-shell/src/lib/types/workbenchSnapshot.ts prototypes/productized-desktop-shell/src-tauri/src/page_read_model.rs`
- `rg -n "uncovered_snapshot_fields|returns_business_data|workbench_snapshot_active|schema_defined_no_query_migration" prototypes/productized-desktop-shell/src-tauri/src/page_read_model.rs prototypes/productized-desktop-shell/src/lib/pageReadModel.ts`
- `rg -n "from \"\\.\\/lib\\/types\"|from \"\\.\\/types\"" prototypes/productized-desktop-shell/src`

## 7. 复核要求

复核线重点检查：

- 六个目标页是否都有 schema。
- 当前 `WorkbenchSnapshot` 顶层字段是否全部被 coverage matrix 覆盖。
- `query_page_read_model` 是否仍诚实标注 `returns_business_data=false`，没有提前执行 H2-2。
- `types.ts` 是否继续下降且 re-export 兼容。
- 是否新增 command、误改 UI、误改真实执行路径、误碰 `.codex` 或进入批二。

## 8. 不接受为

本包不接受为：

- H2-2 后端按页 query 已完成。
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

- `page_read_model.rs` 新增六个目标页 schema catalog：`projects`、`agents`、`running_workflows`、`memory`、`knowledge`、`settings`。
- 新增 `PageSnapshotFieldCoverage`，覆盖当前 `WorkbenchSnapshot` 20 个顶层字段，`uncovered_snapshot_fields=[]`。
- `query_page_read_model` 现在附带 `target_schema` / coverage metadata，但仍保持 `status="selector_contract_only"`、`returns_business_data=false`。
- `pageReadModel.ts` 同步 schema / coverage 类型。
- `WorkbenchSnapshot` TS 聚合类型迁移到 `types/workbenchSnapshot.ts`，`types.ts` 继续 re-export 兼容旧 import。

形状结果：

- `src/lib/types.ts`：93 -> 43 行，下降 50 行。
- 新增 `src/lib/types/workbenchSnapshot.ts`：49 行。
- `src-tauri/src/page_read_model.rs`：262 -> 737 行，仍低于 Rust 3,000 行新文件/模块上限，且不在 ratchet 水位。
- Tauri command 总数未变：97 total，0 in `lib.rs`。

验证已通过：

- `cargo test --lib page_read_model`：5 passed。
- `cargo test --lib`：473 passed，16 ignored。
- `cargo fmt -- --check`：通过。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：14 passed；R4 page tests 通过。
- `npm run build`：通过，仅既有 Vite chunk size warning。
- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors，0 warnings；`types.ts: 43/4998 (decreased)`。
- `git diff --check`：通过。
- schema / import / query 边界扫描已通过，未发现前端切流或新增 command。

复核结论：

- 独立复核线 Tesla 返回 `STATUS: CLEAR`。
- P0/P1/P2 均无。
- 复核确认六页 schema、20 字段 coverage、query honesty、types re-export 兼容、验证记录和边界声明均可接受。
