# Stage H / H2.2 Real Resume Authorization Readiness Read Model And Readonly UI v1

日期：2026-06-07

状态：已完成。

用途：把 H2 真实 resume 执行前授权矩阵变成产品内只读可见的 readiness 面板，让用户和全局主管能看到哪些字段已确认、哪些字段仍缺失。H2.2 不执行真实 `codex exec resume`，不发送 prompt，不读写 `/Users/yoyi/.codex`，不授权 H3。

## 1. 权威依据

本任务包依据：

- `tasks/2026-06-07-stage-h-h2-general-real-resume-productization-v1.md`
- `tasks/2026-06-07-stage-h-h2-1-real-resume-authorization-matrix-and-execution-decision-freeze-v1.md`
- `evidence/2026-06-07-stage-h-h2-1-real-resume-authorization-matrix-and-execution-decision-freeze-v1.md`
- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`

## 2. 当前事实

- H2 通用真实 resume 产品化任务包已创建，但仍待执行前授权。
- H2.0 已完成执行前 guard，授权矩阵完整时也只返回 `complete_but_not_executed`，不调用真实 runner。
- H2.1 已完成授权矩阵和决策工作表，但它是文档材料，不在产品 UI 中可见。
- H2 真实执行前仍必须确认测试项目、目标 session、`.codex` 最小范围、prompt hash/ref、allowed write roots、readback plan、runtime log、audit、evidence 和 rollback。

## 3. 接受范围

H2.2 接受为：

- H2 real resume 授权准备读模型完成。
- 智能体页只读展示授权矩阵 readiness 面板。
- 离线测试覆盖 readiness 状态、缺失项、禁止误导文案和无执行按钮。
- 权威入口同步到 H2.2 已完成、H2 仍待真实执行授权。

H2.2 不接受为：

- H2 通用真实 resume 产品化完成。
- 真实 `codex exec resume` 已执行。
- prompt 已发送。
- `/Users/yoyi/.codex` 已读写。
- H3 通用真实 send / 新会话可开始。
- 项目工作流真实派发、planned adapters 真实接入或 provider credential / model verification 完成。

## UI 显示边界确认

本任务是否改前端：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [ ] 改前端类型 / Tauri wrapper，但不新增可见 UI。
- [x] 改读模型摘要或状态显示。
- [x] 改已有页面局部 UI。
- [ ] 新增入口、面板、tab、按钮或确认动作。

已读取：

- `docs/workbench-frontend-display-boundary-v1.md`
- `CURRENT.md`
- `tasks/README.md`

本任务允许显示：

- H2 real resume 授权准备状态。
- `blocked_waiting_authorization` / `ready_for_explicit_authorization` readiness 状态。
- target session、project root、target cwd、allowed write roots、prompt summary、prompt hash/ref、`.codex` 最小范围、sandbox、timeout、readback plan、runtime/audit/evidence、rollback、用户确认和全局主管确认的确认 / 缺失状态。
- 明确提示“不会发送 prompt / 不会执行 codex exec resume / 不会读写 /Users/yoyi/.codex”。

本任务禁止显示：

- 执行、发送、resume、确认、授权、重试等按钮。
- “Codex 已收到任务”“真实 Codex 已执行”“prompt 已发送”“`.codex` 已读写”“H2 已完成”“H3 可开始”“readback 0 条”等误导完成态。
- planned adapters 已接入、provider credential 已验证或真实项目工作流已派发。

显示位置：

- 一级入口：不新增。
- 右侧入口：不新增。
- 项目页：不改。
- 画布：不改。
- 记忆入口：不改。
- 知识库入口：不改。
- 智能体入口：在既有智能体页会话 / continuation 边界区域显示只读 H2 授权准备面板。
- 管理入口：不改。

中间版本范围：

- 本轮必须落地：H2 readiness 读模型、只读 UI、离线测试和权威入口同步。
- 本轮只做读模型 / 摘要：是。
- 本轮后置：真实 H2 resume、H3 send、新会话、项目工作流真实派发、真实 Tauri 截图验收。

后端和数据依赖：

- 需要后端正式读模型：不需要，本轮前端从 `session_continuation_previews` 和 `session_continuation_store` 派生只读 readiness。
- 需要审计 / 日志 / 权限 / 状态机：只显示需求，不写审计 / 日志 / 权限状态。
- 不能用假数据伪装：不能把推荐 fixture、缺失项或 preview 写成已确认授权。

UI 文案边界：

- 禁止说：真实 resume 已执行、prompt 已发送、`.codex` 已读写、H2 已完成、H3 可开始、readback 0 条。
- 允许说：H2 real resume 授权准备、等待授权矩阵、不会发送 prompt、不会执行 codex exec resume、不会读写 `/Users/yoyi/.codex`。

验收：

- 类型检查：`npm run typecheck`
- 离线交互测试：`npm run test:offline-interaction`
- 构建：`npm run build`
- 真实窗口 / 截图验收：未完成。
- 未验收项必须写入 evidence / handoff：是。

## 4. 实现要求

- 新增前端只读读模型 helper，从 `SessionContinuationPreview[]` 和 `SessionContinuationStoreV1 | null` 派生 `H2RealResumeAuthorizationReadiness`。
- 新增 TS 类型，明确 readiness item、status、summary、counts 和 warnings。
- 智能体页新增只读 H2 授权准备面板，不新增按钮。
- 样式必须保持局部展示，不改变一级导航或右侧入口。
- 离线测试必须覆盖：
  - 默认状态为 `blocked_waiting_authorization`。
  - prompt hash/ref、`.codex` 最小范围、用户确认、全局主管确认等关键项缺失。
  - UI 显示非执行边界。
  - UI 不出现误导完成态。
  - 不渲染执行 / 发送 / resume / 确认 / 授权 / 重试按钮。

## 5. 回交要求

完成后必须新增：

- `evidence/2026-06-07-stage-h-h2-2-real-resume-authorization-readiness-read-model-and-readonly-ui-v1.md`
- `handoffs/2026-06-07-stage-h-h2-2-real-resume-authorization-readiness-read-model-and-readonly-ui-v1-result.md`

并同步：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`
