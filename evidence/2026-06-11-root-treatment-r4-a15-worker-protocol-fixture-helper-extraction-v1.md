# Root Treatment / R4-A15 Worker Protocol Fixture Helper Extraction v1 Evidence

日期：2026-06-11

状态：已完成并通过复核线 `STATUS: CLEAR`。

任务包：`tasks/2026-06-11-root-treatment-r4-a15-worker-protocol-fixture-helper-extraction-v1.md`

Planning baseline commit：`624b22332b4951bf48351c7a9949c54318590222`

Implementation commit：`99087175c570ba6a6296cb596b4b4544a75a8d20`。

Review result：`STATUS: CLEAR`；无 P0 / P1 / P2。

Checkpoint commit：`327c741f4eaac7a16895ba857e7b036f9f70dfdb`。

## 1. Scope

R4-A15 只做 worker protocol 相关离线 fixture builder 抽离：把 `offline-permission-dialog.test.tsx` 中的 `workerProtocolFixtureForAdapters` 移到独立 helper。

本轮接受范围：

- 新增 `prototypes/productized-desktop-shell/tests/helpers/offlineWorkerProtocolFixtures.ts`。
- 抽离 `workerProtocolFixtureForAdapters`。
- 将原主测试隐式读取 `backendProviderAvailabilitySummaries` 的部分改为显式参数 `providerAvailabilitySummaries`。
- 主测试文件继续保留 adapter / diagnostics 场景流程、UI 渲染和断言。

本轮不接受范围：

- 不接受为 R4 完成。
- 不接受为离线测试全部按域拆分完成。
- 不接受为产品 UI 行为修改、视觉重做或布局重做。
- 不接受为页面真实数据来源迁移。
- 不接受为真实 Tauri / 截图验收完成。
- 不接受为 R3 Level B、真实执行、多 agent 并行真实执行解锁。
- 不接受为 Stage L / Stage K / backlog 功能解冻。

## 2. Changed Files

R4-A15 相关改动：

- `tasks/2026-06-11-root-treatment-r4-a15-worker-protocol-fixture-helper-extraction-v1.md`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `prototypes/productized-desktop-shell/tests/helpers/offlineWorkerProtocolFixtures.ts`
- `evidence/2026-06-11-root-treatment-r4-a15-worker-protocol-fixture-helper-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r4-a15-worker-protocol-fixture-helper-extraction-v1-result.md`

本轮没有修改：

- 前端产品 TS / TSX 源码。
- `prototypes/productized-desktop-shell/src/styles.css`
- Rust / Tauri 后端。
- workflow state / sidecar / DB schema。
- 测试入口脚本 `scripts/run-offline-interaction-test.mjs`。

工作树外部变更：

- `backlog.md` 仍有 unrelated modified 状态。
- 该文件不属于 R4-A15 允许写入范围，本轮没有修改、没有 stage、不会纳入 R4-A15 commit。

## 3. Implementation Notes

抽离策略：

- 新 helper 文件只依赖前端类型 `AgentAdapterDescriptor`、`ProviderAvailabilitySummary`、`SessionOperationDescriptor` 和 `WorkerProtocolReadModel`。
- `workerProtocolFixtureForAdapters(descriptors, providerAvailabilitySummaries, operations)` 保留原 worker adapter、credential requirement、external call risk envelope、adapter checklist、CLI semantics、diagnostic event schema、adapter health、degraded mode 和 data location fixture 语义。
- 主测试通过显式传入 `backendProviderAvailabilitySummaries` 替代原 helper 对主测试闭包变量的隐式依赖。
- 主测试保留原 adapter / diagnostics 场景和断言，仅从 helper 获取 worker protocol fixture。

行数变化：

- `offline-permission-dialog.test.tsx`：从 R4-A14 后的 8,916 行降到 8,741 行。
- 新增 `offlineWorkerProtocolFixtures.ts`：182 行。
- shape gate 记录 ratchet 状态：`8741/9369 (decreased)`。

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
  - `offline-permission-dialog.test.tsx: 8741/9369 (decreased)`
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

- implementation commit `99087175c570ba6a6296cb596b4b4544a75a8d20` 本身只改了 3 个允许文件：测试主文件、测试 helper 和任务包。
- 新 helper 只承载 `workerProtocolFixtureForAdapters`，且只有类型导入与纯数据构造，没有文件读取、进程启动、Tauri 调用或真实运行时状态接触。
- 显式 `providerAvailabilitySummaries` 参数替代了原先对闭包变量的隐式依赖，调用点仍传入同一份 `backendProviderAvailabilitySummaries`。
- 主测试仍保留 adapter / diagnostics 场景流程和断言，没有把场景逻辑搬进 helper。
- `backlog.md` 的 modified 状态与本轮隔离，未计入 R4-A15 结论。

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
