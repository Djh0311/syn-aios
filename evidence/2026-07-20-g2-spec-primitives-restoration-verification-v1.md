# G2 · 定式扶正 验收证据 v1

日期：2026-07-20 · 轻档 · 执行线施工，总指导回收核实物 + 截图对照
任务包：`tasks/2026-07-20-g2-spec-primitives-restoration-package-v1.md`
基线 commit：`8f2f94d`（开工核对 HEAD=`8f2f94d983e39d6b7274ff8529b82fc1c692a2d7`，工作树仅任务包文等未跟踪件）
规则留痕：`decisions/2026-07-20-g2-spec-primitives-restoration-and-retired-style-family-gate-rule-v1.md`
截图：`prototypes/productized-desktop-shell/output/playwright/g2-spec-primitives/{before,after}/`（各 11 张·SHA-256 见目录 SHA256SUMS.txt）

## 一、结论

迁移+删旧+闸上线全做，四闸全绿，视觉差异全部落在 §九 7 项枚举内：

1. **Badge→Pill**：元素级 **103 处 / 31 文件**（包文「102」=行口径，HarnessBoardView:155 三元一行 2 元素；详见 §三口径对平）全部迁完，`components/Badge.tsx` 删除，全仓 `components/Badge` import 与 `<Badge` 双清零（grep 实证 §四）。tone 映射按 §三表逐条：neutral→plain、warning→warn、candidate→candidate、unknown→unknown。
2. **事实行三式→FactRow**：JiaobanAuthorizeStates 4 行（会改的文件/写入范围/不碰/完整路径）、MemoryDetailPanels 4 行（哪来的/和现有记忆/候选边界/证据）、SettingsView SettingFact 组件删 + 4 实例直写 + `.settings-fact-grid` 容器去除（唯一结构例外·已枚举）。
3. **pill 四式→Pill/PillRow**：Pill tone 枚举扩 7（+candidate/unknown/bad·色=G1 既有 token 零新色）；JiaobanDoneStates 5 处 step-badge + done-pills 容器→PillRow（动态两处走 `stepBadgePillTone` 查表）；PWCV prsb 2 处（`runcheckPillTone`）+ canvas-status-pill 2 处（`canvasBadgePillTone`）。退休族 CSS 块枚举删除全做。
4. **hex 白名单 42→39**：`#3f5235`/`#7a2e2e`/`#f7e8e8` grep 双清零（src + 白名单），decisions 同笔更新，只减不增。
5. **gate `retired_style_family` 上线**：本体 `lib/retired-style-family-rule.js`（scan+attach），gate +3 行（495→498 ≤500 软限）；selftest 13/13；catalog 登记（87 条）；decisions 落档；白名单 2 条全登记（不沉默豁免）。

## 二、四闸

| 闸 | 结果 |
|---|---|
| `npx tsc --noEmit` | 0 错 |
| `node scripts/run-offline-interaction-test.mjs` | 24 组全绿（exit 0） |
| shape gate baseline | **13/5/5 零净增**；新规则 0 violations / 2 deferred（白名单全中） |
| shape gate check | 13/5/5（exit 1=既有 13 error 同基线，无新增 finding） |
| `git diff --check` | exit 0 |
| cargo | 免跑：零 Rust 改动（git diff 无 .rs），包 §十三.1 允许 |

## 三、口径对平（包文数字 vs 实测，施工前复 grep 对平）

