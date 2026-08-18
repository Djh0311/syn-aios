---
contract_id: m6-cross-project-and-organization-v1
version: 1
status: FROZEN_M6_CONTRACT_TEXT_V1
evidence_level: STATIC_CONTRACT_AND_FIXTURES_ONLY
schema_authority: m6_cross_project_organization_contract_authority
dependencies: ["identity-scope-v1", "command-v1", "event-audit-outbox-v1", "role-session-v1", "handoff-v1", "attention-decision-v1", "project-orchestration-v1", "connector-capability-v1", "object-ref-navigation-v1"]
hold_refs: []
---

# M6 cross-project and organization contract v1

M6D01 freezes this contract text and the fixtures under
`docs/contracts/fixtures/m6-org-001/` only. It implements no service, repository,
projection, UI, provider, or runtime behavior. Later M6 leaves consume this text as
the unique judgement surface; they do not inherit authority from this file's
existence.

This addendum does not rewrite, replace, or weaken M1–M5. It is intentionally not
an entry in the frozen M1 ten-contract registry `manifest.v1.json`. M3 `RoleSession`
and `Handoff`, M4 `DecisionRequest`, and M5 `ProjectSummary` / execution identity
remain owned by their original contracts.

## contract.owner

`m6_cross_project_organization_contract_authority` owns this schema text.
Runtime state, if later implemented, remains split by the `domain_owner` declared
for each export. The schema authority is not a blanket business-state owner and
is not a substitute for M3, M4, or M5 owners.

M6 is the single future writer of Global Supervisor / organization domain records
listed below. `ProjectSummary` remains owned by each project projector and is
readable only through M5 `ProjectSummaryQueryPort`. `RoleSession` and `Handoff`
remain M3-owned. `DecisionRequest` remains owned by `SOURCE_OWNER_REF`.
`CapabilityGrant` remains owned by the policy / grant domain.

## contract.non-implementation

```json non-implementation
{
  "leaf": "M6D01",
  "freezes": ["contract_text", "fixtures"],
  "does_not_implement": [
    "service",
    "repository",
    "projection",
    "ui",
    "provider",
    "runtime",
    "tauri_command",
    "schema_migration",
    "store"
  ],
  "does_not_register_in": "docs/contracts/manifest.v1.json",
  "manifest_reason": "manifest.v1.json is the frozen exact ten-contract M1 registry; verify-syn-fnd-001.mjs hard-codes count, order, dependencies, and exports. M3-M5 addenda are intentionally unregistered. Listing this contract there would break the accepted M1 verifier."
}
```

## contract.schema

