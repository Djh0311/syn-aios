# 桌面应用静态索引壳 evidence

## 对象

- 任务包：`product-line/tasks/2026-05-27-desktop-app-static-index-shell.md`
- 开发线：桌面应用线
- 原型目录：`product-line/prototypes/desktop-app/`

## 已完成

- 新增零依赖静态网页壳。
- 从 `product-line/prototypes/index-kernel/codex-index.json` 读取静态索引。
- 从 `product-line/tasks/README.md` 轻量解析任务状态。
- 实现 6 个页面：首页、项目页、会话页、skills 页、harness 页、任务线页。
- 页面明确显示候选、未知和 warning，不把候选写成事实。
- harness 页只展示候选入口，没有运行按钮。

## 文件

- `product-line/prototypes/desktop-app/index.html`
- `product-line/prototypes/desktop-app/styles.css`
- `product-line/prototypes/desktop-app/app.js`
- `product-line/prototypes/desktop-app/README.md`

## 展示数据

依据当前静态索引样例，浏览器 smoke 验证读取到：

- 项目：30
- 会话：296
- Skills：50
- Plugins：11
- Harness 候选：132
- 任务状态栏目：4

页面展示范围：

- 首页：项目数、会话数、skills 数、plugins 数、harness 候选数、warning 摘要、数据生成时间、最近项目。
- 项目页：项目路径、会话数、active_hint、authority / handoff / evidence / harness 候选、context warning。
- 会话页：标题、编号、项目路径、更新时间、归档状态、rollout 是否存在、模型 / reasoning 摘要、warning。
- Skills 页：skill 标题、路径、来源、本地 / 系统 / 插件分类、插件名、插件版本、warning。
- Harness 页：候选入口名、路径、所属项目、入口类型、来源、更新时间、大小、warning。
- 任务线页：待派发、进行中、已回收、暂停四类状态，只展示任务候选入口，不展开任务说明正文。

## 未展示内容

- 没有展示 Codex 会话正文。
- 没有展示工具输出、命令输出、输入历史。
- 没有展示记忆正文。
- 没有展示 README / AGENTS / handoff / evidence 正文。
- 没有展示 harness 命令正文。
- 没有读取或展示 `auth.json`、`.env`、密钥、令牌、授权文件。

## 验证

命令验证：

```bash
node --check product-line/prototypes/desktop-app/app.js
```

结果：通过，无语法错误。

本地运行验证：

```bash
cd product-line
python3 -m http.server 8765
```

浏览器打开：

```text
http://localhost:8765/prototypes/desktop-app/
```

浏览器 smoke 验证结果：

- 首页加载成功。
- `读取失败` 未出现。
- 导航按钮数量：6。
- 项目页卡片：30。
- 会话页行数：296。
- Skills 页卡片：50。
- Harness 页卡片：132。
- 任务线栏目：4。
- 页面自身控制台错误：0。

说明：

- 浏览器外壳出现过一次 Statsig 网络超时日志，不是本地页面发起的请求；页面自身没有控制台错误。
- 首次在沙箱内启动本地 server 失败，原因是端口绑定被沙箱限制；获得本地 server 放行后验证通过。

## 风险

- 静态索引 schema 还不是最终协议，当前页面做了容错，但不能作为最终字段合同。
- 项目类型没有证据来源，页面统一显示未知。
- authority / handoff / evidence 只是候选，不代表当前权威。
- harness 只是候选，不代表可运行或有用。
- 任务线解析只是轻量 Markdown 解析，不是最终任务状态协议。
- 直接打开 `index.html` 可能因为浏览器本地文件读取限制而无法读取 JSON；README 建议用本地 server。

## 下一步建议

- 由回收线决定是否接受为阶段 2 静态壳原型。
- 后续若要支持打开文件夹、复制路径、定位日志，需要进入桌面容器能力或浏览器权限设计，不能在纯静态网页里假装已经实现。
- 后续应让索引内核输出稳定任务状态字段，替代 UI 端解析 Markdown。
