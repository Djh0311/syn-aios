# Stage H / H2.8 Real Execution Permission Dialog, Audit Summary, And Readiness Decision Surface v1

日期：2026-06-07

状态：已完成；非真实执行任务。  
用途：在 H2 Phase B 仍缺 existing target session、H3-B 仍未获 final approval 的前提下，补齐真实 resume 执行前的权限弹层、审计摘要和 readiness 决策面，降低后续真实 `codex exec resume` 授权时靠人工口头解释的风险。H2.8 完成不授权真实执行，不创建 fixture，不发送 prompt，不读写用户 Codex 会话数据，不满足 H2 Phase B，也不替代 H3-B final approval。

## 1. 权威依据

本任务包依据：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`
- `docs/workbench-system-architecture-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `tasks/2026-06-07-stage-h-h0-safety-boundary-and-task-package-freeze-v1.md`
- `tasks/2026-06-07-stage-h-h1-codex-local-runner-architecture-and-data-contract-v1.md`
- `tasks/2026-06-07-stage-h-h2-general-real-resume-productization-v1.md`
- `tasks/2026-06-07-stage-h-h2-7-phase-b-authorization-fixture-and-target-session-confirmation-v1.md`
- `tasks/2026-06-07-stage-h-h3-b-real-new-session-final-approval-and-fixture-run-v1.md`
- `evidence/2026-06-07-stage-h-h2-7-phase-b-authorization-fixture-and-target-session-confirmation-v1.md`
- `evidence/2026-06-07-stage-h-h3-b-task-package-creation-and-authority-sync-review-v1.md`
- `handoffs/2026-06-07-stage-h-h2-7-phase-b-authorization-fixture-and-target-session-confirmation-v1-result.md`
- `handoffs/2026-06-07-stage-h-h3-b-task-package-creation-and-authority-sync-review-v1-result.md`

## 2. 当前事实

- H2.5 Phase A 已完成非执行产品路径；Phase B 未授权。
- H2.7 已冻结当前阻断：

```text
h2_phase_b_readiness = blocked_waiting_target_session
phase_b_authorization_request = not_ready
```

- H3-B 任务包已创建并完成创建复核，但未授权、未执行。
- 当前没有用户 / 全局主管明确指定的 H2 existing target session。
- 当前没有已确认的 H2 fixture、permission envelope、allowed write roots、prompt summary/ref/hash、`.codex` 最小读写范围、readback plan、runtime log / audit / rollback 执行包。
- 当前不能进入真实 `codex exec resume`、真实 `codex exec` 新会话、H5 项目工作流真实派发或 planned adapters 真实执行。

## 3. H2.8 目标

H2.8 目标：

- 将 H2 执行前授权矩阵、H2.7 阻断、H1 guard、G1 runtime log、G2 diagnostics 和 E6 runtime attention 收敛成用户能看懂的 readiness 决策面。
- 补齐真实 resume 前的权限弹层内容要求，确保用户看到“做什么、为什么、影响哪里、谁提出、批准后会发生什么、失败如何处理”。
- 补齐审计摘要和 runtime log 引用预览，让执行前就能说明会写哪些 audit / runtime log / readback 状态。
- 明确 readback unavailable / failed / timed out 与真实 0 条结果的显示边界。
- 明确 duplicate queued/running attempt、guard blocked、missing target session、missing fixture、missing permission envelope、missing rollback 的阻断显示。
- 给 H2 Phase B final approval 提供结构化决策材料，但不替代用户 / 全局主管 final approval。

H2.8 不目标：

- 不执行真实 `codex exec resume`。
- 不执行真实 `codex exec`。
- 不发送真实 prompt。
- 不创建或使用 H2 / H3 fixture。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 auth、token、secret、`.env`、keychain、OAuth、provider credential 或完整 transcript / rollout。
- 不实现 H3 通用真实 send / 新会话。
- 不实现 H4 自动重试、cancel / stop、timeout 执行策略或完整 duplicate guard 产品化。
- 不做 H5 项目工作流真实派发。
- 不接入 planned adapters 真实执行。

## 4. 实现范围

### 4.1 Readiness 决策面

应新增或收敛一个只读 readiness summary，来源必须是工作台自有状态和既有 sidecar / read model，而不是读取 `.codex`：

