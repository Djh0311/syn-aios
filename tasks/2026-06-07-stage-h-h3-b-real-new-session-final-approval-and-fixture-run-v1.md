# Stage H / H3-B Real New Session Final Approval And Fixture Run v1

日期：2026-06-07

状态：已执行一次真实 fixture run 并失败分类；产品路径已修补，等待新的 retry 授权。  
用途：在 H3-A 授权冻结和 H3.1 `new_session` 非执行产品路径均已完成的前提下，准备并记录一次受控真实 `codex exec` 新会话 fixture run。H3-B 是高风险真实执行任务；本轮已执行一次隔离 fixture probe，但结果为 `failed_classified`，不等于真实 Codex session 已成功创建，不等于 H3-B 成功完成，不等于 H3 / H4 / H5 或阶段 H 完成。

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
- `tasks/2026-06-07-stage-h-h3-a-new-session-authorization-fixture-and-boundary-freeze-v1.md`
- `tasks/2026-06-07-stage-h-h3-1-new-session-request-guard-permission-envelope-and-noop-runner-v1.md`
- `evidence/2026-06-07-stage-h-h3-a-new-session-authorization-fixture-and-boundary-freeze-v1.md`
- `evidence/2026-06-07-stage-h-h3-1-new-session-request-guard-permission-envelope-and-noop-runner-v1.md`
- `handoffs/2026-06-07-stage-h-h3-a-new-session-authorization-fixture-and-boundary-freeze-v1-result.md`
- `handoffs/2026-06-07-stage-h-h3-1-new-session-request-guard-permission-envelope-and-noop-runner-v1-result.md`

## 2. 当前事实

- H3-A 已完成，只接受为 H3 通用真实 send / 新会话的授权冻结和边界设计。
- H3.1 已完成，只接受为 `new_session` request、guard、permission envelope、command plan preview、no-op runner 和只读 UI 完成。
- H2 Phase B 已在 2026-06-08 对 `/Users/yoyi/Documents/mario test` 授权并完成一次真实 `codex exec resume` 产品化探针；记录见 `evidence/2026-06-08-stage-h-h2-phase-b-mario-test-real-resume-productization-probe-v1.md` 与 `handoffs/2026-06-08-stage-h-h2-phase-b-mario-test-real-resume-productization-probe-v1-result.md`。
- H2.7 当时冻结的历史阻断是：

```text
h2_phase_b_readiness = blocked_waiting_target_session
```

- H3-B 不能继承 H2 Phase B 的执行授权，也不能把 H2 resume 探针成功解释为 H3 真实新会话已授权。
- H3-B 已在 2026-06-08 对 `/Users/yoyi/workspace/product-line/tmp/h3-new-session-fixture` 执行一次真实 `codex exec` new-session probe；记录见 `evidence/2026-06-07-stage-h-h3-b-real-new-session-final-approval-and-fixture-run-v1.md` 与 `handoffs/2026-06-07-stage-h-h3-b-real-new-session-final-approval-and-fixture-run-v1-result.md`。
- 本次 probe 真实发送 prompt，真实启动 Codex CLI，并写入 `/Users/yoyi/.codex` 的最小新会话尝试状态。
- 本次 probe 因当时 command plan 缺少 `--skip-git-repo-check` 失败；readback 为 `readback_failed`，`result_count = null`，未生成 last message。
- 产品路径已补 `new_session` 的 `--skip-git-repo-check`，但未二次真实执行。任何 H3-B retry 必须重新取得执行点授权。

## 3. H3-B 目标

H3-B 目标：

- 在用户 / 全局主管明确 final approval 后，对隔离 fixture 执行一次真实 `codex exec` 新会话。
- 使用 H3.1 已落地的 `new_session` request / guard / command plan / no-op 边界作为执行前契约。
- 确保真实执行仍由结构化 command plan 驱动，不使用 shell 字符串拼接，不把 prompt 放入 argv。
- 真实执行前后记录 fixture hash / diff。
- 记录 continuation / attempt / runtime log / audit / readback / failure classification。
- 验证 readback unavailable / failed / timed out 不显示为真实 0 条结果。
- 验证新 session 可被工作台后续绑定或登记为候选 session，但 H3-B 本身不自动把它用作 H2 Phase B target session。

H3-B 不目标：

- 不做 H2 Phase B real resume。
- 不做 H5 项目工作流真实派发。
- 不做自由聊天式任意 send。
- 不做 planned adapters 真实执行。
- 不做 provider credential store 或 model verification。
- 不做自动重试、自动恢复、跨 provider 调度或多 agent 抽象。
- 不读取 auth、token、secret、`.env`、keychain、OAuth、provider credential 或完整 transcript / rollout。
- 不把成功的新会话探针包装成 H3 全部完成、H5 完成或阶段 H 完成。

## 4. Final Approval 授权包

推荐默认执行包如下；所有“待确认”项必须由用户 / 全局主管在执行前明确确认。

