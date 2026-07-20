# 任务包：G2 · 定式扶正——事实行 3 式→FactRow · pill 5 式→Pill · gate 防再造 v1

日期：2026-07-20
状态：**已出包，待总指导派工**
档位：**轻档**（纯前端呈现层收敛，不碰高危清单 5 条）
执行者：执行线；总指导回收核实物 + 截图对照（感受件纪律，设置页/记忆详情两处形歪=停包问用户）
所属开发线：桌面应用线 / 视觉治理线 G1→G4（G1 已收口 `a1d2a67`）
上位决策：`decisions/2026-07-20-jiaoban-design-direction-three-column-truth-restoration-v1.md`（G2 行）
法源：`prototypes/productized-desktop-shell/DESIGN.md` §三·五（2026-07-14 用户逐字段拍板）：**「小签唯一定式：pill/徽标只有一种——11px·2px 9px·全圆角·语义 token 色；不再各造各的」+「事实行定式：标签灰左·值右对齐·细虚线分隔；全 App 事实类信息同式」**
勘察依据：总指导 2026-07-20 G2 写前勘察（G1 后行号重核；本文数字全部自带分列对平，**禁用「等」字**）
本任务基线 commit：`8f2f94d`

## 一句话目标

spec-* 扶正为唯一：事实行其余 3 式（`jiaoban-fact`/`memory-kv`/`settings-fact`）迁入 `FactRow`，pill 其余 4 式（`Badge`/`jiaoban-step-badge`/`project-canvas-status-pill`/`prsb-pill`）迁入 `Pill`（`running-status-pill` 随死视图留 G4），迁移后删旧式，shape gate 加 `retired_style_family` 防再造规则——**功能零变化、文案零变化、语义零漂移，视觉变化仅限 §九枚举项**。

## 一、勘察实录（全部实测，数字自带分列）

### 1.1 spec-* 现状（`src/components/SpecPrimitives.tsx`）

- FactRow **56 处 / 6 文件**（ProjectOverviewPanels/SettingsView/SkillsBoardView/HarnessBoardView/HomeView/AuditLedgerView）；SegTitle 9；ListRow 5；EmptyState 16；**Pill/PillRow/ExpandRest = 0 用**（本包扶正 Pill/PillRow；ExpandRest 死组件候选挂 G4）。
- spec-* 类名在 SpecPrimitives.tsx 之外直连**仅 1 处**：`ActiveWorkbenchView.tsx:277` 的 `spec-empty`——**有意例外**（该处注释在档：想法箱空态文案「。」「；」混排与基座 what/next 拼接逐字对不上，不改基座），本包**不动**，进新规则白名单。

### 1.2 事实行 4 式（定式=spec-fact-row，其余 3 式迁）

| 式 | 调用点 | CSS 定义 |
|---|---|---|
| `jiaoban-fact` | `JiaobanAuthorizeStates.tsx` 4 行（:91-93 会改的文件、:98-100 写入范围、:102-104 不碰、:109-111 完整路径〔dev-detail 内〕） | `projectWorkflowSidePanel.css:1473-1489`（`.jiaoban-fact`/`-label`/`-value`） |
| `memory-kv` | `MemoryDetailPanels.tsx` 4 行（:156 哪来的、:157 和现有记忆、:158 候选边界、:210 证据） | `memoryCenter.css:534-547`（`.memory-center .memory-kv`+`strong`） |
| `settings-fact` | `SettingsView.tsx` `SettingFact` 组件（:215-221）×4 实例（:100-103 项目/智能体会话/技能/工作流），容器 `.settings-fact-grid`（:99） | `styles.css` 三处共享选择器内（:581 grid 组、:589 卡、:599 span、:609 strong） |

注意：`JiaobanDoneStates.tsx` 的 `.jiaoban-fact-btn`（:232/:236 引用，css :1078-1090）与 `.jiaoban-fact-done`（css :1091）是按钮/已沉淀标记，**非事实行，保留不动**。

### 1.3 pill 5 式（定式=spec-pill，其余 4 式迁 + 1 式随 G4）

