# Root Treatment / R4-A12 Offline Permission Scenario Helper Extraction v1

日期：2026-06-11

状态：已完成并通过复核线 `STATUS: CLEAR_WITH_P2`；P2 已窄修。本文是 Root Treatment / Stage R 的 R4-A12 任务包；R4-A11 已完成并通过复核线 `STATUS: CLEAR_WITH_P2`，P2 已窄修。R4-A12 继续对应官方计划 R4-6：离线测试拆分。R4-A12 只接受为权限弹层离线场景 runner / confirm label helper 抽离完成；不接受为 R4 完成、离线测试全部按域拆分完成、产品 UI 行为修改、视觉重做、真实 Tauri / 截图验收、页面真实数据来源迁移、R3 Level B 或 backlog 功能解冻。

Planning baseline commit：`fe49ed8c25ed30628a029686f25f885e3316fad5`

Implementation commit：`f1b12530e6f37c74d8447e1b189c2ab4d055c23b`。

Review result：`STATUS: CLEAR_WITH_P2`；无 P0 / P1。P2 为 evidence / handoff 中 `git diff --check` 记录偏旧，已回填为最终状态。

Checkpoint commit：`e8fb35a24a5573979173f51accf2a41a6b9b216d`。

## 0. 全局主管理解

已知事实：

- R4-A11 已抽出通用测试 helper，但 `offline-permission-dialog.test.tsx` 仍保留 `Scenario`、`runScenario` 和 `expectedDialogConfirmLabel`。
- 这些函数属于权限弹层离线测试场景 runner，不是业务 fixture，也不是产品运行代码。
- `capturedAction` 是主测试的局部状态，抽离 runner 时需要通过显式 state adapter 传入，不能隐式共享全局变量。

核心判断：

```text
R4-A12 把权限弹层场景 runner 和确认按钮文案矩阵抽到测试 helper，让主测试继续下降；本轮不拆大型 fixture，不改场景断言，不改测试入口。
```

## 1. Execution Mode

Execution Mode：Supervisor-led test scenario helper extraction with review-line readback。

Multi-Agent Policy：

- 主管线负责任务包、实现、验证、evidence、handoff 和 checkpoint。
- 复核线复用既有线程，只读审查 diff 和验证结果，不改代码。
- 本切片继续小步治理，不新建开发线。

## 2. Read / Write Scope

Allowed read paths：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `prototypes/productized-desktop-shell/tests/helpers/offlineInteractionTestUtils.tsx`
- `prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs`
- shape gate output and package scripts as needed.

Allowed write paths：

- `tasks/2026-06-11-root-treatment-r4-a12-offline-permission-scenario-helper-extraction-v1.md`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `prototypes/productized-desktop-shell/tests/helpers/offlinePermissionScenarioUtils.tsx`
- `evidence/2026-06-11-root-treatment-r4-a12-offline-permission-scenario-helper-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r4-a12-offline-permission-scenario-helper-extraction-v1-result.md`
- checkpoint 入口文档只在验证和复核通过后同步。

## 3. Forbidden

R4-A12 禁止：

- 不改前端产品代码、CSS、Rust、Tauri command、sidecar、DB migration、store kind 或 workflow state 顶层结构。
- 不改 UI 视觉风格、布局、导航、文案或页面数据来源。
- 不改测试入口列表。
- 不改测试 fixture 语义、断言目标、成功计数语义或输出文案。
- 不拆大型 fixture，不把多个业务域一次性搬走。
- 不启动 Tauri / Browser / Chrome / Vite dev / 截图工具。
- 不执行真实 `codex exec` / `codex exec resume`。
- 不发送 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript。
- 不解冻 Stage L / Stage K / backlog 功能。

## 4. Implementation Tasks

1. 新增 `tests/helpers/offlinePermissionScenarioUtils.tsx`。
2. 抽出：
   - `OfflinePermissionScenario`
   - `CapturedActionState`
   - `runPermissionScenario`
   - `expectedDialogConfirmLabel`
3. 主测试文件保留 `capturedAction`，通过显式 `{ get, set }` 传给 runner helper。
4. 保留所有业务场景、fixture、业务 action builder 和断言内容在原文件。
5. 写 evidence / handoff，记录行数变化、验证命令、边界和不能声明项。

## 5. Acceptance Criteria

R4-A12 可接受条件：

- 新 helper 文件只包含测试权限弹层场景 runner，不读取文件、不启动进程、不调用 Tauri、不接触工作台运行时状态。
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

本切片只改测试和文档，不默认运行 Rust 测试或 `npm run build`；如未运行必须在 evidence 中说明原因。

## 7. Review Plan

实现后复用既有复核线做只读审查。

复核重点：

- diff 是否只包含 R4-A12 允许范围。
- runner helper 抽离是否保持行为等价。
- `capturedAction` 是否仍由主测试持有，并通过显式 state adapter 传入。
- 是否没有把 R4-A12 冒充成 R4 完成、离线测试全部拆完或真实 Tauri 验收完成。

## 8. 禁止声明

R4-A12 禁止声明：

- R4 完成。
- 离线测试已全部按域拆分完成。
- UI 行为、视觉或布局已修改 / 已验收。
- 页面真实数据来源已迁移。
- `query_workbench_page_read_model` 已被页面真实消费。
- `WorkbenchSnapshot` 已废弃。
- 真实 Tauri / 截图验收完成。
- R3 Level B 已执行。
- Stage L / Stage K / backlog 功能已解冻。