```json contract-schema
{
  "schema_authority": "m6_cross_project_organization_contract_authority",
  "imports": [
    "ActorId",
    "ProjectId",
    "ScopeRef",
    "RoleRef",
    "RoleSessionId",
    "RoleSession",
    "HandoffId",
    "Handoff",
    "HandoffReceipt",
    "DecisionRequest",
    "CommandReceipt",
    "ObjectRef",
    "CorrelationId",
    "OrchestrationId",
    "WorkflowRun",
    "WorkItem",
    "ExecutionGrant",
    "Dispatch",
    "ProjectSummary",
    "CapabilityGrant",
    "PermissionSnapshotRef"
  ],
  "import_rule": "Imported types keep their original owners, required fields, and legal states. This contract may only reference them. It does not re-export, alias, or relax them.",
  "exports": [
    {
      "name": "AdvisoryId",
      "domain_owner": "global_supervisor_domain",
      "required_fields": ["value"],
      "opening_status": "ABSENT"
    },
    {
      "name": "FreshnessJudgement",
      "domain_owner": "global_supervisor_domain",
      "required_fields": [
        "freshness_state",
        "subject_ref",
        "reason_code",
        "judged_at",
        "consumer_gate_ref",
        "cache_reuse"
      ],
      "opening_status": "ABSENT",
      "constants": {
        "freshness_state": ["fresh", "stale", "missing", "denied", "degraded"],
        "cache_reuse": false,
        "silent_representation_forbidden": ["denied->stale", "denied->missing", "denied->fresh", "denied->degraded", "missing->stale", "missing->fresh", "degraded->stale", "degraded->fresh"]
      }
    },
    {
      "name": "ProjectSummaryConsumerGate",
      "domain_owner": "global_supervisor_domain",
      "required_fields": [
        "query_port",
        "consumer_role_session_id",
        "consumer_scope_ref",
        "consumer_expires_at",
        "policy_decision_ref",
        "requested_project_id",
        "project_owner_ref"
      ],
      "opening_status": "ABSENT",
      "constants": {
        "query_port": "ProjectSummaryQueryPort",
        "consumer_scope_kind": "GLOBAL",
        "direct_project_store_read": false,
        "direct_project_projection_read": false,
        "direct_project_root_read": false,
        "raw_transcript_read": false,
        "secret_read": false,
        "project_writeback": false
      }
    },
    {
      "name": "ConsumedProjectSummaryRef",
      "domain_owner": "global_supervisor_domain",
      "required_fields": [
        "summary_id",
        "project_id",
        "project_owner_ref",
        "schema_version",
        "version",
        "source_watermark",
        "summary_hash",
        "policy_decision_ref",
        "freshness_state",
        "source_refs"
      ],
      "opening_status": "ABSENT",
      "join_rule": "Every listed field is mandatory and exact. Any missing or mismatched field fails closed with zero write. freshness_state must be fresh to enter a CrossProjectAdvisory consumed set."
    },
    {
      "name": "AdvisorySourceLink",
      "domain_owner": "global_supervisor_domain",
      "required_fields": [
        "source_link_id",
        "object_ref",
        "project_id",
        "summary_id",
        "title_ref",
        "scrubbed_summary_ref",
        "deep_link_metadata_ref"
      ],
      "opening_status": "ABSENT",
      "forbidden_fields": ["raw_file", "raw_summary", "transcript", "secret"]
    },
    {
      "name": "CrossProjectAdvisory",
      "domain_owner": "global_supervisor_domain",
      "required_fields": [
        "advisory_id",
        "global_role_session_id",
        "consult_handoff_ref",
        "consumed_summaries",
        "policy_decision_ref",
        "generated_at",
        "source_links",
        "lifecycle_status",
        "freshness_state",
        "revision",
        "idempotency_key",
        "created_at"
      ],
      "opening_status": "ABSENT"
    },
    {
      "name": "PerProjectApplicationObservation",
      "domain_owner": "m6_read_model",
      "required_fields": [
        "observation_id",
        "advisory_id",
        "decision_request_id",
        "project_id",
        "project_owner_ref",
        "authoritative_command_receipt_ref",
        "grant_ref",
        "outcome",
        "observed_at",
        "source_receipt_hash"
      ],
      "opening_status": "ABSENT",
      "constants": {
        "outcome": ["applied", "failed", "rolled_back", "unknown"]
      }
    },
    {
      "name": "AdvisoryApplicationProjection",
      "domain_owner": "m6_read_model",
      "required_fields": [
        "application_projection_id",
        "advisory_id",
        "advisory_revision",
        "decision_request_id",
        "observations",
        "partial_apply",
        "compensation_observation_refs",
        "history",
        "projected_at",
        "projection_revision"
      ],
      "opening_status": "ABSENT",
      "invariants": [
        "owns_no_project_result",
        "never_changes_advisory_lifecycle",
        "history_is_append_only"
      ]
    },
    {
      "name": "MemberId",
      "domain_owner": "organization_directory",
      "required_fields": ["value"],
      "opening_status": "ABSENT"
    },
    {
      "name": "ScopeAssignment",
      "domain_owner": "organization_directory",
      "required_fields": [
        "assignment_id",
        "member_id",
        "scope_ref",
        "assigned_by_actor_id",
        "revision",
        "assigned_at",
        "revoked_at"
      ],
      "opening_status": "ABSENT"
    },
    {
      "name": "RoleAssignment",
      "domain_owner": "organization_directory",
      "required_fields": [
        "assignment_id",
        "member_id",
        "role_ref",
        "scope_ref",
        "assigned_by_actor_id",
        "revision",
        "assigned_at",
        "revoked_at"
      ],
      "opening_status": "ABSENT"
    },
    {
      "name": "CapabilityPermissionRef",
      "domain_owner": "directory_projection",
      "required_fields": [
        "ref_id",
        "subject_member_id",
        "kind",
        "source",
        "revision",
        "observed_at"
      ],
      "opening_status": "ABSENT",
      "constants": {
        "kind": ["capability", "permission"],
        "directory_is_authority": false,
        "read_only": true
      }
    },
    {
      "name": "PromotionBinding",
      "domain_owner": "organization_directory",
      "required_fields": [
        "binding_id",
        "member_id",
        "promoted_from",
        "promoted_by_actor_id",
        "explicit_human_command",
        "created_at"
      ],
      "opening_status": "ABSENT",
      "constants": {
        "explicit_human_command": true,
        "source_temporary_agent_type_unchanged": true
      }
    },
    {
      "name": "StableMember",
      "domain_owner": "organization_directory",
      "required_fields": [
        "member_id",
        "membership_lifecycle",
        "scope_assignments",
        "role_assignments",
        "capability_permission_refs",
        "availability_ref",
        "contact_binding_refs",
        "promoted_from",
        "display_name_ref",
        "revision",
        "created_at",
        "deactivated_at"
      ],
      "opening_status": "ABSENT",
      "forbidden_identity_fields": ["provider", "model", "thread", "process"],
      "type_rule": "StableMember and TemporaryAgent are disjoint types. MemberId and TemporaryAgentId are never interchangeable.",
      "nullable_presence_rule": "promoted_from is present and null for a directly registered StableMember, and is an exact TemporaryAgentId only when an explicit PromotionBinding exists. deactivated_at is present and null until membership_lifecycle=DEACTIVATED; deactivation retains all historical refs."
    },
    {
      "name": "TemporaryAgentId",
      "domain_owner": "execution_history_projection",
      "required_fields": ["value"],
      "opening_status": "ABSENT"
    },
    {
      "name": "ExecutionEnvelope",
      "domain_owner": "execution_history_projection",
      "required_fields": [
        "project_id",
        "orchestration_id",
        "workflow_run_id",
        "work_item_id",
        "node_id",
        "dispatch_id",
        "attempt_id",
        "grant_id",
        "worker_role_session_id",
        "authoritative_receipt_ref",
        "trusted_actor_id",
        "hashes"
      ],
      "opening_status": "ABSENT",
      "constants": {
        "report_self_claim": false,
        "missing_field_compatibility": false,
        "runtime_trace_derivation": false
      }
    },
    {
      "name": "TemporaryAgent",
      "domain_owner": "execution_history_projection",
      "required_fields": [
        "temporary_agent_id",
        "execution_envelope",
        "membership_lifecycle",
        "display_name_ref",
        "created_at"
      ],
      "opening_status": "ABSENT",
      "constants": {
        "auto_stabilize": false,
        "identity_is_execution_envelope": true
      }
    },
    {
      "name": "Availability",
      "domain_owner": "directory_projection",
      "required_fields": [
        "availability_id",
        "subject_ref",
        "source",
        "observed_at",
        "ttl",
        "observed_state",
        "effective_state"
      ],
      "opening_status": "ABSENT",
      "constants": {
        "observed_state": ["available", "busy", "offline", "unknown"],
        "stale_becomes": "unknown",
        "authorizes": false
      }
    },
    {
      "name": "ContactReceipt",
      "domain_owner": "organization_directory",
      "required_fields": [
        "contact_receipt_id",
        "from_actor_id",
        "to_member_id",
        "handoff_id",
        "role_session_id",
        "status",
        "capability_granted",
        "recorded_at",
        "source_command_receipt_ref"
      ],
      "opening_status": "ABSENT",
      "constants": {
        "capability_granted": false
      }
    },
    {
      "name": "ConsultHandoff",
      "domain_owner": "consult_handoff_binding",
      "required_fields": [
        "consult_handoff_ref",
        "handoff_id",
        "handoff_revision",
        "status_ref",
        "consult_kind",
        "from_role_session_id",
        "to_role_ref",
        "scope_ref",
        "question_ref",
        "object_refs",
        "receipt_ref",
        "project_write_capability"
      ],
      "opening_status": "ABSENT",
      "constants": {
        "consult_kind": "SECRETARY_TO_GLOBAL_SUPERVISOR",
        "project_write_capability": false,
        "replaces_m3_handoff": false
      },
      "specialization_rule": "ConsultHandoff is an M3 Handoff specialization and typed reference. Handoff truth, legal states, receipts, and permission-request-is-not-a-grant remain owned by handoff-v1 / handoff_aggregate. This type never becomes a second handoff truth."
    },
    {
      "name": "ConsultationId",
      "domain_owner": "consultation_domain",
      "required_fields": ["value"],
      "opening_status": "ABSENT"
    },
    {
      "name": "QuestionPacket",
      "domain_owner": "consultation_domain",
      "required_fields": [
        "question_packet_id",
        "question_ref",
        "source_refs",
        "packet_hash",
        "minimal",
        "schema_version"
      ],
      "opening_status": "ABSENT",
      "constants": {
        "minimal": true
      }
    },
    {
      "name": "ConsultationView",
      "domain_owner": "consultation_domain",
      "required_fields": [
        "view_id",
        "consultation_id",
        "role_session_id",
        "workcell_ref",
        "context_packet_ref",
        "question_packet_id",
        "question_packet_hash",
        "submitted",
        "conclusion_ref",
        "peer_conclusions_readable_before_submit"
      ],
      "opening_status": "ABSENT",
      "constants": {
        "peer_conclusions_readable_before_submit": false
      }
    },
    {
      "name": "MultiViewConsultation",
      "domain_owner": "consultation_domain",
      "required_fields": [
        "consultation_id",
        "question_packet",
        "views",
        "consensus_index_ref",
        "disagreement_index_ref",
        "evidence_index_ref",
        "budget_limit_ref",
        "budget_state",
        "deadline_at",
        "timeout_state",
        "result_state",
        "user_pending_decision_request_id",
        "produces_command",
        "produces_grant",
        "produces_fact"
      ],
      "opening_status": "ABSENT",
      "constants": {
        "produces_command": false,
        "produces_grant": false,
        "produces_fact": false
      },
      "independence_rule": "At least two views receive the same QuestionPacket id and hash. Every view has a distinct RoleSession, Workcell, and context packet, and cannot read any peer conclusion before its own submit.",
      "limit_rule": "budget_limit_ref and deadline_at are explicit inputs; budget_state and timeout_state are explicit results. Neither limit may be inferred from an implicit abort."
    },
    {
      "name": "ChildRunRef",
      "domain_owner": "m5_runtime_execution",
      "required_fields": [
        "child_run_ref",
        "parent_workcell_ref",
        "attempt_id",
        "grant_id",
        "trace_ref"
      ],
      "opening_status": "ABSENT",
      "constants": {
        "creates_stable_member": false,
        "creates_temporary_agent": false,
        "creates_organization_hierarchy": false,
        "reference_only": true
      }
    },
    {
      "name": "LegacyQuarantineRecord",
      "domain_owner": "unknown_quarantine_repository",
      "required_fields": [
        "quarantine_ref",
        "legacy_kind",
        "reason_code",
        "source_refs",
        "payload_mode",
        "mapped_to",
        "recorded_at"
      ],
      "opening_status": "ABSENT",
      "constants": {
        "payload_mode": "REF_ONLY",
        "mapped_to": null
      }
    }
  ],
  "legal_states": {
    "FreshnessJudgement.freshness_state": ["fresh", "stale", "missing", "denied", "degraded"],
    "CrossProjectAdvisory.lifecycle_status": ["DRAFT", "ISSUED", "SUPERSEDED", "STALE", "WITHDRAWN", "QUARANTINED"],
    "CrossProjectAdvisory.freshness_state": ["fresh", "stale"],
    "PerProjectApplicationObservation.outcome": ["applied", "failed", "rolled_back", "unknown"],
    "StableMember.membership_lifecycle": ["ESTABLISHED", "ACTIVE", "DEACTIVATED", "QUARANTINED"],
    "TemporaryAgent.membership_lifecycle": ["PROJECTED", "RETAINED", "QUARANTINED"],
    "Availability.observed_state": ["available", "busy", "offline", "unknown"],
    "Availability.effective_state": ["available", "busy", "offline", "unknown"],
    "ContactReceipt.status": ["RECORDED", "REJECTED", "QUARANTINED"],
    "ConsultHandoff.status_ref": ["CREATED", "ACCEPTED", "REJECTED", "CANCELLED", "EXPIRED", "RETURN_PENDING", "RETURNED", "RETURN_FAILED", "CANCELLED_BY_SOURCE"],
    "MultiViewConsultation.budget_state": ["WITHIN_BUDGET", "BUDGET_EXCEEDED"],
    "MultiViewConsultation.timeout_state": ["WITHIN_TIME", "TIMED_OUT"],
    "MultiViewConsultation.result_state": ["PENDING", "IN_FLIGHT", "SUBMITTED", "ASSEMBLED", "TIMED_OUT", "BUDGET_EXCEEDED", "FAILED", "QUARANTINED"],
    "LegacyQuarantineRecord.legacy_kind": ["SINGLE_PROJECT_REVIEW", "AGENT_CENTER_SESSION", "UNMAPPABLE_EXECUTION", "UNMAPPABLE_SUMMARY", "UNMAPPABLE_OTHER"]
  },
  "identity_disjointness": "MemberId, TemporaryAgentId, AdvisoryId, ConsultationId, ChildRunRef, provider handles, model ids, thread ids, and process ids occupy disjoint identifier spaces and never substitute for one another.",
  "handoff_specialization": "ConsultHandoff references an exact M3 Handoff revision. It does not create a parallel handoff aggregate or grant project write capability.",
  "decision_owner_rule": "User adoption of an advisory creates only an imported DecisionRequest. Each later project mutation remains an independent authoritative command and grant owned by that project owner.",
  "directory_not_authority": "CapabilityPermissionRef and Availability are observations. They never authorize a command, grant, or contact expansion."
}
```

