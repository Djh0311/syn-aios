# Root Treatment / R4-A7 Frontend Core Types Domain Extraction v1 Evidence

日期：2026-06-11

状态：已实现，待复核线回收。

任务包：`tasks/2026-06-11-root-treatment-r4-a7-frontend-core-types-domain-extraction-v1.md`

Planning baseline commit：`272307fab0dfd5a595b052c7551e23df31187d2a`

Implementation commit：待回填。

## 1. Scope

R4-A7 只实现前端 `types.ts` 第一批基础类型分域抽取。

本轮接受范围：

- 新增 `prototypes/productized-desktop-shell/src/lib/workbenchCoreTypes.ts`。
- 从 `types.ts` 迁移基础索引 / 会话 / transcript / project record / skill / plugin / diagnostics 类型。
- `types.ts` 继续 type-only import 并 re-export 迁移类型，保持既有 `../lib/types` import 兼容。
- `types.ts` 行数从 5,149 下降到 4,998。

本轮不接受范围：

- 不接受为 R4 完成。
- 不接受为 `types.ts` 完整拆分完成。
- 不接受为页面真实数据来源迁移。
- 不接受为 `query_workbench_page_read_model` 被页面真实消费。
- 不接受为 `WorkbenchSnapshot` / `load_workbench_snapshot` 废弃。
- 不接受为 UI 重做、布局重做或视觉验收。
- 不接受为真实 Tauri / 截图验收完成。
- 不接受为 R3 Level B、真实执行、多 agent 并行真实执行解锁。

## 2. Changed Files

- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/workbenchCoreTypes.ts`
- `tasks/2026-06-11-root-treatment-r4-a7-frontend-core-types-domain-extraction-v1.md`
- `evidence/2026-06-11-root-treatment-r4-a7-frontend-core-types-domain-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r4-a7-frontend-core-types-domain-extraction-v1-result.md`

## 3. Implementation Notes

迁移到 `workbenchCoreTypes.ts` 的类型：

- `FileCandidate`
- `HarnessCandidate`
- `HarnessEntrypoint`
- `HarnessResource`
- `ProjectRecord`
- `SessionRecord`
- `CodexTranscriptEvent`
- `CodexTranscript`
- `CodexTranscriptViewerBoundary`
- `SkillRecord`
- `PluginRecord`
- `TaskEntry`
- `Diagnostics`
- `IndexSummary`

`types.ts` 只新增 type-only import 和 re-export，不批量改调用方 import，避免 churn。

## 4. Verification

已运行并通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
  - `offline interaction tests passed: 14`
  - `r4 page selectors test passed`
- `npm run build`
  - 通过，保留既有 Vite chunk size warning。
- `node scripts/harness/workbench-shape-gate.js --mode check`
  - `Status: pass`
  - `Errors: 0`
  - `Warnings: 1`
  - 继承警告：`tauri_command_total_increased 97/96`
  - `types.ts: 4998/5149 (decreased)`
- `git diff --check`

未运行 Rust：

- 本切片只改前端类型文件和文档，未改 Rust、Tauri command、sidecar、DB migration 或 workflow state schema。

## 5. Boundary Confirmation

本轮没有：

- 改 UI、CSS、布局、视觉风格、导航入口或页面文案。
- 执行真实 `codex exec` / `codex exec resume`。
- 发送 prompt。
- 读写 `/Users/yoyi/.codex`。
- 读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript。
- 启动 Tauri / Browser / Chrome / Vite dev / 截图工具。
- 新增 Tauri command、sidecar、DB migration、store kind 或 workflow state 顶层结构。
- 改 `App.tsx` 数据加载路径。
- 把 `query_workbench_page_read_model` 接成页面真实数据源。
- 解冻 Stage L / Stage K / backlog 功能。

## 6. Review Request

复核线请只读审查：

- `types.ts` 是否真实下降且仍 re-export 迁移类型。
- 是否没有改运行逻辑、UI、CSS、Rust、Tauri command、sidecar、DB、workflow state schema。
- 是否没有 import churn 或破坏既有调用方。
- 验证是否覆盖 typecheck、离线交互、build 和 shape gate。
