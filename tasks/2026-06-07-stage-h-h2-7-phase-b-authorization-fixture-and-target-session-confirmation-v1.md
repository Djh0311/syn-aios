# Stage H / H2.7 Phase B Authorization, Fixture, And Target Session Confirmation v1

日期：2026-06-07

状态：已完成；不授权真实 `codex exec` / `codex exec resume`。  
用途：在 H2.6 已补齐 runtime log 显式 sidecar writer 后，把 H2.5 Phase B 真实 resume 前仍缺的 fixture、existing target session、`.codex` 最小范围、prompt ref/hash、readback、runtime log、audit、evidence 和 rollback 逐项冻结为可审批材料。若缺 existing target session，本任务必须停在阻断态，不能直接执行 Phase B，也不能用 H3 新会话绕过 H2 Phase B。

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
- `tasks/2026-06-07-stage-h-h2-general-real-resume-productization-v1.md`
- `tasks/2026-06-07-stage-h-h2-5-real-resume-runner-execution-path-and-authorized-fixture-run-v1.md`
- `tasks/2026-06-07-stage-h-h2-6-phase-b-readiness-fixture-session-and-runtime-log-hardening-v1.md`
- `evidence/2026-06-07-stage-h-h2-6-phase-b-readiness-fixture-session-and-runtime-log-hardening-v1.md`
- `handoffs/2026-06-07-stage-h-h2-6-phase-b-readiness-fixture-session-and-runtime-log-hardening-v1-result.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`

## 2. 当前事实

- H2.5 Phase A 已完成，只接受为非执行 runner 产品路径、attempt / audit / readback 分类和 duplicate guard 完成。
- H2.6 已完成，只接受为 Phase B 前置条件复核、runtime log 显式 sidecar writer、损坏 runtime log 阻断和阻断状态冻结完成。
- 当前 readiness 仍是：

```text
h2_phase_b_readiness = blocked_waiting_fixture_and_target_session
runtime_log_writer = explicit_sidecar_writer_ready
phase_b_authorization_request = not_ready
```

- 推荐 fixture `/Users/yoyi/workspace/product-line/tmp/h2-real-resume-fixture` 仍未创建。
- 可用于 H2 Phase B 的 existing target session 尚未由用户 / 全局主管确认。
- 当前没有可作为 H2 Phase B 真实执行证据的 workbench continuation sidecar 实例。
- Phase B 真实 `codex exec resume`、prompt 发送和 `/Users/yoyi/.codex` 最小读写仍未授权。
- H2.7 已执行只读复核，最终输出 `h2_phase_b_readiness = blocked_waiting_target_session`；fixture 和 permission envelope 仍是 secondary blockers。

## 3. 目标

H2.7 目标：

- 生成 H2 Phase B 授权准备材料，而不是执行真实 resume。
- 确认或冻结 fixture 路径、fixture 初始文件、执行前 hash / 目录快照、allowed write roots、rollback / cleanup 方案。
- 确认 existing target session，或明确停在 `blocked_waiting_target_session`。
- 确认 target session 与 fixture / project root / cwd / workflow / node 的绑定方式。
- 确认 `.codex` 最小读写范围，且明确禁止读取 auth、token、secret、`.env`、keychain、OAuth、provider credential 和完整 transcript。
- 确认 prompt summary、prompt ref、prompt hash 生成规则，完整 prompt 不进入 argv、shell string、runtime log、audit、evidence 或 handoff。
- 确认 readback plan、runtime log plan、audit plan、evidence / handoff path、failure classification 和 rollback。
- 输出 Phase B authorization request 草案或阻断结论。

H2.7 不目标：

- 不执行真实 `codex exec`。
- 不执行真实 `codex exec resume`。
- 不发送真实 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 auth、token、secret、`.env`、keychain、OAuth、provider credential 或完整 transcript。
- 不启动 Tauri / GUI / 截图。
- 不做 H3 通用真实 send / 新会话。
- 不自动创建新 Codex session 来绕过 existing target session 缺失。
- 不做 H5 项目工作流真实派发。
- 不做 planned adapters 真实接入或 provider credential / model verification。

## 4. 必须确认的问题

执行 H2.7 时，用户 / 全局主管必须逐项确认或明确阻断：

1. 是否使用推荐 fixture：`/Users/yoyi/workspace/product-line/tmp/h2-real-resume-fixture`。
2. 是否允许在 `product-line/tmp` 下创建或使用该 fixture。
3. fixture 初始文件清单、执行前 hash / 目录快照和 rollback / cleanup 规则是什么。
4. existing target session 是哪个；是否明确授权用于本 fixture / workflow / node。
5. target cwd、project root、allowed write roots 是否全部限制在 fixture 内。
6. 是否授权后续 Phase B 单次真实 `codex exec resume`。
7. 是否授权 Codex CLI resume 必需的 `/Users/yoyi/.codex` 最小读写范围。
8. prompt summary / ref / hash 是什么；完整 prompt 如何避免进入 argv、shell string、runtime log、audit、evidence 或 handoff。
9. readback 使用哪个来源；readback unavailable / failed / timed out 是否保持 `result_count = null`。
10. runtime log、audit event、evidence、handoff 和 failure classification 写入哪里。
11. 如果 guard blocked、duplicate blocked、execution failed、timed out、readback failed 或 readback unavailable，是否停止在 H2.x 修补。
12. 若没有 existing target session，是否停在 H2.7 阻断，还是另拆 H3 通用真实 send / 新会话；H3 不能被解释为 H2 Phase B 已满足。