- `operation = resume`
- `adapter = codex-local`
- `authorization_status`
- `target_session_status`
- `fixture_status`
- `permission_envelope_status`
- `allowed_write_roots_status`
- `prompt_envelope_status`
- `codex_home_scope_status`
- `readback_plan_status`
- `runtime_log_status`
- `audit_status`
- `rollback_status`
- `duplicate_attempt_status`
- `guard_status`
- `diagnostic_status`

推荐状态值：

```text
ready_for_final_approval
blocked_waiting_target_session
blocked_waiting_fixture
blocked_waiting_permission_envelope
blocked_waiting_allowed_write_roots
blocked_waiting_prompt_envelope
blocked_waiting_codex_home_scope
blocked_waiting_readback_plan
blocked_waiting_runtime_log
blocked_waiting_audit
blocked_waiting_rollback
blocked_by_guard
blocked_by_duplicate_attempt
blocked_by_diagnostics
ready_but_not_authorized
```

如果任一必需项缺失，只能显示 blocked / needs user，不得显示可执行。

### 4.2 权限弹层

如果改 `PermissionDialog` 或等价确认弹层，必须展示：

- 操作类型：`codex-local resume`。
- 目标项目、workflow、node、work item。
- target session 脱敏摘要。
- project root、target cwd、allowed write roots。
- denied paths。
- prompt summary、prompt ref、prompt hash；完整 prompt 不展示。
- 任务记忆包摘要：included / excluded / review materials / lint blocking。
- `/Users/yoyi/.codex` 最小副作用说明。
- sandbox、timeout、duplicate guard。
- readback plan 和 readback 不可信时的显示方式。
- runtime log refs / audit refs 将如何写。
- rollback / cleanup / diff plan。
- 批准、拒绝和 blocked 后分别发生什么。

禁止显示：

- 自由聊天输入框。
- 裸“执行 / 发送 / resume”按钮绕过权限。
- 未授权时显示“Codex 已收到任务”。
- readback unavailable / failed / timed out 显示为“结果 0 条”。
- 完整 prompt、raw transcript、raw stdout/stderr、secret、token、auth、`.env`、keychain、OAuth、provider credential。

### 4.3 审计摘要和 runtime log 预览

必须明确区分 audit 与 runtime log：

- audit 记录用户确认、拒绝、guard blocked、执行开始 / 结束 / 失败的决策事实。
- runtime log 记录脱敏运行过程、duration、exit code、timeout、failure category 和 source refs。
- readback result 独立记录可信状态，不被 audit 或 runtime log 替代。

H2.8 可以预览将写入的 audit / runtime log / readback 类型，但不能实际写真实执行记录。

### 4.4 UI 位置

推荐只在既有位置补局部摘要：

- 智能体页：H2 resume readiness / permission preview。
- 右侧待办：需要用户补 target session / fixture / permission envelope 的事项。
- 右侧运行中：只能显示 waiting authorization / blocked readiness，不显示 running。
- 管理入口：只显示 runtime log / audit 边界摘要，不铺 raw log。

不得新增一级入口、主导航、右侧顶级入口或自由会话控制中心。

## 5. UI 显示边界确认

本任务预计涉及 UI：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [x] 可能改前端类型 / Tauri wrapper。
- [x] 可能改读模型摘要或状态显示。
- [x] 可能改已有页面局部 UI。
- [ ] 不新增一级入口、主导航或右侧入口。

必须遵守：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`

如执行本任务时改可见 UI，必须补：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`

如涉及真实 Tauri 检查，必须明确截图证据；普通浏览器 smoke 不能冒充真实 Tauri 验收。

## 6. 验收要求

任务完成后必须证明：

- H2 readiness 决策面能明确区分 blocked、needs user、ready but not authorized。
- 缺 target session 时仍显示 `blocked_waiting_target_session`，不能显示 ready。
- 缺 fixture / permission envelope / prompt envelope / `.codex` 最小范围 / readback plan / runtime log / audit / rollback 时均不能显示可执行。
- 权限弹层或等价确认摘要覆盖目标、风险、写入范围、prompt 摘要、任务记忆包、readback、runtime log、audit 和 rollback。
- readback unavailable / failed / timed out 不显示为真实 0 条。
- duplicate queued/running attempt 会阻断 final approval。
- planned adapters 仍是 planned / unavailable / blocked。
- 秘书只解释风险和查看建议，不批准、不执行、不重试、不确认 worker 汇报。

