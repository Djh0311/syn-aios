# Root Treatment R4-A4 Projects Agents Page Selector Consumption Handoff v1

日期：2026-06-11

结论：R4-A4 implementation 已完成，状态为 `implemented_pending_review`。

## 做了什么

- `pageSelectors.ts` 新增 split-input selector wrappers：
  - `deriveProjectsPageReadModelFromParts`
  - `deriveAgentsPageReadModelFromParts`
- 保留 R4-A3 snapshot wrappers：
  - `deriveProjectsPageReadModel`
  - `deriveAgentsPageReadModel`
- `ProjectsView.tsx` 的 `ProjectGallery` 消费 Projects page selector，替换首屏统计和项目卡片计数的重复派生。
- `AgentView.tsx` 的对话项目选择和开发者 Codex 控制项目选择消费 Agents page selector。
- `r4-page-selectors.test.ts` 增加 split-input selector 和 snapshot wrapper 等价断言。

## 关键文件

- `prototypes/productized-desktop-shell/src/lib/pageSelectors.ts`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/tests/r4-page-selectors.test.ts`
- `tasks/2026-06-11-root-treatment-r4-a4-projects-agents-page-selector-consumption-v1.md`
- `evidence/2026-06-11-root-treatment-r4-a4-projects-agents-page-selector-consumption-v1.md`

## 验证

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 14`，包含 R4 selector 测试。
- `node scripts/harness/workbench-shape-gate.js --mode check`：通过，0 errors / 1 warning。
- shape warning：`tauri_command_total_increased` 当前 97 / baseline 96；这是 R4-A2 已接受的只读 command skeleton 遗留，R4-A4 没有新增 command。
- `git diff --check`：通过。
- `npm run build`：通过，仅既有 Vite chunk-size warning。

## 边界

- 页面仍使用既有 props 和 `WorkbenchSnapshot` 拆分数据，不消费 `query_workbench_page_read_model` 作为真实数据源。
- selector 仍是前端纯函数，不写 store、不发命令、不读敏感路径。
- 没有新增 Tauri command、sidecar、DB migration、production read-cut、stop-write 或真实 Codex runner 改动。
- 未执行真实 Codex，未读写 `/Users/yoyi/.codex`，未读取 secret/token/credential/full transcript。
- 未改视觉风格、布局或 CSS，未启动真实 Tauri / 浏览器 / 截图验收。

## 不能声明

- 不能声明 R4 完成。
- 不能声明页面真实数据来源已迁移。
- 不能声明 `query_workbench_page_read_model` 已被页面真实消费。
- 不能声明 `WorkbenchSnapshot` 已废弃。
- 不能声明 `ProjectsView` / `AgentView` 已拆分完成。
- 不能声明 UI 已重做或视觉已验收。
- 不能声明 R3 Level B 已执行。
- 不能声明多 agent 并行真实执行已解锁。

## 给复核线

请只读复核：

- R4-A4 是否符合任务包范围。
- Projects / Agents 页面是否真的消费 R4-A3 selectors。
- selector split-input wrappers 是否保持 snapshot wrappers 兼容。
- 是否误改视觉、CSS、布局、Tauri command、sidecar、DB、真实执行路径或敏感读取路径。
- shape gate warning 是否确为 R4-A2 遗留 command total warning，而非 R4-A4 新增。

建议输出单行结论：

```text
STATUS: CLEAR
```

或：

```text
STATUS: BLOCKED
```

如有问题，请按 P0/P1/P2 分级，并给出文件 / 行号。

## 下一步建议

若复核线 CLEAR，主管线可 checkpoint 同步入口文档。下一步建议 R4-A5 二选一：

- 继续补 Running / Memory 首批 selector 分域，再做页面消费。
- 开始更细粒度拆 `ProjectsView.tsx` / `AgentView.tsx` 子组件，但仍不改视觉。
