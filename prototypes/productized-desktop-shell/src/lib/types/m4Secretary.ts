// M4C06 renderer contract for the server-owned Secretary application service.
//
// These values deliberately carry only typed / opaque references, codes and
// hashes.  They are not a renderer-owned source model and never contain a
// provider body, prompt, response, path, URL, credential or command payload.

export type M4SecretaryTypedRef = string;
export type M4SecretaryOpaqueRef = string;
export type M4SecretaryHash = string;
export type M4SecretaryCanonicalRevision = string;

export type M4SecretaryContextDto = Readonly<{
  context_ref: M4SecretaryOpaqueRef;
  role_session_ref: M4SecretaryOpaqueRef;
  scope_ref: M4SecretaryTypedRef;
  scope_source_watermark: M4SecretaryHash;
  snapshot_hash: M4SecretaryHash;
  reconstruction_code: string;
}>;

export type M4SecretarySourceBackedBriefItemDto = Readonly<{
  item_ref: M4SecretaryTypedRef;
  item_kind_code: string;
  source_owner_ref: M4SecretaryTypedRef;
  source_object_ref: M4SecretaryTypedRef;
  source_object_type: string;
  source_route_ref: M4SecretaryOpaqueRef;
  source_summary_ref: M4SecretaryOpaqueRef;
  why_code: string;
  priority_rank: number;
  priority_code: string;
  status_code: string;
  source_status_code: string;
  due_at_utc: string | null;
  last_change_at_utc: string;
  change_hash: M4SecretaryHash;
  coordination_revision: M4SecretaryCanonicalRevision;
}>;

// Personal actions are an explicitly-created aggregate.  They are never
// cloned from an InboxItem or OpenLoop by this renderer projection.
export type M4SecretaryPersonalActionBriefItemDto = Readonly<{
  personal_action_ref: M4SecretaryTypedRef;
  explicit_user_command_ref: M4SecretaryOpaqueRef;
  status_code: string;
  due_at_utc: string | null;
  revision_hash: M4SecretaryHash;
  coordination_revision: M4SecretaryCanonicalRevision;
}>;

export type M4SecretaryDeterministicBriefDto = Readonly<{
  brief_ref: M4SecretaryOpaqueRef;
  brief_hash: M4SecretaryHash;
  context_ref: M4SecretaryOpaqueRef;
  scope_source_watermark: M4SecretaryHash;
  attention_items: readonly M4SecretarySourceBackedBriefItemDto[];
  personal_actions: readonly M4SecretaryPersonalActionBriefItemDto[];
}>;

export type M4SecretaryInvocationReceiptDto = Readonly<{
  invocation_ref: M4SecretaryOpaqueRef;
  terminal_receipt_ref: M4SecretaryOpaqueRef;
  outcome_code: string;
  result_ref: M4SecretaryOpaqueRef | null;
  result_hash: M4SecretaryHash | null;
  error_code: string | null;
}>;

export type M4SecretaryModelEnhancementStatus = "AVAILABLE" | "FAILED" | "PENDING" | "REPLAYED" | "UNAVAILABLE";

export type M4SecretaryModelEnhancementOutcomeDto = Readonly<{
  status: M4SecretaryModelEnhancementStatus;
  invocation_ref: M4SecretaryOpaqueRef | null;
  enhancement_ref: M4SecretaryOpaqueRef | null;
  enhancement_hash: M4SecretaryHash | null;
  invocation_receipt: M4SecretaryInvocationReceiptDto | null;
  recovery_code: string | null;
}>;

// Renderer-only local coordination objects. These values are read from the
// same server-owned M4 snapshot as the deterministic brief, but are kept out
// of that model-facing brief. In particular, a PersonalAction title is shown
// only through this local DTO.
export type M4SecretarySourceLinkDto = Readonly<{
  link_kind: string;
  source_owner_ref: M4SecretaryTypedRef;
  object_type: string;
  canonical_source_object_id: M4SecretaryTypedRef;
  expected_source_revision: M4SecretaryCanonicalRevision;
  opaque_route_ref: M4SecretaryOpaqueRef;
}>;

