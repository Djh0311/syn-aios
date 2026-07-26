# L3 Obsidian 嵌入可行性与路线选择证据 v1

- 日期：2026-07-23
- 类型：只读架构/公开接口勘查 + 隔离安装探针
- 状态：**路线证据已足够；官方 App 临时副本已普通启动一次，但因自动打开来源不明的既有 vault 立即退出，不算隔离 vault 或完整 UI 验收通过**
- 结论正本：`decisions/2026-07-23-l3-obsidian-full-interface-in-syn-route-v1.md`

## 1. 勘查问题

1. 能否把未经修改的完整 Obsidian 桌面 UI 真正放进当前 Syn Tauri 窗口的视图树，并完整保留功能？
2. 若不能，什么路线在完整功能、视觉接近、合规、开发量和长期维护之间最好？
3. 当前 Syn 已有知识库资产能复用什么，缺口是什么？

## 2. 本地事实

- 仓库：`/Users/yoyi/workspace/product-line`；勘查基线 HEAD=`e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`；staged 为空；工作树已有多组已知脏改，勘查未覆盖或清理。
- Tauri：`tauri=2.11.2`；`tauri.conf.json` 只有一个主窗口；默认 capability 为 `core:default` + `core:window:allow-set-title`。
- 既有知识库：`knowledge_vault.rs` 固定 Syn 自管 vault，list/read/create/write/ai_write 五命令；`KnowledgeBaseView.tsx` 有原生 Markdown 列表/阅读/编辑；页面仍显示 Obsidian-compatible 占位。
- 勘查开始前：`/Applications/Obsidian.app` 不存在；`obsidian` 不在 PATH。

承重文件 SHA-256：

- `knowledge_vault.rs`：`b6b408ff56bb30fb5b293224df9ba1786206b3adf0dc2a1bb7b1f4773707853a`
- `command_registry.rs`：`8bde1852105d6b2d36861d247e5de12e399de175d1e72563dcbeac0350e1bf8b`
- `KnowledgeBaseView.tsx`：`6da9f6ff7cf0570ed67c078e82701f24672f5f0ab7304f0333c36b293c20f972`
- `knowledgeVault.ts`：`f0f106ef61925cf9368c498c001f73fd7c5a18b24aa0d5a8300b562840668587`
- `tauri.ts`：`47bfb2978960856159124cba8f4eed325951db8e0ae649992c314d46d6527fa2`

## 3. 公开接口证据

### Obsidian

- 官方部署说明明确 Obsidian 不是 Web 应用，只能部署本地 App；所以不存在官方 URL 可供 Tauri child webview 承载。
- 官方 URI 支持 open/new/daily/search 和 vault/file/path 定位。
- 官方 CLI（1.12.7+）面向外部脚本/集成，支持 vault/file、open/create/read/search、命令面板、主题、插件、工作区和开发者命令；App 必须运行。
- 官方开发文档支持在 Obsidian 内开发插件和自定义 view。
- 服务条款禁止修改、派生、逆向/解包和重新分发；因此 `app.asar`/渲染器重托管不进入工程候选。

### Tauri / Electron / macOS

- Tauri 使用 TAO 创建自身窗口、WRY 接入自身 WebView。JS `Webview.reparent(window)` 的目标是 Tauri `Window/Webview/WebviewWindow`，不是任意外部窗口。
- Electron 文档显示每个应用自己的 main process 创建/管理 `BrowserWindow` 与 `webContents`；WebContentsView承载的是该 main process 拥有的 WebContents 或加载的网页。
- Apple `NSWindow.addChildWindow` 接受 `NSWindow`；Accessibility 对其他进程给出的是 `AXUIElement` 和位置/动作控制。公开文档没有从外部 `AXUIElement` 转换/接管为 Syn `NSView` 的接口。

## 4. 隔离安装探针

