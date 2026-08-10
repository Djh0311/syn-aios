---
contract_id: m4-secretary-attention-daily-resolution-v1
version: 1
status: FROZEN_M4_IMPLEMENTATION_RESOLUTION_V1
evidence_level: CONTRACT_AND_SOURCE_RESOLUTION_ONLY
extends:
  - identity-scope-v1
  - role-session-v1
  - handoff-v1
  - event-audit-outbox-v1
  - m3-role-session-turn-handoff-resolution-v1
stage: M4
leaf: M4C01
---

# M4 Secretary / Attention / Daily 实施补充合同 v1

状态：**M4 实施解释已冻结。** 本文件把已经确定的产品边界转成可施工参数；它不重开产品讨论，不修改 M1/M3 冻结合同，也不证明后续产品代码、真实模型、真实来源或真实日常使用已经通过。

## 1. 效力与读取规则

- M1 继续拥有 identity、scope、RoleSession、Handoff、event、audit、receipt 与 outbox 的词汇和安全边界；M3 继续拥有 RoleSession / Turn / Handoff 的通用实现合同。
- 本文件唯一可供程序消费的主体是下方**唯一一个**标注为 `json m4-resolution-v1` 的代码块。消费者必须拒绝缺失、重复、非 JSON 或 `format` 不相等的内容。
- M4 只拥有可撤销的个人协调状态。项目、任务、工作流、授权、正式记忆、个人模型、Skill、外部来源事实和凭据仍由各自 owner 持有。
- 本合同中的固定 ID、路径后缀、窗口和保留参数是 M4 v1 的施工参数，不是新的长期产品正本；改变其语义需要新的合同版本。

