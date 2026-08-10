//! M4C02 ordinary-product Secretary composition.
//!
//! This module is the only bridge from the fixed M4 personal identity into
//! the M3 RoleSession repository/read runtime. It accepts one backend-resolved
//! app-data root, never a renderer path, cwd, project root, or acceptance
//! permit. Creating or restoring the RoleSession records local M3 ledger rows
//! only; this module does not claim or dispatch the registered provider effect.

use crate::m3_role_session::{
    CorrelationId, OpaqueRef, PermissionSnapshotDescriptor, RequestIdempotencyKey, RoleSessionId,
    RoleSessionState, ServerResolvedBinding, Sha256Digest,
};
use crate::m3_role_session_read_model::{
    M3OrdinarySecretaryReadBinding, M3RoleSessionReadRuntimeSlot, M3SecretaryReadHost,
};
use crate::m3_role_session_repository::{
    CreateRoleSessionCommand, M3CommandMetadata, M3OrdinaryRoleSessionRepositoryConfig,
    M3ReadPermissionDisposition, M3RoleSessionDirectoryQuery, M3RoleSessionSnapshotQuery,
    M3RoleSessionSqliteRepository, M3SessionBindingReadState, QuarantineRoleSessionCommand,
    ResumeRoleSessionCommand, M3_ORDINARY_ROLE_SESSION_RELATIVE_PATH,
};
use crate::mcp::identity_kernel::{
    resolve_m4_primary_secretary_identity, M4PrimarySecretaryIdentity,
    M4_PRIMARY_SECRETARY_ACTOR_ID, M4_PRIMARY_SECRETARY_SCOPE_ID,
};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

const M4_SECRETARY_ROLE_SESSION_ID_MATERIAL: &str =
    "syn.m4.secretary-role-session/personal-primary/v1";
const M4_SECRETARY_CREATE_MATERIAL: &str =
    "syn.m4.secretary-role-session-create/personal-primary/v1";

pub(crate) fn install_ordinary_product_secretary_runtime(
    app_data_root: &Path,
) -> Result<M3RoleSessionReadRuntimeSlot, String> {
    let canonical_app_data_root = admit_server_app_data_root(app_data_root)?;
    let repository = M3RoleSessionSqliteRepository::open_ordinary_product(
        &M3OrdinaryRoleSessionRepositoryConfig {
            db_path: canonical_app_data_root.join(M3_ORDINARY_ROLE_SESSION_RELATIVE_PATH),
            app_data_root: canonical_app_data_root,
        },
    )
    .map_err(|error| error.code)?;
    let identity =
        resolve_m4_primary_secretary_identity().map_err(|error| error.code().to_string())?;
    let binding = identity
        .m3_server_resolved_binding()
        .map_err(|error| error.code().to_string())?;
    let role_session_id =
        bootstrap_or_restore_secretary_role_session(&repository, &identity, &binding)?;

    M3RoleSessionReadRuntimeSlot::from_ordinary_product_secretary(M3OrdinarySecretaryReadBinding {
        host: M3SecretaryReadHost::server_fixed(),
        repository,
        binding,
        role_session_id,
    })
}

/// Resolve the app-data root before any store path is constructed. The caller
/// is app composition code, not IPC; the root must already be an absolute,
/// clean server path. A symlinked alias is rejected rather than normalized.
fn admit_server_app_data_root(app_data_root: &Path) -> Result<PathBuf, String> {
    if !app_data_root.is_absolute()
        || app_data_root
            .components()
            .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
    {
        return Err("m4_secretary_app_data_root_clean_absolute_required".to_string());
    }
    fs::create_dir_all(app_data_root)
        .map_err(|_| "m4_secretary_app_data_root_create_failed".to_string())?;
    let canonical = fs::canonicalize(app_data_root)
        .map_err(|_| "m4_secretary_app_data_root_unavailable".to_string())?;
    if canonical != app_data_root {
        return Err("m4_secretary_app_data_root_identity_changed".to_string());
    }
    Ok(canonical)
}

fn bootstrap_or_restore_secretary_role_session(
    repository: &M3RoleSessionSqliteRepository,
    identity: &M4PrimarySecretaryIdentity,
    binding: &ServerResolvedBinding,
) -> Result<RoleSessionId, String> {
    // Directory order is only pagination mechanics. Resolve the complete
    // server-authorized set before choosing anything, so startup cannot turn
    // recency into identity authority.
    let entries = list_secretary_role_session_candidates(repository, binding)?;
    let live_candidates = entries
        .iter()
        .filter(|entry| {
            matches!(
                entry.session.status,
                RoleSessionState::Created | RoleSessionState::Active | RoleSessionState::Suspended
            )
        })
        .collect::<Vec<_>>();
    let has_mismatched_candidate = live_candidates
        .iter()
        .any(|entry| !matches!(&entry.permission, M3ReadPermissionDisposition::Current));
    if live_candidates.len() > 1 || has_mismatched_candidate {
        for entry in live_candidates {
            quarantine_secretary_role_session_candidate(
                repository,
                binding,
                entry.session.role_session_id.clone(),
                entry.session.revision,
            )?;
        }
        return Err(if has_mismatched_candidate {
            "m4_secretary_role_session_mismatched"
        } else {
            "m4_secretary_role_session_ambiguous"
        }
        .to_string());
    }

    let Some(entry) = entries.iter().find(|entry| {
        matches!(
            entry.session.status,
            RoleSessionState::Created | RoleSessionState::Active | RoleSessionState::Suspended
        )
    }) else {
        if entries
            .iter()
            .any(|entry| entry.session.status == RoleSessionState::Quarantined)
        {
            return Err("m4_secretary_role_session_quarantined".to_string());
        }
        if entries
            .iter()
            .any(|entry| entry.session.status == RoleSessionState::Closed)
        {
            return Err("m4_secretary_role_session_closed".to_string());
        }
        return create_secretary_role_session(repository, binding)
            .map(|outcome| outcome.role_session_id);
    };
    match entry.session.status {
        RoleSessionState::Active => {
            if !matches!(&entry.permission, M3ReadPermissionDisposition::Current) {
                return Err("m4_secretary_permission_revalidation_required".to_string());
            }
            Ok(entry.session.role_session_id.clone())
        }
        RoleSessionState::Suspended => {
            if !matches!(&entry.permission, M3ReadPermissionDisposition::Current) {
                return Err("m4_secretary_permission_revalidation_required".to_string());
            }
            let snapshot = repository
                .load_authorized_role_session_snapshot(&M3RoleSessionSnapshotQuery {
                    role_session_id: entry.session.role_session_id.clone(),
                    binding: binding.clone(),
                })
                .map_err(|error| error.code)?
                .ok_or_else(|| "m4_secretary_resume_session_missing".to_string())?;
            if !matches!(
                snapshot.current_binding,
                M3SessionBindingReadState::Verified { .. }
            ) {
                return Err("m4_secretary_resume_binding_unavailable".to_string());
            }
            let permission = permission_descriptor(identity, binding)?;
            let revision = entry.session.revision;
            let material = format!(
                "syn.m4.secretary-role-session-resume/personal-primary/v1/revision:{revision}"
            );
            let outcome = repository
                .resume_role_session(&ResumeRoleSessionCommand {
                    role_session_id: entry.session.role_session_id.clone(),
                    binding: binding.clone(),
                    previous_permission: Some(permission.clone()),
                    current_permission: Some(permission),
                    expected_session_revision: revision,
                    metadata: metadata_for(repository, "resume", &material)?,
                })
                .map_err(|error| error.code)?;
            let session = outcome
                .role_session
                .ok_or_else(|| "m4_secretary_resume_session_missing".to_string())?;
            if session.status != RoleSessionState::Active {
                return Err("m4_secretary_resume_not_active".to_string());
            }
            Ok(session.role_session_id)
        }
        RoleSessionState::Quarantined => Err("m4_secretary_role_session_quarantined".to_string()),
        RoleSessionState::Closed => Err("m4_secretary_role_session_closed".to_string()),
        RoleSessionState::Created => Err("m4_secretary_role_session_incomplete".to_string()),
    }
}

