//! M6D02 ordinary-product Global Supervisor RoleSession.
//!
//! One server-fixed global-scope RoleSession is created or restored through
//! the M3 repository already installed by ordinary M4/M3 composition. This
//! module does not own a parallel RoleSession store, schema, or in-memory
//! identity registry. Identity is the canonical server-owned actor / role /
//! GLOBAL scope / current-object / execution-channel / permission refs; it is
//! never derived from cwd, project path, display name, provider, model,
//! thread, process, renderer input, env, or fixture.

use crate::m3_role_session::{
    CorrelationId, OpaqueRef, RequestIdempotencyKey, RoleSession, RoleSessionId, RoleSessionState,
    ServerResolvedBinding, Sha256Digest,
};
use crate::m3_role_session_repository::{
    CreateRoleSessionCommand, M3CommandMetadata, M3ReadPermissionDisposition,
    M3RoleSessionDirectoryQuery, M3RoleSessionSnapshotQuery, M3RoleSessionSqliteRepository,
    QuarantineRoleSessionCommand,
};
use serde::Serialize;

pub(crate) const M6_ORG_GLOBAL_SCOPE_KIND: &str = "GLOBAL";
pub(crate) const M6_ORG_GLOBAL_ROLE_SESSION_UNAVAILABLE: &str =
    "m6_org_global_role_session_unavailable";
pub(crate) const M6_ORG_GLOBAL_ROLE_SESSION_AMBIGUOUS: &str =
    "m6_org_global_role_session_ambiguous";
pub(crate) const M6_ORG_GLOBAL_ROLE_SESSION_MISMATCHED: &str =
    "m6_org_global_role_session_mismatched";
pub(crate) const M6_ORG_GLOBAL_ROLE_SESSION_QUARANTINED: &str =
    "m6_org_global_role_session_quarantined";
pub(crate) const M6_ORG_GLOBAL_ROLE_SESSION_CLOSED: &str = "m6_org_global_role_session_closed";
pub(crate) const M6_ORG_GLOBAL_ROLE_SESSION_INCOMPLETE: &str =
    "m6_org_global_role_session_incomplete";
pub(crate) const M6_ORG_GLOBAL_ROLE_SESSION_MISSING_AFTER_ESTABLISHED: &str =
    "m6_org_global_role_session_missing_after_established";
pub(crate) const M6_ORG_GLOBAL_ROLE_SESSION_CORRUPT: &str = "m6_org_global_role_session_corrupt";
pub(crate) const M6_ORG_GLOBAL_ROLE_SESSION_BINDING_REJECTED: &str =
    "m6_org_global_role_session_binding_rejected";
pub(crate) const M6_ORG_GLOBAL_ROLE_SESSION_PROJECT_WRITE_REJECTED: &str =
    "m6_org_global_role_session_project_write_rejected";
pub(crate) const M6_ORG_GLOBAL_ROLE_SESSION_SOURCE_UNAVAILABLE: &str =
    "m6_org_global_role_session_source_unavailable";

const M6_ORG_GLOBAL_ACTOR_MATERIAL: &str =
    "syn.m6.org.global-supervisor.actor/organization-primary/v1";
const M6_ORG_GLOBAL_ROLE_MATERIAL: &str =
    "syn.m6.org.global-supervisor.role/global_supervisor/organization-primary/v1";
const M6_ORG_GLOBAL_SCOPE_MATERIAL: &str =
    "syn.m6.org.global-supervisor.scope/GLOBAL/organization-primary/v1";
const M6_ORG_GLOBAL_OBJECT_MATERIAL: &str =
    "syn.m6.org.global-supervisor.object/organization-workbench/v1";
const M6_ORG_GLOBAL_CHANNEL_MATERIAL: &str =
    "syn.m6.org.global-supervisor.channel/organization-read/v1";
const M6_ORG_GLOBAL_PERMISSION_MATERIAL: &str =
    "syn.m6.org.global-supervisor.permission/read-only-global/v1";
const M6_ORG_GLOBAL_SESSION_ID_MATERIAL: &str =
    "syn.m6.org.global-supervisor.role-session/organization-primary/v1";
const M6_ORG_GLOBAL_CREATE_MATERIAL: &str =
    "syn.m6.org.global-supervisor.create/organization-primary/v1";

/// Minimal Global Supervisor context. Only summary refs and source refs are
/// representable; raw files, summaries, transcripts, secrets, untrimmed
/// memory, provider responses, prompts, stdout/stderr, and tool output are
/// not fields of this type.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct M6OrgGlobalRoleSessionContextDto {
    pub(crate) summary_refs: Vec<String>,
    pub(crate) source_refs: Vec<String>,
}

