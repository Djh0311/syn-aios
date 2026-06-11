# Root Treatment / R4-A10 styles.css Ink Style Asset Inventory v1

日期：2026-06-11

状态：已完成并通过复核线 `STATUS: CLEAR`。本文是 Root Treatment / Stage R 的 R4-A10 任务包；R4-A9 已完成并通过复核线 `STATUS: CLEAR`。R4-A10 只接受为 `styles.css` 现有水墨风格资产清单和后续 CSS 治理依据完成；不接受为 R4 完成、UI 视觉重做、CSS 源码拆分、布局重做、移动端适配、Xuanji / Mobbin / inkwash 外部风格复刻、真实 Tauri / 截图验收完成、R3 Level B 或 backlog 功能解冻。

Planning baseline commit：`cf909f8eef11d71331d5b5f71ec1dba347134f95`

Implementation commit：`1201306c38a36799548e5be454c9a16c3a66a742`。

Review result：`STATUS: CLEAR`，无 P0 / P1 / P2。

Checkpoint commit：待回填。

## 0. 全局主管理解

已知事实：

- 官方计划 R4-5 是 `styles.css` 风格资产提炼，产物是 ink style tokens / docs，验收目标是水墨颜色、字体、质感规则明确。
- `prototypes/productized-desktop-shell/src/styles.css` 当前约 8,464 行，已经超过普通维护阈值，但 R4-A10 不是 CSS 拆文件任务。
- 当前 UI 边界仍要求桌面 Tauri 工作台，不做手机端 UI，不做移动端适配。
- 用户明确要求不要改风格；Xuanji / Mobbin / inkwash 只能学习信息层级或布局表达，不允许复制源码、命名、图标或视觉资产。

核心判断：

```text
R4-A10 先把当前 styles.css 的风格资产、token 分层、桌面壳规则、重复硬编码和后续治理边界盘清楚，形成下一步 CSS 拆分和 UI Shell 校准的依据；本轮不改 CSS。
```

## 1. Execution Mode

Execution Mode：Supervisor-led documentation inventory with read-only review。

Multi-Agent Policy：

- 主管线负责任务包、资产清单、验证、证据、handoff 和 checkpoint。
- 复核线复用既有线程，只读审查，不改代码。
- 本切片是文档型治理任务，不新增开发线程，避免上下文维护成本。

## 2. Read / Write Scope

Allowed read paths：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/2026-06-08-workbench-ui-information-architecture-and-developer-settings-plan-v1.md`
- `docs/plans/2026-06-08-workbench-ui-redesign-brief-for-external-ai-v1.md`
- `docs/design/2026-06-08-workbench-ui-style-moodboard-v1.svg`
- `prototypes/productized-desktop-shell/src/styles.css`
- shape gate output and package scripts as needed.

Allowed write paths：

- `tasks/2026-06-11-root-treatment-r4-a10-styles-css-ink-style-asset-inventory-v1.md`
- `docs/design/2026-06-11-root-treatment-r4-a10-ink-style-assets-v1.md`
- `evidence/2026-06-11-root-treatment-r4-a10-styles-css-ink-style-asset-inventory-v1.md`
- `handoffs/2026-06-11-root-treatment-r4-a10-styles-css-ink-style-asset-inventory-v1-result.md`
- checkpoint 入口文档只在复核 CLEAR 后同步。

## 3. Forbidden

R4-A10 禁止：

- 不改 `styles.css` 源码。
- 不改 UI 视觉风格、布局、导航、文案、组件结构或页面数据来源。
- 不做 CSS 文件拆分，不新增 CSS module，不引入 design token build 工具。
- 不做手机端 UI、移动端导航、mobile-first responsive 任务。
- 不复制 Xuanji / Mobbin / inkwash 外部项目源码、命名、图标或视觉资产。
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

1. 只读盘点 `styles.css` 的 token 定义、颜色、字体、间距、圆角、阴影、动效、桌面壳尺寸和断点规则。
2. 新增 `docs/design/2026-06-11-root-treatment-r4-a10-ink-style-assets-v1.md`，记录现有风格资产、可复用 token、重复硬编码、风险和后续治理建议。
3. 写 evidence / handoff，记录检查命令、指标、边界和不能声明项。
4. 运行 shape gate / diff check；如只改文档，不跑 npm / cargo 并说明原因。
5. 复核线只读审查资产清单是否足够支撑后续 CSS 治理，以及是否没有实际改 UI / CSS。

## 5. Acceptance Criteria

R4-A10 可接受条件：

- 明确列出 `styles.css` 当前行数、变量数量、硬编码颜色和主要断点。
- 明确区分旧基础 token、水墨壳 token、Source parity / UI redesign 局部 token 和 Stage K 桌面对话 workspace 补丁。
- 明确颜色、字体、质感、间距、圆角、阴影、动效和桌面壳尺寸的当前可复用资产。
- 明确后续 CSS 治理建议和禁止直接重做 UI 的边界。
- 不修改产品源码、CSS、Rust、Tauri command、sidecar、DB、workflow state schema 或真实执行路径。
- shape gate 和 `git diff --check` 通过。

## 6. Verification Plan

必须运行：

- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

可选运行：

- `npm run typecheck`

本切片只改文档，不默认运行 `npm run test:offline-interaction`、`npm run build` 或 Rust 测试；如未运行必须在 evidence 中说明原因。

## 7. Review Plan

实现后复用既有复核线做只读审查。

复核重点：

- 是否没有修改 `styles.css` 或任何产品代码。
- 风格资产清单是否覆盖颜色、字体、质感、间距、阴影、动效和壳层尺寸。
- 是否明确指出当前 CSS 的债务和后续治理路线。
- 是否没有把 R4-A10 冒充成 UI 重做、CSS 拆分或 R4 完成。

## 8. 禁止声明

R4-A10 禁止声明：

- R4 完成。
- UI 视觉已重做或已验收。
- `styles.css` 已拆分或 CSS 债务已解决。
- 页面真实数据来源已迁移。
- `query_workbench_page_read_model` 已被页面真实消费。
- `WorkbenchSnapshot` 已废弃。
- 真实 Tauri / 截图验收完成。
- R3 Level B 已执行。
- Stage L / Stage K / backlog 功能已解冻。
