// SYN-FND-003: Identity / Scope / Policy Kernel.
//
// 通用 project-scoped resolver 仍是既有 staged foundation。M4C02 只接入本文件内
// 无入参、后端固定的 primary Secretary / PersonalScope resolver；它不会把通用
// resolver、前端声明或项目 cwd 变成个人身份权威。
//
// Contract: docs/contracts/identity-scope-v1.md
// Evidence level: STATIC_OPENING_ONLY → will be upgraded after focused tests.

#![allow(dead_code)] // most generic foundation types remain staged; warnings are expected

use sha2::{Digest, Sha256};
use std::fmt;

// ============================================================================
// §1  Core Identity Types (from identity-scope-v1 contract)
// ============================================================================

/// Unique actor identifier. Server-resolved, never caller-claimed.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ActorId(pub String);

impl fmt::Display for ActorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Stable project identifier, derived from project_root via deterministic hash.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ProjectId(pub String);

impl ProjectId {
    /// Derive project_id from a project root path (deterministic).
    pub fn from_root(project_root: &str) -> Self {
        Self(format!("project:{}", stable_id(project_root)))
    }
}

impl fmt::Display for ProjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Scope reference — identifies the resolution context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopeRef {
    pub kind: ScopeKind,
    pub scope_id: String,
    pub revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScopeKind {
    Personal,
    Global,
    Project,
}

impl fmt::Display for ScopeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Personal => write!(f, "personal"),
            Self::Global => write!(f, "global"),
            Self::Project => write!(f, "project"),
        }
    }
}

/// Role reference — identifies the actor's role in the current scope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleRef {
    pub role_id: String,
    pub kind: RoleKind,
    pub revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RoleKind {
    Secretary,
    GlobalSupervisor,
    ProjectSupervisor,
    StableMember,
    TemporaryAgent,
    Worker,
    User,
    System,
}

impl RoleKind {
    /// Parse from string (for backward compatibility with existing code).
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "secretary" => Some(Self::Secretary),
            "global_director" | "global_supervisor" => Some(Self::GlobalSupervisor),
            "project_director" | "project_supervisor" => Some(Self::ProjectSupervisor),
            "stable_member" => Some(Self::StableMember),
            "temporary_agent" => Some(Self::TemporaryAgent),
            "worker" => Some(Self::Worker),
            "user" => Some(Self::User),
            "system" => Some(Self::System),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Secretary => "secretary",
            Self::GlobalSupervisor => "global_supervisor",
            Self::ProjectSupervisor => "project_supervisor",
            Self::StableMember => "stable_member",
            Self::TemporaryAgent => "temporary_agent",
            Self::Worker => "worker",
            Self::User => "user",
            Self::System => "system",
        }
    }
}

/// Current object reference — what the actor is currently working on.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrentObjectRef {
    pub object_type: String,
    pub object_id: String,
    pub source_owner_ref: String,
    pub scope_ref: ScopeRef,
    pub binding_revision: u64,
    pub binding_source_ref: String,
}

/// Execution channel — determines the risk class and side-effect mode.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionChannel {
    pub kind: ChannelKind,
    pub risk_class: RiskClass,
    pub side_effect_mode: SideEffectMode,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ChannelKind {
    Daily,
    Development,
}

impl ChannelKind {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "daily" => Some(Self::Daily),
            "development" => Some(Self::Development),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RiskClass {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SideEffectMode {
    ReadOnly,
    WriteLocal,
    WriteExternal,
}

/// Permission profile — what the actor is allowed to do.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PermissionProfile {
    pub profile_id: String,
    pub allow_capabilities: Vec<String>,
    pub deny_capabilities: Vec<String>,
    pub constraints: Vec<String>,
    pub revision: u64,
}

/// Permission snapshot — immutable reference to a permission profile at a point in time.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PermissionSnapshotRef {
    pub snapshot_id: String,
    pub profile_id: String,
    pub actor_id: ActorId,
    pub scope_ref: ScopeRef,
    pub execution_channel: ExecutionChannel,
    pub revision: u64,
    pub snapshot_hash: String,
    pub issued_at: String,
}

/// Complete identity snapshot — the resolved identity for a command.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentitySnapshot {
    pub actor_id: ActorId,
    pub role_ref: RoleRef,
    pub scope_ref: ScopeRef,
    pub current_object_ref: Option<CurrentObjectRef>,
    pub execution_channel: ExecutionChannel,
    pub permission_snapshot_ref: PermissionSnapshotRef,
    pub owner_fingerprint: String,
    pub resolved_at: String,
}

// ============================================================================
// §2  Identity Resolution
// ============================================================================

/// Result of identity resolution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IdentityResolution {
    Resolved(IdentitySnapshot),
    Denied(String),
    Quarantined(String),
}

// ============================================================================
// §2.1  M4C02 Primary Secretary Identity
// ============================================================================

/// The fixed M4C02 server-owned identity profile.  This is deliberately not a
/// frontend-selectable profile, project locator, cwd alias, or caller claim.
pub const M4_PRIMARY_SECRETARY_PROFILE_ID: &str = "m4-secretary-personal-primary-v1";
pub const M4_PRIMARY_SECRETARY_ACTOR_ID: &str = "actor:local-primary-user";

pub const M4_PRIMARY_SECRETARY_ROLE_ID: &str = "role:secretary:personal-primary";
pub const M4_PRIMARY_SECRETARY_ROLE_KIND: &str = "secretary";
pub const M4_PRIMARY_SECRETARY_ROLE_REVISION: u64 = 1;

pub const M4_PRIMARY_SECRETARY_SCOPE_ID: &str = "scope:personal:primary";
pub const M4_PRIMARY_SECRETARY_SCOPE_KIND: &str = "personal";
pub const M4_PRIMARY_SECRETARY_SCOPE_REVISION: u64 = 1;

pub const M4_PRIMARY_SECRETARY_CURRENT_OBJECT_TYPE: &str = "personal_workbench";
pub const M4_PRIMARY_SECRETARY_CURRENT_OBJECT_ID: &str = "personal-workbench:primary";
pub const M4_PRIMARY_SECRETARY_CURRENT_OBJECT_SOURCE_OWNER_REF: &str = "identity_scope_kernel";
pub const M4_PRIMARY_SECRETARY_CURRENT_OBJECT_BINDING_REVISION: u64 = 1;
pub const M4_PRIMARY_SECRETARY_CURRENT_OBJECT_BINDING_SOURCE_REF: &str =
    "m4-secretary-bootstrap:v1";

pub const M4_PRIMARY_SECRETARY_CHANNEL_KIND: &str = "daily";
pub const M4_PRIMARY_SECRETARY_RISK_CLASS: &str = "low";
pub const M4_PRIMARY_SECRETARY_SIDE_EFFECT_MODE: &str = "write_local";

pub const M4_PRIMARY_SECRETARY_PERMISSION_PROFILE_ID: &str =
    "permission:m4-secretary-local-coordination:v1";
