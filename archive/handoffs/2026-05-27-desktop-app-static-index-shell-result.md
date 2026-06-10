# 桌面应用静态索引壳交接

## 回收对象

- 任务包：`product-line/tasks/2026-05-27-desktop-app-static-index-shell.md`
- 开发线：桌面应用线
- Evidence：`product-line/evidence/2026-05-27-desktop-app-static-index-shell.md`
- 原型目录：`product-line/prototypes/desktop-app/`

## 结论

静态索引壳原型已完成，可进入回收评审。

这不是最终桌面应用，也不是 Electron / Tauri 壳。它是低依赖只读网页壳，用当前静态索引样例验证第一版页面结构。

## 先说薄弱点

- 项目类型仍是未知，页面没有自动判定 ERP / 游戏。依据：任务包禁止自动判定项目类型。
- authority / handoff / evidence 只显示候选路径，不能说它们就是当前权威。依据：任务包禁止自动判定 authority 文件就是当前权威。
- harness 只显示候选入口，不判断是否可运行、是否有用。依据：当前索引只有候选元数据，没有运行因果证据。
- 任务线页只是轻量解析 `tasks/README.md`，不是最终任务状态协议。依据：任务包允许“静态解析困难时先展示任务队列文档入口和当前待派发任务”。
- 纯静态网页不能可靠实现打开文件夹、定位日志等桌面能力。依据：浏览器权限边界；当前实现没有引入 Electron / Tauri。

## 已实现页面

- 首页
- 项目页
- 会话页
- Skills 页
- Harness 页
- 任务线页

## 页面现在能展示的数据

- 首页：项目数、会话数、skills 数、plugins 数、harness 候选数、warning 摘要、数据生成时间、最近项目。
- 项目页：项目路径、线程数、活跃 / 归档线程数、最近更新时间、authority / handoff / evidence / harness 候选、context warning。
- 会话页：会话标题、编号、项目路径、更新时间、归档状态、rollout 是否存在、模型 / reasoning 摘要、warning。
- Skills 页：本地、系统、插件 skill 分类，展示 skill 路径、插件名、插件版本、warning。
- Harness 页：候选入口名、路径、所属项目、入口类型、来源、更新时间、大小、warning；没有运行按钮。
- 任务线页：待派发、进行中、已回收、暂停四类状态；只展示任务候选入口，不展开任务说明正文。

## 静态壳里未实现或只是占位的交互

- 打开项目文件夹：未实现。
- 复制路径：未实现。
- 定位 rollout 日志：未实现。
- 运行 harness：明确不实现。
- 自动安装 skills：明确不实现。
- 写入 Codex 状态库：明确不实现。
- 标记 current / paused / historical / superseded：未实现，属于后续阶段。

## 正文和敏感内容边界

- 没有读取或展示 Codex 会话正文。
- 没有展示工具输出、命令输出、输入历史或记忆正文。
- 没有展示 README / AGENTS / handoff / evidence 正文。
- 没有展示 harness 命令正文。
- 没有读取或展示 `auth.json`、`.env`、密钥、令牌、授权文件。

## 验证方式和结果

- `node --check product-line/prototypes/desktop-app/app.js`：通过。
- 本地 server：`cd product-line && python3 -m http.server 8765`。
- 浏览器打开：`http://localhost:8765/prototypes/desktop-app/`。
- Smoke 结果：6 个导航页面可切换；项目 30、会话 296、skills 50、harness 候选 132、任务线栏目 4；页面自身控制台错误 0。

## 建议回收判断

建议接受为阶段 2 静态网页壳原型，但不要把它接受为最终桌面应用。

后续如果要做真正桌面能力，建议先明确容器选择：

- 继续纯网页：只能做只读展示和复制类轻交互。
- Electron / Tauri：可以做打开文件夹、定位日志、系统通知等桌面动作，但要新增依赖和权限设计。

## 当前入口

- 原型入口：`product-line/prototypes/desktop-app/index.html`
- 运行说明：`product-line/prototypes/desktop-app/README.md`
- 静态数据源：`product-line/prototypes/index-kernel/codex-index.json`
