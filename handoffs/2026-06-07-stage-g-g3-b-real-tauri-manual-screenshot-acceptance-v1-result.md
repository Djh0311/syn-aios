# Handoff: Stage G / G3-B Real Tauri Manual Screenshot Acceptance v1

日期：2026-06-07

## 回收结论

G3-B 当前不能接受为完成。

当前状态：

```text
blocked_waiting_remaining_safe_screenshot_coverage
```

## 执行摘要

本轮完成了 G3-B 的真实 Tauri 前置确认、目标窗口区域截图和部分清单采集：

- 读取并复核 G3-A task / evidence / handoff。
- 读取并复核 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`。
- 读取并复核 `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`。
- 读取并复核 `docs/workbench-frontend-display-boundary-v1.md`。
- 确认 G3-A 冻结截图目录为 `evidence/tauri-verification/2026-06-07-stage-g-g3/`。
- 记录初次 Tauri 启动授权被拒，随后主管线程获得授权并启动真实 Tauri。
- 确认目标 Tauri 进程 `codex-governance-workbench` 可见。
- 确认目标窗口 `Codex 治理工作台` bounds 为 `95,44,1280,820`。
- 使用只截目标 Tauri 窗口区域的方式采集 10 / 13 张编号截图。

## 截图状态

截图目录：

```text
/Users/yoyi/workspace/product-line/evidence/tauri-verification/2026-06-07-stage-g-g3/
```

截图结果：

```text
10 / 13 screenshots captured
```

已生成截图文件：

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

额外探针 / 基线：

- `00-window-probe.png`
- `_window-baseline.png`

未生成截图文件：

- `05-agent-session-center.png`
- `06-send-resume-boundary.png`
- `09-task-memory-packet-preview.png`

## 未覆盖项

以下 G3-A 冻结项未完成真实 Tauri 截图：

- 智能体会话中心。
- send / resume 边界。
- 任务记忆包预览。

未覆盖原因：

- 智能体会话中心：进入全局智能体页会自动读取所选可读会话 transcript；本轮边界禁止读取 full transcript。
- send / resume 边界：相关真实派发确认动作会调用 `codex exec resume` 并写 `/Users/yoyi/.codex`；本轮禁止执行真实 send / resume。
- 任务记忆包预览：当前真实 Tauri 项目状态未能安全显示稳定后端预览结果；未用 mock 或普通浏览器替代。

## 边界确认

本轮未发生：

- 未执行 `codex exec` / `codex exec resume`。
- 未发送真实 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret / auth / token / `.env` / provider credential / keychain / OAuth。
- 未改产品功能代码。
- 未把普通浏览器 smoke 当真实 Tauri。
- 未同步 G3 / G4 / G5 / 阶段 G 完成。

过程偏差：

- 曾创建非最终临时 Tauri 前台探针，已删除。
- 曾误截非目标窗口临时探针，发现后立即删除，未保留为 evidence。

## 建议全局主管复核

- 10 张已采集截图是否均为真实 Tauri 目标窗口区域。
- `01-permission-dialog.png` 是否满足权限弹层覆盖要求，尤其是“不确认前不写入、只生成候选、不写正式记忆”的边界文案。
- `03-project-workflow-canvas.png` 与 `04-workflow-node-detail.png` 是否可分别接受为画布和节点详情证据，还是需要单独节点抽屉 / 更清晰详情截图。
- `13-admin-runtime-log-diagnostics.png` 是否足够覆盖管理入口中的 runtime log + diagnostics。
- 对 `05-agent-session-center.png` 是否允许设计不读取 full transcript 的安全 fixture。
- 对 `06-send-resume-boundary.png` 是否允许只截 disabled / preview 状态，不进入会调用 `codex exec resume` 的确认路径。
- 对 `09-task-memory-packet-preview.png` 是否需要补一个只读后端预览 fixture。

## 下一步

建议下一步保持 G3-B 未完成，等待全局主管复核后决定：

```text
补齐剩余 3 张安全截图，或进入 G3-C 缺口矩阵但不得标记 G3-B accepted。
```

不得将本轮回收为 G3-B 完成。