## contract.truth-source

Truth for M6-owned records is the exact join of the declared required fields plus
the imported owner receipts those fields reference. Directory rows, UI labels,
session names, provider threads, cached summaries, and model prose are not truth.

- `ProjectSummary` truth stays with each project projector and is admitted only
  through `ProjectSummaryQueryPort` after the consumer gate.
- `Handoff` truth stays with `handoff_aggregate`. `ConsultHandoff` is a typed
  reference to that truth.
- `DecisionRequest` truth stays with `SOURCE_OWNER_REF`.
- Per-project application truth stays with each project owner's authoritative
  command, grant, and receipt. `AdvisoryApplicationProjection` only cites those
  receipts.
- Temporary-agent identity truth is the complete M5 execution envelope. A worker
  report body, missing-field fill, or runtime trace is not identity.
- Child runs remain M5 execution references. They are not members and do not
  create organization hierarchy.

## contract.legal-states

The schema `legal_states` map is the sole enum authority for every state-bearing
export in this contract. Advisory lifecycle never includes application outcomes.
Application outcomes never rewrite advisory lifecycle. Temporary-agent lifecycle
never becomes a StableMember lifecycle except by an explicit human promotion that
creates a new `StableMember` with `promoted_from` and leaves the original
`TemporaryAgent` typed as temporary.

