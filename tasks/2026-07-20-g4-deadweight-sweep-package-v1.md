# 任务包：G4 · 死重清扫——死视图迁宿主后删 + 死组件五件 + 挂账六件 + 重复定义对平 v1

日期：2026-07-20
状态：**已出包，待总指导派工**
档位：**轻档**（纯前端死重删除+离线测试宿主迁移，不碰高危清单 5 条；不修宪、零行为变化——删的全是不可达件）
执行者：执行线；总指导回收核实物（无感受件·无需用户最后一眼，离线断言+四闸为凭）
所属开发线：桌面应用线 / 视觉治理线 G1✓→G2✓→G3✓→**G4**（线尾）
上位决策：`decisions/2026-07-20-jiaoban-design-direction-three-column-truth-restoration-v1.md`（G4 行）
用户追加拍板（2026-07-20 本代会话）：**RunningWorkflowsView=迁宿主后删**——离线测试 3 处场景（其把死视图当壳跑活功能真断言）先迁移到被测组件本体，断言平移、覆盖不丢，再删视图
勘察依据：总指导 2026-07-20 G4 写前勘察（G3 后行号重核；数字自带分列，禁用「等」字；重复定义数量标「待实测」）
本任务基线 commit：`0e9c61d`

## 一句话目标

App 不可达件全清：RunningWorkflowsView 1196 行死视图（离线宿主迁出后删）+死组件五件（Metric/WorkflowStatePanel/ProjectWorkflowRecoveryPanels/ProjectRuleStatusBar/CandidateMemoryItem）+ExpandRest 死导出+连坐 CSS/帮助函数+休眠 hex 3 条核销+残注释修正+同文件完全相同重复块删除——**离线覆盖不丢（断言平移）、活面零变化、gate 白名单只减不增**。

## 一、勘察实录（全部实测）

### 1.1 死视图纠缠面（用户拍「迁宿主后删」）

- `RunningWorkflowsView.tsx`（1196 行）：**App 零 import**（src 全仓仅自引用），但离线测试 3 文件 8 处当宿主渲染活功能：`tests/offline-permission-dialog.test.tsx` :388/:486（PCR6 等断言）、`tests/helpers/offlineL5MemoryDailyLoopScenario.tsx` :19/:38/:98（记忆候选收件箱真断言）、`tests/helpers/offlineL3OperationControlScenario.tsx` :26/:46/:108（操作控制 decision-only action 真断言）。两 scenario helper 仅被 offline-permission-dialog.test.tsx 引用。
- `WorkflowStatePanel.tsx`（87 行）：App 零 import；测试 1 处 `offline-permission-dialog.test.tsx:3472-3481`——该块**断言的就是 WorkflowStatePanel 本体**（「事实层面板」文案），组件死则块随废（走预登记，无迁移对象）。
- 连坐：`running-status-pill` CSS（`styles.css:287-308`，G2 白名单挂账）；`.workflow-state-panel` CSS（`styles.css:3043` 独占块+`:5155` 共享选择器成员）；gate `retired_style_family` 白名单①（RunningWorkflowsView 条）核销 2→1。

### 1.2 死组件五件（App+测试全零引用，逐件 grep 清零已核）

| 件 | 位置 | 连坐 |
|---|---|---|
| `Metric` | `components/Metric.tsx`（15 行） | `.metric-grid`（styles.css:1187-1191）、`.metric-grid.compact-metrics`（:1193-1195）、`.metric` 独占块（:1206-1209）、`.metric span/strong/small`（:1211-1225+尾部）；**共享选择器只摘成员**：:1197 `.metric,`（保 .panel/.item-card）、:3696 `.metric-grid,`（@media 块内）、:5144 `.metric,` |
| `WorkflowStatePanel` | `components/WorkflowStatePanel.tsx`（87 行） | §1.1 连坐 CSS+测试块退役 |
| `K3B1RecoveryCard` | `views/projects/ProjectWorkflowRecoveryPanels.tsx`（186 行整文件死；scenario 注释自证「UI 删除任务」后残件） | 零 CSS 专类 |
| `ProjectRuleStatusBar` | `ProjectWorkflowCanvasView.tsx:898-915` | `runcheckPillTone`（:887）+`runCheckStatusLabel`（:918-，唯一调用点:908 随死）；**`canvasBadgePillTone`（:888-）保留**（:1016/:1060 活引用）；CSS：`.project-rule-status-bar`（:8430-8438）+`.prsb-headline`（:8440-8445）独占删、`:8526` 共享选择器摘成员（保 .canvas-hud button/select/.workflow-state-actions）、`.canvas-hud-top .project-rule-status-bar`（:8557-8560）独占删 |
| `CandidateMemoryItem` | `views/memory/MemoryListPanels.tsx:49-79`（定义即唯一引用） | 零 |

