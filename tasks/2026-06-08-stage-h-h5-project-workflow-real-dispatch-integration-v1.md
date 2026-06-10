# Stage H / H5 Project Workflow Real Dispatch Integration v1

日期：2026-06-08

状态：Level A 非真实产品路径集成已完成并通过全局主管复核；Level B 授权与 fixture freeze 已创建，真实项目派发执行仍必须单独授权。  
用途：把阶段 C 的 prepared dispatch、M6 task memory packet、H1/H2/H3 `codex-local` runner 契约、G1/G2 runtime / diagnostics、C5/C6 worker report / process fact / final review 链路串成可开发、可验收、可审计的 H5 任务包边界。Level A 完成不等于 H5 已完成，不等于真实项目派发已授权，不等于真实 worker 或 Codex 已执行。

## 1. 权威依据

本任务包依据：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`
- `docs/workflow-task-package-design-v1.md`
- `docs/memory-layer-design-v1.md`
- `tasks/2026-06-04-workflow-c4-project-director-task-decomposition-and-authorized-prepared-auto-dispatch-v1.md`
- `tasks/2026-06-04-workflow-c5-worker-structured-report-process-fact-confirmation-and-failure-visibility-v1.md`
- `tasks/2026-06-05-workflow-c6-global-final-result-review-user-result-view-and-stage-c-acceptance-v1.md`
- `tasks/2026-06-04-memory-layer-m4-task-memory-packet-builder-and-preview-v1.md`
- `tasks/2026-06-04-memory-layer-m6-workflow-task-package-injection-and-end-to-end-loop-v1.md`
- `tasks/2026-06-06-stage-g-g1-runtime-log-boundary-and-minimal-store-v1.md`
- `tasks/2026-06-06-stage-g-g2-diagnostics-health-and-degraded-state-v1.md`
- `tasks/2026-06-07-stage-h-h1-codex-local-runner-architecture-and-data-contract-v1.md`
- `tasks/2026-06-07-stage-h-h2-general-real-resume-productization-v1.md`
- `tasks/2026-06-07-stage-h-h3-1-new-session-request-guard-permission-envelope-and-noop-runner-v1.md`
- `tasks/2026-06-07-stage-h-h3-b-real-new-session-final-approval-and-fixture-run-v1.md`
- `evidence/2026-06-04-workflow-c4-project-director-task-decomposition-and-authorized-prepared-auto-dispatch-v1.md`
- `evidence/2026-06-04-workflow-c5-worker-structured-report-process-fact-confirmation-and-failure-visibility-v1.md`
- `evidence/2026-06-05-workflow-c6-global-final-result-review-user-result-view-and-stage-c-acceptance-v1.md`
- `evidence/2026-06-04-memory-layer-m4-task-memory-packet-builder-and-preview-v1.md`
- `evidence/2026-06-04-memory-layer-m6-workflow-task-package-injection-and-end-to-end-loop-v1.md`
- `evidence/2026-06-07-stage-g-g1-runtime-log-boundary-and-minimal-store-v1.md`
- `evidence/2026-06-07-stage-g-g2-diagnostics-health-and-degraded-state-v1.md`
- `evidence/2026-06-08-stage-h-h2-phase-b-mario-test-real-resume-productization-probe-v1.md`
- `evidence/2026-06-07-stage-h-h3-1-new-session-request-guard-permission-envelope-and-noop-runner-v1.md`
- `evidence/2026-06-07-stage-h-h3-b-task-package-creation-and-authority-sync-review-v1.md`
- `evidence/2026-06-08-stage-h-h5-project-workflow-real-dispatch-integration-level-a-v1.md`
- `handoffs/2026-06-08-stage-h-h5-project-workflow-real-dispatch-integration-level-a-v1-result.md`
- `evidence/2026-06-08-stage-h-h5-supervisor-acceptance-review-v1.md`
- `handoffs/2026-06-08-stage-h-h5-supervisor-acceptance-review-v1-result.md`

## 2. 当前事实

- C1-C6 已完成受控工作流闭环，但没有真实 worker / Codex 执行。C4 只创建 `state: "prepared"` dispatch；C5 worker report 和 process fact 是结构化记录与项目主管确认链路；C6 是最终复核 / 用户决定 / 阶段验收摘要链路。
- M4/M6 已完成任务记忆包 preview / injection：任务包 artifact 可以携带 frozen `TaskPackageMemoryPacketSnapshot`，prepared prompt 使用 artifact 中同一份 snapshot，不重新召回记忆；candidate / observation / knowledge hit 不能进入 included list。
- G1/G2 已完成 runtime log 最小 store 和 diagnostics / degraded state 只读底座；runtime log 与 audit 分离，诊断不自动修复、不自动重试。
- H1 已完成 `CodexLocalRunner` 契约和 guard；H2 已完成一次 `mario test` 真实 resume 探针，证明受控 `resume` 最小产品路径可行；H2 不接受为 H5 完成。
- H3.1 已完成 `new_session` 非执行产品路径；H3-B 已执行一次隔离 fixture 真实 `codex exec` new-session probe，但结果为 `failed_classified`，产品路径已补 `--skip-git-repo-check`，未二次真实执行，不能继承 H2 授权。
- H4 readback / failure / timeout / duplicate guard 已完成 Level A 非真实产品化；真实失败 / 超时探针 H4-Level-B 尚未授权或执行。
- H5-Level-A 非真实产品路径集成已完成并通过全局主管复核：后端 bridge 可生成 / 预览 / 校验 `prepared dispatch -> permission envelope -> CodexLocalExecutionRequest -> attempt/readback/runtime/audit preview -> worker report candidate/process fact handoff` 受控链路，但不调用真实 runner、不发送 prompt、不写 runtime log / audit / workflow state、不读写 `/Users/yoyi/.codex`。

## 3. 目标

H5-Level-A 目标：

- 设计并冻结 `prepared dispatch -> permission dialog -> CodexLocalExecutionRequest -> continuation / attempt -> runtime log / audit -> readback -> worker report candidate -> project director process fact decision -> C6 final review` 的最小产品路径。
- 明确 C4 `prepared` dispatch 如何桥接到 H1/H2/H3 runner，而不让前端或 Markdown 直接拼 CLI。
- 明确 M6 frozen task memory packet 如何注入真实 dispatch evidence、permission preview、prompt summary 和 worker handoff。
- 明确 runtime log、audit、readback、diagnostics、worker report、process fact 和 UI 状态的写入 / 显示 / 禁止边界。
- 为后续 H5-Level-B 真实项目派发列出执行点授权条件、测试矩阵和停止条件。

H5-Level-B 目标，需另行授权：

- 在 H3-B retry 已重新授权并回收清楚、且 H4 安全链路满足后，对隔离项目执行一次真实项目工作流最小闭环。
- 真实派发后完整记录 evidence / handoff，不把一次 demo 冒充复杂业务自动编排完成。

## 4. 非目标

本任务包 Level A 不做：

- 不执行真实 `codex exec`。
- 不执行真实 `codex exec resume`。
- 不发送 prompt。
- 不创建真实 Codex session。
- 不读写 `/Users/yoyi/.codex`。
- 不创建真实项目派发 run。
- 不修改任何真实项目文件或工作台运行状态。Level A 允许非真实产品代码集成，但必须使用 fake / no-op / preview / guard inspection，不调用真实 runner。
- 不读取 auth、token、secret、`.env`、keychain、OAuth、provider credential 或完整 transcript / rollout。
- 不接 planned adapters 真实执行。
- 不做 provider credential store 或 model verification。
- 不做自动重试、自动恢复或跨 provider 调度。
- 不让 worker report、readback、tool output、observation 或 candidate 直接写正式事实 / 正式记忆。

## 5. 前置条件

进入 H5-Level-A 设计可用的前置：

- H0 / H1 已完成并通过复核。
- H2 受控真实 resume 最小路径已有一次 `mario test` 探针证据。
- H3.1 new session 非执行产品路径已完成。
- C4/C5/C6、M4/M6、G1/G2、F1-F5 已完成。

进入 H5-Level-B 真实项目派发必须同时满足：

- H3-B retry 已经由用户 / 全局主管在执行点重新授权并回收清楚，或全局主管明确批准 H5-Level-B 使用 resume 路径且不依赖 new session。
- H4 Level A 非真实产品化已通过主管复核；如 H5-Level-B 需要真实失败 / 超时探针证据，则 H4-Level-B 必须另行授权并回收。
- H5-Level-B 单独任务包已确认 fixture project、workflow、node、work item、adapter、operation、target session 或 new session 策略、allowed write roots、`.codex` 最小范围、prompt summary/ref/hash、runtime log、audit、readback、rollback 和 evidence path。
- 没有 queued / running duplicate dispatch。
- diagnostics 没有 blocking degraded state。
- task memory packet fingerprint 未 stale，且 lint blocking 已阻断或已被用户明确确认。

## 6. C4 Prepared Dispatch 到 H Runner 的桥接

建议新增或复用一个后端应用服务边界，命名可以是 `WorkflowRealDispatchBridge` 或等价命令，但必须服从 H1 `CodexLocalRunner` 契约。

输入只允许来自：

- C4 已存在且 `state = "prepared"` 的 dispatch。
- 与该 dispatch 绑定的 task package artifact。
- C3 active authorization 和 C1 guard 校验结果。
- M6 frozen task memory packet snapshot。
- 用户确认后的 permission envelope。

桥接步骤：

1. 读取 prepared dispatch、work item、workflow node、task package artifact 和 memory packet fingerprint。
2. 复核 C2 proposal / C3 authorization / C1 guard 回链仍有效。
3. 复核 dispatch 仍是 `prepared`，没有 running / succeeded / failed / cancelled attempt。
4. 复核 task package artifact 中的 memory packet snapshot 未 stale；若 stale，回到 task package regenerate / user review，不执行。
5. 根据 node 绑定选择 operation：
   - existing target session 存在且授权完整：`resume`。
   - 需要创建新 worker session：只允许在 H3-B 完成并有 H5-Level-B 授权后使用 `new_session`。
6. 构造 `CodexLocalExecutionRequest`：
   - `adapter_id = codex-local`
   - `operation_id = resume | new_session`
   - `project_id / workflow_id / workflow_node_id / work_item_id / dispatch_id`
   - `cwd / project_root / allowed_write_roots`
   - `prompt_ref / prompt_hash / prompt_summary`
   - `task_memory_packet_ref / memory_packet_fingerprint`
   - `readback_plan`
   - `runtime_log_context`
   - `audit_context`
7. 调用 H1 guard inspection；guard allowed 后仍必须等待 permission dialog 的 explicit approval。
8. 真实执行只能由后端 runner 触发；前端不得直接拼 `codex` 命令。

## 7. M6 Task Memory Packet 注入

真实 dispatch 使用的记忆上下文必须来自 M6 frozen snapshot：

- included 只能是 active formal memory。
- candidate / observation / knowledge hit 只能作为 excluded / review materials。
- memory packet snapshot、fingerprint、generated_at、store revisions、warnings 和 stale flag 必须进入 dispatch evidence。
- permission dialog 只显示摘要：included count、excluded count、review materials count、lint warnings、fingerprint、stale status。
- prompt body 不进入普通 evidence、runtime log、audit 或 UI；只记录 prompt summary/ref/hash。
- 如果 formal / candidate / observation / lint revision 变化导致 stale，真实 dispatch 必须阻断。

## 8. Permission Dialog

H5 权限弹层必须显示：

- project、workflow、node、work item、prepared dispatch id。
- adapter、operation、target session 或 new session 策略。
- cwd、project root、allowed write roots、denied paths。
- sandbox、timeout、duplicate guard 状态。
- prompt summary、prompt ref、prompt hash。
- task memory packet 摘要、fingerprint、stale / lint 状态。
- readback plan、expected result type、readback unavailable / failed / timed out 的显示边界。
- runtime log refs preview、audit refs preview、diagnostic preflight。
- `.codex` 最小副作用说明：只有 Level B 单独授权后才允许真实 Codex home 读写。
- rollback / cleanup / hash-diff plan。

权限弹层禁止：

- 显示完整 prompt。
- 显示 raw stdout / stderr。
- 显示完整 transcript / rollout。
- 显示 secret / token / credential / `.env`。
- 在未执行前显示“Codex 已收到任务”“worker 执行中”“真实派发已开始”。
- 在 readback unavailable / failed / timed out 时显示“真实 0 条结果”。

## 9. Runtime Log / Audit / Readback

Runtime log：

- 必须写 dispatch attempt、readback、permission wait、diagnostic event 等脱敏摘要。
- 只保存状态、分类、refs、时间、actor 和脱敏摘要。
- 不保存 audit event 本体、完整 prompt、完整 transcript、raw credential、secret 或 raw stdout/stderr。

Audit：

- 必须记录 user confirmation / global approval、guard result、dispatch started、dispatch completed / failed、readback status、worker report recorded、process fact decision。
- audit 可以引用 runtime log，但不能替代 runtime log。

Readback：

- `succeeded` 可以进入 worker report candidate 或等价结果摘要。
- `readback_unavailable` / `readback_failed` / `timed_out` 必须保留 `result_count = null` 或等价未知状态。
- 真实 0 条结果只能用于 readback 成功且明确返回 0 条的情况。
- 不读取完整 transcript / rollout 作为 Level A 或默认 readback；Level B 若需读取 transcript metadata，必须在授权包中写清最小范围。

Diagnostics：

- H5 preflight 必须读取 G2 diagnostic summary。
- blocking degraded state 阻断真实 dispatch。
- warning degraded state 进入 permission dialog 和 runtime log，不自动修复。

## 10. Worker Report / Process Fact 回链

真实 dispatch readback 成功后，H5 只允许生成 worker report candidate 或等价结构化结果摘要。

回链规则：

- worker report 进入 C5 `worker_structured_report_recorded` audit / read model。
- worker report 不是正式事实、不是正式记忆。
- 项目主管可以做 `confirm_process_fact` / `request_rework` / `block_and_escalate`。
- 只有低风险、本项目、非 sensitive / secret 的 confirmed process fact 可以写 `observations.v1.json`，状态仍是 observation。
- observation / candidate / formal memory 仍走 M3-M13 记忆状态机。
- C6 final review 和 user result decision 仍是独立链路，不能由 worker 或秘书代替。

## 11. 失败 / 重试边界

失败分类至少覆盖：

- `guard_blocked`
- `user_rejected`
- `diagnostics_blocked`
- `memory_packet_stale`
- `duplicate_dispatch_blocked`
- `permission_missing`
- `execution_failed`
- `timed_out`
- `cancelled`
- `readback_unavailable`
- `readback_failed`
- `process_fact_rework_required`
- `process_fact_blocked_and_escalated`

重试规则：

- 不做静默自动重试。
- retry 必须生成新的 permission preview 和 audit。
- retry 必须引用前一次 attempt 和 failure reason。
- retry 必须重新检查 task memory packet stale、diagnostics、duplicate guard、allowed write roots 和 prompt hash。
- cancel / stop 不能默认为 kill；H4 未完成前只能显示边界或手动停止建议，不得声称取消产品化完成。

## 12. UI 显示边界

允许显示：

- 项目工作流节点从 prepared 到 awaiting authorization / queued / running / succeeded / failed / readback unavailable 的状态摘要。
- 节点详情里的 task package、memory packet、permission、runtime log、audit、readback 和 worker report refs。
- 运行中入口显示正在执行 / 卡住 / 需要权限。
- 通知入口显示 dispatch outcome / readback / failure。
- 待办入口显示用户需要批准、重试确认、process fact 决策或 final review。
- 管理入口显示脱敏 runtime log、diagnostic summary、audit refs 和 data location。

禁止显示：

- 一级“任务包管理器”或“日志管理器”主入口。
- React Flow 节点状态作为事实源；React Flow 仍只是渲染映射。
- planned adapters 可执行。
- provider credential / model verified。
- 自由聊天式裸 Codex 控制台。
- 未执行前的“worker 已执行 / Codex 已收到任务”。
- readback 失败时的“0 条结果”。
- worker report 已成为正式事实或系统已记住。

## 13. 测试矩阵

H5-Level-A 设计 / 非执行集成验收：

| 场景 | 期望 |
| --- | --- |
| prepared dispatch 缺 active authorization | guard blocked，不构造 runner request |
| prepared dispatch 已有 running duplicate | duplicate blocked，不执行 |
| memory packet stale | blocked，需要重建任务包或用户复核 |
| lint blocking 命中 included memory | blocked 或进入用户确认，不执行 |
| diagnostics blocking degraded | blocked，不执行 |
| existing session resume request 完整 | 构造 H1 request + permission preview，但不真实执行 |
| new session request 完整但 H3-B 未完成 | blocked waiting H3-B，不执行 |
| readback unavailable | result_count unknown，不显示 0 |
| worker report candidate | 只能进入 C5 report/process fact 决策链路 |
| UI 文案扫描 | 无“Codex 已收到任务 / worker 已执行 / 系统已记住”等误导完成态 |

H5-Level-B 真实执行验收，需另行授权：

| 场景 | 期望 |
| --- | --- |
| 隔离项目最小真实 dispatch 成功 | continuation / attempt / runtime log / audit / readback / worker report 可追溯 |
| 执行失败 | 写 failure reason、runtime log、audit，不自动重试 |
| 超时 | 写 timed_out，不伪装成功 |
| readback failed | result_count unknown，不进入 final review accepted |
| process fact confirmed | 只写 observation，不写正式记忆 |
| request_rework | workflow 保持可理解的返工状态 |
| block_and_escalate | 阻断并上报全局主管 / 用户待办 |
| 项目文件变化 | hash / diff / evidence 清楚记录 |
| `.codex` 副作用 | 只在授权范围内发生并如实记录 |

如 Level A 改产品代码，至少运行：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib codex_local_runner
cargo test --lib task_memory_injection
cargo test --lib runtime_log_store
cargo test --lib runtime_session_attention
cargo test --lib dispatch_readback_stats
cargo test --lib worker_structured_report
cargo test --lib process_fact
cargo test --lib
rustfmt --check ...
```