fn list_secretary_role_session_candidates(
    repository: &M3RoleSessionSqliteRepository,
    binding: &ServerResolvedBinding,
) -> Result<Vec<crate::m3_role_session_repository::M3RoleSessionDirectoryEntry>, String> {
    let mut entries = Vec::new();
    let mut after = None;
    loop {
        let page = repository
            .list_authorized_role_session_directory(&M3RoleSessionDirectoryQuery {
                binding: binding.clone(),
                after: after.clone(),
                limit: 100,
            })
            .map_err(|error| error.code)?;
        entries.extend(page.entries);
        let Some(next_cursor) = page.next_cursor else {
            return Ok(entries);
        };
        after = Some(next_cursor);
    }
}

fn quarantine_secretary_role_session_candidate(
    repository: &M3RoleSessionSqliteRepository,
    binding: &ServerResolvedBinding,
    role_session_id: RoleSessionId,
    expected_session_revision: u64,
) -> Result<(), String> {
    let material = format!(
        "syn.m4.secretary-role-session-quarantine/personal-primary/v1/role-session:{}/revision:{expected_session_revision}",
        role_session_id.as_str(),
    );
    let outcome = repository
        .quarantine_role_session(&QuarantineRoleSessionCommand {
            role_session_id: role_session_id.clone(),
            binding: binding.clone(),
            expected_session_revision,
            metadata: metadata_for(repository, "quarantine", &material)?,
        })
        .map_err(|_| "m4_secretary_role_session_quarantine_failed".to_string())?;
    let session = outcome
        .role_session
        .ok_or_else(|| "m4_secretary_role_session_quarantine_missing".to_string())?;
    if session.role_session_id != role_session_id || session.status != RoleSessionState::Quarantined
    {
        return Err("m4_secretary_role_session_quarantine_invalid".to_string());
    }
    Ok(())
}

struct SecretaryCreateOutcome {
    role_session_id: RoleSessionId,
    replayed: bool,
}

fn create_secretary_role_session(
    repository: &M3RoleSessionSqliteRepository,
    binding: &ServerResolvedBinding,
) -> Result<SecretaryCreateOutcome, String> {
    let role_session_id = role_session_id()?;
    let outcome = repository
        .create_role_session(&CreateRoleSessionCommand {
            role_session_id: role_session_id.clone(),
            binding: binding.clone(),
            metadata: metadata_for(repository, "create", M4_SECRETARY_CREATE_MATERIAL)?,
        })
        .map_err(|error| error.code)?;
    let session = outcome
        .role_session
        .ok_or_else(|| "m4_secretary_create_session_missing".to_string())?;
    if session.role_session_id != role_session_id || session.status != RoleSessionState::Active {
        return Err("m4_secretary_create_session_invalid".to_string());
    }
    Ok(SecretaryCreateOutcome {
        role_session_id,
        replayed: outcome.replayed,
    })
}

fn permission_descriptor(
    identity: &M4PrimarySecretaryIdentity,
    binding: &ServerResolvedBinding,
) -> Result<PermissionSnapshotDescriptor, String> {
    let refs = |namespace: &str, values: &[String]| -> Result<BTreeSet<OpaqueRef>, String> {
        values
            .iter()
            .map(|value| opaque_ref(namespace, value))
            .collect()
    };
    Ok(PermissionSnapshotDescriptor {
        snapshot_ref: binding.permission_snapshot_ref.clone(),
        allowed_capability_refs: refs(
            "capability",
            &identity.permission_profile.allow_capabilities,
        )?,
        denied_capability_refs: refs("capability", &identity.permission_profile.deny_capabilities)?,
        constraint_refs: refs("constraint", &identity.permission_profile.constraints)?,
    })
}

fn role_session_id() -> Result<RoleSessionId, String> {
    RoleSessionId::try_from_canonical(sealed_ref("session", M4_SECRETARY_ROLE_SESSION_ID_MATERIAL))
        .map_err(|_| "m4_secretary_role_session_id_invalid".to_string())
}

fn metadata_for(
    repository: &M3RoleSessionSqliteRepository,
    operation: &str,
    material: &str,
) -> Result<M3CommandMetadata, String> {
    Ok(M3CommandMetadata {
        receipt_id: opaque_ref("receipt", &format!("{material}/receipt"))?,
        event_id: opaque_ref("event", &format!("{material}/event"))?,
        audit_id: opaque_ref("audit", &format!("{material}/audit"))?,
        correlation_id: CorrelationId::try_from_canonical(sealed_ref(
            "correlation",
            &format!("{material}/correlation"),
        ))
        .map_err(|_| "m4_secretary_correlation_id_invalid".to_string())?,
        request_idempotency_key: RequestIdempotencyKey::try_from_canonical(sealed_ref(
            "request",
            &format!("{material}/idempotency/{operation}"),
        ))
        .map_err(|_| "m4_secretary_idempotency_key_invalid".to_string())?,
        occurred_at: repository
            .capture_server_utc_now()
            .map_err(|_| "m4_secretary_server_clock_unavailable".to_string())?,
    })
}

fn opaque_ref(namespace: &str, material: &str) -> Result<OpaqueRef, String> {
    OpaqueRef::try_from_canonical(sealed_ref(namespace, material))
        .map_err(|_| "m4_secretary_opaque_ref_invalid".to_string())
}

fn sealed_ref(namespace: &str, material: &str) -> String {
    format!(
        "{namespace}:sha256:{}",
        Sha256Digest::of_bytes(material.as_bytes()).as_str()
    )
}

// -------------------------------------------------------------------------
// M4C03 structured-source admission and deterministic projection policy.
// -------------------------------------------------------------------------

pub(crate) const M4_WORKFLOW_ATTENTION_SOURCE_TYPE: &str =
    "structured_internal_workflow_attention_ref";
