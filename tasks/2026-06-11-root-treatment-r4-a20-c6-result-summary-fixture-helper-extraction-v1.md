# Root Treatment / R4-A20 C6 Result Summary Fixture Helper Extraction v1

日期：2026-06-11

状态：已完成并通过复核线 `STATUS: CLEAR`。本文是 Root Treatment / Stage R 的 R4-A20 任务包；R4-A20 继续对应官方计划 R4-6：离线测试拆分。R4-A20 只接受为 C6 result summary / workflow state 离线 fixture helper 抽离完成；不接受为 R4 完成、离线测试全部按域拆分完成、产品 UI 行为修改、真实 Tauri / 截图验收、页面真实数据来源迁移、R3 Level B 或 backlog 功能解冻。

Planning baseline commit：`47bd235220bb03b092b8211b489d5ea108ac8a40`

Implementation commit：`4af2e62160622a21e38e9515845350f3984f89d4`

Review result：`STATUS: CLEAR`；P0 / P1 / P2 无。

Checkpoint commit：`dc4f34c8fb6213ea333b84336e927c7d200616a6`

## 0. 全局主管理解

已知事实：

- R4-A19 已把 pending workflow result summary / derived workflow fixture 抽到 `offlineDerivedWorkflowFixtures.ts`。
- R4-A19 复核线给出 P2：该 helper 629 行，略超 600 行软阈值，后续增长时应避免继续塞入同一 helper。
- `offline-permission-dialog.test.tsx` 仍保留 `c6WorkflowResultSummary` 和 `workflowStateWithC6ResultSummary` 纯对象构造，C6 场景断言仍在主测试。

核心判断：

```text
R4-A20 新建 C6 result summary 专用 fixture helper，只搬纯对象构造；主测试继续保留 C6 断言和交互验证，确保行为语义不变。
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
- `tasks/2026-06-11-root-treatment-r4-a19-derived-workflow-fixture-helper-extraction-v1.md`
- `evidence/2026-06-11-root-treatment-r4-a19-derived-workflow-fixture-helper-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r4-a19-derived-workflow-fixture-helper-extraction-v1-result.md`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- existing files under `prototypes/productized-desktop-shell/tests/helpers/`
- shape gate output and package scripts as needed.

Allowed write paths：

- `tasks/2026-06-11-root-treatment-r4-a20-c6-result-summary-fixture-helper-extraction-v1.md`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `prototypes/productized-desktop-shell/tests/helpers/offlineC6ResultSummaryFixtures.ts`
- `evidence/2026-06-11-root-treatment-r4-a20-c6-result-summary-fixture-helper-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r4-a20-c6-result-summary-fixture-helper-extraction-v1-result.md`
- checkpoint 入口文档只在验证和复核通过后同步。

## 3. Forbidden

R4-A20 禁止：

- 不改前端产品代码、CSS、Rust、Tauri command、sidecar、DB migration、store kind 或 workflow state 顶层结构。
- 不改 UI 视觉风格、布局、导航、文案或页面数据来源。
- 不改 C6 场景断言、按钮查找、确认弹层检查或测试输出文案。
- 不改测试入口列表。
- 不改测试 fixture 语义、成功计数语义或输出文案。
- 不启动 Tauri / Browser / Chrome / Vite dev / 截图工具。
- 不执行真实 `codex exec` / `codex exec resume`。
- 不发送 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript。
- 不解冻 Stage L / Stage K / backlog 功能。

## 4. Implementation Tasks

1. 新增 `tests/helpers/offlineC6ResultSummaryFixtures.ts`。
2. 抽出 `c6ResultSummaryFixtures(...)`。
3. helper 返回原常量对应的同名字段：
   - `c6WorkflowResultSummary`
   - `workflowStateWithC6ResultSummary`
4. 主测试保留原常量名，通过 destructuring 获取同等 fixture，确保 C6 后续断言引用不变。
5. 写 evidence / handoff，记录行数变化、验证命令、边界和不能声明项。

## 5. Acceptance Criteria

R4-A20 可接受条件：

- 新 helper 文件只包含 C6 result summary / workflow state 离线 fixture builder，不读取文件、不启动进程、不调用 Tauri、不接触工作台运行时状态。
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

- diff 是否只包含 R4-A20 允许范围。
- 新 helper 是否只包含纯 fixture 数据构造。
- 主测试是否只从 helper 引入同名 fixture，未改 C6 断言语义。
- 是否没有产品代码、CSS、Rust、Tauri、DB、sidecar 或 workflow schema 修改。
- 验证命令是否通过，shape gate 是否无新增阻断。

## 8. Checkpoint Plan

复核通过后：

1. 写入 evidence / handoff。
2. 提交实现 commit。
3. 同步 checkpoint 入口文档。
4. 提交 checkpoint commit。
5. 回填本任务包 / evidence / handoff 中的 commit hash，并提交 hash backfill。

## 9. 不能声明

R4-A20 完成后仍不能声明：

- R4 全部完成。
- 离线测试全部按域拆分完成。
- 产品 UI、真实 Tauri、截图验收或真实数据来源迁移完成。
- Stage L / Stage K / backlog 功能已解冻。
- 真实 Codex 执行、真实 resume、真实 new session、真实 provider/model verification 完成。
