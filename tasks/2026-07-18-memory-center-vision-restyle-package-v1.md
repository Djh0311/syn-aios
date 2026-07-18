# 任务包:记忆中心按愿景稿重排(容器内滚)v1

日期:2026-07-18 · 档位:**轻档·纯前端**(高危 5 条零碰·rust 面零碰) · 执行者:执行线 · 上位:用户 07-18 开工令原话「**记忆中心的ui按照这一份来,滚动只允许在容器内**」;「这一份」=愿景稿 `prototypes/design-mockups/full-workbench-vision-mockup/index.html` 记忆层页(DOM `#pg-memory` L743-879 · CSS `.memwrap/.mlist/.mrow/.mgroup/.mmain/.mhead-strip/.mem-body/.lifecycle` L275-298 · 动效 L295-298;设计意图正本 DESIGN-NOTES-M1-v5 §七)。与 P3-A 收口并行(用户令);**P3-A 未提交文件零碰**(见红线 4)。

## 背景(一句)

记忆中心两轮定稿(`877d54e`+`7b261c9`)治好了「没法看」,但整页仍在 `.stage-pad` 里一根大柱子滚;愿景稿的形态是**全高双栏、左右各自内滚**+四语义组+单选三态详情。本包=把现有真数据/真命令换上愿景稿的脸,零新机器。

## 勘察既定事实(写包前已核,施工按此走别再猜)

- 滚动链:`.stage` overflow:hidden;整页滚在 `.stage-pad`(styles.css:7836,overflow:auto)。**零页滚做法=memoryCenter.css 覆写 `.stage-pad.memory-center{overflow:hidden;padding:0;height:100%}`,不动 App.tsx/ActiveWorkbenchView.tsx**(两者是 P3-A 在途文件)。
- 现视图:`MemoryCenterView.tsx`(786 行)双栏已在,但详情**正式+候选两卡同时堆叠**(L503-519);治理八面板收在 `<details>`(L526-777);列表=搜索+全部/候选/正式 chips+DailyMemoryCandidateInbox+合并列表。
- 真数据在手:`memoryLintStore.findings`(status=open · severity=blocking/needs_review,memoryCenter.ts:411-412)→「要你看」组;候选 `source_refs`/status;正式 scope 字段/M9 生命周期真命令(现已接线,含 preview+confirm);`summary.task_package_summary`/`pageReadModel` 各计数。
- 测试面:`memory-center-daily-inbox.test.tsx`+`offlineL5MemoryDailyLoopScenario.tsx`+`offlineMemoryCenter*Fixtures.ts`+`r4-page-selectors`;runner 手工登记制(`scripts/run-offline-interaction-test.mjs`)。

## A·干什么

1. **A1 布局容器化(用户红线)**:`.memory-center` 改愿景稿 `.memwrap` 结构——全高 flex;左列表 302px `overflow-y:auto`;右主栏 flex:1 `overflow-y:auto`+`max-width:620px` 居中列;现 `spec-scroll` 嵌套滚合并(容器内只留一层滚);窄视口(577 走查口径)两栏纵排各自内滚,轨道 `minmax(0,1fr)`(100f2d1 教训);**实测断言:html/body/.stage-pad 零滚,滚动只在 .mlist/.mmain**。
2. **A2 左列表四语义组**(愿景稿 L753-770):`候选·等你确认 N`(收编 DailyMemoryCandidateInbox 行——收件箱语义并入组,不再单独一块;meta 行=来源+日期)/`要你看 N`(=lint findings open 逐条真数据)/`正式·按项目 N`/`正式·全局 N`(scope 真字段分组);组头计数全真数;搜索框保留(既有能力,样式融入列表顶);筛选 chips 改按项目(真有数据的项目才显);列表底=「更多治理 ▸」收纳注(触发见 A5)。`.mrow` 两行式(claim ellipsis+meta)。
3. **A3 右栏单选三态详情**(治堆叠双卡):跨组**单一 selection**;
   - 候选卡:大字 claim(`.mem-body`)+「哪来的」(source_refs 真源+回链)+「和现有记忆」内联(lint 真结果有则显)+**按钮按真状态两步走**:`candidate_needs_review`→[属实(确认)];`candidate_confirmed` 未采纳→[记住(转正式)];[不要];沿现 `buildDailyMemoryCandidateDecisionAction`/`buildAdoptMemoryCandidateAction`。**禁把两步并一步**(canon §3:candidate_confirmed≠memory_active)。
   - 要你看卡:finding 人话+证据行+动作**只接真命令**(改写→M9 edit、废弃→M9 deprecate,对象=findings 指向的正式记忆);没有真命令支撑的动作(如「仍属实,保留」若无落库路径)**整个不做按钮**,不造假、不加提示牌。
   - 正式卡:大字 claim+scope/status 行+**来龙去脉时间线只放真事实**(source_refs 攒自哪单/created·confirmed 时间/版本号[M9 真]);「被引用 N 次」**仅当有真派生源才出该行,否则整行不出**(批2 假数 Badge 前科);生命周期按钮=现 M9 真接线原样保留,愿景稿样式(愿景稿此处是 Phase D toast,产品已领先,别倒退成 toast)。
