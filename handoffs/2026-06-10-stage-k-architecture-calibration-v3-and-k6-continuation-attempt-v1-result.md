# Stage K / Architecture Calibration v3 And K6 Continuation Attempt Handoff v1

日期：2026-06-10

结论：`architecture_calibration_v3_written_gate_passed_fresh_home_captured_navigation_screenshots_blocked_by_click_delivery`

本轮完成了收口前架构校准计划 v3，并按计划跑了 architecture gate strict 和高风险主路径抽样复核。没有发现阻断 K6 继续的 P0/P1 架构问题。随后继续 K6 fresh Tauri 截图：在 GUI 权限下成功枚举两个 Tauri 窗口，并捕获新窗口 `28971 / pid 6869` 的 fresh 首页 window-only 截图；但 CGEvent 点击未能驱动左侧导航进入智能体页，因此首页以外核心导航截图仍未完成。K6 和 Stage K 仍未完成。

## 本轮新增

- `docs/plans/2026-06-10-stage-k-architecture-calibration-plan-v3.md`
- `evidence/2026-06-10-stage-k-architecture-calibration-v3-and-k6-continuation-attempt-v1.md`
- `handoffs/2026-06-10-stage-k-architecture-calibration-v3-and-k6-continuation-attempt-v1-result.md`
- `evidence/tauri-verification/2026-06-10-stage-k-k6/06-home-screencapturekit-fresh-dev.png`
- `evidence/tauri-verification/2026-06-10-stage-k-k6/07-home-after-agent-click-topcoords.png`
- `evidence/tauri-verification/2026-06-10-stage-k-k6/08-home-after-focus-agent-click.png`
- `evidence/tauri-verification/2026-06-10-stage-k-k6/09-home-after-flipped-agent-click.png`
- `evidence/tauri-verification/2026-06-10-stage-k-k6/10-home-after-activation-agent-click.png`
- `evidence/tauri-verification/2026-06-10-stage-k-k6/11-home-after-old-window-closed-agent-click.png`

## 验证结果

`node scripts/harness/stage-k-architecture-gate.js --target /Users/yoyi/workspace/product-line --strict`

结果：

```text
Status: pass
Errors: 0
Warnings: 0
Info: 36
```

## 抽样复核结论

- 执行主路径：普通智能体页走 `runRealExecutionProductCommand*`，legacy workflow dispatch / workflow machine 在 UI 和 Tauri wrapper 均 blocked。
- readback：未知 / 不可用 / 失败 / 超时仍保持“未知 / 不可用”口径，没有抽样发现显示为真实 0 条结果。
- 记忆层：observation / candidate / knowledge hit 不自动写 FormalMemory；正式记忆仍通过 adoption / lifecycle / audit / confirmation。
- UI 导航：普通入口为项目、智能体、想法箱、知识库、记忆层、Skill、Harness、运行中工作流；开发者入口在 devNavItems。

## K6 截图续跑状态

已恢复：

- GUI 权限下 `ScreenCaptureKit --list --title "Codex 治理工作台"` 成功，确认当前有两个 Tauri 窗口：
  - 新窗口：`window_id=28971 pid=6869 title="Codex 治理工作台 · 首屏已挂载"`
  - 旧窗口：`window_id=28591 pid=49082 title="Codex 治理工作台"`
- 本轮只使用新窗口 `28971` 作为 fresh K6 证据，不混用旧窗口。
- 成功生成 fresh 首页截图：
  - `evidence/tauri-verification/2026-06-10-stage-k-k6/06-home-screencapturekit-fresh-dev.png`
  - `PNG image data, 1280 x 820, 8-bit/color RGBA, non-interlaced`
  - `sha256 9305700ec6a3651811406f24917d3e788b399f7b9c27901d714a49abe9b0ace2`

仍未完成 / 阻断点：

- 沙箱内 ScreenCaptureKit harness 曾挂起，已清理。
- `screencapture -l` 和 `screencapture -R` 仍不可作为本轮可靠截图路径。
- AppleScript 可见进程枚举成功，但窗口详情读取被辅助功能权限拒绝。
- CGEvent 点击左侧“智能体”导航未生效；激活 PID 6869 后再点击仍停留在首页。
- 用户指出本地有两个 Tauri 窗口；主管线经授权关闭旧窗口 `pid=49082`。
- 重新枚举后只剩新窗口 `window_id=28971 pid=6869 title="Codex 治理工作台 · 首屏已挂载"`。
- 关闭旧窗口后再次激活 PID 6869 并点击“智能体”，截图 `11-home-after-old-window-closed-agent-click.png` 仍停留在首页。
- `07` 到 `11` 五张截图均是“尝试点击后仍为首页”的证据，不能冒充智能体页截图。

仍在运行：

- Tauri dev tool session `32081` 仍在运行。
- 本轮没有停止 Tauri dev，避免打断用户可能正在查看的窗口。
- 旧 Tauri 进程 `49082` 已按用户现场问题处理并关闭；后续截图只应使用新窗口 `28971 / pid 6869`。

## 边界确认

- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 未启动 K3-B1 retry。
- 未启动 K3-B2。
- 未改产品代码。
- 未使用全屏截图。

## 下一步建议

优先路径：

1. 保留 ScreenCaptureKit window-only harness 作为截图主路径。
2. 解决导航驱动问题：优先让用户在真实窗口中手动点到目标页后，由主管线截 window-only 图；或另做一个非产品执行的受控 K6 navigation harness。
3. 如果无法补首页以外截图，K6 final 只能写成 `accepted_with_deferred_navigation_screenshots` 或继续阻断，不能声明完整 dogfood。
4. 只有补到核心路径截图或形成明确缺口矩阵后，才能写 K6 final evidence / handoff。

不建议：

- 不要为了补图使用全屏截图冒充 window-only。
- 不要触发真实 Phase B / Codex send。
- 不要恢复 K3-B1 retry 或启动 K3-B2。
- 不要把 v3 plan 或 gate pass 当作 Stage K 完成。
