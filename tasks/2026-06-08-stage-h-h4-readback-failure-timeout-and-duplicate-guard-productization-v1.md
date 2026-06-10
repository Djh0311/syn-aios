# Stage H / H4 Readback Failure Timeout And Duplicate Guard Productization v1

日期：2026-06-08

状态：已完成 Level A 非真实执行产品化；H4-Level-B 真实失败 / 超时探针必须另行授权。  
用途：在 H2 Phase B `mario test` 真实 resume 探针已完成、H3.1 `new_session` 非执行产品路径已完成、H3-B 真实新会话 fixture run 已执行一次但结果为 `failed_classified` 且等待新的 retry 授权的前提下，把 readback / failure / timeout / duplicate guard 从分散能力收敛成 H 阶段统一产品边界。H4 不是执行线；本任务包 Level A 不授权真实 `codex exec` / `codex exec resume`，不发送 prompt，不创建真实 Codex session，不读写 `/Users/yoyi/.codex`。

## 1. 权威依据

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/README.md`
- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`
- `docs/workbench-system-architecture-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `tasks/2026-06-07-stage-h-h0-safety-boundary-and-task-package-freeze-v1.md`
- `tasks/2026-06-07-stage-h-h1-codex-local-runner-architecture-and-data-contract-v1.md`
- `tasks/2026-06-07-stage-h-h2-general-real-resume-productization-v1.md`
- `tasks/2026-06-07-stage-h-h2-8-real-execution-permission-dialog-audit-summary-and-readiness-decision-surface-v1.md`
- `tasks/2026-06-07-stage-h-h3-a-new-session-authorization-fixture-and-boundary-freeze-v1.md`
- `tasks/2026-06-07-stage-h-h3-1-new-session-request-guard-permission-envelope-and-noop-runner-v1.md`
- `tasks/2026-06-07-stage-h-h3-b-real-new-session-final-approval-and-fixture-run-v1.md`
- `evidence/2026-06-08-stage-h-h2-phase-b-mario-test-real-resume-productization-probe-v1.md`
- `handoffs/2026-06-08-stage-h-h2-phase-b-mario-test-real-resume-productization-probe-v1-result.md`
- `evidence/2026-06-07-stage-h-h3-1-new-session-request-guard-permission-envelope-and-noop-runner-v1.md`
- `handoffs/2026-06-07-stage-h-h3-1-new-session-request-guard-permission-envelope-and-noop-runner-v1-result.md`
- `evidence/2026-06-07-stage-g-g1-runtime-log-boundary-and-minimal-store-v1.md`
- `handoffs/2026-06-07-stage-g-g1-runtime-log-boundary-and-minimal-store-v1-result.md`
- `evidence/2026-06-07-stage-g-g2-diagnostics-health-and-degraded-state-v1.md`
- `handoffs/2026-06-07-stage-g-g2-diagnostics-health-and-degraded-state-v1-result.md`

## 2. 当前事实

- H2 Phase B 已在 2026-06-08 对 `/Users/yoyi/Documents/mario test` session `019e798a-6ce5-76c3-b8ee-33bd0fda841f` 完成一次真实 `codex exec resume` 探针；readback 固定标记成功，`result_count = 1`。
- H2 Phase B 不接受为 H3 真实新会话、H5 项目工作流真实派发、自动重试、取消、恢复或完整 failure recovery 产品化。
- H3.1 已完成 `new_session` request / guard / permission envelope / command plan preview / no-op runner，只接受为非执行产品路径；H3-B 已执行一次隔离 fixture 真实 `codex exec` new-session probe，但结果为 `failed_classified`，产品路径已补 `--skip-git-repo-check`，未二次真实执行，任何 retry 必须重新授权。
- G1 已完成 runtime log 最小 store，覆盖 app session、workflow run、dispatch attempt、readback、permission wait、diagnostic event，且 runtime log 与 audit 不互相替代。
- G2 已完成 diagnostics / health / degraded state 只读模型，可解释 store integrity、runtime attention、readback boundary、runtime log error 和 blocked / degraded 状态，但不自动修复、不自动重试。
- 现有代码已具备分散基础：`CodexLocalFailureReason`、`CodexLocalReadbackResult`、`SessionContinuationAttempt`、`ReadbackBoundaryStatus`、`RuntimeLogStoreV1`、`DiagnosticSummary`、H2.8 duplicate guard decision surface 和相关离线测试。

## 3. 目标

H4 目标：

- 统一 `codex-local` 真实 / 非真实执行链路中的 readback status、failure reason、timeout、cancel / stop 边界、duplicate guard 和 stale run cleanup 规则。
- 把 H2 resume、H3.1 new_session no-op、未来 H3-B real new session 和 H5 project dispatch 使用的失败 / 读回边界收敛到同一套读模型和 UI 解释。
- 明确 `readback_unavailable`、`readback_failed`、`readback_timed_out`、`timed_out`、`blocked_by_guard`、`duplicate_blocked`、`user_rejected`、`cancel_requested` 等状态的用户可见含义、runtime log 记录、audit 记录和 diagnostic 联动。
- 保持 `result_count = null` 规则：unavailable / failed / timed out / not attempted 都表示结果数未知，不能显示成真实 0 条结果。
- 明确 duplicate guard 判定：同一 adapter + operation + continuation / work item / workflow node / target session 范围内存在 queued / running / waiting_permission / running real attempt 时，阻断新的真实执行。
- 明确自动重试仍不进入 H4 默认范围；任何 retry 只能输出 preview / user confirmation requirement，不能静默重试。

## 4. 非目标

H4 默认不做：

- 不执行真实 `codex exec`。
- 不执行真实 `codex exec resume`。
- 不发送真实 prompt。
- 不创建真实 Codex session。
- 不创建 H3-B fixture run。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 auth、token、secret、`.env`、keychain、OAuth、provider credential 或完整 transcript / rollout。
- 不读取 raw stdout / stderr 或完整 transcript 作为 UI 内容。
- 不实现自动重试、自动恢复、自动 kill、自动 stop 或后台守护。
- 不接入 planned adapters 的真实执行。
- 不做 provider credential store、model verification、跨 provider 调度或阶段 I 多 agent 抽象。
- 不把 H4 Level A 写成真实 Codex 执行、H3-B 成功、H5 可直接真实派发或阶段 H 完成。

## 5. 前置条件

Level A 执行前必须满足：

- H0 / H1 / H2 / H2.8 / H3-A / H3.1 / G1 / G2 当前证据已读。
- 不依赖 H3-B 已成功；H3-B 当前是一次真实 fixture run `failed_classified`，后续 retry 仍需执行点授权。
- 只允许修改产品代码中的类型、store、读模型、UI、测试和文档；不得调用真实 Codex runner。
- 如需要模拟失败 / 超时，只能使用 fake runner、no-op runner、fixture sidecar 或单测注入结果，不得通过真实 `codex exec` 制造失败。

Level B 执行前必须另行授权：

- 只有用户 / 全局主管明确批准 H4-Level-B real failure / timeout probe，才允许真实失败 / 超时探针。
- 授权包必须逐项确认 fixture、operation、work item / workflow / node、allowed write roots、`.codex` 最小范围、prompt summary/ref/hash、timeout、readback、runtime log、audit、evidence、handoff 和 rollback。
- 未确认任一项即停止，不得真实执行。

## 6. 可复用能力复核

类型 / contract：

- `src-tauri/src/types.rs` 已有 `CodexLocalReadbackPlan`、`CodexLocalReadbackResult`、`CodexLocalFailureReason`、`CodexLocalExecutionAttempt`、`SessionContinuationAttempt`、`SessionContinuationReadbackSummary`、`ReadbackBoundaryStatus`、`RuntimeLogStoreV1`、`DiagnosticSummary`。
- `src/lib/types.ts` 已有 H2.8 decision surface、duplicate attempt、readback boundary、runtime log 和 diagnostic 类型。

Store / writer：

- `src-tauri/src/session_continuation_store.rs` 已写入 continuation / attempt / audit，并在 H2 Phase A / Phase B 路径中处理 duplicate running attempt、user rejected、prompt hash mismatch、guard blocked、readback unavailable / failed / timed_out。
- `src-tauri/src/runtime_log_store.rs` 已支持显式追加 dispatch attempt / readback runtime log，损坏 runtime log sidecar 会阻断 H2 attempt，且 runtime log 只保存脱敏摘要和 refs。

Runner / command：

- `src-tauri/src/codex_local_runner.rs` 已区分 fake dry-run、Phase A no-op runner、Phase B real process runner；已有结构化 command plan、stdin prompt、timeout 分类、failure reason、duplicate active attempt guard 和 `result_count = None` 规则。
- H3.1 已让 `new_session` 进入 guard / command plan preview，但没有真实 runner。

UI / read model：

- `src-tauri/src/runtime_session_attention.rs` 已把 `readback_unavailable`、`readback_failed`、`timed_out` 等派生为 attention 和 session run status summary。
- `src/lib/h2RealResumeAuthorization.ts` 已有 H2.8 duplicate guard、permission preview、readback decision boundary 和 `result_count: null` 的前端派生。
- `src/views/AgentView.tsx` 已展示 H3.1 no-op、H2.8 decision surface、duplicate guard、readback unavailable、runtime log preview 等只读边界。
- `src/App.tsx` 管理入口已展示 runtime log 与 diagnostic summary。
- `src/lib/projectCanvas.ts` / `src/lib/secretaryReadModel.ts` 已能解释 timed out / failed / readback unavailable，不允许秘书批准、重试或确认事实。

测试：

- Rust 已有 `codex_local_runner`、`session_continuation_store`、`runtime_log_store`、`runtime_session_attention` 定向测试。
- 前端 `tests/offline-permission-dialog.test.tsx` 已覆盖 H3.1 new_session preview、H2.8 duplicate guard、readback unavailable、G1 runtime log、G2 diagnostics 和秘书风险边界。

## 7. 建议最小任务拆分

### H4-A1：failure / readback taxonomy 收敛

默认非真实执行。

产出：

- 补一份共享状态矩阵或 helper，把 attempt status、readback status、failure reason、runtime attention kind、diagnostic degraded state 的映射收敛。
- 明确 `readback_succeeded` 只有在可信来源读回且 `result_count` 为数字时成立。
- 明确 unavailable / failed / timed_out / not_attempted 均保持 `result_count = null`。
- 单测覆盖 H2 resume、H3.1 new_session no-op、guard blocked、user rejected、timeout、readback failed / unavailable。

### H4-A2：duplicate guard 与 stale run cleanup 规则

默认非真实执行。

产出：

- 统一 duplicate guard 判定字段：adapter、operation、continuation_id、work_item_id、workflow_id、node_id、session_id / target session、active status。
- queued / running / waiting_permission / running real attempt 必须阻断真实执行。
- stale cleanup 只能把工作台自有 stale attempt 标为 `cancelled` / `stale_cancelled` 或等价状态，并写 audit / runtime log；不能 kill agent，不能改 Codex 原生状态。
- stale cleanup 必须要求 expected revision，避免覆盖并发写。

### H4-A3：runtime log / audit / diagnostics 联动

默认非真实执行。

产出：

- dispatch attempt、readback、failure、timeout、duplicate blocked、user rejected、cancel requested、stale cleanup 均有 runtime log category / status / severity 映射。
- audit event 记录谁做了决定、前后状态、原因和 refs；audit 不替代 runtime log。
- diagnostic summary 能把 corrupted sidecar、blocked duplicate、readback failed / unavailable、timeout、stale run 显示为 degraded / warning / blocked。
- raw prompt、raw stdout/stderr、完整 transcript、credential、secret 不能进入 runtime log、audit、diagnostic 或普通 UI。

### H4-A4：UI / read model 产品化

默认非真实执行。

产出：

- 智能体页：运行状态、目标 session / work item、readback status、failure reason、timeout、duplicate guard、runtime log ref、audit ref 的只读摘要。
- 项目画布 / 节点详情：失败和 readback 边界进入节点 attention，不把 worker report 或 observation 直接写正式事实。
- 右侧入口：运行中、通知、待办保持分离；运行中显示 active / blocked / readback boundary，通知显示结果和风险，待办显示需要用户确认 / 修补的动作。
- 管理入口：显示脱敏 runtime log 和 diagnostic summary，不新增顶级入口。
- 秘书：只能解释风险和查看建议，不提供批准、发送、resume、new session、retry、stop、kill action proposal。

### H4-A5：测试、evidence 和入口同步

默认非真实执行。

产出：

- Rust 定向测试和前端离线测试覆盖测试矩阵。
- 若改可见 UI，补 `npm run test:offline-interaction`、`npm run typecheck`、`npm run build`。
- 若改 Rust，补相关 `cargo test --lib ...`、`cargo test --lib` 和 `rustfmt --check`。
- 新增 H4 evidence / handoff，并同步 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`、`docs/plans/README.md` 和 H-I plan。

