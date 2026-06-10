# Stage H / H2.3 Real Resume Request Builder And Codex Local Guard Bridge v1

日期：2026-06-07

状态：已完成。

用途：把 H2 真实 resume 授权矩阵完整时的 continuation，转换成 H1 `CodexLocalExecutionRequest`，并接入 H1 `inspect_codex_local_execution_guard` 做执行前 guard 复核。H2.3 仍然不调用真实 runner，不执行 `codex exec resume`，不发送 prompt，不读写 `/Users/yoyi/.codex`，不授权 H3。

## 1. 权威依据

本任务包依据：

- `tasks/2026-06-07-stage-h-h2-general-real-resume-productization-v1.md`
- `tasks/2026-06-07-stage-h-h2-1-real-resume-authorization-matrix-and-execution-decision-freeze-v1.md`
- `tasks/2026-06-07-stage-h-h2-2-real-resume-authorization-readiness-read-model-and-readonly-ui-v1.md`
- `tasks/2026-06-07-stage-h-h1-codex-local-runner-architecture-and-data-contract-v1.md`
- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`

## 2. 当前事实

- H2 通用真实 resume 产品化任务包已创建，但仍待执行前授权。
- H2.0 已完成执行前授权矩阵 guard，授权完整时也只返回 `complete_but_not_executed`。
- H2.1 已冻结执行前授权矩阵和主管决策材料。
- H2.2 已把授权准备状态展示成智能体页只读 UI。
- H1 已具备 `CodexLocalExecutionRequest`、结构化 argv plan、prompt stdin ref/hash 边界和 `codex-local` guard。
- H2.3 的目标是在 H2 授权矩阵和 H1 runner contract 之间补桥，不进入真实执行。

## 3. 接受范围

H2.3 接受为：

- H2 授权矩阵完整时构建 H1 `CodexLocalExecutionRequest`。
- H2 预检输出携带 `codex_local_request` 和 `codex_local_guard`。
- H2 预检接入 H1 `inspect_codex_local_execution_guard`，guard 阻断时返回 `blocked_by_codex_local_guard`。
- 单测覆盖 incomplete matrix 不构建 request / guard，complete matrix 构建 request / guard 但仍不执行。
- 权威入口同步到 H2.3 已完成、H2 真实 resume 仍待任务包级明确授权。

H2.3 不接受为：

- H2 通用真实 resume 产品化完成。
- 真实 `codex exec resume` 已执行。
- prompt 已发送。
- `/Users/yoyi/.codex` 已读写。
- H3 通用真实 send / 新会话可开始。
- 项目工作流真实派发、planned adapters 真实接入或 provider credential / model verification 完成。

## UI 显示边界确认

本任务是否改前端：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [x] 改前端类型 / Tauri wrapper，但不新增可见 UI。
- [ ] 改读模型摘要或状态显示。
- [ ] 改已有页面局部 UI。
- [ ] 新增入口、面板、tab、按钮或确认动作。

已读取：

- `docs/workbench-frontend-display-boundary-v1.md`
- `CURRENT.md`
- `tasks/README.md`

本任务允许显示 / 暴露：

- H2 preflight 输出里的 `codex_local_request` 摘要。
- H2 preflight 输出里的 `codex_local_guard` 摘要。
- `complete_but_not_executed`、`blocked_by_codex_local_guard`、`blocked_waiting_authorization`。
- `h2_request_builder_only`、`codex_local_guard_only_no_runner_call`、`prompt_not_sent`、`codex_home_not_touched` 等非执行边界 warning。

本任务禁止显示：

- 执行、发送、resume、确认、授权、重试等新按钮。
- “Codex 已收到任务”“真实 Codex 已执行”“prompt 已发送”“`.codex` 已读写”“H2 已完成”“H3 可开始”“readback 0 条”等误导完成态。
- planned adapters 已接入、provider credential 已验证或真实项目工作流已派发。

显示位置：

- 一级入口：不新增。
- 右侧入口：不新增。
- 项目页：不改。
- 画布：不改。
- 记忆入口：不改。
- 知识库入口：不改。
- 智能体入口：不新增可见面板，本轮只扩展 H2 preflight command / TS 类型输出。
- 管理入口：不改。

中间版本范围：

- 本轮必须落地：H2 request builder、H1 guard bridge、输出类型、单测和权威入口同步。
- 本轮只做读模型 / 摘要：否，本轮改后端预检逻辑和输出类型，但仍是非执行 preflight。
- 本轮后置：真实 H2 resume、H3 send、新会话、项目工作流真实派发、真实 Tauri 截图验收。

后端和数据依赖：

- 需要后端正式读模型：需要，H2 preflight 输出需要携带 request / guard。
- 需要审计 / 日志 / 权限 / 状态机：沿用 H2.0 `SessionContinuationStore` attempt / audit，新增 guard 结果但不写 runtime log。
- 不能用假数据伪装：不能把 command plan、guard allowed 或 dry-run allowed 写成真实 runner 已执行。

UI 文案边界：

- 禁止说：真实 resume 已执行、prompt 已发送、`.codex` 已读写、H2 已完成、H3 可开始、readback 0 条。
- 允许说：H2 request builder、CodexLocal guard bridge、complete but not executed、guard only no runner call。

验收：

- Rust 格式检查：`rustfmt --check src/session_continuation_store.rs src/types.rs`
- Rust 定向测试：`cargo test --lib session_continuation`
- Rust 定向测试：`cargo test --lib codex_local`
- 前端类型检查：`npm run typecheck`
- 真实窗口 / 截图验收：不适用，本轮无可见 UI 改动。
- 未验收项必须写入 evidence / handoff：是。

## 4. 实现要求

- `InspectControlledSessionContinuationRealResumeOutput` 增加 `codex_local_request` 和 `codex_local_guard` 可选输出。
- `inspect_real_resume_authorization` 在授权矩阵完整时构建 H1 `CodexLocalExecutionRequest`。
- request 必须使用结构化字段表达 operation、session、cwd、allowed write roots、sandbox、prompt hash/ref、readback plan、runtime/audit refs，不把 prompt 放入 shell 命令。
- `inspect_real_resume_authorization` 必须调用 H1 guard inspection，不调用真实 runner。
- guard 阻断时，H2 preflight 状态必须进入 `blocked_by_codex_local_guard`，并把 guard reason 合并进 `missing_or_invalid_items`。
- 授权矩阵不完整时，必须不构建 `codex_local_request` / `codex_local_guard`。
- 授权矩阵完整且 guard 允许时，只能返回 `complete_but_not_executed`，不能执行真实 resume。
- 测试必须覆盖：
  - incomplete matrix 不构建 request / guard。
  - complete matrix 构建 request / guard。
  - attempt 仍为 `h2_real_resume_preflight_no_execution`。
  - `prompt_sent=false`。
  - `real_codex_executed=false`。
  - `writes_codex_home=false`。
  - command plan 使用 `program="codex"`、结构化 argv、非 shell invocation。
  - command argv 不包含 prompt hash 或 prompt body。

## 5. 回交要求

完成后必须新增：

- `evidence/2026-06-07-stage-h-h2-3-real-resume-request-builder-and-codex-local-guard-bridge-v1.md`
- `handoffs/2026-06-07-stage-h-h2-3-real-resume-request-builder-and-codex-local-guard-bridge-v1-result.md`

并同步：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`

## 6. 停止条件

出现以下任一情况必须停止并回交阻断：

- 需要执行真实 `codex exec` / `codex exec resume`。
- 需要发送真实 prompt。
- 需要读写 `/Users/yoyi/.codex`。
- 需要读取 auth/token/.env/secret/keychain/OAuth/provider credential/full transcript。
- 需要新增可见执行按钮或 UI 操作入口。
- H1 guard 未接入或 guard reason 无法追踪。
