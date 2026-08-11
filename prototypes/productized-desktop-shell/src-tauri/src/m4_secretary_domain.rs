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
    M3ReadPermissionDisposition, M3RepositoryCommandOutcome, M3RoleSessionDirectoryQuery,
    M3RoleSessionSnapshotQuery, M3RoleSessionSqliteRepository, M3SessionBindingReadState,
    QuarantineRoleSessionCommand, ResumeRoleSessionCommand, M3_ORDINARY_ROLE_SESSION_RELATIVE_PATH,
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

#[derive(Clone)]
pub(crate) struct M4OrdinarySecretaryRuntimeInstallation {
    pub(crate) read_runtime: M3RoleSessionReadRuntimeSlot,
    pub(crate) repository: M3RoleSessionSqliteRepository,
    pub(crate) binding: ServerResolvedBinding,
    pub(crate) role_session_id: RoleSessionId,
    pub(crate) permission: PermissionSnapshotDescriptor,
    /// Present only in the process that committed CREATE_ROLE_SESSION.  The
    /// M3 repository clone shares the process-local fresh-dispatch permit, so
    /// the first explicit Secretary message may lazily bind the provider.  A
    /// reopened process deliberately receives None and must not bypass M3's
    /// restart/orphan rule.
    pub(crate) fresh_session_start: Option<M3RepositoryCommandOutcome>,
}

pub(crate) fn install_ordinary_product_secretary_runtime(
    app_data_root: &Path,
) -> Result<M3RoleSessionReadRuntimeSlot, String> {
    install_ordinary_product_secretary_composition(app_data_root)
        .map(|installation| installation.read_runtime)
}

