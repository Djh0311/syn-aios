# Root Treatment / R4-A10 styles.css Ink Style Asset Inventory v1 Evidence

日期：2026-06-11

状态：已完成并通过复核线 `STATUS: CLEAR`。

任务包：`tasks/2026-06-11-root-treatment-r4-a10-styles-css-ink-style-asset-inventory-v1.md`

设计资产清单：`docs/design/2026-06-11-root-treatment-r4-a10-ink-style-assets-v1.md`

Planning baseline commit：`cf909f8eef11d71331d5b5f71ec1dba347134f95`

Implementation commit：`1201306c38a36799548e5be454c9a16c3a66a742`。

Review result：`STATUS: CLEAR`，无 P0 / P1 / P2。

Checkpoint commit：`1bff08276f284a3eb4e38ff504379bf1fa4f81a2`。

## 1. Scope

R4-A10 只做 `styles.css` 风格资产清单和后续治理建议。

本轮接受范围：

- 只读盘点 `styles.css` 的 token、颜色、字体、质感、间距、圆角、阴影、动效、桌面壳尺寸和断点。
- 新增 `docs/design/2026-06-11-root-treatment-r4-a10-ink-style-assets-v1.md`。
- 写任务包、evidence 和 handoff。

本轮不接受范围：

- 不接受为 R4 完成。
- 不接受为 UI 视觉重做、布局重做或视觉验收。
- 不接受为 CSS 源码拆分或 CSS 债务解决。
- 不接受为页面真实数据来源迁移。
- 不接受为真实 Tauri / 截图验收完成。
- 不接受为 R3 Level B、真实执行、多 agent 并行真实执行解锁。
- 不接受为 Stage L / Stage K / backlog 功能解冻。

## 2. Changed Files

- `tasks/2026-06-11-root-treatment-r4-a10-styles-css-ink-style-asset-inventory-v1.md`
- `docs/design/2026-06-11-root-treatment-r4-a10-ink-style-assets-v1.md`
- `evidence/2026-06-11-root-treatment-r4-a10-styles-css-ink-style-asset-inventory-v1.md`
- `handoffs/2026-06-11-root-treatment-r4-a10-styles-css-ink-style-asset-inventory-v1-result.md`

本轮没有修改：

- `prototypes/productized-desktop-shell/src/styles.css`
- 前端 TS / TSX 产品代码
- Rust / Tauri 后端
- workflow state / sidecar / DB schema

## 3. Read Evidence

只读参考：

- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/2026-06-08-workbench-ui-information-architecture-and-developer-settings-plan-v1.md`
- `docs/plans/2026-06-08-workbench-ui-redesign-brief-for-external-ai-v1.md`
- `prototypes/productized-desktop-shell/src/styles.css`

关键计划依据：

- R4-5 产物为 `styles.css` 风格资产提炼 / ink style tokens / docs。
- R4 不做布局重做，不做无限画布，不做 UI 视觉反馈 MCP 工具，不复制 Xuanji 源码、命名、图标或视觉资产。
- UI 只按桌面 Tauri 工作台设计和验收，不做手机端 UI。

## 4. Shape / Inventory Metrics

只读统计：

- `wc -l prototypes/productized-desktop-shell/src/styles.css`：8,464 行。
- CSS 变量定义命中：107。
- hex 颜色命中：100。
- `rgba()` / `rgb()` 命中：234。
- `@media` 命中：9。
- `@keyframes` 命中：1。
- 顶层 class 规则命中：1,345。

主要 token 段：

- 第 1 行：早期基础 `:root`。
- 第 3205 行：`Inkwash shell replacement` 水墨壳 `:root`。
- 第 6138 行：`Source parity v2 refinements` 右侧展开尺寸。
- 第 7639 行：`UI redesign checkpoint` 桌面 `--ui-*` token。
- 第 7855 行：`Step 1` 桌面壳尺寸 token。

## 5. Implementation Notes

新增设计资产清单覆盖：

- token 分层。
- 基础纸面色、墨色、线条、语义色。
- 字体资产。
- 间距、圆角、阴影。
- 桌面壳尺寸和断点。
- 动效和状态反馈。
- 当前 CSS 债务：token 重名、硬编码颜色、补丁段落叠加。
- 后续 CSS 治理建议：按 token / shell / common / home / agent / projects / memory-knowledge / developer 分区。

本轮没有做任何 CSS 等价迁移或视觉修改。

## 6. Verification

已运行并通过：

- `node scripts/harness/workbench-shape-gate.js --mode check`
  - `Status: pass`
  - `Errors: 0`
  - `Warnings: 1`
  - 继承 warning：`tauri_command_total_increased 97/96`
  - `styles.css: 8464/8464 (same)`
- `git diff --check`
  - 在 `git add --intent-to-add` 让 4 个新文档进入 diff 可见状态后运行。
  - 无输出，检查通过。

未运行 npm / cargo：

- 本切片只新增 Markdown 文档，不修改产品代码、CSS、Rust、Tauri command、sidecar、DB migration 或 workflow state schema。

## 7. Boundary Confirmation

本轮没有：

- 修改 `styles.css`。
- 改 UI 视觉风格、布局、导航入口、组件结构或页面文案。
- 执行真实 `codex exec` / `codex exec resume`。
- 发送 prompt。
- 读写 `/Users/yoyi/.codex`。
- 读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript。
- 启动 Tauri / Browser / Chrome / Vite dev / 截图工具。
- 新增 Tauri command、sidecar、DB migration、store kind 或 workflow state 顶层结构。
- 改 `App.tsx` 数据加载路径。
- 把 `query_workbench_page_read_model` 接成页面真实数据源。
- 解冻 Stage L / Stage K / backlog 功能。

过程偏差：

- 主管自审时有一条 `rg` 命令把 Markdown 反引号放入 shell 双引号，shell 尝试执行 `styles.css` 并返回 `command not found`。
- 该偏差没有产生文件改动，没有执行真实 Codex，没有读写 `/Users/yoyi/.codex`，没有触碰 secret / token / transcript。
- 后续同类搜索应使用单引号或 `rg -n -- 'pattern' path`。

## 8. Review Result

复核线回交：

- `STATUS: CLEAR`
- P0：无
- P1：无
- P2：无

复核证据摘要：

- 复核线确认任务包与官方计划 R4-5 对齐，目标是 `styles.css` 风格资产提炼 / ink style tokens / docs。
- 复核线确认设计资产清单覆盖 token 分层、颜色、字体、间距、圆角、阴影、桌面壳尺寸、断点、动效、债务与拆分建议。
- 复核线确认当前 diff 只包含 4 个 Markdown 文档，没有 `styles.css`、产品代码、Rust、Tauri command、sidecar、DB、workflow state schema 改动。
- 复核线确认验证口径准确：shape gate pass，继承 warning `tauri_command_total_increased 97/96`；`git diff --check` pass；未跑 npm/cargo 的原因是本轮只改 Markdown 文档。
- 复核线确认不能声明项收敛，没有冒领 UI 重做、CSS 拆分、视觉验收、R4 完成、页面真实数据源迁移、R3 Level B 或 backlog 解冻。
