# G4 · 死重清扫 验收证据 v1

日期：2026-07-20 · 轻档 · 执行线施工，总指导回收核实物（无感受件）
任务包：`tasks/2026-07-20-g4-deadweight-sweep-package-v1.md`
基线 commit：`0e9c61d`（开工核对 HEAD=`0e9c61d`，工作树干净=G3 已收口）
预登记：`evidence/2026-07-20-g4-retired-assertions-preregistration-v1.md`
拍板留痕：`decisions/2026-07-20-g4-deadweight-sweep-v1.md`

## 一、结论

死视图迁宿主后删 + 死组件五件 + 挂账六件（PillRow aria 已于 G3 清·本包清：休眠 hex 3 条核销、残注释改述、ExpandRest 死导出、retired 白名单①核销、重复定义口径对平、预登记制）全做；四闸全绿；离线 25 组组数不变全绿。

## 二、四闸

| 闸 | 结果 |
|---|---|
| `npx tsc --noEmit` | 0 错 |
| `node scripts/run-offline-interaction-test.mjs` | **25 组全绿**（exit 0·组数不变） |
| shape gate baseline | 13/5/5 零净增；retired_style_family 0 violations / **1 deferred**（白名单 2→1 实证）；hardcoded hex 0 violations / 68 deferred（白名单 39→36） |
| shape gate check | 13/5/5 零净增 |
| `git diff --check` | exit 0 |
| cargo | 免跑：零 Rust 改动（git diff 无 .rs），包 §十.1 允许 |
| selftest 四册 | machine-face 18/18、hardcoded-hex 13/13、dedup 8/8、retired-style-family 13/13（不回归；case 5 照 §十.2 改夹具不改规则：核销①后 running-status-pill 再现=error + spec-empty deferred=1） |

## 三、宿主迁移（预登记 M1-M4·覆盖不丢）

| 组 | 迁移 | 断言平移 |
|---|---|---|
| M1 | L5 scenario 3 处 RunningWorkflowsView → 直渲 `DailyMemoryCandidateInbox`（本体·memory-center-daily-inbox.test.tsx 同法先例） | 收件箱文本 3 + 单条采纳 6 + 弹窗 2 + 批量 3 + 暂不 3 + 拒绝 3 = **20 条全平移** |
| M2 | L3 scenario → 直渲 `PermissionDialog`+字面 action | dialog 12+7=**19 条全平移** + 语义核 3 条（原 24 条按钮壳断言的语义核·does_execute_in_l3=false/confirmed_recorded/readback null） |
| M3 | 主文件 combined → AgentView 单面 | `combinedMarkupForbiddenTexts` 10 条全平移 |
| M4 | 主文件 combined → RightDetailPanel 单面 | `stageJCombinedForbiddenTexts` 全平移 |

迁移后活功能断言 ≥ 迁移前（壳除外）✓（§十.4）。退役=R1-R8 全在预登记档逐条（壳判据：死视图/死组件自留 chrome、无活宿主的按钮生成面；拿不准的 L3 按钮生成面按「生成面=死视图内联 JSX·App 零 import」判壳，语义核保留进 M2——此为本包最大判断点，提请总指导重点核）。

## 四、清零对账（grep 逐条·src 全零）

| 项 | 结果 |
|---|---|
| 4 删文件（RunningWorkflowsView/WorkflowStatePanel/Metric/ProjectWorkflowRecoveryPanels） | ls 四连 No such file ✓；删除前逐件复 grep 零 import（命令输出存 /tmp 复核过） |
| `running-status-pill` | src 0（CSS :287-310 系删+视图删） |
| `workflow-state-panel` | src 0（:3043-3052 独占块删+:5155 摘成员） |
| `.metric`/`.metric-grid` | src 0（:1187-1196 两块+:1206-1229 四块删；:1197/:3696/:5144 摘成员保 .panel/.item-card/.content-grid.four 等活成员） |
| `project-rule-status-bar`/`.prsb-headline` | src 0（:8429-8444 删+:8556-8560 删+:8526 摘成员） |
| `runcheckPillTone`/`runCheckStatusLabel`（PWCV 局部） | 0（`canvasBadgePillTone` 保 3=定义 1+活引用 2 对照 ✓） |
| `CandidateMemoryItem` | 0 |
| `ExpandRest`/`.spec-expand` | 0 |
| 休眠 hex `#a86a00`/`#b23b3b`/`#666` | src 0 + 白名单摘 3（39→36） |
| 残注释 styles.css:10058 | 改述「对齐 FactRow 定式」✓ |

