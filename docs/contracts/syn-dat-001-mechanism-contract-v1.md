---
contract_id: syn-dat-001-mechanism-contract-v1
version: 1
status: FROZEN_V1
evidence_level: STATIC_OPENING_ONLY
schema_authority: m2_transaction_foundation_authority
dependencies: ["event-audit-outbox-v1", "command-v1", "identity-scope-v1"]
hold_refs: ["HOLD-DB-JSON-RUNTIME-TRUTH", "HOLD-UNKNOWN-QUARANTINE-STORE", "HOLD-DB-BLOCKED-WRITE", "HOLD-RAW-TRANSCRIPT-RETENTION"]
---

# SYN-DAT-001: 事务底座机制合同 v1

## 合同概述

本文档冻结 M2 阶段事务底座的持久化与运行时状态机，为 DAT-003—006 提供统一的合同基础。本文档不修改生产 schema，仅定义接口、状态机、约束和语义。

## 1. 冻结的 reference_slice_id

**reference_slice_id**: `workflow-state-sidecar`
**aggregate**: `workflow_state`
**command**: `update_work_item_state`

选择理由：
1. workflow domain 相对独立，数据量可控
2. 现有 sidecar `workflow-state.v0.json` 已有清晰的读写入口
3. M1 已验证 `update_work_item_state` 命令的运行时行为
4. 无真实外部副作用，适合 vertical slice 验证

## 2. 事务状态机冻结

### 2.1 Command Receipt 持久化状态机

**domain_owner**: `application_command_receipt_ledger`

**legal_states**: `DENIED`, `NEEDS_CONFIRMATION`, `COMMITTED`, `EXTERNAL_PENDING`, `EXTERNAL_RESULT`, `PROJECTION_DEGRADED`, `FAILED`

**transitions**:
- `NEEDS_CONFIRMATION` → `COMMITTED`: commit_domain_fact (guards: unit_of_work_success, all_mutations_atomic)
- `NEEDS_CONFIRMATION` → `DENIED`: record_denial (guards: policy_violation, scrubbed_denial_record)
- `COMMITTED` → `EXTERNAL_PENDING`: declare_external_effect_intent (guards: outbox_item_declared, effect_id_unique)
- `EXTERNAL_PENDING` → `EXTERNAL_RESULT`: result_command_received (guards: effect_id_match, idempotent_result)
- `COMMITTED` → `PROJECTION_DEGRADED`: projection_failure (guards: domain_fact_preserved, error_receipt_created)

**persistence_rules**:
- `receipt_id`: uuid_v7
- `command_id`: immutable_after_creation
- `idempotency_key`: unique_per_command_type
- `request_hash`: sha256_of_normalized_payload
- `actor_id`: server_resolved
- `scope_ref`: immutable_after_creation
- `current_object_ref`: snapshot_at_admission
- `policy_decision_ref`: immutable_after_decision
- `status`: terminal_after_commit_or_deny
- `correlation_id`: chain_link
- `accepted_at`: server_timestamp
- `result_ref`: optional_reference_to_result
- `result_hash`: optional_sha256_of_result
- `committed_revision`: optional_optimistic_lock
- `error_code`: optional_error_identifier

### 2.2 Unit of Work (UoW) 状态机

**domain_owner**: `unit_of_work_coordinator`

**legal_states**: `DECLARED`, `IN_PROGRESS`, `COMMITTED`, `ROLLED_BACK`, `FAILED`

**transitions**:
- `DECLARED` → `IN_PROGRESS`: begin_uow (guards: single_writer_check, no_concurrent_uow_on_aggregate)
- `IN_PROGRESS` → `COMMITTED`: commit_uow (guards: all_mutations_valid, event_audited, outbox_declared, receipt_persisted)
- `IN_PROGRESS` → `ROLLED_BACK`: rollback_uow (guards: pre_commit_failure, no_domain_mutation_persisted)
- `IN_PROGRESS` → `FAILED`: uow_failure (guards: post_commit_failure, domain_mutation_persisted_but_receipt_lost)

