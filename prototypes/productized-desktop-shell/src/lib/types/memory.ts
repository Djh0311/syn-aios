export type MemoryScope = {
  scope_id: string;
  scope_type:
    | "user_preference"
    | "global"
    | "project"
    | "workflow"
    | "session"
    | "role_limited"
    | "document_limited";
  user_id?: string | null;
  project_id?: string | null;
  workflow_id?: string | null;
  session_id?: string | null;
  role_ids: string[];
  document_refs: string[];
  permission_policy_ref?: string | null;
  model_export_policy: "local_only" | "allowed_with_redaction" | "blocked";
  valid_from: string;
  valid_until?: string | null;
};

export type MemorySourceRef = {
  source_ref_id: string;
  source_type:
    | "user_confirmed_proposal"
    | "workflow_summary"
    | "stage_report"
    | "director_review"
    | "handoff"
    | "evidence"
    | "audit_event"
    | "session_summary"
    | "knowledge_doc"
    | "observation_ref"
    | "manual_note";
  source_id?: string | null;
  source_path?: string | null;
  source_title?: string | null;
  anchor?: string | null;
  source_created_at?: string | null;
  captured_at: string;
  authority_level:
    | "user_confirmed"
    | "current_authority_doc"
    | "audit"
    | "evidence"
    | "handoff"
    | "derived_summary"
    | "knowledge_material"
    | "unverified_note";
  sensitive_level: "public" | "project" | "private" | "secret";
  content_hash?: string | null;
};

export type MemoryLifecycleStatus =
  | "candidate_draft"
  | "candidate_needs_review"
  | "candidate_confirmed"
  | "candidate_rejected"
  | "candidate_quarantined"
  | "candidate_superseded"
  | "candidate_discarded"
  | "memory_active"
  | "memory_conflicted"
  | "memory_deprecated"
  | "memory_frozen"
  | "memory_archived";

export type MemoryAuditRef = {
  audit_ref_id: string;
  audit_event_id?: string | null;
  event_type: string;
  actor_id: string;
  actor_role: "user" | "secretary" | "project_director" | "system" | "agent" | string;
  target_kind: "memory_candidate" | "memory_record" | "memory_conflict" | string;
  target_id: string;
  before_status?: MemoryLifecycleStatus | null;
  after_status?: MemoryLifecycleStatus | null;
  reason: string;
  created_at: string;
};

export type MemoryConflict = {
  conflict_id: string;
  conflict_type: string;
  left_ref: string;
  right_ref: string;
  severity: "low" | "medium" | "high" | "blocking";
  status: "open" | "acknowledged" | "resolved" | "dismissed";
  summary: string;
  recommended_action: string;
  source_refs: MemorySourceRef[];
  audit_refs: MemoryAuditRef[];
  created_at: string;
  updated_at: string;
};

export type MemoryCandidateAdoptionRef = {
  adopted_memory_id: string;
  adopted_version_id: string;
  adopted_audit_event_id: string;
  adopted_at: string;
  adopted_by_role: string;
  adoption_reason: string;
};

export type MemoryCandidate = {
  candidate_id: string;
  candidate_key: string;
  schema_version: "memory_governance.v1";
  scope: MemoryScope;
  memory_type:
    | "user_preference"
    | "global_blueprint"
    | "project_memory"
    | "workflow_summary"
    | "session_summary"
    | "mature_pattern";
  claim: string;
  body: string;
  source_refs: MemorySourceRef[];
  generated_by_role: string;
  generated_from:
    | "explicit_user_confirmation"
    | "workflow_closeout"
    | "stage_handoff"
    | "secretary_suggestion"
    | "knowledge_summary"
    | "manual_entry"
    | `observation:${string}`;
  status: MemoryLifecycleStatus;
  risk_level: "low" | "medium" | "high";
  sensitive_level: "public" | "project" | "private" | "secret";
  requires_user_confirmation: boolean;
  review_reason: string;
  conflicts: MemoryConflict[];
  audit_refs: MemoryAuditRef[];
  adoption?: MemoryCandidateAdoptionRef | null;
  created_at: string;
  updated_at: string;
};

export type MemoryRecord = {
  memory_id: string;
  schema_version: "memory_governance.v1";
  record_version: number;
  scope: MemoryScope;
  memory_type: MemoryCandidate["memory_type"];
  claim: string;
  body: string;
  source_refs: MemorySourceRef[];
  status: MemoryLifecycleStatus;
  supersedes_memory_id?: string | null;
  superseded_by_memory_id?: string | null;
  conflict_refs: string[];
  audit_refs: MemoryAuditRef[];
  created_at: string;
  updated_at: string;
};

export type MemoryVersion = {
  version_id: string;
  memory_id: string;
  version_number: number;
  change_type:
    | "created"
    | "manual_revision"
    | "deprecated"
    | "frozen"
    | "unfrozen"
    | "archived"
    | "merged_target_revision"
    | "merged_record_created"
    | "merged_source_deprecated"
    | "split_record_created"
    | "split_source_deprecated"
    | "promoted_to_global"
    | "demoted_to_project";
  change_summary: string;
  record_snapshot: MemoryRecord;
  source_refs: MemorySourceRef[];
  changed_by_role: "user" | "project_director" | "global_director" | "system";
  reviewed_by?: string | null;
  created_at: string;
};

