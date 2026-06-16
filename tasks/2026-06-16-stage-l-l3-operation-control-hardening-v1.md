# Stage L / L3 Operation Control Hardening v1

日期：2026-06-16

状态：实现、本地验证与独立复核完成；Aquinas 复审 `STATUS: CLEAR`；提交前停止。本文是 Stage L 的 L3 任务包，用于把 retry / stop / restart / resume 四个运行控制操作硬化成「可确认、可审计、可回收的产品化控制面」。L3 默认不授权任何新的真实执行：不执行真实 `codex exec` / `codex exec resume`、不真 kill / stop / restart 任何真实进程或会话、不放宽既有 real-resume 门、不读写 `/Users/yoyi/.codex`、不启动 K3-B1 retry / K3-B2。L3 把四个操作做成「点击 → 风险确认 → 只记录决策 + 推进产品状态 + 写审计」，绝不调用 runner。

一句话判据：如果 retry / stop / restart / resume 在产品层都能被用户确认、留下审计、显示明确状态（含「未执行」「待处理」「被门挡」），并且验证证明没有任何新的真实执行路径被接通、既有 real-resume 门没有被放宽，则 L3 可进入复核。

## 0. 全局主管理解（基于 2026-06-16 只读代码普查）

已知事实：

- 真正的 retry / stop / restart 操作 **今天并不存在**。前端 `src/lib/runQueue.ts` 的派生层把 `can_retry` / `can_stop` / `can_restart` / `can_resume` 全部 hardcode 为 `false`；`src/lib/sessionOperations.ts` 把 `stop` / `restart` 标为 `blocked`，把 `resume` / `new_session` / `send_message` 标为 `requires_future_task`。
- `resume` 是唯一有真实执行 wiring 的操作：`src-tauri/src/session_continuation_store.rs` 的 `run_real_resume_phase_b_with_runner()`，冻结在 `mario test` 项目、`codex-local` adapter、J1-B / J2-B 执行点，必须经用户确认 + permission envelope 批准后才到 `phase_b_real_resume_executed`。**高风险，L3 一寸不动它，不扩大到其他项目。**
- 已有可复用 plumbing：`UserConfirmationKind`（含 `retry_confirmation` / `stop_cancel_confirmation`，见 `runQueue.ts`）、`RunQueueReadModel`（run_queue_items / user_confirmation_queue / failure_control_summaries / operation_control_summary）、`RealExecutionProductCommandFailureStopRetryItem` / `...Summary`（`types.rs`）、`RealExecutionProductCommandReadbackBoundary` 与 `...PermissionEnvelope`（`src/lib/types/execution.ts`）、duplicate guard、`workflow_audit.rs`（骨架，L1 刚加了一个事件，仍很小）、`runtime_log_store.rs`、记忆 capture / observation / candidate 链路。
- 前置已完成：K5 任务包（`tasks/2026-06-10-stage-k-k5-running-todo-failure-recovery-and-operation-control-ux-v1.md`）做了**只读**的 operation control UX 摘要；J4（`tasks/2026-06-09-stage-j-j4-run-queue-failure-control-and-user-confirmation-queue-v1.md`）做了确认队列。L3 在它们之上硬化，不重造。
- 运行控制 UI 现落在 `src/views/RunningWorkflowsView.tsx`（约 601 行）：已有「运行队列」与「失败控制」段，把 can_retry / can_stop 等当只读标志展示，并已有「重试、停止、恢复和重启都必须先进入确认，不会自动调用 runner」的说明。

本任务的核心判断：

```text
L3 要解决「四个操作如何被确认、留痕、可回收地表达成产品」，不是「真的去 kill / restart / resume 一个真实运行」。
```

## 1. 权威依据

必须读取并服从：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `docs/plans/2026-06-10-stage-l-post-k-deferred-closure-and-daily-use-hardening-plan-v1.md`（§1 / §2 L3 行 / §4 真实执行前置工作表 / §5 接受口径）
- `evidence/2026-06-10-stage-l-l1-k3-b1-blocked-recovery-product-path-v1.md`（L1 是「产品面 only、不接真实执行」的范例，照此对齐风格与边界）
- `tasks/2026-06-10-stage-k-k5-running-todo-failure-recovery-and-operation-control-ux-v1.md`（K5 只读操作控制 UX，L3 在其上硬化）
- `tasks/2026-06-09-stage-j-j4-run-queue-failure-control-and-user-confirmation-queue-v1.md`（确认队列）
- `docs/workbench-system-architecture-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`

