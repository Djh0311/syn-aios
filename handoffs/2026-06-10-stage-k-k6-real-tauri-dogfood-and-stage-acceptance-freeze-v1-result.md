# Stage K / K6 Real Tauri Dogfood And Stage Acceptance Freeze v1 Handoff

日期：2026-06-10

结论：`blocked_by_tauri_webview_blank_window`

K6 已执行但未通过。真实 Tauri dev / release 窗口均能启动，窗口标题、窗口 ID 和窗口 bounds 可识别，窗口级截图可生成；但内容区持续白屏，HTML 加载兜底也未显示。不能把本轮接受为 K6 完成或 Stage K 完成。

## 本轮做了什么

- 核对 K6 任务包和当前权威入口。
- 运行 K6 静态验证和 Stage K architecture gate。
- 启动真实 Tauri dev 桌面壳并采集窗口级截图。
- 修补 `index.html`：移除远程 Google Fonts 阻塞点，加入中文 HTML 加载兜底。
- 对修补后再次采集真实 Tauri dev 截图。
- 构建 release no-bundle 并启动 release binary 做第二路径探针。
- 停止本轮 dev / release Tauri 进程，确认 `5173` 端口释放。

## 改动文件

- `prototypes/productized-desktop-shell/index.html`

改动说明：

- 移除外部 Google Fonts 请求，桌面壳不再依赖远程字体样式。
- 在 `#root` 增加轻量加载兜底 `正在加载工作台...`，用于提升用户可见性和后续故障定位。

## 新增证据

- `evidence/2026-06-10-stage-k-k6-real-tauri-dogfood-and-stage-acceptance-freeze-v1.md`
- `evidence/tauri-verification/2026-06-10-stage-k-k6/01-home.png`
- `evidence/tauri-verification/2026-06-10-stage-k-k6/01-home-retry-after-wait.png`
- `evidence/tauri-verification/2026-06-10-stage-k-k6/01-home-after-font-fix.png`
- `evidence/tauri-verification/2026-06-10-stage-k-k6/01-home-after-html-fallback.png`
- `evidence/tauri-verification/2026-06-10-stage-k-k6/01-home-after-refresh.png`
- `evidence/tauri-verification/2026-06-10-stage-k-k6/01-home-release.png`

## 验证结果

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，14 项。
- `npm run build`：通过，仅既有 Vite chunk-size warning。
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
- 未把普通浏览器 smoke 当真实 Tauri 验收。

## 环境残留

本轮启动的 dev / release Tauri 已停止，`5173` 端口已释放。只读进程复核显示一个本轮开始前已存在的旧进程仍在运行：

```text
49082 target/debug/codex-governance-workbench
```

主管线没有将该预存进程作为本轮启动进程直接 kill。

## 下一步建议

进入 K6.1：Tauri WebView 内容渲染 / 截图链路修复。

K6.1 的目标应是先判断白屏到底来自：

- WebView 未加载 HTML / dist / devUrl。
- WebView 加载了页面但 JS / CSS 没渲染。
- 页面实际可见但 macOS `screencapture -l` 对 WebView 内容截为空白。

K6.1 通过前，K6 不得收口为完成，Stage K 不得冻结为完成。