**atomicity_guarantee**: domain_state_event_audit_outbox_receipt_all_or_none

**failure_behavior**: pre_commit_failure_rolls_back_all_mutations

**single_writer_rule**: one_aggregate_one_uow_one_writer_at_a_time

### 2.3 Event Envelope 持久化状态机

**domain_owner**: `event_ledger_repository`

**legal_states**: `DECLARED`, `PERSISTED`, `REPLAYED`, `FAILED`

**transitions**:
- `DECLARED` → `PERSISTED`: commit_event (guards: uow_committed, event_id_unique, schema_version_valid)
- `PERSISTED` → `REPLAYED`: replay_event (guards: idempotent_replay, correlation_chain_valid)
- `PERSISTED` → `FAILED`: event_corruption (guards: source_ref_valid, original_event_preserved)

**persistence_rules**:
- `event_id`: uuid_v7
- `event_type`: semantic_type_string
- `occurred_at`: server_timestamp
- `actor_id`: server_resolved
- `scope_ref`: immutable_after_creation
- `source_ref`: source_domain_reference
- `source_revision`: source_domain_revision
- `command_id`: link_to_command_receipt
- `correlation_id`: chain_link
- `causation_id`: causal_chain_reference
- `trace_context`: optional_opentelemetry
- `schema_version`: semantic_version
- `sensitivity`: public_internal_restricted_secret
- `summary_ref`: reference_to_summary
- `payload_ref`: reference_to_payload
- `payload_hash`: sha256_of_payload

**forbidden_material**: raw_transcript, prompt, tool_output, secret, credential, token

### 2.4 Audit Record 持久化状态机

**domain_owner**: `audit_ledger_repository`

**legal_states**: `DECLARED`, `PERSISTED`, `REVIEWED`, `ARCHIVED`

**transitions**:
- `DECLARED` → `PERSISTED`: commit_audit (guards: scrubbed, no_sensitive_material, source_refs_valid)
- `PERSISTED` → `REVIEWED`: review_audit (guards: reviewer_authorized, review_timestamp)
- `REVIEWED` → `ARCHIVED`: archive_audit (guards: retention_period_met, no_pending_investigations)

**persistence_rules**:
- `audit_id`: uuid_v7
- `action`: allowed_denied_committed_degraded_quarantined
- `decision`: scrubbed_decision_summary
- `reason_code`: semantic_reason_code
- `actor_id`: server_resolved
- `scope_ref`: immutable_after_creation
- `subject_ref`: reference_to_subject
- `command_id`: link_to_command_receipt
- `correlation_id`: chain_link
- `occurred_at`: server_timestamp
- `sensitivity`: public_internal_restricted_secret
- `scrub_result`: scrubbing_metadata
- `source_refs`: source_domain_reference

**forbidden_material**: raw_content, original_value, secret, credential

### 2.5 Outbox Item 持久化状态机

**domain_owner**: `outbox_repository`

**legal_states**: `DECLARED`, `AVAILABLE`, `LEASED`, `DELIVERED`, `RETRY_WAIT`, `POISON`, `CANCELLED`, `RESULT_RECEIVED`

**transitions**:
- `DECLARED` → `AVAILABLE`: commit_uow (guards: owning_command_committed, receipt_status_external_pending)
- `AVAILABLE` → `LEASED`: claim_outbox_item (guards: claimer_authorized, lease_token_unique, not_expired)
- `LEASED` → `DELIVERED`: deliver_external_effect (guards: effect_executed, result_command_type_valid)
- `LEASED` → `RETRY_WAIT`: delivery_failure (guards: retry_count_under_limit, backoff_calculated)
- `RETRY_WAIT` → `AVAILABLE`: retry_wait_expired (guards: backoff_period_elapsed)
- `LEASED` → `POISON`: permanent_failure (guards: retry_count_exceeded, manual_intervention_required)
- `DECLARED` → `CANCELLED`: cancel_outbox_item (guards: cancel_authorized, not_yet_leased)
- `DELIVERED` → `RESULT_RECEIVED`: result_command_received (guards: effect_id_match, idempotent_result)