export type MemoryAuditEvent = {
  audit_event_id: string;
  event_type:
    | "memory_record_created"
    | "memory_record_create_rejected"
    | "memory_candidate_adopted_to_formal_memory"
    | `formal_memory_${FormalMemoryLifecycleOperationKind}_recorded`;
  actor_id: string;
  actor_role: "user" | "project_director" | "global_director" | "system";
  project_id?: string | null;
  workflow_id?: string | null;
  session_id?: string | null;
  target_kind: "memory_record" | "memory_lifecycle_operation";
  target_id?: string | null;
  before_state?: string | null;
  after_state?: string | null;
  reason: string;
  source_refs: MemorySourceRef[];
  status: "succeeded" | "failed";
  created_at: string;
};

export type MemoryEntityKind =
  | "project"
  | "workflow"
  | "session"
  | "role"
  | "knowledge_doc"
  | "tool"
  | "model"
  | "harness"
  | "proposal"
  | "memory_record"
  | "memory_candidate";

export type MemoryRelationKind = "entity" | "temporal" | "causal" | "semantic";

export type MemoryRelationSourceKind =
  | "manual"
  | "formal_memory"
  | "memory_candidate"
  | "observation"
  | "knowledge_doc"
  | "task_package"
  | "llm_inferred"
  | "similarity_hit";

export type MemoryRelationStatus = "candidate" | "confirmed" | "rejected" | "quarantined" | "conflicted";

export type MemoryEntityAliasDecisionKind = "confirm_alias" | "reject_alias";

export type MemoryEntityMergeDecisionKind = "confirm_merge" | "reject_merge";

export type MemoryRelationCandidateDecisionKind = "confirm_relation" | "reject_relation" | "quarantine_relation";

export type MemoryRelationSource = {
  source_kind: MemoryRelationSourceKind;
  source_id?: string | null;
  source_path?: string | null;
  source_title?: string | null;
  authority_level: string;
  sensitive_level: string;
};

export type MemoryEntityAlias = {
  alias_id: string;
  alias: string;
  source_kind: MemoryRelationSourceKind;
  source_id?: string | null;
  created_at: string;
};

export type MemoryEntity = {
  entity_id: string;
  entity_kind: MemoryEntityKind;
  canonical_key: string;
  display_name: string;
  aliases: MemoryEntityAlias[];
  source_refs: MemoryRelationSource[];
  status: string;
  created_at: string;
  updated_at: string;
  warnings: string[];
};

export type MemoryEntityRegistry = {
  entities: MemoryEntity[];
  updated_at: string;
  warnings: string[];
};

export type MemoryEntityCandidate = {
  candidate_id: string;
  entity_kind: MemoryEntityKind;
  display_name: string;
  normalized_key: string;
  source_kind: MemoryRelationSourceKind;
  source_id?: string | null;
  source_path?: string | null;
  source_title?: string | null;
  source_refs: MemoryRelationSource[];
  confidence_kind: string;
  status: MemoryRelationStatus;
  reason: string;
  created_at: string;
  warnings: string[];
};

export type MemoryEntityMergeCandidate = {
  merge_candidate_id: string;
  left_entity_candidate_id: string;
  right_entity_candidate_id: string;
  left_label: string;
  right_label: string;
  normalized_key: string;
  source_kind: MemoryRelationSourceKind;
  status: MemoryRelationStatus;
  requires_user_confirmation: boolean;
  reason: string;
  created_at: string;
  warnings: string[];
};

export type MemoryRelationCandidate = {
  candidate_id: string;
  relation_kind: MemoryRelationKind;
  subject_entity_id: string;
  object_entity_id: string;
  subject_label: string;
  object_label: string;
  predicate: string;
  source_kind: MemoryRelationSourceKind;
  source_refs: MemoryRelationSource[];
  confidence_kind: string;
  status: MemoryRelationStatus;
  requires_user_confirmation: boolean;
  reason: string;
  created_at: string;
  warnings: string[];
};

export type MemoryRelation = {
  relation_id: string;
  relation_kind: MemoryRelationKind;
  subject_entity_id: string;
  object_entity_id: string;
  subject_label: string;
  object_label: string;
  predicate: string;
  source_kind: MemoryRelationSourceKind;
  source_refs: MemoryRelationSource[];
  status: MemoryRelationStatus;
  confirmed_by: string;
  confirmation_role: string;
  confirmation_reason: string;
  created_at: string;
  updated_at: string;
  warnings: string[];
};

export type MemoryRelationAuditEvent = {
  audit_event_id: string;
  event_type: string;
  actor_id: string;
  actor_role: string;
  target_kind: string;
  target_id: string;
  before_status?: MemoryRelationStatus | null;
  after_status?: MemoryRelationStatus | null;
  reason: string;
  created_at: string;
  warnings: string[];
};

