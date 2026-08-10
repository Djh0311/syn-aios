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

export type M4SecretaryApplicationOutcomeDto = Readonly<{
  context: M4SecretaryContextDto;
  deterministic_brief: M4SecretaryDeterministicBriefDto;
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

export type SecretaryHomeLoadState = "loading" | "ready" | "empty" | "degraded" | "error";

export type SecretaryHomeSourceAuthority = "M4_APPLICATION_SERVICE" | "CANONICAL_SNAPSHOT_SUMMARY" | "NONE";

export type SecretaryHomeSourceOwner = Readonly<{
  availability: "AVAILABLE" | "NOT_EMITTED_BY_SUMMARY";
  source_owner_ref: M4SecretaryTypedRef | null;
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
  module_entries: readonly SecretaryProfessionalModuleEntry[];
  model_enhancement: SecretaryHomeModelEnhancement;
  handoff: SecretaryHomeHandoff;
  degradation_code: string | null;
}>;