export type M4SecretaryPersonalActionLocalDto = Readonly<{
  personal_action_id: M4SecretaryTypedRef;
  explicit_user_command_ref: M4SecretaryOpaqueRef;
  title: string;
  status: "OPEN" | "COMPLETED" | "CANCELLED";
  due_at_utc: string | null;
  revision: M4SecretaryCanonicalRevision;
}>;

export type M4SecretaryNotificationLocalDto = Readonly<{
  notification_id: M4SecretaryTypedRef;
  source_ref: M4SecretarySourceLinkDto;
  subject_ref: M4SecretaryTypedRef;
  notification_purpose_code: string;
  delivery_channel: "IN_APP";
  status: "PENDING" | "DELIVERED" | "READ" | "DISMISSED";
  created_at_utc: string;
  delivered_at_utc: string | null;
  read_at_utc: string | null;
  dismissed_at_utc: string | null;
  revision: M4SecretaryCanonicalRevision;
}>;

export type M4SecretaryReminderLocalDto = Readonly<{
  reminder_id: M4SecretaryTypedRef;
  owner_ref: M4SecretaryTypedRef;
  explicit_schedule_command_id: M4SecretaryOpaqueRef;
  scheduled_for_utc: string;
  iana_timezone: string;
  status: "SCHEDULED" | "FIRED" | "SNOOZED" | "DISMISSED" | "CANCELLED";
  last_fired_at_utc: string | null;
  snoozed_until_utc: string | null;
  revision: M4SecretaryCanonicalRevision;
}>;

export type M4SecretaryDecisionLocalDto = Readonly<{
  decision_projection_id: M4SecretaryTypedRef;
  source_identity_key: M4SecretaryOpaqueRef;
  source_event_key: M4SecretaryOpaqueRef;
  source_ref: M4SecretaryTypedRef;
  owner_status: "OPEN" | "ANSWERED" | "EXPIRED" | "WITHDRAWN";
  local_visibility_status: "UNREAD" | "READ" | "DISMISSED";
  decision_by_utc: string | null;
  source_revision: M4SecretaryCanonicalRevision;
  revision: M4SecretaryCanonicalRevision;
}>;

export type M4SecretaryLocalObjectsDto = Readonly<{
  personal_actions: readonly M4SecretaryPersonalActionLocalDto[];
  notifications: readonly M4SecretaryNotificationLocalDto[];
  reminders: readonly M4SecretaryReminderLocalDto[];
  decisions: readonly M4SecretaryDecisionLocalDto[];
  reminder_owner_refs: readonly M4SecretaryTypedRef[];
}>;

export type M4SecretaryApplicationOutcomeDto = Readonly<{
  context: M4SecretaryContextDto;
  deterministic_brief: M4SecretaryDeterministicBriefDto;
  local_objects: M4SecretaryLocalObjectsDto;
  model_enhancement: M4SecretaryModelEnhancementOutcomeDto | null;
}>;

export type M4SecretaryHomeContextReadyEnvelopeDto = Readonly<{
  status: "READY";
  application_outcome: M4SecretaryApplicationOutcomeDto;
  reason: null;
}>;

export type M4SecretaryHomeContextUnavailableEnvelopeDto = Readonly<{
  status: "UNAVAILABLE";
  application_outcome: null;
  reason: string;
}>;

export type M4SecretaryHomeContextEnvelopeDto =
  | M4SecretaryHomeContextReadyEnvelopeDto
  | M4SecretaryHomeContextUnavailableEnvelopeDto;

export type M4SecretaryHandoffStatus = "UNAVAILABLE" | "PENDING" | "RETURNED" | "FAILED";

export type M4SecretaryHandoffReceiptDto = Readonly<{
  receipt_ref: M4SecretaryOpaqueRef;
  handoff_ref: M4SecretaryOpaqueRef;
  receipt_kind_code: string;
  status_code: string;
  result_ref: M4SecretaryOpaqueRef | null;
  result_hash: M4SecretaryHash | null;
}>;

export type M4SecretaryHandoffOutcomeDto = Readonly<{
  status: M4SecretaryHandoffStatus;
  handoff_ref: M4SecretaryOpaqueRef | null;
  request_receipt_ref: M4SecretaryOpaqueRef | null;
  returned_receipt: M4SecretaryHandoffReceiptDto | null;
  recovery_code: string | null;
}>;