export type MemoryEntityRelationStoreV1 = {
  store_version: "memory_entity_relations.v1";
  project_id?: string | null;
  workflow_id?: string | null;
  revision: number;
  registry: MemoryEntityRegistry;
  entity_candidates: MemoryEntityCandidate[];
  merge_candidates: MemoryEntityMergeCandidate[];
  relation_candidates: MemoryRelationCandidate[];
  relations: MemoryRelation[];
  audit_events: MemoryRelationAuditEvent[];
  updated_at: string;
  warnings: string[];
};

export type MemoryEntityRelationStoreSummary = {
  sidecar_name: "memory-entity-relations.v1.json" | string;
  revision: number;
  entity_count: number;
  entity_candidate_count: number;
  merge_candidate_count: number;
  relation_candidate_count: number;
  confirmed_relation_count: number;
  display_text: string;
  warnings: string[];
};

export type PreviewMemoryEntityRelationCandidatesInput = {
  project_root: string;
  project_id?: string | null;
  workflow_id?: string | null;
};

export type MemoryEntityRelationPreviewOutput = {
  store_revision: number;
  entity_candidates: MemoryEntityCandidate[];
  merge_candidates: MemoryEntityMergeCandidate[];
  relation_candidates: MemoryRelationCandidate[];
  summary: MemoryEntityRelationStoreSummary;
  warnings: string[];
};

export type RecordMemoryEntityAliasDecisionInput = {
  project_root: string;
  entity_candidate_id: string;
  decision: MemoryEntityAliasDecisionKind;
  actor_id: string;
  actor_role: "project_director" | "user" | "global_director" | string;
  reason: string;
  expected_store_revision?: number | null;
};

export type RecordMemoryEntityMergeDecisionInput = {
  project_root: string;
  merge_candidate_id: string;
  decision: MemoryEntityMergeDecisionKind;
  actor_id: string;
  actor_role: "project_director" | "user" | "global_director" | string;
  confirmed_by?: "project_director" | "user" | string | null;
  reason: string;
  expected_store_revision?: number | null;
};

export type RecordMemoryRelationCandidateDecisionInput = {
  project_root: string;
  relation_candidate_id: string;
  decision: MemoryRelationCandidateDecisionKind;
  actor_id: string;
  actor_role: "project_director" | "user" | "global_director" | string;
  confirmed_by?: "project_director" | "user" | string | null;
  reason: string;
  expected_store_revision?: number | null;
};

export type RecordMemoryEntityAliasDecisionOutput = {
  store_revision: number;
  entity?: MemoryEntity | null;
  candidate: MemoryEntityCandidate;
  audit_event: MemoryRelationAuditEvent;
  warnings: string[];
};

export type RecordMemoryEntityMergeDecisionOutput = {
  store_revision: number;
  entity?: MemoryEntity | null;
  merge_candidate: MemoryEntityMergeCandidate;
  audit_event: MemoryRelationAuditEvent;
  warnings: string[];
};

export type RecordMemoryRelationCandidateDecisionOutput = {
  store_revision: number;
  relation?: MemoryRelation | null;
  relation_candidate: MemoryRelationCandidate;
  audit_event: MemoryRelationAuditEvent;
  warnings: string[];
};

export type MemoryRelationTaskExplanation = {
  relation_id: string;
  relation_kind: MemoryRelationKind;
  linked_entity_id: string;
  linked_label: string;
  explanation: string;
  source_count: number;
};

export type MaturePatternCandidateStatus = "candidate" | "confirmed" | "rejected" | "quarantined" | "changes_requested";

export type MaturePatternDecisionKind = "confirm_as_formal_memory" | "reject" | "quarantine" | "request_changes";

export type MemoryClusterMemberRef = {
  member_ref_id: string;
  member_kind: string;
  member_id: string;
  project_id?: string | null;
  title: string;
  source_refs: MemorySourceRef[];
};

export type MaturePatternCandidate = {
  candidate_id: string;
  pattern_kind: string;
  scope: MemoryScope;
  title: string;
  claim: string;
  body: string;
  source_refs: MemorySourceRef[];
  member_refs: MemoryClusterMemberRef[];
  signal_refs: string[];
  status: MaturePatternCandidateStatus;
  requires_user_confirmation: boolean;
  review_summary: string;
  created_at: string;
  updated_at: string;
  warnings: string[];
};

export type MemoryClusterReport = {
  report_id: string;
  report_kind: string;
  scope_type: string;
  title: string;
  project_ids: string[];
  member_refs: MemoryClusterMemberRef[];
  source_refs: MemorySourceRef[];
  status: string;
  staleness: string;
  display_text: string;
  created_at: string;
  warnings: string[];
};

export type MaturePatternAuditEvent = {
  audit_event_id: string;
  event_type: string;
  actor_id: string;
  actor_role: string;
  target_kind: string;
  target_id: string;
  before_status?: MaturePatternCandidateStatus | null;
  after_status?: MaturePatternCandidateStatus | null;
  formal_memory_id?: string | null;
  reason: string;
  created_at: string;
  warnings: string[];
};

export type MemoryPatternStoreV1 = {
  store_version: "memory_patterns.v1";
  project_id?: string | null;
  workflow_id?: string | null;
  revision: number;
  mature_pattern_candidates: MaturePatternCandidate[];
  cluster_reports: MemoryClusterReport[];
  audit_events: MaturePatternAuditEvent[];
  updated_at: string;
  warnings: string[];
};

