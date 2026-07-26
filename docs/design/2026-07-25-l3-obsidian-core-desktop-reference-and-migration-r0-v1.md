# L3 Obsidian 核心桌面参考与 Syn 单壳迁移表 R0 v1

- 日期：2026-07-25
- 类型：N2R 设计参考与只读迁移审计，不是开发任务包或验收 evidence
- 状态：**R0_REFERENCE_CAPTURED / N2R_IMPLEMENTATION_NOT_AUTHORIZED**
- 上游决策：`decisions/2026-07-23-l3-syn-native-knowledge-workspace-route-v2.md`
- 实施计划：`docs/plans/2026-07-23-l3-syn-native-knowledge-workspace-small-stage-plan-v2.md`
- Syn 设计语言：`docs/design/2026-07-14-syn-redesign-design-brief-v1.md`

## 1. 本轮结论

当前 Syn 知识库不是一个连续桌面工作区。`KnowledgeBaseView` 先后渲染：

1. 页面级标题；
2. `NativeKnowledgeWorkspace`；
3. `KnowledgeGraphView`；
4. `KnowledgeCanvasView`；
5. `KnowledgeWorkspaceMaintenancePanel`；
6. 旧统计条和旧知识三栏；
7. 第二套 `KnowledgeVaultNotesPanel`；
8. 页面级警告。

因此用户看到的“双显示容器”是代码结构事实，不只是视觉感受；实际竞争主层级超过两个。N2R 的核心不是再造知识能力，而是把 N0-N5 已有能力迁入唯一桌面工作台，并删除重复主层级。

本轮已冻结 Obsidian 核心桌面结构、交互基线、Syn 品牌替换规则和现有功能迁移归属，并在官方 Obsidian `1.12.7`、Default 主题、light mode、16 px 正文、0 应用缩放的真实 macOS 界面中形成一套 `984 × 768` 参考截图和实测几何。

这只完成了 R0 参照冻结。Syn 尚未开始 N2R 单壳实现，因此仍不得宣称“1:1 完成”“像素级通过”或“高保真验收通过”。

## 2. 参考基线

### 2.1 版本与捕获条件

- 行为与结构基线：Obsidian Desktop `1.12.7` Public。
- 观察项：`1.13.3` Desktop Catalyst 只作为后续变化观察，不进入当前实现基线。
- 平台：macOS 桌面版。
- 插件：只启用实现基准所需的官方核心插件；不使用社区插件或社区主题。
- 主题：Obsidian Default；基础颜色由“跟随系统”固定为 light；正文设置值 `16 px`，应用缩放设置值 `0`。
- 实际参考截图：整窗逻辑像素 `984 × 768`。这是本轮真实捕获条件，不冒充 `1440 × 900` 或 `1180 × 760`。
- Syn 后续视觉验收窗口：`1440 × 900 CSS px` 基准、`1180 × 760 CSS px` 紧凑；它们是待实现后的 Syn 测试视口，不是本轮 Obsidian 实图尺寸。
- 演示数据：只使用 `/private/tmp/Syn Obsidian R0 Demo` 无隐私演示 vault，包含合成 Markdown、属性、双链、搜索结果、Graph 和 Canvas。
- 安全说明：Obsidian 首次打开时自身短暂恢复了既有默认 vault 的导航界面；没有打开、展开或读取其中笔记正文，随即切换到上述演示 vault。所有归档截图均来自演示 vault。
- 来源：通过 Homebrew 官方 cask 元数据取得 Obsidian 官方 GitHub release DMG；下载文件 SHA-256 为 `3b85c13b4ce55512e86e170a7cd2a494e2db695ac888c0601e153cb85b77881b`，与 cask 元数据一致。
- 完整性保留项：安装包内 App 与安装后的 App 均显示 TeamIdentifier `6JSW4SJWN9` 和 stapled notarization ticket，但本机 `codesign --verify` 对两者都返回 `invalid signature (code or signature have been modified)`。未绕过 Gatekeeper，应用通过正常系统路径启动；该异常不被写成“签名验证通过”。

