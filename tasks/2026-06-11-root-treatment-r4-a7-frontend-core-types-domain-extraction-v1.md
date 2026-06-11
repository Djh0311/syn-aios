# Root Treatment / R4-A7 Frontend Core Types Domain Extraction v1

日期：2026-06-11

状态：待执行。本文是 Root Treatment / Stage R 的 R4-A7 任务包；R4-A6 已完成并通过复核线 `STATUS: CLEAR`。R4-A7 只接受为前端 `types.ts` 第一批基础类型分域抽取；不接受为 R4 完成、页面真实数据来源迁移、`query_workbench_page_read_model` 被页面真实消费、UI 重做、真实 Tauri 验收、R3 Level B 或多 agent 并行真实执行解锁。

Planning baseline commit：`272307fab0dfd5a595b052c7551e23df31187d2a`
Implementation commit：待回填。
Review result：待回填。
Checkpoint commit：待回填。

## 0. 全局主管理解

已知事实：

- R4-A1 到 R4-A6 已完成 page read model contract、Projects / Agents / Running / Memory / Knowledge / Settings selector 分域和页面最小消费。
- 官方计划 R4-2 是 TS 类型分域，验收目标是 `types.ts` 行数下降。
- `prototypes/productized-desktop-shell/src/lib/types.ts` 当前约 5,149 行，是 shape gate ratchet 文件。
- `types.ts` 前部的项目索引、会话、transcript、skill/plugin/task/diagnostics/index summary 类型是基础类型，迁移到独立模块后可从 `types.ts` re-export，保持现有页面和 helper import 不变。

核心判断：

```text
R4-A7 先抽取 `types.ts` 的基础索引 / 会话 / transcript / project record 类型到 `workbenchCoreTypes.ts`，让 `types.ts` 行数实际下降，同时不改任何运行逻辑和 UI。
```

## 1. Execution Mode

Execution Mode：Supervisor-led implementation with read-only review。

Multi-Agent Policy：

- 主管线负责任务包、实现、验证、证据、handoff 和 checkpoint。
- 复核线复用既有线程，只读审查，不改代码。
- 本切片集中在一个类型模块抽取，不新增开发线程，避免上下文维护成本。

## 2. Read / Write Scope

Allowed read paths：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/*.ts`
- `prototypes/productized-desktop-shell/src/views/*.tsx`
- `prototypes/productized-desktop-shell/tests/r4-page-selectors.test.ts`
- shape gate output and package scripts as needed.

Allowed write paths：

- `tasks/2026-06-11-root-treatment-r4-a7-frontend-core-types-domain-extraction-v1.md`
- `evidence/2026-06-11-root-treatment-r4-a7-frontend-core-types-domain-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r4-a7-frontend-core-types-domain-extraction-v1-result.md`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/workbenchCoreTypes.ts`
- `prototypes/productized-desktop-shell/tests/r4-page-selectors.test.ts` only if type-only test coverage must be adjusted.

## 3. Forbidden

R4-A7 禁止：

- 不改 UI、CSS、布局、视觉风格、导航入口或页面文案。
- 不新增 Tauri command。
- 不新增 sidecar、DB migration、store kind 或 workflow state 顶层结构。
- 不改 `App.tsx` 数据加载路径。
- 不切 `query_workbench_page_read_model` 为页面真实数据源。
- 不废弃或弱化 `WorkbenchSnapshot` / `load_workbench_snapshot`。
- 不执行真实 `codex exec` / `codex exec resume`。
- 不发送 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript。
- 不启动 Tauri / Browser / Chrome / Vite dev / 截图工具。
- 不解冻 Stage L / Stage K / backlog 功能。

## 4. Implementation Tasks

1. 新增 `src/lib/workbenchCoreTypes.ts`，迁移 `FileCandidate`、`HarnessCandidate`、`HarnessEntrypoint`、`HarnessResource`、`ProjectRecord`、`SessionRecord`、`CodexTranscriptEvent`、`CodexTranscript`、`CodexTranscriptViewerBoundary`、`SkillRecord`、`PluginRecord`、`TaskEntry`、`Diagnostics`、`IndexSummary`。
2. `src/lib/types.ts` 改为 type-only import + re-export 这些基础类型，保证现有 `import type { ProjectRecord } from "../lib/types"` 仍可用。
3. 不批量改调用方 import，避免无价值 churn。
4. 运行 typecheck / 离线交互 / build / shape gate / diff check。
5. 写 evidence / handoff，记录 `types.ts` 行数下降、验证结果和禁止声明。

## 5. Acceptance Criteria

R4-A7 可接受条件：

- `types.ts` 行数下降，且 shape gate 通过。
- 迁移后的基础类型在 `workbenchCoreTypes.ts` 中有单一源。
- `types.ts` 继续 re-export 迁移类型，既有 import 不破坏。
- `npm run typecheck`、`npm run test:offline-interaction`、`npm run build` 通过。
- 不新增产品能力、不改 UI、不改 Rust、不改数据读取路径。

## 6. Verification Plan

必须运行：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

如未运行 Rust 说明原因；本切片默认不改 Rust。

## 7. Review Plan

实现后复用既有复核线 `019eb51c-61fe-7fc3-8973-b22a4ce58911` 做只读审查。

复核重点：

- `types.ts` 是否真实下降且仍 re-export 迁移类型。
- 是否没有改运行逻辑、UI、CSS、Rust、Tauri command、sidecar、DB、workflow state schema。
- 是否没有 import churn 或破坏现有调用方。
- 验证是否覆盖 typecheck、离线交互、build 和 shape gate。

## 8. 禁止声明

R4-A7 禁止声明：

- R4 完成。
- 页面真实数据来源已迁移。
- `query_workbench_page_read_model` 已被页面真实消费。
- `WorkbenchSnapshot` 已废弃。
- `types.ts` 已完整拆分完成。
- UI 已重做或视觉已验收。
- 真实 Tauri / 截图验收完成。
- R3 Level B 已执行。
- 多 agent 并行真实执行已解锁。