- 官方下载页解析到官方 `obsidianmd/obsidian-releases` 的 `Obsidian-1.12.7.dmg`。
- 下载文件大小为 `213,334,249` 字节；文件 SHA-256 与官方 GitHub Release API 提供的摘要一致。
- 探针只把 DMG/App 放在 `/private/tmp`，没有复制到 `/Applications`，没有读取或导入任何既有 vault。
- mount 沙箱限制经受控权限处理；签名/Gatekeeper 命令行检查出现异常后没有清 quarantine、没有关闭 Gatekeeper。
- 官方 App 临时副本随后通过普通 macOS 启动进入 Obsidian 1.12.7 真界面；它自动加载了名为“Obsidian Vault”的来源不明 vault。勘查线没有点开内容、设置、菜单、插件或 Canvas，立即从菜单退出。
- 目标测试目录 `/private/tmp/syn-obsidian-recon-20260723/Syn-Obsidian-Recon` 已创建且经指导线复核为空，尚未注册为 Obsidian vault；`/Applications/Obsidian.app` 仍不存在，CLI 仍未注册。
- 官方 DMG 的本机 SHA-256 经指导线复核为 `3b85c13b4ce55512e86e170a7cd2a494e2db695ac888c0601e153cb85b77881b`。

该探针的价值是验证了官方包来源、普通启动可达和数据隔离停点，不把“进入过界面”写成“安装/隔离 vault 可用”。O1 必须用可证明的独立配置或用户确认的空环境继续完成 CLI 注册和隔离 vault 实测。

## 5. 路线对比与裁决

| 候选 | 结果 |
| --- | --- |
| Tauri child webview 直接加载 Obsidian | 无官方 Web 应用，路线不成立 |
| 接管外部 Electron renderer/window | Tauri/Electron/macOS 公开接口均无此跨进程承载合同 |
| 解包/重托管 Obsidian renderer | 许可、安全、插件/Node 运行时和更新维护均不接受 |
| Syn 全量改 Electron | 仍不能取得另一 Obsidian app 的 webContents，且改造量巨大 |
| 官方 App + CLI/URI + 共享 vault | 完整功能、公开接口、最小维护，选为基础主路线 |
| 公共 Accessibility 伴随窗口 | 可保留完整 UI并最大化视觉贴合，但不是真嵌入；需要真实稳定性矩阵 |
| Obsidian 插件反向承载 Syn | 完整功能和单窗口可靠，但宿主方向反转；列为 O4B 后备 |

工程裁决：截至当前证据，**真嵌入没有受支持、合规、可维护的路线；进入最大化集成。** 这一结论不依赖安装探针能否最终启动；安装实测决定的是 O1/O4 具体实现和验收，不改变跨进程视图模型。

## 6. 未验证项

- 官方 App 在本机普通首次启动、版本和 CLI 注册；
- URI 的 open/new/search；
- Graph、Canvas、插件、主题、命令面板；
- Accessibility 伴随窗口在 Spaces/全屏/弹窗/多窗口下的稳定性；
- Syn 真实 App 与 Obsidian 同源 vault 的双向刷新、冲突和重启恢复；
- 主管 knowledge capability 的真实对话调用。

这些全部属于当前开发任务 O1-O6，不得由本路线 evidence 外推为已完成。

## 7. 来源

- <https://obsidian.md/help/Getting%2Bstarted/Download%2Band%2Binstall%2BObsidian>
- <https://obsidian.md/help/teams/deploy>
- <https://obsidian.md/help/cli>
- <https://obsidian.md/help/Extending%2BObsidian/Obsidian%2BURI>
- <https://docs.obsidian.md/>
- <https://obsidian.md/terms>
- <https://v2.tauri.app/concept/architecture/>
- <https://v2.tauri.app/reference/javascript/api/namespacewebview/>
- <https://www.electronjs.org/docs/latest/tutorial/process-model>
- <https://www.electronjs.org/docs/latest/api/web-contents-view>
- <https://developer.apple.com/documentation/appkit/nswindow/addchildwindow(_:ordered:)>
- <https://developer.apple.com/documentation/applicationservices/axuielement_h>