```json m4-resolution-v1
{
  "format": "syn.m4.secretary-attention-daily-resolution/v1",
  "machine_block_tag": "m4-resolution-v1",
  "contract_id": "m4-secretary-attention-daily-resolution-v1",
  "version": 1,
  "status": "FROZEN_M4_IMPLEMENTATION_RESOLUTION_V1",
  "scope": {
    "stage": "M4",
    "leaf": "M4C01",
    "default": "fail_closed",
    "evidence_level": "contract_and_source_resolution_only",
    "implementation_authority": "stage_06_current_leaf_only"
  },
  "opening_snapshot": {
    "git_commit": "7b1b63f3a30e3ea926ea85de61a36b77f41f764c",
    "commit_rule": "local_mainline_snapshot_only_not_a_remote_verification",
    "hash_rule": "unless_an_entry_says_otherwise_frozen_input_and_source_evidence_sha256_values_are_recomputed_from_the_exact_blob_at_git_commit",
    "mutable_path_blobs": {
      "docs/current-state.md": "fa262f9f952ebf0477fb5d17e333197ddc126469",
      "docs/plans/2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md": "543e7b69ae6ec85be2efe0de6f6ceb8fbf6e277c",
      "docs/plans/2026-08-01-syn-stage-4-secretary-attention-and-daily-rhythm-plan-v1.md": "22fb0ed08abf49a401d8a9dbaa81b811428937a8"
    },
    "working_tree_rule": "M4C01_edits_to_the_plan_and_current_state_are_expected_to_differ_from_the_opening_blobs; validation_compares_the_recorded_sha256_to_the_recorded_commit_blob_not_to_the_post_correction_working_tree"
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
      "path": "docs/contracts/m3-role-session-turn-handoff-resolution-v1.md",
      "sha256": "946c756b30a8e73aaad441e49ba39a5c9cbd7c7d47241ed97fa19d02783bac48",
      "rule": "read_only_m3_implementation_contract"
    },
    {
      "path": "docs/product/syn-product-canon-v1.md",
      "sha256": "b95a8db131ae8e4f1f79aaafc426a66f20e79ea840b0665ba9df7b3ee1695efa",
      "rule": "product_direction"
    },
    {
      "path": "docs/product/authority-register-v1.md",
      "sha256": "8839ff281ebd93b7338a42cb412c115544863169105feead29ec618cef128155",
      "rule": "document_authority"
    },
    {
      "path": "docs/workbench-system-architecture-v1.md",
      "sha256": "5e5efea3711ba7aa115afd427ab74263655c9b2fad703073c77f62e88a5a3510",
      "rule": "system_boundary"
    },
    {
      "path": "docs/plans/2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md",
      "sha256": "eb85b6ecd449f495d45d2b67528102a2986793726302ab888a42cceaa60859b0",
      "rule": "m4_parent_plan_opening_snapshot"
    },
    {
      "path": "docs/plans/2026-08-01-syn-stage-4-secretary-attention-and-daily-rhythm-plan-v1.md",
      "sha256": "cc09487d6cded201e86d6eb3c67964bd47aea764326e1c47933eddaf90b5612a",
      "rule": "m4_plan_opening_snapshot_before_m4c01_status_and_fact_correction; later status_only edits do not change this resolution; semantic edits require explicit contract review"
    },
    {
      "path": "docs/current-state.md",
      "sha256": "aadbe9e4e164f0bb108d7430236694eb9b048549f10a9e50c6cde3319b30d7be",
      "rule": "opening_current_state_before_stage_06_activation_correction"
    },
    {
      "path": "docs/harness/reports/M2C03-lite-closeout-and-guidance-handoff.md",
      "sha256": "2772cf0ffa03f3697176e500eaf1500c428357d8d78233640d835f2afd7e61f4",
      "rule": "m2_bounded_reference_slice_evidence"
    }
  ],
  "source_evidence": [
    {
      "path": "prototypes/productized-desktop-shell/src-tauri/src/lib.rs",
      "sha256": "b94321347b66d9696108ebcc0a62653f3a044fa40eea2397f4d6fbf4250f6e24",
      "opening_fact": "ordinary_AppState_builds_m3_role_session_read_runtime_with_Default_default_None"
    },
    {
      "path": "prototypes/productized-desktop-shell/src-tauri/src/commands.rs",
      "sha256": "ecead59c4f81a77bde2ff062b37231376683bba2acccfe99a851c0e06048a382",
      "opening_fact": "m3_commands_are_registered_but_fail_closed_with_M3_BINDING_UNAVAILABLE_without_runtime"
    },
    {
      "path": "prototypes/productized-desktop-shell/src-tauri/src/m3_role_session_read_model.rs",
      "sha256": "f3f48b42520d025f41e8fc7a779a7f174d85ffd35aaa0089a4da2c796de0470d",
      "opening_fact": "runtime_slot_has_only_isolated_acceptance_constructor_and_defaults_to_unavailable"
    },
    {
      "path": "prototypes/productized-desktop-shell/src-tauri/src/m3_acceptance.rs",
      "sha256": "67d1a64c9908be87e6a132c84a33d277d3384c3e495bf4b6b31b574b4821eab8",
      "opening_fact": "installer_is_gated_by_debug_explicit_m3c07_mode_and_validated_isolated_profile"
    },
    {
      "path": "prototypes/productized-desktop-shell/src-tauri/src/secretary_agent.rs",
      "sha256": "19ebcc916d5223e2458452b35443e3347dc201fa32a801a0bafb206857220661",
      "opening_fact": "secretary_is_one_shot_read_only_has_fixed_historical_project_cwd_and_no_role_session_store_or_audit"
    },
    {
      "path": "prototypes/productized-desktop-shell/src-tauri/src/mcp/identity_kernel.rs",
      "sha256": "8dc21e21cda547690776ba2e34df53f8e2305bc0e6149ce10d3fd91c071f1478",
      "opening_fact": "identity_types_are_staged_and_unwired_while_resolver_always_derives_project_scope"
    },
    {
      "path": "prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_repository.rs",
      "sha256": "ad0a7d81b393654e99f198f8f47afbf62389edc4a3b8a86d71a7e9d7bcfe49c1",
      "opening_fact": "only_low_level_immediate_transaction_and_busy_retry_mechanics_are_candidates_for_m4_reuse"
    }
  ],
  "ordinary_product_m3_bridge": {
    "owner": "M4C02_app_composition",
    "required": true,
    "opening_status": "absent_not_an_M3_defect",
    "runtime_rule": "ordinary_product_AppState_must_receive_a_server_constructed_M3RoleSessionReadRuntimeSlot_backed_by_the_M3_owned_repository; Default_None_is_not_a_product_runtime",
    "repository_path": {
      "root": "server_resolved_Tauri_app_data_root",
      "relative_path": "conversation/m3-role-session-v1.sqlite3",
      "forbidden_inputs": [
        "frontend_path",
        "current_working_directory",
        "selected_project_root",
        "raw_environment_alias"
      ]
    },
    "constructor_rule": "add_an_ordinary_product_constructor_with_canonical_app_data_path_admission_without_weakening_open_rehearsal_or_isolated_acceptance_gates",
    "host_and_routing_rule": "extend_the_server_owned_runtime_with_a_Secretary_host_and_a_server_only_personal_scope_lookup; existing_Agent_and_Jiaoban_project_locator_commands_remain_project_scoped_and_a_PersonalScope_is_never_encoded_as_a_fake_project_locator_or_project_root",
    "identity_rule": "role_scope_current_object_channel_permission_and_owner_fingerprint_are_resolved_server_side_before_repository_or_provider_use",
    "bootstrap_rule": "on_startup_lookup_the_exact_non_quarantined_Secretary_session_by_owner_fingerprint_and_resume_or_reuse_it; create_only_when_absent; multiple_or_mismatched_candidates_are_quarantined_and_never_selected_by_recency",
    "failure_rule": "missing_unreadable_unverified_or_mismatched_runtime_fails_closed_and_keeps_M3_BINDING_UNAVAILABLE_or_a_more_specific_scrubbed_code",
    "isolation_rule": "M3C07_acceptance_install_and_normal_product_install_are_separate_named_paths_and_cannot_fallback_to_each_other",
    "non_claim": "M3_generic_contract_and_isolated_implementation_are_complete_but_ordinary_product_injection_begins_in_M4C02"
  },
  "secretary_identity": {
    "profile_id": "m4-secretary-personal-primary-v1",
    "actor_id": "actor:local-primary-user",
    "actor_resolution": "server_resolved_local_profile_not_frontend_claimed",
    "role_ref": {
      "role_id": "role:secretary:personal-primary",
      "role_kind": "secretary",
      "role_revision": 1
    },
    "scope_ref": {
      "scope_kind": "personal",
      "scope_id": "scope:personal:primary",
      "scope_revision": 1
    },
    "current_object_ref": {
      "object_type": "personal_workbench",
      "object_id": "personal-workbench:primary",
      "source_owner_ref": "identity_scope_kernel",
      "scope_ref": {
        "scope_kind": "personal",
        "scope_id": "scope:personal:primary",
        "scope_revision": 1
      },
      "binding_revision": 1,
      "binding_source_ref": "m4-secretary-bootstrap:v1"
    },
    "execution_channel": {
      "channel_kind": "daily",
      "risk_class": "low",
      "side_effect_mode": "write_local"
    },
    "permission_profile": {
      "profile_id": "permission:m4-secretary-local-coordination:v1",
      "allow_capabilities": [
        "read_personal_coordination",
        "read_registered_internal_source_refs",
        "write_m4_coordination_state",
        "create_explicit_standalone_personal_action",
        "request_registered_owner_command",
        "create_m3_handoff",
        "read_m3_handoff_receipt"
      ],
      "deny_capabilities": [
        "write_project_fact",
        "write_project_task",
        "write_workflow_state",
        "write_authorization",
        "write_formal_memory",
        "write_personal_model",
        "write_skill",
        "write_external_source_fact",
        "use_external_connector",
        "read_or_write_credential",
        "execute_unregistered_tool",
        "send_external_message"
      ],
      "constraints": [
        "source_ref_required",
        "scrubbed_summary_only",
        "local_coordination_writes_only",
        "owner_command_requires_explicit_user_intent_and_owner_receipt"
      ],
      "revision": 1
    },
    "permission_snapshot_rule": "permission_snapshot_authority_mints_an_immutable_snapshot_with_snapshot_id_profile_id_actor_id_exact_scope_exact_channel_revision_snapshot_hash_and_issued_at; renderer_inputs_and_local_cache_cannot_supply_or_refresh_it",
    "owner_fingerprint_rule": "use_the_M3_v1_owner_fingerprint_algorithm_over_server_resolved_actor_role_scope_current_object_and_channel",
    "stability_rule": "the_same_local_profile_and_contract_revision_resolve_the_same_identity_across_restart; rotation_requires_a_new_revision_and_audited_migration",
    "revocation_rule": "permission_snapshot_may_be_revoked_or_narrowed_without_deleting_coordination_history; new_commands_then_fail_closed"
  },
  "m2_reuse_boundary": {
    "authoritative_available_port": {
      "port_version": "workflow-state-sidecar.repository.m2.v1",
      "status": "bounded_reference_slice_only",
      "rule": "not_a_general_workbench_transaction_port_and_not_an_M4_repository"
    },
    "allowed_mechanical_reuse": [
      "WorkbenchSqliteRepository_immediate_transaction_engine",
      "bounded_SQLITE_BUSY_retry_mechanics",
      "physical_event_audit_receipt_ledger_shapes_after_explicit_M4_mapping"
    ],
    "m4_must_own": [
      "versioned_M4_schema",
      "M4_repository_port",
      "M4_command_admission_and_idempotency",
      "same_transaction_M4_state_event_audit_and_receipt",
      "M4_projection_checkpoints",
      "M4_rebuild_and_rollback_adapters"
    ],
    "forbidden_reuse": [
      "private_unwired_generic_m2_traits_as_authority",
      "WorkflowStateSidecarRepositoryV1_as_M4_repository",
      "workflow_state_sidecar_projector_or_checkpoint",
      "R4_adapter_or_cutover_switch",
      "work_item_revision_as_M4_revision",
      "M2_owner_names_as_M4_domain_owners"
    ],
    "rule": "M2_is_a_bounded_reference_slice; every_reused_low_level_mechanism_has_an_explicit_M4_owned_adapter_and_test"
  },
  "storage_and_single_writer": {
    "database": {
      "owner": "m4_secretary_repository",
      "root": "server_resolved_Tauri_app_data_root",
      "relative_path": "secretary/m4-secretary-v1.sqlite3",
      "schema_version": 1,
      "journal_mode": "WAL",
      "foreign_keys": "ON",
      "busy_timeout_ms": 250,
      "busy_retry_limit": 3
    },
    "single_writer": "all_M4_domain_state_event_audit_receipt_and_projector_checkpoint_writes_enter_through_M4SecretaryRepository",
    "readers": "read_models_use_read_only_transactions_and_never_repair_or_promote_state",
    "cross_database_rule": "M3_and_M4_keep_separate_owned_databases_and_exchange_only_typed_refs_receipts_and_events; no_cross_database_atomicity_claim",
    "path_rule": "ordinary_paths_are_canonical_descendants_of_app_data_root; isolated_paths_are_canonical_descendants_of_the_validated_M4_fixture_root; aliases_symlinks_cwd_and_frontend_paths_fail_closed",
    "sensitivity_rule": "store_opaque_refs_hashes_scrubbed_summaries_and_reason_codes_only; raw_transcripts_prompts_provider_bodies_tool_output_credentials_email_bodies_calendar_bodies_and_file_contents_are_forbidden"
  },
  "objects": {
    "SourceRecordRef": {
      "truth_owner": "source_domain",
      "m4_owner": "reference_and_ingestion_receipt_only",
      "required_fields": [
        "source_owner_ref",
        "scope_ref",
        "source_type",
        "canonical_source_object_id",
        "source_revision",
        "source_event_id",
        "source_owner_watermark",
        "occurred_at_utc",
        "source_link",
        "source_status_code",
        "attention_signals",
        "due_at_utc",
        "sensitivity",
        "scrubbed_summary_ref",
        "payload_hash"
      ]
    },
    "InboxItem": {
      "truth_owner": "m4_personal_inbox_projection",
      "required_fields": [
        "inbox_item_id",
        "source_ref",
        "dedupe_key",
        "status",
        "priority_reason",
        "received_at_utc",
        "last_source_change_at_utc",
        "scrubbed_summary_ref",
        "sensitivity",
        "revision"
      ],
      "rule": "projection_only; a_registered_versioned_source_policy_may_independently_project_one_OpenLoop_from_the_same_admitted_source_in_the_same_M4_UoW; InboxItem_itself_is_not_command_authority_and_never_creates_PersonalAction"
    },
    "OpenLoop": {
      "truth_owner": "m4_secretary_coordination_domain",
      "required_fields": [
        "open_loop_id",
        "source_ref",
        "status",
        "why_open",
        "priority_reason",
        "owner_ref",
        "due_at_utc",
        "snoozed_until_utc",
        "last_source_revision",
        "projection_policy_ref",
        "closure_reason_code",
        "revision"
      ],
      "creation_rule": "only_an_admitted_source_classified_by_a_registered_deterministic_attention_policy_or_an_explicit_user_track_command_may_create_it; model_output_cannot_create_it",
      "rule": "tracks_user_attention_and_closure_only_and_never_owns_source_business_completion"
    },
    "PersonalAction": {
      "truth_owner": "m4_personal_action_aggregate",
      "required_fields": [
        "personal_action_id",
        "explicit_user_command_ref",
        "title",
        "status",
        "due_at_utc",
        "revision"
      ],
      "creation_rule": "only_an_explicit_user_command_to_create_a_standalone_personal_todo_may_create_it",
      "forbidden_rule": "InboxItem_OpenLoop_notification_reminder_decision_or_model_output_never_auto_clones_into_PersonalAction"
    },
    "Notification": {
      "truth_owner": "m4_notification_domain",
      "required_fields": [
        "notification_id",
        "source_ref",
        "subject_ref",
        "notification_purpose_code",
        "delivery_channel",
        "status",
        "created_at_utc",
        "delivered_at_utc",
        "read_at_utc",
        "dismissed_at_utc",
        "revision"
      ],
      "channel_rule": "M4_v1_supports_in_app_delivery_only",
      "rule": "delivery_read_and_dismiss_state_only_not_source_business_state"
    },
    "Reminder": {
      "truth_owner": "m4_reminder_domain",
      "required_fields": [
        "reminder_id",
        "owner_ref",
        "explicit_schedule_command_id",
        "scheduled_for_utc",
        "iana_timezone",
        "status",
        "last_fired_at_utc",
        "snoozed_until_utc",
        "revision"
      ],
      "rule": "local_schedule_and_delivery_state_bound_to_a_source_or_explicit_personal_action_ref"
    },
    "DecisionRequestProjection": {
      "truth_owner": "source_domain",
      "m4_owner": "reference_projection_and_local_visibility_overlay_only",
      "required_fields": [
        "decision_projection_id",
        "source_ref",
        "owner_status",
        "local_visibility_status",
        "decision_by_utc",
        "source_revision",
        "revision"
      ],
      "forbidden_fields": [
        "executable_confirmation_payload",
        "raw_pendingAction_callback",
        "credential",
        "owner_mutation_closure"
      ],
      "rule": "restart_reloads_the_source_owner_and_requires_fresh_owner_confirmation"
    },
    "DailyBrief": {
      "truth_owner": "m4_daily_projector",
      "required_fields": [
        "daily_window_id",
        "scope_source_watermark",
        "projector_version",
        "ordered_item_refs",
        "generated_at_utc"
      ],
      "rule": "deterministic_current_window_projection_rebuildable_from_source_refs"
    },
    "DailyReport": {
      "truth_owner": "m4_daily_projector",
      "required_fields": [
        "daily_report_id",
        "daily_window_id",
        "report_version",
        "status",
        "scope_source_watermark",
        "projector_version",
        "ordered_item_refs",
        "supersedes_report_ref",
        "generated_at_utc"
      ],
      "rule": "versioned_immutable_report_reference_not_a_formal_memory_project_fact_or_task"
    }
  },
  "lifecycle": {
    "InboxItem": {
      "states": [
        "NEW",
        "READ",
        "DISMISSED",
        "EXPIRED",
        "QUARANTINED"
      ],
      "transitions": [
        "NEW->READ",
        "NEW|READ->DISMISSED",
        "NEW|READ|DISMISSED->NEW_on_strictly_new_source_revision",
        "NEW|READ|DISMISSED->EXPIRED_on_owner_expiry",
        "ANY_NON_QUARANTINED->QUARANTINED_on_invalid_source_binding"
      ]
    },
    "OpenLoop": {
      "states": [
        "OPEN",
        "ACKNOWLEDGED",
        "SNOOZED",
        "CLOSED",
        "DISMISSED"
      ],
      "transitions": [
        "OPEN->ACKNOWLEDGED",
        "OPEN|ACKNOWLEDGED->SNOOZED",
        "SNOOZED->OPEN_on_clock",
        "OPEN|ACKNOWLEDGED|SNOOZED->CLOSED",
        "OPEN|ACKNOWLEDGED|SNOOZED|DISMISSED->CLOSED_on_newer_source_status_COMPLETED_CANCELLED_or_EXPIRED",
        "OPEN|ACKNOWLEDGED|SNOOZED->DISMISSED",
        "CLOSED|DISMISSED->OPEN_on_explicit_reopen_or_newer_non_terminal_source_revision_matching_attention_policy"
      ],
      "source_terminal_mapping": {
        "COMPLETED": "CLOSED_with_SOURCE_COMPLETED",
        "CANCELLED": "CLOSED_with_SOURCE_CANCELLED",
        "EXPIRED": "CLOSED_with_SOURCE_EXPIRED"
      },
      "semantic_rule": "ACKNOWLEDGED_means_user_has_seen_it; CLOSED_means_secretary_stops_tracking_it_and_may_reference_a_source_terminal_receipt; neither_is_an_M4_claim_or_write_that_the_source_business_item_is_complete"
    },
    "PersonalAction": {
      "states": [
        "OPEN",
        "COMPLETED",
        "CANCELLED"
      ],
      "transitions": [
        "OPEN->COMPLETED",
        "OPEN->CANCELLED",
        "COMPLETED|CANCELLED->OPEN_on_explicit_user_reopen"
      ]
    },
    "Notification": {
      "states": [
        "PENDING",
        "DELIVERED",
        "READ",
        "DISMISSED"
      ],
      "transitions": [
        "PENDING->DELIVERED",
        "DELIVERED->READ",
        "PENDING|DELIVERED|READ->DISMISSED"
      ]
    },
    "Reminder": {
      "states": [
        "SCHEDULED",
        "FIRED",
        "SNOOZED",
        "DISMISSED",
        "CANCELLED"
      ],
      "transitions": [
        "SCHEDULED->FIRED",
        "SCHEDULED|FIRED->SNOOZED",
        "SNOOZED->FIRED_on_clock",
        "SCHEDULED|FIRED|SNOOZED->DISMISSED",
        "SCHEDULED|SNOOZED->CANCELLED"
      ]
    },
    "DecisionRequestProjection": {
      "owner_states": [
        "OPEN",
        "ANSWERED",
        "EXPIRED",
        "WITHDRAWN"
      ],
      "rule": "owner_state_is_mirrored_from_source_events_only; M4_read_or_dismiss_changes_only_local_visibility"
    },
    "carry_over_rule": "an_unclosed_non_snoozed_item_is_selected_into_the_next_daily_window_without_creating_a_new_domain_object_or_resetting_its_revision",
    "owner_writeback_rule": "a_user_request_to_change_source_business_state_calls_a_registered_source_owner_command_with_source_ref_expected_source_revision_new_idempotency_key_and_explicit_intent; M4_persists_only_the_scrubbed_owner_receipt_ref_and_never_applies_the_result_itself"
  },
  "source_ingestion": {
    "m4_v1_allowlist": [
      "structured_internal_workflow_attention_ref",
      "structured_internal_runtime_attention_ref",
      "structured_internal_handoff_receipt_ref",
      "structured_internal_decision_request_ref",
      "explicit_personal_action_command",
      "local_timer_event"
    ],
    "project_summary_rule": "M5_owned_ProjectSummary_may_be_consumed_only_after_its_contract_and_source_owner_are_available; absence_is_a_declared_unavailable_source_not_an_M4_blocker",
    "external_source_rule": "M8_owns_real_connectors_credentials_and_external_source_records; M4_accepts_only_contract_allowed_scrubbed_refs_after_M8_exists",
    "admission_required_fields": [
      "source_owner_ref",
      "scope_ref",
      "source_type",
      "canonical_source_object_id",
      "source_revision",
      "source_event_id",
      "source_owner_watermark",
      "occurred_at_utc",
      "source_link",
      "source_status_code",
      "attention_signals",
      "due_at_utc",
      "sensitivity",
      "scrubbed_summary_ref",
      "payload_hash"
    ],
    "source_revision_schema": {
      "type": "unsigned_64_bit_integer",
      "encoding_in_hashes": "base10_ASCII_without_sign_or_leading_zero_except_zero",
      "rule": "strictly_increases_per_source_owner_ref_source_type_and_canonical_source_object_id; equal_revision_requires_the_exact_same_event_id_owner_watermark_and_payload_hash_or_is_quarantined"
    },
    "source_status_codes": {
      "non_terminal": [
        "OPEN",
        "BLOCKED",
        "WAITING_USER",
        "INFORMATIONAL"
      ],
      "terminal": [
        "COMPLETED",
        "CANCELLED",
        "EXPIRED"
      ],
      "mapping_rule": "each_registered_source_adapter_has_a_versioned_total_mapping_from_owner_status_to_one_code; unknown_owner_status_is_QUARANTINED_not_INFERRED"
    },
    "attention_signals_schema": {
      "required_boolean_fields": [
        "external_commitment",
        "time_sensitive",
        "requires_user_decision",
        "source_blocked",
        "attention_required",
        "material_change"
      ],
      "authority": "registered_deterministic_source_adapter_only_not_frontend_or_model",
      "due_at_rule": "due_at_utc_is_an_RFC3339_UTC_instant_or_null_and_cannot_be_inferred_by_a_model"
    },
    "source_link_schema": {
      "required_fields": [
        "link_kind",
        "source_owner_ref",
        "object_type",
        "canonical_source_object_id",
        "expected_source_revision",
        "opaque_route_ref"
      ],
      "allowed_link_kinds": [
        "INTERNAL_ROUTE",
        "HANDOFF_REF",
        "OWNER_COMMAND_REF"
      ],
      "rule": "opaque_route_ref_is_server_minted_and_resolved_by_the_registered_owner_adapter; raw_filesystem_paths_external_URLs_callbacks_and_executable_payloads_are_forbidden_in_M4_v1"
    },
    "unknown_rule": "unknown_expired_unjoinable_scope_mismatched_or_sensitive_input_is_rejected_or_quarantined_before_active_projection",
    "source_owner_watermark_rule": "opaque_owner_supplied_watermark_is_bound_to_the_exact_owner_scope_revision_event_and_payload_hash; M4_compares_it_for_equality_only_and_never_uses_it_as_a_cross_owner_clock",
    "scope_source_watermark": {
      "algorithm": "sha256",
      "domain_separator": "syn.m4.scope-source-watermark/v1",
      "entry_fields": [
        "source_owner_ref",
        "scope_ref",
        "source_type",
        "canonical_source_object_id",
        "source_revision_base10",
        "source_event_id",
        "source_owner_watermark",
        "payload_hash"
      ],
      "entry_sort": [
        "source_owner_ref_ascending_utf8",
        "scope_ref_ascending_utf8",
        "source_type_ascending_utf8",
        "canonical_source_object_id_ascending_utf8"
      ],
      "encoding": "domain_separator_utf8_then_for_each_sorted_entry_each_field_as_u32_big_endian_byte_length_followed_by_canonical_utf8_bytes",
      "empty_value": "sha256_of_domain_separator_utf8_with_zero_entries",
      "change_rule": "changed_means_before_digest_differs_from_after_digest_after_one_committed_ingestion_UoW; it_is_not_a_numeric_or_cross_owner_monotonic_comparison",
      "material_event_rule": "material_only_when_the_registered_adapter_marks_material_change_and_normalized_status_attention_signals_due_at_or_payload_hash_changed; revision_only_heartbeats_may_change_the_watermark_but_are_not_model_eligible"
    }
  },
  "attention_policy": {
    "policy_id": "m4-attention-policy",
    "policy_version": 1,
    "policy_ref": "m4-attention-policy:v1",
    "inputs": [
      "source_status_code",
      "attention_signals",
      "due_at_utc",
      "received_at_utc",
      "last_source_change_at_utc",
      "carry_over_from_window_ref",
      "explicit_user_track_command_ref"
    ],
    "automatic_open_loop_predicate": "source_status_is_non_terminal_and_at_least_one_of_external_commitment_time_sensitive_requires_user_decision_source_blocked_or_attention_required_is_true",
    "explicit_track_rule": "an_explicit_user_track_command_with_an_exact_source_ref_may_create_or_reopen_an_OpenLoop_even_for_INFORMATIONAL_or_terminal_source_status_but_the_reason_is_EXPLICIT_USER_FOLLOW_UP_and_never_changes_the_source_status",
    "terminal_rule": "a_terminal_source_status_never_auto_creates_an_OpenLoop_and_closes_an_existing_policy_created_OpenLoop_with_SOURCE_TERMINAL_without_claiming_owner_completion",
    "quarantine_precedence": "invalid_unknown_sensitive_scope_mismatched_or_conflicting_source_is_quarantined_before_policy_evaluation",
    "priority_precedence": [
      "rank_0_when_external_commitment_or_time_sensitive",
      "rank_1_when_requires_user_decision_or_source_blocked",
      "rank_2_when_attention_required_or_material_change_or_explicit_user_track",
      "rank_3_when_carry_over_and_no_higher_rule_matches",
      "rank_4_otherwise"
    ],
    "conflict_rule": "quarantine_then_terminal_then_explicit_user_track_then_lowest_numeric_priority_rank; the_model_and_frontend_never_resolve_policy_conflicts"
  },
  "dedupe": {
    "id_encoding": "for_every_rule_hash_domain_separator_utf8_then_each_listed_component_as_u32_big_endian_byte_length_followed_by_canonical_utf8_bytes; output_is_the_listed_ASCII_prefix_followed_by_lowercase_64_hex_sha256",
    "object_id_rules": {
      "InboxItem": {
        "prefix": "inbox:",
        "domain_separator": "syn.m4.inbox-item/v1",
        "components": [
          "source_identity_key"
        ]
      },
      "OpenLoop": {
        "prefix": "open-loop:",
        "domain_separator": "syn.m4.open-loop/v1",
        "components": [
          "source_identity_key",
          "projection_policy_ref"
        ]
      },
      "PersonalAction": {
        "prefix": "personal-action:",
        "domain_separator": "syn.m4.personal-action/v1",
        "components": [
          "explicit_user_command_id"
        ]
      },
      "Notification": {
        "prefix": "notification:",
        "domain_separator": "syn.m4.notification/v1",
        "components": [
          "subject_ref",
          "notification_purpose_code"
        ]
      },
      "Reminder": {
        "prefix": "reminder:",
        "domain_separator": "syn.m4.reminder/v1",
        "components": [
          "owner_ref",
          "explicit_schedule_command_id"
        ]
      },
      "DecisionRequestProjection": {
        "prefix": "decision-projection:",
        "domain_separator": "syn.m4.decision-projection/v1",
        "components": [
          "source_identity_key"
        ]
      },
      "DailyReport": {
        "prefix": "daily-report:",
        "domain_separator": "syn.m4.daily-report/v1",
        "components": [
          "daily_window_id",
          "report_version_base10"
        ]
      }
    },
    "source_identity_key": {
      "algorithm": "sha256",
      "domain_separator": "syn.m4.source-identity/v1",
      "components": [
        "source_owner_ref",
        "scope_ref",
        "source_type",
        "canonical_source_object_id"
      ],
      "encoding": "domain_separator_utf8_then_for_each_component_u32_big_endian_byte_length_followed_by_canonical_utf8_bytes",
      "output": "source_colon_followed_by_lowercase_64_hex_sha256"
    },
    "source_event_key": {
      "algorithm": "sha256",
      "domain_separator": "syn.m4.source-event/v1",
      "components": [
        "source_identity_key",
        "source_revision_base10",
        "source_event_id",
        "payload_hash"
      ],
      "encoding": "domain_separator_utf8_then_for_each_component_u32_big_endian_byte_length_followed_by_canonical_utf8_bytes",
      "output": "source-event_colon_followed_by_lowercase_64_hex_sha256"
    },
    "rules": [
      "same_source_event_key_returns_original_ingestion_receipt",
      "same_event_id_with_different_identity_revision_or_payload_hash_is_quarantined",
      "a_new_revision_updates_the_existing_source_projection_and_does_not_create_a_duplicate_OpenLoop",
      "different_source_owners_scopes_or_object_ids_never_merge_business_or_coordination_objects",
      "display_grouping_may_show_related_items_but_keeps_each_source_ref_and_owner_visible"
    ]
  },
  "priority_and_order": {
    "model_authority": "none",
    "required_visible_fields": [
      "priority_reason_code",
      "priority_reason_text",
      "source_owner",
      "source_link",
      "last_change_at",
      "current_status"
    ],
    "tiers": [
      {
        "rank": 0,
        "code": "EXTERNAL_COMMITMENT_OR_TIME_CRITICAL",
        "meaning": "external_commitment_or_time_sensitive_including_overdue_or_due_within_24_hours"
      },
      {
        "rank": 1,
        "code": "USER_DECISION_OR_BLOCKER",
        "meaning": "user_decision_required_or_source_reports_blocked"
      },
      {
        "rank": 2,
        "code": "ACTIVE_CHANGED_ATTENTION",
        "meaning": "new_or_materially_changed_open_attention"
      },
      {
        "rank": 3,
        "code": "CARRIED_OVER",
        "meaning": "unclosed_item_carried_from_an_earlier_daily_window"
      },
      {
        "rank": 4,
        "code": "INFORMATIONAL",
        "meaning": "source_backed_information_without_current_action"
      }
    ],
    "stable_sort_tuple": [
      "priority_rank_ascending",
      "due_at_utc_ascending_null_last",
      "last_source_change_at_utc_descending",
      "source_owner_ref_ascending_utf8",
      "canonical_source_object_id_ascending_utf8",
      "m4_object_id_ascending_utf8"
    ],
    "enhancement_rule": "a_model_may_explain_an_existing_reason_only_after_an_explicit_user_message_or_contract_allowed_event; it_cannot_change_rank_dedupe_state_or_owner"
  },
  "timezone_and_daily": {
    "timezone_source": "server_resolved_OS_IANA_timezone",
    "fallback_rule": "missing_invalid_or_non_IANA_timezone_disables_scheduler_and_returns_a_scrubbed_configuration_error; UTC_is_not_silently_substituted",
    "persisted_fields": [
      "iana_timezone",
      "local_date",
      "window_start_utc",
      "window_end_utc",
      "utc_offset_at_start_seconds",
      "utc_offset_at_end_seconds",
      "timezone_rules_version"
    ],
    "day_window": "local_calendar_day_half_open_interval_start_inclusive_end_exclusive_converted_with_timezone_rules",
    "scheduler": {
      "in_process_tick_seconds": 60,
      "daily_close_grace_minutes": 5,
      "daily_report_rule": "the_first_tick_or_startup_after_local_midnight_plus_grace_closes_the_previous_local_day_exactly_once",
      "daily_brief_rule": "the_current_window_brief_is_reprojected_only_after_an_admitted_source_event_coordination_command_or_explicit_user_open_refresh_and_never_requires_a_model",
      "timezone_change_rule": "an_OS_timezone_change_is_recorded_as_a_new_scheduler_configuration_revision_effective_for_unmaterialized_windows_only; existing_window_ids_and_reports_are_immutable"
    },
    "daily_window_id": {
      "algorithm": "sha256",
      "domain_separator": "syn.m4.daily-window/v1",
      "components": [
        "scope_id",
        "iana_timezone",
        "local_date",
        "window_start_utc",
        "window_end_utc",
        "timezone_rules_version"
      ],
      "encoding": "domain_separator_utf8_then_for_each_component_u32_big_endian_byte_length_followed_by_canonical_utf8_bytes",
      "output": "daily-window_colon_followed_by_lowercase_64_hex_sha256"
    },
    "dst_rule": "derive_UTC_boundaries_from_local_calendar_midnights_so_23_or_25_hour_days_create_exactly_one_window_and_repeated_or_missing_hours_never_duplicate_a_window",
    "catch_up": {
      "maximum_closed_windows_per_startup": 7,
      "order": "oldest_first",
      "beyond_limit": "record_a_scrubbed_CATCH_UP_TRUNCATED_receipt_and_leave_older_windows_unmaterialized_until_explicit_user_request",
      "empty_window_rule": "advance_scheduler_checkpoint_and_record_zero_invocation_counter_without_creating_a_model_turn"
    },
    "idempotency": "the_same_daily_window_id_projector_version_and_scope_source_watermark_returns_the_existing_report_version",
    "correction": "a_changed_scope_source_watermark_or_explicit_user_correction_creates_a_new_immutable_report_version_and_marks_the_previous_version_SUPERSEDED_without_overwrite",
    "report_version_schema": "unsigned_64_bit_integer_starting_at_1_encoded_as_base10_ASCII_without_leading_zero_and_incremented_by_exactly_1_per_daily_window_id",
    "report_version_states": [
      "GENERATED",
      "SUPERSEDED",
      "FAILED"
    ],
    "retention": {
      "open_coordination": "retained_until_explicit_terminal_transition_no_automatic_TTL",
      "terminal_coordination_visibility_days": 90,
      "daily_report_visibility_days": 365,
      "physical_deletion": "not_part_of_M4; terminal_records_versions_events_audit_receipts_and_source_refs_are_preserved_in_stage_06"
    }
  },
  "event_driven_and_model_gate": {
    "work_triggers": [
      "explicit_user_message",
      "explicit_user_coordination_command",
      "admitted_structured_source_event",
      "M3_handoff_receipt_event",
      "local_TimerFired_event",
      "internal_failure_recovery_event"
    ],
    "mechanical_first": "ingest_validate_dedupe_project_sort_and_generate_deterministic_brief_or_report_before_any_model_decision",
    "model_eligibility_rule": "a_model_turn_requires_either_an_explicit_user_message_or_a_material_admitted_source_event_that_changed_the_scope_source_watermark_and_a_named_enhancement_purpose; timer_ticks_scheduler_checkpoint_writes_empty_windows_coordination_only_commands_and_internal_recovery_events_are_never_model_eligible_by_themselves",
    "zero_event_rule": "if_no_material_admitted_source_event_changes_the_scope_source_watermark_and_no_explicit_user_message_exists_then_agent_turn_count_and_model_invocation_count_are_both_exactly_zero_even_when_a_timer_tick_or_startup_check_occurs",
    "background_rule": "no_polling_agent_loop_no_heartbeat_model_call_and_no_model_call_merely_because_the_App_is_open",
    "scheduler_run_ledger": {
      "required_fields": [
        "scheduler_run_id",
        "configuration_revision",
        "window_ref",
        "scope_source_watermark_before",
        "scope_source_watermark_after",
        "admitted_material_event_count",
        "agent_turn_count",
        "model_invocation_count",
        "outcome_code",
        "recorded_at_utc"
      ],
      "empty_window_receipt": "admitted_material_event_count_0_agent_turn_count_0_model_invocation_count_0"
    },
    "invocation_ledger": {
      "owner": "m4_secretary_repository",
      "required_fields": [
        "invocation_id",
        "trigger_event_ref",
        "role_session_id",
        "turn_id",
        "purpose_code",
        "budget_class",
        "outcome_code",
        "started_at_utc",
        "terminal_at_utc"
      ],
      "forbidden_fields": [
        "raw_prompt",
        "raw_response",
        "provider_token",
        "tool_output"
      ]
    },
    "model_failure_rule": "deterministic_context_brief_and_daily_report_remain_available_and_the_failure_is_a_scrubbed_receipt_not_a_projection_rollback"
  },
  "daily_memory_boundary": {
    "m4_owns": [
      "wall_clock_scheduler",
      "daily_window_id",
      "DailyBrief",
      "DailyReport_version",
      "DailyWindowClosed_event",
      "DailyReportVersioned_event",
      "source_refs_and_projection_checkpoint"
    ],
    "m7_owns": [
      "formal_memory",
      "PersonalFact",
      "personal_model",
      "Skill",
      "memory_consolidation_annotation"
    ],
    "handoff_rule": "M7_may_consume_DailyWindowClosed_and_DailyReportVersioned_idempotently_and_create_a_separate_M7_owned_artifact_or_annotation_ref",
    "forbidden_rule": "M4_never_promotes_a_report_or_attention_to_formal_memory_and_M7_never_mutates_M4_report_attention_or_coordination_rows_in_place"
  },
  "daily_handoff_events": {
    "common_envelope": {
      "contract": "event-audit-outbox-v1.WorkbenchEventEnvelope",
      "required_fields": [
        "event_id",
        "event_type",
        "occurred_at",
        "actor_id",
        "scope_ref",
        "source_ref",
        "source_revision",
        "command_id",
        "correlation_id",
        "causation_id",
        "trace_context",
        "schema_version",
        "sensitivity",
        "summary_ref",
        "payload_ref",
        "payload_hash"
      ],
      "sensitivity": "SCRUBBED_INTERNAL_REF_ONLY",
      "payload_hash_rule": "sha256_of_canonical_JSON_object_with_keys_sorted_ascending_utf8_no_insignificant_whitespace_and_integer_versions_as_base10"
    },
    "DailyWindowClosed": {
      "event_type": "DailyWindowClosed",
      "schema_version": "syn.m4.daily-window-closed/v1",
      "typed_payload_fields": [
        "scope_ref",
        "daily_window_id",
        "iana_timezone",
        "local_date",
        "window_start_utc",
        "window_end_utc",
        "scope_source_watermark",
        "projector_version",
        "closed_at_utc"
      ],
      "source_ref_rule": "daily_window_id",
      "source_revision_rule": "projector_version",
      "idempotency_key": {
        "algorithm": "sha256",
        "domain_separator": "syn.m4.daily-window-closed-idempotency/v1",
        "components": [
          "daily_window_id",
          "projector_version"
        ],
        "encoding": "domain_separator_utf8_then_for_each_component_u32_big_endian_byte_length_followed_by_canonical_utf8_bytes"
      }
    },
    "DailyReportVersioned": {
      "event_type": "DailyReportVersioned",
      "schema_version": "syn.m4.daily-report-versioned/v1",
      "typed_payload_fields": [
        "scope_ref",
        "daily_window_id",
        "daily_report_id",
        "report_version",
        "report_ref",
        "supersedes_report_ref",
        "scope_source_watermark",
        "projector_version",
        "generated_at_utc"
      ],
      "source_ref_rule": "daily_report_id",
      "source_revision_rule": "report_version_base10",
      "idempotency_key": {
        "algorithm": "sha256",
        "domain_separator": "syn.m4.daily-report-versioned-idempotency/v1",
        "components": [
          "daily_window_id",
          "report_version_base10"
        ],
        "encoding": "domain_separator_utf8_then_for_each_component_u32_big_endian_byte_length_followed_by_canonical_utf8_bytes"
      },
      "m7_join_key": [
        "daily_window_id",
        "report_version"
      ]
    },
    "handoff_rule": "M7_consumes_the_envelope_and_typed_payload_by_idempotency_key_and_may_store_only_its_own_artifact_or_annotation_ref; event_payload_contains_no_report_body_or_memory_candidate"
  },
  "downstream_boundaries": {
    "M5": "owns_ProjectSummary; M4_consumes_an_available_typed_ref_only",
    "M6": "owns_GlobalSupervisor_consult_success; M4_creates_M3_Handoff_and_consumes_receipts_only",
    "M7": "owns_formal_memory_personal_model_and_Skill; M4_emits_source_backed_daily_events_only",
    "M8": "owns_real_connectors_credentials_and_external_source_facts; M4_has_no_connector_or_credential_code",
    "M9_M10": "own_later_productization_or_quality_scope; M4_does_not_implement_them"
  },
  "migration_and_rollback": {
    "legacy_inputs": [
      "secretaryReadModel_deterministic_summary",
      "right_rail_notification_and_todo_projection",
      "runtime_attention_projection",
      "React_pendingAction_visibility",
      "memory_daily_inbox_candidate"
    ],
    "migration_steps": [
      "shadow_read_legacy_projection_without_writing_source_owner",
      "map_each_candidate_to_an_exact_registered_source_owner_object_revision_and_watermark",
      "reread_the_canonical_source_before_active_projection",
      "compare_source_status_priority_reason_and_link_under_a_versioned_parity_matrix",
      "quarantine_or_expire_unknown_stale_ambiguous_or_unjoinable_candidates",
      "keep_legacy_read_path_compatibility_read_only_until_later_parity_retirement"
    ],
    "forbidden_migration": [
      "copy_raw_transcript_or_pendingAction_executable_payload",
      "infer_project_or_owner_from_cwd_label_or_route",
      "promote_memory_candidate_to_DailyReport_or_formal_memory",
      "auto_create_OpenLoop_or_PersonalAction_from_an_unverified_legacy_item",
      "physically_delete_legacy_data"
    ],
    "rollback": "disable_M4_ingestion_scheduler_and_read_projection_then_select_the_guarded_legacy_read_only_display; preserve_M1_M3_guards_M4_committed_coordination_events_audit_receipts_quarantine_and_report_versions",
    "never_restore": [
      "fixed_project_cwd_as_Secretary_authority",
      "frontend_cache_as_identity_or_lifecycle_truth",
      "implicit_cross_scope_access",
      "owner_business_mutation_from_coordination_state"
    ]
  },
  "evidence_levels": [
    {
      "level": "CONTRACT",
      "proves": "machine_block_owner_state_boundary_and_parameters",
      "does_not_prove": "runtime_implementation"
    },
    {
      "level": "UNIT",
      "proves": "state_dedupe_sort_timezone_idempotency_and_gate_functions",
      "does_not_prove": "restart_or_App_behavior"
    },
    {
      "level": "TEMP_INTEGRATION",
      "proves": "isolated_database_transaction_restart_rebuild_and_fake_model_behavior",
      "does_not_prove": "ordinary_App_UI_or_real_data"
    },
    {
      "level": "NON_TEST_BUILD",
      "proves": "ordinary_product_code_path_compiles_or_bundles",
      "does_not_prove": "visible_UI_or_runtime_interaction"
    },
    {
      "level": "ISOLATED_PRODUCT_APP",
      "proves": "debug_App_with_synthetic_sources_isolated_config_and_fake_provider_supports_the_observed_scenario",
      "does_not_prove": "real_daily_use_real_model_real_connector_or_release"
    },
    {
      "level": "REAL_USE",
      "proves": "only_the_explicitly_authorized_named_real_profile_source_and_scenario",
      "current_stage_status": "excluded_and_not_required_for_M4_stage_06"
    }
  ],
  "implementation_leaves": [
    {
      "leaf": "M4C02",
      "owns": "ordinary_product_M3_runtime_bridge_and_Secretary_PersonalScope"
    },
    {
      "leaf": "M4C03",
      "owns": "M4_schema_repository_source_ingestion_Inbox_and_OpenLoop_projection"
    },
    {
      "leaf": "M4C04",
      "owns": "coordination_lifecycle_explicit_PersonalAction_and_owner_writeback_port"
    },
    {
      "leaf": "M4C05",
      "owns": "Secretary_application_service_persistent_context_and_M3_Handoff"
    },
    {
      "leaf": "M4C06",
      "owns": "home_context_continuous_conversation_and_source_deep_links"
    },
    {
      "leaf": "M4C07",
      "owns": "DailyReport_scheduler_timezone_catch_up_and_zero_model_evidence"
    },
    {
      "leaf": "M4C08",
      "owns": "legacy_shadow_parity_compatibility_and_rollback"
    },
    {
      "leaf": "M4C09",
      "owns": "isolated_debug_product_App_acceptance_with_synthetic_data"
    },
    {
      "leaf": "M4C10",
      "owns": "full_regression_document_sync_independent_acceptance_and_stage_closeout"
    }
  ],
  "validation": {
    "machine_block": {
      "required_count": 1,
      "required_tag": "m4-resolution-v1",
      "required_format": "syn.m4.secretary-attention-daily-resolution/v1",
      "parse_rule": "strict_JSON_no_comments_no_duplicate_top_level_keys"
    },
    "opening_snapshot": {
      "commit": "7b1b63f3a30e3ea926ea85de61a36b77f41f764c",
      "rule": "read_each_recorded_path_from_the_recorded_commit_blob_verify_optional_recorded_blob_oid_then_sha256_exact_bytes_and_compare_to_frozen_inputs_or_source_evidence",
      "working_tree_exception": [
        "docs/current-state.md",
        "docs/plans/2026-08-01-syn-stage-4-secretary-attention-and-daily-rhythm-plan-v1.md"
      ]
    },
    "must_prove": [
      "M1_four_contract_sha256_values_equal_the_recorded_exact_values",
      "ordinary_product_M3_bridge_is_distinct_from_M3C07_isolated_acceptance_and_never_uses_a_fake_project_locator_for_PersonalScope",
      "Secretary_identity_permission_profile_and_owner_fingerprint_inputs_are_server_fixed",
      "M2_reuse_is_limited_to_named_low_level_mechanics_and_M4_owns_schema_repository_UoW_receipt_event_audit_and_checkpoint",
      "SourceRecordRef_admission_includes_scope_canonical_object_u64_revision_status_signals_typed_link_and_exact_owner_watermark",
      "scope_source_watermark_object_ID_source_event_ID_daily_window_ID_and_daily_report_ID_have_exact_hash_components_encoding_and_output",
      "attention_policy_has_a_version_exact_predicate_priority_precedence_and_source_terminal_mapping",
      "OpenLoop_never_auto_clones_to_PersonalAction_and_coordination_never_writes_source_business_state",
      "empty_or_non_material_timer_window_records_agent_turn_count_0_and_model_invocation_count_0",
      "DailyWindowClosed_and_DailyReportVersioned_freeze_typed_M7_join_fields_without_memory_write",
      "migration_is_shadow_parity_compatibility_read_only_and_rollback_preserves_committed_evidence",
      "every_implementation_leaf_from_M4C02_through_M4C10_appears_once"
    ],
    "evidence_limit": "passing_M4C01_validation_proves_contract_and_source_resolution_only_and_cannot_be_reported_as_service_App_real_data_model_connector_or_release_acceptance",
    "review_scope_rule": "M4C01_review_may_require_closure_of_a_contradiction_or_missing_mechanical_definition_already_named_by_the_leaf_but_does_not_add_new_product_owners_sources_external_actions_or_M5_through_M10_implementation"
  },
  "stop_conditions": [
    "M1_or_M3_contract_conflict",
    "source_owner_or_source_revision_cannot_be_resolved_exactly",
    "OpenLoop_or_Inbox_automatically_creates_PersonalAction",
    "coordination_action_mutates_source_business_state_without_owner_command_receipt",
    "ordinary_product_bridge_uses_acceptance_gate_or_fixed_cwd",
    "M4_writes_project_workflow_authorization_formal_memory_skill_or_external_fact",
    "empty_event_window_attempts_an_agent_or_model_invocation",
    "raw_transcript_prompt_provider_body_tool_output_credential_or_external_body_enters_M4_storage",
    "frontend_local_state_becomes_identity_lifecycle_or_permission_truth",
    "synthetic_or_local_evidence_is_described_as_real_daily_use_or_full_release"
  ],
  "non_goals": [
    "real_personal_data",
    "real_model_or_provider",
    "real_Codex_message",
    "real_account_or_credential",
    "email_calendar_file_or_other_external_connector",
    "network_external_write",
    "M5_through_M10_product_implementation",
    "deployment_release_push_merge_or_rebase",
    "physical_legacy_deletion"
  ]
}
```

## 2. 人话解释

- Secretary 的稳定身份来自后端解析的个人范围，不再来自项目 cwd、当前路由或前端缓存。普通产品模式正式注入 M3 运行时是 M4C02 的前置施工，不是补做 M3 缺陷。
- M4 持久化“用户需要继续看住什么”，但不替项目或来源完成业务事项。看过、稍后提醒、关闭关注与完成源事项是四件不同的事。
- `OpenLoop` 和独立个人待办是不同对象。只有用户明确说“创建个人待办”才会产生 `PersonalAction`，任何关注项、日报或模型解释都不会自动克隆待办。
- 日报先走确定性投影；没有新事件也没有用户消息时，Agent 和模型调用都必须精确为零。模型故障不影响确定性简报和日报。
- M5、M7、M8 的对象只通过 typed ref（类型化引用）和事件交接；M4 不提前实现项目摘要、正式记忆、个人模型、Skill 或真实连接器。

## 3. 后续实现纪律

后续 leaf 只能实现 JSON 中对应的写面。发现产品正本或 M1/M3 合同真正冲突时停在冲突点；普通实现选择、表结构、DTO、测试夹具和界面细节由当前任务包在本合同内冻结，不再把第 1 版已经明确的核心需求退回给用户。