如 Level B 真实执行，必须额外提供：

- 执行前后 fixture hash / diff。
- exit code / status。
- prompt_sent / real_codex_executed / writes_codex_home 真实值。
- continuation / runtime log / audit / readback refs。
- worker report / process fact / final review refs。
- evidence / handoff。

## 14. 禁止声称事项

H5-Level-A 完成后不能声称：

- H5 已完成。
- 项目工作流真实派发已授权。
- 真实 worker 已执行。
- 真实 Codex 已执行。
- prompt 已发送。
- `/Users/yoyi/.codex` 已读写。
- H3-B retry 已授权、已执行或已成功。
- H4-Level-B 真实失败 / 超时探针已完成。
- worker report 已成为正式事实。
- observation / candidate 已成为正式记忆。
- planned adapters 已接入。
- provider credential / model 已验证。
- 阶段 H 已完成。

H5-Level-B 单项目跑通后也不能声称：

- 复杂业务自动编排完成。
- 任意项目无限制执行可用。
- 自动重试 / 自动恢复完成。
- 多 agent / 多模型中立抽象完成。

## 15. 执行点授权条件

进入 H5-Level-B 前，用户 / 全局主管必须逐项确认：

1. 是否允许执行真实项目工作流派发。
2. 使用哪个隔离 fixture project，是否允许创建 / 修改该 fixture。
3. 绑定哪个 workflow、node、work item、prepared dispatch。
4. 使用 `resume` 还是 `new_session`；如果是 `new_session`，H3-B 是否已完成或给出明确授权。
5. 是否授权真实 `codex exec` / `codex exec resume`。
6. 是否授权触碰 `/Users/yoyi/.codex` 的最小必要范围。
7. allowed write roots、denied paths 和 rollback / cleanup 策略。
8. prompt summary/ref/hash 规则，完整 prompt 不进入 argv、shell、普通 evidence、runtime log 或 audit。
9. task memory packet fingerprint / stale / lint 状态。
10. readback plan 和 readback failure 显示边界。
11. runtime log / audit / diagnostics / evidence / handoff 路径。
12. 失败、超时、重复派发、用户拒绝、readback 不可信时是否停止在 H4/H5 修补，不继续扩面。