| 式 | 调用点 | 现状色 | CSS 定义 |
|---|---|---|---|
| `.badge`（`components/Badge.tsx`，tone=neutral/candidate/warning/unknown） | **102 处 / 31 文件**（分列见 §四.1；显式 tone 34 = candidate 8 + warning 8 + unknown 15 + neutral 3，默认 neutral 68） | 24px 高·700 重·`--text-sm`·3px 8px·全圆角·语义 token | `styles.css:3451-3481`；`.badge-row` 共享选择器 `:3436`（tsx 零引用已核） |
| `jiaoban-step-badge`（tone-green/yellow/red/gray） | `JiaobanDoneStates.tsx` 5 处（:68 动态、:226 动态、:425 green、:427 yellow、:429 gray） | 硬编码 `#2e7d4f`/`#a86a00`/`#b23b3b`/`#666`+rgba | `projectWorkflowSidePanel.css:1055-1076` |
| `project-canvas-status-pill`（ready/accepted/running/warning/blocked/failed） | `ProjectWorkflowCanvasView.tsx` 4 span（:1006、:1050，`badgeItem.tone` 动态） | 方角带边·token+rgba | `styles.css:9061-9084` |
| `prsb-pill`（warning/runcheck.runnable/runcheck.warning/runcheck.blocked） | `ProjectWorkflowCanvasView.tsx` 2 span（:896、:900） | token+硬编码 `#3f5235`/`#7a2e2e`/`#f7e8e8` | `styles.css:8481-8508` |
| `running-status-pill` | `RunningWorkflowsView.tsx` 2 span（:599、:603）——**死视图（1196 行，G4 整删）** | — | `styles.css:287-308` **本包不动** |

另有 `jiaoban-done-pills`（pill 行容器）1 处 `JiaobanDoneStates.tsx:424`，css `styles.css:9610-9616`，迁 `PillRow`。`.jiaoban-step-row tone-*` 是行 tone 非 pill，**不动**。`spec-pill` 现零调用、定义在 `styles.css:10161-10165`（plain/ok/warn/run 四 tone）。

### 1.4 不在本包（移交 G4，挂账）

styles.css 死壳残段与同文件重复定义清扫：审计口径「112 处」，总指导复测口径「同 @media 上下文重复 219 选择器/251 处」——**两套口径未对平**，且属开放式删除，与定式扶正不同险级。移交 G4 死重清扫，G4 出包前先对平口径。本包只删 §四枚举的退休族 CSS。

## 二、核心拍板口径（不许自由发挥）

1. **DESIGN.md 已拍定式**：pill 唯一形=spec-pill（11px/2px 9px/全圆角/语义 token 色），事实行唯一形=spec-fact-row（标签灰左/值右/细虚线/tabular-nums）。迁移=向已定式收敛，不是重新设计。
2. **语义零漂移**：tone 映射严格按 §三表逐条，文案/结构/逻辑/事件零改（结构例外仅 §四.3 settings grid 容器去除一项，已枚举）。
3. **视觉变化仅限 §九枚举**，发现第 8 项先停，回总指导补枚举。
4. 离线测试/渲染断言/视觉夹具逐字不动；断言挂=修施工不修断言（测试面已核：仅 `report-fact-confirm-recall.test.tsx` 引用 `jiaoban-fact-btn`=保留类；`sc-badge`/对象 `.badge` 属性与本包无关）。

## 三、tone 映射表（交付合同）

`Pill` tone 枚举扩为 7：`plain/ok/warn/run` + **新增 `candidate`（--candidate-bg/--candidate）、`unknown`（--unknown-bg/--unknown）、`bad`（--danger-bg/--danger）**，色值全取 G1 正典既有 token，零新色。

