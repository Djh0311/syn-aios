# Evidence: Stage H / H6 Real Execution UI Productization And Tauri Acceptance v1

日期：2026-06-08

## 结论

H6 结论冻结为 `accepted_with_deferred_items`。

接受为：

- 真实执行状态 UI 产品化 checkpoint 完成。
- 智能体页、项目工作流侧栏和权限弹层已能用产品语言解释 H2 / H5 既有真实执行证据、权限、runtime / audit / readback、任务包、任务记忆包、worker report / process fact 边界。
- `readback unavailable / failed / timed out` 不被写成真实 0 条结果；主管线已把一处“真实 0 条结果”可见标签改为“读回成功但未命中目标”。
- H6 已按合并型 checkpoint 收口，没有继续拆小 probe。

不接受为：

- 阶段 H 完成。
- 通用自由 send / resume 控制台完成。
- 任意项目 / 任意 session 自由执行开放。
- H3-B retry 成功或新会话产品化完成。
- H4-Level-B 真实失败 / 超时探针完成。
- 自动重试、自动恢复、stop / kill / restart 产品化完成。
- planned adapters 真实接入、provider credential / model verification 完成。
- 真实 Tauri H6 关键截图清单完整完成。

## 范围依据

任务包：

- `tasks/2026-06-08-stage-h-h6-real-execution-ui-productization-and-tauri-acceptance-v1.md`

开发线回交：

- `evidence/2026-06-08-stage-h-h6-real-execution-ui-productization-devline-v1.md`
- `handoffs/2026-06-08-stage-h-h6-real-execution-ui-productization-devline-v1-result.md`

真实 Tauri 部分截图：

- `evidence/tauri-verification/2026-06-08-stage-h-h6/00-window-probe.png`
- `evidence/tauri-verification/2026-06-08-stage-h-h6/_nav-probe-project.png`

## 实现和主管修补

开发线实现：

- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
  - 新增 H6 真实执行状态合并摘要。
  - 会话正文从自动读取改为用户手动“重新读取”，避免默认展开 full transcript。
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
  - 项目工作流侧栏新增 H6 项目工作流真实执行摘要。
  - 汇总 H5 dispatch、任务包、任务记忆包、权限、attempt、readback、worker report candidate、process fact handoff。
- `prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
  - 真实执行类确认弹层补充真实 Codex、`/Users/yoyi/.codex` 副作用和失败 / readback 边界。

主管修补：

- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
  - 将 readback 成功但无目标命中标签从“真实 0 条结果”改为“读回成功但未命中目标”。

## 验证记录

开发线验证：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
```

开发线结果：

```text
typecheck: passed
offline interaction tests passed: 12
build: passed, only existing Vite chunk-size warning
```

验证线补充验证：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib h5_project_dispatch_bridge
cargo test --lib session_continuation
cargo test --lib codex_local_runner
cargo test --lib runtime_log
cargo test --lib diagnostics
cargo test --lib workflow_authorization
rustfmt --check src/h5_project_dispatch_bridge.rs src/session_continuation_store.rs src/codex_local_runner.rs src/runtime_log_store.rs src/types.rs src/commands.rs
```

验证线结果：

```text
frontend: passed
h5_project_dispatch_bridge: 4 passed
session_continuation: 16 passed, 4 ignored
codex_local_runner: 11 passed
runtime_log: 5 passed
diagnostics: 1 passed
workflow_authorization: 1 passed
rustfmt --check: passed
```

主管修补后复跑：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
```

主管复跑结果：

```text
typecheck: passed
offline interaction tests passed: 12
build: passed, only existing Vite chunk-size warning
```

禁止文案扫描结果：

- `src/views/ProjectsView.tsx` 已无“真实 0 条结果”可见标签。
- 剩余命中为测试黑名单、边界说明或 `canvasSurfaceBoundaries.ts` 禁止文案常量。
- `src/lib/projectCanvas.ts` 仍有“readback unavailable 不显示成真实 0 条结果”这类禁止说明，不是完成态文案。

## 真实 Tauri 验收

本轮真实 Tauri 状态：

- 已启动真实 Tauri dev。
- 已确认目标窗口为 `Codex 治理工作台`。
- 已采集窗口探针和导航探针。
- 已停止 Tauri dev，未留下刻意运行的后台验收进程。

完成截图：

```text
evidence/tauri-verification/2026-06-08-stage-h-h6/00-window-probe.png
evidence/tauri-verification/2026-06-08-stage-h-h6/_nav-probe-project.png
```

未完成截图：

```text
01-permission-dialog-real-execution-boundary.png
02-agent-session-center-runtime-state.png
03-send-resume-boundary.png
04-project-workflow-real-execution-state.png
05-workflow-node-execution-detail.png
06-task-memory-packet-preview.png
07-running-panel.png
08-notifications-panel.png
09-todos-panel.png
10-admin-runtime-diagnostics-audit.png
```

阻断原因：

- 验证线启动真实 Tauri 后能截到目标窗口。
- 尝试导航后出现 `could not create image from rect`。
- 后续窗口查询显示 Tauri dev 进程仍在，但 `window 1` 不存在 / 无可截图窗口。
- 因此 H6 真实 Tauri 关键截图清单未完成，不能声明 H6 Tauri acceptance 完整通过。

## 边界确认

本轮没有：

- 执行新的真实 `codex exec`。
- 执行新的真实 `codex exec resume`。
- 发送新的真实 prompt。
- 读写 `/Users/yoyi/.codex`。
- 读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout 正文。
- 新增一级入口、自由 Codex 控制台、裸执行按钮或绕过权限的 send / resume。
- 把 worker report、readback、observation、candidate 或 knowledge hit 写成正式事实 / 正式记忆。

## Acceptance Matrix

| 项目 | 结论 | 证据 |
| --- | --- | --- |
| 智能体页真实执行状态摘要 | accepted | `AgentView.tsx` H6 panel |
| 默认不自动读取 full transcript | accepted | `AgentView.tsx` 手动“重新读取” |
| 项目工作流 H6 摘要 | accepted | `ProjectsView.tsx` H6 card |
| 权限弹层真实执行边界 | accepted | `PermissionDialog.tsx` real Codex boundary |
| unknown readback 不写成 0 | accepted | 主管修补 + 前端复跑 |
| 前端验证 | accepted | typecheck / offline / build passed |
| Rust 边界回归 | accepted | 验证线定向测试 passed |
| 真实 Tauri 窗口启动和探针 | partial | 2 张 PNG |
| H6 10 张关键截图清单 | blocked / deferred | 窗口导航后无可截图 window |

## 下一步

下一步进入 H7：H 阶段最终验收和冻结。

H7 需要复核 H1-H6 的可接受项、deferred 项、执行授权边界、真实 Tauri 缺口、H3-B retry 是否仍需要、H4-Level-B 是否仍保留，以及 I 阶段是否具备前置条件。
