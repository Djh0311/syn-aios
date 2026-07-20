# 任务包：人话工程①②——错误翻译族前后端收编 + shape gate 机器话上脸机械规则 v1

日期：2026-07-20
状态：**已出包，待总指导派工**
档位：**轻档**（前端/后端命令逻辑/重构，不碰高危清单 5 条）
执行者：执行线；总指导回收核实物
所属开发线：桌面应用线 / 人话工程
上位计划：`docs/plans/2026-07-16-master-execution-plan-conversation-first-v1.md` §三「人话工程」行（三件套，不做大翻译层；本包=①②，③清单随走查攒不在本包）
法源：交互宪法 §四.3「禁机器内部术语上脸」（`decisions/2026-07-14-interaction-model-canon-v1.md`）
勘察依据：总指导 2026-07-20 写前勘察（全文 file:line 见本包 §二）
本任务基线 commit：`24bde2e`（S1B/S1C/H1/H2/R1 收口基线）

## 一句话目标

把前端散装的错误翻译函数收进单一模块 `src/lib/humanize.ts`、后端删掉两个 07-09 遗留薄委托壳直调既有单一真源，并给 shape gate 加一条「UI 组件禁直渲机器格式错误串」机械规则（既有违规白名单登记、新增零容忍、基线 13/5/5 零净增）。

## 一、用户拍板与边界

总执行计划 §三原文：「三件套，不做大翻译层（治本=字串出生即人话，大层=重流程+治标）：①散装 humanize* 收编成前后端各一人话模块（照 `run_error_translation.rs` 单一真源样）；②shape gate 加机械规则=UI 组件禁直渲机器格式错误串；③用户真机点名的机器话清单逐条进模块（随走查攒）。」

本包只做①②。**不做大翻译层、不做③清单、不收纯枚举状态标签族**（`*Label` 全量约 167 个，另包；重复簇治理另包，清单见 §九挂账）。

## 二、勘察结论（写前已读透产/消/依赖三段）

### 2.1 后端：07-09 已收编过一轮，本包只剩收尾

- 单一真源已存在：`src-tauri/src/run_error_translation.rs`（319 行，三段 `{family,human,raw_snippet}`、七族+unknown、三 API：`classify_provider_failure_human` :42 / `classify_run_error` :66 / `humanize_error_for_display` :183；内联测试 9 个）。
- **承重红线**（同文件 :12-14 明令）：`codex_provider_unavailable:` / `consult_last_message_read_failed:` 两前缀是 director retry 承重标记（`director_agent.rs:1384/1396/1399`），只读不改；`classify_codex_resume_failure`（`workflow_execution_entrypoints.rs:217-268`）产机器 warning 标签，零碰（撞 `lib.rs:9939` 测试）。
- 待收尾仅两处薄委托壳（2026-07-09 A·收编遗留，函数体即一行委托）：
  - `secretary_agent.rs:187-189 humanize_consult_error` → 调用点 :221
  - `global_supervisor_agent.rs:420-422 humanize_consult_error` → 调用点 :530、:843
  - 处理：删壳，调用点改直调 `run_error_translation::humanize_error_for_display`。
- `codex_local_runner.rs:402-404 classify_codex_provider_failure` 是供给前缀路由承重壳，**保留不动**。

### 2.2 前端：错误翻译族 10 函数散 7 文件（本包收编对象）

| 函数 | 现位置 | 调用方 |
|---|---|---|
| `humanizePreviewError` | `ProjectJiaobanPanel.tsx:273-277` | 同文件 :844 |
| `isAlreadyConfirmedRejection` | `ProjectJiaobanPanel.tsx:1712-1720` | 被 humanizeAuthorizeError 用 |
| `humanizeProviderUnavailable` | `ProjectJiaobanPanel.tsx:1724-1735` | 同文件 :1741 |
| `humanizeAuthorizeError` | `ProjectJiaobanPanel.tsx:1739-1753` | 同文件 :1006、:1062 |
| `normalizeTranscriptError`（含映射表 1013-1077） | `AgentConversationShell.tsx:1019` | 同文件 :1190 → :1244-1247 |
| `friendlyLiveTitle` | `TranscriptViews.tsx:470-487` | 同文件 :437 |
| `friendlyLiveDetail` | `TranscriptViews.tsx:489-497` | 同文件 :454 |
| `historyErrorFamilyLabel` | `jiaoban/JiaobanHistory.tsx:46-65` | 同文件 :267 |
| `humanizeVerdict` | `jiaoban/JiaobanHistory.tsx:86-104` | 同文件 :229-230 |
| `humanizeWriteRoots` | `jiaoban/JiaobanAuthorizeStates.tsx:386-391` | 同文件 :96 |
| `humanizeChainProgress` | `jiaoban/JiaobanRunningStates.tsx:130-141` | 同文件 :31；Panel :1677-1682 有 re-export 先例 |