// Exact `operate_secretary_coordination` command boundary.  The request is
// constrained to the C04 transitions actually registered by the server; it
// has no source-owner, PersonalAction, callback, title/body, path, or provider
// fields.  Snooze is the sole action that carries a timestamp.
export type M4SecretaryCoordinationActionCode =
  | "INBOX_MARK_READ"
  | "INBOX_DISMISS"
  | "OPEN_LOOP_ACKNOWLEDGE"
  | "OPEN_LOOP_SNOOZE"
  | "OPEN_LOOP_CLOSE"
  | "OPEN_LOOP_DISMISS"
  | "OPEN_LOOP_REOPEN"
  | "OPEN_LOOP_CARRY_OVER";

export type M4SecretaryCoordinationActionRequestDto = Readonly<{
  action: M4SecretaryCoordinationActionCode;
  item_ref: M4SecretaryTypedRef;
  expected_revision: M4SecretaryCanonicalRevision;
  idempotency_key: M4SecretaryOpaqueRef;
  snoozed_until_utc?: string | null;
}>;

// Only repository-minted refs, status codes, a canonical revision and replay
// state cross this boundary.  Source-owner facts and execution bodies do not.
export type M4SecretaryCoordinationActionReceiptDto = Readonly<{
  command_receipt_ref: M4SecretaryOpaqueRef;
  coordination_event_ref: M4SecretaryOpaqueRef;
  aggregate_kind_code: string;
  item_ref: M4SecretaryTypedRef;
  coordination_revision: M4SecretaryCanonicalRevision;
  outcome_code: string;
  replayed: boolean;
}>;

export type M4SecretaryPersonalObjectActionCode =
  | "PERSONAL_ACTION_CREATE"
  | "PERSONAL_ACTION_COMPLETE"
  | "PERSONAL_ACTION_CANCEL"
  | "PERSONAL_ACTION_REOPEN"
  | "REMINDER_CREATE"
  | "REMINDER_SNOOZE"
  | "REMINDER_DISMISS"
  | "REMINDER_CANCEL"
  | "NOTIFICATION_READ"
  | "NOTIFICATION_DISMISS"
  | "DECISION_READ"
  | "DECISION_DISMISS";

export type M4SecretaryPersonalObjectRequestDto =
  | Readonly<{
      action: "PERSONAL_ACTION_CREATE";
      title: string;
      due_at_utc?: string | null;
      idempotency_key: M4SecretaryOpaqueRef;
    }>
  | Readonly<{
      action: "PERSONAL_ACTION_COMPLETE" | "PERSONAL_ACTION_CANCEL" | "PERSONAL_ACTION_REOPEN";
      item_ref: M4SecretaryTypedRef;
      expected_revision: M4SecretaryCanonicalRevision;
      idempotency_key: M4SecretaryOpaqueRef;
    }>
  | Readonly<{
      action: "REMINDER_CREATE";
      owner_ref: M4SecretaryTypedRef;
      scheduled_for_utc: string;
      iana_timezone: string;
      idempotency_key: M4SecretaryOpaqueRef;
    }>
  | Readonly<{
      action: "REMINDER_SNOOZE";
      item_ref: M4SecretaryTypedRef;
      expected_revision: M4SecretaryCanonicalRevision;
      snoozed_until_utc: string;
      idempotency_key: M4SecretaryOpaqueRef;
    }>
  | Readonly<{
      action: "REMINDER_DISMISS" | "REMINDER_CANCEL";
      item_ref: M4SecretaryTypedRef;
      expected_revision: M4SecretaryCanonicalRevision;
      idempotency_key: M4SecretaryOpaqueRef;
    }>
  | Readonly<{
      action: "NOTIFICATION_READ" | "NOTIFICATION_DISMISS";
      item_ref: M4SecretaryTypedRef;
      expected_revision: M4SecretaryCanonicalRevision;
      idempotency_key: M4SecretaryOpaqueRef;
    }>
  | Readonly<{
      action: "DECISION_READ" | "DECISION_DISMISS";
      item_ref: M4SecretaryTypedRef;
      expected_revision: M4SecretaryCanonicalRevision;
      idempotency_key: M4SecretaryOpaqueRef;
    }>;