**persistence_rules**:
- `outbox_item_id`: uuid_v7
- `owning_command_id`: immutable_after_creation
- `owning_command_receipt_ref`: immutable_after_creation
- `effect_id`: unique_per_outbox_item
- `capability_id`: reference_to_capability
- `scope_ref`: immutable_after_creation
- `subject_ref`: reference_to_subject
- `payload_ref`: reference_to_payload
- `payload_hash`: sha256_of_payload
- `result_command_type`: command_type_for_result
- `idempotency_key`: unique_per_effect
- `correlation_id`: chain_link
- `status`: managed_by_state_machine
- `created_at`: server_timestamp

**lease_rules**:
- `lease_duration_seconds`: configurable_default_300
- `max_retry_count`: configurable_default_3
- `backoff_strategy`: exponential_with_jitter
- `poison_threshold`: retry_count_exceeded

### 2.6 Current Snapshot 持久化状态机

**domain_owner**: `source_domain_projector`

**legal_states**: `STALE`, `CURRENT`, `REBUILDING`, `FAILED`

**transitions**:
- `STALE` → `CURRENT`: apply_new_event (guards: event_source_watermark_advanced, snapshot_hash_changed)
- `STALE` → `REBUILDING`: rebuild_snapshot (guards: source_state_available, projector_id_valid)
- `REBUILDING` → `CURRENT`: rebuild_complete (guards: rebuild_hash_matches_source, no_data_loss)
- `REBUILDING` → `FAILED`: rebuild_failure (guards: error_receipt_created, source_state_preserved)

**persistence_rules**:
- `object_ref`: reference_to_aggregate
- `object_revision`: optimistic_lock_version
- `source_watermark`: event_watermark_at_snapshot
- `snapshot_hash`: sha256_of_snapshot_content
- `projector_id`: projector_identifier
- `built_at`: server_timestamp

**rebuild_semantics**: snapshot_can_be_rebuilt_from_authoritative_state_and_events

### 2.7 Projection Checkpoint 持久化状态机

**domain_owner**: `PROJECTOR_ID`

**legal_states**: `IDLE`, `ADVANCING`, `CAUGHT_UP`, `DEGRADED`, `FAILED`

**transitions**:
- `IDLE` → `ADVANCING`: start_projection (guards: projector_id_valid, source_watermark_known)
- `ADVANCING` → `CAUGHT_UP`: projection_caught_up (guards: no_new_events, snapshot_current)
- `ADVANCING` → `DEGRADED`: projection_failure (guards: domain_fact_preserved, error_receipt_created)
- `DEGRADED` → `ADVANCING`: projection_recovery (guards: failure_resolved, recovery_checkpoint_set)
- `FAILED` → `IDLE`: projector_reset (guards: manual_intervention, checkpoint_can_be_discarded)

**persistence_rules**:
- `projector_id`: projector_identifier
- `projector_version`: semantic_version
- `last_event_id`: last_processed_event
- `source_watermark`: event_watermark_at_checkpoint
- `status`: managed_by_state_machine
- `error_receipt_ref`: optional_error_reference
- `updated_at`: server_timestamp

**checkpoint_semantics**: checkpoint_can_be_discarded_and_projector_can_be_rebuilt_from_source

## 3. 外键/唯一/索引约束冻结

### 3.1 必须的唯一约束

