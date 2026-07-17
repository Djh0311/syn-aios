# P1-C 中栏对话中心实渲量尺 v1

## 结论

- 已在 Chromium `1280 × 820` 视口中，用 `ProjectWorkspaceShell` 的真实组件树和项目 CSS 完成 DOM 实渲。
- `conversation` 态的三栏实测为 `218px / 500px / 486px`，栏间距 `14px`；中栏满足真机目标 `500px`。
- `say` 态的三栏实测为 `218px / 806px / 180px`，中栏正文 `.project-jiaoban-col` 为 `720px`，左右各留 `41px`；满足 `say` 态正文最大宽 `720px`。
- 右区仍是原有「工序图 / 治理保证 / 怎么跑」三视图；本次没有改其组件或挂点。

## 实际入层

本次浏览器证据的实际组件入层为：

```text
ProjectWorkspaceShell
  -> ProjectJiaobanPanelBrowser
  -> renderLayout
  -> JiaobanMergedLayout
  -> .project-jiaoban-main
  -> .project-jiaoban-col
  -> JiaobanConversationStream
```

`ProjectJiaobanPanelBrowser` 是 `ProjectJiaobanPanel` 在存在 `window` 时调用的浏览器分支。量尺页只负责装入固定离线夹具；页面中的布局、组件和 CSS 均来自产品源码。取证后，临时量尺入口已从最终 diff 删除。

## conversation 态 DOM 量测原文

```json
{
  "viewport": { "width": 1280, "height": 820 },
  "shell": { "width": 1280, "height": 820 },
  "workspace": { "x": 24, "y": 96, "width": 1232, "height": 696 },
  "gridTemplateColumns": "218px 500px 486px",
  "columnGap": "14px",
  "middleRegionWidth": 500,
  "middleMainWidth": 500,
  "middlePaperWidth": 490,
  "middlePaperHeight": 840.531,
  "middleOverflowY": "auto",
  "canvasWidth": 486,
  "messageCount": 5,
  "proposalMessageCount": 1,
  "waitingAnswerInputCount": 1,
  "visibleOldProposalPrimaryActions": 0,
  "canvasTabs": ["工序图", "治理保证", "怎么跑"]
}
```

截图：[conversation 1280×820](../../output/playwright/2026-07-17-p1-c-conversation-1280x820.png)

## say 态 DOM 量测原文

```json
{
  "viewport": { "width": 1280, "height": 820 },
  "gridTemplateColumns": "218px 806px 180px",
  "columnGap": "14px",
  "middleRegionWidth": 806,
  "conversationColumn": {
    "x": 299,
    "y": 108,
    "width": 720,
    "maxWidth": "720px",
    "leftMargin": 41,
    "rightMargin": 41
  },
  "sayComposerWidth": 678,
  "messageCount": 1
}
```

截图：[say 1280×820](../../output/playwright/2026-07-17-p1-c-say-1280x820.png)

## 视觉核对

- 中栏只有一个现有纸面外框；方案、主管追问、用户答复和阶段交货按时间纵排，用既有虚线分隔，没有新增消息气泡、卡形或提示牌。
- 方案仍使用原 `JiaobanAuthorizeState` 内容；交货仍使用旧七态对应组件。已答追问折叠为一行，待答追问独占输入区。
- 左栏只改为「项目对话」及单子锚点；右区三视图名称、顺序和挂点保持原样。
- 浏览器控制台只有量尺页 `favicon.ico` 的 `404`，没有组件运行错误。

## 本证据没有覆盖的层

这不是完整桌面应用验收。为避免浏览器中触发 Tauri IPC，本次没有经过：

```text
App -> WorkbenchShell -> ActiveWorkbenchView -> ProjectsView -> ProjectDetail
```

也没有覆盖 WKWebView、Tauri 命令执行、真实 resident 进程或人工点击后的持久化回读。因此，本证据只证明 `ProjectWorkspaceShell` 入层后的真实 DOM/CSS 尺寸与结构；最终人工走查仍应重启桌面 App 后完成。