官方参考：

- [Obsidian 1.12.7 Desktop (Public)](https://obsidian.md/changelog/2026-03-23-desktop-v1.12.7/)
- [Obsidian 官方下载](https://obsidian.md/download)
- [Homebrew Obsidian cask 元数据](https://formulae.brew.sh/api/cask/obsidian.json)
- [Obsidian 1.13.3 Desktop (Early access)](https://obsidian.md/changelog/2026-07-21-desktop-v1.13.3/)
- [Workspace](https://obsidian.md/help/workspace)
- [Sidebar](https://obsidian.md/help/sidebar)
- [Ribbon](https://obsidian.md/help/ribbon)
- [Tabs](https://obsidian.md/help/tabs)
- [Status bar](https://obsidian.md/help/status-bar)
- [Graph view](https://obsidian.md/help/plugins/graph)
- [Canvas](https://obsidian.md/help/plugins/canvas)
- [Workspaces](https://obsidian.md/help/plugins/workspaces)
- [Properties](https://obsidian.md/help/properties)

### 2.2 实图参考集

| 文件 | 冻结内容 |
| --- | --- |
| [01 workspace / note / graph / backlinks](assets/2026-07-25-obsidian-r0/01-workspace-note-graph-backlinks-984x768.jpg) | 唯一桌面壳、活动栏、文件树、编辑态、Graph、右侧反链和状态区 |
| [02 reading / graph / backlinks](assets/2026-07-25-obsidian-r0/02-reading-graph-backlinks-984x768.jpg) | 阅读态与同组 Graph、右侧上下文 |
| [03 search / reading / graph / backlinks](assets/2026-07-25-obsidian-r0/03-search-reading-graph-backlinks-984x768.jpg) | 左栏搜索结果替换文件树，不新增容器 |
| [04 canvas / graph / context](assets/2026-07-25-obsidian-r0/04-canvas-graph-context-984x768.jpg) | Canvas 与 Graph 作为中央标签组，不纵向堆叠 |
| [05 command palette](assets/2026-07-25-obsidian-r0/05-command-palette-overlay-984x768.jpg) | 命令面板为临时 overlay |
| [06 quick switcher](assets/2026-07-25-obsidian-r0/06-quick-switcher-overlay-984x768.jpg) | 快速切换为临时 overlay |
| [07 appearance baseline](assets/2026-07-25-obsidian-r0/07-appearance-default-light-984x768.jpg) | `1.12.7`、Default 主题、light、16 px、0 缩放 |
| [08 collapsed left sidebar](assets/2026-07-25-obsidian-r0/08-left-sidebar-collapsed-ribbon-visible-984x768.jpg) | 左侧栏折叠后活动栏仍保留 |

所有截图均为 `984 × 768` JPEG；摘要见 [SHA256SUMS](assets/2026-07-25-obsidian-r0/SHA256SUMS)。视觉实现时以 01、03、04、05、06、08 为结构和交互主参照，以 07 固定环境条件。

### 2.3 `984 × 768` 参考几何

以下数值从 01/08 截图的分隔线和设置值测得，属于这次固定窗口的实现参考，不应按比例盲目拉伸：

| 区域 | 参考坐标/尺寸 |
| --- | --- |
| 整窗 | `984 × 768` |
| 集成标签/窗口顶栏 | `y = 0–38`，约 `39 px` |
| 视图工具栏 | `y = 39–73`，约 `35 px` |
| 最左活动栏 | `x = 0–41`，约 `42 px`；左栏折叠后仍存在 |
| 左侧栏展开态 | `x = 42–329`，约 `288 px` |
| 中央第 1 标签组 | `x = 330–564`，约 `235 px` |
| 中央第 2 标签组 | `x = 565–798`，约 `234 px` |
| 右侧上下文栏 | `x = 799–983`，约 `185 px` |
| 中央底部状态区 | 约 `26 px` 高 |
| 左栏 vault/footer 区 | 约 `41 px` 高 |
| 正文设置 | `16 px`；应用缩放 `0` |

关键裁决：截图中的“两块内容”是同一个中央工作区里的两个可调整标签组，不是两个独立全宽页面容器。折叠左栏时，中央工作区横向接管空间，活动栏不消失。

### 2.4 已观察的交互

- 左右侧栏可独立折叠；左栏折叠后活动栏保持在 `42 px` 宽参考带内。
- 文件、搜索在左侧同一侧栏切换；反链、出链、标签、属性和大纲在右侧同一上下文栏切换。
- 编辑/阅读、Graph、Canvas 都位于中央标签组；标签组可并排。
- 命令面板和快速切换均为覆盖层，关闭后返回原工作区。
- 实测快捷键入口包括：快速切换 `⌘O`、命令面板 `⌘P`、搜索 `⌘⇧F`、Graph `⌘G`、新标签 `⌘T`、下一标签 `⌃Tab`。

## 3. 设计意图

- 使用者：在 macOS 上长时间浏览、检索、编辑和关联知识的单人桌面用户。
- 核心任务：在文件、笔记、链接关系、Graph 和 Canvas 之间持续切换而不离开上下文。
- 唯一视觉焦点：中间内容工作区。
- Syn 签名：确认式 AI 写入、审计状态和主管引用进入右侧上下文、底部状态或临时确认层，不另建知识仪表盘。
- 视觉语言：沿用 Syn 已拍板的暖纸底、青绿状态色和宋黑分工；状态色是唯一情绪来源。
- 密度：桌面工具密度，壳固定、容器内滚；不使用大卡片、营销式留白或重复大标题。

明确拒绝：

- 卡片仪表盘；
- 多个全宽知识模块纵向堆叠；
- 两套文件列表、两套编辑器或两个主标题区；
- Graph、Canvas、维护区作为编辑器下方的第二层页面；
- 为了相似而复制 Obsidian 商标、品牌图形、原始图标包或私有资产。

## 4. 冻结的单壳结构

```text
┌────────────────────────────────────────────────────────────────────┐
│ 应用标题 / 当前 vault / 窗口动作                                  │
├───┬────────────────┬────────────────────────────┬─────────────────┤
│活 │ 左侧栏         │ 中间标签组工作区           │ 右侧上下文栏    │
│动 │ 文件           │ Markdown / Preview         │ Properties      │
│栏 │ 搜索           │ Graph / Canvas             │ Backlinks       │
│   │ 标签 / 来源    │ 可左右或上下分栏           │ Outline / Context│
├───┴────────────────┴────────────────────────────┴─────────────────┤
│ 低层级状态：当前文件 / 模式 / 保存 / 冲突 / 字数 / 引用状态       │
└────────────────────────────────────────────────────────────────────┘
```

### 4.1 活动栏

- 固定在最左侧；左侧栏折叠后仍可见。
- 使用图标触发文件、搜索、Graph、Canvas、命令、设置等高频动作。
- 支持 tooltip、active、hover、focus-visible 和 disabled。
- 可重排/隐藏属于后续增强；本轮实现至少保持固定顺序和稳定语义。

### 4.2 左侧栏

- 文件、搜索、标签、来源是同一侧栏内的互斥标签视图，不并排制造多个列表。
- 支持折叠和宽度调整；状态可恢复。
- 文件树保留新建目录/笔记、展开、选择和上下文动作。
- 搜索结果在左栏呈现，点击后只打开或聚焦中间标签。

### 4.3 中间标签组

- Markdown 编辑、预览、分栏、Graph、Canvas 都是中间标签或工作模式。
- 标签可打开、关闭、切换、重排；支持向右/向下分栏和调整分栏尺寸。
- 当前选中内容决定右侧上下文；Graph/Canvas 不再额外占据页面高度。
- 中间区域是唯一主滚动和主要视觉焦点。

### 4.4 右侧上下文栏

- 属性、反链、大纲、当前来源/引用上下文是互斥或上下分组标签。
- 可折叠、调整宽度和固定当前上下文。
- AI 提议、引用来源和审计摘要只在与当前笔记相关时进入这里；需要确认的写动作仍走既有确认层。

### 4.5 底部状态区

- 只承载低层级、短文本状态：当前路径、编辑模式、保存中/已保存、冲突、字数和引用状态。
- 错误需要行动时进入 toast、对话框或右侧上下文，不把状态栏扩成第二工具区。

### 4.6 滚动与响应

- `KnowledgeBaseView` 占满应用壳分配的可用高度，外层 `overflow: hidden`。
- 活动栏不滚动；左栏列表、中间内容、右栏内容分别内部滚动。
- 基准和紧凑窗口继续维持桌面三段结构；紧凑窗口优先折叠右栏或左栏，不把侧栏堆到编辑器下方。
- 移动端不在本阶段范围内。

## 5. 核心交互基线

以下行为按 Obsidian 官方桌面模型冻结，但使用 Syn 自有文案、图标与状态：

1. 左右侧栏可独立折叠和调整宽度；活动栏在左栏折叠后仍保留。
2. 中间标签属于标签组，可重排、移动、向右/向下分栏和调整尺寸。
3. 文件、搜索、Graph、Canvas、属性、反链和大纲只改变现有工作台中的视图或标签，不追加新页面容器。
4. 当前笔记切换时，反链、属性、大纲和局部图随上下文更新；固定视图保留其绑定目标。
5. 快速打开和命令面板使用临时 overlay，完成或取消后焦点回到原控件。
6. 工作区恢复保存标签、分栏、侧栏可见性/宽度和当前选择；它是可重建偏好，不是知识真相源。
7. loading、empty、error、conflict、disabled、hover、active、focus-visible 和 reduced-motion 均有显式状态。
8. 未保存草稿、外部变化和 CAS 冲突继续 fail closed；界面重构不得静默覆盖。

## 6. 当前双容器迁移表

| 当前实现/区域 | 当前问题 | N2R 唯一归属 | 处理 |
| --- | --- | --- | --- |
| `KnowledgeBaseView > .pg-head` | 页面级大标题占用工作区高度 | 应用 chrome / 当前 vault 标题 | 从知识内容区移除或并入应用标题 |
| `NativeKnowledgeWorkspace > header` | 第二个大标题区 | 中间标签组标题或空态 | 删除独立 header |
| 快速打开 | 作为工作台内常驻工具行 | 临时命令 overlay + 左侧搜索 | 迁移，完成后恢复焦点 |
| `native-workspace-spine` | 只是一列，不是完整侧栏标签模型 | 左侧栏的“文件”视图 | 保留树能力，重组外壳 |
| `native-workspace-paper` | 已有编辑/预览能力但被卡片包裹 | 中间标签组 | 保留能力，移除卡片层 |
| `native-workspace-margin` | 属性/反链常驻混排 | 右侧上下文栏 | 拆成 Properties / Backlinks / Outline 标签 |
| `native-workspace-tools` | 保存、命令和警告混在一行 | 状态栏 / 命令面板 / toast | 按语义拆分 |
| `KnowledgeGraphView` | 编辑器下方独立全宽容器 | 中间 Graph 标签或局部 linked view | 迁移，不再纵向渲染 |
| `KnowledgeCanvasView` | 编辑器下方独立全宽容器 | 中间 Canvas 标签 | 迁移，不再纵向渲染 |
| `KnowledgeWorkspaceMaintenancePanel` | 附件、备份、恢复长期占据主页面 | 设置、命令和按需抽屉 | 保留能力，移出主壳常驻层 |
| `.knowledge-base-stats` | 仪表盘统计与写作任务竞争焦点 | 状态区、来源视图或次级页面 | 从主工作区移除 |
| 旧 `.knowledge-document-list` | 与文件树形成第二列表 | 左侧栏“来源”视图 | 迁移唯一业务信息 |
| 旧 `.knowledge-detail-panel` | 与右侧属性/反链形成第二详情 | 右侧“来源上下文”标签 | 迁移来源锚点和引用 |
| 旧 `.knowledge-boundary-panel` | 记忆捕获成为第三主栏 | 记忆中心或次级 route | 不留在知识主壳 |
| `KnowledgeVaultNotesPanel` | 第二套 vault 编辑器 | 中间唯一 Markdown 编辑器 | 核对独有能力后退场 |
| Obsidian 兼容/bridge 状态 | 可选互操作被误当主状态 | 设置或显式“外部打开”动作 | 默认隐藏于主工作区 |
| `.knowledge-warning-list` | 页面底部追加并拉长页面 | toast、状态栏或右侧上下文 | 按严重度迁移 |

## 7. 不重复合同

完成 N2R 后同时满足：

- DOM 中只有一个知识工作台根；
- 一个文件树来源；
- 一个 Markdown 编辑状态来源；
- 一个标签/分栏模型；
- 一个右侧上下文模型；
- Graph、Canvas、编辑和预览共享同一中间区域；
- 旧统计、旧三栏、Vault Notes 和维护面板不再形成第二主容器；
- 不以 CSS 隐藏重复容器冒充迁移完成；
- 删除旧渲染前，逐项证明独有业务行为已迁移或明确移到次级 route。

## 8. Syn 品牌替换与不复制资产

### 保留为 Syn

- 产品名、vault 名、文案、确认提示和错误提示；
- 暖纸底、青绿状态色、宋黑分工和既有状态语义；
- 现有通用语义图标或重新绘制的自有通用线性图标；
- 确认式 AI 写、冲突保护、审计与主管只读引用；
- 固定 Syn vault、typed host command、路径限制和 CAS。

### 不复制

- Obsidian 名称、商标、logo 和品牌图形；
- Obsidian 原始图标包、CSS、源码、私有命令 ID 和内部工作区文件；
- 社区插件、社区主题、Sync、Publish、移动端或插件市场；
- 任何需要解包、逆向、改签、注入或绕过平台保护的资产和机制。

## 9. N2R 实现验收矩阵

| 维度 | 必须证明 | 当前状态 |
| --- | --- | --- |
| 结构 | 单壳、活动栏、左栏、中间标签组、右栏、状态栏 | 合同已冻结，未实现 |
| 去重 | 旧统计/三栏/Vault Notes/Graph/Canvas/维护区不再纵向竞争 | 迁移表已冻结，未实现 |
| 交互 | 标签、分栏、侧栏、overlay、焦点与恢复 | 合同已冻结，未实现 |
| 状态 | loading/empty/error/conflict/disabled/focus/reduced-motion | 合同已冻结，未实现 |
| 视觉 | 同条件截图、尺寸、密度、分隔、图标和色阶对照 | R0 参考已捕获；Syn 尚未实现或对照 |
| 安全 | vault、路径、CAS、audit、MCP allowlist 均不放宽 | 既有合同，实施时回归 |
| 真实闭环 | 未安装 Obsidian 时完成原十二项代表性验收 | HOLD |

## 10. 当前停点

R0 的结构、交互、品牌、迁移和真实参考捕获已经完成。当前仍不得：

- 修改 N2R 业务代码或样式；
- 启动 Syn、进入 I5、Gate 0 或原十二项；
- 访问真实 store/vault、Codex CLI/MCP 或调用工具；
- 读取用户现有 Obsidian vault、登录、启用 Sync/Publish、安装插件或主题；
- stage、commit 或 push。

下一步是基于本参考形成精确 N2R 界面返工范围，并由用户单独授权实现。未获得该授权前，不修改业务代码，也不把 R0 参考完成解释成 Syn 界面已经完成。