export type MemorySystemAcceptanceGate = {
  gate_id: string;
  label: string;
  status: string;
  evidence: string;
  blocking_reason?: string | null;
};

export type MemorySystemAcceptanceSummary = {
  summary_id: string;
  scope_label: string;
  gate_count: number;
  passed_count: number;
  blocked_count: number;
  deferred_count: number;
  gates: MemorySystemAcceptanceGate[];
  display_text: string;
  warnings: string[];
  created_at: string;
};

export type MemoryPatternStoreSummary = {
  sidecar_name: string;
  revision: number;
  mature_pattern_candidate_count: number;
  cluster_report_count: number;
  confirmed_pattern_count: number;
  display_text: string;
  warnings: string[];
};

export type PreviewMaturePatternsInput = {
  project_root: string;
  project_id?: string | null;
  workflow_id?: string | null;
};

export type MaturePatternPreviewOutput = {
  store_revision: number;
  mature_pattern_candidates: MaturePatternCandidate[];
  cluster_reports: MemoryClusterReport[];
  acceptance_summary: MemorySystemAcceptanceSummary;
  summary: MemoryPatternStoreSummary;
  warnings: string[];
};

export type RecordMaturePatternDecisionInput = {
  project_root: string;
  candidate_id: string;
  decision: MaturePatternDecisionKind;
  actor_id: string;
  actor_role: "user" | "project_director" | "global_director" | string;
  confirmed_by?: "user" | string | null;
  reason: string;
  expected_pattern_store_revision?: number | null;
  expected_formal_store_revision?: number | null;
};

export type RecordMaturePatternDecisionOutput = {
  store_revision: number;
  candidate: MaturePatternCandidate;
  formal_memory_output?: CreateFormalMemoryRecordOutput | null;
  audit_event: MaturePatternAuditEvent;
  acceptance_summary: MemorySystemAcceptanceSummary;
  warnings: string[];
};

export type FormalMemoryStoreV1 = {
  store_version: "formal_memory_store.v1";
  project_id?: string | null;
  workflow_id?: string | null;
  revision: number;
  records: MemoryRecord[];
  versions: MemoryVersion[];
  audit_events: MemoryAuditEvent[];
  updated_at: string;
  warnings: string[];
};

export type CreateFormalMemoryRecordInput = {
  project_root: string;
  project_id?: string | null;
  workflow_id?: string | null;
  scope: MemoryScope;
  memory_type: MemoryRecord["memory_type"];
  claim: string;
  body: string;
  source_refs: MemorySourceRef[];
  actor_id: string;
  actor_role: MemoryAuditEvent["actor_role"];
  reason: string;
  audit_event_type?: MemoryAuditEvent["event_type"] | null;
  expected_store_revision?: number | null;
};

export type CreateFormalMemoryRecordOutput = {
  record: MemoryRecord;
  version: MemoryVersion;
  audit_event: MemoryAuditEvent;
  store_revision: number;
  warnings: string[];
};

export type FormalMemoryLifecycleOperationKind =
  | "revise"
  | "deprecate"
  | "freeze"
  | "unfreeze"
  | "archive"
  | "merge"
  | "split"
  | "promote_to_global"
  | "demote_to_project";

export type FormalMemoryRevisePlan = {
  claim?: string | null;
  body?: string | null;
  source_refs?: MemorySourceRef[] | null;
};

export type FormalMemoryMergePlan = {
  source_memory_ids: string[];
  target_memory_id?: string | null;
  merged_claim: string;
  merged_body: string;
  memory_type?: MemoryRecord["memory_type"] | null;
  scope?: MemoryScope | null;
  source_refs: MemorySourceRef[];
};

export type FormalMemorySplitRecordDraft = {
  claim: string;
  body: string;
  memory_type?: MemoryRecord["memory_type"] | null;
  scope?: MemoryScope | null;
  source_refs: MemorySourceRef[];
};

export type FormalMemorySplitPlan = {
  source_memory_id: string;
  split_records: FormalMemorySplitRecordDraft[];
};

export type FormalMemoryScopeChangePlan = {
  target_scope: MemoryScope;
  applicability: string;
};

export type FormalMemoryLifecyclePreviewInput = {
  project_root: string;
  project_id?: string | null;
  workflow_id?: string | null;
  operation_kind: FormalMemoryLifecycleOperationKind;
  memory_id?: string | null;
  memory_ids: string[];
  revise?: FormalMemoryRevisePlan | null;
  merge?: FormalMemoryMergePlan | null;
  split?: FormalMemorySplitPlan | null;
  scope_change?: FormalMemoryScopeChangePlan | null;
  actor_id: string;
  actor_role: "user" | "project_director" | "global_director" | string;
  reason: string;
  expected_store_revision?: number | null;
  expected_record_versions: Record<string, number>;
};

export type FormalMemoryLifecycleInput = FormalMemoryLifecyclePreviewInput & {
  confirmed_by?: string | null;
  confirmation_summary?: string | null;
};