pub const M4_PRIMARY_SECRETARY_PERMISSION_PROFILE_REVISION: u64 = 1;
pub const M4_PRIMARY_SECRETARY_ALLOW_CAPABILITIES: &[&str] = &[
    "read_personal_coordination",
    "read_registered_internal_source_refs",
    "write_m4_coordination_state",
    "create_explicit_standalone_personal_action",
    "request_registered_owner_command",
    "create_m3_handoff",
    "read_m3_handoff_receipt",
];
pub const M4_PRIMARY_SECRETARY_DENY_CAPABILITIES: &[&str] = &[
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
    "send_external_message",
];
pub const M4_PRIMARY_SECRETARY_CONSTRAINTS: &[&str] = &[
    "source_ref_required",
    "scrubbed_summary_only",
    "local_coordination_writes_only",
    "owner_command_requires_explicit_user_intent_and_owner_receipt",
];

/// Immutable v1 permission-snapshot identity.  The timestamp is tied to the
/// frozen M4 contract revision rather than a renderer or local-clock value, so
/// the resolver returns the same snapshot after a restart.
pub const M4_PRIMARY_SECRETARY_PERMISSION_SNAPSHOT_ID: &str =
    "permission-snapshot:m4-secretary-personal-primary:v1";
pub const M4_PRIMARY_SECRETARY_PERMISSION_SNAPSHOT_REVISION: u64 = 1;
pub const M4_PRIMARY_SECRETARY_PERMISSION_SNAPSHOT_ISSUED_AT: &str = "2026-08-10T04:25:04Z";
pub const M4_PRIMARY_SECRETARY_PERMISSION_SNAPSHOT_HASH_DOMAIN_SEPARATOR: &str =
    "syn.m4.secretary-permission-snapshot/v1";

/// Each M3 binding field is a three-part opaque envelope:
/// `namespace:sha256:<64 lowercase hex>`. Its digest covers this domain,
/// then the namespace and server-fixed canonical material as u32-big-endian
/// byte-length-prefixed UTF-8 components. Typed M4 identity values remain
/// separate and never cross the M3 repository boundary as plaintext.
pub const M4_PRIMARY_SECRETARY_M3_OPAQUE_REFERENCE_DOMAIN_SEPARATOR: &str =
    "syn.m4.primary-secretary-m3-opaque-ref/v1";
pub const M4_PRIMARY_SECRETARY_M3_ACTOR_NAMESPACE: &str = "actor";
pub const M4_PRIMARY_SECRETARY_M3_ROLE_NAMESPACE: &str = "role";
pub const M4_PRIMARY_SECRETARY_M3_SCOPE_NAMESPACE: &str = "scope";
pub const M4_PRIMARY_SECRETARY_M3_OBJECT_NAMESPACE: &str = "object";
pub const M4_PRIMARY_SECRETARY_M3_CHANNEL_NAMESPACE: &str = "channel";
pub const M4_PRIMARY_SECRETARY_M3_PERMISSION_NAMESPACE: &str = "permission";
pub const M4_PRIMARY_SECRETARY_M3_OWNER_FINGERPRINT_DOMAIN_SEPARATOR: &str =
    "syn.m3.role-session-owner/v1";

/// The fixed server resolution receipt is versioned with the M4 contract, not
/// with a caller-supplied or wall-clock value.
pub const M4_PRIMARY_SECRETARY_RESOLVED_AT: &str = "2026-08-10T04:25:04Z";

/// M4C02's typed M3 hand-off fields.  Consumers construct an M3
/// `ServerResolvedBinding` only from this output; project paths, frontend
/// cache, routes, and caller-supplied identity fields never enter it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct M4PrimarySecretaryM3Binding {
    pub actor_id: String,
    pub role_ref: String,
    pub scope_ref: String,
    pub current_object_ref: String,
    pub execution_channel: String,
    pub permission_snapshot_ref: String,
    pub owner_fingerprint: String,
}

/// Fixed primary Secretary identity returned to the ordinary-product M4C02
/// composition path.  It carries the complete contract permission profile as
/// well as the standard identity snapshot and M3 binding fields.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct M4PrimarySecretaryIdentity {
    pub profile_id: String,
    pub identity_snapshot: IdentitySnapshot,
    pub permission_profile: PermissionProfile,
    pub m3_binding: M4PrimarySecretaryM3Binding,
}

/// Scrubbed, fail-closed outcomes for the fixed Secretary resolver.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum M4PrimarySecretaryIdentityError {
    ContractInvariantMismatch,
    PermissionSnapshotMismatch,
    M3OwnerFingerprintUnavailable,
    M3OwnerFingerprintMismatch,
}

impl M4PrimarySecretaryIdentityError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::ContractInvariantMismatch => "m4_primary_secretary_contract_invariant_mismatch",
            Self::PermissionSnapshotMismatch => "m4_primary_secretary_permission_snapshot_mismatch",
            Self::M3OwnerFingerprintUnavailable => {
                "m4_primary_secretary_m3_owner_fingerprint_unavailable"
            }
            Self::M3OwnerFingerprintMismatch => {
                "m4_primary_secretary_m3_owner_fingerprint_mismatch"
            }
        }
    }
}

impl fmt::Display for M4PrimarySecretaryIdentityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code())
    }
}

impl std::error::Error for M4PrimarySecretaryIdentityError {}

impl M4PrimarySecretaryIdentity {
    /// Materializes the M3-native binding only after the fixed M4C02 identity
    /// has been revalidated.  M3 re-derives the owner fingerprint itself;
    /// a mismatching field never falls back to the generic project resolver.
    pub(crate) fn m3_server_resolved_binding(
        &self,
    ) -> Result<crate::m3_role_session::ServerResolvedBinding, M4PrimarySecretaryIdentityError>
    {
        verify_m4_primary_secretary_identity(self)?;
        let binding = crate::m3_role_session::ServerResolvedBinding::from_server_canonical(
            self.m3_binding.actor_id.clone(),
            self.m3_binding.role_ref.clone(),
            self.m3_binding.scope_ref.clone(),
            self.m3_binding.current_object_ref.clone(),
            self.m3_binding.execution_channel.clone(),
            self.m3_binding.permission_snapshot_ref.clone(),
        )
        .map_err(|_| M4PrimarySecretaryIdentityError::M3OwnerFingerprintUnavailable)?;

        if binding.owner_fingerprint.as_str() != self.m3_binding.owner_fingerprint {
            return Err(M4PrimarySecretaryIdentityError::M3OwnerFingerprintMismatch);
        }
        Ok(binding)
    }
}

/// Resolves the one ordinary-product primary Secretary identity.
///
/// It intentionally takes no caller argument: actor, role, personal scope,
/// current object, daily local-write channel, permission snapshot, and owner
/// fingerprint all come from the frozen server-owned M4 contract.  This path
/// does not call or extend [`resolve_identity`], whose project-scoped behavior
/// remains unchanged for its existing callers.
pub fn resolve_m4_primary_secretary_identity(
) -> Result<M4PrimarySecretaryIdentity, M4PrimarySecretaryIdentityError> {
    let identity = build_m4_primary_secretary_identity()?;
    verify_m4_primary_secretary_identity(&identity)?;
    Ok(identity)
}

