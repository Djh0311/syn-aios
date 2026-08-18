//! M6 organization DTOs.  These shapes contain identifiers, versions,
//! watermarks, hashes, scrubbed refs, and outcomes only; no raw project data.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum M6OrgFreshnessState {
    Fresh,
    Stale,
    Missing,
    Denied,
    Degraded,
}

impl M6OrgFreshnessState {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Fresh => "fresh",
            Self::Stale => "stale",
            Self::Missing => "missing",
            Self::Denied => "denied",
            Self::Degraded => "degraded",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct M6OrgProjectSummaryQueryInput {
    pub(crate) summary_id: Option<String>,
    pub(crate) project_id: String,
    pub(crate) project_owner_ref: Option<String>,
    pub(crate) policy_decision_ref: Option<String>,
    pub(crate) expected_schema_version: Option<String>,
    pub(crate) expected_version: Option<u64>,
    pub(crate) expected_source_watermark: Option<i64>,
    pub(crate) expected_summary_hash: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct M6OrgConsultHandoffRefInput {
    pub(crate) consult_handoff_ref: String,
    pub(crate) handoff_id: String,
    pub(crate) handoff_revision: u64,
    pub(crate) status_ref: String,
    pub(crate) receipt_ref: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct M6OrgCrossProjectAdvisoryRequest {
    pub(crate) project_queries: Vec<M6OrgProjectSummaryQueryInput>,
    pub(crate) consult_handoff: M6OrgConsultHandoffRefInput,
    pub(crate) idempotency_key: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct M6OrgFreshnessJudgement {
    pub(crate) freshness_state: M6OrgFreshnessState,
    pub(crate) subject_ref: String,
    pub(crate) reason_code: String,
    pub(crate) judged_at_ms: i64,
    pub(crate) consumer_gate_ref: String,
    pub(crate) cache_reuse: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct M6OrgConsumedProjectSummaryRef {
    pub(crate) summary_id: String,
    pub(crate) project_id: String,
    pub(crate) project_owner_ref: String,
    pub(crate) schema_version: String,
    pub(crate) version: u64,
    pub(crate) source_watermark: i64,
    pub(crate) summary_hash: String,
    pub(crate) policy_decision_ref: String,
    pub(crate) freshness_state: M6OrgFreshnessState,
    pub(crate) source_refs: Vec<String>,
    pub(crate) orchestration_id: String,
    pub(crate) fact_count: u32,
    pub(crate) unverified_claim_count: u32,
    pub(crate) open_run_count: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct M6OrgAdvisorySourceLink {
    pub(crate) source_link_id: String,
    pub(crate) object_ref: String,
    pub(crate) project_id: String,
    pub(crate) summary_id: String,
    pub(crate) title_ref: String,
    pub(crate) scrubbed_summary_ref: String,
    pub(crate) deep_link_metadata_ref: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct M6OrgAdvisoryFinding {
    pub(crate) finding_id: String,
    pub(crate) finding_kind: String,
    pub(crate) reason_code: String,
    pub(crate) priority: u32,
    pub(crate) summary_refs: Vec<String>,
    pub(crate) source_link_refs: Vec<String>,
    pub(crate) explanation_ref: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct M6OrgCrossProjectAdvisory {
    pub(crate) advisory_id: String,
    pub(crate) global_role_session_id: String,
    pub(crate) consult_handoff_ref: String,
    pub(crate) consumed_summaries: Vec<M6OrgConsumedProjectSummaryRef>,
    pub(crate) policy_decision_ref: String,
    pub(crate) generated_at_ms: i64,
    pub(crate) source_links: Vec<M6OrgAdvisorySourceLink>,
    pub(crate) findings: Vec<M6OrgAdvisoryFinding>,
    pub(crate) lifecycle_status: String,
    pub(crate) freshness_state: M6OrgFreshnessState,
    pub(crate) revision: u64,
    pub(crate) idempotency_key: String,
    pub(crate) request_hash: String,
    pub(crate) created_at_ms: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct M6OrgAdvisoryAdoptionRequest {
    pub(crate) advisory_id: String,
    pub(crate) actor_ref: String,
    pub(crate) user_confirmed: bool,
    pub(crate) idempotency_key: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct M6OrgDecisionRequest {
    pub(crate) decision_request_id: String,
    pub(crate) source_owner_ref: String,
    pub(crate) source_object_ref: String,
    pub(crate) source_revision: u64,
    pub(crate) requesting_actor_id: String,
    pub(crate) required_actor_ref: String,
    pub(crate) required_scope_ref: String,
    pub(crate) question_schema_ref: String,
    pub(crate) allowed_answer_schema_ref: String,
    pub(crate) decision_command_type: String,
    pub(crate) status: String,
    pub(crate) idempotency_key: String,
    pub(crate) created_at_ms: i64,
    pub(crate) expires_at_ms: i64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum M6OrgApplicationOutcome {
    Applied,
    Failed,
    RolledBack,
    Unknown,
}

impl M6OrgApplicationOutcome {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Applied => "applied",
            Self::Failed => "failed",
            Self::RolledBack => "rolled_back",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct M6OrgApplicationReceiptObservationRequest {
    pub(crate) advisory_id: String,
    pub(crate) decision_request_id: String,
    pub(crate) project_id: String,
    pub(crate) project_owner_ref: String,
    pub(crate) authoritative_command_receipt_ref: String,
    pub(crate) grant_ref: String,
    pub(crate) outcome: M6OrgApplicationOutcome,
    pub(crate) observed_at_ms: i64,
    pub(crate) source_receipt_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct M6OrgPerProjectApplicationObservation {
    pub(crate) observation_id: String,
    pub(crate) advisory_id: String,
    pub(crate) decision_request_id: String,
    pub(crate) project_id: String,
    pub(crate) project_owner_ref: String,
    pub(crate) authoritative_command_receipt_ref: String,
    pub(crate) grant_ref: String,
    pub(crate) outcome: M6OrgApplicationOutcome,
    pub(crate) observed_at_ms: i64,
    pub(crate) source_receipt_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct M6OrgAdvisoryApplicationProjection {
    pub(crate) application_projection_id: String,
    pub(crate) advisory_id: String,
    pub(crate) advisory_revision: u64,
    pub(crate) decision_request_id: String,
    pub(crate) observations: Vec<M6OrgPerProjectApplicationObservation>,
    pub(crate) partial_apply: bool,
    pub(crate) compensation_observation_refs: Vec<String>,
    pub(crate) history: Vec<M6OrgPerProjectApplicationObservation>,
    pub(crate) projected_at_ms: i64,
    pub(crate) projection_revision: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct M6OrgProjectWriteAttemptRequest {
    pub(crate) project_id: String,
    pub(crate) mutation_kind: String,
}

/// Renderer-safe Secretary input.  The question and sources are references;
/// raw question text and source bodies are deliberately not representable.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct M6OrgSecretaryConsultStartRequest {
    pub(crate) question_ref: String,
    pub(crate) source_refs: Vec<String>,
    pub(crate) project_queries: Vec<M6OrgProjectSummaryQueryInput>,
    pub(crate) accept_by_utc: String,
    pub(crate) return_by_utc: String,
    pub(crate) idempotency_key: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum M6OrgConsultDecision {
    Accept,
    Reject,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum M6OrgConsultRejectionReason {
    OutOfScope,
    InsufficientEvidence,
    PolicyDenied,
    RecipientUnavailable,
}

impl M6OrgConsultRejectionReason {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::OutOfScope => "OUT_OF_SCOPE",
            Self::InsufficientEvidence => "INSUFFICIENT_EVIDENCE",
            Self::PolicyDenied => "POLICY_DENIED",
            Self::RecipientUnavailable => "RECIPIENT_UNAVAILABLE",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct M6OrgGlobalSupervisorConsultDecisionRequest {
    pub(crate) handoff_id: String,
    pub(crate) decision: M6OrgConsultDecision,
    pub(crate) rejection_reason: Option<M6OrgConsultRejectionReason>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct M6OrgSecretaryConsultReadRequest {
    pub(crate) handoff_id: String,
}

/// Typed projection of the exact M3 Handoff plus its current M3 receipt.  It
/// is rebuilt from M3 on every response and is never an authority source.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct M6OrgConsultHandoffBinding {
    pub(crate) consult_handoff_ref: String,
    pub(crate) handoff_id: String,
    pub(crate) handoff_revision: u64,
    pub(crate) status_ref: String,
    pub(crate) consult_kind: String,
    pub(crate) from_role_session_id: String,
    pub(crate) to_role_ref: String,
    pub(crate) to_recipient_ref: String,
    pub(crate) scope_ref: String,
    pub(crate) question_ref: String,
    pub(crate) object_refs: Vec<String>,
    pub(crate) receipt_ref: String,
    pub(crate) project_write_capability: bool,
}

/// M6 remembers only request material and result references needed to replay
/// M3 commands.  Lifecycle state and current receipt remain M3-owned.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct M6OrgConsultHandoffProjection {
    pub(crate) handoff_id: String,
    pub(crate) idempotency_key: String,
    pub(crate) request_hash: String,
    pub(crate) request: M6OrgSecretaryConsultStartRequest,
    pub(crate) source_session_revision: u64,
    pub(crate) source_validation_receipt_ref: String,
    pub(crate) advisory: Option<M6OrgCrossProjectAdvisory>,
    pub(crate) advisory_blocked_reasons: Vec<String>,
    pub(crate) accepted_receipt_ref: Option<String>,
    pub(crate) accepted_revision: Option<u64>,
    pub(crate) returned_receipt_ref: Option<String>,
    pub(crate) rejection_reason: Option<M6OrgConsultRejectionReason>,
    pub(crate) created_at_ms: i64,
    pub(crate) updated_at_ms: i64,
}
