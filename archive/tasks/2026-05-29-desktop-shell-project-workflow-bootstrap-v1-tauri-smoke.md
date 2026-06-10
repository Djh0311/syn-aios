# 任务包：项目默认工作流草稿真实窗口创建 smoke

## 任务名

在真实 Tauri 窗口中创建一个项目默认工作流草稿，并验证 UI 刷新和工作台状态文件存在。

## 所属开发线

验证线。

当前派给验证线，因为任务核心是 smoke 验证，不新增功能。

## 背景

桌面应用线已实现项目默认工作流草稿初始化 v1。

依据：

- `product-line/tasks/2026-05-29-desktop-shell-project-workflow-bootstrap-v1.md`
- `product-line/evidence/2026-05-29-desktop-shell-project-workflow-bootstrap-v1.md`
- `product-line/handoffs/2026-05-29-desktop-shell-project-workflow-bootstrap-v1-result.md`
- `product-line/handoffs/2026-05-29-desktop-shell-project-workflow-bootstrap-v1-review.md`

总指导回收结论：

- 接受为“项目默认工作流草稿初始化 v1 已实现”。
- 不能说真实 Tauri 窗口点击创建链路已验证。

本任务补这个缺口。

## 目标

在真实 Tauri dev 窗口中验证：

- 工作台能正常启动。
- 项目页能显示“项目工作流草稿”区。
- 选中一个索引内项目后，可以打开“创建默认工作流草稿”确认弹层。
- 用户确认后，工作台自己的状态文件被创建或更新。
- UI 刷新后显示当前项目已有本地工作流草稿。
- UI 显示 workflow id、state、nodes、edges。
- 默认节点数量应为 7，默认边数量应为 6。

真实状态文件路径：

```text
/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json
```

这次允许创建这个文件，因为它是工作台自己的本地状态文件，不是 Codex 状态库。

## 允许读取

- `product-line/STAGE_PLAN.md`
- `product-line/README.md`
- `product-line/tasks/README.md`
- `product-line/tasks/2026-05-29-desktop-shell-project-workflow-bootstrap-v1.md`
- `product-line/evidence/2026-05-29-desktop-shell-project-workflow-bootstrap-v1.md`
- `product-line/handoffs/2026-05-29-desktop-shell-project-workflow-bootstrap-v1-result.md`
- `product-line/handoffs/2026-05-29-desktop-shell-project-workflow-bootstrap-v1-review.md`
- `product-line/prototypes/productized-desktop-shell/`
- `product-line/prototypes/index-kernel/codex-index.json`

允许做存在性检查，不读取内容：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`

说明：

- 优先通过真实 UI 验证 workflow / node / edge 数量。
- 不需要读取真实状态文件正文。

## 允许写入

- `product-line/evidence/`
- `product-line/handoffs/`

真实 Tauri 窗口中，允许用户确认后由工作台写入：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/`

## 禁止事项

- 不直接用脚本写真实状态文件。
- 不绕过 UI 调用后端命令。
- 不写 `/Users/yoyi/.codex`。
- 不改真实 Codex 状态库。
- 不写项目业务目录。
- 不读取或展示 `auth.json`、`.env`、密钥、令牌、授权文件内容。
- 不读取或展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 不自动运行 harness。
- 不把索引候选自动升级成“已验证能力”。
- 不接入非 Codex agent。
- 不做知识库、向量搜索、LM 调度。
- 不做 release 打包。
- 不拉取外网依赖。
- 不实现自动编排执行。
- 不删除真实状态文件，除非总指导或用户明确要求。

## 验收标准

- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过。
- `npm run build` 通过。
- `cargo test --offline` 通过。
- 真实 Tauri dev 窗口启动成功。
- 窗口标题为 `Codex 治理工作台`。
- 项目页显示“项目工作流草稿”区。
- 创建前显示“未创建”或等价状态。
- 点击创建按钮后出现确认弹层。
- 确认弹层显示：
  - 写入工作台自己的 `workflow-state.v0.json`
  - 不写 `.codex`
  - 不写 Codex 状态库
  - 不写项目业务目录
  - 不派发真实 Codex 会话
  - 不生成任务包文件
- 用户确认创建。
- 创建后 UI 显示“已创建”或等价状态。
- 创建后 UI 显示：
  - `state=draft` 或等价状态
  - `nodes=7`
  - `edges=6`
- 真实状态文件存在。
- 5173 无监听残留，或说明仍在运行供用户继续试用。

## 必须回传

1. 薄弱点
2. 做了什么
3. 读了哪些文件或目录
4. 新增了哪些 evidence / handoff
5. 验证命令和结果
6. 真实 Tauri 窗口看到的关键文本
7. 选中了哪个项目
8. 是否点击确认创建
9. 真实状态文件任务前后是否存在
10. 是否保留真实状态文件
11. 是否触碰禁止事项
12. 清理了哪些进程，5173 是否无监听
13. 风险和下一步建议

## 总指导回收重点

回收时必须判断：

- 是否是真实 Tauri 窗口验证，不是普通浏览器。
- 是否通过 UI 创建，而不是脚本直写。
- 是否只写工作台自己的状态文件。
- 是否没有写 `.codex`、Codex 状态库或项目业务目录。
- 是否没有读取会话正文或敏感内容。
- 是否 UI 能显示已创建 workflow、7 个节点、6 条边。
- 是否适合继续做任务包草稿生成 v1。
