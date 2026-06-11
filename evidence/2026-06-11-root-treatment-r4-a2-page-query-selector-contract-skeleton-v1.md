# Root Treatment R4-A2 Page Query Selector Contract Skeleton Evidence v1

日期：2026-06-11

结论：`accepted_selector_contract_only`

R4-A2 已完成为 page query selector contract skeleton。接受范围是只读 Tauri command、后端 query contract 纯函数、前端 wrapper / 类型和小测试；不接受为任一页面真实数据来源迁移、R4 完成、`WorkbenchSnapshot` 废弃、UI 重做、真实 Tauri / 截图验收、R3 Level B 或多 agent 并行真实执行解锁。

## Review Gate

- R4-A1 复核 STATUS：`CLEAR_WITH_P2`。
- R4-A1 P0/P1：无。
- R4-A1 P2：`6519ad3` / `03fd247` 未在 task/evidence/handoff 全量回填。
- P2 处理：主管线已补齐 R4-A1 task/evidence/handoff 元数据，并提交 `e727e3dc928bd658926944d11cb83ee8c602e4af`。

## Implementation

- Task package：`tasks/2026-06-11-root-treatment-r4-a2-page-query-selector-contract-skeleton-v1.md`
- Task package preparation commit：`e727e3dc928bd658926944d11cb83ee8c602e4af`
- Implementation commit：`bcc59c53ab871401aac17d1cc79ba2c84a7cd5b2`
- Checkpoint commit：`cb9bb80`
- 后端 query contract：`prototypes/productized-desktop-shell/src-tauri/src/page_read_model.rs`
- Tauri command wrapper：`prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- Tauri command registry：`prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs`
- 前端类型：`prototypes/productized-desktop-shell/src/lib/pageReadModel.ts`
- 前端 wrapper：`prototypes/productized-desktop-shell/src/lib/tauri.ts`
- 离线测试：`prototypes/productized-desktop-shell/tests/r4-page-read-model-query-contract.test.ts`

## Contract

`query_workbench_page_read_model` 输入：

- `page_id`

成功输出：

- requested page id / page label。
- matching R4-A1 page contract。
- selector plan：`selector_contract_only`、`not_migrated`、`not_connected_to_pages`。
- source boundary：`workbench_snapshot_active=true`、`returns_business_data=false`、`writes_stores=false`、`tauri_command_migrates_page=false`。
- warnings：`r4_a2_skeleton_no_page_data_query`、`workbench_snapshot_still_active`、`do_not_claim_workbench_snapshot_deprecated`。

拒绝输出：

- 空 `page_id` 返回 `page_id_required`。
- 未知 `page_id` 返回 `unknown_page_id:<page_id>`。

## Shape Gate

`node scripts/harness/workbench-shape-gate.js --mode check` 通过。

重要分类：

- Tauri commands 从 96 增加到 97，shape gate 给出 warning。
- 该 warning 属于 R4-A2 任务包明确允许的只读 command skeleton：`query_workbench_page_read_model`。
- `lib.rs`、`types.rs`、`types.ts`、`offline-permission-dialog.test.tsx` 均未增长。
- 未新增 sidecar JSON kind。

## Verification

- `node scripts/harness/workbench-shape-gate.js --mode check`：通过，0 errors / 1 warning。warning 为 Tauri command total increased，已分类为 R4-A2 允许的只读 skeleton command。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，输出 `offline interaction tests passed: 14`、`r4 page read model settings test passed`、`r4 page read model query contract test passed`。
- `npm run build`：通过，仅保留既有 Vite chunk-size warning。
- `cargo test --lib page_read_model`：通过，3 passed。
- `cargo test --lib`：通过，471 passed / 16 ignored。
- `cargo fmt -- --check`：通过。
- `git diff --check`：通过。

## Scans

- `queryWorkbenchPageReadModel|query_workbench_page_read_model|query_page_read_model`：产品 UI 页面无调用；仅 wrapper / command / pure helper / tests 命中。
- 误导文案扫描：命中仅在 R4-A2 task 禁止声明 / 扫描条款中，不是正向冒领。
- 敏感 / 真实执行扫描：命中仅在 R4-A2 task 禁止条款中；R4-A2 产品代码未新增 `codex exec`、`.codex`、secret/token/credential/full transcript 读取路径。

## Boundary

- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript。
- 未新增 sidecar JSON。
- 未迁移页面真实数据来源。
- 未废弃 `WorkbenchSnapshot` 或 `load_workbench_snapshot`。
- 未改 UI 视觉风格 / 布局。
- 未启动 Tauri / Browser / Chrome / Vite dev / 截图工具。

## Deferred

- 下一步仍需决定先做 Projects / Agents 首批 selector 分域，还是先做 TS 类型分域。
- 至少 4 个主页面使用按页查询的 R4 目标尚未完成。
- `WorkbenchSnapshot` 仍是当前页面主数据来源。
- R3 Level B 仍未执行。
- Stage L / L1-L6 继续 `deferred_during_root_treatment`。