impl M6OrgGlobalRoleSessionContextDto {
    fn minimal(
        mut summary_refs: Vec<String>,
        mut source_refs: Vec<String>,
    ) -> Result<Self, String> {
        summary_refs.retain(|value| !value.trim().is_empty());
        source_refs.retain(|value| !value.trim().is_empty());
        summary_refs.sort();
        summary_refs.dedup();
        source_refs.sort();
        source_refs.dedup();
        if summary_refs.is_empty() || source_refs.is_empty() {
            return Err(M6_ORG_GLOBAL_ROLE_SESSION_INCOMPLETE.to_string());
        }
        Ok(Self {
            summary_refs,
            source_refs,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M6OrgGlobalSummaryConsumerLease {
    pub(crate) role_session_id: String,
    pub(crate) role_session_revision: u64,
    pub(crate) scope_kind: String,
    pub(crate) consumer_expires_at_ms: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "availability", rename_all = "snake_case")]
pub(crate) enum M6OrgGlobalRoleSessionStatusDto {
    Ready {
        role_session_id: String,
        revision: u64,
        state: String,
        scope_kind: String,
        read_only: bool,
        project_write_capability: bool,
        provider_handle_authorizes: bool,
        context: M6OrgGlobalRoleSessionContextDto,
    },
    Unavailable {
        error: String,
    },
}

impl M6OrgGlobalRoleSessionStatusDto {
    fn ready(
        session: &RoleSession,
        slot: &M6OrgGlobalRoleSessionSlot,
        context: M6OrgGlobalRoleSessionContextDto,
    ) -> Self {
        Self::Ready {
            role_session_id: session.role_session_id.as_str().to_string(),
            revision: session.revision,
            state: session.status.as_str().to_string(),
            scope_kind: M6_ORG_GLOBAL_SCOPE_KIND.to_string(),
            read_only: true,
            project_write_capability: authorize_attempted_project_write(slot).is_ok(),
            provider_handle_authorizes: false,
            context,
        }
    }

    fn unavailable(error: impl Into<String>) -> Self {
        Self::Unavailable {
            error: error.into(),
        }
    }
}

#[derive(Clone)]
struct M6OrgGlobalRoleSessionRuntime {
    repository: M3RoleSessionSqliteRepository,
    binding: ServerResolvedBinding,
    role_session_id: RoleSessionId,
}

#[derive(Clone)]
pub(crate) struct M6OrgGlobalRoleSessionAuthoritySeed {
    pub(crate) repository: M3RoleSessionSqliteRepository,
    pub(crate) binding: ServerResolvedBinding,
    pub(crate) role_session: RoleSession,
}

#[derive(Clone, Default)]
pub(crate) struct M6OrgGlobalRoleSessionSlot {
    runtime: Option<M6OrgGlobalRoleSessionRuntime>,
}

impl M6OrgGlobalRoleSessionSlot {
    pub(crate) fn unavailable() -> Self {
        Self { runtime: None }
    }

    #[cfg(test)]
    pub(crate) fn is_installed(&self) -> bool {
        self.runtime.is_some()
    }

    pub(crate) fn status(&self) -> M6OrgGlobalRoleSessionStatusDto {
        match self.load_established_session() {
            Ok(session) => M6OrgGlobalRoleSessionStatusDto::ready(
                &session,
                self,
                M6OrgGlobalRoleSessionContextDto {
                    summary_refs: Vec::new(),
                    source_refs: Vec::new(),
                },
            ),
            Err(error) => M6OrgGlobalRoleSessionStatusDto::unavailable(error),
        }
    }

    pub(crate) fn summary_consumer_lease(
        &self,
        now_ms: i64,
    ) -> Result<M6OrgGlobalSummaryConsumerLease, String> {
        if now_ms < 0 {
            return Err(M6_ORG_GLOBAL_ROLE_SESSION_INCOMPLETE.to_string());
        }
        let session = self.load_established_session()?;
        Ok(M6OrgGlobalSummaryConsumerLease {
            role_session_id: session.role_session_id.as_str().to_string(),
            role_session_revision: session.revision,
            scope_kind: M6_ORG_GLOBAL_SCOPE_KIND.to_string(),
            consumer_expires_at_ms: now_ms
                .checked_add(60_000)
                .ok_or_else(|| M6_ORG_GLOBAL_ROLE_SESSION_INCOMPLETE.to_string())?,
        })
    }

    pub(crate) fn status_with_minimal_context(
        &self,
        summary_refs: Vec<String>,
        source_refs: Vec<String>,
    ) -> Result<M6OrgGlobalRoleSessionStatusDto, String> {
        let session = self.load_established_session()?;
        let context = M6OrgGlobalRoleSessionContextDto::minimal(summary_refs, source_refs)?;
        Ok(M6OrgGlobalRoleSessionStatusDto::ready(
            &session, self, context,
        ))
    }

    pub(crate) fn authority_seed(&self) -> Result<M6OrgGlobalRoleSessionAuthoritySeed, String> {
        let role_session = self.load_established_session()?;
        let runtime = self
            .runtime
            .as_ref()
            .ok_or_else(|| M6_ORG_GLOBAL_ROLE_SESSION_UNAVAILABLE.to_string())?;
        Ok(M6OrgGlobalRoleSessionAuthoritySeed {
            repository: runtime.repository.clone(),
            binding: runtime.binding.clone(),
            role_session,
        })
    }

    fn load_established_session(&self) -> Result<RoleSession, String> {
        let runtime = self
            .runtime
            .as_ref()
            .ok_or_else(|| M6_ORG_GLOBAL_ROLE_SESSION_UNAVAILABLE.to_string())?;
        validate_exact_global_binding(&runtime.binding)?;
        let snapshot = runtime
            .repository
            .load_authorized_role_session_snapshot(&M3RoleSessionSnapshotQuery {
                role_session_id: runtime.role_session_id.clone(),
                binding: runtime.binding.clone(),
            })
            .map_err(map_repository_error)?
            .ok_or_else(|| M6_ORG_GLOBAL_ROLE_SESSION_MISSING_AFTER_ESTABLISHED.to_string())?;
        if snapshot.session.role_session_id != runtime.role_session_id
            || snapshot.session.status != RoleSessionState::Active
            || !snapshot.session.matches_binding_identity(&runtime.binding)
            || snapshot.session.permission_snapshot_ref != runtime.binding.permission_snapshot_ref
            || !matches!(snapshot.permission, M3ReadPermissionDisposition::Current)
        {
            return Err(M6_ORG_GLOBAL_ROLE_SESSION_MISMATCHED.to_string());
        }
        Ok(snapshot.session)
    }
}

/// Installs the ordinary-product Global Supervisor runtime on the exact M3
/// repository handle cloned from ordinary M4/M3 composition.
pub(crate) fn install_ordinary_product_runtime(
    repository: M3RoleSessionSqliteRepository,
) -> Result<M6OrgGlobalRoleSessionSlot, String> {
    let binding = server_fixed_global_supervisor_binding()?;
    validate_exact_global_binding(&binding)?;
    let role_session_id =
        bootstrap_or_restore_global_supervisor_role_session(&repository, &binding)?;
    Ok(M6OrgGlobalRoleSessionSlot {
        runtime: Some(M6OrgGlobalRoleSessionRuntime {
            repository,
            binding,
            role_session_id,
        }),
    })
}

/// Explicit attempted project-write authorization path. The Global Supervisor
/// runtime exposes no project write capability; this returns the stable
/// fail-closed error before any mutation.
pub(crate) fn authorize_attempted_project_write(
    slot: &M6OrgGlobalRoleSessionSlot,
) -> Result<(), String> {
    let _ = slot;
    Err(M6_ORG_GLOBAL_ROLE_SESSION_PROJECT_WRITE_REJECTED.to_string())
}

pub(crate) fn validate_exact_global_binding(binding: &ServerResolvedBinding) -> Result<(), String> {
    let expected = server_fixed_global_supervisor_binding()?;
    if binding.actor_id != expected.actor_id
        || binding.role_ref != expected.role_ref
        || binding.scope_ref != expected.scope_ref
        || binding.current_object_ref != expected.current_object_ref
        || binding.execution_channel != expected.execution_channel
        || binding.permission_snapshot_ref != expected.permission_snapshot_ref
        || binding.owner_fingerprint != expected.owner_fingerprint
    {
        return Err(M6_ORG_GLOBAL_ROLE_SESSION_BINDING_REJECTED.to_string());
    }
    Ok(())
}

fn server_fixed_global_supervisor_binding() -> Result<ServerResolvedBinding, String> {
    ServerResolvedBinding::from_server_canonical(
        sealed_ref("actor", M6_ORG_GLOBAL_ACTOR_MATERIAL),
        sealed_ref("role", M6_ORG_GLOBAL_ROLE_MATERIAL),
        sealed_ref("scope", M6_ORG_GLOBAL_SCOPE_MATERIAL),
        sealed_ref("object", M6_ORG_GLOBAL_OBJECT_MATERIAL),
        sealed_ref("channel", M6_ORG_GLOBAL_CHANNEL_MATERIAL),
        sealed_ref("permission", M6_ORG_GLOBAL_PERMISSION_MATERIAL),
    )
    .map_err(|_| M6_ORG_GLOBAL_ROLE_SESSION_SOURCE_UNAVAILABLE.to_string())
}

fn bootstrap_or_restore_global_supervisor_role_session(
    repository: &M3RoleSessionSqliteRepository,
    binding: &ServerResolvedBinding,
) -> Result<RoleSessionId, String> {
    let entries = list_global_supervisor_candidates(repository, binding)?;
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
        .any(|entry| !matches!(entry.permission, M3ReadPermissionDisposition::Current));
    if live_candidates.len() > 1 || has_mismatched_candidate {
        for entry in &live_candidates {
            quarantine_global_supervisor_candidate(
                repository,
                binding,
                entry.session.role_session_id.clone(),
                entry.session.revision,
            )?;
        }
        return Err(if has_mismatched_candidate {
            M6_ORG_GLOBAL_ROLE_SESSION_MISMATCHED
        } else {
            M6_ORG_GLOBAL_ROLE_SESSION_AMBIGUOUS
        }
        .to_string());
    }

    let Some(entry) = live_candidates.into_iter().next() else {
        if entries
            .iter()
            .any(|entry| entry.session.status == RoleSessionState::Quarantined)
        {
            return Err(M6_ORG_GLOBAL_ROLE_SESSION_QUARANTINED.to_string());
        }
        if entries
            .iter()
            .any(|entry| entry.session.status == RoleSessionState::Closed)
        {
            return Err(M6_ORG_GLOBAL_ROLE_SESSION_CLOSED.to_string());
        }
        return create_global_supervisor_role_session(repository, binding);
    };

    match entry.session.status {
        RoleSessionState::Active => {
            if !matches!(entry.permission, M3ReadPermissionDisposition::Current) {
                return Err(M6_ORG_GLOBAL_ROLE_SESSION_MISMATCHED.to_string());
            }
            if !entry.session.matches_binding_identity(binding) {
                return Err(M6_ORG_GLOBAL_ROLE_SESSION_MISMATCHED.to_string());
            }
            Ok(entry.session.role_session_id.clone())
        }
        RoleSessionState::Created | RoleSessionState::Suspended => {
            Err(M6_ORG_GLOBAL_ROLE_SESSION_INCOMPLETE.to_string())
        }
        RoleSessionState::Quarantined => Err(M6_ORG_GLOBAL_ROLE_SESSION_QUARANTINED.to_string()),
        RoleSessionState::Closed => Err(M6_ORG_GLOBAL_ROLE_SESSION_CLOSED.to_string()),
    }
}

fn list_global_supervisor_candidates(
    repository: &M3RoleSessionSqliteRepository,
    binding: &ServerResolvedBinding,
) -> Result<Vec<crate::m3_role_session_repository::M3RoleSessionDirectoryEntry>, String> {
    let mut entries = Vec::new();
    let mut after = None;
    loop {
        let page = repository
            .list_authorized_role_session_directory(&M3RoleSessionDirectoryQuery {
                binding: binding.clone(),
                after,
                limit: 100,
            })
            .map_err(map_repository_error)?;
        entries.extend(page.entries);
        let Some(next_cursor) = page.next_cursor else {
            return Ok(entries);
        };
        after = Some(next_cursor);
    }
}

fn quarantine_global_supervisor_candidate(
    repository: &M3RoleSessionSqliteRepository,
    binding: &ServerResolvedBinding,
    role_session_id: RoleSessionId,
    expected_session_revision: u64,
) -> Result<(), String> {
    let material = format!(
        "syn.m6.org.global-supervisor.quarantine/organization-primary/v1/role-session:{}/revision:{expected_session_revision}",
        role_session_id.as_str(),
    );
    let outcome = repository
        .quarantine_role_session(&QuarantineRoleSessionCommand {
            role_session_id: role_session_id.clone(),
            binding: binding.clone(),
            expected_session_revision,
            metadata: metadata_for(repository, "quarantine", &material)?,
        })
        .map_err(|_| M6_ORG_GLOBAL_ROLE_SESSION_SOURCE_UNAVAILABLE.to_string())?;
    let session = outcome
        .role_session
        .ok_or_else(|| M6_ORG_GLOBAL_ROLE_SESSION_SOURCE_UNAVAILABLE.to_string())?;
    if session.role_session_id != role_session_id || session.status != RoleSessionState::Quarantined
    {
        return Err(M6_ORG_GLOBAL_ROLE_SESSION_SOURCE_UNAVAILABLE.to_string());
    }
    Ok(())
}

fn create_global_supervisor_role_session(
    repository: &M3RoleSessionSqliteRepository,
    binding: &ServerResolvedBinding,
) -> Result<RoleSessionId, String> {
    let role_session_id = role_session_id()?;
    let outcome = repository
        .create_role_session(&CreateRoleSessionCommand {
            role_session_id: role_session_id.clone(),
            binding: binding.clone(),
            metadata: metadata_for(repository, "create", M6_ORG_GLOBAL_CREATE_MATERIAL)?,
        })
        .map_err(map_create_error)?;
    let session = outcome
        .role_session
        .ok_or_else(|| M6_ORG_GLOBAL_ROLE_SESSION_SOURCE_UNAVAILABLE.to_string())?;
    if session.role_session_id != role_session_id || session.status != RoleSessionState::Active {
        return Err(M6_ORG_GLOBAL_ROLE_SESSION_SOURCE_UNAVAILABLE.to_string());
    }
    Ok(role_session_id)
}

fn role_session_id() -> Result<RoleSessionId, String> {
    RoleSessionId::try_from_canonical(sealed_ref("session", M6_ORG_GLOBAL_SESSION_ID_MATERIAL))
        .map_err(|_| M6_ORG_GLOBAL_ROLE_SESSION_SOURCE_UNAVAILABLE.to_string())
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
        .map_err(|_| M6_ORG_GLOBAL_ROLE_SESSION_SOURCE_UNAVAILABLE.to_string())?,
        request_idempotency_key: RequestIdempotencyKey::try_from_canonical(sealed_ref(
            "request",
            &format!("{material}/idempotency/{operation}"),
        ))
        .map_err(|_| M6_ORG_GLOBAL_ROLE_SESSION_SOURCE_UNAVAILABLE.to_string())?,
        occurred_at: repository
            .capture_server_utc_now()
            .map_err(|_| M6_ORG_GLOBAL_ROLE_SESSION_SOURCE_UNAVAILABLE.to_string())?,
    })
}

fn opaque_ref(namespace: &str, material: &str) -> Result<OpaqueRef, String> {
    OpaqueRef::try_from_canonical(sealed_ref(namespace, material))
        .map_err(|_| M6_ORG_GLOBAL_ROLE_SESSION_SOURCE_UNAVAILABLE.to_string())
}

fn sealed_ref(namespace: &str, material: &str) -> String {
    format!(
        "{namespace}:sha256:{}",
        Sha256Digest::of_bytes(material.as_bytes()).as_str()
    )
}

fn map_create_error(
    error: crate::m3_role_session_repository::M3RoleSessionRepositoryError,
) -> String {
    if error.code.contains("m3_role_session_not_found") {
        M6_ORG_GLOBAL_ROLE_SESSION_MISSING_AFTER_ESTABLISHED.to_string()
    } else {
        map_repository_error(error)
    }
}

fn map_repository_error(
    error: crate::m3_role_session_repository::M3RoleSessionRepositoryError,
) -> String {
    if error.code.contains("schema")
        || error.code.contains("corrupt")
        || error.code.contains("verify")
    {
        M6_ORG_GLOBAL_ROLE_SESSION_CORRUPT.to_string()
    } else if error.code.contains("not_found") {
        M6_ORG_GLOBAL_ROLE_SESSION_MISSING_AFTER_ESTABLISHED.to_string()
    } else {
        M6_ORG_GLOBAL_ROLE_SESSION_SOURCE_UNAVAILABLE.to_string()
    }
}

#[cfg(test)]
fn open_scratch_repository(root: &std::path::Path) -> M3RoleSessionSqliteRepository {
    use crate::m3_role_session_repository::{
        M3OrdinaryRoleSessionRepositoryConfig, M3_ORDINARY_ROLE_SESSION_RELATIVE_PATH,
    };
    M3RoleSessionSqliteRepository::open_ordinary_product(&M3OrdinaryRoleSessionRepositoryConfig {
        app_data_root: root.to_path_buf(),
        db_path: root.join(M3_ORDINARY_ROLE_SESSION_RELATIVE_PATH),
    })
    .expect("open ordinary M3 repository on scratch root")
}

#[cfg(test)]
fn scratch_app_data_root(label: &str) -> (std::path::PathBuf, std::path::PathBuf) {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQUENCE: AtomicU64 = AtomicU64::new(1);
    let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let fixture_root = std::env::temp_dir().join(format!(
        "syn-m6d02-{label}-{}-{sequence}",
        std::process::id()
    ));
    let root = fixture_root.join(crate::m1_project_index::M1_ORDINARY_APP_DATA_DIR_NAME);
    std::fs::create_dir_all(&root).expect("create M6D02 scratch app-data root");
    let root = std::fs::canonicalize(&root).expect("canonicalize M6D02 scratch app-data root");
    (fixture_root, root)
}

#[cfg(test)]
fn write_ordinary_seeds(parent: &std::path::Path) -> (std::path::PathBuf, std::path::PathBuf) {
    let seed_dir = parent.join("synthetic-ordinary-product-seeds");
    std::fs::create_dir_all(&seed_dir).expect("create M6D02 seed dir");
    let index_seed = seed_dir.join("codex-index.json");
    let tasks_seed = seed_dir.join("README.md");
    std::fs::write(&index_seed, r#"{"projects":[]}"#).expect("write M6D02 index seed");
    std::fs::write(&tasks_seed, "# synthetic\n").expect("write M6D02 tasks seed");
    (index_seed, tasks_seed)
}

#[cfg(test)]
fn assert_ready_status(status: &M6OrgGlobalRoleSessionStatusDto) -> (String, u64, String) {
    match status {
        M6OrgGlobalRoleSessionStatusDto::Ready {
            role_session_id,
            revision,
            state,
            scope_kind,
            read_only,
            project_write_capability,
            provider_handle_authorizes,
            context,
        } => {
            assert_eq!(scope_kind, M6_ORG_GLOBAL_SCOPE_KIND);
            assert!(*read_only);
            assert!(!*project_write_capability);
            assert!(!*provider_handle_authorizes);
            assert!(context.summary_refs.is_empty());
            assert!(context.source_refs.is_empty());
            (role_session_id.clone(), *revision, state.clone())
        }
        M6OrgGlobalRoleSessionStatusDto::Unavailable { error } => {
            panic!("expected ready Global Supervisor status, got {error}")
        }
    }
}

#[cfg(test)]
mod m6d02_tests {
    use super::*;
    use crate::m3_role_session_repository::M3_ORDINARY_ROLE_SESSION_RELATIVE_PATH;
    use crate::mcp::identity_kernel::resolve_m4_primary_secretary_identity;
    use crate::AppState;

    #[test]
    fn m6d02_m3_persistence_round_trip_same_identity() {
        let (fixture_root, root) = scratch_app_data_root("round-trip");
        let repository = open_scratch_repository(&root);
        let first = install_ordinary_product_runtime(repository).expect("first install");
        let (first_id, first_revision, first_state) = assert_ready_status(&first.status());
        assert_eq!(first_state, "ACTIVE");
        drop(first);

        let reopened = open_scratch_repository(&root);
        let restored = install_ordinary_product_runtime(reopened).expect("restore install");
        let (restored_id, restored_revision, restored_state) =
            assert_ready_status(&restored.status());
        assert_eq!(restored_id, first_id);
        assert_eq!(restored_revision, first_revision);
        assert_eq!(restored_state, "ACTIVE");
        let _ = std::fs::remove_dir_all(fixture_root);
    }

    #[test]
    fn m6d02_drop_reopen_restart_same_identity() {
        let (fixture_root, root) = scratch_app_data_root("restart");
        let first = install_ordinary_product_runtime(open_scratch_repository(&root))
            .expect("first process install");
        let (first_id, _, _) = assert_ready_status(&first.status());
        drop(first);
        let second = install_ordinary_product_runtime(open_scratch_repository(&root))
            .expect("restarted process install");
        let (second_id, _, _) = assert_ready_status(&second.status());
        assert_eq!(first_id, second_id);
        assert_eq!(
            first_id,
            role_session_id().expect("server-fixed session id").as_str()
        );
        let _ = std::fs::remove_dir_all(fixture_root);
    }

    #[test]
    fn m6d02_identity_is_server_fixed_not_path_derived() {
        let (first_fixture, first_root) = scratch_app_data_root("path-a");
        let (second_fixture, second_root) = scratch_app_data_root("path-b");
        assert_ne!(first_root, second_root);
        let first = install_ordinary_product_runtime(open_scratch_repository(&first_root))
            .expect("install on first root");
        let second = install_ordinary_product_runtime(open_scratch_repository(&second_root))
            .expect("install on second root");
        let (first_id, _, _) = assert_ready_status(&first.status());
        let (second_id, _, _) = assert_ready_status(&second.status());
        assert_eq!(first_id, second_id);
        let binding = server_fixed_global_supervisor_binding().expect("fixed binding");
        assert!(!binding
            .actor_id
            .as_str()
            .contains(first_root.to_string_lossy().as_ref()));
        assert!(!binding.scope_ref.as_str().contains("project"));
        assert!(M6_ORG_GLOBAL_SCOPE_MATERIAL.contains("GLOBAL"));
        assert!(M6_ORG_GLOBAL_ROLE_MATERIAL.contains("global_supervisor"));
        let _ = std::fs::remove_dir_all(first_fixture);
        let _ = std::fs::remove_dir_all(second_fixture);
    }

    #[test]
    fn m6d02_project_and_secretary_scope_bindings_are_rejected() {
        let expected = server_fixed_global_supervisor_binding().expect("fixed binding");
        validate_exact_global_binding(&expected).expect("canonical binding must pass");

        let secretary = resolve_m4_primary_secretary_identity()
            .expect("fixed Secretary identity")
            .m3_server_resolved_binding()
            .expect("Secretary M3 binding");
        let secretary_error = validate_exact_global_binding(&secretary)
            .expect_err("Secretary personal scope must fail global binding");
        assert_eq!(secretary_error, M6_ORG_GLOBAL_ROLE_SESSION_BINDING_REJECTED);
        assert_ne!(secretary.role_ref, expected.role_ref);
        assert_ne!(secretary.scope_ref, expected.scope_ref);

        let project = ServerResolvedBinding::from_server_canonical(
            sealed_ref("actor", "syn.m6d02.fake.project-supervisor.actor/v1"),
            sealed_ref(
                "role",
                "syn.m6d02.fake.project-supervisor.role/project_supervisor/v1",
            ),
            sealed_ref(
                "scope",
                "syn.m6d02.fake.project-supervisor.scope/PROJECT/v1",
            ),
            sealed_ref("object", "syn.m6d02.fake.project-supervisor.object/v1"),
            sealed_ref("channel", "syn.m6d02.fake.project-supervisor.channel/v1"),
            sealed_ref(
                "permission",
                "syn.m6d02.fake.project-supervisor.permission/v1",
            ),
        )
        .expect("fake project supervisor binding");
        let project_error = validate_exact_global_binding(&project)
            .expect_err("Project Supervisor scope must fail global binding");
        assert_eq!(project_error, M6_ORG_GLOBAL_ROLE_SESSION_BINDING_REJECTED);
        assert_ne!(project.role_ref, expected.role_ref);
        assert_ne!(project.scope_ref, expected.scope_ref);
        assert_ne!(project.role_ref.as_str(), secretary.role_ref.as_str());
    }

    #[test]
    fn m6d02_project_supervisor_session_is_not_selected_as_global() {
        let (fixture_root, root) = scratch_app_data_root("scope-isolation");
        let repository = open_scratch_repository(&root);
        let project_binding = ServerResolvedBinding::from_server_canonical(
            sealed_ref("actor", "syn.m6d02.fake.project-supervisor.actor/v1"),
            sealed_ref(
                "role",
                "syn.m6d02.fake.project-supervisor.role/project_supervisor/v1",
            ),
            sealed_ref(
                "scope",
                "syn.m6d02.fake.project-supervisor.scope/PROJECT/v1",
            ),
            sealed_ref("object", "syn.m6d02.fake.project-supervisor.object/v1"),
            sealed_ref("channel", "syn.m6d02.fake.project-supervisor.channel/v1"),
            sealed_ref(
                "permission",
                "syn.m6d02.fake.project-supervisor.permission/v1",
            ),
        )
        .expect("project binding");
        let project_session_id = RoleSessionId::try_from_canonical(sealed_ref(
            "session",
            "syn.m6d02.fake.project-supervisor.session/v1",
        ))
        .expect("project session id");
        repository
            .create_role_session(&CreateRoleSessionCommand {
                role_session_id: project_session_id.clone(),
                binding: project_binding,
                metadata: metadata_for(
                    &repository,
                    "create",
                    "syn.m6d02.fake.project-supervisor.create/v1",
                )
                .expect("project create metadata"),
            })
            .expect("create project supervisor session");

        let installed =
            install_ordinary_product_runtime(repository).expect("install global after project");
        let (global_id, _, _) = assert_ready_status(&installed.status());
        assert_ne!(global_id, project_session_id.as_str());
        let _ = std::fs::remove_dir_all(fixture_root);
    }

    #[test]
    fn m6d02_read_only_project_write_rejected_with_zero_mutation() {
        let (fixture_root, root) = scratch_app_data_root("read-only");
        let installed = install_ordinary_product_runtime(open_scratch_repository(&root))
            .expect("install read-only runtime");
        let (role_session_id, revision, state) = assert_ready_status(&installed.status());
        let db_path = root.join(M3_ORDINARY_ROLE_SESSION_RELATIVE_PATH);
        let before = std::fs::read(&db_path).expect("read established M3 bytes");

        let error = authorize_attempted_project_write(&installed)
            .expect_err("project write must fail closed");
        assert_eq!(error, M6_ORG_GLOBAL_ROLE_SESSION_PROJECT_WRITE_REJECTED);

        let (after_id, after_revision, after_state) = assert_ready_status(&installed.status());
        assert_eq!(after_id, role_session_id);
        assert_eq!(after_revision, revision);
        assert_eq!(after_state, state);
        assert_eq!(
            std::fs::read(&db_path).expect("reread established M3 bytes"),
            before
        );
        let _ = std::fs::remove_dir_all(fixture_root);
    }

    #[test]
    fn m6d02_ambiguous_live_candidates_are_quarantined_and_fail_closed() {
        let (fixture_root, root) = scratch_app_data_root("ambiguous");
        let repository = open_scratch_repository(&root);
        let first = install_ordinary_product_runtime(repository.clone()).expect("first install");
        drop(first);
        let binding = server_fixed_global_supervisor_binding().expect("fixed binding");
        let second_id = RoleSessionId::try_from_canonical(sealed_ref(
            "session",
            "syn.m6.org.global-supervisor.role-session/ambiguous-second/v1",
        ))
        .expect("second session id");
        repository
            .create_role_session(&CreateRoleSessionCommand {
                role_session_id: second_id,
                binding: binding.clone(),
                metadata: metadata_for(
                    &repository,
                    "create",
                    "syn.m6.org.global-supervisor.create/ambiguous-second/v1",
                )
                .expect("second create metadata"),
            })
            .expect("create ambiguous candidate");

        let error = match install_ordinary_product_runtime(repository.clone()) {
            Ok(_) => panic!("ambiguous candidates must not install"),
            Err(error) => error,
        };
        assert_eq!(error, M6_ORG_GLOBAL_ROLE_SESSION_AMBIGUOUS);
        let entries = list_global_supervisor_candidates(&repository, &binding)
            .expect("list quarantined candidates");
        assert_eq!(entries.len(), 2);
        assert!(entries
            .iter()
            .all(|entry| entry.session.status == RoleSessionState::Quarantined));
        let _ = std::fs::remove_dir_all(fixture_root);
    }

    #[test]
    fn m6d08_ordinary_appstate_deliberately_fails_closed_without_legacy_fallback() {
        let (fixture_root, root) = scratch_app_data_root("m6d08-ordinary-fail-closed");
        let (index_seed, tasks_seed) = write_ordinary_seeds(&fixture_root);
        let repository = open_scratch_repository(&root);
        let first = install_ordinary_product_runtime(repository.clone())
            .expect("establish first Global Supervisor candidate");
        drop(first);
        let binding = server_fixed_global_supervisor_binding().expect("fixed binding");
        repository
            .create_role_session(&CreateRoleSessionCommand {
                role_session_id: RoleSessionId::try_from_canonical(sealed_ref(
                    "session",
                    "syn.m6.org.global-supervisor.role-session/m6d08-ambiguous-second/v1",
                ))
                .expect("second session id"),
                binding,
                metadata: metadata_for(
                    &repository,
                    "create",
                    "syn.m6.org.global-supervisor.create/m6d08-ambiguous-second/v1",
                )
                .expect("second create metadata"),
            })
            .expect("create second live Global Supervisor candidate");

        let error = match AppState::try_new_with_tauri_ordinary_product_seeds(
            &root,
            &index_seed,
            &tasks_seed,
        ) {
            Ok(_) => panic!("ordinary AppState must not start with an unavailable fallback"),
            Err(error) => error,
        };
        assert_eq!(error, M6_ORG_GLOBAL_ROLE_SESSION_AMBIGUOUS);

        let lib = include_str!("lib.rs");
        let ordinary = lib
            .split("SharedProductAuthorityProfile::OrdinaryInstalled =>")
            .nth(1)
            .expect("ordinary authority branch");
        let ordinary = ordinary
            .split("SharedProductAuthorityProfile::IsolatedUninstalled =>")
            .next()
            .expect("ordinary branch boundary");
        assert!(ordinary.contains("install_ordinary_product_runtime("));
        assert!(ordinary.contains("m6_repository,"));
        assert!(ordinary.contains(")?;"));
        assert!(!ordinary.contains("M6OrgGlobalRoleSessionSlot::unavailable()"));
        let isolated = lib
            .split("SharedProductAuthorityProfile::IsolatedUninstalled =>")
            .nth(1)
            .expect("isolated authority branch")
            .split("};")
            .next()
            .expect("isolated branch boundary");
        assert!(isolated.contains("M6OrgGlobalRoleSessionSlot::unavailable()"));
        let _ = std::fs::remove_dir_all(fixture_root);
    }

    #[test]
    fn m6d02_permission_mismatch_is_quarantined_and_fails_closed() {
        let (fixture_root, root) = scratch_app_data_root("permission-mismatch");
        let repository = open_scratch_repository(&root);
        let expected = server_fixed_global_supervisor_binding().expect("fixed binding");
        let mismatched = ServerResolvedBinding::from_server_canonical(
            expected.actor_id.as_str().to_string(),
            expected.role_ref.as_str().to_string(),
            expected.scope_ref.as_str().to_string(),
            expected.current_object_ref.as_str().to_string(),
            expected.execution_channel.as_str().to_string(),
            sealed_ref(
                "permission",
                "syn.m6.org.global-supervisor.permission/mismatched/v1",
            ),
        )
        .expect("mismatched permission binding");
        assert_eq!(mismatched.owner_fingerprint, expected.owner_fingerprint);
        assert_ne!(
            mismatched.permission_snapshot_ref,
            expected.permission_snapshot_ref
        );
        repository
            .create_role_session(&CreateRoleSessionCommand {
                role_session_id: role_session_id().expect("fixed session id"),
                binding: mismatched,
                metadata: metadata_for(
                    &repository,
                    "create",
                    "syn.m6.org.global-supervisor.create/mismatched-permission/v1",
                )
                .expect("mismatch create metadata"),
            })
            .expect("create mismatched candidate");

        let error = match install_ordinary_product_runtime(repository.clone()) {
            Ok(_) => panic!("permission mismatch must not install"),
            Err(error) => error,
        };
        assert_eq!(error, M6_ORG_GLOBAL_ROLE_SESSION_MISMATCHED);
        let entries = list_global_supervisor_candidates(&repository, &expected)
            .expect("list quarantined mismatch");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].session.status, RoleSessionState::Quarantined);
        let _ = std::fs::remove_dir_all(fixture_root);
    }

