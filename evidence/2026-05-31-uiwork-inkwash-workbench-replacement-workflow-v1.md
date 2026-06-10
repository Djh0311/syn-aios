# Evidence: uiwork 水墨工作台界面替换工作流 v1

## 薄弱点

- 这轮完成的是“用四角色工作流改工作台 UI”，不是复杂业务自动编排完成。
- 浏览器验收存在限制：普通 Vite 页面不在 Tauri 窗口里运行，无法读取 Tauri 后端数据，所以首页星图会被读取失败空态挡住。
- 截图没有成功落地：in-app browser 的截图接口两次超时；整屏 `screencapture` 被系统拒绝，原因是可能截到无关隐私。
- workflow state 中存在一个历史中间 run 仍显示 `running`：`workflow-machine-run:workflow-users-yoyi-documents-uiwork-default:workflow-users-yoyi-documents-uiwork-default-inkwash-ui-replacement-v1:1780224885195`。最终有效 run 已是 `accepted`，但这条旧 run 需要后续清账。

## 目标

用 `/Users/yoyi/Documents/uiwork` 下四个 Codex 会话，通过工作流机器把水墨 HTML 原型接入真实工作台：

- 源稿：`/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/ui-prototype/inkwash-full.html`
- 目标工程：`/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell`
- 任务包：`/Users/yoyi/workspace/product-line/tasks/2026-05-31-inkwash-ui-workbench-replacement-workflow-v1.md`

## 会话绑定

- ui总指导：`019e7d93-02e0-79b3-9619-0afeea0d1efb`
- ui开发线：`019e7d92-e405-7090-8b34-bc0865284584`
- ui验证线：`019e7d92-c7ae-7f21-a250-b79a21c15426`
- ui回收线：`019e7d92-ad1d-7910-8894-21574db1ebd4`

四条绑定均已进入 `codex-index.json`，`project_root=/Users/yoyi/Documents/uiwork`，`rollout_exists=true`。

## 本轮做了什么

- 刷新 `codex-index.json`，让 uiwork 四个会话进入索引。
- 修了工作流机器的硬编码问题：不再写死马里奥 demo prompt，支持通用目标。
- 给工作流机器增加 `execution_root`，允许 workflow root 是 `/Users/yoyi/Documents/uiwork`，但真实改动发生在 `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell`。
- 修复 rerun 问题：`needs_changes` 重新运行前会回到可派发状态。
- 注册真实 uiwork workflow state。
- 通过工作流机器跑了两轮 UI 替换。

## 工作流运行结果

第一轮：

- run id：`workflow-machine-run:workflow-users-yoyi-documents-uiwork-default:workflow-users-yoyi-documents-uiwork-default-inkwash-ui-replacement-v1:1780224330335`
- 结果：`needs_changes`
- 原因摘要：外壳已迁移，但中心 stage 没有达到源稿结构，缺少首页星图相关布局。

第二轮有效结果：

- run id：`workflow-machine-run:workflow-users-yoyi-documents-uiwork-default:workflow-users-yoyi-documents-uiwork-default-inkwash-ui-replacement-v1:1780225012481`
- 结果：`accepted`
- work item：`workflow:users-yoyi-documents-uiwork:default:inkwash-ui-replacement-v1`
- work item 最终状态：`accepted`
- 四个执行节点最终均为 `ready_for_review`

## 写入边界

- 是否执行真实 `codex exec resume`：是，由工作流机器调用四个 uiwork 会话。
- 是否写 `/Users/yoyi/.codex`：是，因为执行了真实 `codex exec resume`。
- 是否写真实 workflow state：是。
- 是否修改目标工程文件：是，由工作流执行。
- 是否读取 `auth.json`、`.env`、密钥、token、授权文件：否。
- 是否读取完整 transcript：否。
- 是否修改 `/Users/yoyi/Documents/mario test`：否。

## 改动文件

工作流引擎 / UI 启动相关：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`

由 uiwork 工作流替换 UI：

- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/src/views/HomeView.tsx`

构建产物：

- `prototypes/productized-desktop-shell/dist/index.html`
- `prototypes/productized-desktop-shell/dist/assets/index-DjgbW7hh.css`
- `prototypes/productized-desktop-shell/dist/assets/index-mDo39SmI.js`

任务索引：

- `tasks/2026-05-31-inkwash-ui-workbench-replacement-workflow-v1.md`
- `tasks/README.md`
- `CURRENT.md`

## 浏览器 / 桌面验收

普通浏览器 `http://127.0.0.1:5175/`：

- 页面非空。
- 顶栏、左栏、主舞台、右侧状态栏、底部 dock DOM 结构存在。
- 在 `1440x900` 临时视口下无横向溢出。
- 左侧导航按钮可切换 active 状态。
- 弱点：普通浏览器无法读取 Tauri 数据，主区域显示“读取失败：当前页面不在 Tauri 窗口中运行”，因此不能证明真实数据页的视觉最终形态。

Tauri dev：

- 已启动过 `npm run tauri:dev`。
- 进程存在时曾只读读到窗口标题 `Codex 治理工作台`。
- 后续系统可访问窗口计数变成 0，无法用 Computer Use 获取稳定可访问树。
- 结论：桌面壳启动验证有部分依据，但视觉截图证据不足。

## 验证命令

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 4`。
- `npm run build`：通过。
- `CARGO_HOME=/Users/yoyi/.cargo CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/target cargo test --offline`：通过，`69 passed; 1 ignored`。
- `/Users/yoyi/miniconda3/bin/python3 prototypes/index-kernel/build_index.py --check prototypes/index-kernel/codex-index.json`：通过，`validation_ok`。
- `rg -F 'codex exec resume' tasks evidence handoffs CURRENT.md`：固定字符串搜索完成；输出为历史记录命中，没有触发命令替换。

## 回收判断

接受为：

- uiwork 四角色工作流已真实执行并把目标工作台 UI 改成水墨壳方向。
- 通用工作流机器已支持非马里奥目标和跨 execution root 目标工程。
- 最终有效 run 收口为 `accepted`。
- 现有类型检查、离线交互、构建、Rust 测试和索引校验均通过。

不接受为：

- 复杂业务自动编排完成。
- “原模原样”已经被截图级证明。
- Tauri 真实窗口视觉验收完全闭环。

## 后续建议

- 单独清理 uiwork 历史中间 run 的 `running` 旧账。
- 做一次专门的 Tauri 窗口视觉验收任务，不用整屏截图，优先找只截目标窗口或应用内导出 DOM/截图的安全方式。
- 下一轮 UI 迭代继续走工作流，不手工绕过 uiwork。
