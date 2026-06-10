# Task Package: Stage G / G3-A Real Tauri Acceptance Plan And Fixture Freeze v1

状态：已完成。  
用途：冻结 G3 真实 Tauri 验收的范围、fixture、截图路径、手动清单、权限边界和降级规则。G3-A 只接受为验收计划和 fixture freeze，不接受为真实 Tauri 已启动、截图已采集、G3 整体完成、G4 回放或 G5 最终冻结。

## 0. 先说薄弱点

- 普通浏览器 smoke、Vite HTTP、DOM 检查不能替代真实 Tauri 窗口验收。
- 截图容易误截 Codex 主窗口或普通浏览器窗口，必须记录窗口标题、路径和步骤。
- 权限弹层 fixture 如果不稳定，不能为了截图触发真实 Codex 或敏感写入。
- G3-B 可能需要启动 Tauri、端口检查、截图权限或进程清理；必须在执行前明确授权。

## 1. 已知事实 / 未知 / 假设

已知事实：

- G1 Runtime Log Boundary And Minimal Store 已完成。
- G2 Diagnostics Health And Degraded State 已完成。
- UI 显示边界规则要求涉及布局、导航、右侧入口、画布、项目页、智能体页、记忆页或秘书入口的验收必须有真实窗口或浏览器截图证据；G3 的目标是补真实 Tauri。

未知：

- 当前机器是否允许启动真实 Tauri。
- macOS 截图权限和窗口定位是否稳定。
- 权限确认弹层是否有低风险 fixture 可稳定触发。

假设：

- G3-B 使用真实 Tauri 窗口作为主证据。
- 普通浏览器只可作为辅助，不可替代 G3。
- G3-B 不执行真实 `codex exec` / `codex exec resume`，除非用户另行明确授权。

## 2. G3 拆分

G3 拆为三段：

- G3-A：验收计划和 fixture freeze。只写任务包 / evidence / handoff / 入口文档，不启动 Tauri。
- G3-B：真实 Tauri 手动截图验收。需要单独授权启动 Tauri、截图和必要端口检查 / 清理。
- G3-C：截图证据回收和缺口矩阵。整理截图、步骤、失败项、deferred 项，不包装失败为通过。

## 3. G3-B 截图路径

建议截图目录：

```text
evidence/tauri-verification/2026-06-07-stage-g-g3/
```

建议文件名：

- `01-permission-dialog.png`
- `02-projects.png`
- `03-project-workflow-canvas.png`
- `04-workflow-node-detail.png`
- `05-agent-session-center.png`
- `06-send-resume-boundary.png`
- `07-memory-center.png`
- `08-knowledge-base.png`
- `09-task-memory-packet-preview.png`
- `10-running.png`
- `11-notifications.png`
- `12-todos.png`
- `13-admin-runtime-log-diagnostics.png`

每张截图必须记录：

- 操作路径。
- 使用 fixture。
- 预期显示。
- 实际结果。
- 是否真实 Tauri。
- 是否存在未覆盖项。

## 4. 最小覆盖清单

G3-B 至少覆盖：

- 权限确认弹层：人话说明、风险、影响范围、提出者、允许一次 / 拒绝 / 查看详情；不能触发真实 Codex。
- 项目页：项目列表、项目详情、项目内工作流 / 智能体 / 文档 / 记忆 / 设置边界。
- 项目工作流画布：画布为主，节点显示摘要，不铺 raw transcript / schema / 完整日志。
- 节点详情：任务包摘要、任务记忆包、权限、readback、失败、audit / evidence / handoff 引用。
- 智能体：`codex-local` 可用边界、planned adapters 不可执行、operation boundary、send / resume preview / stub / readback unavailable。
- 记忆中心：正式记忆、候选、来源、版本、审计、lint/conflict、lifecycle 操作必须走确认。
- 知识库：资料和笔记空间，不直接写正式记忆，不和记忆中心混成一套。
- 任务记忆包预览：included / excluded / review materials、lint/blocking、召回理由；candidate / knowledge hit 不能说成正式记忆。
- 通知、待办、运行中：三者分开，不混成一个列表。
- 管理：runtime log、diagnostics、health、最近错误、数据位置；raw log/internal id 只进详情或开发者模式。

## 5. Fixture 边界

默认 fixture：

- 使用当前工作台自有受控 fixture / 现有项目数据。
- 权限弹层优先使用 preview / guard 类低风险路径。
- send / resume 使用 E5 Level A stub / preview / guard 状态。
- G2 diagnostics 使用 `WorkbenchSnapshot.diagnostic_summary` 只读摘要。
- G1 runtime log 使用 `WorkbenchSnapshot.runtime_log_store` 脱敏摘要。

禁止 fixture：

- 真实 `codex exec` / `codex exec resume`。
- 真实 prompt 发送。
- 读写 `/Users/yoyi/.codex`。
- 读取完整 transcript / rollout、auth、token、`.env`、secret、keychain、OAuth、provider credential。
- 为了截图写正式记忆、改 workflow state、写 observation、写 candidate 或写 runtime log。

## 6. 降级规则

如果 G3-B 遇到以下情况：

- Tauri 启动失败。
- 端口占用无法清理。
- macOS 截图权限不足。
- 窗口定位不稳定。
- 权限弹层 fixture 不稳定。
- 只能做普通浏览器 smoke。

必须记录为：

```text
真实窗口 / 截图验收未完成
```

可以保留命令输出、HTTP smoke 或普通浏览器 DOM 作为辅助证据，但不得回收 G3-B 为完成。

## 7. UI 显示边界确认

本任务是否改前端：

- [x] 不改前端、不改读模型、不改 UI 文案。
- [ ] 改前端类型 / Tauri wrapper，但不新增可见 UI。
- [ ] 改读模型摘要或状态显示。
- [ ] 改已有页面局部 UI。
- [ ] 新增入口、面板、tab、按钮或确认动作。

已读取 / 继承：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- G1 / G2 task, evidence, handoff

本任务允许：

- 冻结 G3-B 的截图清单、路径、fixture 和边界。
- 同步权威入口到 G3-B 待开始。

本任务禁止：

- 启动 Tauri。
- 截图。
- 声称真实 Tauri 验收完成。
- 声称 G4 / G5 / 阶段 G 完成。

## 8. 验收

本任务默认文档验收：

- 任务包存在。
- evidence / handoff 存在。
- 权威入口同步到 G3-A 已完成，G3-B 待开始。
- 旧口径扫描无 G3-A 待开始残留。
- 未出现 G3-B / G3 / G4 / G5 已完成冒领。

## 9. 下一步

下一步只能进入：

```text
G3-B Real Tauri Manual Screenshot Acceptance 待开始
```

G3-B 执行前必须确认是否授权启动真实 Tauri、截图和必要端口检查 / 清理。
