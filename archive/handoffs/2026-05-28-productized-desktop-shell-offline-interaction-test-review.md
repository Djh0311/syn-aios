# 产品化桌面壳离线前端交互测试总指导回收意见

## 回收对象

- 任务包：`product-line/tasks/2026-05-28-productized-desktop-shell-offline-interaction-test.md`
- 开发线：验证线
- 验证线 evidence：`product-line/evidence/2026-05-28-productized-desktop-shell-offline-interaction-test.md`
- 验证线 handoff：`product-line/handoffs/2026-05-28-productized-desktop-shell-offline-interaction-test-result.md`
- 被验证产物：`product-line/prototypes/productized-desktop-shell/`

## 结论

接受为“权限确认弹层的离线前端交互测试结果”。

不接受为“完整 Tauri WebView 端到端自动化通过”，也不接受为“Finder / 剪贴板真实动作验证通过”。

依据：

- 验证线新增 `npm run test:offline-interaction`。
- 测试覆盖项目页 `打开目录`、项目页 `复制路径`、会话页 `定位`、会话页 `复制`。
- 每个场景都验证 `PermissionDialog` 显示动作、目标路径、路径来源、取消、确认执行。
- 测试不启动 Tauri，不启动 dev server，不调用后端命令，不执行 Finder、`open`、`pbcopy`，不读取剪贴板。
- 总指导线复跑 `npm run test:offline-interaction`、`npm run typecheck`、`npm run build`、`cargo test --offline` 均通过。
- 总指导线复核 `lsof -nP -iTCP:5173 -sTCP:LISTEN` 无监听输出。

## 先说薄弱点

- 这不是 Tauri WebView 端到端点击证据。依据：测试在 Node 中渲染 React 组件并检查组件输出，不启动 Tauri。
- 这不验证点击 `确认执行` 后的后端调用链。依据：测试明确只点 `取消`，并验证不触发确认执行。
- 这不验证 Finder 打开、Finder 定位、系统剪贴板内容。依据：测试不执行系统动作，验证线也明确列为非覆盖对象。
- 当前 UI 方向已经调整为 Codex 工作台首页四入口和项目级可视化工作流；这份测试只补旧产品化桌面壳一期的权限弹层证据，不代表旧 UI 可作为最终方向继续扩展。

## 已接受内容

- `package.json` 新增 `test:offline-interaction` 命令。
- `tsconfig.json` 纳入 `tests`，让测试文件进入类型检查。
- `scripts/run-offline-interaction-test.mjs` 使用本地 `esbuild` 编译并运行测试。
- `tests/offline-permission-dialog.test.tsx` 覆盖 4 个权限确认场景。
- 4 个场景均验证：
  - `PendingAction` 内容正确。
  - `PermissionDialog` 显示 `本机动作确认`。
  - 显示动作名称。
  - 显示 `目标路径` 和具体路径。
  - 显示 `路径来源` 和具体来源。
  - 显示 `取消` 和 `确认执行`。
  - 点击 `取消` 触发关闭回调。
  - 不触发确认执行。

## 总指导线复跑验证

在 `product-line/prototypes/productized-desktop-shell/`：

```bash
npm run test:offline-interaction
npm run typecheck
npm run build
```

结果：

- `npm run test:offline-interaction` 通过，4 个场景通过。
- `npm run typecheck` 通过。
- `npm run build` 通过。

在 `product-line/prototypes/productized-desktop-shell/src-tauri/`：

```bash
CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home \
CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target \
cargo test --offline
```

结果：

- 3 个 Rust 单测通过。

端口复核：

```bash
lsof -nP -iTCP:5173 -sTCP:LISTEN
```

结果：

- 无监听输出。

## 安全边界判断

接受当前安全边界。

依据：

- 验证线记录未写 `/Users/yoyi/.codex`。
- 验证线记录未改真实 Codex 状态库。
- 验证线记录未读取或展示授权文件内容、密钥、令牌、`.env` 内容。
- 验证线记录未读取或展示会话正文、工具输出、命令输出、输入历史或记忆正文。
- 验证线记录未运行真实 harness。
- 验证线记录未实际执行 Finder 打开、Finder 定位、`open`、`pbcopy`。
- 验证线记录未读取系统剪贴板。
- 验证线记录未拉取外网依赖。

## 与 UI 重设计的关系

这份测试可以作为产品化桌面壳一期的安全交互补证。

但当前 UI 方向已经确认调整为：

- 首页四入口。
- 项目级可视化工作流。
- Skill 管理看板。
- Harness 管理看板。

因此这份测试回收后，不应继续把旧索引浏览壳当作最终 UI 方向扩展。

下一步仍应执行信息架构线任务：

- `product-line/tasks/2026-05-28-codex-workbench-ui-ia-redesign.md`

## 当前接收范围

接收：

- 权限确认弹层的前端离线交互证据。
- 4 个动作按钮到 `PermissionDialog` 的组件链路验证。
- 不触发后端和系统动作的安全测试方式。

不接收：

- Tauri WebView 端到端 UI 自动化通过。
- Finder / 剪贴板真实动作验证通过。
- 旧索引浏览 UI 作为最终方向。
- 完整桌面发布版。