    #[test]
    fn m6d02_context_shape_is_minimal_refs_only() {
        let source = include_str!("m6_org_global_role_session.rs");
        let start = source
            .find("pub(crate) struct M6OrgGlobalRoleSessionContextDto {")
            .expect("context dto");
        let block = &source[start..];
        let end = block.find('}').expect("context dto close");
        let context = &block[..=end];
        assert!(context.contains("summary_refs"));
        assert!(context.contains("source_refs"));
        for forbidden in [
            "raw_file",
            "raw_summary",
            "transcript",
            "secret",
            "untrimmed_memory",
            "provider_response",
            "prompt",
            "stdout",
            "stderr",
            "tool_output",
        ] {
            assert!(
                !context.contains(forbidden),
                "context dto must not contain {forbidden}"
            );
        }
    }

    #[test]
    fn m6d02_missing_established_session_fails_closed() {
        let (fixture_root, root) = scratch_app_data_root("missing");
        let repository = open_scratch_repository(&root);
        let first =
            install_ordinary_product_runtime(repository.clone()).expect("establish session");
        drop(first);

        let db_path = root.join(M3_ORDINARY_ROLE_SESSION_RELATIVE_PATH);
        let connection = rusqlite::Connection::open(&db_path).expect("open established M3 db");
        connection
            .pragma_update(None, "foreign_keys", "OFF")
            .expect("disable fk to simulate missing established row");
        let deleted = connection
            .execute("DELETE FROM m3_role_sessions", [])
            .expect("delete established session row");
        assert_eq!(deleted, 1);
        drop(connection);

        let error = match install_ordinary_product_runtime(repository) {
            Ok(_) => panic!("missing established session must not recreate"),
            Err(error) => error,
        };
        assert_eq!(error, M6_ORG_GLOBAL_ROLE_SESSION_MISSING_AFTER_ESTABLISHED);
        let remaining = rusqlite::Connection::open(&db_path)
            .expect("reopen established M3 db")
            .query_row("SELECT COUNT(*) FROM m3_role_sessions", [], |row| {
                row.get::<_, i64>(0)
            })
            .expect("count remaining sessions");
        assert_eq!(remaining, 0);
        let _ = std::fs::remove_dir_all(fixture_root);
    }

