# 任务包：桌面壳工作流事实层 v0 真实窗口验证

## 任务名

验证产品化桌面壳在真实 Tauri 窗口里展示工作流事实层 v0，并确认初始化动作受控。

## 所属开发线

验证线。

当前先派给验证线，因为任务核心是 smoke 验证和安全边界复核。

## 背景

桌面应用线已实现工作流事实层 v0 最小读写底座。

总指导回收结论：

- 接受为“工作流事实层 v0 最小读写底座已实现”。
- 不接受为“可编辑工作流完成”。
- 真实 Tauri 窗口里 v0 面板和确认弹层还没有验证。

依据：

- `product-line/handoffs/2026-05-28-desktop-shell-workflow-state-v0-review.md`
- `product-line/evidence/2026-05-28-desktop-shell-workflow-state-v0.md`
- `product-line/handoffs/2026-05-28-desktop-shell-workflow-state-v0-result.md`
- `product-line/decisions/2026-05-28-workflow-state-storage-v0.md`

## 目标

- 在真实 Tauri dev 窗口中启动 `product-line/prototypes/productized-desktop-shell/`。
- 验证窗口能读取索引并加载工作流事实层状态。
- 在真实状态文件不存在时，验证页面显示：
  - `本地事实层 v0`
  - `exists=false`
  - `未初始化`
  - `状态文件不存在；不会自动创建。`
  - `workflow-state.v0.json`
- 点击“初始化工作流事实层”按钮，只验证确认弹层出现。
- 确认弹层必须显示：
  - 目标路径
  - `Tauri 应用数据目录`
  - 写入边界
  - `workflow-state.v0.json`
  - `backups`
  - 不写 `.codex`
  - 不写 Codex 状态库
  - 不写项目业务目录
  - 追加 audit event
  - 原子替换
- 不点击“确认执行”。
- 取消弹层。
- 验证后复核真实状态文件仍不存在。
- 验证后清理 Tauri / Vite / cargo-tauri 进程，并复核 5173 无监听残留。

## 允许读取

- `product-line/STAGE_PLAN.md`
- `product-line/README.md`
- `product-line/DEV_LINES.md`
- `product-line/PROTOTYPE_WORK_LINES.md`
- `product-line/tasks/README.md`
- `product-line/tasks/2026-05-28-desktop-shell-workflow-state-v0.md`
- `product-line/handoffs/2026-05-28-desktop-shell-workflow-state-v0-review.md`
- `product-line/evidence/2026-05-28-desktop-shell-workflow-state-v0.md`
- `product-line/handoffs/2026-05-28-desktop-shell-workflow-state-v0-result.md`
- `product-line/decisions/2026-05-28-workflow-state-storage-v0.md`
- `product-line/prototypes/productized-desktop-shell/`

允许做存在性检查，不读取内容：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`

## 允许写入

- `product-line/evidence/`
- `product-line/handoffs/`

## 禁止事项

- 不点击初始化弹层里的“确认执行”。
- 不创建真实 `workflow-state.v0.json`。
- 不写 `/Users/yoyi/.codex`。
- 不改真实 Codex 状态库。
- 不写项目业务目录。
- 不读取或展示 `auth.json`、`.env`、密钥、令牌、授权文件内容。
- 不读取或展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 不自动运行 harness。
- 不把索引候选自动升级成本地事实。
- 不接入非 Codex agent。
- 不做知识库、向量搜索、LM 调度。
- 不做 release 打包。
- 不拉取外网依赖。

## 验收标准

- 有 evidence 和 handoff。
- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过。
- `npm run build` 通过。
- `cargo test --offline` 通过。
- 真实 Tauri 窗口 smoke 通过。
- 缺真实状态文件时显示 `exists=false` 和未初始化 warning。
- 初始化确认弹层可打开。
- 确认弹层显示目标路径、来源和写入边界。
- 没有点击确认执行。
- 验证后真实状态文件仍不存在。
- 验证后 5173 无监听残留。

## 必须回传

1. 做了什么
2. 读了哪些文件或目录
3. 新增了哪些 evidence / handoff
4. 验证命令和结果
5. 真实 Tauri 窗口看到的关键文本
6. 是否打开初始化确认弹层
7. 是否点击确认执行
8. 真实状态文件验证前后是否存在
9. 是否触碰禁止事项
10. 清理了哪些进程，5173 是否无监听
11. 风险和下一步建议

## 总指导回收重点

回收时必须判断：

- 是否是真实 Tauri 窗口验证。
- 是否没有创建真实状态文件。
- 是否没有点击确认执行。
- 是否确认弹层清楚展示写入边界。
- 是否没有写 `.codex`、Codex 状态库或项目业务目录。
- 是否清理进程。
