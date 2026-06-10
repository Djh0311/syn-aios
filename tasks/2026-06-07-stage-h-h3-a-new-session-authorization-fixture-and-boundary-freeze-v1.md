# Stage H / H3-A New Session Authorization, Fixture, And Boundary Freeze v1

日期：2026-06-07

状态：已完成；不授权真实 `codex exec` / `codex exec resume`。  
用途：在 H2 Phase B 因缺 existing target session 阻断后，单独准备 H3 通用真实 send / 新会话产品化的第一段任务。H3-A 只冻结新会话授权、fixture、guard、权限、UI、记忆、运行日志、审计和 readback 边界；不创建真实 Codex session，不发送 prompt，不读写 `/Users/yoyi/.codex`。

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
- `tasks/2026-06-07-stage-h-h2-7-phase-b-authorization-fixture-and-target-session-confirmation-v1.md`
- `evidence/2026-06-07-stage-h-h2-7-phase-b-authorization-fixture-and-target-session-confirmation-v1.md`
- `handoffs/2026-06-07-stage-h-h2-7-phase-b-authorization-fixture-and-target-session-confirmation-v1-result.md`

## 2. 当前事实

- H0 已完成阶段 H 安全边界和任务包冻结，并已通过全局主管复核。
- H1 已完成 `CodexLocalRunner` 架构和数据契约，并已通过全局主管复核。
- H2.7 已完成为 H2 Phase B 授权准备复核和阻断状态冻结。
- H2 Phase B 当前结论仍是：

```text
h2_phase_b_readiness = blocked_waiting_target_session
phase_b_authorization_request = not_ready
```

- H2 Phase B 缺 existing target session，fixture 和 permission envelope 仍是 secondary blockers。
- H2 Phase B 真实 `codex exec resume` 未授权，prompt 未发送，`/Users/yoyi/.codex` 未读写。
- H3 不能被解释为 H2 Phase B 已满足，也不能用新会话绕过 H2 的 existing target session 阻断。
- 两条只读分线已经给出主管参考：
  - H3 设计线建议可以拆 H3，但第一段必须是非执行准备 / 授权冻结。
  - H3 安全线建议先做 H3-A，只读冻结 guard、权限、审计、记忆和 UI 边界，再另拆 H3-B 真实新会话。

## 3. H3-A 目标

H3-A 目标：

- 冻结 H3 通用真实 send / 新会话的产品边界。
- 定义新会话 request / send request 的最小数据结构和授权矩阵。
- 明确项目绑定、角色绑定、任务包绑定、workflow node / work item 绑定和 authorization scope。
- 冻结隔离 fixture 原则、target cwd、project root、allowed write roots、denied paths、sandbox、timeout 和 rollback。
- 冻结 `.codex` 最小读写范围，但不在 H3-A 读写 `.codex`。
- 冻结 prompt summary / ref / hash 规则，完整 prompt 不进入 argv、shell string、runtime log、audit、evidence 或 handoff。
- 冻结 task memory packet included / excluded / review materials 的执行前展示和审计要求。
- 冻结 continuation record、runtime log sidecar、audit event、readback result、failure reason、duplicate guard 和 evidence / handoff 路径要求。
- 明确 H3-B 单次真实新会话执行的 final approval 前置条件。

H3-A 不目标：

- 不执行真实 `codex exec`。
- 不执行真实 `codex exec resume`。
- 不发送真实 prompt。
- 不创建真实 Codex session。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 auth、token、secret、`.env`、keychain、OAuth、provider credential 或完整 transcript / rollout。
- 不启动 Tauri / GUI / 截图。
- 不新增自由聊天输入框、裸 send 按钮或裸 resume 按钮。
- 不接入 Claude Code / OpenClaw / OpenCode / OpenCode-like planned adapters 的真实执行。
- 不做 provider credential store、model verification、自动重试、自动恢复或跨 provider 调度。
- 不把 H3-A 当作 H2 Phase B 已授权、H2 完成、H3 完成或阶段 H 完成。

## 3.1 执行结论

H3-A 已按文档 / 授权冻结任务完成。

本轮结论：

