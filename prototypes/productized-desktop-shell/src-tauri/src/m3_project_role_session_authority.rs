//! Server-only M3 project-role-session authority for M3O01 / M3O02.
//!
//! Provision / load / restore consume a typed `M1ProjectId` and revalidate it
//! with the M1 restricted verifier. They do not accept raw claim strings, mint
//! ProjectId, open the M3 repository, or fabricate ActorId / RoleRef / Scope /
//! binding / permission snapshot sources.

#![allow(dead_code)]

use crate::m1_project_index::{M1ProjectId, M1TypedProjectIdVerifierHandle};

pub(crate) const M3_PROJECT_ROLE_SESSION_AUTHORITY_PORT_VERSION: &str =
    "m3.project-role-session-authority.port.v2";

pub(crate) const M3_AUTHORITY_UNAVAILABLE: &str = "m3_project_role_session_authority_unavailable";
pub(crate) const M3_PROJECT_ID_VERIFIER_UNAVAILABLE: &str = "m3_project_id_verifier_unavailable";
pub(crate) const M3_IDENTITY_SOURCE_UNAVAILABLE: &str = "m3_identity_source_unavailable";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum M3ProjectRole {
    ProjectSupervisor,
    Worker,
    IndependentReviewer,
}

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
    pub(crate) role_session_id: String,
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
}

impl M3ProjectRoleSessionAuthorityHandle {
    pub(crate) fn install_ordinary_product(verifier: M1TypedProjectIdVerifierHandle) -> Self {
        Self {
            verifier: Some(verifier),
        }
    }

    #[cfg(test)]
    pub(crate) fn install_without_verifier() -> Self {
        Self { verifier: None }
    }

    fn fail_closed_after_typed_revalidation(
        &self,
        project_id: &M1ProjectId,
    ) -> Result<M3ProjectRoleSessionView, M3ProjectRoleSessionAuthorityError> {
        let verifier = self.verifier.as_ref().ok_or_else(|| {
            M3ProjectRoleSessionAuthorityError::new(M3_PROJECT_ID_VERIFIER_UNAVAILABLE)
        })?;
        verifier
            .verify_typed_project_id(project_id)
            .map_err(|error| M3ProjectRoleSessionAuthorityError::new(error.code))?;
        Err(M3ProjectRoleSessionAuthorityError::new(
            M3_IDENTITY_SOURCE_UNAVAILABLE,
        ))
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
        self.fail_closed_after_typed_revalidation(&request.project_id)
    }

    fn load(
        &self,
        request: &M3ProjectRoleSessionRequest,
    ) -> Result<M3ProjectRoleSessionView, M3ProjectRoleSessionAuthorityError> {
        self.fail_closed_after_typed_revalidation(&request.project_id)
    }

    fn restore(
        &self,
        request: &M3ProjectRoleSessionRestoreRequest,
    ) -> Result<M3ProjectRoleSessionView, M3ProjectRoleSessionAuthorityError> {
        self.fail_closed_after_typed_revalidation(&request.project_id)
    }
}

#[cfg(test)]
mod m3_project_role_session_authority_tests {
    use super::*;
    use crate::m1_project_index::{
        M1ProjectIndexAuthorityHandle, M1RegisterExactAliasRequest, M1_ORDINARY_APP_DATA_DIR_NAME,
        M1_ORDINARY_REGISTRY_RELATIVE_PATH, M1_PROJECT_ID_FOREIGN_ROOT,
        M1_PROJECT_INDEX_UNAVAILABLE, M1_TYPED_PROJECT_ID_VERIFIER_PORT_VERSION,
    };
    use std::path::{Path, PathBuf};

    fn request(project_id: M1ProjectId) -> M3ProjectRoleSessionRequest {
        M3ProjectRoleSessionRequest {
            project_id,
            role: M3ProjectRole::ProjectSupervisor,
        }
    }

    fn restore_request(project_id: M1ProjectId) -> M3ProjectRoleSessionRestoreRequest {
        M3ProjectRoleSessionRestoreRequest {
            project_id,
            role: M3ProjectRole::Worker,
            role_session_id: "role-session-fixture".to_string(),
        }
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
            "m3o02-ordinary-{}-{}",
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
            "m3o02-isolated-{}-{}",
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
        assert!(!source.contains(&format!("resolve_{}", "m4_primary_secretary")));
        assert!(!source.contains(&format!("resolve_{}(", "identity")));
        assert!(!source.contains(&format!("official_{}", "project_id")));
        assert!(!source.contains(&format!("m5_{}", "m3_identity")));
        assert!(!source.contains(&format!("create_{}", "role_session")));
        assert!(!source.contains(&format!("{}::from_root", "ProjectId")));
    }

