# Evidence: Stage G / G3-B Real Tauri Manual Screenshot Acceptance v1

日期：2026-06-07

## 结论

G3-B 未完成，当前状态为：

```text
blocked_waiting_remaining_safe_screenshot_coverage
```

原因：

- 真实 Tauri 已由主管线程授权启动。
- 目标 Tauri 窗口已确认可见。
- 目标窗口区域截图探针已成功。
- 本轮已采集 10 / 13 张编号截图。
- 剩余 3 项存在当前边界风险或未能形成安全可见截图路径，不能冒领完成。

## 已完成

- 只读复核 G3-A task / evidence / handoff。
- 只读复核 UI 边界文档和权威入口。
- 确认当前入口为 G3-B Real Tauri Manual Screenshot Acceptance。
- 确认 G3-A 冻结截图目录：

```text
/Users/yoyi/workspace/product-line/evidence/tauri-verification/2026-06-07-stage-g-g3/
```

- 记录真实 Tauri 状态：
  - 初次授权请求被自动审查拒绝。
  - 随后主管线程获得授权并启动真实 Tauri。
  - 可见进程包含 `codex-governance-workbench`。
  - 目标窗口标题为 `Codex 治理工作台`。
  - 目标窗口 bounds 为 `95, 44, 1280, 820`。
- 记录安全截图方式：

```text
screencapture -x -R95,44,1280,820 <output.png>
```

- 采集 10 张真实 Tauri 编号截图。

## 截图目录状态

目录：

```text
evidence/tauri-verification/2026-06-07-stage-g-g3/
```

当前编号截图：

```text
10 / 13
```

文件列表：

```text
00-window-probe.png
01-permission-dialog.png
02-projects.png
03-project-workflow-canvas.png
04-workflow-node-detail.png
07-memory-center.png
08-knowledge-base.png
10-running.png
11-notifications.png
12-todos.png
13-admin-runtime-log-diagnostics.png
_window-baseline.png
```

## 已采集截图说明

- `01-permission-dialog.png`：知识库“提出记忆候选”低风险路径打开的本机动作确认弹层；截图后用 Escape 取消，未确认写入。
- `02-projects.png`：真实 Tauri 项目入口 / 项目列表。
- `03-project-workflow-canvas.png`：真实 Tauri 项目工作流画布。
- `04-workflow-node-detail.png`：真实 Tauri 项目工作流画布右侧节点详情状态。
- `07-memory-center.png`：真实 Tauri 记忆中心。
- `08-knowledge-base.png`：真实 Tauri 知识库。
- `10-running.png`：真实 Tauri 右侧“项目运行”入口。
- `11-notifications.png`：真实 Tauri 右侧“通知中心”入口。
- `12-todos.png`：真实 Tauri 右侧“待办中心”入口。
- `13-admin-runtime-log-diagnostics.png`：真实 Tauri 右侧“管理”入口，包含 diagnostics / runtime log 边界。

## 未采集截图

- `05-agent-session-center.png`
- `06-send-resume-boundary.png`
- `09-task-memory-packet-preview.png`

未覆盖原因：

- `05-agent-session-center.png`：进入全局智能体页的实现会自动加载所选可读会话 transcript；本轮边界禁止读取 full transcript，因此未进入补图。
- `06-send-resume-boundary.png`：项目工作流 send / resume 相关确认动作文案明确会调用 `codex exec resume` 并写 `/Users/yoyi/.codex`；本轮禁止真实 send / resume 和写 `/Users/yoyi/.codex`，因此未触发。
- `09-task-memory-packet-preview.png`：任务记忆包预览面板在当前真实项目状态下未能形成安全可见的后端预览结果；未用 mock 或普通浏览器替代。

## 授权与阻断记录

初次请求启动真实 Tauri：

```text
npm run tauri:dev
```

初次结果：

```text
rejected_by_auto_review
```

后续主管线程补充事实：

```text
真实 Tauri 已启动；Vite Local: http://127.0.0.1:5173/
可见进程包含 codex-governance-workbench
目标窗口 bounds: 95, 44, 1280, 820
00-window-probe.png 区域截图成功
```

本轮截图策略：

```text
不全屏截图；只截目标 Tauri 窗口区域 95,44,1280,820。
```

过程偏差记录：

- 本轮曾创建一个临时前台探针确认 Tauri 已置前，已删除。
- 本轮曾误截到非目标窗口的临时探针，发现后立即删除，未保留为 evidence。
- 该偏差不计入 G3-B 证据，也不改变本轮未完成结论。

## 未发生

- 未执行真实 `codex exec`。
- 未执行真实 `codex exec resume`。
- 未发送真实 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secrets / auth / token / `.env` / provider credential / keychain / OAuth。
- 未改产品功能代码。
- 未写 workflow state。
- 未写正式记忆、候选、observation 或 runtime log。
- 未用普通浏览器 smoke 替代真实 Tauri。

## 验证

已执行：

- `sed -n '1,240p' CURRENT.md`
- `sed -n '1,220p' tasks/README.md`
- `sed -n '1,220p' AUTHORITY.md`
- `sed -n '1,220p' STAGE_PLAN.md`
- `sed -n '1,260p' docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`
- `sed -n '1,260p' docs/workbench-frontend-display-boundary-v1.md`
- `sed -n '1,260p' tasks/2026-06-07-stage-g-g3-a-real-tauri-acceptance-plan-and-fixture-freeze-v1.md`
- `sed -n '1,260p' evidence/2026-06-07-stage-g-g3-a-real-tauri-acceptance-plan-and-fixture-freeze-v1.md`
- `sed -n '1,260p' handoffs/2026-06-07-stage-g-g3-a-real-tauri-acceptance-plan-and-fixture-freeze-v1-result.md`
- `osascript -e 'tell application "System Events" to tell process "codex-governance-workbench" to get {name of every window, position of window 1, size of window 1}'`
- `osascript -e 'tell application "System Events" to tell process "codex-governance-workbench" to get entire contents of window 1'`
- `screencapture -x -R95,44,1280,820 ...`
- `find evidence/tauri-verification/2026-06-07-stage-g-g3 -maxdepth 1 -type f -print | sort`

未执行：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `cargo test`

说明：本轮未改产品代码；代码验证不能替代真实窗口截图。G3-B 因剩余 3 张截图未覆盖，仍不能接受为完成。

## 当前结论

G3-B 不能接受为完成。下一步需要全局主管复核 10 张已采集截图，并决定是否另行设计安全 fixture 补齐剩余 3 项。
