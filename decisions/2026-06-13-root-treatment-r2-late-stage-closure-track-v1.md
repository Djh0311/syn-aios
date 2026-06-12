# Decision: Root Treatment / R2 后段收口轨道 v1

日期：2026-06-13

状态：**已确认，作为当前 R2 后段收口口径生效。**用户已确认按主管线建议推进：R2 后段按“明确下降轨道 + 冻结 deferred”口径收口，并转入 R4-H1；同时要求本轮有明确停止边界，不再按旧的无限续跑目标推进。

来源：

- `handoffs/2026-06-12-supervisor-line-takeover-duty-summary-claude-v1.md`
- `handoffs/2026-06-13-supervisor-brain-switch-post-t12-t13-t14-cross-check-v1.md`
- `tasks/2026-06-12-root-treatment-r2-t0-inline-tests-migration-reassessment-after-r3-a13-v1.md`
- `tasks/2026-06-12-root-treatment-r2-t13-deferred-inline-tests-reassessment-v1.md`
- `tasks/2026-06-12-root-treatment-r2-t14-rust-workflow-governance-boundary-and-director-review-rejection-test-extraction-v1.md`

## 1. 确认结论

R2 后段按以下口径收口：

```text
R2 后段进入“明确下降轨道 + 冻结 deferred”状态。
当前不再继续开低收益 R2-T 迁移包。
后续若重开 R2，必须先提出能明确降低 lib.rs 棘轮指标、且不触碰禁迁清单的下降轨道。
```

当前事实：

- `lib.rs` 当前行数：5,567。
- shape gate `lib.rs` waterline：5,567。
- T 系列可迁切片已到底。
- `lib.rs` 剩余 inline tests：35 个 = 禁迁 34 + deferred 1。

本文不把 R2 冒充为完成：`lib.rs <= 3,000` 目标尚未达成，R2 仍有结构债。本文只是停止“没有明确下降轨道”的继续拆包，把火力转向 R4 硬目标；R3 Level B 仍只保留窗口计划，不在本轮执行。

## 2. 禁迁 34 清单

### K3-B runtime prompt guard 2

- `k3_b_tauri_command_guard_rejects_runtime_prompt_body`
- `k3_b_tauri_command_guard_allows_no_real_harness_request`

### real-state / ignored real-state 2

- `reads_real_static_index_summary`
- `real_task_package_file_generation_confirmation_v1`

### cross-store memory adoption / formal memory adjacency 13

- `memory_candidate_adoption_project_director_low_risk_project_memory`
- `memory_candidate_adoption_rejects_user_preference_without_user`
- `memory_candidate_adoption_rejects_secret_without_blocked_export`
- `memory_candidate_adoption_rejects_cross_project_project_director`
- `memory_candidate_adoption_rejects_rejected_or_discarded_candidate`
- `memory_candidate_adoption_rejects_already_adopted_candidate`
- `memory_candidate_adoption_rejects_context_binding_mismatch`
- `memory_candidate_rejection_does_not_create_formal_memory`
- `formal_memory_store_rejects_missing_source_refs`
- `formal_memory_store_rejects_candidate_status`
- `formal_memory_store_keeps_candidate_store_separate`
- `formal_memory_store_damaged_json_is_not_overwritten`
- `formal_memory_store_revision_conflict_is_rejected`

### workflow node dispatch / legacy guard 12

- `workflow_node_dispatch_prepare_requires_binding_and_safe_probe_prompt`
- `workflow_node_dispatch_prepare_rejects_non_ready_work_item`
- `workflow_node_dispatch_started_marks_actual_dispatch_node_running`
- `workflow_node_dispatch_execute_uses_stub_and_advances_to_review`
- `legacy_real_execution_entrypoints_are_blocked_for_product_routing`
- `workflow_node_dispatch_execute_without_stub_stats_uses_native_readback`
- `workflow_node_dispatch_execute_rejects_user_reviewed_instruction_without_payload`
- `workflow_node_dispatch_execute_user_reviewed_instruction_uses_codex_options`
- `workflow_node_dispatch_readback_restores_user_reviewed_instruction_payload`
- `workflow_node_dispatch_user_reviewed_failure_writes_control_and_attempt`
- `workflow_node_dispatch_user_reviewed_timeout_writes_timed_out_attempt`
- `workflow_node_dispatch_user_reviewed_instruction_validates_permission_fields`