## 16. 停止条件

出现以下任一情况，必须停止，不得进入真实项目派发：

- H3-B retry 未授权 / 未回收清楚，且本次需要新建真实 session。
- H4 Level A / Level B 边界不足以阻断真实派发风险。
- 未确认 fixture、workflow、node、work item 或 prepared dispatch。
- 未确认 allowed write roots、denied paths、`.codex` 最小范围。
- task memory packet stale 或 lint blocking 未处理。
- prompt ref/hash 缺失或需要把 prompt 放入 argv / shell 字符串。
- 需要读取 auth/token/secret/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- diagnostics blocking degraded。
- duplicate queued/running attempt 存在。
- runtime log、audit、readback、rollback、evidence 或 handoff 方案缺失。
- 用户 / 全局主管未明确授权真实执行。

## 17. 接受范围

H5-Level-A 完成后最多接受为：

- H5 项目工作流真实派发集成任务包已创建。
- H5-Level-A 非真实产品路径集成已完成并通过全局主管复核。
- `prepared dispatch -> permission envelope -> CodexLocalExecutionRequest -> attempt/readback/runtime/audit preview -> worker report candidate/process fact handoff` 受控链路已可由产品代码生成 / 预览 / 校验。
- H5-Level-B 真实项目派发的前置、授权项、停止条件和测试矩阵已列明。

