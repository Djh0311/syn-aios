# Root Treatment / R4-H2 Workbench Snapshot Page Query First Slice v1

日期：2026-06-13

状态：已创建，待用户最终车道裁决后执行。

性质：R4 硬目标首批任务包草案。本文只准备任务包，不代表已经执行。

Planning baseline：待执行时以当时 `HEAD` 回填。

## 0. 全局主管理解

当前状态：

- `load_workbench_snapshot` 仍返回完整 `WorkbenchSnapshot`。
- `query_workbench_page_read_model` 已存在，但当前只是 selector contract skeleton。
- 当前 `page_read_model.rs` 对已知页面返回 `status="selector_contract_only"`、`returns_business_data=false`、`workbench_snapshot_active=true`。
- 前端已有 `pageSelectors.ts`，但 selectors 仍从完整 `WorkbenchSnapshot` 派生。

本包目标不是切 UI，也不是废弃完整 snapshot，而是让按页查询先返回第一批真实业务读模型数据，建立从“contract only”到“page query data”的最小可验证桥。

## 1. 目标

实现第一批后端按页查询业务数据：

- `home`：首页摘要，包含项目数、会话数、运行中/待处理摘要、索引生成时间、warning count。
- `projects`：项目列表摘要，包含项目 root/name、会话数量、workflow summary count、warning count、最近更新时间近似值。

本包完成后：

- `query_workbench_page_read_model({ page_id: "home" })` 和 `{ page_id: "projects" }` 返回 `returns_business_data=true`。
- 其他页面仍可保持 `selector_contract_only`，不得冒充已迁移。
- 前端页面可以暂不消费该 query；UI 切换另拆后续任务包。

## 2. 形状影响

预期：

- 不增长任何 ratchet 文件水位；若必须触碰 `lib.rs`，净效果必须不增长。
- `page_read_model.rs` 可以增加实现，但单文件必须低于 Rust 3,000 行。
- TS 类型应优先写入 `pageReadModel.ts` 或 R4-H1 拆出的类型域，不得让 `types.ts` 回涨。
- 不新增 Tauri command；复用现有 `query_workbench_page_read_model`。

## 3. 允许范围

允许修改：

- `prototypes/productized-desktop-shell/src-tauri/src/page_read_model.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs` 仅限把已有 query command 接入必要输入，不新增 command。
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs` 仅限在不增长水位的前提下抽出或复用读取 helper；如会增长，必须暂停并改任务包。
- `prototypes/productized-desktop-shell/src/lib/pageReadModel.ts`
- 必要的离线测试文件。
- 当前任务包、evidence、handoff。

允许新增：

- `prototypes/productized-desktop-shell/src-tauri/src/page_read_model_tests.rs` 或同等测试文件。
- 前端 page read model 类型文件，前提是低于 2,000 行。

## 4. 禁止范围

禁止：

- 删除或破坏 `load_workbench_snapshot`。
- 把所有页面一次性切换到按页查询。
- 修改 UI 视觉、布局、文案或交互。
- 修改 DB/schema/sidecar schema/workflow state schema。
- 新增 Tauri command。
- 读取真实 `.codex` 或发送 prompt。
- 执行真实 `codex exec` / `codex exec resume`。
- 启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 把 `home/projects` 之外页面标记为 `returns_business_data=true`。

## 5. 实现步骤

1. 为 `PageReadModelQueryResult` 增加可选 payload 字段，或新增 page-specific result union；字段必须向后兼容。
2. 增加 `HomePageReadModel` 和 `ProjectsPageReadModel` 的 Rust/TS 契约。
3. 在 `query_workbench_page_read_model` 中对 `home` / `projects` 派生业务读模型。
4. 确保 `source_boundary` 精准表达：`returns_business_data=true`，`writes_stores=false`，不声称 UI 已迁移。
5. 为未知 page、未迁移 page、home/projects 成功路径补 Rust 测试。
6. 补前端类型检查和离线断言，确保 result_count / unknown 类口径不被改成 0。

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

- `rg -n "selector_contract_only|returns_business_data|workbench_snapshot_active|query_workbench_page_read_model" prototypes/productized-desktop-shell/src-tauri/src prototypes/productized-desktop-shell/src`
- `rg -n "load_workbench_snapshot\\(" prototypes/productized-desktop-shell/src`

## 7. 复核要求

复核线重点检查：

- `home/projects` 是否真的返回业务数据，不再只是 contract skeleton。
- 其他页面是否仍诚实标注未迁移。
- 是否新增 command 或误改完整 snapshot。
- 是否让 ratchet 文件回涨。
- 是否误触 UI、schema、真实执行或 `.codex`。

## 8. 不接受为

本包不接受为：

- `WorkbenchSnapshot` 已废弃。
- 所有页面按页查询完成。
- 前端页面已切换数据源。
- R4 完成。
- R3 Level B 执行。
- 真实 Codex 执行、`.codex` 读写、UI 重做或 backlog 功能解冻。
