//! Server-only M3 project-role-session authority for M3O01 / M3O02 / M3O03.
//!
//! Provision / load / restore consume a typed `M1ProjectId`, revalidate it
//! with the M1 restricted verifier, then consume the M3-owned
//! ProjectRoleIdentitySource. They do not accept raw claim strings, path /
//! locator / M5 material, or a caller-selected role_session_id.

#![allow(dead_code)]

use crate::m1_project_index::{M1ProjectId, M1TypedProjectIdVerifierHandle};
use crate::m3_project_role_identity_source::{
    M3ProjectRoleIdentityBundle, M3ProjectRoleIdentitySourceHandle,
};
use crate::m3_role_session::{
    CorrelationId, OpaqueRef, RequestIdempotencyKey, RoleSession, RoleSessionState,
    ServerResolvedBinding,
};
use crate::m3_role_session_repository::{
    CreateRoleSessionCommand, M3CommandMetadata, M3OrdinaryRoleSessionRepositoryConfig,
    M3RoleSessionSnapshotQuery, M3RoleSessionSqliteRepository,
    M3_ORDINARY_ROLE_SESSION_RELATIVE_PATH,
};
use std::path::Path;

pub(crate) const M3_PROJECT_ROLE_SESSION_AUTHORITY_PORT_VERSION: &str =
    "m3.project-role-session-authority.port.v3";

pub(crate) const M3_AUTHORITY_UNAVAILABLE: &str = "m3_project_role_session_authority_unavailable";
pub(crate) const M3_PROJECT_ID_VERIFIER_UNAVAILABLE: &str = "m3_project_id_verifier_unavailable";
pub(crate) const M3_IDENTITY_SOURCE_UNAVAILABLE: &str = "m3_identity_source_unavailable";
pub(crate) const M3_BINDING_DRIFT: &str = "m3_project_role_session_binding_drift";
pub(crate) const M3_PERMISSION_DRIFT: &str = "m3_project_role_session_permission_drift";
pub(crate) const M3_SESSION_UNAVAILABLE: &str = "m3_project_role_session_unavailable";
pub(crate) const M3_SESSION_INACTIVE: &str = "m3_project_role_session_inactive";
pub(crate) const M3_SESSION_DUPLICATE: &str = "m3_project_role_session_duplicate";

pub(crate) use crate::m3_project_role_identity_source::M3ProjectRole;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M3ProjectRoleSessionView {
    pub(crate) project_id: String,
    pub(crate) role: M3ProjectRole,
    pub(crate) actor_id: String,
    pub(crate) role_session_id: String,
    pub(crate) binding: String,
    pub(crate) permission_snapshot: String,
    pub(crate) owner_fingerprint: String,
    pub(crate) session_revision: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M3ProjectRoleSessionRequest {
    pub(crate) project_id: M1ProjectId,
    pub(crate) role: M3ProjectRole,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M3ProjectRoleSessionRestoreRequest {
    pub(crate) project_id: M1ProjectId,
    pub(crate) role: M3ProjectRole,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M3ProjectRoleSessionAuthorityError {
    pub(crate) code: String,
}

impl M3ProjectRoleSessionAuthorityError {
    pub(crate) fn new(code: impl Into<String>) -> Self {
        Self { code: code.into() }
    }

    pub(crate) fn unavailable() -> Self {
        Self::new(M3_AUTHORITY_UNAVAILABLE)
    }
}

impl std::fmt::Display for M3ProjectRoleSessionAuthorityError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.code)
    }
}

impl std::error::Error for M3ProjectRoleSessionAuthorityError {}

pub(crate) trait M3ProjectRoleSessionAuthorityPort {
    fn provision(
        &self,
        request: &M3ProjectRoleSessionRequest,
    ) -> Result<M3ProjectRoleSessionView, M3ProjectRoleSessionAuthorityError>;
    fn load(
        &self,
        request: &M3ProjectRoleSessionRequest,
    ) -> Result<M3ProjectRoleSessionView, M3ProjectRoleSessionAuthorityError>;
    fn restore(
        &self,
        request: &M3ProjectRoleSessionRestoreRequest,
    ) -> Result<M3ProjectRoleSessionView, M3ProjectRoleSessionAuthorityError>;
}

#[derive(Clone, Debug)]
pub(crate) struct M3ProjectRoleSessionAuthorityHandle {
    verifier: Option<M1TypedProjectIdVerifierHandle>,
    identity_source: Option<M3ProjectRoleIdentitySourceHandle>,
}

impl M3ProjectRoleSessionAuthorityHandle {
    pub(crate) fn install_ordinary_product(
        verifier: M1TypedProjectIdVerifierHandle,
        app_data_root: &Path,
    ) -> Result<Self, M3ProjectRoleSessionAuthorityError> {
        let identity_source =
            M3ProjectRoleIdentitySourceHandle::install_ordinary_product(app_data_root)
                .map_err(|error| M3ProjectRoleSessionAuthorityError::new(error.code))?;
        Ok(Self {
            verifier: Some(verifier),
            identity_source: Some(identity_source),
        })
    }