另：`ExpandRest`（`SpecPrimitives.tsx:68-75`，全 App 零调用=G2 挂账死导出）+`.spec-expand`（`styles.css:10080`）；`jiaoban-linklike` 活（6 文件引用）不动。

### 1.3 挂账清两件

- **休眠 hex 3 条**：`#a86a00`/`#b23b3b`/`#666`（hardcoded-hex-rule.js 白名单 sidePanel 组内）src 引用已随 step-badge 删除清零（grep 实证），核销 39→36，`decisions/2026-07-20-hardcoded-hex-gate-rule-and-whitelist-v1.md` 同笔。
- **残注释**：`styles.css:10058` spec 区注释仍提 `.jiaoban-fact`（已退休），改述为「对齐 FactRow 定式」口径（纯注释一字修，闸不吃注释但正本 hygiene）。

### 1.4 重复定义（口径对平后施工）

审计口径「112 处」vs 总指导复测「同文件同 @media 上下文重复 219 选择器/251 处」——**口径未对平**。本包口径：执行线先出自有确定性清单（同文件同上下文同选择器多次定义=候选），分两类：**①完全相同**（属性逐字一致=纯死重，删）**②分叉重复**（属性不同=层叠覆盖，**不碰**，清单入 evidence 挂账）。数量一律标「待实测」，以执行线清单为唯一对账源；本包不预设删几行。

## 二、核心拍板口径（不许自由发挥）

