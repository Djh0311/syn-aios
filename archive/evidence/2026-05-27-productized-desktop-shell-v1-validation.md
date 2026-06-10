# 产品化桌面壳一期 UI 行为验证 evidence

任务包：`product-line/tasks/2026-05-27-productized-desktop-shell-v1-validation.md`

验证时间：2026-05-28

## 结论

本轮完成了产品化桌面壳一期验证线回收，但权限弹窗的“真实 WebView 点击后可见文本”没有稳定拿到自动化证据，不能说 UI 点击链条完全通过。

依据：

- `npm run typecheck` 通过。
- `npm run build` 通过。
- `cargo test --offline` 通过，Rust 单测 3 个通过。
- Tauri dev 窗口可以启动，窗口标题为 `Codex 治理工作台`，窗口尺寸为 `1280, 820`。
- 窗口正文读取到静态索引摘要，显示项目、会话、Skills、Plugins、Warning 等计数。
- 6 个页面通过 macOS System Events 验证可以切换。
- 项目页按钮、会话页动作按钮的源码链路会先设置 `pendingAction`，再渲染 `PermissionDialog`。
- 后端命令会重新读取索引并做白名单检查。
- 验证结束后清理了本轮启动的 Tauri / Vite / cargo-tauri 进程，`5173` 无监听残留。

## 执行命令

在 `product-line/prototypes/productized-desktop-shell/` 执行：

```bash
npm run typecheck
npm run build
```

结果：

- `typecheck`：通过。
- `build`：通过，产物写入 `dist/`。

在 `product-line/prototypes/productized-desktop-shell/src-tauri/` 执行：

```bash
CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline
```

结果：

- `path_whitelist_accepts_only_index_projects_and_rollouts`：通过。
- `snapshot_keeps_metadata_without_session_body`：通过。
- `reads_real_static_index_summary`：通过。

启动 Tauri dev：

```bash
CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target npm run tauri:dev
```

结果：

- Vite 监听 `http://127.0.0.1:5173/`。
- Tauri 运行 `codex-governance-workbench`。
- 窗口标题：`Codex 治理工作台`。

## UI 验证

### 窗口和索引摘要

System Events 读取窗口内容，确认窗口正文显示：

- 状态：`已读取索引。所有本机动作仍需用户点击并确认。`
- 应用名：`CodexGovernanceWorkbench / Codex 治理工作台`
- 摘要计数：项目 30、会话 296、Skills 50、Plugins 11、Warning 68。
- 范围说明：当前只治理 Codex，不做知识库、多 agent、向量搜索或模型调度。

依据：

- Tauri 窗口可由 System Events 读取。
- `src/App.tsx` 启动后调用 `loadWorkbenchSnapshot()`，成功后设置状态文案。

### 6 个页面切换

已验证页面：

- 首页：出现 `首页总览`。
- 项目：出现 `项目页`。
- 会话：出现 `会话页`。
- Skills / Plugins：出现 `Skills / Plugins 页`。
- 任务线 / 证据：出现 `任务线 / evidence / handoff 页`。
- 诊断：出现 `诊断页`。

依据：

- System Events 点击侧边栏按钮后读取窗口文本。
- `src/App.tsx` 的 `navItems` 只包含上述 6 项。
- `renderActiveView()` 对应渲染 6 个视图。

## 本机动作确认弹层

这一项结论是“源码链路通过，真实 WebView 自动点击证据不稳定”。

已确认的源码链路：

- `src/views/ProjectsView.tsx`：
  - `打开目录` 设置 `kind: "open-project"`、`label: "打开项目目录"`、`source: "索引内项目路径"`。
  - `复制路径` 设置 `kind: "copy"`、`label: "复制项目路径"`、`source: "索引内项目路径"`。
- `src/views/SessionsView.tsx`：
  - `定位` 设置 `kind: "reveal-rollout"`、`label: "定位 rollout 文件"`、`source: "索引内 rollout 路径"`。
  - `复制` 设置 `kind: "copy"`、`label: "复制 rollout 路径"`、`source: "索引内 rollout 路径"`。
- `src/App.tsx`：
  - 本机动作按钮只调用 `setPendingAction`。
  - `confirmAction()` 只有在 `PermissionDialog` 的确认按钮触发后才调用 `runPathAction()`。