## contract.cross-scope

Cross-project read is permitted only as versioned `ProjectSummary` consumption
through `ProjectSummaryQueryPort` under an exact global `RoleSession`, global
scope, unexpired consumer, and policy decision. The consumer must also name the
project owner. Model-side dereference of a source link must re-enter that
project owner's policy gateway. A user click on a deep link is navigation, not
admission of raw project material into the global session.

Any join across project, owner, advisory, handoff, summary version, watermark,
hash, member type, or execution envelope that is missing or mismatched fails
closed before write.

## contract.project-summary-consumer-gate

```json consumer-gate
{
  "only_port": "ProjectSummaryQueryPort",
  "required_consumer": {
    "role_session": "exact global RoleSession",
    "scope": "exact global scope; project scope of another owner is foreign",
    "expiry": "consumer_expires_at must be in the future at judgement time",
    "policy": "policy_decision_ref must be an allow decision for this read"
  },
  "required_summary_fields": [
    "summary_id",
    "project_id",
    "schema_version",
    "version",
    "source_watermark",
    "summary_hash",
    "source_refs"
  ],
  "required_owner": "project_owner_ref",
  "forbidden_reads": [
    "project_store",
    "project_projection_other_than_ProjectSummaryQueryPort",
    "project_root",
    "raw_transcript",
    "secret",
    "untrimmed_memory"
  ],
  "forbidden_writes": [
    "project_aggregate",
    "project_summary_source",
    "runner",
    "grant",
    "workflow"
  ]
}
```

## contract.freshness

```json freshness-state-machine
{
  "states": ["fresh", "stale", "missing", "denied", "degraded"],
  "non_degrading": true,
  "precedence": ["denied", "missing", "degraded", "stale", "fresh"],
  "evaluation_order": [
    {
      "when": "consumer RoleSession missing, not global, expired, or policy deny",
      "state": "denied"
    },
    {
      "when": "requested project owner mismatch, foreign project owner, or consumer scope does not include the project",
      "state": "denied"
    },
    {
      "when": "read is not through ProjectSummaryQueryPort or attempts a forbidden read or writeback",
      "state": "denied"
    },
    {
      "when": "no summary record exists for the requested project and the consumer is otherwise authorized",
      "state": "missing"
    },
    {
      "when": "summary exists but a required field is absent, unreadable, or the projector reports an incomplete or unverifiable rebuild",
      "state": "degraded"
    },
    {
      "when": "required fields are present and authorized but schema_version, version, source_watermark, or summary_hash is behind the current projector watermark or does not match the expected exact values",
      "state": "stale"
    },
    {
      "when": "port is ProjectSummaryQueryPort, consumer gate exact, owner exact, every required field present, and version plus watermark plus hash match the current projector values",
      "state": "fresh"
    }
  ],
  "forbidden_transitions": [
    "denied must not be represented as stale, missing, degraded, or fresh",
    "missing must not be represented as stale, denied, degraded, or fresh",
    "degraded must not be represented as stale, missing, denied, or fresh",
    "stale must not be represented as fresh",
    "denied, missing, and degraded must not be replaced with cached content",
    "stale must not be replaced with a previous fresh cache as if current"
  ],
  "cache_rule": "A prior fresh or stale payload is not a substitute for a current denied, missing, or degraded judgement. Cache reuse is always false for those three states.",
  "join_rule": "Only freshness_state=fresh may enter CrossProjectAdvisory.consumed_summaries. A stale summary cannot be consumed as fresh. A later source_watermark or version change on an already recorded advisory marks that advisory stale and never overwrites its historical consumed_summaries.",
  "field_absence_rule": "Absence of source_watermark, summary_hash, schema_version, version, summary_id, project_id, or project_owner_ref at join time is fail-closed zero-write. It is not tolerated as stale and is not filled from cache."
}
```

