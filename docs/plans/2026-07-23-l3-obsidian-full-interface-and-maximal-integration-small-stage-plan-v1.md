# L3 Obsidian 完整界面与最大化接入小阶段计划 v1

- 日期：2026-07-23
- 状态：**已被 v2 取代，O4/O4B 与强制 O1 停止**（当前计划：`docs/plans/2026-07-23-l3-syn-native-knowledge-workspace-small-stage-plan-v2.md`）
- 阶段归属：L3 知识库第二片；延续 `docs/plans/2026-07-14-post-m5-stage-plan-v2.md`，不是另起产品阶段
- 路线决策：`decisions/2026-07-23-l3-obsidian-full-interface-in-syn-route-v1.md`
- 开发合同：`tasks/2026-07-23-l3-obsidian-full-interface-and-maximal-integration-development-package-v1.md`

> v1 已产生的固定 vault、冲突保护、typed 兼容桥与知识只读 capability 可并入 v2；伴随窗口、Obsidian 中心化 UI 和把安装/CLI 作为完成前置不再执行。

## 0. 小阶段目标

让知识库从“Syn 自管 Markdown 第一片”进入**可真实日用的 Obsidian 集成阶段**：

- 官方 Obsidian 安装可用，且只打开 Syn 自管 vault；
- Syn 能明确显示安装、运行、CLI、vault 和联动状态；
- 用户能从 Syn 打开完整 Obsidian UI，直接使用原版编辑、设置、Graph、Canvas、插件和命令；
- Syn 与 Obsidian 对同一 Markdown 真相源双向可见；
- 项目主管能通过 Syn capability plane 只读搜索、读取、引用和打开知识，写入仍走用户确认；
- 争取用受管伴随窗口让 Obsidian 视觉上贴合 Syn 知识库区域；做不到稳定时自动降级到独立窗口或 Obsidian 内 Syn 面板，不伪称真嵌入。

“完整可用”的含义是：运行未经修改的官方 Obsidian，因此其原生功能没有被 Syn 的自制编辑器替代或裁剪；验收会覆盖代表性核心能力，但不会用有限测试声称逐个验证全部第三方插件。

## 1. 已知、未知与工作假设

### 已知

- Obsidian 没有官方 Web 应用；Tauri child webview 无合法页面可加载。
- 官方 CLI/URI/插件面足以建立可靠外部集成；CLI 需要 Obsidian 1.12.7+ 并要求 App 运行。
- 当前 Syn vault、Markdown 浏览/编辑、确认式 AI 写和 Batch 2 审计已经存在，必须复用而不是重造第二份数据源。
- 公开 Tauri/Electron/macOS 接口没有显示可把另一应用的 Electron 窗口变成 Syn 内部 `NSView/WebView` 的能力。

### 未知

- 当前官方 macOS 安装包在本机 Gatekeeper 下能否无异常通过首次启动；必须真机验证，不能用下载成功替代。
- macOS 辅助功能托管能否在 Spaces、全屏、弹窗、多窗口和焦点切换下达到日用稳定。
- 受管伴随窗口与 Obsidian 内 Syn 插件，哪一个用户体验更接近最终目标；由实物验收决定，不靠文档先定。

### 假设

- 用户优先要保留完整 Obsidian 功能；真嵌入不可达时，允许用未经修改的官方 App 做最大化集成。
- 产品 vault 继续使用既有 Syn 自管目录；不接触用户其他 vault。
- 本小阶段以 macOS 当前产品环境为验收平台，不在同一包扩展 Windows/Linux。

## 2. 阶段图

| 阶段 | 目标 | 可并行关系 | 阶段完成证据 |
| --- | --- | --- | --- |
| O0 路线勘查 | 判定真嵌入、许可、CLI/URI/插件与窗口路线 | 已由指导线完成；实机补证可并行 | 路线 decision + 勘查 evidence |
| O1 官方安装与隔离 vault | 安装、签名/Gatekeeper、首次启动、CLI、测试 vault | O0 后；可与代码只读勘查并行 | 安装来源、版本、签名、CLI、vault 路径和零既有 vault 访问 |
| O2 Obsidian 集成核心 | 状态探测、启动、open/search/read/command typed bridge | O1 后；后端测试和前端状态模型可并行 | Rust tests + `cargo check --lib` + CLI fixture/real probes |
| O3 同源 vault 与知识库 UI | 共享 Markdown、刷新、降级、打开完整 Obsidian | O2 合同冻结后与 O4 探针并行 | TS/offline tests + Syn/Obsidian 双向编辑实证 |
| O4 最大化视觉承载 | 公共 API 伴随窗口，失败则可靠降级 | O2/O3 后 | 窗口矩阵截图/视频与稳定性裁决 |
| O5 Syn capability plane | 主管只读 search/read/open/cite，写仍确认 | O2/O3 后，可与 O4 并行 | registry/allowlist/binding 定向测试，零越权写 |
| O6 真实 App 验收收口 | 代表性 Obsidian 功能 + Syn 闭环 + 重启恢复 | O1-O5 后 | 真实 App evidence、CURRENT/AUTHORITY 回写 |
| O4B 反向单窗口后备 | 仅当 O4 稳定性不达标时，用官方插件在 Obsidian 内呈现 Syn 面板 | O4 裁决后 | 插件安装/卸载、权限、单窗口实证 |

