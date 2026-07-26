# 决策：L3 Obsidian 完整界面接入 Syn 的路线 v1

- 日期：2026-07-23
- 状态：**已被 v2 取代，保留为路线勘查历史**（当前正本：`decisions/2026-07-23-l3-syn-native-knowledge-workspace-route-v2.md`）
- 阶段归属：`docs/plans/2026-07-14-post-m5-stage-plan-v2.md` 的 L3 知识库后续片，不是改阶段目标
- 实施计划：`docs/plans/2026-07-23-l3-obsidian-full-interface-and-maximal-integration-small-stage-plan-v1.md`
- 开发合同：`tasks/2026-07-23-l3-obsidian-full-interface-and-maximal-integration-development-package-v1.md`

> 2026-07-23 用户确认停止继续投入伴随窗口和 Obsidian 中心化路线，改做 Syn 原生知识工作区；本文件只保存当时的可行性裁决，不再授权 O1-O6 续跑。

## 1. 用户目标不降格

目标优先级固定为：

1. Obsidian 官方桌面版的完整界面出现在 Syn 知识库工作区内；
2. 编辑、设置、菜单、命令面板、核心/社区插件、Graph、Canvas、弹窗和快捷键等原版能力可正常使用；
3. Syn 与 Obsidian 共用当前 Syn 自管 Markdown vault，并能启动、定位、搜索、读取、打开和安全写入；
4. 若 1 在公开、合规、可维护的技术边界内不可达，继续完成尽量接近的单窗口/伴随窗口集成，不把外部窗口伪称为原生嵌入。

## 2. 已确认事实

### 2.1 当前 Syn 与知识库

- Syn 桌面壳是 Tauri 2.11 单窗口应用，前端运行在系统 WebView；当前 capability 只开放主窗口默认能力和 set-title。
- L3 第一片已经有 Syn 自管 vault、Markdown 浏览/编辑、`[[双链]]` 与确认式 AI 写入；Markdown 文件仍是真相源。
- 当前知识库页面仍明确写着“Obsidian-compatible 占位 / 未执行 Obsidian 原生同步”，没有 Obsidian 安装、进程、CLI、窗口或插件接线。
- 本轮勘查前 `/Applications/Obsidian.app` 不存在，`obsidian` CLI 也不在 PATH。

### 2.2 Obsidian 官方接口

- Obsidian 官方说明它是本地桌面应用，**没有 Web 版应用**，因此没有可直接装进 Tauri child webview 的官方网页入口。
- 官方 URI 能打开 vault、文件、搜索、新建和每日笔记。
- Obsidian 1.12.7+ 提供官方 CLI；应用运行时可定向 vault/文件执行 read/create/open/search、命令面板命令、主题、插件、工作区和开发者命令。CLI 是当前最完整、最省维护的外部集成接口。
- 官方开发面支持在 Obsidian 内开发插件与自定义 view；这能把 Syn 面板放进 Obsidian，但承载方向与用户原始要求相反。

### 2.3 为什么“把原版 Obsidian 进程直接塞进 Tauri”没有受支持路线

- Tauri 的 child webview / `reparent` 管理的是由**同一 Tauri 应用创建**的 WebView，并且目标是 Tauri 自己的 Window/Webview；它不是接管其他应用窗口或 Electron `webContents` 的 API。
- Obsidian 是独立 Electron 应用。Electron 的 `BrowserWindow`/`WebContentsView` 由该 Electron 应用自己的 main process 创建和管理；把 Syn 改成 Electron也不会获得另一个 Obsidian 进程里的 `webContents`。
- macOS `NSWindow.addChildWindow` 接受本进程可持有的 `NSWindow` 对象；辅助功能 API能观察/调整另一个进程窗口的位置、大小和动作，但只返回 `AXUIElement`，不能把外部窗口转换为 Syn 的 `NSView` 子树。
- Obsidian 条款禁止修改、派生、解包/逆向和重新分发软件。提取 `app.asar`、改造渲染器或把私有资源重新托管在 Syn 内，不进入可选方案。

因此，本决策不宣称“数学上绝对不可能”，而是作出工程裁决：**截至 2026-07-23，没有发现同时满足公开接口、许可、完整功能和可维护性的原版 Obsidian 真嵌入路线。**

## 3. 路线裁决