## contract.advisory-exact-join

```json advisory-exact-join
{
  "required": [
    "advisory_id",
    "global_role_session_id",
    "consult_handoff_ref",
    "consumed_summaries[]",
    "policy_decision_ref",
    "generated_at",
    "source_links[]"
  ],
  "consult_handoff_exact_required": [
    "handoff_id",
    "handoff_revision",
    "status_ref",
    "receipt_ref"
  ],
  "each_consumed_summary_required": [
    "summary_id",
    "project_id",
    "project_owner_ref",
    "schema_version",
    "version",
    "source_watermark",
    "summary_hash",
    "policy_decision_ref",
    "freshness_state=fresh",
    "source_refs"
  ],
  "failure": "FAIL_CLOSED_ZERO_WRITE",
  "mismatch_examples": [
    "missing watermark",
    "hash mismatch",
    "schema_version mismatch",
    "version mismatch",
    "project_id mismatch",
    "owner mismatch",
    "handoff revision mismatch",
    "non-global RoleSession",
    "freshness_state other than fresh"
  ]
}
```

## contract.adoption-and-application

```json adoption-application-boundary
{
  "user_adoption": {
    "creates": ["DecisionRequest"],
    "does_not_create": [
      "project command",
      "ExecutionGrant",
      "workflow",
      "project fact",
      "advisory lifecycle change to applied"
    ],
    "mutated_targets": ["DecisionRequest.status"]
  },
  "per_project_apply": {
    "actor": "each project owner independently",
    "path": "authoritative command / grant / receipt",
    "m6_role": "observe receipts only"
  },
  "AdvisoryApplicationProjection": {
    "outcomes": ["applied", "failed", "rolled_back", "unknown"],
    "owns_project_result": false,
    "changes_advisory_lifecycle": false,
    "partial_apply": "two or more observations for the same advisory_id and decision_request_id may carry different outcomes; the projection records partial_apply=true and never invents a compensating write",
    "compensation": "rollback or compensation is observed only when a later authoritative project-owner receipt with outcome=rolled_back or a dedicated compensation receipt arrives",
    "history": "append-only immutable observations; a later observation never overwrites an earlier one"
  }
}
```

## contract.members

`StableMember` is identified only by `MemberId`. `TemporaryAgent` is identified
only by `TemporaryAgentId` bound to a complete `ExecutionEnvelope`. Display
names are labels. Same-name collision never merges, substitutes, or promotes
records; both records remain and an explicit human choice is required.

Provider, model, thread, and process identifiers are never member identity.
Replacing a provider or runtime must not change `MemberId`, remembered refs, or
permissions.

Capability and permission exist in the directory only as `CapabilityPermissionRef`
values with `source + revision + observed_at`. The directory is never the
authorization authority. A later policy revision is observed as a new ref; the
old ref is retained.

Availability carries `source`, `observed_at`, and `ttl`. When
`now > observed_at + ttl`, `effective_state` becomes `unknown`. Unknown or
stale availability cannot authorize a command, grant, assignment, or contact
expansion.

Contact records a `ContactReceipt` and may start a session or M3 `Handoff`. It
grants no capability.

Deactivation sets `membership_lifecycle=DEACTIVATED` and retains historical
refs. Physical deletion is out of M6 and is deferred until a later M9 decision.

Explicit human promotion creates or binds a `StableMember` with
`promoted_from=<TemporaryAgentId>` and a `PromotionBinding`. The source
`TemporaryAgent` keeps its type, envelope, and history.

## contract.temporary-agent-envelope

```json execution-envelope
{
  "required_fields": [
    "project_id",
    "orchestration_id",
    "workflow_run_id",
    "work_item_id",
    "node_id",
    "dispatch_id",
    "attempt_id",
    "grant_id",
    "worker_role_session_id",
    "authoritative_receipt_ref",
    "trusted_actor_id",
    "hashes"
  ],
  "forbidden_identity_sources": [
    "report self-claim",
    "missing-field compatibility",
    "runtime-trace derivation",
    "session name",
    "parent/child session heuristic"
  ],
  "rebuild_rule": "TemporaryAgent is rebuilt only from these immutable refs. Report bodies are not copied into the projection.",
  "child_run_rule": "ChildRunRef is reference-only. It creates no StableMember, TemporaryAgent, or organization hierarchy."
}
```

## contract.multi-view

A `MultiViewConsultation` dispatches the same minimal sourced `QuestionPacket`
to two or more independent views. Each view has its own `RoleSession`,
`workcell_ref`, and `context_packet_ref`. Before `submitted=true`, a view must
not read another view's `conclusion_ref`. After submit, the system may assemble
side-by-side consensus, disagreement, and evidence indexes.

Budget and timeout are explicit states, not implicit aborts. The only product
result of a consultation is a user-pending `DecisionRequest`. Every consultation
records an explicit `budget_limit_ref` and `deadline_at` alongside its budget and
timeout states. The consultation produces no project command, grant, or formal
fact.

