# Root Treatment / R4-A5 Running Memory Selector Domain And Page Consumption v1

日期：2026-06-11

状态：implemented_pending_review。本文是 Root Treatment / Stage R 的 R4-A5 任务包；R4-A4 已完成并通过复核线 `STATUS: CLEAR`。R4-A5 只接受为 Running Workflows / Memory Center 首批前端纯 selector 分域和页面最小消费；不接受为页面真实数据来源迁移、R4 完成、`query_workbench_page_read_model` 被页面真实消费、UI 重做、真实 Tauri 验收、R3 Level B 或多 agent 并行真实执行解锁。

Planning baseline commit：`930bde34bffe551fb7ec7840313576e1f3ad9493`
Implementation commit：`955783f4629176d930fd0b2fb1d881aa6a289c0d`
Review result：待回填。
Checkpoint commit：待回填。

## 0. 全局主管理解

已知事实：

- R4-A3 / R4-A4 已建立 Projects / Agents selector 和页面最小消费路径。
- `RunningWorkflowsView.tsx` 目前直接从 `snapshot`、`workflowState`、memory stores 和 `deriveRunQueueReadModel` 派生页面摘要。
- `MemoryCenterView.tsx` 目前直接调用 `deriveMemoryManagementSummary` 并在页面内消费大量 summary 字段。
- R4 目标是 read model / frontend slimming，不是视觉风格重做、布局重做或真实执行能力开发。

核心判断：

```text
R4-A5 把 Running / Memory 两个页面的首批页面级 summary 包进 `pageSelectors.ts` 的前端纯 selector，并让页面用 selector 输出承接已有摘要字段。页面仍使用既有 props，不切 `query_workbench_page_read_model`，不废弃 `WorkbenchSnapshot`。
```

## 1. Execution Mode

Execution Mode：Supervisor-led implementation with read-only review.

Multi-Agent Policy：

- 主管线负责任务包、实现、验证、证据、handoff 和 checkpoint。
- 复核线复用既有线程，只读审查，不改代码。
- 不新增开发线程；本切片集中在两个中型前端页面和一个 selector/test 文件，拆开发线会增加上下文维护成本。

## 2. Read / Write Scope