/// Revalidates a previously resolved primary Secretary identity before a
/// downstream M3 or M4 command admits it.  Any drift is a scrubbed error, not
/// a reason to substitute a project identity or weaken the permission profile.
pub fn verify_m4_primary_secretary_identity(
    identity: &M4PrimarySecretaryIdentity,
) -> Result<(), M4PrimarySecretaryIdentityError> {
    let expected = build_m4_primary_secretary_identity()?;

    if identity.permission_profile != expected.permission_profile
        || identity.identity_snapshot.permission_snapshot_ref
            != expected.identity_snapshot.permission_snapshot_ref
    {
        return Err(M4PrimarySecretaryIdentityError::PermissionSnapshotMismatch);
    }

    if identity.m3_binding.owner_fingerprint != expected.m3_binding.owner_fingerprint
        || identity.identity_snapshot.owner_fingerprint
            != expected.identity_snapshot.owner_fingerprint
    {
        return Err(M4PrimarySecretaryIdentityError::M3OwnerFingerprintMismatch);
    }

    if identity != &expected {
        return Err(M4PrimarySecretaryIdentityError::ContractInvariantMismatch);
    }

    Ok(())
}

fn build_m4_primary_secretary_identity(
) -> Result<M4PrimarySecretaryIdentity, M4PrimarySecretaryIdentityError> {
    let permission_profile = PermissionProfile {
        profile_id: M4_PRIMARY_SECRETARY_PERMISSION_PROFILE_ID.to_string(),
        allow_capabilities: string_values(M4_PRIMARY_SECRETARY_ALLOW_CAPABILITIES),
        deny_capabilities: string_values(M4_PRIMARY_SECRETARY_DENY_CAPABILITIES),
        constraints: string_values(M4_PRIMARY_SECRETARY_CONSTRAINTS),
        revision: M4_PRIMARY_SECRETARY_PERMISSION_PROFILE_REVISION,
    };
    let scope_ref = ScopeRef {
        kind: ScopeKind::Personal,
        scope_id: M4_PRIMARY_SECRETARY_SCOPE_ID.to_string(),
        revision: M4_PRIMARY_SECRETARY_SCOPE_REVISION,
    };
    let execution_channel = ExecutionChannel {
        kind: ChannelKind::Daily,
        risk_class: RiskClass::Low,
        side_effect_mode: SideEffectMode::WriteLocal,
    };
    let role_ref = RoleRef {
        role_id: M4_PRIMARY_SECRETARY_ROLE_ID.to_string(),
        kind: RoleKind::Secretary,
        revision: M4_PRIMARY_SECRETARY_ROLE_REVISION,
    };
    let current_object_ref = CurrentObjectRef {
        object_type: M4_PRIMARY_SECRETARY_CURRENT_OBJECT_TYPE.to_string(),
        object_id: M4_PRIMARY_SECRETARY_CURRENT_OBJECT_ID.to_string(),
        source_owner_ref: M4_PRIMARY_SECRETARY_CURRENT_OBJECT_SOURCE_OWNER_REF.to_string(),
        scope_ref: scope_ref.clone(),
        binding_revision: M4_PRIMARY_SECRETARY_CURRENT_OBJECT_BINDING_REVISION,
        binding_source_ref: M4_PRIMARY_SECRETARY_CURRENT_OBJECT_BINDING_SOURCE_REF.to_string(),
    };
    let permission_snapshot_ref = PermissionSnapshotRef {
        snapshot_id: M4_PRIMARY_SECRETARY_PERMISSION_SNAPSHOT_ID.to_string(),
        profile_id: permission_profile.profile_id.clone(),
        actor_id: ActorId(M4_PRIMARY_SECRETARY_ACTOR_ID.to_string()),
        scope_ref: scope_ref.clone(),
        execution_channel: execution_channel.clone(),
        revision: M4_PRIMARY_SECRETARY_PERMISSION_SNAPSHOT_REVISION,
        snapshot_hash: m4_primary_secretary_permission_snapshot_hash()?,
        issued_at: M4_PRIMARY_SECRETARY_PERMISSION_SNAPSHOT_ISSUED_AT.to_string(),
    };
    let m3_binding = build_m4_primary_secretary_m3_binding(
        &role_ref,
        &scope_ref,
        &current_object_ref,
        &execution_channel,
        &permission_snapshot_ref,
    )?;
    let owner_fingerprint = m3_binding.owner_fingerprint.clone();

    Ok(M4PrimarySecretaryIdentity {
        profile_id: M4_PRIMARY_SECRETARY_PROFILE_ID.to_string(),
        identity_snapshot: IdentitySnapshot {
            actor_id: ActorId(M4_PRIMARY_SECRETARY_ACTOR_ID.to_string()),
            role_ref,
            scope_ref,
            current_object_ref: Some(current_object_ref),
            execution_channel,
            permission_snapshot_ref,
            owner_fingerprint: owner_fingerprint.clone(),
            resolved_at: M4_PRIMARY_SECRETARY_RESOLVED_AT.to_string(),
        },
        permission_profile,
        m3_binding,
    })
}

