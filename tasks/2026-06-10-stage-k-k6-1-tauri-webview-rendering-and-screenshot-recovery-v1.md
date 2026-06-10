# Stage K / K6.1 Tauri WebView Rendering And Screenshot Recovery v1

日期：2026-06-10

状态：已执行但未通过，结论为 `blocked_by_window_capture_webview_layer_after_app_mount`。

本任务包承接 K6 阻断结论 `blocked_by_tauri_webview_blank_window`。目标是修复或明确分类真实 Tauri 窗口内容区白屏 / 截图空白问题，恢复 K6 真实桌面 dogfood 的前置条件。

本文不授权真实 `codex exec` / `codex exec resume`，不发送 prompt，不读写 `/Users/yoyi/.codex`，不读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout，不启动 K3-B1 retry，不启动 K3-B2，不实现真实 retry / stop / restart / resume。

## 1. 当前事实

- K6 已执行但未通过，记录见 `evidence/2026-06-10-stage-k-k6-real-tauri-dogfood-and-stage-acceptance-freeze-v1.md` 与 `handoffs/2026-06-10-stage-k-k6-real-tauri-dogfood-and-stage-acceptance-freeze-v1-result.md`。
- Tauri dev / release 窗口均可启动。
- CoreGraphics 可识别窗口标题、PID、window id 和 bounds。
- `screencapture -l` 可生成窗口级 PNG。
- PNG 只显示标题栏和白色内容区。
- `index.html` 已移除外部 Google Fonts，并加入 HTML 加载兜底，但白屏仍存在。
- `npm run typecheck`、`npm run test:offline-interaction`、`npm run build` 和 Stage K architecture gate strict 均通过。

## 2. 目标

K6.1 本轮交付：

1. 区分白屏根因类别：WebView 未加载页面、页面加载但 JS/CSS 未渲染、截图工具无法捕获 WebView 内容。
2. 如果是产品代码 / Tauri 配置问题，做最小修补。
3. 如果是截图工具限制，形成可复现的替代验收方案，但不得把普通浏览器 smoke 冒充真实 Tauri。
4. 恢复至少一张真实 Tauri 首页可见 UI 截图，或形成明确的不可完成证据。
5. 更新 K6.1 evidence / handoff，并决定是否可回到 K6 继续截图清单。

## 3. 允许范围

- 读取和修改 `prototypes/productized-desktop-shell/index.html`、`src/main.tsx`、`src/App.tsx`、`src/styles.css`、`src-tauri/tauri.conf.json` 以及必要的 Tauri 启动 / 日志配置。
- 增加最小的桌面壳启动状态 / 加载失败提示。
- 增加只在开发 / 验收使用的前端错误边界或 console-to-visible diagnostics，但普通产品层不得铺 raw log。
- 运行 `npm run typecheck`、`npm run test:offline-interaction`、`npm run build`。
- 运行 Stage K architecture gate strict。
- 启动真实 Tauri dev / release 进行 UI dogfood。
- 使用 window-only `screencapture -l <window_id>` 截取目标 Tauri 窗口。
- 使用只读窗口元数据枚举定位目标窗口。

## 4. 禁止范围

- 不执行真实 Codex。
- 不发送 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取完整 transcript / rollout / secret / token / `.env`。
- 不用全屏截图捕获无关用户数据。
- 不把普通浏览器 / Vite DOM smoke 当真实 Tauri 验收。
- 不删除或 kill 非本轮启动的用户进程，除非用户另行明确授权。
- 不改 Product Command / workflow state / memory sidecar schema。
- 不启动 K3-B1 retry 或 K3-B2。

## 5. 建议排查顺序

1. 保留 `index.html` HTML 级兜底，确认 dev / release 是否能显示该兜底。
2. 在不泄漏敏感信息的前提下，为 Tauri 启动路径增加最小可见错误边界。
3. 对比 devUrl 和 production dist 两条路径。
4. 检查 Tauri config 的 `devUrl`、`frontendDist`、CSP、asset protocol 和 window ready 状态。
5. 如果页面实际可见但截图白屏，记录截图工具限制，并尝试更安全的 window-only 替代方案；不得使用全屏截图。
6. 修补后重新跑静态验证和真实 Tauri 截图。

## 6. 验收标准

可接受为完成：

- 至少一张真实 Tauri 窗口截图能显示工作台首页或明确错误兜底，不再只是白色内容区。
- 能明确解释 dev / release 路径的差异或共同根因。
- 修补不引入真实 Codex 执行、不读写 `.codex`、不改 workflow / memory schema。
- 静态验证通过。
- evidence / handoff 明确说明 K6 是否可以恢复继续。

不接受为完成：

- 只用普通浏览器证明 UI 可见。
- 继续只有白屏截图但声明 K6 可以继续。
- 用全屏截图或无边界截图捕获用户桌面。
- 通过执行真实 Codex 或读写 `.codex` 来证明 UI。
