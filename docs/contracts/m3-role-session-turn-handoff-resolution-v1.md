---
contract_id: m3-role-session-turn-handoff-resolution-v1
version: 1
status: FROZEN_M3_IMPLEMENTATION_RESOLUTION_V1
evidence_level: CONTRACT_AND_SOURCE_RESOLUTION_ONLY
extends:
  - role-session-v1
  - handoff-v1
stage: M3
leaf: M3C01
---

# M3 RoleSession / Turn / Handoff 实施补充合同 v1

状态：**M3 实施补充合同 v1 已冻结；只冻结 M3 的实现解释和迁移边界，不改写 M1 冻结合同，也不证明任何产品代码已经实现。**

本文件补充 [M1 RoleSession 合同](role-session-v1.md)、[M1 Handoff 合同](handoff-v1.md)和[M3 阶段计划](../plans/2026-08-01-syn-stage-3-role-session-and-explicit-handoff-plan-v1.md)。产品方向与资料最小充分原则仍分别以[产品正本](../product/syn-product-canon-v1.md)和[知识基础设施正本](../product/knowledge-infrastructure-canon-v1.md)为准。

## 1. 效力与解析规则

- M1 的对象名、合法状态、owner、失败语义和 `FROZEN_V1` 文本保持原样；本文件不得修改、覆盖或重新解释其历史验收。
- 本文件唯一可供程序消费的主体是下方**唯一一个**标注为 `json m3-resolution-v1` 的代码块。消费者必须拒绝缺失、重复、非 JSON 或 `format` 不相等的内容。
- `FROZEN_M3_IMPLEMENTATION_RESOLUTION_V1` 只表示 M3C01 已冻结后续实现判断；它不形成真实服务提供方（provider）、真实消息、真实账号、凭据、外部连接或发布权限。
- 文中 `orphaned`、`ambiguous` 和 `collision` 是原因/处置分类，**不是**对 M1 `RoleSession` 或 `Turn` 新增状态值。必须映射回 M1 允许的 `SUSPENDED`、`FAILED` 或 `QUARANTINED`。

