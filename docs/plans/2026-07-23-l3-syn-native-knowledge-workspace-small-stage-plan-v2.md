# L3 Syn 原生知识工作区小阶段计划 v2

- 初始日期：2026-07-23
- 界面路线修订：2026-07-25
- 状态：**N0-N5 既有离线能力已收口；N2R-R0 真实参考已冻结；R1、R2、R3A、R3B、经 R3C-R1 修正后的 R3C、R3D 与 R3E（活动栏 ribbon / 右栏层级 / Graph 规模化）的 synthetic 范围均已获指导接受；R2 差距矩阵已全收，下一段是 R4 最终视觉对照；N6 真实 App 验收继续 HOLD**
- 决策：`decisions/2026-07-23-l3-syn-native-knowledge-workspace-route-v2.md`
- R0 参考：`docs/design/2026-07-25-l3-obsidian-core-desktop-reference-and-migration-r0-v1.md`
- 原开发包：`tasks/2026-07-23-l3-syn-native-knowledge-workspace-development-package-v2.md`
- 取代：`docs/plans/2026-07-23-l3-obsidian-full-interface-and-maximal-integration-small-stage-plan-v1.md`

## 0. 完成目标

在 Syn 原生技术与安全边界内交付一个不依赖 Obsidian即可真实使用的知识工作区，并让其核心桌面壳在信息架构、空间比例、面板组织、标签页模型和高频交互上高保真对齐冻结的 Obsidian 参考。

本阶段复刻核心桌面体验，不嵌入 Obsidian，也不复制其品牌资产、私有代码、插件生态、Sync、Publish 或移动端。

## 1. 阶段顺序

| 阶段 | 目标 | 状态/完成证据 |
| --- | --- | --- |
| N0 转向冻结 | 收口 v1 WIP，停止伴随窗口和 Obsidian 进程中心化路线 | 已完成 |
| N1 Vault 与索引 | 受限目录/文件树、可重建全文索引、标签/属性、链接/反链投影 | 离线已完成 |
| N2 编辑工作区 | Markdown 编辑/预览、标签页、分栏、快速打开、命令面板、保存冲突 | 能力离线已完成，界面结构待 N2R 收口 |
| N3 图谱 | 全局/局部关系图、筛选、从节点打开笔记 | 离线已完成 |
| N4 JSON Canvas | `.canvas` 读写、节点/连线、冲突保护 | 离线已完成 |
| N5 附件与恢复 | 受限附件、工作区恢复、备份/恢复和外部变更刷新 | 离线已完成 |
| N2R 高保真单壳返工 | 冻结参考，消除双容器，将既有能力迁入唯一桌面工作台 | **R0 完成；R1/R2/R3A/R3B/R3C/R3D/R3E synthetic 接受；R3 全收，下一段 R4** |
| N6 AI/MCP 与验收 | 主管只读 search/read/open/cite、确认式 AI 写、全闭环真机验收 | HOLD |

N2R 不重做 N1-N5 的后端能力。它先于新的 UI discovery、Gate 0 和十二项真实 App 验收完成。

## 2. N0：转向冻结

- 保留 typed bridge 与知识 capability；桥接入口降为可选兼容。
- 页面不再把 Obsidian ready 作为知识库主状态。
- 不恢复受管伴随窗口、伪嵌入、Electron 迁移、`app.asar` 或私有 API 路线。

完成证据沿用既有 N0 离线记录。

## 3. N1：Vault、路径与可重建索引

- 使用受限相对路径宿主类型；拒绝绝对路径、`..`、控制字符、选项形态、符号链接和大小写漂移。
- 笔记树、标签、Frontmatter、wikilink、backlink、搜索索引从 vault 文件重建。
- Markdown、Canvas、附件具有明确扩展名、大小和目录边界。

既有实现和离线证据继续有效；N2R 不得放宽这些合同。

## 4. N2：原生编辑能力

- 支持 Markdown 源码编辑、渲染预览、`[[双链]]` 跳转、标签和声明式安全属性。
- 支持多个标签页、左右分栏、快速打开和 Syn 自有命令面板。
- 保存使用 revision/mtime/hash；外部变化时显式处理，不默认覆盖。
- 工作区布局是可重建用户偏好，不进入知识真相源。

