# Root Treatment / R4-A10 Ink Style Assets v1

日期：2026-06-11

状态：已完成并通过复核线 `STATUS: CLEAR`。本文是 R4-A10 的风格资产清单，只记录当前 `styles.css` 已经存在的水墨 / 桌面壳资产和后续治理建议；不改变产品事实、不改 CSS、不重做 UI。

任务包：`tasks/2026-06-11-root-treatment-r4-a10-styles-css-ink-style-asset-inventory-v1.md`

## 1. 结论

当前 `styles.css` 已经形成了可复用的水墨桌面壳风格，但资产分布混杂：

- 顶部 `:root` 是早期后台式基础 token。
- 中段 `:root` 覆盖为水墨壳 token。
- `Source parity v2 refinements` 又增加右侧栏展开尺寸。
- 后段多个 `@media (min-width: 1181px)` 叠加了 UI redesign、inkwash shell、首页星图、真实 Tauri 数据清理和智能体桌面对话 workspace。

本轮建议：

```text
后续 CSS 治理先按“桌面壳 / 首页 / 智能体 / 项目画布 / 对象页 / 对话与权限 / 开发者详情”分区，再逐步把重复硬编码颜色收敛到 token；不要先改视觉，也不要直接套外部项目风格。
```

## 2. 文件指标

只读统计：

| 项 | 当前值 | 说明 |
| --- | ---: | --- |
| `styles.css` 行数 | 8,464 | `wc -l` 结果 |
| CSS 变量定义命中 | 107 | 包括重复覆盖和局部 `@media` 内 token |
| hex 颜色命中 | 100 | 不含 rgba / var |
| `rgba()` / `rgb()` 命中 | 234 | 大量用于纸面、墨线、状态弱化 |
| `@media` 命中 | 9 | 主要围绕 `1180px / 1181px` 桌面断点 |
| `@keyframes` 命中 | 1 | `pulse`，用于运行 / loading health dot |
| 顶层 class 规则命中 | 1,345 | 粗略指标，只用于说明文件体量 |

最密集硬编码 hex：

- `#fbfcf8`：24 次，常用作白纸 / 轻面板背景。
- `#d8d3c5`：4 次，常用作纸面线条或边界。
- `#2e2a25`：3 次，深墨文本。
- `#5b554b`：3 次，次级墨色。

这些值后续适合进入 token 或被现有 `--rice` / `--panel` / `--hair` 族吸收；R4-A10 不直接替换。

## 3. Token 分层

### 3.1 早期基础 token

位置：`styles.css` 第 1 行开始的 `:root`。

主要资产：

- Surface：`--bg`、`--bg-subtle`、`--panel`、`--panel-soft`、`--panel-raised`。
- Ink：`--ink`、`--ink-mid`、`--muted`、`--hair`、`--line`。
- Status：`--accent`、`--run`、`--warning`、`--danger`、`--candidate`、`--planned`、`--unknown`、`--ok` 及对应背景。
- Typography：`--text-xs` 到 `--text-2xl`。
- Spacing：`--space-1` 到 `--space-8`。
- Radius：`--r-sm`、`--r-md`、`--r-lg`。
- Shadow：`--shadow-sm`、`--shadow`、`--shadow-lg`。
- Shell dimensions：`--sidebar-w`、`--topbar-h`、`--dock-h`、`--right-rail-w`、`--right-panel-w`。

判断：

- 这层适合保留为兼容基础，但它的后台式视觉和后续水墨壳 token 已有语义重叠。
- 后续拆分时不要一次性删除；先把仍被通用组件使用的 token 标为 `legacy base`。

### 3.2 水墨壳 token

位置：`styles.css` 第 3205 行开始的 `:root`，注释为 `Inkwash shell replacement`。

主要资产：

- Paper / rice：`--rice`、`--rice-2`。
- Shell：`--shell`、`--shell-2`。
- Ink：`--ink-deep`、`--ink-mid`、`--ink-light`、`--ink-mist`。
- Hairline：`--hair`、`--hair-2`。
- Accent：`--vermil`、`--tea`、`--terra`。
- Compatibility mapping：把 `--bg`、`--panel`、`--ink`、`--muted`、`--line`、`--accent` 等映射到水墨壳。
- Shell dimensions：`--rail-left`、`--rail-right`、`--topbar-h`、`--dock-h`。

判断：

- 这是当前桌面工作台风格正本。
- 后续 token 化应优先以这层为主，而不是回到早期 `--accent: #2a6b5e` 的后台风格。

### 3.3 右侧栏展开 token

位置：`styles.css` 第 6138 行开始的 `:root`。

主要资产：

- `--rail-right-expanded: 320px`

判断：

- 这是局部 Source parity 修补，不应散落在通用 token 层。
- 后续应并入 shell dimensions 分区，形成 `right rail collapsed / expanded` 一组尺寸。

### 3.4 UI redesign 局部 token

