# 桌面应用静态壳验证交接

## 状态

验证线任务已完成，可回收。

## 做了什么

- 复核静态网页壳能通过本地 server 打开。
- 复核静态壳能读取 `codex-index.json` 和 `tasks/README.md`。
- 复核 6 个页面可切换。
- 复核核心数量、warning 展示和任务队列栏目。
- 复核源码没有读取敏感文件或正文类字段。
- 用 Chrome headless + DevTools Protocol 补了浏览器 smoke。

## 验证了哪些文件和页面

文件：

- `product-line/prototypes/desktop-app/index.html`
- `product-line/prototypes/desktop-app/app.js`
- `product-line/prototypes/desktop-app/styles.css`
- `product-line/prototypes/index-kernel/codex-index.json`
- `product-line/tasks/README.md`

页面：

- 首页：`home`
- 项目页：`projects`
- 会话页：`sessions`
- Skills 页：`skills`
- Harness 页：`harness`
- 任务线页：`tasks`

## 使用了哪些命令或浏览器工具

语法检查：

```bash
node --check product-line/prototypes/desktop-app/app.js
```

本地 server：

```bash
cd product-line
python3 -m http.server 8765
```

HTTP 检查：

```bash
curl -sS -I http://127.0.0.1:8765/prototypes/desktop-app/
curl -sS -I http://127.0.0.1:8765/prototypes/desktop-app/app.js
curl -sS -I http://127.0.0.1:8765/prototypes/index-kernel/codex-index.json
curl -sS -I http://127.0.0.1:8765/tasks/README.md
```

浏览器 smoke：

- Google Chrome headless。
- Chrome DevTools Protocol。
- Node 内置 `WebSocket` 和 `http`。

没有使用：

- Playwright wrapper。原因：本地无 `playwright-cli`，Node 包 `playwright` / `puppeteer` 不存在，wrapper 会通过 `npx --package @playwright/cli` 拉网络依赖；任务包禁止安装网络依赖。
- harness。任务包明确禁止自动运行 harness。

## 哪些验证通过

通过：

- `node --check` 通过。
- `index.html` HTTP 200，大小 5822。
- `app.js` HTTP 200，大小 18349。
- `styles.css` HTTP 200，大小 8401。
- `codex-index.json` HTTP 200，大小 326389。
- `tasks/README.md` HTTP 200，大小 2698。
- 本地 server 验证后已清理：`kill 44065`，复查 8765 无监听进程。
- 6 个导航按钮和 6 个 view 一一对应。
- 浏览器中 6 个页面可切换：`view-home`、`view-projects`、`view-sessions`、`view-skills`、`view-harness`、`view-tasks`。
- 页面控制台错误：0。
- 页面异常：0。
- DOM 计数：项目卡 30、会话行 296、skill 卡 50、harness 卡 132、任务列 4。
- 当前静态索引数量：项目 30、会话 296、skills 50、plugins 11、harness 候选 132。
- warning 摘要：`title_truncated=65`。
- 任务队列解析：待派发 1、进行中 0、已回收 8、暂停 1。

正文和敏感边界：

- `app.js` 只 fetch `../index-kernel/codex-index.json` 和 `../../tasks/README.md`。
- 源码关键词检查没有命中 `auth.json`、`.env`、`secret`、`token`、`authorization`、`payload.content`、`first_user_message`、`preview`、`writeFile`、`child_process`、`exec(`、`spawn(`。
- 未发现页面读取或展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。

## 哪些验证无法完成，原因是什么

未完成：

- Playwright 方式没有完成。

原因：

- 本地没有全局 `playwright-cli`。
- Node 包 `playwright` 和 `puppeteer` 不存在。
- 可用 wrapper 需要通过 `npx --package @playwright/cli` 获取依赖，任务包禁止安装网络依赖。

替代证据：

- HTTP 响应检查。
- 静态 DOM 和源码检查。
- 静态索引数据解析。
- Chrome headless + CDP 浏览器 smoke。

## 是否建议退回桌面应用线修改

不建议退回。

依据：

- 静态壳本轮验证满足任务包验收标准。
- 发现的问题是 favicon 404 和静态网页能力边界，不影响阶段 2 静态壳验收。

建议后续修：

- 加 favicon，减少 404 噪音。
- 指标 DOM 文本增加分隔或 `aria-label`，让自动化读取更清楚。
- 真桌面能力另开容器方案，不在静态壳里硬补。

## 风险和下一步建议

风险：

- 静态壳不是最终桌面应用，不能打开文件夹、定位日志、执行系统动作。
- 静态索引含本机工作上下文：项目路径、rollout 路径、skill 路径、标题、模型和 token 统计。
- 会话标题仍可能带上下文；当前 `title_truncated=65` 只能说明截断生效。
- 本地 server 在沙箱内不能直接启动，需要提升权限。

下一步：

- 回收线接受本轮验证后，可继续把静态壳作为阶段 2 只读入口。
- 若要进入桌面能力阶段，先确认 Electron / Tauri / 继续纯网页路线和权限边界。
