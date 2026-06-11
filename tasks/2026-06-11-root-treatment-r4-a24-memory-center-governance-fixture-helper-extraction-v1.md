# Root Treatment / R4-A24 Memory Center Governance Fixture Helper Extraction v1

日期：2026-06-11

状态：实现完成，复核线 `STATUS: CLEAR`，等待 implementation commit。本文是 Root Treatment / Stage R 的 R4-A24 任务包；R4-A24 继续对应官方计划 R4-6：离线测试拆分。R4-A24 只接受为 memory lint / entity relation / relation preview 相关离线 fixture helper 抽离完成；不接受为 R4 完成、离线测试全部按域拆分完成、产品 UI 行为修改、真实 Tauri / 截图验收、页面真实数据来源迁移、R3 Level B 或 backlog 功能解冻。

Planning baseline commit：`d9accb18277a6a0629e4e3e0f93306bb2822135c`

Implementation commit：`TBD`

Review result：`STATUS: CLEAR`；P0/P1/P2 none；复核线程 `019eb51c-61fe-7fc3-8973-b22a4ce58911`

Checkpoint commit：`TBD`

## 0. 全局主管理解

已知事实：

- R4-A23 已把 Memory Center core stores 抽出，主测试降至 6,193 行。
- `runMemoryManagementCenterScenario` 中仍有 memory lint / maintenance report、entity relation store、entity / relation preview 三组同一治理输入 cluster。
- mature pattern store / preview 和后续 summary、MemoryCenterView render、权限弹层、UI 文案检查、forbidden text 断言必须留在主测试，不能混入本切片。

核心判断：

```text
R4-A24 只抽 Memory Center governance fixtures；主测试继续保留 mature pattern、summary、render 和所有断言。
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
- `tasks/2026-06-11-root-treatment-r4-a23-memory-center-core-store-fixture-helper-extraction-v1.md`
- `evidence/2026-06-11-root-treatment-r4-a23-memory-center-core-store-fixture-helper-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r4-a23-memory-center-core-store-fixture-helper-extraction-v1-result.md`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- existing files under `prototypes/productized-desktop-shell/tests/helpers/`
- shape gate output and package scripts as needed.

Allowed write paths：

- `tasks/2026-06-11-root-treatment-r4-a24-memory-center-governance-fixture-helper-extraction-v1.md`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `prototypes/productized-desktop-shell/tests/helpers/offlineMemoryCenterGovernanceFixtures.ts`
- `evidence/2026-06-11-root-treatment-r4-a24-memory-center-governance-fixture-helper-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r4-a24-memory-center-governance-fixture-helper-extraction-v1-result.md`
- checkpoint 入口文档只在验证和复核通过后同步。

## 3. Forbidden

R4-A24 禁止：

- 不改前端产品代码、CSS、Rust、Tauri command、sidecar、DB migration、store kind 或 workflow state 顶层结构。
- 不改 UI 视觉风格、布局、导航、文案或页面数据来源。
- 不改 `runMemoryManagementCenterScenario` 的 mature pattern、summary、MemoryCenterView render、UI 文案检查或 forbidden text 断言。
- 不抽 memory pattern / mature pattern store 和 preview；这些留给后续切片。
- 不改测试入口列表。
- 不启动 Tauri / Browser / Chrome / Vite dev / 截图工具。
- 不执行真实 `codex exec` / `codex exec resume`。
- 不发送 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript。
- 不解冻 Stage L / Stage K / backlog 功能。

## 4. Implementation Tasks

1. 新增 `tests/helpers/offlineMemoryCenterGovernanceFixtures.ts`。
2. 抽出 `memoryCenterGovernanceFixtures()`。
3. helper 返回原场景需要的治理输入：
   - `memoryLintStore`
   - `memoryEntityRelationStore`
   - `memoryEntityRelationPreview`
4. 主测试通过 helper 获取 fixture，继续在原场景内保留 mature pattern、deriveMemoryManagementSummary、MemoryCenterView render 和所有断言。
5. 写 evidence / handoff，记录行数变化、验证命令、边界和不能声明项。

## 5. Acceptance Criteria

R4-A24 可接受条件：

- 新 helper 文件只包含 memory center governance 离线 fixture builder，不读取文件、不启动进程、不调用 Tauri、不接触工作台运行时状态。
- `offline-permission-dialog.test.tsx` 行数继续下降。
- `npm run test:offline-interaction` 通过。
- `npm run typecheck` 通过。
- shape gate 通过；允许继承既有 `tauri_command_total_increased 97/96` warning。
- `git diff --check` 通过。
- 不修改产品代码、CSS、Rust、Tauri command、sidecar、DB、workflow state schema 或真实执行路径。

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

- diff 是否只包含 R4-A24 允许范围。
- 新 helper 是否只包含 memory lint / entity relation / relation preview fixture builder。
- 主测试是否未改 mature pattern、summary/UI/forbidden text 断言语义。
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

R4-A24 完成后仍不能声明：

- R4 全部完成。
- 离线测试全部按域拆分完成。
- 产品 UI、真实 Tauri、截图验收或真实数据来源迁移完成。
- Stage L / Stage K / backlog 功能已解冻。
- 真实 Codex 执行、真实 resume、真实 new session、真实 provider/model verification 完成。