export type SecretaryHomeLoadState = "loading" | "ready" | "empty" | "degraded" | "error";

export type SecretaryHomeSourceAuthority = "M4_APPLICATION_SERVICE" | "CANONICAL_SNAPSHOT_SUMMARY" | "NONE";

export type SecretaryHomeSourceOwner = Readonly<{
  availability: "AVAILABLE" | "NOT_EMITTED_BY_SUMMARY";
  source_owner_ref: M4SecretaryTypedRef | null;
}>;

// M4R04 source-return contract. The renderer submits only the server-minted
// route capability. Owner/type/id/revision and the finite target all come back
// from the ordinary server resolver; none is selected or reconstructed here.
export const M4_SECRETARY_SOURCE_ROUTE_RESOLUTION_SCHEMA =
  "syn.m4.secretary.source-route-resolution.v1" as const;

export const M4_WORK_ITEM_SOURCE_OWNER_REF =
  "owner:m2-workflow-state-work-item:v1" as const;
export const M4_PROPOSAL_SOURCE_OWNER_REF =
  "owner:project-consultation-proposal:v1" as const;

export type M4SecretarySourceRouteRequestDto = Readonly<{
  source_route_ref: M4SecretaryOpaqueRef;
}>;

export type M4SecretarySourceNavigationTarget =
  | Readonly<{
      kind: "WORK_ITEM";
      project_id: M4SecretaryTypedRef;
      workflow_id: M4SecretaryTypedRef;
      work_item_id: M4SecretaryTypedRef;
      source_revision: M4SecretaryCanonicalRevision;
    }>
  | Readonly<{
      kind: "CONSULTATION_PROPOSAL";
      project_id: M4SecretaryTypedRef;
      workflow_id: M4SecretaryTypedRef;
      proposal_id: M4SecretaryTypedRef;
      source_revision: M4SecretaryCanonicalRevision;
    }>;

export type M4SecretarySourceRouteResolutionDto = Readonly<{
  schema_version: typeof M4_SECRETARY_SOURCE_ROUTE_RESOLUTION_SCHEMA;
  source_owner_ref: M4SecretaryTypedRef;
  source_object_type: "workflow_attention" | "proposal_decision";
  canonical_source_object_id: M4SecretaryTypedRef;
  source_revision: M4SecretaryCanonicalRevision;
  source_route_ref: M4SecretaryOpaqueRef;
  target: M4SecretarySourceNavigationTarget;
}>;

// This is deliberately separate from Workbench NavigationFocus's open-ended
// `{ kind: string, id: string }`. Only a parsed server resolution may create it.
export type SecretarySourceFocus = Readonly<{
  attempt_id: number;
  source_owner_ref: M4SecretarySourceRouteResolutionDto["source_owner_ref"];
  source_object_type: M4SecretarySourceRouteResolutionDto["source_object_type"];
  canonical_source_object_id: M4SecretarySourceRouteResolutionDto["canonical_source_object_id"];
  source_revision: M4SecretarySourceRouteResolutionDto["source_revision"];
  source_route_ref: M4SecretarySourceRouteResolutionDto["source_route_ref"];
  target: M4SecretarySourceNavigationTarget;
}>;

export type SecretarySourceFocusOutcome = Readonly<{
  attempt_id: number;
  source_route_ref: M4SecretaryOpaqueRef;
  target_kind: M4SecretarySourceNavigationTarget["kind"];
  status: "CONSUMED" | "FAILED";
  error_code:
    | null
    | "SECRETARY_SOURCE_TARGET_PROJECT_MISSING"
    | "SECRETARY_SOURCE_TARGET_AMBIGUOUS"
    | "SECRETARY_SOURCE_TARGET_RECORD_MISSING";
}>;

export type SecretarySourceRouteViewState = Readonly<{
  source_route_ref: M4SecretaryOpaqueRef | null;
  phase: "IDLE" | "RESOLVING" | "CONSUMING" | "CONSUMED" | "FAILED";
  error_code: string | null;
}>;

