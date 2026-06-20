# 会话流 · 加载抖动修复 + 滚动预加载 + 过程事件按轮收纳 方案 v1

> 日期：2026-06-20 · 作者：主导线 · 状态：待执行线落地
> 一句话判据：只动「智能体」视图的**会话流渲染**（transcript 加载/滚动/分组/显示）。不碰后端、不碰发送回路、不碰别的视图。

## 0. 缘起 / 症状（用户实机报告）

1. **向上滑动看更早信息时频闪跳动**：「加载更早对话」在运行态/静止态之间来回闪；消息在「加载」和「收纳」之间来回跳。
2. 要求：**去掉最上方的「加载更早」选项栏**，改成**向上滑动距顶就预加载**，不要滑到顶才加载。
3. 要求：**过程事件按对话分、收纳在每一轮输出的最上方**（像 codex），主流**只留 agent 的最终输出**；思考过程 / 工具调用不再平铺在对话框里。
4. 通用化：第 3 条对**以后接入的其它 agent 同样适用**（turn = 过程[折叠] + 最终输出，与 adapter 无关）。

## 1. Bug 定位（主导线读码，证据在手；落点是 2026-06-20 快照，当指针，执行线重核行号）

会话流是**自定义虚拟化 + 固定估算高度 + 多个滚动副作用互相打架**。文件 `src/views/agent/TranscriptViews.tsx`（除注明外）。按嫌疑排序：

- **#1 根因 · 固定高度虚拟化对不上真实高度**：`ESTIMATED_MESSAGE_HEIGHT=132` 写死（:8）；spacer 总高 / `offsetTop` / `firstVisible` 索引全按它算（`useVirtualMessageWindow` :293-309）。真实消息含 markdown、长文（>680 字还折叠），不可能等高。窗口一滑，挂载项真实高度 ≠ 估算 → 真实 `scrollHeight` 在脚下变 → `handleScroll`（:95-101）再触发 → 重算再跳。**抖动发动机。**
- **#2 · 滚到顶自动加载 + 锚定回弹链式重触发**：`scrollTop<24` 触发（:100）；加载完用**估算** scrollHeight 做锚定补偿（:83-93）→ 过/欠冲把视图又留在顶附近；`older_before_line` cursor 一变，guard `autoRequestedOlderCursorRef` 就重置（:62-66）→ 立刻再请求下一页 → `olderLoading` true/false 反复。**「加载更早在运行/静止间闪」字面就是它。**
- **#3 · 粘底与往上滚抢滚动**：`isNearBottom` 为真时每次条数变化都 `requestAnimationFrame(scrollToLatest)`（:77-81）；开会话默认置 true（:74）→ 测量不准就和用户上滚打架，顶/底弹。
- **#4 · relay 运行时 poll 每 tick 灌 live events（放大器，已确认）**：reader 渲染的是 `transcriptWithPendingMessages`（`AgentConversationShell.tsx:752`），它合并 `manualRelayReceipt.live_events`（:610/633/691），poll 每 tick 变 → transcript 每 tick 换身份 → #1/#2/#3 被持续放大。**「运行态持续抖」的实锤，不只是按钮文案。**
- **#5 · 帮凶**：`displayEvents` 把 tool/reasoning 状态项也算进主流（:41-48）→ 条数更多、估算误差更大。正好就是第 3 条要砍的「过程事件平铺」。

### 可能误判（用户明确要的）
- 读码非看现场。主因极可能是 **#1+#2**，#3/#4 是放大。**要分清哪条主导，得真机加最小日志看 `scrollTop / events.length` 的抖动序列**——这步执行线在真机做（主导线起不了 app）。
- 若真实消息其实接近 132，#1 就轻；但有 markdown 基本不可能等高。
- **最关键的误判预防**：第 2 条要的「去栏 + 滚动预加载」，**若只调低阈值、不先修锚定**，会触发更频繁 → **抖得更狠**。修复顺序必须先稳锚定，再上预加载。

## 2. 修法（**顺序不可颠倒**）

### Part 0 · 先复现确诊（执行线，真机）
- 真机复现抖动，加最小日志打 `scrollTop / events.length / olderLoading / isNearBottom` 的时间序列，确认主因是否如 #1/#2。**用它当 before；修完同样录一遍当 after 证据。**

### Part 1 · 先把渲染/锚定弄稳（bug 本体，必须最先）
- **推荐**：去掉固定估算虚拟化。页大小才 80（`DEFAULT_TRANSCRIPT_PAGE_LIMIT`），桌面聊天，**常规量直接整段渲染** + 正确的滚动锚定即可，先把抖动发动机拆了。真有超大会话再单独上**量真实高度**的动态虚拟列表（成熟库 / `ResizeObserver`），别再用写死的 132。
- 锚定改对：① 底部跟随（贴底时新消息到底）；② **prepend 更早页时按真实高度差锚定**（记录 prepend 前后 `scrollHeight` 实测差，不用估算）；③ 去掉 #3 里「条数一变就拉到底」的粗暴逻辑，只在确实贴底时跟随。
- 收口 #4：relay 运行时 live events 合并别让整个 transcript 换身份触发全量重排（稳定 key / 只追加尾部 / 合并节流）。

### Part 2 · 去加载栏 + 距顶预加载（**Part 1 稳了之后**）
- 删 `transcriptPageBoundary` 按钮（:128-139, 159）。
- `scrollTop<24` 换成**距顶 N 个视口高就预加载**（predictive）；保留 `autoRequestedOlderCursorRef` 的「同 cursor 不重复请求」防抖，避免链式触发。
- 顶部「已到达最早片段」可留作静默提示，不留按钮。

