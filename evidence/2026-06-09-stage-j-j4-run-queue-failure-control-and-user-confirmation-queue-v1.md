# Stage J / J4 Run Queue, Failure Control, And User Confirmation Queue v1 Evidence

日期：2026-06-09

状态：已完成，结论为 `accepted_with_deferred_items`。复核线最终确认无 P0/P1/P2，允许主管线把 J4 收口；当前不能声明 J5 / J6 或 Stage J 完成。

## 1. 结论

J4 已新增前端派生读模型 `run_queue_read_model.v1`，把已有 WorkbenchSnapshot、WorkflowStateSnapshot、J3 MemoryCaptureStore 和 MemoryCandidateStore 组织为三类用户可理解对象：

- 运行队列：当前运行、等待用户、guard 阻断、失败、读回不可用 / 失败、超时、重复阻断、完成待复核。
- 用户确认队列：执行确认、重试确认、停止 / 取消确认、结果确认、过程事实确认、记忆候选确认、正式化确认类型、捕获补偿确认。
- 失败控制摘要：失败分类、用户下一步、是否可恢复、是否需要重新确认、是否涉及记忆捕获补偿，以及 `readback_result_count=null` 的 unknown-result 边界。

本轮没有新增真实 `codex exec` / `codex exec resume` 调用，没有发送 prompt，没有读写 `/Users/yoyi/.codex`，没有读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout，没有启动 Tauri / Browser / Chrome / 截图工具。

## 2. 改动范围

- `prototypes/productized-desktop-shell/src/lib/runQueue.ts`
  - 新增 `RunQueueItem`、`UserConfirmationQueueItem`、`FailureControlSummary`、`RunQueueReadModel`。
  - 新增 `deriveRunQueueReadModel(...)`，只从现有 snapshot / workflowState / memoryCaptureStore 派生只读状态。
  - 从已确认但未 adoption 的 `candidate_confirmed` 记忆候选派生 `memory_formalization_confirmation`，但不写 FormalMemory。
  - workflow / runtime attention 的 capture refs 只按精确 `workflow_node_id` 回链；无 node 时不把整个 workflow 的 capture refs 铺给普通运行项。
- `prototypes/productized-desktop-shell/src/views/RunningWorkflowsView.tsx`
  - 接入 J4 运行队列、待确认、失败控制三个面板。
  - `result_count=null` 显示为“未知 / 不可用”，不显示为 0。
  - retry / stop / restart / resume 只解释为确认事项，不渲染真实动作按钮。
- `prototypes/productized-desktop-shell/src/components/RightDetailPanel.tsx`
  - 右侧 `运行中` 接入同一 J4 read model。
  - 右侧 `待办` 合入用户确认队列摘要。
- `prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts`
  - 秘书新增 `run_queue_boundary` 风险和 `inspect_run_queue` 查看建议。
  - 秘书侧同样接入 `memoryCaptureStore` / `memoryCandidateStore`，能解释 capture compensation 和 formalization 确认。
  - 秘书仍不生成批准、重试、停止、恢复、发送等执行类 action proposal。
- `prototypes/productized-desktop-shell/src/App.tsx`
  - 把 J3 `memoryCaptureStore` 和 `memoryCandidateStore` 传给运行中页和右侧面板。
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
  - 新增 J4 离线场景，覆盖 run queue 契约、确认类型、失败控制、capture compensation 和 UI 禁止文案。

## 3. 数据契约

`RunQueueItem` 覆盖任务包要求字段：

- 绑定：`project_id`、`project_root`、`workflow_id`、`workflow_node_id`、`run_unit_id`、`product_command_id`、`product_attempt_id`、`adapter_id`、`session_id`。
- 状态：`status`、`status_reason`、`user_visible_summary`、`next_step_label`、`requires_user_action`。
- 动作边界：`can_retry`、`can_stop`、`can_restart`、`can_resume` 当前均不触发真实动作。
- 读回：`readback_status`、`readback_result_count`，未知状态保持 `null`。
- 回链：`runtime_log_refs`、`audit_refs`、`capture_event_refs`、`observation_refs`、`memory_candidate_refs`。

`UserConfirmationQueueItem` 覆盖：

- `execute_confirmation`
- `retry_confirmation`
- `stop_cancel_confirmation`
- `result_confirmation`
- `process_fact_confirmation`
- `memory_candidate_confirmation`
- `memory_formalization_confirmation`
- `capture_compensation_confirmation`

`FailureControlSummary` 覆盖：

- `blocked_by_guard`
- `duplicate_blocked`
- `failed`
- `readback_unavailable`
- `readback_failed`
- `timed_out`
- `memory_capture_compensation_needed`

## 4. 边界证据

- J4 是前端派生 read model，未新增 Tauri command、Rust runner、sidecar schema 或 workflow state 顶层结构。
- retry / stop / restart / resume 默认不会调用 runner；测试断言 confirmation 队列 `confirmation_command_kind !== "runner_call"`。
- 手动停止请求进入 `stop_cancel_confirmation`，不是直接停止真实进程。
- 已确认候选进入 `memory_formalization_confirmation`，只是提示继续走正式记忆生命周期预览 / 确认，不自动写 FormalMemory。
- duplicate blocked 进入 failure control，不会生成新的真实执行动作。
- J3 capture 半完成状态进入 `capture_compensation_confirmation` 和 failure control；不会自动写 FormalMemory。
- MemoryCandidate / observation 只显示为候选和观察回链，不冒充正式记忆。
- 复核过程中曾识别两个 P2：秘书侧 capture compensation 输入不完整、workflow 级 capture refs 关联可能过宽。主管线已修补并重新验证通过。

## 5. 验证结果

已通过：

- `npm run typecheck`
- `npm run test:offline-interaction`：`offline interaction tests passed: 14`
- `npm run build`：通过，仅保留既有 Vite chunk size warning。

P2 修补后重新验证仍通过：

- `npm run typecheck`
- `npm run test:offline-interaction`：`offline interaction tests passed: 14`
- `npm run build`：通过，仅保留既有 Vite chunk size warning。

本轮未跑 Rust 测试，因为没有改 Rust / Tauri 后端。

## 6. 扫描结果

禁止误导文案扫描：

- `自动重试中 / 已自动修复 / 已写正式记忆 / 结果数：0 / runner_call`
- 产品代码新增 J4 路径无误导命中。
- 命中分类：测试中的禁止断言、既有 `canvasSurfaceBoundaries.ts` 黑名单常量、既有 TS 类型字段 `runner_call_allowed`。

真实执行关键词扫描：

- `codex exec / codex exec resume / run_real_execution_product_command_phase_b / runRealExecutionProductCommandPhaseB`
- J4 新增路径无真实执行调用命中。
- 命中分类：既有 Phase B wrapper、历史边界文案、测试 fixture / 禁止断言。

## 7. 复核结论

复核线已确认：

- 无 P0。
- 无 P1。
- 无需要保留的 P2。
- J4 只接受为运行队列、失败控制和用户确认队列前端派生读模型完成。
- J4 不接受为真实 retry / stop / restart、FormalMemory 自动写入、J5 / J6 或 Stage J 完成。

主管线可把 J4 收口为 `accepted_with_deferred_items`，但不得声明 Stage J 完成。