| 旧 | → | 新 |
|---|---|---|
| Badge `neutral`（含默认 68 处） | → | `plain` |
| Badge `warning` | → | `warn` |
| Badge `candidate` | → | `candidate` |
| Badge `unknown` | → | `unknown` |
| step-badge `tone-green` | → | `ok` |
| step-badge `tone-yellow` | → | `warn` |
| step-badge `tone-red` | → | `bad` |
| step-badge `tone-gray` | → | `plain` |
| canvas-status-pill `ready`/`accepted` | → | `ok` |
| canvas-status-pill `running` | → | `run` |
| canvas-status-pill `warning`/`blocked`/`failed` | → | `warn` |
| prsb-pill 无 tone | → | `plain` |
| prsb-pill `warning` | → | `warn` |
| prsb-pill `runcheck.runnable` | → | `ok` |
| prsb-pill `runcheck.warning` | → | `warn` |
| prsb-pill `runcheck.blocked` | → | `bad` |

动态 tone 调用点（JiaobanDoneStates:68/:226、ProjectWorkflowCanvasView:896/:900/:1006/:1050）用本地映射函数或查表对象落到上表，**不许另造分支语义**。

## 四、施工清单（逐文件枚举）

### 4.1 Badge→Pill（102 处 / 31 文件，逐文件计数对账）

`components/DailyMemoryCandidateInbox.tsx` 1、`components/WorkflowStatePanel.tsx` 1、`views/KnowledgeBaseView.tsx` 4、`views/memory/MemoryWorkbenchSummary.tsx` 2、`views/memory/MemoryListPanels.tsx` 9、`views/MemoryCenterView.tsx` 8、`views/projects/ProjectJiaobanPanel.tsx` 2、`views/projects/ProjectWorkflowExecutionPanels.tsx` 5、`views/projects/ProjectWorkflowRecoveryPanels.tsx` 1、`views/projects/ProjectWorkspaceShell.tsx` 1、`views/projects/ProjectWorkflowRunCheckPanel.tsx` 1、`views/projects/jiaoban/JiaobanDoneStates.tsx` 1、`views/projects/jiaoban/JiaobanRunningStates.tsx` 2、`views/projects/jiaoban/JiaobanBlockedStates.tsx` 2、`views/projects/ProjectOverviewPanels.tsx` 3、`views/projects/ProjectWorkflowGovernancePanels.tsx` 5、`views/projects/ProjectWorkflowUnifiedExecutionCard.tsx` 1、`views/projects/ProjectWorkflowCanvasView.tsx` 4、`views/projects/ProjectTaskDraftPanels.tsx` 7、`views/projects/ProjectReferencePanels.tsx` 2、`views/projects/ProjectWorkflowDerivedPanels.tsx` 4、`views/projects/ProjectWorkflowSidePanel.tsx` 1、`views/SettingsView.tsx` 5、`views/OfflineRoleOrchestrationPanel.tsx` 2、`views/SkillsBoardView.tsx` 1、`views/agent/AgentAdapterBoundaryPanels.tsx` 4、`views/agent/TranscriptViews.tsx` 2、`views/agent/AgentContinuationBoundaryPanels.tsx` 8、`views/RunningWorkflowsView.tsx` 9（死视图，import 一并迁 Pill，文件本体 G4 整删）、`views/HarnessBoardView.tsx` 2、`views/AuditLedgerView.tsx` 2。

完成后删 `components/Badge.tsx`；删 `styles.css:3451-3481` `.badge` 系；`:3436` 共享选择器移除 `.badge-row,`（保留 `.action-row`）。

### 4.2 事实行三式→FactRow

1. `JiaobanAuthorizeStates.tsx`：4 行 `jiaoban-fact`→`<FactRow k={原标题}>{原值}</FactRow>`（k=「会改的文件」「写入范围」「不碰」「完整路径」，值节点原样搬入）。
2. `MemoryDetailPanels.tsx`：4 行 `memory-kv`→FactRow（k=「哪来的」「和现有记忆」「候选边界」「证据」；原 `<strong>` 标签不进 k，FactRow 自带标签样式）。
3. `SettingsView.tsx`：删 `SettingFact` 组件（:215-221），:99 `.settings-fact-grid` 容器去除（结构例外枚举项），4 实例直写 `<FactRow k="项目">{…}</FactRow>` 等 4 条。

