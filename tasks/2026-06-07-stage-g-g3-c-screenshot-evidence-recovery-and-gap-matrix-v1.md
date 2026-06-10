# Task Package: Stage G / G3-C Screenshot Evidence Recovery And Gap Matrix v1

状态：已完成 / accepted_with_deferred_items。  
用途：作为全局主管对 G3-B 真实 Tauri 部分截图回交的复核和缺口矩阵；本任务只回收证据、冻结缺口和判断 G4 前置是否满足，不新增产品能力，不补做 G3-B 剩余截图。

## 0. 先说薄弱点

- G3-B 没有完成 13 / 13 张真实 Tauri 截图，只完成 10 / 13。
- G3-C 不能把 G3-B 包装成完成，也不能声称 G3 全量真实 Tauri 验收完成。
- 剩余 `05-agent-session-center.png`、`06-send-resume-boundary.png`、`09-task-memory-packet-preview.png` 都有真实边界原因：full transcript、真实 resume / `.codex` 写入、当前项目状态无稳定后端预览。
- G4 可以进入离线端到端回放，但 G4 必须引用这些缺口，不得把 deferred 项改写成已验收。

## 1. 已知事实 / 未知 / 假设

已知事实：

- G1 Runtime Log Boundary And Minimal Store 已完成。
- G2 Diagnostics Health And Degraded State 已完成。
- G3-A Real Tauri Acceptance Plan And Fixture Freeze 已完成。
- G3-B worker 已回交，状态为 `blocked_waiting_remaining_safe_screenshot_coverage`。
- 真实 Tauri 已启动并可见；目标窗口为 `codex-governance-workbench / Codex 治理工作台`。
- 目标窗口区域截图方式为：

```text
screencapture -x -R95,44,1280,820 <output.png>
```

- 截图目录为：

```text
/Users/yoyi/workspace/product-line/evidence/tauri-verification/2026-06-07-stage-g-g3/
```

- 当前有 10 张编号截图和 2 张探针 / 基线图。

未知：

- 是否存在不读取 full transcript 的安全智能体页 fixture。
- 是否存在不触发真实 `codex exec resume`、不写 `/Users/yoyi/.codex` 的 send / resume boundary 截图路径。
- 是否存在当前项目状态下稳定、只读、非 mock 的任务记忆包预览截图路径。

假设：

- G3-C 允许把剩余 3 项冻结为 deferred，只要后续 G4/G5 不冒领为已完成。
- G4 默认离线 fixture 回放，不依赖 G3-B 13 / 13 全量截图完成。

## 2. 证据矩阵

| G3-A 清单项 | 截图文件 | 主管复核状态 | 结论 |
| --- | --- | --- | --- |
| 权限确认弹层 | `01-permission-dialog.png` | 已抽看，是真实 Tauri 窗口；弹层显示写入边界和只生成候选 | accepted |
| 项目页 | `02-projects.png` | 文件存在，PNG 有效，来自目标窗口区域 | accepted |
| 项目工作流画布 | `03-project-workflow-canvas.png` | 已抽看，显示项目工作流、节点摘要和 guard 阻断 | accepted |
| 节点详情 | `04-workflow-node-detail.png` | 文件存在，PNG 有效，来自目标窗口区域 | accepted |
| 智能体会话中心 | `05-agent-session-center.png` | 未采集；进入路径存在读取 full transcript 风险 | deferred |
| send / resume 边界 | `06-send-resume-boundary.png` | 未采集；真实确认路径会触发 `codex exec resume` / `.codex` 写入风险 | deferred |
| 记忆中心 | `07-memory-center.png` | 文件存在，PNG 有效，来自目标窗口区域 | accepted |
| 知识库 | `08-knowledge-base.png` | 已抽看，显示知识库边界和 Obsidian-compatible 占位 | accepted |
| 任务记忆包预览 | `09-task-memory-packet-preview.png` | 未采集；当前项目状态未能形成安全可见后端预览 | deferred |
| 运行中 | `10-running.png` | 文件存在，PNG 有效，来自目标窗口区域 | accepted |
| 通知 | `11-notifications.png` | 文件存在，PNG 有效，来自目标窗口区域 | accepted |
| 待办 | `12-todos.png` | 文件存在，PNG 有效，来自目标窗口区域 | accepted |
| 管理 runtime log + diagnostics | `13-admin-runtime-log-diagnostics.png` | 已抽看，显示管理入口中的诊断和 runtime log 边界 | accepted |

## 3. 截图目录索引

已采集编号截图：

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

探针 / 基线：

- `00-window-probe.png`
- `_window-baseline.png`

未采集：

- `05-agent-session-center.png`
- `06-send-resume-boundary.png`
- `09-task-memory-packet-preview.png`

## 4. UI 显示边界确认

本任务是否改前端：

- [x] 不改前端、不改读模型、不改 UI 文案。
- [ ] 改前端类型 / Tauri wrapper，但不新增可见 UI。
- [ ] 改读模型摘要或状态显示。
- [ ] 改已有页面局部 UI。
- [ ] 新增入口、面板、tab、按钮或确认动作。

本任务允许声明：

- G3-B 已回交。
- G3-B 真实 Tauri 部分截图证据为 10 / 13。
- G3-C 缺口矩阵已完成。
- G4 可以进入离线回放准备和执行，但必须携带 G3 缺口。

本任务禁止声明：

- G3-B 已完成。
- G3 真实 Tauri 全量验收完成。
- 13 / 13 截图已完成。
- 智能体页、send / resume、任务记忆包预览已经真实窗口验收完成。
- G4 / G5 / 阶段 G 已完成。

## 5. 验收

验收命令 / 操作：

- `ls -lh /Users/yoyi/workspace/product-line/evidence/tauri-verification/2026-06-07-stage-g-g3`
- `file /Users/yoyi/workspace/product-line/evidence/tauri-verification/2026-06-07-stage-g-g3/*.png`
- 抽看 `01-permission-dialog.png`
- 抽看 `03-project-workflow-canvas.png`
- 抽看 `08-knowledge-base.png`
- 抽看 `13-admin-runtime-log-diagnostics.png`
- 读取 G3-B worker handoff。

当前结论：

```text
accepted_with_deferred_items
```

接受为：

- G3-B 截图证据回收。
- 10 / 13 真实 Tauri 截图索引和主管复核完成。
- 3 个缺口原因冻结。
- G4 离线端到端回放的前置证据矩阵满足。

不接受为：

- G3-B 完成。
- G3 全量真实 Tauri 验收完成。
- 阶段 G 完成。

## 6. 下一步

下一步进入 G4 Middle Version End-to-End Acceptance Replay。G4 必须：

- 默认走离线 fixture / 读模型回放。
- 引用 G3-C 的 deferred 项。
- 不执行真实 `codex exec` / `codex exec resume`。
- 不发送真实 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不把 G3 缺口包装成已完成。
