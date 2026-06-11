# Root Treatment / R4-A17 Authorization Workflow Fixture Helper Extraction v1 Evidence

日期：2026-06-11

状态：已完成并通过复核线 `STATUS: CLEAR`。

任务包：`tasks/2026-06-11-root-treatment-r4-a17-authorization-workflow-fixture-helper-extraction-v1.md`

Planning baseline commit：`973631184082ec5929f1cc536aef19c52db5a32d`

Implementation commit：`a69bf7f4b74c4d47c43d3880caa93bd65612bfcd`。

Review result：`STATUS: CLEAR`；无 P0 / P1 / P2。

Checkpoint commit：`TBD`。

## 1. Scope

R4-A17 只做授权 / 咨询 / 项目主管计划 / run check 相关离线 fixture helper 抽离：把 `offline-permission-dialog.test.tsx` 中的 C2-C4 authorization workflow fixture cluster 移到独立 helper。

本轮接受范围：

- 新增 `prototypes/productized-desktop-shell/tests/helpers/offlineAuthorizationWorkflowFixtures.ts`。
- 抽离 `authorizationWorkflowFixtures(projectRoot, sessionThreadId, workflowProjectId, workflowId)`。
- helper 返回原常量对应的同名字段：blocked / runnable workflow run check、plan authorization store、project consultation proposal store、pending global store、confirmed / active proposal store、project director task plan。
- 主测试保留原常量名、summary 派生调用和 C2-C4 场景断言。

本轮不接受范围：

- 不接受为 R4 完成。
- 不接受为离线测试全部按域拆分完成。
- 不接受为产品 UI 行为修改、视觉重做或布局重做。
- 不接受为页面真实数据来源迁移。
- 不接受为真实 Tauri / 截图验收完成。
- 不接受为 R3 Level B、真实执行、多 agent 并行真实执行解锁。
- 不接受为 Stage L / Stage K / backlog 功能解冻。

## 2. Changed Files

R4-A17 相关改动：

- `tasks/2026-06-11-root-treatment-r4-a17-authorization-workflow-fixture-helper-extraction-v1.md`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `prototypes/productized-desktop-shell/tests/helpers/offlineAuthorizationWorkflowFixtures.ts`
- `evidence/2026-06-11-root-treatment-r4-a17-authorization-workflow-fixture-helper-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r4-a17-authorization-workflow-fixture-helper-extraction-v1-result.md`

本轮没有修改：

- 前端产品 TS / TSX 源码。
- `prototypes/productized-desktop-shell/src/styles.css`
- Rust / Tauri 后端。
- workflow state / sidecar / DB schema。
- 测试入口脚本 `scripts/run-offline-interaction-test.mjs`。

工作树外部变更：

- `backlog.md` 仍有 unrelated modified 状态。
- 该文件不属于 R4-A17 允许写入范围，本轮没有修改、没有 stage、不会纳入 R4-A17 commit。

## 3. Implementation Notes

抽离策略：

- 新 helper 文件只依赖前端类型 `AutoDispatchGuardResult`、`PlanAuthorizationStoreV1`、`ProjectConsultationProposalStoreV1`、`ProjectDirectorTaskPlan` 和 `WorkflowRunCheck`。
- helper 只做对象构造，不读取文件、不启动进程、不调用 Tauri、不接触真实工作台运行时状态。
- 主测试通过 destructuring 继续获得原同名常量，后续场景断言仍引用原常量名。
- `project.project_root`、`session.thread_id`、`workflowProjectId`、`workflowId` 改为 helper 显式参数。
- `summarizePlanAuthorizationStore` / `summarizeProjectConsultationProposalStore` 派生调用仍留在主测试。
- `workflowStateWithProjectWorkflow`、`workflowStateWithDerivedWorkflow`、`workflowStateWithC6ResultSummary` 等大型 fixture 仍留在主测试，没有搬动。

行数变化：

- `offline-permission-dialog.test.tsx`：从 R4-A16 后的 8,618 行降到 8,281 行。
- 新增 `offlineAuthorizationWorkflowFixtures.ts`：370 行。
- shape gate 记录 ratchet 状态：`8281/9369 (decreased)`。

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
  - `offline-permission-dialog.test.tsx: 8281/9369 (decreased)`
- `git diff --check`
  - 无输出，检查通过。

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

- `a69bf7f` 的实现 diff 只包含任务包、主离线测试、新 helper 3 个允许文件；`backlog.md` 是外部 unrelated modified。
- 新 helper 只包含授权 / 咨询 / 项目主管计划 / run check 离线 fixture builder，只有类型导入和对象构造，没有文件读取、进程启动、Tauri 调用或真实 Codex 路径。
- 显式参数化保持了原语义：`projectRoot`、`sessionThreadId`、`workflowProjectId`、`workflowId` 都作为 helper 入参，并被用于原字段位置。
- 主测试保留了原常量名、summary 派生调用和 C2-C4 场景断言。
- 禁止搬动的大 fixture 仍留在主测试，没有被抽走。

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
