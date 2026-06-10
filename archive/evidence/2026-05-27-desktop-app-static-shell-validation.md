# 桌面应用静态壳验证证据

## 结论先说

薄弱点：

- 这仍是静态网页壳，不是真桌面应用。依据：原型目录只有 `index.html`、`styles.css`、`app.js`、`README.md`，没有 Electron / Tauri 容器配置。
- 页面会请求 `/favicon.ico`，本地 server 返回 404。依据：Chrome smoke 时 server 日志出现 `GET /favicon.ico` 404；这不影响主页面加载，但属于可修的小噪音。
- Chrome headless 启动时出现 Google Updater / Crashpad 日志。依据：验证输出来自 Chrome 进程，不是页面 `console.error`；页面 CDP 捕获到的 `consoleErrors=[]`、`pageErrors=[]`。
- 本轮启动了本地 server。验证后用 `lsof -ti tcp:8765` 找到 PID `44065`，执行 `kill 44065`，复查 8765 已无监听进程。

可用结果：

- `node --check product-line/prototypes/desktop-app/app.js` 通过。
- 本地 server 能返回 `index.html`、`app.js`、`styles.css`、`codex-index.json`、`tasks/README.md`。
- Chrome headless + DevTools Protocol 完成浏览器 smoke：6 个页面可切换，页面控制台错误 0，页面异常 0。
- 核心数量和当前静态索引一致：项目 30、会话 296、skills 50、plugins 11、harness 候选 132。
- 页面源码只 fetch 静态索引和任务队列 Markdown，没有发现读取授权文件、正文类字段或危险执行接口。

## 本轮读取范围

按任务包读取：

- `product-line/prototypes/desktop-app/`
- `product-line/prototypes/index-kernel/codex-index.json`
- `product-line/tasks/README.md`
- `product-line/handoffs/2026-05-27-desktop-app-static-index-shell-review.md`
- `product-line/handoffs/2026-05-27-desktop-app-static-index-shell-result.md`

补充读取：

- `product-line/tasks/2026-05-27-desktop-app-static-shell-validation.md`，用于确认本轮任务目标。
- `product-line/prototypes/desktop-app/README.md`，用于确认本地运行说明。

本轮没有读取或展示：

- `/Users/yoyi/.codex` 真实文件内容。
- `auth.json`、`.env`、密钥、令牌、授权文件。
- Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。

## 本轮写入

- `product-line/evidence/2026-05-27-desktop-app-static-shell-validation.md`
- `product-line/handoffs/2026-05-27-desktop-app-static-shell-validation-result.md`

未修改：

- `product-line/prototypes/desktop-app/index.html`
- `product-line/prototypes/desktop-app/app.js`
- `product-line/prototypes/desktop-app/styles.css`
- `product-line/prototypes/index-kernel/codex-index.json`

临时写入：

- `/tmp/desktop-shell-index.html`
- `/tmp/desktop-shell-app.js`
- `/tmp/desktop-shell-codex-index.json`
- `/tmp/desktop-shell-tasks.md`
- `/tmp/desktop-shell-styles.css`
- `/private/tmp/chrome-desktop-shell-validation`
- `/private/tmp/chrome-desktop-shell-cdp`

这些是本轮本地验证临时文件或 Chrome 临时 profile。

## 验证命令和结果

语法检查：

```bash
node --check product-line/prototypes/desktop-app/app.js
```

结果：通过，退出码 0。

本地 server：

```bash
cd product-line
python3 -m http.server 8765
```

说明：

- 沙箱内直接启动返回 `PermissionError: [Errno 1] Operation not permitted`。
- 提升权限后启动成功。

HTTP HEAD：

```bash
curl -sS -I http://127.0.0.1:8765/prototypes/desktop-app/
curl -sS -I http://127.0.0.1:8765/prototypes/desktop-app/app.js
curl -sS -I http://127.0.0.1:8765/prototypes/index-kernel/codex-index.json
curl -sS -I http://127.0.0.1:8765/tasks/README.md
```

结果：

- `index.html`：`HTTP/1.0 200 OK`，`Content-Length: 5822`
- `app.js`：`HTTP/1.0 200 OK`，`Content-Length: 18349`
- `codex-index.json`：`HTTP/1.0 200 OK`，`Content-Length: 326389`
- `tasks/README.md`：`HTTP/1.0 200 OK`，`Content-Length: 2698`

HTTP GET：

```bash
curl -sS -o /tmp/desktop-shell-index.html -w '%{http_code} %{size_download}\n' http://localhost:8765/prototypes/desktop-app/
curl -sS -o /tmp/desktop-shell-app.js -w '%{http_code} %{size_download}\n' http://localhost:8765/prototypes/desktop-app/app.js
curl -sS -o /tmp/desktop-shell-codex-index.json -w '%{http_code} %{size_download}\n' http://localhost:8765/prototypes/index-kernel/codex-index.json
curl -sS -o /tmp/desktop-shell-tasks.md -w '%{http_code} %{size_download}\n' http://localhost:8765/tasks/README.md
curl -sS -o /tmp/desktop-shell-styles.css -w '%{http_code} %{size_download}\n' http://localhost:8765/prototypes/desktop-app/styles.css
```

结果：

- `index.html`：`200 5822`
- `app.js`：`200 18349`
- `codex-index.json`：`200 326389`
- `tasks/README.md`：`200 2698`
- `styles.css`：`200 8401`