位置：`styles.css` 第 7639 行 `@media (min-width: 1181px)` 内。

主要资产：

- `--ui-bg`
- `--ui-rail`
- `--ui-rail-strong`
- `--ui-surface`
- `--ui-surface-soft`
- `--ui-line`
- `--ui-line-strong`
- `--ui-text`
- `--ui-muted`
- `--ui-accent`
- `--ui-shadow`

判断：

- 这层是后期 shell 产品化校准，视觉接近当前桌面壳，但命名和水墨壳 token 并行。
- 后续治理应决定保留 `--ui-*` 作为桌面壳公开 token，还是把它们映射回 `--rice / --shell / --ink-* / --vermil`。
- 不建议同时长期保留两套主视觉 token，否则后续改一处会漏另一处。

### 3.5 Step 1 桌面壳尺寸 token

位置：`styles.css` 第 7855 行 `@media (min-width: 1181px)` 内。

主要资产：

- `--rail-left: 72px`
- `--rail-right: 58px`
- `--rail-right-expanded: 384px`
- `--topbar-h: 56px`
- `--dock-h: 64px`

判断：

- 这是当前桌面版壳层实际生效的尺寸基准。
- 它体现用户要求的“不要做手机端 UI”，后续 UI 验收应以桌面宽度为主。

## 4. 颜色资产

### 4.1 基础纸面色

当前可复用：

- 米纸背景：`--rice: #f5f1e8`
- 二级米纸：`--rice-2: #efeadd`
- 壳层纸面：`--shell: #ebe5d4`
- 壳层深纸面：`--shell-2: #e3dcc7`
- 白纸面板常见硬编码：`#fbfcf8`
- UI surface：`--ui-surface: #fffdf8`
- UI soft surface：`--ui-surface-soft: #f8f3e9`

使用原则：

- 主背景用 rice / shell，不用纯白大面积铺底。
- 内容卡片可以用 `#fbfcf8` / `--ui-surface`，但后续应统一 token。
- 项目画布和对话区允许轻微纸纹 / radial gradient，但不要再加装饰性 orb / bokeh。

### 4.2 墨色

当前可复用：

- 主墨：`--ink-deep: #1c1f24`
- 中墨：`--ink-mid: #4a4d54`
- 浅墨：`--ink-light: #8a8a85`
- 雾墨：`--ink-mist: #b6b3a8`
- UI text：`--ui-text: #1e2226`
- UI muted：`--ui-muted: #6c6d68`

使用原则：

- 页面标题和主内容用 `--ink-deep` / `--ui-text`。
- 辅助说明用 `--ink-mid` 或 `--ui-muted`。
- 内部状态、路径、边界说明不要用强色抢主工作对象。

### 4.3 线条和纸纤维

当前可复用：

- `--hair: rgba(28, 31, 36, 0.1)`
- `--hair-2: rgba(28, 31, 36, 0.18)`
- `--ui-line: rgba(32, 35, 39, 0.14)`
- `--ui-line-strong: rgba(32, 35, 39, 0.22)`

使用原则：

- 壳层分割线优先 dashed / thin hairline，不做厚边框卡片堆。
- 页面主区域保持纸面连续，卡片只用于重复项、详情面板、弹层和确实需要 framed 的工具。

### 4.4 语义色

当前可复用：

- 朱砂：`--vermil: #a14242`，主 accent / 当前选中 / 高风险提示。
- 茶绿：`--tea: #6e7f5b`，成功 / 可用 / accepted 倾向。
- 陶土：`--terra: #b87341`，warning / pending / 需要注意。
- 早期运行蓝：`--run: #1a5c8a`，运行中状态仍可保留，但要避免和水墨主风格冲突。
- unknown 紫灰：`--unknown: #5a5068`，读回不可用 / 降级状态。

使用原则：

- 高风险真实执行和 destructive action 才用朱砂强提示。
- `planned / unavailable / no credential / model unverified` 不应用成功色。
- `result_count = null` 必须显示未知 / 不可用，不得用空态视觉暗示 0 条。

## 5. 字体资产

当前主要字体：

- 主 UI：`"DM Sans", ui-sans-serif, system-ui, -apple-system, sans-serif`
- 中文产品 UI：`"Noto Sans SC"`、`"PingFang SC"`、系统 sans-serif
- 水墨标题 / 印章 / 高级标题：`"Noto Serif SC"`、`"Songti SC"`、`"STSong"`、serif
- 等宽：`"JetBrains Mono"`、`"SF Mono"`、`"DM Mono"`、`Menlo`、`Consolas`

使用原则：

- 普通操作界面用 sans-serif，保证密集信息可扫读。
- 水墨 serif 只用于品牌、页面标题、节点标题、印章感 glyph，不用于长段正文。
- 等宽只用于 id、状态码、命令 preview、代码块和内部详情；普通用户区应尽量减少等宽暴露。

## 6. 间距、圆角、阴影