| 路线 | 完整原版功能 | 真正在 Syn 视图树内 | 合规/维护 | 裁决 |
| --- | --- | --- | --- | --- |
| Tauri child webview 加载 Obsidian | 否；官方无 Web 版 | 只能加载网页 | 不成立 | 拒绝 |
| 接管外部 Electron BrowserWindow/webContents | 未发现公开跨进程接口 | 否 | 不可维护 | 拒绝 |
| 解包 `app.asar` / 重托管渲染器 | 高风险且插件/Node 环境不完整 | 表面可做 | 违反许可边界 | 禁止 |
| 把 Syn 全量迁到 Electron | 仍不能接管另一 Electron app | 否 | 大改且不解决根因 | 拒绝 |
| 官方 Obsidian + CLI/URI + 共享 vault | 是，原版 App 自己运行 | 否 | 最稳、官方支持 | **基础主路线** |
| macOS 公共辅助功能托管“伴随窗口” | 是 | 否；只是位置/焦点联动 | 需辅助功能授权，有边角风险 | **最大化视觉路线，必须诚实标注** |
| Obsidian 插件内承载 Syn 面板 | 是 | 反向：Syn 在 Obsidian 内 | 官方插件面，可维护 | **伴随窗口不稳时的单窗口后备** |

## 4. 最终产品路线

### 4.1 必做基础

- 安装并运行未经修改的官方 Obsidian 1.12.7+。
- 只向 Obsidian 打开 Syn 自管 vault；不发现、不读取、不导入用户其他 vault。
- 以官方 CLI 为首选控制层、URI 为降级层；Syn 前端不得提交任意 shell 或任意命令字符串。
- Markdown 文件继续是真相源；现有确认式 AI 写入与 Batch 2 审计不绕过。
- Obsidian 未安装、CLI 未启用或应用未运行时，Syn 现有原生 Markdown 页面继续可用。

### 4.2 最大化视觉承载

- 先用 macOS 公共辅助功能接口做“受管伴随窗口”：在 Syn 知识库内容区显示时，让真实 Obsidian 窗口跟随该区域的位置、大小、显隐和焦点；允许一键分离回普通 Obsidian 窗口。
- 必须验移动、缩放、最小化、Spaces、全屏、Cmd-Tab、设置/插件弹窗和多窗口；任何不能保持的状态都自动退回“并排/独立打开”，不得锁死或遮挡用户。
- UI 与文档统一叫“Obsidian 伴随工作区”或“已连接 Obsidian”，只有真正进入 Syn 视图树后才可使用“嵌入”一词。

### 4.3 后备单窗口路线

若伴随窗口无法通过稳定性门，则不继续堆坐标修补；改为官方 Obsidian 插件在 Obsidian 内增加 Syn 面板，用户在一个 Obsidian 窗口里使用完整 Obsidian 与 Syn 知识动作。承载方向反转必须明示。

## 5. 生产安全边界

- 禁止解包、修改、签名替换、绕过 Gatekeeper、清除 quarantine、私有 API、屏幕抓取冒充 UI 或重新分发 Obsidian。
- Obsidian CLI 的 `eval`、`dev:cdp`、`dev:dom` 只可用于受控开发探针，不能成为生产写路或任意代码执行入口。
- 生产 CLI 只允许宿主枚举的只读/低风险命令；写入继续走用户确认与现有知识库审计。
- 首次让 Obsidian 打开 Syn 真实 vault 前必须生成可恢复备份和 manifest；不得扫描其他 vault。
- 本决策不授权 stage、commit、push，也不授权修改用户既有 Obsidian 配置或购买 Sync/Publish。

## 6. 复议条件

只有出现以下新事实才重开“真嵌入”路线：Obsidian 官方发布可嵌入 Web/SDK，或提供合法的跨宿主 view/WebContents 接口；Tauri/macOS 提供受支持的外部应用视图承载 API；或 Obsidian 给出明确书面集成许可与技术接口。单纯能靠坐标、截图或逆向“看起来像”不构成复议事实。

## 7. 一手来源

- Obsidian 安装：<https://obsidian.md/help/Getting%2Bstarted/Download%2Band%2Binstall%2BObsidian>
- Obsidian 无 Web 应用：<https://obsidian.md/help/teams/deploy>
- Obsidian CLI：<https://obsidian.md/help/cli>
- Obsidian URI：<https://obsidian.md/help/Extending%2BObsidian/Obsidian%2BURI>
- Obsidian 开发文档：<https://docs.obsidian.md/>
- Obsidian 条款：<https://obsidian.md/terms>
- Tauri 架构：<https://v2.tauri.app/concept/architecture/>
- Tauri Webview API：<https://v2.tauri.app/reference/javascript/api/namespacewebview/>
- Electron 进程模型：<https://www.electronjs.org/docs/latest/tutorial/process-model>
- Electron WebContentsView：<https://www.electronjs.org/docs/latest/api/web-contents-view>
- Apple `NSWindow.addChildWindow`：<https://developer.apple.com/documentation/appkit/nswindow/addchildwindow(_:ordered:)>
- Apple Accessibility `AXUIElement`：<https://developer.apple.com/documentation/applicationservices/axuielement_h>
