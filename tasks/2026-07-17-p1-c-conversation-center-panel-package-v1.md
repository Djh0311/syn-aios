# 任务包:P1-C 中栏对话 UI(消息流+输入框)v1

日期:2026-07-17 · 档位:**轻档·前端为主+两处后端小补**(高危 5 条零碰) · 执行者:执行线 · 上位:总执行计划 §一 P1-C(防跑偏总则 7 条先读)· 前置 P1-A/B 已收口(`66ad542`)· 勘察笔记(先读透):`/private/tmp/claude-501/-Users-yoyi-workspace-product-line/cb32c0e7-68c5-4963-b125-c9fd5640ab66/scratchpad/p1c-frontend-recon.md`

## 背景(一句)

后端问答闭环已通(P1-B),但 `supervisor_message` 前端零消费者:主管的问题今天会以 Err 机器串裸上脸。P1-C 把中栏立成**对话消息流**:方案卡/主管追问/交货卡=消息,你回答=输入框,渲染语言=七律+定式(正本坐标见勘察 §3,不发明新视觉语言)。

## A·干什么

1. **A1 对话消息流(中栏新形态)**:消息列表组件——按时间序渲染:用户原话/主管追问(`supervisor_message` waiting_user=待答态·answered=已答折叠)/方案卡(既有定式=`DESIGN.md §三·五`,整卡作为一条消息)/交货卡消息形态(呈现改,沉淀行为照旧;【属实,沉淀】按钮语义零改,P3-C 才动)。「主管在看」等待态(consult 回合进行中的呼吸态,禁 spinner 裸转)。旧七态相位机**不拆**(P1-D 的活):对话流挂进 `renderLayout` 接缝(勘察 §1 坐标),态机与消息流并存过渡。
2. **A2 输入框+回答通道**:待答的主管追问下挂输入框→`submit_supervisor_resident_answer`(P1-B wrapper 现成);`already_answered` 幂等回执上脸人话。首问入口沿既有 consult 路径,本包不改下单形态(P1-D 收敛)。
3. **A3 接线三补(勘察 §2 三决策,此处拍定)**:①`onProposalStoreRefresh` 刷新链接上 `project_blackboards`(App.tsx 刷新链分家=修);②**后端小补**:`supervisor_message` 读模型 entry 补结构化 `question_id` 字段(**serde `default`+`skip_serializing_if` 双件,P1-A 前科点名**)——前端禁解析 Err 串取 id;③`humanizeConsultError` 认 `supervisor_resident_question_waiting_user:` 前缀出人话+`projectCanvas.ts:2039` 补该 kind 的 label 分支(勘察:现状裸机器串渗画布)。后端改动仅限此二处+读模型派生,其余后端零碰。
4. **A4 左栏与右区**:右区三视图**保留原挂点零改**;左栏演化最小步=项目对话锚点(点单子跳中栏对应消息位),九态列表照旧不重构。
5. **A5 验收路径**:①离线套件:新组件 DOM 断言按 runner 手工登记(勘察 §5:现 **26 项**基线,被测组件禁 hooks/`findElement` 平铺工装);②**实渲量尺铁律**(ledger M-2026-07-16):从 `ProjectWorkspaceShell` 整壳入层链入、真机中栏 **500px**(07-16 已改,330 是旧值教训)、say 态 720 限宽、窗口 1280×820,报告注明入层与缺层;③真机走查=用户重启 App 看(WKWebView css 热更不可靠,勘察 §6)。

## B·红线(违者停手报回)

1. **渲染语言只用正本**:七律=`DESIGN.md §五`,定式=`DESIGN.md §三·五`,交互宪法=`decisions/2026-07-14-interaction-model-canon-v1.md`——逐条对照施工,不自创卡形/不加新提示牌(防跑偏总则 1);词表禁「审批」等违宪词。
2. **别接混两条通道**:卡住脸「直接回它一句」占位框=P3-C worker 求助通道,**零碰**;本包输入框只接 resident 主管追问(`submit_supervisor_resident_answer`)。
3. 旧七态相位机/止血件(`JiaobanAuthorizeStates.tsx:485` 正则族)**不删不改**(P1-D 退场);`RunningWorkflowsView` 是治理测试载体勿动;guard/`final_mark`/S1/写域/高危 5 条零碰。
4. 后端仅 A3②③ 两点+派生函数;新持久化字段 serde 双件;pilot 面零改。
5. 四闸口径:cargo **988/0/47** 只增不减·typecheck 0·离线 **26** 基线只增不减·shape gate **仓根跑** 13/5/5 零净增;闸数字对应最终 diff(禁跑完闸再改码)。

## C·交付

1. 代码+离线 DOM 测试(消息流三型渲染/待答→已答态变/幂等回执/等待态);
2. 实渲量尺报告(入层链+500px 实渲截图或 DOM 量测原文+注明缺层);
3. 真机走查交接一句(告诉用户重启 App 看哪几处);
4. 10 项回传模板;被闸拦过的事如实列(含 A3② 后端补丁的 M5 对账测试全绿证明)。
