# Task Package: Stage G / G3-B Real Tauri Manual Screenshot Acceptance v1

状态：未完成 / blocked_waiting_remaining_safe_screenshot_coverage。  
用途：按 G3-A 冻结清单执行真实 Tauri 手动截图验收；本轮已确认真实 Tauri 启动、目标窗口可见、目标窗口区域截图可用，并完成 10 / 13 张编号截图。因剩余 3 项存在边界风险或当前 UI 状态未能安全覆盖，不能回收为 G3-B 完成。

## 0. 先说薄弱点

- G3-B 的核心验收物是 13 张真实 Tauri 窗口截图；普通浏览器 smoke、DOM 测试或文档复核都不能替代。
- 本轮只允许截目标 Tauri 窗口区域，不允许全屏截图，避免捕获无关敏感窗口。
- 智能体页存在自动读取可读会话 transcript 的实现风险；在本轮“不得读取 full transcript”的边界下，不能为补图强行进入。
- send / resume 派发确认文案明确会调用 `codex exec resume` 并写 `/Users/yoyi/.codex`；本轮不得执行或触发真实 send / resume。
- 任务记忆包预览在当前真实 Tauri 项目状态下未能形成安全可见的后端预览截图；不能用 mock 或普通浏览器替代。

## 1. 已知事实 / 未知 / 假设

已知事实：

- G1 Runtime Log Boundary And Minimal Store 已完成。
- G2 Diagnostics Health And Degraded State 已完成。
- G3-A Real Tauri Acceptance Plan And Fixture Freeze 已完成。
- G3-A 冻结的截图目录为：

```text
evidence/tauri-verification/2026-06-07-stage-g-g3/
```

- 初次真实 Tauri 启动授权请求被自动审查拒绝。
- 随后主管线程获得授权并在 `prototypes/productized-desktop-shell` 执行 `npm run tauri:dev`，Vite 显示 `Local: http://127.0.0.1:5173/`。
- 主管线程确认可见进程列表包含 `codex-governance-workbench`。
- 目标窗口标题为 `Codex 治理工作台`。
- 目标窗口 bounds 为 `95, 44, 1280, 820`。
- 全屏 `screencapture` 因可能捕获无关敏感窗口被拒绝。
- 区域截图探针已成功：

```text
evidence/tauri-verification/2026-06-07-stage-g-g3/00-window-probe.png
```

- 本轮继续使用安全区域截图方式：

```text
screencapture -x -R95,44,1280,820 <output.png>
```

- 本轮完成 10 / 13 张编号截图。

未知：

- 是否存在一个不会读取完整 transcript 的安全智能体页截图路径。
- 是否存在一个只显示 send / resume preview / stub 且不会触发 `codex exec resume`、不会写 `/Users/yoyi/.codex` 的安全真实 Tauri 截图路径。
- 是否存在一个当前项目可见、稳定、不会写 sidecar 的任务记忆包预览截图路径。

假设：

- 在边界不确定时，G3-B 必须保守记录未覆盖，不得为补齐截图冒领或越界。
- G3-B 未完成时不能推进为 G3-C accepted 回收、G4 回放或 G5 最终冻结。

## 2. 本轮执行结果

已完成：