1. **迁宿主=覆盖不丢**：每处场景迁移=识别被断言文案/行为的真实提供者组件→同 fixture 直渲该组件→断言逐条平移；**退役断言仅限宿主壳文案**（如「运行页」chrome 类），逐条进预登记 `evidence/2026-07-20-g4-retired-assertions-preregistration-v1.md`（照 07-18 s1-p1-b 先例：删前登记、理由、替代覆盖）。拿不准哪条是壳哪条是功能=停，回总指导。
2. **活面零变化**：删除件全=App 不可达；迁移只动 tests/**；src 活组件一行不动（除 §1.2 表列死件本体与连坐）。
3. **共享选择器只摘成员不删块**（§1.2 表逐处标明）；gate 白名单只减不增（retired 2→1、hex 39→36，decisions 同笔）。
4. 离线 25 组**组数不变**（场景改宿主不删组）；断言净退役数=预登记清单数，对平。
5. 重复定义只删「完全相同」；分叉重复一根手指不许碰。

## 三、施工清单（逐文件枚举）

1. **宿主迁移**（tests/）：offline-permission-dialog.test.tsx（:388/:486 两处+:3472 WorkflowStatePanel 块）+offlineL5MemoryDailyLoopScenario.tsx（3 处）+offlineL3OperationControlScenario.tsx（3 处），按 §二.1；预登记落 evidence。
2. **删除文件**：`src/views/RunningWorkflowsView.tsx`、`src/components/WorkflowStatePanel.tsx`、`src/components/Metric.tsx`、`src/views/projects/ProjectWorkflowRecoveryPanels.tsx`。
3. **死件摘除**：`ProjectWorkflowCanvasView.tsx`（:887 runcheckPillTone、:898-915 组件、:918- runCheckStatusLabel；保 :888 canvasBadgePillTone）；`MemoryListPanels.tsx`（:49-79 CandidateMemoryItem）；`SpecPrimitives.tsx`（ExpandRest）。
4. **CSS 连坐**（styles.css）：:287-308 running-status-pill 系、:1187-1195 metric-grid 两块、:1197 摘 `.metric,`、:1206-1225+ metric 独占块系、:3043 workflow-state-panel 独占块、:3696 摘 `.metric-grid,`、:5144 摘 `.metric,`、:5155 摘 `.workflow-state-panel,`、:10080 .spec-expand、:10058 残注释改述；projectWorkflowSidePanel.css 零碰。
5. **prsb CSS**（styles.css）：:8430-8438、:8440-8445 删；:8526 摘成员；:8557-8560 删。
6. **gate 两件**：`scripts/harness/lib/retired-style-family-rule.js` 白名单①删（2→1）；`scripts/harness/lib/hardcoded-hex-rule.js` 白名单摘 3 条（39→36）；两 decisions 同笔；gate 本体零碰。
7. **重复定义**：§一.4 口径施工+清单入 evidence。
8. **落档**：`evidence/2026-07-20-g4-deadweight-sweep-verification-v1.md`、`decisions/2026-07-20-g4-deadweight-sweep-v1.md`（含白名单核销与分叉重复挂账清单指针）、`CURRENT.md`（收口后总指导笔）。

## 四、允许读取

本包、上位决策、G1/G2/G3 包与证据、两 gate 规则文件、`prototypes/productized-desktop-shell/`（src/**、tests/**、scripts/**）、`docs/harness-catalog.md`。

## 五、允许写入

§三列出的文件（含 3 新建：预登记、evidence、decisions）。**不许碰**：Rust、`scripts/run-offline-interaction-test.mjs` 注册清单（25 组组数不变）、两 gate 本体（.js 主文件）、gate selftest 四册、projectWorkflowSidePanel.css、memoryCenter.css。

## 六、禁止事项

1. 活组件/活样式/活断言零碰；共享选择器删块=打回；分叉重复删一处=打回。
2. 离线 25 组逐字原则：只允许迁移改写§三.1 三文件与预登记内退役；其余测试文件一字不动。
3. 白名单只减不增；不许为过关塞违规。
4. 零文案改（残注释 §1.3 除外）、零行为变、零 Rust。
5. 不 stage、不 commit。
6. 删了任何有 import 的东西=事故（删除前逐件复 grep 清零，命令进 evidence）。

## 七、变更辐射面

- tests 三文件宿主迁移 → 离线 25 组全绿为准入；预登记=唯一退役口径。
- gate 白名单 2→1/39→36 → 之后 RunningWorkflowsView 文件名再现=error（已核销条目不再 deferred）；hex 3 值再现=error。
- 重复定义删除 → 层叠终值不变为凭（完全相同块后者=前者，删前者零影响；执行线给抽样对账）。

## 八、五态旅程走查

全部 zero-touch（死件五态皆不可达；迁移只在测试面）。

## 九、形状影响

- 任务类型：**治理任务包**（死重清扫+挂账清+闸口径收编）。
- 新增代码落点：零（净删）；测试三文件改写。
- 棘轮文件：`styles.css` 只降（预 −60 起+重复定义待实测）；`workbench-shape-gate.js` 零碰。
- 预计行数：src −1196−87−15−186−35（RuleStatusBar 系）−31（CandidateMemoryItem）−8（ExpandRest）≈ **−1558**；CSS 另计；tests 净减。
- 新增 Tauri command：无。新增 sidecar JSON：无。
- 退役断言：全预登记，不沉默删测。
- 本任务基线 commit：`0e9c61d`。完成 commit：总指导核收后另定（执行线不 commit）。

## 十、验收标准

1. 四闸：`npx tsc --noEmit`=0；`node scripts/run-offline-interaction-test.mjs` **25 组全绿**；shape gate baseline+check **13/5/5 零净增**（retired 白名单 2→1 后 deferred=1、hex 39→36）；`git diff --check` 过。Rust 零碰，cargo 免跑（理由明写）。
2. selftest 四册（18/13/8/13）不回归（白名单条目删后 selftest 若断言旧条目数=执行线须同步修 selftest 夹具并明写，**改夹具不改规则**）。
3. 清零对账（evidence grep 表逐条）：4 删文件名、running-status-pill、workflow-state-panel、metric/metric-grid、project-rule-status-bar/prsb-headline、runcheckPillTone、runCheckStatusLabel、CandidateMemoryItem、ExpandRest/spec-expand、休眠 hex 3 值——src 全零（canvasBadgePillTone 保 2 活引用为对照）。
4. 预登记 vs 实际退役断言逐条对平；宿主迁移后活功能断言条数 ≥ 迁移前功能断言条数（壳断言除外）。
5. 重复定义：完全相同块删除清单+分叉重复挂账清单双表入 evidence；删除后层叠抽样对账 3 处。

## 十一、必须回传（按 TASK_TEMPLATE 10 项）

做了什么 / 改了哪些文件 / 新增哪些测试或证据 / 哪些结论有依据 / 哪些仍不确定 / 风险和下一步建议 / shape gate baseline+check 摘要 / start-end commit / 是否新增 command·sidecar·触碰棘轮文件 / **被闸拦过的事**（无也必须写「无」）。

## 十二、总指导回收动作

- 亲跑四闸不信回传；清零表逐条对实物；预登记逐条核「壳 vs 功能」判得对不对；共享选择器摘成员处逐处 diff 核（保活成员在）；重复定义两表抽查。
- 判断 接受 / 需要修改 / 暂停 / 废弃，记 `docs/harness-catch-log.md`；收口 commit 同笔回写 CURRENT（G 线四包全收+视觉治理线收官）。
