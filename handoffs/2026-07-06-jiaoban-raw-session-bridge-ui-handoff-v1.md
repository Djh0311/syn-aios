# 回交:交办「看原始对话」桥·主路径入口 · 执行线(UI)→ 主导线 v1

日期:2026-07-06 · 包:`tasks/2026-07-06-jiaoban-raw-session-bridge-ui-v1.md` · **子线未 commit**,工作树留给主导线核。全程轻档(纯前端)。

## 一句话结论

交办主路径四张脸(批卡收纳行 / 干 / 交货 / 卡住)都接上了「看原始对话」一键钻智能体页入口,走**已存在**的 `onOpenAgentSession`,App/AgentView/穿线/后端全 0-diff。三闸绿。**渲染类真机我验不了(computer-use 抓不到 Tauri dev 二进制),留你 Cmd+R——见 §5。**

## 0. 接手即发现:工作树里有半成品(无 handoff·来路不明)

进来时 `ProjectJiaobanPanel.tsx` 已有未提交改动(`JiaobanRawSessionLink` 组件 + `JiaobanSessionPicker` export/加 prop + 批卡收纳行接入)。我在其上续写。已跟你确认过「接着做完」再动手。

## 1. 落点清单(§2.1 五处)

- **组件 `JiaobanRawSessionLink`**(半成品建·我沿用):三态诚实——`sessionChoice`=真 thread_id→「看原始对话」/ 哨兵(`NEW_SESSION_CHOICE`)或未定 + 有 `latestSessionThreadId`→「看最近对话」/ 皆无→零渲染;无 `onOpenAgentSession`→不显(降级)。
- **批卡收纳行**(picker 内·半成品接入):真 thread 显入口,新建/未定不显(会话还没生·诚实,不传 `latestSessionThreadId`)。
- **干脸 `JiaobanRunningState`**(我加):进度下方入口。existing 跑中→看原始对话;哨兵单→`latestSession` 兜底看最近对话。
- **交货脸 `JiaobanDoneState`**(我加):产出行下方入口,同三态。
- **卡住脸 `JiaobanBlockedState`**(我加·**面级**):按 picker 内注释「防一脸双入口」,面级放而非行内(§2.1 卡住脸要显 + 注释一致)。
- **主组件**:算一次 `latestSessionThreadId`(按 `updated_at_ms` 倒序头条·`[...projectSessions]` 复制不 mutate),透传三脸。
- **CSS**(`projectWorkflowSidePanel.css` 纯加 4 selector):`jiaoban-session-summary-row`(批卡那行 flex 容器·**修好半成品缺 CSS 导致的不并排**)+ `jiaoban-raw-session-link`(复用 `jiaoban-linklike` 只加 nowrap/间距·不抢主按钮位)。
- **离线测试**(新 `tests/raw-session-bridge.test.tsx` + 跑器加 1 行):4 组 DOM 断言。

## 2. ⚠️ 审查线逮到一个真 bug,已修(诚实报备)

起了只读审查线(Explore·无 Edit 工具)复核,逮到**批卡透传链断**——我接手时看到组件接入(picker 内)就假设批卡通了、**没端到端追传参链**,漏了:① `Browser→JiaobanAuthorizeState` 没传 `onOpenAgentSession` ② `AuthorizeState→JiaobanSessionPicker` 没传。后果:批卡收纳行的 `JiaobanRawSessionLink` 拿到 `undefined`→`return null`→**批卡入口永不显示**(§4① 验收过不了)。

核实物(`grep onOpenAgentSession` + 读两处调用)确认 agent 对、非误报。**修**:
- 接通两跳(`onOpenAgentSession={onOpenAgentSession}`);
- **把 `JiaobanAuthorizeState.onOpenAgentSession` 改必填**(去 `?`)——上游任一处漏传直接 tsc 报错。比运行时测试更硬地防这类「组件接了、上游忘喂、静默不显」的假绿。修后 tsc 绿(证明接线通了)。

半成品的这个未完成部分是我该做完的,漏追链是我的疏漏,审查线兜住了。

**审查线另报的 issue#3(卡住脸 picker 不传 `onOpenAgentSession`)= 误判,不改**:那是设计——卡住脸走**面级**入口(已加),picker 行内不传是「防一脸双入口」,`sessionChoice` 为真 thread 时面级显「看原始对话」,覆盖到位。

## 3. 测试防线分工(回应审查线「假绿」担忧·诚实)

- **接线防回归 = tsc 必填**(§2 修点):`AuthorizeState.onOpenAgentSession` 必填,漏传=编译错。这是主防线。
- **组件契约 = 离线 DOM 测试**(`raw-session-bridge.test.tsx` 4 组):`JiaobanRawSessionLink` 三态 + 点击回调收到 thread_id(无 hooks·直接调 `el.props.onClick()`)+ 诚实词表(哨兵不吹大成「本单/原始对话」)+ picker 收/不收 `onOpenAgentSession` 两支(对应批卡传 / 卡住脸不传两条真实生产路径)。
- **为什么不端到端渲染 Browser**:主组件带 tauri 副作用、离线跑不了(codebase 惯例只离线测子组件);`AuthorizeState` 24 props 渲染 ROI 低于 tsc 必填。接线交给类型系统兜。

## 4. 三闸 + 0-diff 自证

- **tsc**:绿(必填接线通过)。
- **offline**:`15 passed`(含 `raw-session-bridge: 4 组全过`)。
- **build**:`✓ built`(chunk size warning 是预存·非本包)。
- **0-diff**(`git diff --name-only -- prototypes/`):只 `ProjectJiaobanPanel.tsx` / `projectWorkflowSidePanel.css` / `scripts/run-offline-interaction-test.mjs`(+1 行)+ 新 `tests/raw-session-bridge.test.tsx`。死线逐一 diff 空:**App.tsx / ActiveWorkbenchView.tsx / ProjectsView.tsx / AgentView.tsx / lib/** / src-tauri/** 全 0-diff**(命令自证跑过)。`onOpenAgentSession` 回调通道是**现成的**(没新穿一条);`sessionChoice` 只读(没改会话选择语义);哨兵单没硬造/反查 thread_id(只 `latestSession` 兜底)。
- CURRENT.md / decisions-07-02 / tasks-07-06 的改动是主导线预留、我没碰。

## 5. 真机待验(§4·你 Cmd+R·我做不了)

渲染/交互类必须真机(记忆 `ux-render-bugs-measure-before-guessing`)。待验:
1. **批卡**:选「接现有:X」→ 收纳行尾出现「看原始对话」且**与摘要按钮并排一行**(修的 CSS 是否对)→ 点它→跳智能体页**且该会话选中**;选「开个新的」→ 收纳行**不显**入口。
2. **交货脸**:existing 单显「看原始对话」/ 哨兵单显「看最近对话」→ 点→跳转选中。
3. **干脸 / 卡住脸**:入口在、不挤主按钮、点跳转选中。
4. 词表:界面只见「看原始对话」/「看最近对话」,不露 thread_id、不见「本单对话」。

## 6. 回交动作

§4 证据 + 落点 + 审查修复上文全。**子线不 commit**。前端主路径「看原始对话」桥落地,接线由 tsc 必填兜死;剩渲染真机你验一遍即可收口。
