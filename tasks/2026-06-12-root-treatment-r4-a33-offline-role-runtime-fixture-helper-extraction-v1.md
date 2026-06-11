# Root Treatment / R4-A33 Offline Role Runtime Fixture Helper Extraction v1

日期：2026-06-12

状态：已完成，implementation / checkpoint hash 待回填。

Planning baseline commit：`f86f53e`

Implementation commit：待回填

Review result：`STATUS: CLEAR`；P0/P1/P2 none；复核线程 `019eb51c-61fe-7fc3-8973-b22a4ce58911`

Checkpoint commit：待回填

本文是 Root Treatment / Stage R 的 R4-A33 任务包；R4-A33 继续对应官方计划 R4-6：离线测试拆分。R4-A33 只接受为离线角色编排 action/form fixture 与运行关注 summary fixture 抽离完成；不接受为 R4 完成、离线测试全部按域拆分完成、产品 UI 行为修改、真实 Tauri / 截图验收、页面真实数据来源迁移、R3 Level B、真实 Codex 执行或 backlog 功能解冻。

## 0. 全局主管理解

已知事实：

- R4-A32 已完成并 checkpoint，`offline-permission-dialog.test.tsx` 当前约 4,681 行。
- R4-A17 起采用中等粒度 fixture cluster 拆分，但不能为了行数目标移动行为断言或场景语义。
- `runOfflineRoleOrchestrationScenario` 中仍内联离线角色派发 expected action 和 `FormData` stub；`runRuntimeSessionAttentionScenario` 中仍内联 E6 session run summary fixture。
- 这些对象属于纯测试数据；按钮查找、点击、表单提交、UI 文案检查、deep equality 断言和 forbidden 文案断言仍应留在主测试。

核心判断：

```text
R4-A33 只抽离线角色编排和运行关注的纯 fixture builder；主测试继续保留交互流程、render、点击、断言和测试入口列表。
```

## 1. Execution Mode

Execution Mode：Supervisor-led offline fixture helper extraction with review-line readback。

Multi-Agent Policy：

- 主管线负责任务包、实现、验证、evidence、handoff 和 checkpoint。
- 复核线复用既有线程，只读审查 diff 和验证结果，不改代码。
- 本切片按中等粒度 fixture cluster 推进；如果安全 cluster 低于 250 行，不为凑行数跨入行为断言。

## 2. Read / Write Scope

Allowed read paths：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`
- R4-A32 task / evidence / handoff
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- existing files under `prototypes/productized-desktop-shell/tests/helpers/`
- shape gate output and package scripts as needed.

Allowed write paths：

- `tasks/2026-06-12-root-treatment-r4-a33-offline-role-runtime-fixture-helper-extraction-v1.md`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `prototypes/productized-desktop-shell/tests/helpers/offlineRuntimeDiagnosticFixtures.ts`
- `prototypes/productized-desktop-shell/tests/helpers/offlineRoleOrchestrationFixtures.ts`
- `evidence/2026-06-12-root-treatment-r4-a33-offline-role-runtime-fixture-helper-extraction-v1.md`
- `handoffs/2026-06-12-root-treatment-r4-a33-offline-role-runtime-fixture-helper-extraction-v1-result.md`
- checkpoint 入口文档只在验证和复核通过后同步。

External changes not owned by R4-A33：

- `backlog.md`
- `docs/own-agent-and-company-vision-v1.md`

## 3. Forbidden

R4-A33 禁止：

- 不改前端产品代码、CSS、Rust、Tauri command、sidecar、DB migration、store kind 或 workflow state 顶层结构。
- 不改 UI 视觉风格、布局、导航、文案或页面数据来源。
- 不改 `runOfflineRoleOrchestrationScenario` / `runRuntimeSessionAttentionScenario` 的 render、按钮查找、点击、表单提交、UI 文案检查、forbidden 文案检查、deep equality 行为断言或测试入口列表。
- 不把 expected visible text 列表搬进 helper 来隐藏验收语义。
- 不启动 Tauri / Browser / Chrome / Vite dev / 截图工具。
- 不执行真实 `codex exec` / `codex exec resume`。
- 不发送 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript。
- 不解冻 Stage L / Stage K / backlog 功能。

## 4. Implementation Tasks

1. 扩展 `tests/helpers/offlineRuntimeDiagnosticFixtures.ts`：
   - 新增 `runtimeSessionSummaryFixture(sessionId, attention)`，仅生成 `SessionRunStatusSummary` fixture。
2. 新增 `tests/helpers/offlineRoleOrchestrationFixtures.ts`：
   - `missingOfflineDispatchBlock`
   - `offlineRoleDispatchFormDataFixture`
   - `missingOfflineRoleDispatchFormDataFixture`
   - `expectedOfflineRoleDispatchAction`
3. 更新 `offline-permission-dialog.test.tsx`：
   - 使用 runtime summary helper 替代内联 `SessionRunStatusSummary[]`。
   - 使用 role orchestration helper 替代内联 expected action 和 `FormData` stub。
   - 保留 parse、render、点击、submit、visible text、forbidden text 和 deep equality 断言。
4. 写 evidence / handoff，记录行数变化、验证命令、边界和不能声明项。

## 5. Acceptance Criteria

R4-A33 可接受条件：

- 新增 helper 只包含离线角色编排 action/form fixture 和运行关注 summary fixture，不读取文件、不启动进程、不调用 Tauri、不接触工作台运行时状态。
- `offline-permission-dialog.test.tsx` 行数继续下降。
- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过。
- shape gate 通过；允许继承既有 `tauri_command_total_increased 97/96` warning。
- `git diff --check` 通过。
- 不修改产品代码、CSS、Rust、Tauri command、sidecar、DB、workflow state schema 或真实执行路径。

## 6. Verification Plan

必须运行：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

本切片只改 TS 测试 helper 和文档，不默认运行 Rust 测试或 `npm run build`；如未运行必须在 evidence 中说明原因。

## 7. Review Plan

实现后复用既有复核线做只读审查。

复核重点：

- diff 是否只包含 R4-A33 允许范围，且不包含 `backlog.md` / `docs/own-agent-and-company-vision-v1.md`。
- helper 是否只包含离线角色编排 action/form fixture 和运行关注 summary fixture。
- 主测试是否未改 render、点击、表单提交、UI 文案检查、forbidden 文案检查、deep equality 行为断言或测试入口列表。
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

R4-A33 完成后仍不能声明：

- R4 全部完成。
- 离线测试全部按域拆分完成。
- 产品 UI、真实 Tauri、截图验收或页面真实数据来源迁移完成。
- Stage L / Stage K / backlog 功能已解冻。
- 真实 Codex 执行、真实 resume、真实 new session、真实 provider/model verification 完成。
