# Root Treatment R4-A2 Page Query Selector Contract Skeleton Handoff v1

日期：2026-06-11

结论：R4-A2 已完成，状态为 `accepted_selector_contract_only`。

## 做了什么

- 复核线回收 R4-A1：`CLEAR_WITH_P2`，无 P0/P1；P2 是 R4-A1 checkpoint/hash 回填不完整。
- 主管线已补齐 R4-A1 task/evidence/handoff 的 `6519ad3` / `03fd247` 元数据，并提交准备 commit。
- 新增只读 page query selector contract skeleton：`query_workbench_page_read_model`。
- 后端 `page_read_model.rs` 新增 query input/result、selector plan、source boundary 和纯函数。
- 前端 `pageReadModel.ts` / `tauri.ts` 新增类型和 `queryWorkbenchPageReadModel` wrapper。
- 新增小测试 `r4-page-read-model-query-contract.test.ts`，并纳入 offline runner。

Task package preparation commit：`e727e3dc928bd658926944d11cb83ee8c602e4af`
Implementation commit：`bcc59c53ab871401aac17d1cc79ba2c84a7cd5b2`

## 关键文件

- `prototypes/productized-desktop-shell/src-tauri/src/page_read_model.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs`
- `prototypes/productized-desktop-shell/src/lib/pageReadModel.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs`
- `prototypes/productized-desktop-shell/tests/r4-page-read-model-query-contract.test.ts`

## 验证

- `node scripts/harness/workbench-shape-gate.js --mode check`：通过，0 errors / 1 warning。warning 为 Tauri command total increased，已分类为 R4-A2 允许的只读 command skeleton。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 14`，并输出 R4-A1 / R4-A2 小测试通过。
- `npm run build`：通过，仅既有 Vite chunk-size warning。
- `cargo test --lib page_read_model`：通过，3 passed。
- `cargo test --lib`：通过，471 passed / 16 ignored。
- `cargo fmt -- --check`：通过。
- `git diff --check`：通过。

## 边界

- `query_workbench_page_read_model` 只返回合同、selector plan 和 source boundary，不返回完整页面业务数据。
- `queryWorkbenchPageReadModel` 目前没有被任何 UI 页面调用。
- `WorkbenchSnapshot` / `load_workbench_snapshot` 仍是当前页面主数据来源。
- 没有新增 sidecar、DB migration、production read-cut、stop-write 或真实 Codex runner 改动。
- 未执行真实 Codex，未读写 `/Users/yoyi/.codex`，未读取 secret/token/credential/full transcript。
- 未改视觉风格或布局，未启动真实 Tauri / 截图验收。

## 不能声明

- 不能声明 R4 完成。
- 不能声明所有页面已按页查询。
- 不能声明任一真实页面已经迁移到 `query_workbench_page_read_model`。
- 不能声明 `WorkbenchSnapshot` 已废弃。
- 不能声明 `ProjectsView` / `AgentView` 已拆分完成。
- 不能声明 UI 已重做或视觉已验收。
- 不能声明 R3 Level B 已执行。
- 不能声明多 agent 并行真实执行已解锁。

## 下一步建议

建议进入 R4-A3：Projects / Agents 首批 selector 分域，仍不改视觉风格。目标是先让两个高风险页面在前端内部使用 typed selector functions 从现有 `WorkbenchSnapshot` 切片，降低大页面直接读巨型 snapshot 的耦合；不要在下一步就全面迁移所有页面。