| 表 | 唯一约束 | 理由 |
|---|---|---|
| command_receipts | (command_id, idempotency_key) | 幂等键唯一性 |
| command_receipts | receipt_id | 主键唯一性 |
| events | event_id | 主键唯一性 |
| events | (command_id, event_type, event_id) | 同一命令的事件链唯一性 |
| audit_records | audit_id | 主键唯一性 |
| audit_records | (command_id, audit_id) | 同一命令的审计记录唯一性 |
| outbox_items | outbox_item_id | 主键唯一性 |
| outbox_items | (owning_command_id, effect_id) | 同一命令的效果唯一性 |
| outbox_items | (effect_id, idempotency_key) | 效果幂等键唯一性 |
| current_snapshots | (object_ref, projector_id) | 每个对象的每个投影器一个快照 |
| projection_checkpoints | (projector_id, projector_version) | 每个投影器版本一个检查点 |

### 3.2 必须的外键约束

| 表 | 外键 | 参考 | 级联 |
|---|---|---|---|
| command_receipts | command_id | commands.command_id | RESTRICT |
| events | command_id | command_receipts.command_id | RESTRICT |
| events | correlation_id | correlation_chains.correlation_id | RESTRICT |
| audit_records | command_id | command_receipts.command_id | RESTRICT |
| outbox_items | owning_command_id | command_receipts.command_id | RESTRICT |
| outbox_items | owning_command_receipt_ref | command_receipts.receipt_id | RESTRICT |
| current_snapshots | projector_id | projectors.projector_id | RESTRICT |
| projection_checkpoints | projector_id | projectors.projector_id | RESTRICT |

### 3.3 必须的索引

| 表 | 索引 | 列 | 用途 |
|---|---|---|---|
| command_receipts | idx_receipt_status | status | 状态查询 |
| command_receipts | idx_receipt_actor | actor_id | 行为者查询 |
| events | idx_event_type | event_type | 事件类型查询 |
| events | idx_event_occurred | occurred_at | 时间范围查询 |
| events | idx_event_scope | scope_ref | 范围查询 |
| audit_records | idx_audit_action | action | 操作类型查询 |
| audit_records | idx_audit_occurred | occurred_at | 时间范围查询 |
| outbox_items | idx_outbox_status | status | 状态查询 |
| outbox_items | idx_outbox_lease | expires_at | 租约过期查询 |
| current_snapshots | idx_snapshot_watermark | source_watermark | 水印查询 |
| projection_checkpoints | idx_checkpoint_watermark | source_watermark | 水印查询 |

## 4. Receipt 丢失、Lease、Quarantine、重建、Rollback 语义

### 4.1 Receipt 丢失语义

**scenario**: commit_succeeds_but_receipt_lost

**detection**: command_id_not_found_in_receipt_ledger

**recovery_options**:
- **REPLAY_COMMAND**: 重新执行命令，使用相同 idempotency_key
  - guards: idempotency_key_unique, request_hash_matches, no_side_effects_before_commit
  - outcome: existing_receipt_returned_if_found
- **FAIL_CLOSED**: 拒绝进一步操作，要求人工干预
  - guards: cannot_determine_command_outcome
  - outcome: command_marked_as_unrecoverable

**prevention**: receipt_persistence_must_be_atomic_with_domain_mutation

### 4.2 Lease 语义

**lease_duration_seconds**: 300
**max_lease_extensions**: 2

**lease_acquisition**:
- required: item_status_available, no_active_lease, claimer_authorized
- atomic: lease_token_generation_and_status_update_must_be_atomic

**lease_expiry**:
- detection: expires_at_less_than_current_timestamp
- recovery: lease_returns_to_available_status
- poison_detection: retry_count_exceeded_threshold

**lease_violations**:
- **duplicate_lease**: lease_token_already_exists_for_item → reject_new_lease_request
- **lease_held_too_long**: current_timestamp_exceeds_max_lease_duration → lease_force_expired, item_available

### 4.3 Quarantine 语义

**quarantine_triggers**:
- **unknown_input**: input_cannot_be_classified → block_and_quarantine_ref_only → requires_manual_classification
- **corrupt_input**: input_fails_integrity_check → block_and_preserve_value_free_provenance → requires_manual_repair_or_rebuild
- **sensitive_input**: input_contains_forbidden_material → scrub_and_stop_before_ordinary_store → requires_scrubbing_and_reclassification

