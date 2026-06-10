# Evidence: Stage H / H6 Real Execution UI Productization Devline v1

日期：2026-06-08

## 结论

H6 开发线已完成最小 UI / 读模型产品化改动：

- 智能体页新增 H6 真实执行状态合并摘要，聚合 `codex-local`、operation、target session、readiness、attempt、runtime/audit/readback 和 failure boundary。
- 智能体页取消选中会话后的自动 transcript 读取；会话正文只在用户点击重新读取时读取。
- 项目工作流节点侧栏新增 H6 项目工作流真实执行摘要，聚合 H5 command / dispatch、任务包、任务记忆包、权限、attempt、readback、worker report candidate 和 process fact handoff。
- 真实执行类权限弹层补充真实 Codex、Codex home、失败处理边界说明。

本轮没有新增一级入口、没有把智能体页改成自由 Codex 控制台、没有新增真实执行按钮。

## 修改文件

- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`

## 验证

已通过：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
```

结果摘要：

```text
typecheck: passed
offline interaction tests passed: 12
build: passed
```

保留提示：

```text
vite build chunk-size warning: 部分 chunk 超过 500 kB
```

## 边界确认

本轮没有：

- 执行真实 `codex exec`。
- 执行真实 `codex exec resume`。
- 发送 prompt。
- 读写 `/Users/yoyi/.codex`。
- 读取 secret/token/.env/keychain/OAuth/provider credential。
- 读取 full transcript / rollout 作为本轮验证动作。
- 同步 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md` 或 H-I plan。
- 启动真实 Tauri 或采集 GUI 截图。

## UI 显示边界

本轮触碰 UI：

- 改读模型摘要或状态显示。
- 改已有页面局部 UI。

本轮未触碰：

- 一级入口。
- 右侧入口结构。
- 新 tab。
- 新真实执行确认动作。

新增文案保持边界：

- `readback unavailable / failed / timed out` 保持 result_count unknown / null，不显示为 0 条。
- prepared dispatch、worker report、process fact、candidate 不写成正式事实或正式记忆。
- planned adapters 不显示为真实可执行。

## Tauri 验收状态

真实 Tauri 截图验收未执行。原因：

- H6 委派边界要求如需 GUI/Tauri 权限则回交授权清单，不自行执行。
- 本轮开发线只负责 UI / 读模型产品化和前端验证。

建议验证线 / 主管线后续按 H6 任务包截图清单执行：

```text
evidence/tauri-verification/2026-06-08-stage-h-h6/
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

## 剩余风险

- 未进行真实 Tauri 截图，不能声明 H6 Tauri acceptance 完成。
- H5 后端 preview command 已存在，但本轮未新增 H5 preview 调用入口；项目侧栏先用既有 workflow dispatch / attempt / task memory / report 读模型做合并摘要。
- 真实执行类旧按钮仍存在于既有工作流控制区；本轮只加强权限弹层和摘要边界，不执行、不授权。

## 需要主管复核

需要。

复核重点：

- 是否接受“取消自动 transcript 读取 + 手动重新读取”为补齐 G3-B 智能体页安全截图路径。
- H6 合并摘要是否足以覆盖智能体页与项目工作流页的真实执行状态表达。
- 验证线是否获得 GUI/Tauri 权限后补 H6 截图清单。