    #[test]
    fn m6d02_corrupt_established_source_fails_closed() {
        let (fixture_root, root) = scratch_app_data_root("corrupt");
        let first = install_ordinary_product_runtime(open_scratch_repository(&root))
            .expect("establish session");
        drop(first);
        let db_path = root.join(M3_ORDINARY_ROLE_SESSION_RELATIVE_PATH);
        std::fs::write(&db_path, b"not-an-m3-role-session-database")
            .expect("corrupt established M3 source");
        let error = match M3RoleSessionSqliteRepository::open_ordinary_product(
            &crate::m3_role_session_repository::M3OrdinaryRoleSessionRepositoryConfig {
                app_data_root: root.clone(),
                db_path: db_path.clone(),
            },
        ) {
            Ok(repository) => match install_ordinary_product_runtime(repository) {
                Ok(_) => panic!("corrupt source must not install"),
                Err(error) => error,
            },
            Err(error) => {
                if error.code.contains("schema") || error.code.contains("verify") {
                    M6_ORG_GLOBAL_ROLE_SESSION_CORRUPT.to_string()
                } else {
                    M6_ORG_GLOBAL_ROLE_SESSION_SOURCE_UNAVAILABLE.to_string()
                }
            }
        };
        assert!(
            error == M6_ORG_GLOBAL_ROLE_SESSION_CORRUPT
                || error == M6_ORG_GLOBAL_ROLE_SESSION_SOURCE_UNAVAILABLE,
            "corrupt source must fail closed, got {error}"
        );
        let _ = std::fs::remove_dir_all(fixture_root);
    }

