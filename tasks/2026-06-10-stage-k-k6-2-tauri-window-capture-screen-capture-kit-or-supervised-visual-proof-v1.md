# Stage K / K6.2 Tauri Window Capture ScreenCaptureKit Or Supervised Visual Proof v1

日期：2026-06-10

状态：已完成。

完成结论：`accepted_window_capture_proof_restored_with_deferred_navigation_screenshots`。K6.2 已通过 ScreenCaptureKit window-only capture 恢复真实 Tauri 可见窗口截图链路，成功产出 `evidence/tauri-verification/2026-06-10-stage-k-k6-2/01-home-screencapturekit-window.png`。该截图可见真实 Tauri 首页 UI，不再是 K6.1 的白屏截图。K6.2 不接受为 K6 全量 dogfood 完成、Stage K 完成、首页以外导航截图完成、K3-B1 retry 成功或 K3-B2 可开始。

记录见：

- `../evidence/2026-06-10-stage-k-k6-2-tauri-window-capture-screen-capture-kit-or-supervised-visual-proof-v1.md`
- `../handoffs/2026-06-10-stage-k-k6-2-tauri-window-capture-screen-capture-kit-or-supervised-visual-proof-v1-result.md`

本任务包承接 K6.1 阻断结论 `blocked_by_window_capture_webview_layer_after_app_mount`。目标不是继续修改 UI，而是恢复真实 Tauri 可视验收证据链：优先探索 ScreenCaptureKit 的 window-only 捕获；如果不可行，则冻结人工可视回证方案；如果两者都不可行，则继续阻断 K6 / Stage K。

本文不授权真实 `codex exec` / `codex exec resume`，不发送 prompt，不读写 `/Users/yoyi/.codex`，不读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout，不启动 K3-B1 retry，不启动 K3-B2，不实现真实 retry / stop / restart / resume。

## 1. 当前事实

- K6 已执行但未通过，阻断为 `blocked_by_tauri_webview_blank_window`。
- K6.1 已进一步分类：真实 Tauri dev 中前端入口脚本已运行，React App 首屏已挂载。
- window-only `screencapture -l` 仍只能捕获标题栏和白色内容区。
- 精确区域截图被系统拒绝：`could not create image from rect`。
- CoreGraphics `CGWindowListCreateImage` 在当前 macOS SDK 不可用，提示改用 ScreenCaptureKit。
- 不能使用全屏截图。

## 2. 目标

K6.2 本轮交付：

1. 判断 ScreenCaptureKit 是否能以 window-only 方式捕获目标 Tauri 窗口内容。
2. 如可行，产出至少一张真实 Tauri 首页可见 UI 截图。
3. 如不可行，冻结人工可视回证协议，明确它能证明什么、不能证明什么。
4. 更新 K6.2 evidence / handoff，并决定 K6 是否可恢复继续。

## 3. 允许范围

- 只读读取 K6 / K6.1 evidence、handoff、任务包和当前入口。
- 读取必要的 Apple / macOS ScreenCaptureKit 本地接口文档或命令帮助；如需联网查官方文档，必须只引用 Apple 官方文档。
- 编写最小临时或仓库内 harness，用于按 window id / bundle / PID 捕获目标 Tauri 窗口。
- 启动真实 Tauri dev / release 进行 window-only 可视验收。
- 只捕获目标 Tauri 窗口，不捕获全屏。
- 记录窗口 title、PID、window id、bounds、时间戳、命令和输出。
- 运行 `npm run typecheck`、`npm run test:offline-interaction`、`npm run build`、Stage K architecture gate strict、必要的 Tauri no-bundle build。

## 4. 禁止范围

- 不执行真实 Codex。
- 不发送 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取完整 transcript / rollout / secret / token / `.env`。
- 不用全屏截图捕获用户桌面。
- 不把普通浏览器 / Vite DOM smoke 当真实 Tauri 验收。
- 不删除或 kill 非本轮启动的用户进程，除非用户另行明确授权。
- 不改 Product Command / workflow state / memory sidecar schema。
- 不启动 K3-B1 retry 或 K3-B2。

## 5. 建议执行顺序

1. 启动真实 Tauri dev，确认 DEV-only 标题探针仍能进入 `首屏已挂载`。
2. 用窗口枚举定位本轮 Tauri PID / window id / bounds，并确认旧进程不处理。
3. 尝试 ScreenCaptureKit window-only 捕获。
4. 如果 ScreenCaptureKit 需要系统权限或实现成本超出本轮，停止并记录为权限 / 工具阻断，不绕路全屏截图。
5. 若截图成功，采集首页 / 智能体 / 运行中工作流最小 3 张恢复截图，再决定是否回到 K6 完整清单。
6. 若截图失败，写人工可视回证协议或继续阻断，不声明 K6 完成。

## 6. 验收标准

可接受为完成：

- 至少一张真实 Tauri window-only 截图显示工作台可见 UI；或
- 形成明确、可复核的人工可视回证协议，并明确不等同自动截图；或
- 形成明确的 ScreenCaptureKit / 系统权限阻断证据。

不接受为完成：

- 只有普通浏览器截图。
- 继续只有白屏截图但声明 K6 可以继续。
- 使用全屏截图。
- 通过执行真实 Codex 或读写 `.codex` 来证明 UI。
- 修改 UI 产品层来掩盖截图工具问题。

## 7. 回收摘要

K6.2 实际完成路径：

1. 新增最小 ScreenCaptureKit window-only capture harness：`scripts/harness/stage-k-screencapturekit-window-capture.swift`。
2. 新增后续导航辅助 click harness：`scripts/harness/stage-k-cgevent-click.swift`。
3. 启动真实 Tauri dev，并枚举到目标窗口 `window_id=28761` / `pid=85499` / `title="Codex 治理工作台 · 首屏已挂载"` / `frame={x:95,y:44,w:1280,h:820}`。
4. 使用 ScreenCaptureKit 按 window id 捕获目标窗口内容。
5. 产出真实 Tauri 首页截图：`evidence/tauri-verification/2026-06-10-stage-k-k6-2/01-home-screencapturekit-window.png`。
6. 文件核验：1280 x 820 PNG，sha256 `17a0cc17b0ee274dd31aa1e2a6553e9ef044845f2f9f96f11174a176ca9ab2c4`。
7. 目视核验：截图不是白屏，可见首页、项目、智能体、运行中工作流、Skill、Harness 和右侧秘书栏。
8. Fresh verify：Stage K architecture gate strict 通过，`npm run typecheck` 通过，`npm run test:offline-interaction` 通过 14 项，`npm run build` 通过且仅保留既有 Vite chunk-size warning，两个 Swift harness 编译通过。

仍然 deferred：

- 未补齐首页以外的导航截图。
- 未完成 K6 原始截图清单。
- 未完成 K6 全量真实 Tauri dogfood。
- 未完成 Stage K 最终冻结。

下一步应回到 K6 主任务，用 ScreenCaptureKit window-only harness 补齐核心路径截图，不再把旧 `screencapture -l` 白屏链路作为唯一截图方式。
