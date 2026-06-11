# Root Treatment / R4-A6 Knowledge Settings Selector Domain And Page Consumption v1

日期：2026-06-11

状态：已完成并通过复核线 `STATUS: CLEAR`。本文是 Root Treatment / Stage R 的 R4-A6 任务包；R4-A5 已完成并通过复核线 `STATUS: CLEAR`。R4-A6 只接受为 Knowledge Base / Settings 首批前端纯 selector 分域和页面最小消费；不接受为页面真实数据来源迁移、R4 完成、`query_workbench_page_read_model` 被页面真实消费、UI 重做、真实 Tauri 验收、R3 Level B 或多 agent 并行真实执行解锁。

Planning baseline commit：`c248f9bb390458ba64f2a809ec6876c543b5ff91`
Implementation commit：`9a175ff22e3177511e5b7749b7bf0c79eb47db98`
Review result：`STATUS: CLEAR`，复核线 `019eb51c-61fe-7fc3-8973-b22a4ce58911`。
Checkpoint commit：待回填。

## 0. 全局主管理解

已知事实：

- R4-A3 / R4-A4 已覆盖 Projects / Agents selector 分域和页面消费。
- R4-A5 已覆盖 Running Workflows / Memory Center selector 分域和页面消费，并通过复核线 `STATUS: CLEAR`。
- `KnowledgeBaseView.tsx` 当前直接调用 `deriveKnowledgeBaseSummary` 并消费首屏统计、边界摘要、资料列表和捕获事件。
- `SettingsView.tsx` 当前直接从 `snapshot`、`workflowState`、diagnostics、runtime log 和 `page_read_model_inventory` 派生首屏统计和内部边界摘要。
- R4 目标是 read model / frontend slimming，不是视觉风格重做、布局重做或真实执行能力开发。

核心判断：

```text
R4-A6 把 Knowledge / Settings 两个页面的首屏 summary 包进 `pageSelectors.ts` 的前端纯 selector，并让页面用 selector 输出承接已有摘要字段。页面仍使用既有 props，不切 `query_workbench_page_read_model`，不废弃 `WorkbenchSnapshot`。
```

## 1. Execution Mode

Execution Mode：Supervisor-led implementation with read-only review.

Multi-Agent Policy：

- 主管线负责任务包、实现、验证、证据、handoff 和 checkpoint。
- 复核线复用既有线程，只读审查，不改代码。
- 不新增开发线程；本切片集中在两个小型前端页面和一个 selector/test 文件，拆开发线会增加上下文维护成本。

## 2. Read / Write Scope

Allowed read paths：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`
- `tasks/2026-06-11-root-treatment-r4-a5-running-memory-selector-domain-and-page-consumption-v1.md`
- `evidence/2026-06-11-root-treatment-r4-a5-running-memory-selector-domain-and-page-consumption-v1.md`
- `handoffs/2026-06-11-root-treatment-r4-a5-running-memory-selector-domain-and-page-consumption-v1-result.md`
- `prototypes/productized-desktop-shell/src/lib/pageSelectors.ts`
- `prototypes/productized-desktop-shell/src/lib/knowledgeBase.ts`
- `prototypes/productized-desktop-shell/tests/r4-page-selectors.test.ts`
- `prototypes/productized-desktop-shell/src/views/KnowledgeBaseView.tsx`
- `prototypes/productized-desktop-shell/src/views/SettingsView.tsx`
- `prototypes/productized-desktop-shell/src/App.tsx` only if needed to confirm props wiring.

Allowed write paths：

- `tasks/2026-06-11-root-treatment-r4-a6-knowledge-settings-selector-domain-and-page-consumption-v1.md`
- `evidence/2026-06-11-root-treatment-r4-a6-knowledge-settings-selector-domain-and-page-consumption-v1.md`
- `handoffs/2026-06-11-root-treatment-r4-a6-knowledge-settings-selector-domain-and-page-consumption-v1-result.md`
- `prototypes/productized-desktop-shell/src/lib/pageSelectors.ts`
- `prototypes/productized-desktop-shell/tests/r4-page-selectors.test.ts`
- `prototypes/productized-desktop-shell/src/views/KnowledgeBaseView.tsx`
- `prototypes/productized-desktop-shell/src/views/SettingsView.tsx`
- `prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs` only if test registration must change.

## 3. Forbidden

R4-A6 禁止：

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

1. 扩展 `pageSelectors.ts`，新增 `deriveKnowledgeBasePageReadModelFromParts` / `deriveKnowledgeBasePageReadModel`，只包装 Knowledge 页首屏摘要、资料数量、正式记忆链接、候选链接、任务引用、捕获事件、Obsidian boundary 和 source boundary。
2. 扩展 `pageSelectors.ts`，新增 `deriveSettingsPageReadModelFromParts` / `deriveSettingsPageReadModel`，只包装 Settings 页首屏常规统计、adapter/provider/diagnostic/runtime log/page contract 数量和 developer boundary 摘要。
3. `KnowledgeBaseView.tsx` 使用 Knowledge selector 输出替换页头 meta 和 stat strip；保留现有 `summary` 局部变量供资料列表、详情、候选 action 和捕获事件使用。
4. `SettingsView.tsx` 使用 Settings selector 输出替换页头 meta、常规统计、内部边界摘要和页面合同数量；保留既有 `snapshot` / `pageReadModelInventory` 供合同清单渲染。
5. 扩展 `r4-page-selectors.test.ts`，覆盖 Knowledge / Settings split-input selector、source boundary、knowledge hit / candidate 不等于 formal memory、developer details 不外露 raw materials，以及敏感词不外露。
6. 写 evidence / handoff，回收边界和验证结果。

## 5. Acceptance Criteria

R4-A6 可接受条件：

- `KnowledgeBaseView.tsx` 明确消费 Knowledge page selector 输出。
- `SettingsView.tsx` 明确消费 Settings page selector 输出。
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

- 是否真的让 Knowledge / Settings 页面消费 selector。
- selector 是否保持前端纯函数和 source boundary。
- 是否避免新增 Tauri command、sidecar、DB migration、真实执行路径。
- 是否避免视觉 / CSS / 布局变更。
- 是否避免把 knowledge hit / candidate 说成正式记忆。
- 是否避免把 Settings 开发者边界摘要扩成普通用户首屏 raw materials。

## 8. 禁止声明

R4-A6 禁止声明：

- R4 完成。
- 页面真实数据来源已迁移。
- `query_workbench_page_read_model` 已被页面真实消费。
- `WorkbenchSnapshot` 已废弃。
- `KnowledgeBaseView` / `SettingsView` 已拆分完成。
- UI 已重做或视觉已验收。
- 真实 Tauri / 截图验收完成。
- R3 Level B 已执行。
- 多 agent 并行真实执行已解锁。
