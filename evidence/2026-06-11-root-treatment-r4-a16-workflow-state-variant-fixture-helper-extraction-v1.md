# Root Treatment / R4-A16 Workflow State Variant Fixture Helper Extraction v1 Evidence

日期：2026-06-11

状态：已完成并通过复核线 `STATUS: CLEAR`。

任务包：`tasks/2026-06-11-root-treatment-r4-a16-workflow-state-variant-fixture-helper-extraction-v1.md`

Planning baseline commit：`f64897fdf08907be456d4d7081054a23cda434ac`

Implementation commit：`cdb71a6dbdc9e216fc39da921c0cb77caa21e6b6`。

Review result：`STATUS: CLEAR`；无 P0 / P1 / P2。

Checkpoint commit：`b5ae36120c80709f8639b4a71248cbc5b3ef1954`。

## 1. Scope

R4-A16 只做 workflow state 变体离线 fixture helper 抽离：把 `offline-permission-dialog.test.tsx` 中的 4 个 workflow state 变体 builder 移到独立 helper。

本轮接受范围：

- 新增 `prototypes/productized-desktop-shell/tests/helpers/offlineWorkflowStateVariantFixtures.ts`。
- 抽离以下纯 fixture builder：
  - `workflowStateReadyForReviewFixture`
  - `workflowStateWithPreparedOfflineDispatchFixture`
  - `workflowStateWithCompletedOfflineDispatchFixture`
  - `workflowStateWithGeneratedTaskFileFixture`
- 主测试保留原常量名：`workflowStateReadyForReview`、`workflowStateWithPreparedOfflineDispatch`、`workflowStateWithCompletedOfflineDispatch`、`workflowStateWithGeneratedTaskFile`。
- 将原主测试闭包中的 `project.project_root` 改为 helper 显式参数。

本轮不接受范围：

- 不接受为 R4 完成。
- 不接受为离线测试全部按域拆分完成。
- 不接受为产品 UI 行为修改、视觉重做或布局重做。
- 不接受为页面真实数据来源迁移。
- 不接受为真实 Tauri / 截图验收完成。
- 不接受为 R3 Level B、真实执行、多 agent 并行真实执行解锁。
- 不接受为 Stage L / Stage K / backlog 功能解冻。

## 2. Changed Files

R4-A16 相关改动：

- `tasks/2026-06-11-root-treatment-r4-a16-workflow-state-variant-fixture-helper-extraction-v1.md`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `prototypes/productized-desktop-shell/tests/helpers/offlineWorkflowStateVariantFixtures.ts`
- `evidence/2026-06-11-root-treatment-r4-a16-workflow-state-variant-fixture-helper-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r4-a16-workflow-state-variant-fixture-helper-extraction-v1-result.md`

本轮没有修改：

- 前端产品 TS / TSX 源码。
- `prototypes/productized-desktop-shell/src/styles.css`
- Rust / Tauri 后端。
- workflow state / sidecar / DB schema。
- 测试入口脚本 `scripts/run-offline-interaction-test.mjs`。

工作树外部变更：

- `backlog.md` 仍有 unrelated modified 状态。
- 该文件不属于 R4-A16 允许写入范围，本轮没有修改、没有 stage、不会纳入 R4-A16 commit。

## 3. Implementation Notes

抽离策略：

- 新 helper 文件只依赖前端类型 `WorkflowStateSnapshot`。
- helper 只做对象构造，不读取文件、不启动进程、不调用 Tauri、不接触真实工作台运行时状态。
- 主测试通过 helper 继续生成同名常量，后续场景断言仍引用原常量名。
- `workflowStateWithPreparedOfflineDispatchFixture` 和 `workflowStateWithCompletedOfflineDispatchFixture` 显式接收 `projectRoot`，并用于 `project_root`、`execution_cwd`、`allowed_reads`、`allowed_writes`。
- `workflowStateWithProjectWorkflow`、`workflowStateWithDerivedWorkflow`、`workflowStateWithC6ResultSummary` 等大型 fixture 仍留在主测试，没有搬动。

行数变化：

- `offline-permission-dialog.test.tsx`：从 R4-A15 后的 8,741 行降到 8,618 行。
- 新增 `offlineWorkflowStateVariantFixtures.ts`：160 行。
- shape gate 记录 ratchet 状态：`8618/9369 (decreased)`。

## 4. Verification

已运行并通过：

- `npm run test:offline-interaction`
  - `offline interaction tests passed: 14`
  - `r4 page read model settings test passed`
  - `r4 page read model query contract test passed`
  - `r4 page selectors test passed`
- `npm run typecheck`
  - `tsc --noEmit` 通过。
- `node scripts/harness/workbench-shape-gate.js --mode check`
  - `Status: pass`
  - `Errors: 0`
  - `Warnings: 1`
  - 继承 warning：`tauri_command_total_increased 97/96`
  - `offline-permission-dialog.test.tsx: 8618/9369 (decreased)`
- `git diff --check`
  - 无输出，检查通过。

命令过程说明：

- 首次 shape gate 曾在 `prototypes/productized-desktop-shell` 子目录运行，因相对路径找不到 `scripts/harness/workbench-shape-gate.js` 而失败；随后按既有正确方式在 `product-line` 根目录重跑并通过。这是 cwd 命令位置问题，不是代码失败。

未运行：

- `npm run build`：本切片只改测试 helper 和文档，不改产品源码。
- Rust 测试：本切片未改 Rust / Tauri 后端。

## 5. Boundary Confirmation

本轮没有：

- 修改前端产品代码、CSS、Rust、Tauri command、sidecar、DB migration、store kind 或 workflow state 顶层结构。
- 修改 UI 视觉风格、布局、导航、文案或页面数据来源。
- 修改离线测试入口列表。
- 启动 Tauri / Browser / Chrome / Vite dev / 截图工具。
- 执行真实 `codex exec` / `codex exec resume`。
- 发送 prompt。
- 读写 `/Users/yoyi/.codex`。
- 读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript。
- 解冻 Stage L / Stage K / backlog 功能。

## 6. Review Result

复核线回交：

- `STATUS: CLEAR`
- P0：无。
- P1：无。
- P2：无。

复核结论：

- `cdb71a6` 的实际 diff 只包含任务包、主离线测试、新 helper 3 个允许文件；`backlog.md` 是外部 unrelated modified。
- 新 helper 仅包含 4 个 workflow state 变体 builder，只有类型导入和对象构造，没有文件读取、进程启动、Tauri 调用、真实工作台状态或真实 Codex 路径接触。
- 抽离后的字段和值保持等价，`project.project_root` 已显式传入 prepared / completed 两个变体。
- 主测试保留了原常量名和原场景断言，四个常量仍在原文件中定义并被场景继续消费。
- 禁止搬动的大 fixture 仍留在主测试：`workflowStateWithProjectWorkflow`、`workflowStateWithDerivedWorkflow`、`workflowStateWithC6ResultSummary` 还在原位。

## 7. Cannot Claim

不能声明：

- R4 完成。
- 离线测试已全部按域拆分完成。
- UI 行为、视觉或布局已修改 / 已验收。
- 页面真实数据来源已迁移。
- `query_workbench_page_read_model` 已被页面真实消费。
- `WorkbenchSnapshot` 已废弃。
- 真实 Tauri / 截图验收完成。
- R3 Level B 已执行。
- Stage L / Stage K / backlog 功能已解冻。