## 5. 决策输出

H2.7 必须产出以下之一。

### 5.1 授权准备就绪

只有所有必要项都被用户 / 全局主管明确确认时，才允许输出：

```text
h2_phase_b_readiness = authorization_ready_waiting_final_user_approval
phase_b_authorization_request = ready_for_final_approval
```

即便输出该状态，也仍不等于 Phase B 已执行。下一步必须单独发起 Phase B final approval / real fixture run 任务。

### 5.2 缺 fixture 阻断

如果 fixture 未确认或不允许创建 / 使用，输出：

```text
h2_phase_b_readiness = blocked_waiting_fixture
phase_b_authorization_request = not_ready
```

### 5.3 缺 existing target session 阻断

如果没有用户 / 全局主管明确提供 existing target session，输出：

```text
h2_phase_b_readiness = blocked_waiting_target_session
phase_b_authorization_request = not_ready
```

禁止读取 `/Users/yoyi/.codex` 搜索 session，禁止复用 E5 Level B `mario test` session 作为默认值，禁止自动创建新 session。

### 5.4 缺权限 envelope 阻断

如果 `.codex` 最小范围、allowed write roots、prompt hash/ref、readback、runtime log、audit、evidence 或 rollback 任一项未确认，输出：

```text
h2_phase_b_readiness = blocked_waiting_permission_envelope
phase_b_authorization_request = not_ready
```

## 6. UI 显示边界确认

本任务默认不改 UI：

- [x] 不改前端、不改读模型、不改 UI 文案。
- [ ] 改前端类型 / Tauri wrapper，但不新增可见 UI。
- [ ] 改读模型摘要或状态显示。
- [ ] 改已有页面局部 UI。
- [ ] 新增一级入口、主导航、右侧入口、面板、tab、按钮或确认动作。

已确认必须遵守：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`

如果执行 H2.7 时决定补 UI、权限弹层、readiness 面板或 readback 状态展示，必须先修订本任务包 UI 显示边界章节，并补对应离线交互测试；不能顺手新增真实执行按钮。

## 7. 验收要求

H2.7 默认为文档 / 授权准备任务，不要求运行 `npm` / `cargo`。

必须完成：

- 读取 H2.6 evidence / handoff 和当前权威入口。
- 生成 Phase B 授权准备结论。
- 明确 fixture、target session、permission envelope、readback、runtime log、audit、evidence 和 rollback 的确认状态。
- 如果缺 existing target session，必须明确阻断，不得进入 Phase B。
- 如果全部就绪，也只能输出待最终批准状态，不得执行真实 resume。
- 新增 H2.7 evidence / handoff，或在本任务未执行时保持只创建任务包。
- 同步 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`、`docs/plans/README.md`、`docs/plans/middleware-version-stage-plan-v1.md` 和 H-I 阶段计划。

必须扫描确认：

- 不出现 H2 已完成、H3 可开始、Phase B 已执行、真实 resume 已执行、prompt 已发送、`.codex` 已读写等误导口径。
- 高风险词若出现，只能出现在禁止项、待授权项或历史 evidence 引用中。

## 8. 接受范围

H2.7 完成后可接受为：

- H2 Phase B 授权准备材料完成。
- fixture / existing target session / permission envelope / rollback 的确认状态已冻结。
- 若缺 target session，阻断状态已明确冻结。
- 若全部就绪，Phase B final approval 请求可被全局主管复核。

H2.7 不接受为：

- H2 通用真实 resume 产品化完成。
- H2.5 Phase B 已授权或已执行。
- 真实 `codex exec resume` 已执行。
- prompt 已发送。
- `/Users/yoyi/.codex` 已读写。
- fixture 已真实执行。
- target session 已被自动发现。
- H3 通用真实 send / 新会话完成或可直接开始。
- H5 项目工作流真实派发完成。
- 阶段 H 完成。

## 9. 下一步

H2.7 执行后：

- 如果输出 `authorization_ready_waiting_final_user_approval`，下一步只能另开 Phase B final approval / real fixture run 任务，并在执行点再次请求用户 / 全局主管明确授权。
- 如果输出 `blocked_waiting_target_session`，下一步应要求用户提供 existing target session，或另拆 H3 通用真实 send / 新会话任务包；不能把 H3 当作 H2 Phase B 已满足。
- 如果输出 `blocked_waiting_fixture` 或 `blocked_waiting_permission_envelope`，必须先补对应缺口，不能真实 resume。
