# P1-C 修单 2：纯对话与右区方案卡实渲证据 v1

## 结论

- 中栏已回归纯对话：只有你的消息、主管追问、已答折叠、主管短讯和输入/等待态；方案卡与交货卡整卡不再进入消息流。`binding / running / waiting_decision` 的既有处置块随工序图移到右区；红线要求零碰的 `blocked` P3-C 保持原位，作为唯一明确例外。
- 方案存在时，右区默认显示「方案」；实体卡保持 DESIGN §三·五的既有段落序，批准动作与卡同处右区。交货态同理默认显示「交货」。
- 主管短讯「方案好了，放你右手边了——看一眼，能跑就批。」是对话中的可点击话语。实点验证：先切到「工序图」，再点短讯，右区回到 `方案` tab 和 `jiaoban-canvas-view-proposal` panel。
- 点名的旧方案预警与只读单说明两条横幅已经从呈现删除；旧方案判定与只读动作机制仍在。
- `say` 态保持原窄右区提示；没有新增 token、卡形或横幅。

## 实际入层

```text
ProjectWorkspaceShell
  -> ProjectJiaobanPanelBrowser
  -> renderLayout
  -> JiaobanMergedLayout
  -> JiaobanConversationStream（中栏纯对话）
  -> canvasViews（右区方案 / 交货 / 工序图 / 治理保证 / 怎么跑）
```

量尺页只为产品组件树装入固定离线夹具，并 mock 四个挂载期只读命令。布局、交办面、对话流、方案卡、右区视图与 CSS 都来自正式产品源码；取证后临时量尺入口已从最终 diff 删除。

## 对话态：1280 × 820

```json
{
  "viewport": { "width": 1280, "height": 820 },
  "gridTemplateColumns": "218px 500px 486px",
  "columnGap": "14px",
  "middle": {
    "trackWidth": 500,
    "contentWidth": 496,
    "dataConversationPhase": "proposal",
    "border": "0px none",
    "borderRadius": "0px"
  },
  "conversation": {
    "messageCount": 4,
    "noticeCount": 1,
    "entityCardsInMiddle": 0,
    "noticeText": "方案好了，放你右手边了——看一眼，能跑就批。"
  },
  "right": {
    "activeTab": "方案",
    "tabCount": 4,
    "proposalCardCount": 1,
    "bannerCount": 0,
    "card": {
      "width": 462,
      "background": "rgb(255, 255, 255)",
      "border": "1px solid rgba(28, 31, 36, 0.1)",
      "borderRadius": "12px",
      "padding": "20px",
      "gap": "14px"
    },
    "approveAction": true
  }
}
```

截图：[对话态 1280×820](../../output/playwright/2026-07-17-p1-c-rework2-conversation-1280x820.png)

## say 态：1280 × 820

```json
{
  "viewport": { "width": 1280, "height": 820 },
  "gridTemplateColumns": "218px 806px 180px",
  "columnGap": "14px",
  "middle": {
    "width": 720,
    "dataConversationPhase": "composer",
    "border": "1px solid rgba(28, 31, 36, 0.18)",
    "borderRadius": "12px",
    "padding": "20px"
  },
  "rightTabCount": 0,
  "removedBannerCount": 0
}
```

截图：[say 态 1280×820](../../output/playwright/2026-07-17-p1-c-rework2-say-1280x820.png)

浏览器控制台：`0 errors / 0 warnings`；仅一条 React DevTools 开发提示。

## 七律逐条核对

1. **纸上放卡**：中栏不再放方案/交货实体卡；方案/交货 view 各只放一张既有实体卡。运行族处置沿既有工序图 view 下接，不新造第二种卡壳。
2. **虚线只做占位**：没有新增虚线容器；事实行的既有细分隔保持原义。
3. **统一网格**：右卡实测 `20px` 内边距、`14px` 组距、`12px` 既有圆角；左缘与原卡一致。
4. **小签唯一定式**：未新增 pill；原有状态 pill 和语义色不变。
5. **事实行定式**：方案/交货卡原事实行结构与段落序不动。
6. **长单分层**：治理保证、怎么跑、工序图继续作为右区展开视图；没有把机器细则摊进对话。
7. **一代视觉**：只复用既有米纸、白卡、石绿按钮与圆角 token；无新色、无新 token、无新卡形。

## A3：全对话面同类横幅扫描

已删除且只删呈现：

- 旧方案预警：`⚠ 这是 N 天前的旧方案……`。`proposalIsStale` 仍继续改变可用动作。
- 只读单说明：`⚠ 这单是只读的——AI 只看不改……`。`willWrite` 仍继续决定只读动作路径。

扫描后没有发现第三条“解释模式/建议重来”式同类横幅。以下可见警示不是同类提示牌，均保留：

- 真实咨询、resident 回答、账本读取错误的 `role="alert"`；
- 方案/交货卡里的主管审阅结论、证据缺口与预演警告；
- 左栏“卡住”状态点；
- P3-C 的「⚠ 卡住了」及其回话/错误通道；
- 开放问题止血件、最后停因、人类可读的安全事实。

这些项目表达真实错误、阻断、审阅或证据事实，不是被点名的解释型横幅；本单按红线没有删除、换形或扩写。

## 最终闸口

- Rust：`cargo test` → `988 passed; 0 failed; 47 ignored`。
- TypeScript：`npm run typecheck` → exit `0`。
- 离线交互：手工 runner `27` 个入口全绿；基线 `26` 只增登记 1 项，原入口未删。
- 仓根 shape gate：`Status: fail / 13 errors / 5 warnings / 5 info`。数字与要求口径一致，状态是既有债务，不冒充绿闸。
- `git diff --check`：最终收口复核通过。

## 覆盖边界

- 这是 Chromium 中的正式组件树、DOM 与 CSS 证据，不是完整 Tauri/WKWebView、真实 resident 进程或用户机器验收。
- 本轮按任务要求停在“总指导看形”；双态截图通过后，才进入用户真机走查。
- `blocked` 中栏仍是 P3-C 原有卡住脸；这是“P3-C 零碰”红线的显式例外，不把它冒充为已改造成普通短讯。
- 历史记录目前只有方案创建时间，没有独立交货时间；交货短讯因此按“无可靠时间，放在已知对话之后”呈现，没有伪造时间。未来读模型若提供交货时间，应改用真实字段。