fn build_m4_primary_secretary_m3_binding(
    role_ref: &RoleRef,
    scope_ref: &ScopeRef,
    current_object_ref: &CurrentObjectRef,
    execution_channel: &ExecutionChannel,
    permission_snapshot_ref: &PermissionSnapshotRef,
) -> Result<M4PrimarySecretaryM3Binding, M4PrimarySecretaryIdentityError> {
    let role_revision = role_ref.revision.to_string();
    let scope_revision = scope_ref.revision.to_string();
    let current_scope_revision = current_object_ref.scope_ref.revision.to_string();
    let binding_revision = current_object_ref.binding_revision.to_string();
    let permission_snapshot_revision = permission_snapshot_ref.revision.to_string();
    let scope_kind = scope_ref.kind.to_string();
    let current_scope_kind = current_object_ref.scope_ref.kind.to_string();
    let channel_kind = channel_kind_str(&execution_channel.kind);
    let risk_class = risk_class_str(&execution_channel.risk_class);
    let side_effect_mode = side_effect_mode_str(&execution_channel.side_effect_mode);

    let actor_id = m4_primary_secretary_m3_opaque_reference(
        M4_PRIMARY_SECRETARY_M3_ACTOR_NAMESPACE,
        &[M4_PRIMARY_SECRETARY_ACTOR_ID],
    )?;
    let m3_role_ref = m4_primary_secretary_m3_opaque_reference(
        M4_PRIMARY_SECRETARY_M3_ROLE_NAMESPACE,
        &[
            role_ref.role_id.as_str(),
            role_ref.kind.as_str(),
            role_revision.as_str(),
        ],
    )?;
    let m3_scope_ref = m4_primary_secretary_m3_opaque_reference(
        M4_PRIMARY_SECRETARY_M3_SCOPE_NAMESPACE,
        &[
            scope_kind.as_str(),
            scope_ref.scope_id.as_str(),
            scope_revision.as_str(),
        ],
    )?;
    let m3_current_object_ref = m4_primary_secretary_m3_opaque_reference(
        M4_PRIMARY_SECRETARY_M3_OBJECT_NAMESPACE,
        &[
            current_object_ref.object_type.as_str(),
            current_object_ref.object_id.as_str(),
            current_object_ref.source_owner_ref.as_str(),
            current_scope_kind.as_str(),
            current_object_ref.scope_ref.scope_id.as_str(),
            current_scope_revision.as_str(),
            binding_revision.as_str(),
            current_object_ref.binding_source_ref.as_str(),
        ],
    )?;
    let m3_execution_channel = m4_primary_secretary_m3_opaque_reference(
        M4_PRIMARY_SECRETARY_M3_CHANNEL_NAMESPACE,
        &[channel_kind, risk_class, side_effect_mode],
    )?;
    let permission_snapshot_ref = m4_primary_secretary_m3_opaque_reference(
        M4_PRIMARY_SECRETARY_M3_PERMISSION_NAMESPACE,
        &[
            M4_PRIMARY_SECRETARY_PROFILE_ID,
            permission_snapshot_ref.snapshot_id.as_str(),
            permission_snapshot_ref.profile_id.as_str(),
            permission_snapshot_ref.actor_id.0.as_str(),
            scope_kind.as_str(),
            permission_snapshot_ref.scope_ref.scope_id.as_str(),
            scope_revision.as_str(),
            channel_kind,
            risk_class,
            side_effect_mode,
            permission_snapshot_revision.as_str(),
            permission_snapshot_ref.snapshot_hash.as_str(),
            permission_snapshot_ref.issued_at.as_str(),
        ],
    )?;

    // M3 owns the fingerprint algorithm.  Its five inputs are the final
    // sealed envelopes, never the typed personal identity values above.
    let owner_fingerprint = crate::m3_role_session::owner_fingerprint_for_components(
        actor_id.as_str(),
        m3_role_ref.as_str(),
        m3_scope_ref.as_str(),
        m3_current_object_ref.as_str(),
        m3_execution_channel.as_str(),
    )
    .map_err(|_| M4PrimarySecretaryIdentityError::M3OwnerFingerprintUnavailable)?
    .as_str()
    .to_string();

    Ok(M4PrimarySecretaryM3Binding {
        actor_id,
        role_ref: m3_role_ref,
        scope_ref: m3_scope_ref,
        current_object_ref: m3_current_object_ref,
        execution_channel: m3_execution_channel,
        permission_snapshot_ref,
        owner_fingerprint,
    })
}

fn m4_primary_secretary_m3_opaque_reference(
    namespace: &str,
    canonical_material: &[&str],
) -> Result<String, M4PrimarySecretaryIdentityError> {
    if !m4_primary_secretary_m3_namespace_is_valid(namespace)
        || canonical_material.iter().any(|component| {
            component.is_empty()
                || component
                    .chars()
                    .any(|character| character.is_control() || character.is_whitespace())
        })
    {
        return Err(M4PrimarySecretaryIdentityError::ContractInvariantMismatch);
    }

    let mut hasher = Sha256::new();
    hasher.update(M4_PRIMARY_SECRETARY_M3_OPAQUE_REFERENCE_DOMAIN_SEPARATOR.as_bytes());
    update_m4_primary_secretary_length_prefixed_component(&mut hasher, namespace)?;
    for component in canonical_material {
        update_m4_primary_secretary_length_prefixed_component(&mut hasher, component)?;
    }
    Ok(format!("{namespace}:sha256:{:x}", hasher.finalize()))
}

fn m4_primary_secretary_m3_namespace_is_valid(namespace: &str) -> bool {
    (1..=64).contains(&namespace.len())
        && namespace
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase())
        && namespace.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

fn m4_primary_secretary_permission_snapshot_hash() -> Result<String, M4PrimarySecretaryIdentityError>
{
    let mut hasher = Sha256::new();
    hasher.update(M4_PRIMARY_SECRETARY_PERMISSION_SNAPSHOT_HASH_DOMAIN_SEPARATOR.as_bytes());

    let profile_revision = M4_PRIMARY_SECRETARY_PERMISSION_PROFILE_REVISION.to_string();
    let role_revision = M4_PRIMARY_SECRETARY_ROLE_REVISION.to_string();
    let scope_revision = M4_PRIMARY_SECRETARY_SCOPE_REVISION.to_string();
    let binding_revision = M4_PRIMARY_SECRETARY_CURRENT_OBJECT_BINDING_REVISION.to_string();
    let snapshot_revision = M4_PRIMARY_SECRETARY_PERMISSION_SNAPSHOT_REVISION.to_string();
    for component in [
        M4_PRIMARY_SECRETARY_PROFILE_ID,
        M4_PRIMARY_SECRETARY_PERMISSION_PROFILE_ID,
        M4_PRIMARY_SECRETARY_ACTOR_ID,
        M4_PRIMARY_SECRETARY_ROLE_ID,
        M4_PRIMARY_SECRETARY_ROLE_KIND,
        role_revision.as_str(),
        M4_PRIMARY_SECRETARY_SCOPE_ID,
        M4_PRIMARY_SECRETARY_SCOPE_KIND,
        scope_revision.as_str(),
        M4_PRIMARY_SECRETARY_CURRENT_OBJECT_TYPE,
        M4_PRIMARY_SECRETARY_CURRENT_OBJECT_ID,
        M4_PRIMARY_SECRETARY_CURRENT_OBJECT_SOURCE_OWNER_REF,
        M4_PRIMARY_SECRETARY_SCOPE_KIND,
        M4_PRIMARY_SECRETARY_SCOPE_ID,
        scope_revision.as_str(),
        binding_revision.as_str(),
        M4_PRIMARY_SECRETARY_CURRENT_OBJECT_BINDING_SOURCE_REF,
        M4_PRIMARY_SECRETARY_CHANNEL_KIND,
        M4_PRIMARY_SECRETARY_RISK_CLASS,
        M4_PRIMARY_SECRETARY_SIDE_EFFECT_MODE,
        M4_PRIMARY_SECRETARY_PERMISSION_SNAPSHOT_ID,
        M4_PRIMARY_SECRETARY_PERMISSION_SNAPSHOT_ISSUED_AT,
        profile_revision.as_str(),
        snapshot_revision.as_str(),
    ] {
        update_m4_primary_secretary_length_prefixed_component(&mut hasher, component)?;
    }

    for (group, values) in [
        (
            "allow_capabilities",
            M4_PRIMARY_SECRETARY_ALLOW_CAPABILITIES,
        ),
        ("deny_capabilities", M4_PRIMARY_SECRETARY_DENY_CAPABILITIES),
        ("constraints", M4_PRIMARY_SECRETARY_CONSTRAINTS),
    ] {
        update_m4_primary_secretary_length_prefixed_component(&mut hasher, group)?;
        update_m4_primary_secretary_length_prefixed_component(
            &mut hasher,
            &values.len().to_string(),
        )?;
        for value in values {
            update_m4_primary_secretary_length_prefixed_component(&mut hasher, value)?;
        }
    }

    Ok(format!("{:x}", hasher.finalize()))
}