export type FormalMemoryRequiredApproval = {
  required: boolean;
  approval_kind: "user_confirmation" | "project_director_or_user_confirmation" | string;
  required_actor_role: "user" | "project_director_or_user" | string;
  reason: string;
};

export type FormalMemoryLifecycleStatusChange = {
  memory_id: string;
  before_status: MemoryLifecycleStatus;
  after_status: MemoryLifecycleStatus;
};

export type FormalMemoryLifecycleImpactSummary = {
  affected_memory_ids: string[];
  created_memory_ids: string[];
  status_changes: FormalMemoryLifecycleStatusChange[];
  created_memory_count: number;
  new_version_count: number;
  task_packet_eligibility_change: string;
  source_ref_count: number;
  display_text: string;
  warnings: string[];
};

export type FormalMemoryLifecyclePreview = {
  preview_id: string;
  operation_kind: FormalMemoryLifecycleOperationKind;
  store_revision: number;
  target_memory_ids: string[];
  impact: FormalMemoryLifecycleImpactSummary;
  required_approval: FormalMemoryRequiredApproval;
  before_records: MemoryRecord[];
  proposed_records: MemoryRecord[];
  display_text: string;
  warnings: string[];
};

export type FormalMemoryLifecycleOutput = {
  operation_id: string;
  preview: FormalMemoryLifecyclePreview;
  records: MemoryRecord[];
  versions: MemoryVersion[];
  audit_event: MemoryAuditEvent;
  store_revision: number;
  warnings: string[];
};

export type MemoryCandidateStoreV1 = {
  store_version: "memory_candidate_store.v1";
  project_id?: string | null;
  workflow_id?: string | null;
  revision: number;
  candidates: MemoryCandidate[];
  events: MemoryAuditRef[];
  updated_at: string;
};

export type CreateMemoryCandidateInput = {
  project_root: string;
  project_id?: string | null;
  workflow_id?: string | null;
  scope: MemoryScope;
  memory_type: MemoryCandidate["memory_type"];
  claim: string;
  body: string;
  source_refs: MemorySourceRef[];
  generated_by_role: string;
  generated_from: MemoryCandidate["generated_from"];
  risk_level: MemoryCandidate["risk_level"];
  sensitive_level: MemoryCandidate["sensitive_level"];
  requires_user_confirmation: boolean;
  review_reason: string;
  expected_store_revision?: number | null;
};

export type CreateMemoryCandidateOutput = {
  candidate: MemoryCandidate;
  audit_event: MemoryAuditRef;
  store_revision: number;
  warnings: string[];
};

export type RecordMemoryCandidateDecisionInput = {
  project_root: string;
  candidate_key: string;
  requested_status: MemoryLifecycleStatus;
  reason: string;
  actor_id: string;
  actor_role: string;
  expected_store_revision?: number | null;
};

export type RecordMemoryCandidateDecisionOutput = {
  candidate: MemoryCandidate;
  audit_event: MemoryAuditRef;
  store_revision: number;
  warnings: string[];
};

export type AdoptMemoryCandidateInput = {
  project_root: string;
  candidate_key: string;
  actor_id: string;
  actor_role: string;
  adoption_reason: string;
  expected_candidate_store_revision?: number | null;
  expected_formal_store_revision?: number | null;
};

export type AdoptMemoryCandidateOutput = {
  candidate_key: string;
  candidate_status: MemoryLifecycleStatus;
  record: MemoryRecord;
  version: MemoryVersion;
  audit_event: MemoryAuditEvent;
  adoption: MemoryCandidateAdoptionRef;
  candidate_store_revision: number;
  formal_store_revision: number;
  warnings: string[];
};

export type ObservationStatus = "recorded" | "candidate_created" | "ignored" | "quarantined";

export type ObservationType =
  | "worker_report"
  | "process_fact"
  | "project_director_confirmation"
  | "global_director_review"
  | "plan_adopted"
  | "result_acceptance";

export type ObservationSourceRef = {
  source_ref_id: string;
  source_kind:
    | "workflow_event"
    | "worker_report"
    | "director_review"
    | "task_package"
    | "evidence"
    | "handoff"
    | "user_confirmation";
  source_id: string;
  project_id?: string | null;
  workflow_id?: string | null;
  session_id?: string | null;
  file_path?: string | null;
  evidence_ref?: string | null;
  summary: string;
  sensitive_level: "public" | "internal" | "sensitive" | "secret";
  created_at: string;
};

export type ObservationAuditRef = {
  audit_ref_id: string;
  event_type:
    | "observation_recorded"
    | "observation_ignored"
    | "observation_quarantined"
    | "observation_candidate_created";
  actor_id: string;
  actor_role: "worker" | "project_director" | "global_director" | "user" | "system" | string;
  target_kind: "observation";
  target_id: string;
  before_status?: ObservationStatus | null;
  after_status?: ObservationStatus | null;
  reason: string;
  created_at: string;
};