当前资产：

- 间距：`--space-1` 到 `--space-8`，4px 基准。
- 圆角：早期 `--r-sm: 6px`、`--r-md: 8px`、`--r-lg: 12px`；后期桌面壳大量使用 `0`、`2px`、`6px`、`8px`、`10px`、`14px`、`16px`、`18px`。
- 阴影：早期 `--shadow-sm / --shadow / --shadow-lg`，水墨壳中 `--shadow: none`，UI redesign 又引入 `--ui-shadow: 0 18px 60px rgba(33, 31, 26, 0.08)`。

使用原则：

- 当前水墨壳更偏纸面和线条，阴影应少用。
- 主壳、左栏、右栏、底栏不应卡片化。
- 重复项目卡、弹层、重要详情可以使用轻阴影。
- 卡片圆角应克制；治理期后续可把常见 `8px / 14px / 16px` 收敛成 token。

## 7. 壳层和桌面断点

当前桌面壳基准：

- 桌面断点：`min-width: 1181px`。
- 窄栏：`--rail-left: 72px`。
- 右侧窄栏：`--rail-right: 58px`。
- 右侧展开：`--rail-right-expanded: 384px`。
- 顶栏：`--topbar-h: 56px`。
- 底栏：`--dock-h: 64px`。

边界：

- 当前 UI 只按桌面 Tauri 验收；`max-width: 1180px` 的规则只能作为极窄窗口防溢出回退。
- 后续不得把 R4-A10 解释为移动端适配依据。

## 8. 动效和状态反馈

当前资产：

- `@keyframes pulse` 用于 loading / running health dot。
- hover / focus transition 常见 `0.12s`、`0.16s`、`120ms`。
- 导航 hover 有轻微 translate，但后续 Step 1 壳层又把主导航 transform 收回。

使用原则：

- 动效只表达状态变化、焦点和展开，不做装饰性动画。
- 真实执行、权限确认、读回失败等高风险状态优先清晰文案和状态色，不靠动画制造紧张感。

## 9. 当前 CSS 债务

### 9.1 Token 重名和覆盖

`--bg`、`--panel`、`--ink`、`--muted`、`--line`、`--accent`、`--warning` 等在早期基础 token 和水墨壳 token 中语义被覆盖。

风险：

- 新开发者很难判断哪个 token 是当前桌面壳正本。
- 后续如果只改顶部 `:root`，桌面断点下可能不生效。

建议：

- 下一步 CSS 治理先建立 `style-token-map` 文档或注释区，标明 canonical token。
- 后续拆文件时把 legacy base 和 desktop inkwash shell 分开。

### 9.2 硬编码颜色重复

`#fbfcf8`、rgba 纸面、hairline 和状态色散落在多处。

风险：

- UI 细节会越来越不一致。
- 后续想调纸面亮度或主 accent 会漏改。

建议：

- 先只替换高频、无争议的纸面色和 line 色。
- 高风险状态色先不机械替换，避免改变语义。

### 9.3 补丁段落叠加

后段存在多个按历史任务追加的段落：

- UI redesign checkpoint。
- Step 1 inkwash shell align。
- Step 2 home constellation。
- Desktop data cleanup。
- Stage K / K1 agent desktop chat workspace。

风险：

- 后写规则依赖 cascade 覆盖，改前面可能看不出实际效果。
- 真实 Tauri 与浏览器 smoke 可能出现差异。

建议：

- R4 后续若拆 CSS，不按时间线拆，应按产品区域拆。
- 每个区域保留同一层级的 desktop override，避免跨段覆盖。

## 10. 后续 CSS 治理建议

建议拆分顺序：

1. `styles/tokens.css`：只放 canonical token、legacy compatibility mapping、desktop shell dimensions。
2. `styles/shell.css`：左侧栏、顶栏、右侧栏、底栏、stage frame、secretary affordance。
3. `styles/common.css`：按钮、输入框、notice、badge、metric、panel、dialog。
4. `styles/home.css`：首页星图和节点。
5. `styles/agent.css`：智能体对话 workspace、session list、transcript、composer。
6. `styles/projects.css`：项目列表、项目工作区、工作流画布、节点详情。
7. `styles/memory-knowledge.css`：记忆中心、知识库、对象化列表。
8. `styles/developer.css`：开发者详情、raw table、boundary panels。

治理顺序：

- 先文档化 canonical token。
- 再搬 shell 和 common，保证 `git diff` 可审。
- 每次搬运只做等价迁移，不改视觉。
- 每批跑 shape gate / typecheck / offline interaction；视觉改动任务另开 UI 任务包。

## 11. 不能声明

R4-A10 不能声明：

- UI 已重做。
- CSS 已拆分。
- `styles.css` 债务已解决。
- 视觉已通过真实 Tauri 验收。
- Xuanji / Mobbin / inkwash 参考已经落地为产品 UI。
- R4 已完成。