```json m3-resolution-v1
{
  "format": "syn.m3.role-session-turn-handoff-resolution/v1",
  "machine_block_tag": "m3-resolution-v1",
  "contract_id": "m3-role-session-turn-handoff-resolution-v1",
  "version": 1,
  "status": "FROZEN_M3_IMPLEMENTATION_RESOLUTION_V1",
  "scope": {
    "stage": "M3",
    "leaf": "M3C01",
    "implementation_mode": "offline_contract_resolution_only",
    "default": "fail_closed"
  },
  "frozen_inputs": [
    {
      "path": "docs/contracts/role-session-v1.md",
      "sha256": "77c82932e728d4982ebb501b167f274cc31d2076957602771904d96dc399b2ca",
      "rule": "read_only_m1_contract"
    },
    {
      "path": "docs/contracts/handoff-v1.md",
      "sha256": "3378f02f5dfb06e4db39125b5828eeda9440fc2c25ddbee3fe4e951fa6c386bf",
      "rule": "read_only_m1_contract"
    },
    {
      "path": "docs/contracts/identity-scope-v1.md",
      "sha256": "3cb0073c0fffc2423e3450ce9d9e3c683065cdd075bf618e0d406cc1475e3ea4",
      "rule": "read_only_m1_contract"
    },
    {
      "path": "docs/contracts/event-audit-outbox-v1.md",
      "sha256": "15a24d8040da054794e340fe7839b273dce0f60a2c1708513d1b998c8e968e99",
      "rule": "read_only_m1_contract"
    },
    {
      "path": "docs/plans/2026-08-01-syn-stage-3-role-session-and-explicit-handoff-plan-v1.md",
      "sha256": "9403851ece470c32bac5071e2613495a6f0e525214dbd6990a1cd2d28d1ce013",
      "rule": "m3_execution_boundary_snapshot; later status-only closeout edits do not change this resolution, semantic boundary edits require explicit contract review"
    },
    {
      "path": "docs/product/syn-product-canon-v1.md",
      "sha256": "b95a8db131ae8e4f1f79aaafc426a66f20e79ea840b0665ba9df7b3ee1695efa",
      "rule": "product_direction"
    },
    {
      "path": "docs/product/knowledge-infrastructure-canon-v1.md",
      "sha256": "92ab5fb2d80f686278facea6679f674ee64c8639da695fd76639cf8b1127e829",
      "rule": "minimum_sufficient_context"
    }
  ],
  "source_evidence": [
    {
      "path": "prototypes/productized-desktop-shell/src-tauri/src/mcp/identity_kernel.rs",
      "use": "server_resolved_identity_scope_role_channel_permission_snapshot_source",
      "opening_limit": "staged_not_connected_to_tauri_commands"
    },
    {
      "path": "prototypes/productized-desktop-shell/src-tauri/src/mcp/supervisor_conversation_binding.rs",
      "use": "durable_per_turn_supervisor_binding_source",
      "opening_limit": "not_a_general_rolesession_and_not_a_new_storage_owner"
    },
    {
      "path": "prototypes/productized-desktop-shell/src-tauri/src/manual_relay/conversation_transport.rs",
      "use": "legacy_profiled_transport_adapter_source",
      "opening_limit": "adapter_does_not_own_scope_or_authorization"
    },
    {
      "path": "prototypes/productized-desktop-shell/src-tauri/src/codex_db.rs",
      "use": "read_only_provider_thread_catalog_source",
      "opening_limit": "catalog_presence_is_not_owner_binding_or_resume_proof"
    },
    {
      "path": "prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_repository.rs",
      "use": "bounded_immediate_transaction_engine_and_m2_sidecar_boundary_source",
      "opening_limit": "workflow_sidecar_is_not_a_general_m3_repository_or_unit_of_work_port"
    },
    {
      "path": "prototypes/productized-desktop-shell/src/views/projects/jiaoban/useJiaobanConversationState.ts",
      "use": "frontend_process_local_cache_source",
      "opening_limit": "display_compatibility_only_not_migration_truth"
    }
  ],
  "owners": {
    "RoleSession": {
      "truth_owner": "conversation_domain",
      "write_owner": "m3_role_session_repository",
      "implementation_leaf": "M3C03"
    },
    "Turn": {
      "truth_owner": "role_session_aggregate",
      "write_owner": "m3_role_session_repository",
      "implementation_leaf": "M3C03_and_M3C04"
    },
    "ProviderHandle": {
      "truth_owner": "conversation_role_session_repository",
      "write_owner": "m3_role_session_repository",
      "implementation_leaf": "M3C03",
      "adapter_rule": "adapter_may_validate_and_use_handle_but_never_own_or_authorize_it"
    },
    "ConversationContext": {
      "truth_owner": "application_projection",
      "write_owner": "m3_context_projection",
      "implementation_leaf": "M3C03_and_M3C06",
      "rebuild_rule": "rebuildable_from_bounded_source_references_only"
    },
    "Handoff": {
      "truth_owner": "handoff_aggregate",
      "write_owner": "handoff_aggregate_through_m3_role_session_repository",
      "implementation_leaf": "M3C05",
      "source_result_rule": "source_owner_alone_applies_returned_result_through_a_new_command"
    }
  },
  "stable_keys": {
    "role_session_join": [
      "role_session_id",
      "role_ref",
      "scope_ref",
      "current_object_ref",
      "execution_channel"
    ],
    "server_resolved_binding_fields": [
      "actor_id",
      "role_ref",
      "scope_ref",
      "current_object_ref",
      "execution_channel",
      "permission_snapshot_ref",
      "owner_fingerprint"
    ],
    "owner_fingerprint": {
      "algorithm": "sha256",
      "domain_separator": "syn.m3.role-session-owner/v1",
      "encoding": "domain_separator_utf8_then_for_each_component_u32_big_endian_byte_length_followed_by_server_canonical_utf8_bytes_in_listed_order",
      "components": [
        "server_resolved_actor_id",
        "role_ref",
        "scope_ref",
        "current_object_ref",
        "execution_channel"
      ],
      "excluded_drift_fields": [
        "permission_snapshot_ref",
        "provider_handle_ref",
        "session_revision"
      ],
      "reason": "permission snapshots rotate or narrow and provider handles reverify without changing the immutable owner identity"
    },
    "role_session_create_idempotency": {
      "key_components": [
        "operation_kind",
        "server_resolved_actor_id",
        "request_idempotency_key"
      ],
      "immutable_request_fields": [
        "role_ref",
        "scope_ref",
        "current_object_ref",
        "execution_channel",
        "permission_snapshot_ref",
        "owner_fingerprint"
      ],
      "same_key_same_immutable_input": "return_original_receipt",
      "same_key_different_input": "reject_idempotency_key_reuse"
    },
    "turn_idempotency": {
      "key_components": [
        "role_session_id",
        "operation_kind",
        "request_idempotency_key"
      ],
      "immutable_request_fields": [
        "input_hash",
        "expected_session_revision",
        "conversation_context_ref",
        "provider_handle_ref"
      ],
      "same_key_same_immutable_input": "return_original_receipt_without_new_provider_effect",
      "same_key_different_input": "reject_idempotency_key_reuse"
    },
    "provider_handle_natural_key": {
      "components": [
        "provider_kind",
        "provider_namespace_ref",
        "provider_conversation_ref"
      ],
      "namespace_rule": "opaque_server_resolved_provider_instance_or_profile_namespace_not_client_supplied_not_a_credential_and_not_a_raw_endpoint; missing_or_unverifiable_namespace_is_ambiguous_and_cannot_bind",
      "normalization": "opaque_provider_identifier_no_case_fold_or_path_rewrite",
      "uniqueness_rule": "one_non_quarantined_handle_may_bind_to_one_owner_fingerprint_only",
      "conflict_rule": "same_natural_key_with_different_owner_fingerprint_is_collision_quarantine"
    },
    "handoff_idempotency": {
      "key_components": [
        "handoff_id",
        "operation_kind",
        "request_idempotency_key"
      ],
      "immutable_request_fields": [
        "handoff_revision",
        "actor_id",
        "recipient_ref",
        "decision_or_result_hash"
      ],
      "decision_rule": "recipient_decision_and_return_receipt_are_single_assignment_per_handoff_revision"
    }
  },
  "state_mappings": {
    "RoleSession": {
      "allowed_states_from_m1": [
        "CREATED",
        "ACTIVE",
        "SUSPENDED",
        "CLOSED",
        "QUARANTINED"
      ],
      "allowed_transitions": [
        "CREATED->ACTIVE",
        "CREATED->QUARANTINED",
        "ACTIVE->SUSPENDED",
        "ACTIVE->CLOSED",
        "ACTIVE->QUARANTINED",
        "SUSPENDED->ACTIVE",
        "SUSPENDED->CLOSED",
        "SUSPENDED->QUARANTINED"
      ],
      "recovery_dispositions": [
        {
          "condition": "restart_receipt_missing_or_unverifiable",
          "session_state": "SUSPENDED",
          "turn_state": "FAILED",
          "reason": "restart_orphan_requires_user_visible_resolution"
        },
        {
          "condition": "owner_scope_or_handle_mapping_ambiguous",
          "session_state": "QUARANTINED",
          "reason": "ambiguous_binding_never_guesses_project_or_role"
        }
      ]
    },
    "Turn": {
      "allowed_states_from_m1": [
        "ACCEPTED",
        "STARTING",
        "ACTIVE",
        "SUCCEEDED",
        "FAILED",
        "CANCELLED",
        "TIMED_OUT"
      ],
      "allowed_transitions": [
        "ACCEPTED->STARTING",
        "ACCEPTED->FAILED",
        "STARTING->ACTIVE",
        "STARTING->SUCCEEDED",
        "STARTING->FAILED",
        "STARTING->CANCELLED",
        "STARTING->TIMED_OUT",
        "ACTIVE->ACTIVE",
        "ACTIVE->SUCCEEDED",
        "ACTIVE->FAILED",
        "ACTIVE->CANCELLED",
        "ACTIVE->TIMED_OUT"
      ],
      "terminal_rule": "terminal_receipt_is_immutable_and_replay_returns_the_same_receipt"
    },
    "Handoff": {
      "allowed_transitions_from_m1": [
        "CREATED->ACCEPTED",
        "CREATED->REJECTED",
        "CREATED->CANCELLED",
        "CREATED->EXPIRED",
        "ACCEPTED->RETURN_PENDING",
        "RETURN_PENDING->RETURNED",
        "RETURN_PENDING->RETURN_FAILED",
        "RETURN_FAILED->RETURN_PENDING",
        "RETURN_FAILED->CANCELLED_BY_SOURCE"
      ],
      "accepted_never_expires": true,
      "returned_result_rule": "returned_receipt_is_not_source_object_mutation"
    }
  },
  "invariants": [
    {
      "id": "INV-M3-OWNER-001",
      "rule": "server_resolves_and_revalidates_actor_role_scope_current_object_channel_permission_and_owner_fingerprint_before_every_start_resume_accept_return_or_provider_effect"
    },
    {
      "id": "INV-M3-SCOPE-002",
      "rule": "client_supplied_role_scope_station_channel_profile_or_permission_claim_is_never_truth; default_is_no_cross_scope"
    },
    {
      "id": "INV-M3-HANDLE-003",
      "rule": "provider_handle_is_a_verified_reference_not_a_session_identity_or_permission_grant"
    },
    {
      "id": "INV-M3-EFFECT-004",
      "rule": "a_provider_effect_requires_durable_effect_registration_and_authoritative_readback; no_receipt_means_no_automatic_resume_or_resend"
    },
    {
      "id": "INV-M3-HANDOFF-005",
      "rule": "handoff_permission_request_is_a_request_never_a_grant_and_handoff_never_transfers_source_owner_mutation_rights"
    },
    {
      "id": "INV-M3-CONTEXT-006",
      "rule": "context_is_minimum_sufficient_reference_material; retrieval_hit_never_becomes_fact_memory_skill_enablement_or_permission_by_itself"
    },
    {
      "id": "INV-M3-NOCOPY-007",
      "rule": "default_is_no_copy_of_raw_transcript_prompt_provider_response_stdout_stderr_tool_arguments_or_credentials"
    }
  ],
  "idempotency": {
    "command_boundary": "persist_or_replay_a_scrubbed_command_receipt_before_any_adapter_effect",
    "request_fingerprint": "sha256_with_operation_specific_domain_separator_over_the_same_u32_big_endian_length_prefixed_encoding_of_all_immutable_request_fields",
    "effect_boundary": "same_verified_key_and_same_immutable_input_must_never_issue_a_second_provider_effect",
    "divergence_boundary": "same_key_with_different_input_hash_revision_recipient_or_result_hash_fails_closed",
    "restart_boundary": "only_a_matching_durable_attempt_receipt_allows_readback_or_resume; absence_is_not_permission_to_retry"
  },
  "sensitivity": {
    "may_persist": [
      "opaque_source_ref",
      "opaque_provider_handle_ref",
      "opaque_provider_namespace_ref",
      "owner_fingerprint",
      "input_hash",
      "result_hash",
      "scrubbed_summary_ref_when_allowed",
      "scrubbed_receipt_metadata"
    ],
    "must_not_persist": [
      "credential",
      "raw_transcript_body",
      "prompt_body",
      "provider_response_body",
      "stdout",
      "stderr",
      "tool_argument_body",
      "unrestricted_permission_material"
    ],
    "raw_retention": "HOLD-RAW-TRANSCRIPT-RETENTION_remains_unresolved_outside_M3C01"
  },
  "permission_drift": {
    "recheck_points": [
      "role_session_create",
      "role_session_resume",
      "turn_start",
      "turn_stop",
      "handoff_accept",
      "handoff_return",
      "provider_effect_dispatch"
    ],
    "same_or_narrower": "may_continue_only_after_new_server_resolved_snapshot_is_persisted_and_audited",
    "wider_scope_capability_or_side_effect": "suspend_session_and_require_new_independent_user_or_policy_grant",
    "mismatch_or_unknown": "fail_closed_without_provider_effect",
    "handoff_rule": "permission_request_never_bypasses_snapshot_revalidation"
  },
  "collision_orphan_restart": {
    "collision": {
      "trigger": "provider_handle_natural_key_maps_to_multiple_or_conflicting_owner_fingerprints",
      "action": "quarantine_binding_preserve_provenance_emit_scrubbed_audit",
      "forbidden": [
        "guess_project",
        "reuse_wrong_thread",
        "overwrite_existing_handle"
      ]
    },
    "orphan": {
      "trigger": "legacy_or_restart_record_lacks_exact_owner_scope_role_channel_or_receipt_proof",
      "action": "preserve_source_reference_mark_user_visible_orphan_or_ambiguous_and_do_not_auto_bind"
    },
    "restart": {
      "recover_only_if": [
        "durable_attempt_receipt_exists",
        "receipt_matches_role_session_turn_handle_owner_and_idempotency_key",
        "current_permission_revalidation_passes"
      ],
      "otherwise": "turn_failed_session_suspended_or_quarantined_without_silent_provider_resume_retry_or_resend"
    }
  },
  "minimum_conversation_context": {
    "owner": "application_projection",
    "required_reference_fields": [
      "role_session_id",
      "objective_ref",
      "scope_ref",
      "current_object_ref",
      "included_material_refs",
      "included_skill_refs",
      "source_watermark",
      "freshness_or_staleness_marker",
      "known_gaps",
      "known_conflicts_or_uncertainties",
      "excluded_material_refs_with_reason",
      "retrieval_status",
      "request_more_material_ref",
      "projection_version"
    ],
    "optional_reference_fields": [
      "scrubbed_summary_ref",
      "source_link_labels"
    ],
    "must_be_rebuildable": true,
    "retrieval_status_values": [
      "COMPLETE",
      "DEGRADED",
      "UNAVAILABLE",
      "NOT_REQUESTED"
    ],
    "excluded_material_reason_values": [
      "OUT_OF_SCOPE",
      "PERMISSION_DENIED",
      "STALE",
      "SUPERSEDED",
      "CONFLICTING",
      "IRRELEVANT",
      "SOURCE_UNAVAILABLE"
    ],
    "not_in_m3": [
      "complete_knowledge_retrieval",
      "external_knowledge_sync",
      "skill_discovery_or_enablement",
      "memory_packet_lifecycle",
      "raw_transcript_body"
    ],
    "missing_context_behavior": "declare_gap_and_offer_explicit_request_for_more_material_without_fabricating_context"
  },
  "handoff_mapping": {
    "from_m1": {
      "request_fields": [
        "from_role_session_id",
        "from_actor_id",
        "to_role_ref",
        "to_recipient_ref",
        "scope_ref",
        "requested_outcome_ref",
        "object_refs",
        "risk_class",
        "permission_request",
        "source_permission_snapshot_ref"
      ],
      "receipt_fields": [
        "handoff_id",
        "handoff_revision",
        "receipt_kind",
        "actor_id",
        "role_session_id",
        "status",
        "result_ref",
        "result_hash",
        "source_command_receipt_ref",
        "correlation_id"
      ]
    },
    "m3_rules": [
      "create_accept_reject_cancel_expire_return_and_retry_use_expected_revision_and_idempotency_key",
      "wrong_recipient_stale_revision_scope_mismatch_divergent_result_or_missing_original_object_fails_closed",
      "manual_offline_paste_is_unverified_claim_not_handoff_receipt_or_grant",
      "accepted_handoff_never_expires",
      "return_receipt_may_be_recorded_but_only_source_owner_new_command_applies_result_to_source_object"
    ]
  },
  "handoff_timeouts_and_retry": {
    "pre_accept": {
      "accept_by_required": true,
      "server_validation": "accept_by_must_be_a_valid_bounded_utc_instant_after_created_at; missing_invalid_or_unbounded_value_rejects_create",
      "after_deadline": "CREATED_may_transition_once_to_EXPIRED; accept_after_deadline_fails_closed",
      "cancel_rule": "only_source_owner_may_cancel_while_CREATED_with_expected_revision",
      "cas_winner_rule": "accept_reject_cancel_and_expire_compete_on_the_same_expected_revision_and_exactly_one_transition_wins",
      "cas_loser_rule": "a_loser_returns_a_stale_revision_receipt_referencing_the_winning_transition_and_never_reapplies_an_effect"
    },
    "accepted_and_return": {
      "accepted_never_expires": true,
      "request_return_rule": "ACCEPTED_to_RETURN_PENDING_requires_a_server_clock_validated_bounded_return_by_utc_instant_after_the_request_time_persisted_with_the_return_request_receipt",
      "return_timeout_rule": "at_return_by_an_explicit_idempotent_timeout_command_may_transition_RETURN_PENDING_to_RETURN_FAILED_with_timeout_reason; timeout_never_silently_cancels_accepted_work",
      "result_timeout_cas_rule": "return_result_and_return_timeout_compete_on_the_same_expected_revision_and_exactly_one_wins; the_loser_receives_a_stale_revision_receipt_referencing_the_winner",
      "late_result_rule": "a_result_arriving_after_timeout_is_not_auto_applied; source_may_explicitly_retry_RETURN_FAILED_to_RETURN_PENDING_with_a_new_bounded_return_by",
      "retry_rule": "RETURN_FAILED_to_RETURN_PENDING_requires_expected_revision_new_idempotency_key_new_bounded_return_by_and_the_same_bounded_outcome_scope_recipient; changing_recipient_scope_or_outcome_requires_a_new_handoff",
      "source_cancel_rule": "only_RETURN_FAILED_may_transition_to_CANCELLED_BY_SOURCE; accepted_or_return_pending_work_is_preserved_until_explicit_return_resolution"
    },
    "no_global_duration_default": "M3C01_does_not_invent_product_timeout_minutes; callers_supply_a_bounded_deadline_under_later_role_policy"
  },
  "persistence_bridge": {
    "m3_owner": "m3_role_session_repository",
    "m2_authority_surface": {
      "port_version": "workflow-state-sidecar.repository.m2.v1",
      "status": "bounded_workflow_state_reference_slice_only",
      "generic_m2_ports_status": "private_unwired_reference_candidates_not_public_M3_unit_of_work",
      "not_covered": [
        "live_workbench_data",
        "DAT_007_live_cutover",
        "real_account_or_release"
      ]
    },
    "allowed_low_level_reuse": [
      "WorkbenchSqliteRepository.with_immediate_transaction_busy_retry_engine",
      "shared_receipt_event_audit_physical_ledger_shapes_only_after_explicit_M3_field_and_owner_mapping"
    ],
    "required": [
      "M3_owned_versioned_schema_and_repository_port",
      "M3_owned_command_correlation_idempotency_and_revision",
      "same_transaction_M3_receipt_event_audit_and_domain_state",
      "fresh_scratch_schema_introspection_and_fail_closed_migration_tests"
    ],
    "forbidden_reuse": [
      "WorkflowStateSidecarRepositoryV1_as_RoleSession_repository",
      "with_m2_reference_command_transaction",
      "workflow_state_snapshot_or_projector",
      "M2_workflow_projection_checkpoint_or_recovery_owner",
      "M2_R4_fake_external_adapter_as_provider_or_outbox",
      "work_item_revision_as_session_or_handoff_revision"
    ],
    "boundary": "M2_workflow_state_sidecar_remains_a_bounded_reference_slice_and_never_becomes_the_M3_domain_owner"
  },
  "migration_matrix": [
    {
      "source": "Codex SQLite and rollout indexes",
      "classification": "SHADOW_ELIGIBLE_HANDLE_REFERENCE",
      "allowed_material": [
        "opaque_provider_conversation_ref",
        "opaque_provider_namespace_ref_when_host_verified",
        "source_ref",
        "source_hash",
        "verified_owner_fingerprint_if_present"
      ],
      "forbidden_material": [
        "raw_transcript_body",
        "credentials",
        "provider_response_body"
      ],
      "default_action": "import_only_to_isolated_shadow_repository_after_exact_namespace_owner_scope_validation; missing_namespace_is_ambiguous"
    },
    {
      "source": "durable supervisor conversation binding",
      "classification": "SHADOW_ELIGIBLE_PER_TURN_BINDING",
      "allowed_material": [
        "role_project_workflow_turn_refs",
        "thread_ref",
        "run_ref",
        "lifecycle_ref",
        "source_hash"
      ],
      "forbidden_material": [
        "user_message_snapshot_body",
        "capability_output_body",
        "new_storage_owner_assumption"
      ],
      "default_action": "treat_as_source_evidence_not_as_durable_rolesession"
    },
    {
      "source": "valid continuation records",
      "classification": "SHADOW_ELIGIBLE_RESUME_REFERENCE",
      "allowed_material": [
        "continuation_ref",
        "verified_handle_ref",
        "terminal_or_durable_attempt_receipt_ref",
        "source_hash"
      ],
      "default_action": "candidate_only_then_revalidate_current_policy_before_bind_or_resume"
    },
    {
      "source": "legacy manual relay and conversation transport records",
      "classification": "ADAPTER_ONLY",
      "allowed_material": [
        "bounded_compatibility_reference",
        "receipt_reference"
      ],
      "default_action": "retain_as_guarded_adapter_without_migration_truth_or_scope_authority"
    },
    {
      "source": "Jiaoban and Agent Center module_or_React_cache",
      "classification": "DISPLAY_ONLY_PARITY_TELEMETRY",
      "allowed_material": [
        "same_process_display_parity_signal"
      ],
      "default_action": "never_import_as_rolesession_turn_owner_scope_permission_or_handle_truth"
    },
    {
      "source": "raw transcript or provider response body",
      "classification": "NO_COPY_GLOBAL_RETENTION_HOLD",
      "allowed_material": [
        "opaque_reference",
        "allowed_scrubbed_summary_reference",
        "content_hash_when_policy_allows"
      ],
      "default_action": "do_not_copy_or_resolve_retention_in_m3",
      "named_hold": "HOLD-RAW-TRANSCRIPT-RETENTION"
    },
    {
      "source": "any unmatched thread or record",
      "classification": "ORPHAN_OR_AMBIGUOUS",
      "allowed_material": [
        "provenance_ref",
        "source_hash",
        "failure_reason"
      ],
      "default_action": "preserve_and_quarantine_without_auto_project_assignment"
    }
  ],
  "rollback": {
    "allowed": [
      "switch_ui_to_legacy_read_fallback",
      "disable_new_m3_read_projection",
      "preserve_m3_provenance_and_receipts_for_review",
      "keep_export_or_manifest_until_M9_retirement"
    ],
    "forbidden": [
      "remove_m1_thread_owner_scope_or_station_3b_guard",
      "replay_provider_effect",
      "restore_cross_project_bypass",
      "promote_frontend_cache_to_truth_owner",
      "delete_or_overwrite_unresolved_orphans"
    ]
  },
  "implementation_sequence": [
    {
      "leaf": "M3C02",
      "responsibility": "server_owner_scope_guard_before_spawn"
    },
    {
      "leaf": "M3C03",
      "responsibility": "repository_schema_and_shadow_import"
    },
    {
      "leaf": "M3C04",
      "responsibility": "transport_port_and_fake_provider_restart_semantics"
    },
    {
      "leaf": "M3C05",
      "responsibility": "explicit_handoff_state_machine_and_return_receipts"
    },
    {
      "leaf": "M3C06",
      "responsibility": "read_model_and_frontend_cache_demoted_to_compatibility_display"
    },
    {
      "leaf": "M3C07",
      "responsibility": "isolated_desktop_layered_acceptance_with_fake_provider_only"
    },
    {
      "leaf": "M3C08",
      "responsibility": "integration_regression_current_state_and_stage_closeout"
    }
  ],
  "validation": [
    {
      "id": "VAL-M3C01-001",
      "level": "contract_parse",
      "must_prove": "exactly_one_m3-resolution-v1_json_block_parses_and_has_this_exact_format_and_contract_id"
    },
    {
      "id": "VAL-M3C01-002",
      "level": "frozen_input",
      "must_prove": "role-session-v1_and_handoff-v1_sha256_match_frozen_inputs_and_their_files_have_no_diff"
    },
    {
      "id": "VAL-M3C01-003",
      "level": "state_fixture",
      "must_prove": "only_m1_state_values_and_handoff_transitions_are_accepted; collision_orphan_restart_follow_the_explicit_mapping"
    },
    {
      "id": "VAL-M3C01-004",
      "level": "security_fixture",
      "must_prove": "cross_scope_client_claim_permission_upgrade_handle_collision_missing_receipt_and_raw_copy_requests_fail_closed"
    },
    {
      "id": "VAL-M3C01-005",
      "level": "migration_fixture",
      "must_prove": "each_migration_matrix_classification_keeps_provenance_and_never_promotes_cache_or_raw_transcript_to_truth"
    },
    {
      "id": "VAL-M3C01-006",
      "level": "change_hygiene",
      "must_prove": "relative_markdown_links_resolve_and_git_diff_check_is_clean"
    },
    {
      "id": "VAL-M3C01-007",
      "level": "persistence_boundary",
      "must_prove": "M3_owns_its_repository_and_schema_and_never_routes_RoleSession_or_Handoff_through_the_M2_workflow_sidecar_projector_checkpoint_or_R4_adapter"
    },
    {
      "id": "VAL-M3C01-008",
      "level": "key_divergence",
      "must_prove": "owner_fingerprint_and_provider_namespace_are_deterministic_while_same_idempotency_key_with_changed_immutable_input_is_rejected_not_rekeyed"
    }
  ],
  "explicit_non_goals": [
    "rewrite_or_modify_m1_frozen_contracts",
    "resolve_HOLD-RAW-TRANSCRIPT-RETENTION",
    "copy_raw_transcript_or_provider_response_bodies",
    "silently_map_threads_across_projects_or_scopes",
    "activate_real_provider_processes_or_send_real_messages",
    "implement_full_knowledge_retrieval_external_sync_memory_governance_or_skill_enablement",
    "treat_offline_fake_provider_or_isolated_desktop_evidence_as_real_provider_success"
  ]
}
```

