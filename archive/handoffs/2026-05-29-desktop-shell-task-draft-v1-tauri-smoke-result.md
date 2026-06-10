# Handoff: task draft v1 Tauri smoke

## 结论

接受为“任务包草稿真实 Tauri 窗口 smoke 带缺口通过”。

不接受为“完整可视 UI 回归通过”，不接受为“真实任务包 markdown 生成完成”，不接受为“真实 Codex 会话派发完成”，不接受为“自动编排执行完成”。

依据：

- 真实 Tauri dev 窗口已经打开并进入项目页。
- 真实窗口中已选择索引内项目 `agent world`。
- 真实窗口中已打开 `创建任务包草稿` 确认弹层。
- 确认弹层显示写入边界：只登记到工作台自己的 `workflow-state.v0.json`，不生成真实任务包文件，不派发真实 Codex 会话，不启动 Codex CLI。
- 已尝试点击 `确认执行`。
- 真实状态文件在本轮确认尝试后发生写入，`stat` 修改时间为 `May 29 13:20:43 2026`。
- 前端 typecheck、离线交互测试、build 和 Rust 离线测试通过。

## 先说薄弱点

- 没有截图 evidence。普通截图失败，提权截图因会捕获窗口外内容被拒绝。
- 没有无障碍树 evidence。Computer Use 权限不可用。
- 因此没有直接看到提交后的任务草稿列表刷新结果。
- 输入法污染了标题和目标说明，本轮创建出来的草稿文本不是干净的预期英文。
- 本轮没有读取状态文件正文，所以不能逐字段确认新 `work_items[]` 和 `artifacts[]` 内容。

## 做了什么

- 读取上游 review、任务包、evidence 和 handoff。
- 复核上游 review 的下一步建议。
- 复跑前端和 Rust 验证命令。
- 启动真实 Tauri dev 窗口。
- 在真实窗口里进入项目页，选择 `agent world`。
- 在已有 workflow 的项目下填写任务包草稿表单。
- 打开创建任务包草稿确认弹层。
- 核对目标、来源、默认指派和写入边界。
- 尝试点击 `确认执行`。
- 不读取真实状态文件正文，只用 `test -e` 和 `stat` 复核存在和写入时间。
- 停止 Tauri dev session。
- 复核 `5173` 无监听。

## 文件

新增 evidence：

- `product-line/evidence/2026-05-29-desktop-shell-task-draft-v1-tauri-smoke.md`

新增 handoff：

- `product-line/handoffs/2026-05-29-desktop-shell-task-draft-v1-tauri-smoke-result.md`

上游入口：

- `product-line/handoffs/2026-05-29-desktop-shell-task-draft-v1-review.md`
- `product-line/tasks/2026-05-29-desktop-shell-task-draft-v1.md`
- `product-line/evidence/2026-05-29-desktop-shell-task-draft-v1.md`
- `product-line/handoffs/2026-05-29-desktop-shell-task-draft-v1-result.md`

## 验证命令

通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline`

离线交互测试输出：

- `offline interaction tests passed: 3`

Rust 测试：

- 16 个测试通过。

## 真实窗口观察

窗口：

- 标题：`Codex 治理工作台`

项目：

- 名称：`agent world`
- 路径：`/Users/yoyi/gameai/agent world`

状态文件面板：

- `exists=true`
- `workflows=1`
- `nodes=7`
- `edges=6`
- `audit events=2`

确认弹层：

- 标题：`创建任务包草稿`
- 目标路径：`/Users/yoyi/gameai/agent world`
- 来源：`索引内项目路径`
- 边界：`只登记到工作台自己的 workflow-state.v0.json；不生成真实任务包文件、不派发真实 Codex 会话、不启动 Codex CLI。`
- 默认指派：`codex-dev`

输入污染：

- 预期标题：`task draft tauri smoke`
- 实际可见标题约为：`task draft他日smoke`
- 预期目标说明：`register work_items and artifacts through real Tauri window`
- 实际可见目标说明混入中文输入法结果。

## 状态文件

路径：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`

复核：

- 本轮开始前存在。
- 本轮确认尝试后 `stat` 显示修改时间为 `May 29 13:20:43 2026`。
- 文件保留。
- 正文没有读取。

## 清理

已做：

- 停止 Tauri dev session。
- 复核 `5173` 无监听。

未能做：

- `pgrep` / `ps` 受当前环境限制，不能可靠读取进程列表。

## 边界复核

没有证据显示发生了以下行为：

- 生成真实 `product-line/tasks/*.md`。
- 读取真实状态文件正文。
- 启动 Codex CLI。
- 派发真实 Codex 会话。
- 运行 harness。
- 写 `/Users/yoyi/.codex`。
- 改 Codex 状态库。
- 写项目业务目录。
- 读取或展示密钥、授权文件、会话正文、工具输出、输入历史或记忆正文。

## 建议

- 如需完整回收，下一步应补一个“只截 Tauri 窗口”或“授予无障碍读取权限”的可视验证。
- 补验前先切换英文输入法，重新登记一个干净标题的草稿，或专门验证同名/脏输入的展示和去重行为。
