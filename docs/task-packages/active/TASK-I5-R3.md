# Task Package: TASK-I5-R3 — 索引文件名 UI 可发现性离线返工

authority-schema: harness-active/v3
authority-id: TASK-I5-R3
authority-status: ACTIVE
outcome: PENDING
mode: DEVELOPMENT
owner: 执行线
acceptance-owner: 指导线
accepted-by: PENDING
updated-at: 2026-07-28T00:00:00+08:00
goal: 让 Markdown basename 在文件树中直接可见，保留 exact relative-path 点击。
next-action: 按红先行顺序完成两文件离线返工并跑聚焦验证。
git-disposition: PENDING

> 本文件是 v0.5 节点 `TASK-I5-R3`（`.git/adaptive-harness/` 正主）的 v3 读取投影；
> 事实以节点为准，本文件不单独授权。

## 负责哪块

在 NativeKnowledgeWorkspace.tsx 与配套聚焦测试内做最小改动：用真实
WorkspaceTreeEntry fixture 先让"可见文本没有 00-index.md"断言失败，
再让文件树每个 Markdown 行直接显示 basename。

## 边界（允许读写、禁止）

### 允许读写

仅 write-scope 列出的组件文件与它的聚焦测试文件：
`prototypes/productized-desktop-shell/src/components/NativeKnowledgeWorkspace.tsx`、
`prototypes/productized-desktop-shell/src/knowledge-workbench-shell.test.tsx`。

### 禁止

其余一切（样式、图、画布、Rust 后端、seed、launcher、既有产品 WIP）
不在本包；不启动任何 App/Vite/浏览器/Computer Use；不做 stage/commit/push。

## 交付什么

basename 直显 + exact relative-path 点击保留，红先行顺序满足。

## 怎么验证

typecheck 与 37-entry offline interaction 聚焦门。

## 遇到什么必须停

需要动 scope 外文件、需要运行授权、或 Git 现实无法唯一核对时停止。
