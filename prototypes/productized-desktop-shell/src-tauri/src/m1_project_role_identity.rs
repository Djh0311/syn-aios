//! Server-only project-role identity authority for M1I01.
//!
//! This module mints and returns immutable project / role / actor / scope /
//! current-object / channel / permission / owner-fingerprint snapshots. It does
//! not provision M3 RoleSession records, issue ExecutionGrant, or expose a
//! renderer surface.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;
use uuid::Uuid;

pub(crate) const M1_PROJECT_ROLE_IDENTITY_VERSION: &str = "m1.project-role-identity.v1";
pub(crate) const M1_NO_CAPABILITY_PROFILE_ID: &str =
    "permission:m1-project-role-session-no-capability:v1";
pub(crate) const M1_NO_CAPABILITY_PROFILE_REVISION: u64 = 1;
pub(crate) const M1_PERMISSION_SNAPSHOT_DOMAIN: &str =
    "syn.m1.project-index.permission-snapshot/v1";
pub(crate) const M1_OWNER_FINGERPRINT_DOMAIN: &str = "syn.m1.project-index.owner-fingerprint/v1";
const M1_NO_CAPABILITY_DENY: &[&str] = &[
    "execute",
    "issue_execution_grant",
    "write_project_fact",
    "write_authorization",
    "write_workflow_state",
    "spawn_provider",
    "use_external_connector",
    "read_or_write_credential",
    "send_external_message",
];
const M1_NO_CAPABILITY_CONSTRAINTS: &[&str] = &[
    "session_identity_only",
    "no_execution_grant",
    "least_privilege_no_capability",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum M1ProjectRole {
    ProjectSupervisor,
    Worker,
    IndependentReviewer,
}

impl M1ProjectRole {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::ProjectSupervisor => "project_supervisor",
            Self::Worker => "worker",
            Self::IndependentReviewer => "independent_reviewer",
        }
    }

    pub(crate) fn all() -> [Self; 3] {
        [
            Self::ProjectSupervisor,
            Self::Worker,
            Self::IndependentReviewer,
        ]
    }
}