收编目标：**新建 `src/lib/humanize.ts`**，上述函数**函数体逐字原样迁入**，原位置改 re-export 保导入面（照 Panel :1677-1682 既有先例）。不放 `format.ts`——它是状态/枚举标签层，错误翻译与枚举标签两类混一个文件会长成新巨石。

顺势一处呈现修（bounded）：`App.tsx:783-786 messageOf` 族（notice 直渲 error.message，stage1 体检已点名；同类调用点 `App.tsx:175/206/281`）接 `humanize.ts` 薄委托，语义照后端 `humanize_error_for_display`：**命中族出人话，unknown 回退原文**——未命中时显示串逐字不变。

### 2.3 shape gate：扩展点与 500 行软限坑

- 实现：`scripts/harness/workbench-shape-gate.js`（现 489 行，纯 Node 只读，仓根直跑；`docs/agent-mistake-ledger.md:379` 固化 cwd 纪律）。
- 扩展点（照去重门先例）：常量区（仿 :82-105）+ 扫描函数（仿 `scanHelperDuplicates` :298-328，`walkFiles` :148 现成）+ `buildReport` 挂载（:355-358 旁）+ 自测（仿 `workbench-shape-gate.dedup.selftest.js` 夹具树模式）+ `docs/harness-catalog.md` 登记。
- **坑**：`JS_GATE_SOFT_LIMIT=500`（:15），现 489 行，加规则必破 → warnings 5→6 破基线。解法二选一（执行线择一并在 evidence 说明）：a) 新规则拆 `scripts/harness/lib/machine-face-rule.js` 再 require（harness-catalog.md:98 有 lib/ 先例，**首选**）；b) 调 `JS_GATE_SOFT_LIMIT` 并在 decisions 留痕。
- 基线纪律：13 errors/5 warnings/5 infos 全为历史债，**零净增**。

### 2.4 既有违规与合规格板（白名单底稿）

- 合规格板（规则不得误伤）：`jiaoban/JiaobanHistory.tsx:261-278`——`raw_snippet` 收 `<details>查看原文</details>` 下钻（有离线断言锁 `tests/jiaoban-history-and-secretary-board.test.tsx:123-145`）。规则须按上下文排除或白名单单条登记。
- 既有违规（进 `MACHINE_FACE_DEFER_WHITELIST`，仿 `DEDUP_DEFER_WHITELIST` :92-105 `pattern|path` 形+理由）：
  - `main.tsx:46`（启动失败屏 `<code>{error.message}</code>`，:33 还进窗口标题）
  - `TranscriptViews.tsx:428`、`:603`（`<pre>stderr: {event.stderr}</pre>` 转录详情面）
  - `App.tsx:175/206/281` 经 `messageOf` 直渲（**本包顺势治平**，治平后不进白名单）
- warn-only 档（不拦，照 `converged_helper_redefined` :417-419 先例）：`setXxx(error.message …)` 进 state 形（`AuditLedgerView.tsx:87`、`SecretaryBrief.tsx:116`、`ProjectJiaobanPanel.tsx:613`、`:1139` 等）——误报面大，先观察。
- 对话层已核无直渲：`JiaobanConversation.tsx:252-255/509-513` 的错误全部产自 `useJiaobanConversationState.ts:19-23` 五句人话常量，不违规。

## 三、交付结果

