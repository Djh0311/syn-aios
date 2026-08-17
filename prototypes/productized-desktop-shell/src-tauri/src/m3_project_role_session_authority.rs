//! Server-only M3 project-role-session authority for M3O01.
//!
//! This port is the only legal provision / load / restore owner for the three
//! exact project roles. It does not mint ProjectId, consume path / locator /
//! scratch / M5 helper claims, or treat the M1 read port as an ordinary
//! issuance source. Ordinary product installation still fail-closes until an
//! independent authorized canonical ProjectId source exists.

#![allow(dead_code)]

use std::path::Path;

pub(crate) const M3_PROJECT_ROLE_SESSION_AUTHORITY_PORT_VERSION: &str =
    "m3.project-role-session-authority.port.v1";

pub(crate) const M3_AUTHORITY_UNAVAILABLE: &str = "m3_project_role_session_authority_unavailable";
pub(crate) const M3_CANONICAL_PROJECT_ID_SOURCE_UNAVAILABLE: &str =
    "m3_canonical_project_id_source_unavailable";
pub(crate) const M3_PROJECT_ID_PATH_CLAIM_REJECTED: &str = "m3_project_id_path_claim_rejected";
pub(crate) const M3_PROJECT_ID_INDEX_LOCATOR_CLAIM_REJECTED: &str =
    "m3_project_id_index_locator_claim_rejected";
pub(crate) const M3_PROJECT_ID_SCRATCH_CLAIM_REJECTED: &str = "m3_project_id_scratch_claim_rejected";
pub(crate) const M3_PROJECT_ID_M5_HELPER_CLAIM_REJECTED: &str =
    "m3_project_id_m5_helper_claim_rejected";

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
    pub(crate) project_id_claim: String,
    pub(crate) role: M3ProjectRole,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M3ProjectRoleSessionRestoreRequest {
    pub(crate) project_id_claim: String,
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
    _ordinary_product: (),
}

impl M3ProjectRoleSessionAuthorityHandle {
    pub(crate) fn install_ordinary_product() -> Self {
        Self {
            _ordinary_product: (),
        }
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
        fail_closed_without_ordinary_canonical_source(&request.project_id_claim)
    }

    fn load(
        &self,
        request: &M3ProjectRoleSessionRequest,
    ) -> Result<M3ProjectRoleSessionView, M3ProjectRoleSessionAuthorityError> {
        fail_closed_without_ordinary_canonical_source(&request.project_id_claim)
    }

    fn restore(
        &self,
        request: &M3ProjectRoleSessionRestoreRequest,
    ) -> Result<M3ProjectRoleSessionView, M3ProjectRoleSessionAuthorityError> {
        fail_closed_without_ordinary_canonical_source(&request.project_id_claim)
    }
}

fn fail_closed_without_ordinary_canonical_source(
    claim: &str,
) -> Result<M3ProjectRoleSessionView, M3ProjectRoleSessionAuthorityError> {
    reject_illegitimate_project_id_claim(claim)?;
    Err(M3ProjectRoleSessionAuthorityError::new(
        M3_CANONICAL_PROJECT_ID_SOURCE_UNAVAILABLE,
    ))
}

fn reject_illegitimate_project_id_claim(
    claim: &str,
) -> Result<(), M3ProjectRoleSessionAuthorityError> {
    if is_scratch_claim(claim) {
        return Err(M3ProjectRoleSessionAuthorityError::new(
            M3_PROJECT_ID_SCRATCH_CLAIM_REJECTED,
        ));
    }
    if is_m5_helper_claim(claim) {
        return Err(M3ProjectRoleSessionAuthorityError::new(
            M3_PROJECT_ID_M5_HELPER_CLAIM_REJECTED,
        ));
    }
    if is_path_claim(claim) {
        return Err(M3ProjectRoleSessionAuthorityError::new(
            M3_PROJECT_ID_PATH_CLAIM_REJECTED,
        ));
    }
    if is_index_locator_claim(claim) {
        return Err(M3ProjectRoleSessionAuthorityError::new(
            M3_PROJECT_ID_INDEX_LOCATOR_CLAIM_REJECTED,
        ));
    }
    Ok(())
}