H5-Level-A 完成后不接受为：

- H5 已完成。
- H5-Level-B 已授权。
- 真实项目工作流派发已执行。
- 任何新的真实 Codex 执行已发生。

## 18. Level B 授权包

H5-Level-B 真实项目工作流派发的执行前授权与 fixture freeze 已单独创建：

- `tasks/2026-06-08-stage-h-h5-level-b-project-workflow-real-dispatch-authorization-and-fixture-freeze-v1.md`
- `evidence/2026-06-08-stage-h-h5-level-b-project-workflow-real-dispatch-authorization-and-fixture-freeze-v1.md`
- `handoffs/2026-06-08-stage-h-h5-level-b-project-workflow-real-dispatch-authorization-and-fixture-freeze-v1-result.md`

该任务包只冻结后续 H5-Level-B1 的推荐执行包，不执行真实 Codex、不发送 prompt、不读写 `/Users/yoyi/.codex`，也不把 H5 写成完成。

## 19. 回交要求

执行 H5-Level-A 后必须回交：

- 创建 / 修改文件。
- 是否改产品代码；如有，列出验证命令。
- H5 bridge、memory packet、permission、runtime log、audit、readback、worker report、process fact 和 UI 边界是否落地。
- H3-B / H4 前置是否仍阻断 Level B。
- 不接受范围扫描结果。

执行 H5-Level-B 后必须新增独立 evidence / handoff，并写清：

- 是否真实执行了 `codex exec` 或 `codex exec resume`。
- 是否发送 prompt。
- 是否读写 `/Users/yoyi/.codex`，范围是什么。
- 使用的 project / workflow / node / work item / prepared dispatch / target session。
- task memory packet fingerprint 和 stale / lint 状态。
- runtime log / audit / readback / worker report / process fact refs。
- 文件 hash / diff / rollback 结果。
- 失败分类或成功证据。
- 接受范围和不接受范围。