这些能力已经离线存在，但当前页面把新工作台、Graph、Canvas、维护面板和旧知识界面纵向叠放，不能视为 N2 界面完成。

## 5. N2R：Obsidian 核心桌面高保真单壳返工

### R0 参考冻结

当前状态：

- `docs/design/2026-07-25-l3-obsidian-core-desktop-reference-and-migration-r0-v1.md` 已冻结 Obsidian `1.12.7` Public 的核心桌面结构/交互基线、Syn 品牌替换规则、双容器迁移表和不复制资产清单；
- `1.13.3` Catalyst 只作变化观察，不进入当前实现基线；
- 用户已授权安装/打开官方 Obsidian；`1.12.7`、Default、light、16 px、0 缩放的 `984 × 768` 真实界面已用无隐私演示 vault 捕获；
- `1440 × 900`/`1180 × 760` 保留为 Syn 实现后的验收视口，不冒充本轮 Obsidian 实图；
- R0 已完成，但这不自动授权 N2R 实现，也不等于 Syn 已高保真完成。

在改代码前记录：

- Obsidian 桌面版本、默认主题、明暗模式、窗口尺寸和系统缩放；
- 文件树、编辑、预览、搜索、反链、Graph、Canvas 的基准截图；
- 活动栏、左右侧栏、标签栏、工作区、状态区的尺寸与行为；
- Syn 产品名、字体、颜色、图标和确认式 AI 动作的替换规则；
- 不复制的商标、原始图标包、品牌图形和受限资产。

没有这套冻结参考，不得使用“1:1 完成”或“高保真通过”的结论。

### R1 结构收口

`KnowledgeBaseView` 只保留一个固定高度知识工作台：

- 最左活动栏；
- 左侧文件/搜索/标签侧栏；
- 中间标签页工作区；
- 右侧属性/反链/标签/大纲侧栏；
- 底部低层级状态区。

主页面不得纵向堆叠多个知识容器。各栏内部滚动，外层应用壳不滚动。

2026-07-25 指导线独立复核并接受 R1 的 React-only 范围，精确口径为
`ACCEPTED_N2R_R1_OFFLINE / NOT_REAL_APP_ACCEPTED`。结构合同通过不等于视觉、高保真或真实 App 通过。

### R2 隔离浏览器视觉基线与差距审计

- 任务包：`tasks/2026-07-26-l3-syn-n2r-r2-isolated-browser-visual-baseline-package-v1.md`；
- 使用真实 React 组件、真实生产 CSS、纯合成知识数据和 fresh browser context；
- 在 `1440 × 900`、`1180 × 760`、`900 × 760` 采集 Markdown/Preview/Graph/Canvas、侧栏、overlay、内部滚动和键盘焦点；
- 只记录当前像素、量尺和差距，不在本包修改生产代码或样式；
- 由于现有 isolated runtime profile 尚未隔离 Knowledge 路由的 WebView `localStorage`，本包不是 Syn/Tauri App 验收，也不越过 Home-only discovery。
- 2026-07-26 首轮九图与量尺的隔离、几何、焦点和 `1 px` overflow 事实成立；指导线发现多项 R0 视觉差距漏判及 Search 未进入结果态后，退回一次 evidence/补图窄返工。
- 窄返工已补齐真实 Search 结果态，并把 overflow、Search/command/quick-open、标签组/分栏、Canvas、活动栏、Graph 与右栏层级写入完整 P1/P2 矩阵。指导最终口径为 `ACCEPTED_N2R_R2_BASELINE / NEEDS_N2R_R3_VISUAL_CONVERGENCE / NOT_REAL_APP_ACCEPTED`；R3 仍须用户另行授权。

### R3 既有能力、交互与视觉收敛

2026-07-26 用户已授权第一包
`tasks/2026-07-26-l3-syn-n2r-r3a-search-and-overlay-convergence-package-v1.md`。
R3A 只关闭 `900×760` overflow、左栏 Search 与 command/quick-open 的结果、选择、键盘、焦点和紧凑层级；标签组/分栏、Canvas、Graph、活动栏和右栏留在后续串行包。

2026-07-26 指导线确认 R3A 的行为、隔离与量尺合同成立，但 Quick Open/Command 使用不透明全屏 backdrop，底层工作区完全不可辨认，未形成冻结 R0 的临时浮层关系。用户随后授权并完成 `tasks/2026-07-26-l3-syn-n2r-r3a-r1-overlay-layering-rework-package-v1.md`；指导独立复核一行 CSS、浏览器合同和新图后，以 `ACCEPTED_N2R_R3A_SEARCH_OVERLAY / NOT_REAL_APP_ACCEPTED` 收口。