### 4.3 pill 四式→Pill

1. `SpecPrimitives.tsx`：Pill tone 枚举扩 7；`styles.css:10161-10165` 区 +3 行（`spec-pill-candidate`/`spec-pill-unknown`/`spec-pill-bad`）。
2. `JiaobanDoneStates.tsx`：5 处 step-badge→Pill（:68/:226 动态走映射，:425→ok、:427→warn、:429→plain）；:424 `.jiaoban-done-pills` 容器→`<PillRow>`。
3. `ProjectWorkflowCanvasView.tsx`：:1006/:1050 canvas-status-pill→Pill（动态映射）；:896/:900 prsb-pill→Pill（动态映射）。
4. CSS 删除：`projectWorkflowSidePanel.css:1055-1076`（step-badge 系）、`:1473-1489`（jiaoban-fact 系）；`memoryCenter.css:534-547`（memory-kv 系）；`styles.css:8481-8508`（prsb-pill 系，**保留** `.project-rule-status-bar` 与 `.prsb-headline`）、`:9061-9084`（canvas-status-pill 系）、`:9610-9616`（jiaoban-done-pills）、`:3451-3481`（badge 系）；`styles.css:581` 共享选择器移除 `.settings-fact-grid,`、`:589` 移除 `.settings-fact,`、`:599` 移除 `.settings-fact span,`、`:609` 移除 `.settings-fact strong,`（四处均保留其余共享选择器成员）。
5. `running-status-pill`（`styles.css:287-308`+死视图 2 span）**不动**。

### 4.4 hex 白名单核销

prsb 迁移退休 `#3f5235`/`#7a2e2e`/`#f7e8e8` 3 条（`decisions/2026-07-20-hardcoded-hex-gate-rule-and-whitelist-v1.md` 18 条 b 类兜底行内），白名单 42→39，decisions 同笔更新，**只减不增**。

## 五、shape gate 新规则 `retired_style_family`

1. 本体 `scripts/harness/lib/retired-style-family-rule.js`（照 `machine-face-rule.js` 形：scan+attach；gate 本体现 495 行，**只许 +require/挂载/打印 ≤3 行，破 500 软限=打回**）。
2. 扫描面 src 全部 .tsx/.ts/.css，两类 error：
   - **退休族再现**：精确类名 `jiaoban-fact`（不匹配 `jiaoban-fact-btn`/`jiaoban-fact-done`）、`memory-kv`、`settings-fact`（含 `settings-fact-grid`）、`badge`（精确单词边界，不匹配 `sc-badge` 等复合名；`badge-row` 同属退休）、`jiaoban-step-badge`、`project-canvas-status-pill`、`prsb-pill`、`jiaoban-done-pills`；及对 `components/Badge` 的 import。
   - **spec-* 直连**：tsx 中 className 含 `spec-fact-row`/`spec-fact-k`/`spec-fact-v`/`spec-pill`（含各 tone）/`spec-pill-row`/`spec-seg-title`/`spec-list-row`/`spec-list-badge`/`spec-list-claim`/`spec-list-time`/`spec-empty`/`spec-expand`/`spec-bad`，而文件 ≠ `SpecPrimitives.tsx`。
3. 白名单 2 条（全登记 decisions，不沉默豁免）：①`RunningWorkflowsView.tsx` 的 `running-status-pill`（G4 整删时连带清）②`ActiveWorkbenchView.tsx:277` 的 `spec-empty`（有意例外，注释在档）。
4. 三件套：selftest（`workbench-shape-gate.retired-style-family.selftest.js`：退休类名→error、复合名/保留名→不误伤、白名单→deferred、SpecPrimitives 内 spec-*→不误伤）+ `docs/harness-catalog.md` 登记 + decisions 落档（`decisions/2026-07-20-g2-spec-primitives-restoration-and-retired-style-family-gate-rule-v1.md`）。

## 六、允许读取