## 五、重复定义（§一.4 口径对平）

执行线确定性清单：递归解析器（@media 上下文栈·注释剥离·选择器/块体归一化）扫全 6 CSS 文件 + 逐对人工抽检（.brand/.action-row/.jiaoban-history-new/.nav-group/.app-shell.right-pane-open/.right-pane-open .status-rail 6 对）：

- **完全相同（属性逐字一致）= 0 组 → 零删除**（扫描器 3 版互验：朴素哈希法报 5 组「同 selector+body」全部=跨 @media 上下文（base 与 min-width 媒体变体），按包口径非重复）。
- **分叉重复 56 组**（styles.css 54 + sidePanel 2）= 桌面皮 base+覆盖、媒体变体、层叠覆盖设计件——**一根手指不碰**，清单挂账（全表在 decisions §5 口径注记 + 本档 §六抽检）。
- 口径对平：审计「112 处」≈ 本口径 56 组 117 处出现；总指导「219 选择器/251 处」疑含跨 @media 上下文重复（如 .nav-group base 与 @media(min-width:1181px) 变体）。按包 §一.4 以执行线清单为唯一对账源。
- 层叠抽样 3 处（分叉·不动实证）：`.brand` @834 vs @3750（gap 10px/8px+色差·桌面皮覆盖）、`.action-row` @3363 vs @3369（布局/外边距分工）、`.jiaoban-history-new` @468 vs @1507（fixed 尺寸两种写法·后者覆盖）。零删除 → 层叠终值不变（自明）。

## 六、行数与棘轮

| 文件 | 前→后 |
|---|---|
| src 总计（git diff） | +99/−1969（14 文件） |
| styles.css | 10200 → **10105**（−95：连坐块 91+摘成员 5-注释改述 0，只降 ✓） |
| offline-permission-dialog.test.tsx（棘轮） | 3525 → **3408**（−117·水位线 3404 逼近，只降 ✓） |
| workbench-shape-gate.js | 零碰（498） |
| sidePanel/memoryCenter css | 零碰 ✓ |

无新 Tauri command、无新 sidecar JSON；未 stage、未 commit；start = end = `0e9c61d`。

## 七、枚举外事项与被闸拦过的事

1. **勘察差 1**：包 §1.1 列主测试文件 RunningWorkflowsView 2 处（:388/:486），实测 **5 处**（另 :605 J4 combined、:2478/:2493 壳文案两处）——三处全按壳/迁移归入预登记（M4/R2/R3），L3/L5 两 helper 计数与包文一致（各 3 处）。
2. **DailyMemoryCandidateInbox 宿主真相**：`<DailyMemoryCandidateInbox>` 生产引用唯死视图一处（MemoryCenterView/HomeView 仅用 derive 函数）——组件本体经 M1 迁移成测试直渲宿主（memory-center-daily-inbox 先例同形），App 不可达件属性未变，若后续「测试宿主不算活」再拍删除，挂账。
3. **lib 死函数候选**：`buildOperationControlMemoryCaptureInput` 随死视图零引用（lib 不动纪律·挂账）；死视图其余 running-* CSS 死壳未在本包枚举（挂账下一把刀，连同 G2 移交死壳口径）。
4. 被闸拦过的事：**无**（四闸直过；重复定义扫描器三轮迭代=方法学自查非闸拦——朴素法误报跨上下文 5 组、栈 bug 两版，递归正解+人工抽检定案）。