## 3. 各阶段合同

### O0：路线勘查与词义冻结

完成项：

- 读取当前 Tauri/知识库架构，确认无现成 Obsidian 集成。
- 核对 Obsidian 安装、无 Web 应用、CLI、URI、插件、条款；核对 Tauri/Electron/macOS 公开窗口模型。
- 冻结四个词：`真嵌入`、`受管伴随窗口`、`独立打开`、`反向承载`。产品状态不得混称。

完成门：路线 decision 已落；开发线不得再尝试 `app.asar`、私有 API或把截图流当完整 UI。

### O1：官方安装、CLI 与数据隔离

1. 从官方 `obsidian.md`/官方 release 下载；记录版本、大小、发布摘要、代码签名与 Gatekeeper 结果。
2. 不关闭 Gatekeeper、不清 quarantine、不覆盖异常；异常时保留最早错误并换官方稳定版本或等待用户处理。
3. 首次实测只用全新 `Syn-Obsidian-Integration-Test` vault，不读任何已知或未知既有 vault。
4. 在 Obsidian 设置里启用官方 CLI；验证 version、vaults、open/create/read/search、commands、Canvas/Graph/设置入口。
5. 开产品 vault 前，对既有 Syn vault 做只读 manifest + 可恢复备份；Obsidian 只允许新增自己的 `.obsidian/` 配置和用户显式操作产生的内容。

完成门：官方 App 能普通启动；CLI/URI 可复现；测试库完成新建→编辑→搜索→重启闭环；没有绕过系统安全，也没有读取其他 vault。

### O2：后端 typed bridge

实现一个独立 `obsidian_integration` 模块，职责限定为：

- 检测 `not_installed / installed / app_not_running / cli_not_enabled / ready / incompatible`；
- 解析并固定官方 app/CLI 路径，不信任前端传入程序路径；
- typed commands：open vault、open note、open search、read、search、list commands、run allowlisted command、focus/quit/restart 状态；
- 每条命令固定 argv、超时、输出上限和人话错误；禁止经过 shell，禁止前端提交任意 CLI subcommand；
- 开发探针可调用 screenshot/CDP，但 production handler 不暴露 `eval`、`dev:cdp`、任意 JavaScript 或任意文件路径；
- vault id/path 由宿主固定为 Syn 自管 vault，路径锁复用现有 knowledge vault 规则。

完成门：注入假 CLI 的成功、超时、非零、输出超限、未知版本、路径/参数攻击均有离线测试；production Rust 通过 `cargo check --lib`。

### O3：同源 vault 与知识库页面

- Knowledge 页面把“占位”改为真实状态区：安装/CLI/vault/伴随状态、打开 Obsidian、搜索、打开当前笔记、断开/重试。
- 保留当前 Syn Markdown 浏览器作为 Obsidian 不可用时的降级面，不再重复造 Graph/Canvas/插件 UI。
- Obsidian ready 时，所有“打开/搜索/命令”走 typed bridge；Syn 直接写和 AI 写仍走现有路径锁、用户确认和 Batch 2 audit。
- 增加外部文件变化感知；至少在窗口重新聚焦、手动刷新和写完成后稳定刷新列表/正文。若可靠 file watcher 会扩大依赖，可先用有界 refresh，不以轮询风暴换实时。
- 冲突策略：编辑前记录 mtime/hash，保存时若 Obsidian 已改动则拒绝覆盖并提示重读；不做最后写入者无声获胜。

完成门：同一笔记能在 Syn 新建→Obsidian 编辑→Syn 看见；Obsidian 新建→Syn 看见；冲突不静默覆盖；Obsidian 关闭时 Syn 原生面仍可读写。

### O4：受管伴随窗口

先实现最小公共 API 探针，再决定是否产品化：

- 只使用 macOS Accessibility/公开窗口接口定位 Obsidian 主窗口，需用户显式授予辅助功能权限；
- Syn 知识库页提供内容区 frame，后端只同步真实 Obsidian 窗口的位置、大小、显隐和焦点；
- 提供“贴合 Syn / 分离窗口 / 重新连接”三态；权限缺失、窗口不存在或状态不稳时立即退回独立打开；
- 不隐藏 Obsidian 身份，不截屏重放，不劫持菜单，不阻断原生弹窗。

稳定性矩阵：移动、连续缩放、最小化/恢复、关闭/重开、Cmd-Tab、不同 Space、Syn/Obsidian 全屏、设置窗口、插件安装弹窗、Canvas 拖拽、多窗口。关键项任一出现遮挡、输入错位、窗口丢失或无法恢复，O4 判为“不适合默认开启”。

完成门：通过则默认可选开启、默认可分离；不通过则保留一键独立打开并进入 O4B，不继续无限修坐标。

