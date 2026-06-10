# Stage K / Architecture Calibration v3 And K6 Continuation Attempt Evidence v1

日期：2026-06-10

结论：`architecture_calibration_v3_written_gate_passed_fresh_home_captured_navigation_screenshots_blocked_by_click_delivery`

本轮按“原目标不变，先写架构校准计划，然后继续工作”的要求执行。已新增 Stage K 收口前架构校准计划 v3，并重新运行 Stage K architecture gate。gate strict 通过，0 error / 0 warning。随后继续 K6 fresh Tauri 截图。最初在沙箱内的 ScreenCaptureKit / `screencapture` 通道失败；改用已授权 GUI 权限后，成功枚举并捕获新 Tauri dev 窗口 `28971` 的 fresh 首页 window-only 截图。后续尝试用 CGEvent 点击左侧“智能体”导航，但窗口画面始终停留在首页，因此本轮只新增 fresh 首页证明，未完成首页以外核心导航截图。K6 和 Stage K 仍不能声明完成。

本轮没有执行真实 `codex exec` / `codex exec resume`，没有发送 prompt，没有读写 `/Users/yoyi/.codex`，没有读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout，没有启动 K3-B1 retry，没有启动 K3-B2。

## 新增文档

- `docs/plans/2026-06-10-stage-k-architecture-calibration-plan-v3.md`

v3 定位：

- 不替代 v1 / v2。
- v1 是 K2.5 前置架构校准。
- v2 是 K3-B1 retry 被安全审查拒绝后的推进策略。
- v3 是 K4 / K5 / K6.2 后、K6 / Stage K 收口前的架构校准和继续推进计划。

v3 明确：

- Stage K 原目标不变。
- 不暂停 Stage K 做大重写。
- K3-B1 retry 和 K3-B2 继续冻结。
- 先跑 architecture gate 和只读抽样复核，再继续 K6 真实 Tauri dogfood。
- P0/P1 阻断，P2 进入 deferred 或后续局部修补。

## Architecture Gate

命令：

```text
node scripts/harness/stage-k-architecture-gate.js --target /Users/yoyi/workspace/product-line --strict
```

结果：

```text
Status: pass
Errors: 0
Warnings: 0
Info: 36
```

分类摘要：

- `Command::new("codex")` 未在批准 runner 之外形成 error。
- `prompt_body` 命中均为批准 runtime boundary、commands/types/UI 入参边界、测试或文档边界。
- `result_count=0` 命中均为测试 fixture；产品显示仍要求 unknown / failed / unavailable 保持 null。
- formal memory 风险文案命中均为否定边界或受控正式记忆操作，不是 candidate / observation 自动写正式记忆。

## 只读架构抽样复核

### 执行主路径

抽样文件：

- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`

发现：

- 普通智能体页真实入口调用 `runRealExecutionProductCommandPhaseA` / `runRealExecutionProductCommandPhaseB` / `runRealExecutionProductCommandNewSessionPhaseB`，属于 Stage K Product Command 主路径。
- `execute_workflow_node_dispatch` 和 `run_workflow_machine` 在 `App.tsx` 中抛出 `legacyProductCommandBlockedNotice(...)`，不会由普通 UI 直接执行。
- Tauri command wrapper `execute_workflow_node_dispatch`、`read_workflow_node_dispatch_result`、`run_workflow_machine` 返回 `legacy_product_command_blocked_message(...)`。
- CLI `__run_workflow_machine_real` 返回 `legacy_product_command_blocked_message(...)`。

结论：本轮抽样未发现普通 UI 可绕过 Product Command 直接真实调用 Codex 的 P0/P1。

### readback 语义

抽样文件：

- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/views/RunningWorkflowsView.tsx`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/lib/runQueue.ts`
- `prototypes/productized-desktop-shell/src/lib/projectCanvas.ts`

发现：

- `result_count = null` 显示为 `未知 / 不可用` 或等价文案。
- 多处明确写明读回不可用 / 失败 / 超时不能显示成 0 条结果。
- 未发现普通产品文案把 readback unavailable / failed 包装成真实 0 条结果。

结论：本轮抽样未发现 readback unknown -> 0 的 P0/P1。

### 记忆层边界

抽样文件：

- `prototypes/productized-desktop-shell/src/lib/memoryCenter.ts`
- `prototypes/productized-desktop-shell/src/lib/knowledgeBase.ts`
- `prototypes/productized-desktop-shell/src/lib/runQueue.ts`
- `prototypes/productized-desktop-shell/src/views/MemoryCenterView.tsx`
- `prototypes/productized-desktop-shell/src/views/KnowledgeBaseView.tsx`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`

发现：

- 多处明确写明 observation / candidate / knowledge hit 不是正式记忆。
- 知识库资料只生成候选，不写正式记忆。
- 正式记忆操作仍走 lifecycle / adoption / user confirmation / audit 链路。