    #[cfg(test)]
    pub(crate) fn install_without_verifier() -> Self {
        Self {
            verifier: None,
            identity_source: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn install_with_verifier_only(verifier: M1TypedProjectIdVerifierHandle) -> Self {
        Self {
            verifier: Some(verifier),
            identity_source: None,
        }
    }

    fn revalidate_typed_project_id(
        &self,
        project_id: &M1ProjectId,
    ) -> Result<M1ProjectId, M3ProjectRoleSessionAuthorityError> {
        let verifier = self.verifier.as_ref().ok_or_else(|| {
            M3ProjectRoleSessionAuthorityError::new(M3_PROJECT_ID_VERIFIER_UNAVAILABLE)
        })?;
        verifier
            .verify_typed_project_id(project_id)
            .map_err(|error| M3ProjectRoleSessionAuthorityError::new(error.code))
    }

    fn require_identity_source(
        &self,
    ) -> Result<&M3ProjectRoleIdentitySourceHandle, M3ProjectRoleSessionAuthorityError> {
        self.identity_source
            .as_ref()
            .ok_or_else(|| M3ProjectRoleSessionAuthorityError::new(M3_IDENTITY_SOURCE_UNAVAILABLE))
    }

    fn open_repository(
        source: &M3ProjectRoleIdentitySourceHandle,
        create_if_missing: bool,
    ) -> Result<M3RoleSessionSqliteRepository, M3ProjectRoleSessionAuthorityError> {
        let root = source.ordinary_app_data_root();
        let db_path = root.join(M3_ORDINARY_ROLE_SESSION_RELATIVE_PATH);
        if !create_if_missing && !db_path.exists() {
            return Err(M3ProjectRoleSessionAuthorityError::new(
                M3_SESSION_UNAVAILABLE,
            ));
        }
        M3RoleSessionSqliteRepository::open_ordinary_product(
            &M3OrdinaryRoleSessionRepositoryConfig {
                app_data_root: root.to_path_buf(),
                db_path,
            },
        )
        .map_err(|error| M3ProjectRoleSessionAuthorityError::new(error.code))
    }

    fn load_matching_session(
        repository: &M3RoleSessionSqliteRepository,
        bundle: &M3ProjectRoleIdentityBundle,
    ) -> Result<Option<RoleSession>, M3ProjectRoleSessionAuthorityError> {
        let binding = bundle
            .server_binding()
            .map_err(|error| M3ProjectRoleSessionAuthorityError::new(error.code))?;
        let role_session_id = bundle
            .bound_role_session_id()
            .map_err(|error| M3ProjectRoleSessionAuthorityError::new(error.code))?;
        let snapshot = repository
            .load_authorized_role_session_snapshot(&M3RoleSessionSnapshotQuery {
                role_session_id,
                binding: binding.clone(),
            })
            .map_err(map_repository_error)?;
        match snapshot {
            None => Ok(None),
            Some(snapshot) => {
                verify_session_matches_bundle(&snapshot.session, bundle, &binding)?;
                Ok(Some(snapshot.session))
            }
        }
    }

    fn provision_session(
        &self,
        request: &M3ProjectRoleSessionRequest,
    ) -> Result<M3ProjectRoleSessionView, M3ProjectRoleSessionAuthorityError> {
        let project_id = self.revalidate_typed_project_id(&request.project_id)?;
        let source = self.require_identity_source()?;
        let bundle = source
            .prepare_or_continue_provision(&project_id, request.role)
            .map_err(|error| M3ProjectRoleSessionAuthorityError::new(error.code))?;
        let repository = Self::open_repository(source, true)?;
        reject_duplicate_active_sessions(&repository, &bundle)?;
        let session = match Self::load_matching_session(&repository, &bundle)? {
            Some(session) => session,
            None if bundle.readable => {
                return Err(M3ProjectRoleSessionAuthorityError::new(
                    M3_SESSION_UNAVAILABLE,
                ));
            }
            None => {
                create_bound_role_session(&repository, &bundle)?;
                Self::load_matching_session(&repository, &bundle)?.ok_or_else(|| {
                    M3ProjectRoleSessionAuthorityError::new(M3_SESSION_UNAVAILABLE)
                })?
            }
        };
        reject_duplicate_active_sessions(&repository, &bundle)?;
        let readable = if bundle.readable {
            bundle
        } else {
            source
                .mark_readable_if_exact_match(&bundle)
                .map_err(|error| M3ProjectRoleSessionAuthorityError::new(error.code))?
        };
        view_from_bundle_and_session(&readable, &session)
    }

    fn load_session(
        &self,
        project_id: &M1ProjectId,
        role: M3ProjectRole,
    ) -> Result<M3ProjectRoleSessionView, M3ProjectRoleSessionAuthorityError> {
        let project_id = self.revalidate_typed_project_id(project_id)?;
        let source = self.require_identity_source()?;
        let bundle = source
            .load_readable(&project_id, role)
            .map_err(|error| M3ProjectRoleSessionAuthorityError::new(error.code))?;
        let repository = Self::open_repository(source, false)?;
        reject_duplicate_active_sessions(&repository, &bundle)?;
        let session = Self::load_matching_session(&repository, &bundle)?
            .ok_or_else(|| M3ProjectRoleSessionAuthorityError::new(M3_SESSION_UNAVAILABLE))?;
        view_from_bundle_and_session(&bundle, &session)
    }
}

/// Server-only AppState slot boundary. A missing handle is the uninstalled
/// authority, not an invitation for the caller to mint a session or a code.
pub(crate) fn require_installed_authority(
    slot: Option<&M3ProjectRoleSessionAuthorityHandle>,
) -> Result<&dyn M3ProjectRoleSessionAuthorityPort, M3ProjectRoleSessionAuthorityError> {
    slot.map(|handle| handle as &dyn M3ProjectRoleSessionAuthorityPort)
        .ok_or_else(M3ProjectRoleSessionAuthorityError::unavailable)
}

impl M3ProjectRoleSessionAuthorityPort for M3ProjectRoleSessionAuthorityHandle {
    fn provision(
        &self,
        request: &M3ProjectRoleSessionRequest,
    ) -> Result<M3ProjectRoleSessionView, M3ProjectRoleSessionAuthorityError> {
        self.provision_session(request)
    }

    fn load(
        &self,
        request: &M3ProjectRoleSessionRequest,
    ) -> Result<M3ProjectRoleSessionView, M3ProjectRoleSessionAuthorityError> {
        self.load_session(&request.project_id, request.role)
    }

    fn restore(
        &self,
        request: &M3ProjectRoleSessionRestoreRequest,
    ) -> Result<M3ProjectRoleSessionView, M3ProjectRoleSessionAuthorityError> {
        self.load_session(&request.project_id, request.role)
    }
}

fn create_bound_role_session(
    repository: &M3RoleSessionSqliteRepository,
    bundle: &M3ProjectRoleIdentityBundle,
) -> Result<(), M3ProjectRoleSessionAuthorityError> {
    let binding = bundle
        .server_binding()
        .map_err(|error| M3ProjectRoleSessionAuthorityError::new(error.code))?;
    let role_session_id = bundle
        .bound_role_session_id()
        .map_err(|error| M3ProjectRoleSessionAuthorityError::new(error.code))?;
    let occurred_at = repository
        .capture_server_utc_now()
        .map_err(map_repository_error)?;
    let metadata = provision_metadata(bundle, occurred_at)?;
    repository
        .create_role_session(&CreateRoleSessionCommand {
            role_session_id,
            binding,
            metadata,
        })
        .map_err(map_repository_error)?;
    Ok(())
}

fn provision_metadata(
    bundle: &M3ProjectRoleIdentityBundle,
    occurred_at: String,
) -> Result<M3CommandMetadata, M3ProjectRoleSessionAuthorityError> {
    let material = format!(
        "syn.m3.project-role-identity-source.v1|create|{}|{}",
        bundle.project_id,
        bundle.role.as_str()
    );
    Ok(M3CommandMetadata {
        receipt_id: opaque_ref("receipt", &format!("{material}|receipt"))?,
        event_id: opaque_ref("event", &format!("{material}|event"))?,
        audit_id: opaque_ref("audit", &format!("{material}|audit"))?,
        correlation_id: CorrelationId::try_from_canonical(opaque_string(
            "correlation",
            &format!("{material}|correlation"),
        )?)
        .map_err(|_| M3ProjectRoleSessionAuthorityError::new(M3_SESSION_UNAVAILABLE))?,
        request_idempotency_key: RequestIdempotencyKey::try_from_canonical(opaque_string(
            "idempotency",
            &format!("{material}|idempotency"),
        )?)
        .map_err(|_| M3ProjectRoleSessionAuthorityError::new(M3_SESSION_UNAVAILABLE))?,
        occurred_at,
    })
}

fn opaque_ref(
    namespace: &str,
    material: &str,
) -> Result<OpaqueRef, M3ProjectRoleSessionAuthorityError> {
    OpaqueRef::try_from_canonical(opaque_string(namespace, material)?)
        .map_err(|_| M3ProjectRoleSessionAuthorityError::new(M3_SESSION_UNAVAILABLE))
}

fn opaque_string(
    namespace: &str,
    material: &str,
) -> Result<String, M3ProjectRoleSessionAuthorityError> {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(material.as_bytes());
    Ok(format!("{namespace}:sha256:{:x}", hasher.finalize()))
}

fn verify_session_matches_bundle(
    session: &RoleSession,
    bundle: &M3ProjectRoleIdentityBundle,
    binding: &ServerResolvedBinding,
) -> Result<(), M3ProjectRoleSessionAuthorityError> {
    if session.role_session_id.as_str() != bundle.role_session_id {
        return Err(M3ProjectRoleSessionAuthorityError::new(M3_BINDING_DRIFT));
    }
    if !session.matches_binding_identity(binding)
        || session.owner_fingerprint.as_str() != bundle.owner_fingerprint
    {
        return Err(M3ProjectRoleSessionAuthorityError::new(M3_BINDING_DRIFT));
    }
    if session.permission_snapshot_ref.as_str() != bundle.permission_snapshot_ref {
        return Err(M3ProjectRoleSessionAuthorityError::new(M3_PERMISSION_DRIFT));
    }
    if session.status != RoleSessionState::Active {
        return Err(M3ProjectRoleSessionAuthorityError::new(M3_SESSION_INACTIVE));
    }
    Ok(())
}

fn reject_duplicate_active_sessions(
    repository: &M3RoleSessionSqliteRepository,
    bundle: &M3ProjectRoleIdentityBundle,
) -> Result<(), M3ProjectRoleSessionAuthorityError> {
    let binding = bundle
        .server_binding()
        .map_err(|error| M3ProjectRoleSessionAuthorityError::new(error.code))?;
    let active = repository
        .list_active_sessions_for_project_role(&binding.role_ref, &binding.scope_ref)
        .map_err(map_repository_error)?;
    if active.len() > 1 {
        return Err(M3ProjectRoleSessionAuthorityError::new(
            M3_SESSION_DUPLICATE,
        ));
    }
    Ok(())
}

fn view_from_bundle_and_session(
    bundle: &M3ProjectRoleIdentityBundle,
    session: &RoleSession,
) -> Result<M3ProjectRoleSessionView, M3ProjectRoleSessionAuthorityError> {
    Ok(M3ProjectRoleSessionView {
        project_id: bundle.project_id.clone(),
        role: bundle.role,
        actor_id: bundle.actor_id.clone(),
        role_session_id: bundle.role_session_id.clone(),
        binding: bundle.owner_fingerprint.clone(),
        permission_snapshot: bundle.permission_snapshot_ref.clone(),
        owner_fingerprint: bundle.owner_fingerprint.clone(),
        session_revision: session.revision,
    })
}

fn map_repository_error(
    error: crate::m3_role_session_repository::M3RoleSessionRepositoryError,
) -> M3ProjectRoleSessionAuthorityError {
    let code = if error.code.contains("binding") {
        M3_BINDING_DRIFT
    } else if error.code.contains("permission") {
        M3_PERMISSION_DRIFT
    } else {
        M3_SESSION_UNAVAILABLE
    };
    M3ProjectRoleSessionAuthorityError::new(code)
}

#[cfg(test)]
mod m3_project_role_session_authority_tests {
    use super::*;
    use crate::m1_project_index::{
        M1ProjectIndexAuthorityHandle, M1RegisterExactAliasRequest, M1_ORDINARY_APP_DATA_DIR_NAME,
        M1_ORDINARY_REGISTRY_RELATIVE_PATH, M1_PROJECT_ID_FOREIGN_ROOT,
        M1_PROJECT_INDEX_UNAVAILABLE, M1_TYPED_PROJECT_ID_VERIFIER_PORT_VERSION,
    };
    use crate::m3_project_role_identity_source::{
        M3_IDENTITY_SOURCE_MISSING, M3_IDENTITY_SOURCE_NOT_READABLE,
        M3_PROJECT_ROLE_IDENTITY_SOURCE_PORT_VERSION,
    };
    use std::path::{Path, PathBuf};

    fn request(project_id: M1ProjectId, role: M3ProjectRole) -> M3ProjectRoleSessionRequest {
        M3ProjectRoleSessionRequest { project_id, role }
    }

    fn restore_request(
        project_id: M1ProjectId,
        role: M3ProjectRole,
    ) -> M3ProjectRoleSessionRestoreRequest {
        M3ProjectRoleSessionRestoreRequest { project_id, role }
    }

    fn assert_code(
        result: Result<M3ProjectRoleSessionView, M3ProjectRoleSessionAuthorityError>,
        code: &str,
    ) {
        match result {
            Ok(_) => panic!("expected fail-closed {code}, got a session view"),
            Err(error) => assert_eq!(error.code, code),
        }
    }

    fn ordinary_named_root() -> PathBuf {
        let parent = std::env::temp_dir().join(format!(
            "m3o03-ordinary-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        let root = parent.join(M1_ORDINARY_APP_DATA_DIR_NAME);
        std::fs::create_dir_all(&root).expect("create ordinary app-data root");
        std::fs::canonicalize(&root).expect("canonicalize ordinary app-data root")
    }

    fn ordinary_app_state(app_data_root: &Path) -> crate::AppState {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        crate::AppState::try_new_with_ordinary_product_ports(
            app_data_root,
            &manifest_dir.join("../../index-kernel/codex-index.json"),
            &manifest_dir.join("../../../tasks/README.md"),
            crate::m4_secretary_conversation::M4SecretaryConversationProviderConfig::Unavailable,
        )
        .expect("ordinary product AppState must construct")
    }

    fn isolated_acceptance_app_state() -> (PathBuf, crate::AppState) {
        let root = std::env::temp_dir().join(format!(
            "m3o03-isolated-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(root.join("app-data")).expect("create isolated profile");
        let root = std::fs::canonicalize(&root).expect("canonicalize isolated profile");
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let paths = crate::acceptance_runtime_profile::RuntimePaths {
            root: root.clone(),
            index_path: manifest_dir.join("../../index-kernel/codex-index.json"),
            tasks_path: manifest_dir.join("../../../tasks/README.md"),
            project_root: root.join("project"),
            workflow_state_path: root.join("workflow-state.json"),
            app_data_root: root.join("app-data"),
            vault_root: root.join("vault"),
            recovery_backups_root: root.join("recovery"),
            canvas_root: root.join("canvas"),
            codex_db_path: root.join("codex.sqlite"),
            app_log_dir: root.join("logs"),
        };
        let state = crate::AppState::try_new_with_isolated_product_profile(&paths)
            .expect("isolated acceptance AppState must construct");
        (root, state)
    }

    fn m3_role_session_count(app_data_root: &Path) -> i64 {
        let path = app_data_root.join("conversation/m3-role-session-v1.sqlite3");
        if !path.exists() {
            return 0;
        }
        let connection = rusqlite::Connection::open(path).expect("open m3 db");
        connection
            .query_row("SELECT COUNT(*) FROM m3_role_sessions", [], |row| {
                row.get(0)
            })
            .unwrap_or(0)
    }

    fn register_typed_id(app_data_root: &Path, alias: &str) -> M1ProjectId {
        M1ProjectIndexAuthorityHandle::install_ordinary_product(app_data_root)
            .expect("install m1")
            .register_exact_alias(&M1RegisterExactAliasRequest {
                exact_alias: alias.to_string(),
            })
            .expect("register typed id")
            .project_id
    }

    fn assert_unavailable_slot(state: &crate::AppState) {
        match state.m3_project_role_session_authority_port() {
            Ok(_) => panic!("uninstalled AppState slot must not yield an authority port"),
            Err(error) => assert_eq!(error.code, M3_AUTHORITY_UNAVAILABLE),
        }
    }

    fn _request_project_id_is_typed(request: &M3ProjectRoleSessionRequest) -> &M1ProjectId {
        &request.project_id
    }

    fn _restore_request_has_no_caller_session_id(
        request: &M3ProjectRoleSessionRestoreRequest,
    ) -> &M1ProjectId {
        &request.project_id
    }

    #[test]
    fn m3_request_api_consumes_typed_m1_project_id_not_raw_claim() {
        let source = include_str!("m3_project_role_session_authority.rs");
        assert!(source.contains("pub(crate) project_id: M1ProjectId"));
        assert!(
            !source.contains(&format!("{}_{}", "project_id", "claim")),
            "raw string claim field must not remain on the M3 request API"
        );
        assert!(source.contains(M3_IDENTITY_SOURCE_UNAVAILABLE));
        assert_ne!(
            M3_PROJECT_ROLE_SESSION_AUTHORITY_PORT_VERSION,
            M1_TYPED_PROJECT_ID_VERIFIER_PORT_VERSION
        );
        assert_ne!(
            M3_PROJECT_ROLE_SESSION_AUTHORITY_PORT_VERSION,
            M3_PROJECT_ROLE_IDENTITY_SOURCE_PORT_VERSION
        );
        assert!(!source.contains(&format!("resolve_{}", "m4_primary_secretary")));
        assert!(!source.contains(&format!("resolve_{}(", "identity")));
        assert!(!source.contains(&format!("official_{}", "project_id")));
        assert!(!source.contains(&format!("m5_{}", "m3_identity")));
        assert!(!source.contains(&format!("{}::from_root", "ProjectId")));
        assert!(!source.contains(&format!("resume_{}", "role_session")));
        let restore_start = source
            .find("struct M3ProjectRoleSessionRestoreRequest")
            .expect("restore request type");
        let restore_block = &source[restore_start..restore_start + 220];
        assert!(
            !restore_block.contains("role_session_id"),
            "restore must not accept a caller-selected role_session_id"
        );
    }

    #[test]
    fn m3_project_role_session_authority_uninstalled_app_state_returns_unavailable() {
        let legacy = crate::AppState::try_new().expect("legacy AppState must construct");
        assert_unavailable_slot(&legacy);
        let fixture = crate::AppState {
            index_path: PathBuf::from("/m3o03/fixture/index.json"),
            tasks_path: PathBuf::from("/m3o03/fixture/tasks.md"),
            workflow_state_path: PathBuf::from("/m3o03/fixture/workflow-state.json"),
            m3_role_session_read_runtime: Default::default(),
            m1_project_index: None,
            m3_project_role_session_authority: None,
            m5_store_path: None,
            m6_org_global_role_session: Default::default(),
        };
        assert_unavailable_slot(&fixture);
    }

    #[test]
    fn m3_project_role_session_authority_isolated_acceptance_app_state_returns_unavailable() {
        let (root, isolated) = isolated_acceptance_app_state();
        assert_unavailable_slot(&isolated);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn m3_project_role_session_authority_without_verifier_fails_before_writes() {
        let root = ordinary_named_root();
        let project_id = register_typed_id(&root, "syn-m3o03-no-verifier");
        let before = m3_role_session_count(&root);
        let port = M3ProjectRoleSessionAuthorityHandle::install_without_verifier();
        assert_code(
            port.provision(&request(
                project_id.clone(),
                M3ProjectRole::ProjectSupervisor,
            )),
            M3_PROJECT_ID_VERIFIER_UNAVAILABLE,
        );
        assert_code(
            port.load(&request(
                project_id.clone(),
                M3ProjectRole::ProjectSupervisor,
            )),
            M3_PROJECT_ID_VERIFIER_UNAVAILABLE,
        );
        assert_code(
            port.restore(&restore_request(project_id, M3ProjectRole::Worker)),
            M3_PROJECT_ID_VERIFIER_UNAVAILABLE,
        );
        assert_eq!(m3_role_session_count(&root), before);
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }

    #[test]
    fn m3_verifier_without_identity_source_fails_closed_without_writes() {
        let root = ordinary_named_root();
        let project_id = register_typed_id(&root, "syn-m3o03-no-source");
        let verifier = M1ProjectIndexAuthorityHandle::install_ordinary_product(&root)
            .expect("install m1")
            .restricted_typed_project_id_verifier();
        let port = M3ProjectRoleSessionAuthorityHandle::install_with_verifier_only(verifier);
        let before = m3_role_session_count(&root);
        assert_code(
            port.provision(&request(
                project_id.clone(),
                M3ProjectRole::ProjectSupervisor,
            )),
            M3_IDENTITY_SOURCE_UNAVAILABLE,
        );
        assert_code(
            port.load(&request(project_id.clone(), M3ProjectRole::Worker)),
            M3_IDENTITY_SOURCE_UNAVAILABLE,
        );
        assert_code(
            port.restore(&restore_request(
                project_id,
                M3ProjectRole::IndependentReviewer,
            )),
            M3_IDENTITY_SOURCE_UNAVAILABLE,
        );
        assert_eq!(m3_role_session_count(&root), before);
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }

    #[test]
    fn m3_ordinary_provision_is_idempotent_and_load_restore_use_source_bound_session() {
        let root = ordinary_named_root();
        let state = ordinary_app_state(&root);
        let project_id = state
            .m1_project_index_authority()
            .expect("ordinary m1")
            .register_exact_alias(&M1RegisterExactAliasRequest {
                exact_alias: "syn-m3o03-same-root".to_string(),
            })
            .expect("register")
            .project_id;
        let port = state
            .m3_project_role_session_authority_port()
            .expect("ordinary m3");
        let before = m3_role_session_count(&root);
        let first = port
            .provision(&request(
                project_id.clone(),
                M3ProjectRole::ProjectSupervisor,
            ))
            .expect("provision");
        let second = port
            .provision(&request(
                project_id.clone(),
                M3ProjectRole::ProjectSupervisor,
            ))
            .expect("idempotent provision");
        assert_eq!(first, second);
        assert_eq!(m3_role_session_count(&root), before + 1);
        let loaded = port
            .load(&request(
                project_id.clone(),
                M3ProjectRole::ProjectSupervisor,
            ))
            .expect("load");
        assert_eq!(loaded.role_session_id, first.role_session_id);
        let restored = port
            .restore(&restore_request(
                project_id.clone(),
                M3ProjectRole::ProjectSupervisor,
            ))
            .expect("restore");
        assert_eq!(restored.role_session_id, first.role_session_id);
        assert_code(
            port.load(&request(project_id.clone(), M3ProjectRole::Worker)),
            M3_IDENTITY_SOURCE_MISSING,
        );
        assert_code(
            port.restore(&restore_request(project_id.clone(), M3ProjectRole::Worker)),
            M3_IDENTITY_SOURCE_MISSING,
        );

        let worker = port
            .provision(&request(project_id.clone(), M3ProjectRole::Worker))
            .expect("provision worker");
        let reviewer = port
            .provision(&request(
                project_id.clone(),
                M3ProjectRole::IndependentReviewer,
            ))
            .expect("provision reviewer");
        assert_ne!(first.actor_id, worker.actor_id);
        assert_ne!(first.actor_id, reviewer.actor_id);
        assert_ne!(first.owner_fingerprint, reviewer.owner_fingerprint);
        assert_eq!(m3_role_session_count(&root), before + 3);

        let store = std::fs::read_to_string(root.join("m3/project-role-identity-source-v1.json"))
            .expect("read identity source");
        assert!(!store.contains(root.to_string_lossy().as_ref()));
        assert!(!store.contains("ExecutionGrant"));
        assert!(store.contains("\"allow_capabilities\": []"));

        let lib = include_str!("lib.rs");
        assert!(
            lib.contains("restricted_typed_project_id_verifier()"),
            "ordinary composition must wire the restricted M1 verifier"
        );
        assert!(
            lib.contains("SharedProductAuthorityProfile::IsolatedUninstalled"),
            "isolated acceptance must explicitly leave M1/M3 uninstalled"
        );
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }

    #[test]
    fn m3_deleted_established_source_json_fails_closed_and_does_not_rebuild() {
        let root = ordinary_named_root();
        let state = ordinary_app_state(&root);
        let project_id = state
            .m1_project_index_authority()
            .expect("ordinary m1")
            .register_exact_alias(&M1RegisterExactAliasRequest {
                exact_alias: "syn-m3o03-deleted-source".to_string(),
            })
            .expect("register")
            .project_id;
        let port = state
            .m3_project_role_session_authority_port()
            .expect("ordinary m3");
        let _view = port
            .provision(&request(
                project_id.clone(),
                M3ProjectRole::ProjectSupervisor,
            ))
            .expect("first provision");
        let source_path = root.join("m3/project-role-identity-source-v1.json");
        let marker_path = root.join(".m3-project-role-identity-source.established");
        assert!(source_path.is_file());
        assert!(marker_path.is_file());
        let before = m3_role_session_count(&root);
        std::fs::remove_file(&source_path).expect("delete established source json");
        assert_code(
            port.provision(&request(
                project_id.clone(),
                M3ProjectRole::ProjectSupervisor,
            )),
            M3_IDENTITY_SOURCE_MISSING,
        );
        assert_code(
            port.load(&request(
                project_id.clone(),
                M3ProjectRole::ProjectSupervisor,
            )),
            M3_IDENTITY_SOURCE_MISSING,
        );
        assert_code(
            port.restore(&restore_request(
                project_id,
                M3ProjectRole::ProjectSupervisor,
            )),
            M3_IDENTITY_SOURCE_MISSING,
        );
        assert!(!source_path.exists());
        assert!(marker_path.is_file());
        assert_eq!(m3_role_session_count(&root), before);
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }

    #[test]
    fn m3_duplicate_active_project_role_sessions_fail_closed() {
        let root = ordinary_named_root();
        let state = ordinary_app_state(&root);
        let project_id = state
            .m1_project_index_authority()
            .expect("ordinary m1")
            .register_exact_alias(&M1RegisterExactAliasRequest {
                exact_alias: "syn-m3o03-duplicate-active".to_string(),
            })
            .expect("register")
            .project_id;
        let port = state
            .m3_project_role_session_authority_port()
            .expect("ordinary m3");
        let view = port
            .provision(&request(project_id.clone(), M3ProjectRole::Worker))
            .expect("first provision");
        let db_path = root.join("conversation/m3-role-session-v1.sqlite3");
        let connection = rusqlite::Connection::open(&db_path).expect("open m3 db");
        connection
            .execute(
                "INSERT INTO m3_role_sessions (
                     role_session_id, actor_id, role_ref, scope_ref, current_object_ref,
                     execution_channel, permission_snapshot_ref, owner_fingerprint, state,
                     revision, created_at, last_resumed_at, resolution_reason
                 )
                 SELECT
                     'session:sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd',
                     actor_id, role_ref, scope_ref, current_object_ref,
                     execution_channel, permission_snapshot_ref, owner_fingerprint, 'ACTIVE',
                     revision, created_at, last_resumed_at, resolution_reason
                 FROM m3_role_sessions
                 WHERE role_session_id = ?1",
                rusqlite::params![view.role_session_id],
            )
            .expect("insert extra active same project/role session");
        assert_code(
            port.provision(&request(project_id.clone(), M3ProjectRole::Worker)),
            M3_SESSION_DUPLICATE,
        );
        assert_code(
            port.load(&request(project_id.clone(), M3ProjectRole::Worker)),
            M3_SESSION_DUPLICATE,
        );
        assert_code(
            port.restore(&restore_request(project_id, M3ProjectRole::Worker)),
            M3_SESSION_DUPLICATE,
        );
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }

    #[test]
    fn m3_load_restore_do_not_create_or_complete_prepared_identity() {
        let root = ordinary_named_root();
        let state = ordinary_app_state(&root);
        let project_id = state
            .m1_project_index_authority()
            .expect("ordinary m1")
            .register_exact_alias(&M1RegisterExactAliasRequest {
                exact_alias: "syn-m3o03-no-create".to_string(),
            })
            .expect("register")
            .project_id;
        let source = M3ProjectRoleIdentitySourceHandle::install_ordinary_product(&root)
            .expect("install source");
        let prepared = source
            .prepare_or_continue_provision(&project_id, M3ProjectRole::Worker)
            .expect("leave prepared");
        assert!(!prepared.readable);
        let port = state
            .m3_project_role_session_authority_port()
            .expect("ordinary m3");
        let before = m3_role_session_count(&root);
        assert_code(
            port.load(&request(project_id.clone(), M3ProjectRole::Worker)),
            M3_IDENTITY_SOURCE_NOT_READABLE,
        );
        assert_code(
            port.restore(&restore_request(project_id.clone(), M3ProjectRole::Worker)),
            M3_IDENTITY_SOURCE_NOT_READABLE,
        );
        assert_eq!(m3_role_session_count(&root), before);
        let completed = port
            .provision(&request(project_id, M3ProjectRole::Worker))
            .expect("same input completes prepared");
        assert_eq!(completed.role_session_id, prepared.role_session_id);
        assert_eq!(m3_role_session_count(&root), before + 1);
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }

    #[test]
    fn m3_inactive_or_missing_session_and_permission_drift_fail_closed() {
        let root = ordinary_named_root();
        let state = ordinary_app_state(&root);
        let project_id = state
            .m1_project_index_authority()
            .expect("ordinary m1")
            .register_exact_alias(&M1RegisterExactAliasRequest {
                exact_alias: "syn-m3o03-session-faults".to_string(),
            })
            .expect("register")
            .project_id;
        let port = state
            .m3_project_role_session_authority_port()
            .expect("ordinary m3");
        let view = port
            .provision(&request(
                project_id.clone(),
                M3ProjectRole::ProjectSupervisor,
            ))
            .expect("provision");
        let db_path = root.join("conversation/m3-role-session-v1.sqlite3");
        let connection = rusqlite::Connection::open(&db_path).expect("open m3 db");
        connection
            .execute(
                "UPDATE m3_role_sessions SET state = 'SUSPENDED', resolution_reason = 'PERMISSION_MISMATCH_OR_UNKNOWN' WHERE role_session_id = ?1",
                rusqlite::params![view.role_session_id],
            )
            .expect("suspend");
        assert_code(
            port.load(&request(
                project_id.clone(),
                M3ProjectRole::ProjectSupervisor,
            )),
            M3_SESSION_INACTIVE,
        );
        connection
            .execute(
                "UPDATE m3_role_sessions SET state = 'ACTIVE', resolution_reason = NULL, permission_snapshot_ref = ?1 WHERE role_session_id = ?2",
                rusqlite::params![
                    "permission:sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
                    view.role_session_id
                ],
            )
            .expect("drift permission");
        assert_code(
            port.restore(&restore_request(
                project_id.clone(),
                M3ProjectRole::ProjectSupervisor,
            )),
            M3_PERMISSION_DRIFT,
        );

        let missing_project = state
            .m1_project_index_authority()
            .expect("ordinary m1")
            .register_exact_alias(&M1RegisterExactAliasRequest {
                exact_alias: "syn-m3o03-readable-without-session".to_string(),
            })
            .expect("register missing-session project")
            .project_id;
        let source = M3ProjectRoleIdentitySourceHandle::install_ordinary_product(&root)
            .expect("install source");
        let prepared = source
            .prepare_or_continue_provision(&missing_project, M3ProjectRole::Worker)
            .expect("prepare unread session");
        source
            .mark_readable_if_exact_match(&prepared)
            .expect("mark readable without session");
        assert_code(
            port.load(&request(missing_project, M3ProjectRole::Worker)),
            M3_SESSION_UNAVAILABLE,
        );
        let _ = view;
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }

    #[test]
    fn m3_foreign_root_and_registry_faults_fail_before_writes() {
        let root_a = ordinary_named_root();
        let root_b = ordinary_named_root();
        let foreign_id = register_typed_id(&root_a, "syn-m3o03-foreign");
        let state_b = ordinary_app_state(&root_b);
        let port_b = state_b
            .m3_project_role_session_authority_port()
            .expect("ordinary m3 on b");
        let before_b = m3_role_session_count(&root_b);
        assert_code(
            port_b.provision(&request(
                foreign_id.clone(),
                M3ProjectRole::ProjectSupervisor,
            )),
            M1_PROJECT_ID_FOREIGN_ROOT,
        );
        assert_eq!(m3_role_session_count(&root_b), before_b);
        assert!(!root_b
            .join("m3/project-role-identity-source-v1.json")
            .exists());

        let root = ordinary_named_root();
        let state = ordinary_app_state(&root);
        let typed = state
            .m1_project_index_authority()
            .expect("ordinary m1")
            .register_exact_alias(&M1RegisterExactAliasRequest {
                exact_alias: "syn-m3o03-faults".to_string(),
            })
            .expect("register")
            .project_id;
        let port = state
            .m3_project_role_session_authority_port()
            .expect("ordinary m3");
        let registry_path = root.join(M1_ORDINARY_REGISTRY_RELATIVE_PATH);
        let marker_path = root.join(".m1-project-index.established");
        let before = m3_role_session_count(&root);

        std::fs::write(
            &registry_path,
            r#"{
              "schema_version": "m1.project-index.registry.v2",
              "registry_revision": 0,
              "projects": []
            }"#,
        )
        .expect("unknown registry");
        assert_code(
            port.load(&request(typed.clone(), M3ProjectRole::ProjectSupervisor)),
            "m1_project_id_unknown",
        );
        assert_eq!(m3_role_session_count(&root), before);

        std::fs::write(&registry_path, "{not-json").expect("corrupt");
        assert_code(
            port.restore(&restore_request(typed.clone(), M3ProjectRole::Worker)),
            "m1_project_index_registry_malformed",
        );
        assert_eq!(m3_role_session_count(&root), before);

        std::fs::remove_file(&registry_path).expect("delete registry");
        assert!(marker_path.is_file());
        assert_code(
            port.provision(&request(typed.clone(), M3ProjectRole::ProjectSupervisor)),
            "m1_project_index_registry_missing",
        );
        assert_eq!(m3_role_session_count(&root), before);

        std::fs::remove_file(&marker_path).expect("delete marker");
        std::fs::remove_dir_all(root.join("m1")).expect("delete m1 dir");
        assert_code(
            port.load(&request(typed, M3ProjectRole::ProjectSupervisor)),
            M1_PROJECT_INDEX_UNAVAILABLE,
        );
        assert_eq!(m3_role_session_count(&root), before);

        let _ = std::fs::remove_dir_all(root_a.parent().expect("parent"));
        let _ = std::fs::remove_dir_all(root_b.parent().expect("parent"));
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }
}
