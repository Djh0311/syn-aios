# 产品化桌面壳一期 UI 行为验证 handoff

任务包：`product-line/tasks/2026-05-27-productized-desktop-shell-v1-validation.md`

回收时间：2026-05-28

## 回收结论

验证线任务已完成，但不是全绿发布结论。

薄弱点在权限弹窗：源码链路和后端白名单验证通过，Tauri 窗口里项目页按钮也能被辅助功能树看到；但 macOS System Events 点击 WebView 内部按钮不稳定，没有稳定抓到弹窗实际文本。因此只能说“本机动作确认链路有源码和测试依据”，不能说“端到端 UI 点击证据完整”。

## 已完成

1. 复跑基础验证：
   - `npm run typecheck`：通过。
   - `npm run build`：通过。
   - `cargo test --offline`：通过，3 个 Rust 单测通过。
2. 启动 Tauri dev：
   - Vite 成功监听 `127.0.0.1:5173`。
   - Tauri 应用进程为 `codex-governance-workbench`。
   - 窗口标题为 `Codex 治理工作台`。
3. 验证窗口读取静态索引：
   - 页面显示项目、会话、Skills、Plugins、Warning 摘要计数。
   - 页面显示“当前只治理 Codex”的边界说明。
4. 验证 6 个页面切换：
   - 首页。
   - 项目。
   - 会话。
   - Skills / Plugins。
   - 任务线 / 证据。
   - 诊断。
5. 验证本机动作安全链路：
   - 项目页 `打开目录` 和 `复制路径` 会设置待确认动作。
   - 会话页 `定位` 和 `复制` 会设置待确认动作。
   - `PermissionDialog` 显示动作、目标路径、路径来源、取消、确认执行。
   - 后端再次执行索引白名单检查。
6. 扫描敏感内容边界：
   - 未发现读取 `.env`、密钥、令牌、授权文件内容、会话正文、工具输出、命令输出、输入历史、记忆正文的代码路径。
7. 清理进程：
   - 已清理本轮 `cargo-tauri dev`、Vite、Tauri 应用进程。
   - 复核 `5173` 无监听残留。

## 新增文件

- `product-line/evidence/2026-05-27-productized-desktop-shell-v1-validation.md`
- `product-line/handoffs/2026-05-27-productized-desktop-shell-v1-validation-result.md`

## 依据文件

- `product-line/prototypes/productized-desktop-shell/src/App.tsx`
- `product-line/prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `product-line/prototypes/productized-desktop-shell/src/views/SessionsView.tsx`
- `product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`

## 未完成或不确定

- 权限弹窗没有拿到稳定的 Tauri WebView 端到端点击证据。
- Finder 打开、Finder 定位和剪贴板实际内容不是本任务必须项，本轮没有执行确认动作，也没有读取剪贴板内容。
- Playwright 兜底验证未跑成，原因是当前环境无法解析 npm registry，不能拉取 `@playwright/cli`。

## 禁止事项状态

未触碰禁止事项：

- 没写 `/Users/yoyi/.codex`。
- 没改真实 Codex 状态库。
- 没读取或展示授权文件内容、密钥、令牌、`.env` 内容。
- 没读取或展示会话正文、工具输出、命令输出、输入历史、记忆正文。
- 没运行 harness。
- 没做个人知识库、多 agent、向量搜索、模型调度、复杂画布编排。
- 没做 release 打包、签名、自动更新、系统托盘、通知或登录项。
- 没接受任意用户输入路径执行本机动作。
- 没读取系统剪贴板内容。

## 回收建议

可以回收为：产品化桌面壳一期 UI 行为验证已完成，带一个明确缺口：权限弹窗端到端点击证据不足。

不建议回收为：完整桌面发布版、完整 Finder/剪贴板验证、完整端到端 UI 自动化通过。

下一步建议只补一件事：增加不依赖外网的前端交互测试，验证点击 `打开目录`、`复制路径`、`定位 rollout` 后弹出 `PermissionDialog` 并显示动作、路径、来源。