4. **A4 体检行**(`.mhead-strip`):`N 条候选等确认`/`N 条要你看`/`出方案会带上 N 条`(第三个=pageReadModel/task_package 真数,无真数则该 chip 不出);点击=选中对应组首条。替换现 MemoryCenterStats 位。
5. **A5 治理收纳**:左底「更多治理 ▸」把右栏切到「治理面」(现 `<details>` 内八面板+MemoryWorkbenchSummary 原样组件搬入,容器内滚;showAll 开关保留);再点回详情。收纳≠删除,八面板能力/文案零改。
6. **A6 视觉**:token 同源(--hair/--line/--panel/--accent 系);衬线标题、`.fcard`/`.mgroup`/`.mrow`/`.mem-body` 按愿景稿移植进 memoryCenter.css;转正入列 `arrive`/`ring` 动效可做(尊重 prefers-reduced-motion);空态沿批2「答下一步」文案。
7. **A7 验收路径**:①离线 DOM(新测试文件+runner 预登记):四组渲染真数/单选切换三态/lint 组逐条/体检行计数/两步按钮状态/治理面切换;既有 `memory-center-daily-inbox`+L5 场景断言**翻案逐条列清单**(P1-E 精神);②**浏览器实渲量尺**(夹具渲染):1280×820+577 两视口,断言+实测 `document.documentElement.scrollHeight==clientHeight`、`.stage-pad.memory-center` 零滚、`.mlist/.mmain` 可滚;③四闸:typecheck 0/离线套件全过/gate 仓根 13 零净增/cargo 999·0·45 不动(rust 零碰仍须亲跑证明);④截图先过总指导看形,再用户真机最后一眼(感受件铁律)。

## B·红线(违者停手报回)

1. **纯呈现层**:零新命令/零新写点/零新 sidecar kind;一切动作只路由既有 PendingAction 构建器;UI 不写事实(蓝图 §5.1;防跑偏总则 3)。
2. **真数铁律**(批2 前科):任何计数/次数/徽标,无真数据源=整件不显示;禁占位假数、禁「示意」内容上产品脸。
3. **既有能力不退场**:搜索/生命周期真接线/维护运行/八面板/收件箱语义(并入组=形变,决策动作不减)全保留;L1 治理语义/两步确认制零改。
4. **并行纪律**:P3-A 未提交文件(App.tsx/ActiveWorkbenchView.tsx/ProjectsView.tsx/ProjectWorkspaceShell.tsx/jiaoban 族/types.rs 等)**一个不碰**;本包文件面=MemoryCenterView.tsx/views/memory/*/memoryCenter.css/新测试+runner 登记,超出即停手报回。k3_b1_recovery.rs/CURRENT/catch-log/design-mockups 同样不碰(他线在途)。
5. 防跑偏总则 1:不新增确认点/提示牌/治理工件;愿景稿里标「示意」的交互(toast 之类)不搬。
6. rust 面零碰;cargo/gate 数字回传须对应最终 diff。

## C·交付

1. 代码+新离线断言(registered)+既有断言翻案清单;
2. 量尺证据:两视口截图+零页滚实测数(scrollHeight/clientHeight 原值);
3. 10 项回传模板;真机走查点(交用户:开记忆层——整页不滚/两栏各自滚/点候选出卡/两步转正/要你看真发现/更多治理切换)。
