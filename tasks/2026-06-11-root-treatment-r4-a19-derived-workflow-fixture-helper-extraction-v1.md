# Root Treatment / R4-A19 Derived Workflow Fixture Helper Extraction v1

日期：2026-06-11

状态：已完成并通过复核线 `STATUS: CLEAR_WITH_P2`。本文是 Root Treatment / Stage R 的 R4-A19 任务包；R4-A19 继续对应官方计划 R4-6：离线测试拆分。R4-A19 只接受为 pending workflow result summary / derived workflow 相关离线 fixture helper 抽离完成；不接受为 R4 完成、离线测试全部按域拆分完成、产品 UI 行为修改、视觉重做、真实 Tauri / 截图验收、页面真实数据来源迁移、R3 Level B 或 backlog 功能解冻。

Planning baseline commit：`0f7a7b6500e660ab2376ac7602512c8b390b9750`

Implementation commit：`a37fba01d183e6751199e2910c90c09a69923c6c`

Review result：`STATUS: CLEAR_WITH_P2`；P0 / P1 无。P2-1 为任务包元数据回填，已在 checkpoint 收尾中处理；P2-2 为 helper 629 行略超 600 软阈值，记录为后续增长时再拆，不阻断本轮。

Checkpoint commit：`TBD`

## 0. 全局主管理解

已知事实：

- R4-A18 已把基础 `workflowState` / `workflowStateWithProjectWorkflow` fixture 抽到 `offlineProjectWorkflowStateFixtures.ts`。
- `offline-permission-dialog.test.tsx` 仍包含 `pendingWorkflowResultSummary` 和 `workflowStateWithDerivedWorkflow` 大块纯对象构造。
- `workflowStateWithC6ResultSummary` 仍是 C6-specific fixture，本轮不移动，避免一次切片过大。

核心判断：

```text
R4-A19 把 pending workflow result summary 与 derived workflow 纯 fixture builder 成组搬出主测试文件；保留 C6 result summary 和场景断言在主测试，继续降低主测试体积但不改变测试语义。
```

## 1. Execution Mode

Execution Mode：Supervisor-led offline fixture helper extraction with review-line readback。

Multi-Agent Policy：

- 主管线负责任务包、实现、验证、evidence、handoff 和 checkpoint。
- 复核线复用既有线程，只读审查 diff 和验证结果，不改代码。
- 本切片继续中等粒度治理，不新建开发线。

## 2. Read / Write Scope

Allowed read paths：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`
- `tasks/2026-06-11-root-treatment-r4-a18-project-workflow-state-fixture-helper-extraction-v1.md`
- `evidence/2026-06-11-root-treatment-r4-a18-project-workflow-state-fixture-helper-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r4-a18-project-workflow-state-fixture-helper-extraction-v1-result.md`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- existing files under `prototypes/productized-desktop-shell/tests/helpers/`
- shape gate output and package scripts as needed.

Allowed write paths：

- `tasks/2026-06-11-root-treatment-r4-a19-derived-workflow-fixture-helper-extraction-v1.md`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `prototypes/productized-desktop-shell/tests/helpers/offlineDerivedWorkflowFixtures.ts`
- `evidence/2026-06-11-root-treatment-r4-a19-derived-workflow-fixture-helper-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r4-a19-derived-workflow-fixture-helper-extraction-v1-result.md`
- checkpoint 入口文档只在验证和复核通过后同步。

## 3. Forbidden

R4-A19 禁止：

- 不改前端产品代码、CSS、Rust、Tauri command、sidecar、DB migration、store kind 或 workflow state 顶层结构。
- 不改 UI 视觉风格、布局、导航、文案或页面数据来源。
- 不改测试入口列表。
- 不改测试 fixture 语义、断言目标、成功计数语义或输出文案。
- 不搬 C6 result summary 大 fixture 或场景断言。
- 不启动 Tauri / Browser / Chrome / Vite dev / 截图工具。
- 不执行真实 `codex exec` / `codex exec resume`。
- 不发送 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript。
- 不解冻 Stage L / Stage K / backlog 功能。

## 4. Implementation Tasks

1. 新增 `tests/helpers/offlineDerivedWorkflowFixtures.ts`。
2. 抽出 `derivedWorkflowStateFixtures(...)`。
3. helper 返回原常量对应的同名字段：
   - `pendingWorkflowResultSummary`
   - `workflowStateWithDerivedWorkflow`
4. 主测试保留原常量名，通过 destructuring 获取同等 fixture，确保 C6 result summary 和后续断言引用不变。
5. 写 evidence / handoff，记录行数变化、验证命令、边界和不能声明项。

## 5. Acceptance Criteria

R4-A19 可接受条件：

- 新 helper 文件只包含 pending workflow result summary / derived workflow 离线 fixture builder，不读取文件、不启动进程、不调用 Tauri、不接触工作台运行时状态。
- `offline-permission-dialog.test.tsx` 行数继续下降。
- `npm run test:offline-interaction` 通过。
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

本切片只改测试 helper 和文档，不默认运行 Rust 测试或 `npm run build`；如未运行必须在 evidence 中说明原因。

## 7. Review Plan

实现后复用既有复核线做只读审查。

复核重点：

- diff 是否只包含 R4-A19 允许范围。
- helper 抽离是否保持 pending workflow result summary / derived workflow fixture 行为等价。
- 主测试是否仍保留原常量名和后续 C6 / 场景断言。
- 是否没有搬动 C6 result summary 大 fixture 或业务断言。
- 是否没有把 R4-A19 冒充成 R4 完成、离线测试全部拆完或真实 Tauri 验收完成。

## 8. 禁止声明

R4-A19 禁止声明：

- R4 完成。
- 离线测试已全部按域拆分完成。
- UI 行为、视觉或布局已修改 / 已验收。
- 页面真实数据来源已迁移。
- `query_workbench_page_read_model` 已被页面真实消费。
- `WorkbenchSnapshot` 已废弃。
- 真实 Tauri / 截图验收完成。
- R3 Level B 已执行。
- Stage L / Stage K / backlog 功能已解冻。
