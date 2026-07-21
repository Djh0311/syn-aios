# G4 · 退役断言预登记（删前登记·照 07-18 s1-p1-b 先例）

日期：2026-07-20 · 任务包：`tasks/2026-07-20-g4-deadweight-sweep-package-v1.md` §二.1
原则：迁移=断言逐条平移到被测组件本体；**退役仅限宿主壳断言**（死视图/死组件自留 chrome 与「按钮在且能点」的壳胶合），每条给理由与替代覆盖。

## 一、真迁移（断言平移·覆盖不丢）

| 组 | 原宿主 | 新宿主 | 断言 |
|---|---|---|---|
| M1 | offlineL5MemoryDailyLoopScenario 3 处 RunningWorkflowsView | 直渲 `DailyMemoryCandidateInbox`（被测组件本体·既有 memory-center-daily-inbox.test.tsx 同法先例） | 收件箱文本 3 + 单条采纳 5（按钮/onClick/action kind/candidate_key/actor_role/path）+ 弹窗 2 + 批量 3 + 暂不处理 3 + 拒绝 3，逐条平移 |
| M2 | offlineL3OperationControlScenario dialog 段 | 直渲 `PermissionDialog` + 字面 `record-operation-control-decision` action（字段=原死视图 buildOperationControlAction 产物语义） | `l3OperationControlDialogExpectedTexts` 12 条 + `l3OperationControlForbiddenTexts` 7 条逐字平移；另锁字面 action 语义字段 3 条（does_execute_in_l3=false / status_after_confirmation / readback_result_count=null，原按钮断言的语义核） |
| M3 | 主文件 :482-497 combined markup（AgentView+RunningWorkflowsView） | 摘掉死视图半边，保留 AgentView | `combinedMarkupForbiddenTexts` 10 条逐字平移到活面 |
| M4 | 主文件 :605-631 combined markup（RunningWorkflowsView+RightDetailPanel） | 摘掉死视图半边，保留 RightDetailPanel | `stageJCombinedForbiddenTexts` 逐字平移到活面 |

## 二、退役登记（壳断言·随死件退场）

| 组 | 位置 | 条数 | 理由与替代覆盖 |
|---|---|---|---|
| R1 | 主文件 :387-400（PCR6/K3/PCR7 Running 面） | 8+8+5=21 条 includes | 死视图自留 SummaryTile 面板（统一执行命令/自动编排段）文案；同一读模型语义在活面继续断言（同场景 developerPanelsText 4 组 27 条 + projectDetailText + rightRail 2 组） |
| R2 | 主文件 :2477-2487 | 15 条 | 「运行中工作流」页 chrome（死视图本体文案）；无活面（视图整删） |
| R3 | 主文件 :2490-2503 | 3+4=7 条 | 死视图空画布文案与 forbidden；空态「不得显示成 0」语义随死视图退场（活面空态由 EmptyState 定式与各面 forbidden 覆盖） |
| R4 | 主文件 :3472-3509（WorkflowStatePanel 块） | 文本组+按钮 2+action 1+dialog 组 | 组件死则块随废（包 §1.1 明写·无迁移对象；初始化入口随组件死，PermissionDialog initialize 语义不受本块影响） |
| R5 | offlineL3OperationControlScenario :25-43 | 24+15=39 条 | 死视图 L3 决策面/运行队列 chrome；L3 action 语义核由 M2 字面断言+弹窗继续覆盖 |
| R6 | offlineL3OperationControlScenario :45-71 | 4 按钮×6=24 条 | 「按钮在且 onClick 生成 action」=死视图内联 JSX 壳胶合，无活宿主（App 零 import 实证）；语义核 3 条并入 M2 |
| R7 | offlineL3OperationControlScenario :96-115 | 3 条 | 死视图 confirmed_recorded 渲染态（决策已登记/仍未执行/disabled）；无活宿主 |
| R8 | offlineL5MemoryDailyLoopScenario 尾块（operation control 按钮） | 5 条 | 同 R5/R6 死视图 L3 面；capture input 系 lib `buildOperationControlMemoryCaptureInput` 产物，随死视图不可达（lib 函数留仓挂账·非本包枚举面） |

## 三、对账口径

- 退役=R1-R8；迁移=M1-M4。离线 25 组组数不变（scenario 函数签名不变、callsite 不动）。
- 宿主迁移后活功能断言条数 ≥ 迁移前功能断言条数（壳除外）：L5 收件箱 22 条全平移；L3 弹窗 19 条全平移+语义核 3 条（原 24 条按钮壳中保留语义核 3）；M3/M4 forbidden 组全平移。
