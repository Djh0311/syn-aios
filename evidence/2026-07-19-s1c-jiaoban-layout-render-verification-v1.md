# S1C 交办三栏重排·浏览器渲染与四闸证据 v1

日期：2026-07-19  
任务包：`tasks/2026-07-19-s1c-jiaoban-layout-conversation-left-history-as-proposal-index-package-v1.md`  
修宪：`decisions/2026-07-19-interaction-canon-amendment-3-conversation-left-proposal-index-v1.md`

## 结论

S1C 的代码、离线语义、浏览器实渲量尺和四闸已收口：桌面为「对话左｜历届方案索引中｜方案/交货实体右」，577px 按同一阅读顺序纵排；页面壳无横纵滚动，溢出留在各自容器。点击历届方案只切右侧实体，左侧对话滚动值实测保持不变。尚未声称用户在完整 Tauri 壳中的最后一眼已通过。

## 浏览器实渲口径

直接在普通浏览器进入完整项目页会命中既有 `ensureTauriRuntime` 门槛，因此本次增加只用于验收的浏览器夹具：

- `tests/jiaoban-layout-visual-fixture.html`
- `tests/jiaoban-layout-visual-fixture.tsx`

夹具直接挂载生产组件 `JiaobanMergedLayout`、`JiaobanProposalIndex`、`JiaobanConversationStream`、`JiaobanConversationComposer`、`JiaobanAuthorizeState`、`JiaobanHistoryDetail`，并加载 App 的生产 CSS；只注入测试数据和隐藏量尺，不复制布局实现。浏览器控制台无产品运行时错误，唯一 error 为本地 Vite 的 `favicon.ico` 404。

## 1280 × 900

截图：`prototypes/productized-desktop-shell/output/playwright/s1c-jiaoban-layout/jiaoban-1280.png`（SHA-256 `e6d3c6740cfcd6a2cc5416d87808e639c77ee15643cd54550a7be38e745d35c4`）

- 页面：`1280 × 900`，`scrollWidth == clientWidth == 1280`，`scrollHeight == clientHeight == 900`。
- 布局容器：`1232 × 776`，无自身滚动。
- 左对话：`left=24, width=482, height=776`；内容高 `1565`，可内部滚动。
- 中索引：`left=520, width=240, height=776`；内容高 `927`，可内部滚动。
- 右实体：`left=774, width=482, height=776`。
- 顺序断言：`conversation.left < proposalIndex.left < canvas.left`，PASS。
- 页面无横滚、无纵滚：PASS。

## 577 × 900

截图：`prototypes/productized-desktop-shell/output/playwright/s1c-jiaoban-layout/jiaoban-577.png`（SHA-256 `500b7cc726f2dbc592aba2deacac30f939897b55baffca13a2cbf625c4266df1`）

- 页面：`577 × 900`，`scrollWidth == clientWidth == 577`，`scrollHeight == clientHeight == 900`。
- 布局容器：`529 × 776`，无自身滚动。
- 左对话：`top=96, height=299`；内容高 `1500`，可内部滚动。
- 中索引：`top=409, height=177`；滚动区 `175/190`，可内部滚动。
- 右实体：`top=600, height=272`；滚动区 `229/557`，可内部滚动。
- 顺序断言：`conversation.top < proposalIndex.top < canvas.top`，PASS。
- 页面无横滚、无纵滚：PASS。

## 点击隔离与诚实兜底

Playwright 先把左侧 `.project-jiaoban-main` 滚到 `scrollTop=320`，再真实点击唯一带「旧单·无方案记录」的索引行：

- 点击后左侧滚动：`320 → 320`，PASS。
- 选中 id：`proposal-legacy`。
- 右侧实体：`aria-label="历史单详情"`。
- 索引可见「旧单·无方案记录」：PASS。
- 未触发对话锚点或对话滚动副作用。

## 四闸与工程检查

从对应目录执行：

- `cargo test --offline --lib`：`1008 passed / 0 failed / 44 ignored`。
- `pnpm typecheck`：PASS。
- `pnpm test:offline-interaction`：PASS；runner 15 项，S1C 相关组为：
  - `history-and-board`：7 组通过；
  - `jiaoban-merged-layout`：6 组通过；
  - `jiaoban-conversation-center`：既有消息流、分组与 P3-A 断言通过。
- shape baseline：`13 errors / 5 warnings / 5 infos`，命令 PASS。
- shape check：同为 `13 / 5 / 5`，按历史债策略预期非零；零净增。
- `git diff --check`：PASS。
- `ProjectJiaobanPanel.tsx`：`+23/-31`，净减 8 行，水线零增长。

## 范围声明

- S1C 未修改 Rust、Tauri command、sidecar、数据源、九态判据、批准动作或方案/交货卡内部。
- 工作树原有 S1B Rust 与并行脏项均保留，未 stage、未 commit、未清理。
- 本证据证明生产组件在浏览器夹具中的实渲与自动闸；完整 Tauri 壳中的用户最后一眼仍由用户确认，不能从本证据外推为已过。
