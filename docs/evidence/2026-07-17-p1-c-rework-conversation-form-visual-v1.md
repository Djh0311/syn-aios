# P1-C 对话回合与实体卡形态返工实渲证据 v1

## 结论

- 本单只改呈现：消息回合样式、方案/交货实体卡上下文样式，以及中栏的纯呈现 `data-conversation-phase` 挂点；没有改命令、hook、读模型或相位推导。
- `conversation` 态已退掉消息横线：用户回合右收为既有浅底，主管回合左侧直排；方案消息是本栏唯一完整实体框。
- 方案实体卡实测为白底、细实线、`12px` 圆角、`20px` 内边距、`14px` 组距；中栏纸框和主管意见内框均已退场。
- 交货 current done 复用同一实体卡壳；其旧“结果（人话）/复核体检单/主管意见”只在 delivery 消息上下文退框。主管退回处置、P3-C 回话框和历史交货根卡不命中该规则。
- `say` 态仍是 `720px` 中栏单框，没有新增卡、提示牌或 token。

## 实际入层

```text
ProjectWorkspaceShell
  -> ProjectJiaobanPanelBrowser
  -> renderLayout
  -> JiaobanMergedLayout
  -> .project-jiaoban-main
  -> .project-jiaoban-col[data-conversation-phase]
  -> JiaobanConversationStream
```

量尺页只装入固定离线夹具，并为四个挂载期只读命令提供浏览器 mock：正式记忆读取、运行历史读取、方案预演读取、批前边界意见读取。产品布局、组件和 CSS 均来自正式源码；取证后临时入口已从最终 diff 删除。

## conversation 态量尺

```json
{
  "viewport": { "width": 1280, "height": 820 },
  "gridTemplateColumns": "218px 500px 486px",
  "columnGap": "14px",
  "middle": {
    "dataConversationPhase": "proposal",
    "border": "0px none",
    "borderRadius": "0px",
    "padding": "20px"
  },
  "userTurn": {
    "justifySelf": "end",
    "borderRadius": "8px",
    "padding": "8px 12px",
    "background": "panel-soft"
  },
  "supervisorTurn": {
    "justifySelf": "start",
    "border": "0px none",
    "background": "transparent"
  },
  "proposalCard": {
    "width": 456,
    "background": "rgb(255, 255, 255)",
    "border": "1px solid rgba(28, 31, 36, 0.1)",
    "borderRadius": "12px",
    "padding": "20px",
    "gap": "14px"
  },
  "boundaryOpinionGroup": {
    "border": "0px none",
    "borderRadius": "0px",
    "padding": "0px",
    "background": "transparent"
  },
  "messageCount": 4,
  "canvasTabCount": 3
}
```

截图：[conversation 1280×820](../../output/playwright/2026-07-17-p1-c-rework-conversation-1280x820.png)

## say 态量尺

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
  "composer": {
    "width": 678,
    "border": "0px none",
    "background": "transparent",
    "padding": "0px"
  },
  "messageCount": 1
}
```

截图：[say 1280×820](../../output/playwright/2026-07-17-p1-c-rework-say-1280x820.png)

## 七律与边界核对

1. 一栏只留实体方案/交货卡一个完整框；中栏承载框和卡内旧意见/结果小卡均按消息类型退框。
2. 消息之间不再画横线；既有事实行/占位分隔语义不扩张。
3. 实体卡使用既有 `space-5 / r-lg / hair / panel-raised`，对应 `20px / 12px / 细实线 / 白底`，组距 `14px`。
4. 没有新造 pill；原状态 pill 与动作形态不变。
5. 事实行和交货体检行保留既有结构，没有拍平为散文。
6. 治理与怎么跑继续走既有层级和右区入口，没有把长内容展开成新提示牌。
7. 只使用既有石绿、米纸、圆角语言，没有新增颜色 token 或卡形。

`data-conversation-phase="legacy"` 明确排除外框退场规则，因此 P3-C 的 `.jiaoban-blocked*` 与回话框不命中；右区三视图选择器也不命中。交货内层规则把 `.role-loop-plain` 收窄到 `aria-label="结果（人话）"`，不会碰 `aria-label="主管退回处置"` 的止血件。

## 闸口

- Rust：`cargo test` → `988 passed; 0 failed; 47 ignored`。
- TypeScript：`npm run typecheck` → exit `0`。
- 离线交互：手工 runner `27` 项全绿；`jiaoban-conversation-center` 5 组通过。
- 仓根 shape gate：按要求从仓根执行，原始结果保持 `13 errors / 5 warnings / 5 info`；这是既有债务面，不冒充绿闸。
- `git diff --check` 在最终收口复核中执行。

## 本证据没有覆盖的层

- 这是浏览器中的真实组件树/DOM/CSS 证据，不是完整桌面 App、Tauri IPC、WKWebView 或 resident 进程验收。按返工单，先交截图看形；截图获准后才请用户做真机走查。
- 交货实体卡没有另加第三张交付截图；current done 与 delivered history 的卡壳/退框路径由真实组件树和 scoped selector 审查确认。
- `binding / running / waiting` 仍归旧 `legacy` 过渡相位；为严格保护 P3-C，本单没有扩大相位标形。它们若同时带历史方案，旧外框仍可能保留，此限制不在本次双态形态闸中冒充已消失。
