# 决策：G2 定式扶正 + shape gate 新增 retired_style_family 机械规则 v1

日期：2026-07-20
任务包：`tasks/2026-07-20-g2-spec-primitives-restoration-package-v1.md`（G2 定式扶正·轻档）
法源：`prototypes/productized-desktop-shell/DESIGN.md` §三·五（2026-07-14 用户逐字段拍板：pill 唯一形=spec-pill 11px/2px 9px/全圆角/语义 token 色；事实行唯一形=spec-fact-row 标签灰左/值右/细虚线/tabular-nums）
上位：`decisions/2026-07-20-jiaoban-design-direction-three-column-truth-restoration-v1.md`（G2 行）

## 拍板

1. **spec-* 扶正为唯一**：
   - 事实行其余 3 式迁入 `FactRow`：`jiaoban-fact`（JiaobanAuthorizeStates 4 行）、`memory-kv`（MemoryDetailPanels 4 行）、`settings-fact`（SettingsView SettingFact 组件删 + 4 实例直写 FactRow + `.settings-fact-grid` 容器去除=唯一结构例外枚举项）。
   - pill 其余 4 式迁入 `Pill`/`PillRow`：`Badge`（103 处/31 文件·包文 102 为行口径见 §口径对平）、`jiaoban-step-badge`（5 处）、`project-canvas-status-pill`（2 处）、`prsb-pill`（2 处·宿主 ProjectRuleStatusBar 实测零渲染=死组件·挂 G4）；`jiaoban-done-pills` 容器迁 `PillRow`（1 处）。`running-status-pill` 随死视图留 G4（本包不动）。
   - `Pill` tone 枚举扩 7：`plain/ok/warn/run` + `candidate`（--candidate-bg/--candidate）、`unknown`（--unknown-bg/--unknown）、`bad`（--danger-bg/--danger），色值全取 G1 正典既有 token，零新色。
   - tone 映射严格按包 §三表逐条（neutral→plain、warning→warn、candidate→candidate、unknown→unknown；step-badge green→ok/yellow→warn/red→bad/gray→plain；canvas ready/accepted→ok、running→run、warning/blocked/failed→warn；prsb 无 tone→plain、warning→warn、runcheck.runnable→ok、runcheck.warning→warn、runcheck.blocked→bad）。动态调用点走本地查表对象（`stepBadgePillTone`/`canvasBadgePillTone`/`runcheckPillTone`/`badgePillTone`），不另造分支语义。
   - 迁移后删退休式：`components/Badge.tsx` 删除；退休族 CSS 块枚举删除（styles.css badge 系/badge-row 共享成员/settings-fact 四共享成员/canvas-status-pill 系/prsb-pill 系/jiaoban-done-pills；sidePanel step-badge 系/jiaoban-fact 系；memoryCenter memory-kv 系）。`.jiaoban-fact-btn`/`.jiaoban-fact-done`/`.jiaoban-step-row tone-*`/`.project-rule-status-bar`/`.prsb-headline`/`running-status-pill` 保留不动。

2. **shape gate 新增 `retired_style_family`**（防再造）：
   - 本体拆 `scripts/harness/lib/retired-style-family-rule.js`（照 machine-face-rule 形：scan+attach）；gate 本体 +3 行（require/挂载/打印），495→498 行 ≤500 软限。
   - 扫描面 src/** .tsx/.ts（字符串字面量面）+ .css（原始行跳注释）；两类 error：退休族再现（精确类名 jiaoban-fact〔不匹配 -btn/-done〕/memory-kv/settings-fact〔含 -grid〕/badge〔精确词界·不匹配 sc-badge 等复合名·badge-row 同属退休〕/jiaoban-step-badge/project-canvas-status-pill/prsb-pill/jiaoban-done-pills/running-status-pill〔仅 ts/tsx 引用面·styles.css 定义段留 G4〕+ 对 components/Badge 的 import）；spec-* 直连（tsx 字符串面含 spec-fact-row/-k/-v、spec-pill〔含各 tone·归一族去重〕、spec-pill-row、spec-seg-title、spec-list-row/-badge/-claim/-time、spec-empty、spec-expand、spec-bad，文件 ≠ SpecPrimitives.tsx；spec-scroll 不在列）。
   - 去重粒度 pattern|path（同族同文件取首命中行·计数进 detail.count）。
   - **白名单 2 条（本档登记·不沉默豁免）**：
     ① `running-status-pill|prototypes/productized-desktop-shell/src/views/RunningWorkflowsView.tsx`——死视图（1196 行）引用面，G4 整删时连带清；
     ② `spec-direct:spec-empty|prototypes/productized-desktop-shell/src/components/ActiveWorkbenchView.tsx`——:277 有意例外（想法箱空态文案「。」「；」混排与基座 what/next 拼接逐字对不上·不改基座·该处注释在档）。
   - 白名单纪律照 hex 先例：只减不增，不得为过关塞新违规。

3. **hex 白名单核销**：prsb 迁移退休 `#3f5235`/`#7a2e2e`/`#f7e8e8` 3 条（styles.css prsb-pill 系随删），白名单 42→39，同步落 `decisions/2026-07-20-hardcoded-hex-gate-rule-and-whitelist-v1.md`，只减不增。step-badge 退休连带 `#a86a00`/`#b23b3b`/`#666` 3 条白名单条目转休眠（sidePanel 已无引用·`#2e7d4f` 另有 2 处活引用保留），挂 G4 死重清扫核销。

## 口径对平（包文数字 vs 实测）

- Badge「102 处」=行口径；元素级实测 **103**（HarnessBoardView:155 三元一行 2 元素），tone 分布按行口径全对平（显式 34=candidate 8+warning 8+unknown 15+neutral 3，默认 68 行/69 元素）。
- FactRow「56 处」=含闭合标签行口径（47 开 + 9 闭）；元素级基线 **47** → 迁移后 **59**（+4 授权卡 +4 记忆 +4 设置）；行口径 56→68 同成立。
- sidePanel jiaoban-fact CSS 块实测 1473-1491（包文 :1473-1489）；memoryCenter memory-kv 块 534-547（与包文一致，连尾随空行删至 548）；styles.css jiaoban-done-pills 块连注释 9609-9616（包文 :9610-9616·注释引用退休类名一并删）。
- `ProjectRuleStatusBar`（prsb-pill 宿主）实测零调用=死组件；`project-canvas-status-pill` 两渲染点均在 `.project-canvas-fullbleed-stage` 内 `display:none`（生产不可见）——两族迁移照做（防再造闸覆盖），截图面无对应实体，披露在证据。

## 纪律

- 零文案改、零逻辑改、零 Rust 改、零测试/夹具改；结构例外仅 settings grid 容器去除一项（已枚举）。
- 视觉变化仅限包 §九 7 项枚举。
- selftest：`scripts/harness/workbench-shape-gate.retired-style-family.selftest.js`（13 断言：退休族 8 族 tsx/css→error、Badge import→error、复合名/保留名→不误伤、白名单→deferred 2、spec-* 直连→error、SpecPrimitives 本体→不误伤、干净树→0）。
- 施工后实测：新规则 violations 0、deferred 2（白名单全中）、shape 13/5/5 零净增、既有 selftest 18/13/8 不回归。