### O4B：Obsidian 反向承载 Syn 面板（条件分支）

- 用官方插件 API 建一个最小 Syn view；插件只安装到 Syn 专用 vault。
- 首片只显示连接状态、当前项目/笔记、向 Syn 打开/搜索/引用；不得获得任意 filesystem、任意命令或主管写能力。
- 通信若需本地端点，只绑定 loopback，使用每次启动随机 token、明确 origin、大小/速率限制；没有认证时不开放。
- 插件可完全禁用/卸载；禁用后 Obsidian 和 Syn vault 都不受损。

完成门：用户在单个 Obsidian 窗口里可用完整 Obsidian 和 Syn 知识动作；文案明确“Syn 面板在 Obsidian 内”，不倒置事实。

### O5：知识能力进入 Syn MCP 统一层

首批只开放：

- `knowledge_search`：宿主固定 vault 内全文搜索，返回 slug/title/片段/mtime；
- `knowledge_read`：按受验证 slug 读取，输出上限；
- `knowledge_open`：让宿主在 Obsidian 打开目标笔记，只改变 UI，不写知识；
- `knowledge_cite`：生成含 vault/slugs/mtime 的结构化引用，不把引用自动升级为正式记忆。

`tools/list` 与 `tools/call` 必须复用现有 registry/role allowlist 和可信 turn binding；前端/模型不能扩大 vault、工具或命令。AI 写入仍只经现有 PendingAction + 用户确认；本阶段不公开 `knowledge_write`。

完成门：缺 binding、错项目、路径穿越、未知工具、大小写变体、wildcard、过大输出均 fail closed；自然回复不因知识工具失败被吞；未确认时 vault 写入增量为 0。

### O6：真实 App 代表性验收

必须在真实 Syn + 官方 Obsidian + Syn 专用 vault 上验证：

1. 安装检测、首次启动、CLI ready、重新启动恢复；
2. Syn 打开 vault/笔记/搜索；Obsidian 设置、命令面板、主题、核心插件入口可用；
3. Markdown 编辑、wikilink、backlink、Graph、Canvas 各完成一次真实操作；
4. 安装/启用/禁用一个可撤销的测试社区插件，验证命令可列出/执行；
5. Syn↔Obsidian 双向新建/编辑/刷新与冲突拒绝；
6. 主管对话完成一次 knowledge search/read/open/cite，自然回复引用真实笔记；
7. 一次 AI 知识写入保持“提议→用户允许→落盘→审计”，拒绝时零写；
8. 伴随窗口按 O4 矩阵裁决；失败时独立窗口/O4B 降级可用；
9. 退出后无残余桥进程、无未知监听端口、无真实项目/其他 vault 访问；
10. 重启 Syn 与 Obsidian 后仍能继续使用同一知识库。

验收输出必须把“原版 Obsidian 功能保留”“本轮代表性功能已验”“伴随窗口通过/降级”“未逐一验证所有第三方插件”分开写。

## 4. 统一验证闸

- Rust 定向单测与相关 knowledge/M5-B 回归；
- `cargo check --lib`；
- `npm run typecheck`；
- 离线交互 runner；
- 新的 Obsidian integration 前端场景测试；
- shape gate 在开工时重新冻结实际 baseline，只接受本包零净增，历史结构债单列；
- `git diff --check`；
- staged 始终为空；不 commit/push；
- 真实 App evidence 包含版本、命令、屏幕证据、vault manifest 前后差和失败项，不含私密路径正文、token、完整 CLI 环境或其他 vault 名称。

## 5. 自主推进与停点

开发线可在本计划内部自动从 O1 推进到 O6，不需要每个小阶段重新向用户问“是否继续”；只在以下情况停：

- macOS 明确要求用户亲手授权辅助功能/自动化或处理 Gatekeeper；
- 需要访问/迁移任何现有非 Syn vault；
- 需要购买 Obsidian Sync/Publish、登录账号或对外发布插件；
- 必须解包/修改 Obsidian、使用私有 API、放宽 Syn sandbox/MCP allowlist；
- 必须修改任务包白名单外的承重 schema、M5 bridge、conversation transport 或真实业务项目；
- dirty overlap 无法安全 merge；
- stage、commit、push。

遇到 O4 技术失败不算全计划阻塞，按既定分支转独立窗口/O4B继续完成最大化实现。

## 6. 小阶段完成定义

同时满足以下条件才可宣布完成：

- 官方 Obsidian 与 Syn 专用 vault 真实可用；
- Syn 页面不再是 Obsidian 占位，而有真实状态和动作；
- CLI/URI typed bridge、同源刷新、冲突保护和降级已验证；
- 完整 Obsidian UI 可从 Syn 进入并完成代表性原生功能；
- 伴随窗口有诚实裁决，通过或稳定降级；
- 主管知识只读能力进入统一 capability plane，写入闸未放宽；
- 真实 App 验收、离线验证、CURRENT/AUTHORITY/总计划回写齐全；
- 无其他 vault/真实项目越界，无 stage/commit/push。
