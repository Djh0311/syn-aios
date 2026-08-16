// Frozen M5R07 read/write DTOs for the existing project shell.
// These do not own execution truth.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct M5SupervisorOpenRequest {
    pub project_id: String,
    pub role_session_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct M5SupervisorOpenResponse {
    pub binding_id: String,
    pub project_id: String,
    pub role_session_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct M5SupervisorTurnRequest {
    pub binding_id: String,
    pub project_id: String,
    pub kind: String,
    pub text: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct M5SupervisorTurnResponse {
    pub kind: String,
    pub created_proposal: bool,
    pub created_grant: bool,
    pub spawned: bool,
    pub text: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct M5ProjectSummaryRead {
    pub project_id: String,
    pub version: u64,
    pub watermark_ms: i64,
    pub fact_count: u32,
    pub unverified_claim_count: u32,
    pub open_run_count: u32,
    pub stale: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct M5GlobalAdviceFixture {
    pub advice_id: String,
    pub project_id: String,
    pub summary: String,
    pub source_ref: String,
    pub writable: bool,
}

impl M5GlobalAdviceFixture {
    pub(crate) fn frozen(project_id: &str) -> Self {
        Self {
            advice_id: "m5r07.global-advice.fixture.v1".into(),
            project_id: project_id.to_string(),
            summary: "readonly fixture; does not write any project".into(),
            source_ref: format!("summary:{project_id}"),
            writable: false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct M5LegacyPathManifest {
    pub entry_id: String,
    pub class: String,
    pub physically_deleted: bool,
}

pub(crate) fn legacy_execution_manifest() -> Vec<M5LegacyPathManifest> {
    vec![
        M5LegacyPathManifest {
            entry_id: "RUN-006".into(),
            class: "blocked".into(),
            physically_deleted: false,
        },
        M5LegacyPathManifest {
            entry_id: "RUN-008".into(),
            class: "guarded-legacy".into(),
            physically_deleted: false,
        },
        M5LegacyPathManifest {
            entry_id: "M5-SE-001".into(),
            class: "new-grant".into(),
            physically_deleted: false,
        },
    ]
}