## contract.formal-actions

```json action-flow
[
  {
    "id": "consume-project-summary",
    "command": "ConsumeProjectSummary",
    "policy": "m6-project-summary-consumer-policy",
    "state_owner": "global_supervisor_domain",
    "state_target": "FreshnessJudgement.freshness_state",
    "preconditions": [
      "query_port is ProjectSummaryQueryPort",
      "consumer global RoleSession, scope, expiry, and policy are exact",
      "project_owner_ref is exact",
      "no direct project store, projection, root, transcript, or secret read",
      "no project writeback"
    ],
    "state": "NONE->fresh|stale|missing|denied|degraded",
    "event": "ProjectSummaryFreshnessJudged",
    "audit": "SCRUBBED_SUMMARY_CONSUME_RECORD",
    "outbox": {"mode": "NONE", "reason": "consumption is a read judgement"},
    "failure": "FAIL_CLOSED"
  },
  {
    "id": "record-cross-project-advisory",
    "command": "RecordCrossProjectAdvisory",
    "policy": "m6-advisory-join-policy",
    "state_owner": "global_supervisor_domain",
    "state_target": "CrossProjectAdvisory.lifecycle_status",
    "preconditions": [
      "advisory_id, global_role_session_id, consult_handoff_ref, policy_decision_ref, generated_at, and source_links present",
      "consult_handoff_ref resolves exact handoff_id, handoff_revision, status_ref, and receipt_ref",
      "each consumed summary has summary_id, project_id, project_owner_ref, schema_version, version, source_watermark, summary_hash, policy_decision_ref, freshness_state=fresh, and source_refs",
      "every join field exact"
    ],
    "state": "NONE->ISSUED|QUARANTINED",
    "event": "CrossProjectAdvisoryRecorded",
    "audit": "SCRUBBED_ADVISORY_RECORD",
    "outbox": {"mode": "NONE", "reason": "advisory recording is internal advice state"},
    "failure": "FAIL_CLOSED"
  },
  {
    "id": "mark-advisory-stale",
    "command": "MarkAdvisoryStaleOnWatermarkChange",
    "policy": "m6-advisory-freshness-policy",
    "state_owner": "global_supervisor_domain",
    "state_target": "CrossProjectAdvisory.lifecycle_status",
    "preconditions": [
      "existing advisory and immutable consumed_summaries history are exact",
      "current ProjectSummary version or source_watermark differs from the recorded value",
      "no silent recompute and no history overwrite"
    ],
    "state": "ISSUED->STALE",
    "event": "CrossProjectAdvisoryMarkedStale",
    "audit": "SCRUBBED_ADVISORY_STALENESS_RECORD",
    "outbox": {"mode": "NONE", "reason": "staleness records source drift without project mutation"},
    "failure": "FAIL_CLOSED"
  },
  {
    "id": "adopt-advisory",
    "command": "AdoptCrossProjectAdvisory",
    "policy": "m6-advisory-adoption-policy",
    "state_owner": "SOURCE_OWNER_REF",
    "state_target": "DecisionRequest.status",
    "preconditions": [
      "CrossProjectAdvisory.lifecycle_status is ISSUED",
      "actor is the required user",
      "no project command, grant, or fact is created"
    ],
    "state": "NONE->PENDING",
    "event": "AdvisoryAdoptionDecisionRequested",
    "audit": "SCRUBBED_ADOPTION_RECORD",
    "outbox": {"mode": "NONE", "reason": "adoption creates only a DecisionRequest"},
    "failure": "FAIL_CLOSED"
  },
  {
    "id": "observe-application-receipt",
    "command": "ObserveAdvisoryApplicationReceipt",
    "policy": "m6-application-projection-policy",
    "state_owner": "m6_read_model",
    "state_target": "PerProjectApplicationObservation.outcome",
    "preconditions": [
      "DecisionRequest exists",
      "authoritative project-owner command receipt and grant path are exact",
      "advisory lifecycle is not mutated",
      "projection does not own the project result"
    ],
    "state": "NONE->applied|failed|rolled_back|unknown",
    "event": "AdvisoryApplicationObserved",
    "audit": "SCRUBBED_APPLICATION_OBSERVATION",
    "outbox": {"mode": "NONE", "reason": "observation cites an existing project-owner receipt"},
    "failure": "FAIL_CLOSED"
  },
  {
    "id": "register-stable-member",
    "command": "RegisterStableMember",
    "policy": "m6-member-identity-policy",
    "state_owner": "organization_directory",
    "state_target": "StableMember.membership_lifecycle",
    "preconditions": [
      "MemberId is new and not a TemporaryAgentId",
      "identity is not a provider, model, thread, or process",
      "same display name does not merge records"
    ],
    "state": "NONE->ESTABLISHED|QUARANTINED",
    "event": "StableMemberRegistered",
    "audit": "SCRUBBED_MEMBER_RECORD",
    "outbox": {"mode": "NONE", "reason": "directory write is internal"},
    "failure": "FAIL_CLOSED"
  },
  {
    "id": "promote-temporary-agent",
    "command": "PromoteTemporaryAgent",
    "policy": "m6-member-promotion-policy",
    "state_owner": "organization_directory",
    "state_target": "StableMember.membership_lifecycle",
    "preconditions": [
      "explicit human command",
      "TemporaryAgent envelope exact",
      "promoted_from set to TemporaryAgentId",
      "source TemporaryAgent type and history unchanged"
    ],
    "state": "NONE->ESTABLISHED|ACTIVE",
    "event": "TemporaryAgentPromoted",
    "audit": "SCRUBBED_PROMOTION_RECORD",
    "outbox": {"mode": "NONE", "reason": "promotion is an internal directory bind"},
    "failure": "FAIL_CLOSED"
  },
  {
    "id": "observe-availability",
    "command": "ObserveMemberAvailability",
    "policy": "m6-availability-policy",
    "state_owner": "directory_projection",
    "state_target": "Availability.effective_state",
    "preconditions": [
      "source, observed_at, and ttl present",
      "stale TTL yields unknown",
      "availability is not used as permission"
    ],
    "state": "NONE->available|busy|offline|unknown",
    "event": "AvailabilityObserved",
    "audit": "SCRUBBED_AVAILABILITY_RECORD",
    "outbox": {"mode": "NONE", "reason": "availability is a directory observation"},
    "failure": "FAIL_CLOSED"
  },
  {
    "id": "record-contact",
    "command": "RecordMemberContact",
    "policy": "m6-contact-policy",
    "state_owner": "organization_directory",
    "state_target": "ContactReceipt.status",
    "preconditions": [
      "target is a StableMember",
      "capability_granted is false",
      "directory is not treated as grant authority"
    ],
    "state": "NONE->RECORDED|REJECTED|QUARANTINED",
    "event": "MemberContactRecorded",
    "audit": "SCRUBBED_CONTACT_RECORD",
    "outbox": {"mode": "OPTIONAL", "reason": "session or handoff start may be an external effect later"},
    "failure": "FAIL_CLOSED"
  },
  {
    "id": "project-temporary-agent",
    "command": "ProjectTemporaryAgent",
    "policy": "m6-temporary-agent-envelope-policy",
    "state_owner": "execution_history_projection",
    "state_target": "TemporaryAgent.membership_lifecycle",
    "preconditions": [
      "complete ExecutionEnvelope present",
      "no report self-claim",
      "no missing-field compatibility",
      "no runtime-trace derivation"
    ],
    "state": "NONE->PROJECTED|QUARANTINED",
    "event": "TemporaryAgentProjected",
    "audit": "SCRUBBED_TEMPORARY_AGENT_RECORD",
    "outbox": {"mode": "NONE", "reason": "projection copies refs only"},
    "failure": "FAIL_CLOSED"
  },
  {
    "id": "start-multi-view-consultation",
    "command": "StartMultiViewConsultation",
    "policy": "m6-consultation-isolation-policy",
    "state_owner": "consultation_domain",
    "state_target": "MultiViewConsultation.result_state",
    "preconditions": [
      "one shared minimal sourced QuestionPacket",
      "each view has a distinct RoleSession, workcell_ref, and context_packet_ref",
      "peer_conclusions_readable_before_submit is false"
    ],
    "state": "NONE->PENDING|IN_FLIGHT|QUARANTINED",
    "event": "MultiViewConsultationStarted",
    "audit": "SCRUBBED_CONSULTATION_RECORD",
    "outbox": {"mode": "NONE", "reason": "consultation start is internal coordination"},
    "failure": "FAIL_CLOSED"
  },
  {
    "id": "submit-consultation-view",
    "command": "SubmitConsultationView",
    "policy": "m6-consultation-isolation-policy",
    "state_owner": "consultation_domain",
    "state_target": "ConsultationView.submitted",
    "preconditions": [
      "view has not read peer conclusions",
      "question_packet_hash matches the shared packet",
      "RoleSession and workcell remain the view's own"
    ],
    "state": "false->true",
    "event": "ConsultationViewSubmitted",
    "audit": "SCRUBBED_CONSULTATION_VIEW_RECORD",
    "outbox": {"mode": "NONE", "reason": "submit records an isolated opinion"},
    "failure": "FAIL_CLOSED"
  },
  {
    "id": "assemble-consultation-result",
    "command": "AssembleMultiViewConsultation",
    "policy": "m6-consultation-result-policy",
    "state_owner": "consultation_domain",
    "state_target": "MultiViewConsultation.result_state",
    "preconditions": [
      "budget_state and timeout_state are explicit",
      "result creates only a user-pending DecisionRequest",
      "no command, grant, or fact is produced"
    ],
    "state": "SUBMITTED->ASSEMBLED|TIMED_OUT|BUDGET_EXCEEDED|FAILED",
    "event": "MultiViewConsultationAssembled",
    "audit": "SCRUBBED_CONSULTATION_RESULT",
    "outbox": {"mode": "NONE", "reason": "assembly produces only a pending decision"},
    "failure": "FAIL_CLOSED"
  },
  {
    "id": "quarantine-unmappable-legacy",
    "command": "QuarantineUnmappableLegacy",
    "policy": "m6-legacy-migration-policy",
    "state_owner": "unknown_quarantine_repository",
    "state_target": "LegacyQuarantineRecord.mapped_to",
    "preconditions": [
      "record cannot be mapped field-for-field onto this contract",
      "no guess-fill",
      "payload_mode is REF_ONLY"
    ],
    "state": "NONE->null",
    "event": "LegacyRecordQuarantined",
    "audit": "SCRUBBED_QUARANTINE_RECORD",
    "outbox": {"mode": "NONE", "reason": "quarantine is an internal isolation write"},
    "failure": "FAIL_CLOSED"
  }
]
```

