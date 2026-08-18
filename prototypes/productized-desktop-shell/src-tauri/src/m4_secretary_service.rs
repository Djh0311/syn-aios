//! M4C05 Secretary application-service core.
//!
//! This module is deliberately adapter-free. It accepts only typed, scrubbed
//! reads and writes through narrow ports, rebuilds its context and brief
//! mechanically, and never becomes a source-owner, provider, path, or
//! credential authority.

use crate::m4_secretary_domain::m4_parse_rfc3339_utc_key;
use crate::m4_secretary_read_model::{
    sort_m4c04_coordination_snapshot, M4CoordinationSnapshot, M4DecisionRead, M4InboxItemRead,
    M4NotificationRead, M4OpenLoopRead, M4PersonalActionRead, M4ReminderRead, M4SourceLinkRead,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::collections::BTreeSet;

const M4C05_PERSONAL_SCOPE_REF: &str = "scope:personal:primary";
const M4C05_SECRETARY_ROLE_REF: &str = "role:secretary:personal-primary";
const M4C05_PERSONAL_OBJECT_REF: &str = "personal-workbench:primary";
const M4C05_DAILY_CHANNEL_CODE: &str = "DAILY";
const M4C05_ACTIVE_SESSION_CODE: &str = "ACTIVE";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4SecretaryServiceError {
    pub(crate) code: String,
}

impl M4SecretaryServiceError {
    pub(crate) fn new(code: impl Into<String>) -> Self {
        Self { code: code.into() }
    }
}

impl std::fmt::Display for M4SecretaryServiceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.code)
    }
}

impl std::error::Error for M4SecretaryServiceError {}

/// A server-minted typed reference. It intentionally has no path, URL,
/// credential, callback, raw prompt, or raw provider-body variant.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub(crate) struct M4SecretaryTypedRef(String);