export type ObservationRecord = {
  observation_id: string;
  observation_key: string;
  schema_version: "memory_observation.v1";
  project_id?: string | null;
  workflow_id?: string | null;
  scope: MemoryScope;
  observation_type: ObservationType;
  summary: string;
  source_refs: ObservationSourceRef[];
  status: ObservationStatus;
  generated_by_role: "worker" | "project_director" | "global_director" | "user" | "system" | string;
  actor_id: string;
  risk_level: "low" | "medium" | "high";
  sensitive_level: "public" | "internal" | "sensitive" | "secret";
  candidate_key?: string | null;
  audit_refs: ObservationAuditRef[];
  created_at: string;
  updated_at: string;
};

export type ObservationStoreV1 = {
  store_version: "observation_store.v1";
  project_id?: string | null;
  workflow_id?: string | null;
  revision: number;
  observations: ObservationRecord[];
  events: ObservationAuditRef[];
  updated_at: string;
  warnings: string[];
};

export type MemoryCaptureSourceType =
  | "user_action"
  | "product_command"
  | "runtime_log"
  | "readback"
  | "worker_report"
  | "operation_control_decision"
  | "process_fact_decision"
  | "final_review";

export type MemoryCaptureSensitivity = "public" | "internal" | "project_confidential" | "secret";

export type MemoryCaptureCandidatePolicy =
  | "observation_only"
  | "candidate_allowed"
  | "audit_only"
  | "blocked_sensitive";

export type MemoryCaptureSourceRef = {
  source_ref_id: string;
  source_type: MemoryCaptureSourceType;
  source_id: string;
  project_id?: string | null;
  workflow_id?: string | null;
  workflow_node_id?: string | null;
  run_unit_id?: string | null;
  product_command_id?: string | null;
  product_attempt_id?: string | null;
  runtime_log_ref?: string | null;
  audit_ref_id?: string | null;
  readback_ref?: string | null;
  task_package_ref?: string | null;
  memory_packet_ref?: string | null;
  evidence_ref?: string | null;
  summary: string;
  sensitive_level: MemoryCaptureSensitivity;
  created_at: string;
};

export type MemoryCaptureCandidateDraft = {
  memory_type: MemoryCandidate["memory_type"];
  claim: string;
  body: string;
  review_reason: string;
  requires_user_confirmation: boolean;
  actor_role: "project_director";
};

export type CaptureMemoryEventInput = {
  project_root: string;
  project_id?: string | null;
  workflow_id?: string | null;
  workflow_node_id?: string | null;
  run_unit_id?: string | null;
  product_command_id?: string | null;
  product_attempt_id?: string | null;
  runtime_log_ref?: string | null;
  audit_refs: string[];
  readback_ref?: string | null;
  task_package_ref?: string | null;
  memory_packet_ref?: string | null;
  scope: MemoryScope;
  source_type: MemoryCaptureSourceType;
  source_refs: MemoryCaptureSourceRef[];
  summary: string;
  evidence_summary: string;
  sensitivity: MemoryCaptureSensitivity;
  candidate_policy: MemoryCaptureCandidatePolicy;
  generated_by_role: "worker" | "project_director" | "global_director" | "user" | "system";
  actor_id: string;
  risk_level: ObservationRecord["risk_level"];
  reason: string;
  candidate?: MemoryCaptureCandidateDraft | null;
  expected_capture_store_revision?: number | null;
  expected_observation_store_revision?: number | null;
  expected_candidate_store_revision?: number | null;
};

export type MemoryCaptureEventRecord = {
  capture_event_id: string;
  event_key: string;
  schema_version: "memory_capture_event.v1";
  source_type: MemoryCaptureSourceType;
  source_ref_id: string;
  project_id?: string | null;
  workflow_id?: string | null;
  workflow_node_id?: string | null;
  run_unit_id?: string | null;
  product_command_id?: string | null;
  product_attempt_id?: string | null;
  runtime_log_ref?: string | null;
  audit_refs: string[];
  readback_ref?: string | null;
  task_package_ref?: string | null;
  memory_packet_ref?: string | null;
  summary: string;
  evidence_summary: string;
  sensitivity: MemoryCaptureSensitivity;
  candidate_policy: MemoryCaptureCandidatePolicy;
  blocked_reason?: string | null;
  observation_id?: string | null;
  candidate_key?: string | null;
  created_by: string;
  created_at: string;
  updated_at: string;
};

export type MemoryCaptureStoreV1 = {
  store_version: "memory_capture_store.v1";
  project_id?: string | null;
  workflow_id?: string | null;
  revision: number;
  events: MemoryCaptureEventRecord[];
  updated_at: string;
  warnings: string[];
};

export type CaptureMemoryEventOutput = {
  capture_event: MemoryCaptureEventRecord;
  observation?: ObservationRecord | null;
  candidate?: MemoryCandidate | null;
  observation_store_revision?: number | null;
  candidate_store_revision?: number | null;
  capture_store_revision: number;
  warnings: string[];
};

export type CreateObservationInput = {
  project_root: string;
  project_id?: string | null;
  workflow_id?: string | null;
  scope: MemoryScope;
  observation_type: ObservationType;
  summary: string;
  source_refs: ObservationSourceRef[];
  generated_by_role: "worker" | "project_director" | "global_director" | "user" | "system";
  actor_id: string;
  risk_level: ObservationRecord["risk_level"];
  sensitive_level: ObservationRecord["sensitive_level"];
  reason: string;
  expected_store_revision?: number | null;
};