fn update_m4_primary_secretary_length_prefixed_component(
    hasher: &mut Sha256,
    component: &str,
) -> Result<(), M4PrimarySecretaryIdentityError> {
    let byte_length = u32::try_from(component.as_bytes().len())
        .map_err(|_| M4PrimarySecretaryIdentityError::ContractInvariantMismatch)?;
    hasher.update(byte_length.to_be_bytes());
    hasher.update(component.as_bytes());
    Ok(())
}

fn string_values(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

/// Resolve identity from inputs. This is the core kernel function.
///
/// # Arguments
/// * `actor_id` - The claimed actor ID (from auth, not frontend)
/// * `project_root` - The project root (trust anchor)
/// * `role_str` - The claimed role
/// * `channel_str` - The execution channel
/// * `caller_boolean` - Whether the caller claims authorization (input only, not truth)
///
/// # Returns
/// `IdentityResolution::Resolved` if all checks pass, `Denied` or `Quarantined` otherwise.
pub fn resolve_identity(
    actor_id: &str,
    project_root: &str,
    role_str: &str,
    channel_str: &str,
    _caller_boolean: bool,
) -> IdentityResolution {
    // 1. Validate actor_id is non-empty
    if actor_id.trim().is_empty() {
        return IdentityResolution::Denied(
            "identity_kernel_rejected: actor_id 不能为空".to_string(),
        );
    }

    // 2. Resolve role（未知角色使用 TemporaryAgent 兜底，不拒绝）
    let role_kind = match RoleKind::from_str(role_str) {
        Some(r) => r,
        None => RoleKind::TemporaryAgent, // 未知角色兜底为临时代理
    };

    // 3. Resolve channel（未知通道兜底为 Daily=只读，fail-safe：不给写权限）
    let channel_kind = match ChannelKind::from_str(channel_str) {
        Some(c) => c,
        None => ChannelKind::Daily, // 未知通道兜底为只读日常通道，不得落到 WriteLocal
    };

    // 4. Derive project_id from project_root (deterministic, not caller-claimed)
    let project_id = ProjectId::from_root(project_root);

    // 5. Build scope
    let scope_ref = ScopeRef {
        kind: ScopeKind::Project,
        scope_id: project_id.0.clone(),
        revision: 1,
    };

    // 6. Determine risk class and side-effect mode from channel
    let (risk_class, side_effect_mode) = match &channel_kind {
        ChannelKind::Daily => (RiskClass::Low, SideEffectMode::ReadOnly),
        ChannelKind::Development => (RiskClass::Medium, SideEffectMode::WriteLocal),
    };

    let execution_channel = ExecutionChannel {
        kind: channel_kind.clone(),
        risk_class,
        side_effect_mode,
    };

    // 7. Build permission profile based on role
    let (allow_capabilities, deny_capabilities) = match role_kind {
        RoleKind::Secretary => (
            vec![
                "read_project".to_string(),
                "read_personal".to_string(),
                "read_global".to_string(),
            ],
            vec![
                "write_project_files".to_string(),
                "execute_code".to_string(),
            ],
        ),
        RoleKind::GlobalSupervisor => (
            vec![
                "read_project".to_string(),
                "read_global".to_string(),
                "review_cross_project".to_string(),
            ],
            vec![
                "write_project_files".to_string(),
                "execute_code".to_string(),
            ],
        ),
        RoleKind::ProjectSupervisor => (
            vec![
                "read_project".to_string(),
                "write_project_workflow".to_string(),
                "dispatch_workers".to_string(),
                "review_workers".to_string(),
            ],
            vec!["execute_code".to_string()],
        ),
        RoleKind::Worker => (
            vec![
                "read_project".to_string(),
                "write_project_files".to_string(),
            ],
            vec!["dispatch_workers".to_string(), "review_workers".to_string()],
        ),
        RoleKind::User => (
            vec![
                "read_project".to_string(),
                "write_project_files".to_string(),
                "execute_code".to_string(),
                "dispatch_workers".to_string(),
                "review_workers".to_string(),
            ],
            vec![],
        ),
        RoleKind::System => (
            vec![
                "read_project".to_string(),
                "write_project_files".to_string(),
                "execute_code".to_string(),
            ],
            vec![],
        ),
        _ => (vec![], vec!["*".to_string()]),
    };

    let permission_profile = PermissionProfile {
        profile_id: format!(
            "profile:{}:{}",
            role_kind.as_str(),
            channel_kind_str(&execution_channel.kind)
        ),
        allow_capabilities,
        deny_capabilities,
        constraints: vec![],
        revision: 1,
    };

    // 8. Build permission snapshot
    let snapshot_hash = format!(
        "snap:{}:{}:{}",
        actor_id,
        role_kind.as_str(),
        channel_kind_str(&execution_channel.kind)
    );
    let permission_snapshot_ref = PermissionSnapshotRef {
        snapshot_id: format!("snap-{}", &snapshot_hash[..12]),
        profile_id: permission_profile.profile_id.clone(),
        actor_id: ActorId(actor_id.to_string()),
        scope_ref: scope_ref.clone(),
        execution_channel: execution_channel.clone(),
        revision: 1,
        snapshot_hash,
        issued_at: crate::mcp::storage::iso_now(),
    };

    // 9. Caller boolean is INPUT ONLY — it does not grant or deny
    //    The kernel resolves identity independently of caller claims.
    //    If caller_boolean is true but the kernel hasn't verified it,
    //    that's fine — the kernel's resolution stands on its own.

    IdentityResolution::Resolved(IdentitySnapshot {
        actor_id: ActorId(actor_id.to_string()),
        role_ref: RoleRef {
            role_id: role_kind.as_str().to_string(),
            kind: role_kind,
            revision: 1,
        },
        scope_ref,
        current_object_ref: None,
        execution_channel,
        permission_snapshot_ref,
        owner_fingerprint: format!("fp:{}:{}", project_id.0, actor_id),
        resolved_at: crate::mcp::storage::iso_now(),
    })
}

fn channel_kind_str(kind: &ChannelKind) -> &'static str {
    match kind {
        ChannelKind::Daily => "daily",
        ChannelKind::Development => "development",
    }
}

fn risk_class_str(risk_class: &RiskClass) -> &'static str {
    match risk_class {
        RiskClass::Low => "low",
        RiskClass::Medium => "medium",
        RiskClass::High => "high",
        RiskClass::Critical => "critical",
    }
}

fn side_effect_mode_str(side_effect_mode: &SideEffectMode) -> &'static str {
    match side_effect_mode {
        SideEffectMode::ReadOnly => "read_only",
        SideEffectMode::WriteLocal => "write_local",
        SideEffectMode::WriteExternal => "write_external",
    }
}

// ============================================================================
// §3  Policy Judgment
// ============================================================================

/// Policy decision for a command.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PolicyDecision {
    Allowed,
    Denied(String),
    NeedsConfirmation(String),
}