Allowed read paths：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`
- `tasks/2026-06-11-root-treatment-r4-a4-projects-agents-page-selector-consumption-v1.md`
- `evidence/2026-06-11-root-treatment-r4-a4-projects-agents-page-selector-consumption-v1.md`
- `handoffs/2026-06-11-root-treatment-r4-a4-projects-agents-page-selector-consumption-v1-result.md`
- `prototypes/productized-desktop-shell/src/lib/pageSelectors.ts`
- `prototypes/productized-desktop-shell/src/lib/runQueue.ts`
- `prototypes/productized-desktop-shell/src/lib/memoryCenter.ts`
- `prototypes/productized-desktop-shell/tests/r4-page-selectors.test.ts`
- `prototypes/productized-desktop-shell/src/views/RunningWorkflowsView.tsx`
- `prototypes/productized-desktop-shell/src/views/MemoryCenterView.tsx`
- `prototypes/productized-desktop-shell/src/App.tsx` only if needed to confirm props wiring.

Allowed write paths：

- `tasks/2026-06-11-root-treatment-r4-a5-running-memory-selector-domain-and-page-consumption-v1.md`
- `evidence/2026-06-11-root-treatment-r4-a5-running-memory-selector-domain-and-page-consumption-v1.md`
- `handoffs/2026-06-11-root-treatment-r4-a5-running-memory-selector-domain-and-page-consumption-v1-result.md`
- `prototypes/productized-desktop-shell/src/lib/pageSelectors.ts`
- `prototypes/productized-desktop-shell/tests/r4-page-selectors.test.ts`
- `prototypes/productized-desktop-shell/src/views/RunningWorkflowsView.tsx`
- `prototypes/productized-desktop-shell/src/views/MemoryCenterView.tsx`
- `prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs` only if test registration must change.

## 3. Forbidden

R4-A5 禁止：

- 不新增 Tauri command。
- 不新增 sidecar、DB migration、store kind 或 workflow state 顶层结构。
- 不改 `App.tsx` 数据加载路径，除非只做 props 类型确认。
- 不切 `query_workbench_page_read_model` 为页面真实数据源。
- 不废弃或弱化 `WorkbenchSnapshot` / `load_workbench_snapshot`。
- 不改视觉风格、布局、CSS、交互入口或文案层级。
- 不执行真实 `codex exec` / `codex exec resume`。
- 不发送 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript。
- 不启动 Tauri / Browser / Chrome / Vite dev / 截图工具。
- 不解冻 Stage L / Stage K / backlog 功能。

## 4. Implementation Tasks

1. 扩展 `pageSelectors.ts`，新增 `deriveRunningWorkflowsPageReadModelFromParts` / `deriveRunningWorkflowsPageReadModel`，只包装 Running 页首屏摘要、运行队列数量、权限、读回、失败控制、operation control 和记忆待处理计数。
2. 扩展 `pageSelectors.ts`，新增 `deriveMemoryCenterPageReadModelFromParts` / `deriveMemoryCenterPageReadModel`，只包装 Memory 页首屏摘要、正式记忆、候选、捕获、观察、lint、实体关系、成熟模式和 action 数量。
3. `RunningWorkflowsView.tsx` 使用 Running selector 输出替换首屏 summary tile 和 header 中的重复派生；保留现有 runQueue 局部变量供详细列表使用。
4. `MemoryCenterView.tsx` 使用 Memory selector 输出替换首屏 stat strip / memory workbench 数字；保留现有 `summary` 局部变量供详细列表、操作按钮和高级模块使用。
5. 扩展 `r4-page-selectors.test.ts`，覆盖 Running / Memory split-input selector、source boundary、readback unknown 不变 0、candidate 不等于 formal memory、developer details / internal source 不外露。
6. 写 evidence / handoff，回收边界和验证结果。

## 5. Acceptance Criteria

R4-A5 可接受条件：

- `RunningWorkflowsView.tsx` 明确消费 Running page selector 输出。
- `MemoryCenterView.tsx` 明确消费 Memory page selector 输出。
- 新 selector 为前端纯函数，不写 store、不发 command、不读敏感路径。
- 页面视觉和布局不变。
- `WorkbenchSnapshot` 仍是页面数据来源，不声明废弃。
- `query_workbench_page_read_model` 仍未作为页面真实数据源消费。
- 相关测试和 shape gate 通过。

## 6. Verification Plan

必须运行：

- `node scripts/harness/workbench-shape-gate.js --mode check`
- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `git diff --check`

如未运行 Rust 说明原因；本切片默认不改 Rust。

## 7. Review Plan

实现后复用既有复核线 `019eb51c-61fe-7fc3-8973-b22a4ce58911` 做只读审查。

复核重点：

- 是否真的让 Running / Memory 页面消费 selector。
- selector 是否保持前端纯函数和 source boundary。
- 是否避免新增 Tauri command、sidecar、DB migration、真实执行路径。
- 是否避免视觉 / CSS / 布局变更。
- 是否避免把 readback unavailable 显示成 0 或把 candidate / observation / knowledge hit 说成正式记忆。

## 8. 禁止声明

R4-A5 禁止声明：

- R4 完成。
- 页面真实数据来源已迁移。
- `query_workbench_page_read_model` 已被页面真实消费。
- `WorkbenchSnapshot` 已废弃。
- `RunningWorkflowsView` / `MemoryCenterView` 已拆分完成。
- UI 已重做或视觉已验收。
- 真实 Tauri / 截图验收完成。
- R3 Level B 已执行。
- 多 agent 并行真实执行已解锁。
