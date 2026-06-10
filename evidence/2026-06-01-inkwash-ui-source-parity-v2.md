# 水墨源稿对齐工作台 UI v2 evidence

时间：2026-06-01 16:15:31 CST

## 做了什么

- 将真实工作台外壳改为源稿式顶栏、左侧图标栏、中央宣纸舞台、底部 dock。
- 左侧导航补齐：首页、项目、想法箱、建议方案、工作流、智能体、知识库、记忆、技能、harness、工具、模型 / 凭据、设置。
- 通知、待办、审计、运行中工作流放到右侧图标栏；点击后展开水墨详情面板。
- 顶部品牌改为“本地 AI 工作台”，保留“刷 / 重新读取”按钮，并继续调用 `reload()`。
- 首页继续使用星图式五节点，并将当前工作流数量改为来自 `workflowState`。
- 项目页空态改为源稿式纸面结构，不在无 Tauri 数据时假装有项目。
- 工作流普通浏览器失败态改为水墨画布空态，不伪造画布数据。

## 改动文件

- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/App.tsx`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/views/HomeView.tsx`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/views/CanvasView.tsx`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/styles.css`

## 截图

源稿截图：

- 未完成。原因：Browser 工具拒绝访问 `file:///Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/ui-prototype/inkwash-full.html`，提示 URL 被安全策略阻止。没有绕过该策略。

目标基准截图：

- `/Users/yoyi/workspace/product-line/evidence/inkwash-ui-source-parity-v2/baseline-target-before-home.png`

目标最终截图：

- `/Users/yoyi/workspace/product-line/evidence/inkwash-ui-source-parity-v2/wide-target-home.png`
- `/Users/yoyi/workspace/product-line/evidence/inkwash-ui-source-parity-v2/wide-target-project.png`
- `/Users/yoyi/workspace/product-line/evidence/inkwash-ui-source-parity-v2/wide-target-workflow.png`
- `/Users/yoyi/workspace/product-line/evidence/inkwash-ui-source-parity-v2/wide-target-right-notifications.png`

补充普通视口截图：

- `/Users/yoyi/workspace/product-line/evidence/inkwash-ui-source-parity-v2/final-target-home.png`
- `/Users/yoyi/workspace/product-line/evidence/inkwash-ui-source-parity-v2/final-target-project.png`
- `/Users/yoyi/workspace/product-line/evidence/inkwash-ui-source-parity-v2/final-target-workflow.png`
- `/Users/yoyi/workspace/product-line/evidence/inkwash-ui-source-parity-v2/final-target-right-notifications.png`

## 验证命令

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，输出 `offline interaction tests passed: 2`。
- `npm run build`：通过；Vite 输出 chunk 大于 500 kB 的体积警告。

## 明确未做

- 没有执行真实 `codex exec resume`。
- 没有写 `/Users/yoyi/.codex`。
- 没有写真实 workflow state。
- 没有读取敏感文件、`.env`、auth、token、密钥。
- 没有读取完整 transcript / rollout JSONL 正文。
- 没有联网安装依赖。
- 没有做 Tauri 真窗口截图。

## 与源稿仍不一致

- 右侧栏没有改成源稿 320px 常驻栏，而是保留图标窄栏 + 展开详情；这是用户明确允许的偏离。
- 源稿左侧通知、待办、审计没有保留在左侧；已移到右侧；这是用户明确允许的偏离。
- 顶部新增“刷 / 重新读取”；这是用户明确要求保留的偏离。
- 想法箱、建议方案、知识库、记忆、工具、模型 / 凭据、设置目前是源稿式只读入口或占位，真实数据不足处明确标注，未伪造数据。
- 普通浏览器没有 Tauri 数据桥，所以截图里的项目、工作流、会话数量为 0；真实数据态仍需 Tauri 窗口验证。
- 源稿截图基准缺失，不能宣称截图级完全对齐。

## 风险

- `src/styles.css` 已较大，继续追加样式会增加维护成本。
- 真实 Tauri 数据态下项目页和工作流页可能暴露普通浏览器截图看不到的布局问题。
- `vite build` 的 JS chunk 超过 500 kB，当前不是失败，但后续可考虑按视图拆分。