2026-07-26 用户授权
`tasks/2026-07-26-l3-syn-n2r-r3b-central-tab-groups-and-split-package-v1.md`。
R3B 只收敛中央单排真实标签组、最多两个左右组、同一 Markdown 草稿的源码/预览投影、可调分隔、焦点/ARIA 与可丢弃 UI 偏好迁移；Graph/Canvas 仅验证进入组内，不重做内容视觉。活动栏、右栏和后续 R3 仍未授权。

2026-07-26 指导线独立复核 R3B 的实际 reducer、唯一草稿写路、偏好迁移、dirty/conflict 闭锁、浏览器 raw 报告与九张截图，并复跑 typecheck、37-entry runner、shape 和 selftest；以 `ACCEPTED_N2R_R3B_CENTRAL_TAB_GROUPS_AND_SPLIT / NOT_REAL_APP_ACCEPTED` 收口。R0 01/02 只在中央组关系维度通过；Canvas、Graph 内容视觉、活动栏和右栏仍属剩余 GAP。

同日用户批准并由指导线派发
`tasks/2026-07-26-l3-syn-n2r-r3c-canvas-first-convergence-package-v1.md`。
R3C 只关闭 R2 Canvas P1：连续画布成为中央组主焦点，当前路径/状态收为紧凑 chrome，Canvas 文件/新建和节点 inspector 改为按需面板，节点/视口工具浮于画布内；R3B 组状态、Graph、活动栏、右栏、Rust、真实数据和真实 App 均冻结。

同日执行线完成 R3C 主结构并回交 `7 contexts / 131 assertions / 0 failed`。指导独立复核确认 Canvas-first 视觉、隔离和量尺证据成立，但 fresh synthetic 反例证明：从空态“选择画布”打开文件面板后按 Escape，焦点固定跳到顶部“画布”，没有回到实际触发器。R3C 因此为 `NEEDS_N2R_R3C_REWORK / NOT_ACCEPTED`；指导当时只允许把最窄 R3C-R1 作为下一授权候选，不得直接进入 R3D。

用户随后明确回复“可以”，指导线已派发
`tasks/2026-07-26-l3-syn-n2r-r3c-r1-file-panel-opener-focus-return-rework-package-v1.md`。
R3C-R1 只记录本次实际 opener：Escape/显式关闭回同一入口，选择成功后回连续 stage；只允许写 Canvas 组件、Canvas 聚焦合同和新 synthetic evidence。CSS、fixture、runner、R3B、Graph、活动栏、右栏、Rust、真实数据与真实 App 继续冻结。

R3C-R1 执行线回交 `5 contexts / 73 assertions / 0 failed`，指导线独立核 actual opener 实现、activeElement/DOM identity、三张图与规定门禁后，以
`ACCEPTED_N2R_R3C_R1_FOCUS_RETURN / ACCEPTED_N2R_R3C_CANVAS_FIRST / NOT_REAL_APP_ACCEPTED`
收口。该接受只覆盖 Canvas-first synthetic 范围；shape `17 / 5 / 5` 历史债、完整 R0 与真实 App 均未通过。

用户再次明确回复“可以”后，指导线派发
`tasks/2026-07-26-l3-syn-n2r-r3d-graph-convergence-package-v1.md`。
R3D 只关闭 R2 Graph P2：把中央 Graph 的大矩形卡片阵列和常驻管理页式 chrome 收为轻量节点/细连线的连续关系舞台，并补齐节点键盘打开、焦点/ARIA、`1440/1180/900` 量尺与 pure-synthetic 浏览器证据。现有 `@xyflow/react`、只读 typed client、全局/局部/筛选和打开原生笔记能力必须复用；Canvas、组壳、fixture、runner、活动栏、右栏、Rust、真实数据与真实 App 继续冻结。

