# 决策：交办页设计方向=三栏归真（手账概念留作下一代参照）v1

日期：2026-07-20 · 用户拍板
关联：交互宪法 `decisions/2026-07-14-interaction-model-canon-v1.md`、修宪 3 号 `decisions/2026-07-19-interaction-canon-amendment-3-conversation-left-proposal-index-v1.md`、`prototypes/productized-desktop-shell/DESIGN.md`

## 起源

07-20 总指导前端审查（架构+视觉双审计，报告含 file:line）发现实现层漂离自己的正本：

- 视觉：`--bg:#f3f0ea` 已是运行时死值（被覆盖成 #f5f1e8/#f6f2e9）；≥1181px 桌面皮整体违反七律（白卡+朱红选中）且是用户主分辨率；88 token 四个命名时代、237 处硬编码 hex、21 种字号；styles.css 约 3000 行死壳+112 处文件内重复定义；spec-* 定式组件全 App 仅 1 处直连使用（事实行实有 4 种实现、pill 5 种）。
- 架构：App.tsx 上帝对象+47 变体 PendingAction 上帝开关（PermissionDialog 127 处 `kind===`）；五层 prop drilling；4 个模块级 Map 缓存模拟状态存活；读模型双轨+轮询刷新；1196 行死视图与占位面。

## 候选与拍板

- **A · 三栏归真**（样张 `prototypes/design-mockups/jiaoban-redesign-specimen-v1.html`）：保留修宪 3 号三栏布局，归真 token、七律落地、栏间发丝线、索引降级为条目（有框=正式文件，无框=索引）、批=盖章。
- **B · 项目手账**（概念稿 `prototypes/design-mockups/jiaoban-journal-concept-v1.html`）：抛开三栏的长文叙事形态。

**用户拍板：A。** 三栏方向不变，按样张归真；B 留档作下一代参照，不进当前排期。

随之拍定：

- **盖章=「批」的签名时刻**：石绿印章落纸（multiply 混合、-12°、按压动效、reduced-motion 退化），全 App 唯一一次重彩；朱砂只做黄牌/危险，不做选中色。
- **边框规矩**：栏间发丝线分隔；虚线只做占位；正式文件（方案卡/交货卡）有框，索引/对话无框。

## 拆包排期（视觉治理线，轻档·全部离线可验）

| 包 | 内容 | 验收要点 |
|---|---|---|
| **G1 · token 归真** | 单一 :root 正典、`--bg` 定回拍板值、桌面皮违规部分退役或治平、237 hex 归 token、字号/字重/等宽栈收敛、webfont 决断；shape gate 加「禁新硬编码 hex」机械规则（旧债白名单登记） | 行为不变+像素级前后对照；13/5/5 零净增 |
| **G2 · 定式扶正** | spec-* 扶正为唯一：事实行 4 式→FactRow、pill 5 式→Pill，迁移后删旧式；gate 加防再造规则；顺清 styles.css 死壳与重复定义 | 渲染断言全绿；死重前后行数对照 |
| **G3 · 盖章时刻** | 批准动作落印章签名（1280/577 两态+reduced-motion）；黄牌改朱砂页边批注式样 | 离线 DOM 断言+实渲量尺+用户最后一眼 |
| **G4 · 死重清扫** | RunningWorkflowsView 1196 行死视图、占位页、re-export 门面瘦身 | grep 引用清零+四闸 |

**架构两刀另议**（写操作注册表杀上帝开关、状态容器+事件订阅代轮询）：动行为面，不混视觉线，视觉线收口后单独排。

## 边界

- **不修宪**：三栏布局=修宪 3 号既定，G 线零布局变更；呈现层归真不触碰五态/打断/人闸。
- 每包行为不变+前后对照证据；样张是「形」参照，具体像素以七律与宪法为准。
- 与主线关系：H2 续验→底1 首单仍是主线（等 Codex 额度）；G 线轻档离线，可立即串行开 G1。