1. `src/lib/humanize.ts`：前端错误翻译族单一真源（10 函数迁入+App notice 薄委托），原位置 re-export 零导入面破损。
2. 后端两壳删除，调用点直调；`run_error_translation.rs` 本体零改。
3. `scripts/harness/`：新机械规则（名建议 `machine_face_on_ui`）error 级拦新增直渲、warn-only 记 state 形、`MACHINE_FACE_DEFER_WHITELIST` 登记 §2.4 既有违规；配套 selftest；`harness-catalog.md` 登记；500 行软限按 §2.3 解法处理。
4. `decisions/2026-07-20-machine-face-gate-rule-and-defer-whitelist-v1.md`：规则语义+白名单条目+豁免理由留痕（shape gate 豁免不许沉默）。
5. evidence `evidence/2026-07-20-human-language-modules-and-machine-face-rule-verification-v1.md`：前后指标+四闸结果。

## 四、允许读取

- 本包、总执行计划 §三、交互宪法 §四.3、`docs/plans/2026-07-14-syn-frontend-stage1-audit-v1.md` §四.3
- `prototypes/productized-desktop-shell/` 全部源码与测试
- `scripts/harness/`、`docs/harness-catalog.md`、`docs/harness-catch-log.md`、`docs/agent-mistake-ledger.md`

## 五、允许写入

- `prototypes/productized-desktop-shell/src/lib/humanize.ts`（新建）
- §2.2 表列七个前端文件（仅删函数体改 re-export；`App.tsx` 限 messageOf 族三处调用点+薄委托）
- `src-tauri/src/secretary_agent.rs`、`src-tauri/src/global_supervisor_agent.rs`（仅删壳改直调）
- `scripts/harness/workbench-shape-gate.js`、`scripts/harness/lib/`（新建规则文件如选解法 a）、`scripts/harness/workbench-shape-gate.machine-face.selftest.js`（新建）
- 受迁移影响的既有离线测试文件（仅导入路径随 re-export 保持则零改；断言文本逐字不得动）
- `docs/harness-catalog.md`、`decisions/2026-07-20-machine-face-gate-rule-and-defer-whitelist-v1.md`（新建）、本包 evidence（新建）
- `CURRENT.md` 最小回写（收口后）

## 六、禁止事项

1. **函数体逐字原样迁移**：任何人话输出串、判据、分支、兜底语义零变化；App notice 薄委托除外（其 unknown 回退必须=原文逐字）。
2. 零碰 `run_error_translation.rs` 本体、`codex_provider_unavailable:`/`consult_last_message_read_failed:` 两承重前缀、`classify_codex_resume_failure`、`director_agent.rs` retry 判读、`lib.rs:9939` 测试。
3. 不新增 Tauri command、不新增 sidecar JSON 种类、不碰 `lib.rs`。
4. 不收纯枚举状态标签族（`*Label`）、不动 §九挂账的重复簇、不做③清单、不建大翻译层。
5. 白名单只许登记 §2.4 列出的既有违规；新增违规零容忍（error 级）；不得为过关把新违规塞进白名单。
6. 不 stage、不 commit。
7. 离线测试断言文本逐字不动；若迁移导致断言失败=迁移破语义，修迁移不修断言。

## 七、变更辐射面

本变更改变「错误翻译函数住在使用处」的假设 → 依赖者逐个核：

- `ProjectJiaobanPanel.tsx`（棘轮文件）：迁出 4 函数=净减约 60 行，导入面靠 re-export 保持；离线测试经 Panel 导入的全链路透传。
- `AgentConversationShell.tsx` / `TranscriptViews.tsx` / `JiaobanHistory.tsx` / `JiaobanAuthorizeStates.tsx` / `JiaobanRunningStates.tsx`：同上。
- 离线断言：`tests/jiaoban-history-and-secretary-board.test.tsx:117/123-145`、`jiaoban-conversation-center.test.tsx` 等直接/间接断言人话输出的测试=行为闸，全绿才算语义保真。
- shape gate 新规则影响**之后所有包**：白名单格式、selftest、catalog 登记三件套是本包交付物；此后新增直渲=error。
- `App.tsx` notice 三处显示串变化（仅命中族时）=有意呈现修，evidence 须给出前后对照样例。

## 八、五态旅程走查