R3D 执行线回交 `8 contexts / 75 assertions / 0 failed`（red 先行 `35 / 17`）。指导线不采信自报数字：把 green runner 拷到会话临时目录独立重跑，结果与执行线 `green-browser-evidence.json` **深度 diff 零处差异**，四张截图 **SHA-256 逐张相同**；并自算 17 个冻结 hash 全 MATCH、HEAD/staged/5173 不变，自跑 typecheck、37-entry runner（Graph 19、R3B 16/0、R3C 14/0 + R3C-R1 4/0）、shape baseline `17/5/5` 与 check `exit 1` 同债、hex 13/13、machine-face 18/18、`git diff --check` 干净，并按 mtime 序（red 16:17–16:20 → CSS 16:40 → 组件 16:46）确认 red-first 未事后补红。指导另行排查了唯一伸出 Graph 子树的 CSS 规则 `.knowledge-workbench-central:not(:has(> …__conflict)) > .knowledge-workbench-groups`：`central` 只有条件冲突条（row 1）与 groups（row 2）两个子元素，冲突条在场即失效、不在场时 `1/-1` 与 `grid-row:2` 几何等价，且 R3C 已有同形 Canvas 先例，故"未改同组 Markdown/Canvas computed style"成立。

指导结论为 **`ACCEPTED_N2R_R3D_GRAPH_CONVERGENCE / NOT_REAL_APP_ACCEPTED`**：只接受 synthetic 范围的 Graph 关系舞台、40px chrome、按需筛选、轻节点/细边、ARIA/键盘与 typed handoff。同时记入两条 catch（见 `docs/harness-catch-log.md` 07-26 两行）并转为后续必办项：① 确定性环形布局的半轴恒为 `110/160`、与节点数无关，节点盒固定 `136×40`，椭圆周长约 `855 px`，因此 6 节点刚好排满（间距约 `143 px`）、12 节点即互压、后端 `MAX_GRAPH_NODES = 512` 时必然堆叠——冻结夹具恰好停在临界点，规模外行为零断言覆盖；② 冻结夹具对局部请求仍回 global 投影，`局部` 只证到请求与 UI 层。shape `17 / 5 / 5` 历史债（含 styles.css 棘轮 `12419 / 水位 8464`）、完整 R0、活动栏、右栏与真实 App 均未通过。

### R3E 活动栏 ribbon + 右栏层级 + Graph 布局规模化（合并包）

用户拍板把剩余三项合并，`tasks/2026-07-26-l3-syn-n2r-r3e-activity-rail-right-context-and-graph-scale-package-v1.md` 于同日派发（派发前用户另拍两处收紧：大 `n` fitView 只作俯瞰、明写不可点不可读；右栏"大纲"取删字不取实现）。包内强制两项新规矩：改动前留**基线副本 + manifest**（否则 hunk 边界物理上不可核，R3D 的教训），以及四个冻结 green runner 拷到仓外重跑作**回归锁**。因 `NativeKnowledgeWorkspace.tsx` 距 `.tsx` 2000 行硬限仅剩 22 行，包内强制抽出两个新组件。

执行线开工即抓到写所有权冲突（两个窄写目标已被在途 WIP 偏离派发 hash），按 §10 停止零写入并回交 `BLOCKED_N2R_R3E_WRITE_OWNERSHIP_CONFLICT`；用户指示接着做后从既有 WIP 接管，并修好继承来的 D3 缺陷（`LAYOUT_PITCH` 定义了却没用、浮点边界把 6 掉成 5，纯函数 `465` 处失败 → `0`）。回交 `8 contexts / 126 assertions / 0 failed` + 纯函数 `1…512` 全绿 + 回归锁 `369 / 0`。

指导线独立复核（详见 evidence §10）：拷 runner 自跑得 `126/0` 且与执行线 JSON **深度 diff 零差异**；**不复刻算法**、自写探针 bundle 产品导出函数全量扫描 `n = 1…512`，低于下界 `0` 个、按更严的矩形重叠判据 `0` 对、确定性复算全等（最紧 `n=478 → 161.01`）；`shasum -c` 校验基线副本 7/7 后自行 diff 全部窄写文件、CSS 范围全在白名单内；自跑全部门禁（typecheck 0、离线 exit 0 / 37 入口、shape finding 集合差为空、两 selftest、diff/staged 干净）；核实 R3B/R3C/R3C-R1 三目录与 R3D 四图/JSON 均未被覆盖。R3D 那条竞态断言：指导线自跑原版两次均失败、修后 `75/0`，并以"回焦三函数在派发基线与收口版之间逐字节相同、R3E 五个 hunk 无一处触碰"+ 自写逐帧探针（detached 后 `8 ms` 起焦点稳定回到同一 opener DOM 节点）确认那是取样竞态、不是行为回归。

