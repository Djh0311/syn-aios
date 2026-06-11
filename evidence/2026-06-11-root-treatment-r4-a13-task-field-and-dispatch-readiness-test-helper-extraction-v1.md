# Root Treatment / R4-A13 Task Field And Dispatch Readiness Test Helper Extraction v1 Evidence

日期：2026-06-11

状态：已完成并通过复核线 `STATUS: CLEAR`。

任务包：`tasks/2026-06-11-root-treatment-r4-a13-task-field-and-dispatch-readiness-test-helper-extraction-v1.md`

Planning baseline commit：`be76c26747065cb8239462154737eaf23b49c77c`

Implementation commit：`843d765825554c034b4490d69ab4a581fb5ec2bb`。

Review result：`STATUS: CLEAR`；无 P0 / P1 / P2。

Checkpoint commit：待回填。

## 1. Scope

R4-A13 只做任务字段 / 派发准备相关离线测试 helper 抽离：把 `offline-permission-dialog.test.tsx` 中的 not-ready 派发准备 fixture、任务字段保存 action builder、派发字段修正 action builder 和字段列表 parser 移到独立 helper。

本轮接受范围：

- 新增 `prototypes/productized-desktop-shell/tests/helpers/offlineTaskFieldTestUtils.ts`。
- 抽离：
  - `buildNotReadyDispatchReadiness`
  - `buildUpdateTaskFieldsAction`
  - `buildCorrectDispatchFieldsAction`
  - 内部 `listValue`
- 主测试文件继续保留场景流程、UI 渲染、按钮查找、权限弹层断言和业务断言。

本轮不接受范围：

- 不接受为 R4 完成。
- 不接受为离线测试全部按域拆分完成。
- 不接受为产品 UI 行为修改、视觉重做或布局重做。
- 不接受为页面真实数据来源迁移。
- 不接受为真实 Tauri / 截图验收完成。
- 不接受为 R3 Level B、真实执行、多 agent 并行真实执行解锁。
- 不接受为 Stage L / Stage K / backlog 功能解冻。

## 2. Changed Files

R4-A13 相关改动：

- `tasks/2026-06-11-root-treatment-r4-a13-task-field-and-dispatch-readiness-test-helper-extraction-v1.md`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `prototypes/productized-desktop-shell/tests/helpers/offlineTaskFieldTestUtils.ts`
- `evidence/2026-06-11-root-treatment-r4-a13-task-field-and-dispatch-readiness-test-helper-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r4-a13-task-field-and-dispatch-readiness-test-helper-extraction-v1-result.md`

本轮没有修改：

- 前端产品 TS / TSX 源码。
- `prototypes/productized-desktop-shell/src/styles.css`
- Rust / Tauri 后端。
- workflow state / sidecar / DB schema。
- 测试入口脚本 `scripts/run-offline-interaction-test.mjs`。

工作树外部变更：

- `backlog.md` 仍有 unrelated modified 状态。
- 该文件不属于 R4-A13 允许写入范围，本轮没有修改、没有 stage、不会纳入 R4-A13 commit。

## 3. Implementation Notes

抽离策略：

- 新 helper 文件只依赖前端类型 `PendingAction`、`TaskPackageDispatchReadiness` 和 `TaskPackageFields`。
- `buildNotReadyDispatchReadiness(projectRoot)` 保留原 not-ready 派发准备 fixture 的字段、阻断原因、artifact path 和 memory injection summary。
- `buildUpdateTaskFieldsAction` / `buildCorrectDispatchFieldsAction` 保留原 payload、label、source、boundary 和字段列表解析语义。
- 主测试保留原场景和断言，仅从 helper 获取 fixture / action payload builder。

行数变化：

- `offline-permission-dialog.test.tsx`：从 R4-A12 后的 9,185 行降到 9,116 行。
- 新增 `offlineTaskFieldTestUtils.ts`：86 行。
- shape gate 记录 ratchet 状态：`9116/9369 (decreased)`。

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
  - 第一次在 `prototypes/productized-desktop-shell` 错误 cwd 运行，失败为 `MODULE_NOT_FOUND`，原因是脚本路径相对 product-line 根目录。
  - 第二次在 `/Users/yoyi/workspace/product-line` 运行通过。
  - `Status: pass`
  - `Errors: 0`
  - `Warnings: 1`
  - 继承 warning：`tauri_command_total_increased 97/96`
  - `offline-permission-dialog.test.tsx: 9116/9369 (decreased)`
- `git diff --check`
  - 在 `git add --intent-to-add` 覆盖新文件后运行。
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

- implementation commit `843d765825554c034b4490d69ab4a581fb5ec2bb` 本身只改了 3 个允许文件：测试主文件、测试 helper 和任务包。
- helper 只承载 3 个允许抽离的 builder 和内部 `listValue`，仅依赖类型导入，没有 I/O、进程启动、Tauri 调用或真实执行路径。
- 主测试仍保留场景总流程和断言，helper 仅在 not-ready readiness 和两个 action payload builder 位置被接入。
- shape gate 第一次因错误 cwd 触发 `MODULE_NOT_FOUND`、第二次在仓库根目录通过，这一记录已明确归因为命令落点错误，不属于实现缺陷，可接受。
- `backlog.md` 的 modified 状态与本轮隔离，未计入 R4-A13 结论。

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