export type CreateObservationOutput = {
  observation: ObservationRecord;
  audit_event: ObservationAuditRef;
  store_revision: number;
  warnings: string[];
};

export type CreateMemoryCandidateFromObservationInput = {
  project_root: string;
  observation_key: string;
  actor_id: string;
  actor_role: "project_director" | "global_director" | "user";
  memory_type: MemoryCandidate["memory_type"];
  claim: string;
  body: string;
  review_reason: string;
  requires_user_confirmation: boolean;
  expected_observation_store_revision?: number | null;
  expected_candidate_store_revision?: number | null;
};

export type CreateMemoryCandidateFromObservationOutput = {
  observation: ObservationRecord;
  candidate: MemoryCandidate;
  observation_audit_event: ObservationAuditRef;
  candidate_audit_event: MemoryAuditRef;
  observation_store_revision: number;
  candidate_store_revision: number;
  warnings: string[];
};

export type ObservationStoreSummary = {
  sidecar_name: "observations.v1.json";
  revision: number;
  observation_count: number;
  recorded_count: number;
  candidate_created_count: number;
  ignored_count: number;
  quarantined_count: number;
  recent_audit_event?: ObservationAuditRef | null;
  recent_candidate_key?: string | null;
  display_text: string;
  warnings: string[];
};

export type MemoryLintFindingSeverity = "blocking" | "needs_review" | "info";

export type MemoryLintFindingStatus = "open" | "acknowledged" | "resolved" | "dismissed";

export type MemoryLintFindingType =
  | "duplicate_claim"
  | "claim_conflict"
  | "source_permission_revoked"
  | "authority_superseded"
  | "stale_memory"
  | "missing_source"
  | "candidate_conflicts_with_active_memory"
  | "entity_drift"
  | "relation_source_revoked"
  | "sensitive_export_risk"
  | "private_source_risk"
  | "derived_index_stale"
  | "mature_pattern_signal";

export type MemoryLintRunIntent = "candidate_adoption_guard" | "task_packet_guard" | "maintenance_preview" | "maintenance_run";

export type MemoryLintRunStatus = "succeeded" | "blocked" | "failed";

export type MemoryLintFinding = {
  finding_id: string;
  schema_version: "memory_governance.v1" | string;
  finding_type: MemoryLintFindingType;
  severity: MemoryLintFindingSeverity;
  status: MemoryLintFindingStatus;
  source_kind: "memory_record" | "memory_candidate" | "lint_run" | string;
  source_id: string;
  target_memory_id?: string | null;
  target_candidate_key?: string | null;
  scope_type?: string | null;
  memory_type?: string | null;
  claim?: string | null;
  summary: string;
  recommended_action:
    | "block_adoption"
    | "exclude_from_task_packet"
    | "review_and_deprecate"
    | "review_source_permission"
    | "review_staleness"
    | "no_action"
    | string;
  evidence_refs: MemorySourceRef[];
  audit_event_id?: string | null;
  created_at: string;
  updated_at: string;
};

export type MemoryLintRunRecord = {
  run_id: string;
  lint_intent: MemoryLintRunIntent;
  actor_id: string;
  actor_role: "project_director" | "global_director" | "system" | string;
  finding_ids: string[];
  blocking_count: number;
  status: MemoryLintRunStatus;
  reason: string;
  report_id?: string | null;
  created_at: string;
};

export type MemoryMaintenanceCheckKind =
  | "expired_or_stale"
  | "source_integrity"
  | "duplicate_and_conflict"
  | "entity_relation_drift"
  | "permission_revocation"
  | "sensitive_export_risk"
  | "index_status"
  | "mature_pattern_signal";

export type MemoryMaintenanceCheckSummary = {
  check_kind: MemoryMaintenanceCheckKind;
  checked_count: number;
  finding_count: number;
  blocking_count: number;
  needs_review_count: number;
  info_count: number;
  display_text: string;
};

export type MemoryMaintenanceRecommendation = {
  recommendation_id: string;
  severity: MemoryLintFindingSeverity;
  target_kind: string;
  target_id?: string | null;
  action_label: string;
  display_text: string;
};

export type MemoryMaintenanceIndexStatus = {
  status: string;
  formal_store_revision: number;
  lint_store_revision: number;
  entity_relation_store_revision: number;
  checked_at: string;
  display_text: string;
  warnings: string[];
};

export type MemoryMaintenanceReport = {
  report_id: string;
  run_id: string;
  checked_memory_count: number;
  checked_candidate_count: number;
  checked_observation_count: number;
  checked_relation_count: number;
  open_count: number;
  blocking_count: number;
  needs_review_count: number;
  info_count: number;
  check_summaries: MemoryMaintenanceCheckSummary[];
  recommendations: MemoryMaintenanceRecommendation[];
  index_status: MemoryMaintenanceIndexStatus;
  display_text: string;
  warnings: string[];
  created_at: string;
};