## contract.events

Events carry typed identifiers, revisions, freshness states, source refs,
receipt refs, and hashes. They never carry raw files, raw summaries, transcripts,
prompts, provider responses, secrets, stdout, stderr, or tool output.

## contract.audit

Consumer-gate denials, freshness judgements, advisory joins, adoption,
application observations, member registration, promotion, deactivation, contact,
availability staleness, envelope rejection, multi-view isolation breaches, and
legacy quarantine are audited as scrubbed records.

## contract.outbox

M6D01 defines no external effect. Later leaves may attach optional outbox items
for contact or notification delivery. Those items still cannot grant capability
or write a project.

## contract.sensitivity

```json forbidden-material
{
  "forbidden_fields": [
    "secret",
    "secret_value",
    "credential",
    "credential_value",
    "password",
    "token",
    "access_token",
    "refresh_token",
    "api_key",
    "private_key",
    "client_secret",
    "transcript",
    "raw_transcript",
    "transcript_body",
    "prompt",
    "prompt_body",
    "provider_response",
    "provider_payload",
    "provider_output",
    "stdout",
    "stderr",
    "tool_output",
    "raw_tool_output",
    "raw_file",
    "raw_summary",
    "untrimmed_memory"
  ],
  "allowed_opaque_refs_only": [
    "credential_ref",
    "secret_ref",
    "transcript_ref",
    "tool_output_ref",
    "payload_ref",
    "prompt_ref",
    "provider_response_ref",
    "stdout_ref",
    "stderr_ref",
    "provider_handle_ref"
  ],
  "rule": "Forbidden material never enters contract payloads, advisories, directory rows, consultation packets, or fixtures except as opaque refs in the allowed set."
}
```

