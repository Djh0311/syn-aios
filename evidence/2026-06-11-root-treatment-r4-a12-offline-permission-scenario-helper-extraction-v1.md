# Root Treatment / R4-A12 Offline Permission Scenario Helper Extraction v1 Evidence

日期：2026-06-11

状态：已完成并通过复核线 `STATUS: CLEAR_WITH_P2`；P2 已窄修。

任务包：`tasks/2026-06-11-root-treatment-r4-a12-offline-permission-scenario-helper-extraction-v1.md`

Planning baseline commit：`fe49ed8c25ed30628a029686f25f885e3316fad5`

Implementation commit：`f1b12530e6f37c74d8447e1b189c2ab4d055c23b`。

Review result：`STATUS: CLEAR_WITH_P2`；无 P0 / P1。P2 为 `git diff --check` 记录偏旧，已回填为最终状态。

Checkpoint commit：`e8fb35a24a5573979173f51accf2a41a6b9b216d`。

## 1. Scope

R4-A12 只做离线权限弹层场景 runner 抽离：把 `offline-permission-dialog.test.tsx` 中的 `Scenario`、`runScenario` 和确认按钮文案矩阵移到独立 helper。

本轮接受范围：

- 新增 `prototypes/productized-desktop-shell/tests/helpers/offlinePermissionScenarioUtils.tsx`。
- 抽离：
  - `OfflinePermissionScenario`
  - `CapturedActionState`
  - `runPermissionScenario`
  - `expectedDialogConfirmLabel`
- 主测试文件继续持有 `capturedAction`，通过显式 state adapter 传入 runner helper。
- 保留业务场景、fixture、业务 action builder 和断言内容在原文件。

本轮不接受范围：

- 不接受为 R4 完成。
- 不接受为离线测试全部按域拆分完成。
- 不接受为产品 UI 行为修改、视觉重做或布局重做。
- 不接受为页面真实数据来源迁移。
- 不接受为真实 Tauri / 截图验收完成。
- 不接受为 R3 Level B、真实执行、多 agent 并行真实执行解锁。
- 不接受为 Stage L / Stage K / backlog 功能解冻。

## 2. Changed Files

R4-A12 相关改动：

- `tasks/2026-06-11-root-treatment-r4-a12-offline-permission-scenario-helper-extraction-v1.md`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `prototypes/productized-desktop-shell/tests/helpers/offlinePermissionScenarioUtils.tsx`
- `evidence/2026-06-11-root-treatment-r4-a12-offline-permission-scenario-helper-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r4-a12-offline-permission-scenario-helper-extraction-v1-result.md`

本轮没有修改：

- 前端产品 TS / TSX 源码。
- `prototypes/productized-desktop-shell/src/styles.css`
- Rust / Tauri 后端。
- workflow state / sidecar / DB schema。
- 测试入口脚本 `scripts/run-offline-interaction-test.mjs`。

工作树外部变更：

- `backlog.md` 仍有 unrelated modified 状态，新增“Agent 成本记账”条目。
- 该文件不属于 R4-A12 允许写入范围，本轮没有修改、没有 stage、不会纳入 R4-A12 commit。

## 3. Implementation Notes

抽离策略：

- 新 helper 文件只依赖 `React`、`PermissionDialog`、`PendingAction` 类型和 R4-A11 通用测试 helper。
- 主测试文件新增 `capturedActionState`，显式提供 `get` / `set`，避免 helper 依赖主测试全局变量。
- `runPermissionScenario` 保留原断言顺序和文本：
  - 按钮存在。
  - `onClick` 后 pending action payload 深等。
  - Permission dialog 包含目标路径、路径来源、取消按钮和按 action kind 派生的确认按钮文案。
  - 点击取消只触发关闭，不触发确认。

行数变化：

- `offline-permission-dialog.test.tsx`：从 R4-A11 后的 9,293 行降到 9,185 行。
- 新增 `offlinePermissionScenarioUtils.tsx`：126 行。
- shape gate 记录 ratchet 状态：`9185/9369 (decreased)`。

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
  - `offline-permission-dialog.test.tsx: 9185/9369 (decreased)`
- `git diff --check`
  - 在 `git add --intent-to-add` 覆盖新文件后再次运行。
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

- `STATUS: CLEAR_WITH_P2`
- P0：无。
- P1：无。
- P2：evidence / handoff 中 `git diff --check` 记录偏旧，但新文件已经进入 diff 可见范围且复核线再次只读运行 `git diff --check` 通过。

主管线已窄修 P2：本 evidence 和 handoff 均已回填最终 `git diff --check` 状态。

复核结论：

- diff 范围符合任务边界；`backlog.md` 是外部已有 modified，已隔离，不纳入 R4-A12。
- helper 抽离内容与任务包一致：`OfflinePermissionScenario`、`CapturedActionState`、`runPermissionScenario`、`expectedDialogConfirmLabel` 已迁入 helper。
- `capturedAction` 仍由主测试持有，并通过显式 `{ get, set }` adapter 传给 helper。
- 测试入口列表和产品源码未改。
- 修完 P2 后可以 checkpoint。

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