fn is_scratch_claim(value: &str) -> bool {
    value.starts_with("scratch-") || value.starts_with("project:scratch-")
}

fn is_m5_helper_claim(value: &str) -> bool {
    value.starts_with("m5:")
        || value.contains("official_project_id")
        || value.contains("resolve_project_id_from_index")
        || value.contains("m5_m3_identity")
}

fn is_path_claim(value: &str) -> bool {
    value.contains('/')
        || value.contains('\\')
        || value.starts_with('.')
        || Path::new(value).is_absolute()
}

fn is_index_locator_claim(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with("project:")
        && !is_scratch_claim(value)
        && !is_m5_helper_claim(value)
        && !is_path_claim(value)
}

#[cfg(test)]
mod m3_project_role_session_authority_tests {
    use super::*;
    use std::path::PathBuf;

    fn request(claim: &str) -> M3ProjectRoleSessionRequest {
        M3ProjectRoleSessionRequest {
            project_id_claim: claim.to_string(),
            role: M3ProjectRole::ProjectSupervisor,
        }
    }

    fn restore_request(claim: &str) -> M3ProjectRoleSessionRestoreRequest {
        M3ProjectRoleSessionRestoreRequest {
            project_id_claim: claim.to_string(),
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

    #[test]
    fn m3_project_role_session_authority_installed_port_has_no_ordinary_canonical_source() {
        let port = M3ProjectRoleSessionAuthorityHandle::install_ordinary_product();
        assert_code(
            port.provision(&request("project:aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee")),
            M3_CANONICAL_PROJECT_ID_SOURCE_UNAVAILABLE,
        );
        assert_code(
            port.load(&request("project:aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee")),
            M3_CANONICAL_PROJECT_ID_SOURCE_UNAVAILABLE,
        );
        assert_code(
            port.restore(&restore_request(
                "project:aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
            )),
            M3_CANONICAL_PROJECT_ID_SOURCE_UNAVAILABLE,
        );
    }

    #[test]
    fn m3_project_role_session_authority_rejects_path_index_scratch_and_m5_claims() {
        let port = M3ProjectRoleSessionAuthorityHandle::install_ordinary_product();
        assert_code(
            port.provision(&request("/tmp/syn-project")),
            M3_PROJECT_ID_PATH_CLAIM_REJECTED,
        );
        assert_code(
            port.load(&request("./relative-project")),
            M3_PROJECT_ID_PATH_CLAIM_REJECTED,
        );
        assert_code(
            port.restore(&restore_request("isolated-profile-locator")),
            M3_PROJECT_ID_INDEX_LOCATOR_CLAIM_REJECTED,
        );
        assert_code(
            port.provision(&request("scratch-demo")),
            M3_PROJECT_ID_SCRATCH_CLAIM_REJECTED,
        );
        assert_code(
            port.load(&request("project:scratch-demo")),
            M3_PROJECT_ID_SCRATCH_CLAIM_REJECTED,
        );
        assert_code(
            port.restore(&restore_request("m5:official_project_id")),
            M3_PROJECT_ID_M5_HELPER_CLAIM_REJECTED,
        );
        assert_code(
            port.provision(&request("m5_m3_identity::official_project_id")),
            M3_PROJECT_ID_M5_HELPER_CLAIM_REJECTED,
        );
    }

    #[test]
    fn m3_project_role_session_authority_does_not_consume_m1_read_port_as_ordinary_source() {
        let port = M3ProjectRoleSessionAuthorityHandle::install_ordinary_product();
        assert_ne!(
            M3_PROJECT_ROLE_SESSION_AUTHORITY_PORT_VERSION,
            crate::m1_project_index::M1_PROJECT_INDEX_PORT_VERSION
        );
        assert_code(
            port.provision(&request("project:11111111-2222-3333-4444-555555555555")),
            M3_CANONICAL_PROJECT_ID_SOURCE_UNAVAILABLE,
        );
    }

    fn uninstalled_fixture_app_state() -> crate::AppState {
        crate::AppState {
            index_path: PathBuf::from("/m3o01r01/fixture/index.json"),
            tasks_path: PathBuf::from("/m3o01r01/fixture/tasks.md"),
            workflow_state_path: PathBuf::from("/m3o01r01/fixture/workflow-state.json"),
            m3_role_session_read_runtime: Default::default(),
            m1_project_index: None,
            m3_project_role_session_authority: None,
            m5_store_path: None,
        }
    }

    fn installed_fixture_app_state() -> crate::AppState {
        crate::AppState {
            index_path: PathBuf::from("/m3o01r01/ordinary/index.json"),
            tasks_path: PathBuf::from("/m3o01r01/ordinary/tasks.md"),
            workflow_state_path: PathBuf::from("/m3o01r01/ordinary/workflow-state.json"),
            m3_role_session_read_runtime: Default::default(),
            m1_project_index: None,
            m3_project_role_session_authority: Some(
                M3ProjectRoleSessionAuthorityHandle::install_ordinary_product(),
            ),
            m5_store_path: None,
        }
    }

    fn isolated_acceptance_app_state() -> (PathBuf, crate::AppState) {
        let root = std::env::temp_dir().join(format!(
            "m1i01r03r01-m3-isolated-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
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

    fn ordinary_product_app_data_root() -> PathBuf {
        let parent = std::env::temp_dir().join(format!(
            "m3o01r01-ordinary-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let root = parent.join("local.codex.governance.workbench");
        std::fs::create_dir_all(&root).expect("create ordinary app-data root");
        std::fs::canonicalize(&root).expect("canonicalize ordinary app-data root")
    }

    fn assert_unavailable_slot(state: &crate::AppState) {
        match state.m3_project_role_session_authority_port() {
            Ok(_) => panic!("uninstalled AppState slot must not yield an authority port"),
            Err(error) => assert_eq!(error.code, M3_AUTHORITY_UNAVAILABLE),
        }
    }

    #[test]
    fn m3_project_role_session_authority_uninstalled_app_state_returns_unavailable() {
        let legacy = crate::AppState::try_new().expect("legacy AppState must construct");
        assert_unavailable_slot(&legacy);
        assert_unavailable_slot(&uninstalled_fixture_app_state());
    }

    #[test]
    fn m3_project_role_session_authority_isolated_acceptance_app_state_returns_unavailable() {
        let (root, isolated) = isolated_acceptance_app_state();
        assert_unavailable_slot(&isolated);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn m3_project_role_session_authority_ordinary_slot_stays_fail_closed() {
        let lib = include_str!("lib.rs");
        assert!(
            lib.contains("M3ProjectRoleSessionAuthorityHandle::install_ordinary_product()"),
            "ordinary product constructor must still install the authority"
        );
        assert!(
            lib.contains("SharedProductAuthorityProfile::IsolatedUninstalled"),
            "isolated acceptance must explicitly leave M1/M3 uninstalled"
        );
        assert!(
            lib.matches("m3_project_role_session_authority: None,").count() >= 3,
            "acceptance/legacy constructors must leave the authority uninstalled"
        );

        let fixture = installed_fixture_app_state();
        let fixture_port = fixture
            .m3_project_role_session_authority_port()
            .expect("installed slot must expose the authority");
        assert_code(
            fixture_port.provision(&request("project:aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee")),
            M3_CANONICAL_PROJECT_ID_SOURCE_UNAVAILABLE,
        );

        let ordinary_root = ordinary_product_app_data_root();
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let ordinary = crate::AppState::try_new_with_ordinary_product_ports(
            &ordinary_root,
            &manifest_dir.join("../../index-kernel/codex-index.json"),
            &manifest_dir.join("../../../tasks/README.md"),
            crate::m4_secretary_conversation::M4SecretaryConversationProviderConfig::Unavailable,
        )
        .expect("ordinary product AppState must construct");
        let port = ordinary
            .m3_project_role_session_authority_port()
            .expect("ordinary installation must expose the installed authority");
        assert_code(
            port.provision(&request("project:aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee")),
            M3_CANONICAL_PROJECT_ID_SOURCE_UNAVAILABLE,
        );
        assert_code(
            port.load(&request("project:aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee")),
            M3_CANONICAL_PROJECT_ID_SOURCE_UNAVAILABLE,
        );
        assert_code(
            port.restore(&restore_request(
                "project:aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
            )),
            M3_CANONICAL_PROJECT_ID_SOURCE_UNAVAILABLE,
        );
    }
}
