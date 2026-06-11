# Root Treatment R4-A3 Projects Agents Selector Domain Split Handoff v1

日期：2026-06-11

结论：R4-A3 已完成并通过复核线 `STATUS: CLEAR`。

## 做了什么

- 新增 `pageSelectors.ts`，从现有 `WorkbenchSnapshot` / `WorkflowStateSnapshot` 派生 Projects / Agents 首批轻量 page read model。
- 新增 `deriveProjectsPageReadModel`：
  - 项目列表。
  - 项目 / 会话 / workflow summary 计数。
  - evidence / handoff / authority 文件计数。
  - 用户摘要和只读 source boundary。
- 新增 `deriveAgentsPageReadModel`：
  - 项目选择摘要。
  - 可读 / 缺回放 / 已归档会话计数。
  - adapter / operation / provider boundary 计数。
  - conversation-first 和 developer details collapsed 标志。
- 新增 `r4-page-selectors.test.ts` 并接入 offline runner。

Implementation commit：`c7ced1abcbf6c33a9cf8271a2450850cfdb5b491`
Evidence/docs commit：`8ba90f6dcbea1d3dc94d4d226d5aec723ac3f069`
Checkpoint sync commit：`275527a0e2a2aa309cd17207e159363976b17167`

## 关键文件

- `prototypes/productized-desktop-shell/src/lib/pageSelectors.ts`
- `prototypes/productized-desktop-shell/tests/r4-page-selectors.test.ts`
- `prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs`
- `tasks/2026-06-11-root-treatment-r4-a3-projects-agents-selector-domain-split-v1.md`
- `evidence/2026-06-11-root-treatment-r4-a3-projects-agents-selector-domain-split-v1.md`

## 验证

- `node scripts/harness/workbench-shape-gate.js --mode check`：通过，0 errors / 1 warning。
- shape warning：`tauri_command_total_increased` 当前 97 / baseline 96；这是 R4-A2 已接受的只读 command skeleton 遗留，R4-A3 没有新增 command。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 14`，包含 R4-A3 selector 测试。
- `npm run build`：通过，仅既有 Vite chunk-size warning。
- `git diff --check`：通过。

## 边界

- selector 只是前端纯函数，当前没有被 `ProjectsView.tsx` / `AgentView.tsx` 消费。
- `WorkbenchSnapshot` / `load_workbench_snapshot` 仍是当前页面主数据来源。
- 没有新增 Tauri command、sidecar、DB migration、production read-cut、stop-write 或真实 Codex runner 改动。
- 未执行真实 Codex，未读写 `/Users/yoyi/.codex`，未读取 secret/token/credential/full transcript。
- 未改视觉风格或布局，未启动真实 Tauri / 截图验收。

## 不能声明

- 不能声明 R4 完成。
- 不能声明 Projects / Agents 页面已迁移到新 selector 或新 command。
- 不能声明 `WorkbenchSnapshot` 已废弃。
- 不能声明 `ProjectsView` / `AgentView` 已拆分完成。
- 不能声明 UI 已重做或视觉已验收。
- 不能声明 R3 Level B 已执行。
- 不能声明多 agent 并行真实执行已解锁。

## 复核结果

- 复核线：`019eb51c-61fe-7fc3-8973-b22a4ce58911`
- 结论：`STATUS: CLEAR`
- P0/P1/P2：无。
- 确认 R4-A3 符合任务包范围，新 selector 保持 read-only / pure function，没有冒领页面迁移、`WorkbenchSnapshot` 废弃、真实执行或 UI 重做。

## 下一步建议

下一步建议 R4-A4：让 Projects / Agents 页面以最小 diff 消费 selectors，但仍不改视觉、不迁移真实数据来源、不废弃 `WorkbenchSnapshot`。
