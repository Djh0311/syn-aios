# Stage K / K6 Real Tauri Dogfood And Stage Acceptance Freeze v1 Evidence

日期：2026-06-10

结论：`blocked_by_tauri_webview_blank_window`

本轮按 K6 任务包执行真实 Tauri dogfood 和阶段验收收口，但不能接受为 K6 完成或 Stage K 完成。真实 Tauri 桌面壳可以启动，窗口标题和窗口 ID 可以识别，窗口级截图可以生成；但 dev 模式和 release no-bundle 模式的内容区均为白屏，HTML 加载兜底也未显示。因此本轮只能接受为 K6 真实 Tauri 阻断证据和修补前置完成，下一步应进入 K6.1 桌面壳 WebView 渲染 / 截图链路修复。

本轮没有执行真实 `codex exec` / `codex exec resume`，没有发送 prompt，没有读写 `/Users/yoyi/.codex`，没有读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout，没有启动 K3-B1 retry，没有启动 K3-B2，没有实现真实 retry / stop / restart / resume。

## 执行依据

- 架构校准计划已存在：`docs/plans/2026-06-10-stage-k-architecture-calibration-plan-v2.md`。
- K6 任务包已存在：`tasks/2026-06-10-stage-k-k6-real-tauri-dogfood-and-stage-acceptance-freeze-v1.md`。
- K6 任务包要求：真实 Tauri UI / 产品链路验收、截图证据、缺口矩阵和 Stage K 完成项 / deferred 项冻结；不能把普通浏览器 smoke 冒充真实 Tauri。

## 本轮代码修补

文件：`prototypes/productized-desktop-shell/index.html`

修补内容：

- 移除 Google Fonts 远程 stylesheet / preconnect，避免桌面壳首屏依赖外网字体或被外部样式请求阻塞。
- 在 `#root` 内加入轻量中文加载兜底 `正在加载工作台...`。若 Tauri 能加载 HTML 但 JS 未执行，应至少显示该兜底；本轮截图仍未显示，说明问题更靠近 WebView 页面加载 / 渲染 / 截图链路。

该修补不改变产品功能、不新增真实执行入口、不改变 Product Command / workflow state / sidecar schema。

## 静态验证

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 14`。
- `npm run build`：通过，仅保留既有 Vite chunk-size warning。
- `node scripts/harness/stage-k-architecture-gate.js --target /Users/yoyi/workspace/product-line --strict`：通过，`Errors: 0` / `Warnings: 0`。
- `../tauri-capability-probe/.tauri-cli/bin/cargo-tauri build --no-bundle`：通过，release binary 产物生成于 `prototypes/productized-desktop-shell/src-tauri/target/release/codex-governance-workbench`；Rust release build 保留既有 dead-code / unused warnings。

## 真实 Tauri dev 探针

命令：

```bash
npm run tauri:dev
```

结果：

- Vite dev server 成功监听 `http://127.0.0.1:5173/`。
- Tauri dev 编译成功并启动 `target/debug/codex-governance-workbench`。
- CoreGraphics 可识别真实窗口：
  - title：`Codex 治理工作台`
  - PID：`44824`
  - window id：`28718`
  - bounds：`X=95, Y=44, Width=1280, Height=820`
- 窗口级截图成功生成，但内容区白屏。
- `Command+R` 刷新后仍白屏。
- HTML 兜底加入并触发 Vite reload 后仍未显示。

截图证据：

- `evidence/tauri-verification/2026-06-10-stage-k-k6/01-home.png`
- `evidence/tauri-verification/2026-06-10-stage-k-k6/01-home-retry-after-wait.png`
- `evidence/tauri-verification/2026-06-10-stage-k-k6/01-home-after-font-fix.png`
- `evidence/tauri-verification/2026-06-10-stage-k-k6/01-home-after-html-fallback.png`
- `evidence/tauri-verification/2026-06-10-stage-k-k6/01-home-after-refresh.png`

文件检查：

- `01-home.png`：PNG image data，`2696 x 1776`。
- `01-home-after-refresh.png`：PNG image data，`2696 x 1776`。

## 真实 Tauri release no-bundle 探针

命令：

```bash
../tauri-capability-probe/.tauri-cli/bin/cargo-tauri build --no-bundle
/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/target/release/codex-governance-workbench
```

结果：

- release no-bundle 构建成功。
- release Tauri 窗口成功启动。
- CoreGraphics 可识别 release 窗口：
  - title：`Codex 治理工作台`
  - PID：`56653`
  - window id：`28738`
  - bounds：`X=95, Y=44, Width=1280, Height=820`
- 窗口级截图成功生成，但内容区仍白屏。

截图证据：

- `evidence/tauri-verification/2026-06-10-stage-k-k6/01-home-release.png`

## 进程和端口

- 本轮 dev Tauri session 已停止。
- 本轮 release Tauri process `56653` 已停止。
- 端口 `5173` 已释放。
- 只读进程复核显示仍有一个本轮开始前就已存在的旧进程：`49082 target/debug/codex-governance-workbench`。本轮未将其作为自己启动的进程处理，也未 kill 该预存进程。

## 缺口矩阵

| 项 | K6 要求 | 本轮事实 | 结论 |
| --- | --- | --- | --- |
| 真实 Tauri 启动 | 启动真实桌面壳 | dev / release 均可启动窗口 | 部分满足 |
| 窗口识别 | 识别标题 / 窗口区域 | CoreGraphics 可识别标题、PID、window id、bounds | 满足 |
| 首页截图 | 真实窗口截图显示首页 | 截图只有标题栏和白色内容区 | 不满足 |
| 核心路径截图 | 覆盖首页、智能体、运行中、项目、记忆等 | 因白屏无法继续导航 | 不满足 |
| Stage K 完成项 / deferred freeze | 基于真实 Tauri dogfood 冻结 | 真实 UI 未确认可见，不能冻结完成 | 不满足 |
| 普通 UI 信息层级复核 | 在真实壳中复核普通 UI | 白屏，无法复核 | 不满足 |

## 本轮不能声明

- 不能声明 K6 完成。
- 不能声明 Stage K 完成。
- 不能声明真实 Tauri UI dogfood 通过。
- 不能声明 K1-K5 普通 UI 在真实壳中已可理解展示。
- 不能声明 12 张截图清单已覆盖。
- 不能声明 K3-B1 retry 成功或 K3-B2 可以开始。
- 不能声明真实 retry / stop / restart / resume 已实现。
- 不能声明任意项目无限制自由控制台完成。

## 下一步

进入 K6.1：Tauri WebView 内容渲染 / 截图链路修复。K6.1 应优先区分两类问题：

1. 真实 Tauri WebView 是否实际未加载页面。
2. 页面实际可见但 `screencapture -l` 对 WebView 内容只截到白屏。

K6.1 修复前，不得把 K6 或 Stage K 收口为完成。
