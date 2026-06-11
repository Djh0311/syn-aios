# Root Treatment / R4-A15 Worker Protocol Fixture Helper Extraction v1

日期：2026-06-11

状态：已完成并通过复核线 `STATUS: CLEAR`。本文是 Root Treatment / Stage R 的 R4-A15 任务包；R4-A15 继续对应官方计划 R4-6：离线测试拆分。R4-A15 只接受为 worker protocol 相关离线 fixture builder 抽离完成；不接受为 R4 完成、离线测试全部按域拆分完成、产品 UI 行为修改、视觉重做、真实 Tauri / 截图验收、页面真实数据来源迁移、R3 Level B 或 backlog 功能解冻。

Planning baseline commit：`624b22332b4951bf48351c7a9949c54318590222`

Implementation commit：`99087175c570ba6a6296cb596b4b4544a75a8d20`

Review result：`STATUS: CLEAR`；无 P0 / P1 / P2。

Checkpoint commit：待回填。

## 0. 全局主管理解

已知事实：

- R4-A11 / A12 / A13 / A14 已连续抽出通用 helper、权限弹层场景 runner、任务字段 / 派发准备 helper、runtime / diagnostic fixture helper。
- `offline-permission-dialog.test.tsx` 仍保留 `workerProtocolFixtureForAdapters`，用于构造 adapter-neutral worker protocol 离线读模型。
- 该函数只构造离线测试数据，不读取文件、不启动进程、不调用 Tauri、不接触真实工作台状态。

核心判断：

```text
R4-A15 把 worker protocol 纯 fixture builder 搬出主测试文件，继续降低主测试体积；本轮不搬 adapter / diagnostics 场景断言，不改测试语义。
```

## 1. Execution Mode

Execution Mode：Supervisor-led offline fixture helper extraction with review-line readback。

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
- `prototypes/productized-desktop-shell/tests/helpers/offlineRuntimeDiagnosticFixtures.ts`
- shape gate output and package scripts as needed.

Allowed write paths：

- `tasks/2026-06-11-root-treatment-r4-a15-worker-protocol-fixture-helper-extraction-v1.md`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `prototypes/productized-desktop-shell/tests/helpers/offlineWorkerProtocolFixtures.ts`
- `evidence/2026-06-11-root-treatment-r4-a15-worker-protocol-fixture-helper-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r4-a15-worker-protocol-fixture-helper-extraction-v1-result.md`
- checkpoint 入口文档只在验证和复核通过后同步。

## 3. Forbidden

R4-A15 禁止：

- 不改前端产品代码、CSS、Rust、Tauri command、sidecar、DB migration、store kind 或 workflow state 顶层结构。
- 不改 UI 视觉风格、布局、导航、文案或页面数据来源。
- 不改测试入口列表。
- 不改测试 fixture 语义、断言目标、成功计数语义或输出文案。
- 不搬 adapter / diagnostics 场景断言，不拆大型业务 fixture。
- 不启动 Tauri / Browser / Chrome / Vite dev / 截图工具。
- 不执行真实 `codex exec` / `codex exec resume`。
- 不发送 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript。
- 不解冻 Stage L / Stage K / backlog 功能。

## 4. Implementation Tasks

1. 新增 `tests/helpers/offlineWorkerProtocolFixtures.ts`。
2. 抽出 `workerProtocolFixtureForAdapters`。
3. 将原本隐式读取 `backendProviderAvailabilitySummaries` 的部分改为显式参数传入，保持调用方传入同一份 summary 以维持行为等价。
4. 主测试保留 adapter / diagnostics 场景流程和断言，仅从 helper 获取 worker protocol fixture。
5. 写 evidence / handoff，记录行数变化、验证命令、边界和不能声明项。

## 5. Acceptance Criteria

R4-A15 可接受条件：

- 新 helper 文件只包含 worker protocol 离线 fixture builder，不读取文件、不启动进程、不调用 Tauri、不接触工作台运行时状态。
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

- diff 是否只包含 R4-A15 允许范围。
- helper 抽离是否保持 worker protocol fixture 行为等价。
- 显式 provider availability 参数是否正确替代原来的隐式依赖。
- 主测试是否仍保留 adapter / diagnostics 场景流程和断言。
- 是否没有把 R4-A15 冒充成 R4 完成、离线测试全部拆完或真实 Tauri 验收完成。

## 8. 禁止声明

R4-A15 禁止声明：

- R4 完成。
- 离线测试已全部按域拆分完成。
- UI 行为、视觉或布局已修改 / 已验收。
- 页面真实数据来源已迁移。
- `query_workbench_page_read_model` 已被页面真实消费。
- `WorkbenchSnapshot` 已废弃。
- 真实 Tauri / 截图验收完成。
- R3 Level B 已执行。
- Stage L / Stage K / backlog 功能已解冻。