export type MemoryLintStoreV1 = {
  store_version: "memory_lint_store.v1";
  project_id?: string | null;
  workflow_id?: string | null;
  revision: number;
  findings: MemoryLintFinding[];
  runs: MemoryLintRunRecord[];
  maintenance_reports?: MemoryMaintenanceReport[];
  updated_at: string;
  warnings: string[];
};

export type MemoryLintRunInput = {
  project_root: string;
  project_id?: string | null;
  workflow_id?: string | null;
  actor_id: string;
  actor_role: "project_director" | "global_director" | "system";
  lint_intent: MemoryLintRunIntent;
  candidate_key?: string | null;
  task_id?: string | null;
  revoked_source_ids: string[];
  expected_formal_store_revision?: number | null;
  expected_candidate_store_revision?: number | null;
  expected_lint_store_revision?: number | null;
  dry_run?: boolean | null;
};

export type MemoryLintRunOutput = {
  store: MemoryLintStoreV1;
  run: MemoryLintRunRecord;
  report?: MemoryMaintenanceReport | null;
  new_findings: MemoryLintFinding[];
  blocking_count: number;
  open_count: number;
  warnings: string[];
};

export type TaskMemoryPacketExclusionReason =
  | "candidate_unconfirmed"
  | "permission_blocked"
  | "conflicted"
  | "stale"
  | "model_export_blocked"
  | "token_limit"
  | "not_relevant"
  | "status_not_active"
  | "observation_not_formal_memory"
  | "knowledge_hit_not_formal_memory"
  | "llm_summary_not_formal_memory";

export type TaskMemoryPacketBuildInput = {
  project_root: string;
  project_id?: string | null;
  workflow_id?: string | null;
  task_id?: string | null;
  role_id: string;
  task_goal: string;
  retrieval_intent:
    | "worker_task"
    | "project_director_review"
    | "global_director_review"
    | "result_acceptance";
  target_model_id?: string | null;
  model_context_policy: "local_only" | "external_model_context";
  max_memory_items: number;
  max_estimated_tokens: number;
  expected_formal_store_revision?: number | null;
  expected_candidate_store_revision?: number | null;
  expected_observation_store_revision?: number | null;
};

export type TaskMemoryPacketItem = {
  memory_id: string;
  memory_type: string;
  scope_type: string;
  claim: string;
  body: string;
  source_refs: MemorySourceRef[];
  retrieval_reason: string;
  relation_explanations?: MemoryRelationTaskExplanation[];
  estimated_tokens: number;
  model_export_policy: string;
};

export type TaskMemoryPacketExcludedItem = {
  source_kind: "memory_record" | "memory_candidate" | "observation" | "knowledge_hit" | "llm_summary" | string;
  source_id: string;
  claim?: string | null;
  reason: TaskMemoryPacketExclusionReason;
  detail: string;
};

export type TaskMemoryPacketReviewMaterial = {
  source_kind: "memory_candidate" | "observation" | "knowledge_hit" | string;
  source_id: string;
  title: string;
  reason: TaskMemoryPacketExclusionReason;
};

export type TaskMemoryPacketPreview = {
  packet_id: string;
  schema_version: "task_memory_packet.v1" | string;
  project_id?: string | null;
  workflow_id?: string | null;
  task_id?: string | null;
  role_id: string;
  retrieval_intent: string;
  included_memories: TaskMemoryPacketItem[];
  excluded_items: TaskMemoryPacketExcludedItem[];
  review_materials: TaskMemoryPacketReviewMaterial[];
  estimated_tokens: number;
  max_estimated_tokens: number;
  generated_at: string;
  warnings: string[];
};

export type TaskMemoryPacketBuildOutput = {
  preview: TaskMemoryPacketPreview;
  formal_store_revision: number;
  candidate_store_revision: number;
  observation_store_revision: number;
  lint_store_revision?: number | null;
  entity_relation_store_revision?: number | null;
  warnings: string[];
};

export type TaskPackageMemoryPacketStoreRevisions = {
  formal_store_revision: number;
  candidate_store_revision: number;
  observation_store_revision: number;
  lint_store_revision?: number | null;
  entity_relation_store_revision?: number | null;
};

export type TaskPackageMemoryPacketSnapshot = {
  snapshot_id: string;
  schema_version: "task_package_memory_packet_snapshot.v1" | string;
  source_packet_id: string;
  project_id?: string | null;
  workflow_id?: string | null;
  work_item_id: string;
  task_package_artifact_id?: string | null;
  role_id: string;
  retrieval_intent: "worker_task" | string;
  included_memories: TaskMemoryPacketItem[];
  excluded_items: TaskMemoryPacketExcludedItem[];
  review_materials: TaskMemoryPacketReviewMaterial[];
  store_revisions: TaskPackageMemoryPacketStoreRevisions;
  estimated_tokens: number;
  max_estimated_tokens: number;
  fingerprint: string;
  generated_at: string;
  stale: boolean;
  stale_reasons: string[];
  warnings: string[];
};

export type TaskPackageMemoryInjectionSummary = {
  snapshot_id?: string | null;
  included_count: number;
  excluded_count: number;
  review_material_count: number;
  stale: boolean;
  stale_reasons: string[];
  display_text: string;
  warnings: string[];
};
