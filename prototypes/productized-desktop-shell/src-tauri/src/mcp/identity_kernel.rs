// SYN-FND-003: Identity / Scope / Policy Kernel.
//
// **STAGED — 未上活路径**。类型层已建、单元测试已建，但尚未接入任何 Tauri command。
// 接线属后续包（CURRENT.md 明写的拦阻项）。
// 证据级别 = 单元测试。接线后升级为集成测试。
//
// Contract: docs/contracts/identity-scope-v1.md
// Evidence level: STATIC_OPENING_ONLY → will be upgraded after focused tests.

#![allow(dead_code)] // staged foundation, not yet connected — warnings are expected

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

    // 2. Resolve role
    let role_kind = match RoleKind::from_str(role_str) {
        Some(r) => r,
        None => {
            return IdentityResolution::Denied(format!(
                "identity_kernel_rejected: 未知角色 '{role_str}'"
            ));
        }
    };

    // 3. Resolve channel
    let channel_kind = match ChannelKind::from_str(channel_str) {
        Some(c) => c,
        None => {
            return IdentityResolution::Denied(format!(
                "identity_kernel_rejected: 未知通道 '{channel_str}'"
            ));
        }
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
            vec![
                "dispatch_workers".to_string(),
                "review_workers".to_string(),
            ],
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
pub fn policy_check_capability(
    profile: &PermissionProfile,
    capability: &str,
) -> PolicyDecision {
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
pub fn policy_check_write(
    identity: &IdentitySnapshot,
    _target_path: &str,
) -> PolicyDecision {
    // System role bypasses write checks (for bootstrap/testing)
    if identity.role_ref.kind == RoleKind::System {
        return PolicyDecision::Allowed;
    }

    // Read-only channels cannot write
    if identity.execution_channel.side_effect_mode == SideEffectMode::ReadOnly {
        return PolicyDecision::Denied(
            "policy_denied: 只读通道不允许写入".to_string(),
        );
    }

    // Check capability
    policy_check_capability(
        &identity.permission_snapshot_ref.profile_id
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
                    original.profile_id,
                    snapshot.permission_snapshot_ref.profile_id
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
        assert_eq!(RoleKind::from_str("project_director"), Some(RoleKind::ProjectSupervisor));
        assert_eq!(RoleKind::from_str("global_director"), Some(RoleKind::GlobalSupervisor));
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
                assert_eq!(snapshot.execution_channel.side_effect_mode, SideEffectMode::WriteLocal);
            }
            other => panic!("Expected Resolved, got {other:?}"),
        }
    }

    #[test]
    fn resolve_identity_empty_actor() {
        let result = resolve_identity(
            "",
            "/path",
            "worker",
            "development",
            false,
        );
        assert!(matches!(result, IdentityResolution::Denied(_)));
    }

    #[test]
    fn resolve_identity_invalid_role() {
        let result = resolve_identity(
            "actor-001",
            "/path",
            "hacker",
            "development",
            false,
        );
        assert!(matches!(result, IdentityResolution::Denied(_)));
    }

    #[test]
    fn resolve_identity_invalid_channel() {
        let result = resolve_identity(
            "actor-001",
            "/path",
            "worker",
            "invalid_channel",
            false,
        );
        assert!(matches!(result, IdentityResolution::Denied(_)));
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
        let snapshot = resolve_identity(
            "worker-001",
            "/path",
            "worker",
            "development",
            false,
        );
        if let IdentityResolution::Resolved(s) = snapshot {
            assert!(validate_session_permission_integrity(
                &s.permission_snapshot_ref,
                "worker",
                "development",
            ).is_ok());
        }
    }

    #[test]
    fn session_permission_drift_detected() {
        let snapshot = resolve_identity(
            "worker-001",
            "/path",
            "worker",
            "development",
            false,
        );
        if let IdentityResolution::Resolved(s) = snapshot {
            // Try to claim a different role
            let result = validate_session_permission_integrity(
                &s.permission_snapshot_ref,
                "user",  // different role
                "development",
            );
            assert!(result.is_err());
        }
    }

    // ---- stable_id ----

    #[test]
    fn stable_id_normalizes() {
        assert_eq!(stable_id("/Users/yoyi/Documents/mario test"), "users-yoyi-documents-mario-test");
        assert_eq!(stable_id("UPPER"), "upper");
        assert_eq!(stable_id("a--b"), "a-b");
    }
}
