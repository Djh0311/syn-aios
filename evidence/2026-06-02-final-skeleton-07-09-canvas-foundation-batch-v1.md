# Evidence：final-skeleton-07 到 final-skeleton-09 画布基础批次 v1

日期：2026-06-02

## 结论

本批次已完成画布基础批次的前端最小实现：

- `final-skeleton-07`：新增项目画布组件状态样例。
- `final-skeleton-08`：新增项目工作流画布读模型，并用 React Flow / xyflow 渲染最小只读项目画布。
- `final-skeleton-09`：项目工作流页主区域收敛为画布，任务包、账本、状态机、子汇报、权限请求、黑板候选和审计摘要进入右侧节点详情 / 控制面板。

本批次不接受为完整低代码编辑器、通用节点自动化平台、真实工作流状态写入、真实 Codex 执行或 MCP canvas run。

## 改动文件

- `prototypes/productized-desktop-shell/src/lib/projectCanvas.ts`
  - 新增 `ProjectWorkflowCanvasReadModel` 纯前端读模型。
  - 新增 `deriveProjectWorkflowCanvasReadModel`，从现有 `ProjectWorkflowSummary` / `ProjectBlackboard` / 当前工作项派生画布节点、边、详情和状态。
  - 新增 `projectCanvasStateExamples`，覆盖空画布、四角色、执行中、等待权限、失败、回收中、accepted。
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
  - 项目工作流页改为左侧 React Flow 画布、右侧节点详情和控制面板。
  - React Flow 只消费派生读模型，不写回 workflow state。
  - 保留原有工作项控制卡，放入右侧面板继续承接确认动作。
- `prototypes/productized-desktop-shell/src/styles.css`
  - 新增项目画布、静态画布、React Flow 节点、右侧详情和状态样例样式。
  - 补窄屏布局，画布和详情上下排列。
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
  - 新增项目画布读模型和组件状态样例断言。
  - 更新项目工作流页断言为“主画布 + 右侧详情”方向。
  - 原有绑定、派发、权限确认、总指导回收等确认动作继续用工作项控制卡测试。

## 验证

已通过：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
```

结果：

- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过，输出 `offline interaction tests passed: 4`。
- `npm run build` 通过。

构建提醒：

- Vite 提示主 chunk 超过 500 kB。本轮不改拆包策略，因为任务范围是画布基础实现，不是构建优化。

## 截图说明

本轮未产出截图。

原因：

- 当前可用工具没有暴露 in-app browser 截图能力。
- 项目内没有 Playwright / Puppeteer 依赖，`playwright` 命令也不在 PATH。
- 不启动真实 Tauri 窗口，因为这轮红线是不读写 `/Users/yoyi/.codex`，真实 Tauri 启动可能读取本地 Codex 索引。

替代验证：

- 使用离线 React 静态渲染测试验证项目画布、节点详情、状态样例和原有确认动作。
- `npm run build` 验证 React Flow 依赖和 CSS 能进入生产构建。

## 边界确认

本轮没有：

- 写真实 `workflow state`。
- 修改工作流状态机。
- 启动 MCP canvas run。
- 执行真实 Codex。
- 读写 `/Users/yoyi/.codex`。
- 新增通用低代码编辑器语义。
- 把独立 `CanvasView` 当作项目事实源。

## 剩余风险

- 没有截图证据，真实视觉布局仍建议下一轮在允许的 Tauri / 浏览器验收线里补。
- React Flow 已进入项目工作流页，主 chunk 体积提示需要后续单独考虑拆包。
- 右侧控制卡仍承载较多内部动作，已经从主画布移走，但还没有按节点类型细分成更小的详情组件。