### H4-Level-B：真实失败 / 超时探针授权项

默认不执行。只有 Level A 完成并另获用户 / 全局主管明确批准后才可拆。

可选探针范围：

- 只允许隔离 fixture。
- 只允许一次明确设计的失败或 timeout probe。
- 必须证明 prompt hash、allowed write roots、`.codex` 最小范围、readback plan、runtime log、audit、evidence、rollback 和 stop condition。
- 探针结果只接受为 H4-Level-B 指定失败 / 超时路径证据，不接受为自动重试、H5 项目真实派发、H4 全部完成或阶段 H 完成。

## 8. UI 显示边界确认

H4 预计会改读模型和局部 UI，必须遵守：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`

允许显示：

- attempt status：waiting authorization、queued、running、succeeded、failed、timed out、readback unavailable、readback failed、duplicate blocked、user rejected、cancel requested、stale cancelled。
- readback boundary：status、reason、attempted、real_readback_performed、`result_count`、source refs、user message。
- failure summary：failure code、retryable、user action required、recommended next step。
- runtime log refs、audit refs、diagnostic status 和脱敏 source refs。

禁止显示：

- 未执行前显示“Codex 已收到任务”。
- 未授权前显示“真实新会话已创建”“真实 send 已执行”“真实 resume 已执行”。
- readback unavailable / failed / timed out 显示为“真实 0 条结果”。
- 完整 prompt、完整 transcript、rollout、raw stdout/stderr、auth/token/secret/credential。
- 自动重试按钮、静默重试、裸 stop / kill 按钮、自由聊天输入框、绕过任务包的发送 / resume / new session 按钮。
- planned adapters 可执行或 provider/model/credential 已验证。

## 9. Runtime Log / Audit / Readback 边界

Runtime log：

- 记录运行事实摘要、状态、时间、duration、source refs、audit refs 和脱敏 warnings。
- 不保存 audit event 本体，不保存 prompt body，不保存 raw transcript，不保存 raw provider material。

Audit：

- 记录用户 / 主管 / 系统决策、前后状态、原因、store revision 和相关 refs。
- 不替代 runtime log，不替代 readback result。

Readback：

- 只允许可信受控来源，例如 workbench-managed last message、明确授权的 readback parser 或后续任务定义的安全摘要。
- readback 失败或不可用必须保留失败状态，不写正式事实、不写正式记忆。
- `result_count = 0` 只允许在“真实 readback performed 且可信来源确认 0 条结果”时出现；否则必须是 `null`。

## 10. Duplicate Guard 判定

阻断条件：

- 同一 continuation_id 已有 queued / running / waiting_permission / running real attempt。
- 同一 operation + work_item_id + workflow_id + node_id 已有 active attempt。
- resume 时同一 session_id / target session 已有 active attempt。
- new_session 时同一 work_item_id 已有 active new_session attempt。
- runtime log sidecar 损坏且会导致 attempt 无法追溯。
- expected store revision 不匹配。

不阻断但必须提示：

- 历史 succeeded / failed / timed_out attempt 存在。
- stale cancelled / user rejected 存在。
- readback unavailable 历史存在。

处理规则：

- duplicate blocked 写 attempt / audit / runtime log，但不调用 runner。
- stale cleanup 只能处理工作台自有状态，不能 kill Codex，不能改 `/Users/yoyi/.codex`。
- 任何 retry 必须另走 preview + user confirmation；H4 不静默重试。

## 11. `result_count = null` 规则

必须保持 `result_count = null`：

- `readback_unavailable`
- `readback_failed`
- `readback_timed_out`
- `timed_out`
- `not_attempted`
- `blocked_by_guard`
- `duplicate_blocked`
- `user_rejected`
- `cancel_requested`
- `stale_cancelled`

允许 `result_count = 0` 的唯一条件：

- `real_readback_performed = true`
- readback 来源可信且已授权
- readback status 是 succeeded / equivalent
- 结果确认为 0 条，而不是未读取 / 失败 / 超时

## 12. 测试矩阵

Level A 必测：

- success with managed readback：`result_count = 1` 或 fixture 指定数字。
- real-readback zero fixture：只用 fake / fixture 安全模拟 `result_count = 0`，证明 0 与 unavailable 不混淆。
- readback unavailable：`result_count = null`，UI 显示结果数未知。
- readback failed：`result_count = null`，runtime attention blocks continuation。
- readback timed out / attempt timed out：`result_count = null`，不自动重试。
- guard blocked：不调用 runner，写 blocked attempt / audit / runtime log。
- duplicate queued / running：优先阻断，状态为 duplicate blocked。
- user rejected：不调用 runner，写 audit，UI 显示用户拒绝不是失败。
- runtime log sidecar corrupted：阻断真实 attempt，diagnostic degraded。
- stale cleanup：只更新工作台自有 stale attempt，写 audit / runtime log，不 kill。
- H3.1 new_session no-op：保持 prompt_sent=false、real_codex_executed=false、writes_codex_home=false。
- planned adapter：保持 unavailable / planned，不出现执行按钮。
- secretary：不生成 retry / stop / send / resume / new session action proposal。

建议命令：

```text
cargo test --lib codex_local_runner
cargo test --lib session_continuation
cargo test --lib runtime_log
cargo test --lib runtime_session_attention
cargo test --lib g2_diagnostic
cargo test --lib
rustfmt --check src/codex_local_runner.rs src/session_continuation_store.rs src/runtime_log_store.rs src/runtime_session_attention.rs src/types.rs src/lib.rs
npm run test:offline-interaction
npm run typecheck
npm run build
```

如果 H4 只改文档，不要求运行代码测试；如果改产品代码，必须按实际改动补测试。

## 13. 禁止声称事项

H4 创建或 Level A 完成后，禁止声称：

- H4 已完成，除非已有 H4 evidence / handoff 和验收命令。
- 真实 `codex exec` 已执行。
- 真实 `codex exec resume` 已执行。
- prompt 已发送。
- 真实 Codex session 已创建。
- `/Users/yoyi/.codex` 已读写。
- H3-B retry 已授权或已执行。
- H3-B 已成功或真实新会话已成功创建。
- H5 项目工作流真实派发完成。
- 自动重试、自动恢复、自动 stop / kill 产品化完成。
- readback unavailable / failed / timed out 等于 0 条结果。
- planned adapters 已真实接入。
- provider credential / model verification 已完成。
- 阶段 H 已完成。

## 14. 执行点授权条件

Level A 可执行条件：

- 用户 / 主管线接受本任务包。
- 执行线确认只做非真实产品路径。
- 不调用真实 runner，不读写 `/Users/yoyi/.codex`。
- 不创建真实 fixture run。

Level B 可执行条件：

- Level A 已完成并有 evidence / handoff。
- 用户 / 全局主管逐项批准真实失败 / 超时探针。
- 授权文本必须明确包含：fixture、operation、allowed write roots、`.codex` 最小范围、prompt summary/ref/hash、timeout、readback plan、runtime log、audit、evidence、handoff、rollback、停止条件。
- 执行前必须再次扫描 duplicate active attempt。
- 任一授权项缺失即停止。

## 15. 回交要求

H4 执行后必须新增：

- `evidence/2026-06-08-stage-h-h4-readback-failure-timeout-and-duplicate-guard-productization-v1.md`
- `handoffs/2026-06-08-stage-h-h4-readback-failure-timeout-and-duplicate-guard-productization-v1-result.md`

回交必须写清：

- 是否只执行 Level A 非真实产品路径。
- 是否执行任何真实 Codex；默认应为否。
- 是否读写 `/Users/yoyi/.codex`；默认应为否。
- 改了哪些类型、store、command、UI/read model 和测试。
- result_count null 规则如何验证。
- duplicate guard 如何验证。
- runtime log / audit / diagnostics 如何联动。
- H3-B 是否仍独立授权，H4 是否依赖 H3-B 已完成。
- 下一步是否可以进入 H5，或是否必须先补 H3-B / H4-Level-B。