## contract.idempotency

Advisory recording, adoption `DecisionRequest` creation, application
observation, member registration, promotion, contact, temporary-agent
projection, consultation start, and quarantine use stable idempotency keys.
A divergent replay of the same key is quarantined or rejected. It is never
merged by display name or heuristic.

## contract.failure

The following fail closed with zero mutation of project aggregates, advisory
history overwrite, or unauthorized directory promotion:

1. Missing or mismatched advisory join field.
2. Stale summary consumed as fresh.
3. Denied, missing, or degraded represented as stale or fresh, or replaced by cache.
4. Foreign project owner or owner mismatch.
5. Missing watermark, hash, schema version, or version at join time.
6. Direct project store, projection, root, transcript, or secret read.
7. Any M6 project writeback.
8. TemporaryAgent presented as StableMember, or the reverse.
9. Stale or unknown availability used as permission.
10. Incomplete execution envelope, report self-claim, missing-field compatibility, or runtime-trace derivation.
11. ChildRunRef used to create a member or organization hierarchy.
12. Adoption that writes a project command, grant, or fact.
13. Application projection that changes advisory lifecycle or claims a project result.
14. Multi-view peer-conclusion read before submit, or consultation result as command, grant, or fact.
15. Auto-promotion of a legacy single-project review or Agent Center session.
16. Unmappable legacy guess-filled into a first-class record.
17. Directory treated as authorization authority.
18. Provider, model, thread, or process used as member identity.
19. Forbidden sensitive material in a payload.

## contract.rollback

```json rollback-limits
{
  "allowed": [
    "return display to in-project review",
    "return display to Agent Center session listing",
    "export and rebuild the organization directory from retained refs",
    "observe per-project rolled_back receipts in AdvisoryApplicationProjection history"
  ],
  "forbidden": [
    "restore cross-project raw project read",
    "un-quarantine by guessing missing join fields",
    "overwrite advisory history to look fresh",
    "treat a rolled-back project as if the advisory lifecycle changed",
    "physically delete retained member or temporary-agent refs"
  ],
  "physical_deletion": "deferred past M6 to a later M9 decision"
}
```

## contract.compatibility

```json migration-matrix
{
  "rows": [
    {
      "source": "legacy single-project global review",
      "disposition": "LEGACY_ADAPTER_HISTORY",
      "auto_promote_to": null,
      "rule": "remains adapter or history; never becomes CrossProjectAdvisory"
    },
    {
      "source": "existing Agent Center session",
      "disposition": "NO_AUTO_PROMOTE",
      "auto_promote_to": null,
      "rule": "never becomes StableMember; only an explicit identity contract may enter the directory"
    },
    {
      "source": "TemporaryAgent",
      "disposition": "REBUILD_FROM_IMMUTABLE_REFS",
      "auto_promote_to": null,
      "rule": "rebuild from the execution envelope only; do not copy report bodies"
    },
    {
      "source": "summary version or watermark change",
      "disposition": "MARK_ADVISORY_STALE",
      "auto_promote_to": null,
      "rule": "mark the existing advisory stale; do not silently recompute or overwrite history"
    },
    {
      "source": "unmappable legacy record",
      "disposition": "QUARANTINE",
      "auto_promote_to": null,
      "rule": "enter LegacyQuarantineRecord with payload_mode=REF_ONLY; never guess-fill"
    }
  ]
}
```

## contract.fixtures

Fixtures live at `docs/contracts/fixtures/m6-org-001/`. Each case has `id`,
`contract_id`, `rule_id`, `polarity`, `category`, `input`, `expected_code`, and
`expected_mutated_targets`. Negative polarity requires rejection or quarantine
and `expected_mutated_targets=[]`. These fixtures are offline mechanical cases.
They are not a running service, repository, or UI.

`rule_id` names a formal action id except for the three reference-only invariant
rules `consult-handoff-reference-only`, `capability-permission-read-only`, and
`child-run-reference-only`; those three validate an exact imported/ref shape and
cannot create or mutate the imported owner record.

## contract.non-goals

This leaf does not implement Global Supervisor sessions, query services, member
directories, temporary-agent stores, consultation runtimes, isolated App
acceptance, real providers, or any `src-tauri` behavior. It does not register
this contract in the M1 manifest. It does not authorize push, merge, deploy, or
release.

## contract.holds

Real provider selection, physical deletion, isolated new-shell acceptance, and
all service / repository / projection implementation remain later leaves. No M1
`hold_refs` are added, because this addendum is outside the frozen M1 registry.