| 授权项 | 推荐值 / 待确认值 | 是否阻断真实执行 | 说明 |
| --- | --- | --- | --- |
| operation | `new_session` | 是 | H3-B 只允许 `codex-local` 新会话探针。 |
| adapter | `codex-local` | 是 | planned adapters 仍不可执行。 |
| fixture project | `/Users/yoyi/workspace/product-line/tmp/h3-new-session-fixture` | 是 | 推荐隔离 fixture；本任务包创建时不创建该目录。 |
| project root | 同 fixture project | 是 | 必须为绝对路径，不能含 `..`。 |
| target cwd | 同 fixture project | 是 | 必须在 project root / allowed write roots 内。 |
| work item | 待创建或待绑定 H3 fixture work item | 是 | `new_session` 必须绑定 work item，不能创建自由会话。 |
| workflow / node | 待创建或待绑定 H3 fixture workflow / node | 是 | 必须有项目 / 工作流 / 节点上下文。 |
| allowed write roots | fixture project | 是 | 不默认写真实业务项目。 |
| denied paths | secret / auth / token / `.env` / keychain / OAuth / provider credential / full transcript / rollout | 是 | 任一需要读取或写入即停止。 |
| prompt summary | `H3 real new session safe probe` | 是 | 只记录摘要，不记录完整 prompt。 |
| prompt ref | `workbench-managed:h3-real-new-session-safe-probe:v1` | 是 | 真实执行前必须由执行包或产品路径生成。 |
| prompt hash | 真实执行前由完整 prompt 计算 SHA-256 | 是 | 不允许伪造 hash。 |
| `.codex` 范围 | 仅限 Codex CLI 创建新会话必需最小范围 | 是 | 必须承认真实新会话会写 Codex home。 |
| command | `codex exec -C <fixture> --sandbox <sandbox> --add-dir <fixture> --json --output-last-message <managed-path>` | 是 | 不通过 shell，不把 prompt 放入 argv。 |
| sandbox | 受控 sandbox，禁止 dangerous bypass | 是 | 禁止 `--dangerously-bypass-approvals-and-sandbox`。 |
| timeout | 建议 120000 ms | 是 | 超时写 failure reason，不自动重试。 |
| readback plan | workbench-managed last message + attempt/runtime refs | 是 | unavailable / failed / timed out 不得显示为 0 条结果。 |
| runtime log | 必须写脱敏 runtime log ref | 是 | raw stdout/stderr 不进入普通 UI。 |
| audit | 必须写用户确认、执行开始、执行结束 / 失败 | 是 | audit 不替代 runtime log。 |
| evidence path | `evidence/2026-06-07-stage-h-h3-b-real-new-session-final-approval-and-fixture-run-v1.md` | 是 | 真实执行结果写入独立 evidence。 |
| handoff path | `handoffs/2026-06-07-stage-h-h3-b-real-new-session-final-approval-and-fixture-run-v1-result.md` | 是 | 真实执行结果写入独立 handoff。 |
| rollback | 执行前后 hash + diff + cleanup / rollback note | 是 | 失败不能包装成通过。 |

## 5. 用户批准前必须回答

进入 H3-B 真实执行前，用户 / 全局主管必须明确回答：

1. 是否使用推荐 fixture：`/Users/yoyi/workspace/product-line/tmp/h3-new-session-fixture`。
2. 是否授权创建或使用该 fixture 项目。
3. 是否授权本次执行真实 `codex exec` 新会话。
4. 是否授权本次执行触碰 `/Users/yoyi/.codex` 的新会话必需最小范围。
5. 是否确认 allowed write roots 只限 fixture project。
6. 是否确认 `new_session` 必须绑定 work item / workflow / node，不能创建自由会话。
7. 是否确认 prompt summary/ref/hash 规则，并接受完整 prompt 不进入任务包 / argv / shell string / evidence。
8. 是否确认 readback unavailable / failed / timed out 保持状态，不显示为 0 条结果。
9. 是否确认执行前后 hash / diff、runtime log、audit、readback 和 failure classification 写入 H3-B evidence / handoff。
10. 如果 guard blocked、execution failed、timeout 或 readback 不可信，是否停止在 H3.x 修补，不进入 H4 / H5。

## 6. 停止条件

出现以下任一情况，必须停止，不得执行真实新会话：

- 未确认 fixture。
- 未确认 work item / workflow / node 绑定。
- 未确认 `.codex` 最小范围。
- 未确认 allowed write roots。
- 未确认 prompt summary/ref/hash 规则。
- 需要读取 auth/token/secret/`.env`/keychain/OAuth/provider credential/full transcript。
- 需要使用 shell 字符串拼接或 dangerous sandbox bypass。
- 发现 duplicate queued/running attempt。
- 需要写真实业务项目而非隔离 fixture。
- runtime log、audit、readback 或 rollback 方案缺失。
- 用户 / 全局主管未明确授权真实 `codex exec` 新会话。

## 7. UI 显示边界确认

本任务预计可能涉及 UI，但本任务包创建本身不改 UI：