impl fmt::Display for M1ProjectRole {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct M1ScopeRef {
    pub(crate) scope_kind: String,
    pub(crate) scope_id: String,
    pub(crate) scope_revision: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct M1CurrentObjectRef {
    pub(crate) object_type: String,
    pub(crate) object_id: String,
    pub(crate) source_owner_ref: String,
    pub(crate) scope_ref: M1ScopeRef,
    pub(crate) binding_revision: u64,
    pub(crate) binding_source_ref: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct M1ExecutionChannel {
    pub(crate) channel_kind: String,
    pub(crate) risk_class: String,
    pub(crate) side_effect_mode: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct M1PermissionSnapshot {
    pub(crate) snapshot_id: String,
    pub(crate) profile_id: String,
    pub(crate) actor_id: String,
    pub(crate) scope_ref: M1ScopeRef,
    pub(crate) execution_channel: M1ExecutionChannel,
    pub(crate) revision: u64,
    pub(crate) snapshot_hash: String,
    pub(crate) issued_at: String,
    pub(crate) allow_capabilities: Vec<String>,
    pub(crate) deny_capabilities: Vec<String>,
    pub(crate) constraints: Vec<String>,
    pub(crate) profile_revision: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct M1IdentityRevisions {
    pub(crate) registry_revision: u64,
    pub(crate) project_revision: u64,
    pub(crate) identity_revision: u64,
    pub(crate) permission_revision: u64,
    pub(crate) binding_revision: u64,
    pub(crate) resolver_revision: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M1ProjectRoleIdentitySnapshot {
    pub(crate) project_id: String,
    pub(crate) role: M1ProjectRole,
    pub(crate) actor_id: String,
    pub(crate) session_identity_id: String,
    pub(crate) scope: M1ScopeRef,
    pub(crate) current_object: M1CurrentObjectRef,
    pub(crate) channel: M1ExecutionChannel,
    pub(crate) permission_snapshot: M1PermissionSnapshot,
    pub(crate) owner_fingerprint: String,
    pub(crate) revisions: M1IdentityRevisions,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct M1StoredRoleIdentity {
    pub(crate) role: M1ProjectRole,
    pub(crate) role_id: String,
    pub(crate) role_revision: u64,
    pub(crate) actor_id: String,
    pub(crate) session_identity_id: String,
    pub(crate) permission_snapshot_id: String,
    pub(crate) permission_revision: u64,
    pub(crate) snapshot_hash: String,
    pub(crate) owner_fingerprint: String,
    pub(crate) identity_revision: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M1ProjectIdentityContext {
    pub(crate) project_id: String,
    pub(crate) scope_id: String,
    pub(crate) scope_revision: u64,
    pub(crate) current_object_id: String,
    pub(crate) binding_revision: u64,
    pub(crate) issued_at: String,
    pub(crate) registry_revision: u64,
    pub(crate) project_revision: u64,
    pub(crate) resolver_revision: u64,
}

pub(crate) fn mint_prefixed_uuid(prefix: &str) -> String {
    format!("{prefix}{}", Uuid::new_v4())
}

pub(crate) fn no_capability_channel() -> M1ExecutionChannel {
    M1ExecutionChannel {
        channel_kind: "development".to_string(),
        risk_class: "low".to_string(),
        side_effect_mode: "read_only".to_string(),
    }
}

pub(crate) fn mint_project_role_identities(
    context: &M1ProjectIdentityContext,
) -> Result<[M1StoredRoleIdentity; 3], String> {
    let supervisor = mint_one_role(context, M1ProjectRole::ProjectSupervisor)?;
    let worker = mint_one_role(context, M1ProjectRole::Worker)?;
    let reviewer = mint_one_role(context, M1ProjectRole::IndependentReviewer)?;
    assert_distinct_role_identities(&supervisor, &worker, &reviewer)?;
    Ok([supervisor, worker, reviewer])
}

pub(crate) fn project_role_identity_snapshot(
    context: &M1ProjectIdentityContext,
    stored: &M1StoredRoleIdentity,
) -> Result<M1ProjectRoleIdentitySnapshot, String> {
    let expected_hash = permission_snapshot_hash(context, stored)?;
    if stored.snapshot_hash != expected_hash {
        return Err("m1_project_role_identity_permission_drift".to_string());
    }
    let expected_fingerprint = owner_fingerprint(context, stored, &expected_hash)?;
    if stored.owner_fingerprint != expected_fingerprint {
        return Err("m1_project_role_identity_fingerprint_drift".to_string());
    }
    let scope = project_scope(context);
    let channel = no_capability_channel();
    Ok(M1ProjectRoleIdentitySnapshot {
        project_id: context.project_id.clone(),
        role: stored.role,
        actor_id: stored.actor_id.clone(),
        session_identity_id: stored.session_identity_id.clone(),
        scope: scope.clone(),
        current_object: M1CurrentObjectRef {
            object_type: "project".to_string(),
            object_id: context.current_object_id.clone(),
            source_owner_ref: "project_index".to_string(),
            scope_ref: scope.clone(),
            binding_revision: context.binding_revision,
            binding_source_ref: M1_PROJECT_ROLE_IDENTITY_VERSION.to_string(),
        },
        channel: channel.clone(),
        permission_snapshot: M1PermissionSnapshot {
            snapshot_id: stored.permission_snapshot_id.clone(),
            profile_id: M1_NO_CAPABILITY_PROFILE_ID.to_string(),
            actor_id: stored.actor_id.clone(),
            scope_ref: scope,
            execution_channel: channel,
            revision: stored.permission_revision,
            snapshot_hash: stored.snapshot_hash.clone(),
            issued_at: context.issued_at.clone(),
            allow_capabilities: Vec::new(),
            deny_capabilities: string_values(M1_NO_CAPABILITY_DENY),
            constraints: string_values(M1_NO_CAPABILITY_CONSTRAINTS),
            profile_revision: M1_NO_CAPABILITY_PROFILE_REVISION,
        },
        owner_fingerprint: stored.owner_fingerprint.clone(),
        revisions: M1IdentityRevisions {
            registry_revision: context.registry_revision,
            project_revision: context.project_revision,
            identity_revision: stored.identity_revision,
            permission_revision: stored.permission_revision,
            binding_revision: context.binding_revision,
            resolver_revision: context.resolver_revision,
        },
    })
}

fn mint_one_role(
    context: &M1ProjectIdentityContext,
    role: M1ProjectRole,
) -> Result<M1StoredRoleIdentity, String> {
    let mut stored = M1StoredRoleIdentity {
        role,
        role_id: mint_prefixed_uuid("role:"),
        role_revision: 1,
        actor_id: mint_prefixed_uuid("actor:"),
        session_identity_id: mint_prefixed_uuid("session-identity:"),
        permission_snapshot_id: mint_prefixed_uuid("permission-snapshot:"),
        permission_revision: 1,
        snapshot_hash: String::new(),
        owner_fingerprint: String::new(),
        identity_revision: 1,
    };
    stored.snapshot_hash = permission_snapshot_hash(context, &stored)?;
    stored.owner_fingerprint = owner_fingerprint(context, &stored, &stored.snapshot_hash)?;
    Ok(stored)
}

fn project_scope(context: &M1ProjectIdentityContext) -> M1ScopeRef {
    M1ScopeRef {
        scope_kind: "project".to_string(),
        scope_id: context.scope_id.clone(),
        scope_revision: context.scope_revision,
    }
}

fn permission_snapshot_hash(
    context: &M1ProjectIdentityContext,
    stored: &M1StoredRoleIdentity,
) -> Result<String, String> {
    let channel = no_capability_channel();
    Ok(domain_digest(
        M1_PERMISSION_SNAPSHOT_DOMAIN,
        &[
            stored.permission_snapshot_id.as_str(),
            M1_NO_CAPABILITY_PROFILE_ID,
            stored.actor_id.as_str(),
            context.scope_id.as_str(),
            channel.channel_kind.as_str(),
            channel.risk_class.as_str(),
            channel.side_effect_mode.as_str(),
            &stored.permission_revision.to_string(),
            context.issued_at.as_str(),
        ],
    ))
}

fn owner_fingerprint(
    context: &M1ProjectIdentityContext,
    stored: &M1StoredRoleIdentity,
    snapshot_hash: &str,
) -> Result<String, String> {
    Ok(domain_digest(
        M1_OWNER_FINGERPRINT_DOMAIN,
        &[
            context.project_id.as_str(),
            stored.role.as_str(),
            stored.role_id.as_str(),
            stored.actor_id.as_str(),
            stored.session_identity_id.as_str(),
            context.scope_id.as_str(),
            context.current_object_id.as_str(),
            snapshot_hash,
        ],
    ))
}

fn assert_distinct_role_identities(
    supervisor: &M1StoredRoleIdentity,
    worker: &M1StoredRoleIdentity,
    reviewer: &M1StoredRoleIdentity,
) -> Result<(), String> {
    let records = [supervisor, worker, reviewer];
    for left in 0..records.len() {
        for right in (left + 1)..records.len() {
            if records[left].actor_id == records[right].actor_id
                || records[left].session_identity_id == records[right].session_identity_id
                || records[left].role_id == records[right].role_id
                || records[left].permission_snapshot_id == records[right].permission_snapshot_id
                || records[left].owner_fingerprint == records[right].owner_fingerprint
            {
                return Err("m1_project_role_identity_not_distinct".to_string());
            }
        }
    }
    if reviewer.role != M1ProjectRole::IndependentReviewer
        || supervisor.role == reviewer.role
        || worker.role == reviewer.role
    {
        return Err("m1_project_role_identity_reviewer_not_distinct".to_string());
    }
    Ok(())
}

fn domain_digest(domain: &str, parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(domain.as_bytes());
    hasher.update([0u8]);
    for part in parts {
        let len = u32::try_from(part.len()).unwrap_or(u32::MAX).to_be_bytes();
        hasher.update(len);
        hasher.update(part.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

fn string_values(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_context() -> M1ProjectIdentityContext {
        M1ProjectIdentityContext {
            project_id: "project:11111111-1111-4111-8111-111111111111".to_string(),
            scope_id: "scope:22222222-2222-4222-8222-222222222222".to_string(),
            scope_revision: 1,
            current_object_id: "object:33333333-3333-4333-8333-333333333333".to_string(),
            binding_revision: 1,
            issued_at: "2026-08-17T00:00:00Z".to_string(),
            registry_revision: 1,
            project_revision: 1,
            resolver_revision: 1,
        }
    }

    #[test]
    fn m1_project_role_identity_mints_three_distinct_no_capability_roles() {
        let context = sample_context();
        let minted = mint_project_role_identities(&context).expect("mint roles");
        assert_eq!(minted[0].role, M1ProjectRole::ProjectSupervisor);
        assert_eq!(minted[1].role, M1ProjectRole::Worker);
        assert_eq!(minted[2].role, M1ProjectRole::IndependentReviewer);

        let snapshots = minted
            .iter()
            .map(|stored| project_role_identity_snapshot(&context, stored).expect("snapshot"))
            .collect::<Vec<_>>();
        assert_ne!(snapshots[0].actor_id, snapshots[1].actor_id);
        assert_ne!(snapshots[0].actor_id, snapshots[2].actor_id);
        assert_ne!(snapshots[1].actor_id, snapshots[2].actor_id);
        assert_ne!(
            snapshots[0].session_identity_id,
            snapshots[2].session_identity_id
        );
        assert_ne!(
            snapshots[0].owner_fingerprint,
            snapshots[2].owner_fingerprint
        );
        for snapshot in &snapshots {
            assert!(snapshot.permission_snapshot.allow_capabilities.is_empty());
            assert_eq!(
                snapshot.permission_snapshot.profile_id,
                M1_NO_CAPABILITY_PROFILE_ID
            );
            assert_eq!(snapshot.channel.side_effect_mode, "read_only");
            assert_eq!(snapshot.scope.scope_kind, "project");
            assert_eq!(snapshot.current_object.source_owner_ref, "project_index");
            assert!(!snapshot.owner_fingerprint.is_empty());
            assert_eq!(snapshot.revisions.identity_revision, 1);
        }
    }
}