/// Check if a specific capability is allowed by the permission profile.
pub fn policy_check_capability(profile: &PermissionProfile, capability: &str) -> PolicyDecision {
    // Check deny list first (explicit deny wins)
    for denied in &profile.deny_capabilities {
        if denied == "*" || denied == capability {
            return PolicyDecision::Denied(format!(
                "policy_denied: 能力 '{}' 被权限配置 '{}' 拒绝",
                capability, profile.profile_id
            ));
        }
    }

    // Check allow list
    for allowed in &profile.allow_capabilities {
        if allowed == "*" || allowed == capability {
            return PolicyDecision::Allowed;
        }
    }

    // Not in allow list = denied
    PolicyDecision::Denied(format!(
        "policy_denied: 能力 '{}' 不在权限配置 '{}' 的允许列表中",
        capability, profile.profile_id
    ))
}

/// Check if a write to a specific path is allowed.
pub fn policy_check_write(identity: &IdentitySnapshot, _target_path: &str) -> PolicyDecision {
    // System role bypasses write checks (for bootstrap/testing)
    if identity.role_ref.kind == RoleKind::System {
        return PolicyDecision::Allowed;
    }

    // Read-only channels cannot write
    if identity.execution_channel.side_effect_mode == SideEffectMode::ReadOnly {
        return PolicyDecision::Denied("policy_denied: 只读通道不允许写入".to_string());
    }

    // Check capability
    policy_check_capability(
        &identity
            .permission_snapshot_ref
            .profile_id
            .split(':')
            .nth(1)
            .map(|_| PermissionProfile {
                profile_id: identity.permission_snapshot_ref.profile_id.clone(),
                allow_capabilities: vec!["write_project_files".to_string()],
                deny_capabilities: vec![],
                constraints: vec![],
                revision: 1,
            })
            .unwrap_or_else(|| PermissionProfile {
                profile_id: "default".to_string(),
                allow_capabilities: vec![],
                deny_capabilities: vec!["*".to_string()],
                constraints: vec![],
                revision: 1,
            }),
        "write_project_files",
    )
}

/// Validate that a session's permission profile hasn't drifted.
/// Returns Ok(()) if the snapshot matches, Err(reason) if drift detected.
pub fn validate_session_permission_integrity(
    original: &PermissionSnapshotRef,
    current_role: &str,
    current_channel: &str,
) -> Result<(), String> {
    // Re-resolve from the stored role and channel
    let re_resolved = resolve_identity(
        &original.actor_id.0,
        &original.scope_ref.scope_id,
        current_role,
        current_channel,
        false,
    );

    match re_resolved {
        IdentityResolution::Resolved(snapshot) => {
            if snapshot.permission_snapshot_ref.profile_id != original.profile_id {
                return Err(format!(
                    "permission_drift_detected: 原始 profile '{}' ≠ 当前解析 profile '{}'",
                    original.profile_id, snapshot.permission_snapshot_ref.profile_id
                ));
            }
            if snapshot.permission_snapshot_ref.snapshot_hash != original.snapshot_hash {
                return Err(format!(
                    "permission_drift_detected: 原始 snapshot hash ≠ 当前解析 hash"
                ));
            }
            Ok(())
        }
        other => Err(format!(
            "permission_drift_detected: 重新解析失败: {other:?}"
        )),
    }
}

// ============================================================================
// §4  Helpers
// ============================================================================