建议验证命令：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib session_continuation
cargo test --lib codex_local_runner
cargo test --lib runtime_log
cargo test --lib runtime_session_attention
cargo test --lib workflow_authorization
cargo test --lib
rustfmt --check ...
```

如果本任务实际只写前端 / 读模型，不改 Rust，可在 evidence 中说明未跑 Rust 定向测试的理由；但不能省略与改动相关的测试。

## 7. 扫描要求

完成后必须扫描：

```text
rg -n -F 'Codex 已收到任务' prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/src-tauri/src
rg -n -F '真实 resume 已执行' prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/src-tauri/src
rg -n -F 'prompt 已发送' prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/src-tauri/src
rg -n -F 'readback 0 条' prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/src-tauri/src
rg -n -F 'planned adapter 已接入' prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/src-tauri/src
```

命中必须分类；测试 fixture / forbidden phrase 常量可以保留，但产品完成态文案不能出现。

敏感路径 / 真实执行扫描必须使用固定字符串或安全引用，避免 shell command substitution：

```text
rg -n -F 'Command::new("codex")' prototypes/productized-desktop-shell/src-tauri/src
rg -n -F 'codex exec resume' prototypes/productized-desktop-shell/src-tauri/src prototypes/productized-desktop-shell/src
rg -n -F '.codex' prototypes/productized-desktop-shell/src-tauri/src prototypes/productized-desktop-shell/src
```

## 8. 接受范围

H2.8 完成后可接受为：

- H2 真实执行前 readiness 决策面完成。
- H2 权限弹层 / 审计摘要 / runtime log preview / readback 边界加固完成。
- H2 Phase B final approval 材料更可执行、更可审核。

H2.8 不接受为：

- H2 Phase B 已授权。
- 真实 `codex exec resume` 已执行。
- 真实 prompt 已发送。
- `/Users/yoyi/.codex` 已读写。
- H2 通用真实 resume 产品化完成。
- H3 通用真实 send / 新会话完成。
- H4 failure / timeout / cancel / retry 产品化完成。
- H5 项目工作流真实派发完成。
- 阶段 H 完成。

## 9. 回交要求

完成 H2.8 后必须新增：

- `evidence/2026-06-07-stage-h-h2-8-real-execution-permission-dialog-audit-summary-and-readiness-decision-surface-v1.md`
- `handoffs/2026-06-07-stage-h-h2-8-real-execution-permission-dialog-audit-summary-and-readiness-decision-surface-v1-result.md`

必须同步：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`

回交必须写清：

- 是否改产品代码 / UI。
- 是否执行真实 `codex exec` / `codex exec resume`。
- 是否发送 prompt。
- 是否读写 `/Users/yoyi/.codex`。
- readiness 当前是否仍 blocked，以及 blocked reason。
- H2.8 接受范围和不接受范围。
- 下一步是继续 H2.x 修补、请求 H2 Phase B final approval，还是转入 H3-B final approval。

## 10. 回收结果

H2.8 已完成并回收，记录见：

- `evidence/2026-06-07-stage-h-h2-8-real-execution-permission-dialog-audit-summary-and-readiness-decision-surface-v1.md`
- `handoffs/2026-06-07-stage-h-h2-8-real-execution-permission-dialog-audit-summary-and-readiness-decision-surface-v1-result.md`

实现接受为：

- H2 真实执行前 readiness 决策面完成。
- H2 权限弹层预览、审计摘要、runtime log preview、readback 边界和 duplicate guard 决策面加固完成。
- 智能体页和秘书只读提示能解释 final approval 前缺项，但不批准、不执行、不发送 prompt、不重试。

仍不接受为：

- H2 Phase B 已授权。
- 真实 `codex exec resume` 已执行。
- 真实 prompt 已发送。
- H2 通用真实 resume 产品化完成。
- H3-B 已授权或已执行。
- 阶段 H 完成。
