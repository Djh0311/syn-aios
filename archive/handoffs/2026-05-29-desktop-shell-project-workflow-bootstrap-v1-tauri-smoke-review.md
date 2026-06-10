# 项目默认工作流草稿真实窗口创建 smoke 总指导回收意见

## 回收对象

- 任务包：`product-line/tasks/2026-05-29-desktop-shell-project-workflow-bootstrap-v1-tauri-smoke.md`
- 开发线：验证线
- Evidence：`product-line/evidence/2026-05-29-desktop-shell-project-workflow-bootstrap-v1-tauri-smoke.md`
- Handoff：`product-line/handoffs/2026-05-29-desktop-shell-project-workflow-bootstrap-v1-tauri-smoke-result.md`

## 结论

接受为“真实 Tauri 窗口创建项目默认工作流草稿 smoke 通过”。

不接受为“自动编排执行完成”，不接受为“任务包生成完成”，不接受为“节点状态流转完成”。

依据：

- 真实 Tauri dev 窗口启动成功。
- 窗口标题为 `Codex 治理工作台`。
- 验证线选中索引内项目 `agent world`。
- 创建前 UI 显示项目工作流草稿未创建。
- 点击 `创建默认工作流草稿` 后出现确认弹层。
- 确认弹层显示目标路径、索引内项目来源和写入边界。
- 确认弹层说明写入工作台自己的 `workflow-state.v0.json`，不写 `.codex`、不写 Codex 状态库、不写项目业务目录。
- 页面文本说明不派发真实 Codex 会话，不生成任务包文件。
- 用户确认创建后，UI 显示 `state draft`、`nodes 7`、`edges 6`。
- 真实工作台状态文件任务前不存在，任务后存在，并按任务包要求保留。
- 验证后 5173 无监听残留。

## 先说薄弱点

- 这次只验证一个项目：`agent world`。
- 重复创建行为只由 Rust 单测覆盖，没有在真实窗口里重复点创建验证。
- 真实状态文件现在存在，这是本任务允许的结果；后续验证和开发要意识到工作台已经有本地状态。
- 创建的 workflow 仍是草稿骨架，不是执行记录。
- 没有派发真实 Codex 会话，没有生成任务包文件，没有运行 harness，没有状态流转。

## 接受内容

接受真实窗口能力：

- 启动真实 Tauri dev 窗口。
- 进入项目页。
- 选中索引内项目。
- 打开创建默认工作流草稿确认弹层。
- 通过 UI 确认创建。
- 创建后 UI 刷新并显示 workflow 草稿状态。
- 工作台自己的状态文件创建并保留。

接受验证命令结果：

- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过。
- `npm run build` 通过。
- `cargo test --offline` 通过，10 个 Rust 测试通过。

## 真实状态文件

路径：

```text
/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json
```

任务前：

- 不存在。

任务后：

- 存在。
- 按任务包要求保留。

总指导线只做存在性复核，没有读取文件正文。

## 总指导线复核

复核命令：

```bash
test -e '/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json'
```

结果：

- 返回码 0。
- 判断为真实工作台状态文件存在。

复核命令：

```bash
lsof -nP -iTCP:5173 -sTCP:LISTEN
```

结果：

- 返回码 1。
- 无输出。
- 判断为 5173 当前无监听残留。

## 安全和范围判断

接受当前安全边界。

依据：

- 没有直接用脚本写真实状态文件。
- 没有绕过 UI 调用后端命令。
- 没有写 `/Users/yoyi/.codex`。
- 没有修改 Codex 状态库。
- 没有写项目业务目录。
- 没有读取或展示 auth、env、密钥、令牌、授权文件内容。
- 没有读取或展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 没有自动运行 harness。
- 没有把索引候选升级成已验证能力。
- 没有接入非 Codex agent。
- 没有做知识库、向量搜索、LM 调度。
- 没有 release 打包或拉取网络依赖。
- 没有自动编排执行。

## 当前状态

这条任务从“待派发”改为“已回收”。

当前可以说：

- 桌面壳能在真实 Tauri 窗口里为索引内项目创建默认工作流草稿。
- UI 能显示创建后的 workflow id、draft 状态、7 个节点和 6 条边。
- 工作台自己的状态文件已经存在并保留。

仍不能说：

- 自动编排执行完成。
- 任务包生成完成。
- 节点状态流转完成。
- 多会话调度完成。

## 下一步建议

下一步建议派桌面应用线做“任务包草稿生成 v1”。

目标应保持很窄：

- 在已有项目 workflow 下，登记一个任务包草稿到 `work_items[]` 和 `artifacts[]`。
- 不自动写真实任务包文件。
- 不派发真实 Codex 会话。
- 不做状态流转。
- 继续走用户确认、audit、备份和原子替换。