## 2. 目标

L3 必须完成：

- 为 retry / stop / restart / resume 各建立明确的**操作状态模型**：每个操作至少能表达 `not_applicable` / `available` / `pending_confirmation` / `confirmed_recorded` / `rejected` / `blocked`，并明确 `confirmed_recorded` 不等于已执行、不等于已成功。
- 每个操作给出**风险披露**：该操作若真执行会写什么（仅工作台状态 / 项目文件 / `/Users/yoyi/.codex`），当前是否被安全门挡、被哪道门挡。
- 建立**确认流**：用户点操作 → 看到风险说明 → 确认后只「记录决策 + 推进产品状态 + 追加审计事件」，**不调用 runner、不发 prompt、不触发真实执行**。
- 建立**审计**：每个操作决策写 `operation_decision_recorded` 类审计事件（谁、何时、选了哪个操作、是否确认风险、主管线复核结论），不存 prompt body / secret / `.codex` 原文。
- 接入 **runtime log / readback 标记**：记录 operation_kind 与 operation_status；未真执行时 readback 不显示成功，`result_count` 为 `null`，UI 显示未知 / 不可用，不显示 0 条。
- 接入**记忆捕获边界**：操作决策可进 capture / observation / candidate，用户确认后才可进 FormalMemory；不自动正式化。
- 建立**控制面 UI**：在 `RunningWorkflowsView.tsx`（必要时联动 `ProjectWorkflowSidePanel.tsx` 详情层）把四个操作做成可确认控件 + 状态显示 + 失败显示 + 审计 / readback 链接；普通用户层讲清「现在能做什么 / 不能做什么 / 为什么被挡 / 下一步谁处理」。
- 继续保持桌面端 Tauri 工作台边界，不做手机端 UI。

## 3. 不做项

L3 不做：

- 不新增任何真实执行调用：不新增 `Command::new("codex")` / 调 runner / `invoke` 触发真实工作 / spawn 真实进程。
- 不放宽既有 real-resume 门：`run_real_resume_phase_b_with_runner()` 及其授权条件保持原样，不扩大到 `mario test` 以外项目或新执行点。
- 不真 kill / stop / restart / resume 任何真实进程、真实会话或真实工作流运行。
- 不自动批准、不自动继承授权；不让 agent 自治批准 retry / stop / restart / resume。
- 不读写 `/Users/yoyi/.codex`；不读 secret / token / `.env` / keychain / OAuth / provider credential / 完整 transcript / rollout / prompt body。
- 不自动写 FormalMemory。
- 不把「已确认的操作」显示或记录成「已执行」「已成功」「自动重试已启用」。
- 不启动 K3-B1 retry、不启动 K3-B2。
- 不通过 Browser / Chrome / shell / test harness 间接实现同一真实执行结果。
- 不做手机端 UI。

## 4. 操作模型冻结

对 retry / stop / restart / resume 四个操作，各须在读模型中明确以下字段（参考 L1 `k3_b1_recovery.rs` 的 `does_execute_codex` / `requires_separate_task_package` / `boundary` 字段风格）：

- `operation_id`：`retry` / `stop` / `restart` / `resume`。
- `applies_to`：适用条件（如 running_session_only / failed_run_unit / bound_or_existing_session）。
- `would_write_if_real`：若真执行会写什么（`workbench_state_only` / `project_files` / `codex_home`）。
- `current_gate`：当前门状态（如 `blocked_no_runtime_handle` / `requires_future_task` / `gated_real_resume_mario_test_only`）。
- `does_execute_in_l3`：恒为 `false`。
- `status_after_confirmation`：点击并确认后落到的状态（如 `confirmed_recorded` / `pending_confirmation`），不得是「已执行 / 已成功」。
- `requires_separate_authorized_window`：真要执行该操作时是否必须另开独立授权窗口（retry / stop / restart 对真实运行均为 `true`；resume 真执行沿用既有 J1-B / J2-B 授权，不在 L3 放宽）。

特别约束：

- `resume` 的 `confirmed_recorded` **不等于**走过 real-resume phase B；L3 不接通 `run_real_resume_phase_b_with_runner()`。
- `stop` / `restart` 在没有真实 runtime handle 的前提下，只能记录「用户请求了停止 / 重启 + 待处理」，不得宣称真实进程被终止或重启。

## 5. 产品状态模型

L3 至少要能表达每个操作的：

