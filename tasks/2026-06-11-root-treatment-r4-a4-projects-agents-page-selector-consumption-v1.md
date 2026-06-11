# Root Treatment / R4-A4 Projects Agents Page Selector Consumption v1

日期：2026-06-11

状态：已完成，复核线 `STATUS: CLEAR`。本文是 Root Treatment / Stage R 的 R4-A4 任务包；R4-A3 已完成并通过复核线 `STATUS: CLEAR`。R4-A4 只接受为 Projects / Agents 页面以最小 diff 消费 R4-A3 前端纯 selector，不接受为页面真实数据来源迁移、R4 完成、`WorkbenchSnapshot` 废弃、UI 重做或真实执行解锁。

规划基线 commit：`17dd1243e1b62e8ef86b9bb865d964a6cceae02f`
Implementation commit：`58804f43cb3a666ddae66eecd1390def253c2ed2`
Implementation hash backfill commit：`d04a28a`
Review result：`STATUS: CLEAR`，无 P0 / P1 / P2。
Checkpoint commit：`0edb6c4a76e579c03f1b12ad80feba14f0a93bf6`

## 0. 全局主管理解

已知事实：

- R4-A3 新增 `pageSelectors.ts` 和 `r4-page-selectors.test.ts`；R4-A4 已让 `ProjectsView.tsx` / `AgentView.tsx` 以最小 diff 消费 selector。
- `ProjectsView.tsx` / `AgentView.tsx` 当前仍从 `App.tsx` 接收 `WorkbenchSnapshot` 拆出的 props。
- R4 目标是读模型和前端瘦身，不是视觉重做、布局重做或真实执行能力开发。

核心判断：

```text
R4-A4 只把首批重复派生逻辑接到 R4-A3 selector 上：Projects 页面消费项目列表统计 read model；Agents 页面消费项目选项、会话摘要和边界计数 read model。页面仍使用既有 props，仍不切 `query_workbench_page_read_model`，仍不废弃 `WorkbenchSnapshot`。
```

## 1. Execution Mode

Execution Mode：Supervisor-led implementation with read-only review.

Multi-Agent Policy：

- 主管线负责实现、验证、证据、handoff 和 checkpoint。
- 复核线复用既有线程，只读审查，不改代码。
- 不新增开发线程；本切片共享文件集中，拆开发线会提高协调成本。

## 2. Read / Write Scope

Allowed read paths：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`
- `tasks/2026-06-11-root-treatment-r4-a3-projects-agents-selector-domain-split-v1.md`
- `evidence/2026-06-11-root-treatment-r4-a3-projects-agents-selector-domain-split-v1.md`
- `handoffs/2026-06-11-root-treatment-r4-a3-projects-agents-selector-domain-split-v1-result.md`
- `prototypes/productized-desktop-shell/src/lib/pageSelectors.ts`
- `prototypes/productized-desktop-shell/tests/r4-page-selectors.test.ts`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/App.tsx` only if needed to confirm data source wiring.

Allowed write paths：

- `tasks/2026-06-11-root-treatment-r4-a4-projects-agents-page-selector-consumption-v1.md`
- `evidence/2026-06-11-root-treatment-r4-a4-projects-agents-page-selector-consumption-v1.md`
- `handoffs/2026-06-11-root-treatment-r4-a4-projects-agents-page-selector-consumption-v1-result.md`
- `prototypes/productized-desktop-shell/src/lib/pageSelectors.ts`
- `prototypes/productized-desktop-shell/tests/r4-page-selectors.test.ts`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs` only if test registration must change.

## 3. Forbidden

R4-A4 禁止：

- 不新增 Tauri command。
- 不新增 sidecar、DB migration、store kind 或 workflow state 顶层结构。
- 不改 `App.tsx` 数据加载路径，除非仅为类型确认；默认不传整包 `WorkbenchSnapshot` 到页面。
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

1. 扩展或适配 `pageSelectors.ts`，让 selector 可从页面现有拆分 props 派生，同时保持 R4-A3 snapshot API 可用。
2. `ProjectsView.tsx` 使用 `deriveProjectsPageReadModel` 替换项目总数、会话数、workflow count、warning count 和 gallery item 的重复统计。
3. `AgentView.tsx` 使用 `deriveAgentsPageReadModel` 替换 `agentProjectOptions` / 会话摘要 / adapter boundary 计数的重复派生；普通 UI 不增加新的开发者边界面板。
4. 扩展 `r4-page-selectors.test.ts`，覆盖 split-input selector 和页面消费所需字段。
5. 写 evidence / handoff，回收边界和验证结果。

## 5. Acceptance Criteria

R4-A4 可接受条件：

- `ProjectsView.tsx` 明确消费 R4-A3 selector 输出。
- `AgentView.tsx` 明确消费 R4-A3 selector 输出。
- R4-A3 原有 snapshot selector 调用仍兼容。
- 页面视觉和布局不变。
- `WorkbenchSnapshot` 仍是页面数据来源，不声明废弃。
- selector 仍为前端纯函数，不写 store、不发命令、不读敏感路径。
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

- 是否真的让 Projects / Agents 页面消费 selector。
- 是否保持页面视觉和数据源边界不变。
- 是否避免新增 Tauri command、sidecar、DB migration、真实执行路径。
- 是否避免 `ProjectsView.tsx` / `AgentView.tsx` 继续增肥超出最小接线。

## 8. 禁止声明

R4-A4 禁止声明：

- R4 完成。
- 页面真实数据来源已迁移。
- `query_workbench_page_read_model` 已被页面真实消费。
- `WorkbenchSnapshot` 已废弃。
- `ProjectsView` / `AgentView` 已拆分完成。
- UI 已重做或视觉已验收。
- 真实 Tauri / 截图验收完成。
- R3 Level B 已执行。
- 多 agent 并行真实执行已解锁。
