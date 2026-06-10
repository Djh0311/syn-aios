# Stage H / H3-A New Session Authorization, Fixture, And Boundary Freeze Evidence v1

日期：2026-06-07

状态：H3-A 已完成；H3-B 未开始。  
结论：接受为 H3 通用真实 send / 新会话的授权冻结、fixture / guard / permission / UI / memory / runtime log / audit / readback 边界准备完成；不接受为真实 `codex exec` 已执行、prompt 已发送、真实 Codex session 已创建、`/Users/yoyi/.codex` 已读写、H2 Phase B 已满足、H3 产品化完成或阶段 H 完成。

## 1. 本轮范围

本轮执行：

- `tasks/2026-06-07-stage-h-h3-a-new-session-authorization-fixture-and-boundary-freeze-v1.md`

本轮允许并完成：

- 读取 H0 / H1 / H2.7 / H-I 计划和当前权威入口。
- 基于 H2.7 `blocked_waiting_target_session` 事实，冻结 H3-A 与 H2 Phase B 的边界。
- 冻结 H3 新会话 / send 的 request、guard、permission、prompt、fixture、runtime log、audit、readback 和 UI 边界。
- 明确 H3-B 单次真实新会话 fixture run 的 final approval 前置条件。

本轮禁止且未做：

- 未执行真实 `codex exec`。
- 未执行真实 `codex exec resume`。
- 未发送真实 prompt。
- 未创建真实 Codex session。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 auth、token、secret、`.env`、keychain、OAuth、provider credential 或完整 transcript / rollout。
- 未创建 H3 fixture。
- 未启动 Tauri / GUI / 截图。
- 未改产品代码或 UI。
- 未新增可见执行按钮或自由聊天入口。

## 2. 主管复核依据

已复核：

- H3-A 任务包：`tasks/2026-06-07-stage-h-h3-a-new-session-authorization-fixture-and-boundary-freeze-v1.md`
- H-I 计划 H2 / H3 段落：`docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`
- H2.7 evidence：`evidence/2026-06-07-stage-h-h2-7-phase-b-authorization-fixture-and-target-session-confirmation-v1.md`
- H2.7 handoff：`handoffs/2026-06-07-stage-h-h2-7-phase-b-authorization-fixture-and-target-session-confirmation-v1-result.md`
- 当前入口：`CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`

关键事实：

- H2 Phase B 仍是 `blocked_waiting_target_session`。
- H2 fixture 未创建，permission envelope 未确认。
- H3-A 不能被解释为 H2 Phase B 已满足，也不能用 H3 新会话绕过 H2 final approval。
- H3 真实新会话必须绑定 project / workflow / node / work item / role / task package / authorization scope。

## 3. H3-A 冻结结论

```text
h3_a_readiness = completed_boundary_freeze
h3_b_readiness = blocked_waiting_fixture_and_permission_envelope
real_codex_execution = not_authorized
codex_home_access = not_authorized
```

H3-A 已冻结：

- operation 只能是 `new_session` 或受控 `send_message`，不能混同 H2 `resume`。
- adapter 必须是 `codex-local`；planned adapters 保持 planned / unavailable。
- 必须绑定 project、workflow、node、work item、role、task package 和 authorization scope。
- 必须绑定 task memory packet，并展示 included / excluded / review materials。
- 必须提供 prompt summary / ref / hash；完整 prompt 不进入 argv、shell string、runtime log、audit、evidence 或 handoff。
- project root、target cwd、allowed write roots 必须是绝对路径，且不能包含 `..` 逃逸。
- allowed write roots 默认只允许隔离 fixture；真实业务项目必须另行高风险授权。
- denied paths 覆盖 secret、auth、token、`.env`、keychain、OAuth、provider credential、完整 transcript / rollout 和未授权真实业务目录。
- duplicate running attempt 必须阻断。
- readback unavailable / failed / timed out 必须保持 `result_count = null`，不能显示成真实 0 条。

## 4. H3-B 阻断项

H3-B 当前不能开始真实执行。

缺口：

- H3 隔离 fixture 未创建。
- fixture 初始文件、执行前 hash、执行后 diff、rollback / cleanup 规则未确认。
- 真实 `codex exec` 新会话未获用户 / 全局主管 final approval。
- `/Users/yoyi/.codex` 最小读写范围未获 final approval。
- project root、target cwd、allowed write roots 未绑定到具体 fixture。
- prompt summary / ref / hash 未冻结到一次真实执行。
- readback plan、runtime log、audit、evidence / handoff 路径未冻结到一次真实执行。
- failure / timeout / readback unavailable / guard blocked / duplicate blocked 的停止策略未冻结到 H3-B real run。

因此：

```text
h3_b_authorization_request = not_ready
```

## 5. UI 显示边界

本轮未改 UI。

如后续 H3-B 或 H3.x 涉及 UI，必须遵守：

- 不新增裸执行按钮。
- 不新增自由聊天输入框绕过任务包。
- 智能体页只能显示 readiness、权限说明、状态摘要、readback / failure / audit 引用。
- 权限弹层必须解释操作、影响范围、风险、写入路径、`.codex` 范围、prompt 摘要、审计和回滚。
- 审计和日志进入管理入口；通知、待办、运行中不能混成一个列表。
- 秘书不得批准权限、派发任务或确认 worker 汇报。
- 涉及可见 UI 变化时必须补离线交互测试；涉及导航、页面、画布或确认弹层时必须安排真实窗口 / 截图验收或明确 incomplete。

## 6. 验收与扫描

本轮是文档 / 授权冻结任务，未改产品代码，因此未运行 `npm` / `cargo`。

已执行文档扫描：

- H3-A 在 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`、`docs/plans/README.md`、`docs/plans/middleware-version-stage-plan-v1.md` 和 H-I 计划中可见。
- “H3 已完成 / H3-A 已完成 / 真实新会话已创建 / 真实 send 已执行 / planned adapters 已接入”等冒领词没有出现在完成声明中；仅出现在禁止项、不接受范围或扫描要求语境。

过程偏差：

- 权威入口同步前有一次扫描命令误把 Markdown 反引号放入 shell 双引号，触发了 shell 命令替换并尝试运行 `codex exec`。输出显示没有 stdin prompt，访问 `/Users/yoyi/.codex/state_5.sqlite` 因 readonly database 失败。该偏差不是产品路径，没有发送任务，没有写项目文件；但本轮不能声称过程上完全没有触发 Codex CLI。后续扫描已改用安全的单引号 / `-e` 形式完成。

## 7. 接受范围

接受为：

- H3-A 授权冻结完成。
- H3 新会话 / send 的边界、guard、permission、prompt、fixture、runtime log、audit、readback 和 UI 约束冻结完成。
- H3-B 前置条件和阻断项明确。

不接受为：

- H3 通用真实 send / 新会话产品化完成。
- H3-B 已授权或已执行。
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

## 8. 下一步

下一步可以选择：

1. 拆 H3-B final approval / real new session fixture run，但必须先让用户 / 全局主管明确批准 fixture、allowed write roots、prompt ref/hash、`.codex` 最小范围、readback、runtime log、audit、evidence 和 rollback。
2. 若不进入真实执行，先拆 H3.x 代码路径任务：实现 `CodexLocalNewSessionRequest` / `CodexLocalSendMessageRequest`、guard、permission envelope 和 no-op runner，但仍不执行真实 `codex exec`。
3. 继续停在 H3-A 后，要求用户先决定 H3-B fixture 和授权范围。