- 只读复核 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`。
- 只读复核 `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`。
- 只读复核 `docs/workbench-frontend-display-boundary-v1.md`。
- 只读复核 G3-A task / evidence / handoff。
- 确认 G3-A 冻结截图目录。
- 记录初次授权被拒、后续主管线程获授权并启动真实 Tauri 的事实。
- 确认真实 Tauri 进程 `codex-governance-workbench` 可见。
- 确认目标窗口标题和 bounds：`Codex 治理工作台` / `95,44,1280,820`。
- 使用只截目标 Tauri 窗口区域的安全方式采集截图。
- 完成权限弹层、项目页、项目工作流画布、节点详情、记忆、知识库、运行中、通知、待办、管理 runtime log + diagnostics 共 10 张编号截图。

未完成：

- `05-agent-session-center.png`
- `06-send-resume-boundary.png`
- `09-task-memory-packet-preview.png`

阻断原因：

```text
blocked_waiting_remaining_safe_screenshot_coverage
```

## 3. G3-A 冻结截图清单

已采集：

- `01-permission-dialog.png`
- `02-projects.png`
- `03-project-workflow-canvas.png`
- `04-workflow-node-detail.png`
- `07-memory-center.png`
- `08-knowledge-base.png`
- `10-running.png`
- `11-notifications.png`
- `12-todos.png`
- `13-admin-runtime-log-diagnostics.png`

未采集：

- `05-agent-session-center.png`
- `06-send-resume-boundary.png`
- `09-task-memory-packet-preview.png`

当前状态：

```text
10 / 13 screenshots captured
```

额外探针 / 基线：

- `00-window-probe.png`
- `_window-baseline.png`

说明：

- `00-window-probe.png` 是主管补充的目标窗口区域截图探针。
- `_window-baseline.png` 是已有窗口基线。
- 本轮曾创建一个非最终临时前台探针确认 Tauri 已置前，已删除。
- 本轮曾误截到非目标窗口的临时探针，发现后立即删除，未保留为 evidence。

## 4. UI 显示边界确认

本任务是否改前端：

- [x] 不改前端、不改读模型、不改 UI 文案。
- [ ] 改前端类型 / Tauri wrapper，但不新增可见 UI。
- [ ] 改读模型摘要或状态显示。
- [ ] 改已有页面局部 UI。
- [ ] 新增入口、面板、tab、按钮或确认动作。

已读取：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`
- `tasks/2026-06-07-stage-g-g3-a-real-tauri-acceptance-plan-and-fixture-freeze-v1.md`
- `evidence/2026-06-07-stage-g-g3-a-real-tauri-acceptance-plan-and-fixture-freeze-v1.md`
- `handoffs/2026-06-07-stage-g-g3-a-real-tauri-acceptance-plan-and-fixture-freeze-v1-result.md`

本任务允许显示：

- 真实 Tauri 已启动且目标窗口可见。
- 目标窗口区域截图已成功。
- 已完成的真实 Tauri 编号截图。
- 剩余未覆盖项及边界原因。
- G3-B 仍未完成。

本任务禁止显示：

- G3-B 已完成。
- G3 真实 Tauri 验收已完成。
- G4 / G5 / 阶段 G 已完成。
- 普通浏览器 smoke 替代真实 Tauri。
- 进入智能体页后读取完整 transcript 作为截图代价。
- 触发真实 `codex exec` / `codex exec resume` 或写 `/Users/yoyi/.codex`。

## 5. 验收

本轮验收结论：

```text
blocked_waiting_remaining_safe_screenshot_coverage
```

验证命令 / 操作：

- `osascript -e 'tell application "System Events" to tell process "codex-governance-workbench" to get {name of every window, position of window 1, size of window 1}'`
- `osascript -e 'tell application "System Events" to tell process "codex-governance-workbench" to get entire contents of window 1'`
- `screencapture -x -R95,44,1280,820 /Users/yoyi/workspace/product-line/evidence/tauri-verification/2026-06-07-stage-g-g3/<file>.png`
- `find evidence/tauri-verification/2026-06-07-stage-g-g3 -maxdepth 1 -type f -print | sort`

未运行：

- 未运行 npm / cargo 验证，因为本轮未改产品代码。
- 未执行真实 `codex exec`。
- 未执行真实 `codex exec resume`。
- 未发送真实 prompt。

## 6. 下一步

下一步建议：

- 由全局主管复核当前 10 张真实 Tauri 截图是否可作为部分证据保留。
- 单独设计安全路径补 `05-agent-session-center.png`，前提是不读取完整 transcript。
- 单独设计只读 send / resume boundary 截图路径，前提是不触发真实 resume、不写 `/Users/yoyi/.codex`。
- 单独设计任务记忆包预览 fixture，前提是不写 sidecar、不用 mock 冒充后端预览。

不得把本轮记录解释为 G3-B 完成。
