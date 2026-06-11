# Root Treatment / R4-A11 Offline Interaction Test Domain Extraction v1

日期：2026-06-11

状态：已完成并通过复核线 `STATUS: CLEAR_WITH_P2`；P2 已窄修。本文是 Root Treatment / Stage R 的 R4-A11 任务包；R4-A10 已完成并通过复核线 `STATUS: CLEAR`。R4-A11 对应官方计划 R4-6：离线测试拆分。R4-A11 只接受为 `offline-permission-dialog.test.tsx` 测试底座 / 域拆分第一批治理完成；不接受为 R4 完成、产品 UI 行为修改、视觉重做、真实 Tauri / 截图验收、页面真实数据来源迁移、R3 Level B 或 backlog 功能解冻。

Planning baseline commit：`7dedb792b563ed6842becefc1a72d1454d8dd286`

Implementation commit：`40cc37b9e3bf862e03468f7ce2063712e0ccfa96`。

Review result：`STATUS: CLEAR_WITH_P2`；无 P0 / P1。P2 为 evidence / handoff 中 `git diff --check` 记录过期，已回填为最终状态。

Checkpoint commit：待回填。

## 0. 全局主管理解

已知事实：

- 官方计划 R4-6 是离线测试拆分，验收目标是主测试文件行数下降且测试仍绿。
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx` 当前约 9,369 行，混合 fixture、场景、通用 helper 和 UI 断言。
- 当前离线测试入口是 `prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs`，已包含多个测试 entry。
- 治理期要求少拆任务、可回滚、先做低风险结构治理，不顺手解冻功能。

核心判断：

```text
R4-A11 第一批只抽离通用测试 helper，让主测试文件减少基础工具代码并为后续按域拆分场景铺路；本轮不改产品代码、不改测试语义、不改测试入口列表。
```

## 1. Execution Mode

Execution Mode：Supervisor-led test helper extraction with review-line readback。

Multi-Agent Policy：

- 主管线负责任务包、实现、验证、evidence、handoff 和 checkpoint。
- 复核线复用既有线程，只读审查 diff 和验证结果，不改代码。
- 本切片不新建多条开发线，避免把单个测试 helper 抽离拆得过细。

## 2. Read / Write Scope

Allowed read paths：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`
- `tasks/2026-06-11-root-treatment-r4-a10-styles-css-ink-style-asset-inventory-v1.md`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs`
- shape gate output and package scripts as needed.

Allowed write paths：

- `tasks/2026-06-11-root-treatment-r4-a11-offline-interaction-test-domain-extraction-v1.md`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `prototypes/productized-desktop-shell/tests/helpers/offlineInteractionTestUtils.tsx`
- `evidence/2026-06-11-root-treatment-r4-a11-offline-interaction-test-domain-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r4-a11-offline-interaction-test-domain-extraction-v1-result.md`
- checkpoint 入口文档只在验证和复核通过后同步。

## 3. Forbidden

R4-A11 禁止：

- 不改前端产品代码、CSS、Rust、Tauri command、sidecar、DB migration、store kind 或 workflow state 顶层结构。
- 不改 UI 视觉风格、布局、导航、文案或页面数据来源。
- 不改 `scripts/run-offline-interaction-test.mjs` 的测试 entry 列表，除非验证证明必须且另行记录原因。
- 不改测试 fixture 语义、断言目标或输出成功文案。
- 不启动 Tauri / Browser / Chrome / Vite dev / 截图工具。
- 不执行真实 `codex exec` / `codex exec resume`。
- 不发送 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript。
- 不解冻 Stage L / Stage K / backlog 功能。

## 4. Implementation Tasks

1. 新增 `tests/helpers/offlineInteractionTestUtils.tsx`。
2. 从 `offline-permission-dialog.test.tsx` 抽出通用工具：
   - `ReactElementLike`
   - `visibleText`
   - `findElement`
   - `findButtonByText`
   - `findButtonContainingText`
   - `buttonTextsInMarkup`
   - `assert`
   - `assertDeepEqual`
3. 保留场景函数、fixture、`runScenario`、`expectedDialogConfirmLabel` 和业务 action builder 在原文件，避免第一批拆分过大。
4. 更新导入和类型引用，保持测试行为不变。
5. 写 evidence / handoff，记录行数变化、验证命令、边界和不能声明项。

## 5. Acceptance Criteria

R4-A11 可接受条件：

- 新 helper 文件只包含测试通用工具，不引用产品运行时状态、不读取文件、不启动进程。
- `offline-permission-dialog.test.tsx` 行数下降。
- `npm run test:offline-interaction` 通过，成功项仍为既有离线交互测试集合。
- `npm run typecheck` 通过。
- shape gate 通过；允许继承既有 `tauri_command_total_increased 97/96` warning。
- `git diff --check` 通过。
- 不修改产品代码、Rust、Tauri command、sidecar、DB、workflow state schema 或真实执行路径。

## 6. Verification Plan

必须运行：

- `npm run test:offline-interaction`
- `npm run typecheck`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

可选运行：

- `npm run build`

本切片只改测试和文档，不默认运行 Rust 测试；如未运行必须在 evidence 中说明原因。

## 7. Review Plan

实现后复用既有复核线做只读审查。

复核重点：

- diff 是否只包含任务包、测试 helper、主测试导入 / helper 删除、evidence / handoff 和 checkpoint 文档。
- helper 抽离是否保持行为等价，没有改断言语义。
- 主测试文件行数是否下降，测试是否仍绿。
- 是否没有把 R4-A11 冒充成 R4 完成、UI 重做、测试体系彻底拆完或真实 Tauri 验收完成。

## 8. 禁止声明

R4-A11 禁止声明：

- R4 完成。
- 离线测试已全部按域拆分完成。
- UI 行为、视觉或布局已修改 / 已验收。
- 页面真实数据来源已迁移。
- `query_workbench_page_read_model` 已被页面真实消费。
- `WorkbenchSnapshot` 已废弃。
- 真实 Tauri / 截图验收完成。
- R3 Level B 已执行。
- Stage L / Stage K / backlog 功能已解冻。