指导结论为 **`ACCEPTED_R3E_D1_ACTIVITY_RIBBON / ACCEPTED_R3E_D2_RIGHT_CONTEXT / ACCEPTED_R3E_D3_GRAPH_SCALE`（synthetic 范围）/ `NOT_REAL_APP_ACCEPTED`**。指导线同日收尾两笔：① 把 R3D 已验收目录的 runner 复原为验收原件 `1cee08e8…`，修好的版本改为 R3E 自有工具 `raw/regression-r3d-green-browser-evidence-r3e-fixed.mjs`（`c956dfd8…`，把全部 3 处 detached 后立刻取样统一改为有界轮询、并删掉两处固定 `waitForTimeout(40)`，自跑 `75/0`）——已验收证据目录恢复不可变，后续包的回归工具改登记新文件；② 活动栏悬停提示试加后**撤回**：合同把"不得用 title 替代可访问名称"实现成"整条禁止 title"，改动会使合同 `16/1`，属规则改动不该由复核方顺手塞入，已逐字节还原并转为下一段的用户决定项。

R3 至此三项差距全收（Search/overlay、标签组/分栏、Canvas、Graph、活动栏、右栏）。**下一段是 R4 最终视觉对照**；活动栏悬停提示、同心环不表达团块结构（512 呈"洋葱"）两项留作 R4 权衡输入。完整 R0、真实 Syn/Tauri App、真实 store/vault、N6 与发布验收仍未通过。

- Markdown 编辑、预览、分栏进入中间标签页；
- Graph、Canvas 作为中间工作模式或标签页；
- 文件、搜索、标签进入左侧栏；
- 属性、反链、大纲进入右侧栏；
- 旧统计、记忆捕获、维护和 Obsidian 兼容入口迁入命令、设置、按需抽屉或次级页面；
- 不删除仍有业务价值的能力，但禁止用第二主容器保留它们。
- 标签页打开、关闭、切换、恢复；
- 侧栏折叠、调整宽度和焦点恢复；
- 快速打开与命令面板的键盘路径；
- 文件树选择、展开、重命名和上下文操作；
- 编辑/预览/Graph/Canvas 切换；
- loading、empty、error、conflict、disabled、focus 和 reduced-motion 状态。

优先复用现有组件、语义 HTML 和既有可访问控件；不得为仿外观退化键盘或 ARIA 合同。

R3 的精确写入面只能由 R2 的 P0/P1/P2 差距矩阵形成，不得在 R2 执行前预先扩白。

### R4 最终视觉对照

- 中间内容区是唯一焦点；
- 紧凑桌面密度、侧栏比例、标签页高度、分隔与状态区逐项对照冻结参考；
- Syn 自有颜色、字体和品牌资产替换 Obsidian 品牌层；
- 不出现卡片仪表盘、全宽模块堆叠、重复标题区或多个竞争主面板；
- 基准窗口和紧凑窗口均无重叠、截断或外层滚动。

## 6. N3：知识关系图

- 复用 `@xyflow/react`，不新增第二图框架。
- 图数据只来自已验证 wikilink/反链投影。
- 提供全局图、当前笔记局部图、搜索/标签筛选和孤立节点。
- 节点打开 Syn 原生标签页，图布局不改写 Markdown。
- 在 N2R 中作为中间工作模式，不再作为编辑器下方的独立全宽容器。

## 7. N4：JSON Canvas

- 使用 JSON Canvas 1.0；支持 text/file/link/group 节点及边。
- 未识别字段 roundtrip 保留，非法结构拒绝。
- 文件节点只能引用固定 vault 内允许文件。
- 写入复用冲突保护、原子写和审计边界。
- 在 N2R 中作为中间工作模式，不再纵向追加到主页面。

## 8. N5：附件、刷新与恢复

- 只允许用户显式导入允许类型/大小的附件到固定附件目录。
- Markdown 和 Canvas 使用相对引用。
- 使用有界刷新；不为界面返工扩大文件监视权限。
- 记录标签页、分栏、侧栏和选中笔记；重启失败回到安全默认布局。
- 不静默删除知识文件。