server 清理：

```bash
lsof -ti tcp:8765
kill 44065
lsof -ti tcp:8765
```

结果：

- 清理前 PID：`44065`
- 清理后无输出，`lsof` 退出码为 1，表示 8765 没有监听进程。

## 数据解析复核

命令：用 Node 解析 `product-line/prototypes/index-kernel/codex-index.json`。

结果：

```json
{
  "projects": 30,
  "threads": 296,
  "skills": 50,
  "plugins": 11,
  "harness_candidates": 132,
  "warning_summary": {
    "title_truncated": 65
  },
  "generated_at": "2026-05-27T10:23:52Z"
}
```

任务队列解析结果：

```json
{
  "pending": 1,
  "active": 0,
  "done": 8,
  "paused": 1
}
```

## 静态 DOM 和源码检查

`index.html` 检查结果：

```json
{
  "nav": ["home", "projects", "sessions", "skills", "harness", "tasks"],
  "views": ["home", "projects", "sessions", "skills", "harness", "tasks"]
}
```

`app.js` fetch 目标：

```json
[
  ["INDEX_URL", "../index-kernel/codex-index.json"],
  ["TASKS_URL", "../../tasks/README.md"]
]
```

敏感或正文类关键词检查：

- 检查词：`auth.json`、`.env`、`secret`、`token`、`authorization`、`localStorage`、`sessionStorage`、`writeFile`、`child_process`、`exec(`、`spawn(`、`payload.content`、`first_user_message`、`preview`
- 结果：`forbidden_hits=[]`

## 浏览器 smoke

工具：

- 本机 Google Chrome headless。
- Chrome DevTools Protocol。
- Node 内置 `WebSocket` 和 `http`，没有安装 Playwright / Puppeteer 依赖。

Playwright 情况：

- `playwright-cli` 不存在。
- Node 包 `playwright` / `puppeteer` 不存在。
- wrapper 需要 `npx --package @playwright/cli` 拉依赖，任务包禁止安装网络依赖，所以未使用。

Chrome CDP smoke 结果：

```json
{
  "dataLoaded": "已加载页面只使用静态索引和任务队列 Markdown，不读取 Codex 会话正文。",
  "metrics": [
    "项目301 个 active_hint 为真",
    "会话29658 个已归档",
    "Skills507 个本地或系统，43 个插件",
    "Plugins11来自插件 manifest 元数据",
    "Harness 候选1322 个项目根缺失"
  ],
  "navResults": [
    "view-home",
    "view-projects",
    "view-sessions",
    "view-skills",
    "view-harness",
    "view-tasks"
  ],
  "counts": {
    "projectCards": 30,
    "sessionRows": 296,
    "skillCards": 50,
    "harnessCards": 132,
    "taskColumns": 4
  },
  "consoleErrors": [],
  "pageErrors": []
}
```

说明：

- `metrics` 文本来自 DOM `textContent`，数字和文字之间没有空格，例如 `项目301 个 active_hint 为真` 应读作项目 `30`、`1` 个 active_hint。
- 6 个导航页均可切换。
- 控制台错误为 0。
- 页面异常为 0。

## 正文和敏感内容风险

本轮未发现静态壳主动读取敏感文件或正文类字段。

依据：

- `app.js` 只 fetch `../index-kernel/codex-index.json` 和 `../../tasks/README.md`。
- 源码关键词检查没有命中 `auth.json`、`.env`、`payload.content`、`first_user_message`、`preview`、`writeFile`、`child_process`、`exec(`、`spawn(`。
- 页面渲染的会话页字段来自静态索引元数据：标题、编号、项目路径、更新时间、归档、rollout 存在、模型 / 推理、warning。

仍有风险：

- 静态索引本身包含项目路径、rollout 路径、skill 路径、标题、模型和 token 统计，这些不是密钥，但属于本机工作上下文。
- 会话标题可能来自用户输入或任务包摘要，`title_truncated=65` 表明有 65 条标题被截断；截断降低泄漏面，但不能证明标题完全不含敏感信息。

## 是否建议退回桌面应用线修改

不建议退回。

依据：

- 任务包要求的本地读取、6 页切换、数量复核、warning 展示、正文/敏感边界复核均有证据通过。
- 发现的 favicon 404 是小噪音，不影响阶段 2 静态壳验收。

建议后续修：

- 加一个静态 favicon 或关闭 favicon 请求噪音。
- 指标 DOM 可加分隔符或 `aria-label`，让自动化读取更清楚。
- 如果要进入真正桌面应用阶段，需要另开容器和权限边界设计，不应在静态壳里补桌面能力。

## 风险和下一步

风险：

- 本轮 smoke 是 Chrome headless + CDP，不是 Playwright；但它是实际浏览器复核。
- 本地 server 需要提升权限启动，说明沙箱默认不允许绑定端口。
- 未做移动端/多 viewport 布局截图复核；任务包没有强制要求布局截图。
- 静态壳仍不能打开文件夹、定位日志、执行系统动作。

下一步建议：

- 回收线接受本轮验证后，阶段 2 可以把静态壳作为只读入口继续推进。
- 真桌面能力不要混在静态网页壳里补，先确定 Electron / Tauri / 继续纯网页的路线。