- `not_applicable`（当前对象不支持该操作）
- `available`（可发起确认）
- `pending_confirmation`（已发起、等用户确认风险）
- `confirmed_recorded`（用户已确认，决策已记录、已审计；**未执行**）
- `rejected`（用户取消 / 主管线否决）
- `blocked`（被门挡，附原因）

状态约束：

- 任一操作的 `confirmed_recorded` 不解锁真实执行、不解锁 K3-B2、不放宽 real-resume 门。
- duplicate 提交不重复生成 `confirmed_recorded`。
- 被 `blocked` 的操作不允许自动换路径绕过。

## 6. 后端 / 数据边界

优先复用现有事实源：`RunQueueReadModel` / `failure_control_summaries`、`RealExecutionProductCommandPermissionEnvelope`、`...ReadbackBoundary`、`workflow_audit`、`runtime_log_store`、记忆 capture / observation / candidate 链路。

新增代码的落点（**重要：以下既有文件均已远超新文件尺寸闸，禁止往里塞大块代码**）：

- `lib.rs`（5567）、`real_execution_command.rs`（8754）、`session_continuation_store.rs`（5218）、`project_workflow_automation.rs`（5054）：只允许最小必要的注册 / 转发改动，新逻辑不落这里。
- 新逻辑落**新模块**，建议 `src-tauri/src/operation_control.rs`（操作读模型、状态契约、约束、guard、单测）；如需持久化，新增最小 sidecar store 模块。
- 审计事件可在 `workflow_audit.rs`（121 行，未超闸）增补；runtime log 标记可在 `runtime_log_store.rs`（1020 行，未超闸）增补。
- 前端新类型落新文件或 `src/lib/types/` 现有文件的小幅扩展；控制面逻辑优先放新组件，避免把 `RunningWorkflowsView.tsx` 顶过 2000 行。

如新增工作台自有 sidecar，必须满足：

- 只写工作台自有状态目录或任务包明确允许的范围。
- schema 最小化：只存操作状态、用户选择、风险确认、refs 和 hash。
- 不存 prompt body、secret、token、完整 transcript、`/Users/yoyi/.codex` 原文、rollout、provider credential。
- corrupt JSON / revision conflict 不覆盖原文件；duplicate submission 不重复生成 `confirmed_recorded`。

禁止：

- 禁止把 `/Users/yoyi/.codex` 当读源。
- 禁止通过测试 / helper 隐式触发 `codex exec` / `codex exec resume` / runner。
- 禁止把任何操作的 `confirmed_recorded` 标记为「已执行」。

## 7. UI 显示边界

落点：`src/views/RunningWorkflowsView.tsx`，必要时联动 `ProjectWorkflowSidePanel.tsx` 详情层。遵守 `docs/workbench-frontend-display-boundary-v1.md` 与 `docs/plans/task-package-ui-display-boundary-rule-v1.md`。

普通用户主层应显示：

- 每个操作当前能不能做、为什么（available / pending / blocked + 原因）。
- 点了之后会发生什么：只记录并待处理，不会自动跑。
- 哪些事不能做：不自动重试、不自动执行、不解锁 K3-B2。
- 下一步谁处理：用户、全局主管、或后续独立授权窗口。

开发者 / 详情层可显示：operation_id、current_gate、would_write_if_real、permission envelope 引用、audit / runtime / readback refs、duplicate scope。

开发者层不得默认铺开：prompt body、full transcript、secret / token / 凭据、raw `.codex` 内容、大段内部 JSON。

UI 禁止文案：`自动重试已启用`、`已执行`、`已成功`、`真实恢复已完成`、`安全审查已绕过`、`读回 0 条`、`已获得通用真实执行授权`、`K3-B2 可开始`。

## 8. 运行日志 / 审计 / readback 边界

- runtime log 记录：操作类型、操作状态、是否进入待处理、是否仍被门挡；不记录 prompt body / secret / `.codex` 原文。
- audit 记录：谁提交了操作决策、时间、操作类型、风险是否确认、主管线复核结论。
- readback：未执行真实操作时 readback status 不显示成功；`result_count` 为 `null` 显示未知 / 不可用；用户提交的材料单独标 user-submitted，不伪装成系统 readback。

## 9. 记忆层边界

允许：记录「用户对某操作做了某决策」的 capture event、相关 observation、生成候选记忆供用户确认。

禁止：自动写 FormalMemory；把「已确认操作」自动当成「已执行成功」经验；保存 prompt body / secret / 完整 transcript / `.codex` 原文。

候选记忆建议文案须含不确定性，例如：