```text
h3_a_readiness = completed_boundary_freeze
h3_b_readiness = blocked_waiting_fixture_and_permission_envelope
real_codex_execution = not_authorized
codex_home_access = not_authorized
```

判断：

- H3-A 可以接受为 H3 通用真实 send / 新会话的授权冻结完成。
- H3-B 仍不能执行，原因是 H3 fixture、permission envelope、prompt envelope、`.codex` 最小范围、allowed write roots、readback、runtime log、audit、evidence 和 rollback 仍未获得 final approval。
- H3-A 不满足 H2 Phase B existing target session，不改变 H2 `blocked_waiting_target_session` 结论。
- 真实 `codex exec` 新会话必须另拆 H3-B final approval / real new session fixture run 任务。

## 4. 必须冻结的数据契约

H3-A 需要设计或复核以下契约；可以复用 H1 类型，但必须清楚区分 `new_session` / `send_message` 和 H2 `resume`：

- `CodexLocalNewSessionRequest` 或等价 request。
- `CodexLocalSendMessageRequest` 或等价 request。
- `CodexLocalNewSessionGuard` 或等价 guard 输出。
- `CodexLocalSessionBinding`：工作台自有 session binding，不把 Codex thread id 当永久业务主键。
- `CodexLocalPromptEnvelope`：只保存 prompt summary / ref / hash，不把完整 prompt 放进命令字符串和证据文档。
- `CodexLocalPermissionEnvelope`：展示做什么、为什么、影响范围、风险、写入路径、`.codex` 范围、审计和回滚。
- `CodexLocalExecutionAttempt`：继续使用 H1/H2 的 attempt / runtime log / audit / readback 分离边界。
- `CodexLocalReadbackResult`：`readback_unavailable` / `readback_failed` / `timed_out` 的 `result_count` 必须保持 `null`，不能显示成真实 0 条。

## 5. Guard 和权限矩阵

H3-A 必须冻结 guard 条件：

- adapter 必须是 `codex-local`。
- operation 必须是 `new_session` 或受控 `send_message`；不得混同 H2 `resume`。
- 必须绑定 project / workflow / node / work item / role / task package。
- 必须绑定授权范围；缺 authorization scope 时阻断。
- 必须绑定 task memory packet；缺 included / excluded / review materials 摘要时阻断或进入 review。
- 必须提供 prompt summary / ref / hash；缺任一项时阻断。
- `project_root`、`target_cwd`、`allowed_write_roots` 必须是绝对路径，且不能包含 `..` 逃逸。
- allowed write roots 默认只能指向隔离 fixture；真实业务项目必须另行高风险授权。
- denied paths 必须覆盖 secret、auth、token、`.env`、keychain、OAuth、provider credential、完整 transcript / rollout 和用户未授权真实业务目录。
- duplicate running attempt 必须阻断。
- planned adapters 必须保持 planned / unavailable，不得由 H3-A 变成可执行。
- 缺任一关键项时输出 `blocked_waiting_authorization`、`blocked_waiting_fixture`、`blocked_waiting_permission_envelope` 或等价状态。

## 6. H3-B 前置条件

只有 H3-A 完成并通过全局主管复核后，才允许另拆 H3-B。

H3-B 单次真实新会话执行前必须由用户 / 全局主管明确确认：

1. 是否创建或使用 H3 隔离 fixture。
2. fixture 路径、初始文件、执行前 hash、执行后 diff、rollback / cleanup 规则。
3. 是否允许真实 `codex exec` 创建新 session。
4. 是否允许 Codex CLI 新会话必需的 `/Users/yoyi/.codex` 最小读写范围。
5. project root、target cwd、allowed write roots 是否全部限制在 fixture 内。
6. prompt summary / ref / hash 是什么；完整 prompt 如何避免进入 argv、shell string、runtime log、audit、evidence 或 handoff。
7. sandbox、timeout、readback plan、runtime log、audit、evidence / handoff 路径。
8. 如果 execution failed、timed out、readback unavailable、readback failed、guard blocked 或 duplicate blocked，是否停止在 H3.x 修补。
9. H3-B 结果不能回填为 H2 Phase B target session 已满足，除非后续另有 H2 final approval 任务明确使用该 session。