| 项 | 包文 | 实测 | 说明 |
|---|---|---|---|
| Badge 总数 | 102 处/31 文件 | **103 元素**/31 文件（102 行） | HarnessBoardView:155 三元一行 2 `<Badge>` 元素；31 文件逐文件计数除 HarnessBoardView（包 2/实 3）外全部对平；显式 tone 34（candidate 8+warning 8+unknown 15+neutral 3）全对平，默认 neutral 68 行/69 元素 |
| FactRow 基线 | 56 处/6 文件 | **47 元素**/6 文件（56=47 开+9 闭行口径） | 迁移后 59 元素（+12）；行口径 56→68 同成立 |
| ListRow | 5 | 5（grep 6 含 AgentSessionList:299 注释提及） | 对平 |
| sidePanel jiaoban-fact CSS | :1473-1489 | **1473-1491**（19 行） | 包文少算 `.jiaoban-fact-value` 收尾 2 行 |
| memoryCenter memory-kv CSS | :534-547 | 534-547（连尾随空行删至 548） | 对平 |
| styles.css done-pills | :9610-9616 | 9609-9616 | 9609 注释行引用退休类名 `jiaoban-step-badge`，一并删（披露） |
| prsb-pill 宿主 | 调用点 2 span | `ProjectRuleStatusBar` **零调用=死组件** | 迁移照做（防再造闸覆盖）；截图面无实体 |
| canvas-status-pill | 调用点 2 span | 两渲染点均在 `.project-canvas-fullbleed-stage` 内 `display:none`（styles.css:8577 区·生产不可见） | 同上 |

## 四、迁移对账 grep 清零表（逐条）

| 项 | 命令口径 | 结果 |
|---|---|---|
| `components/Badge.tsx` | 文件删除 | ✅ 已删（git status D） |
| Badge import | `grep -rn 'components/Badge' src tests` | 0 |
| `<Badge` | `grep -rEo '<Badge' src --include='*.tsx'` | 0 |
| `badge` 精确/`badge-row`（class 面） | 见下注 | 0（残留 3 处合法复合名：`spec-list-badge`/`rail-icon-badge`/`sc-badge`，词界带连字符排除，闸不误伤） |
| `jiaoban-step-badge` | src 全扩展名 | 0 |
| `project-canvas-status-pill` | 同上 | 0 |
| `prsb-pill` | 同上 | 0（PWCV:886 注释提及族名·闸只扫字符串面不吃注释） |
| `jiaoban-done-pills` | 同上 | 0 |
| `jiaoban-fact` 精确 | 同上（排 -btn/-done） | 0（styles.css:10058 注释提及·闸跳注释行；`.jiaoban-fact-btn`/`-done` 保留面未碰） |
| `memory-kv` | 同上 | 0 |
| `settings-fact`（含 -grid） | 同上 | 0 |
| FactRow | 元素 | 47 → **59**（行口径 56→68，与包 §十三.3 同数） |
| Pill | 元素/行 | 0 → **112 元素 / 111 行**（包 §十三.3=111 行口径同数；元素 +1=HarnessBoardView 三元双元素） |
| PillRow | — | 0 → **1** |
| hex 3 值 | `grep -rn '#3f5235\|#7a2e2e\|#f7e8e8' src` | 0；白名单 42→39 |

## 五、CSS 行数对照（§十二预估 vs 实测）

| 文件 | 预估 | 实测 |
|---|---|---|
| styles.css | ≈ −91（删 95/增 3  pill tone） | **−94**（+3/−97：badge 系 31+canvas pill 24+prsb 29（含空行）+done-pills 8（含注释行）+共享成员收编 5；增 3 tone 行） |
| sidePanel css | −39 | **−41**（step-badge 22 + jiaoban-fact 19，见 §三口径） |
| memoryCenter css | −14 | **−15**（块 14+尾随空行 1） |
| workbench-shape-gate.js | +≤3 | **+3**（495→498） |
| TSX | 净减（Badge.tsx −10、SettingFact −7、容器 −1） | 全 diff +264/−425（44 文件；含 gate/decisions/catalog） |

## 六、selftest

既有三件套不回归：machine-face **18/18**、hardcoded-hex **13/13**、dedup **8/8**；新规则 retired-style-family **13/13**（退休族 8 族 tsx/css→error、Badge import→error、复合名/保留名→不误伤、白名单→deferred 2、spec-* 直连 4 类→error、SpecPrimitives 本体→不误伤、干净树→0）。

## 七、截图对照（差异仅限 §九 枚举）

