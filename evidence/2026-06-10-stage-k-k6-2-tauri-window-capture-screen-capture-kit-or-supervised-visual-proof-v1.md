# Stage K / K6.2 Tauri Window Capture ScreenCaptureKit Or Supervised Visual Proof v1 Evidence

日期：2026-06-10

结论：`accepted_window_capture_proof_restored_with_deferred_navigation_screenshots`

K6.2 已恢复可信的真实 Tauri window-only 可视证据链：使用仓库内最小 ScreenCaptureKit harness 按目标 Tauri window id 捕获窗口内容，成功产出一张非白屏真实 Tauri 首页截图。截图显示窗口标题为 `Codex 治理工作台 · 首屏已挂载`，内容区可见首页、五个主对象区块和右侧秘书栏；这证明 K6.1 的白屏问题主要是旧 `screencapture -l` / CoreGraphics 截图链路不能捕获 WebView 内容，而不是 Tauri App 首屏没有渲染。

本结论只接受为 K6.2 恢复可见窗口证明完成，不接受为 K6 全量 dogfood 完成、Stage K 完成、K1-K5 全部真实 Tauri 导航截图完成、K3-B1 retry 成功或 K3-B2 可开始。K6.2 后续可回到 K6 截图清单继续补首页以外的核心路径截图。

本轮没有执行真实 `codex exec` / `codex exec resume`，没有发送 prompt，没有读写 `/Users/yoyi/.codex`，没有读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout，没有启动 K3-B1 retry，没有启动 K3-B2，没有使用全屏截图，没有 kill 旧 Tauri 进程。

## 执行依据

- K6.2 任务包：`tasks/2026-06-10-stage-k-k6-2-tauri-window-capture-screen-capture-kit-or-supervised-visual-proof-v1.md`。
- K6.1 阻断记录：`evidence/2026-06-10-stage-k-k6-1-tauri-webview-rendering-and-screenshot-recovery-v1.md`。
- K6.1 交接：`handoffs/2026-06-10-stage-k-k6-1-tauri-webview-rendering-and-screenshot-recovery-v1-result.md`。
- Stage K 架构校准计划：`docs/plans/2026-06-10-stage-k-architecture-calibration-plan-v2.md`。

## 本轮新增 / 使用的 harness

- `scripts/harness/stage-k-screencapturekit-window-capture.swift`
  - 用 ScreenCaptureKit 枚举窗口。
  - 支持按 title 过滤窗口。
  - 支持按 window id 捕获单个 desktop-independent window。
  - 输出 PNG。
- `scripts/harness/stage-k-cgevent-click.swift`
  - 后续可用于低层点击导航辅助。
  - 本轮 K6.2 未用它完成额外导航截图。

这些脚本是验证辅助工具，不是产品 UI、不是真实 Codex 执行路径、不是自动工作流能力。

## 关键命令和事实

ScreenCaptureKit harness 编译命令：

```text
CLANG_MODULE_CACHE_PATH=/private/tmp/stage-k-clang-module-cache SWIFT_MODULE_CACHE_PATH=/private/tmp/stage-k-swift-module-cache xcrun swiftc -parse-as-library scripts/harness/stage-k-screencapturekit-window-capture.swift -o /private/tmp/stage-k-screencapturekit-window-capture
```

点击 harness 编译命令：

```text
CLANG_MODULE_CACHE_PATH=/private/tmp/stage-k-clang-module-cache SWIFT_MODULE_CACHE_PATH=/private/tmp/stage-k-swift-module-cache xcrun swiftc scripts/harness/stage-k-cgevent-click.swift -o /private/tmp/stage-k-cgevent-click
```

真实 Tauri dev 已经在本轮 K6.2 中启动，并通过 ScreenCaptureKit 枚举到目标窗口：

```text
window_id=28761 pid=85499 title="Codex 治理工作台 · 首屏已挂载" frame={x:95,y:44,w:1280,h:820}
```

旧的预先存在 Tauri 窗口仍可枚举，但本轮未处理、未 kill：

```text
window_id=28591 pid=49082 title="Codex 治理工作台"
```

成功窗口截图命令：

```text
/private/tmp/stage-k-screencapturekit-window-capture --capture-window-id 28761 --output /Users/yoyi/workspace/product-line/evidence/tauri-verification/2026-06-10-stage-k-k6-2/01-home-screencapturekit-window.png
```

## 截图证据