    #[test]
    fn m3_project_role_session_authority_uninstalled_app_state_returns_unavailable() {
        let legacy = crate::AppState::try_new().expect("legacy AppState must construct");
        assert_unavailable_slot(&legacy);
        let fixture = crate::AppState {
            index_path: PathBuf::from("/m3o02/fixture/index.json"),
            tasks_path: PathBuf::from("/m3o02/fixture/tasks.md"),
            workflow_state_path: PathBuf::from("/m3o02/fixture/workflow-state.json"),
            m3_role_session_read_runtime: Default::default(),
            m1_project_index: None,
            m3_project_role_session_authority: None,
            m5_store_path: None,
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
        let project_id = register_typed_id(&root, "syn-m3o02-no-verifier");
        let before = m3_role_session_count(&root);
        let port = M3ProjectRoleSessionAuthorityHandle::install_without_verifier();
        assert_code(
            port.provision(&request(project_id.clone())),
            M3_PROJECT_ID_VERIFIER_UNAVAILABLE,
        );
        assert_code(
            port.load(&request(project_id.clone())),
            M3_PROJECT_ID_VERIFIER_UNAVAILABLE,
        );
        assert_code(
            port.restore(&restore_request(project_id)),
            M3_PROJECT_ID_VERIFIER_UNAVAILABLE,
        );
        assert_eq!(m3_role_session_count(&root), before);
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }

    #[test]
    fn m3_valid_same_root_typed_id_fails_identity_source_unavailable_without_writes() {
        let root = ordinary_named_root();
        let state = ordinary_app_state(&root);
        let project_id = state
            .m1_project_index_authority()
            .expect("ordinary m1")
            .register_exact_alias(&M1RegisterExactAliasRequest {
                exact_alias: "syn-m3o02-same-root".to_string(),
            })
            .expect("register")
            .project_id;
        let port = state
            .m3_project_role_session_authority_port()
            .expect("ordinary m3");
        let before = m3_role_session_count(&root);
        assert_code(
            port.provision(&request(project_id.clone())),
            M3_IDENTITY_SOURCE_UNAVAILABLE,
        );
        assert_code(
            port.load(&request(project_id.clone())),
            M3_IDENTITY_SOURCE_UNAVAILABLE,
        );
        assert_code(
            port.restore(&restore_request(project_id)),
            M3_IDENTITY_SOURCE_UNAVAILABLE,
        );
        assert_eq!(m3_role_session_count(&root), before);
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
    fn m3_foreign_root_and_registry_faults_fail_before_writes() {
        let root_a = ordinary_named_root();
        let root_b = ordinary_named_root();
        let foreign_id = register_typed_id(&root_a, "syn-m3o02-foreign");
        let state_b = ordinary_app_state(&root_b);
        let port_b = state_b
            .m3_project_role_session_authority_port()
            .expect("ordinary m3 on b");
        let before_b = m3_role_session_count(&root_b);
        assert_code(
            port_b.provision(&request(foreign_id.clone())),
            M1_PROJECT_ID_FOREIGN_ROOT,
        );
        assert_eq!(m3_role_session_count(&root_b), before_b);

        let root = ordinary_named_root();
        let state = ordinary_app_state(&root);
        let typed = state
            .m1_project_index_authority()
            .expect("ordinary m1")
            .register_exact_alias(&M1RegisterExactAliasRequest {
                exact_alias: "syn-m3o02-faults".to_string(),
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
        assert_code(port.load(&request(typed.clone())), "m1_project_id_unknown");
        assert_eq!(m3_role_session_count(&root), before);

        std::fs::write(&registry_path, "{not-json").expect("corrupt");
        assert_code(
            port.restore(&restore_request(typed.clone())),
            "m1_project_index_registry_malformed",
        );
        assert_eq!(m3_role_session_count(&root), before);

        std::fs::remove_file(&registry_path).expect("delete registry");
        assert!(marker_path.is_file());
        assert_code(
            port.provision(&request(typed.clone())),
            "m1_project_index_registry_missing",
        );
        assert_eq!(m3_role_session_count(&root), before);

        std::fs::remove_file(&marker_path).expect("delete marker");
        std::fs::remove_dir_all(root.join("m1")).expect("delete m1 dir");
        assert_code(port.load(&request(typed)), M1_PROJECT_INDEX_UNAVAILABLE);
        assert_eq!(m3_role_session_count(&root), before);

        let _ = std::fs::remove_dir_all(root_a.parent().expect("parent"));
        let _ = std::fs::remove_dir_all(root_b.parent().expect("parent"));
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }
}