### Part 3 · 过程事件按轮收纳、只留最终输出（change B/C，可与 Part 1/2 并行）
- 核心在 `conversationTurns` + `isCodexNativeStatusEvent` / `displayEvents` 的分类（:39-56, :284-291）。
- 重构成**按 turn 分组**：每一轮 = ①「过程」折叠（思考 / 工具调用 / 工具结果 / reasoning，**收纳在该轮顶部**）+ ② 主流只显该轮**最终 `agent_message`**。
- **与 adapter 无关**：turn 模型不写死 codex 语义——「过程事件」「最终输出」用通用分类，别的 agent 接入同样适用。
- **继承收口死线**（接上 `2026-06-20-ui-internal-field-disclosure-sweep` 那条）：过程事件是**收纳进折叠、不是删**——每轮折叠里必须可达。藏 ≠ 删 ≠ 静默。
- 现有底部「过程事件（N）」总折叠（:222-242）与新「按轮折叠」二选一或并存，执行线定，别两套语义打架。

## 3. 边界 / 高危
- **轻档**，纯前端会话渲染。**不碰后端、不碰发送回路（manual_relay / composer）、不碰别的视图。**
- 死线：过程事件、更早消息**不许丢**——折叠里 / 滚动加载后都要可达。
- **与 codex-layout 那轮同文件**（`TranscriptViews` / `AgentConversationShell`）：排序或分段，别同文件打架。

## 4. 验证（完成必附真证据）
- **真机**（执行线，最硬）：Part 0 的 before/after 抖动序列对比——上滑加载更早**不再频闪**；运行态下滚动稳；去栏后距顶预加载顺滑。
- typecheck + `test:offline-interaction` 绿。
- 结构断言（offline，照搬收口那套）：① 主流不含 tool/reasoning 项、只含最终 `agent_message`；② 过程事件在「每轮折叠」内可达（剥折叠后正常态不含、完整 markup 含）；③ 顶部无「加载更早」按钮。
- 扫 diff：无后端改动；无「更早消息 / 过程事件被删而非收纳」的分支。

## 5. 落地前确认
- Part 1 选「整段渲染」还是「动态高度虚拟化」——执行线按真机量级定，但**不许再用写死的 132 估算**。
- 真机确诊主因（Part 0）后若发现主导原因不在 #1/#2，回报主导线再调方案，别闷头改偏。

---

## v1.1 真机回归修订（2026-06-20 · 主导线 · 主机验后）— 本节优先级高于上文对应部分

> 用户真机验收：**频闪 bug 已消除** ✓。但发现两处残留——加载时仍卡、轮分类没分对。下面是读重写后真代码（`TranscriptViews.tsx` 现版 + `lib/conversationTurns.ts`）的定位。落点是 2026-06-20 现版行号，当指针。

### R1 · 加载仍卡（修 Part 1 的两处叠加）
- **a. 锚定改 `useLayoutEffect`（残留跳）**：prepend 后校正 scrollTop 那段现在是 `useEffect`（`TranscriptViews.tsx:106-115`）→ 在浏览器**绘制之后**才跑 → 新内容先以错位置画一帧再跳回 = 可见“卡一下”。改 `useLayoutEffect`（绘制前同步校正）。换会话跳底（:83-89）、贴底跟随（:93-101）同改。
- **b. 砍整段渲染的挂载成本（真卡顿）**：无窗口化，一次 prepend ~80 条 markdown 全量挂 DOM = 主线程一次性大渲染；会话越长 DOM 越大、越卡。**不强制重上虚拟化**——优先用 CSS `content-visibility:auto` + `contain-intrinsic-size`（屏外消息跳过布局/绘制），和/或原生 `overflow-anchor:auto`（连手写锚定数学一起省）。仍不够再上“量真实高度”的轻窗口化。

### R2 · 轮分类没分对（修 Part 3 的 `buildConversationTurns`）
- **现状 bug**：`buildConversationTurns`（`TranscriptViews.tsx:236-269`）一遇 `assistant_message` 就设 `final` 并**立刻 flush**（:259-262）；`lib/conversationTurns.ts:64` 又把**所有**干净 assistant_message 都收进来、不区分中间/最终。叠加结果：**一轮里 agent 发多条消息（codex 常见：工具前先来句“我看下这个文件”的前导 + 真正答案），每条都成独立“最终输出”进主流** → 前导“过程消息”漏进最终输出。
- **修法（adapter 无关）**：轮边界改成**按 `user_message` 划**，不在 assistant_message 处 flush；一轮内**末条 assistant_message = final**，**之前的 assistant_message 连同思考/工具一起折进 process**。任何 agent 接入同理：轮 = user →[过程：思考/工具/中间消息]→[末条=最终]。
- **死线继承**：中间消息是**折进 per-turn 过程折叠、不是删**——折叠内必须可达。

### R3 · final 判据先对真实数据确认（防误判）
- 执行线落地前**对一条真实 rollout 看事件序列**，确认“中间消息 vs 最终”到底靠什么认：多条 `agent_message`？`turn.completed`？某 metadata？**别凭空猜数据形状。** 稳妥默认 = 本轮末条 assistant = final；若真实数据有更强的完成标记，优先用它。

### 验证（本节，完成必附）
- **真机 before/after（最硬）**：加载更早**不再卡/跳**；构造“一轮多条 assistant_message”时**主流只剩末条**、前导消息在过程折叠内可达。
- typecheck + `test:offline-interaction` 绿；**加结构断言**：fixture 造“一轮两条 assistant_message” → 断言主流只含末条、前一条在 `chat-turn-process` 折叠内可达（剥折叠后正常态不含）。
- 扫 diff：仍无后端；中间消息是折叠非删。
