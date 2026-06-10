# Evidence: task draft v1 Tauri smoke

## 结论

带缺口通过。

先说薄弱点：

- 这次没有拿到真实窗口截图，也没有拿到无障碍树里的草稿列表文本，所以不能说“已经用可视证据确认草稿列表刷新成功”。
- 输入时中文输入法污染了标题和目标说明。依据：真实窗口中可见输入内容和预期英文不一致。
- 用户给的是 review 文件，不是新的任务包文件；本轮动作是根据 review 里的“下一步建议”推导出来的真实窗口 smoke。
- 这次只验证“真实 Tauri 窗口触发登记并写入工作台状态文件”这条链路，不验证真实 markdown 任务包生成、不验证 Codex 会话派发、不验证自动编排执行。

可以接受的部分：

- 真实 Tauri dev 窗口已经启动。
- 在真实窗口里进入了项目页并选择了索引内项目 `agent world`。
- 创建任务包草稿确认弹层已经打开，并显示了目标路径、来源和写入边界。
- 已尝试点击 `确认执行`。
- 真实工作台状态文件在点击确认尝试后发生了写入。依据：状态文件 `stat` 显示修改时间为 `May 29 13:20:43 2026`。
- 代码、离线交互测试、构建和 Rust 单测都通过。

## 范围

- 上游 review：`product-line/handoffs/2026-05-29-desktop-shell-task-draft-v1-review.md`
- 上游任务包：`product-line/tasks/2026-05-29-desktop-shell-task-draft-v1.md`
- 原实现 evidence：`product-line/evidence/2026-05-29-desktop-shell-task-draft-v1.md`
- 原实现 handoff：`product-line/handoffs/2026-05-29-desktop-shell-task-draft-v1-result.md`
- 原型：`product-line/prototypes/productized-desktop-shell/`
- 真实状态文件路径：`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`

没有读取真实状态文件正文。

## 已知、未知和假设

已知：

- 上游 review 明确不接受“真实 Tauri 窗口创建任务草稿已验证”。
- 上游 review 建议下一步做“任务包草稿真实窗口创建 smoke”。
- 真实状态文件在本轮开始前已经存在。
- 真实窗口里项目 `agent world` 已有本地 workflow 草稿。
- 真实窗口里确认弹层显示边界：只登记到工作台自己的 `workflow-state.v0.json`；不生成真实任务包文件；不派发真实 Codex 会话；不启动 Codex CLI。
- 状态文件修改时间在确认尝试后更新到 `May 29 13:20:43 2026`。

未知：

- 草稿列表是否在窗口中最终刷新显示，因为截图和无障碍读取都失败。
- 真实状态文件里新草稿的具体字段值，因为没有读取正文。

假设：

- 本轮只做真实窗口 smoke，不生成真实 `product-line/tasks/*.md`。
- 本轮不读状态文件正文，不读密钥、会话正文、工具输出、输入历史或记忆正文。

## 真实窗口观察

启动命令：

```bash
CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target npm run tauri:dev
```

观察到：

- 窗口标题：`Codex 治理工作台`
- 选中项目：`agent world`
- 项目路径：`/Users/yoyi/gameai/agent world`
- 状态文件面板：`exists=true`
- 状态文件面板：`workflows=1`
- 状态文件面板：`nodes=7`
- 状态文件面板：`edges=6`
- 状态文件面板：`audit events=2`
- 创建任务包草稿表单可见。
- 表单提交前显示当前 workflow 下还没有任务包草稿。

输入污染：

- 预期标题：`task draft tauri smoke`
- 实际可见标题约为：`task draft他日smoke`
- 预期目标说明：`register work_items and artifacts through real Tauri window`
- 实际可见目标说明混入中文输入法结果，约为：`register我入坑艾特没事爱你的artifacts through real Taurinewindow`

确认弹层：

- 标题：`创建任务包草稿`
- 目标路径：`/Users/yoyi/gameai/agent world`
- 来源：`索引内项目路径`
- 边界：`只登记到工作台自己的 workflow-state.v0.json；不生成真实任务包文件、不派发真实 Codex 会话、不启动 Codex CLI。`
- 默认指派：`codex-dev`

随后尝试点击 `确认执行`。

## 状态文件证据

本轮开始时状态文件存在：

```bash
test -e '/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json'
```

结果：

- 返回码 0。

点击确认尝试后复核：

```bash
stat -f '%Sm %z %N' '/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json'
```

结果：

- `May 29 13:20:43 2026 11999 /Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`

判断：

- 真实状态文件在本轮真实窗口操作期间发生写入。
- 由于没有读取正文，只能证明文件被工作台链路写过，不能证明新草稿字段内容。

## 验证命令

在 `product-line/prototypes/productized-desktop-shell/` 执行：

```bash
npm run typecheck
npm run test:offline-interaction
npm run build
```

结果：

- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过，输出 `offline interaction tests passed: 3`。
- `npm run build` 通过。

Rust 离线测试：

```bash
CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline
```

结果：

- 16 个 Rust 测试通过。

## 截图和进程验证缺口

截图：

- 普通 `screencapture -x /private/tmp/codex-workbench-smoke.png` 失败，输出 `could not create image from display`。
- 提权截图请求被拒绝，理由是全屏截图会捕获 Tauri 窗口外的无关内容。
- 因此没有可视截图 evidence。

无障碍读取：

- Computer Use 不能读取目标 Tauri app，返回 `Invalid app: codex-governance-workbench`。
- 列出应用后尝试读取前台 app，返回 `Computer Use permissions are not granted`。
- 因此没有无障碍树 evidence。

进程检查：

- `lsof -nP -iTCP:5173 -sTCP:LISTEN` 在停止 Tauri dev session 后无输出，返回码 1。
- `pgrep` / `ps` 在当前环境下受限，不能作为本轮进程清理依据。

## 清理

- 已向 Tauri dev session 发送 `Ctrl-C`。
- 端口 `5173` 复核为无监听。
- 真实状态文件保留。

## 禁止项复核

没有做：

- 读取真实 `workflow-state.v0.json` 正文。
- 生成真实 `product-line/tasks/*.md` 任务包文件。
- 启动 Codex CLI。
- 派发真实 Codex 会话。
- 运行 harness。
- 写 `/Users/yoyi/.codex`。
- 修改 Codex 状态库。
- 写项目业务目录。
- 读取或展示 `.env`、`auth.json`、密钥、令牌、授权文件内容。
- 读取或展示 Codex 会话正文、工具输出、输入历史、记忆正文。

## 后续建议

- 如果要把这条从“带缺口通过”升级成“完整真实窗口通过”，需要用户明确允许只截取 Tauri 窗口或授予无障碍读取权限。
- 下一轮最好先切换英文输入法，避免草稿标题和目标说明再次被输入法污染。