`output/playwright/g2-spec-primitives/` 前后各 11 张（生产夹具+生产 CSS；app 面=浏览器预览壳；交货卡/画布/审计=/tmp esbuild 夹具照 G1 先例·夹具不入库）：

| 面 | 前后 | 差异（枚举项） |
|---|---|---|
| 交办×1280 / ×577 | **像素级一致** | §九.4 近似等价落地实证（补 tabular-nums/word-break 在本夹具值下无渲染差） |
| 交货卡×1280 | 异 | §九.5+§九.1：已交货/完成 3 步/⚠1 项/只读单/步条自述 全收 spec-pill 形；绿/黄/灰 硬编码→ok/warn/plain token（朱砂系 bad 本夹具无实体） |
| 记忆中心详情（candidate/lint）×1280 | 异 | §九.3：kv 行→标准事实行（标签灰左+细虚线+值右），形正不歪 |
| 设置页×1280 | 异 | §九.2：4 数值卡→4 标准事实行（最大视觉差面），形正不歪；头 pill 同 token 收小 |
| 画布页×1280 | 异 | §九.6/§九.1：底栏 draft Badge→Pill（candidate）；status pill/prsb 生产不可见（§三披露）无实体可比 |
| 记忆中心列表×1280 | 异 | §九.1：顶 3 pill 同 token 收小 |
| 智能体页×1280 | **像素级一致** | 主视口无 Badge 实体（该面 Badge 在边界面板/转录深层） |
| 审计账本×1280 | 异 | §九.1：3 筛选 pill 同 token 收小 |
| 首页×1280 | **像素级一致** | HomeView 无 Badge 引用（本来就不在 31 文件） |

差异全部落 §九 7 项；设置页/记忆详情两重点面形正（已逐张看）。3 张像素级一致面=无实体/近似等价，如实披露不作差异充数。

## 八、枚举外事项与被闸拦过的事

1. **PillRow 容器 aria-label**：`.jiaoban-done-pills` 自带 `aria-label="这单概览"`，PillRow 定式组件无 aria 面，随枚举的容器迁移落地后该 aria-label 不再输出（grep 全仓零引用、离线断言零依赖）。结构变化=包 §4.3.2 枚举项本体，未自扩组件 API；总指导若要保 aria 可另拍。
2. **styles.css:10058 注释**仍提「对齐样板 .jiaoban-fact」（历史注·闸跳注释行不吃）；零文案改纪律下未动，挂 G4 死重清扫顺手件。
3. **step-badge 退休连带 3 hex 白名单条目转休眠**：`#a86a00`/`#b23b3b`/`#666`（sidePanel 已零引用；`#2e7d4f` 另有 2 处活引用保留）。包 §4.4 只枚举 prsb 3 条核销，故 dormant 留单（只减不增不破），挂 G4 核销。
4. **死组件披露**：`ProjectRuleStatusBar`（prsb 宿主）零调用、`canvas-status-pill` 生产不可见（display:none 上下文）——迁移+退休 CSS 照包执行，G4 死重清扫候选加这两条。
5. **包文行号差**：sidePanel jiaoban-fact CSS 块 :1473-1489→实 1473-1491；done-pills :9610-9616→实 9609-9616（注释行）。按实测块删，断口逐块复核（§五）。
6. 被闸拦过的事：**无**（施工后四闸直过；selftest 首轮 2 断言因去重粒度设计（badge-row 并入 badge 族、spec-pill tone 归一族）预期写错，改断言不改规则——规则行为即包 §五.2 族口径）。

## 九、红线自查

零文案改、零逻辑改（tone helper 返回值按 §三表映射·无分支新增；lib/memoryCenter.ts badge_tone 未碰·查表在视图层）、零 Rust、零测试/夹具改；白名单只减不增（86→42→39）；未 stage、未 commit；视觉差异全部落 §九枚举（§七对照）；start = end = `8f2f94d983e39d6b7274ff8529b82fc1c692a2d7`。