/// Deterministic stable ID from a value (lowercase + hyphens).
/// This is the same logic as `stable_id` in lib.rs.
fn stable_id(value: &str) -> String {
    value
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_m3_opaque_reference_envelope(value: &str, expected_namespace: &str) {
        let parts = value.split(':').collect::<Vec<_>>();
        assert_eq!(parts.len(), 3, "M3 opaque refs have exactly three parts");
        assert_eq!(parts[0], expected_namespace);
        assert_eq!(parts[1], "sha256");
        assert_eq!(parts[2].len(), 64);
        assert!(parts[2]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)));
    }

    // ---- ActorId ----

    #[test]
    fn actor_id_display() {
        let id = ActorId("user-001".to_string());
        assert_eq!(format!("{id}"), "user-001");
    }

    // ---- ProjectId ----

    #[test]
    fn project_id_from_root_deterministic() {
        let p1 = ProjectId::from_root("/Users/yoyi/Documents/mario test");
        let p2 = ProjectId::from_root("/Users/yoyi/Documents/mario test");
        assert_eq!(p1, p2);
    }

    #[test]
    fn project_id_from_different_roots() {
        let p1 = ProjectId::from_root("/path/a");
        let p2 = ProjectId::from_root("/path/b");
        assert_ne!(p1, p2);
    }

    // ---- RoleKind ----

    #[test]
    fn role_kind_from_str_valid() {
        assert_eq!(RoleKind::from_str("worker"), Some(RoleKind::Worker));
        assert_eq!(RoleKind::from_str("user"), Some(RoleKind::User));
        assert_eq!(RoleKind::from_str("system"), Some(RoleKind::System));
        assert_eq!(
            RoleKind::from_str("project_director"),
            Some(RoleKind::ProjectSupervisor)
        );
        assert_eq!(
            RoleKind::from_str("global_director"),
            Some(RoleKind::GlobalSupervisor)
        );
    }

    #[test]
    fn role_kind_from_str_invalid() {
        assert_eq!(RoleKind::from_str("hacker"), None);
        assert_eq!(RoleKind::from_str(""), None);
    }

    // ---- resolve_identity ----

    #[test]
    fn resolve_identity_valid_worker() {
        let result = resolve_identity(
            "worker-001",
            "/Users/yoyi/codex-workflow-mario-test",
            "worker",
            "development",
            false,
        );
        match result {
            IdentityResolution::Resolved(snapshot) => {
                assert_eq!(snapshot.actor_id.0, "worker-001");
                assert_eq!(snapshot.role_ref.kind, RoleKind::Worker);
                assert_eq!(snapshot.execution_channel.kind, ChannelKind::Development);
                assert_eq!(
                    snapshot.execution_channel.side_effect_mode,
                    SideEffectMode::WriteLocal
                );
            }
            other => panic!("Expected Resolved, got {other:?}"),
        }
    }

    #[test]
    fn resolve_identity_empty_actor() {
        let result = resolve_identity("", "/path", "worker", "development", false);
        assert!(matches!(result, IdentityResolution::Denied(_)));
    }

    #[test]
    fn resolve_identity_invalid_role_uses_default() {
        let result = resolve_identity("actor-001", "/path", "hacker", "development", false);
        // 未知角色兜底 TemporaryAgent（内核对该角色 allow 空、deny *，fail-safe 零权限）
        match result {
            IdentityResolution::Resolved(snapshot) => {
                assert_eq!(snapshot.role_ref.kind, RoleKind::TemporaryAgent);
                assert_eq!(
                    snapshot.permission_snapshot_ref.profile_id,
                    "profile:temporary_agent:development",
                    "未知角色必须挂到 deny-all 的 temporary_agent profile"
                );
            }
            other => panic!("未知角色应走零权限兜底而不是拒绝：{other:?}"),
        }
    }

    #[test]
    fn resolve_identity_invalid_channel_uses_default() {
        let result = resolve_identity("actor-001", "/path", "worker", "invalid_channel", false);
        // 未知通道兜底 Daily=ReadOnly（fail-safe），不得落到 WriteLocal
        match result {
            IdentityResolution::Resolved(snapshot) => {
                assert_eq!(snapshot.execution_channel.kind, ChannelKind::Daily);
                assert_eq!(
                    snapshot.execution_channel.side_effect_mode,
                    SideEffectMode::ReadOnly,
                    "未知通道必须只读，不给写权限"
                );
            }
            other => panic!("未知通道应走只读兜底而不是拒绝：{other:?}"),
        }
    }

    // ---- M4C02 primary Secretary identity ----

    #[test]
    fn m4_primary_secretary_resolver_is_server_fixed_and_contract_exact() {
        let primary = resolve_m4_primary_secretary_identity().expect("fixed Secretary identity");
        let snapshot = &primary.identity_snapshot;
        let current_object = snapshot
            .current_object_ref
            .as_ref()
            .expect("fixed Secretary current object");

        assert_eq!(primary.profile_id, M4_PRIMARY_SECRETARY_PROFILE_ID);
        assert_eq!(snapshot.actor_id.0, M4_PRIMARY_SECRETARY_ACTOR_ID);
        assert_eq!(snapshot.role_ref.role_id, M4_PRIMARY_SECRETARY_ROLE_ID);
        assert_eq!(snapshot.role_ref.kind, RoleKind::Secretary);
        assert_eq!(
            snapshot.role_ref.revision,
            M4_PRIMARY_SECRETARY_ROLE_REVISION
        );
        assert_eq!(snapshot.scope_ref.kind, ScopeKind::Personal);
        assert_eq!(snapshot.scope_ref.scope_id, M4_PRIMARY_SECRETARY_SCOPE_ID);
        assert_eq!(
            snapshot.scope_ref.revision,
            M4_PRIMARY_SECRETARY_SCOPE_REVISION
        );
        assert_eq!(
            current_object.object_type,
            M4_PRIMARY_SECRETARY_CURRENT_OBJECT_TYPE
        );
        assert_eq!(
            current_object.object_id,
            M4_PRIMARY_SECRETARY_CURRENT_OBJECT_ID
        );
        assert_eq!(
            current_object.source_owner_ref,
            M4_PRIMARY_SECRETARY_CURRENT_OBJECT_SOURCE_OWNER_REF
        );
        assert_eq!(current_object.scope_ref, snapshot.scope_ref);
        assert_eq!(
            current_object.binding_revision,
            M4_PRIMARY_SECRETARY_CURRENT_OBJECT_BINDING_REVISION
        );
        assert_eq!(
            current_object.binding_source_ref,
            M4_PRIMARY_SECRETARY_CURRENT_OBJECT_BINDING_SOURCE_REF
        );
        assert_eq!(snapshot.execution_channel.kind, ChannelKind::Daily);
        assert_eq!(snapshot.execution_channel.risk_class, RiskClass::Low);
        assert_eq!(
            snapshot.execution_channel.side_effect_mode,
            SideEffectMode::WriteLocal
        );
        assert_eq!(
            primary.permission_profile.profile_id,
            M4_PRIMARY_SECRETARY_PERMISSION_PROFILE_ID
        );
        assert_eq!(
            primary.permission_profile.allow_capabilities,
            string_values(M4_PRIMARY_SECRETARY_ALLOW_CAPABILITIES)
        );
        assert_eq!(
            primary.permission_profile.deny_capabilities,
            string_values(M4_PRIMARY_SECRETARY_DENY_CAPABILITIES)
        );
        assert_eq!(
            primary.permission_profile.constraints,
            string_values(M4_PRIMARY_SECRETARY_CONSTRAINTS)
        );
        assert!(verify_m4_primary_secretary_identity(&primary).is_ok());
    }

    #[test]
    fn m4_primary_secretary_permission_snapshot_is_stable_and_complete() {
        let first = resolve_m4_primary_secretary_identity().expect("first fixed resolution");
        let second = resolve_m4_primary_secretary_identity().expect("second fixed resolution");
        let permission_snapshot = &first.identity_snapshot.permission_snapshot_ref;

        assert_eq!(first, second, "restart must not rotate the v1 snapshot");
        assert_eq!(
            permission_snapshot.snapshot_id,
            M4_PRIMARY_SECRETARY_PERMISSION_SNAPSHOT_ID
        );
        assert_eq!(
            permission_snapshot.profile_id,
            M4_PRIMARY_SECRETARY_PERMISSION_PROFILE_ID
        );
        assert_eq!(
            permission_snapshot.actor_id.0,
            M4_PRIMARY_SECRETARY_ACTOR_ID
        );
        assert_eq!(permission_snapshot.scope_ref.kind, ScopeKind::Personal);
        assert_eq!(
            permission_snapshot.scope_ref.scope_id,
            M4_PRIMARY_SECRETARY_SCOPE_ID
        );
        assert_eq!(
            permission_snapshot.execution_channel,
            first.identity_snapshot.execution_channel
        );
        assert_eq!(
            permission_snapshot.revision,
            M4_PRIMARY_SECRETARY_PERMISSION_SNAPSHOT_REVISION
        );
        assert_eq!(
            permission_snapshot.issued_at,
            M4_PRIMARY_SECRETARY_PERMISSION_SNAPSHOT_ISSUED_AT
        );
        assert_eq!(
            permission_snapshot.snapshot_hash,
            "2c022cc84caf83d5bb4a4402fbc1492023ce44c5b4954b4685bee2e6f8b61edd"
        );
    }

    #[test]
    fn m4_primary_secretary_owner_fingerprint_is_m3_byte_compatible() {
        let primary = resolve_m4_primary_secretary_identity().expect("fixed Secretary identity");
        let m3_fields = &primary.m3_binding;
        let expected = crate::m3_role_session::owner_fingerprint_for_components(
            &m3_fields.actor_id,
            &m3_fields.role_ref,
            &m3_fields.scope_ref,
            &m3_fields.current_object_ref,
            &m3_fields.execution_channel,
        )
        .expect("M3 accepts fixed server components");
        let binding = primary
            .m3_server_resolved_binding()
            .expect("M4C02 bridge returns an M3 binding");

        assert_eq!(
            M4_PRIMARY_SECRETARY_M3_OWNER_FINGERPRINT_DOMAIN_SEPARATOR,
            crate::m3_role_session::OWNER_FINGERPRINT_DOMAIN_SEPARATOR
        );
        assert_m3_opaque_reference_envelope(
            &m3_fields.actor_id,
            M4_PRIMARY_SECRETARY_M3_ACTOR_NAMESPACE,
        );
        assert_m3_opaque_reference_envelope(
            &m3_fields.role_ref,
            M4_PRIMARY_SECRETARY_M3_ROLE_NAMESPACE,
        );
        assert_m3_opaque_reference_envelope(
            &m3_fields.scope_ref,
            M4_PRIMARY_SECRETARY_M3_SCOPE_NAMESPACE,
        );
        assert_m3_opaque_reference_envelope(
            &m3_fields.current_object_ref,
            M4_PRIMARY_SECRETARY_M3_OBJECT_NAMESPACE,
        );
        assert_m3_opaque_reference_envelope(
            &m3_fields.execution_channel,
            M4_PRIMARY_SECRETARY_M3_CHANNEL_NAMESPACE,
        );
        assert_m3_opaque_reference_envelope(
            &m3_fields.permission_snapshot_ref,
            M4_PRIMARY_SECRETARY_M3_PERMISSION_NAMESPACE,
        );
        assert_eq!(
            m3_fields.actor_id,
            "actor:sha256:0ca883943d2c2b3b1b0b3c389c3f584fc5e7f7628a23be92347394a47df849bd"
        );
        assert_eq!(
            m3_fields.role_ref,
            "role:sha256:393b9dd73277836635d410947694c056575c2ebe626ca39ead1f2d9a8d77d84e"
        );
        assert_eq!(
            m3_fields.scope_ref,
            "scope:sha256:1a12d716470dbd77319d37a2903a081a3ce32b39256d7970baf95433d567e935"
        );
        assert_eq!(
            m3_fields.current_object_ref,
            "object:sha256:fbfa0ae5e00b534a077598db657714ecc4020849c37ff94599b316aff94e2194"
        );
        assert_eq!(
            m3_fields.execution_channel,
            "channel:sha256:2a231639cbcc6dc0a1e2bf6b9d8484ee5d1a5e10ab377aa2f8a2bc3f4e7d476a"
        );
        assert_eq!(
            m3_fields.permission_snapshot_ref,
            "permission:sha256:4115691728fad2a8a7b3e4010ae00bd26cb61835bdc2df5b7efbe370d3fe9d89"
        );
        assert_eq!(
            m3_fields.owner_fingerprint,
            "aafa17796201adde09b86db5139152490c360088476f4ae6b2e3f8a6bd6e4904"
        );
        assert_eq!(m3_fields.owner_fingerprint, expected.as_str());
        assert_eq!(
            primary.identity_snapshot.owner_fingerprint,
            expected.as_str()
        );
        assert_eq!(binding.owner_fingerprint.as_str(), expected.as_str());
        assert_eq!(binding.actor_id.as_str(), m3_fields.actor_id);
        assert_eq!(binding.role_ref.as_str(), m3_fields.role_ref);
        assert_eq!(binding.scope_ref.as_str(), m3_fields.scope_ref);
        assert_eq!(
            binding.current_object_ref.as_str(),
            m3_fields.current_object_ref
        );
        assert_eq!(
            binding.execution_channel.as_str(),
            m3_fields.execution_channel
        );
        assert_eq!(
            binding.permission_snapshot_ref.as_str(),
            m3_fields.permission_snapshot_ref
        );
    }

    #[test]
    fn m4_primary_secretary_identity_drift_fails_closed() {
        let mut cross_scope =
            resolve_m4_primary_secretary_identity().expect("fixed Secretary identity");
        cross_scope.identity_snapshot.scope_ref = ScopeRef {
            kind: ScopeKind::Project,
            scope_id: "project:foreign".to_string(),
            revision: 1,
        };

        assert_eq!(
            verify_m4_primary_secretary_identity(&cross_scope),
            Err(M4PrimarySecretaryIdentityError::ContractInvariantMismatch)
        );
        assert!(cross_scope.m3_server_resolved_binding().is_err());
    }

    #[test]
    fn generic_project_resolver_remains_separate_from_m4_primary_secretary() {
        let primary = resolve_m4_primary_secretary_identity().expect("fixed Secretary identity");
        let generic = resolve_identity(
            M4_PRIMARY_SECRETARY_ACTOR_ID,
            "/project/that-must-not-become-personal-scope",
            "secretary",
            "daily",
            true,
        );
        let generic = match generic {
            IdentityResolution::Resolved(snapshot) => snapshot,
            other => panic!("generic resolver unexpectedly denied: {other:?}"),
        };

        assert_eq!(generic.scope_ref.kind, ScopeKind::Project);
        assert_eq!(generic.current_object_ref, None);
        assert_eq!(generic.execution_channel.kind, ChannelKind::Daily);
        assert_eq!(
            generic.execution_channel.side_effect_mode,
            SideEffectMode::ReadOnly
        );
        assert_ne!(
            generic.owner_fingerprint,
            primary.identity_snapshot.owner_fingerprint
        );
    }

    // ---- policy_check_capability ----

    #[test]
    fn policy_allow_capability() {
        let profile = PermissionProfile {
            profile_id: "test".to_string(),
            allow_capabilities: vec!["read_project".to_string()],
            deny_capabilities: vec![],
            constraints: vec![],
            revision: 1,
        };
        assert_eq!(
            policy_check_capability(&profile, "read_project"),
            PolicyDecision::Allowed
        );
    }

    #[test]
    fn policy_deny_capability_explicit() {
        let profile = PermissionProfile {
            profile_id: "test".to_string(),
            allow_capabilities: vec!["*".to_string()],
            deny_capabilities: vec!["execute_code".to_string()],
            constraints: vec![],
            revision: 1,
        };
        assert!(matches!(
            policy_check_capability(&profile, "execute_code"),
            PolicyDecision::Denied(_)
        ));
    }

    #[test]
    fn policy_deny_capability_not_in_allow() {
        let profile = PermissionProfile {
            profile_id: "test".to_string(),
            allow_capabilities: vec!["read_project".to_string()],
            deny_capabilities: vec![],
            constraints: vec![],
            revision: 1,
        };
        assert!(matches!(
            policy_check_capability(&profile, "execute_code"),
            PolicyDecision::Denied(_)
        ));
    }

    #[test]
    fn policy_deny_wildcard() {
        let profile = PermissionProfile {
            profile_id: "test".to_string(),
            allow_capabilities: vec![],
            deny_capabilities: vec!["*".to_string()],
            constraints: vec![],
            revision: 1,
        };
        assert!(matches!(
            policy_check_capability(&profile, "anything"),
            PolicyDecision::Denied(_)
        ));
    }

    // ---- validate_session_permission_integrity ----

    #[test]
    fn session_permission_integrity_ok() {
        let snapshot = resolve_identity("worker-001", "/path", "worker", "development", false);
        if let IdentityResolution::Resolved(s) = snapshot {
            assert!(validate_session_permission_integrity(
                &s.permission_snapshot_ref,
                "worker",
                "development",
            )
            .is_ok());
        }
    }

    #[test]
    fn session_permission_drift_detected() {
        let snapshot = resolve_identity("worker-001", "/path", "worker", "development", false);
        if let IdentityResolution::Resolved(s) = snapshot {
            // Try to claim a different role
            let result = validate_session_permission_integrity(
                &s.permission_snapshot_ref,
                "user", // different role
                "development",
            );
            assert!(result.is_err());
        }
    }

    // ---- stable_id ----

    #[test]
    fn stable_id_normalizes() {
        assert_eq!(
            stable_id("/Users/yoyi/Documents/mario test"),
            "users-yoyi-documents-mario-test"
        );
        assert_eq!(stable_id("UPPER"), "upper");
        assert_eq!(stable_id("a--b"), "a-b");
    }
}