## 2. 人类可读的实施约束

### 2.1 M1 不变，M3 只补实现落点

`role-session-v1` 和 `handoff-v1` 继续是对象词汇、状态和安全边界的冻结来源。M3 新增的 repository、schema、adapter、projection 和验收代码只能实现上述 JSON 已冻结的解释；发现 M1 合同与当前实现无法同时满足时，停止在冲突点，不能静默改 M1 文本、修改对象含义或借前端缓存兜底。

### 2.2 默认不复制、默认不跨范围

默认动作是 `no-copy / no-cross-scope`：不复制原始对话、提示词、模型回复、stdout/stderr、工具参数、凭据或真实服务提供方返回体；不凭 thread id、UI 选择、旧 cache 或用户界面传入的 role/profile 推断项目、角色、权限或 Station。需要跨项目时，必须先取得明确目标项目和范围，再由服务器创建独立、可追溯的作用域；归属不唯一时只保留来源并隔离。

### 2.3 最小上下文不是新的真源

`ConversationContext` 只保存可重建的来源引用、当前对象、资料新鲜度和已知缺口。它按角色、范围、对象、任务与权限装配最小充分材料；命中的资料不会自动变成正式事实、长期记忆、技能启用、权限或外部行动。完整检索、外部同步、技能发现、记忆包生命周期和原始对话保留策略均不在本叶解决。

### 2.4 外部效果与真实服务提供方边界

后续 M3 只能在临时库、isolated profile（隔离配置）和 fake provider（假服务提供方）中验证 effect registration、receipt、readback、重启和幂等。任何真实 provider 进程、真实 Codex 消息、真实账号、凭据、外部 connector、真实项目 root 或发布仍需独立授权；离线、fake 或隔离结论不得改写为真实消息成功。

## 3. 后续叶的使用方式

- M3C02 先把服务器 owner/scope resolver 接到 existing-thread 的 spawn 前；它不得借此创建 RoleSession 真源。
- M3C03 再建立 repository、schema 和 shadow import；原始 transcript、前端 cache 和无法精确绑定的 thread 不进入真源。
- M3C04 只把冻结后的 binding/context/grant 交给 transport port，并用 fake provider 检查重启不重复副作用。
- M3C05 按 `handoff_mapping` 实现状态机与 receipt；返回结果仍要由 source owner 的新命令应用。
- M3C06 让后端 read model 成为恢复入口，Jiaoban 与 Agent Center cache 只保留兼容显示/回切作用。
- M3C07 与 M3C08 分层记录隔离证据、回切边界和未进入的真实 provider 边界；不自动进入 M4/M5。
