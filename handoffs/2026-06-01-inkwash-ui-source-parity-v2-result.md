# 水墨源稿对齐工作台 UI v2 handoff

## 当前状态

任务已执行到可构建状态。工作台 UI 已按 `inkwash-full.html` 的水墨外壳方向重做，保留真实工作台的确认弹层、项目 / 会话 / 工作流 / 任务包入口和 reload 入口。

## 当前权威入口

- 任务包：`/Users/yoyi/workspace/product-line/tasks/2026-06-01-inkwash-ui-source-parity-v2.md`
- evidence：`/Users/yoyi/workspace/product-line/evidence/2026-06-01-inkwash-ui-source-parity-v2.md`
- 目标工程：`/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell`
- 源稿：`/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/ui-prototype/inkwash-full.html`

## 文件改动

- `src/App.tsx`：扩展左侧导航；顶部栏按源稿重排；新增右侧详情展开面板；新增源稿式占位入口。
- `src/views/HomeView.tsx`：首页星图继续保留，工作流数量改接 `workflowState`，智能体近期项优先用真实 session。
- `src/views/ProjectsView.tsx`：无项目索引时显示源稿式项目空态。
- `src/views/CanvasView.tsx`：普通浏览器 / 非 Tauri 失败态改为源稿式画布空态。
- `src/styles.css`：补右侧展开面板、项目页、画布、宽屏布局和源稿式占位样式。

## 验证结果

- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过。
- `npm run build` 通过；保留 Vite chunk 体积警告。

## 截图路径

- 宽屏首页：`/Users/yoyi/workspace/product-line/evidence/inkwash-ui-source-parity-v2/wide-target-home.png`
- 宽屏项目页：`/Users/yoyi/workspace/product-line/evidence/inkwash-ui-source-parity-v2/wide-target-project.png`
- 宽屏工作流页：`/Users/yoyi/workspace/product-line/evidence/inkwash-ui-source-parity-v2/wide-target-workflow.png`
- 宽屏右侧通知展开：`/Users/yoyi/workspace/product-line/evidence/inkwash-ui-source-parity-v2/wide-target-right-notifications.png`

## 没有做的事

- 没执行 `codex exec resume`。
- 没写 `/Users/yoyi/.codex`。
- 没写真实 workflow state。
- 没读敏感文件或完整 transcript。
- 没做 Tauri 真窗口截图。
- 没拿到源稿截图；Browser 工具拒绝 `file://` 源稿访问。

## 下一步建议

- 用 Tauri 真窗口验证真实索引数据态，重点看项目列表、项目内工作流、右侧详情长内容是否爆版。
- 如果需要截图级对齐，先把源稿用允许的方式托管到本地 HTTP 或由用户提供源稿截图，再做逐屏对比。
- 后续可把 `styles.css` 里的水墨壳样式拆成更清楚的分段，降低继续迭代的成本。
