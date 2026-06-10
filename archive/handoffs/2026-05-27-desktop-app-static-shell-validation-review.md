# 桌面应用静态壳验证回收意见

## 回收对象

- 任务包：`product-line/tasks/2026-05-27-desktop-app-static-shell-validation.md`
- 开发线：验证线
- 回传 evidence：`product-line/evidence/2026-05-27-desktop-app-static-shell-validation.md`
- 回传 handoff：`product-line/handoffs/2026-05-27-desktop-app-static-shell-validation-result.md`

## 结论

接受为桌面应用静态壳验证结果。

这个结论只表示：阶段 2 静态网页壳可以作为只读入口继续推进。它仍不是真桌面应用，不能打开文件夹、定位日志或执行系统动作。

## 先说薄弱点

- 仍是静态网页壳，不是真桌面应用。依据：原型目录没有 Electron / Tauri 容器配置。
- 页面请求 `/favicon.ico` 并得到 404，这是噪音。依据：验证线 evidence 记录 server 日志出现 favicon 404。
- Playwright 没有使用。依据：本地无 `playwright-cli`，也没有 `playwright` / `puppeteer` 包；wrapper 需要网络拉依赖，而任务包禁止安装网络依赖。
- 静态索引仍包含本机工作上下文，例如项目路径、rollout 路径、skill 路径、标题、模型和 token 统计。依据：验证线 evidence 的风险说明。
- `title_truncated=65` 表明标题截断仍在发生；这降低展示面，但不能证明标题完全不含敏感上下文。依据：验证线数据解析复核。

## 复核结果

回收线复核通过：

- `node --check product-line/prototypes/desktop-app/app.js` 通过。
- 当前静态索引数量：项目 30、会话 296、skills 50、plugins 11、harness 候选 132。
- `index.html` 包含 6 个视图：home、projects、sessions、skills、harness、tasks。
- `app.js` fetch 目标为 `../index-kernel/codex-index.json` 和 `../../tasks/README.md`。
- `app.js` 未命中 `auth.json`、`.env`、`payload.content`、`first_user_message`、`preview`、`writeFile`、`child_process`、`exec(`、`spawn(`。
- 8765 端口无监听进程，验证 server 已清理。

接受验证线证据：

- 本地 server 返回 `index.html`、`app.js`、`styles.css`、`codex-index.json`、`tasks/README.md`，全部 HTTP 200。
- Chrome headless + DevTools Protocol browser smoke 通过。
- 6 个页面可切换。
- 页面控制台错误 0，页面异常 0。
- DOM 计数：项目卡 30、会话行 296、skill 卡 50、harness 卡 132、任务列 4。

## 当前生效结论

- 阶段 2 静态网页壳验收通过。
- 静态壳可以作为只读治理入口继续使用。
- 不能把静态壳称为最终桌面应用。
- 真桌面能力必须先做路线决策和权限边界设计。
- 后续可修小噪音：favicon 404、指标 DOM 文本缺少分隔。

## 派生任务

- 新增总指导线决策任务：`product-line/tasks/2026-05-27-desktop-container-route-decision.md`

## 状态

已回收，接受；下一步由总指导线做桌面容器路线决策，不直接开 Electron / Tauri 实现。