```text
用户对某运行单元发起了 stop / restart / retry / resume 的产品确认；该决策已记录待处理，未触发真实执行，真实操作仍需独立授权。
```

## 10. 分线职责

- 全局主管线：审核 L3 是否越界、最终决定 accepted / blocked / accepted_with_deferred_items；不把「已确认操作」升级为「已执行」。
- 执行 / runner 线：只处理操作状态、product command 边界、runtime log、audit、readback 状态；不调用真实 Codex、不读写 `/Users/yoyi/.codex`。
- 工作流线：处理 run unit / attempt 与操作状态的关系、blocked gate；不解锁真实执行。
- UI / Tauri 线：做可理解的操作控制面与详情层；不新增真实执行按钮、不把开发者信息铺进主界面。
- 记忆线：接 capture / observation / candidate 边界；不自动正式化。
- 复核线：只读复核 P0/P1/P2、安全、UI 信息层级、架构边界、测试与 evidence；不写产品代码、不替主管线做最终接受。

## 11. 建议实施切片

L3 作为一个任务包内的四个切片执行：

1. L3-A：操作读模型与状态契约（四操作的状态模型、约束、guard、单测；新模块 `operation_control.rs`）。
2. L3-B：控制面 UI（四操作确认控件、风险披露、状态 / 失败显示、审计 / readback 链接；落 `RunningWorkflowsView.tsx` / 必要时 side panel）。
3. L3-C：runtime / audit / readback / memory capture 边界接线。
4. L3-D：证据、handoff、扫描、主管线接受复核。

如任一切片需要真实 `codex exec` / `codex exec resume` / 真实 kill / restart，必须停止并拆出独立授权任务包。

## 12. 验证要求

若本任务改产品代码，至少运行：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- 相关 Rust 单测：operation control / workflow_audit / runtime_log_store / memory_capture_bus / run queue 派生相关。
- `cargo fmt -- --check`

必须扫描并分类（按 `git status --short` 的 modified / untracked 文件显式列出，避免漏未跟踪新文件）：

- `自动重试已启用`
- `安全审查已绕过`
- `已执行` / `已成功`
- `result_count: 0`
- `codex exec`
- `codex exec resume`
- `Command::new`
- `/Users/yoyi/.codex`

扫描命中必须分类。命中可来自历史证据、任务包、禁止项、测试 fixture、guard 或 L3 新增的否定边界声明，但**不能来自 L3 新增的真实执行路径**。

## 13. 接受标准

L3 可接受为：

- retry / stop / restart / resume 在产品层都能被确认、留审计、显示明确状态（含「未执行」「待处理」「被门挡」）。
- 控制面可确认、可审计、可回收（rejected / 重新发起 / 待主管复核闭合）。
- runtime log / audit / readback / memory capture 边界明确。
- 没有任何新的真实执行路径被接通；既有 real-resume 门未放宽。
- 没有真实 Codex 执行、prompt 发送或 `.codex` 读写。

L3 不接受为：

- 自动重试 / 自动 stop / restart / resume 已实现。
- 任一操作「已执行 / 已成功」。
- 通用真实执行恢复策略完成。
- K3-B1 retry 成功或 K3-B2 可开始。
- 安全审查可绕过。
- FormalMemory 自动写入。
- Stage L 完成。

## 14. evidence / handoff 要求

完成后必须新增：

- `evidence/2026-06-16-stage-l-l3-operation-control-hardening-v1.md`
- `handoffs/2026-06-16-stage-l-l3-operation-control-hardening-v1-result.md`
- 独立复核文件 `evidence/2026-06-16-stage-l-l3-operation-control-hardening-review-<line>-v1.md`

evidence 必须含：实际改动范围、是否改产品代码、是否新增任何真实执行调用、是否读写 `/Users/yoyi/.codex`、UI 显示边界确认、runtime / audit / readback / memory capture 边界确认、测试与扫描结果、P0/P1/P2 复核结论。

handoff 必须含：当前状态、四操作各自的产品状态与门状态、既有 real-resume 门是否未动、用户可选下一步、下一步建议。

## 15. L3 后续

L3 完成后：

- 若用户要真的执行某个操作（真实 retry / stop / restart，或把 resume 扩到新执行点），必须另开独立授权任务包，列明执行点字段、权限 envelope、用户确认、runtime log、audit、readback、rollback，并重新通过安全审查。
- L3 不改变 L2（K3-B2）与 K3-B1 的 blocked / deferred 状态。
- 真实浏览器 / Tauri 视觉验收随 L1 残余一并结转 L4。