pub(crate) fn install_ordinary_product_secretary_composition(
    app_data_root: &Path,
) -> Result<M4OrdinarySecretaryRuntimeInstallation, String> {
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
    let bootstrap =
        bootstrap_or_restore_secretary_role_session_installation(&repository, &identity, &binding)?;
    let permission = permission_descriptor(&identity, &binding)?;

    let read_runtime = M3RoleSessionReadRuntimeSlot::from_ordinary_product_secretary(
        M3OrdinarySecretaryReadBinding {
            host: M3SecretaryReadHost::server_fixed(),
            repository: repository.clone(),
            binding: binding.clone(),
            role_session_id: bootstrap.role_session_id.clone(),
        },
    )?;
    Ok(M4OrdinarySecretaryRuntimeInstallation {
        read_runtime,
        repository,
        binding,
        role_session_id: bootstrap.role_session_id,
        permission,
        fresh_session_start: bootstrap.fresh_session_start,
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
    bootstrap_or_restore_secretary_role_session_installation(repository, identity, binding)
        .map(|installation| installation.role_session_id)
}

struct SecretaryBootstrapInstallation {
    role_session_id: RoleSessionId,
    fresh_session_start: Option<M3RepositoryCommandOutcome>,
}

fn bootstrap_or_restore_secretary_role_session_installation(
    repository: &M3RoleSessionSqliteRepository,
    identity: &M4PrimarySecretaryIdentity,
    binding: &ServerResolvedBinding,
) -> Result<SecretaryBootstrapInstallation, String> {
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
        return create_secretary_role_session(repository, binding).map(|outcome| {
            SecretaryBootstrapInstallation {
                role_session_id: outcome.role_session_id,
                fresh_session_start: (!outcome.replayed).then_some(outcome.repository_outcome),
            }
        });
    };
    match entry.session.status {
        RoleSessionState::Active => {
            if !matches!(&entry.permission, M3ReadPermissionDisposition::Current) {
                return Err("m4_secretary_permission_revalidation_required".to_string());
            }
            Ok(SecretaryBootstrapInstallation {
                role_session_id: entry.session.role_session_id.clone(),
                fresh_session_start: None,
            })
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
            Ok(SecretaryBootstrapInstallation {
                role_session_id: session.role_session_id,
                fresh_session_start: None,
            })
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
    repository_outcome: M3RepositoryCommandOutcome,
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
        .as_ref()
        .ok_or_else(|| "m4_secretary_create_session_missing".to_string())?;
    if session.role_session_id != role_session_id || session.status != RoleSessionState::Active {
        return Err("m4_secretary_create_session_invalid".to_string());
    }
    Ok(SecretaryCreateOutcome {
        role_session_id,
        replayed: outcome.replayed,
        repository_outcome: outcome,
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

/// Ordinary-product owner publications are normalized by the registered
/// dispatcher before they enter M4.  The enum keeps that finite boundary
/// explicit; renderer input and arbitrary adapter strings never reach the
/// downstream projection transaction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum M4RegisteredPublicationKind {
    WorkItemAttention,
    ProposalDecision,
}

impl M4RegisteredPublicationKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::WorkItemAttention => "WORK_ITEM_ATTENTION",
            Self::ProposalDecision => "PROPOSAL_DECISION",
        }
    }
}

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
    m4_source_status_and_attention_is_open_loop(source.source_status, &source.attention_signals)
}

/// The single automatic-open-loop predicate shared by admitted M4 sources
/// and owner-side readers.  Callers must first obtain the status and flags
/// from their registered mapper; this function deliberately contains no
/// owner-specific status table.
pub(crate) fn m4_source_status_and_attention_is_open_loop(
    source_status: M4SourceStatus,
    attention: &M4AttentionSignals,
) -> bool {
    !source_status.is_terminal()
        && (attention.external_commitment
            || attention.time_sensitive
            || attention.requires_user_decision
            || attention.source_blocked
            || attention.attention_required)
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

// -------------------------------------------------------------------------
// M4C04 coordination lifecycle, explicit personal actions, and typed owner
// writeback intents. These are deliberately pure domain rules: persistence,
// source-owner dispatch, UI, providers, and daily projection remain outside
// this module and are wired by their owning leaves.
// -------------------------------------------------------------------------

pub(crate) const M4_IN_APP_DELIVERY_CHANNEL: &str = "IN_APP";
const M4_COORDINATION_CLOSE_REASON: &str = "USER_STOPPED_TRACKING";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum M4InboxItemStatus {
    New,
    Read,
    Dismissed,
    Expired,
    Quarantined,
}

impl M4InboxItemStatus {
    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "NEW" => Some(Self::New),
            "READ" => Some(Self::Read),
            "DISMISSED" => Some(Self::Dismissed),
            "EXPIRED" => Some(Self::Expired),
            "QUARANTINED" => Some(Self::Quarantined),
            _ => None,
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::New => "NEW",
            Self::Read => "READ",
            Self::Dismissed => "DISMISSED",
            Self::Expired => "EXPIRED",
            Self::Quarantined => "QUARANTINED",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum M4OpenLoopStatus {
    Open,
    Acknowledged,
    Snoozed,
    Closed,
    Dismissed,
}

impl M4OpenLoopStatus {
    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "OPEN" => Some(Self::Open),
            "ACKNOWLEDGED" => Some(Self::Acknowledged),
            "SNOOZED" => Some(Self::Snoozed),
            "CLOSED" => Some(Self::Closed),
            "DISMISSED" => Some(Self::Dismissed),
            _ => None,
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Open => "OPEN",
            Self::Acknowledged => "ACKNOWLEDGED",
            Self::Snoozed => "SNOOZED",
            Self::Closed => "CLOSED",
            Self::Dismissed => "DISMISSED",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum M4PersonalActionStatus {
    Open,
    Completed,
    Cancelled,
}

impl M4PersonalActionStatus {
    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "OPEN" => Some(Self::Open),
            "COMPLETED" => Some(Self::Completed),
            "CANCELLED" => Some(Self::Cancelled),
            _ => None,
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Open => "OPEN",
            Self::Completed => "COMPLETED",
            Self::Cancelled => "CANCELLED",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum M4NotificationStatus {
    Pending,
    Delivered,
    Read,
    Dismissed,
}

impl M4NotificationStatus {
    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "PENDING" => Some(Self::Pending),
            "DELIVERED" => Some(Self::Delivered),
            "READ" => Some(Self::Read),
            "DISMISSED" => Some(Self::Dismissed),
            _ => None,
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "PENDING",
            Self::Delivered => "DELIVERED",
            Self::Read => "READ",
            Self::Dismissed => "DISMISSED",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum M4ReminderStatus {
    Scheduled,
    Fired,
    Snoozed,
    Dismissed,
    Cancelled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum M4DecisionOwnerStatus {
    Open,
    Answered,
    Expired,
    Withdrawn,
}

impl M4DecisionOwnerStatus {
    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "OPEN" => Some(Self::Open),
            "ANSWERED" => Some(Self::Answered),
            "EXPIRED" => Some(Self::Expired),
            "WITHDRAWN" => Some(Self::Withdrawn),
            _ => None,
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Open => "OPEN",
            Self::Answered => "ANSWERED",
            Self::Expired => "EXPIRED",
            Self::Withdrawn => "WITHDRAWN",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum M4DecisionLocalVisibilityStatus {
    Unread,
    Read,
    Dismissed,
}

impl M4DecisionLocalVisibilityStatus {
    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "UNREAD" => Some(Self::Unread),
            "READ" => Some(Self::Read),
            "DISMISSED" => Some(Self::Dismissed),
            _ => None,
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Unread => "UNREAD",
            Self::Read => "READ",
            Self::Dismissed => "DISMISSED",
        }
    }
}

impl M4ReminderStatus {
    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "SCHEDULED" => Some(Self::Scheduled),
            "FIRED" => Some(Self::Fired),
            "SNOOZED" => Some(Self::Snoozed),
            "DISMISSED" => Some(Self::Dismissed),
            "CANCELLED" => Some(Self::Cancelled),
            _ => None,
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Scheduled => "SCHEDULED",
            Self::Fired => "FIRED",
            Self::Snoozed => "SNOOZED",
            Self::Dismissed => "DISMISSED",
            Self::Cancelled => "CANCELLED",
        }
    }
}

/// A complete, scrubbed source reference. It carries no source-owner callback,
/// executable payload, raw body, or credential; the route is an opaque
/// server-minted reference that a later registered owner adapter may resolve.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4SourceRecordRef {
    pub(crate) source_owner_ref: String,
    pub(crate) scope_ref: String,
    pub(crate) source_type: String,
    pub(crate) canonical_source_object_id: String,
    pub(crate) source_revision: u64,
    pub(crate) source_event_id: String,
    pub(crate) source_owner_watermark: String,
    pub(crate) occurred_at_utc: String,
    pub(crate) source_link: M4SourceLinkInput,
    pub(crate) source_status: M4SourceStatus,
    pub(crate) attention_signals: M4AttentionSignals,
    pub(crate) due_at_utc: Option<String>,
    pub(crate) sensitivity: String,
    pub(crate) scrubbed_summary_ref: String,
    pub(crate) payload_hash: String,
}

pub(crate) fn m4_validate_source_record_ref(source_ref: &M4SourceRecordRef) -> Result<(), String> {
    let input = M4WorkflowAttentionSourceInput {
        source_owner_ref: source_ref.source_owner_ref.clone(),
        scope_ref: source_ref.scope_ref.clone(),
        source_type: source_ref.source_type.clone(),
        canonical_source_object_id: source_ref.canonical_source_object_id.clone(),
        source_revision: source_ref.source_revision,
        source_event_id: source_ref.source_event_id.clone(),
        source_owner_watermark: source_ref.source_owner_watermark.clone(),
        occurred_at_utc: source_ref.occurred_at_utc.clone(),
        source_link: source_ref.source_link.clone(),
        owner_status_code: source_ref.source_status.as_str().to_string(),
        attention_signals: source_ref.attention_signals.clone(),
        due_at_utc: source_ref.due_at_utc.clone(),
        sensitivity: source_ref.sensitivity.clone(),
        scrubbed_summary_ref: source_ref.scrubbed_summary_ref.clone(),
        payload_hash: source_ref.payload_hash.clone(),
    };
    match classify_workflow_attention_source(&input)? {
        M4WorkflowAttentionAdmission::Admitted(_) => Ok(()),
        M4WorkflowAttentionAdmission::Quarantined(candidate) => Err(format!(
            "m4_source_record_ref_not_admitted:{}",
            candidate.reason_code
        )),
    }
}

pub(crate) fn m4_source_record_identity_key(
    source_ref: &M4SourceRecordRef,
) -> Result<String, String> {
    m4_validate_source_record_ref(source_ref)?;
    m4_source_identity_key(
        &source_ref.source_owner_ref,
        &source_ref.scope_ref,
        &source_ref.source_type,
        &source_ref.canonical_source_object_id,
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4InboxItem {
    pub(crate) inbox_item_id: String,
    pub(crate) source_ref: M4SourceRecordRef,
    pub(crate) dedupe_key: String,
    pub(crate) status: M4InboxItemStatus,
    pub(crate) priority_reason: M4PriorityReason,
    pub(crate) received_at_utc: String,
    pub(crate) last_source_change_at_utc: String,
    pub(crate) scrubbed_summary_ref: String,
    pub(crate) sensitivity: String,
    pub(crate) revision: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4OpenLoop {
    pub(crate) open_loop_id: String,
    pub(crate) source_ref: M4SourceRecordRef,
    pub(crate) status: M4OpenLoopStatus,
    pub(crate) why_open_code: String,
    pub(crate) priority_reason: M4PriorityReason,
    pub(crate) owner_ref: String,
    pub(crate) due_at_utc: Option<String>,
    pub(crate) snoozed_until_utc: Option<String>,
    pub(crate) last_source_revision: u64,
    pub(crate) projection_policy_ref: String,
    pub(crate) closure_reason_code: Option<String>,
    pub(crate) revision: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4PersonalAction {
    pub(crate) personal_action_id: String,
    pub(crate) explicit_user_command_ref: String,
    pub(crate) title: String,
    pub(crate) status: M4PersonalActionStatus,
    pub(crate) due_at_utc: Option<String>,
    pub(crate) revision: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4Notification {
    pub(crate) notification_id: String,
    pub(crate) source_ref: M4SourceRecordRef,
    pub(crate) subject_ref: String,
    pub(crate) notification_purpose_code: String,
    pub(crate) delivery_channel: String,
    pub(crate) status: M4NotificationStatus,
    pub(crate) created_at_utc: String,
    pub(crate) delivered_at_utc: Option<String>,
    pub(crate) read_at_utc: Option<String>,
    pub(crate) dismissed_at_utc: Option<String>,
    pub(crate) revision: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4Reminder {
    pub(crate) reminder_id: String,
    pub(crate) owner_ref: String,
    pub(crate) explicit_schedule_command_id: String,
    pub(crate) scheduled_for_utc: String,
    pub(crate) iana_timezone: String,
    pub(crate) status: M4ReminderStatus,
    pub(crate) last_fired_at_utc: Option<String>,
    pub(crate) snoozed_until_utc: Option<String>,
    pub(crate) revision: u64,
}

/// A typed local projection of an owner-side proposal decision.  Owner status
/// and local visibility are deliberately independent axes: local read/dismiss
/// commands never manufacture an owner-side answer or expiry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4DecisionProjection {
    pub(crate) decision_projection_id: String,
    pub(crate) source_identity_key: String,
    pub(crate) source_event_key: String,
    pub(crate) source_ref: String,
    pub(crate) owner_status: M4DecisionOwnerStatus,
    pub(crate) local_visibility_status: M4DecisionLocalVisibilityStatus,
    pub(crate) decision_by_utc: Option<String>,
    pub(crate) source_revision: u64,
    pub(crate) revision: u64,
}

/// Fully server-normalized publication passed from the ordinary owner outbox
/// dispatcher into M4.  `source_object_type` is native provenance only; M4
/// still admits the publication through its fixed structured-source adapter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4RegisteredSourcePublication {
    pub(crate) publication_sequence: u64,
    pub(crate) publication_id: String,
    pub(crate) adapter_id: String,
    pub(crate) publication_kind: M4RegisteredPublicationKind,
    pub(crate) native_scope_seal: String,
    pub(crate) source_owner_ref: String,
    pub(crate) source_object_type: String,
    pub(crate) canonical_source_object_id: String,
    pub(crate) source_revision: u64,
    pub(crate) source_event_id: String,
    pub(crate) source_owner_watermark: String,
    pub(crate) occurred_at_utc: String,
    pub(crate) source_status: M4SourceStatus,
    pub(crate) decision_owner_status: Option<M4DecisionOwnerStatus>,
    pub(crate) attention_signals: M4AttentionSignals,
    pub(crate) due_at_utc: Option<String>,
    pub(crate) opaque_route_ref: String,
    pub(crate) scrubbed_summary_ref: String,
    pub(crate) payload_hash: String,
}

pub(crate) fn m4_personal_action_id(explicit_user_command_id: &str) -> Result<String, String> {
    m4_internal_id(
        "personal-action:",
        "syn.m4.personal-action/v1",
        &[explicit_user_command_id],
    )
}

pub(crate) fn m4_notification_id(
    subject_ref: &str,
    notification_purpose_code: &str,
) -> Result<String, String> {
    m4_internal_id(
        "notification:",
        "syn.m4.notification/v1",
        &[subject_ref, notification_purpose_code],
    )
}

pub(crate) fn m4_reminder_id(
    owner_ref: &str,
    explicit_schedule_command_id: &str,
) -> Result<String, String> {
    m4_internal_id(
        "reminder:",
        "syn.m4.reminder/v1",
        &[owner_ref, explicit_schedule_command_id],
    )
}

pub(crate) fn m4_decision_projection_id(source_identity_key: &str) -> Result<String, String> {
    m4_internal_id(
        "decision-projection:",
        "syn.m4.decision-projection/v1",
        &[source_identity_key],
    )
}

pub(crate) fn m4_validate_registered_source_publication(
    publication: &M4RegisteredSourcePublication,
) -> Result<(), String> {
    if publication.publication_sequence == 0 {
        return Err("m4_registered_publication_sequence_invalid".to_string());
    }
    for (field, value) in [
        ("publication_id", publication.publication_id.as_str()),
        ("adapter_id", publication.adapter_id.as_str()),
        ("native_scope_seal", publication.native_scope_seal.as_str()),
        ("source_owner_ref", publication.source_owner_ref.as_str()),
        (
            "source_object_type",
            publication.source_object_type.as_str(),
        ),
        (
            "canonical_source_object_id",
            publication.canonical_source_object_id.as_str(),
        ),
        ("source_event_id", publication.source_event_id.as_str()),
        (
            "source_owner_watermark",
            publication.source_owner_watermark.as_str(),
        ),
        ("occurred_at_utc", publication.occurred_at_utc.as_str()),
        ("opaque_route_ref", publication.opaque_route_ref.as_str()),
        (
            "scrubbed_summary_ref",
            publication.scrubbed_summary_ref.as_str(),
        ),
    ] {
        m4_validate_reference_text(field, value)?;
    }
    for (field, value) in [
        ("adapter_id", publication.adapter_id.as_str()),
        ("source_owner_ref", publication.source_owner_ref.as_str()),
        (
            "source_object_type",
            publication.source_object_type.as_str(),
        ),
        (
            "canonical_source_object_id",
            publication.canonical_source_object_id.as_str(),
        ),
    ] {
        m4_validate_identifier(field, value)?;
    }
    for (field, value) in [
        ("publication_id", publication.publication_id.as_str()),
        ("native_scope_seal", publication.native_scope_seal.as_str()),
        ("source_event_id", publication.source_event_id.as_str()),
        (
            "source_owner_watermark",
            publication.source_owner_watermark.as_str(),
        ),
        ("opaque_route_ref", publication.opaque_route_ref.as_str()),
        (
            "scrubbed_summary_ref",
            publication.scrubbed_summary_ref.as_str(),
        ),
    ] {
        if !m4_is_opaque_reference(value) {
            return Err(format!(
                "m4_registered_publication_opaque_ref_invalid:{field}"
            ));
        }
    }
    if m4_parse_rfc3339_utc_key(&publication.occurred_at_utc).is_none()
        || publication
            .due_at_utc
            .as_deref()
            .is_some_and(|value| m4_parse_rfc3339_utc_key(value).is_none())
    {
        return Err("m4_registered_publication_timestamp_invalid".to_string());
    }
    if !m4_is_lower_hex_digest(&publication.payload_hash) {
        return Err("m4_registered_publication_payload_hash_invalid".to_string());
    }
    match (
        publication.publication_kind,
        publication.decision_owner_status,
    ) {
        (M4RegisteredPublicationKind::WorkItemAttention, None)
        | (M4RegisteredPublicationKind::ProposalDecision, Some(_)) => {}
        _ => return Err("m4_registered_publication_kind_binding_invalid".to_string()),
    }
    Ok(())
}

pub(crate) fn m4_validate_decision_projection(
    decision: &M4DecisionProjection,
) -> Result<(), String> {
    m4_validate_typed_reference(
        "decision_source_identity_key",
        &decision.source_identity_key,
    )?;
    m4_validate_typed_reference("decision_source_event_key", &decision.source_event_key)?;
    m4_validate_typed_reference("decision_source_ref", &decision.source_ref)?;
    if decision.source_ref != decision.source_identity_key
        || decision.decision_projection_id
            != m4_decision_projection_id(&decision.source_identity_key)?
    {
        return Err("m4_decision_projection_identity_mismatch".to_string());
    }
    if let Some(value) = decision.decision_by_utc.as_deref() {
        m4_validate_utc("decision_by_utc", value)?;
    }
    Ok(())
}

pub(crate) fn m4_validate_inbox_item(item: &M4InboxItem) -> Result<(), String> {
    m4_validate_source_record_ref(&item.source_ref)?;
    let source_identity_key = m4_source_record_identity_key(&item.source_ref)?;
    if item.inbox_item_id != m4_inbox_item_id(&source_identity_key)? {
        return Err("m4_inbox_item_id_mismatch".to_string());
    }
    if item.dedupe_key != source_identity_key {
        return Err("m4_inbox_item_dedupe_key_mismatch".to_string());
    }
    m4_validate_priority_for_source(&item.priority_reason, &item.source_ref)?;
    m4_validate_utc("inbox_received_at_utc", &item.received_at_utc)?;
    m4_validate_utc(
        "inbox_last_source_change_at_utc",
        &item.last_source_change_at_utc,
    )?;
    if item.scrubbed_summary_ref != item.source_ref.scrubbed_summary_ref {
        return Err("m4_inbox_item_summary_ref_mismatch".to_string());
    }
    if item.sensitivity != M4_SCRUBBED_SENSITIVITY
        || item.sensitivity != item.source_ref.sensitivity
    {
        return Err("m4_inbox_item_sensitivity_invalid".to_string());
    }
    Ok(())
}

pub(crate) fn m4_validate_open_loop(open_loop: &M4OpenLoop) -> Result<(), String> {
    m4_validate_source_record_ref(&open_loop.source_ref)?;
    let source_identity_key = m4_source_record_identity_key(&open_loop.source_ref)?;
    if open_loop.open_loop_id != m4_open_loop_id(&source_identity_key)? {
        return Err("m4_open_loop_id_mismatch".to_string());
    }
    if open_loop.projection_policy_ref != M4_ATTENTION_POLICY_REF {
        return Err("m4_open_loop_policy_ref_invalid".to_string());
    }
    if open_loop.last_source_revision != open_loop.source_ref.source_revision {
        return Err("m4_open_loop_source_revision_mismatch".to_string());
    }
    if open_loop.owner_ref != open_loop.source_ref.source_owner_ref {
        return Err("m4_open_loop_owner_ref_mismatch".to_string());
    }
    m4_validate_identifier("open_loop_why_open_code", &open_loop.why_open_code)?;
    m4_validate_priority_for_source(&open_loop.priority_reason, &open_loop.source_ref)?;
    match open_loop.snoozed_until_utc.as_deref() {
        Some(value) => {
            m4_validate_utc("open_loop_snoozed_until_utc", value)?;
            if open_loop.status != M4OpenLoopStatus::Snoozed {
                return Err("m4_open_loop_snooze_state_mismatch".to_string());
            }
        }
        None if open_loop.status == M4OpenLoopStatus::Snoozed => {
            return Err("m4_open_loop_snooze_missing".to_string())
        }
        None => {}
    }
    match open_loop.closure_reason_code.as_deref() {
        Some(value) => {
            m4_validate_identifier("open_loop_closure_reason_code", value)?;
            if open_loop.status != M4OpenLoopStatus::Closed {
                return Err("m4_open_loop_closure_state_mismatch".to_string());
            }
        }
        None if open_loop.status == M4OpenLoopStatus::Closed => {
            return Err("m4_open_loop_closure_reason_missing".to_string())
        }
        None => {}
    }
    if open_loop.status == M4OpenLoopStatus::Snoozed && open_loop.closure_reason_code.is_some() {
        return Err("m4_open_loop_snoozed_closed_conflict".to_string());
    }
    if open_loop.status != M4OpenLoopStatus::Snoozed && open_loop.snoozed_until_utc.is_some() {
        return Err("m4_open_loop_snooze_state_mismatch".to_string());
    }
    Ok(())
}

pub(crate) fn m4_validate_personal_action(action: &M4PersonalAction) -> Result<(), String> {
    m4_validate_opaque_reference(
        "personal_action_explicit_user_command",
        &action.explicit_user_command_ref,
    )?;
    if action.personal_action_id != m4_personal_action_id(&action.explicit_user_command_ref)? {
        return Err("m4_personal_action_id_mismatch".to_string());
    }
    m4_validate_personal_action_title(&action.title)?;
    if let Some(due_at_utc) = action.due_at_utc.as_deref() {
        m4_validate_utc("personal_action_due_at_utc", due_at_utc)?;
    }
    Ok(())
}

pub(crate) fn m4_validate_notification(notification: &M4Notification) -> Result<(), String> {
    m4_validate_source_record_ref(&notification.source_ref)?;
    m4_validate_typed_reference("notification_subject_ref", &notification.subject_ref)?;
    m4_validate_coordination_code(
        "notification_purpose_code",
        &notification.notification_purpose_code,
    )?;
    if notification.notification_id
        != m4_notification_id(
            &notification.subject_ref,
            &notification.notification_purpose_code,
        )?
    {
        return Err("m4_notification_id_mismatch".to_string());
    }
    if notification.delivery_channel != M4_IN_APP_DELIVERY_CHANNEL {
        return Err("m4_notification_delivery_channel_invalid".to_string());
    }
    let created = m4_validate_utc("notification_created_at_utc", &notification.created_at_utc)?;
    let delivered = m4_validate_optional_utc(
        "notification_delivered_at_utc",
        notification.delivered_at_utc.as_deref(),
    )?;
    let read = m4_validate_optional_utc(
        "notification_read_at_utc",
        notification.read_at_utc.as_deref(),
    )?;
    let dismissed = m4_validate_optional_utc(
        "notification_dismissed_at_utc",
        notification.dismissed_at_utc.as_deref(),
    )?;
    if delivered.is_some_and(|value| value < created)
        || read.is_some_and(|value| delivered.map_or(true, |delivered| value < delivered))
        || dismissed.is_some_and(|value| {
            value < created
                || delivered.is_some_and(|delivered| value < delivered)
                || read.is_some_and(|read| value < read)
        })
    {
        return Err("m4_notification_timestamp_order_invalid".to_string());
    }
    match notification.status {
        M4NotificationStatus::Pending
            if delivered.is_none() && read.is_none() && dismissed.is_none() => {}
        M4NotificationStatus::Delivered
            if delivered.is_some() && read.is_none() && dismissed.is_none() => {}
        M4NotificationStatus::Read
            if delivered.is_some() && read.is_some() && dismissed.is_none() => {}
        M4NotificationStatus::Dismissed if dismissed.is_some() => {}
        _ => return Err("m4_notification_state_timestamp_mismatch".to_string()),
    }
    Ok(())
}

pub(crate) fn m4_validate_reminder(reminder: &M4Reminder) -> Result<(), String> {
    m4_validate_typed_reference("reminder_owner_ref", &reminder.owner_ref)?;
    m4_validate_opaque_reference(
        "reminder_explicit_schedule_command_id",
        &reminder.explicit_schedule_command_id,
    )?;
    if reminder.reminder_id
        != m4_reminder_id(&reminder.owner_ref, &reminder.explicit_schedule_command_id)?
    {
        return Err("m4_reminder_id_mismatch".to_string());
    }
    let scheduled = m4_validate_utc("reminder_scheduled_for_utc", &reminder.scheduled_for_utc)?;
    m4_validate_iana_timezone(&reminder.iana_timezone)?;
    let fired = m4_validate_optional_utc(
        "reminder_last_fired_at_utc",
        reminder.last_fired_at_utc.as_deref(),
    )?;
    let snoozed = m4_validate_optional_utc(
        "reminder_snoozed_until_utc",
        reminder.snoozed_until_utc.as_deref(),
    )?;
    if fired.is_some_and(|value| value < scheduled) {
        return Err("m4_reminder_timestamp_order_invalid".to_string());
    }
    match reminder.status {
        M4ReminderStatus::Scheduled if fired.is_none() && snoozed.is_none() => {}
        M4ReminderStatus::Fired if fired.is_some() && snoozed.is_none() => {}
        M4ReminderStatus::Snoozed if snoozed.is_some() => {}
        M4ReminderStatus::Dismissed | M4ReminderStatus::Cancelled if snoozed.is_none() => {}
        _ => return Err("m4_reminder_state_timestamp_mismatch".to_string()),
    }
    Ok(())
}

fn m4_validate_priority_for_source(
    priority: &M4PriorityReason,
    source_ref: &M4SourceRecordRef,
) -> Result<(), String> {
    let expected = m4_priority_reason(&source_ref.attention_signals)?;
    if priority != &expected {
        return Err("m4_priority_reason_source_mismatch".to_string());
    }
    Ok(())
}

fn m4_validate_utc(field: &str, value: &str) -> Result<M4UtcSortKey, String> {
    m4_parse_rfc3339_utc_key(value).ok_or_else(|| format!("m4_utc_timestamp_invalid:{field}"))
}

fn m4_validate_optional_utc(
    field: &str,
    value: Option<&str>,
) -> Result<Option<M4UtcSortKey>, String> {
    value.map(|value| m4_validate_utc(field, value)).transpose()
}

fn m4_validate_opaque_reference(field: &str, value: &str) -> Result<(), String> {
    m4_validate_reference_text(field, value)?;
    if !m4_is_opaque_reference(value) {
        return Err(format!("m4_opaque_reference_invalid:{field}"));
    }
    Ok(())
}

fn m4_validate_typed_reference(field: &str, value: &str) -> Result<(), String> {
    m4_validate_reference_text(field, value)?;
    if !(m4_is_opaque_reference(value) || m4_is_m4_deterministic_id(value)) {
        return Err(format!("m4_typed_reference_invalid:{field}"));
    }
    Ok(())
}

fn m4_is_m4_deterministic_id(value: &str) -> bool {
    let Some((prefix, digest)) = value.split_once(':') else {
        return false;
    };
    matches!(
        prefix,
        "source"
            | "source-event"
            | "inbox"
            | "open-loop"
            | "personal-action"
            | "notification"
            | "reminder"
            | "decision-projection"
    ) && m4_is_lower_hex_digest(digest)
}

fn m4_validate_iana_timezone(value: &str) -> Result<(), String> {
    let valid = !value.is_empty()
        && value.len() <= 128
        && value.contains('/')
        && !value.starts_with('/')
        && !value.ends_with('/')
        && value.split('/').all(|segment| {
            !segment.is_empty()
                && segment
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'+'))
        });
    if !valid {
        return Err("m4_reminder_iana_timezone_invalid".to_string());
    }
    Ok(())
}

/// Enforce the local title boundary before SQLite. The database repeats the
/// structural checks, while this admission gate additionally rejects every
/// control character so a value accepted for write cannot later fail the
/// read-model validator. Do not add broad keyword heuristics here; ordinary
/// titles such as email-related work remain valid user text.
fn m4_validate_personal_action_title(value: &str) -> Result<(), String> {
    let character_count = value.chars().count();
    let ascii_lowercase = value.to_ascii_lowercase();
    let valid = (1..=160).contains(&character_count)
        && !value.starts_with(' ')
        && !value.ends_with(' ')
        && !value.chars().any(char::is_control)
        && !value.contains('\n')
        && !value.contains('\r')
        && !value.contains('\\')
        && !value.starts_with('/')
        && !ascii_lowercase.starts_with("http://")
        && !ascii_lowercase.starts_with("https://");
    if !valid {
        return Err("m4_personal_action_title_invalid".to_string());
    }
    Ok(())
}

fn m4_validate_coordination_code(field: &str, value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 96
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(format!("m4_coordination_code_invalid:{field}"));
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4CoordinationCommandMetadata {
    pub(crate) idempotency_key: String,
    pub(crate) occurred_at_utc: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4StateTransitionResult<T> {
    pub(crate) aggregate: T,
    pub(crate) previous_revision: u64,
    pub(crate) idempotency_fingerprint: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4CreateResult<T> {
    pub(crate) aggregate: T,
    pub(crate) idempotency_fingerprint: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4InboxReadCommand {
    pub(crate) inbox_item_id: String,
    pub(crate) expected_revision: u64,
    pub(crate) metadata: M4CoordinationCommandMetadata,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4InboxDismissCommand {
    pub(crate) inbox_item_id: String,
    pub(crate) expected_revision: u64,
    pub(crate) metadata: M4CoordinationCommandMetadata,
}

pub(crate) fn m4_mark_inbox_item_read(
    item: &M4InboxItem,
    command: &M4InboxReadCommand,
) -> Result<M4StateTransitionResult<M4InboxItem>, String> {
    m4_validate_inbox_item(item)?;
    let fingerprint = m4_require_cas(
        "inbox-read",
        &item.inbox_item_id,
        item.revision,
        &command.inbox_item_id,
        command.expected_revision,
        &command.metadata,
        &[],
    )?;
    if item.status != M4InboxItemStatus::New {
        return Err("m4_inbox_read_transition_not_allowed".to_string());
    }
    let mut next = item.clone();
    next.status = M4InboxItemStatus::Read;
    next.revision = next
        .revision
        .checked_add(1)
        .ok_or_else(|| "m4_revision_overflow".to_string())?;
    Ok(M4StateTransitionResult {
        aggregate: next,
        previous_revision: item.revision,
        idempotency_fingerprint: fingerprint,
    })
}

pub(crate) fn m4_dismiss_inbox_item(
    item: &M4InboxItem,
    command: &M4InboxDismissCommand,
) -> Result<M4StateTransitionResult<M4InboxItem>, String> {
    m4_validate_inbox_item(item)?;
    let fingerprint = m4_require_cas(
        "inbox-dismiss",
        &item.inbox_item_id,
        item.revision,
        &command.inbox_item_id,
        command.expected_revision,
        &command.metadata,
        &[],
    )?;
    if !matches!(
        item.status,
        M4InboxItemStatus::New | M4InboxItemStatus::Read
    ) {
        return Err("m4_inbox_dismiss_transition_not_allowed".to_string());
    }
    let mut next = item.clone();
    next.status = M4InboxItemStatus::Dismissed;
    next.revision = next
        .revision
        .checked_add(1)
        .ok_or_else(|| "m4_revision_overflow".to_string())?;
    Ok(M4StateTransitionResult {
        aggregate: next,
        previous_revision: item.revision,
        idempotency_fingerprint: fingerprint,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4RefreshInboxFromSourceCommand {
    pub(crate) inbox_item_id: String,
    pub(crate) expected_revision: u64,
    pub(crate) source_ref: M4SourceRecordRef,
    pub(crate) metadata: M4CoordinationCommandMetadata,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4ExpireInboxItemCommand {
    pub(crate) inbox_item_id: String,
    pub(crate) expected_revision: u64,
    pub(crate) source_ref: M4SourceRecordRef,
    pub(crate) metadata: M4CoordinationCommandMetadata,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4QuarantineInboxItemCommand {
    pub(crate) inbox_item_id: String,
    pub(crate) expected_revision: u64,
    pub(crate) quarantine_reason_code: String,
    pub(crate) metadata: M4CoordinationCommandMetadata,
}

pub(crate) fn m4_refresh_inbox_item_from_new_source(
    item: &M4InboxItem,
    command: &M4RefreshInboxFromSourceCommand,
) -> Result<M4StateTransitionResult<M4InboxItem>, String> {
    m4_validate_inbox_item(item)?;
    let source_fingerprint = m4_source_record_fingerprint(&command.source_ref)?;
    let fingerprint = m4_require_cas(
        "inbox-new-source-revision",
        &item.inbox_item_id,
        item.revision,
        &command.inbox_item_id,
        command.expected_revision,
        &command.metadata,
        &[source_fingerprint.as_str()],
    )?;
    if !matches!(
        item.status,
        M4InboxItemStatus::New | M4InboxItemStatus::Read | M4InboxItemStatus::Dismissed
    ) {
        return Err("m4_inbox_new_source_transition_not_allowed".to_string());
    }
    m4_validate_source_record_ref(&command.source_ref)?;
    m4_require_same_source_identity(&item.source_ref, &command.source_ref)?;
    if command.source_ref.source_revision <= item.source_ref.source_revision {
        return Err("m4_inbox_source_revision_not_newer".to_string());
    }
    let mut next = item.clone();
    next.source_ref = command.source_ref.clone();
    next.status = M4InboxItemStatus::New;
    next.priority_reason = m4_priority_reason(&command.source_ref.attention_signals)?;
    next.last_source_change_at_utc = command.source_ref.occurred_at_utc.clone();
    next.scrubbed_summary_ref = command.source_ref.scrubbed_summary_ref.clone();
    next.sensitivity = command.source_ref.sensitivity.clone();
    next.revision = next
        .revision
        .checked_add(1)
        .ok_or_else(|| "m4_revision_overflow".to_string())?;
    m4_validate_inbox_item(&next)?;
    Ok(M4StateTransitionResult {
        aggregate: next,
        previous_revision: item.revision,
        idempotency_fingerprint: fingerprint,
    })
}

pub(crate) fn m4_expire_inbox_item_from_source(
    item: &M4InboxItem,
    command: &M4ExpireInboxItemCommand,
) -> Result<M4StateTransitionResult<M4InboxItem>, String> {
    m4_validate_inbox_item(item)?;
    let source_fingerprint = m4_source_record_fingerprint(&command.source_ref)?;
    let fingerprint = m4_require_cas(
        "inbox-owner-expiry",
        &item.inbox_item_id,
        item.revision,
        &command.inbox_item_id,
        command.expected_revision,
        &command.metadata,
        &[source_fingerprint.as_str()],
    )?;
    if !matches!(
        item.status,
        M4InboxItemStatus::New | M4InboxItemStatus::Read | M4InboxItemStatus::Dismissed
    ) {
        return Err("m4_inbox_expiry_transition_not_allowed".to_string());
    }
    m4_validate_source_record_ref(&command.source_ref)?;
    m4_require_same_source_identity(&item.source_ref, &command.source_ref)?;
    if command.source_ref.source_status != M4SourceStatus::Expired {
        return Err("m4_inbox_owner_expiry_status_required".to_string());
    }
    if command.source_ref.source_revision <= item.source_ref.source_revision {
        return Err("m4_inbox_expiry_source_revision_not_newer".to_string());
    }
    let mut next = item.clone();
    next.source_ref = command.source_ref.clone();
    next.status = M4InboxItemStatus::Expired;
    next.priority_reason = m4_priority_reason(&command.source_ref.attention_signals)?;
    next.last_source_change_at_utc = command.source_ref.occurred_at_utc.clone();
    next.scrubbed_summary_ref = command.source_ref.scrubbed_summary_ref.clone();
    next.sensitivity = command.source_ref.sensitivity.clone();
    next.revision = next
        .revision
        .checked_add(1)
        .ok_or_else(|| "m4_revision_overflow".to_string())?;
    m4_validate_inbox_item(&next)?;
    Ok(M4StateTransitionResult {
        aggregate: next,
        previous_revision: item.revision,
        idempotency_fingerprint: fingerprint,
    })
}

pub(crate) fn m4_quarantine_inbox_item(
    item: &M4InboxItem,
    command: &M4QuarantineInboxItemCommand,
) -> Result<M4StateTransitionResult<M4InboxItem>, String> {
    m4_validate_inbox_item(item)?;
    let fingerprint = m4_require_cas(
        "inbox-quarantine",
        &item.inbox_item_id,
        item.revision,
        &command.inbox_item_id,
        command.expected_revision,
        &command.metadata,
        &[command.quarantine_reason_code.as_str()],
    )?;
    if item.status == M4InboxItemStatus::Quarantined {
        return Err("m4_inbox_quarantine_transition_not_allowed".to_string());
    }
    m4_validate_identifier(
        "inbox_quarantine_reason_code",
        &command.quarantine_reason_code,
    )?;
    let mut next = item.clone();
    next.status = M4InboxItemStatus::Quarantined;
    next.revision = next
        .revision
        .checked_add(1)
        .ok_or_else(|| "m4_revision_overflow".to_string())?;
    Ok(M4StateTransitionResult {
        aggregate: next,
        previous_revision: item.revision,
        idempotency_fingerprint: fingerprint,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4AcknowledgeOpenLoopCommand {
    pub(crate) open_loop_id: String,
    pub(crate) expected_revision: u64,
    pub(crate) metadata: M4CoordinationCommandMetadata,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4SnoozeOpenLoopCommand {
    pub(crate) open_loop_id: String,
    pub(crate) expected_revision: u64,
    pub(crate) snoozed_until_utc: String,
    pub(crate) metadata: M4CoordinationCommandMetadata,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4CloseOpenLoopCommand {
    pub(crate) open_loop_id: String,
    pub(crate) expected_revision: u64,
    pub(crate) metadata: M4CoordinationCommandMetadata,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4DismissOpenLoopCommand {
    pub(crate) open_loop_id: String,
    pub(crate) expected_revision: u64,
    pub(crate) metadata: M4CoordinationCommandMetadata,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4ReopenOpenLoopCommand {
    pub(crate) open_loop_id: String,
    pub(crate) expected_revision: u64,
    pub(crate) metadata: M4CoordinationCommandMetadata,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4CloseOpenLoopFromTerminalSourceCommand {
    pub(crate) open_loop_id: String,
    pub(crate) expected_revision: u64,
    pub(crate) source_ref: M4SourceRecordRef,
    pub(crate) metadata: M4CoordinationCommandMetadata,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4ReopenOpenLoopFromSourceCommand {
    pub(crate) open_loop_id: String,
    pub(crate) expected_revision: u64,
    pub(crate) source_ref: M4SourceRecordRef,
    pub(crate) metadata: M4CoordinationCommandMetadata,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4OpenLoopClockCommand {
    pub(crate) open_loop_id: String,
    pub(crate) expected_revision: u64,
    pub(crate) metadata: M4CoordinationCommandMetadata,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4CarryOverOpenLoopCommand {
    pub(crate) open_loop_id: String,
    pub(crate) expected_revision: u64,
    pub(crate) metadata: M4CoordinationCommandMetadata,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4CarryOverOpenLoopResult {
    pub(crate) open_loop_id: String,
    pub(crate) retained_revision: u64,
    pub(crate) idempotency_fingerprint: String,
}

pub(crate) fn m4_acknowledge_open_loop(
    open_loop: &M4OpenLoop,
    command: &M4AcknowledgeOpenLoopCommand,
) -> Result<M4StateTransitionResult<M4OpenLoop>, String> {
    m4_validate_open_loop(open_loop)?;
    let fingerprint = m4_require_cas(
        "open-loop-acknowledge",
        &open_loop.open_loop_id,
        open_loop.revision,
        &command.open_loop_id,
        command.expected_revision,
        &command.metadata,
        &[],
    )?;
    if open_loop.status != M4OpenLoopStatus::Open {
        return Err("m4_open_loop_acknowledge_transition_not_allowed".to_string());
    }
    let mut next = open_loop.clone();
    next.status = M4OpenLoopStatus::Acknowledged;
    next.revision = next
        .revision
        .checked_add(1)
        .ok_or_else(|| "m4_revision_overflow".to_string())?;
    Ok(M4StateTransitionResult {
        aggregate: next,
        previous_revision: open_loop.revision,
        idempotency_fingerprint: fingerprint,
    })
}

pub(crate) fn m4_snooze_open_loop(
    open_loop: &M4OpenLoop,
    command: &M4SnoozeOpenLoopCommand,
) -> Result<M4StateTransitionResult<M4OpenLoop>, String> {
    m4_validate_open_loop(open_loop)?;
    let fingerprint = m4_require_cas(
        "open-loop-snooze",
        &open_loop.open_loop_id,
        open_loop.revision,
        &command.open_loop_id,
        command.expected_revision,
        &command.metadata,
        &[command.snoozed_until_utc.as_str()],
    )?;
    if !matches!(
        open_loop.status,
        M4OpenLoopStatus::Open | M4OpenLoopStatus::Acknowledged
    ) {
        return Err("m4_open_loop_snooze_transition_not_allowed".to_string());
    }
    let occurred_at = m4_validate_utc(
        "open_loop_snooze_occurred_at_utc",
        &command.metadata.occurred_at_utc,
    )?;
    let snoozed_until = m4_validate_utc("open_loop_snoozed_until_utc", &command.snoozed_until_utc)?;
    if snoozed_until <= occurred_at {
        return Err("m4_open_loop_snooze_time_not_future".to_string());
    }
    let mut next = open_loop.clone();
    next.status = M4OpenLoopStatus::Snoozed;
    next.snoozed_until_utc = Some(command.snoozed_until_utc.clone());
    next.closure_reason_code = None;
    next.revision = next
        .revision
        .checked_add(1)
        .ok_or_else(|| "m4_revision_overflow".to_string())?;
    Ok(M4StateTransitionResult {
        aggregate: next,
        previous_revision: open_loop.revision,
        idempotency_fingerprint: fingerprint,
    })
}

/// Closing a coordination loop changes only M4's local tracking state. The
/// retained source reference is intentionally byte-for-byte unchanged.
pub(crate) fn m4_close_open_loop(
    open_loop: &M4OpenLoop,
    command: &M4CloseOpenLoopCommand,
) -> Result<M4StateTransitionResult<M4OpenLoop>, String> {
    m4_validate_open_loop(open_loop)?;
    let fingerprint = m4_require_cas(
        "open-loop-close",
        &open_loop.open_loop_id,
        open_loop.revision,
        &command.open_loop_id,
        command.expected_revision,
        &command.metadata,
        &[],
    )?;
    if !matches!(
        open_loop.status,
        M4OpenLoopStatus::Open | M4OpenLoopStatus::Acknowledged | M4OpenLoopStatus::Snoozed
    ) {
        return Err("m4_open_loop_close_transition_not_allowed".to_string());
    }
    let mut next = open_loop.clone();
    next.status = M4OpenLoopStatus::Closed;
    next.snoozed_until_utc = None;
    next.closure_reason_code = Some(M4_COORDINATION_CLOSE_REASON.to_string());
    next.revision = next
        .revision
        .checked_add(1)
        .ok_or_else(|| "m4_revision_overflow".to_string())?;
    Ok(M4StateTransitionResult {
        aggregate: next,
        previous_revision: open_loop.revision,
        idempotency_fingerprint: fingerprint,
    })
}

pub(crate) fn m4_dismiss_open_loop(
    open_loop: &M4OpenLoop,
    command: &M4DismissOpenLoopCommand,
) -> Result<M4StateTransitionResult<M4OpenLoop>, String> {
    m4_validate_open_loop(open_loop)?;
    let fingerprint = m4_require_cas(
        "open-loop-dismiss",
        &open_loop.open_loop_id,
        open_loop.revision,
        &command.open_loop_id,
        command.expected_revision,
        &command.metadata,
        &[],
    )?;
    if !matches!(
        open_loop.status,
        M4OpenLoopStatus::Open | M4OpenLoopStatus::Acknowledged | M4OpenLoopStatus::Snoozed
    ) {
        return Err("m4_open_loop_dismiss_transition_not_allowed".to_string());
    }
    let mut next = open_loop.clone();
    next.status = M4OpenLoopStatus::Dismissed;
    next.snoozed_until_utc = None;
    next.closure_reason_code = None;
    next.revision = next
        .revision
        .checked_add(1)
        .ok_or_else(|| "m4_revision_overflow".to_string())?;
    Ok(M4StateTransitionResult {
        aggregate: next,
        previous_revision: open_loop.revision,
        idempotency_fingerprint: fingerprint,
    })
}

pub(crate) fn m4_reopen_open_loop(
    open_loop: &M4OpenLoop,
    command: &M4ReopenOpenLoopCommand,
) -> Result<M4StateTransitionResult<M4OpenLoop>, String> {
    m4_validate_open_loop(open_loop)?;
    let fingerprint = m4_require_cas(
        "open-loop-reopen",
        &open_loop.open_loop_id,
        open_loop.revision,
        &command.open_loop_id,
        command.expected_revision,
        &command.metadata,
        &[],
    )?;
    if !matches!(
        open_loop.status,
        M4OpenLoopStatus::Closed | M4OpenLoopStatus::Dismissed
    ) {
        return Err("m4_open_loop_reopen_transition_not_allowed".to_string());
    }
    let mut next = open_loop.clone();
    next.status = M4OpenLoopStatus::Open;
    next.snoozed_until_utc = None;
    next.closure_reason_code = None;
    next.revision = next
        .revision
        .checked_add(1)
        .ok_or_else(|| "m4_revision_overflow".to_string())?;
    Ok(M4StateTransitionResult {
        aggregate: next,
        previous_revision: open_loop.revision,
        idempotency_fingerprint: fingerprint,
    })
}

/// A terminal source snapshot can stop local tracking, but this only projects
/// an already-owned source fact. It does not issue a source-owner command or
/// change any source field outside the copied source reference.
pub(crate) fn m4_close_open_loop_from_terminal_source(
    open_loop: &M4OpenLoop,
    command: &M4CloseOpenLoopFromTerminalSourceCommand,
) -> Result<M4StateTransitionResult<M4OpenLoop>, String> {
    m4_validate_open_loop(open_loop)?;
    let source_fingerprint = m4_source_record_fingerprint(&command.source_ref)?;
    let fingerprint = m4_require_cas(
        "open-loop-source-terminal-close",
        &open_loop.open_loop_id,
        open_loop.revision,
        &command.open_loop_id,
        command.expected_revision,
        &command.metadata,
        &[source_fingerprint.as_str()],
    )?;
    if !matches!(
        open_loop.status,
        M4OpenLoopStatus::Open
            | M4OpenLoopStatus::Acknowledged
            | M4OpenLoopStatus::Snoozed
            | M4OpenLoopStatus::Dismissed
    ) {
        return Err("m4_open_loop_source_terminal_close_not_allowed".to_string());
    }
    m4_require_same_source_identity(&open_loop.source_ref, &command.source_ref)?;
    if command.source_ref.source_revision <= open_loop.last_source_revision {
        return Err("m4_open_loop_source_revision_not_newer".to_string());
    }
    let Some(closure_reason_code) = command.source_ref.source_status.terminal_closure_reason()
    else {
        return Err("m4_open_loop_terminal_source_status_required".to_string());
    };
    let mut next = m4_open_loop_with_new_source(open_loop, &command.source_ref)?;
    next.status = M4OpenLoopStatus::Closed;
    next.snoozed_until_utc = None;
    next.closure_reason_code = Some(closure_reason_code.to_string());
    next.revision = next
        .revision
        .checked_add(1)
        .ok_or_else(|| "m4_revision_overflow".to_string())?;
    m4_validate_open_loop(&next)?;
    Ok(M4StateTransitionResult {
        aggregate: next,
        previous_revision: open_loop.revision,
        idempotency_fingerprint: fingerprint,
    })
}

/// A later, admitted non-terminal source revision may reopen only a closed or
/// dismissed policy loop when the frozen deterministic attention predicate is
/// true. It creates no new OpenLoop and retains its deterministic ID.
pub(crate) fn m4_reopen_open_loop_from_new_source(
    open_loop: &M4OpenLoop,
    command: &M4ReopenOpenLoopFromSourceCommand,
) -> Result<M4StateTransitionResult<M4OpenLoop>, String> {
    m4_validate_open_loop(open_loop)?;
    let source_fingerprint = m4_source_record_fingerprint(&command.source_ref)?;
    let fingerprint = m4_require_cas(
        "open-loop-source-reopen",
        &open_loop.open_loop_id,
        open_loop.revision,
        &command.open_loop_id,
        command.expected_revision,
        &command.metadata,
        &[source_fingerprint.as_str()],
    )?;
    if !matches!(
        open_loop.status,
        M4OpenLoopStatus::Closed | M4OpenLoopStatus::Dismissed
    ) {
        return Err("m4_open_loop_source_reopen_not_allowed".to_string());
    }
    m4_require_same_source_identity(&open_loop.source_ref, &command.source_ref)?;
    if command.source_ref.source_revision <= open_loop.last_source_revision {
        return Err("m4_open_loop_source_revision_not_newer".to_string());
    }
    if !m4_source_ref_matches_automatic_open_loop_policy(&command.source_ref) {
        return Err("m4_open_loop_source_attention_policy_not_matched".to_string());
    }
    let mut next = m4_open_loop_with_new_source(open_loop, &command.source_ref)?;
    next.status = M4OpenLoopStatus::Open;
    next.why_open_code = "AUTOMATIC_ATTENTION_POLICY".to_string();
    next.snoozed_until_utc = None;
    next.closure_reason_code = None;
    next.revision = next
        .revision
        .checked_add(1)
        .ok_or_else(|| "m4_revision_overflow".to_string())?;
    m4_validate_open_loop(&next)?;
    Ok(M4StateTransitionResult {
        aggregate: next,
        previous_revision: open_loop.revision,
        idempotency_fingerprint: fingerprint,
    })
}

pub(crate) fn m4_reopen_snoozed_open_loop_on_clock(
    open_loop: &M4OpenLoop,
    command: &M4OpenLoopClockCommand,
) -> Result<M4StateTransitionResult<M4OpenLoop>, String> {
    m4_validate_open_loop(open_loop)?;
    let fingerprint = m4_require_cas(
        "open-loop-snooze-clock",
        &open_loop.open_loop_id,
        open_loop.revision,
        &command.open_loop_id,
        command.expected_revision,
        &command.metadata,
        &[],
    )?;
    if open_loop.status != M4OpenLoopStatus::Snoozed {
        return Err("m4_open_loop_clock_transition_not_allowed".to_string());
    }
    let observed_at = m4_validate_utc(
        "open_loop_clock_observed_at_utc",
        &command.metadata.occurred_at_utc,
    )?;
    let snoozed_until = m4_validate_utc(
        "open_loop_clock_snoozed_until_utc",
        open_loop
            .snoozed_until_utc
            .as_deref()
            .ok_or_else(|| "m4_open_loop_snooze_missing".to_string())?,
    )?;
    if observed_at < snoozed_until {
        return Err("m4_open_loop_snooze_not_due".to_string());
    }
    let mut next = open_loop.clone();
    next.status = M4OpenLoopStatus::Open;
    next.snoozed_until_utc = None;
    next.revision = next
        .revision
        .checked_add(1)
        .ok_or_else(|| "m4_revision_overflow".to_string())?;
    Ok(M4StateTransitionResult {
        aggregate: next,
        previous_revision: open_loop.revision,
        idempotency_fingerprint: fingerprint,
    })
}

/// Carry-over is a selection, not a state transition: it retains the object
/// ID and revision and deliberately does not construct a daily-domain object.
pub(crate) fn m4_select_open_loop_for_carry_over(
    open_loop: &M4OpenLoop,
    command: &M4CarryOverOpenLoopCommand,
) -> Result<M4CarryOverOpenLoopResult, String> {
    m4_validate_open_loop(open_loop)?;
    let fingerprint = m4_require_cas(
        "open-loop-carry-over",
        &open_loop.open_loop_id,
        open_loop.revision,
        &command.open_loop_id,
        command.expected_revision,
        &command.metadata,
        &[],
    )?;
    if !matches!(
        open_loop.status,
        M4OpenLoopStatus::Open | M4OpenLoopStatus::Acknowledged
    ) {
        return Err("m4_open_loop_carry_over_not_eligible".to_string());
    }
    Ok(M4CarryOverOpenLoopResult {
        open_loop_id: open_loop.open_loop_id.clone(),
        retained_revision: open_loop.revision,
        idempotency_fingerprint: fingerprint,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4CreateNotificationCommand {
    pub(crate) source_ref: M4SourceRecordRef,
    pub(crate) subject_ref: String,
    pub(crate) notification_purpose_code: String,
    pub(crate) delivery_channel: String,
    pub(crate) metadata: M4CoordinationCommandMetadata,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum M4NotificationTransition {
    Deliver,
    Read,
    Dismiss,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4NotificationTransitionCommand {
    pub(crate) notification_id: String,
    pub(crate) expected_revision: u64,
    pub(crate) transition: M4NotificationTransition,
    pub(crate) metadata: M4CoordinationCommandMetadata,
}

pub(crate) fn m4_create_notification(
    command: &M4CreateNotificationCommand,
) -> Result<M4CreateResult<M4Notification>, String> {
    m4_validate_source_record_ref(&command.source_ref)?;
    let source_fingerprint = m4_source_record_fingerprint(&command.source_ref)?;
    m4_validate_typed_reference("notification_subject_ref", &command.subject_ref)?;
    m4_validate_coordination_code(
        "notification_purpose_code",
        &command.notification_purpose_code,
    )?;
    if command.delivery_channel != M4_IN_APP_DELIVERY_CHANNEL {
        return Err("m4_notification_delivery_channel_invalid".to_string());
    }
    m4_validate_coordination_metadata(&command.metadata)?;
    let notification_id =
        m4_notification_id(&command.subject_ref, &command.notification_purpose_code)?;
    let notification = M4Notification {
        notification_id: notification_id.clone(),
        source_ref: command.source_ref.clone(),
        subject_ref: command.subject_ref.clone(),
        notification_purpose_code: command.notification_purpose_code.clone(),
        delivery_channel: command.delivery_channel.clone(),
        status: M4NotificationStatus::Pending,
        created_at_utc: command.metadata.occurred_at_utc.clone(),
        delivered_at_utc: None,
        read_at_utc: None,
        dismissed_at_utc: None,
        revision: 1,
    };
    m4_validate_notification(&notification)?;
    Ok(M4CreateResult {
        aggregate: notification,
        idempotency_fingerprint: m4_command_fingerprint(
            "notification-create",
            &notification_id,
            0,
            &command.metadata,
            &[
                source_fingerprint.as_str(),
                command.subject_ref.as_str(),
                command.notification_purpose_code.as_str(),
                command.delivery_channel.as_str(),
            ],
        )?,
    })
}

pub(crate) fn m4_transition_notification(
    notification: &M4Notification,
    command: &M4NotificationTransitionCommand,
) -> Result<M4StateTransitionResult<M4Notification>, String> {
    m4_validate_notification(notification)?;
    let operation = match command.transition {
        M4NotificationTransition::Deliver => "notification-deliver",
        M4NotificationTransition::Read => "notification-read",
        M4NotificationTransition::Dismiss => "notification-dismiss",
    };
    let fingerprint = m4_require_cas(
        operation,
        &notification.notification_id,
        notification.revision,
        &command.notification_id,
        command.expected_revision,
        &command.metadata,
        &[],
    )?;
    let mut next = notification.clone();
    match command.transition {
        M4NotificationTransition::Deliver
            if notification.status == M4NotificationStatus::Pending =>
        {
            next.status = M4NotificationStatus::Delivered;
            next.delivered_at_utc = Some(command.metadata.occurred_at_utc.clone());
        }
        M4NotificationTransition::Read
            if notification.status == M4NotificationStatus::Delivered =>
        {
            next.status = M4NotificationStatus::Read;
            next.read_at_utc = Some(command.metadata.occurred_at_utc.clone());
        }
        M4NotificationTransition::Dismiss
            if matches!(
                notification.status,
                M4NotificationStatus::Pending
                    | M4NotificationStatus::Delivered
                    | M4NotificationStatus::Read
            ) =>
        {
            next.status = M4NotificationStatus::Dismissed;
            next.dismissed_at_utc = Some(command.metadata.occurred_at_utc.clone());
        }
        _ => return Err("m4_notification_transition_not_allowed".to_string()),
    }
    next.revision = next
        .revision
        .checked_add(1)
        .ok_or_else(|| "m4_revision_overflow".to_string())?;
    m4_validate_notification(&next)?;
    Ok(M4StateTransitionResult {
        aggregate: next,
        previous_revision: notification.revision,
        idempotency_fingerprint: fingerprint,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4CreateReminderCommand {
    pub(crate) owner_ref: String,
    pub(crate) explicit_schedule_command_id: String,
    pub(crate) scheduled_for_utc: String,
    pub(crate) iana_timezone: String,
    pub(crate) metadata: M4CoordinationCommandMetadata,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum M4ReminderTransition {
    Fire,
    Snooze { snoozed_until_utc: String },
    Dismiss,
    Cancel,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4ReminderTransitionCommand {
    pub(crate) reminder_id: String,
    pub(crate) expected_revision: u64,
    pub(crate) transition: M4ReminderTransition,
    pub(crate) metadata: M4CoordinationCommandMetadata,
}

pub(crate) fn m4_create_reminder(
    command: &M4CreateReminderCommand,
) -> Result<M4CreateResult<M4Reminder>, String> {
    m4_validate_typed_reference("reminder_owner_ref", &command.owner_ref)?;
    m4_validate_opaque_reference(
        "reminder_explicit_schedule_command_id",
        &command.explicit_schedule_command_id,
    )?;
    m4_validate_utc("reminder_scheduled_for_utc", &command.scheduled_for_utc)?;
    m4_validate_iana_timezone(&command.iana_timezone)?;
    m4_validate_coordination_metadata(&command.metadata)?;
    let reminder_id = m4_reminder_id(&command.owner_ref, &command.explicit_schedule_command_id)?;
    let reminder = M4Reminder {
        reminder_id: reminder_id.clone(),
        owner_ref: command.owner_ref.clone(),
        explicit_schedule_command_id: command.explicit_schedule_command_id.clone(),
        scheduled_for_utc: command.scheduled_for_utc.clone(),
        iana_timezone: command.iana_timezone.clone(),
        status: M4ReminderStatus::Scheduled,
        last_fired_at_utc: None,
        snoozed_until_utc: None,
        revision: 1,
    };
    m4_validate_reminder(&reminder)?;
    Ok(M4CreateResult {
        aggregate: reminder,
        idempotency_fingerprint: m4_command_fingerprint(
            "reminder-create",
            &reminder_id,
            0,
            &command.metadata,
            &[
                command.owner_ref.as_str(),
                command.explicit_schedule_command_id.as_str(),
                command.scheduled_for_utc.as_str(),
                command.iana_timezone.as_str(),
            ],
        )?,
    })
}

pub(crate) fn m4_transition_reminder(
    reminder: &M4Reminder,
    command: &M4ReminderTransitionCommand,
) -> Result<M4StateTransitionResult<M4Reminder>, String> {
    m4_validate_reminder(reminder)?;
    let operation = match command.transition {
        M4ReminderTransition::Fire => "reminder-fire",
        M4ReminderTransition::Snooze { .. } => "reminder-snooze",
        M4ReminderTransition::Dismiss => "reminder-dismiss",
        M4ReminderTransition::Cancel => "reminder-cancel",
    };
    let immutable_transition_fields = match &command.transition {
        M4ReminderTransition::Snooze { snoozed_until_utc } => vec![snoozed_until_utc.as_str()],
        M4ReminderTransition::Fire
        | M4ReminderTransition::Dismiss
        | M4ReminderTransition::Cancel => Vec::new(),
    };
    let fingerprint = m4_require_cas(
        operation,
        &reminder.reminder_id,
        reminder.revision,
        &command.reminder_id,
        command.expected_revision,
        &command.metadata,
        &immutable_transition_fields,
    )?;
    let occurred_at = m4_validate_utc(
        "reminder_transition_occurred_at_utc",
        &command.metadata.occurred_at_utc,
    )?;
    let scheduled_for = m4_validate_utc("reminder_scheduled_for_utc", &reminder.scheduled_for_utc)?;
    let mut next = reminder.clone();
    match &command.transition {
        M4ReminderTransition::Fire if reminder.status == M4ReminderStatus::Scheduled => {
            if occurred_at < scheduled_for {
                return Err("m4_reminder_fire_not_due".to_string());
            }
            next.status = M4ReminderStatus::Fired;
            next.last_fired_at_utc = Some(command.metadata.occurred_at_utc.clone());
        }
        M4ReminderTransition::Fire if reminder.status == M4ReminderStatus::Snoozed => {
            let snoozed_until = m4_validate_utc(
                "reminder_snoozed_until_utc",
                reminder
                    .snoozed_until_utc
                    .as_deref()
                    .ok_or_else(|| "m4_reminder_snooze_missing".to_string())?,
            )?;
            if occurred_at < snoozed_until {
                return Err("m4_reminder_snooze_not_due".to_string());
            }
            next.status = M4ReminderStatus::Fired;
            next.last_fired_at_utc = Some(command.metadata.occurred_at_utc.clone());
            next.snoozed_until_utc = None;
        }
        M4ReminderTransition::Snooze { snoozed_until_utc }
            if matches!(
                reminder.status,
                M4ReminderStatus::Scheduled | M4ReminderStatus::Fired
            ) =>
        {
            let snoozed_until = m4_validate_utc("reminder_snoozed_until_utc", snoozed_until_utc)?;
            if snoozed_until <= occurred_at {
                return Err("m4_reminder_snooze_time_not_future".to_string());
            }
            next.status = M4ReminderStatus::Snoozed;
            next.snoozed_until_utc = Some(snoozed_until_utc.clone());
        }
        M4ReminderTransition::Dismiss
            if matches!(
                reminder.status,
                M4ReminderStatus::Scheduled | M4ReminderStatus::Fired | M4ReminderStatus::Snoozed
            ) =>
        {
            next.status = M4ReminderStatus::Dismissed;
            next.snoozed_until_utc = None;
        }
        M4ReminderTransition::Cancel
            if matches!(
                reminder.status,
                M4ReminderStatus::Scheduled | M4ReminderStatus::Snoozed
            ) =>
        {
            next.status = M4ReminderStatus::Cancelled;
            next.snoozed_until_utc = None;
        }
        _ => return Err("m4_reminder_transition_not_allowed".to_string()),
    }
    next.revision = next
        .revision
        .checked_add(1)
        .ok_or_else(|| "m4_revision_overflow".to_string())?;
    m4_validate_reminder(&next)?;
    Ok(M4StateTransitionResult {
        aggregate: next,
        previous_revision: reminder.revision,
        idempotency_fingerprint: fingerprint,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum M4DecisionLocalTransition {
    Read,
    Dismiss,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4DecisionLocalTransitionCommand {
    pub(crate) decision_projection_id: String,
    pub(crate) expected_revision: u64,
    pub(crate) transition: M4DecisionLocalTransition,
    pub(crate) metadata: M4CoordinationCommandMetadata,
}

pub(crate) fn m4_transition_decision_local_visibility(
    decision: &M4DecisionProjection,
    command: &M4DecisionLocalTransitionCommand,
) -> Result<M4StateTransitionResult<M4DecisionProjection>, String> {
    m4_validate_decision_projection(decision)?;
    let operation = match command.transition {
        M4DecisionLocalTransition::Read => "decision-read",
        M4DecisionLocalTransition::Dismiss => "decision-dismiss",
    };
    let fingerprint = m4_require_cas(
        operation,
        &decision.decision_projection_id,
        decision.revision,
        &command.decision_projection_id,
        command.expected_revision,
        &command.metadata,
        &[],
    )?;
    let mut next = decision.clone();
    match command.transition {
        M4DecisionLocalTransition::Read
            if decision.local_visibility_status == M4DecisionLocalVisibilityStatus::Unread =>
        {
            next.local_visibility_status = M4DecisionLocalVisibilityStatus::Read;
        }
        M4DecisionLocalTransition::Dismiss
            if matches!(
                decision.local_visibility_status,
                M4DecisionLocalVisibilityStatus::Unread | M4DecisionLocalVisibilityStatus::Read
            ) =>
        {
            next.local_visibility_status = M4DecisionLocalVisibilityStatus::Dismissed;
        }
        _ => return Err("m4_decision_local_transition_not_allowed".to_string()),
    }
    next.revision = next
        .revision
        .checked_add(1)
        .ok_or_else(|| "m4_revision_overflow".to_string())?;
    m4_validate_decision_projection(&next)?;
    Ok(M4StateTransitionResult {
        aggregate: next,
        previous_revision: decision.revision,
        idempotency_fingerprint: fingerprint,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4CreatePersonalActionCommand {
    pub(crate) explicit_user_command_id: String,
    pub(crate) title: String,
    pub(crate) due_at_utc: Option<String>,
    pub(crate) metadata: M4CoordinationCommandMetadata,
}

/// This sum type makes automatic paths explicit and rejects all of them at the
/// one creation gate. A source, OpenLoop, or model event therefore cannot
/// accidentally become a standalone personal Todo.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum M4PersonalActionCreationRequest {
    ExplicitUserStandaloneTodo(M4CreatePersonalActionCommand),
    SourceProjection { source_ref: M4SourceRecordRef },
    OpenLoopProjection { open_loop_id: String },
    ModelEvent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum M4PersonalActionTransition {
    Complete,
    Cancel,
    Reopen,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4PersonalActionTransitionCommand {
    pub(crate) personal_action_id: String,
    pub(crate) expected_revision: u64,
    pub(crate) transition: M4PersonalActionTransition,
    pub(crate) metadata: M4CoordinationCommandMetadata,
}

pub(crate) fn m4_create_personal_action(
    request: &M4PersonalActionCreationRequest,
) -> Result<M4CreateResult<M4PersonalAction>, String> {
    let M4PersonalActionCreationRequest::ExplicitUserStandaloneTodo(command) = request else {
        return Err("m4_personal_action_explicit_standalone_user_command_required".to_string());
    };
    m4_validate_opaque_reference(
        "personal_action_explicit_user_command_id",
        &command.explicit_user_command_id,
    )?;
    m4_validate_personal_action_title(&command.title)?;
    if let Some(due_at_utc) = command.due_at_utc.as_deref() {
        m4_validate_utc("personal_action_due_at_utc", due_at_utc)?;
    }
    m4_validate_coordination_metadata(&command.metadata)?;
    let personal_action_id = m4_personal_action_id(&command.explicit_user_command_id)?;
    let due_presence = if command.due_at_utc.is_some() {
        "PRESENT"
    } else {
        "ABSENT"
    };
    let due_at_utc = command.due_at_utc.as_deref().unwrap_or("ABSENT");
    let action = M4PersonalAction {
        personal_action_id: personal_action_id.clone(),
        explicit_user_command_ref: command.explicit_user_command_id.clone(),
        title: command.title.clone(),
        status: M4PersonalActionStatus::Open,
        due_at_utc: command.due_at_utc.clone(),
        revision: 1,
    };
    m4_validate_personal_action(&action)?;
    Ok(M4CreateResult {
        aggregate: action,
        idempotency_fingerprint: m4_command_fingerprint(
            "personal-action-create",
            &personal_action_id,
            0,
            &command.metadata,
            &[
                command.explicit_user_command_id.as_str(),
                command.title.as_str(),
                due_presence,
                due_at_utc,
            ],
        )?,
    })
}

pub(crate) fn m4_transition_personal_action(
    action: &M4PersonalAction,
    command: &M4PersonalActionTransitionCommand,
) -> Result<M4StateTransitionResult<M4PersonalAction>, String> {
    m4_validate_personal_action(action)?;
    let operation = match command.transition {
        M4PersonalActionTransition::Complete => "personal-action-complete",
        M4PersonalActionTransition::Cancel => "personal-action-cancel",
        M4PersonalActionTransition::Reopen => "personal-action-reopen",
    };
    let fingerprint = m4_require_cas(
        operation,
        &action.personal_action_id,
        action.revision,
        &command.personal_action_id,
        command.expected_revision,
        &command.metadata,
        &[],
    )?;
    let mut next = action.clone();
    match command.transition {
        M4PersonalActionTransition::Complete if action.status == M4PersonalActionStatus::Open => {
            next.status = M4PersonalActionStatus::Completed;
        }
        M4PersonalActionTransition::Cancel if action.status == M4PersonalActionStatus::Open => {
            next.status = M4PersonalActionStatus::Cancelled;
        }
        M4PersonalActionTransition::Reopen
            if matches!(
                action.status,
                M4PersonalActionStatus::Completed | M4PersonalActionStatus::Cancelled
            ) =>
        {
            next.status = M4PersonalActionStatus::Open;
        }
        _ => return Err("m4_personal_action_transition_not_allowed".to_string()),
    }
    next.revision = next
        .revision
        .checked_add(1)
        .ok_or_else(|| "m4_revision_overflow".to_string())?;
    m4_validate_personal_action(&next)?;
    Ok(M4StateTransitionResult {
        aggregate: next,
        previous_revision: action.revision,
        idempotency_fingerprint: fingerprint,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum M4SourceOwnerCommandIntent {
    RequestCompletion,
    RequestCancellation,
    RequestReopen,
}

impl M4SourceOwnerCommandIntent {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::RequestCompletion => "REQUEST_COMPLETION",
            Self::RequestCancellation => "REQUEST_CANCELLATION",
            Self::RequestReopen => "REQUEST_REOPEN",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4PrepareSourceOwnerWritebackCommand {
    pub(crate) source_ref: M4SourceRecordRef,
    pub(crate) expected_source_revision: u64,
    pub(crate) fresh_idempotency_key: String,
    pub(crate) explicit_intent: M4SourceOwnerCommandIntent,
    pub(crate) requested_at_utc: String,
}

/// The only M4C04 output for a source business action. It intentionally holds
/// typed refs and an allowlisted intent only; dispatch and owner receipts are a
/// repository/registered-adapter seam, not a domain-side effect.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4SourceOwnerWritebackIntent {
    pub(crate) source_ref: M4SourceRecordRef,
    pub(crate) expected_source_revision: u64,
    pub(crate) idempotency_key: String,
    pub(crate) explicit_intent: M4SourceOwnerCommandIntent,
    pub(crate) requested_at_utc: String,
    pub(crate) intent_fingerprint: String,
}

pub(crate) fn m4_source_owner_writeback_idempotency_key(
    request_nonce: &str,
) -> Result<String, String> {
    m4_validate_opaque_reference("source_owner_writeback_request_nonce", request_nonce)?;
    m4_internal_id(
        "writeback-idempotency:sha256:",
        "syn.m4.source-owner-writeback-idempotency/v1",
        &[request_nonce],
    )
}

pub(crate) fn m4_source_owner_writeback_fingerprint(
    source_ref: &M4SourceRecordRef,
    expected_source_revision: u64,
    idempotency_key: &str,
    explicit_intent: M4SourceOwnerCommandIntent,
) -> Result<String, String> {
    let source_record_fingerprint = m4_source_record_fingerprint(source_ref)?;
    let revision = expected_source_revision.to_string();
    m4_internal_id(
        "owner-writeback:",
        "syn.m4.source-owner-writeback-intent/v1",
        &[
            &source_record_fingerprint,
            &revision,
            idempotency_key,
            explicit_intent.as_str(),
        ],
    )
}

pub(crate) fn m4_prepare_source_owner_writeback(
    command: &M4PrepareSourceOwnerWritebackCommand,
    previously_used_idempotency_keys: &BTreeSet<String>,
) -> Result<M4SourceOwnerWritebackIntent, String> {
    m4_validate_source_record_ref(&command.source_ref)?;
    if command.expected_source_revision != command.source_ref.source_revision {
        return Err("m4_source_owner_writeback_expected_revision_mismatch".to_string());
    }
    m4_validate_utc(
        "source_owner_writeback_requested_at_utc",
        &command.requested_at_utc,
    )?;
    if !command
        .fresh_idempotency_key
        .starts_with("writeback-idempotency:sha256:")
        || !m4_is_opaque_reference(&command.fresh_idempotency_key)
    {
        return Err("m4_source_owner_writeback_idempotency_key_invalid".to_string());
    }
    if previously_used_idempotency_keys.contains(&command.fresh_idempotency_key) {
        return Err("m4_source_owner_writeback_idempotency_key_not_fresh".to_string());
    }
    let intent_fingerprint = m4_source_owner_writeback_fingerprint(
        &command.source_ref,
        command.expected_source_revision,
        &command.fresh_idempotency_key,
        command.explicit_intent,
    )?;
    Ok(M4SourceOwnerWritebackIntent {
        source_ref: command.source_ref.clone(),
        expected_source_revision: command.expected_source_revision,
        idempotency_key: command.fresh_idempotency_key.clone(),
        explicit_intent: command.explicit_intent,
        requested_at_utc: command.requested_at_utc.clone(),
        intent_fingerprint,
    })
}

pub(crate) fn m4_validate_source_owner_writeback_intent(
    intent: &M4SourceOwnerWritebackIntent,
) -> Result<(), String> {
    m4_validate_source_record_ref(&intent.source_ref)?;
    if intent.expected_source_revision != intent.source_ref.source_revision {
        return Err("m4_source_owner_writeback_expected_revision_mismatch".to_string());
    }
    if !intent
        .idempotency_key
        .starts_with("writeback-idempotency:sha256:")
        || !m4_is_opaque_reference(&intent.idempotency_key)
    {
        return Err("m4_source_owner_writeback_idempotency_key_invalid".to_string());
    }
    m4_validate_utc(
        "source_owner_writeback_requested_at_utc",
        &intent.requested_at_utc,
    )?;
    if intent.intent_fingerprint
        != m4_source_owner_writeback_fingerprint(
            &intent.source_ref,
            intent.expected_source_revision,
            &intent.idempotency_key,
            intent.explicit_intent,
        )?
    {
        return Err("m4_source_owner_writeback_fingerprint_mismatch".to_string());
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum M4SourceOwnerWritebackOutcome {
    Succeeded,
    Rejected,
    Failed,
}

impl M4SourceOwnerWritebackOutcome {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Succeeded => "SUCCEEDED",
            Self::Rejected => "REJECTED",
            Self::Failed => "FAILED",
        }
    }
}

/// A scrubbed owner receipt. It is intentionally unable to carry a callback,
/// executable request body, credential, or a new source status; applying an
/// owner result remains the next admitted source-event flow.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4SourceOwnerWritebackResult {
    pub(crate) source_ref: M4SourceRecordRef,
    pub(crate) expected_source_revision: u64,
    pub(crate) idempotency_key: String,
    pub(crate) intent_fingerprint: String,
    pub(crate) outcome: M4SourceOwnerWritebackOutcome,
    pub(crate) owner_receipt_ref: String,
    pub(crate) recorded_at_utc: String,
}

pub(crate) fn m4_validate_source_owner_writeback_result(
    intent: &M4SourceOwnerWritebackIntent,
    result: &M4SourceOwnerWritebackResult,
) -> Result<(), String> {
    m4_validate_source_owner_writeback_intent(intent)?;
    if result.source_ref != intent.source_ref
        || result.expected_source_revision != intent.expected_source_revision
        || result.idempotency_key != intent.idempotency_key
        || result.intent_fingerprint != intent.intent_fingerprint
    {
        return Err("m4_source_owner_writeback_result_intent_mismatch".to_string());
    }
    m4_validate_opaque_reference(
        "source_owner_writeback_owner_receipt_ref",
        &result.owner_receipt_ref,
    )?;
    m4_validate_utc(
        "source_owner_writeback_result_recorded_at_utc",
        &result.recorded_at_utc,
    )?;
    let _ = result.outcome.as_str();
    Ok(())
}

/// Hash every immutable field of an admitted source reference before it is
/// folded into a command fingerprint. This prevents two distinct source
/// snapshots at the same aggregate revision from becoming replay-equivalent.
pub(crate) fn m4_source_record_fingerprint(
    source_ref: &M4SourceRecordRef,
) -> Result<String, String> {
    m4_validate_source_record_ref(source_ref)?;
    let revision = source_ref.source_revision.to_string();
    let link_revision = source_ref.source_link.expected_source_revision.to_string();
    let external_commitment = m4_bool_component(source_ref.attention_signals.external_commitment);
    let time_sensitive = m4_bool_component(source_ref.attention_signals.time_sensitive);
    let requires_user_decision =
        m4_bool_component(source_ref.attention_signals.requires_user_decision);
    let source_blocked = m4_bool_component(source_ref.attention_signals.source_blocked);
    let attention_required = m4_bool_component(source_ref.attention_signals.attention_required);
    let material_change = m4_bool_component(source_ref.attention_signals.material_change);
    let due_presence = if source_ref.due_at_utc.is_some() {
        "PRESENT"
    } else {
        "ABSENT"
    };
    let due_at_utc = source_ref.due_at_utc.as_deref().unwrap_or("");
    m4_internal_id(
        "source-record:",
        "syn.m4.source-record-ref/v1",
        &[
            &source_ref.source_owner_ref,
            &source_ref.scope_ref,
            &source_ref.source_type,
            &source_ref.canonical_source_object_id,
            &revision,
            &source_ref.source_event_id,
            &source_ref.source_owner_watermark,
            &source_ref.occurred_at_utc,
            &source_ref.source_link.link_kind,
            &source_ref.source_link.source_owner_ref,
            &source_ref.source_link.object_type,
            &source_ref.source_link.canonical_source_object_id,
            &link_revision,
            &source_ref.source_link.opaque_route_ref,
            source_ref.source_status.as_str(),
            external_commitment,
            time_sensitive,
            requires_user_decision,
            source_blocked,
            attention_required,
            material_change,
            due_presence,
            due_at_utc,
            &source_ref.sensitivity,
            &source_ref.scrubbed_summary_ref,
            &source_ref.payload_hash,
        ],
    )
}

fn m4_bool_component(value: bool) -> &'static str {
    if value {
        "1"
    } else {
        "0"
    }
}

pub(crate) fn m4_coordination_command_fingerprint(
    operation: &str,
    object_id: &str,
    expected_revision: u64,
    idempotency_key: &str,
) -> Result<String, String> {
    m4_coordination_command_fingerprint_with_fields(
        operation,
        object_id,
        expected_revision,
        idempotency_key,
        &[],
    )
}

pub(crate) fn m4_coordination_command_fingerprint_with_fields(
    operation: &str,
    object_id: &str,
    expected_revision: u64,
    idempotency_key: &str,
    immutable_command_fields: &[&str],
) -> Result<String, String> {
    m4_validate_identifier("coordination_operation", operation)?;
    m4_validate_typed_reference("coordination_object_id", object_id)?;
    m4_validate_opaque_reference("coordination_idempotency_key", idempotency_key)?;
    let revision = expected_revision.to_string();
    for field in immutable_command_fields {
        m4_validate_reference_text("coordination_immutable_command_field", field)?;
    }
    let mut components = vec![operation, object_id, revision.as_str(), idempotency_key];
    components.extend_from_slice(immutable_command_fields);
    m4_internal_id(
        "coordination-command:",
        "syn.m4.coordination-command/v1",
        &components,
    )
}

fn m4_validate_coordination_metadata(
    metadata: &M4CoordinationCommandMetadata,
) -> Result<(), String> {
    m4_validate_opaque_reference("coordination_idempotency_key", &metadata.idempotency_key)?;
    m4_validate_utc("coordination_occurred_at_utc", &metadata.occurred_at_utc)?;
    Ok(())
}

fn m4_command_fingerprint(
    operation: &str,
    object_id: &str,
    expected_revision: u64,
    metadata: &M4CoordinationCommandMetadata,
    immutable_command_fields: &[&str],
) -> Result<String, String> {
    m4_validate_coordination_metadata(metadata)?;
    m4_coordination_command_fingerprint_with_fields(
        operation,
        object_id,
        expected_revision,
        &metadata.idempotency_key,
        immutable_command_fields,
    )
}

fn m4_require_cas(
    operation: &str,
    actual_object_id: &str,
    actual_revision: u64,
    command_object_id: &str,
    expected_revision: u64,
    metadata: &M4CoordinationCommandMetadata,
    immutable_command_fields: &[&str],
) -> Result<String, String> {
    if actual_object_id != command_object_id {
        return Err("m4_coordination_target_mismatch".to_string());
    }
    if actual_revision != expected_revision {
        return Err("m4_expected_revision_conflict".to_string());
    }
    m4_command_fingerprint(
        operation,
        actual_object_id,
        expected_revision,
        metadata,
        immutable_command_fields,
    )
}

fn m4_require_same_source_identity(
    current: &M4SourceRecordRef,
    replacement: &M4SourceRecordRef,
) -> Result<(), String> {
    if m4_source_record_identity_key(current)? != m4_source_record_identity_key(replacement)? {
        return Err("m4_source_record_identity_mismatch".to_string());
    }
    Ok(())
}

fn m4_source_ref_matches_automatic_open_loop_policy(source_ref: &M4SourceRecordRef) -> bool {
    !source_ref.source_status.is_terminal()
        && (source_ref.attention_signals.external_commitment
            || source_ref.attention_signals.time_sensitive
            || source_ref.attention_signals.requires_user_decision
            || source_ref.attention_signals.source_blocked
            || source_ref.attention_signals.attention_required)
}

fn m4_open_loop_with_new_source(
    open_loop: &M4OpenLoop,
    source_ref: &M4SourceRecordRef,
) -> Result<M4OpenLoop, String> {
    m4_require_same_source_identity(&open_loop.source_ref, source_ref)?;
    let mut next = open_loop.clone();
    next.source_ref = source_ref.clone();
    next.last_source_revision = source_ref.source_revision;
    next.priority_reason = m4_priority_reason(&source_ref.attention_signals)?;
    next.owner_ref = source_ref.source_owner_ref.clone();
    next.due_at_utc = source_ref.due_at_utc.clone();
    Ok(next)
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

#[cfg(test)]
mod m4c04_domain_tests {
    use super::*;

    fn opaque(namespace: &str, material: &str) -> String {
        m4_internal_id(
            &format!("{namespace}:sha256:"),
            "syn.m4c04.domain-test/v1",
            &[material],
        )
        .expect("make M4C04 opaque reference")
    }

    fn metadata(label: &str, occurred_at_utc: &str) -> M4CoordinationCommandMetadata {
        M4CoordinationCommandMetadata {
            idempotency_key: opaque("idempotency", label),
            occurred_at_utc: occurred_at_utc.to_string(),
        }
    }

    fn source(
        revision: u64,
        status: M4SourceStatus,
        attention_signals: M4AttentionSignals,
    ) -> M4SourceRecordRef {
        let revision_text = revision.to_string();
        M4SourceRecordRef {
            source_owner_ref: "workflow_owner".to_string(),
            scope_ref: M4_PRIMARY_SECRETARY_SCOPE_ID.to_string(),
            source_type: M4_WORKFLOW_ATTENTION_SOURCE_TYPE.to_string(),
            canonical_source_object_id: "work-item-42".to_string(),
            source_revision: revision,
            source_event_id: opaque("source-event", &format!("event-{revision}")),
            source_owner_watermark: opaque("watermark", &format!("watermark-{revision}")),
            occurred_at_utc: format!("2026-08-10T12:{:02}:00Z", revision % 60),
            source_link: M4SourceLinkInput {
                link_kind: "INTERNAL_ROUTE".to_string(),
                source_owner_ref: "workflow_owner".to_string(),
                object_type: M4_WORKFLOW_ATTENTION_OBJECT_TYPE.to_string(),
                canonical_source_object_id: "work-item-42".to_string(),
                expected_source_revision: revision,
                opaque_route_ref: opaque("route", "work-item-42"),
            },
            source_status: status,
            attention_signals,
            due_at_utc: Some("2026-08-11T10:00:00Z".to_string()),
            sensitivity: M4_SCRUBBED_SENSITIVITY.to_string(),
            scrubbed_summary_ref: opaque("summary", &format!("summary-{revision}")),
            payload_hash: m4_internal_id("", "syn.m4c04.domain-test-payload/v1", &[&revision_text])
                .expect("make M4C04 payload hash"),
        }
    }

    fn attention_source(revision: u64) -> M4SourceRecordRef {
        source(
            revision,
            M4SourceStatus::Open,
            M4AttentionSignals {
                attention_required: true,
                material_change: true,
                ..Default::default()
            },
        )
    }

    fn inbox(
        source_ref: M4SourceRecordRef,
        status: M4InboxItemStatus,
        revision: u64,
    ) -> M4InboxItem {
        let identity = m4_source_record_identity_key(&source_ref).expect("derive source identity");
        M4InboxItem {
            inbox_item_id: m4_inbox_item_id(&identity).expect("derive inbox ID"),
            source_ref: source_ref.clone(),
            dedupe_key: identity,
            status,
            priority_reason: m4_priority_reason(&source_ref.attention_signals)
                .expect("derive inbox priority"),
            received_at_utc: "2026-08-10T11:00:00Z".to_string(),
            last_source_change_at_utc: source_ref.occurred_at_utc.clone(),
            scrubbed_summary_ref: source_ref.scrubbed_summary_ref.clone(),
            sensitivity: source_ref.sensitivity.clone(),
            revision,
        }
    }

    fn open_loop(
        source_ref: M4SourceRecordRef,
        status: M4OpenLoopStatus,
        revision: u64,
    ) -> M4OpenLoop {
        let identity = m4_source_record_identity_key(&source_ref).expect("derive source identity");
        M4OpenLoop {
            open_loop_id: m4_open_loop_id(&identity).expect("derive OpenLoop ID"),
            source_ref: source_ref.clone(),
            status,
            why_open_code: "AUTOMATIC_ATTENTION_POLICY".to_string(),
            priority_reason: m4_priority_reason(&source_ref.attention_signals)
                .expect("derive OpenLoop priority"),
            owner_ref: source_ref.source_owner_ref.clone(),
            due_at_utc: source_ref.due_at_utc.clone(),
            snoozed_until_utc: (status == M4OpenLoopStatus::Snoozed)
                .then(|| "2026-08-10T14:00:00Z".to_string()),
            last_source_revision: source_ref.source_revision,
            projection_policy_ref: M4_ATTENTION_POLICY_REF.to_string(),
            closure_reason_code: (status == M4OpenLoopStatus::Closed)
                .then(|| M4_COORDINATION_CLOSE_REASON.to_string()),
            revision,
        }
    }

    #[test]
    fn m4c04_inbox_lifecycle_applies_cas_and_source_only_transitions() {
        let initial = inbox(attention_source(7), M4InboxItemStatus::New, 4);
        let read_command = M4InboxReadCommand {
            inbox_item_id: initial.inbox_item_id.clone(),
            expected_revision: 4,
            metadata: metadata("inbox-read", "2026-08-10T12:10:00Z"),
        };
        let read = m4_mark_inbox_item_read(&initial, &read_command).expect("mark inbox read");
        assert_eq!(read.aggregate.status, M4InboxItemStatus::Read);
        assert_eq!(read.aggregate.revision, 5);
        assert_eq!(
            m4_mark_inbox_item_read(&read.aggregate, &read_command),
            Err("m4_expected_revision_conflict".to_string())
        );

        let dismiss_command = M4InboxDismissCommand {
            inbox_item_id: read.aggregate.inbox_item_id.clone(),
            expected_revision: 5,
            metadata: metadata("inbox-dismiss", "2026-08-10T12:11:00Z"),
        };
        let dismissed =
            m4_dismiss_inbox_item(&read.aggregate, &dismiss_command).expect("dismiss inbox");
        assert_eq!(dismissed.aggregate.status, M4InboxItemStatus::Dismissed);
        assert_eq!(
            m4_dismiss_inbox_item(
                &dismissed.aggregate,
                &M4InboxDismissCommand {
                    expected_revision: 6,
                    metadata: metadata("inbox-dismiss-again", "2026-08-10T12:12:00Z"),
                    ..dismiss_command.clone()
                }
            ),
            Err("m4_inbox_dismiss_transition_not_allowed".to_string())
        );
        assert_eq!(
            m4_mark_inbox_item_read(
                &dismissed.aggregate,
                &M4InboxReadCommand {
                    inbox_item_id: dismissed.aggregate.inbox_item_id.clone(),
                    expected_revision: 6,
                    metadata: metadata("inbox-read-dismissed", "2026-08-10T12:12:00Z"),
                },
            ),
            Err("m4_inbox_read_transition_not_allowed".to_string())
        );

        let refreshed_source = attention_source(8);
        let refreshed = m4_refresh_inbox_item_from_new_source(
            &dismissed.aggregate,
            &M4RefreshInboxFromSourceCommand {
                inbox_item_id: dismissed.aggregate.inbox_item_id.clone(),
                expected_revision: 6,
                source_ref: refreshed_source.clone(),
                metadata: metadata("inbox-refresh", "2026-08-10T12:13:00Z"),
            },
        )
        .expect("new source revision returns Inbox to NEW");
        assert_eq!(refreshed.aggregate.status, M4InboxItemStatus::New);
        assert_eq!(refreshed.aggregate.revision, 7);
        assert_eq!(refreshed.aggregate.inbox_item_id, initial.inbox_item_id);
        assert_eq!(refreshed.aggregate.dedupe_key, initial.dedupe_key);

        let expired_source = source(
            9,
            M4SourceStatus::Expired,
            M4AttentionSignals {
                material_change: true,
                ..Default::default()
            },
        );
        let expired = m4_expire_inbox_item_from_source(
            &refreshed.aggregate,
            &M4ExpireInboxItemCommand {
                inbox_item_id: refreshed.aggregate.inbox_item_id.clone(),
                expected_revision: 7,
                source_ref: expired_source,
                metadata: metadata("inbox-expire", "2026-08-10T12:14:00Z"),
            },
        )
        .expect("owner expiry expires Inbox");
        assert_eq!(expired.aggregate.status, M4InboxItemStatus::Expired);

        let quarantined = m4_quarantine_inbox_item(
            &expired.aggregate,
            &M4QuarantineInboxItemCommand {
                inbox_item_id: expired.aggregate.inbox_item_id.clone(),
                expected_revision: 8,
                quarantine_reason_code: "INVALID_SOURCE_BINDING".to_string(),
                metadata: metadata("inbox-quarantine", "2026-08-10T12:15:00Z"),
            },
        )
        .expect("invalid binding quarantines active or terminal Inbox evidence");
        assert_eq!(quarantined.aggregate.status, M4InboxItemStatus::Quarantined);
        assert_eq!(quarantined.aggregate.revision, 9);
    }

    #[test]
    fn m4c04_open_loop_user_lifecycle_keeps_owner_source_unchanged() {
        let initial = open_loop(attention_source(7), M4OpenLoopStatus::Open, 10);
        let acknowledged = m4_acknowledge_open_loop(
            &initial,
            &M4AcknowledgeOpenLoopCommand {
                open_loop_id: initial.open_loop_id.clone(),
                expected_revision: 10,
                metadata: metadata("loop-ack", "2026-08-10T12:10:00Z"),
            },
        )
        .expect("acknowledge OpenLoop");
        assert_eq!(
            acknowledged.aggregate.status,
            M4OpenLoopStatus::Acknowledged
        );
        assert_eq!(acknowledged.aggregate.source_ref, initial.source_ref);

        let snooze_command = M4SnoozeOpenLoopCommand {
            open_loop_id: acknowledged.aggregate.open_loop_id.clone(),
            expected_revision: 11,
            snoozed_until_utc: "2026-08-10T12:30:00Z".to_string(),
            metadata: metadata("loop-snooze", "2026-08-10T12:11:00Z"),
        };
        let snoozed = m4_snooze_open_loop(&acknowledged.aggregate, &snooze_command)
            .expect("snooze acknowledged loop");
        assert_eq!(snoozed.aggregate.status, M4OpenLoopStatus::Snoozed);
        assert_eq!(snoozed.aggregate.source_ref, initial.source_ref);
        assert_eq!(
            m4_select_open_loop_for_carry_over(
                &snoozed.aggregate,
                &M4CarryOverOpenLoopCommand {
                    open_loop_id: snoozed.aggregate.open_loop_id.clone(),
                    expected_revision: 12,
                    metadata: metadata("loop-carry-snoozed", "2026-08-10T12:12:00Z"),
                },
            ),
            Err("m4_open_loop_carry_over_not_eligible".to_string())
        );
        assert_eq!(
            m4_reopen_snoozed_open_loop_on_clock(
                &snoozed.aggregate,
                &M4OpenLoopClockCommand {
                    open_loop_id: snoozed.aggregate.open_loop_id.clone(),
                    expected_revision: 12,
                    metadata: metadata("loop-clock-early", "2026-08-10T12:29:00Z"),
                },
            ),
            Err("m4_open_loop_snooze_not_due".to_string())
        );

        let closed_from_snooze = m4_close_open_loop(
            &snoozed.aggregate,
            &M4CloseOpenLoopCommand {
                open_loop_id: snoozed.aggregate.open_loop_id.clone(),
                expected_revision: 12,
                metadata: metadata("loop-close-snoozed", "2026-08-10T12:20:00Z"),
            },
        )
        .expect("a snoozed loop may be closed locally");
        assert_eq!(
            closed_from_snooze.aggregate.status,
            M4OpenLoopStatus::Closed
        );
        assert_eq!(closed_from_snooze.aggregate.source_ref, initial.source_ref);

        let reopened_on_clock = m4_reopen_snoozed_open_loop_on_clock(
            &snoozed.aggregate,
            &M4OpenLoopClockCommand {
                open_loop_id: snoozed.aggregate.open_loop_id.clone(),
                expected_revision: 12,
                metadata: metadata("loop-clock", "2026-08-10T12:30:00Z"),
            },
        )
        .expect("clock reopens snoozed loop");
        assert_eq!(reopened_on_clock.aggregate.status, M4OpenLoopStatus::Open);
        let carried = m4_select_open_loop_for_carry_over(
            &reopened_on_clock.aggregate,
            &M4CarryOverOpenLoopCommand {
                open_loop_id: reopened_on_clock.aggregate.open_loop_id.clone(),
                expected_revision: 13,
                metadata: metadata("loop-carry", "2026-08-10T12:31:00Z"),
            },
        )
        .expect("unclosed non-snoozed loop carries without mutation");
        assert_eq!(carried.open_loop_id, initial.open_loop_id);
        assert_eq!(carried.retained_revision, 13);

        let closed = m4_close_open_loop(
            &reopened_on_clock.aggregate,
            &M4CloseOpenLoopCommand {
                open_loop_id: reopened_on_clock.aggregate.open_loop_id.clone(),
                expected_revision: 13,
                metadata: metadata("loop-close", "2026-08-10T12:32:00Z"),
            },
        )
        .expect("close OpenLoop locally");
        assert_eq!(closed.aggregate.source_ref, initial.source_ref);
        assert_eq!(
            closed.aggregate.closure_reason_code.as_deref(),
            Some(M4_COORDINATION_CLOSE_REASON)
        );
        assert_eq!(
            m4_snooze_open_loop(
                &closed.aggregate,
                &M4SnoozeOpenLoopCommand {
                    open_loop_id: closed.aggregate.open_loop_id.clone(),
                    expected_revision: 14,
                    snoozed_until_utc: "2026-08-10T13:00:00Z".to_string(),
                    metadata: metadata("loop-snooze-closed", "2026-08-10T12:33:00Z"),
                },
            ),
            Err("m4_open_loop_snooze_transition_not_allowed".to_string())
        );
        let reopened = m4_reopen_open_loop(
            &closed.aggregate,
            &M4ReopenOpenLoopCommand {
                open_loop_id: closed.aggregate.open_loop_id.clone(),
                expected_revision: 14,
                metadata: metadata("loop-reopen", "2026-08-10T12:34:00Z"),
            },
        )
        .expect("explicit reopen");
        assert_eq!(reopened.aggregate.status, M4OpenLoopStatus::Open);
        assert_eq!(reopened.aggregate.source_ref, initial.source_ref);
        let dismissed = m4_dismiss_open_loop(
            &reopened.aggregate,
            &M4DismissOpenLoopCommand {
                open_loop_id: reopened.aggregate.open_loop_id.clone(),
                expected_revision: 15,
                metadata: metadata("loop-dismiss", "2026-08-10T12:35:00Z"),
            },
        )
        .expect("dismiss OpenLoop locally");
        assert_eq!(dismissed.aggregate.status, M4OpenLoopStatus::Dismissed);
        let reopened_from_dismissal = m4_reopen_open_loop(
            &dismissed.aggregate,
            &M4ReopenOpenLoopCommand {
                open_loop_id: dismissed.aggregate.open_loop_id.clone(),
                expected_revision: 16,
                metadata: metadata("loop-reopen-dismissed", "2026-08-10T12:36:00Z"),
            },
        )
        .expect("explicit reopen from dismissal");
        assert_eq!(
            reopened_from_dismissal.aggregate.status,
            M4OpenLoopStatus::Open
        );
    }

    #[test]
    fn m4c04_open_loop_source_transitions_project_existing_owner_facts_only() {
        let initial = open_loop(attention_source(7), M4OpenLoopStatus::Open, 5);
        let terminal_source = source(
            8,
            M4SourceStatus::Completed,
            M4AttentionSignals {
                material_change: true,
                ..Default::default()
            },
        );
        let closed = m4_close_open_loop_from_terminal_source(
            &initial,
            &M4CloseOpenLoopFromTerminalSourceCommand {
                open_loop_id: initial.open_loop_id.clone(),
                expected_revision: 5,
                source_ref: terminal_source.clone(),
                metadata: metadata("loop-source-close", "2026-08-10T12:20:00Z"),
            },
        )
        .expect("terminal source closes local OpenLoop");
        assert_eq!(closed.aggregate.open_loop_id, initial.open_loop_id);
        assert_eq!(closed.aggregate.source_ref, terminal_source);
        assert_eq!(
            closed.aggregate.source_ref.source_status,
            M4SourceStatus::Completed
        );
        assert_eq!(
            closed.aggregate.closure_reason_code.as_deref(),
            Some("SOURCE_COMPLETED")
        );

        let reopened_source = attention_source(9);
        let reopened = m4_reopen_open_loop_from_new_source(
            &closed.aggregate,
            &M4ReopenOpenLoopFromSourceCommand {
                open_loop_id: closed.aggregate.open_loop_id.clone(),
                expected_revision: 6,
                source_ref: reopened_source.clone(),
                metadata: metadata("loop-source-reopen", "2026-08-10T12:21:00Z"),
            },
        )
        .expect("new attention source reopens same OpenLoop");
        assert_eq!(reopened.aggregate.open_loop_id, initial.open_loop_id);
        assert_eq!(reopened.aggregate.source_ref, reopened_source);
        assert_eq!(reopened.aggregate.status, M4OpenLoopStatus::Open);

        let non_attention_source = source(10, M4SourceStatus::Informational, Default::default());
        assert_eq!(
            m4_reopen_open_loop_from_new_source(
                &closed.aggregate,
                &M4ReopenOpenLoopFromSourceCommand {
                    open_loop_id: closed.aggregate.open_loop_id.clone(),
                    expected_revision: 6,
                    source_ref: non_attention_source,
                    metadata: metadata("loop-source-no-attention", "2026-08-10T12:22:00Z"),
                },
            ),
            Err("m4_open_loop_source_attention_policy_not_matched".to_string())
        );
    }

    #[test]
    fn m4c04_notification_state_machine_is_cas_bound() {
        let source_ref = attention_source(7);
        let source_identity = m4_source_record_identity_key(&source_ref).expect("source identity");
        let subject_ref = m4_open_loop_id(&source_identity).expect("OpenLoop subject ID");
        for invalid_code in [
            String::new(),
            "attention_required".to_string(),
            "A".repeat(97),
        ] {
            assert_eq!(
                m4_create_notification(&M4CreateNotificationCommand {
                    source_ref: source_ref.clone(),
                    subject_ref: subject_ref.clone(),
                    notification_purpose_code: invalid_code,
                    delivery_channel: M4_IN_APP_DELIVERY_CHANNEL.to_string(),
                    metadata: metadata("notification-create-invalid-code", "2026-08-10T11:59:00Z",),
                }),
                Err("m4_coordination_code_invalid:notification_purpose_code".to_string())
            );
        }
        let created = m4_create_notification(&M4CreateNotificationCommand {
            source_ref: source_ref.clone(),
            subject_ref: subject_ref.clone(),
            notification_purpose_code: "ATTENTION_REQUIRED".to_string(),
            delivery_channel: M4_IN_APP_DELIVERY_CHANNEL.to_string(),
            metadata: metadata("notification-create", "2026-08-10T12:00:00Z"),
        })
        .expect("create in-app notification");
        assert_eq!(created.aggregate.status, M4NotificationStatus::Pending);
        assert_eq!(
            m4_transition_notification(
                &created.aggregate,
                &M4NotificationTransitionCommand {
                    notification_id: created.aggregate.notification_id.clone(),
                    expected_revision: 1,
                    transition: M4NotificationTransition::Read,
                    metadata: metadata("notification-read-pending", "2026-08-10T12:01:00Z"),
                },
            ),
            Err("m4_notification_transition_not_allowed".to_string())
        );
        let delivered = m4_transition_notification(
            &created.aggregate,
            &M4NotificationTransitionCommand {
                notification_id: created.aggregate.notification_id.clone(),
                expected_revision: 1,
                transition: M4NotificationTransition::Deliver,
                metadata: metadata("notification-deliver", "2026-08-10T12:01:00Z"),
            },
        )
        .expect("deliver notification");
        let read = m4_transition_notification(
            &delivered.aggregate,
            &M4NotificationTransitionCommand {
                notification_id: delivered.aggregate.notification_id.clone(),
                expected_revision: 2,
                transition: M4NotificationTransition::Read,
                metadata: metadata("notification-read", "2026-08-10T12:02:00Z"),
            },
        )
        .expect("read delivered notification");
        let dismissed = m4_transition_notification(
            &read.aggregate,
            &M4NotificationTransitionCommand {
                notification_id: read.aggregate.notification_id.clone(),
                expected_revision: 3,
                transition: M4NotificationTransition::Dismiss,
                metadata: metadata("notification-dismiss", "2026-08-10T12:03:00Z"),
            },
        )
        .expect("dismiss read notification");
        assert_eq!(dismissed.aggregate.status, M4NotificationStatus::Dismissed);
        let directly_dismissed = m4_transition_notification(
            &created.aggregate,
            &M4NotificationTransitionCommand {
                notification_id: created.aggregate.notification_id.clone(),
                expected_revision: 1,
                transition: M4NotificationTransition::Dismiss,
                metadata: metadata("notification-dismiss-pending", "2026-08-10T12:01:00Z"),
            },
        )
        .expect("pending notification may be dismissed");
        assert_eq!(
            directly_dismissed.aggregate.status,
            M4NotificationStatus::Dismissed
        );
        assert_eq!(
            m4_transition_notification(
                &dismissed.aggregate,
                &M4NotificationTransitionCommand {
                    notification_id: dismissed.aggregate.notification_id.clone(),
                    expected_revision: 3,
                    transition: M4NotificationTransition::Dismiss,
                    metadata: metadata("notification-stale", "2026-08-10T12:04:00Z"),
                },
            ),
            Err("m4_expected_revision_conflict".to_string())
        );
    }

    #[test]
    fn m4c04_reminder_state_machine_covers_fire_snooze_dismiss_and_cancel() {
        let owner_ref =
            m4_personal_action_id(&opaque("user-command", "owner")).expect("personal owner ID");
        let create = M4CreateReminderCommand {
            owner_ref: owner_ref.clone(),
            explicit_schedule_command_id: opaque("schedule-command", "one"),
            scheduled_for_utc: "2026-08-10T12:10:00Z".to_string(),
            iana_timezone: "Asia/Shanghai".to_string(),
            metadata: metadata("reminder-create", "2026-08-10T12:00:00Z"),
        };
        let scheduled = m4_create_reminder(&create).expect("create reminder");
        assert_eq!(
            m4_transition_reminder(
                &scheduled.aggregate,
                &M4ReminderTransitionCommand {
                    reminder_id: scheduled.aggregate.reminder_id.clone(),
                    expected_revision: 1,
                    transition: M4ReminderTransition::Fire,
                    metadata: metadata("reminder-fire-early", "2026-08-10T12:09:00Z"),
                },
            ),
            Err("m4_reminder_fire_not_due".to_string())
        );
        let fired = m4_transition_reminder(
            &scheduled.aggregate,
            &M4ReminderTransitionCommand {
                reminder_id: scheduled.aggregate.reminder_id.clone(),
                expected_revision: 1,
                transition: M4ReminderTransition::Fire,
                metadata: metadata("reminder-fire", "2026-08-10T12:10:00Z"),
            },
        )
        .expect("fire scheduled reminder");
        let snoozed = m4_transition_reminder(
            &fired.aggregate,
            &M4ReminderTransitionCommand {
                reminder_id: fired.aggregate.reminder_id.clone(),
                expected_revision: 2,
                transition: M4ReminderTransition::Snooze {
                    snoozed_until_utc: "2026-08-10T12:20:00Z".to_string(),
                },
                metadata: metadata("reminder-snooze", "2026-08-10T12:11:00Z"),
            },
        )
        .expect("snooze fired reminder");
        assert_eq!(
            m4_transition_reminder(
                &snoozed.aggregate,
                &M4ReminderTransitionCommand {
                    reminder_id: snoozed.aggregate.reminder_id.clone(),
                    expected_revision: 3,
                    transition: M4ReminderTransition::Fire,
                    metadata: metadata("reminder-fire-snoozed-early", "2026-08-10T12:19:00Z"),
                },
            ),
            Err("m4_reminder_snooze_not_due".to_string())
        );
        let fired_again = m4_transition_reminder(
            &snoozed.aggregate,
            &M4ReminderTransitionCommand {
                reminder_id: snoozed.aggregate.reminder_id.clone(),
                expected_revision: 3,
                transition: M4ReminderTransition::Fire,
                metadata: metadata("reminder-fire-snoozed", "2026-08-10T12:20:00Z"),
            },
        )
        .expect("clock fires snoozed reminder");
        let dismissed = m4_transition_reminder(
            &fired_again.aggregate,
            &M4ReminderTransitionCommand {
                reminder_id: fired_again.aggregate.reminder_id.clone(),
                expected_revision: 4,
                transition: M4ReminderTransition::Dismiss,
                metadata: metadata("reminder-dismiss", "2026-08-10T12:21:00Z"),
            },
        )
        .expect("dismiss reminder");
        assert_eq!(dismissed.aggregate.status, M4ReminderStatus::Dismissed);
        assert_eq!(
            m4_transition_reminder(
                &dismissed.aggregate,
                &M4ReminderTransitionCommand {
                    reminder_id: dismissed.aggregate.reminder_id.clone(),
                    expected_revision: 5,
                    transition: M4ReminderTransition::Cancel,
                    metadata: metadata("reminder-cancel-dismissed", "2026-08-10T12:22:00Z"),
                },
            ),
            Err("m4_reminder_transition_not_allowed".to_string())
        );

        let scheduled_cancel = m4_create_reminder(&M4CreateReminderCommand {
            owner_ref,
            explicit_schedule_command_id: opaque("schedule-command", "two"),
            scheduled_for_utc: "2026-08-10T13:00:00Z".to_string(),
            iana_timezone: "Asia/Shanghai".to_string(),
            metadata: metadata("reminder-create-cancel", "2026-08-10T12:00:00Z"),
        })
        .expect("create cancellable reminder");
        let cancelled = m4_transition_reminder(
            &scheduled_cancel.aggregate,
            &M4ReminderTransitionCommand {
                reminder_id: scheduled_cancel.aggregate.reminder_id.clone(),
                expected_revision: 1,
                transition: M4ReminderTransition::Cancel,
                metadata: metadata("reminder-cancel", "2026-08-10T12:01:00Z"),
            },
        )
        .expect("cancel scheduled reminder");
        assert_eq!(cancelled.aggregate.status, M4ReminderStatus::Cancelled);

        let scheduled_snooze = m4_create_reminder(&M4CreateReminderCommand {
            owner_ref: m4_personal_action_id(&opaque("user-command", "snoozed-owner"))
                .expect("snoozed owner ID"),
            explicit_schedule_command_id: opaque("schedule-command", "three"),
            scheduled_for_utc: "2026-08-10T14:00:00Z".to_string(),
            iana_timezone: "Asia/Shanghai".to_string(),
            metadata: metadata("reminder-create-snoozed", "2026-08-10T12:00:00Z"),
        })
        .expect("create scheduled reminder for early snooze");
        let snoozed_then_cancelled = m4_transition_reminder(
            &scheduled_snooze.aggregate,
            &M4ReminderTransitionCommand {
                reminder_id: scheduled_snooze.aggregate.reminder_id.clone(),
                expected_revision: 1,
                transition: M4ReminderTransition::Snooze {
                    snoozed_until_utc: "2026-08-10T12:30:00Z".to_string(),
                },
                metadata: metadata("reminder-snooze-scheduled", "2026-08-10T12:01:00Z"),
            },
        )
        .expect("scheduled reminder may be snoozed");
        let cancelled_snooze = m4_transition_reminder(
            &snoozed_then_cancelled.aggregate,
            &M4ReminderTransitionCommand {
                reminder_id: snoozed_then_cancelled.aggregate.reminder_id.clone(),
                expected_revision: 2,
                transition: M4ReminderTransition::Cancel,
                metadata: metadata("reminder-cancel-snoozed", "2026-08-10T12:02:00Z"),
            },
        )
        .expect("snoozed reminder may be cancelled");
        assert_eq!(
            cancelled_snooze.aggregate.status,
            M4ReminderStatus::Cancelled
        );
    }

    #[test]
    fn m4c04_personal_action_creation_is_explicit_and_title_matches_schema() {
        let explicit_command = M4CreatePersonalActionCommand {
            explicit_user_command_id: opaque("user-command", "standalone-todo"),
            title: "跟进 alice@example.com 的预算".to_string(),
            due_at_utc: Some("2026-08-11T09:00:00Z".to_string()),
            metadata: metadata("personal-action-create", "2026-08-10T12:00:00Z"),
        };
        let created = m4_create_personal_action(
            &M4PersonalActionCreationRequest::ExplicitUserStandaloneTodo(explicit_command.clone()),
        )
        .expect("explicit standalone personal Todo creates action");
        assert_eq!(created.aggregate.status, M4PersonalActionStatus::Open);
        assert_eq!(
            m4_create_personal_action(&M4PersonalActionCreationRequest::SourceProjection {
                source_ref: attention_source(7),
            }),
            Err("m4_personal_action_explicit_standalone_user_command_required".to_string())
        );
        assert_eq!(
            m4_create_personal_action(&M4PersonalActionCreationRequest::OpenLoopProjection {
                open_loop_id: open_loop(attention_source(7), M4OpenLoopStatus::Open, 1)
                    .open_loop_id,
            }),
            Err("m4_personal_action_explicit_standalone_user_command_required".to_string())
        );
        assert_eq!(
            m4_create_personal_action(&M4PersonalActionCreationRequest::ModelEvent),
            Err("m4_personal_action_explicit_standalone_user_command_required".to_string())
        );

        let completed = m4_transition_personal_action(
            &created.aggregate,
            &M4PersonalActionTransitionCommand {
                personal_action_id: created.aggregate.personal_action_id.clone(),
                expected_revision: 1,
                transition: M4PersonalActionTransition::Complete,
                metadata: metadata("personal-action-complete", "2026-08-10T12:01:00Z"),
            },
        )
        .expect("complete explicit action");
        let reopened = m4_transition_personal_action(
            &completed.aggregate,
            &M4PersonalActionTransitionCommand {
                personal_action_id: completed.aggregate.personal_action_id.clone(),
                expected_revision: 2,
                transition: M4PersonalActionTransition::Reopen,
                metadata: metadata("personal-action-reopen", "2026-08-10T12:02:00Z"),
            },
        )
        .expect("reopen completed action");
        let cancelled = m4_transition_personal_action(
            &reopened.aggregate,
            &M4PersonalActionTransitionCommand {
                personal_action_id: reopened.aggregate.personal_action_id.clone(),
                expected_revision: 3,
                transition: M4PersonalActionTransition::Cancel,
                metadata: metadata("personal-action-cancel", "2026-08-10T12:03:00Z"),
            },
        )
        .expect("cancel re-opened action");
        assert_eq!(
            cancelled.aggregate.status,
            M4PersonalActionStatus::Cancelled
        );
        let reopened_from_cancel = m4_transition_personal_action(
            &cancelled.aggregate,
            &M4PersonalActionTransitionCommand {
                personal_action_id: cancelled.aggregate.personal_action_id.clone(),
                expected_revision: 4,
                transition: M4PersonalActionTransition::Reopen,
                metadata: metadata("personal-action-reopen-cancel", "2026-08-10T12:04:00Z"),
            },
        )
        .expect("reopen cancelled action");
        assert_eq!(
            reopened_from_cancel.aggregate.status,
            M4PersonalActionStatus::Open
        );

        for title in [
            "",
            " title",
            "title ",
            "line\nnext",
            "title\twith-tab",
            "C:\\private",
            "/absolute/path",
            "https://example.invalid/todo",
            "HTTP://example.invalid/todo",
        ] {
            assert_eq!(
                m4_validate_personal_action_title(title),
                Err("m4_personal_action_title_invalid".to_string()),
                "title fixture should fail local admission: {title:?}"
            );
        }
        assert!(m4_validate_personal_action_title(&"x".repeat(160)).is_ok());
        assert_eq!(
            m4_validate_personal_action_title(&"x".repeat(161)),
            Err("m4_personal_action_title_invalid".to_string())
        );
    }

    #[test]
    fn m4c04_owner_writeback_is_typed_fresh_and_receipt_bound() {
        let source_ref = attention_source(7);
        let idempotency_key =
            m4_source_owner_writeback_idempotency_key(&opaque("writeback-nonce", "one"))
                .expect("derive fresh writeback key");
        let command = M4PrepareSourceOwnerWritebackCommand {
            source_ref: source_ref.clone(),
            expected_source_revision: 7,
            fresh_idempotency_key: idempotency_key.clone(),
            explicit_intent: M4SourceOwnerCommandIntent::RequestCompletion,
            requested_at_utc: "2026-08-10T12:00:00Z".to_string(),
        };
        let intent =
            m4_prepare_source_owner_writeback(&command, &std::collections::BTreeSet::new())
                .expect("prepare typed writeback intent");
        assert_eq!(intent.source_ref, source_ref);
        assert_eq!(intent.expected_source_revision, 7);
        assert_eq!(
            intent.explicit_intent,
            M4SourceOwnerCommandIntent::RequestCompletion
        );
        m4_validate_source_owner_writeback_intent(&intent).expect("validate typed intent");

        let result = M4SourceOwnerWritebackResult {
            source_ref: intent.source_ref.clone(),
            expected_source_revision: intent.expected_source_revision,
            idempotency_key: intent.idempotency_key.clone(),
            intent_fingerprint: intent.intent_fingerprint.clone(),
            outcome: M4SourceOwnerWritebackOutcome::Succeeded,
            owner_receipt_ref: opaque("owner-receipt", "completion-7"),
            recorded_at_utc: "2026-08-10T12:01:00Z".to_string(),
        };
        m4_validate_source_owner_writeback_result(&intent, &result)
            .expect("validate scrubbed owner receipt");
        assert_eq!(result.source_ref.source_status, M4SourceStatus::Open);

        let mut used = std::collections::BTreeSet::new();
        used.insert(idempotency_key);
        assert_eq!(
            m4_prepare_source_owner_writeback(&command, &used),
            Err("m4_source_owner_writeback_idempotency_key_not_fresh".to_string())
        );
        let mut raw_callback_command = command.clone();
        raw_callback_command.source_ref.source_link.opaque_route_ref =
            "callback://owner/execute".to_string();
        assert_eq!(
            m4_prepare_source_owner_writeback(
                &raw_callback_command,
                &std::collections::BTreeSet::new()
            ),
            Err("m4_source_record_ref_not_admitted:SOURCE_LINK_ROUTE_REF_INVALID".to_string())
        );
        let mut raw_credential_result = result.clone();
        raw_credential_result.owner_receipt_ref = "credential=raw-secret".to_string();
        assert_eq!(
            m4_validate_source_owner_writeback_result(&intent, &raw_credential_result),
            Err("m4_opaque_reference_invalid:source_owner_writeback_owner_receipt_ref".to_string())
        );
        let mut mismatched_result = result;
        mismatched_result.idempotency_key = opaque("writeback-idempotency", "different");
        assert_eq!(
            m4_validate_source_owner_writeback_result(&intent, &mismatched_result),
            Err("m4_source_owner_writeback_result_intent_mismatch".to_string())
        );
    }

    #[test]
    fn m4c04_fingerprints_include_immutable_command_fields_and_validate_inputs() {
        let loop_state = open_loop(attention_source(7), M4OpenLoopStatus::Open, 1);
        let common_metadata = metadata("same-snooze-key", "2026-08-10T12:00:00Z");
        let first_snooze = m4_snooze_open_loop(
            &loop_state,
            &M4SnoozeOpenLoopCommand {
                open_loop_id: loop_state.open_loop_id.clone(),
                expected_revision: 1,
                snoozed_until_utc: "2026-08-10T12:10:00Z".to_string(),
                metadata: common_metadata.clone(),
            },
        )
        .expect("first snooze");
        let second_snooze = m4_snooze_open_loop(
            &loop_state,
            &M4SnoozeOpenLoopCommand {
                open_loop_id: loop_state.open_loop_id.clone(),
                expected_revision: 1,
                snoozed_until_utc: "2026-08-10T12:20:00Z".to_string(),
                metadata: common_metadata,
            },
        )
        .expect("second snooze");
        assert_ne!(
            first_snooze.idempotency_fingerprint,
            second_snooze.idempotency_fingerprint
        );

        let explicit_command_id = opaque("user-command", "same-action");
        let action_a = m4_create_personal_action(
            &M4PersonalActionCreationRequest::ExplicitUserStandaloneTodo(
                M4CreatePersonalActionCommand {
                    explicit_user_command_id: explicit_command_id.clone(),
                    title: "第一件事".to_string(),
                    due_at_utc: None,
                    metadata: metadata("same-action-key", "2026-08-10T12:00:00Z"),
                },
            ),
        )
        .expect("first action request");
        let action_b = m4_create_personal_action(
            &M4PersonalActionCreationRequest::ExplicitUserStandaloneTodo(
                M4CreatePersonalActionCommand {
                    explicit_user_command_id: explicit_command_id,
                    title: "第二件事".to_string(),
                    due_at_utc: Some("2026-08-11T09:00:00Z".to_string()),
                    metadata: metadata("same-action-key", "2026-08-10T12:01:00Z"),
                },
            ),
        )
        .expect("second action request");
        assert_eq!(
            action_a.aggregate.personal_action_id,
            action_b.aggregate.personal_action_id
        );
        assert_ne!(
            action_a.idempotency_fingerprint,
            action_b.idempotency_fingerprint
        );

        let owner_ref = m4_personal_action_id(&opaque("user-command", "same-reminder-owner"))
            .expect("owner ID");
        let schedule_command_id = opaque("schedule-command", "same-reminder");
        let reminder_a = m4_create_reminder(&M4CreateReminderCommand {
            owner_ref: owner_ref.clone(),
            explicit_schedule_command_id: schedule_command_id.clone(),
            scheduled_for_utc: "2026-08-10T13:00:00Z".to_string(),
            iana_timezone: "Asia/Shanghai".to_string(),
            metadata: metadata("same-reminder-key", "2026-08-10T12:00:00Z"),
        })
        .expect("first reminder request");
        let reminder_b = m4_create_reminder(&M4CreateReminderCommand {
            owner_ref,
            explicit_schedule_command_id: schedule_command_id,
            scheduled_for_utc: "2026-08-10T14:00:00Z".to_string(),
            iana_timezone: "America/New_York".to_string(),
            metadata: metadata("same-reminder-key", "2026-08-10T12:01:00Z"),
        })
        .expect("second reminder request");
        assert_eq!(
            reminder_a.aggregate.reminder_id,
            reminder_b.aggregate.reminder_id
        );
        assert_ne!(
            reminder_a.idempotency_fingerprint,
            reminder_b.idempotency_fingerprint
        );

        let subject_ref = loop_state.open_loop_id.clone();
        let notification_a = m4_create_notification(&M4CreateNotificationCommand {
            source_ref: attention_source(7),
            subject_ref: subject_ref.clone(),
            notification_purpose_code: "ATTENTION_REQUIRED".to_string(),
            delivery_channel: M4_IN_APP_DELIVERY_CHANNEL.to_string(),
            metadata: metadata("same-notification-key", "2026-08-10T12:00:00Z"),
        })
        .expect("first notification request");
        let notification_b = m4_create_notification(&M4CreateNotificationCommand {
            source_ref: attention_source(8),
            subject_ref,
            notification_purpose_code: "ATTENTION_REQUIRED".to_string(),
            delivery_channel: M4_IN_APP_DELIVERY_CHANNEL.to_string(),
            metadata: metadata("same-notification-key", "2026-08-10T12:01:00Z"),
        })
        .expect("second notification request");
        assert_eq!(
            notification_a.aggregate.notification_id,
            notification_b.aggregate.notification_id
        );
        assert_ne!(
            notification_a.idempotency_fingerprint,
            notification_b.idempotency_fingerprint
        );

        let mut invalid_sensitivity = attention_source(7);
        invalid_sensitivity.sensitivity = "RAW_SOURCE_BODY".to_string();
        assert_eq!(
            m4_validate_source_record_ref(&invalid_sensitivity),
            Err("m4_source_record_ref_not_admitted:SENSITIVITY_NOT_ADMITTED".to_string())
        );
        assert_eq!(
            m4_create_reminder(&M4CreateReminderCommand {
                owner_ref: loop_state.open_loop_id,
                explicit_schedule_command_id: opaque("schedule-command", "invalid-zone"),
                scheduled_for_utc: "2026-08-10T13:00:00Z".to_string(),
                iana_timezone: "UTC".to_string(),
                metadata: metadata("invalid-zone", "2026-08-10T12:00:00Z"),
            }),
            Err("m4_reminder_iana_timezone_invalid".to_string())
        );
    }

    #[test]
    fn m4c04_status_parsers_fail_closed_and_ids_are_frozen() {
        assert_eq!(
            M4InboxItemStatus::parse("NEW"),
            Some(M4InboxItemStatus::New)
        );
        assert_eq!(
            M4OpenLoopStatus::parse("SNOOZED"),
            Some(M4OpenLoopStatus::Snoozed)
        );
        assert_eq!(
            M4PersonalActionStatus::parse("COMPLETED"),
            Some(M4PersonalActionStatus::Completed)
        );
        assert_eq!(
            M4NotificationStatus::parse("DELIVERED"),
            Some(M4NotificationStatus::Delivered)
        );
        assert_eq!(
            M4ReminderStatus::parse("CANCELLED"),
            Some(M4ReminderStatus::Cancelled)
        );
        for unknown in ["", "open", "UNKNOWN", "PENDING "] {
            assert_eq!(M4InboxItemStatus::parse(unknown), None);
            assert_eq!(M4OpenLoopStatus::parse(unknown), None);
            assert_eq!(M4PersonalActionStatus::parse(unknown), None);
            assert_eq!(M4NotificationStatus::parse(unknown), None);
            assert_eq!(M4ReminderStatus::parse(unknown), None);
        }

        let explicit_user_command_id = opaque("user-command", "frozen-personal-action");
        let subject_ref = m4_open_loop_id(
            &m4_source_record_identity_key(&attention_source(7)).expect("source identity"),
        )
        .expect("OpenLoop ID");
        let schedule_command_id = opaque("schedule-command", "frozen-reminder");
        assert_eq!(
            m4_personal_action_id(&explicit_user_command_id).expect("personal action ID"),
            "personal-action:3659d10974cb50a9cd1791510b086d9f7c4bbc7009f118b896dd24da248d7351"
        );
        assert_eq!(
            m4_notification_id(&subject_ref, "ATTENTION_REQUIRED").expect("notification ID"),
            "notification:a1aa2f2f8d49a0584e6cb80d6b63eec40b4a7acd312b3406c7db2cbcd2ed2465"
        );
        assert_eq!(
            m4_reminder_id(&subject_ref, &schedule_command_id).expect("reminder ID"),
            "reminder:d01a6eebc68d66da5d87941811092caa6756dfedc44466059b5dce8f74df872f"
        );
        assert_eq!(
            m4_source_record_fingerprint(&attention_source(7)).expect("source fingerprint"),
            "source-record:5885d65bd9320ac53a8eb432bae7c89173401da0a5f45d1d476e9df024a93d79"
        );
    }
}