本包、上位决策、DESIGN.md、G1 包/证据、hex 白名单 decisions、`prototypes/productized-desktop-shell/`（src/**、tests/**）、`scripts/harness/**`、`docs/harness-catalog.md`。

## 七、允许写入

- §4.1 列出的 31 个 Badge 文件 + `JiaobanAuthorizeStates.tsx`/`MemoryDetailPanels.tsx`/`SettingsView.tsx`/`JiaobanDoneStates.tsx`/`ProjectWorkflowCanvasView.tsx`/`SpecPrimitives.tsx`
- 删除 `src/components/Badge.tsx`
- `src/styles.css`、`src/views/projects/projectWorkflowSidePanel.css`、`src/views/memory/memoryCenter.css`
- `scripts/harness/lib/retired-style-family-rule.js`（新建）、`scripts/harness/workbench-shape-gate.js`（+≤3 行）、`scripts/harness/workbench-shape-gate.retired-style-family.selftest.js`（新建）
- `docs/harness-catalog.md`、`decisions/2026-07-20-g2-spec-primitives-restoration-and-retired-style-family-gate-rule-v1.md`（新建）、`decisions/2026-07-20-hardcoded-hex-gate-rule-and-whitelist-v1.md`（§4.4 核销）、`evidence/2026-07-20-g2-spec-primitives-restoration-verification-v1.md`（新建）
- `CURRENT.md` 最小回写（收口后，总指导笔）

## 八、禁止事项

1. 零文案改、零逻辑改、零 Rust 改、零测试/夹具改；除 §四枚举外零 TSX 结构变化。
2. `.jiaoban-fact-btn`/`.jiaoban-fact-done`/`.jiaoban-step-row tone-*`/`.project-rule-status-bar`/`.prsb-headline`/`running-status-pill` 不许碰。
3. `ActiveWorkbenchView.tsx:277` `spec-empty` 直连不许「顺手扶正」。
4. 白名单只减不增；不许为过关把违规塞进白名单。
5. 不 stage、不 commit。
6. 视觉变化不得超出 §九枚举；设置页/记忆详情截图若形歪（读着费劲/信息层级塌），**停包回总指导**，不许自行发挥找补。

## 九、视觉变化枚举（进截图对照，共 7 项）

1. **全 App 小签收敛**（主项）：102 处 Badge 形（24px 高/700 重/12px）→ spec-pill 形（11px/常规重/2px 9px）；色=同语义 token 不变。
2. 设置页「常规」4 数值卡（衬线大数+左竖线+浅底）→ 4 条标准事实行——**最大视觉差，截图重点**。
3. 记忆中心详情 4 kv 行（68px 标签列+深色 600 标签）→ 标准事实行（标签灰左+细虚线+值右）——长文值右对齐按定式，**截图重点**。
4. 授权卡 4 事实行：近似等价（spec-fact-row 本就照 jiaoban-fact 拍）；差=补 tabular-nums、值换行规则 `overflow-wrap`→`word-break`。
5. 交货卡概览/闸条/步条 badge：硬编码四色→语义 token（green→ok、yellow→warn、red→bad 朱砂系、gray→plain）；尺寸收敛。
6. 画布 status pill/prsb：方角带边→全圆角无边；`runcheck.blocked` 红系→danger token；3 硬编码 hex 退休。
7. Pill 新增 candidate/unknown/bad 三 tone 首次有实体引用，色值=既有 token。

## 十、变更辐射面

- 102 处 Badge+9 处其他 pill 调用点换组件 → 全 App 各面小签：离线交互 24 组全绿+渲染断言全绿+截图对照。
- `settings-fact` 共享选择器收编 → `.developer-boundary-item`/`.running-summary-grid` 等同组选择器**外观不许变**（收编只删 settings 成员，截图核对设置页开发者边界区）。
- gate 新规则 → 之后所有包退休族再现=error。
- 死视图 `RunningWorkflowsView.tsx` 内 9 处 Badge 随迁（文件仍被 tsc 编译，import 必须解析）；`running-status-pill` 残留已白名单。

## 十一、五态旅程走查

- 说：方案卡/授权卡事实行定式统一（§九.4）。
- 批：授权卡 pill/徽章形收敛；批准动作零碰。
- 干：画布状态 pill 语义色不变形收敛（§九.6）。
- 交货：交货卡概览 pill 行迁 PillRow+spec-pill（§九.5）；[属实,沉淀] 按钮（jiaoban-fact-btn）原样。
- 卡住：卡住脸 Badge→Pill（§九.1）；黄牌语义色不变。

## 十二、形状影响

- 任务类型：**治理任务包**（定式收敛+防再造闸）。
- 新增代码落点：`scripts/harness/lib/retired-style-family-rule.js`+selftest；组件零新增（用既有 FactRow/Pill/PillRow）。
- 棘轮文件：`styles.css` 只降不升（删 ≈91 行：badge 31+canvas pill 24+prsb 28+done-pills 7+共享选择器收编；增 3 行 pill tone）；`workbench-shape-gate.js` +≤3 行。
- 预计行数：TSX 净减（Badge.tsx −10、SettingFact −7、容器 −1，迁移面 ±0）；sidePanel css −39；memoryCenter css −14。
- 新增 Tauri command：无。新增 sidecar JSON：无。
- shape gate 豁免：白名单 2 条全登记 decisions。
- 本任务基线 commit：`8f2f94d`。完成 commit：总指导核收后另定（执行线不 commit）。

## 十三、验收标准

1. 四闸：`npx tsc --noEmit`=0；`node scripts/run-offline-interaction-test.mjs` 24 组全绿；shape gate baseline+check **13/5/5 零净增**（新规则 findings 仅 2 条白名单 deferred，零 error）；`git diff --check` 过。Rust 零碰，cargo 免跑（理由：零 Rust 改动，明写进证据）。
2. selftest：既有三件套 **18/13/8 不回归** + 新规则 selftest 全绿。
3. 迁移对账（evidence 给 grep 清零表，逐条）：`components/Badge.tsx` 删除且 import 清零；`badge`（精确）/`badge-row`/`jiaoban-step-badge`/`project-canvas-status-pill`/`prsb-pill`/`jiaoban-done-pills`/`jiaoban-fact`（精确）/`memory-kv`/`settings-fact` 在 src 清零（白名单 2 条例外）；FactRow 56→**68**（+4 授权卡 +4 记忆 +4 设置，SettingsView 内 SettingFact 删除）；Pill 0→**111**（102 Badge + 5 step-badge + 2 canvas + 2 prsb）；PillRow 0→**1**。
4. CSS 行数对照：§十二预估 vs 实测，进 evidence。
5. hex 白名单 42→39，grep `#3f5235`/`#7a2e2e`/`#f7e8e8` 清零。
6. **截图对照（感受件）**：交办授权卡/交货卡（含闸条步条）/记忆中心详情/设置页/画布页 ×1280 + 交办 ×577，施工前后各一套；Badge 面抽样：记忆中心列表/智能体页/审计账本/首页 ×1280。差异仅限 §九 7 项；先过总指导看形，设置页/记忆详情形歪=停包问用户。

## 十四、必须回传（按 TASK_TEMPLATE 10 项）

做了什么 / 改了哪些文件 / 新增哪些测试或证据 / 哪些结论有依据 / 哪些仍不确定 / 风险和下一步建议 / shape gate baseline+check 摘要（含新规则三数与 selftest 数）/ start-end commit / 是否新增 command·sidecar·触碰棘轮文件 / **被闸拦过的事**（无也必须写「无」）。

## 十五、总指导回收动作

- 亲跑四闸不信回传；对账表逐行对实物（grep 清零逐条+FactRow/Pill/PillRow 计数）；CSS 删除块抽 3 块对 HEAD 前后；截图对照逐张看，差异出枚举=打回；设置页/记忆详情两重点面看形。
- 判断 接受 / 需要修改 / 暂停 / 废弃，记 `docs/harness-catch-log.md`；收口 commit 同笔回写 CURRENT（G2 收口+挂账：死壳/重复定义移交 G4、ExpandRest 死组件候选）。
