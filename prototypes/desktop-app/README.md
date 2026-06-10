# 桌面应用静态索引壳

这是阶段 2 的只读静态网页壳原型。

## 读取范围

- 读取：`product-line/prototypes/index-kernel/codex-index.json`
- 读取：`product-line/tasks/README.md`
- 不读取 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 不运行 harness。
- 不写 Codex 状态库。

## 本地运行

从 `product-line` 目录启动本地静态 server：

```bash
python3 -m http.server 8765
```

然后打开：

```text
http://localhost:8765/prototypes/desktop-app/
```

直接双击 `index.html` 时，浏览器可能因为本地文件读取限制而无法加载 JSON。这个原型推荐用上面的本地 server。

## 页面

- 首页：项目数、会话数、skills 数、plugins 数、harness 候选数、warning 摘要、数据生成时间。
- 项目页：项目路径、线程数、authority / handoff / evidence / harness 候选、context warning。
- 会话页：会话标题、编号、项目路径、更新时间、归档状态、rollout 是否存在、warning。
- Skills 页：本地、系统、插件 skill 的来源、路径、插件名和 warning。
- Harness 页：候选入口、所属项目、入口类型、来源、更新时间；没有运行按钮。
- 任务线页：从任务队列 Markdown 解析待派发、进行中、已回收、暂停。

## 已知限制

- 项目类型不自动判定，证据不足时显示未知。
- authority / handoff / evidence 只作为候选展示，不自动判定为当前权威。
- harness 只作为候选展示，不判断是否可用。
- 任务线解析是轻量 Markdown 解析，不是最终任务状态协议。