export type SecretaryTypedDeepLinkDescriptor =
  | Readonly<{
      kind: "M4_SOURCE_ROUTE";
      source_owner_ref: M4SecretaryTypedRef;
      source_object_ref: M4SecretaryTypedRef;
      source_object_type: string;
      source_route_ref: M4SecretaryOpaqueRef;
      executable_payload: null;
    }>
  | Readonly<{
      kind: "CANONICAL_SNAPSHOT_SUMMARY_ROUTE";
      source_kind_code: string;
      summary_route_ref: M4SecretaryOpaqueRef;
      executable_payload: null;
    }>;

export type SecretaryHomeAttentionItem = Readonly<{
  item_ref: M4SecretaryTypedRef;
  item_kind_code: string;
  source_authority: "M4_COORDINATION" | "CANONICAL_SNAPSHOT_SUMMARY";
  source_owner: SecretaryHomeSourceOwner;
  source_object_ref: M4SecretaryTypedRef;
  source_object_type: string;
  deep_link: SecretaryTypedDeepLinkDescriptor;
  why_code: string;
  priority_rank: number;
  priority_reason_code: string;
  status_code: string;
  source_status_code: string;
  last_change_at_utc: string | null;
  due_at_utc: string | null;
  change_hash: M4SecretaryHash;
  coordination_revision: M4SecretaryCanonicalRevision | null;
}>;

export type SecretaryHomePersonalAction = Readonly<{
  personal_action_ref: M4SecretaryTypedRef;
  explicit_user_command_ref: M4SecretaryOpaqueRef;
  status_code: string;
  due_at_utc: string | null;
  revision_hash: M4SecretaryHash;
  coordination_revision: M4SecretaryCanonicalRevision | null;
  source_authority: "M4_COORDINATION";
}>;

// Decision requests and pending coordination work expose only the owner route
// needed to open the specialised module.  They never persist a command body.
export type SecretaryProfessionalModuleEntry = Readonly<{
  entry_ref: M4SecretaryOpaqueRef;
  entry_kind: "SOURCE_OWNER_ROUTE";
  source_owner: SecretaryHomeSourceOwner;
  deep_link: SecretaryTypedDeepLinkDescriptor;
  action_payload: null;
}>;

export type SecretaryHomeRoleSessionRecovery = Readonly<{
  status: "LOADING" | "RESTORED" | "UNAVAILABLE";
  role_session_ref: M4SecretaryOpaqueRef | null;
  context_ref: M4SecretaryOpaqueRef | null;
  recovery_code: string | null;
}>;

export type SecretaryHomeModelEnhancement =
  | M4SecretaryModelEnhancementOutcomeDto
  | Readonly<{
      status: "NOT_REQUESTED";
      invocation_ref: null;
      enhancement_ref: null;
      enhancement_hash: null;
      invocation_receipt: null;
      recovery_code: null;
    }>;

export type SecretaryHomeHandoff = M4SecretaryHandoffOutcomeDto | Readonly<{
  status: "NOT_LOADED";
  handoff_ref: null;
  request_receipt_ref: null;
  returned_receipt: null;
  recovery_code: null;
}>;

export type SecretaryHomeReadModel = Readonly<{
  schema_version: "syn.m4.secretary.home.v1";
  state: SecretaryHomeLoadState;
  source_authority: SecretaryHomeSourceAuthority;
  context: M4SecretaryContextDto | null;
  deterministic_brief: Readonly<{
    brief_ref: M4SecretaryOpaqueRef;
    brief_hash: M4SecretaryHash;
    context_ref: M4SecretaryOpaqueRef;
    scope_source_watermark: M4SecretaryHash;
  }> | null;
  scope_source_watermark: M4SecretaryHash | null;
  role_session_recovery: SecretaryHomeRoleSessionRecovery;
  attention_items: readonly SecretaryHomeAttentionItem[];
  personal_actions: readonly SecretaryHomePersonalAction[];
  local_objects: M4SecretaryLocalObjectsDto;
  module_entries: readonly SecretaryProfessionalModuleEntry[];
  model_enhancement: SecretaryHomeModelEnhancement;
  handoff: SecretaryHomeHandoff;
  degradation_code: string | null;
}>;
