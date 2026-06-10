# 产品化桌面壳离线前端交互测试 evidence

任务包：`product-line/tasks/2026-05-28-productized-desktop-shell-offline-interaction-test.md`

验证时间：2026-05-28

## 结论

本轮补上了权限确认弹层的离线前端交互测试。这个测试不依赖外网，不启动 Tauri，不启动 dev server，不调用后端命令，也不执行 Finder、`open`、`pbcopy` 或读取剪贴板。

需要明确边界：这补的是前端组件交互证据，不是完整 Tauri WebView 端到端自动化，也不是 Finder / 剪贴板真实动作验证。

## 改动文件

- `product-line/prototypes/productized-desktop-shell/package.json`
- `product-line/prototypes/productized-desktop-shell/tsconfig.json`
- `product-line/prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs`
- `product-line/prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

## 测试实现

新增命令：

```bash
npm run test:offline-interaction
```

实现方式：

- 用本地已有 `esbuild` 编译 `tests/offline-permission-dialog.test.tsx` 到临时目录。
- 在 Node 中执行编译后的测试文件。
- 直接渲染 React 组件树，不需要浏览器、DOM、Playwright、网络依赖。
- 通过组件按钮的 `onClick` 捕获 `PendingAction`。
- 把捕获到的 `PendingAction` 传给 `PermissionDialog`。
- 检查弹层文本是否包含动作、路径、来源、取消、确认执行。
- 点击 `取消` 回调，验证不会触发确认执行。

`tsconfig.json` 已把 `tests` 纳入 `npm run typecheck`，避免测试文件绕过类型检查。

## 覆盖场景

离线测试覆盖 4 个场景：

1. 项目页 `打开目录`
   - 期望 action：`open-project`
   - 期望动作名：`打开项目目录`
   - 期望来源：`索引内项目路径`
2. 项目页 `复制路径`
   - 期望 action：`copy`
   - 期望动作名：`复制项目路径`
   - 期望来源：`索引内项目路径`
3. 会话页 `定位`
   - 期望 action：`reveal-rollout`
   - 期望动作名：`定位 rollout 文件`
   - 期望来源：`索引内 rollout 路径`
4. 会话页 `复制`
   - 期望 action：`copy`
   - 期望动作名：`复制 rollout 路径`
   - 期望来源：`索引内 rollout 路径`

每个场景都验证弹层包含：

- `本机动作确认`
- 动作名称
- `目标路径`
- 具体路径
- `路径来源`
- 具体来源
- `取消`
- `确认执行`

## 验证命令和结果

在 `product-line/prototypes/productized-desktop-shell/` 执行：

```bash
npm run test:offline-interaction
```

结果：

```text
offline interaction tests passed: 4
```

执行：

```bash
npm run typecheck
```

结果：通过。

执行：

```bash
npm run build
```

结果：通过。

在 `product-line/prototypes/productized-desktop-shell/src-tauri/` 执行：

```bash
CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline
```

结果：

- `path_whitelist_accepts_only_index_projects_and_rollouts`：通过。
- `snapshot_keeps_metadata_without_session_body`：通过。
- `reads_real_static_index_summary`：通过。

## 禁止事项核对

本轮未做：

- 未写 `/Users/yoyi/.codex`。
- 未修改真实 Codex 状态库。
- 未读取或展示 `auth.json`、`.env`、密钥、令牌、授权文件内容。
- 未读取或展示会话正文、工具输出、命令输出、输入历史或记忆正文。
- 未运行真实 harness。
- 未实际执行 Finder 打开、Finder 定位、`open`、`pbcopy`。
- 未读取系统剪贴板。
- 未拉取外网依赖。
- 未做个人知识库、多 agent、向量搜索、模型调度、复杂画布编排。
- 未做 release 打包、签名、自动更新、系统托盘、通知或登录项。

## 进程和端口清理

本轮离线测试本身没有启动 Tauri、Vite 或 dev server。

验证过程中发现进入本轮前已有同一产品壳相关残留：

- `cargo-tauri dev`
- `vite --host 127.0.0.1`
- `codex-governance-workbench`
- `127.0.0.1:5173` 监听

已按 PID 定向清理。

最终复核：

- `lsof -nP -iTCP:5173 -sTCP:LISTEN` 无输出。
- `ps` 只剩本次查询命令和 `rg` 自身，没有 `codex-governance-workbench`、`vite --host 127.0.0.1`、`cargo-tauri dev` 残留。

## 风险和下一步

风险：

- 当前测试是组件级离线交互测试，不覆盖 Tauri WebView 的真实辅助功能点击。
- 当前测试不点击 `确认执行`，因此不验证后端命令调用链；后端白名单仍由 Rust 单测覆盖。

下一步建议：

- 如果后续要补完整桌面端 UI 自动化，需要另建稳定 Tauri UI 自动化入口。
- 当前任务目标是补权限弹层前端交互证据，这轮已经覆盖。