- 说：对话层错误脸不变（已是人话常量，§2.4 已核）。
- 批：authorize/preview 失败人话不变（函数原样迁移，调用点同款）。
- 干：transcript 错误区、live title/detail 不变；`TranscriptViews.tsx:428/603` 转录详情 stderr 进白名单（另包治理）。
- 交货：历史失败卡两层脸不变（合规格板，规则豁免下钻形）。
- 卡住：App notice 命中族时出人话（本包唯一有意呈现变），未命中逐字不变。

## 九、形状影响

- 任务类型：**治理任务包**（行为不变+形状改善）+一处 bounded 呈现修（App notice）。
- 新增代码落点：`src/lib/humanize.ts`（约 250-350 行，< 2000 新文件限）；`scripts/harness/lib/machine-face-rule.js`（如选解法 a）+ selftest。
- 棘轮文件：`ProjectJiaobanPanel.tsx` 净减（棘轮水位只降不升，顺向）；`workbench-shape-gate.js` 受 500 软限约束按 §2.3 处理；`lib.rs` 零碰。
- 预计行数：Panel −60 左右；`AgentConversationShell.tsx` −70 左右（含映射表）；其余五文件各 −10~-30；`humanize.ts` +300 左右；gate 规则+自测 +150 左右。
- 新增 Tauri command：无。新增 sidecar JSON：无。
- shape gate 豁免：`MACHINE_FACE_DEFER_WHITELIST` 条目全部登记 `decisions/2026-07-20-…-v1.md`，不沉默豁免。
- 本任务基线 commit：`24bde2e`。完成 commit：总指导核收后另定（执行线不 commit）。

## 十、验收标准

1. 四闸全绿：
   - `cd prototypes/productized-desktop-shell/src-tauri && cargo test --lib`（当前口径 1009/0/44，只增不减；若执行线判断迁移零风险可跑定向+全量各一遍）
   - `cd prototypes/productized-desktop-shell && npx tsc --noEmit` = 0
   - `node scripts/run-offline-interaction-test.mjs` 全绿（26 组注册套件）
   - `node scripts/harness/workbench-shape-gate.js --mode baseline` + `--mode check`：13/5/5 零净增（含 gate 自身软限不新增 warning）
   - `git diff --check` 通过
2. 迁移保真：grep 证明 §2.2 十函数在新模块、原位置 re-export；后端两壳 grep 清零、调用点直调。
3. 规则有效性：selftest 证明 `main.tsx:46` 形/`TranscriptViews.tsx:603` 形被拦、`<details>` 下钻形不误伤、state 形仅 warn。
4. 白名单与 decisions 条目一一对应；App notice 三处不进白名单（已治平）。
5. App notice 前后对照样例进 evidence（命中族出人话/未命中逐字不变各一例）。

## 十一、必须回传（按 TASK_TEMPLATE 10 项）

做了什么 / 改了哪些文件 / 新增哪些测试或证据 / 哪些结论有依据 / 哪些仍不确定 / 风险和下一步建议 / shape gate baseline+check 摘要 / start-end commit / 是否新增 command·sidecar·触碰棘轮文件 / **被闸拦过的事**（无也必须写「无」）。

## 十二、总指导回收动作

- 亲跑四闸不信回传；grep 迁移面与白名单逐条对实物；抽查三个人话输出串前后逐字一致。
- 判断 接受 / 需要修改 / 暂停 / 废弃，并记 `docs/harness-catch-log.md`。

## 十三、挂账（不进本包）

- 纯枚举状态标签族（`*Label` 约 167 个）收编：另包。
- 重复簇治理：`stateLabel`×3（`projectWorkflowLabels.ts:5`/`PermissionDialog.tsx:1200`/`RunningWorkflowsView.tsx:734`）、`confirmationKindLabel` 族×2（`RightDetailPanel.tsx:698/712/769/778` vs `RunningWorkflowsView.tsx:1046/1060/1086/1099`）、`automation*Label`×3、`readbackStatusLabel`×3：另包，清单起点即此节。
- `TranscriptViews.tsx:428/603` 转录详情 stderr 直渲、`main.tsx:46` 启动屏：白名单登记，随③清单另包治平。
- ③用户真机点名机器话清单：随走查攒（stage1 体检 §四.3 条目可作底稿）。
