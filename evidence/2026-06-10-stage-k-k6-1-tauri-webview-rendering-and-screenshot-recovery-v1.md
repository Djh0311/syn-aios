# Stage K / K6.1 Tauri WebView Rendering And Screenshot Recovery v1 Evidence

日期：2026-06-10

结论：`blocked_by_window_capture_webview_layer_after_app_mount`

K6.1 已按任务包执行，但不能接受为 K6 恢复或 Stage K 完成。本轮把 K6 的 `blocked_by_tauri_webview_blank_window` 进一步分类：真实 Tauri dev 窗口可启动，前端入口脚本已运行，React App 首屏已挂载，说明不是单纯 WebView 未加载 HTML，也不是 React 完全未进入首屏；但 `screencapture -l <window_id>` 仍只能捕获标题栏和白色内容区，无法捕获 WebView 内容。精确区域截图被系统拒绝，CoreGraphics 旧 window image API 在当前 macOS SDK 已不可用并提示改用 ScreenCaptureKit。因此 K6 仍不能恢复继续，下一步应进入 K6.2：ScreenCaptureKit 窗口级截图或明确的人工可视回证方案。

本轮没有执行真实 `codex exec` / `codex exec resume`，没有发送 prompt，没有读写 `/Users/yoyi/.codex`，没有读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout，没有启动 K3-B1 retry，没有启动 K3-B2，没有实现或触发真实 retry / stop / restart / resume。

## 执行依据

- 架构校准计划：`docs/plans/2026-06-10-stage-k-architecture-calibration-plan-v2.md`。
- K6.1 任务包：`tasks/2026-06-10-stage-k-k6-1-tauri-webview-rendering-and-screenshot-recovery-v1.md`。
- K6 阻断记录：`evidence/2026-06-10-stage-k-k6-real-tauri-dogfood-and-stage-acceptance-freeze-v1.md`。

## 本轮代码修补

- `prototypes/productized-desktop-shell/vite.config.ts`
  - 新增 `base: "./"`，让 release `dist/index.html` 使用相对 asset 路径，避免 Tauri 自定义协议下绝对 `/assets/...` 路径风险。
- `prototypes/productized-desktop-shell/src/main.tsx`
  - 新增最小前端启动错误边界。
  - 新增 DEV-only Tauri 标题探针：前端入口脚本运行时可把窗口标题改为 `前端已加载`。
  - 新增 DEV-only 可见启动探针：入口脚本运行时临时插入 `启动诊断：前端脚本已运行`，App 挂载后移除。
- `prototypes/productized-desktop-shell/src/App.tsx`
  - 新增 DEV-only App 挂载标题探针：React 首屏 commit 后把窗口标题改为 `首屏已挂载`。
  - App 挂载后移除入口可见启动探针。
- `prototypes/productized-desktop-shell/src/styles.css`
  - 新增启动错误兜底和 DEV-only 可见启动探针样式。
- `prototypes/productized-desktop-shell/src-tauri/capabilities/default.json`
  - 新增最小窗口标题权限 `core:window:allow-set-title`，仅用于 Tauri 启动诊断。
- `prototypes/productized-desktop-shell/src/vite-env.d.ts`
  - 新增 Vite `ImportMeta.env` 类型声明。

上述修补不改变 Product Command / workflow state / memory sidecar schema，不新增真实执行入口，不改变 Codex runner 行为。

## 关键诊断事实

- `npm run tauri:dev` 成功启动真实 Tauri dev。
- Vite 成功监听 `http://127.0.0.1:5173/`。
- 窗口枚举可识别本轮 Tauri 窗口：
  - PID：`73221`
  - window id：`28746`
  - 初始标题探针后标题：`Codex 治理工作台 · 前端已加载`
  - 完整 reload 后标题：`Codex 治理工作台 · 首屏已挂载`
- `首屏已挂载` 说明 React App 已经完成首屏 commit。
- 对 window id `28746` 执行 `screencapture -l` 成功生成 PNG，但内容区仍为白色。
- 精确区域截图 `screencapture -R 95,44,1280,820 ...` 失败，错误为 `could not create image from rect`。
- CoreGraphics `CGWindowListCreateImage` 方式失败，当前 macOS SDK 报错：该 API 在 macOS 15 不可用，应使用 ScreenCaptureKit。
- 本轮启动的 Tauri dev session 已停止，端口 `5173` 已释放；只剩开始前已存在的旧进程 `49082`，本轮未 kill。

## 截图证据

- `evidence/tauri-verification/2026-06-10-stage-k-k6-1/01-home-after-k6-1-fix.png`
- `evidence/tauri-verification/2026-06-10-stage-k-k6-1/03-home-after-app-mounted-title-probe.png`

说明：两张均为真实 Tauri window-only 截图，但内容区仍白；第二张标题栏显示 `首屏已挂载`，是本轮最关键的分类证据。

## 验证结果

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 14`。
- `npm run build`：通过，仅保留既有 Vite chunk-size warning。
- `node scripts/harness/stage-k-architecture-gate.js --target /Users/yoyi/workspace/product-line --strict`：通过，`Errors: 0` / `Warnings: 0`。
- `../tauri-capability-probe/.tauri-cli/bin/cargo-tauri build --no-bundle`：通过，release binary 产物生成；Rust release build 保留既有 dead-code / unused warnings。

## 分类结论

K6.1 排除了以下一部分可能性：

- 不是单纯 `dist` 绝对资源路径问题：已改为相对路径，静态 build 和 Tauri no-bundle build 通过。
- 不是前端入口脚本完全未运行：窗口标题已进入 `前端已加载`。
- 不是 React App 完全未挂载：完整 reload 后窗口标题已进入 `首屏已挂载`。

仍未解决的问题：

- window-only 截图仍无法捕获 WebView 内容。
- 当前安全边界下不能使用全屏截图。
- 精确区域截图不可用。
- CoreGraphics 旧窗口截图 API 在当前系统不可用。

因此本轮更准确的阻断分类是：

```text
blocked_by_window_capture_webview_layer_after_app_mount
```

## 本轮不能声明

- 不能声明 K6.1 完成。
- 不能声明 K6 可以恢复继续。
- 不能声明 Stage K 完成。
- 不能声明真实 Tauri UI dogfood 通过。
- 不能声明 K1-K5 普通 UI 在真实壳中已完成截图验收。
- 不能把标题探针当成 UI 截图验收。
- 不能把普通浏览器 smoke 当真实 Tauri 验收。
- 不能声明 K3-B1 retry 成功或 K3-B2 可以开始。

## 下一步

进入 K6.2：Tauri window capture recovery via ScreenCaptureKit or supervised visual proof。

K6.2 应优先选择其中一条可验收路径：

1. 实现或使用 ScreenCaptureKit 的 window-only capture，按 window id 捕获目标 Tauri 窗口内容。
2. 如果 ScreenCaptureKit 权限或实现成本不适合当前回合，则冻结人工可视回证流程：用户肉眼确认真实 Tauri 窗口内容，主管线记录窗口标题 / PID / window id / bounds / 时间戳 / 操作路径，并明确该证据不能等同自动截图。
3. 如仍无法取得可见 UI 截图或可信可视回证，则 K6 / Stage K 继续保持阻断。
