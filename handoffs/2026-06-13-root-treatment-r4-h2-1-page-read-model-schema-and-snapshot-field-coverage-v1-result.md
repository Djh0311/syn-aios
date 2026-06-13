# Handoff: Root Treatment / R4-H2-1 Page Read Model Schema And Snapshot Field Coverage v1

日期：2026-06-13

状态：已完成，并通过独立复核。

## 1. 完成内容

H2-1 已完成实现和验证：

- 六个目标页 schema catalog 已落在 `page_read_model.rs`。
- `WorkbenchSnapshot` 20 个顶层字段 coverage matrix 已覆盖，无遗漏字段。
- `query_page_read_model` 返回 schema / coverage metadata，但仍不返回业务 page payload。
- TS `pageReadModel.ts` 同步 schema / coverage 类型。
- `WorkbenchSnapshot` TS 聚合类型从 `types.ts` 迁移到 `types/workbenchSnapshot.ts`，`types.ts` 继续 re-export。

## 2. 文件

修改：

- `prototypes/productized-desktop-shell/src-tauri/src/page_read_model.rs`
- `prototypes/productized-desktop-shell/src/lib/pageReadModel.ts`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/tests/r4-page-read-model-query-contract.test.ts`
- `tasks/2026-06-13-root-treatment-r4-h2-1-page-read-model-schema-and-snapshot-field-coverage-v1.md`

新增：

- `prototypes/productized-desktop-shell/src/lib/types/workbenchSnapshot.ts`
- `evidence/2026-06-13-root-treatment-r4-h2-1-page-read-model-schema-and-snapshot-field-coverage-v1.md`
- `handoffs/2026-06-13-root-treatment-r4-h2-1-page-read-model-schema-and-snapshot-field-coverage-v1-result.md`

## 3. 关键结果

六页 schema：

- `projects`
- `agents`
- `running_workflows`
- `memory`
- `knowledge`
- `settings`

字段覆盖：

- 当前 `WorkbenchSnapshot` 顶层字段 20 个。
- `snapshot_field_coverage.len() == 20`。
- `uncovered_snapshot_fields=[]`。

形状：

- `types.ts`：93 -> 43，下降 50 行。
- `types/workbenchSnapshot.ts`：49 行。
- `page_read_model.rs`：737 行，低于 3,000 行。
- Tauri commands：仍为 97 total，0 in `lib.rs`。

## 4. 验证

已通过：

- `cargo test --lib page_read_model`：5 passed。
- `cargo test --lib`：473 passed，16 ignored。
- `cargo fmt -- --check`：通过。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：14 passed。
- `npm run build`：通过，仅既有 Vite chunk size warning。
- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors，0 warnings。
- `git diff --check`：通过。

## 5. 边界确认

本轮未新增 Tauri command，未修改 `load_workbench_snapshot` 行为，未实现业务 page payload，未让前端切到按页 query，未拆 View，未改 UI/CSS/布局/文案/交互，未改 DB/sidecar/workflow state schema，未执行真实 Codex，未发送 prompt，未读写 `/Users/yoyi/.codex`，未启动 Tauri/Browser/Chrome/Vite dev/screenshot，未进入 R3 Level B 或批二。

## 6. 复核输入

建议复核线重点看：

- `page_read_model.rs` 的六页 schema 是否覆盖目标页且没有冒领业务 payload。
- coverage matrix 是否覆盖 `WorkbenchSnapshot` 当前 20 个顶层字段。
- `types.ts` re-export 是否保持旧 import 兼容。
- 是否有 UI / command / schema / 真实执行越界。
- 任务包、evidence、handoff 是否与实现事实一致。

## 7. 不接受为

本轮不接受为 H2-2 后端按页 query 完成、H2-3 前端切流完成、`WorkbenchSnapshot` 废弃、批一完成、批二开始、R3 Level B 执行、真实 Codex 执行、`.codex` 读写或 backlog 功能解冻。

## 8. 复核结论

独立复核线 Tesla 返回 `STATUS: CLEAR`。

- P0/P1/P2：无。
- 复核确认六页 schema、20 字段 coverage、query honesty、types compatibility、shape / validation records、boundary claims 均可接受。
- 复核确认 H2-2 / H2-3 / 批二未执行。