结论：本轮抽样未发现 candidate / observation / knowledge hit 自动写 FormalMemory 的 P0/P1。

### UI 信息层级

抽样文件：

- `prototypes/productized-desktop-shell/src/lib/workbenchNavigation.ts`

发现：

- 普通主入口为 `项目 / 智能体 / 想法箱 / 知识库 / 记忆层 / Skill / Harness / 运行中工作流`。
- `设置` 单独保留为底部入口。
- `建议方案 / 实验画布 / 工具 / 模型/凭据` 保留为 `devNavItems`。
- 右侧入口为 `秘书 / 通知 / 待办 / 运行中 / 管理`。

结论：导航层级符合 Stage K 当前产品形态；未发现 P0/P1。

## K6 continuation attempt

背景：

- K6.2 已成功用 ScreenCaptureKit window-only harness 捕获 `window_id=28761` 的真实首页截图。
- 当前交接中记录新 Tauri dev 会话仍在运行，之前枚举到窗口 `window_id=28971 pid=6869 title="Codex 治理工作台 · 首屏已挂载" frame={x:95,y:44,w:1280,h:820}`。
- 本轮继续尝试对该 fresh dev 窗口补图。

### 第一轮：沙箱内截图通道失败

尝试 1：ScreenCaptureKit list

```text
/private/tmp/stage-k-screencapturekit-window-capture --list --title "Codex 治理工作台"
```

结果：

- 30s 无输出，工具会话挂起。

尝试 2：ScreenCaptureKit capture by known window id

```text
/private/tmp/stage-k-screencapturekit-window-capture --capture-window-id 28971 --output /Users/yoyi/workspace/product-line/evidence/tauri-verification/2026-06-10-stage-k-k6/06-home-screencapturekit-fresh-dev.png
```

结果：

- 30s 无输出，工具会话挂起。

处理：

```text
pkill -f /private/tmp/stage-k-screencapturekit-window-capture
```

结果：

- 仅清理本轮挂起的 ScreenCaptureKit harness 进程。
- 未 kill Tauri。
- 未 kill 旧 Tauri 进程 `49082`。

尝试 3：window id 截图 fallback

```text
screencapture -l 28971 /Users/yoyi/workspace/product-line/evidence/tauri-verification/2026-06-10-stage-k-k6/06-home-screencapture-fresh-dev.png
```

结果：

```text
could not create image from window
```

尝试 4：旧 window id 对照

```text
screencapture -l 28591 /Users/yoyi/workspace/product-line/evidence/tauri-verification/2026-06-10-stage-k-k6/06-home-screencapture-existing-window.png
```

结果：

```text
could not create image from window
```

尝试 5：已知 bounds 区域截图

```text
screencapture -R95,44,1280,820 /Users/yoyi/workspace/product-line/evidence/tauri-verification/2026-06-10-stage-k-k6/06-home-region-known-bounds.png
```

结果：

```text
rect (95.0, 44.0, 1280.0, 820.0) does not intersect any displays
```

尝试 6：显示器信息

```text
system_profiler SPDisplaysDataType
```

结果：

- 当前只返回 GPU 信息，没有返回可用显示器几何。

尝试 7：AppleScript 只读窗口信息

命令 1：

```text
osascript -e 'tell application "System Events" to get the name of every process whose visible is true'
```

结果：

- 可见进程中存在两个 `codex-governance-workbench`。

命令 2：

```text
osascript -e 'tell application "System Events" to get {name, position, size} of every window of every process whose name is "codex-governance-workbench"'
```

结果：

```text
“osascript”不允许辅助访问。 (-25211)
```

Tauri dev 会话复核：

```text
session 32081
2026-06-10 15:40:38.994 codex-governance-workbench[6869:14559608] error messaging the mach port for IMKCFRunLoopWakeUpReliable
```

阶段结论：

- Tauri dev 会话仍在运行。
- 沙箱内截图 / 窗口查询通道不可用。

### 第二轮：GUI 权限下 fresh 首页截图恢复

非沙箱 GUI 权限下重新枚举窗口：

```text
/private/tmp/stage-k-screencapturekit-window-capture --list --title "Codex 治理工作台"
```

结果：

```text
window_id=28971 pid=6869 app="codex-governance-workbench" bundle="" title="Codex 治理工作台 · 首屏已挂载" frame={x:95,y:44,w:1280,h:820}
window_id=28591 pid=49082 app="codex-governance-workbench" bundle="" title="Codex 治理工作台" frame={x:95,y:44,w:1280,h:820}
```

确认当前确实存在两个 Tauri 窗口。本轮只使用新窗口 `28971`，未使用旧窗口 `28591` 作为 K6 fresh dev 证据。

fresh 首页截图命令：

```text
/private/tmp/stage-k-screencapturekit-window-capture --capture-window-id 28971 --output /Users/yoyi/workspace/product-line/evidence/tauri-verification/2026-06-10-stage-k-k6/06-home-screencapturekit-fresh-dev.png
```