**quarantine_storage**:
- content: reference_only_no_original_values
- metadata: reason_code_scope_observed_at_resolution_state
- retention: indefinite_until_resolution

**quarantine_resolution**:
- reclassify_to_known_type
- rebuild_from_source
- delete_with_audit_trail
- hold_indefinitely

### 4.4 重建语义

**rebuild_triggers**: projector_failure, snapshot_corruption, parity_mismatch, manual_recovery

**rebuild_sources**:
1. **authoritative_domain_state**: current_aggregate_state_if_available (priority: 1)
2. **event_ledger**: replay_all_events_for_aggregate (priority: 2)
3. **legacy_sidecar**: compatibility_projection_if_no_other_source (priority: 3)

**rebuild_guarantees**:
- rebuild_produces_same_hash_as_original
- rebuild_is_deterministic
- rebuild_does_not_lose_data
- rebuild_creates_new_checkpoint

**rebuild_forbidden**:
- rebuild_from_quarantined_data
- rebuild_from_scrubbed_material
- rebuild_from_external_untrusted_source

### 4.5 Rollback 语义

**rollback_triggers**: parity_mismatch_unresolved, corruption_detected, manual_intervention_required, cutover_rejected

**rollback_guarantees**:
- rollback_is_non_destructive
- rollback_retains_last_known_primary_position
- rollback_retains_shadow_evidence
- rollback_blocks_on_unresolved_quarantine
- rollback_blocks_on_unresolved_join_violations
- rollback_blocks_on_unresolved_redaction_violations

**rollback_forbidden**:
- rollback_does_not_replay_command_without_original_keys
- rollback_does_not_recover_scrubber
- rollback_does_not_restore_dual_primary_write
- rollback_does_not_delete_old_records

**rollback_ownership**: each_domain_cutover_has_independent_rollback_owner
**rollback_authorization**: user_visible_cutover_and_deletion_require_explicit_authorization

## 5. 安全 Payload Storage、Payload-ref 完整性、Retention/Scrub 规则

### 5.1 Payload Storage 规则

**storage_principle**: payload_body_outside_boundary_reference_only

**allowed_references**:
- **CONTENT_HASH**: sha256_hex (hash_of_payload_content)
- **STORAGE_REFERENCE**: opaque_reference (reference_to_external_storage_location)

**forbidden_storage**:
- raw_transcript
- prompt_content
- tool_output
- secret_value
- credential_token
- provider_response_full
- stdout_content
- stderr_content

**payload_ref_integrity**:
- validation: hash_must_match_reference_content
- recovery: reference_invalid_or_hash_mismatch_quarantine
- audit: integrity_check_result_recorded_in_audit

### 5.2 Retention 规则

| 类别 | 保留期 | 理由 | 删除策略 |
|---|---|---|---|
| COMMAND_RECEIPTS | 无限期 | immutable_command_chain | never_delete_without_user_authorization |
| EVENTS | 无限期 | event_ledger_is_source_of_truth | never_delete_without_event_chain_integrity_preservation |
| AUDIT_RECORDS | 最少1年 | compliance_and_investigation | archive_after_retention_period_with_audit_trail |
| OUTBOX_ITEMS | 直到送达或中毒 | external_effect_delivery | delete_after_successful_delivery_or_manual_poison_resolution |
| CURRENT_SNAPSHOTS | 直到下一个快照 | current_read_model | replace_with_newer_snapshot |
| PROJECTION_CHECKPOINTS | 无限期 | projection_recovery_point | can_be_discarded_and_projector_rebuilt |

### 5.3 Scrub 规则

**scrub_triggers**:
- sensitive_material_detected
- raw_content_in_event
- credential_in_payload
- secret_in_audit

**scrub_actions**:
- **REPLACE_WITH_REFERENCE**: replace_sensitive_value_with_hash_reference
  - preservation: original_value_never_stored_in_ordinary_tables
