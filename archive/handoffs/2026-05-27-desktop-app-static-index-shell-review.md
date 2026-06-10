# 桌面应用静态索引壳回收意见

## 回收对象

- 任务包：`product-line/tasks/2026-05-27-desktop-app-static-index-shell.md`
- 开发线：桌面应用线
- 原型目录：`product-line/prototypes/desktop-app/`
- 回传 evidence：`product-line/evidence/2026-05-27-desktop-app-static-index-shell.md`
- 回传 handoff：`product-line/handoffs/2026-05-27-desktop-app-static-index-shell-result.md`

## 结论

接受为阶段 2 静态网页壳原型。

不接受为最终桌面应用。它没有 Electron / Tauri 等桌面容器能力，也不能打开文件夹、定位日志或执行系统动作。

## 先说薄弱点

- 这是静态网页壳，不是真桌面应用。依据：实现目录只有 `index.html`、`styles.css`、`app.js`、`README.md`，没有桌面容器配置。
- 项目类型、当前权威、harness 是否可用都没有事实判定。依据：页面文案和 handoff 都明确只展示候选。
- 任务线页是轻量解析 `tasks/README.md`，不是稳定任务状态协议。依据：`parseTasks()` 只按 Markdown 二级标题和列表项解析。
- 回收线没有完成 Playwright 浏览器复核。原因是 Playwright wrapper 需要从 npm 拉 `@playwright/cli`，当前网络受限，报 `ENOTFOUND registry.npmmirror.com`。
- 本地 HTTP 复核不完整：`HEAD /prototypes/desktop-app/` 曾返回 200，但后续 GET 在沙箱环境里不稳定，不能把完整浏览器 smoke 当作本轮回收线亲自确认的证据。

## 复核结果

已通过：

- `node --check product-line/prototypes/desktop-app/app.js` 通过。
- 静态文件存在：`index.html`、`styles.css`、`app.js`、`README.md`。
- `index.html` 包含 6 个视图：home、projects、sessions、skills、harness、tasks。
- `app.js` 读取目标为 `../index-kernel/codex-index.json` 和 `../../tasks/README.md`。
- `app.js` 未发现 `localStorage`、`sessionStorage`、`writeFile`、`child_process`、`exec(`、`spawn(` 等危险写入或执行接口。
- 文件级数据复核：当前静态索引包含项目 30、会话 296、skills 50、plugins 11、harness 候选 132。
- 任务队列最终状态：待派发 1、进行中 0、已回收 8、暂停 1。

部分通过但不能当完整浏览器验证：

- 临时 server 需要提升权限启动，启动后 `HEAD http://127.0.0.1:8765/prototypes/desktop-app/` 返回 200。
- Playwright 浏览器验证未完成，原因是网络受限无法下载 CLI。

## 当前生效结论

- 静态壳可以作为阶段 2 的第一版可视化入口。
- 页面只能展示索引元数据、候选入口和 warning。
- 页面不能展示会话正文、工具输出、命令输出、输入历史、记忆正文、README / AGENTS / handoff / evidence 正文。
- 页面不能自动运行 harness、不能自动判定项目类型、不能自动判定 authority 是当前权威。
- 后续要做打开文件夹、定位日志、复制路径等桌面动作，必须另行选择桌面容器并设计权限边界。

## 派生任务

- 新增验证线任务包：`product-line/tasks/2026-05-27-desktop-app-static-shell-validation.md`

## 状态

已回收，接受为静态网页壳原型；进入验证线补 smoke 和布局检查。