## 7. UI 显示边界确认

本任务默认不改 UI：

- [x] 不改前端、不改读模型、不改 UI 文案。
- [ ] 改前端类型 / Tauri wrapper，但不新增可见 UI。
- [ ] 改读模型摘要或状态显示。
- [ ] 改已有页面局部 UI。
- [ ] 新增一级入口、主导航、右侧入口、面板、tab、按钮或确认动作。

已确认必须遵守：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`

若 H3-A 执行时涉及 UI 或读模型，必须先修订本章节并补充：

- 智能体页只能显示 readiness、权限说明、状态摘要、readback / failure / audit 引用。
- 不能新增裸执行按钮、自由聊天输入框或未授权 send/resume 入口。
- 权限弹层必须用用户能理解的语言说明操作、影响范围、风险、写入路径、`.codex` 范围、prompt 摘要、审计和回滚。
- 审计和日志进入管理入口；通知、待办、运行中不能混成一个列表。
- 秘书不得批准权限、派发任务或确认 worker 汇报。
- 任何可见 UI 变化都必须按 UI 规则补离线测试；如涉及导航、页面、画布或确认弹层，必须安排真实窗口 / 截图验收或明确 incomplete。

## 8. 验收要求

H3-A 默认是文档 / 授权冻结任务，不要求运行 `npm` / `cargo`，除非执行时修改产品代码。

必须完成：

- 读取 H0 / H1 / H2.7 / H-I 计划和当前权威入口。
- 生成 H3 新会话 / send 授权冻结结论。
- 明确 H3 与 H2 Phase B 的边界：H3 不能满足 H2 target session，不能绕过 H2 final approval。
- 明确 fixture、permission envelope、prompt envelope、readback、runtime log、audit、evidence、rollback 和 `.codex` 最小范围的确认状态。
- 明确 H3-B 是否具备 final approval 条件；如果缺条件，必须阻断。
- 新增 H3-A evidence / handoff，或在本任务未执行时保持只创建任务包。
- 同步 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`、`docs/plans/README.md`、`docs/plans/middleware-version-stage-plan-v1.md` 和 H-I 阶段计划。

必须扫描确认：

- 不出现 H3 已完成、真实新会话已创建、真实 send 已执行、prompt 已发送、`.codex` 已读写、H2 已完成、H2 Phase B 已满足、planned adapters 已接入等误导口径。
- 高风险词若出现，只能出现在禁止项、待授权项、历史 evidence 引用或“不接受为”语境中。

如果 H3-A 执行时改产品代码，至少补：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- 相关 Rust 定向测试和 `cargo test --lib`
- 指定 `rustfmt --check`

## 9. 接受范围

H3-A 完成后可接受为：

- H3 通用真实 send / 新会话的授权冻结完成。
- 新会话 request / send request 的最小契约和 guard 条件完成。
- H3-B 单次真实新会话 fixture run 的前置条件、授权矩阵和阻断状态明确。
- H3 与 H2 Phase B 的边界明确：H3 不自动补齐 H2 existing target session。

H3-A 不接受为：

- H3 通用真实 send / 新会话产品化完成。
- 真实 `codex exec` 已执行。
- 真实 Codex session 已创建。
- prompt 已发送。
- `/Users/yoyi/.codex` 已读写。
- H2 Phase B target session 已满足。
- H2 通用真实 resume 产品化完成。
- 项目工作流真实派发完成。
- planned adapters 真实接入。
- provider credential / model verification 完成。
- 阶段 H 完成。

## 10. 下一步

H3-A 执行后：

- 如果输出 `h3_b_readiness = authorization_ready_waiting_final_user_approval`，下一步只能另开 H3-B final approval / real new session fixture run 任务，并在执行点再次请求用户 / 全局主管明确授权。
- 如果输出 `blocked_waiting_fixture`、`blocked_waiting_permission_envelope`、`blocked_waiting_prompt_envelope`、`blocked_waiting_task_binding` 或等价状态，必须先补对应缺口，不能真实创建新 session。
- H3-B 真实执行结果不能直接冒领 H2 Phase B；如需把 H3 新 session 用作后续 resume target，必须另拆 H2 final approval / real fixture run 任务。