结果：

```text
captured window_id=28971 output="/Users/yoyi/workspace/product-line/evidence/tauri-verification/2026-06-10-stage-k-k6/06-home-screencapturekit-fresh-dev.png" size=1280x820
```

文件核验：

```text
PNG image data, 1280 x 820, 8-bit/color RGBA, non-interlaced
sha256 9305700ec6a3651811406f24917d3e788b399f7b9c27901d714a49abe9b0ace2
```

目视结论：

- 截图为真实 Tauri 新窗口，不是普通浏览器。
- 标题栏显示 `Codex 治理工作台 · 首屏已挂载`。
- 内容区可见首页、项目、智能体、运行中工作流、Skill、Harness、右侧秘书栏和底部秘书输入预览。

### 第三轮：导航点击未生效

为继续补首页以外截图，尝试 CGEvent 点击左侧“智能体”导航：

```text
/private/tmp/stage-k-cgevent-click 130 204
/private/tmp/stage-k-cgevent-click 131 204
/private/tmp/stage-k-cgevent-click 131 704
```

同时尝试：

- 点击窗口内部以获取焦点。
- 使用 `NSRunningApplication(processIdentifier: 6869).activate(...)` 激活新 Tauri dev 窗口，返回 `activated=true`。
- 重新点击左侧导航。
- 用户指出本地有两个 Tauri 窗口；主管线确认旧窗口 `pid=49082 / window_id=28591` 可能干扰点击投递，经授权执行 `kill 49082` 关闭旧窗口。
- 重新枚举后只剩新窗口：

```text
window_id=28971 pid=6869 app="codex-governance-workbench" bundle="" title="Codex 治理工作台 · 首屏已挂载" frame={x:95,y:44,w:1280,h:820}
```

- 再次激活 PID 6869 并点击“智能体”，仍未导航。

捕获到的确认截图：

- `evidence/tauri-verification/2026-06-10-stage-k-k6/07-home-after-agent-click-topcoords.png`
- `evidence/tauri-verification/2026-06-10-stage-k-k6/08-home-after-focus-agent-click.png`
- `evidence/tauri-verification/2026-06-10-stage-k-k6/09-home-after-flipped-agent-click.png`
- `evidence/tauri-verification/2026-06-10-stage-k-k6/10-home-after-activation-agent-click.png`
- `evidence/tauri-verification/2026-06-10-stage-k-k6/11-home-after-old-window-closed-agent-click.png`

目视结论：

- 四张点击后截图仍停留在首页。
- 关闭旧窗口后的第 11 张截图仍停留在首页。
- 当前工具未能驱动 Tauri WebView 导航点击。
- 这些文件已按实际内容命名为 `home-after-...`，不冒充智能体页截图。

最终结论：

- 本轮新增了一张可接受的 fresh Tauri 首页截图。
- 首页以外核心导航截图仍未完成。
- K6 仍未完成，Stage K 仍未完成。

## 本轮文件变化

新增：

- `docs/plans/2026-06-10-stage-k-architecture-calibration-plan-v3.md`
- `evidence/2026-06-10-stage-k-architecture-calibration-v3-and-k6-continuation-attempt-v1.md`
- `evidence/tauri-verification/2026-06-10-stage-k-k6/06-home-screencapturekit-fresh-dev.png`
- `evidence/tauri-verification/2026-06-10-stage-k-k6/07-home-after-agent-click-topcoords.png`
- `evidence/tauri-verification/2026-06-10-stage-k-k6/08-home-after-focus-agent-click.png`
- `evidence/tauri-verification/2026-06-10-stage-k-k6/09-home-after-flipped-agent-click.png`
- `evidence/tauri-verification/2026-06-10-stage-k-k6/10-home-after-activation-agent-click.png`
- `evidence/tauri-verification/2026-06-10-stage-k-k6/11-home-after-old-window-closed-agent-click.png`

未改产品代码。

## 保留事项

- K6 仍需补齐真实 Tauri 核心路径截图，或记录为无法完成并冻结 deferred。
- 若继续截图，建议先由用户把 Tauri 窗口置顶并确认屏幕录制 / 辅助功能权限；或者在有稳定显示器几何的会话里重新运行 ScreenCaptureKit harness。
- Tauri dev session `32081` 当前仍在运行；本轮没有停止它，避免打断用户可能正在查看的窗口。
- K3-B1 retry 仍被安全审查拒绝；K3-B2 不得启动。

## 不能声明

- 不能声明 K6 完成。
- 不能声明 Stage K 完成。
- 不能声明 K3-B1 retry 成功。
- 不能声明 K3-B2 可开始。
- 不能声明任意项目无限制自由控制台完成。
- 不能声明真实 retry / stop / restart / resume 已实现。
- 不能声明 planned adapters 真实接入。
- 不能声明 provider credential / model verification 完成。