## 9. N6：AI、MCP 与真实验收

- `knowledge_search/read/open/cite` 继续走精确 registry、role allowlist 和可信 binding。
- `knowledge_open` 默认打开 Syn 原生标签页；外部 Obsidian 打开是显式动作。
- 不公开 `knowledge_write`、`canvas_write` 或任意文件工具。
- AI 写继续走 PendingAction、用户允许、冲突复核和 `knowledge_vault_audit`。

### 9.1 UI 先行门

真实功能验收前先证明：

1. 只有一个知识工作台根；
2. 不再渲染旧统计条、旧三栏和旧 Vault Notes 第二主容器；
3. Graph、Canvas、编辑和预览在中间工作区切换；
4. 左、中、右只做内部滚动；
5. 基准参考对照项逐项有实图；
6. 键盘、焦点、空态、加载、错误和冲突态可用；
7. 未复制受限品牌资产。

### 9.2 原十二项功能门

1. 新建目录、Markdown 笔记和属性；
2. 创建双链并在反链区出现；
3. 全文搜索和快速打开；
4. 分栏编辑和预览；
5. 全局图与局部图打开目标笔记；
6. 新建、编辑、保存并重开 JSON Canvas；
7. 导入一项允许附件并从笔记/Canvas 引用；
8. 模拟外部改动并确认冲突不被覆盖；
9. 主管完成 search/read/open/cite，自然回复含真实引用；
10. AI 写入允许一次、拒绝一次，分别证明单次审计写和零写；
11. 重启 Syn 后恢复知识文件和工作区；
12. 未安装 Obsidian时上述核心闭环仍成立。

## 10. 统一验证

- 前端组件与场景测试；
- `npm run typecheck`；
- 离线交互 runner；
- Rust 定向测试及相关 knowledge/M5-B/capability/binding 回归；
- `cargo check --lib`；
- 目标 Rust fmt；
- shape gate 只要求本包零净增，历史债单列；
- `git diff --check`；
- staged 为空，不 commit/push；
- 获授权后才做真实 App 截图和功能验收。

## 11. 当前执行顺序

1. R0-A 结构/交互/品牌/双容器迁移冻结（已完成）；
2. R0-B 官方 Obsidian + 无隐私演示 vault 的真实参考捕获（已完成）；
3. N2R-R1 React-only 单壳结构收口包（离线施工与窄返工已完成，指导线已接受）；
4. N2R-R2 隔离浏览器视觉基线与差距审计（已获指导接受；现有 UI 仍为多项视觉/交互 GAP）；
5. N2R-R3A Search/overlay synthetic 收敛（含 R3A-R1，已获指导接受）；
6. N2R-R3B 中央标签组与左右分栏（已获指导接受，仅 synthetic/离线范围）；
7. N2R-R3C Canvas-first（含 R3C-R1 回焦窄返工，已获指导接受）；
8. N2R-R3D Graph 关系舞台收敛（已获指导接受，仅 synthetic 范围；环形布局规模外行为记账待办）；
9. N2R-R3E 活动栏 ribbon + 右栏层级 + Graph 布局规模化（合并包，已获指导接受，仅 synthetic 范围；活动栏悬停提示与同心环不表达团块两项留作 R4 输入）；
10. N2R-R3 完成并经离线/浏览器验收后，再做 R4 最终视觉对照；
11. 用户另行授权一次 isolated Home-only UI discovery；
12. UI 先行门；
13. 用户另行授权 Gate 0 与原十二项；
14. 知识真实 App 与对话三句重验继续串行。

并行口径：任何知识 R3 包运行 Vite/隔离浏览器期间，对话线只能做只读准备；其真实 App 三句重验不得与知识线的代码写入、构建、Vite/浏览器采集或任何知识真实 App 同时发生。

## 12. 停点

- 需要嵌入或控制 Obsidian 进程、解包/改签/逆向或复制受限资产；
- 需要修改 M5 schema、DB-primary bridge、CAS 或正式记忆模型；
- 需要任意 filesystem、shell、Electron/Node 注入或放宽主管 sandbox；
- 需要读取或导入 Syn vault 以外的用户资料；
- 需要新增对外服务、登录、购买、发布插件或真实项目执行；
- dirty overlap 无法安全合并；
- 启动真实 App、访问真实 store/vault、Codex CLI/MCP、stage、commit、push。
