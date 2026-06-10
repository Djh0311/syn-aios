# 项目默认工作流草稿初始化 v1 总指导回收意见

## 回收对象

- 任务包：`product-line/tasks/2026-05-29-desktop-shell-project-workflow-bootstrap-v1.md`
- 开发线：桌面应用线
- Evidence：`product-line/evidence/2026-05-29-desktop-shell-project-workflow-bootstrap-v1.md`
- Handoff：`product-line/handoffs/2026-05-29-desktop-shell-project-workflow-bootstrap-v1-result.md`
- 被回收产物：`product-line/prototypes/productized-desktop-shell/`

## 结论

接受为“项目默认工作流草稿初始化 v1 已实现”。

不接受为“自动编排执行完成”，不接受为“任务包生成完成”，不接受为“节点/边编辑完成”，不接受为“状态流转完成”。

依据：

- 后端新增 `bootstrap_project_workflow` 命令。
- 后端从当前 `codex-index.json` 确认项目存在，只允许索引内项目。
- 后端写入路径固定为工作台应用数据目录下的 `workflow-state.v0.json`，不从前端接收输出路径。
- 状态文件不存在时，用户确认的 bootstrap 写入会在同一次写入里初始化最小 v0 状态，并创建项目默认 workflow 草稿。
- 已有状态文件写入前会备份。
- 写入使用临时文件 + 原子替换。
- 写入后重新读取校验。
- 同一项目已有默认 workflow 时不会重复创建 workflow / node / edge / audit。
- 前端项目详情页新增“项目工作流草稿”区，显示是否已有 workflow、workflow id、state、nodes、edges。
- 创建按钮走确认弹层，并说明写入工作台自己的状态文件，不写 `.codex`、不写 Codex 状态库、不写项目业务目录。
- 总指导线复跑验证通过。

## 先说薄弱点

- 这轮没有启动真实 Tauri 窗口点击创建按钮，所以不能说真实窗口创建链路已验证。
- 默认 workflow、node、edge 都是草稿事实，不是执行事实。
- 没有派发真实 Codex 会话，没有生成任务包文件，没有执行 harness，没有做节点拖拽、边编辑或状态转换。
- `stable_id` 当前是简单路径归一化，不是最终 hash；极端路径可能碰撞。
- 当前没有文件锁，多窗口并发写状态文件仍未解决。
- 代码里重复创建 no-op 的判断发生在已存在状态文件备份之后。结果不会重复 workflow / node / edge / audit，但重复点创建时仍可能产生一次多余备份。这个不是本轮退回项，但后续可优化。

## 接受内容

接受后端能力：

- `bootstrap_project_workflow` 命令。
- 索引内项目校验。
- 非索引项目拒绝。
- 缺状态文件时，确认式 bootstrap 初始化 v0 状态并写项目 workflow。
- 已有状态文件写入前备份。
- 原子写入。
- 防重复 workflow 创建。
- 生成项目、workflow、7 个默认节点、6 条默认边和 audit event。

接受前端能力：

- 项目详情页显示项目工作流草稿区。
- 显示 workflow 是否已创建。
- 显示 workflow id、state、node 数、edge 数。
- 未创建时提供“创建默认工作流草稿”按钮。
- 已创建时禁用创建按钮。
- 创建动作走统一确认弹层。
- 文案说明这是写工作台自己的小账本，不派发真实 Codex 会话，不生成任务包文件。

## 默认写入对象

写入 `projects[]`：

- 选中项目的本地项目事实记录。

写入 `workflows[]`：

- 一个默认 workflow。
- `state=draft`
- `workflow_version=1`
- `model_policy=none`

写入 `nodes[]`：

- 总指导 / Director
- Codex 开发线
- 验证线
- 任务包
- Handoff
- Evidence
- Review

写入 `edges[]`：

- 总指导派发任务包。
- 任务包交给 Codex 开发线。
- 开发线产出 handoff。
- 开发线产出 evidence。
- 验证线验证 evidence。
- Review 回收 handoff。

写入 `audit_events[]`：

- `project_default_workflow_bootstrapped`
- 记录用户确认创建项目默认工作流草稿。

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

- 10 个 Rust 单测通过。

真实状态文件复核：

```bash
test -e '/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json'
```

结果：

- 返回码 1。
- 无输出。
- 判断为真实状态文件当前不存在。

端口复核：

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

- 未写 `/Users/yoyi/.codex`。
- 未改真实 Codex 状态库。
- 未写项目业务目录。
- 未读取或展示 auth、env、密钥、令牌、授权文件内容。
- 未读取或展示会话正文、工具输出、命令输出、输入历史、记忆正文。
- 未自动运行 harness。
- 未把索引候选自动升级成已验证能力。
- 未接入非 Codex agent。
- 未做知识库、向量搜索、LM 调度。
- 未做 release 打包。
- 未拉取外网依赖。
- 未实现自动编排执行。

## 当前状态

这条任务从“待派发”改为“已回收”。

当前可以说：

- 桌面壳已经具备为索引内项目创建默认工作流草稿的后端能力。
- 前端已经能展示项目工作流草稿状态和创建入口。
- 默认工作流草稿会写入工作台自己的状态文件。

仍不能说：

- 自动编排执行完成。
- 任务包生成完成。
- 节点状态流转完成。
- 真实 Tauri 窗口点击创建链路已验证。
- 多会话调度完成。

## 下一步建议

下一步建议二选一：

- 验证线：做真实 Tauri 窗口 smoke，用户确认创建一个工作流草稿，然后验证状态文件存在和 UI 刷新；这会真实创建工作台状态文件。
- 桌面应用线：继续做“任务包草稿生成 v1”，把工作流里的任务包节点变成可登记的 `work_items[]` / `artifacts[]`，但仍不自动写真实任务包文件。

如果用户想尽快看可用效果，建议先走真实窗口 smoke；如果想继续补功能，建议做任务包草稿生成 v1。