- `src/components/PermissionDialog.tsx`：
  - 弹层显示 `本机动作确认`。
  - 显示动作 label。
  - 显示 `目标路径`。
  - 显示 `路径来源`。
  - 有 `取消` 和 `确认执行`。

自动化弱点：

- System Events 可以读取项目页按钮，并能看到多个 `打开目录`、`复制路径`。
- 直接按辅助功能层级点击第一个 `打开目录` 失败，报错为无法取得该 button。
- 后续窗口仍存在，但 WebView 子节点读取偶发失败。
- 因此本轮没有拿到“点击后弹层文本实际出现在窗口树中”的稳定证据。

补充尝试：

- 尝试用 Playwright 在浏览器侧做前端兜底验证。
- Playwright wrapper 需要拉取 `@playwright/cli`，当前网络解析失败：`ENOTFOUND registry.npmmirror.com`。
- 因此未形成浏览器侧点击截图或 snapshot 证据。

## 后端白名单

后端白名单验证通过。

依据：

- `src-tauri/src/lib.rs` 中：
  - `copy_indexed_path()` 调用 `allowed.can_copy()`，非项目路径或 rollout 路径会拒绝。
  - `open_indexed_project()` 只允许 `allowed.projects`。
  - `reveal_indexed_rollout()` 只允许 `allowed.rollouts`。
  - `allowed_paths()` 只从索引里的 `project_root` 和 `rollout_path` 构造白名单。
- Rust 单测 `path_whitelist_accepts_only_index_projects_and_rollouts` 通过，包含非白名单授权文件路径样本的拒绝断言。

没有实际打开、复制或定位敏感路径。

## 敏感内容边界

执行了源码扫描：

```bash
rg -n 'auth\.json|\.env|secret|token|authorization|first_user_message|preview|payload\.content|stdout|stderr|raw_memories|MEMORY\.md|writeFile|child_process|exec\(|spawn\(' product-line/prototypes/productized-desktop-shell/src product-line/prototypes/productized-desktop-shell/src-tauri/src product-line/prototypes/productized-desktop-shell/src-tauri/tauri.conf.json
```

结果：

- 命中 `spawn()`：用于 macOS `pbcopy`。
- 命中 `auth.json`：只在 Rust 单测中作为“非白名单必须拒绝”的字符串样本。
- 未发现前端或后端读取 `.env`、密钥、令牌、授权文件内容、会话正文、工具输出、命令输出、输入历史、记忆正文的代码路径。

## 禁止事项核对

本轮未做：

- 未写 `/Users/yoyi/.codex`。
- 未修改真实 Codex 状态库。
- 未读取或展示授权文件内容、密钥、令牌、`.env` 内容。
- 未读取或展示会话正文、工具输出、命令输出、输入历史或记忆正文。
- 未运行 harness。
- 未做个人知识库、多 agent、向量搜索、模型调度、复杂画布编排。
- 未做 release 打包、签名、自动更新、系统托盘、通知或登录项。
- 未接受任意用户输入路径执行本机动作。
- 未读取系统剪贴板内容。

## 清理结果

清理前发现本轮相关进程：

- `cargo-tauri dev`
- `vite --host 127.0.0.1`
- `codex-governance-workbench`

已执行定向 `kill` 清理上述 PID。

清理后验证：

- System Events 返回 `codex-governance-workbench` 不存在。
- `lsof -nP -iTCP:5173 -sTCP:LISTEN` 无输出。

结论：本轮启动的 dev server 和 Tauri 进程已清理，`5173` 没有监听残留。

## 风险和下一步

风险：

- 权限弹窗真实点击链条缺少稳定 UI 自动化证据。源码链路能证明“设计上必须先弹窗”，但还不能替代端到端点击证据。
- macOS System Events 对 Tauri WebView 内部按钮不稳定，可能需要给页面增加测试专用标识或用可离线可用的浏览器测试工具。
- Playwright CLI 当前依赖网络拉包失败，不能作为本轮证据来源。

建议：

- 后续补一个不依赖外网的前端交互测试，直接覆盖按钮点击后 `PermissionDialog` 的文本。
- 如果要做真实 Tauri UI 自动化，建议引入稳定的测试入口，而不是依赖 System Events 的 WebView 子节点路径。
- 本轮结果只能回收为“一期验证完成，带弹窗端到端点击证据缺口”，不能包装成完整桌面发布版。