pub(crate) const M4_WORKFLOW_ATTENTION_OBJECT_TYPE: &str = "workflow_attention";
pub(crate) const M4_SCRUBBED_SENSITIVITY: &str = "SCRUBBED_INTERNAL_REF_ONLY";
pub(crate) const M4_ATTENTION_POLICY_REF: &str = "m4-attention-policy:v1";
pub(crate) const M4_ATTENTION_PROJECTOR_ID: &str = "m4-inbox-open-loop-projector";
pub(crate) const M4_ATTENTION_PROJECTOR_VERSION: i64 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum M4SourceStatus {
    Open,
    Blocked,
    WaitingUser,
    Informational,
    Completed,
    Cancelled,
    Expired,
}

impl M4SourceStatus {
    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "OPEN" => Some(Self::Open),
            "BLOCKED" => Some(Self::Blocked),
            "WAITING_USER" => Some(Self::WaitingUser),
            "INFORMATIONAL" => Some(Self::Informational),
            "COMPLETED" => Some(Self::Completed),
            "CANCELLED" => Some(Self::Cancelled),
            "EXPIRED" => Some(Self::Expired),
            _ => None,
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Open => "OPEN",
            Self::Blocked => "BLOCKED",
            Self::WaitingUser => "WAITING_USER",
            Self::Informational => "INFORMATIONAL",
            Self::Completed => "COMPLETED",
            Self::Cancelled => "CANCELLED",
            Self::Expired => "EXPIRED",
        }
    }

    pub(crate) fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Cancelled | Self::Expired)
    }

    pub(crate) fn terminal_closure_reason(self) -> Option<&'static str> {
        match self {
            Self::Completed => Some("SOURCE_COMPLETED"),
            Self::Cancelled => Some("SOURCE_CANCELLED"),
            Self::Expired => Some("SOURCE_EXPIRED"),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct M4AttentionSignals {
    pub(crate) external_commitment: bool,
    pub(crate) time_sensitive: bool,
    pub(crate) requires_user_decision: bool,
    pub(crate) source_blocked: bool,
    pub(crate) attention_required: bool,
    pub(crate) material_change: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4SourceLinkInput {
    pub(crate) link_kind: String,
    pub(crate) source_owner_ref: String,
    pub(crate) object_type: String,
    pub(crate) canonical_source_object_id: String,
    pub(crate) expected_source_revision: u64,
    pub(crate) opaque_route_ref: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4WorkflowAttentionSourceInput {
    pub(crate) source_owner_ref: String,
    pub(crate) scope_ref: String,
    pub(crate) source_type: String,
    pub(crate) canonical_source_object_id: String,
    pub(crate) source_revision: u64,
    pub(crate) source_event_id: String,
    pub(crate) source_owner_watermark: String,
    pub(crate) occurred_at_utc: String,
    pub(crate) source_link: M4SourceLinkInput,
    pub(crate) owner_status_code: String,
    pub(crate) attention_signals: M4AttentionSignals,
    pub(crate) due_at_utc: Option<String>,
    pub(crate) sensitivity: String,
    pub(crate) scrubbed_summary_ref: String,
    pub(crate) payload_hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4PriorityReason {
    pub(crate) rank: i64,
    pub(crate) code: &'static str,
    pub(crate) reason_ref: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4AdmittedWorkflowAttentionSource {
    pub(crate) source_identity_key: String,
    pub(crate) source_event_key: String,
    pub(crate) source_owner_ref: String,
    pub(crate) scope_ref: String,
    pub(crate) source_type: String,
    pub(crate) canonical_source_object_id: String,
    pub(crate) source_revision: u64,
    pub(crate) source_event_id: String,
    pub(crate) source_owner_watermark: String,
    pub(crate) occurred_at_utc: String,
    /// The first adapter has one fixed typed link kind/object type. Persisting
    /// the server-minted opaque route ref is sufficient to reconstruct the
    /// complete typed link without storing a path, URL, or callback.
    pub(crate) source_link_ref: String,
    pub(crate) source_status: M4SourceStatus,
    pub(crate) attention_signals: M4AttentionSignals,
    pub(crate) due_at_utc: Option<String>,
    pub(crate) sensitivity: String,
    pub(crate) scrubbed_summary_ref: String,
    pub(crate) payload_hash: String,
    pub(crate) priority: M4PriorityReason,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4QuarantineCandidate {
    pub(crate) source_identity_key: String,
    pub(crate) source_event_key: String,
    pub(crate) source_owner_ref: String,
    pub(crate) scope_ref: String,
    pub(crate) source_type: String,
    pub(crate) canonical_source_object_id: String,
    pub(crate) source_revision: u64,
    pub(crate) source_event_id: String,
    pub(crate) source_owner_watermark: String,
    pub(crate) source_link_ref: String,
    pub(crate) payload_hash: String,
    pub(crate) scrubbed_summary_ref: String,
    pub(crate) reason_code: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum M4WorkflowAttentionAdmission {
    Admitted(M4AdmittedWorkflowAttentionSource),
    Quarantined(M4QuarantineCandidate),
}

pub(crate) fn classify_workflow_attention_source(
    input: &M4WorkflowAttentionSourceInput,
) -> Result<M4WorkflowAttentionAdmission, String> {
    for (field, value) in [
        ("source_owner_ref", input.source_owner_ref.as_str()),
        ("scope_ref", input.scope_ref.as_str()),
        ("source_type", input.source_type.as_str()),
        (
            "canonical_source_object_id",
            input.canonical_source_object_id.as_str(),
        ),
        ("source_event_id", input.source_event_id.as_str()),
        (
            "source_owner_watermark",
            input.source_owner_watermark.as_str(),
        ),
        ("occurred_at_utc", input.occurred_at_utc.as_str()),
        ("source_link_kind", input.source_link.link_kind.as_str()),
        (
            "source_link_owner_ref",
            input.source_link.source_owner_ref.as_str(),
        ),
        (
            "source_link_object_type",
            input.source_link.object_type.as_str(),
        ),
        (
            "source_link_object_id",
            input.source_link.canonical_source_object_id.as_str(),
        ),
        (
            "source_link_route_ref",
            input.source_link.opaque_route_ref.as_str(),
        ),
        ("owner_status_code", input.owner_status_code.as_str()),
        ("sensitivity", input.sensitivity.as_str()),
        ("scrubbed_summary_ref", input.scrubbed_summary_ref.as_str()),
    ] {
        m4_validate_reference_text(field, value)?;
    }
    for (field, value) in [
        ("source_owner_ref", input.source_owner_ref.as_str()),
        ("scope_ref", input.scope_ref.as_str()),
        ("source_type", input.source_type.as_str()),
        (
            "canonical_source_object_id",
            input.canonical_source_object_id.as_str(),
        ),
        ("source_link_kind", input.source_link.link_kind.as_str()),
        (
            "source_link_owner_ref",
            input.source_link.source_owner_ref.as_str(),
        ),
        (
            "source_link_object_type",
            input.source_link.object_type.as_str(),
        ),
        (
            "source_link_object_id",
            input.source_link.canonical_source_object_id.as_str(),
        ),
        ("owner_status_code", input.owner_status_code.as_str()),
        ("sensitivity", input.sensitivity.as_str()),
    ] {
        m4_validate_identifier(field, value)?;
    }
    if let Some(due_at_utc) = input.due_at_utc.as_deref() {
        m4_validate_reference_text("due_at_utc", due_at_utc)?;
    }
    if !m4_is_lower_hex_digest(&input.payload_hash) {
        return Err("m4_payload_hash_lower_hex_required".to_string());
    }

    let source_identity_key = m4_source_identity_key(
        &input.source_owner_ref,
        &input.scope_ref,
        &input.source_type,
        &input.canonical_source_object_id,
    )?;
    let source_event_key = m4_source_event_key(
        &source_identity_key,
        input.source_revision,
        &input.source_event_id,
        &input.payload_hash,
    )?;

    let quarantine_reason = if [
        input.source_owner_ref.as_str(),
        input.scope_ref.as_str(),
        input.source_type.as_str(),
        input.canonical_source_object_id.as_str(),
        input.source_link.source_owner_ref.as_str(),
        input.source_link.object_type.as_str(),
        input.source_link.canonical_source_object_id.as_str(),
    ]
    .into_iter()
    .any(m4_looks_like_forbidden_raw_reference)
    {
        Some("RAW_REFERENCE_NOT_ADMITTED")
    } else if input.source_type != M4_WORKFLOW_ATTENTION_SOURCE_TYPE {
        Some("SOURCE_TYPE_NOT_REGISTERED")
    } else if input.scope_ref != M4_PRIMARY_SECRETARY_SCOPE_ID {
        Some("PERSONAL_SCOPE_MISMATCH")
    } else if input.sensitivity != M4_SCRUBBED_SENSITIVITY {
        Some("SENSITIVITY_NOT_ADMITTED")
    } else if input.source_link.link_kind != "INTERNAL_ROUTE" {
        Some("SOURCE_LINK_KIND_NOT_REGISTERED")
    } else if input.source_link.source_owner_ref != input.source_owner_ref {
        Some("SOURCE_LINK_OWNER_MISMATCH")
    } else if input.source_link.object_type != M4_WORKFLOW_ATTENTION_OBJECT_TYPE {
        Some("SOURCE_LINK_OBJECT_TYPE_MISMATCH")
    } else if input.source_link.canonical_source_object_id != input.canonical_source_object_id {
        Some("SOURCE_LINK_OBJECT_ID_MISMATCH")
    } else if input.source_link.expected_source_revision != input.source_revision {
        Some("SOURCE_LINK_REVISION_MISMATCH")
    } else if !m4_is_opaque_reference(&input.source_link.opaque_route_ref) {
        Some("SOURCE_LINK_ROUTE_REF_INVALID")
    } else if !m4_is_opaque_reference(&input.source_event_id) {
        Some("SOURCE_EVENT_ID_INVALID")
    } else if !m4_is_opaque_reference(&input.source_owner_watermark) {
        Some("SOURCE_OWNER_WATERMARK_INVALID")
    } else if !m4_is_opaque_reference(&input.scrubbed_summary_ref) {
        Some("SCRUBBED_SUMMARY_REF_INVALID")
    } else if m4_parse_rfc3339_utc_key(&input.occurred_at_utc).is_none() {
        Some("OCCURRED_AT_UTC_INVALID")
    } else if input
        .due_at_utc
        .as_deref()
        .is_some_and(|value| m4_parse_rfc3339_utc_key(value).is_none())
    {
        Some("DUE_AT_UTC_INVALID")
    } else if M4SourceStatus::parse(&input.owner_status_code).is_none() {
        Some("OWNER_STATUS_UNKNOWN")
    } else {
        None
    };

    if let Some(reason_code) = quarantine_reason {
        return Ok(M4WorkflowAttentionAdmission::Quarantined(
            M4QuarantineCandidate {
                source_identity_key,
                source_event_key,
                source_owner_ref: m4_scrub_quarantine_ref("source-owner", &input.source_owner_ref)?,
                scope_ref: m4_scrub_quarantine_ref("scope", &input.scope_ref)?,
                source_type: m4_scrub_quarantine_ref("source-type", &input.source_type)?,
                canonical_source_object_id: m4_scrub_quarantine_ref(
                    "source-object",
                    &input.canonical_source_object_id,
                )?,
                source_revision: input.source_revision,
                source_event_id: m4_scrub_quarantine_ref(
                    "source-event-id",
                    &input.source_event_id,
                )?,
                source_owner_watermark: m4_scrub_quarantine_ref(
                    "watermark",
                    &input.source_owner_watermark,
                )?,
                source_link_ref: m4_scrub_quarantine_ref(
                    "route",
                    &input.source_link.opaque_route_ref,
                )?,
                payload_hash: input.payload_hash.clone(),
                scrubbed_summary_ref: m4_scrub_quarantine_ref(
                    "summary",
                    &input.scrubbed_summary_ref,
                )?,
                reason_code,
            },
        ));
    }

    let source_status = M4SourceStatus::parse(&input.owner_status_code)
        .ok_or_else(|| "m4_source_status_mapping_unreachable".to_string())?;
    let priority = m4_priority_reason(&input.attention_signals)?;
    Ok(M4WorkflowAttentionAdmission::Admitted(
        M4AdmittedWorkflowAttentionSource {
            source_identity_key,
            source_event_key,
            source_owner_ref: input.source_owner_ref.clone(),
            scope_ref: input.scope_ref.clone(),
            source_type: input.source_type.clone(),
            canonical_source_object_id: input.canonical_source_object_id.clone(),
            source_revision: input.source_revision,
            source_event_id: input.source_event_id.clone(),
            source_owner_watermark: input.source_owner_watermark.clone(),
            occurred_at_utc: input.occurred_at_utc.clone(),
            source_link_ref: input.source_link.opaque_route_ref.clone(),
            source_status,
            attention_signals: input.attention_signals.clone(),
            due_at_utc: input.due_at_utc.clone(),
            sensitivity: input.sensitivity.clone(),
            scrubbed_summary_ref: input.scrubbed_summary_ref.clone(),
            payload_hash: input.payload_hash.clone(),
            priority,
        },
    ))
}

pub(crate) fn m4_priority_reason(signals: &M4AttentionSignals) -> Result<M4PriorityReason, String> {
    let (rank, code) = if signals.external_commitment || signals.time_sensitive {
        (0, "EXTERNAL_COMMITMENT_OR_TIME_CRITICAL")
    } else if signals.requires_user_decision || signals.source_blocked {
        (1, "USER_DECISION_OR_BLOCKER")
    } else if signals.attention_required || signals.material_change {
        (2, "ACTIVE_CHANGED_ATTENTION")
    } else {
        (4, "INFORMATIONAL")
    };
    Ok(M4PriorityReason {
        rank,
        code,
        reason_ref: m4_internal_id("priority-reason:", "syn.m4.priority-reason/v1", &[code])?,
    })
}

pub(crate) fn m4_automatic_open_loop(source: &M4AdmittedWorkflowAttentionSource) -> bool {
    !source.source_status.is_terminal()
        && (source.attention_signals.external_commitment
            || source.attention_signals.time_sensitive
            || source.attention_signals.requires_user_decision
            || source.attention_signals.source_blocked
            || source.attention_signals.attention_required)
}

pub(crate) fn m4_source_identity_key(
    source_owner_ref: &str,
    scope_ref: &str,
    source_type: &str,
    canonical_source_object_id: &str,
) -> Result<String, String> {
    m4_internal_id(
        "source:",
        "syn.m4.source-identity/v1",
        &[
            source_owner_ref,
            scope_ref,
            source_type,
            canonical_source_object_id,
        ],
    )
}

pub(crate) fn m4_source_event_key(
    source_identity_key: &str,
    source_revision: u64,
    source_event_id: &str,
    payload_hash: &str,
) -> Result<String, String> {
    let revision = source_revision.to_string();
    m4_internal_id(
        "source-event:",
        "syn.m4.source-event/v1",
        &[
            source_identity_key,
            &revision,
            source_event_id,
            payload_hash,
        ],
    )
}

pub(crate) fn m4_inbox_item_id(source_identity_key: &str) -> Result<String, String> {
    m4_internal_id("inbox:", "syn.m4.inbox-item/v1", &[source_identity_key])
}

pub(crate) fn m4_open_loop_id(source_identity_key: &str) -> Result<String, String> {
    m4_internal_id(
        "open-loop:",
        "syn.m4.open-loop/v1",
        &[source_identity_key, M4_ATTENTION_POLICY_REF],
    )
}

pub(crate) fn m4_internal_id(
    prefix: &str,
    domain_separator: &str,
    components: &[&str],
) -> Result<String, String> {
    let mut hasher = Sha256::new();
    hasher.update(domain_separator.as_bytes());
    for component in components {
        let length = u32::try_from(component.len())
            .map_err(|_| "m4_hash_component_too_large".to_string())?;
        hasher.update(length.to_be_bytes());
        hasher.update(component.as_bytes());
    }
    Ok(format!("{prefix}{:x}", hasher.finalize()))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4ScopeWatermarkEntry {
    pub(crate) source_owner_ref: String,
    pub(crate) scope_ref: String,
    pub(crate) source_type: String,
    pub(crate) canonical_source_object_id: String,
    pub(crate) source_revision: u64,
    pub(crate) source_event_id: String,
    pub(crate) source_owner_watermark: String,
    pub(crate) payload_hash: String,
}

pub(crate) fn m4_scope_source_watermark(
    entries: &[M4ScopeWatermarkEntry],
) -> Result<String, String> {
    let mut entries = entries.to_vec();
    entries.sort_by(|left, right| {
        (
            &left.source_owner_ref,
            &left.scope_ref,
            &left.source_type,
            &left.canonical_source_object_id,
        )
            .cmp(&(
                &right.source_owner_ref,
                &right.scope_ref,
                &right.source_type,
                &right.canonical_source_object_id,
            ))
    });
    let mut hasher = Sha256::new();
    hasher.update(b"syn.m4.scope-source-watermark/v1");
    for entry in entries {
        let revision = entry.source_revision.to_string();
        for component in [
            entry.source_owner_ref.as_str(),
            entry.scope_ref.as_str(),
            entry.source_type.as_str(),
            entry.canonical_source_object_id.as_str(),
            revision.as_str(),
            entry.source_event_id.as_str(),
            entry.source_owner_watermark.as_str(),
            entry.payload_hash.as_str(),
        ] {
            let length = u32::try_from(component.len())
                .map_err(|_| "m4_hash_component_too_large".to_string())?;
            hasher.update(length.to_be_bytes());
            hasher.update(component.as_bytes());
        }
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub(crate) fn m4_validate_reference_text(field: &str, value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 512
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err(format!("m4_reference_text_invalid:{field}"));
    }
    Ok(())
}

fn m4_validate_identifier(field: &str, value: &str) -> Result<(), String> {
    if !value.is_ascii()
        || value.bytes().any(|byte| {
            !(byte.is_ascii_alphanumeric()
                || matches!(byte, b':' | b'.' | b'_' | b'-' | b'/' | b'@'))
        })
    {
        return Err(format!("m4_identifier_invalid:{field}"));
    }
    Ok(())
}

fn m4_looks_like_forbidden_raw_reference(value: &str) -> bool {
    if m4_is_opaque_reference(value) {
        return false;
    }
    let lower = value.to_ascii_lowercase();
    value.contains('@')
        || value.starts_with('/')
        || value.starts_with("./")
        || value.starts_with("../")
        || value.contains("/./")
        || value.contains("/../")
        || lower.contains("://")
        || [
            "password",
            "credential",
            "api_key",
            "apikey",
            "access_token",
            "refresh_token",
            "bearer",
        ]
        .into_iter()
        .any(|marker| lower.contains(marker))
}

pub(crate) fn m4_scrub_quarantine_ref(namespace: &str, value: &str) -> Result<String, String> {
    if m4_is_opaque_reference(value) {
        return Ok(value.to_string());
    }
    m4_internal_id(
        &format!("{namespace}:sha256:"),
        "syn.m4.quarantine-scrubbed-ref/v1",
        &[namespace, value],
    )
}

pub(crate) fn m4_is_lower_hex_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(crate) fn m4_is_opaque_reference(value: &str) -> bool {
    let mut parts = value.split(':');
    let namespace = parts.next().unwrap_or_default();
    let algorithm = parts.next();
    let digest = parts.next();
    (1..=64).contains(&namespace.len())
        && namespace
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase())
        && namespace.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
        && algorithm == Some("sha256")
        && digest.is_some_and(m4_is_lower_hex_digest)
        && parts.next().is_none()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct M4UtcSortKey {
    year: u32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    nanosecond: u32,
}

pub(crate) fn m4_parse_rfc3339_utc_key(value: &str) -> Option<M4UtcSortKey> {
    let bytes = value.as_bytes();
    let fixed_shape = bytes.len() >= 20
        && bytes.len() <= 30
        && bytes.get(4) == Some(&b'-')
        && bytes.get(7) == Some(&b'-')
        && bytes.get(10) == Some(&b'T')
        && bytes.get(13) == Some(&b':')
        && bytes.get(16) == Some(&b':')
        && bytes.last() == Some(&b'Z')
        && bytes[..4].iter().all(u8::is_ascii_digit)
        && bytes[5..7].iter().all(u8::is_ascii_digit)
        && bytes[8..10].iter().all(u8::is_ascii_digit)
        && bytes[11..13].iter().all(u8::is_ascii_digit)
        && bytes[14..16].iter().all(u8::is_ascii_digit)
        && bytes[17..19].iter().all(u8::is_ascii_digit)
        && (bytes.len() == 20
            || (bytes.len() >= 22
                && bytes.get(19) == Some(&b'.')
                && bytes[20..bytes.len() - 1].iter().all(u8::is_ascii_digit)));
    if !fixed_shape {
        return None;
    }
    let parse = |range: std::ops::Range<usize>| -> Option<u32> {
        std::str::from_utf8(bytes.get(range)?).ok()?.parse().ok()
    };
    let year = parse(0..4)?;
    let month = parse(5..7)?;
    let day = parse(8..10)?;
    let leap_year = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days_in_month = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap_year => 29,
        2 => 28,
        _ => 0,
    };
    let hour = parse(11..13)?;
    let minute = parse(14..16)?;
    let second = parse(17..19)?;
    if day == 0 || day > days_in_month || hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    let nanosecond = if bytes.len() == 20 {
        0
    } else {
        let digits = std::str::from_utf8(&bytes[20..bytes.len() - 1]).ok()?;
        let parsed: u32 = digits.parse().ok()?;
        parsed.checked_mul(10_u32.pow(9_u32.checked_sub(digits.len() as u32)?))?
    };
    Some(M4UtcSortKey {
        year,
        month,
        day,
        hour,
        minute,
        second,
        nanosecond,
    })
}

pub(crate) fn m4_primary_scope_ref() -> &'static str {
    M4_PRIMARY_SECRETARY_SCOPE_ID
}

pub(crate) fn m4_primary_actor_ref() -> &'static str {
    M4_PRIMARY_SECRETARY_ACTOR_ID
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static FIXTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    struct AppDataFixture {
        fixture_root: PathBuf,
        root: PathBuf,
    }

    impl AppDataFixture {
        fn new(label: &str) -> Self {
            let sequence = FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let fixture_root = std::env::temp_dir().join(format!(
                "syn-m4c02-domain-{label}-{}-{sequence}",
                std::process::id()
            ));
            let requested = fixture_root.join("local.codex.governance.workbench");
            fs::create_dir_all(&requested).expect("create M4C02 app-data fixture");
            let root = fs::canonicalize(requested).expect("canonical M4C02 app-data fixture");
            Self { fixture_root, root }
        }

        fn repository(&self) -> M3RoleSessionSqliteRepository {
            M3RoleSessionSqliteRepository::open_ordinary_product(
                &M3OrdinaryRoleSessionRepositoryConfig {
                    app_data_root: self.root.clone(),
                    db_path: self.root.join(M3_ORDINARY_ROLE_SESSION_RELATIVE_PATH),
                },
            )
            .expect("open M4C02 ordinary repository fixture")
        }
    }

    impl Drop for AppDataFixture {
        fn drop(&mut self) {
            let db_path = self.root.join(M3_ORDINARY_ROLE_SESSION_RELATIVE_PATH);
            for suffix in ["", "-wal", "-shm"] {
                let _ = fs::remove_file(PathBuf::from(format!("{}{suffix}", db_path.display())));
            }
            let _ = fs::remove_dir_all(&self.fixture_root);
        }
    }

    #[test]
    fn m4c02_bootstrap_is_idempotent_and_restart_reloads_one_active_session() {
        let fixture = AppDataFixture::new("restart");
        let identity = resolve_m4_primary_secretary_identity().expect("fixed Secretary identity");
        let binding = identity
            .m3_server_resolved_binding()
            .expect("fixed M3 binding");
        let repository = fixture.repository();
        repository
            .set_test_server_utc_now("2026-08-10T12:00:00Z")
            .expect("set deterministic M3 server clock");

        let first = create_secretary_role_session(&repository, &binding)
            .expect("first deterministic create");
        repository
            .set_test_server_utc_now("2026-08-10T12:01:00Z")
            .expect("advance deterministic M3 server clock");
        let replay = create_secretary_role_session(&repository, &binding)
            .expect("exact deterministic create replay");
        assert!(!first.replayed);
        assert!(replay.replayed);
        assert_eq!(first.role_session_id, replay.role_session_id);

        let first_runtime = install_ordinary_product_secretary_runtime(&fixture.root)
            .expect("first ordinary runtime install");
        let first_status = first_runtime.secretary_status().expect("first status");
        drop(first_runtime);
        let restarted_runtime = install_ordinary_product_secretary_runtime(&fixture.root)
            .expect("restart ordinary runtime install");
        let restarted_status = restarted_runtime
            .secretary_status()
            .expect("restarted status");
        assert_eq!(first_status, restarted_status);
        assert_eq!(restarted_status.session_state, "ACTIVE");

        let page = fixture
            .repository()
            .list_authorized_role_session_directory(&M3RoleSessionDirectoryQuery {
                binding,
                after: None,
                limit: 100,
            })
            .expect("list exact Secretary sessions");
        assert_eq!(page.entries.len(), 1);
        assert!(page.next_cursor.is_none());
        assert_eq!(page.entries[0].session.created_at, "2026-08-10T12:00:00Z");
    }

    #[test]
    fn m4c02_bootstrap_keeps_unbound_suspended_session_fail_closed() {
        let fixture = AppDataFixture::new("resume");
        let identity = resolve_m4_primary_secretary_identity().expect("fixed Secretary identity");
        let binding = identity
            .m3_server_resolved_binding()
            .expect("fixed M3 binding");
        let repository = fixture.repository();
        let created = create_secretary_role_session(&repository, &binding)
            .expect("create primary Secretary session");
        let previous_permission =
            permission_descriptor(&identity, &binding).expect("current permission descriptor");
        let wider_snapshot = opaque_ref("permission", "m4c02-wider-snapshot")
            .expect("wider permission snapshot ref");
        let wider_binding = ServerResolvedBinding::from_server_canonical(
            binding.actor_id.as_str().to_string(),
            binding.role_ref.as_str().to_string(),
            binding.scope_ref.as_str().to_string(),
            binding.current_object_ref.as_str().to_string(),
            binding.execution_channel.as_str().to_string(),
            wider_snapshot.as_str().to_string(),
        )
        .expect("same-owner wider binding");
        let mut wider_permission = previous_permission.clone();
        wider_permission.snapshot_ref = wider_snapshot;
        wider_permission
            .allowed_capability_refs
            .insert(opaque_ref("capability", "m4c02-wider-capability").expect("wider cap"));
        let active = repository
            .load_authorized_role_session_snapshot(
                &crate::m3_role_session_repository::M3RoleSessionSnapshotQuery {
                    role_session_id: created.role_session_id.clone(),
                    binding: binding.clone(),
                },
            )
            .expect("load active session")
            .expect("active session exists");
        let suspended = repository
            .resume_role_session(&ResumeRoleSessionCommand {
                role_session_id: created.role_session_id.clone(),
                binding: wider_binding,
                previous_permission: Some(previous_permission),
                current_permission: Some(wider_permission),
                expected_session_revision: active.session.revision,
                metadata: metadata_for(
                    &repository,
                    "resume",
                    "syn.m4.secretary-role-session-resume/wider-fixture/v1",
                )
                .expect("wider resume metadata"),
            })
            .expect("wider request records a suspended outcome")
            .role_session
            .expect("suspended session outcome");
        assert_eq!(suspended.status, RoleSessionState::Suspended);

        let error = bootstrap_or_restore_secretary_role_session(&repository, &identity, &binding)
            .expect_err("an unbound suspended session needs verified binding evidence");
        assert_eq!(error, "m4_secretary_resume_binding_unavailable");
        let snapshot = repository
            .load_authorized_role_session_snapshot(&M3RoleSessionSnapshotQuery {
                role_session_id: created.role_session_id,
                binding,
            })
            .expect("load fail-closed session")
            .expect("fail-closed session exists");
        assert_eq!(snapshot.session.status, RoleSessionState::Suspended);
    }

    #[test]
    fn m4c02_bootstrap_rejects_ambiguous_exact_owner_without_recency_selection() {
        let fixture = AppDataFixture::new("ambiguous");
        let identity = resolve_m4_primary_secretary_identity().expect("fixed Secretary identity");
        let binding = identity
            .m3_server_resolved_binding()
            .expect("fixed M3 binding");
        let repository = fixture.repository();
        let first = create_secretary_role_session(&repository, &binding).expect("primary session");
        let second_role_session_id = RoleSessionId::try_from_canonical(sealed_ref(
            "session",
            "syn.m4.secretary-role-session/ambiguous-fixture/v1",
        ))
        .expect("second session id");
        repository
            .create_role_session(&CreateRoleSessionCommand {
                role_session_id: second_role_session_id.clone(),
                binding: binding.clone(),
                metadata: metadata_for(
                    &repository,
                    "create",
                    "syn.m4.secretary-role-session-create/ambiguous-fixture/v1",
                )
                .expect("second create metadata"),
            })
            .expect("create second exact-owner session");

        assert_eq!(
            bootstrap_or_restore_secretary_role_session(&repository, &identity, &binding)
                .expect_err("multiple exact sessions must fail closed"),
            "m4_secretary_role_session_ambiguous",
        );
        let quarantined = repository
            .list_authorized_role_session_directory(&M3RoleSessionDirectoryQuery {
                binding,
                after: None,
                limit: 100,
            })
            .expect("list quarantined ambiguous candidates");
        assert_eq!(quarantined.entries.len(), 2);
        assert!(quarantined.entries.iter().all(|entry| {
            entry.session.status == RoleSessionState::Quarantined
                && entry.session.resolution_reason
                    == Some(
                        crate::m3_role_session::SessionResolutionReason::OwnerScopeOrHandleMappingAmbiguous,
                    )
        }));
        assert!(quarantined
            .entries
            .iter()
            .any(|entry| entry.session.role_session_id == first.role_session_id));
        assert!(quarantined
            .entries
            .iter()
            .any(|entry| entry.session.role_session_id == second_role_session_id));
    }

    #[test]
    fn m4c02_bootstrap_quarantines_mismatched_secretary_candidate() {
        let fixture = AppDataFixture::new("mismatched");
        let identity = resolve_m4_primary_secretary_identity().expect("fixed Secretary identity");
        let binding = identity
            .m3_server_resolved_binding()
            .expect("fixed M3 binding");
        let repository = fixture.repository();
        let mismatched_snapshot = opaque_ref("permission", "m4c02-mismatched-snapshot")
            .expect("mismatched permission snapshot");
        let mismatched_binding = ServerResolvedBinding::from_server_canonical(
            binding.actor_id.as_str().to_string(),
            binding.role_ref.as_str().to_string(),
            binding.scope_ref.as_str().to_string(),
            binding.current_object_ref.as_str().to_string(),
            binding.execution_channel.as_str().to_string(),
            mismatched_snapshot.as_str().to_string(),
        )
        .expect("same-owner mismatched binding");
        let role_session_id = RoleSessionId::try_from_canonical(sealed_ref(
            "session",
            "syn.m4.secretary-role-session/mismatched-fixture/v1",
        ))
        .expect("mismatched candidate id");
        repository
            .create_role_session(&CreateRoleSessionCommand {
                role_session_id: role_session_id.clone(),
                binding: mismatched_binding,
                metadata: metadata_for(
                    &repository,
                    "create",
                    "syn.m4.secretary-role-session-create/mismatched-fixture/v1",
                )
                .expect("mismatched create metadata"),
            })
            .expect("create mismatched candidate");

        assert_eq!(
            bootstrap_or_restore_secretary_role_session(&repository, &identity, &binding)
                .expect_err("a permission-mismatched candidate must never be reused"),
            "m4_secretary_role_session_mismatched",
        );
        let quarantined = repository
            .list_authorized_role_session_directory(&M3RoleSessionDirectoryQuery {
                binding,
                after: None,
                limit: 100,
            })
            .expect("list quarantined mismatched candidate");
        assert_eq!(quarantined.entries.len(), 1);
        assert_eq!(
            quarantined.entries[0].session.role_session_id,
            role_session_id
        );
        assert_eq!(
            quarantined.entries[0].session.status,
            RoleSessionState::Quarantined
        );
    }

    #[test]
    fn m4c02_app_data_root_alias_fails_before_database_creation() {
        let fixture = AppDataFixture::new("root-alias");
        let alias = fixture.root.join("child/..");
        let error = match install_ordinary_product_secretary_runtime(&alias) {
            Ok(_) => panic!("unclean server path must fail closed"),
            Err(error) => error,
        };
        assert_eq!(error, "m4_secretary_app_data_root_clean_absolute_required",);
        assert!(!fixture
            .root
            .join(M3_ORDINARY_ROLE_SESSION_RELATIVE_PATH)
            .exists());
    }

    #[test]
    fn m4c02_product_composition_uses_tauri_app_data_root_before_app_state_install() {
        let entrypoints = include_str!("index_host_app_entrypoints.rs");
        let tauri_root = entrypoints
            .find(".app_data_dir()")
            .expect("ordinary composition resolves Tauri app-data root");
        let ordinary_constructor = entrypoints
            .find("AppState::try_new_with_tauri_app_data_root(&app_data_root)")
            .expect("ordinary AppState receives the Tauri-resolved root");
        let setup = entrypoints
            .find(".setup(move |app| {")
            .expect("Tauri setup owns ordinary product composition");
        let acceptance_constructor = entrypoints
            .find("AppState::try_new()")
            .expect("acceptance profile constructs its isolated AppState");
        assert!(tauri_root < ordinary_constructor);
        assert!(acceptance_constructor < setup);
        assert!(ordinary_constructor > setup);
        assert_eq!(entrypoints.matches("AppState::try_new()").count(), 1);
        assert_eq!(
            entrypoints
                .matches("AppState::try_new_with_tauri_app_data_root(&app_data_root)")
                .count(),
            1
        );

        let lib = include_str!("lib.rs");
        assert!(lib.contains("fn try_new_with_tauri_app_data_root(app_data_root: &Path)"));
        assert!(lib.contains("m3_role_session_read_runtime: Default::default()"));
        assert!(!entrypoints.contains(
            "AppState::try_new_with_tauri_app_data_root(&default_workflow_state_path())"
        ));
    }
}

#[cfg(test)]
mod m4c03_domain_tests {
    use super::*;

    fn opaque(namespace: &str, material: &str) -> String {
        m4_internal_id(
            &format!("{namespace}:sha256:"),
            "syn.m4c03.domain-test/v1",
            &[material],
        )
        .expect("make M4C03 opaque ref")
    }

    fn valid_input() -> M4WorkflowAttentionSourceInput {
        M4WorkflowAttentionSourceInput {
            // This deliberately contains the letters "token". Admission is
            // structural and does not reject harmless opaque identifiers by
            // broad sensitive-word substring matching.
            source_owner_ref: "tokenization_scheduler".to_string(),
            scope_ref: M4_PRIMARY_SECRETARY_SCOPE_ID.to_string(),
            source_type: M4_WORKFLOW_ATTENTION_SOURCE_TYPE.to_string(),
            canonical_source_object_id: "work-item-42".to_string(),
            source_revision: 7,
            source_event_id: opaque("source-event-id", "event-7"),
            source_owner_watermark: opaque("watermark", "watermark-7"),
            occurred_at_utc: "2026-08-10T12:34:56.123456789Z".to_string(),
            source_link: M4SourceLinkInput {
                link_kind: "INTERNAL_ROUTE".to_string(),
                source_owner_ref: "tokenization_scheduler".to_string(),
                object_type: M4_WORKFLOW_ATTENTION_OBJECT_TYPE.to_string(),
                canonical_source_object_id: "work-item-42".to_string(),
                expected_source_revision: 7,
                opaque_route_ref: opaque("route", "work-item-42"),
            },
            owner_status_code: "BLOCKED".to_string(),
            attention_signals: M4AttentionSignals {
                source_blocked: true,
                material_change: true,
                ..Default::default()
            },
            due_at_utc: Some("2026-08-11T00:00:00Z".to_string()),
            sensitivity: M4_SCRUBBED_SENSITIVITY.to_string(),
            scrubbed_summary_ref: opaque("summary", "work-item-42-revision-7"),
            payload_hash: m4_internal_id("", "syn.m4c03.payload-test/v1", &["payload-7"])
                .expect("make payload hash"),
        }
    }

    #[test]
    fn m4c03_hash_encoding_has_frozen_golden_values() {
        let identity = m4_source_identity_key(
            "owner-a",
            "scope:personal:primary",
            M4_WORKFLOW_ATTENTION_SOURCE_TYPE,
            "work-item-42",
        )
        .expect("derive source identity key");
        assert_eq!(
            identity,
            "source:16d51ecd904cb9fe943e05121627ca0580e10454f0f13c875550c3fa35f5f426"
        );
        let event_key = m4_source_event_key(
            &identity,
            7,
            &format!("event:sha256:{}", "0".repeat(64)),
            &"1".repeat(64),
        )
        .expect("derive source event key");
        assert_eq!(
            event_key,
            "source-event:5ca5ca5fbf4533dacd25510d993b0052b886d9c27dcce88c1c7e3662aef56465"
        );
        assert_eq!(
            m4_scope_source_watermark(&[]).expect("derive empty scope watermark"),
            "4d7c4d299b1c4f2a127a7e2b4132b88c0612691a30be928e88fff2cc3253b947"
        );
    }

    #[test]
    fn m4c03_registered_adapter_maps_status_priority_and_open_predicate_mechanically() {
        let admitted = match classify_workflow_attention_source(&valid_input())
            .expect("classify valid source")
        {
            M4WorkflowAttentionAdmission::Admitted(source) => source,
            other => panic!("expected admission, got {other:?}"),
        };
        assert_eq!(admitted.source_status, M4SourceStatus::Blocked);
        assert_eq!(admitted.priority.rank, 1);
        assert_eq!(admitted.priority.code, "USER_DECISION_OR_BLOCKER");
        assert!(m4_automatic_open_loop(&admitted));

        let mut terminal = valid_input();
        terminal.owner_status_code = "COMPLETED".to_string();
        let terminal = match classify_workflow_attention_source(&terminal)
            .expect("classify terminal source")
        {
            M4WorkflowAttentionAdmission::Admitted(source) => source,
            other => panic!("expected terminal admission, got {other:?}"),
        };
        assert!(!m4_automatic_open_loop(&terminal));
        assert_eq!(
            terminal.source_status.terminal_closure_reason(),
            Some("SOURCE_COMPLETED")
        );
    }

    #[test]
    fn m4c03_unknown_scope_sensitive_link_and_timestamp_fail_before_active_projection() {
        for (mut input, reason) in [
            {
                let mut input = valid_input();
                input.owner_status_code = "UNMAPPED".to_string();
                (input, "OWNER_STATUS_UNKNOWN")
            },
            {
                let mut input = valid_input();
                input.scope_ref = "scope:personal:other".to_string();
                (input, "PERSONAL_SCOPE_MISMATCH")
            },
            {
                let mut input = valid_input();
                input.sensitivity = "RAW_SOURCE_BODY".to_string();
                (input, "SENSITIVITY_NOT_ADMITTED")
            },
            {
                let mut input = valid_input();
                input.source_link.opaque_route_ref = "https://example.invalid/path".to_string();
                (input, "SOURCE_LINK_ROUTE_REF_INVALID")
            },
            {
                let mut input = valid_input();
                input.due_at_utc = Some("2026-02-31T00:00:00Z".to_string());
                (input, "DUE_AT_UTC_INVALID")
            },
        ] {
            // Keep the typed link scope-independent for the explicit scope
            // mismatch fixture; only the top-level personal scope is tested.
            input.source_link.expected_source_revision = input.source_revision;
            match classify_workflow_attention_source(&input).expect("classify quarantine input") {
                M4WorkflowAttentionAdmission::Quarantined(candidate) => {
                    assert_eq!(candidate.reason_code, reason)
                }
                other => panic!("expected quarantine {reason}, got {other:?}"),
            }
        }
    }

    #[test]
    fn m4c03_utc_parser_accepts_real_leap_day_and_orders_fractional_instants() {
        let zero =
            m4_parse_rfc3339_utc_key("2024-02-29T23:59:59Z").expect("accept real UTC leap day");
        let fractional = m4_parse_rfc3339_utc_key("2024-02-29T23:59:59.1Z")
            .expect("accept fractional UTC instant");
        assert!(fractional > zero);
        assert!(m4_parse_rfc3339_utc_key("2026-02-29T00:00:00Z").is_none());
        assert!(m4_parse_rfc3339_utc_key("2026-08-10T12:00:00+08:00").is_none());
    }
}