- `evidence/tauri-verification/2026-06-10-stage-k-k6-2/01-home-screencapturekit-window.png`

截图文件核验：

```text
PNG image data, 1280 x 820, 8-bit/color RGBA, non-interlaced
sha256 17a0cc17b0ee274dd31aa1e2a6553e9ef044845f2f9f96f11174a176ca9ab2c4
```

目视核验：

- 截图显示真实 macOS Tauri 窗口，不是普通浏览器页面。
- 标题栏显示 `Codex 治理工作台 · 首屏已挂载`。
- 内容区不再是白屏。
- 首页主内容可见 `首页`、`项目`、`智能体`、`运行中工作流`、`Skill`、`Harness`。
- 右侧竖向入口可见秘书 / 知识 / 待办 / 运行 / 管理等入口。
- 底部秘书输入预览条可见。

## 与 K6.1 的差异

K6.1 已证明：

- Tauri dev 可启动。
- 前端入口脚本已运行。
- React App 首屏已挂载。
- 但 `screencapture -l` 只能截到标题栏和白色内容区。
- 精确区域截图被系统拒绝。
- CoreGraphics 旧 window image API 在当前 macOS SDK 不可用。

K6.2 新增证明：

- ScreenCaptureKit 的 desktop-independent window capture 可以捕获同一类 Tauri WebView 内容。
- 至少首页真实 Tauri window-only 可见截图链路恢复。
- 后续 K6 不应再使用旧 `screencapture -l` 白屏链路作为唯一窗口截图方式。

## 未完成 / Deferred

- 未完成最小 3 张导航截图：首页 / 智能体 / 运行中工作流。
- 未完成 K6 原清单里的权限弹层、项目工作流、记忆候选确认、记忆正式化、设置开发者区、失败 / readback unavailable 状态截图。
- 未完成 K6 全量真实 Tauri dogfood。
- 未完成 Stage K 最终冻结。
- 未恢复 K3-B1 retry，也未启动 K3-B2。

原因：在成功取得首页截图后，继续采集额外窗口截图时遇到审批 / 安全审查节流，不应通过全屏截图、绕过审批或继续扩大权限来补图。K6.2 的验收标准允许“至少一张真实 Tauri window-only 截图显示工作台可见 UI”作为恢复窗口证明完成，因此本轮只把 K6.2 收口为窗口证明恢复，不冒领 K6 完成。

## 验证结果

- `file evidence/tauri-verification/2026-06-10-stage-k-k6-2/01-home-screencapturekit-window.png`：通过，确认为 1280 x 820 PNG。
- `shasum -a 256 evidence/tauri-verification/2026-06-10-stage-k-k6-2/01-home-screencapturekit-window.png`：通过，hash 为 `17a0cc17b0ee274dd31aa1e2a6553e9ef044845f2f9f96f11174a176ca9ab2c4`。
- 目视核验截图：通过，非白屏真实 Tauri 首页窗口。
- `node scripts/harness/stage-k-architecture-gate.js --target /Users/yoyi/workspace/product-line --strict`：通过，`Errors: 0` / `Warnings: 0`。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 14`。
- `npm run build`：通过，仅保留既有 Vite chunk-size warning。
- ScreenCaptureKit harness 编译：通过。
- CGEvent click harness 编译：通过。

K6.2 本轮没有改产品功能代码；上述前端验证用于确认 checkpoint 收口后的工作树仍干净。后续回到 K6 补完整截图清单时，仍应重新运行 Stage K 计划要求的验证和必要 Rust / Tauri build。

## 边界确认

- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 未使用全屏截图。
- 未 kill 旧 Tauri 进程 `49082`。
- 未改 Product Command / workflow state / memory sidecar schema。
- 未启动 K3-B1 retry。
- 未启动 K3-B2。
- 未实现或触发真实 retry / stop / restart / resume。

## 下一步

建议回到 K6 主任务，而不是再开新的截图恢复任务：

1. 使用 ScreenCaptureKit window-only harness 补齐 K6 最小导航截图清单。
2. 优先覆盖首页、智能体对话页、运行中工作流、项目工作流、记忆候选 / 正式化、设置开发者区和失败 / readback unavailable 状态。
3. 截图过程中继续禁止全屏截图、真实 Codex 执行、K3-B1 retry 和 K3-B2。
4. 补齐后再判断 K6 是否能收口为 `accepted_with_deferred_items` 或继续保留阻断 / deferred。
