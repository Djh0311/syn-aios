# Root Treatment R4-A3 Projects Agents Selector Domain Split Evidence v1

日期：2026-06-11

结论：`accepted`

R4-A3 已实现 Projects / Agents 首批前端 selector 分域。接受范围是新增前端纯 selector、轻量 page read model 类型和离线测试；不接受为页面真实数据来源迁移、R4 完成、`WorkbenchSnapshot` 废弃、`ProjectsView` / `AgentView` 拆分完成、UI 重做、真实 Tauri / 截图验收、R3 Level B 或多 agent 并行真实执行解锁。

## Review Gate

- R4-A2 复核 STATUS：`CLEAR`。
- R4-A2 P0/P1/P2：无阻断。
- R4-A3 进入实现结论：允许。

## Implementation

- Task package：`tasks/2026-06-11-root-treatment-r4-a3-projects-agents-selector-domain-split-v1.md`
- Planning baseline commit：`882b079b42b7f152f38820b40460ef3c652fad7c`
- Implementation commit：`c7ced1abcbf6c33a9cf8271a2450850cfdb5b491`
- Evidence/docs commit：`8ba90f6dcbea1d3dc94d4d226d5aec723ac3f069`
- Review result：`STATUS: CLEAR`
- Selector module：`prototypes/productized-desktop-shell/src/lib/pageSelectors.ts`
- Offline test：`prototypes/productized-desktop-shell/tests/r4-page-selectors.test.ts`
- Offline runner：`prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs`

## Selector Contract

Projects selector：

- `deriveProjectsPageReadModel({ snapshot, workflowState })`
- 输出 `ProjectsPageReadModel`。
- 包含项目列表、项目数、活跃项目数、session count、workflow summary count、evidence / handoff / authority count、warning count 和用户摘要。
- 不包含 raw transcript、full prompt、secret、token 或执行命令。

Agents selector：

- `deriveAgentsPageReadModel({ snapshot })`
- 输出 `AgentsPageReadModel`。
- 包含项目选择摘要、会话可读取 / 缺回放 / 已归档计数、adapter count、planned adapter count、operation boundary count、provider boundary count 和 conversation-first 标志。
- 明确 `developer_details_collapsed=true`，不把开发者边界数据作为普通首屏。

共同 source boundary：

- `generated_from="workbench_snapshot_selector"`
- `workbench_snapshot_active=true`
- `page_ui_migrated=false`
- `tauri_command_consumed=false`
- `writes_stores=false`
- warnings 包含 `do_not_claim_workbench_snapshot_deprecated`

## Shape Gate

`node scripts/harness/workbench-shape-gate.js --mode check` 通过。

分类：

- 0 errors / 1 warning。
- warning 为 `tauri_command_total_increased`：当前 97 / baseline 96。
- 该 warning 是 R4-A2 已接受的只读 command skeleton 遗留，不是 R4-A3 新增；R4-A3 没有新增 Tauri command。
- `ProjectsView.tsx`、`AgentView.tsx`、`types.ts`、`styles.css`、`offline-permission-dialog.test.tsx` 均未增长。
- 新增 `pageSelectors.ts` 275 行，低于 TS 2,000 行上限。
- 新增 `r4-page-selectors.test.ts` 268 行，低于 TS 2,000 行上限。
- 未新增 sidecar JSON kind。

## Verification

- `node scripts/harness/workbench-shape-gate.js --mode check`：通过，0 errors / 1 warning。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，输出：
  - `offline interaction tests passed: 14`
  - `r4 page read model settings test passed`
  - `r4 page read model query contract test passed`
  - `r4 page selectors test passed`
- `npm run build`：通过，仅保留既有 Vite chunk-size warning。
- `git diff --check`：通过。

Rust 未运行说明：

- R4-A3 未改 Rust 后端、Tauri command、Rust type、DB、sidecar 或 runner。
- 本轮验证重点为 shape gate、TS typecheck、离线 selector 测试和 build。

## Review Result

- 复核线：`019eb51c-61fe-7fc3-8973-b22a4ce58911`
- 结论：`STATUS: CLEAR`
- P0/P1/P2：无。
- 复核确认 R4-A3 只新增前端纯 selector 和测试，没有新增 Tauri command、sidecar、DB migration、真实执行路径或 UI 视觉改动。

## Scans

误导文案 / 真实执行 / 敏感词扫描：

- 命中 `R4 完成`、`UI 已重做`、`codex exec`、`.codex`、secret / token / credential / full transcript 的位置均为任务包禁止条款、边界声明或测试黑名单。
- 测试中的 `credential_status` / `requires_credential` 是 provider availability fixture 字段，不是 secret 读取。
- R4-A3 产品代码未新增真实 `codex exec` / `codex exec resume` 路径。
- R4-A3 产品代码未新增 `/Users/yoyi/.codex`、secret、token、credential 或 full transcript 读取路径。

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

- `ProjectsView.tsx` / `AgentView.tsx` 尚未消费 R4-A3 selectors。
- 页面真实数据来源仍是既有 `WorkbenchSnapshot`。
- R4 仍未完成。
- R3 Level B 仍未执行。
- Stage L / L1-L6 继续 `deferred_during_root_treatment`。
