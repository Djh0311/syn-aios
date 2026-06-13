# Handoff: Root Treatment / R4-H2 Batch One Page Query Completion Checklist v1

日期：2026-06-13

状态：批一已收口，停止在批二前。

## 1. 批一完成结论

R4-H2 批一“WorkbenchSnapshot 按页查询”已完成：

- H2-1 定义六个目标页 schema，并覆盖 `WorkbenchSnapshot` 20 个顶层字段。
- H2-2 让既有 `query_workbench_page_read_model` 返回六页后端业务 payload，不新增 Tauri command。
- H2-3 让前端 `App.tsx` 从六页 page query 合成只读数据，不再直接调用 `loadWorkbenchSnapshot()`。
- H2-4 最小伴随项已并入 H2-2 / H2-3：`types.rs` 从 H2-1 waterline 5386 降到 5229。

批一到此停止。不得进入批二 `ProjectsView` / `AgentView` 拆分；批二必须等主管线输出布局设计稿并经用户确认后另行授权。

## 2. Commits

- H2-1：`6c4a050` `feat: define page read model schema coverage`
- H2-2：`a5cd7b3` `feat: return backend page query payloads`
- H2-3：`a8e40eb` `feat: consume workbench page query payloads`

## 3. 记录文件

H2-1：

- `tasks/2026-06-13-root-treatment-r4-h2-1-page-read-model-schema-and-snapshot-field-coverage-v1.md`
- `evidence/2026-06-13-root-treatment-r4-h2-1-page-read-model-schema-and-snapshot-field-coverage-v1.md`
- `handoffs/2026-06-13-root-treatment-r4-h2-1-page-read-model-schema-and-snapshot-field-coverage-v1-result.md`

H2-2：

- `tasks/2026-06-13-root-treatment-r4-h2-2-backend-page-query-business-payload-v1.md`
- `evidence/2026-06-13-root-treatment-r4-h2-2-backend-page-query-business-payload-v1.md`
- `handoffs/2026-06-13-root-treatment-r4-h2-2-backend-page-query-business-payload-v1-result.md`

H2-3：

- `tasks/2026-06-13-root-treatment-r4-h2-3-frontend-page-query-consumption-and-snapshot-ratchet-slimming-v1.md`
- `evidence/2026-06-13-root-treatment-r4-h2-3-frontend-page-query-consumption-and-snapshot-ratchet-slimming-v1.md`
- `handoffs/2026-06-13-root-treatment-r4-h2-3-frontend-page-query-consumption-and-snapshot-ratchet-slimming-v1-result.md`

## 4. 完成判据核对

- 六页读模型独立可查：`projects`、`agents`、`running_workflows`、`memory`、`knowledge`、`settings` 均返回 `page_data_ready`。
- 前端全部切到按页取：`App.tsx` 使用 `loadWorkbenchSnapshotFromPageQueries(queryWorkbenchPageReadModel)`。
- 前端不再直接消费整包 snapshot：`loadWorkbenchSnapshot()` 只剩 `src/lib/tauri.ts` wrapper，App 不调用。
- 相关离线测试全绿：H2-3 验证记录显示 `npm run test:offline-interaction` 14 passed。
- 棘轮下降：`types.rs` 为 5229/5386，shape gate pass。

## 5. 验证汇总

H2-3 最终验证已通过：

- `cargo test --lib page_read_model`：7 passed。
- `cargo test --lib`：475 passed，16 ignored。
- `cargo fmt -- --check`：通过。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：14 passed。
- `npm run build`：通过，仅既有 Vite chunk size warning。
- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors，0 warnings。
- `git diff --check`：通过。

## 6. 独立复核

- H2-1 复核线 Tesla：`STATUS: CLEAR`。
- H2-2 复核线 Volta：`STATUS: CLEAR`。
- H2-3 复核线 Faraday：`STATUS: CLEAR`。

H2-3 保留 P2：runtime helper warning 测试未逐分支覆盖 non-ready / unexpected source / missing slice 三个分支；已有完整六页无 warning、缺页 warning 和 fallback 不冒充完整数据测试。本项不阻断批一收口。

## 7. 边界确认

批一未执行：

- 真实 `codex exec` / `codex exec resume`。
- prompt 发送。
- `/Users/yoyi/.codex` 读写。
- Tauri / Browser / Chrome / Vite dev / screenshot。
- R3 Level B。
- DB / sidecar schema / workflow state schema 修改。
- UI / CSS / 水墨风格 / 布局 / 文案 / 交互修改。
- `ProjectsView` / `AgentView` 拆分。
- backlog 解冻。

## 8. 当前外部脏文件

以下文件仍为外部脏文件或外部未跟踪文件，未纳入 H2-3 commit：

- `AGENTS.md`
- `backlog.md`
- `docs/own-agent-and-company-vision-v1.md`
- `handoffs/2026-06-13-r4-hard-targets-execution-plan-supervisor-v1.md`

## 9. 下一步停止线

当前必须停在批二前。

下一步不是继续拆 View，也不是 UI 重做；下一步应由主管线准备批二设计输入：

- ProjectsView / AgentView 的“布局区块 -> 组件边界”设计稿。
- 用户确认后的批二任务包。
- 批二开始前重新声明边界：不猜布局、不顺手改风格、不把 page query 完成冒充 View 拆分完成。
