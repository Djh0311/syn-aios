# 决策：G4 死重清扫——死视图迁宿主后删 + 死组件五件 + 挂账清 + 重复定义对平 v1

日期：2026-07-20
任务包：`tasks/2026-07-20-g4-deadweight-sweep-package-v1.md`（G4 死重清扫·轻档·视觉治理线收官包）
上位：`decisions/2026-07-20-jiaoban-design-direction-three-column-truth-restoration-v1.md`（G4 行）
预登记：`evidence/2026-07-20-g4-retired-assertions-preregistration-v1.md`（退役断言删前登记·照 07-18 s1-p1-b 先例）

## 拍板固化

1. **RunningWorkflowsView（1196 行死视图·App 零 import 实证）迁宿主后删**：离线场景迁移到被测组件本体——L5 收件箱断言平移直渲 `DailyMemoryCandidateInbox`；L3 弹窗语义平移直渲 `PermissionDialog`+字面 `record-operation-control-decision` action（语义核 3 条锁字面：does_execute_in_l3=false/status_after_confirmation=confirmed_recorded/readback_result_count=null）；主测试两处 combined markup 摘死视图半边保留活面 forbidden。壳断言（死视图 chrome+L3 按钮生成面+WorkflowStatePanel 块）全预登记退役（R1-R8）。
2. **死组件五件+死导出**：删 `WorkflowStatePanel.tsx`/`Metric.tsx`/`ProjectWorkflowRecoveryPanels.tsx`；摘 `ProjectRuleStatusBar`（连 `runcheckPillTone`/`runCheckStatusLabel`·保 `canvasBadgePillTone` 2 活引用）、`CandidateMemoryItem`、`ExpandRest`。
3. **连坐 CSS**：styles.css 删 running-status-pill 系/metric 系/workflow-state-panel 独占块/prsb 两块/`.canvas-hud-top .project-rule-status-bar`/`.spec-expand`；**共享选择器只摘成员**（:1197 `.metric,`、:3696 `.metric-grid,`、:5144 `.metric,`、:5155 `.workflow-state-panel,`、:8526 `.canvas-hud .project-rule-status-bar,`——保活成员逐处 diff 核）；残注释 :10058 改述「对齐 FactRow 定式」。
4. **gate 白名单只减不增**：retired_style_family 2→1（①随死视图整删核销）；hardcoded hex 39→36（休眠 `#a86a00`/`#b23b3b`/`#666` 核销）。两 decisions 同笔。
5. **重复定义口径对平**：执行线确定性清单（同文件同 @media 上下文同选择器多定义·递归解析器+逐对人工抽检）= **完全相同 0 组**（零删除）、**分叉重复 56 组**（styles.css 54 + sidePanel 2·桌面皮 base+覆盖/媒体变体设计件）——分叉一根手指不碰，清单入 evidence 挂账。审计「112 处」≈本口径 56 组 117 处出现；总指导「219/251」疑含跨 @media 上下文重复，两口径以本清单为对账源（包 §一.4 授权）。

## 挂账移交（G 线收官后）

- 死视图其余 running-* CSS（running-section/running-summary-grid/running-stage-band 等）非本包枚举面——死壳残段整扫留待下一治理刀（连同 G2 移交的死壳口径）。
- lib `buildOperationControlMemoryCaptureInput`（memoryDailyLoop.ts）随死视图成零引用——lib 死函数候选，非本包枚举（活组件/lib 一行不动纪律）。
- 分叉重复 56 组（evidence 清单）——层叠覆盖设计件，若治需逐对面审，另包。