    #[test]
    fn m6d02_ordinary_appstate_installs_and_legacy_isolated_stay_unavailable() {
        let (fixture_root, root) = scratch_app_data_root("appstate");
        let (index_seed, tasks_seed) = write_ordinary_seeds(&fixture_root);
        let ordinary =
            AppState::try_new_with_tauri_ordinary_product_seeds(&root, &index_seed, &tasks_seed)
                .expect("ordinary AppState");
        assert!(ordinary.m6_org_global_role_session.is_installed());
        let (ordinary_id, _, _) =
            assert_ready_status(&ordinary.m6_org_global_role_session.status());

        let restarted =
            AppState::try_new_with_tauri_ordinary_product_seeds(&root, &index_seed, &tasks_seed)
                .expect("restarted ordinary AppState");
        let (restarted_id, _, _) =
            assert_ready_status(&restarted.m6_org_global_role_session.status());
        assert_eq!(ordinary_id, restarted_id);

        let isolated_root = fixture_root.join("isolated-profile");
        std::fs::create_dir_all(isolated_root.join("app-data")).expect("isolated profile");
        let isolated_root =
            std::fs::canonicalize(&isolated_root).expect("canonicalize isolated profile");
        let isolated_paths = crate::acceptance_runtime_profile::RuntimePaths {
            root: isolated_root.clone(),
            index_path: index_seed.clone(),
            tasks_path: tasks_seed.clone(),
            project_root: isolated_root.join("project"),
            workflow_state_path: isolated_root.join("workflow-state.json"),
            app_data_root: isolated_root.join("app-data"),
            vault_root: isolated_root.join("vault"),
            recovery_backups_root: isolated_root.join("recovery"),
            canvas_root: isolated_root.join("canvas"),
            codex_db_path: isolated_root.join("codex.sqlite"),
            app_log_dir: isolated_root.join("logs"),
        };
        let isolated = AppState::try_new_with_isolated_product_profile(&isolated_paths)
            .expect("isolated AppState");
        assert!(!isolated.m6_org_global_role_session.is_installed());
        match isolated.m6_org_global_role_session.status() {
            M6OrgGlobalRoleSessionStatusDto::Unavailable { error } => {
                assert_eq!(error, M6_ORG_GLOBAL_ROLE_SESSION_UNAVAILABLE);
            }
            M6OrgGlobalRoleSessionStatusDto::Ready { .. } => {
                panic!("isolated profile must not expose Global Supervisor")
            }
        }

        let legacy = AppState::try_new().expect("legacy AppState");
        assert!(!legacy.m6_org_global_role_session.is_installed());
        match legacy.m6_org_global_role_session.status() {
            M6OrgGlobalRoleSessionStatusDto::Unavailable { error } => {
                assert_eq!(error, M6_ORG_GLOBAL_ROLE_SESSION_UNAVAILABLE);
            }
            M6OrgGlobalRoleSessionStatusDto::Ready { .. } => {
                panic!("legacy profile must not expose Global Supervisor")
            }
        }
        let _ = std::fs::remove_dir_all(fixture_root);
    }

