# 产品化桌面壳一期 UI 行为验证总指导回收意见

## 回收对象

- 任务包：`product-line/tasks/2026-05-27-productized-desktop-shell-v1-validation.md`
- 开发线：验证线
- 验证线 evidence：`product-line/evidence/2026-05-27-productized-desktop-shell-v1-validation.md`
- 验证线 handoff：`product-line/handoffs/2026-05-27-productized-desktop-shell-v1-validation-result.md`
- 被验证产物：`product-line/prototypes/productized-desktop-shell/`

## 结论

接受为“产品化桌面壳一期 UI 行为验证结果”。

不接受为“完整端到端 UI 自动化通过”，也不接受为“完整桌面发布版”。

依据：

- 验证线 evidence 记录了 `npm run typecheck`、`npm run build`、`cargo test --offline` 均通过。
- 验证线 evidence 记录 Tauri 窗口启动、读取索引摘要、6 个页面切换已验证。
- 验证线 evidence 明确说明权限弹窗没有拿到稳定的 Tauri WebView 端到端点击证据。
- 总指导线本轮复跑 `npm run typecheck`、`npm run build`、`cargo test --offline` 仍通过。
- 总指导线本轮复核 `lsof -nP -iTCP:5173 -sTCP:LISTEN` 无监听输出。

## 先说薄弱点

- 权限弹窗端到端点击证据不完整。依据：验证线写明 macOS System Events 对 Tauri WebView 内部按钮点击不稳定，没有稳定抓到弹窗实际文本。
- 本轮不能证明 Finder 打开目录、Finder 定位文件和剪贴板实际内容。依据：任务包将这些列为非必须独立核验项，验证线也没有执行确认动作或读取剪贴板。
- Playwright 兜底验证未完成。依据：验证线记录当前环境无法解析 npm registry，不能拉取 `@playwright/cli`。
- 总指导线本轮没有重新启动 GUI。依据：为避免重复打开和清理窗口，本轮只做轻量命令复跑与证据回收。

## 已接受的验证内容

- 基础命令验证通过：
  - `npm run typecheck`
  - `npm run build`
  - `cargo test --offline`
- Tauri dev 窗口可启动，窗口标题为 `Codex 治理工作台`。
- 窗口正文能读取静态索引摘要。
- 页面能显示项目、会话、Skills、Plugins、Warning 等计数。
- 6 个页面可以切换：
  - 首页
  - 项目
  - 会话
  - Skills / Plugins
  - 任务线 / 证据
  - 诊断
- 本机动作确认链路有源码依据：
  - 项目页按钮先设置 `pendingAction`。
  - 会话页按钮先设置 `pendingAction`。
  - `PermissionDialog` 显示动作、目标路径、路径来源。
  - `confirmAction()` 触发后才调用后端动作。
- 后端白名单有测试依据：
  - 项目打开只允许索引内项目路径。
  - rollout 定位只允许索引内 rollout 路径。
  - 复制只允许索引内项目路径或 rollout 路径。
  - 非白名单授权文件路径样本被拒绝。
- 验证线清理了本轮 Tauri / Vite / cargo-tauri 进程，`5173` 无监听残留。

## 总指导线复跑验证

在 `product-line/prototypes/productized-desktop-shell/`：

```bash
npm run typecheck
npm run build
```

结果：

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

接受当前验证线安全边界。

依据：

- 验证线记录未写 `/Users/yoyi/.codex`。
- 验证线记录未改真实 Codex 状态库。
- 验证线记录未读取或展示授权文件内容、密钥、令牌、`.env` 内容。
- 验证线记录未读取或展示会话正文、工具输出、命令输出、输入历史或记忆正文。
- 验证线记录未运行 harness。
- 验证线记录未做个人知识库、多 agent、向量搜索、模型调度、复杂画布编排。
- 验证线记录未做 release 打包、签名、自动更新、系统托盘、通知或登录项。
- 验证线记录未读取系统剪贴板内容。

## 当前接收范围

接收：

- 产品化桌面壳一期 UI 行为验证结果。
- Tauri 窗口读取索引摘要的验证结论。
- 6 页切换验证结论。
- 本机动作确认链路的源码级验证结论。
- 后端白名单拒绝非白名单路径的测试结论。
- 验证后进程清理和 5173 无监听结论。

不接收：

- 权限弹窗端到端点击完全通过。
- Finder 打开/定位完整验证。
- 剪贴板内容完整验证。
- 完整桌面发布版。
- release 打包、签名、图标、自动更新。

## 下一步建议

下一步建议只补一个小口：离线前端交互测试。

目标是验证点击 `打开目录`、`复制路径`、`定位 rollout` 后，`PermissionDialog` 出现并显示动作、目标路径、路径来源。

这个任务仍应归验证线，不新增常设线；不进入个人知识库、多 agent、向量搜索、模型调度或 release 范围。
