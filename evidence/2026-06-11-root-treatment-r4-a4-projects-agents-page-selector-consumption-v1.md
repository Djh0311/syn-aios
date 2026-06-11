# Root Treatment R4-A4 Projects Agents Page Selector Consumption Evidence v1

日期：2026-06-11

结论：`accepted`

R4-A4 已实现 Projects / Agents 页面以最小 diff 消费 R4-A3 前端纯 selector。接受范围是页面使用 `pageSelectors.ts` 派生的轻量 read model 来承接首批重复统计和项目选择数据；不接受为页面真实数据来源迁移、R4 完成、`WorkbenchSnapshot` 废弃、`ProjectsView` / `AgentView` 拆分完成、UI 重做、真实 Tauri / 截图验收、R3 Level B 或多 agent 并行真实执行解锁。

## Implementation

- Task package：`tasks/2026-06-11-root-treatment-r4-a4-projects-agents-page-selector-consumption-v1.md`
- Planning baseline commit：`17dd1243e1b62e8ef86b9bb865d964a6cceae02f`
- Implementation commit：`58804f43cb3a666ddae66eecd1390def253c2ed2`
- Implementation hash backfill commit：`d04a28a`
- Review result：`STATUS: CLEAR`，无 P0 / P1 / P2。
- Checkpoint commit：待回填。
- Selector module：`prototypes/productized-desktop-shell/src/lib/pageSelectors.ts`
- Projects page：`prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- Agents page：`prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- Offline test：`prototypes/productized-desktop-shell/tests/r4-page-selectors.test.ts`

## Selector Consumption

Projects page：

- `ProjectGallery` now consumes `deriveProjectsPageReadModelFromParts`.
- Header project/session counts come from `ProjectsPageReadModel`.
- Project tile session/workflow/file/warning counts come from `ProjectListItemReadModel`.
- Existing visual structure, CSS classes and recent-update sorting are preserved.

Agents page：

- `AgentSessionCenter` now consumes `deriveAgentsPageReadModelFromParts`.
- Conversation project selector uses `AgentsPageReadModel.project_options`.
- Developer Codex control project selector also uses the same selector-derived project options.
- Existing conversation UI, K2/J1 control boundaries and developer details remain unchanged.

Selector compatibility：

- Original `deriveProjectsPageReadModel({ snapshot, workflowState })` remains available.
- Original `deriveAgentsPageReadModel({ snapshot })` remains available.
- New split-input wrappers allow pages to use existing props without passing full `WorkbenchSnapshot`.
- Source boundary still records `workbench_snapshot_active=true` and `page_ui_migrated=false`.

## Shape Gate

`node scripts/harness/workbench-shape-gate.js --mode check` passed.

分类：

- 0 errors / 1 warning。
- warning 为 `tauri_command_total_increased`：当前 97 / baseline 96。
- 该 warning 是 R4-A2 已接受的只读 command skeleton 遗留，不是 R4-A4 新增；R4-A4 没有新增 Tauri command。
- `AgentView.tsx` 行数从 3365 降到 3360。
- `ProjectsView.tsx` 行数保持 6069。
- `styles.css`、`types.ts`、`offline-permission-dialog.test.tsx`、Rust 文件均未增长。
- 未新增 sidecar JSON kind。

## Verification

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，输出：
  - `offline interaction tests passed: 14`
  - `r4 page read model settings test passed`
  - `r4 page read model query contract test passed`
  - `r4 page selectors test passed`
- `node scripts/harness/workbench-shape-gate.js --mode check`：通过，0 errors / 1 warning。
- `git diff --check`：通过。
- `npm run build`：通过，仅保留既有 Vite chunk-size warning。

Rust 未运行说明：

- R4-A4 未改 Rust 后端、Tauri command、Rust type、DB、sidecar 或 runner。
- 本轮验证重点为 TS typecheck、离线 selector 测试、shape gate 和 build。

## Review

- 复核线：`019eb51c-61fe-7fc3-8973-b22a4ce58911`
- 状态：`STATUS: CLEAR`
- P0：无。
- P1：无。
- P2：无。
- 复核结论：R4-A4 符合任务包范围；Projects / Agents 页面已消费 R4-A3 selectors；split-input wrappers 保持 snapshot wrappers 兼容；未误改视觉、CSS、布局、Tauri command、sidecar、DB、真实执行路径或敏感读取路径；未冒领 R4 完成、页面真实数据来源迁移、`query_workbench_page_read_model` 被真实消费、`WorkbenchSnapshot` 废弃、UI 重做、R3 Level B 或多 agent 并行真实执行。

## Scans

误导文案扫描：

- `R4 完成`、`页面真实数据来源已迁移`、`WorkbenchSnapshot 已废弃`、`UI 已重做` 等命中均在 R4-A4 任务包禁止声明内。
- R4-A4 产品代码没有新增这些冒领文案。

真实执行 / 敏感路径扫描：

- `codex exec`、`/Users/yoyi/.codex`、`prompt_body`、`secret`、`token`、`provider credential` 等命中来自既有 Projects / Agents 执行边界文案、既有 K2/J1 control code、任务包禁止条款或 R4 selector 测试黑名单。
- R4-A4 新增 selector/page 接线没有新增真实 `codex exec` / `codex exec resume` 调用。
- R4-A4 新增 selector/page 接线没有新增 `/Users/yoyi/.codex`、secret、token、credential 或 full transcript 读取路径。

## Boundary

- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript。
- 未新增 Tauri command。
- 未新增 sidecar JSON。
- 未迁移页面真实数据来源。
- 未废弃 `WorkbenchSnapshot` 或 `load_workbench_snapshot`。
- 未改 UI 视觉风格 / 布局 / CSS。
- 未启动 Tauri / Browser / Chrome / Vite dev / 截图工具。

## Deferred

- `ProjectsView.tsx` / `AgentView.tsx` 仍是大文件，尚未完成组件拆分。
- 页面仍未消费 `query_workbench_page_read_model` Tauri command。
- 页面真实数据来源仍是既有 `WorkbenchSnapshot` 拆分 props。
- R4 仍未完成。
- R3 Level B 仍未执行。
- Stage L / L1-L6 继续 `deferred_during_root_treatment`。
