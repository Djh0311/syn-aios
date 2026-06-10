# Stage K / K6.1 Tauri WebView Rendering And Screenshot Recovery v1 Handoff

日期：2026-06-10

结论：`blocked_by_window_capture_webview_layer_after_app_mount`

K6.1 已执行但未通过，不能接受为 K6 恢复或 Stage K 完成。本轮确认真实 Tauri dev 窗口中前端入口脚本已运行，React App 首屏已挂载；但 window-only `screencapture -l` 仍只捕获标题栏和白色内容区。下一步应进入 K6.2，使用 ScreenCaptureKit window-only 捕获或冻结人工可视回证流程。

## 改动文件

- `prototypes/productized-desktop-shell/vite.config.ts`
- `prototypes/productized-desktop-shell/src/main.tsx`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/src-tauri/capabilities/default.json`
- `prototypes/productized-desktop-shell/src/vite-env.d.ts`

## 核心事实

- `dist/index.html` 已从绝对 `/assets/...` 改为相对 `./assets/...`。
- DEV-only 标题探针证明：
  - 前端入口脚本已运行：`Codex 治理工作台 · 前端已加载`。
  - React App 首屏已挂载：`Codex 治理工作台 · 首屏已挂载`。
- 真实 Tauri window-only 截图仍白：
  - `evidence/tauri-verification/2026-06-10-stage-k-k6-1/01-home-after-k6-1-fix.png`
  - `evidence/tauri-verification/2026-06-10-stage-k-k6-1/03-home-after-app-mounted-title-probe.png`
- 精确区域截图失败：`could not create image from rect`。
- CoreGraphics 旧窗口截图 API 在当前 macOS SDK 不可用，系统要求改用 ScreenCaptureKit。
- 本轮 Tauri dev session 已停止，`5173` 端口已释放。
- 开始前已存在的旧 Tauri 进程 `49082` 未处理、未 kill。

## 验证

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，14 项。
- `npm run build`：通过，仅既有 chunk-size warning。
- `node scripts/harness/stage-k-architecture-gate.js --target /Users/yoyi/workspace/product-line --strict`：通过，0 errors / 0 warnings。
- `../tauri-capability-probe/.tauri-cli/bin/cargo-tauri build --no-bundle`：通过，保留既有 Rust warnings。

## 边界确认

- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 未启动 K3-B1 retry。
- 未启动 K3-B2。
- 未实现或触发真实 retry / stop / restart / resume。
- 未使用全屏截图。
- 未 kill 非本轮启动的旧进程。

## 交接建议

K6.2 不应继续盲改 UI。现在已证明 App 首屏能挂载，真正缺的是可信的真实 Tauri 可视证据获取链路。建议 K6.2 只处理：

- ScreenCaptureKit window-only capture 可行性。
- 或人工可视回证协议。
- 或明确冻结 K6 / Stage K 为截图链路阻断。

K6.2 通过前，K6 不得恢复，Stage K 不得收口。
