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
    fn new(code: impl Into<String>) -> Self {
        Self { code: code.into() }
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

    #[test]
    fn m3_project_role_session_authority_missing_port_is_unavailable() {
        let missing: Option<&dyn M3ProjectRoleSessionAuthorityPort> = None;
        assert!(missing.is_none());
        assert_eq!(
            M3ProjectRoleSessionAuthorityError::new(M3_AUTHORITY_UNAVAILABLE).code,
            M3_AUTHORITY_UNAVAILABLE
        );
    }
}