impl M4SecretaryTypedRef {
    pub(crate) fn new(value: impl Into<String>) -> Result<Self, M4SecretaryServiceError> {
        let value = value.into();
        if !m4c05_is_safe_reference(&value) {
            return Err(M4SecretaryServiceError::new(
                "m4c05_typed_reference_invalid",
            ));
        }
        Ok(Self(value))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

/// An opaque, server-resolved reference with a content-addressed suffix.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub(crate) struct M4SecretaryOpaqueRef(String);

impl M4SecretaryOpaqueRef {
    pub(crate) fn new(value: impl Into<String>) -> Result<Self, M4SecretaryServiceError> {
        let value = value.into();
        if !m4c05_is_opaque_reference(&value) {
            return Err(M4SecretaryServiceError::new(
                "m4c05_opaque_reference_invalid",
            ));
        }
        Ok(Self(value))
    }

    fn derived(namespace: &str, hash: &M4SecretaryHash) -> Self {
        Self(format!("{namespace}:sha256:{}", hash.as_str()))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub(crate) struct M4SecretaryHash(String);

impl M4SecretaryHash {
    pub(crate) fn new(value: impl Into<String>) -> Result<Self, M4SecretaryServiceError> {
        let value = value.into();
        if !m4c05_is_lower_hex_hash(&value) {
            return Err(M4SecretaryServiceError::new("m4c05_hash_invalid"));
        }
        Ok(Self(value))
    }

    fn of_texts(domain_separator: &str, fields: &[&str]) -> Self {
        let fields = fields
            .iter()
            .map(|field| field.as_bytes())
            .collect::<Vec<_>>();
        Self::of_bytes(domain_separator, &fields)
    }

    fn of_bytes(domain_separator: &str, fields: &[&[u8]]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(domain_separator.as_bytes());
        for field in fields {
            hasher.update(
                u32::try_from(field.len())
                    .expect("M4C05 hash input length fits u32")
                    .to_be_bytes(),
            );
            hasher.update(field);
        }
        Self(format!("{:x}", hasher.finalize()))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

/// A server-resolved M3 RoleSession view. The adapter must not infer any
/// field from cwd, a project locator, frontend cache, or a provider handle.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct M4SecretaryRoleSessionState {
    pub(crate) role_session_ref: M4SecretaryOpaqueRef,
    pub(crate) role_ref: M4SecretaryTypedRef,
    pub(crate) scope_ref: M4SecretaryTypedRef,
    pub(crate) current_object_ref: M4SecretaryTypedRef,
    pub(crate) execution_channel_code: String,
    pub(crate) session_state_code: String,
    pub(crate) permission_snapshot_ref: M4SecretaryOpaqueRef,
    pub(crate) owner_fingerprint: M4SecretaryHash,
}

impl M4SecretaryRoleSessionState {
    fn validate_fixed_personal_scope(&self) -> Result<(), M4SecretaryServiceError> {
        if self.role_ref.as_str() != M4C05_SECRETARY_ROLE_REF
            || self.scope_ref.as_str() != M4C05_PERSONAL_SCOPE_REF
            || self.current_object_ref.as_str() != M4C05_PERSONAL_OBJECT_REF
            || self.execution_channel_code != M4C05_DAILY_CHANNEL_CODE
            || self.session_state_code != M4C05_ACTIVE_SESSION_CODE
        {
            return Err(M4SecretaryServiceError::new(
                "m4c05_secretary_role_session_mismatch",
            ));
        }
        if !m4c05_is_status_code(&self.execution_channel_code)
            || !m4c05_is_status_code(&self.session_state_code)
        {
            return Err(M4SecretaryServiceError::new(
                "m4c05_secretary_role_session_code_invalid",
            ));
        }
        Ok(())
    }
}

/// Read-only M3 boundary. Its sole result is an already server-resolved,
/// exact PersonalScope Secretary session.
pub(crate) trait M4SecretaryRoleSessionReadPort {
    fn read_personal_secretary_role_session(
        &self,
    ) -> Result<M4SecretaryRoleSessionState, M4SecretaryServiceError>;
}

/// Read-only M4 boundary. The adapter returns a M4-owned coordination
/// snapshot, never a source-owner fact mutation or frontend cache.
pub(crate) trait M4SecretaryCoordinationSnapshotReadPort {
    fn read_coordination_snapshot(
        &self,
        scope_ref: &M4SecretaryTypedRef,
    ) -> Result<M4CoordinationSnapshot, M4SecretaryServiceError>;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct M4SecretaryHandoffRequest {
    pub(crate) request_ref: M4SecretaryOpaqueRef,
    pub(crate) from_role_session_ref: M4SecretaryOpaqueRef,
    pub(crate) scope_ref: M4SecretaryTypedRef,
    pub(crate) to_role_ref: M4SecretaryTypedRef,
    pub(crate) to_recipient_ref: M4SecretaryOpaqueRef,
    pub(crate) requested_outcome_ref: M4SecretaryOpaqueRef,
    pub(crate) object_refs: Vec<M4SecretaryTypedRef>,
    pub(crate) risk_class_code: String,
    pub(crate) reason_ref: M4SecretaryOpaqueRef,
    pub(crate) permission_request_ref: M4SecretaryOpaqueRef,
    pub(crate) correlation_ref: M4SecretaryOpaqueRef,
}

impl M4SecretaryHandoffRequest {
    fn validate(&self) -> Result<(), M4SecretaryServiceError> {
        if self.object_refs.is_empty() || !m4c05_is_status_code(&self.risk_class_code) {
            return Err(M4SecretaryServiceError::new(
                "m4c05_handoff_request_invalid",
            ));
        }
        Ok(())
    }
}

/// This is only M3's scrubbed return receipt shape. It cannot carry source
/// state, a callback, executable payload, credential, or returned body.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct M4SecretaryHandoffReceipt {
    pub(crate) receipt_ref: M4SecretaryOpaqueRef,
    pub(crate) handoff_ref: M4SecretaryOpaqueRef,
    pub(crate) receipt_kind_code: String,
    pub(crate) status_code: String,
    pub(crate) result_ref: Option<M4SecretaryOpaqueRef>,
    pub(crate) result_hash: Option<M4SecretaryHash>,
}

impl M4SecretaryHandoffReceipt {
    fn validate(&self) -> Result<(), M4SecretaryServiceError> {
        if !m4c05_is_status_code(&self.receipt_kind_code)
            || !m4c05_is_status_code(&self.status_code)
            || self.result_ref.is_some() != self.result_hash.is_some()
            || (self.status_code == "RETURNED" && self.result_ref.is_none())
        {
            return Err(M4SecretaryServiceError::new(
                "m4c05_handoff_receipt_invalid",
            ));
        }
        Ok(())
    }
}

/// The generic M3 handoff adapter gives the application service only durable
/// coordination states and scrubbed receipts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum M4SecretaryHandoffPortRecord {
    Unavailable {
        error_code: String,
    },
    Pending {
        handoff_ref: M4SecretaryOpaqueRef,
        request_receipt_ref: M4SecretaryOpaqueRef,
    },
    Returned {
        handoff_ref: M4SecretaryOpaqueRef,
        receipt: M4SecretaryHandoffReceipt,
    },
    Failed {
        handoff_ref: M4SecretaryOpaqueRef,
        receipt: Option<M4SecretaryHandoffReceipt>,
        recovery_code: String,
    },
}

/// M3 remains the sole Handoff writer. M4C05 only asks it to create a
/// bounded request or reload a durable receipt.
pub(crate) trait M4SecretaryHandoffPort {
    fn create_handoff(
        &self,
        request: &M4SecretaryHandoffRequest,
    ) -> Result<M4SecretaryHandoffPortRecord, M4SecretaryServiceError>;

    fn read_handoff_receipt(
        &self,
        handoff_ref: &M4SecretaryOpaqueRef,
    ) -> Result<M4SecretaryHandoffPortRecord, M4SecretaryServiceError>;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum M4SecretaryHandoffStatus {
    Unavailable,
    Pending,
    Returned,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct M4SecretaryHandoffOutcome {
    pub(crate) status: M4SecretaryHandoffStatus,
    pub(crate) handoff_ref: Option<M4SecretaryOpaqueRef>,
    pub(crate) request_receipt_ref: Option<M4SecretaryOpaqueRef>,
    pub(crate) returned_receipt: Option<M4SecretaryHandoffReceipt>,
    pub(crate) recovery_code: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct M4SecretaryContext {
    pub(crate) context_ref: M4SecretaryOpaqueRef,
    pub(crate) role_session_ref: M4SecretaryOpaqueRef,
    pub(crate) scope_ref: M4SecretaryTypedRef,
    pub(crate) scope_source_watermark: M4SecretaryHash,
    pub(crate) snapshot_hash: M4SecretaryHash,
    pub(crate) reconstruction_code: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct M4SecretarySourceBackedBriefItem {
    pub(crate) item_ref: M4SecretaryTypedRef,
    pub(crate) item_kind_code: String,
    pub(crate) source_owner_ref: M4SecretaryTypedRef,
    pub(crate) source_object_type: String,
    pub(crate) source_object_ref: M4SecretaryTypedRef,
    pub(crate) source_route_ref: M4SecretaryOpaqueRef,
    pub(crate) source_summary_ref: M4SecretaryOpaqueRef,
    pub(crate) why_code: String,
    pub(crate) priority_rank: u8,
    pub(crate) priority_code: String,
    pub(crate) status_code: String,
    pub(crate) source_status_code: String,
    /// Canonical decimal M4 coordination revision. It stays a string so a
    /// renderer never routes it through a lossy JavaScript number.
    pub(crate) coordination_revision: String,
    pub(crate) due_at_utc: Option<String>,
    pub(crate) last_change_at_utc: String,
    pub(crate) change_hash: M4SecretaryHash,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct M4SecretaryPersonalActionBriefItem {
    pub(crate) personal_action_ref: M4SecretaryTypedRef,
    pub(crate) explicit_user_command_ref: M4SecretaryOpaqueRef,
    pub(crate) status_code: String,
    pub(crate) due_at_utc: Option<String>,
    /// Canonical decimal M4 coordination revision, retained independently
    /// from the opaque revision hash used for deterministic integrity.
    pub(crate) coordination_revision: String,
    pub(crate) revision_hash: M4SecretaryHash,
}

/// This brief is always mechanical: it excludes titles, raw user text, model
/// prose, and provider output while retaining source ownership and routing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct M4SecretaryDeterministicBrief {
    pub(crate) brief_ref: M4SecretaryOpaqueRef,
    pub(crate) brief_hash: M4SecretaryHash,
    pub(crate) context_ref: M4SecretaryOpaqueRef,
    pub(crate) scope_source_watermark: M4SecretaryHash,
    pub(crate) attention_items: Vec<M4SecretarySourceBackedBriefItem>,
    pub(crate) personal_actions: Vec<M4SecretaryPersonalActionBriefItem>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct M4SecretaryExplicitUserMessageTrigger {
    pub(crate) trigger_ref: M4SecretaryOpaqueRef,
    pub(crate) user_message_ref: M4SecretaryOpaqueRef,
    pub(crate) user_message_hash: M4SecretaryHash,
    pub(crate) idempotency_key_ref: M4SecretaryOpaqueRef,
    pub(crate) purpose_code: String,
}

impl M4SecretaryExplicitUserMessageTrigger {
    fn validate(&self) -> Result<(), M4SecretaryServiceError> {
        if !m4c05_is_status_code(&self.purpose_code) {
            return Err(M4SecretaryServiceError::new(
                "m4c05_user_enhancement_trigger_invalid",
            ));
        }
        Ok(())
    }
}

/// Only the explicit-user variant can cause a model invocation. All other
/// variants still return the deterministic context and brief, with zero
/// invocation-ledger writes and zero model calls.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum M4SecretaryServiceTrigger {
    ReadOnlyQuery,
    TimerTick { tick_ref: M4SecretaryOpaqueRef },
    CoordinationCommand { command_ref: M4SecretaryOpaqueRef },
    StartupRecovery { recovery_ref: M4SecretaryOpaqueRef },
    ExplicitUserMessage(M4SecretaryExplicitUserMessageTrigger),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct M4SecretaryModelInvocationClaim {
    pub(crate) invocation_key_ref: M4SecretaryOpaqueRef,
    pub(crate) idempotency_key_ref: M4SecretaryOpaqueRef,
    pub(crate) immutable_input_hash: M4SecretaryHash,
    pub(crate) role_session_ref: M4SecretaryOpaqueRef,
    pub(crate) context_ref: M4SecretaryOpaqueRef,
    pub(crate) deterministic_brief_hash: M4SecretaryHash,
    pub(crate) trigger_ref: M4SecretaryOpaqueRef,
    pub(crate) user_message_ref: M4SecretaryOpaqueRef,
    pub(crate) user_message_hash: M4SecretaryHash,
    pub(crate) purpose_code: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct M4SecretaryInvocationTerminal {
    pub(crate) invocation_ref: M4SecretaryOpaqueRef,
    pub(crate) outcome_code: String,
    pub(crate) result_ref: Option<M4SecretaryOpaqueRef>,
    pub(crate) result_hash: Option<M4SecretaryHash>,
    pub(crate) error_code: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct M4SecretaryInvocationReceipt {
    pub(crate) invocation_ref: M4SecretaryOpaqueRef,
    pub(crate) terminal_receipt_ref: M4SecretaryOpaqueRef,
    pub(crate) outcome_code: String,
    pub(crate) result_ref: Option<M4SecretaryOpaqueRef>,
    pub(crate) result_hash: Option<M4SecretaryHash>,
    pub(crate) error_code: Option<String>,
}

impl M4SecretaryInvocationReceipt {
    fn validate(&self) -> Result<(), M4SecretaryServiceError> {
        if !m4c05_is_status_code(&self.outcome_code)
            || self
                .error_code
                .as_deref()
                .is_some_and(|code| !m4c05_is_status_code(code))
            || self.result_ref.is_some() != self.result_hash.is_some()
        {
            return Err(M4SecretaryServiceError::new(
                "m4c05_invocation_receipt_invalid",
            ));
        }
        match self.outcome_code.as_str() {
            "SUCCEEDED" if self.result_ref.is_some() && self.error_code.is_none() => Ok(()),
            "FAILED" if self.result_ref.is_none() && self.error_code.is_some() => Ok(()),
            _ => Err(M4SecretaryServiceError::new(
                "m4c05_invocation_receipt_terminal_invalid",
            )),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum M4SecretaryInvocationClaimOutcome {
    DispatchGranted {
        invocation_ref: M4SecretaryOpaqueRef,
    },
    Replay {
        receipt: M4SecretaryInvocationReceipt,
    },
    InFlight {
        invocation_ref: M4SecretaryOpaqueRef,
    },
    Rejected {
        error_code: String,
    },
}

/// The durable invocation ledger is the dispatch gate. A model port is never
/// called before it grants a specific invocation reference.
pub(crate) trait M4SecretaryModelInvocationLedgerPort {
    fn claim_invocation(
        &self,
        claim: &M4SecretaryModelInvocationClaim,
    ) -> Result<M4SecretaryInvocationClaimOutcome, M4SecretaryServiceError>;

    fn terminal_invocation(
        &self,
        terminal: &M4SecretaryInvocationTerminal,
    ) -> Result<M4SecretaryInvocationReceipt, M4SecretaryServiceError>;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct M4SecretaryModelEnhancementRequest {
    pub(crate) invocation_ref: M4SecretaryOpaqueRef,
    pub(crate) context_ref: M4SecretaryOpaqueRef,
    pub(crate) deterministic_brief_ref: M4SecretaryOpaqueRef,
    pub(crate) deterministic_brief_hash: M4SecretaryHash,
    pub(crate) trigger_ref: M4SecretaryOpaqueRef,
    pub(crate) user_message_ref: M4SecretaryOpaqueRef,
    pub(crate) user_message_hash: M4SecretaryHash,
    pub(crate) purpose_code: String,
}

/// A controlled model adapter receives opaque source-backed inputs and returns
/// only a scrubbed enhancement reference plus hash, never a raw response.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum M4SecretaryModelPortOutcome {
    Enhanced {
        enhancement_ref: M4SecretaryOpaqueRef,
        result_hash: M4SecretaryHash,
    },
    Failed {
        error_code: String,
    },
}

pub(crate) trait M4SecretaryControlledModelEnhancementPort {
    fn enhance(
        &self,
        request: &M4SecretaryModelEnhancementRequest,
    ) -> Result<M4SecretaryModelPortOutcome, M4SecretaryServiceError>;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum M4SecretaryModelEnhancementStatus {
    Available,
    Failed,
    Pending,
    Replayed,
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct M4SecretaryModelEnhancementOutcome {
    pub(crate) status: M4SecretaryModelEnhancementStatus,
    pub(crate) invocation_ref: Option<M4SecretaryOpaqueRef>,
    pub(crate) enhancement_ref: Option<M4SecretaryOpaqueRef>,
    pub(crate) enhancement_hash: Option<M4SecretaryHash>,
    pub(crate) invocation_receipt: Option<M4SecretaryInvocationReceipt>,
    pub(crate) recovery_code: Option<String>,
}

/// Renderer-only local objects.  PersonalAction titles are intentionally
/// carried here and never copied into the deterministic/model-facing brief.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct M4SecretaryLocalSourceLink {
    pub(crate) link_kind: String,
    pub(crate) source_owner_ref: String,
    pub(crate) object_type: String,
    pub(crate) canonical_source_object_id: String,
    /// Decimal text keeps the full u64 owner revision across JSON/JavaScript.
    pub(crate) expected_source_revision: String,
    pub(crate) opaque_route_ref: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct M4SecretaryLocalNotification {
    pub(crate) notification_id: String,
    pub(crate) source_ref: M4SecretaryLocalSourceLink,
    pub(crate) subject_ref: String,
    pub(crate) notification_purpose_code: String,
    pub(crate) delivery_channel: String,
    pub(crate) status: String,
    pub(crate) created_at_utc: String,
    pub(crate) delivered_at_utc: Option<String>,
    pub(crate) read_at_utc: Option<String>,
    pub(crate) dismissed_at_utc: Option<String>,
    pub(crate) revision: String,
}

impl From<M4NotificationRead> for M4SecretaryLocalNotification {
    fn from(notification: M4NotificationRead) -> Self {
        let M4SourceLinkRead {
            link_kind,
            source_owner_ref,
            object_type,
            canonical_source_object_id,
            expected_source_revision,
            opaque_route_ref,
        } = notification.source_ref;
        Self {
            notification_id: notification.notification_id,
            source_ref: M4SecretaryLocalSourceLink {
                link_kind,
                source_owner_ref,
                object_type,
                canonical_source_object_id,
                expected_source_revision: expected_source_revision.to_string(),
                opaque_route_ref,
            },
            subject_ref: notification.subject_ref,
            notification_purpose_code: notification.notification_purpose_code,
            delivery_channel: notification.delivery_channel,
            status: notification.status,
            created_at_utc: notification.created_at_utc,
            delivered_at_utc: notification.delivered_at_utc,
            read_at_utc: notification.read_at_utc,
            dismissed_at_utc: notification.dismissed_at_utc,
            revision: notification.revision,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct M4SecretaryLocalObjects {
    pub(crate) personal_actions: Vec<M4PersonalActionRead>,
    pub(crate) notifications: Vec<M4SecretaryLocalNotification>,
    pub(crate) reminders: Vec<M4ReminderRead>,
    pub(crate) decisions: Vec<M4DecisionRead>,
    pub(crate) reminder_owner_refs: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct M4SecretaryApplicationOutcome {
    pub(crate) context: M4SecretaryContext,
    pub(crate) deterministic_brief: M4SecretaryDeterministicBrief,
    pub(crate) local_objects: M4SecretaryLocalObjects,
    pub(crate) model_enhancement: Option<M4SecretaryModelEnhancementOutcome>,
}

/// M4C05 orchestration. The type parameters are all narrow ports so ordinary
/// product composition can adapt M3, M4 SQLite, and a model independently.
pub(crate) struct M4SecretaryApplicationService<'a, RS, CS, HP, LP, MP>
where
    RS: M4SecretaryRoleSessionReadPort,
    CS: M4SecretaryCoordinationSnapshotReadPort,
    HP: M4SecretaryHandoffPort,
    LP: M4SecretaryModelInvocationLedgerPort,
    MP: M4SecretaryControlledModelEnhancementPort,
{
    role_session_port: &'a RS,
    coordination_port: &'a CS,
    handoff_port: &'a HP,
    invocation_ledger_port: &'a LP,
    model_port: &'a MP,
}

impl<'a, RS, CS, HP, LP, MP> M4SecretaryApplicationService<'a, RS, CS, HP, LP, MP>
where
    RS: M4SecretaryRoleSessionReadPort,
    CS: M4SecretaryCoordinationSnapshotReadPort,
    HP: M4SecretaryHandoffPort,
    LP: M4SecretaryModelInvocationLedgerPort,
    MP: M4SecretaryControlledModelEnhancementPort,
{
    pub(crate) fn new(
        role_session_port: &'a RS,
        coordination_port: &'a CS,
        handoff_port: &'a HP,
        invocation_ledger_port: &'a LP,
        model_port: &'a MP,
    ) -> Self {
        Self {
            role_session_port,
            coordination_port,
            handoff_port,
            invocation_ledger_port,
            model_port,
        }
    }

    pub(crate) fn read_deterministic_brief(
        &self,
    ) -> Result<M4SecretaryApplicationOutcome, M4SecretaryServiceError> {
        self.build_mechanical_outcome()
    }

    pub(crate) fn process(
        &self,
        trigger: &M4SecretaryServiceTrigger,
    ) -> Result<M4SecretaryApplicationOutcome, M4SecretaryServiceError> {
        let mut outcome = self.build_mechanical_outcome()?;
        if let M4SecretaryServiceTrigger::ExplicitUserMessage(user_trigger) = trigger {
            outcome.model_enhancement = Some(self.enhance_for_explicit_user_message(
                &outcome.context,
                &outcome.deterministic_brief,
                user_trigger,
            )?);
        }
        Ok(outcome)
    }

    pub(crate) fn request_handoff(
        &self,
        request: &M4SecretaryHandoffRequest,
    ) -> Result<M4SecretaryHandoffOutcome, M4SecretaryServiceError> {
        let role_session = self
            .role_session_port
            .read_personal_secretary_role_session()?;
        role_session.validate_fixed_personal_scope()?;
        request.validate()?;
        if request.from_role_session_ref != role_session.role_session_ref
            || request.scope_ref != role_session.scope_ref
        {
            return Err(M4SecretaryServiceError::new(
                "m4c05_handoff_request_binding_mismatch",
            ));
        }
        Self::map_handoff_record(self.handoff_port.create_handoff(request)?)
    }

    pub(crate) fn read_handoff_receipt(
        &self,
        handoff_ref: &M4SecretaryOpaqueRef,
    ) -> Result<M4SecretaryHandoffOutcome, M4SecretaryServiceError> {
        self.role_session_port
            .read_personal_secretary_role_session()?
            .validate_fixed_personal_scope()?;
        Self::map_handoff_record(self.handoff_port.read_handoff_receipt(handoff_ref)?)
    }

    fn build_mechanical_outcome(
        &self,
    ) -> Result<M4SecretaryApplicationOutcome, M4SecretaryServiceError> {
        let role_session = self
            .role_session_port
            .read_personal_secretary_role_session()?;
        role_session.validate_fixed_personal_scope()?;
        let mut snapshot = self
            .coordination_port
            .read_coordination_snapshot(&role_session.scope_ref)?;
        validate_m4c05_snapshot(&snapshot, &role_session.scope_ref)?;
        sort_m4c04_coordination_snapshot(&mut snapshot)
            .map_err(|_| M4SecretaryServiceError::new("m4c05_coordination_snapshot_invalid"))?;

        let snapshot_bytes = serde_json::to_vec(&snapshot)
            .map_err(|_| M4SecretaryServiceError::new("m4c05_snapshot_hash_encoding_failed"))?;
        let snapshot_hash =
            M4SecretaryHash::of_bytes("syn.m4.secretary.snapshot/v1", &[snapshot_bytes.as_slice()]);
        let scope_source_watermark = M4SecretaryHash::new(snapshot.scope_source_watermark.clone())?;
        let context_hash = M4SecretaryHash::of_texts(
            "syn.m4.secretary.context/v1",
            &[
                role_session.role_session_ref.as_str(),
                role_session.role_ref.as_str(),
                role_session.scope_ref.as_str(),
                role_session.current_object_ref.as_str(),
                &role_session.execution_channel_code,
                role_session.permission_snapshot_ref.as_str(),
                role_session.owner_fingerprint.as_str(),
                scope_source_watermark.as_str(),
                snapshot_hash.as_str(),
            ],
        );
        let context = M4SecretaryContext {
            context_ref: M4SecretaryOpaqueRef::derived("secretary-context", &context_hash),
            role_session_ref: role_session.role_session_ref,
            scope_ref: role_session.scope_ref,
            scope_source_watermark: scope_source_watermark.clone(),
            snapshot_hash,
            reconstruction_code: "REBUILDABLE_FROM_M3_M4_REFS".to_string(),
        };

        let mut attention_items = snapshot
            .inbox_items
            .iter()
            .map(m4c05_brief_item_from_inbox)
            .chain(
                snapshot
                    .open_loops
                    .iter()
                    .map(m4c05_brief_item_from_open_loop),
            )
            .collect::<Result<Vec<_>, _>>()?;
        attention_items.sort_by(m4c05_compare_brief_items);
        let personal_actions = snapshot
            .personal_actions
            .iter()
            .map(m4c05_brief_personal_action)
            .collect::<Result<Vec<_>, _>>()?;

        let brief_hash_input = M4SecretaryBriefHashInput {
            context_ref: &context.context_ref,
            scope_source_watermark: &scope_source_watermark,
            attention_items: &attention_items,
            personal_actions: &personal_actions,
        };
        let brief_hash_bytes = serde_json::to_vec(&brief_hash_input)
            .map_err(|_| M4SecretaryServiceError::new("m4c05_brief_hash_encoding_failed"))?;
        let brief_hash =
            M4SecretaryHash::of_bytes("syn.m4.secretary.brief/v1", &[brief_hash_bytes.as_slice()]);
        let deterministic_brief = M4SecretaryDeterministicBrief {
            brief_ref: M4SecretaryOpaqueRef::derived("secretary-brief", &brief_hash),
            brief_hash,
            context_ref: context.context_ref.clone(),
            scope_source_watermark,
            attention_items,
            personal_actions,
        };
        let reminder_owner_refs = snapshot
            .inbox_items
            .iter()
            .map(|item| item.source_identity_key.clone())
            .chain(
                snapshot
                    .open_loops
                    .iter()
                    .map(|item| item.source_identity_key.clone()),
            )
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let local_objects = M4SecretaryLocalObjects {
            personal_actions: snapshot.personal_actions,
            notifications: snapshot
                .notifications
                .into_iter()
                .map(M4SecretaryLocalNotification::from)
                .collect(),
            reminders: snapshot.reminders,
            decisions: snapshot.decisions,
            reminder_owner_refs,
        };
        Ok(M4SecretaryApplicationOutcome {
            context,
            deterministic_brief,
            local_objects,
            model_enhancement: None,
        })
    }

    fn enhance_for_explicit_user_message(
        &self,
        context: &M4SecretaryContext,
        brief: &M4SecretaryDeterministicBrief,
        trigger: &M4SecretaryExplicitUserMessageTrigger,
    ) -> Result<M4SecretaryModelEnhancementOutcome, M4SecretaryServiceError> {
        trigger.validate()?;
        let immutable_input_hash = M4SecretaryHash::of_texts(
            "syn.m4.secretary.model-invocation-input/v1",
            &[
                context.role_session_ref.as_str(),
                context.context_ref.as_str(),
                brief.brief_hash.as_str(),
                trigger.trigger_ref.as_str(),
                trigger.user_message_ref.as_str(),
                trigger.user_message_hash.as_str(),
                trigger.idempotency_key_ref.as_str(),
                &trigger.purpose_code,
            ],
        );
        let claim = M4SecretaryModelInvocationClaim {
            invocation_key_ref: M4SecretaryOpaqueRef::derived(
                "secretary-invocation",
                &immutable_input_hash,
            ),
            idempotency_key_ref: trigger.idempotency_key_ref.clone(),
            immutable_input_hash,
            role_session_ref: context.role_session_ref.clone(),
            context_ref: context.context_ref.clone(),
            deterministic_brief_hash: brief.brief_hash.clone(),
            trigger_ref: trigger.trigger_ref.clone(),
            user_message_ref: trigger.user_message_ref.clone(),
            user_message_hash: trigger.user_message_hash.clone(),
            purpose_code: trigger.purpose_code.clone(),
        };

        match self.invocation_ledger_port.claim_invocation(&claim)? {
            M4SecretaryInvocationClaimOutcome::DispatchGranted { invocation_ref } => {
                let request = M4SecretaryModelEnhancementRequest {
                    invocation_ref: invocation_ref.clone(),
                    context_ref: context.context_ref.clone(),
                    deterministic_brief_ref: brief.brief_ref.clone(),
                    deterministic_brief_hash: brief.brief_hash.clone(),
                    trigger_ref: trigger.trigger_ref.clone(),
                    user_message_ref: trigger.user_message_ref.clone(),
                    user_message_hash: trigger.user_message_hash.clone(),
                    purpose_code: trigger.purpose_code.clone(),
                };
                match self.model_port.enhance(&request) {
                    Ok(M4SecretaryModelPortOutcome::Enhanced {
                        enhancement_ref,
                        result_hash,
                    }) => {
                        let receipt = self.terminal_model_invocation(
                            &invocation_ref,
                            "SUCCEEDED",
                            Some(enhancement_ref.clone()),
                            Some(result_hash.clone()),
                            None,
                        )?;
                        Ok(M4SecretaryModelEnhancementOutcome {
                            status: M4SecretaryModelEnhancementStatus::Available,
                            invocation_ref: Some(invocation_ref),
                            enhancement_ref: Some(enhancement_ref),
                            enhancement_hash: Some(result_hash),
                            invocation_receipt: Some(receipt),
                            recovery_code: None,
                        })
                    }
                    Ok(M4SecretaryModelPortOutcome::Failed { error_code }) => {
                        let error_code = m4c05_scrubbed_model_error(&error_code);
                        let receipt = self.terminal_model_invocation(
                            &invocation_ref,
                            "FAILED",
                            None,
                            None,
                            Some(error_code.clone()),
                        )?;
                        Ok(M4SecretaryModelEnhancementOutcome {
                            status: M4SecretaryModelEnhancementStatus::Failed,
                            invocation_ref: Some(invocation_ref),
                            enhancement_ref: None,
                            enhancement_hash: None,
                            invocation_receipt: Some(receipt),
                            recovery_code: Some(error_code),
                        })
                    }
                    Err(_) => {
                        let error_code = "MODEL_PORT_UNAVAILABLE".to_string();
                        let receipt = self.terminal_model_invocation(
                            &invocation_ref,
                            "FAILED",
                            None,
                            None,
                            Some(error_code.clone()),
                        )?;
                        Ok(M4SecretaryModelEnhancementOutcome {
                            status: M4SecretaryModelEnhancementStatus::Failed,
                            invocation_ref: Some(invocation_ref),
                            enhancement_ref: None,
                            enhancement_hash: None,
                            invocation_receipt: Some(receipt),
                            recovery_code: Some(error_code),
                        })
                    }
                }
            }
            M4SecretaryInvocationClaimOutcome::Replay { receipt } => {
                receipt.validate()?;
                Ok(M4SecretaryModelEnhancementOutcome {
                    status: M4SecretaryModelEnhancementStatus::Replayed,
                    invocation_ref: Some(receipt.invocation_ref.clone()),
                    enhancement_ref: receipt.result_ref.clone(),
                    enhancement_hash: receipt.result_hash.clone(),
                    invocation_receipt: Some(receipt),
                    recovery_code: None,
                })
            }
            M4SecretaryInvocationClaimOutcome::InFlight { invocation_ref } => {
                Ok(M4SecretaryModelEnhancementOutcome {
                    status: M4SecretaryModelEnhancementStatus::Pending,
                    invocation_ref: Some(invocation_ref),
                    enhancement_ref: None,
                    enhancement_hash: None,
                    invocation_receipt: None,
                    recovery_code: Some("INVOCATION_IN_FLIGHT".to_string()),
                })
            }
            M4SecretaryInvocationClaimOutcome::Rejected { error_code } => {
                Ok(M4SecretaryModelEnhancementOutcome {
                    status: M4SecretaryModelEnhancementStatus::Unavailable,
                    invocation_ref: None,
                    enhancement_ref: None,
                    enhancement_hash: None,
                    invocation_receipt: None,
                    recovery_code: Some(m4c05_scrubbed_model_error(&error_code)),
                })
            }
        }
    }

    fn terminal_model_invocation(
        &self,
        invocation_ref: &M4SecretaryOpaqueRef,
        outcome_code: &str,
        result_ref: Option<M4SecretaryOpaqueRef>,
        result_hash: Option<M4SecretaryHash>,
        error_code: Option<String>,
    ) -> Result<M4SecretaryInvocationReceipt, M4SecretaryServiceError> {
        let terminal = M4SecretaryInvocationTerminal {
            invocation_ref: invocation_ref.clone(),
            outcome_code: outcome_code.to_string(),
            result_ref,
            result_hash,
            error_code,
        };
        let receipt = self.invocation_ledger_port.terminal_invocation(&terminal)?;
        receipt.validate()?;
        if receipt.invocation_ref != *invocation_ref || receipt.outcome_code != outcome_code {
            return Err(M4SecretaryServiceError::new(
                "m4c05_invocation_terminal_receipt_mismatch",
            ));
        }
        Ok(receipt)
    }

    fn map_handoff_record(
        record: M4SecretaryHandoffPortRecord,
    ) -> Result<M4SecretaryHandoffOutcome, M4SecretaryServiceError> {
        match record {
            M4SecretaryHandoffPortRecord::Unavailable { error_code } => {
                if !m4c05_is_status_code(&error_code) {
                    return Err(M4SecretaryServiceError::new(
                        "m4c05_handoff_unavailable_code_invalid",
                    ));
                }
                Ok(M4SecretaryHandoffOutcome {
                    status: M4SecretaryHandoffStatus::Unavailable,
                    handoff_ref: None,
                    request_receipt_ref: None,
                    returned_receipt: None,
                    recovery_code: Some(error_code),
                })
            }
            M4SecretaryHandoffPortRecord::Pending {
                handoff_ref,
                request_receipt_ref,
            } => Ok(M4SecretaryHandoffOutcome {
                status: M4SecretaryHandoffStatus::Pending,
                handoff_ref: Some(handoff_ref),
                request_receipt_ref: Some(request_receipt_ref),
                returned_receipt: None,
                recovery_code: None,
            }),
            M4SecretaryHandoffPortRecord::Returned {
                handoff_ref,
                receipt,
            } => {
                receipt.validate()?;
                if receipt.handoff_ref != handoff_ref || receipt.status_code != "RETURNED" {
                    return Err(M4SecretaryServiceError::new(
                        "m4c05_handoff_returned_receipt_mismatch",
                    ));
                }
                Ok(M4SecretaryHandoffOutcome {
                    status: M4SecretaryHandoffStatus::Returned,
                    handoff_ref: Some(handoff_ref),
                    request_receipt_ref: None,
                    returned_receipt: Some(receipt),
                    recovery_code: None,
                })
            }
            M4SecretaryHandoffPortRecord::Failed {
                handoff_ref,
                receipt,
                recovery_code,
            } => {
                if !m4c05_is_status_code(&recovery_code) {
                    return Err(M4SecretaryServiceError::new(
                        "m4c05_handoff_failed_code_invalid",
                    ));
                }
                if let Some(receipt) = &receipt {
                    receipt.validate()?;
                    if receipt.handoff_ref != handoff_ref {
                        return Err(M4SecretaryServiceError::new(
                            "m4c05_handoff_failed_receipt_mismatch",
                        ));
                    }
                }
                Ok(M4SecretaryHandoffOutcome {
                    status: M4SecretaryHandoffStatus::Failed,
                    handoff_ref: Some(handoff_ref),
                    request_receipt_ref: None,
                    returned_receipt: receipt,
                    recovery_code: Some(recovery_code),
                })
            }
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
struct M4SecretaryBriefHashInput<'a> {
    context_ref: &'a M4SecretaryOpaqueRef,
    scope_source_watermark: &'a M4SecretaryHash,
    attention_items: &'a [M4SecretarySourceBackedBriefItem],
    personal_actions: &'a [M4SecretaryPersonalActionBriefItem],
}

fn validate_m4c05_snapshot(
    snapshot: &M4CoordinationSnapshot,
    expected_scope_ref: &M4SecretaryTypedRef,
) -> Result<(), M4SecretaryServiceError> {
    if snapshot.scope_ref != expected_scope_ref.as_str()
        || !m4c05_is_lower_hex_hash(&snapshot.scope_source_watermark)
    {
        return Err(M4SecretaryServiceError::new(
            "m4c05_coordination_scope_mismatch",
        ));
    }
    m4c05_validate_unique(
        snapshot
            .inbox_items
            .iter()
            .map(|item| item.inbox_item_id.as_str()),
        "m4c05_inbox_identity_ambiguous",
    )?;
    m4c05_validate_unique(
        snapshot
            .open_loops
            .iter()
            .map(|item| item.open_loop_id.as_str()),
        "m4c05_open_loop_identity_ambiguous",
    )?;
    m4c05_validate_unique(
        snapshot
            .personal_actions
            .iter()
            .map(|item| item.personal_action_id.as_str()),
        "m4c05_personal_action_identity_ambiguous",
    )?;
    for inbox in &snapshot.inbox_items {
        m4c05_validate_inbox(inbox)?;
    }
    for open_loop in &snapshot.open_loops {
        m4c05_validate_open_loop(open_loop)?;
    }
    for action in &snapshot.personal_actions {
        M4SecretaryTypedRef::new(action.personal_action_id.clone())?;
        M4SecretaryOpaqueRef::new(action.explicit_user_command_ref.clone())?;
        m4c05_validate_timestamp_opt(action.due_at_utc.as_deref())?;
        if !m4c05_is_status_code(&action.status) {
            return Err(M4SecretaryServiceError::new(
                "m4c05_personal_action_status_invalid",
            ));
        }
    }
    Ok(())
}

fn m4c05_validate_unique<'a>(
    values: impl Iterator<Item = &'a str>,
    error_code: &'static str,
) -> Result<(), M4SecretaryServiceError> {
    let mut seen = BTreeSet::new();
    for value in values {
        if !seen.insert(value) {
            return Err(M4SecretaryServiceError::new(error_code));
        }
    }
    Ok(())
}

fn m4c05_validate_inbox(item: &M4InboxItemRead) -> Result<(), M4SecretaryServiceError> {
    M4SecretaryTypedRef::new(item.inbox_item_id.clone())?;
    M4SecretaryTypedRef::new(item.source_identity_key.clone())?;
    m4c05_validate_source_backing(
        &item.source_owner_ref,
        &item.source_link,
        &item.current_source_status,
        &item.status,
        item.priority_rank,
        &item.priority_reason_code,
        &item.scrubbed_summary_ref,
        item.due_at_utc.as_deref(),
        &item.last_source_change_at_utc,
    )
}

fn m4c05_validate_open_loop(item: &M4OpenLoopRead) -> Result<(), M4SecretaryServiceError> {
    M4SecretaryTypedRef::new(item.open_loop_id.clone())?;
    M4SecretaryTypedRef::new(item.source_identity_key.clone())?;
    if !m4c05_is_status_code(&item.why_open_code) {
        return Err(M4SecretaryServiceError::new("m4c05_open_loop_why_invalid"));
    }
    m4c05_validate_source_backing(
        &item.source_owner_ref,
        &item.source_link,
        &item.current_source_status,
        &item.status,
        item.priority_rank,
        &item.priority_reason_code,
        &item.scrubbed_summary_ref,
        item.due_at_utc.as_deref(),
        &item.last_source_change_at_utc,
    )
}

#[allow(clippy::too_many_arguments)]
fn m4c05_validate_source_backing(
    source_owner_ref: &str,
    source_link: &M4SourceLinkRead,
    source_status: &str,
    status: &str,
    priority_rank: i64,
    priority_code: &str,
    summary_ref: &str,
    due_at_utc: Option<&str>,
    last_change_at_utc: &str,
) -> Result<(), M4SecretaryServiceError> {
    if source_owner_ref != source_link.source_owner_ref
        || !(0..=4).contains(&priority_rank)
        || !m4c05_is_status_code(source_status)
        || !m4c05_is_status_code(status)
        || !m4c05_is_status_code(priority_code)
    {
        return Err(M4SecretaryServiceError::new(
            "m4c05_source_backed_item_invalid",
        ));
    }
    M4SecretaryTypedRef::new(source_owner_ref.to_string())?;
    M4SecretaryTypedRef::new(source_link.object_type.clone())?;
    M4SecretaryTypedRef::new(source_link.canonical_source_object_id.clone())?;
    M4SecretaryOpaqueRef::new(source_link.opaque_route_ref.clone())?;
    M4SecretaryOpaqueRef::new(summary_ref.to_string())?;
    m4c05_validate_timestamp_opt(due_at_utc)?;
    m4c05_validate_timestamp(last_change_at_utc)?;
    Ok(())
}

fn m4c05_validate_timestamp(value: &str) -> Result<(), M4SecretaryServiceError> {
    if m4_parse_rfc3339_utc_key(value).is_none() {
        return Err(M4SecretaryServiceError::new("m4c05_timestamp_invalid"));
    }
    Ok(())
}

fn m4c05_validate_timestamp_opt(value: Option<&str>) -> Result<(), M4SecretaryServiceError> {
    if let Some(value) = value {
        m4c05_validate_timestamp(value)?;
    }
    Ok(())
}

fn m4c05_brief_item_from_inbox(
    item: &M4InboxItemRead,
) -> Result<M4SecretarySourceBackedBriefItem, M4SecretaryServiceError> {
    m4c05_build_source_backed_item(
        item.inbox_item_id.clone(),
        "INBOX_ITEM",
        item.source_owner_ref.clone(),
        &item.source_link,
        item.priority_reason_code.clone(),
        item.priority_rank,
        item.priority_reason_code.clone(),
        item.status.clone(),
        item.current_source_status.clone(),
        item.revision,
        item.due_at_utc.clone(),
        item.last_source_change_at_utc.clone(),
        item.scrubbed_summary_ref.clone(),
    )
}

fn m4c05_brief_item_from_open_loop(
    item: &M4OpenLoopRead,
) -> Result<M4SecretarySourceBackedBriefItem, M4SecretaryServiceError> {
    m4c05_build_source_backed_item(
        item.open_loop_id.clone(),
        "OPEN_LOOP",
        item.source_owner_ref.clone(),
        &item.source_link,
        item.why_open_code.clone(),
        item.priority_rank,
        item.priority_reason_code.clone(),
        item.status.clone(),
        item.current_source_status.clone(),
        item.revision,
        item.due_at_utc.clone(),
        item.last_source_change_at_utc.clone(),
        item.scrubbed_summary_ref.clone(),
    )
}

#[allow(clippy::too_many_arguments)]
fn m4c05_build_source_backed_item(
    item_id: String,
    item_kind_code: &str,
    source_owner_ref: String,
    source_link: &M4SourceLinkRead,
    why_code: String,
    priority_rank: i64,
    priority_code: String,
    status_code: String,
    source_status_code: String,
    coordination_revision: i64,
    due_at_utc: Option<String>,
    last_change_at_utc: String,
    source_summary_ref: String,
) -> Result<M4SecretarySourceBackedBriefItem, M4SecretaryServiceError> {
    let priority_rank = u8::try_from(priority_rank)
        .map_err(|_| M4SecretaryServiceError::new("m4c05_priority_rank_invalid"))?;
    let coordination_revision = m4c05_canonical_revision_from_i64(coordination_revision)?;
    Ok(M4SecretarySourceBackedBriefItem {
        item_ref: M4SecretaryTypedRef::new(item_id.clone())?,
        item_kind_code: item_kind_code.to_string(),
        source_owner_ref: M4SecretaryTypedRef::new(source_owner_ref.clone())?,
        source_object_type: source_link.object_type.clone(),
        source_object_ref: M4SecretaryTypedRef::new(
            source_link.canonical_source_object_id.clone(),
        )?,
        source_route_ref: M4SecretaryOpaqueRef::new(source_link.opaque_route_ref.clone())?,
        source_summary_ref: M4SecretaryOpaqueRef::new(source_summary_ref)?,
        why_code,
        priority_rank,
        priority_code,
        status_code,
        source_status_code,
        coordination_revision: coordination_revision.clone(),
        due_at_utc,
        last_change_at_utc: last_change_at_utc.clone(),
        change_hash: M4SecretaryHash::of_texts(
            "syn.m4.secretary.brief-change/v1",
            &[
                item_kind_code,
                &item_id,
                &source_owner_ref,
                &source_link.object_type,
                &coordination_revision,
                &last_change_at_utc,
                source_link.opaque_route_ref.as_str(),
            ],
        ),
    })
}

fn m4c05_brief_personal_action(
    action: &crate::m4_secretary_read_model::M4PersonalActionRead,
) -> Result<M4SecretaryPersonalActionBriefItem, M4SecretaryServiceError> {
    Ok(M4SecretaryPersonalActionBriefItem {
        personal_action_ref: M4SecretaryTypedRef::new(action.personal_action_id.clone())?,
        explicit_user_command_ref: M4SecretaryOpaqueRef::new(
            action.explicit_user_command_ref.clone(),
        )?,
        status_code: action.status.clone(),
        due_at_utc: action.due_at_utc.clone(),
        coordination_revision: m4c05_canonical_revision(&action.revision)?,
        revision_hash: M4SecretaryHash::of_texts(
            "syn.m4.secretary.personal-action-revision/v1",
            &[&action.personal_action_id, &action.revision],
        ),
    })
}

fn m4c05_canonical_revision(value: &str) -> Result<String, M4SecretaryServiceError> {
    if value.is_empty()
        || (value.len() > 1 && value.starts_with('0'))
        || !value.bytes().all(|byte| byte.is_ascii_digit())
        || value.parse::<u64>().is_err()
    {
        return Err(M4SecretaryServiceError::new(
            "m4c05_coordination_revision_invalid",
        ));
    }
    Ok(value.to_string())
}

fn m4c05_canonical_revision_from_i64(value: i64) -> Result<String, M4SecretaryServiceError> {
    u64::try_from(value)
        .map(|value| value.to_string())
        .map_err(|_| M4SecretaryServiceError::new("m4c05_coordination_revision_invalid"))
}

fn m4c05_compare_brief_items(
    left: &M4SecretarySourceBackedBriefItem,
    right: &M4SecretarySourceBackedBriefItem,
) -> Ordering {
    left.priority_rank
        .cmp(&right.priority_rank)
        .then_with(|| match (&left.due_at_utc, &right.due_at_utc) {
            (Some(left_due), Some(right_due)) => m4_parse_rfc3339_utc_key(left_due)
                .expect("validated M4C05 due timestamp")
                .cmp(&m4_parse_rfc3339_utc_key(right_due).expect("validated M4C05 due timestamp")),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        })
        .then_with(|| {
            m4_parse_rfc3339_utc_key(&right.last_change_at_utc)
                .expect("validated M4C05 change timestamp")
                .cmp(
                    &m4_parse_rfc3339_utc_key(&left.last_change_at_utc)
                        .expect("validated M4C05 change timestamp"),
                )
        })
        .then_with(|| {
            left.source_owner_ref
                .as_str()
                .cmp(right.source_owner_ref.as_str())
        })
        .then_with(|| {
            left.source_object_ref
                .as_str()
                .cmp(right.source_object_ref.as_str())
        })
        .then_with(|| left.item_ref.as_str().cmp(right.item_ref.as_str()))
}

fn m4c05_scrubbed_model_error(value: &str) -> String {
    if m4c05_is_status_code(value) {
        value.to_string()
    } else {
        "MODEL_ENHANCEMENT_FAILED".to_string()
    }
}

fn m4c05_is_status_code(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || matches!(byte, b'_'))
}

fn m4c05_is_lower_hex_hash(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn m4c05_is_opaque_reference(value: &str) -> bool {
    let Some((namespace, digest)) = value.rsplit_once(":sha256:") else {
        return false;
    };
    !namespace.is_empty() && m4c05_is_safe_reference(namespace) && m4c05_is_lower_hex_hash(digest)
}

fn m4c05_is_safe_reference(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    !value.is_empty()
        && value.len() <= 512
        && value.trim() == value
        && !value.chars().any(char::is_control)
        && !value.contains('\\')
        && !value.starts_with('/')
        && !value.starts_with("./")
        && !value.starts_with("../")
        && !value.contains("/./")
        && !value.contains("/../")
        && !lower.contains("://")
        && !value.contains('@')
        && ![
            "password",
            "credential",
            "api_key",
            "apikey",
            "access_token",
            "refresh_token",
            "bearer ",
            "token=",
        ]
        .iter()
        .any(|marker| lower.contains(marker))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m4_secretary_read_model::{
        M4InboxItemRead, M4NotificationRead, M4OpenLoopRead, M4OwnerWritebackReceiptRead,
        M4PersonalActionRead, M4ReminderRead, M4SourceLinkRead,
    };
    use std::cell::{Cell, RefCell};
    use std::collections::BTreeMap;

    fn hash(tag: &str) -> M4SecretaryHash {
        M4SecretaryHash::of_texts("syn.m4c05.test-hash/v1", &[tag])
    }

    fn opaque(namespace: &str, tag: &str) -> M4SecretaryOpaqueRef {
        M4SecretaryOpaqueRef::new(format!("{namespace}:sha256:{}", hash(tag).as_str()))
            .expect("fixture opaque ref")
    }

    fn typed(value: &str) -> M4SecretaryTypedRef {
        M4SecretaryTypedRef::new(value).expect("fixture typed ref")
    }

    fn role_session_state() -> M4SecretaryRoleSessionState {
        M4SecretaryRoleSessionState {
            role_session_ref: opaque("role-session", "primary"),
            role_ref: typed(M4C05_SECRETARY_ROLE_REF),
            scope_ref: typed(M4C05_PERSONAL_SCOPE_REF),
            current_object_ref: typed(M4C05_PERSONAL_OBJECT_REF),
            execution_channel_code: M4C05_DAILY_CHANNEL_CODE.to_string(),
            session_state_code: M4C05_ACTIVE_SESSION_CODE.to_string(),
            permission_snapshot_ref: opaque("permission", "primary"),
            owner_fingerprint: hash("primary-owner"),
        }
    }

    fn source_link(object_id: &str, revision: u64) -> M4SourceLinkRead {
        M4SourceLinkRead {
            link_kind: "INTERNAL_ROUTE".to_string(),
            source_owner_ref: "workflow-owner".to_string(),
            object_type: "workflow_attention".to_string(),
            canonical_source_object_id: object_id.to_string(),
            expected_source_revision: revision,
            opaque_route_ref: opaque("route", object_id).as_str().to_string(),
        }
    }

    fn priority(rank: i64) -> (&'static str, &'static str) {
        match rank {
            0 => ("EXTERNAL_COMMITMENT_OR_TIME_CRITICAL", "外部承诺或时间紧迫"),
            1 => ("USER_DECISION_OR_BLOCKER", "需要你决定或来源已受阻"),
            2 => ("ACTIVE_CHANGED_ATTENTION", "当前需要关注或刚有重要变化"),
            3 => ("CARRIED_OVER", "此前未闭环，继续关注"),
            _ => ("INFORMATIONAL", "来源信息，当前无需行动"),
        }
    }

    fn inbox_item(
        tag: &str,
        rank: i64,
        due_at_utc: Option<&str>,
        last_change_at_utc: &str,
    ) -> M4InboxItemRead {
        let (priority_reason_code, priority_reason_text) = priority(rank);
        M4InboxItemRead {
            inbox_item_id: format!("inbox:{}", hash(tag).as_str()),
            source_identity_key: format!("source:{}", hash(&format!("source-{tag}")).as_str()),
            source_owner_ref: "workflow-owner".to_string(),
            source_link: source_link(&format!("work-item-{tag}"), 3),
            current_source_status: "OPEN".to_string(),
            status: "NEW".to_string(),
            priority_rank: rank,
            priority_reason_code: priority_reason_code.to_string(),
            priority_reason_text: priority_reason_text.to_string(),
            due_at_utc: due_at_utc.map(str::to_string),
            received_at_utc: last_change_at_utc.to_string(),
            last_source_change_at_utc: last_change_at_utc.to_string(),
            scrubbed_summary_ref: opaque("summary", tag).as_str().to_string(),
            sensitivity: "SCRUBBED_INTERNAL_REF_ONLY".to_string(),
            revision: 3,
        }
    }

    fn open_loop_item(
        tag: &str,
        rank: i64,
        due_at_utc: Option<&str>,
        last_change_at_utc: &str,
    ) -> M4OpenLoopRead {
        let (priority_reason_code, priority_reason_text) = priority(rank);
        M4OpenLoopRead {
            open_loop_id: format!("open-loop:{}", hash(tag).as_str()),
            source_identity_key: format!("source:{}", hash(&format!("source-{tag}")).as_str()),
            source_owner_ref: "workflow-owner".to_string(),
            source_link: source_link(&format!("work-item-{tag}"), 4),
            current_source_status: "BLOCKED".to_string(),
            status: "OPEN".to_string(),
            why_open_code: "SOURCE_ATTENTION".to_string(),
            priority_rank: rank,
            priority_reason_code: priority_reason_code.to_string(),
            priority_reason_text: priority_reason_text.to_string(),
            due_at_utc: due_at_utc.map(str::to_string),
            snoozed_until_utc: None,
            closure_reason_code: None,
            last_source_change_at_utc: last_change_at_utc.to_string(),
            scrubbed_summary_ref: opaque("summary", tag).as_str().to_string(),
            sensitivity: "SCRUBBED_INTERNAL_REF_ONLY".to_string(),
            revision: 4,
        }
    }

    fn personal_action() -> M4PersonalActionRead {
        M4PersonalActionRead {
            personal_action_id: format!("personal-action:{}", hash("personal-action").as_str()),
            explicit_user_command_ref: opaque("command", "personal-action").as_str().to_string(),
            title: "USER_TEXT_SHOULD_NOT_SURFACE".to_string(),
            status: "OPEN".to_string(),
            due_at_utc: Some("2026-08-10T12:00:00Z".to_string()),
            revision: "1".to_string(),
        }
    }

    fn coordination_snapshot() -> M4CoordinationSnapshot {
        M4CoordinationSnapshot {
            scope_ref: M4C05_PERSONAL_SCOPE_REF.to_string(),
            scope_source_watermark: hash("scope-watermark").as_str().to_string(),
            inbox_items: vec![inbox_item(
                "inbox-low",
                2,
                Some("2026-08-10T15:00:00Z"),
                "2026-08-10T08:00:00Z",
            )],
            open_loops: vec![open_loop_item(
                "loop-high",
                0,
                Some("2026-08-10T10:00:00Z"),
                "2026-08-10T09:00:00Z",
            )],
            personal_actions: vec![personal_action()],
            notifications: Vec::<M4NotificationRead>::new(),
            reminders: Vec::<M4ReminderRead>::new(),
            decisions: Vec::new(),
            owner_writeback_receipts: Vec::<M4OwnerWritebackReceiptRead>::new(),
        }
    }

    struct FakeRoleSessionPort {
        state: M4SecretaryRoleSessionState,
        reads: Cell<usize>,
    }

    impl FakeRoleSessionPort {
        fn new() -> Self {
            Self {
                state: role_session_state(),
                reads: Cell::new(0),
            }
        }
    }

    impl M4SecretaryRoleSessionReadPort for FakeRoleSessionPort {
        fn read_personal_secretary_role_session(
            &self,
        ) -> Result<M4SecretaryRoleSessionState, M4SecretaryServiceError> {
            self.reads.set(self.reads.get() + 1);
            Ok(self.state.clone())
        }
    }

    struct FakeSnapshotPort {
        snapshot: RefCell<M4CoordinationSnapshot>,
        reads: Cell<usize>,
    }

    impl FakeSnapshotPort {
        fn new(snapshot: M4CoordinationSnapshot) -> Self {
            Self {
                snapshot: RefCell::new(snapshot),
                reads: Cell::new(0),
            }
        }
    }

    impl M4SecretaryCoordinationSnapshotReadPort for FakeSnapshotPort {
        fn read_coordination_snapshot(
            &self,
            scope_ref: &M4SecretaryTypedRef,
        ) -> Result<M4CoordinationSnapshot, M4SecretaryServiceError> {
            self.reads.set(self.reads.get() + 1);
            if scope_ref.as_str() != M4C05_PERSONAL_SCOPE_REF {
                return Err(M4SecretaryServiceError::new("M4C05_TEST_SCOPE_MISMATCH"));
            }
            Ok(self.snapshot.borrow().clone())
        }
    }

    #[derive(Clone)]
    enum FakeHandoffPhase {
        Unavailable,
        Pending {
            handoff_ref: M4SecretaryOpaqueRef,
            request_receipt_ref: M4SecretaryOpaqueRef,
        },
        Returned {
            handoff_ref: M4SecretaryOpaqueRef,
            receipt: M4SecretaryHandoffReceipt,
        },
    }

    struct FakeHandoffPort {
        phase: RefCell<FakeHandoffPhase>,
        creates: Cell<usize>,
        reads: Cell<usize>,
        source_owner_apply_attempts: Cell<usize>,
    }

    impl FakeHandoffPort {
        fn new(phase: FakeHandoffPhase) -> Self {
            Self {
                phase: RefCell::new(phase),
                creates: Cell::new(0),
                reads: Cell::new(0),
                source_owner_apply_attempts: Cell::new(0),
            }
        }

        fn set_phase(&self, phase: FakeHandoffPhase) {
            *self.phase.borrow_mut() = phase;
        }

        fn current_record(&self) -> M4SecretaryHandoffPortRecord {
            match self.phase.borrow().clone() {
                FakeHandoffPhase::Unavailable => M4SecretaryHandoffPortRecord::Unavailable {
                    error_code: "M3_BINDING_UNAVAILABLE".to_string(),
                },
                FakeHandoffPhase::Pending {
                    handoff_ref,
                    request_receipt_ref,
                } => M4SecretaryHandoffPortRecord::Pending {
                    handoff_ref,
                    request_receipt_ref,
                },
                FakeHandoffPhase::Returned {
                    handoff_ref,
                    receipt,
                } => M4SecretaryHandoffPortRecord::Returned {
                    handoff_ref,
                    receipt,
                },
            }
        }
    }

    impl M4SecretaryHandoffPort for FakeHandoffPort {
        fn create_handoff(
            &self,
            _request: &M4SecretaryHandoffRequest,
        ) -> Result<M4SecretaryHandoffPortRecord, M4SecretaryServiceError> {
            self.creates.set(self.creates.get() + 1);
            Ok(self.current_record())
        }

        fn read_handoff_receipt(
            &self,
            _handoff_ref: &M4SecretaryOpaqueRef,
        ) -> Result<M4SecretaryHandoffPortRecord, M4SecretaryServiceError> {
            self.reads.set(self.reads.get() + 1);
            Ok(self.current_record())
        }
    }

    struct FakeInvocationLedger {
        claims: Cell<usize>,
        terminals: Cell<usize>,
        immutable_by_idempotency: RefCell<BTreeMap<String, M4SecretaryHash>>,
        invocation_by_key: RefCell<BTreeMap<String, M4SecretaryOpaqueRef>>,
        receipt_by_invocation: RefCell<BTreeMap<String, M4SecretaryInvocationReceipt>>,
    }

    impl FakeInvocationLedger {
        fn new() -> Self {
            Self {
                claims: Cell::new(0),
                terminals: Cell::new(0),
                immutable_by_idempotency: RefCell::new(BTreeMap::new()),
                invocation_by_key: RefCell::new(BTreeMap::new()),
                receipt_by_invocation: RefCell::new(BTreeMap::new()),
            }
        }
    }

    impl M4SecretaryModelInvocationLedgerPort for FakeInvocationLedger {
        fn claim_invocation(
            &self,
            claim: &M4SecretaryModelInvocationClaim,
        ) -> Result<M4SecretaryInvocationClaimOutcome, M4SecretaryServiceError> {
            self.claims.set(self.claims.get() + 1);
            let idempotency_key = claim.idempotency_key_ref.as_str().to_string();
            let prior_immutable = self
                .immutable_by_idempotency
                .borrow()
                .get(&idempotency_key)
                .cloned();
            if let Some(prior_immutable) = prior_immutable {
                if prior_immutable != claim.immutable_input_hash {
                    return Ok(M4SecretaryInvocationClaimOutcome::Rejected {
                        error_code: "IDEMPOTENCY_KEY_REUSE".to_string(),
                    });
                }
            }

            let invocation_key = claim.invocation_key_ref.as_str().to_string();
            if let Some(invocation_ref) = self
                .invocation_by_key
                .borrow()
                .get(&invocation_key)
                .cloned()
            {
                if let Some(receipt) = self
                    .receipt_by_invocation
                    .borrow()
                    .get(invocation_ref.as_str())
                    .cloned()
                {
                    return Ok(M4SecretaryInvocationClaimOutcome::Replay { receipt });
                }
                return Ok(M4SecretaryInvocationClaimOutcome::InFlight { invocation_ref });
            }

            self.immutable_by_idempotency
                .borrow_mut()
                .insert(idempotency_key, claim.immutable_input_hash.clone());
            let invocation_ref = opaque("invocation", &invocation_key);
            self.invocation_by_key
                .borrow_mut()
                .insert(invocation_key, invocation_ref.clone());
            Ok(M4SecretaryInvocationClaimOutcome::DispatchGranted { invocation_ref })
        }

        fn terminal_invocation(
            &self,
            terminal: &M4SecretaryInvocationTerminal,
        ) -> Result<M4SecretaryInvocationReceipt, M4SecretaryServiceError> {
            self.terminals.set(self.terminals.get() + 1);
            let receipt = M4SecretaryInvocationReceipt {
                invocation_ref: terminal.invocation_ref.clone(),
                terminal_receipt_ref: opaque(
                    "invocation-receipt",
                    &format!(
                        "{}:{}",
                        terminal.invocation_ref.as_str(),
                        terminal.outcome_code
                    ),
                ),
                outcome_code: terminal.outcome_code.clone(),
                result_ref: terminal.result_ref.clone(),
                result_hash: terminal.result_hash.clone(),
                error_code: terminal.error_code.clone(),
            };
            self.receipt_by_invocation.borrow_mut().insert(
                terminal.invocation_ref.as_str().to_string(),
                receipt.clone(),
            );
            Ok(receipt)
        }
    }

    struct FakeModelPort {
        calls: Cell<usize>,
        outcome: RefCell<M4SecretaryModelPortOutcome>,
    }

    impl FakeModelPort {
        fn successful() -> Self {
            Self {
                calls: Cell::new(0),
                outcome: RefCell::new(M4SecretaryModelPortOutcome::Enhanced {
                    enhancement_ref: opaque("enhancement", "success"),
                    result_hash: hash("enhancement-success"),
                }),
            }
        }

        fn failing() -> Self {
            Self {
                calls: Cell::new(0),
                outcome: RefCell::new(M4SecretaryModelPortOutcome::Failed {
                    error_code: "MODEL_PROVIDER_UNAVAILABLE".to_string(),
                }),
            }
        }
    }

    impl M4SecretaryControlledModelEnhancementPort for FakeModelPort {
        fn enhance(
            &self,
            _request: &M4SecretaryModelEnhancementRequest,
        ) -> Result<M4SecretaryModelPortOutcome, M4SecretaryServiceError> {
            self.calls.set(self.calls.get() + 1);
            Ok(self.outcome.borrow().clone())
        }
    }

    fn service<'a>(
        role: &'a FakeRoleSessionPort,
        snapshot: &'a FakeSnapshotPort,
        handoff: &'a FakeHandoffPort,
        ledger: &'a FakeInvocationLedger,
        model: &'a FakeModelPort,
    ) -> M4SecretaryApplicationService<
        'a,
        FakeRoleSessionPort,
        FakeSnapshotPort,
        FakeHandoffPort,
        FakeInvocationLedger,
        FakeModelPort,
    > {
        M4SecretaryApplicationService::new(role, snapshot, handoff, ledger, model)
    }

    fn handoff_request() -> M4SecretaryHandoffRequest {
        M4SecretaryHandoffRequest {
            request_ref: opaque("handoff-request", "request"),
            from_role_session_ref: role_session_state().role_session_ref,
            scope_ref: typed(M4C05_PERSONAL_SCOPE_REF),
            to_role_ref: typed("role:global-supervisor"),
            to_recipient_ref: opaque("recipient", "global-supervisor"),
            requested_outcome_ref: opaque("outcome", "review"),
            object_refs: vec![typed("open-loop:bounded-object")],
            risk_class_code: "LOW".to_string(),
            reason_ref: opaque("reason", "handoff"),
            permission_request_ref: opaque("permission-request", "handoff"),
            correlation_ref: opaque("correlation", "handoff"),
        }
    }

    fn explicit_user_trigger() -> M4SecretaryExplicitUserMessageTrigger {
        M4SecretaryExplicitUserMessageTrigger {
            trigger_ref: opaque("event", "explicit-user"),
            user_message_ref: opaque("message", "explicit-user"),
            user_message_hash: hash("explicit-user-message"),
            idempotency_key_ref: opaque("idempotency", "explicit-user"),
            purpose_code: "EXPLAIN_EXISTING_REASON".to_string(),
        }
    }

    fn handoff_return_receipt(handoff_ref: &M4SecretaryOpaqueRef) -> M4SecretaryHandoffReceipt {
        M4SecretaryHandoffReceipt {
            receipt_ref: opaque("handoff-receipt", "returned"),
            handoff_ref: handoff_ref.clone(),
            receipt_kind_code: "RETURN_RESULT".to_string(),
            status_code: "RETURNED".to_string(),
            result_ref: Some(opaque("result", "returned")),
            result_hash: Some(hash("returned-result")),
        }
    }

    #[test]
    fn m4c06_full_brief_serialization_retains_deep_link_and_canonical_revisions() {
        let role = FakeRoleSessionPort::new();
        let snapshot = FakeSnapshotPort::new(coordination_snapshot());
        let handoff = FakeHandoffPort::new(FakeHandoffPhase::Unavailable);
        let ledger = FakeInvocationLedger::new();
        let model = FakeModelPort::successful();

        let outcome = service(&role, &snapshot, &handoff, &ledger, &model)
            .read_deterministic_brief()
            .expect("build complete typed brief");
        let value = serde_json::to_value(&outcome).expect("serialize full brief");
        let attention_items = value["deterministic_brief"]["attention_items"]
            .as_array()
            .expect("serialized attention items");
        assert_eq!(attention_items.len(), 2);
        for item in attention_items {
            assert_eq!(item["source_object_type"], "workflow_attention");
            assert!(
                item["coordination_revision"].is_string(),
                "revision must remain a canonical decimal string"
            );
        }
        assert_eq!(
            value["deterministic_brief"]["personal_actions"][0]["coordination_revision"],
            "1"
        );
    }

    #[test]
    fn m4c05_deterministic_brief_and_context_rebuild_identically() {
        let role = FakeRoleSessionPort::new();
        let snapshot = FakeSnapshotPort::new(coordination_snapshot());
        let handoff = FakeHandoffPort::new(FakeHandoffPhase::Unavailable);
        let ledger = FakeInvocationLedger::new();
        let model = FakeModelPort::successful();

        let first = service(&role, &snapshot, &handoff, &ledger, &model)
            .read_deterministic_brief()
            .expect("first mechanical rebuild");
        let restarted = service(&role, &snapshot, &handoff, &ledger, &model)
            .read_deterministic_brief()
            .expect("same immutable inputs after service reconstruction");

        assert_eq!(first, restarted);
        assert_eq!(
            first.deterministic_brief.attention_items[0].item_kind_code,
            "OPEN_LOOP"
        );
        assert_eq!(first.deterministic_brief.attention_items.len(), 2);
        assert_eq!(first.deterministic_brief.personal_actions.len(), 1);
        assert!(first
            .deterministic_brief
            .attention_items
            .iter()
            .all(|item| item.item_kind_code != "PERSONAL_ACTION"));
        assert_eq!(
            first.deterministic_brief.personal_actions[0]
                .personal_action_ref
                .as_str(),
            personal_action().personal_action_id
        );

        let mut rotated_state = role_session_state();
        rotated_state.permission_snapshot_ref = opaque("permission", "rotated");
        let rotated_role = FakeRoleSessionPort {
            state: rotated_state,
            reads: Cell::new(0),
        };
        let rotated = service(&rotated_role, &snapshot, &handoff, &ledger, &model)
            .read_deterministic_brief()
            .expect("permission snapshot rotation rebuild");
        assert_ne!(first.context.context_ref, rotated.context.context_ref);
    }

    #[test]
    fn m4c05_read_only_and_non_user_triggers_are_zero_write_zero_model() {
        let role = FakeRoleSessionPort::new();
        let snapshot = FakeSnapshotPort::new(coordination_snapshot());
        let handoff = FakeHandoffPort::new(FakeHandoffPhase::Unavailable);
        let ledger = FakeInvocationLedger::new();
        let model = FakeModelPort::successful();
        let service = service(&role, &snapshot, &handoff, &ledger, &model);

        for trigger in [
            M4SecretaryServiceTrigger::ReadOnlyQuery,
            M4SecretaryServiceTrigger::TimerTick {
                tick_ref: opaque("timer", "tick"),
            },
            M4SecretaryServiceTrigger::CoordinationCommand {
                command_ref: opaque("command", "coordination"),
            },
            M4SecretaryServiceTrigger::StartupRecovery {
                recovery_ref: opaque("recovery", "startup"),
            },
        ] {
            let outcome = service
                .process(&trigger)
                .expect("mechanical non-user result");
            assert!(outcome.model_enhancement.is_none());
        }

        assert_eq!(handoff.creates.get(), 0);
        assert_eq!(handoff.reads.get(), 0);
        assert_eq!(ledger.claims.get(), 0);
        assert_eq!(ledger.terminals.get(), 0);
        assert_eq!(model.calls.get(), 0);
    }

    #[test]
    fn m4c05_handoff_unavailable_pending_returned_survives_rebuild_and_never_applies_source() {
        let role = FakeRoleSessionPort::new();
        let snapshot = FakeSnapshotPort::new(coordination_snapshot());
        let handoff = FakeHandoffPort::new(FakeHandoffPhase::Unavailable);
        let ledger = FakeInvocationLedger::new();
        let model = FakeModelPort::successful();

        let unavailable = service(&role, &snapshot, &handoff, &ledger, &model)
            .request_handoff(&handoff_request())
            .expect("unavailable handoff maps visibly");
        assert_eq!(unavailable.status, M4SecretaryHandoffStatus::Unavailable);
        assert_eq!(
            unavailable.recovery_code.as_deref(),
            Some("M3_BINDING_UNAVAILABLE")
        );

        let handoff_ref = opaque("handoff", "pending");
        handoff.set_phase(FakeHandoffPhase::Pending {
            handoff_ref: handoff_ref.clone(),
            request_receipt_ref: opaque("handoff-request-receipt", "pending"),
        });
        let pending = service(&role, &snapshot, &handoff, &ledger, &model)
            .request_handoff(&handoff_request())
            .expect("pending handoff persists through a new service instance");
        assert_eq!(pending.status, M4SecretaryHandoffStatus::Pending);
        assert_eq!(pending.handoff_ref, Some(handoff_ref.clone()));

        handoff.set_phase(FakeHandoffPhase::Returned {
            handoff_ref: handoff_ref.clone(),
            receipt: handoff_return_receipt(&handoff_ref),
        });
        let returned = service(&role, &snapshot, &handoff, &ledger, &model)
            .read_handoff_receipt(&handoff_ref)
            .expect("returned receipt reloads through a new service instance");
        assert_eq!(returned.status, M4SecretaryHandoffStatus::Returned);
        assert_eq!(
            returned
                .returned_receipt
                .as_ref()
                .and_then(|receipt| receipt.result_ref.as_ref())
                .map(M4SecretaryOpaqueRef::as_str),
            Some(opaque("result", "returned").as_str())
        );
        assert_eq!(handoff.source_owner_apply_attempts.get(), 0);

        let returned_json = serde_json::to_value(&returned).expect("serialize returned mapping");
        let returned_fields = returned_json
            .as_object()
            .expect("returned mapping is an object");
        for forbidden in [
            "source_owner",
            "source_status",
            "source_fact",
            "apply_source",
            "callback",
            "payload",
        ] {
            assert!(
                returned_fields.get(forbidden).is_none(),
                "returned M3 receipt must not expose or apply {forbidden}"
            );
        }
    }

    #[test]
    fn m4c05_explicit_user_claims_terminal_and_exact_replay_never_calls_model_twice() {
        let role = FakeRoleSessionPort::new();
        let snapshot = FakeSnapshotPort::new(coordination_snapshot());
        let handoff = FakeHandoffPort::new(FakeHandoffPhase::Unavailable);
        let ledger = FakeInvocationLedger::new();
        let model = FakeModelPort::successful();
        let trigger = M4SecretaryServiceTrigger::ExplicitUserMessage(explicit_user_trigger());

        let first = service(&role, &snapshot, &handoff, &ledger, &model)
            .process(&trigger)
            .expect("explicit user message claims and terminalizes invocation");
        assert_eq!(
            first
                .model_enhancement
                .as_ref()
                .expect("explicit message has a model outcome")
                .status,
            M4SecretaryModelEnhancementStatus::Available
        );
        assert_eq!(model.calls.get(), 1);
        assert_eq!(ledger.claims.get(), 1);
        assert_eq!(ledger.terminals.get(), 1);

        let replay = service(&role, &snapshot, &handoff, &ledger, &model)
            .process(&trigger)
            .expect("same durable exact claim replays");
        assert_eq!(
            replay
                .model_enhancement
                .as_ref()
                .expect("replay returns stored model outcome")
                .status,
            M4SecretaryModelEnhancementStatus::Replayed
        );
        assert_eq!(model.calls.get(), 1);
        assert_eq!(ledger.claims.get(), 2);
        assert_eq!(ledger.terminals.get(), 1);
    }

    #[test]
    fn m4c05_model_failure_keeps_deterministic_brief_available() {
        let role = FakeRoleSessionPort::new();
        let snapshot = FakeSnapshotPort::new(coordination_snapshot());
        let handoff = FakeHandoffPort::new(FakeHandoffPhase::Unavailable);
        let ledger = FakeInvocationLedger::new();
        let model = FakeModelPort::failing();

        let deterministic = service(&role, &snapshot, &handoff, &ledger, &model)
            .read_deterministic_brief()
            .expect("brief does not need a model");
        let with_failure = service(&role, &snapshot, &handoff, &ledger, &model)
            .process(&M4SecretaryServiceTrigger::ExplicitUserMessage(
                explicit_user_trigger(),
            ))
            .expect("model failure is terminalized without rolling back brief");
        let enhancement = with_failure
            .model_enhancement
            .as_ref()
            .expect("failed explicit invocation remains visible");

        assert_eq!(
            with_failure.deterministic_brief,
            deterministic.deterministic_brief
        );
        assert_eq!(
            enhancement.status,
            M4SecretaryModelEnhancementStatus::Failed
        );
        assert_eq!(
            enhancement.recovery_code.as_deref(),
            Some("MODEL_PROVIDER_UNAVAILABLE")
        );
        assert_eq!(model.calls.get(), 1);
        assert_eq!(ledger.terminals.get(), 1);
        assert_eq!(
            enhancement
                .invocation_receipt
                .as_ref()
                .map(|receipt| receipt.outcome_code.as_str()),
            Some("FAILED")
        );
    }

    #[test]
    fn m4c05_model_facing_dtos_and_receipts_exclude_raw_sensitive_content() {
        let role = FakeRoleSessionPort::new();
        let snapshot = FakeSnapshotPort::new(coordination_snapshot());
        let handoff_ref = opaque("handoff", "serialized-return");
        let handoff = FakeHandoffPort::new(FakeHandoffPhase::Returned {
            handoff_ref: handoff_ref.clone(),
            receipt: handoff_return_receipt(&handoff_ref),
        });
        let ledger = FakeInvocationLedger::new();
        let model = FakeModelPort::successful();

        let outcome = service(&role, &snapshot, &handoff, &ledger, &model)
            .process(&M4SecretaryServiceTrigger::ExplicitUserMessage(
                explicit_user_trigger(),
            ))
            .expect("serialize model outcome");
        let handoff_outcome = service(&role, &snapshot, &handoff, &ledger, &model)
            .read_handoff_receipt(&handoff_ref)
            .expect("serialize handoff outcome");
        let model_facing_serialized = format!(
            "{}{}{}{}",
            serde_json::to_string(&outcome.context).expect("serialize context DTO"),
            serde_json::to_string(&outcome.deterministic_brief)
                .expect("serialize deterministic brief DTO"),
            serde_json::to_string(&outcome.model_enhancement)
                .expect("serialize model enhancement DTO"),
            serde_json::to_string(&handoff_outcome).expect("serialize handoff DTO")
        );

        for raw in [
            "USER_TEXT_SHOULD_NOT_SURFACE",
            "https://raw.example/secret",
            "/private/secret",
            "RAW_TOKEN",
            "RAW_PROMPT",
            "RAW_RESPONSE",
            "RAW_TOOL_OUTPUT",
        ] {
            assert!(
                !model_facing_serialized.contains(raw),
                "model-facing M4C05 DTO leaked raw content marker {raw}"
            );
        }
        assert_eq!(
            outcome.local_objects.personal_actions[0].title, "USER_TEXT_SHOULD_NOT_SURFACE",
            "the renderer-only local object retains the user-authored title"
        );

        let receipt = outcome
            .model_enhancement
            .as_ref()
            .and_then(|enhancement| enhancement.invocation_receipt.as_ref())
            .expect("successful model invocation has a durable receipt");
        let receipt_fields = serde_json::to_value(receipt)
            .expect("serialize durable invocation receipt")
            .as_object()
            .expect("invocation receipt is an object")
            .clone();
        for forbidden in [
            "raw_prompt",
            "raw_response",
            "user_text",
            "tool_output",
            "provider_body",
            "credential",
            "path",
            "url",
            "token",
        ] {
            assert!(
                receipt_fields.get(forbidden).is_none(),
                "persistent receipt must not contain {forbidden}"
            );
        }
    }
}