- **OMIT_FIELD**: completely_remove_field_from_output
  - preservation: field_existence_recorded_in_audit
- **REDACT_CONTENT**: replace_content_with_redacted_placeholder
  - preservation: redaction_metadata_recorded_in_audit

**scrub_invariants**:
- scrub_is_always_applied_before_persistence
- scrub_result_is_always_recorded_in_audit
- scrub_cannot_be_reversed
- scrub_cannot_be_overridden_by_caller
- scrub_applies_to_all_sensitive_material_regardless_of_source

## 6. 四类现状路径纸面追踪

### 6.1 Conversation 路径追踪

| 维度 | 答案 |
|---|---|
| **Owner** | conversation transport (role-session-v1) |
| **事务边界** | 每个 turn 独立事务；session 生命周期跨多个 turn |
| **外部 effect** | 调用外部 AI provider API；结果通过 result command 回写 |
| **失败残留** | 半状态 turn（请求已发但未收到响应）；进程夹具环境性失败 |
| **恢复动作** | 重启后检查未完成的 turn；根据 idempotency_key 决定重试或放弃 |

### 6.2 Workflow 路径追踪

| 维度 | 答案 |
|---|---|
| **Owner** | project workflow (project-orchestration-v1) |
| **事务边界** | 每个命令原子更新 workflow state + event + audit + outbox |
| **外部 effect** | 无直接外部 effect；通过 outbox 触发外部执行 |
| **失败残留** | workflow state 半更新（state 已改但 event 未写）；执行夹具环境性失败 |
| **恢复动作** | rollback 到已知 good state；重建 workflow snapshot；重放 events |

### 6.3 Memory 路径追踪

| 维度 | 答案 |
|---|---|
| **Owner** | memory governance (memory-personal-model-v1) |
| **事务边界** | 每个 capture/observation/candidate 独立事务 |
| **外部 effect** | 无直接外部 effect；通过 promotion policy 触发内部处理 |
| **失败残留** | candidate 半处理（已 capture 但未 promote）；scrub 未完成 |
| **恢复动作** | quarantine 未分类数据；重新 scrub；重新 classification |

### 6.4 Knowledge 路径追踪

| 维度 | 答案 |
|---|---|
| **Owner** | knowledge index (object-ref-navigation-v1) |
| **事务边界** | 每个文件操作独立事务；索引重建批量处理 |
| **外部 effect** | 文件系统读写；通过 file-truth adapter 处理 |
| **失败残留** | 索引与文件不一致（文件已改但索引未更新）；path 解析失败 |
| **恢复动作** | 重建索引；重新扫描文件系统；quarantine 无效 path |

## 7. 合同验证要求

### 7.1 验证层级

| 层级 | 必须证明 | 不能声称 |
|---|---|---|
| Contract/schema lint | owner、FK、状态、禁止字段、migration 顺序一致 | repository 已正确实现 |
| Unit/property | UoW、幂等、scrub、lease、projector 确定性 | 生产入口全接入 |
| Temp SQLite/fixture | rollback、crash point、parity、quarantine、重建 | live store 已迁移 |
| Non-test build | production path 可构建 | App 行为正确 |
| Isolated Tauri | scratch store 冷启动/强退/重启/恢复可见 | 真实数据、provider 或发布通过 |

### 7.2 关键机械断言

1. commit 前任一点失败全部回滚
2. commit 后重试不重复外部动作
3. 投影失败有 durable receipt
4. raw JSON 默认不进入产品 DTO
5. 旧/new count、key、canonical hash 可解释

## 8. 合同冻结声明

本文档冻结了 M2 阶段事务底座的机制合同。所有状态机、约束、语义和规则已冻结，可供 DAT-003—006 实现使用。本文档不授权生产 schema 修改、真实数据迁移或产品代码变更。

**冻结日期**: 2026-08-03
**冻结者**: M2 执行线
**验证状态**: 静态合同冻结，待 DAT-002 实现验证
