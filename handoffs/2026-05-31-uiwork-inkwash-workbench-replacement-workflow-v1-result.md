# Handoff: uiwork 水墨工作台界面替换工作流 v1

## 薄弱点

- 浏览器只能验证外壳和 DOM，不能验证 Tauri 后端数据页，因为普通浏览器会显示 Tauri 读取失败空态。
- 截图证据不足：浏览器截图接口超时，整屏截图被系统拒绝。
- workflow state 里有一条 uiwork 中间 run 仍是 `running`，需要后续清账；最终有效 run 已 `accepted`。

## 已完成

- uiwork 四个 Codex 会话已绑定到工作流。
- 工作流机器已支持通用目标和 `execution_root`。
- 已通过真实四角色工作流把水墨 UI 接入 `productized-desktop-shell`。
- 第一轮回收为 `needs_changes`，第二轮有效 run 回收为 `accepted`。

## 关键对象

- workflow：`workflow:users-yoyi-documents-uiwork:default`
- work item：`workflow:users-yoyi-documents-uiwork:default:inkwash-ui-replacement-v1`
- accepted run：`workflow-machine-run:workflow-users-yoyi-documents-uiwork-default:workflow-users-yoyi-documents-uiwork-default-inkwash-ui-replacement-v1:1780225012481`
- source HTML：`/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/ui-prototype/inkwash-full.html`
- target：`/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell`

## 写入情况

- 执行真实 `codex exec resume`：是。
- 写 `/Users/yoyi/.codex`：是。
- 写真实 workflow state：是。
- 修改目标工程：是。
- 读取敏感文件或完整 transcript：否。

## 验证结果

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，4 个离线交互测试。
- `npm run build`：通过。
- `cargo test --offline`：通过，69 passed，1 ignored。
- 索引校验：`validation_ok`。

## 改动文件

- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/App.tsx`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/styles.css`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/views/HomeView.tsx`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/dist/index.html`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/dist/assets/index-DjgbW7hh.css`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/dist/assets/index-mDo39SmI.js`
- `/Users/yoyi/workspace/product-line/tasks/2026-05-31-inkwash-ui-workbench-replacement-workflow-v1.md`
- `/Users/yoyi/workspace/product-line/tasks/README.md`
- `/Users/yoyi/workspace/product-line/CURRENT.md`

## 当前判断

可以回收为“uiwork 工作流替换工作台 UI 已完成第一版”。不能回收为“截图级原模原样已证明”或“复杂自动化已完成”。

下一步建议：继续用 uiwork 工作流做视觉补差和 Tauri 窗口验收，不要手工直接改 UI。
