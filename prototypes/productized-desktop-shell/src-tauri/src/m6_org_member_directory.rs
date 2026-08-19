//! M6D05 stable organization-member directory.
//!
//! Stable identity is created only by an explicit identity contract.  Provider,
//! model, thread, process, session-count, display-name, and runtime-child clues
//! can only produce a scrubbed quarantine record.  Capability/permission and
//! availability values are observations; this module owns no authorization
//! decision and its contact path can create only an M3-owned Handoff.

use crate::m6_org_consult_handoff::M6OrgMemberContactHandoff;
use crate::m6_org_schema::ensure_m6_org_schema;
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

const MEMBER_ID_PREFIX: &str = "member_";
const IDENTITY_CONTRACT_KIND: &str = "syn.m6.org.stable-member-identity/v1";
const DIRECTORY_EXPORT_SCHEMA: &str = "syn.m6.org.member-directory.export/v1";
const MAX_REF_LEN: usize = 512;
const MAX_TTL_SECONDS: u64 = 31 * 24 * 60 * 60;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum M6OrgMembershipLifecycle {
    Established,
    Active,
    Deactivated,
    Quarantined,
}

impl M6OrgMembershipLifecycle {
    fn as_str(self) -> &'static str {
        match self {
            Self::Established => "ESTABLISHED",
            Self::Active => "ACTIVE",
            Self::Deactivated => "DEACTIVATED",
            Self::Quarantined => "QUARANTINED",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum M6OrgCapabilityPermissionKind {
    Capability,
    Permission,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum M6OrgAvailabilityState {
    Available,
    Busy,
    Offline,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct M6OrgScopeAssignment {
    pub(crate) assignment_id: String,
    pub(crate) member_id: String,
    pub(crate) scope_ref: String,
    pub(crate) assigned_by_actor_id: String,
    pub(crate) revision: u64,
    pub(crate) assigned_at: i64,
    pub(crate) revoked_at: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct M6OrgRoleAssignment {
    pub(crate) assignment_id: String,
    pub(crate) member_id: String,
    pub(crate) role_ref: String,
    pub(crate) scope_ref: String,
    pub(crate) assigned_by_actor_id: String,
    pub(crate) revision: u64,
    pub(crate) assigned_at: i64,
    pub(crate) revoked_at: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct M6OrgCapabilityPermissionRef {
    pub(crate) ref_id: String,
    pub(crate) subject_member_id: String,
    pub(crate) kind: M6OrgCapabilityPermissionKind,
    pub(crate) source: String,
    pub(crate) revision: u64,
    pub(crate) observed_at: i64,
    pub(crate) directory_is_authority: bool,
    pub(crate) read_only: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct M6OrgMemberContactBinding {
    pub(crate) binding_ref: String,
    pub(crate) to_role_ref: String,
    pub(crate) to_recipient_ref: String,
    pub(crate) source: String,
    pub(crate) revision: u64,
    pub(crate) observed_at: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE", deny_unknown_fields)]
pub(crate) enum M6OrgStableIdentityEvidence {
    ExplicitIdentityContract {
        contract_kind: String,
        identity_contract_ref: String,
        source_record_ref: String,
        source_revision: u64,
        observed_at: i64,
        explicit_human_command: bool,
    },
    HeuristicCandidate {
        candidate_kind: M6OrgHeuristicCandidateKind,
        source_refs: Vec<String>,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum M6OrgHeuristicCandidateKind {
    AgentCenterSession,
    ProviderThread,
    RuntimeChild,
    SessionCount,
    DisplayNameMatch,
}

impl M6OrgHeuristicCandidateKind {
    fn reason_code(self) -> &'static str {
        match self {
            Self::AgentCenterSession => "m6_org_member_agent_center_session_not_identity",
            Self::ProviderThread => "m6_org_member_provider_thread_not_identity",
            Self::RuntimeChild => "m6_org_member_runtime_child_not_identity",
            Self::SessionCount => "m6_org_member_session_count_not_identity",
            Self::DisplayNameMatch => "m6_org_member_display_name_not_identity",
        }
    }

    fn legacy_kind(self) -> &'static str {
        match self {
            Self::AgentCenterSession => "AGENT_CENTER_SESSION",
            Self::ProviderThread
            | Self::RuntimeChild
            | Self::SessionCount
            | Self::DisplayNameMatch => "UNMAPPABLE_OTHER",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct M6OrgRegisterStableMemberRequest {
    pub(crate) member_id: String,
    pub(crate) display_name_ref: String,
    pub(crate) identity_evidence: M6OrgStableIdentityEvidence,
    pub(crate) scope_assignments: Vec<M6OrgScopeAssignment>,
    pub(crate) role_assignments: Vec<M6OrgRoleAssignment>,
    pub(crate) capability_permission_refs: Vec<M6OrgCapabilityPermissionRef>,
    pub(crate) memory_refs: Vec<String>,
    pub(crate) contact_bindings: Vec<M6OrgMemberContactBinding>,
    pub(crate) idempotency_key: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct M6OrgUpdateStableMemberRequest {
    pub(crate) member_id: String,
    pub(crate) expected_revision: u64,
    pub(crate) activate: bool,
    pub(crate) display_name_ref: Option<String>,
    pub(crate) added_scope_assignments: Vec<M6OrgScopeAssignment>,
    pub(crate) added_role_assignments: Vec<M6OrgRoleAssignment>,
    pub(crate) added_capability_permission_refs: Vec<M6OrgCapabilityPermissionRef>,
    pub(crate) added_memory_refs: Vec<String>,
    pub(crate) added_contact_bindings: Vec<M6OrgMemberContactBinding>,
    pub(crate) idempotency_key: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct M6OrgDeactivateStableMemberRequest {
    pub(crate) member_id: String,
    pub(crate) expected_revision: u64,
    pub(crate) reason_ref: String,
    pub(crate) idempotency_key: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct M6OrgAvailabilityTtl {
    pub(crate) seconds: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct M6OrgObserveMemberAvailabilityRequest {
    pub(crate) member_id: String,
    pub(crate) source: String,
    pub(crate) source_revision: u64,
    pub(crate) observed_at: i64,
    pub(crate) ttl: M6OrgAvailabilityTtl,
    pub(crate) observed_state: M6OrgAvailabilityState,
    pub(crate) idempotency_key: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct M6OrgListStableMembersRequest {
    pub(crate) include_deactivated: bool,
    pub(crate) available_capability_ref: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct M6OrgContactStableMemberRequest {
    pub(crate) member_id: String,
    pub(crate) contact_binding_ref: String,
    pub(crate) reason_ref: String,
    pub(crate) source_refs: Vec<String>,
    pub(crate) accept_by_utc: String,
    pub(crate) idempotency_key: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct M6OrgStableMember {
    pub(crate) member_id: String,
    pub(crate) membership_lifecycle: M6OrgMembershipLifecycle,
    pub(crate) scope_assignments: Vec<M6OrgScopeAssignment>,
    pub(crate) role_assignments: Vec<M6OrgRoleAssignment>,
    pub(crate) capability_permission_refs: Vec<M6OrgCapabilityPermissionRef>,
    pub(crate) availability_ref: Option<String>,
    pub(crate) contact_binding_refs: Vec<String>,
    pub(crate) contact_bindings: Vec<M6OrgMemberContactBinding>,
    pub(crate) memory_refs: Vec<String>,
    pub(crate) promoted_from: Option<String>,
    pub(crate) display_name_ref: String,
    pub(crate) identity_contract_ref: String,
    pub(crate) identity_source_record_ref: String,
    pub(crate) identity_source_revision: u64,
    pub(crate) revision: u64,
    pub(crate) created_at: i64,
    pub(crate) deactivated_at: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct M6OrgLegacyQuarantineRecord {
    pub(crate) quarantine_ref: String,
    pub(crate) legacy_kind: String,
    pub(crate) reason_code: String,
    pub(crate) source_refs: Vec<String>,
    pub(crate) payload_mode: String,
    pub(crate) mapped_to: Option<String>,
    pub(crate) recorded_at: i64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum M6OrgMemberRegistrationDisposition {
    Registered,
    Quarantined,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct M6OrgStableMemberRegistrationOutcome {
    pub(crate) disposition: M6OrgMemberRegistrationDisposition,
    pub(crate) member: Option<M6OrgStableMember>,
    pub(crate) quarantine: Option<M6OrgLegacyQuarantineRecord>,
    pub(crate) replayed: bool,
    pub(crate) directory_is_authority: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct M6OrgAvailability {
    pub(crate) availability_id: String,
    pub(crate) subject_ref: String,
    pub(crate) source: String,
    pub(crate) source_revision: u64,
    pub(crate) observed_at: i64,
    pub(crate) ttl: M6OrgAvailabilityTtl,
    pub(crate) observed_state: M6OrgAvailabilityState,
    pub(crate) effective_state: M6OrgAvailabilityState,
    pub(crate) authorizes: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct M6OrgContactReceipt {
    pub(crate) contact_receipt_id: String,
    pub(crate) from_actor_id: String,
    pub(crate) to_member_id: String,
    pub(crate) handoff_id: String,
    pub(crate) handoff_receipt_ref: String,
    pub(crate) role_session_id: String,
    pub(crate) status: String,
    pub(crate) capability_granted: bool,
    pub(crate) recorded_at: i64,
    pub(crate) source_command_receipt_ref: String,
    pub(crate) contact_binding_ref: String,
    pub(crate) replayed: bool,
    pub(crate) project_writeback: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct M6OrgStableMemberView {
    pub(crate) member: M6OrgStableMember,
    pub(crate) availability: Option<M6OrgAvailability>,
    pub(crate) contact_receipts: Vec<M6OrgContactReceipt>,
    pub(crate) session_history_refs: Vec<String>,
    pub(crate) directory_is_authority: bool,
    pub(crate) permission_authority: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct M6OrgStableMemberDirectoryResponse {
    pub(crate) members: Vec<M6OrgStableMemberView>,
    pub(crate) judged_at: i64,
    pub(crate) stale_availability_used_as_capability: bool,
    pub(crate) directory_is_authority: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct M6OrgMemberDirectoryExport {
    pub(crate) schema: String,
    pub(crate) member_history: Vec<M6OrgStableMember>,
    pub(crate) availability_history: Vec<M6OrgAvailability>,
    pub(crate) contact_history: Vec<M6OrgContactReceipt>,
    pub(crate) quarantines: Vec<M6OrgLegacyQuarantineRecord>,
}

pub(crate) fn register_for_state(
    state: &crate::AppState,
    request: &M6OrgRegisterStableMemberRequest,
    now_ms: i64,
) -> Result<M6OrgStableMemberRegistrationOutcome, String> {
    register_with_promotion_for_state(state, request, None, now_ms)
}

pub(crate) fn register_promoted_for_state(
    state: &crate::AppState,
    request: &M6OrgRegisterStableMemberRequest,
    temporary_agent_id: &str,
    now_ms: i64,
) -> Result<M6OrgStableMember, String> {
    validate_temporary_agent_id(temporary_agent_id)?;
    let outcome =
        register_with_promotion_for_state(state, request, Some(temporary_agent_id), now_ms)?;
    match (outcome.disposition, outcome.member) {
        (M6OrgMemberRegistrationDisposition::Registered, Some(member))
            if member.promoted_from.as_deref() == Some(temporary_agent_id) =>
        {
            Ok(member)
        }
        _ => Err("m6_org_member_promotion_registration_rejected".to_string()),
    }
}

fn register_with_promotion_for_state(
    state: &crate::AppState,
    request: &M6OrgRegisterStableMemberRequest,
    promoted_from: Option<&str>,
    now_ms: i64,
) -> Result<M6OrgStableMemberRegistrationOutcome, String> {
    require_directory_runtime(state)?;
    validate_nonempty("idempotency_key", &request.idempotency_key)?;
    if promoted_from.is_some()
        && matches!(
            &request.identity_evidence,
            M6OrgStableIdentityEvidence::HeuristicCandidate { .. }
        )
    {
        return Err("m6_org_member_promotion_requires_explicit_identity".to_string());
    }
    let operation = if promoted_from.is_some() {
        "promote_temporary_agent_create"
    } else {
        "register_stable_member"
    };
    // Preserve the M6D05 idempotency preimage for ordinary registrations.
    // Promotion is a distinct command and binds its source TemporaryAgentId
    // into the preimage so it cannot replay as a direct registration.
    let request_hash = match promoted_from {
        Some(temporary_agent_id) => stable_hash(&json!({
            "request": request,
            "promoted_from": temporary_agent_id,
        }))?,
        None => stable_hash(request)?,
    };
    let mut store = M6OrgMemberDirectoryStore::open(&state.m6_org_store_path()?)?;
    if let Some(mut existing) = store
        .load_command_response::<M6OrgStableMemberRegistrationOutcome>(
            &request.idempotency_key,
            operation,
            &request_hash,
        )?
    {
        existing.replayed = true;
        return Ok(existing);
    }

    match &request.identity_evidence {
        M6OrgStableIdentityEvidence::HeuristicCandidate {
            candidate_kind,
            source_refs,
        } => {
            validate_refs("heuristic_source_refs", source_refs, false)?;
            let quarantine = M6OrgLegacyQuarantineRecord {
                quarantine_ref: digest_id(
                    "member-quarantine",
                    &[&request.idempotency_key, candidate_kind.reason_code()],
                ),
                legacy_kind: candidate_kind.legacy_kind().to_string(),
                reason_code: candidate_kind.reason_code().to_string(),
                source_refs: sorted_unique(source_refs.clone()),
                payload_mode: "REF_ONLY".to_string(),
                mapped_to: None,
                recorded_at: now_ms,
            };
            let outcome = M6OrgStableMemberRegistrationOutcome {
                disposition: M6OrgMemberRegistrationDisposition::Quarantined,
                member: None,
                quarantine: Some(quarantine.clone()),
                replayed: false,
                directory_is_authority: false,
            };
            store.record_quarantine(
                &request.idempotency_key,
                &request_hash,
                &quarantine,
                &outcome,
                now_ms,
            )?;
            Ok(outcome)
        }
        M6OrgStableIdentityEvidence::ExplicitIdentityContract {
            contract_kind,
            identity_contract_ref,
            source_record_ref,
            source_revision,
            observed_at,
            explicit_human_command,
        } => {
            validate_member_id(&request.member_id)?;
            if contract_kind != IDENTITY_CONTRACT_KIND
                || !explicit_human_command
                || *source_revision == 0
                || *observed_at < 0
                || *observed_at > now_ms
            {
                return Err("m6_org_member_identity_contract_invalid".to_string());
            }
            validate_ref("identity_contract_ref", identity_contract_ref)?;
            validate_ref("source_record_ref", source_record_ref)?;
            validate_ref("display_name_ref", &request.display_name_ref)?;
            validate_assignments(
                &request.member_id,
                &request.scope_assignments,
                &request.role_assignments,
                &request.capability_permission_refs,
            )?;
            validate_refs("memory_refs", &request.memory_refs, true)?;
            validate_contact_bindings(&request.contact_bindings)?;
            let contact_binding_refs = request
                .contact_bindings
                .iter()
                .map(|binding| binding.binding_ref.clone())
                .collect::<Vec<_>>();
            let member = M6OrgStableMember {
                member_id: request.member_id.clone(),
                membership_lifecycle: M6OrgMembershipLifecycle::Established,
                scope_assignments: sorted_by_id(request.scope_assignments.clone(), |value| {
                    &value.assignment_id
                }),
                role_assignments: sorted_by_id(request.role_assignments.clone(), |value| {
                    &value.assignment_id
                }),
                capability_permission_refs: sorted_by_id(
                    request.capability_permission_refs.clone(),
                    |value| &value.ref_id,
                ),
                availability_ref: None,
                contact_binding_refs: sorted_unique(contact_binding_refs),
                contact_bindings: sorted_by_id(request.contact_bindings.clone(), |value| {
                    &value.binding_ref
                }),
                memory_refs: sorted_unique(request.memory_refs.clone()),
                promoted_from: promoted_from.map(str::to_string),
                display_name_ref: request.display_name_ref.clone(),
                identity_contract_ref: identity_contract_ref.clone(),
                identity_source_record_ref: source_record_ref.clone(),
                identity_source_revision: *source_revision,
                revision: 1,
                created_at: now_ms,
                deactivated_at: None,
            };
            let outcome = M6OrgStableMemberRegistrationOutcome {
                disposition: M6OrgMemberRegistrationDisposition::Registered,
                member: Some(member.clone()),
                quarantine: None,
                replayed: false,
                directory_is_authority: false,
            };
            store.record_registration(
                &request.idempotency_key,
                operation,
                &request_hash,
                &member,
                &outcome,
                now_ms,
            )?;
            Ok(outcome)
        }
    }
}

pub(crate) fn bind_existing_promotion_for_state(
    state: &crate::AppState,
    member_id: &str,
    temporary_agent_id: &str,
    expected_revision: u64,
    idempotency_key: &str,
    now_ms: i64,
) -> Result<M6OrgStableMember, String> {
    require_directory_runtime(state)?;
    validate_member_id(member_id)?;
    validate_temporary_agent_id(temporary_agent_id)?;
    validate_nonempty("idempotency_key", idempotency_key)?;
    let request_hash = stable_hash(&json!({
        "member_id": member_id,
        "temporary_agent_id": temporary_agent_id,
        "expected_revision": expected_revision,
    }))?;
    let mut store = M6OrgMemberDirectoryStore::open(&state.m6_org_store_path()?)?;
    if let Some(existing) = store.load_command_response::<M6OrgStableMember>(
        idempotency_key,
        "bind_stable_member_promotion",
        &request_hash,
    )? {
        return Ok(existing);
    }
    let mut member = store
        .load_member(member_id)?
        .ok_or_else(|| "m6_org_member_not_found".to_string())?;
    if member.revision != expected_revision {
        return Err(format!(
            "m6_org_member_stale_revision:expected={expected_revision}:actual={}",
            member.revision
        ));
    }
    if member.membership_lifecycle == M6OrgMembershipLifecycle::Deactivated {
        return Err("m6_org_member_deactivated".to_string());
    }
    if member.promoted_from.is_some() {
        return Err("m6_org_member_promotion_already_bound".to_string());
    }
    let previous_revision = member.revision;
    member.promoted_from = Some(temporary_agent_id.to_string());
    member.revision = member
        .revision
        .checked_add(1)
        .ok_or_else(|| "m6_org_member_revision_overflow".to_string())?;
    store.record_member_revision(
        idempotency_key,
        "bind_stable_member_promotion",
        &request_hash,
        previous_revision,
        &member,
        now_ms,
    )?;
    Ok(member)
}

pub(crate) fn update_for_state(
    state: &crate::AppState,
    request: &M6OrgUpdateStableMemberRequest,
    now_ms: i64,
) -> Result<M6OrgStableMember, String> {
    require_directory_runtime(state)?;
    validate_member_id(&request.member_id)?;
    validate_nonempty("idempotency_key", &request.idempotency_key)?;
    let request_hash = stable_hash(request)?;
    let mut store = M6OrgMemberDirectoryStore::open(&state.m6_org_store_path()?)?;
    if let Some(existing) = store.load_command_response::<M6OrgStableMember>(
        &request.idempotency_key,
        "update_stable_member",
        &request_hash,
    )? {
        return Ok(existing);
    }
    let mut member = store
        .load_member(&request.member_id)?
        .ok_or_else(|| "m6_org_member_not_found".to_string())?;
    if member.revision != request.expected_revision {
        return Err(format!(
            "m6_org_member_stale_revision:expected={}:actual={}",
            request.expected_revision, member.revision
        ));
    }
    if member.membership_lifecycle == M6OrgMembershipLifecycle::Deactivated {
        return Err("m6_org_member_deactivated".to_string());
    }
    validate_assignments(
        &request.member_id,
        &request.added_scope_assignments,
        &request.added_role_assignments,
        &request.added_capability_permission_refs,
    )?;
    validate_refs("memory_refs", &request.added_memory_refs, true)?;
    validate_contact_bindings(&request.added_contact_bindings)?;
    if let Some(display_name_ref) = &request.display_name_ref {
        validate_ref("display_name_ref", display_name_ref)?;
        member.display_name_ref = display_name_ref.clone();
    }
    merge_by_id(
        &mut member.scope_assignments,
        &request.added_scope_assignments,
        |value| &value.assignment_id,
        "m6_org_member_scope_assignment_collision",
    )?;
    merge_by_id(
        &mut member.role_assignments,
        &request.added_role_assignments,
        |value| &value.assignment_id,
        "m6_org_member_role_assignment_collision",
    )?;
    merge_by_id(
        &mut member.capability_permission_refs,
        &request.added_capability_permission_refs,
        |value| &value.ref_id,
        "m6_org_member_capability_ref_collision",
    )?;
    merge_by_id(
        &mut member.contact_bindings,
        &request.added_contact_bindings,
        |value| &value.binding_ref,
        "m6_org_member_contact_binding_collision",
    )?;
    member.contact_binding_refs = member
        .contact_bindings
        .iter()
        .map(|binding| binding.binding_ref.clone())
        .collect();
    member.memory_refs.extend(request.added_memory_refs.clone());
    member.memory_refs = sorted_unique(member.memory_refs);
    if request.activate {
        member.membership_lifecycle = M6OrgMembershipLifecycle::Active;
    }
    let previous_revision = member.revision;
    member.revision = member
        .revision
        .checked_add(1)
        .ok_or_else(|| "m6_org_member_revision_overflow".to_string())?;
    store.record_member_revision(
        &request.idempotency_key,
        "update_stable_member",
        &request_hash,
        previous_revision,
        &member,
        now_ms,
    )?;
    Ok(member)
}

pub(crate) fn deactivate_for_state(
    state: &crate::AppState,
    request: &M6OrgDeactivateStableMemberRequest,
    now_ms: i64,
) -> Result<M6OrgStableMember, String> {
    require_directory_runtime(state)?;
    validate_member_id(&request.member_id)?;
    validate_ref("reason_ref", &request.reason_ref)?;
    validate_nonempty("idempotency_key", &request.idempotency_key)?;
    let request_hash = stable_hash(request)?;
    let mut store = M6OrgMemberDirectoryStore::open(&state.m6_org_store_path()?)?;
    if let Some(existing) = store.load_command_response::<M6OrgStableMember>(
        &request.idempotency_key,
        "deactivate_stable_member",
        &request_hash,
    )? {
        return Ok(existing);
    }
    let mut member = store
        .load_member(&request.member_id)?
        .ok_or_else(|| "m6_org_member_not_found".to_string())?;
    if member.revision != request.expected_revision {
        return Err(format!(
            "m6_org_member_stale_revision:expected={}:actual={}",
            request.expected_revision, member.revision
        ));
    }
    if member.membership_lifecycle == M6OrgMembershipLifecycle::Deactivated {
        return Err("m6_org_member_already_deactivated".to_string());
    }
    let previous_revision = member.revision;
    member.membership_lifecycle = M6OrgMembershipLifecycle::Deactivated;
    member.deactivated_at = Some(now_ms);
    member.revision = member
        .revision
        .checked_add(1)
        .ok_or_else(|| "m6_org_member_revision_overflow".to_string())?;
    store.record_member_revision(
        &request.idempotency_key,
        "deactivate_stable_member",
        &request_hash,
        previous_revision,
        &member,
        now_ms,
    )?;
    Ok(member)
}

pub(crate) fn observe_availability_for_state(
    state: &crate::AppState,
    request: &M6OrgObserveMemberAvailabilityRequest,
    now_ms: i64,
) -> Result<M6OrgAvailability, String> {
    require_directory_runtime(state)?;
    validate_member_id(&request.member_id)?;
    validate_ref("availability_source", &request.source)?;
    validate_nonempty("idempotency_key", &request.idempotency_key)?;
    if request.source_revision == 0
        || request.observed_at < 0
        || request.observed_at > now_ms
        || request.ttl.seconds == 0
        || request.ttl.seconds > MAX_TTL_SECONDS
    {
        return Err("m6_org_member_availability_shape_invalid".to_string());
    }
    let request_hash = stable_hash(request)?;
    let mut store = M6OrgMemberDirectoryStore::open(&state.m6_org_store_path()?)?;
    if let Some(existing) = store.load_command_response::<M6OrgAvailability>(
        &request.idempotency_key,
        "observe_member_availability",
        &request_hash,
    )? {
        return Ok(effective_availability(existing, now_ms));
    }
    let member = store
        .load_member(&request.member_id)?
        .ok_or_else(|| "m6_org_member_not_found".to_string())?;
    if member.membership_lifecycle == M6OrgMembershipLifecycle::Deactivated {
        return Err("m6_org_member_deactivated".to_string());
    }
    let mut availability = M6OrgAvailability {
        availability_id: digest_id(
            "availability",
            &[
                &request.member_id,
                &request.source,
                &request.source_revision.to_string(),
                &request.observed_at.to_string(),
            ],
        ),
        subject_ref: request.member_id.clone(),
        source: request.source.clone(),
        source_revision: request.source_revision,
        observed_at: request.observed_at,
        ttl: request.ttl.clone(),
        observed_state: request.observed_state,
        effective_state: request.observed_state,
        authorizes: false,
    };
    availability = effective_availability(availability, now_ms);
    store.record_availability(
        &request.idempotency_key,
        &request_hash,
        &availability,
        now_ms,
    )?;
    Ok(availability)
}

pub(crate) fn list_for_state(
    state: &crate::AppState,
    request: &M6OrgListStableMembersRequest,
    now_ms: i64,
) -> Result<M6OrgStableMemberDirectoryResponse, String> {
    require_directory_runtime(state)?;
    if let Some(reference) = &request.available_capability_ref {
        validate_ref("available_capability_ref", reference)?;
    }
    let store = M6OrgMemberDirectoryStore::open(&state.m6_org_store_path()?)?;
    let mut members = Vec::new();
    for mut member in store.list_members()? {
        if member.membership_lifecycle == M6OrgMembershipLifecycle::Deactivated
            && !request.include_deactivated
        {
            continue;
        }
        let availability = store
            .latest_availability(&member.member_id)?
            .map(|value| effective_availability(value, now_ms));
        member.availability_ref = availability
            .as_ref()
            .map(|value| value.availability_id.clone());
        if let Some(required) = &request.available_capability_ref {
            let has_capability = member.capability_permission_refs.iter().any(|reference| {
                reference.kind == M6OrgCapabilityPermissionKind::Capability
                    && reference.ref_id == *required
                    && !reference.directory_is_authority
                    && reference.read_only
            });
            let is_fresh_available = availability
                .as_ref()
                .is_some_and(|value| value.effective_state == M6OrgAvailabilityState::Available);
            if !has_capability || !is_fresh_available {
                continue;
            }
        }
        let contact_receipts = store.list_contact_receipts(&member.member_id)?;
        let session_history_refs = sorted_unique(
            contact_receipts
                .iter()
                .flat_map(|receipt| [receipt.role_session_id.clone(), receipt.handoff_id.clone()])
                .collect(),
        );
        members.push(M6OrgStableMemberView {
            member,
            availability,
            contact_receipts,
            session_history_refs,
            directory_is_authority: false,
            permission_authority: false,
        });
    }
    Ok(M6OrgStableMemberDirectoryResponse {
        members,
        judged_at: now_ms,
        stale_availability_used_as_capability: false,
        directory_is_authority: false,
    })
}

pub(crate) fn export_for_state(
    state: &crate::AppState,
    now_ms: i64,
) -> Result<M6OrgMemberDirectoryExport, String> {
    require_directory_runtime(state)?;
    let store = M6OrgMemberDirectoryStore::open(&state.m6_org_store_path()?)?;
    let export = store.export(now_ms)?;
    verify_export_rebuild(&export)?;
    Ok(export)
}

pub(crate) fn contact_for_state(
    state: &crate::AppState,
    request: &M6OrgContactStableMemberRequest,
    now_ms: i64,
) -> Result<M6OrgContactReceipt, String> {
    require_directory_runtime(state)?;
    validate_member_id(&request.member_id)?;
    validate_ref("contact_binding_ref", &request.contact_binding_ref)?;
    validate_ref("reason_ref", &request.reason_ref)?;
    validate_refs("contact_source_refs", &request.source_refs, false)?;
    validate_nonempty("accept_by_utc", &request.accept_by_utc)?;
    validate_nonempty("idempotency_key", &request.idempotency_key)?;
    let request_hash = stable_hash(request)?;
    let mut store = M6OrgMemberDirectoryStore::open(&state.m6_org_store_path()?)?;
    if let Some(mut existing) = store.load_command_response::<M6OrgContactReceipt>(
        &request.idempotency_key,
        "contact_stable_member",
        &request_hash,
    )? {
        existing.replayed = true;
        return Ok(existing);
    }
    let member = store
        .load_member(&request.member_id)?
        .ok_or_else(|| "m6_org_member_not_found".to_string())?;
    if member.membership_lifecycle == M6OrgMembershipLifecycle::Deactivated {
        return Err("m6_org_member_deactivated".to_string());
    }
    let binding = member
        .contact_bindings
        .iter()
        .find(|binding| binding.binding_ref == request.contact_binding_ref)
        .ok_or_else(|| "m6_org_member_contact_binding_not_found".to_string())?;
    let handoff = crate::m6_org_consult_handoff::start_member_contact_handoff_for_state(
        state,
        &request.member_id,
        binding,
        &request.reason_ref,
        &request.source_refs,
        &request.accept_by_utc,
        &request.idempotency_key,
        now_ms,
    )?;
    let receipt = contact_receipt_from_handoff(
        &request.member_id,
        &request.contact_binding_ref,
        &request.idempotency_key,
        &handoff,
        now_ms,
    );
    store.record_contact(&request.idempotency_key, &request_hash, &receipt, now_ms)?;
    Ok(receipt)
}

fn contact_receipt_from_handoff(
    member_id: &str,
    binding_ref: &str,
    idempotency_key: &str,
    handoff: &M6OrgMemberContactHandoff,
    now_ms: i64,
) -> M6OrgContactReceipt {
    M6OrgContactReceipt {
        contact_receipt_id: digest_id("contact-receipt", &[member_id, idempotency_key]),
        from_actor_id: handoff.from_actor_id.clone(),
        to_member_id: member_id.to_string(),
        handoff_id: handoff.handoff_id.clone(),
        handoff_receipt_ref: handoff.handoff_receipt_ref.clone(),
        role_session_id: handoff.role_session_id.clone(),
        status: "RECORDED".to_string(),
        capability_granted: false,
        recorded_at: now_ms,
        source_command_receipt_ref: handoff.source_command_receipt_ref.clone(),
        contact_binding_ref: binding_ref.to_string(),
        replayed: handoff.replayed,
        project_writeback: false,
    }
}

struct M6OrgMemberDirectoryStore {
    connection: Connection,
}

impl M6OrgMemberDirectoryStore {
    fn open(path: &Path) -> Result<Self, String> {
        let parent = path
            .parent()
            .ok_or_else(|| "m6_org_member_store_parent_missing".to_string())?;
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("m6_org_member_store_parent_create:{error}"))?;
        let connection =
            Connection::open(path).map_err(|error| format!("m6_org_member_store_open:{error}"))?;
        ensure_m6_org_schema(&connection)?;
        Ok(Self { connection })
    }

    fn open_in_memory() -> Result<Self, String> {
        let connection = Connection::open_in_memory()
            .map_err(|error| format!("m6_org_member_store_mem:{error}"))?;
        ensure_m6_org_schema(&connection)?;
        Ok(Self { connection })
    }

    fn load_command_response<T: DeserializeOwned>(
        &self,
        idempotency_key: &str,
        operation: &str,
        request_hash: &str,
    ) -> Result<Option<T>, String> {
        let row = self
            .connection
            .query_row(
                "SELECT operation, request_hash, response_json
                 FROM m6_member_directory_command_receipts
                 WHERE idempotency_key=?1",
                [idempotency_key],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .optional()
            .map_err(|error| format!("m6_org_member_command_receipt_load:{error}"))?;
        let Some((recorded_operation, recorded_hash, response_json)) = row else {
            return Ok(None);
        };
        if recorded_operation != operation || recorded_hash != request_hash {
            return Err("m6_org_member_idempotency_collision".to_string());
        }
        serde_json::from_str(&response_json)
            .map(Some)
            .map_err(|error| format!("m6_org_member_command_receipt_decode:{error}"))
    }

    fn record_registration(
        &mut self,
        idempotency_key: &str,
        operation: &str,
        request_hash: &str,
        member: &M6OrgStableMember,
        outcome: &M6OrgStableMemberRegistrationOutcome,
        now_ms: i64,
    ) -> Result<(), String> {
        if self.load_member(&member.member_id)?.is_some() {
            return Err("m6_org_member_id_already_registered".to_string());
        }
        let member_payload = encode(member, "m6_org_member_registration")?;
        let response_payload = encode(outcome, "m6_org_member_registration_response")?;
        let audit_payload = encode(
            &json!({
                "member_id": member.member_id,
                "identity_contract_ref": member.identity_contract_ref,
                "scope_assignment_count": member.scope_assignments.len(),
                "role_assignment_count": member.role_assignments.len(),
                "capability_permission_ref_count": member.capability_permission_refs.len(),
                "directory_is_authority": false,
                "provider_identity": false,
                "runtime_identity": false
            }),
            "m6_org_member_registration_audit",
        )?;
        let transaction = self
            .connection
            .transaction()
            .map_err(|error| format!("m6_org_member_registration_tx:{error}"))?;
        transaction
            .execute(
                "INSERT INTO m6_stable_member_identities (
                    member_id, identity_contract_ref, registration_idempotency_key, created_at_ms
                 ) VALUES (?1,?2,?3,?4)",
                params![
                    member.member_id,
                    member.identity_contract_ref,
                    idempotency_key,
                    now_ms
                ],
            )
            .map_err(|error| format!("m6_org_member_identity_insert:{error}"))?;
        insert_member_history(&transaction, member, &member_payload, now_ms)?;
        insert_directory_audit(
            &transaction,
            &digest_id("audit-member-register", &[&member.member_id]),
            "StableMemberRegistered",
            &member.member_id,
            &audit_payload,
            now_ms,
        )?;
        insert_command_receipt(
            &transaction,
            idempotency_key,
            operation,
            request_hash,
            &response_payload,
            now_ms,
        )?;
        transaction
            .commit()
            .map_err(|error| format!("m6_org_member_registration_commit:{error}"))
    }

    fn record_quarantine(
        &mut self,
        idempotency_key: &str,
        request_hash: &str,
        quarantine: &M6OrgLegacyQuarantineRecord,
        outcome: &M6OrgStableMemberRegistrationOutcome,
        now_ms: i64,
    ) -> Result<(), String> {
        let payload = encode(quarantine, "m6_org_member_quarantine")?;
        let response = encode(outcome, "m6_org_member_quarantine_response")?;
        let audit = encode(
            &json!({
                "quarantine_ref": quarantine.quarantine_ref,
                "reason_code": quarantine.reason_code,
                "source_ref_count": quarantine.source_refs.len(),
                "payload_mode": "REF_ONLY",
                "mapped_to": null
            }),
            "m6_org_member_quarantine_audit",
        )?;
        let transaction = self
            .connection
            .transaction()
            .map_err(|error| format!("m6_org_member_quarantine_tx:{error}"))?;
        transaction
            .execute(
                "INSERT INTO m6_member_identity_quarantine (
                    quarantine_ref,idempotency_key,reason_code,payload_json,recorded_at_ms
                 ) VALUES (?1,?2,?3,?4,?5)",
                params![
                    quarantine.quarantine_ref,
                    idempotency_key,
                    quarantine.reason_code,
                    payload,
                    now_ms
                ],
            )
            .map_err(|error| format!("m6_org_member_quarantine_insert:{error}"))?;
        insert_directory_audit(
            &transaction,
            &digest_id("audit-member-quarantine", &[&quarantine.quarantine_ref]),
            "StableMemberIdentityQuarantined",
            &quarantine.quarantine_ref,
            &audit,
            now_ms,
        )?;
        insert_command_receipt(
            &transaction,
            idempotency_key,
            "register_stable_member",
            request_hash,
            &response,
            now_ms,
        )?;
        transaction
            .commit()
            .map_err(|error| format!("m6_org_member_quarantine_commit:{error}"))
    }

    fn record_member_revision(
        &mut self,
        idempotency_key: &str,
        operation: &str,
        request_hash: &str,
        expected_previous_revision: u64,
        member: &M6OrgStableMember,
        now_ms: i64,
    ) -> Result<(), String> {
        let payload = encode(member, "m6_org_member_revision")?;
        let audit_payload = encode(
            &json!({
                "member_id": member.member_id,
                "revision": member.revision,
                "membership_lifecycle": member.membership_lifecycle.as_str(),
                "retained_scope_assignment_count": member.scope_assignments.len(),
                "retained_role_assignment_count": member.role_assignments.len(),
                "retained_capability_permission_ref_count": member.capability_permission_refs.len(),
                "retained_memory_ref_count": member.memory_refs.len(),
                "physically_deleted": false,
                "directory_is_authority": false
            }),
            "m6_org_member_revision_audit",
        )?;
        let transaction = self
            .connection
            .transaction()
            .map_err(|error| format!("m6_org_member_revision_tx:{error}"))?;
        let current_revision = transaction
            .query_row(
                "SELECT revision FROM m6_stable_member_history
                 WHERE member_id=?1 ORDER BY revision DESC LIMIT 1",
                [&member.member_id],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(|error| format!("m6_org_member_revision_current:{error}"))?
            .ok_or_else(|| "m6_org_member_not_found".to_string())?;
        if current_revision != expected_previous_revision as i64
            || member.revision != expected_previous_revision.saturating_add(1)
        {
            return Err(format!(
                "m6_org_member_stale_revision:expected={expected_previous_revision}:actual={current_revision}"
            ));
        }
        insert_member_history(&transaction, member, &payload, now_ms)?;
        insert_directory_audit(
            &transaction,
            &digest_id(
                "audit-member-revision",
                &[&member.member_id, &member.revision.to_string()],
            ),
            if member.membership_lifecycle == M6OrgMembershipLifecycle::Deactivated {
                "StableMemberDeactivated"
            } else {
                "StableMemberUpdated"
            },
            &member.member_id,
            &audit_payload,
            now_ms,
        )?;
        insert_command_receipt(
            &transaction,
            idempotency_key,
            operation,
            request_hash,
            &payload,
            now_ms,
        )?;
        transaction
            .commit()
            .map_err(|error| format!("m6_org_member_revision_commit:{error}"))
    }

    fn record_availability(
        &mut self,
        idempotency_key: &str,
        request_hash: &str,
        availability: &M6OrgAvailability,
        now_ms: i64,
    ) -> Result<(), String> {
        let payload = encode(availability, "m6_org_member_availability")?;
        let audit = encode(
            &json!({
                "availability_id": availability.availability_id,
                "subject_ref": availability.subject_ref,
                "source": availability.source,
                "source_revision": availability.source_revision,
                "observed_state": availability.observed_state,
                "effective_state": availability.effective_state,
                "authorizes": false
            }),
            "m6_org_member_availability_audit",
        )?;
        let ttl_seconds = i64::try_from(availability.ttl.seconds)
            .map_err(|_| "m6_org_member_availability_ttl_overflow".to_string())?;
        let transaction = self
            .connection
            .transaction()
            .map_err(|error| format!("m6_org_member_availability_tx:{error}"))?;
        transaction
            .execute(
                "INSERT INTO m6_member_availability_history (
                    availability_id,member_id,source,observed_at_ms,ttl_seconds,payload_json
                 ) VALUES (?1,?2,?3,?4,?5,?6)",
                params![
                    availability.availability_id,
                    availability.subject_ref,
                    availability.source,
                    availability.observed_at,
                    ttl_seconds,
                    payload
                ],
            )
            .map_err(|error| format!("m6_org_member_availability_insert:{error}"))?;
        insert_directory_audit(
            &transaction,
            &digest_id(
                "audit-member-availability",
                &[&availability.availability_id],
            ),
            "AvailabilityObserved",
            &availability.subject_ref,
            &audit,
            now_ms,
        )?;
        insert_command_receipt(
            &transaction,
            idempotency_key,
            "observe_member_availability",
            request_hash,
            &payload,
            now_ms,
        )?;
        transaction
            .commit()
            .map_err(|error| format!("m6_org_member_availability_commit:{error}"))
    }

    fn record_contact(
        &mut self,
        idempotency_key: &str,
        request_hash: &str,
        receipt: &M6OrgContactReceipt,
        now_ms: i64,
    ) -> Result<(), String> {
        if receipt.capability_granted || receipt.project_writeback {
            return Err("m6_org_member_contact_capability_forbidden".to_string());
        }
        let payload = encode(receipt, "m6_org_member_contact")?;
        let audit = encode(
            &json!({
                "contact_receipt_id": receipt.contact_receipt_id,
                "to_member_id": receipt.to_member_id,
                "handoff_id": receipt.handoff_id,
                "role_session_id": receipt.role_session_id,
                "status": receipt.status,
                "capability_granted": false,
                "project_writeback": false
            }),
            "m6_org_member_contact_audit",
        )?;
        let transaction = self
            .connection
            .transaction()
            .map_err(|error| format!("m6_org_member_contact_tx:{error}"))?;
        transaction
            .execute(
                "INSERT INTO m6_member_contact_receipts (
                    contact_receipt_id,member_id,idempotency_key,handoff_id,payload_json,recorded_at_ms
                 ) VALUES (?1,?2,?3,?4,?5,?6)",
                params![
                    receipt.contact_receipt_id,
                    receipt.to_member_id,
                    idempotency_key,
                    receipt.handoff_id,
                    payload,
                    now_ms
                ],
            )
            .map_err(|error| format!("m6_org_member_contact_insert:{error}"))?;
        insert_directory_audit(
            &transaction,
            &digest_id("audit-member-contact", &[&receipt.contact_receipt_id]),
            "MemberContactRecorded",
            &receipt.to_member_id,
            &audit,
            now_ms,
        )?;
        insert_command_receipt(
            &transaction,
            idempotency_key,
            "contact_stable_member",
            request_hash,
            &payload,
            now_ms,
        )?;
        transaction
            .commit()
            .map_err(|error| format!("m6_org_member_contact_commit:{error}"))
    }

    fn load_member(&self, member_id: &str) -> Result<Option<M6OrgStableMember>, String> {
        load_json_optional(
            &self.connection,
            "SELECT payload_json FROM m6_stable_member_history
             WHERE member_id=?1 ORDER BY revision DESC LIMIT 1",
            member_id,
            "m6_org_member_load",
        )
    }

    fn list_members(&self) -> Result<Vec<M6OrgStableMember>, String> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT history.payload_json
                 FROM m6_stable_member_history history
                 JOIN (
                    SELECT member_id, MAX(revision) AS revision
                    FROM m6_stable_member_history GROUP BY member_id
                 ) latest
                 ON latest.member_id=history.member_id AND latest.revision=history.revision
                 ORDER BY history.member_id",
            )
            .map_err(|error| format!("m6_org_member_list_prepare:{error}"))?;
        decode_rows(&mut statement, [], "m6_org_member_list")
    }

    fn latest_availability(&self, member_id: &str) -> Result<Option<M6OrgAvailability>, String> {
        load_json_optional(
            &self.connection,
            "SELECT payload_json FROM m6_member_availability_history
             WHERE member_id=?1 ORDER BY observed_at_ms DESC, availability_id DESC LIMIT 1",
            member_id,
            "m6_org_member_availability_load",
        )
    }

    fn list_contact_receipts(&self, member_id: &str) -> Result<Vec<M6OrgContactReceipt>, String> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT payload_json FROM m6_member_contact_receipts
                 WHERE member_id=?1 ORDER BY recorded_at_ms, contact_receipt_id",
            )
            .map_err(|error| format!("m6_org_member_contact_list_prepare:{error}"))?;
        decode_rows(&mut statement, [member_id], "m6_org_member_contact_list")
    }

    fn export(&self, now_ms: i64) -> Result<M6OrgMemberDirectoryExport, String> {
        let member_history = self.read_all::<M6OrgStableMember>(
            "SELECT payload_json FROM m6_stable_member_history ORDER BY member_id, revision",
            "m6_org_member_export_history",
        )?;
        let availability_history = self
            .read_all::<M6OrgAvailability>(
                "SELECT payload_json FROM m6_member_availability_history
                 ORDER BY member_id, observed_at_ms, availability_id",
                "m6_org_member_export_availability",
            )?
            .into_iter()
            .map(|value| effective_availability(value, now_ms))
            .collect();
        let contact_history = self.read_all::<M6OrgContactReceipt>(
            "SELECT payload_json FROM m6_member_contact_receipts
             ORDER BY member_id, recorded_at_ms, contact_receipt_id",
            "m6_org_member_export_contact",
        )?;
        let quarantines = self.read_all::<M6OrgLegacyQuarantineRecord>(
            "SELECT payload_json FROM m6_member_identity_quarantine
             ORDER BY recorded_at_ms, quarantine_ref",
            "m6_org_member_export_quarantine",
        )?;
        Ok(M6OrgMemberDirectoryExport {
            schema: DIRECTORY_EXPORT_SCHEMA.to_string(),
            member_history,
            availability_history,
            contact_history,
            quarantines,
        })
    }

    fn read_all<T: DeserializeOwned>(
        &self,
        sql: &str,
        error_prefix: &str,
    ) -> Result<Vec<T>, String> {
        let mut statement = self
            .connection
            .prepare(sql)
            .map_err(|error| format!("{error_prefix}_prepare:{error}"))?;
        decode_rows(&mut statement, [], error_prefix)
    }

    fn restore_export(&mut self, export: &M6OrgMemberDirectoryExport) -> Result<(), String> {
        if export.schema != DIRECTORY_EXPORT_SCHEMA {
            return Err("m6_org_member_export_schema_mismatch".to_string());
        }
        if !self.list_members()?.is_empty() {
            return Err("m6_org_member_restore_target_not_empty".to_string());
        }
        validate_export(export)?;
        let transaction = self
            .connection
            .transaction()
            .map_err(|error| format!("m6_org_member_restore_tx:{error}"))?;
        let mut identities = BTreeMap::<String, (String, i64)>::new();
        for member in &export.member_history {
            identities
                .entry(member.member_id.clone())
                .or_insert_with(|| (member.identity_contract_ref.clone(), member.created_at));
        }
        for (member_id, (identity_contract_ref, created_at)) in identities {
            transaction
                .execute(
                    "INSERT INTO m6_stable_member_identities (
                        member_id,identity_contract_ref,registration_idempotency_key,created_at_ms
                     ) VALUES (?1,?2,?3,?4)",
                    params![
                        member_id,
                        identity_contract_ref,
                        digest_id("restored-registration", &[&member_id]),
                        created_at
                    ],
                )
                .map_err(|error| format!("m6_org_member_restore_identity:{error}"))?;
        }
        for member in &export.member_history {
            let payload = encode(member, "m6_org_member_restore_history")?;
            insert_member_history(&transaction, member, &payload, member.created_at)?;
        }
        for availability in &export.availability_history {
            transaction
                .execute(
                    "INSERT INTO m6_member_availability_history (
                        availability_id,member_id,source,observed_at_ms,ttl_seconds,payload_json
                     ) VALUES (?1,?2,?3,?4,?5,?6)",
                    params![
                        availability.availability_id,
                        availability.subject_ref,
                        availability.source,
                        availability.observed_at,
                        i64::try_from(availability.ttl.seconds)
                            .map_err(|_| "m6_org_member_restore_ttl_overflow".to_string())?,
                        encode(availability, "m6_org_member_restore_availability")?
                    ],
                )
                .map_err(|error| format!("m6_org_member_restore_availability:{error}"))?;
        }
        for receipt in &export.contact_history {
            transaction
                .execute(
                    "INSERT INTO m6_member_contact_receipts (
                        contact_receipt_id,member_id,idempotency_key,handoff_id,payload_json,recorded_at_ms
                     ) VALUES (?1,?2,?3,?4,?5,?6)",
                    params![
                        receipt.contact_receipt_id,
                        receipt.to_member_id,
                        digest_id("restored-contact", &[&receipt.contact_receipt_id]),
                        receipt.handoff_id,
                        encode(receipt, "m6_org_member_restore_contact")?,
                        receipt.recorded_at
                    ],
                )
                .map_err(|error| format!("m6_org_member_restore_contact:{error}"))?;
        }
        for quarantine in &export.quarantines {
            transaction
                .execute(
                    "INSERT INTO m6_member_identity_quarantine (
                        quarantine_ref,idempotency_key,reason_code,payload_json,recorded_at_ms
                     ) VALUES (?1,?2,?3,?4,?5)",
                    params![
                        quarantine.quarantine_ref,
                        digest_id("restored-quarantine", &[&quarantine.quarantine_ref]),
                        quarantine.reason_code,
                        encode(quarantine, "m6_org_member_restore_quarantine")?,
                        quarantine.recorded_at
                    ],
                )
                .map_err(|error| format!("m6_org_member_restore_quarantine:{error}"))?;
        }
        transaction
            .commit()
            .map_err(|error| format!("m6_org_member_restore_commit:{error}"))
    }

    #[cfg(test)]
    fn count_rows(&self, table: &str) -> Result<i64, String> {
        let allowed = [
            "m6_stable_member_identities",
            "m6_stable_member_history",
            "m6_member_identity_quarantine",
            "m6_member_availability_history",
            "m6_member_contact_receipts",
            "m6_member_directory_command_receipts",
        ];
        if !allowed.contains(&table) {
            return Err("m6_org_member_test_table_not_allowed".to_string());
        }
        self.connection
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get::<_, i64>(0)
            })
            .map_err(|error| format!("m6_org_member_count_rows:{error}"))
    }
}

fn insert_member_history(
    transaction: &Transaction<'_>,
    member: &M6OrgStableMember,
    payload: &str,
    recorded_at_ms: i64,
) -> Result<(), String> {
    transaction
        .execute(
            "INSERT INTO m6_stable_member_history (
                member_id,revision,membership_lifecycle,payload_json,recorded_at_ms
             ) VALUES (?1,?2,?3,?4,?5)",
            params![
                member.member_id,
                i64::try_from(member.revision)
                    .map_err(|_| "m6_org_member_revision_overflow".to_string())?,
                member.membership_lifecycle.as_str(),
                payload,
                recorded_at_ms
            ],
        )
        .map_err(|error| format!("m6_org_member_history_insert:{error}"))?;
    Ok(())
}

fn insert_command_receipt(
    transaction: &Transaction<'_>,
    idempotency_key: &str,
    operation: &str,
    request_hash: &str,
    response_json: &str,
    now_ms: i64,
) -> Result<(), String> {
    transaction
        .execute(
            "INSERT INTO m6_member_directory_command_receipts (
                idempotency_key,operation,request_hash,response_json,recorded_at_ms
             ) VALUES (?1,?2,?3,?4,?5)",
            params![
                idempotency_key,
                operation,
                request_hash,
                response_json,
                now_ms
            ],
        )
        .map_err(|error| format!("m6_org_member_command_receipt_insert:{error}"))?;
    Ok(())
}

fn insert_directory_audit(
    transaction: &Transaction<'_>,
    event_id: &str,
    event_type: &str,
    target_ref: &str,
    payload_json: &str,
    now_ms: i64,
) -> Result<(), String> {
    transaction
        .execute(
            "INSERT INTO m6_org_audit_events (
                event_id,event_type,target_ref,payload_json,created_at_ms
             ) VALUES (?1,?2,?3,?4,?5)",
            params![event_id, event_type, target_ref, payload_json, now_ms],
        )
        .map_err(|error| format!("m6_org_member_audit_insert:{error}"))?;
    Ok(())
}

fn load_json_optional<T: DeserializeOwned>(
    connection: &Connection,
    sql: &str,
    key: &str,
    error_prefix: &str,
) -> Result<Option<T>, String> {
    let payload = connection
        .query_row(sql, [key], |row| row.get::<_, String>(0))
        .optional()
        .map_err(|error| format!("{error_prefix}:{error}"))?;
    payload
        .map(|payload| {
            serde_json::from_str(&payload).map_err(|error| format!("{error_prefix}_decode:{error}"))
        })
        .transpose()
}

fn decode_rows<P, T>(
    statement: &mut rusqlite::Statement<'_>,
    params: P,
    error_prefix: &str,
) -> Result<Vec<T>, String>
where
    P: rusqlite::Params,
    T: DeserializeOwned,
{
    let rows = statement
        .query_map(params, |row| row.get::<_, String>(0))
        .map_err(|error| format!("{error_prefix}_query:{error}"))?;
    let mut values = Vec::new();
    for row in rows {
        let payload = row.map_err(|error| format!("{error_prefix}_row:{error}"))?;
        values.push(
            serde_json::from_str(&payload)
                .map_err(|error| format!("{error_prefix}_decode:{error}"))?,
        );
    }
    Ok(values)
}

fn require_directory_runtime(state: &crate::AppState) -> Result<(), String> {
    let _ = state.m6_org_global_role_session.authority_seed()?;
    Ok(())
}

fn effective_availability(mut availability: M6OrgAvailability, now_ms: i64) -> M6OrgAvailability {
    let ttl_ms = availability
        .ttl
        .seconds
        .checked_mul(1_000)
        .and_then(|value| i64::try_from(value).ok());
    let expires_at = ttl_ms.and_then(|ttl| availability.observed_at.checked_add(ttl));
    if expires_at.is_none_or(|expires_at| now_ms > expires_at) {
        availability.effective_state = M6OrgAvailabilityState::Unknown;
    }
    availability.authorizes = false;
    availability
}

fn validate_assignments(
    member_id: &str,
    scopes: &[M6OrgScopeAssignment],
    roles: &[M6OrgRoleAssignment],
    capability_refs: &[M6OrgCapabilityPermissionRef],
) -> Result<(), String> {
    let mut ids = BTreeSet::new();
    for assignment in scopes {
        if assignment.member_id != member_id
            || assignment.revision == 0
            || assignment.assigned_at < 0
            || assignment
                .revoked_at
                .is_some_and(|revoked| revoked < assignment.assigned_at)
        {
            return Err("m6_org_member_scope_assignment_invalid".to_string());
        }
        validate_ref("scope_assignment_id", &assignment.assignment_id)?;
        validate_ref("scope_ref", &assignment.scope_ref)?;
        validate_ref("assigned_by_actor_id", &assignment.assigned_by_actor_id)?;
        if !ids.insert(assignment.assignment_id.as_str()) {
            return Err("m6_org_member_scope_assignment_duplicate".to_string());
        }
    }
    ids.clear();
    for assignment in roles {
        if assignment.member_id != member_id
            || assignment.revision == 0
            || assignment.assigned_at < 0
            || assignment
                .revoked_at
                .is_some_and(|revoked| revoked < assignment.assigned_at)
        {
            return Err("m6_org_member_role_assignment_invalid".to_string());
        }
        validate_ref("role_assignment_id", &assignment.assignment_id)?;
        validate_ref("role_ref", &assignment.role_ref)?;
        validate_ref("scope_ref", &assignment.scope_ref)?;
        validate_ref("assigned_by_actor_id", &assignment.assigned_by_actor_id)?;
        if !ids.insert(assignment.assignment_id.as_str()) {
            return Err("m6_org_member_role_assignment_duplicate".to_string());
        }
    }
    ids.clear();
    for reference in capability_refs {
        if reference.subject_member_id != member_id
            || reference.revision == 0
            || reference.observed_at < 0
            || reference.directory_is_authority
            || !reference.read_only
        {
            return Err("m6_org_member_capability_permission_ref_invalid".to_string());
        }
        validate_ref("capability_permission_ref_id", &reference.ref_id)?;
        validate_ref("capability_permission_source", &reference.source)?;
        if !ids.insert(reference.ref_id.as_str()) {
            return Err("m6_org_member_capability_permission_ref_duplicate".to_string());
        }
    }
    Ok(())
}

fn validate_contact_bindings(bindings: &[M6OrgMemberContactBinding]) -> Result<(), String> {
    let mut ids = BTreeSet::new();
    for binding in bindings {
        if binding.revision == 0 || binding.observed_at < 0 {
            return Err("m6_org_member_contact_binding_invalid".to_string());
        }
        validate_ref("contact_binding_ref", &binding.binding_ref)?;
        validate_ref("contact_role_ref", &binding.to_role_ref)?;
        validate_ref("contact_recipient_ref", &binding.to_recipient_ref)?;
        validate_ref("contact_binding_source", &binding.source)?;
        crate::m3_role_session::OpaqueRef::try_from_canonical(binding.to_role_ref.clone())
            .map_err(|_| "m6_org_member_contact_role_ref_invalid".to_string())?;
        crate::m3_role_session::OpaqueRef::try_from_canonical(binding.to_recipient_ref.clone())
            .map_err(|_| "m6_org_member_contact_recipient_ref_invalid".to_string())?;
        if !ids.insert(binding.binding_ref.as_str()) {
            return Err("m6_org_member_contact_binding_duplicate".to_string());
        }
    }
    Ok(())
}

fn validate_export(export: &M6OrgMemberDirectoryExport) -> Result<(), String> {
    let mut members = BTreeMap::<&str, (&str, u64)>::new();
    let mut revisions = BTreeSet::new();
    for member in &export.member_history {
        validate_member_id(&member.member_id)?;
        validate_ref("identity_contract_ref", &member.identity_contract_ref)?;
        let identity = members
            .entry(member.member_id.as_str())
            .or_insert((member.identity_contract_ref.as_str(), 0));
        if identity.0 != member.identity_contract_ref || member.revision != identity.1 + 1 {
            return Err("m6_org_member_export_history_not_contiguous".to_string());
        }
        identity.1 = member.revision;
        if !revisions.insert((member.member_id.as_str(), member.revision)) {
            return Err("m6_org_member_export_history_duplicate".to_string());
        }
    }
    for availability in &export.availability_history {
        if !members.contains_key(availability.subject_ref.as_str())
            || availability.authorizes
            || availability.ttl.seconds == 0
        {
            return Err("m6_org_member_export_availability_invalid".to_string());
        }
    }
    for receipt in &export.contact_history {
        if !members.contains_key(receipt.to_member_id.as_str())
            || receipt.capability_granted
            || receipt.project_writeback
        {
            return Err("m6_org_member_export_contact_invalid".to_string());
        }
    }
    Ok(())
}

fn verify_export_rebuild(export: &M6OrgMemberDirectoryExport) -> Result<(), String> {
    let mut rebuilt = M6OrgMemberDirectoryStore::open_in_memory()?;
    rebuilt.restore_export(export)?;
    let rebuilt_members = rebuilt.list_members()?;
    let mut expected = BTreeMap::<String, M6OrgStableMember>::new();
    for member in &export.member_history {
        expected.insert(member.member_id.clone(), member.clone());
    }
    if rebuilt_members != expected.into_values().collect::<Vec<_>>() {
        return Err("m6_org_member_export_rebuild_mismatch".to_string());
    }
    Ok(())
}

fn validate_member_id(value: &str) -> Result<(), String> {
    validate_ref("member_id", value)?;
    let lowered = value.to_ascii_lowercase();
    if !value.starts_with(MEMBER_ID_PREFIX)
        || lowered.starts_with("member_temporary_agent")
        || lowered.starts_with("member_provider")
        || lowered.starts_with("member_model")
        || lowered.starts_with("member_thread")
        || lowered.starts_with("member_process")
        || lowered.starts_with("member_session")
        || lowered.starts_with("member_child_run")
    {
        return Err("m6_org_member_identity_namespace_rejected".to_string());
    }
    Ok(())
}

fn validate_temporary_agent_id(value: &str) -> Result<(), String> {
    validate_ref("temporary_agent_id", value)?;
    if !value.starts_with("temporary_agent_") || value.starts_with("member_") {
        return Err("m6_org_member_temporary_agent_id_invalid".to_string());
    }
    Ok(())
}

fn validate_refs(field: &str, values: &[String], allow_empty: bool) -> Result<(), String> {
    if !allow_empty && values.is_empty() {
        return Err(format!("m6_org_member_{field}_required"));
    }
    let mut unique = BTreeSet::new();
    for value in values {
        validate_ref(field, value)?;
        if !unique.insert(value) {
            return Err(format!("m6_org_member_{field}_duplicate"));
        }
    }
    Ok(())
}

fn validate_ref(field: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty()
        || value.len() > MAX_REF_LEN
        || value.chars().any(|character| character.is_control())
    {
        return Err(format!("m6_org_member_{field}_invalid"));
    }
    Ok(())
}

fn validate_nonempty(field: &str, value: &str) -> Result<(), String> {
    validate_ref(field, value)
}

fn merge_by_id<T: Clone + Eq>(
    current: &mut Vec<T>,
    added: &[T],
    id: impl Fn(&T) -> &str,
    collision_error: &str,
) -> Result<(), String> {
    for candidate in added {
        if let Some(existing) = current
            .iter()
            .find(|existing| id(existing) == id(candidate))
        {
            if existing != candidate {
                return Err(collision_error.to_string());
            }
        } else {
            current.push(candidate.clone());
        }
    }
    current.sort_by(|left, right| id(left).cmp(id(right)));
    Ok(())
}

fn sorted_by_id<T>(mut values: Vec<T>, id: impl Fn(&T) -> &str) -> Vec<T> {
    values.sort_by(|left, right| id(left).cmp(id(right)));
    values
}

fn sorted_unique(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn stable_hash(value: &impl Serialize) -> Result<String, String> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| format!("m6_org_member_request_serialize:{error}"))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn digest_id(namespace: &str, parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"syn.m6.org.member-directory/v1\0");
    hasher.update(namespace.as_bytes());
    for part in parts {
        hasher.update(b"\0");
        hasher.update(part.as_bytes());
    }
    format!("{namespace}:sha256:{:x}", hasher.finalize())
}

fn encode(value: &impl Serialize, prefix: &str) -> Result<String, String> {
    serde_json::to_string(value).map_err(|error| format!("{prefix}_serialize:{error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AppState;
    use rusqlite::Connection;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    const NOW_MS: i64 = 1_787_097_600_000;
    const ACCEPT_BY: &str = "2026-08-19T00:10:00.000Z";
    static SCRATCH_SEQUENCE: AtomicU64 = AtomicU64::new(1);

    struct Fixture {
        root: PathBuf,
        app_data_root: PathBuf,
        state: AppState,
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    fn fixture(label: &str) -> Fixture {
        let sequence = SCRATCH_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "syn-m6d05-{label}-{}-{sequence}",
            std::process::id()
        ));
        let app_data_root = root.join(crate::m1_project_index::M1_ORDINARY_APP_DATA_DIR_NAME);
        std::fs::create_dir_all(&app_data_root).expect("create M6D05 app-data root");
        let app_data_root =
            std::fs::canonicalize(&app_data_root).expect("canonical M6D05 app-data root");
        let seed_dir = root.join("synthetic-ordinary-product-seeds");
        std::fs::create_dir_all(&seed_dir).expect("create M6D05 seeds");
        let index_seed = seed_dir.join("codex-index.json");
        let tasks_seed = seed_dir.join("README.md");
        std::fs::write(&index_seed, r#"{"projects":[]}"#).expect("write M6D05 index seed");
        std::fs::write(&tasks_seed, "# synthetic M6D05 tasks\n").expect("write M6D05 tasks seed");
        let state = AppState::try_new_with_tauri_ordinary_product_seeds(
            &app_data_root,
            &index_seed,
            &tasks_seed,
        )
        .expect("ordinary M6D05 AppState");
        Fixture {
            root,
            app_data_root,
            state,
        }
    }

    fn test_ref(namespace: &str, material: &str) -> String {
        format!(
            "{namespace}:sha256:{:x}",
            Sha256::digest(material.as_bytes())
        )
    }

    fn contact_binding(member_id: &str) -> M6OrgMemberContactBinding {
        M6OrgMemberContactBinding {
            binding_ref: format!("contact-binding:{member_id}"),
            to_role_ref: test_ref("role", &format!("{member_id}/role")),
            to_recipient_ref: test_ref("actor", &format!("{member_id}/recipient")),
            source: "syn.fixture.explicit-contact-binding/v1".to_string(),
            revision: 1,
            observed_at: NOW_MS,
        }
    }

    fn explicit_request(
        member_id: &str,
        display_name_ref: &str,
        idempotency_key: &str,
    ) -> M6OrgRegisterStableMemberRequest {
        M6OrgRegisterStableMemberRequest {
            member_id: member_id.to_string(),
            display_name_ref: display_name_ref.to_string(),
            identity_evidence: M6OrgStableIdentityEvidence::ExplicitIdentityContract {
                contract_kind: IDENTITY_CONTRACT_KIND.to_string(),
                identity_contract_ref: format!("identity-contract:{member_id}"),
                source_record_ref: format!("identity-source:{member_id}"),
                source_revision: 1,
                observed_at: NOW_MS,
                explicit_human_command: true,
            },
            scope_assignments: vec![M6OrgScopeAssignment {
                assignment_id: format!("scope-assignment:{member_id}:global"),
                member_id: member_id.to_string(),
                scope_ref: "scope:global".to_string(),
                assigned_by_actor_id: "actor:user".to_string(),
                revision: 1,
                assigned_at: NOW_MS,
                revoked_at: None,
            }],
            role_assignments: vec![M6OrgRoleAssignment {
                assignment_id: format!("role-assignment:{member_id}:consultant"),
                member_id: member_id.to_string(),
                role_ref: "role:consultant".to_string(),
                scope_ref: "scope:global".to_string(),
                assigned_by_actor_id: "actor:user".to_string(),
                revision: 1,
                assigned_at: NOW_MS,
                revoked_at: None,
            }],
            capability_permission_refs: vec![M6OrgCapabilityPermissionRef {
                ref_id: "capability:research".to_string(),
                subject_member_id: member_id.to_string(),
                kind: M6OrgCapabilityPermissionKind::Capability,
                source: "policy-owner:fixture".to_string(),
                revision: 1,
                observed_at: NOW_MS,
                directory_is_authority: false,
                read_only: true,
            }],
            memory_refs: vec![format!("memory-ref:{member_id}:profile")],
            contact_bindings: vec![contact_binding(member_id)],
            idempotency_key: idempotency_key.to_string(),
        }
    }

    fn register_member(fixture: &Fixture, member_id: &str, key: &str) -> M6OrgStableMember {
        register_for_state(
            &fixture.state,
            &explicit_request(member_id, "display-name:shared", key),
            NOW_MS,
        )
        .expect("register stable member")
        .member
        .expect("registered member")
    }

    fn activate_member(
        fixture: &Fixture,
        member: &M6OrgStableMember,
        key: &str,
    ) -> M6OrgStableMember {
        update_for_state(
            &fixture.state,
            &M6OrgUpdateStableMemberRequest {
                member_id: member.member_id.clone(),
                expected_revision: member.revision,
                activate: true,
                display_name_ref: None,
                added_scope_assignments: Vec::new(),
                added_role_assignments: Vec::new(),
                added_capability_permission_refs: Vec::new(),
                added_memory_refs: Vec::new(),
                added_contact_bindings: Vec::new(),
                idempotency_key: key.to_string(),
            },
            NOW_MS,
        )
        .expect("activate stable member")
    }

    fn file_hash(path: &Path) -> String {
        if !path.exists() {
            return "MISSING".to_string();
        }
        format!(
            "{:x}",
            Sha256::digest(std::fs::read(path).expect("read product file for hash"))
        )
    }

    #[test]
    fn m6d05_explicit_identity_is_idempotent_and_same_name_never_merges() {
        let fixture = fixture("explicit-identity");
        let first_request = explicit_request("member_alpha", "display-name:shared", "register-a");
        let first =
            register_for_state(&fixture.state, &first_request, NOW_MS).expect("first registration");
        assert_eq!(
            first.disposition,
            M6OrgMemberRegistrationDisposition::Registered
        );
        assert!(!first.replayed);
        let replay = register_for_state(&fixture.state, &first_request, NOW_MS)
            .expect("registration replay");
        assert!(replay.replayed);
        register_for_state(
            &fixture.state,
            &explicit_request("member_beta", "display-name:shared", "register-b"),
            NOW_MS,
        )
        .expect("same-name second member");
        let listed = list_for_state(
            &fixture.state,
            &M6OrgListStableMembersRequest {
                include_deactivated: true,
                available_capability_ref: None,
            },
            NOW_MS,
        )
        .expect("list stable members");
        assert_eq!(listed.members.len(), 2);
        assert_eq!(listed.members[0].member.member_id, "member_alpha");
        assert_eq!(listed.members[1].member.member_id, "member_beta");
        assert_eq!(
            listed.members[0].member.display_name_ref,
            listed.members[1].member.display_name_ref
        );
        let store = M6OrgMemberDirectoryStore::open(&fixture.state.m6_org_store_path().unwrap())
            .expect("open member store");
        assert_eq!(store.count_rows("m6_stable_member_identities").unwrap(), 2);
        assert_eq!(store.count_rows("m6_stable_member_history").unwrap(), 2);
    }

    #[test]
    fn m6d05_heuristic_records_are_ref_only_quarantine_and_unknown_fields_fail() {
        let fixture = fixture("heuristic-quarantine");
        let kinds = [
            M6OrgHeuristicCandidateKind::AgentCenterSession,
            M6OrgHeuristicCandidateKind::ProviderThread,
            M6OrgHeuristicCandidateKind::RuntimeChild,
            M6OrgHeuristicCandidateKind::SessionCount,
            M6OrgHeuristicCandidateKind::DisplayNameMatch,
        ];
        for (index, candidate_kind) in kinds.into_iter().enumerate() {
            let outcome = register_for_state(
                &fixture.state,
                &M6OrgRegisterStableMemberRequest {
                    member_id: format!("member_candidate_{index}"),
                    display_name_ref: "display-name:looks-stable".to_string(),
                    identity_evidence: M6OrgStableIdentityEvidence::HeuristicCandidate {
                        candidate_kind,
                        source_refs: vec![format!("heuristic-ref:{index}")],
                    },
                    scope_assignments: Vec::new(),
                    role_assignments: Vec::new(),
                    capability_permission_refs: Vec::new(),
                    memory_refs: Vec::new(),
                    contact_bindings: Vec::new(),
                    idempotency_key: format!("heuristic-{index}"),
                },
                NOW_MS,
            )
            .expect("quarantine heuristic");
            assert_eq!(
                outcome.disposition,
                M6OrgMemberRegistrationDisposition::Quarantined
            );
            assert!(outcome.member.is_none());
            let quarantine = outcome.quarantine.expect("quarantine record");
            assert_eq!(quarantine.payload_mode, "REF_ONLY");
            assert!(quarantine.mapped_to.is_none());
        }
        let store = M6OrgMemberDirectoryStore::open(&fixture.state.m6_org_store_path().unwrap())
            .expect("open member store");
        assert_eq!(store.count_rows("m6_stable_member_identities").unwrap(), 0);
        assert_eq!(
            store.count_rows("m6_member_identity_quarantine").unwrap(),
            5
        );

        let temporary = explicit_request(
            "temporary_agent_01",
            "display-name:temporary",
            "register-temporary-as-stable",
        );
        assert_eq!(
            register_for_state(&fixture.state, &temporary, NOW_MS).unwrap_err(),
            "m6_org_member_identity_namespace_rejected"
        );
        assert_eq!(store.count_rows("m6_stable_member_identities").unwrap(), 0);

        let mut raw = serde_json::to_value(explicit_request(
            "member_gamma",
            "display-name:gamma",
            "register-gamma",
        ))
        .unwrap();
        raw["provider"] = json!("fake-provider");
        raw["model"] = json!("fake-model");
        raw["thread"] = json!("thread-1");
        assert!(serde_json::from_value::<M6OrgRegisterStableMemberRequest>(raw).is_err());
    }

    #[test]
    fn m6d05_assignments_are_append_only_and_directory_never_changes_authorization() {
        let fixture = fixture("lifecycle-authorization");
        let member = register_member(&fixture, "member_delta", "register-delta");
        let before = crate::m6_org_global_role_session::authorize_attempted_project_write(
            &fixture.state.m6_org_global_role_session,
        );
        let updated = update_for_state(
            &fixture.state,
            &M6OrgUpdateStableMemberRequest {
                member_id: member.member_id.clone(),
                expected_revision: 1,
                activate: true,
                display_name_ref: Some("display-name:delta-v2".to_string()),
                added_scope_assignments: vec![M6OrgScopeAssignment {
                    assignment_id: "scope-assignment:member_delta:portfolio".to_string(),
                    member_id: member.member_id.clone(),
                    scope_ref: "scope:portfolio".to_string(),
                    assigned_by_actor_id: "actor:user".to_string(),
                    revision: 1,
                    assigned_at: NOW_MS,
                    revoked_at: None,
                }],
                added_role_assignments: Vec::new(),
                added_capability_permission_refs: vec![M6OrgCapabilityPermissionRef {
                    ref_id: "permission:project-read".to_string(),
                    subject_member_id: member.member_id.clone(),
                    kind: M6OrgCapabilityPermissionKind::Permission,
                    source: "policy-owner:fixture".to_string(),
                    revision: 2,
                    observed_at: NOW_MS,
                    directory_is_authority: false,
                    read_only: true,
                }],
                added_memory_refs: vec!["memory-ref:member_delta:history-2".to_string()],
                added_contact_bindings: Vec::new(),
                idempotency_key: "update-delta".to_string(),
            },
            NOW_MS,
        )
        .expect("update member");
        assert_eq!(
            updated.membership_lifecycle,
            M6OrgMembershipLifecycle::Active
        );
        assert_eq!(updated.scope_assignments.len(), 2);
        assert_eq!(updated.capability_permission_refs.len(), 2);
        assert_eq!(updated.memory_refs.len(), 2);
        let after = crate::m6_org_global_role_session::authorize_attempted_project_write(
            &fixture.state.m6_org_global_role_session,
        );
        assert_eq!(before, after);
        assert_eq!(
            after.unwrap_err(),
            crate::m6_org_global_role_session::M6_ORG_GLOBAL_ROLE_SESSION_PROJECT_WRITE_REJECTED
        );

        let invalid = M6OrgCapabilityPermissionRef {
            ref_id: "permission:forged".to_string(),
            subject_member_id: member.member_id.clone(),
            kind: M6OrgCapabilityPermissionKind::Permission,
            source: "directory:self".to_string(),
            revision: 1,
            observed_at: NOW_MS,
            directory_is_authority: true,
            read_only: true,
        };
        let result = update_for_state(
            &fixture.state,
            &M6OrgUpdateStableMemberRequest {
                member_id: member.member_id.clone(),
                expected_revision: updated.revision,
                activate: false,
                display_name_ref: None,
                added_scope_assignments: Vec::new(),
                added_role_assignments: Vec::new(),
                added_capability_permission_refs: vec![invalid.clone()],
                added_memory_refs: Vec::new(),
                added_contact_bindings: Vec::new(),
                idempotency_key: "update-delta-forged".to_string(),
            },
            NOW_MS,
        );
        assert_eq!(
            result.unwrap_err(),
            "m6_org_member_capability_permission_ref_invalid"
        );
        let deactivated = deactivate_for_state(
            &fixture.state,
            &M6OrgDeactivateStableMemberRequest {
                member_id: member.member_id.clone(),
                expected_revision: updated.revision,
                reason_ref: "reason:user-deactivated".to_string(),
                idempotency_key: "deactivate-delta".to_string(),
            },
            NOW_MS + 1,
        )
        .expect("deactivate member");
        assert_eq!(
            deactivated.membership_lifecycle,
            M6OrgMembershipLifecycle::Deactivated
        );
        assert_eq!(deactivated.scope_assignments, updated.scope_assignments);
        assert_eq!(
            deactivated.capability_permission_refs,
            updated.capability_permission_refs
        );
        assert_eq!(deactivated.memory_refs, updated.memory_refs);
        let hidden = list_for_state(
            &fixture.state,
            &M6OrgListStableMembersRequest {
                include_deactivated: false,
                available_capability_ref: None,
            },
            NOW_MS + 1,
        )
        .unwrap();
        assert!(hidden.members.is_empty());
        let retained = list_for_state(
            &fixture.state,
            &M6OrgListStableMembersRequest {
                include_deactivated: true,
                available_capability_ref: None,
            },
            NOW_MS + 1,
        )
        .unwrap();
        assert_eq!(retained.members.len(), 1);
    }

    #[test]
    fn m6d05_stale_availability_is_unknown_and_provider_runtime_swap_keeps_identity() {
        let fixture = fixture("availability-provider-swap");
        let registered = register_member(&fixture, "member_epsilon", "register-epsilon");
        let active = activate_member(&fixture, &registered, "activate-epsilon");
        let stale = observe_availability_for_state(
            &fixture.state,
            &M6OrgObserveMemberAvailabilityRequest {
                member_id: active.member_id.clone(),
                source: "fake-provider-runtime-a".to_string(),
                source_revision: 1,
                observed_at: NOW_MS - 10_000,
                ttl: M6OrgAvailabilityTtl { seconds: 5 },
                observed_state: M6OrgAvailabilityState::Available,
                idempotency_key: "availability-epsilon-a".to_string(),
            },
            NOW_MS,
        )
        .expect("record stale availability");
        assert_eq!(stale.effective_state, M6OrgAvailabilityState::Unknown);
        assert!(!stale.authorizes);
        let excluded = list_for_state(
            &fixture.state,
            &M6OrgListStableMembersRequest {
                include_deactivated: false,
                available_capability_ref: Some("capability:research".to_string()),
            },
            NOW_MS,
        )
        .expect("capability lookup with stale availability");
        assert!(excluded.members.is_empty());
        assert!(!excluded.stale_availability_used_as_capability);

        let fresh = observe_availability_for_state(
            &fixture.state,
            &M6OrgObserveMemberAvailabilityRequest {
                member_id: active.member_id.clone(),
                source: "fake-provider-runtime-b".to_string(),
                source_revision: 2,
                observed_at: NOW_MS,
                ttl: M6OrgAvailabilityTtl { seconds: 60 },
                observed_state: M6OrgAvailabilityState::Available,
                idempotency_key: "availability-epsilon-b".to_string(),
            },
            NOW_MS,
        )
        .expect("record replacement provider availability");
        assert_eq!(fresh.effective_state, M6OrgAvailabilityState::Available);
        let included = list_for_state(
            &fixture.state,
            &M6OrgListStableMembersRequest {
                include_deactivated: false,
                available_capability_ref: Some("capability:research".to_string()),
            },
            NOW_MS,
        )
        .expect("capability lookup with fresh availability");
        assert_eq!(included.members.len(), 1);
        let projected = &included.members[0].member;
        assert_eq!(projected.member_id, active.member_id);
        assert_eq!(
            projected.identity_contract_ref,
            active.identity_contract_ref
        );
        assert_eq!(projected.memory_refs, active.memory_refs);
        assert_eq!(
            projected.capability_permission_refs,
            active.capability_permission_refs
        );
    }

    #[test]
    fn m6d05_export_rebuild_preserves_identity_and_all_retained_refs() {
        let fixture = fixture("export-rebuild");
        let first = register_member(&fixture, "member_zeta", "register-zeta");
        let active = activate_member(&fixture, &first, "activate-zeta");
        observe_availability_for_state(
            &fixture.state,
            &M6OrgObserveMemberAvailabilityRequest {
                member_id: active.member_id.clone(),
                source: "fake-runtime-zeta".to_string(),
                source_revision: 1,
                observed_at: NOW_MS,
                ttl: M6OrgAvailabilityTtl { seconds: 60 },
                observed_state: M6OrgAvailabilityState::Busy,
                idempotency_key: "availability-zeta".to_string(),
            },
            NOW_MS,
        )
        .unwrap();
        let export = export_for_state(&fixture.state, NOW_MS).expect("export directory");
        assert_eq!(export.schema, DIRECTORY_EXPORT_SCHEMA);
        assert_eq!(export.member_history.len(), 2);
        let mut rebuilt = M6OrgMemberDirectoryStore::open_in_memory().unwrap();
        rebuilt.restore_export(&export).expect("restore directory");
        let rebuilt_member = rebuilt
            .load_member("member_zeta")
            .unwrap()
            .expect("rebuilt member");
        assert_eq!(rebuilt_member, active);
        assert_eq!(
            rebuilt.latest_availability("member_zeta").unwrap(),
            Some(export.availability_history[0].clone())
        );
        assert_eq!(
            rebuilt.restore_export(&export).unwrap_err(),
            "m6_org_member_restore_target_not_empty"
        );
    }

    #[test]
    fn m6d05_contact_creates_only_m3_handoff_and_capability_false_receipt() {
        let fixture = fixture("contact-handoff");
        crate::m6_org_consult_handoff::bind_fake_secretary_for_member_directory(&fixture.state)
            .expect("bind fake Secretary");
        let registered = register_member(&fixture, "member_eta", "register-eta");
        let active = activate_member(&fixture, &registered, "activate-eta");
        let before = [
            file_hash(&fixture.state.index_path),
            file_hash(&fixture.state.tasks_path),
            file_hash(&fixture.state.workflow_state_path),
        ];
        let request = M6OrgContactStableMemberRequest {
            member_id: active.member_id.clone(),
            contact_binding_ref: active.contact_binding_refs[0].clone(),
            reason_ref: "reason:direct-consult".to_string(),
            source_refs: vec!["source-ref:member-directory".to_string()],
            accept_by_utc: ACCEPT_BY.to_string(),
            idempotency_key: "contact-eta".to_string(),
        };
        let receipt =
            contact_for_state(&fixture.state, &request, NOW_MS).expect("contact stable member");
        assert_eq!(receipt.status, "RECORDED");
        assert!(!receipt.capability_granted);
        assert!(!receipt.project_writeback);
        assert!(!receipt.handoff_id.is_empty());
        let replay = contact_for_state(&fixture.state, &request, NOW_MS)
            .expect("replay stable member contact");
        assert!(replay.replayed);
        assert_eq!(replay.contact_receipt_id, receipt.contact_receipt_id);
        assert_eq!(
            before,
            [
                file_hash(&fixture.state.index_path),
                file_hash(&fixture.state.tasks_path),
                file_hash(&fixture.state.workflow_state_path),
            ]
        );
        let store = M6OrgMemberDirectoryStore::open(&fixture.state.m6_org_store_path().unwrap())
            .expect("open member store");
        assert_eq!(store.count_rows("m6_member_contact_receipts").unwrap(), 1);
        let m3 = Connection::open(
            fixture
                .app_data_root
                .join(crate::m3_role_session_repository::M3_ORDINARY_ROLE_SESSION_RELATIVE_PATH),
        )
        .expect("open M3 store");
        let handoff_count: i64 = m3
            .query_row("SELECT COUNT(*) FROM m3_handoffs", [], |row| row.get(0))
            .expect("count M3 handoffs");
        assert_eq!(handoff_count, 1);
        let listed = list_for_state(
            &fixture.state,
            &M6OrgListStableMembersRequest {
                include_deactivated: false,
                available_capability_ref: None,
            },
            NOW_MS,
        )
        .unwrap();
        assert_eq!(listed.members[0].contact_receipts.len(), 1);
        assert!(listed.members[0]
            .session_history_refs
            .contains(&receipt.handoff_id));
        assert!(listed.members[0]
            .session_history_refs
            .contains(&receipt.role_session_id));
    }

    #[test]
    fn m6d05_ordinary_commands_are_registered_and_not_test_gated() {
        let commands = include_str!("commands.rs");
        let registry = include_str!("command_registry.rs");
        let module = include_str!("m6_org_member_directory.rs");
        for command in [
            "register_global_supervisor_stable_member",
            "update_global_supervisor_stable_member",
            "deactivate_global_supervisor_stable_member",
            "observe_global_supervisor_member_availability",
            "list_global_supervisor_stable_members",
            "export_global_supervisor_member_directory",
            "contact_global_supervisor_stable_member",
        ] {
            assert!(commands.contains(&format!("fn {command}")));
            assert!(registry.contains(command));
        }
        let production = module
            .split("#[cfg(test)]\nmod tests")
            .next()
            .expect("production member directory source");
        assert!(production.contains("authority_seed"));
        assert!(production.contains("m6_org_store_path"));
        assert!(production.contains("start_member_contact_handoff_for_state"));
        assert!(!production.contains("project_root"));
        assert!(!production.contains("open_m5_store"));
        assert!(!production.contains("std::env"));
    }
}