- [x] 当前创建任务包不改前端、不改读模型、不改 UI 文案。
- [ ] 后续执行如改前端类型 / Tauri wrapper，必须补测试。
- [ ] 后续执行如改读模型摘要或状态显示，必须补 UI 显示边界复核。
- [ ] 后续执行如改已有页面局部 UI，必须补离线交互测试。
- [ ] 后续执行不得新增一级入口、主导航或右侧入口。

必须遵守：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`

允许显示：

- H3-B real new session request preview。
- 权限弹层：project、workflow、node、work item、cwd、allowed write roots、sandbox、prompt summary/hash/ref、任务记忆包摘要、`.codex` 最小副作用说明、readback plan、timeout、duplicate guard、failure handling、runtime log refs、audit refs。
- 执行状态：waiting authorization、queued、running、succeeded、failed、timed out、readback unavailable、readback failed、duplicate blocked、user rejected。
- 新 session 候选绑定摘要，但不能把候选 session 直接当 H2 target session。

禁止显示：

- 未执行前显示“Codex 已收到任务”。
- 未执行前显示“真实新会话已创建”。
- readback unavailable / failed / timed out 显示为“真实 0 条结果”。
- 完整 prompt、raw transcript、secret、完整 stdout/stderr。
- planned adapters 可执行。
- provider credential / model verified。
- 绕过权限的自由发送按钮或自由聊天输入框。

如 H3-B 执行时改可见 UI，必须补离线交互测试。真实执行 UI / 权限弹层 / 运行状态如果进入真实 Tauri 检查，必须安排真实 Tauri 或明确记录 screenshot incomplete。

## 8. 验收要求

任务包创建验收：

- 新增本 H3-B task。
- 同步 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`、`docs/plans/README.md`、`docs/plans/middleware-version-stage-plan-v1.md` 和 H-I 阶段计划。
- 扫描确认没有把 H3-B 写成已授权 / 已执行 / 已完成。

真实执行验收必须包含：

- 执行前 fixture 文件 hash / 目录快照。
- 真实 `codex exec` exit code。
- prompt_sent / real_codex_executed / writes_codex_home 的真实值。
- `/Users/yoyi/.codex` 最小读写范围说明。
- continuation record。
- attempt record。
- runtime log。
- audit event。
- readback result 或 readback unavailable / failed 分类。
- 执行后 fixture 文件 hash / diff。
- 新 session id / last message / binding candidate 的脱敏摘要。
- failure / timeout / duplicate / user rejection / guard blocked 的分类证据或测试覆盖。

如真实执行路径改产品代码，至少运行：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib codex_local_runner
cargo test --lib session_continuation
cargo test --lib runtime_log
cargo test --lib runtime_session_attention
cargo test --lib
rustfmt --check ...
```

## 9. 接受范围

H3-B 任务包创建后可接受为：

- H3-B final approval / real new session fixture run 任务包已创建。
- 真实执行前授权项、停止条件、fixture、`.codex` 范围、prompt envelope、readback、runtime log、audit、evidence 和 rollback 要求已冻结。

H3-B 真实执行成功后最多可接受为：

- 经用户 / 全局主管明确授权后，隔离 fixture 的一次 H3 real new session fixture run 完成。
- 真实新 session 创建或明确失败分类可追溯。
- continuation / runtime log / audit / readback 链路可追溯。

本轮 H3-B 已执行一次真实 fixture run，但结果只接受为：

- 一次授权范围内真实 `codex exec` new-session probe 已执行。
- prompt 已发送，`.codex` 最小副作用已发生。
- 失败已分类为 `failed_classified`，readback failed / `result_count = null`。
- fixture 业务文件 hash 前后一致，仅新增工作台自有 `.workbench/h3-b-runs/...` 记录。
- 产品路径已补 `--skip-git-repo-check`，但未二次真实执行。

H3-B 不接受为：

- H2 Phase B 执行授权已继承或可被 H3-B 复用。
- H2 通用真实 resume 产品化完成。
- H3 通用真实 send / 新会话完整产品化完成。
- H4 failure / timeout / duplicate guard 完整产品化完成。
- H5 项目工作流真实派发完成。
- H 阶段完成。
- planned adapters 真实接入。
- provider credential / model verification。
- 自动重试产品化。
- 完整多 agent / 多模型协作抽象。

## 10. 回交要求

真实执行 H3-B 后必须新增：

- `evidence/2026-06-07-stage-h-h3-b-real-new-session-final-approval-and-fixture-run-v1.md`
- `handoffs/2026-06-07-stage-h-h3-b-real-new-session-final-approval-and-fixture-run-v1-result.md`

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

- 是否真实执行了 `codex exec`。
- 是否读写了 `/Users/yoyi/.codex`，范围是什么。
- 是否发送了 prompt。
- 使用的 fixture / project / workflow / node / work item。
- 哪些文件发生变化或保持不变。
- runtime log / audit / readback 的记录位置。
- 新 session 是否只是 binding candidate，是否不能自动满足 H2 Phase B。
- H3-B 接受范围和不接受范围。
- 是否可以继续 H3.x 修补、H4 或必须等待授权。