    #[test]
    fn m6d02_command_registry_has_one_host_fixed_status_entry() {
        let registry = include_str!("command_registry.rs");
        assert_eq!(
            registry
                .matches("load_global_supervisor_role_session_status")
                .count(),
            1
        );
        assert!(registry.contains("tauri::generate_handler!["));
        let commands = include_str!("commands.rs");
        let start = commands
            .find("fn load_global_supervisor_role_session_status(")
            .expect("host-fixed command");
        let span = &commands[start..];
        let end = span
            .find("fn load_global_supervisor_role_session_status_for_state(")
            .expect("state helper");
        let command = &span[..end];
        assert!(command.contains("tauri::State<'_, AppState>"));
        assert!(!command.contains("actor"));
        assert!(!command.contains("role_ref"));
        assert!(!command.contains("scope"));
        assert!(!command.contains("provider"));
        assert!(!command.contains("permission"));
        assert!(!command.contains("project_root"));
        assert!(!command.contains("request:"));
    }

    #[test]
    fn m6d02_ordinary_command_status_is_ready_and_uninstalled_is_unavailable() {
        let (fixture_root, root) = scratch_app_data_root("command");
        let (index_seed, tasks_seed) = write_ordinary_seeds(&fixture_root);
        let ordinary =
            AppState::try_new_with_tauri_ordinary_product_seeds(&root, &index_seed, &tasks_seed)
                .expect("ordinary AppState");
        let dto = crate::load_global_supervisor_role_session_status_for_state(&ordinary)
            .expect("ordinary command status");
        let _ = assert_ready_status(&dto);

        let fixture = AppState {
            index_path: std::path::PathBuf::from("/m6d02/fixture/index.json"),
            tasks_path: std::path::PathBuf::from("/m6d02/fixture/tasks.md"),
            workflow_state_path: std::path::PathBuf::from("/m6d02/fixture/workflow-state.json"),
            m3_role_session_read_runtime: Default::default(),
            m1_project_index: None,
            m3_project_role_session_authority: None,
            m5_store_path: None,
            m6_org_global_role_session: Default::default(),
        };
        let unavailable = crate::load_global_supervisor_role_session_status_for_state(&fixture)
            .expect("uninstalled command status");
        match unavailable {
            M6OrgGlobalRoleSessionStatusDto::Unavailable { error } => {
                assert_eq!(error, M6_ORG_GLOBAL_ROLE_SESSION_UNAVAILABLE);
            }
            M6OrgGlobalRoleSessionStatusDto::Ready { .. } => {
                panic!("uninstalled fixture must stay unavailable")
            }
        }
        let _ = std::fs::remove_dir_all(fixture_root);
    }

    #[test]
    fn m6d02_ordinary_composition_clones_m4_m3_repository_before_conversation_move() {
        let lib = include_str!("lib.rs");
        assert!(lib.contains("mod m6_org_global_role_session;"));
        assert!(lib.contains("m6_org_global_role_session:"));
        assert!(lib.contains("m4_secretary_installation.repository.clone()"));
        assert!(lib.contains("install_ordinary_product_runtime("));
        assert!(lib.contains("m6_repository"));
        assert!(lib.contains("SharedProductAuthorityProfile::IsolatedUninstalled"));
        assert!(lib.contains("M6OrgGlobalRoleSessionSlot::unavailable()"));
        let start = lib
            .find("let m6_repository = m4_secretary_installation.repository.clone();")
            .expect("clone M4 M3 repository before conversation move");
        let end = lib[start..]
            .find("m5_store_path: Some(install_m5_store_path(app_data_root)?)")
            .expect("ordinary slot assignment bound");
        let install_span = &lib[start..start + end];
        assert!(install_span.contains("OrdinaryInstalled"));
        assert!(install_span.contains("IsolatedUninstalled"));
        assert!(!install_span.contains("std::env::var"));
        assert!(!install_span.contains("SYN_M6"));
    }
}
