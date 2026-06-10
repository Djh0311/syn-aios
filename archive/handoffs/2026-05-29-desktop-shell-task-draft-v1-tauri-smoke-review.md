# 任务包草稿真实窗口创建 smoke 总指导回收意见

## 回收对象

- 上游建议：`product-line/handoffs/2026-05-29-desktop-shell-task-draft-v1-review.md`
- Evidence：`product-line/evidence/2026-05-29-desktop-shell-task-draft-v1-tauri-smoke.md`
- Handoff：`product-line/handoffs/2026-05-29-desktop-shell-task-draft-v1-tauri-smoke-result.md`
- 被验证产物：`product-line/prototypes/productized-desktop-shell/`

## 结论

接受为“任务包草稿真实 Tauri 窗口 smoke 带缺口通过”。

不接受为“完整可视 UI 回归通过”，不接受为“逐字段确认真实 `work_items[]` / `artifacts[]` 写入正确”，不接受为“真实任务包 markdown 生成完成”，不接受为“真实 Codex 会话派发完成”，不接受为“自动编排执行完成”。

依据：

- 真实 Tauri dev 窗口已经启动。
- 真实窗口里进入项目页并选择索引内项目 `agent world`。
- 真实窗口里打开了 `创建任务包草稿` 确认弹层。
- 确认弹层显示写入边界：只登记到工作台自己的 `workflow-state.v0.json`，不生成真实任务包文件，不派发真实 Codex 会话，不启动 Codex CLI。
- 已尝试点击 `确认执行`。
- 真实状态文件在确认尝试后修改时间变为 `May 29 13:20:43 2026`。
- `npm run typecheck`、`npm run test:offline-interaction`、`npm run build`、`cargo test --offline` 均通过。

## 先说薄弱点

- 没有截图 evidence。普通截图失败，提权截图因可能捕获窗口外内容被拒绝。
- Computer Use 无障碍读取不可用，所以没有无障碍树 evidence。
- 没有直接看到提交后的任务草稿列表刷新结果。
- 输入法污染了标题和目标说明。
- 没有读取真实 `workflow-state.v0.json` 正文，所以不能逐字段确认新增草稿内容。
- `pgrep` / `ps` 受当前环境限制，不能作为本轮进程清理依据。

## 接受内容

接受以下验证事实：

- 真实 Tauri 窗口可以进入项目页。
- `agent world` 项目下已有 workflow 草稿。
- 创建任务包草稿表单可见。
- 确认弹层能打开并展示正确边界。
- 点击确认后，工作台状态文件发生写入。
- 验证后 5173 无监听残留。
- 没有证据显示生成真实 `product-line/tasks/*.md` 文件。
- 没有证据显示派发真实 Codex 会话。
- 没有证据显示写 `.codex`、Codex 状态库或项目业务目录。

## 总指导线复跑验证

在 `product-line/prototypes/productized-desktop-shell/` 复跑：

```bash
npm run typecheck
npm run test:offline-interaction
npm run build
```

结果：

- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过，输出 `offline interaction tests passed: 3`。
- `npm run build` 通过。

在 `product-line/prototypes/productized-desktop-shell/src-tauri/` 复跑：

```bash
CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home \
CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target \
cargo test --offline
```

结果：

- 16 个 Rust 单测通过。

真实状态文件复核：

```bash
test -e '/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json'
```

结果：

- 返回码 0。
- 判断为真实工作台状态文件存在。
- 总指导线没有读取状态文件正文。

端口复核：

```bash
lsof -nP -iTCP:5173 -sTCP:LISTEN
```

结果：

- 返回码 1。
- 无输出。
- 判断为 5173 当前无监听残留。

## 关于 Computer Use 权限

这轮 evidence 里记录的 Computer Use 问题是：

- 目标 Tauri app 名称读取失败，返回 `Invalid app: codex-governance-workbench`。
- 尝试读取前台 app 时返回 `Computer Use permissions are not granted`。

这不一定说明用户没有在系统设置里授权。macOS 无障碍权限按具体 app / helper / 子进程记录；Codex 主应用、Computer Use helper、终端启动的 Tauri dev app、Tauri dev 子进程可能不是同一个授权对象。授权后也可能需要重启 Codex 或系统设置中的相关进程才生效。

## 安全和范围判断

接受当前安全边界。

依据：

- 未读取真实状态文件正文。
- 未生成真实 `product-line/tasks/*.md`。
- 未启动 Codex CLI。
- 未派发真实 Codex 会话。
- 未运行 harness。
- 未写 `/Users/yoyi/.codex`。
- 未改 Codex 状态库。
- 未写项目业务目录。
- 未读取或展示 `.env`、`auth.json`、密钥、令牌、授权文件内容。
- 未读取或展示 Codex 会话正文、工具输出、输入历史或记忆正文。

## 当前状态

这条 smoke 回收为“带缺口通过”。

当前可以说：

- 真实 Tauri 窗口里能打开任务包草稿确认弹层。
- 点击确认后，工作台状态文件发生写入。
- 这条链路没有证据显示越过边界。

仍不能说：

- 任务草稿列表刷新已被可视证据确认。
- 新草稿字段逐项正确。
- 真实任务包 markdown 文件生成完成。
- 真实 Codex 会话派发完成。
- 自动编排执行完成。

## 下一步建议

下一步建议不要马上做更复杂的自动编排。

优先做二选一：

- 补 UI 验证能力：解决 Computer Use / 截图链路，拿到真实窗口可视证据。
- 继续功能：做“任务包 markdown 生成预览 v1”，但仍不直接写真实 `tasks/*.md`，先只生成预览和 artifact 草稿。