### workflow machine / director review / offline role 5

- `workflow_machine_runs_four_role_loop_to_acceptance`
- `workflow_dispatch_director_review_records_completed_dispatch`
- `offline_role_orchestration_records_dispatch_handoff_and_review`
- `offline_role_dispatch_rejects_missing_ready_work_item`
- `offline_role_dispatch_rejects_duplicate_prepared_dispatch`

## 3. Deferred 1 复评

Deferred 测试：

- `compact_last_message_summary_preserves_workflow_machine_control_marker`

按 R2-T0 原口径复评：

- 该测试是纯函数测试，调用 `compact_last_message_summary` 和 `workflow_machine_final_acceptance`，不读写 store，不构造 runner，不调用 `CodexResumeRunner`，不触发真实执行边界。
- 它不命中 R2-T0 中“workflow machine / director review / offline role 中依赖 runner fixture 的端到端组”这个字面禁迁理由。
- 但它断言的是 workflow machine 控制标记语义，紧贴仍禁迁的 `workflow_machine_runs_four_role_loop_to_acceptance`；单独迁移只有约十行收益，不能构成一个合理的 R2 下降轨道。

确认裁决：

```text
保持 deferred，不升级为禁迁，也不单独立项迁移。
```

复评触发点：

- 未来出现一个能降低 `lib.rs` 至少 250 行的 R2 迁移包，且该包本身需要处理 `compact_last_message_summary` / workflow machine 控制标记附近测试时，可以把该 deferred 作为同包附带项复评。
- workflow machine / shared stub runner / test support 底座被重新设计并通过复核后，可以重新评估。
- 若用户明确要求继续追求 `lib.rs <= 3,000` 的 R2 新下降轨道，必须先写新的 R2 后段评估任务包，而不是直接搬该测试。

## 4. 明确下降轨道规则

本文生效后，后续 R2 包必须同时满足：

- 明确降低 `lib.rs` 棘轮指标，任务包写出预计下降行数。
- 不迁移禁迁 34。
- 不把 deferred 1 单独立项为低收益包。
- 不改产品函数签名、可见性、语义或测试断言。
- 不新增 public API 只为搬测试。
- 不触碰 UI/CSS/TS、DB/schema、sidecar schema、workflow state 顶层结构。
- 不执行真实 `codex exec` / `codex exec resume`，不发送 prompt，不读写 `/Users/yoyi/.codex`。

建议最低收益线：

- 常规 R2 迁移包预计应让 `lib.rs` 下降至少 250 行。
- 低于 250 行的迁移只能并入其他能达标的同域包，不单独开包。

## 5. 后续建议车道

建议下一车道切到 R4 硬目标，顺序为：

1. `types.ts` 分域。
2. `WorkbenchSnapshot` 按页查询先行。
3. 后续再进入 ProjectsView / AgentView 按目标布局区块拆分。

并行准备：

- P2-1 / R3 Level B 窗口计划文档，只写计划，不执行 Level B。

## 6. 不接受为

本文不接受为：

- 用户已最终裁决车道。
- R2 已完成。
- `lib.rs <= 3,000` 已达成。
- 禁迁 34 已永久不可再评估。
- deferred 1 已永久关闭。
- R3 Level B 已执行或已排期。
- R4 硬目标已开始实现。
- 真实 Codex 执行、`.codex` 接触、多 agent 并行真实执行或 backlog 功能解冻。
