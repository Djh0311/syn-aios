# Root Treatment / R4-A3 Projects Agents Selector Domain Split v1

日期：2026-06-11

状态：执行中。本文是 Root Treatment / Stage R 的 R4-A3 任务包；R4-A2 已完成并通过复核线 `STATUS: CLEAR`。R4-A3 只接受为 Projects / Agents 首批前端 selector 分域完成，不接受为页面真实数据来源迁移、R4 完成、`WorkbenchSnapshot` 废弃、UI 重做或真实执行解锁。

规划基线 commit：`882b079b42b7f152f38820b40460ef3c652fad7c`

## 0. 全局主管理解

已知事实：

- R4-A1 已冻结 page read model inventory。
- R4-A2 已建立 `query_workbench_page_read_model` skeleton、后端纯 query contract、前端 wrapper 和小测试。
- 当前 `ProjectsView.tsx` / `AgentView.tsx` 仍由 `App.tsx` 传入整包 `WorkbenchSnapshot` 派生字段。
- R4 目标是读模型和前端瘦身，不是 UI 视觉重做。

核心判断：

```text
R4-A3 先把 Projects / Agents 两个页面的首批轻量 read model selector 从大页面中抽成可测试前端纯函数；页面仍不切源，后续 R4-A4+ 再决定是否让页面消费 selector。
```

## 1. Execution Mode

Execution Mode：Supervisor-led implementation with read-only review.

Multi-Agent Policy：

- 主管线负责实现、验证、证据、checkpoint 和 commit。
- 复核线复用既有线程，只做只读审查，不改代码。
- 不新增更多开发线程，避免上下文维护成本超过本切片收益。

## 2. 权威依据

必须服从：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`
- `tasks/2026-06-11-root-treatment-r4-a1-page-read-model-inventory-and-query-contract-v1.md`
- `tasks/2026-06-11-root-treatment-r4-a2-page-query-selector-contract-skeleton-v1.md`

## 3. 允许范围

允许实现：

- 新增独立前端 selector 模块，例如 `src/lib/pageSelectors.ts`。
- 从现有 `WorkbenchSnapshot` / `WorkflowStateSnapshot` 派生：
  - `ProjectsPageReadModel`：项目列表、项目计数、活跃项目、会话数、workflow summary count、用户首屏摘要、开发者明细是否折叠。
  - `AgentsPageReadModel`：项目选择摘要、会话选择摘要、可读 / 缺回放 / 已归档会话计数、adapter / operation / provider 边界计数、对话优先 UI 建议。
- 新增小离线测试验证：
  - selector schema / generated_from 明确来自 `workbench_snapshot_selector`。
  - `WorkbenchSnapshot` 仍 active。
  - Projects / Agents 输出不包含 raw transcript / secret / full prompt。
  - `result_count=null` 风格字段不被误显示成 0。
  - selector 不声明页面已迁移或 `WorkbenchSnapshot` 已废弃。
- 如必要，可在 `pageReadModel.ts` 增加 selector source boundary 类型。
- 如必要，可在 `page_read_model.rs` 的合同文案里把 R4-A3 next step 改为已进入 selector split，但不得新增 command 或业务数据返回。

## 4. 禁止范围

禁止实现：

- 不让 `ProjectsView.tsx` / `AgentView.tsx` 改为消费新 selector。
- 不修改页面布局、视觉风格、CSS 或信息层级。
- 不新增 Tauri command。
- 不新增 sidecar、DB migration、production read-cut、stop-write。
- 不废弃、删除或弱化 `load_workbench_snapshot` / `WorkbenchSnapshot`。
- 不执行真实 `codex exec` / `codex exec resume`。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript。
- 不启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 不解冻 Stage L / Stage K / backlog 功能。

## 5. 文件落点

允许修改：

- `prototypes/productized-desktop-shell/src/lib/pageSelectors.ts`
- `prototypes/productized-desktop-shell/src/lib/pageReadModel.ts`
- `prototypes/productized-desktop-shell/tests/r4-page-selectors.test.ts`
- `prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs`
- 本任务包、evidence、handoff。
- checkpoint 时同步当前入口文档和正式计划。

尽量避免修改：

- `types.ts`
- `ProjectsView.tsx`
- `AgentView.tsx`
- `styles.css`
- `offline-permission-dialog.test.tsx`
- Rust 后端文件。

## 6. 形状影响

- 新增代码落点：独立 TS selector 模块和独立 TS 测试。
- 预计不增长棘轮大文件。
- 不新增 Tauri command。
- 不新增 sidecar JSON。
- 不修改 Rust 后端。
- 新文件必须低于 TS 2,000 行上限。

## 7. 验收

必须通过：

- `node scripts/harness/workbench-shape-gate.js --mode check`
- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `git diff --check`

如未改 Rust，可不跑 `cargo test --lib`，但 evidence 必须说明原因。

扫描：

- 不得出现“R4 已完成 / 页面已迁移 / WorkbenchSnapshot 已废弃 / UI 已重做”等冒领。
- 不得新增真实 `codex exec` / `codex exec resume` 执行路径。
- 不得新增 `.codex` / secret / token / credential / full transcript 真实读取路径。

## 8. 完成后必须写

预期 evidence：

- `evidence/2026-06-11-root-treatment-r4-a3-projects-agents-selector-domain-split-v1.md`

预期 handoff：

- `handoffs/2026-06-11-root-treatment-r4-a3-projects-agents-selector-domain-split-v1-result.md`

handoff 必须包含：

- implementation commit。
- 新增 selector / test。
- 明确未迁移页面消费路径。
- 明确下一步建议：让 Projects / Agents 页面按小步骤消费 selector，或继续拆 Running / Memory selector。

## 9. 禁止声明

R4-A3 禁止声明：

- R4 完成。
- Projects / Agents 页面已切到新 command 或新数据源。
- `WorkbenchSnapshot` 已废弃。
- `ProjectsView` / `AgentView` 已拆分完成。
- UI 已重做或视觉已验收。
- 真实 Tauri / 截图验收完成。
- R3 Level B 已执行。
- 多 agent 并行真实执行已解锁。
