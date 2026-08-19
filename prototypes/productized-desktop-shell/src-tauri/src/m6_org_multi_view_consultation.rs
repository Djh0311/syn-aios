//! M6D07 independent multi-view consultation.
//!
//! The module creates real, distinct M3 RoleSessions, but owns only M6
//! coordination records. It stores references and hashes instead of prompts or
//! runtime answer bodies, never opens a project store, and cannot create a
//! project command, grant, authorization, or formal fact.

use crate::m3_role_session::{
    CorrelationId, OpaqueRef, RequestIdempotencyKey, RoleSessionId, RoleSessionState,
    ServerResolvedBinding, Sha256Digest,
};
use crate::m3_role_session_repository::{
    CreateRoleSessionCommand, M3CommandMetadata, M3ReadPermissionDisposition,
    M3RoleSessionSnapshotQuery, M3RoleSessionSqliteRepository,
};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::path::Path;

const QUESTION_PACKET_SCHEMA_VERSION: u32 = 1;
const MIN_MULTI_VIEW_COUNT: usize = 2;
const MAX_MULTI_VIEW_COUNT: usize = 4;
const MAX_REF_LEN: usize = 512;
const MAX_SOURCE_REFS: usize = 64;
const MAX_CLAIMS_PER_VIEW: usize = 64;
const MAX_BUDGET_UNITS: u64 = 1_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum M6OrgConsultationEscalationTrigger {
    Routine,
    CrossProjectConflict,
    IrreversibleDecision,
    HighImpactRisk,
}

impl M6OrgConsultationEscalationTrigger {
    fn requires_multi_view(self) -> bool {
        !matches!(self, Self::Routine)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum M6OrgConsultationViewKind {
    RiskAnalyst,
    DependencyAnalyst,
    CounterfactualReviewer,
    OperationsReviewer,
}

impl M6OrgConsultationViewKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::RiskAnalyst => "RISK_ANALYST",
            Self::DependencyAnalyst => "DEPENDENCY_ANALYST",
            Self::CounterfactualReviewer => "COUNTERFACTUAL_REVIEWER",
            Self::OperationsReviewer => "OPERATIONS_REVIEWER",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum M6OrgConsultationBudgetState {
    WithinBudget,
    BudgetExceeded,
}

impl M6OrgConsultationBudgetState {
    fn as_str(self) -> &'static str {
        match self {
            Self::WithinBudget => "WITHIN_BUDGET",
            Self::BudgetExceeded => "BUDGET_EXCEEDED",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum M6OrgConsultationTimeoutState {
    WithinTime,
    TimedOut,
}

impl M6OrgConsultationTimeoutState {
    fn as_str(self) -> &'static str {
        match self {
            Self::WithinTime => "WITHIN_TIME",
            Self::TimedOut => "TIMED_OUT",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum M6OrgConsultationResultState {
    Pending,
    InFlight,
    Submitted,
    Assembled,
    TimedOut,
    BudgetExceeded,
    Failed,
    Quarantined,
}

impl M6OrgConsultationResultState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "PENDING",
            Self::InFlight => "IN_FLIGHT",
            Self::Submitted => "SUBMITTED",
            Self::Assembled => "ASSEMBLED",
            Self::TimedOut => "TIMED_OUT",
            Self::BudgetExceeded => "BUDGET_EXCEEDED",
            Self::Failed => "FAILED",
            Self::Quarantined => "QUARANTINED",
        }
    }

    fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Assembled
                | Self::TimedOut
                | Self::BudgetExceeded
                | Self::Failed
                | Self::Quarantined
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum M6OrgConsultationClaimPosition {
    Support,
    Oppose,
    Uncertain,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct M6OrgStartMultiViewConsultationRequest {
    pub(crate) question_ref: String,
    pub(crate) source_refs: Vec<String>,
    pub(crate) escalation_trigger: M6OrgConsultationEscalationTrigger,
    pub(crate) view_kinds: Vec<M6OrgConsultationViewKind>,
    pub(crate) budget_limit_ref: Option<String>,
    pub(crate) budget_limit_units: Option<u64>,
    pub(crate) deadline_at_ms: Option<i64>,
    pub(crate) idempotency_key: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct M6OrgQuestionPacket {
    pub(crate) question_packet_id: String,
    pub(crate) question_ref: String,
    pub(crate) source_refs: Vec<String>,
    pub(crate) packet_hash: String,
    pub(crate) minimal: bool,
    pub(crate) schema_version: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct M6OrgConsultationClaim {
    pub(crate) topic_ref: String,
    pub(crate) position: M6OrgConsultationClaimPosition,
    pub(crate) evidence_refs: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct M6OrgConsultationView {
    pub(crate) view_id: String,
    pub(crate) consultation_id: String,
    pub(crate) view_kind: M6OrgConsultationViewKind,
    pub(crate) role_session_id: String,
    pub(crate) workcell_ref: String,
    pub(crate) context_packet_ref: String,
    pub(crate) question_packet_id: String,
    pub(crate) question_packet_hash: String,
    pub(crate) dispatch_input_refs: Vec<String>,
    pub(crate) dispatch_payload_hash: String,
    pub(crate) budget_cap_units: u64,
    pub(crate) submitted: bool,
    pub(crate) conclusion_ref: Option<String>,
    pub(crate) runtime_final_answer_hash: Option<String>,
    pub(crate) claims: Vec<M6OrgConsultationClaim>,
    pub(crate) reported_cost_units: Option<u64>,
    pub(crate) peer_conclusions_readable_before_submit: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct M6OrgConsensusEntry {
    pub(crate) topic_ref: String,
    pub(crate) position: M6OrgConsultationClaimPosition,
    pub(crate) view_conclusion_refs: Vec<String>,
    pub(crate) evidence_refs: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct M6OrgDisagreementViewPosition {
    pub(crate) view_id: String,
    pub(crate) conclusion_ref: String,
    pub(crate) position: Option<M6OrgConsultationClaimPosition>,
    pub(crate) evidence_refs: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct M6OrgDisagreementEntry {
    pub(crate) topic_ref: String,
    pub(crate) view_positions: Vec<M6OrgDisagreementViewPosition>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct M6OrgEvidenceIndexEntry {
    pub(crate) view_id: String,
    pub(crate) conclusion_ref: String,
    pub(crate) topic_ref: String,
    pub(crate) evidence_refs: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct M6OrgConsultationDecisionRequest {
    pub(crate) decision_request_id: String,
    pub(crate) consultation_id: String,
    pub(crate) consultation_revision: u64,
    pub(crate) status: String,
    pub(crate) creates_project_command: bool,
    pub(crate) creates_grant: bool,
    pub(crate) creates_formal_fact: bool,
    pub(crate) created_at_ms: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct M6OrgMultiViewConsultation {
    pub(crate) consultation_id: String,
    pub(crate) question_packet: M6OrgQuestionPacket,
    pub(crate) escalation_trigger: M6OrgConsultationEscalationTrigger,
    pub(crate) views: Vec<M6OrgConsultationView>,
    pub(crate) consensus_index_ref: Option<String>,
    pub(crate) consensus: Vec<M6OrgConsensusEntry>,
    pub(crate) disagreement_index_ref: Option<String>,
    pub(crate) disagreements: Vec<M6OrgDisagreementEntry>,
    pub(crate) evidence_index_ref: Option<String>,
    pub(crate) evidence_index: Vec<M6OrgEvidenceIndexEntry>,
    pub(crate) budget_limit_ref: String,
    pub(crate) budget_limit_units: u64,
    pub(crate) budget_state: M6OrgConsultationBudgetState,
    pub(crate) deadline_at_ms: i64,
    pub(crate) timeout_state: M6OrgConsultationTimeoutState,
    pub(crate) result_state: M6OrgConsultationResultState,
    pub(crate) user_pending_decision_request_id: Option<String>,
    pub(crate) decision_request: Option<M6OrgConsultationDecisionRequest>,
    pub(crate) produces_command: bool,
    pub(crate) produces_grant: bool,
    pub(crate) produces_fact: bool,
    pub(crate) revision: u64,
    pub(crate) created_at_ms: i64,
    pub(crate) updated_at_ms: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "route", rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum M6OrgConsultationRouteResponse {
    SingleRole {
        question_packet: M6OrgQuestionPacket,
        escalation_required: bool,
        consultation_created: bool,
    },
    MultiView {
        consultation: M6OrgMultiViewConsultation,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct M6OrgSubmitConsultationViewRequest {
    pub(crate) consultation_id: String,
    pub(crate) view_id: String,
    pub(crate) role_session_id: String,
    pub(crate) workcell_ref: String,
    pub(crate) context_packet_ref: String,
    pub(crate) question_packet_id: String,
    pub(crate) question_packet_hash: String,
    pub(crate) runtime_input_refs: Vec<String>,
    pub(crate) runtime_final_answer_ref: String,
    pub(crate) runtime_final_answer_hash: String,
    pub(crate) claims: Vec<M6OrgConsultationClaim>,
    pub(crate) reported_cost_units: u64,
    pub(crate) peer_conclusions_readable_before_submit: bool,
    pub(crate) idempotency_key: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct M6OrgAssembleMultiViewConsultationRequest {
    pub(crate) consultation_id: String,
    pub(crate) expected_revision: u64,
    pub(crate) idempotency_key: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct M6OrgConsultationHeader {
    consultation_id: String,
    question_packet: M6OrgQuestionPacket,
    escalation_trigger: M6OrgConsultationEscalationTrigger,
    consensus_index_ref: Option<String>,
    consensus: Vec<M6OrgConsensusEntry>,
    disagreement_index_ref: Option<String>,
    disagreements: Vec<M6OrgDisagreementEntry>,
    evidence_index_ref: Option<String>,
    evidence_index: Vec<M6OrgEvidenceIndexEntry>,
    budget_limit_ref: String,
    budget_limit_units: u64,
    budget_state: M6OrgConsultationBudgetState,
    deadline_at_ms: i64,
    timeout_state: M6OrgConsultationTimeoutState,
    result_state: M6OrgConsultationResultState,
    user_pending_decision_request_id: Option<String>,
    produces_command: bool,
    produces_grant: bool,
    produces_fact: bool,
    revision: u64,
    created_at_ms: i64,
    updated_at_ms: i64,
}

struct M6OrgMultiViewStore {
    connection: Connection,
}

impl M6OrgMultiViewStore {
    fn open(path: &Path) -> Result<Self, String> {
        let parent = path
            .parent()
            .ok_or_else(|| "m6_org_multi_view_store_parent_missing".to_string())?;
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("m6_org_multi_view_store_parent_create:{error}"))?;
        let connection = Connection::open(path)
            .map_err(|error| format!("m6_org_multi_view_store_open:{error}"))?;
        crate::m6_org_schema::ensure_m6_org_schema(&connection)?;
        Ok(Self { connection })
    }

    fn load_by_idempotency(
        &self,
        idempotency_key: &str,
    ) -> Result<Option<(String, String)>, String> {
        self.connection
            .query_row(
                "SELECT consultation_id,request_hash FROM m6_multi_view_consultations
                 WHERE idempotency_key=?1",
                [idempotency_key],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(|error| format!("m6_org_multi_view_idempotency_load:{error}"))
    }

    fn load_header(&self, consultation_id: &str) -> Result<M6OrgConsultationHeader, String> {
        load_json_required(
            &self.connection,
            "SELECT header_json FROM m6_multi_view_consultations WHERE consultation_id=?1",
            (consultation_id,),
            "m6_org_multi_view_header_load",
        )
    }

    fn load_views(&self, consultation_id: &str) -> Result<Vec<M6OrgConsultationView>, String> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT payload_json FROM m6_consultation_views
                 WHERE consultation_id=?1 ORDER BY view_id",
            )
            .map_err(|error| format!("m6_org_multi_view_views_prepare:{error}"))?;
        let rows = statement
            .query_map([consultation_id], |row| row.get::<_, String>(0))
            .map_err(|error| format!("m6_org_multi_view_views_query:{error}"))?;
        let mut views = Vec::new();
        for row in rows {
            let payload = row.map_err(|error| format!("m6_org_multi_view_views_row:{error}"))?;
            views.push(
                serde_json::from_str(&payload)
                    .map_err(|error| format!("m6_org_multi_view_views_decode:{error}"))?,
            );
        }
        Ok(views)
    }

    fn load_decision(
        &self,
        consultation_id: &str,
    ) -> Result<Option<M6OrgConsultationDecisionRequest>, String> {
        load_json_optional(
            &self.connection,
            "SELECT payload_json FROM m6_consultation_decision_requests
             WHERE consultation_id=?1",
            (consultation_id,),
            "m6_org_multi_view_decision_load",
        )
    }

    fn load_consultation(
        &self,
        consultation_id: &str,
    ) -> Result<M6OrgMultiViewConsultation, String> {
        let header = self.load_header(consultation_id)?;
        let views = self.load_views(consultation_id)?;
        let decision_request = self.load_decision(consultation_id)?;
        if views.len() < MIN_MULTI_VIEW_COUNT
            || views
                .iter()
                .any(|view| view.consultation_id != header.consultation_id)
        {
            return Err("m6_org_multi_view_store_incomplete".to_string());
        }
        Ok(consultation_from(header, views, decision_request))
    }

    /// Pre-submit isolation boundary: this query loads only the consultation
    /// header and the target view row. It cannot return peer payloads or peer
    /// conclusion refs.
    fn load_submission_target(
        &self,
        consultation_id: &str,
        view_id: &str,
    ) -> Result<(M6OrgConsultationHeader, M6OrgConsultationView), String> {
        let header = self.load_header(consultation_id)?;
        let view = load_json_required(
            &self.connection,
            "SELECT payload_json FROM m6_consultation_views
             WHERE consultation_id=?1 AND view_id=?2",
            (consultation_id, view_id),
            "m6_org_multi_view_target_load",
        )?;
        Ok((header, view))
    }

    fn load_command_receipt(
        &self,
        idempotency_key: &str,
    ) -> Result<Option<(String, String, String)>, String> {
        self.connection
            .query_row(
                "SELECT operation,request_hash,consultation_id
                 FROM m6_consultation_command_receipts WHERE idempotency_key=?1",
                [idempotency_key],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .map_err(|error| format!("m6_org_multi_view_receipt_load:{error}"))
    }

    fn record_start(
        &mut self,
        header: &M6OrgConsultationHeader,
        views: &[M6OrgConsultationView],
        idempotency_key: &str,
        request_hash: &str,
    ) -> Result<(), String> {
        let header_json = encode(header, "m6_org_multi_view_header")?;
        let transaction = self
            .connection
            .transaction()
            .map_err(|error| format!("m6_org_multi_view_start_tx:{error}"))?;
        transaction
            .execute(
                "INSERT INTO m6_multi_view_consultations (
                    consultation_id,idempotency_key,request_hash,result_state,budget_state,
                    timeout_state,revision,header_json,created_at_ms,updated_at_ms
                 ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
                params![
                    header.consultation_id,
                    idempotency_key,
                    request_hash,
                    header.result_state.as_str(),
                    header.budget_state.as_str(),
                    header.timeout_state.as_str(),
                    header.revision as i64,
                    header_json,
                    header.created_at_ms,
                    header.updated_at_ms
                ],
            )
            .map_err(|error| format!("m6_org_multi_view_start_insert:{error}"))?;
        for view in views {
            transaction
                .execute(
                    "INSERT INTO m6_consultation_views (
                        consultation_id,view_id,role_session_id,workcell_ref,context_packet_ref,
                        question_packet_id,question_packet_hash,view_kind,submitted,
                        peer_conclusions_readable_before_submit,payload_json
                     ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,0,0,?9)",
                    params![
                        view.consultation_id,
                        view.view_id,
                        view.role_session_id,
                        view.workcell_ref,
                        view.context_packet_ref,
                        view.question_packet_id,
                        view.question_packet_hash,
                        view.view_kind.as_str(),
                        encode(view, "m6_org_multi_view_view")?
                    ],
                )
                .map_err(|error| format!("m6_org_multi_view_view_insert:{error}"))?;
        }
        insert_audit(
            &transaction,
            &format!("m6-audit:multi-view:start:{}", header.consultation_id),
            "MultiViewConsultationStarted",
            &header.consultation_id,
            &serde_json::json!({
                "question_packet_id": header.question_packet.question_packet_id,
                "question_packet_hash": header.question_packet.packet_hash,
                "view_count": views.len(),
                "minimal": true,
                "peer_conclusions_readable_before_submit": false,
                "produces_command": false,
                "produces_grant": false,
                "produces_fact": false
            }),
            header.created_at_ms,
        )?;
        transaction
            .commit()
            .map_err(|error| format!("m6_org_multi_view_start_commit:{error}"))
    }

    fn record_terminal_without_submission(
        &mut self,
        mut header: M6OrgConsultationHeader,
        request_hash: &str,
        idempotency_key: &str,
        operation: &str,
        state: M6OrgConsultationResultState,
        now_ms: i64,
    ) -> Result<(), String> {
        match state {
            M6OrgConsultationResultState::TimedOut => {
                header.timeout_state = M6OrgConsultationTimeoutState::TimedOut;
            }
            M6OrgConsultationResultState::BudgetExceeded => {
                header.budget_state = M6OrgConsultationBudgetState::BudgetExceeded;
            }
            _ => return Err("m6_org_multi_view_terminal_state_invalid".to_string()),
        }
        header.result_state = state;
        header.revision = header
            .revision
            .checked_add(1)
            .ok_or_else(|| "m6_org_multi_view_revision_overflow".to_string())?;
        header.updated_at_ms = now_ms;
        let header_json = encode(&header, "m6_org_multi_view_terminal")?;
        let transaction = self
            .connection
            .transaction()
            .map_err(|error| format!("m6_org_multi_view_terminal_tx:{error}"))?;
        let changed = transaction
            .execute(
                "UPDATE m6_multi_view_consultations
                 SET result_state=?2,budget_state=?3,timeout_state=?4,revision=?5,
                     header_json=?6,updated_at_ms=?7
                 WHERE consultation_id=?1 AND revision=?8",
                params![
                    header.consultation_id,
                    header.result_state.as_str(),
                    header.budget_state.as_str(),
                    header.timeout_state.as_str(),
                    header.revision as i64,
                    header_json,
                    now_ms,
                    (header.revision - 1) as i64
                ],
            )
            .map_err(|error| format!("m6_org_multi_view_terminal_update:{error}"))?;
        if changed != 1 {
            return Err("m6_org_multi_view_revision_conflict".to_string());
        }
        insert_receipt(
            &transaction,
            idempotency_key,
            operation,
            request_hash,
            &header.consultation_id,
            now_ms,
        )?;
        insert_audit(
            &transaction,
            &format!(
                "m6-audit:multi-view:{operation}:{}:{}",
                header.consultation_id, header.revision
            ),
            if state == M6OrgConsultationResultState::TimedOut {
                "MultiViewConsultationTimedOut"
            } else {
                "MultiViewConsultationBudgetExceeded"
            },
            &header.consultation_id,
            &serde_json::json!({
                "result_state": state.as_str(),
                "partial_results_assembled": false,
                "decision_request_created": false
            }),
            now_ms,
        )?;
        transaction
            .commit()
            .map_err(|error| format!("m6_org_multi_view_terminal_commit:{error}"))
    }

    fn record_submission(
        &mut self,
        mut header: M6OrgConsultationHeader,
        view: &M6OrgConsultationView,
        request_hash: &str,
        idempotency_key: &str,
        now_ms: i64,
    ) -> Result<(), String> {
        let previous_revision = header.revision;
        let transaction = self
            .connection
            .transaction()
            .map_err(|error| format!("m6_org_multi_view_submit_tx:{error}"))?;
        let changed = transaction
            .execute(
                "UPDATE m6_consultation_views
                 SET submitted=1,submission_idempotency_key=?3,submission_hash=?4,
                     reported_cost_units=?5,payload_json=?6
                 WHERE consultation_id=?1 AND view_id=?2 AND submitted=0",
                params![
                    view.consultation_id,
                    view.view_id,
                    idempotency_key,
                    request_hash,
                    view.reported_cost_units.map(|value| value as i64),
                    encode(view, "m6_org_multi_view_submitted_view")?
                ],
            )
            .map_err(|error| format!("m6_org_multi_view_submit_update:{error}"))?;
        if changed != 1 {
            return Err("m6_org_multi_view_view_already_submitted".to_string());
        }
        let submitted_count = transaction
            .query_row(
                "SELECT COUNT(*) FROM m6_consultation_views
                 WHERE consultation_id=?1 AND submitted=1",
                [&header.consultation_id],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|error| format!("m6_org_multi_view_submitted_count:{error}"))?;
        let total_count = transaction
            .query_row(
                "SELECT COUNT(*) FROM m6_consultation_views WHERE consultation_id=?1",
                [&header.consultation_id],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|error| format!("m6_org_multi_view_total_count:{error}"))?;
        header.result_state = if submitted_count == total_count {
            M6OrgConsultationResultState::Submitted
        } else {
            M6OrgConsultationResultState::InFlight
        };
        header.revision = previous_revision
            .checked_add(1)
            .ok_or_else(|| "m6_org_multi_view_revision_overflow".to_string())?;
        header.updated_at_ms = now_ms;
        let header_changed = transaction
            .execute(
                "UPDATE m6_multi_view_consultations
                 SET result_state=?2,revision=?3,header_json=?4,updated_at_ms=?5
                 WHERE consultation_id=?1 AND revision=?6",
                params![
                    header.consultation_id,
                    header.result_state.as_str(),
                    header.revision as i64,
                    encode(&header, "m6_org_multi_view_submit_header")?,
                    now_ms,
                    previous_revision as i64
                ],
            )
            .map_err(|error| format!("m6_org_multi_view_submit_header_update:{error}"))?;
        if header_changed != 1 {
            return Err("m6_org_multi_view_revision_conflict".to_string());
        }
        insert_receipt(
            &transaction,
            idempotency_key,
            "submit_view",
            request_hash,
            &header.consultation_id,
            now_ms,
        )?;
        insert_audit(
            &transaction,
            &format!(
                "m6-audit:multi-view:submit:{}:{}",
                header.consultation_id, view.view_id
            ),
            "ConsultationViewSubmitted",
            &view.view_id,
            &serde_json::json!({
                "consultation_id": header.consultation_id,
                "question_packet_hash": view.question_packet_hash,
                "role_session_id": view.role_session_id,
                "workcell_ref": view.workcell_ref,
                "context_packet_ref": view.context_packet_ref,
                "runtime_final_answer_ref": view.conclusion_ref,
                "runtime_final_answer_hash": view.runtime_final_answer_hash,
                "peer_conclusions_readable_before_submit": false,
                "answer_body_stored": false
            }),
            now_ms,
        )?;
        transaction
            .commit()
            .map_err(|error| format!("m6_org_multi_view_submit_commit:{error}"))
    }

    fn record_assembly(
        &mut self,
        mut header: M6OrgConsultationHeader,
        decision: &M6OrgConsultationDecisionRequest,
        request_hash: &str,
        idempotency_key: &str,
        now_ms: i64,
    ) -> Result<(), String> {
        let previous_revision = header.revision;
        header.result_state = M6OrgConsultationResultState::Assembled;
        header.user_pending_decision_request_id = Some(decision.decision_request_id.clone());
        header.revision = previous_revision
            .checked_add(1)
            .ok_or_else(|| "m6_org_multi_view_revision_overflow".to_string())?;
        header.updated_at_ms = now_ms;
        let transaction = self
            .connection
            .transaction()
            .map_err(|error| format!("m6_org_multi_view_assemble_tx:{error}"))?;
        let changed = transaction
            .execute(
                "UPDATE m6_multi_view_consultations
                 SET result_state='ASSEMBLED',revision=?2,header_json=?3,updated_at_ms=?4
                 WHERE consultation_id=?1 AND revision=?5 AND result_state='SUBMITTED'",
                params![
                    header.consultation_id,
                    header.revision as i64,
                    encode(&header, "m6_org_multi_view_assembled_header")?,
                    now_ms,
                    previous_revision as i64
                ],
            )
            .map_err(|error| format!("m6_org_multi_view_assemble_update:{error}"))?;
        if changed != 1 {
            return Err("m6_org_multi_view_revision_conflict".to_string());
        }
        transaction
            .execute(
                "INSERT INTO m6_consultation_decision_requests (
                    decision_request_id,consultation_id,status,payload_json,created_at_ms
                 ) VALUES (?1,?2,'PENDING_USER_DECISION',?3,?4)",
                params![
                    decision.decision_request_id,
                    decision.consultation_id,
                    encode(decision, "m6_org_multi_view_decision")?,
                    decision.created_at_ms
                ],
            )
            .map_err(|error| format!("m6_org_multi_view_decision_insert:{error}"))?;
        insert_receipt(
            &transaction,
            idempotency_key,
            "assemble",
            request_hash,
            &header.consultation_id,
            now_ms,
        )?;
        insert_audit(
            &transaction,
            &format!("m6-audit:multi-view:assemble:{}", header.consultation_id),
            "MultiViewConsultationAssembled",
            &header.consultation_id,
            &serde_json::json!({
                "consensus_index_ref": header.consensus_index_ref,
                "disagreement_index_ref": header.disagreement_index_ref,
                "evidence_index_ref": header.evidence_index_ref,
                "decision_request_id": decision.decision_request_id,
                "decision_status": decision.status,
                "produces_command": false,
                "produces_grant": false,
                "produces_fact": false,
                "project_writeback": false
            }),
            now_ms,
        )?;
        transaction
            .commit()
            .map_err(|error| format!("m6_org_multi_view_assemble_commit:{error}"))
    }
}

pub(crate) fn start_for_state(
    state: &crate::AppState,
    request: &M6OrgStartMultiViewConsultationRequest,
    now_ms: i64,
) -> Result<M6OrgConsultationRouteResponse, String> {
    validate_start_request(request, now_ms)?;
    let _authority = state.m6_org_global_role_session.authority_seed()?;
    let question_packet = build_question_packet(request)?;
    if !request.escalation_trigger.requires_multi_view() {
        return Ok(M6OrgConsultationRouteResponse::SingleRole {
            question_packet,
            escalation_required: false,
            consultation_created: false,
        });
    }

    let request_hash = stable_hash(request, "m6_org_multi_view_start_request")?;
    let m6_path = state.m6_org_store_path()?;
    let mut store = M6OrgMultiViewStore::open(&m6_path)?;
    if let Some((consultation_id, existing_hash)) =
        store.load_by_idempotency(&request.idempotency_key)?
    {
        if existing_hash != request_hash {
            return Err("m6_org_multi_view_start_idempotency_collision".to_string());
        }
        return Ok(M6OrgConsultationRouteResponse::MultiView {
            consultation: store.load_consultation(&consultation_id)?,
        });
    }

    let consultation_id = digest_id(
        "consultation",
        &[&request.idempotency_key, &question_packet.packet_hash],
    );
    let authority = state.m6_org_global_role_session.authority_seed()?;
    let view_count = request.view_kinds.len();
    let budget_limit_units = request
        .budget_limit_units
        .ok_or_else(|| "m6_org_multi_view_budget_required".to_string())?;
    let per_view_cap = budget_limit_units / view_count as u64;
    if per_view_cap == 0 {
        return Err("m6_org_multi_view_budget_too_small".to_string());
    }
    let mut views = Vec::with_capacity(view_count);
    for kind in &request.view_kinds {
        views.push(create_view(
            &authority.repository,
            &consultation_id,
            &question_packet,
            *kind,
            per_view_cap,
        )?);
    }
    validate_independent_views(&question_packet, &views)?;
    let header = M6OrgConsultationHeader {
        consultation_id: consultation_id.clone(),
        question_packet,
        escalation_trigger: request.escalation_trigger,
        consensus_index_ref: None,
        consensus: Vec::new(),
        disagreement_index_ref: None,
        disagreements: Vec::new(),
        evidence_index_ref: None,
        evidence_index: Vec::new(),
        budget_limit_ref: request
            .budget_limit_ref
            .clone()
            .ok_or_else(|| "m6_org_multi_view_budget_ref_required".to_string())?,
        budget_limit_units,
        budget_state: M6OrgConsultationBudgetState::WithinBudget,
        deadline_at_ms: request
            .deadline_at_ms
            .ok_or_else(|| "m6_org_multi_view_deadline_required".to_string())?,
        timeout_state: M6OrgConsultationTimeoutState::WithinTime,
        result_state: M6OrgConsultationResultState::InFlight,
        user_pending_decision_request_id: None,
        produces_command: false,
        produces_grant: false,
        produces_fact: false,
        revision: 1,
        created_at_ms: now_ms,
        updated_at_ms: now_ms,
    };
    store.record_start(&header, &views, &request.idempotency_key, &request_hash)?;
    Ok(M6OrgConsultationRouteResponse::MultiView {
        consultation: store.load_consultation(&consultation_id)?,
    })
}

pub(crate) fn submit_view_for_state(
    state: &crate::AppState,
    request: &M6OrgSubmitConsultationViewRequest,
    now_ms: i64,
) -> Result<M6OrgMultiViewConsultation, String> {
    validate_submit_request(request, now_ms)?;
    let authority = state.m6_org_global_role_session.authority_seed()?;
    let request_hash = stable_hash(request, "m6_org_multi_view_submit_request")?;
    let m6_path = state.m6_org_store_path()?;
    let mut store = M6OrgMultiViewStore::open(&m6_path)?;
    if let Some((operation, existing_hash, consultation_id)) =
        store.load_command_receipt(&request.idempotency_key)?
    {
        if operation != "submit_view"
            || existing_hash != request_hash
            || consultation_id != request.consultation_id
        {
            return Err("m6_org_multi_view_submit_idempotency_collision".to_string());
        }
        return store.load_consultation(&request.consultation_id);
    }
    let (header, target) =
        store.load_submission_target(&request.consultation_id, &request.view_id)?;
    if header.result_state.is_terminal() {
        return Err("m6_org_multi_view_consultation_terminal".to_string());
    }
    validate_submission_binding(&header, &target, request)?;
    verify_view_role_session(&authority.repository, &header.consultation_id, &target)?;
    if now_ms > header.deadline_at_ms {
        store.record_terminal_without_submission(
            header,
            &request_hash,
            &request.idempotency_key,
            "submit_view",
            M6OrgConsultationResultState::TimedOut,
            now_ms,
        )?;
        return store.load_consultation(&request.consultation_id);
    }
    let already_reported = store
        .connection
        .query_row(
            "SELECT COALESCE(SUM(reported_cost_units),0) FROM m6_consultation_views
             WHERE consultation_id=?1 AND submitted=1",
            [&request.consultation_id],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|error| format!("m6_org_multi_view_cost_sum:{error}"))?;
    let total_cost = u64::try_from(already_reported)
        .ok()
        .and_then(|value| value.checked_add(request.reported_cost_units))
        .ok_or_else(|| "m6_org_multi_view_cost_overflow".to_string())?;
    if request.reported_cost_units > target.budget_cap_units
        || total_cost > header.budget_limit_units
    {
        store.record_terminal_without_submission(
            header,
            &request_hash,
            &request.idempotency_key,
            "submit_view",
            M6OrgConsultationResultState::BudgetExceeded,
            now_ms,
        )?;
        return store.load_consultation(&request.consultation_id);
    }
    let mut submitted = target;
    submitted.submitted = true;
    submitted.conclusion_ref = Some(request.runtime_final_answer_ref.clone());
    submitted.runtime_final_answer_hash = Some(request.runtime_final_answer_hash.clone());
    submitted.claims = request.claims.clone();
    submitted.reported_cost_units = Some(request.reported_cost_units);
    store.record_submission(
        header,
        &submitted,
        &request_hash,
        &request.idempotency_key,
        now_ms,
    )?;
    store.load_consultation(&request.consultation_id)
}

pub(crate) fn assemble_for_state(
    state: &crate::AppState,
    request: &M6OrgAssembleMultiViewConsultationRequest,
    now_ms: i64,
) -> Result<M6OrgMultiViewConsultation, String> {
    validate_ref("consultation_id", &request.consultation_id)?;
    validate_ref("idempotency_key", &request.idempotency_key)?;
    if request.expected_revision == 0 || now_ms < 0 {
        return Err("m6_org_multi_view_assemble_request_invalid".to_string());
    }
    let _authority = state.m6_org_global_role_session.authority_seed()?;
    let request_hash = stable_hash(request, "m6_org_multi_view_assemble_request")?;
    let m6_path = state.m6_org_store_path()?;
    let mut store = M6OrgMultiViewStore::open(&m6_path)?;
    if let Some((operation, existing_hash, consultation_id)) =
        store.load_command_receipt(&request.idempotency_key)?
    {
        if operation != "assemble"
            || existing_hash != request_hash
            || consultation_id != request.consultation_id
        {
            return Err("m6_org_multi_view_assemble_idempotency_collision".to_string());
        }
        return store.load_consultation(&request.consultation_id);
    }
    let consultation = store.load_consultation(&request.consultation_id)?;
    if consultation.revision != request.expected_revision {
        return Err("m6_org_multi_view_revision_conflict".to_string());
    }
    if consultation.result_state.is_terminal() {
        return Ok(consultation);
    }
    let header = store.load_header(&request.consultation_id)?;
    if now_ms > header.deadline_at_ms {
        store.record_terminal_without_submission(
            header,
            &request_hash,
            &request.idempotency_key,
            "assemble",
            M6OrgConsultationResultState::TimedOut,
            now_ms,
        )?;
        return store.load_consultation(&request.consultation_id);
    }
    let total_cost = consultation
        .views
        .iter()
        .filter_map(|view| view.reported_cost_units)
        .try_fold(0_u64, |total, value| total.checked_add(value))
        .ok_or_else(|| "m6_org_multi_view_cost_overflow".to_string())?;
    if total_cost > consultation.budget_limit_units {
        store.record_terminal_without_submission(
            header,
            &request_hash,
            &request.idempotency_key,
            "assemble",
            M6OrgConsultationResultState::BudgetExceeded,
            now_ms,
        )?;
        return store.load_consultation(&request.consultation_id);
    }
    if consultation.result_state != M6OrgConsultationResultState::Submitted
        || consultation.views.iter().any(|view| !view.submitted)
    {
        return Ok(consultation);
    }
    let (consensus, disagreements, evidence_index) = build_indexes(&consultation.views)?;
    let mut assembled_header = header;
    assembled_header.consensus_index_ref = Some(digest_id(
        "consensus-index",
        &[
            &consultation.consultation_id,
            &stable_hash(&consensus, "consensus")?,
        ],
    ));
    assembled_header.consensus = consensus;
    assembled_header.disagreement_index_ref = Some(digest_id(
        "disagreement-index",
        &[
            &consultation.consultation_id,
            &stable_hash(&disagreements, "disagreements")?,
        ],
    ));
    assembled_header.disagreements = disagreements;
    assembled_header.evidence_index_ref = Some(digest_id(
        "evidence-index",
        &[
            &consultation.consultation_id,
            &stable_hash(&evidence_index, "evidence")?,
        ],
    ));
    assembled_header.evidence_index = evidence_index;
    let decision = M6OrgConsultationDecisionRequest {
        decision_request_id: digest_id(
            "decision-request",
            &[&consultation.consultation_id, &request.idempotency_key],
        ),
        consultation_id: consultation.consultation_id.clone(),
        consultation_revision: consultation.revision + 1,
        status: "PENDING_USER_DECISION".to_string(),
        creates_project_command: false,
        creates_grant: false,
        creates_formal_fact: false,
        created_at_ms: now_ms,
    };
    store.record_assembly(
        assembled_header,
        &decision,
        &request_hash,
        &request.idempotency_key,
        now_ms,
    )?;
    store.load_consultation(&request.consultation_id)
}

fn build_question_packet(
    request: &M6OrgStartMultiViewConsultationRequest,
) -> Result<M6OrgQuestionPacket, String> {
    let mut source_refs = request.source_refs.clone();
    source_refs.sort();
    let packet_hash = stable_hash(
        &(
            request.question_ref.as_str(),
            source_refs.as_slice(),
            QUESTION_PACKET_SCHEMA_VERSION,
        ),
        "m6_org_multi_view_question_packet",
    )?;
    Ok(M6OrgQuestionPacket {
        question_packet_id: digest_id("question-packet", &[&request.idempotency_key, &packet_hash]),
        question_ref: request.question_ref.clone(),
        source_refs,
        packet_hash,
        minimal: true,
        schema_version: QUESTION_PACKET_SCHEMA_VERSION,
    })
}

fn create_view(
    repository: &M3RoleSessionSqliteRepository,
    consultation_id: &str,
    question_packet: &M6OrgQuestionPacket,
    kind: M6OrgConsultationViewKind,
    budget_cap_units: u64,
) -> Result<M6OrgConsultationView, String> {
    let view_id = digest_id("consultation-view", &[consultation_id, kind.as_str()]);
    let binding = view_binding(consultation_id, kind)?;
    let role_session_id = RoleSessionId::try_from_canonical(sealed_ref(
        "session",
        &format!(
            "syn.m6.org.multi-view/{consultation_id}/{}/session",
            kind.as_str()
        ),
    ))
    .map_err(|_| "m6_org_multi_view_role_session_id_invalid".to_string())?;
    let material = format!(
        "syn.m6.org.multi-view/{consultation_id}/{}/create",
        kind.as_str()
    );
    let outcome = repository
        .create_role_session(&CreateRoleSessionCommand {
            role_session_id: role_session_id.clone(),
            binding: binding.clone(),
            metadata: m3_metadata(repository, &material)?,
        })
        .map_err(|error| format!("m6_org_multi_view_role_session_create:{}", error.code))?;
    let session = outcome
        .role_session
        .ok_or_else(|| "m6_org_multi_view_role_session_missing".to_string())?;
    if session.role_session_id != role_session_id
        || session.status != RoleSessionState::Active
        || !session.matches_binding_identity(&binding)
    {
        return Err("m6_org_multi_view_role_session_mismatch".to_string());
    }
    let workcell_ref = digest_id("workcell", &[consultation_id, &view_id, kind.as_str()]);
    let context_packet_ref = digest_id(
        "context-packet",
        &[
            consultation_id,
            &view_id,
            role_session_id.as_str(),
            &question_packet.packet_hash,
        ],
    );
    let dispatch_input_refs = vec![
        question_packet.question_packet_id.clone(),
        context_packet_ref.clone(),
    ];
    let dispatch_payload_hash = stable_hash(
        &(
            consultation_id,
            view_id.as_str(),
            role_session_id.as_str(),
            workcell_ref.as_str(),
            context_packet_ref.as_str(),
            question_packet.question_packet_id.as_str(),
            question_packet.packet_hash.as_str(),
            budget_cap_units,
        ),
        "m6_org_multi_view_dispatch",
    )?;
    Ok(M6OrgConsultationView {
        view_id,
        consultation_id: consultation_id.to_string(),
        view_kind: kind,
        role_session_id: role_session_id.as_str().to_string(),
        workcell_ref,
        context_packet_ref,
        question_packet_id: question_packet.question_packet_id.clone(),
        question_packet_hash: question_packet.packet_hash.clone(),
        dispatch_input_refs,
        dispatch_payload_hash,
        budget_cap_units,
        submitted: false,
        conclusion_ref: None,
        runtime_final_answer_hash: None,
        claims: Vec::new(),
        reported_cost_units: None,
        peer_conclusions_readable_before_submit: false,
    })
}

fn verify_view_role_session(
    repository: &M3RoleSessionSqliteRepository,
    consultation_id: &str,
    view: &M6OrgConsultationView,
) -> Result<(), String> {
    let role_session_id = RoleSessionId::try_from_canonical(view.role_session_id.clone())
        .map_err(|_| "m6_org_multi_view_role_session_id_invalid".to_string())?;
    let binding = view_binding(consultation_id, view.view_kind)?;
    let snapshot = repository
        .load_authorized_role_session_snapshot(&M3RoleSessionSnapshotQuery {
            role_session_id: role_session_id.clone(),
            binding: binding.clone(),
        })
        .map_err(|error| format!("m6_org_multi_view_role_session_read:{}", error.code))?
        .ok_or_else(|| "m6_org_multi_view_role_session_missing".to_string())?;
    if snapshot.session.role_session_id != role_session_id
        || snapshot.session.status != RoleSessionState::Active
        || !snapshot.session.matches_binding_identity(&binding)
        || !matches!(snapshot.permission, M3ReadPermissionDisposition::Current)
    {
        return Err("m6_org_multi_view_role_session_mismatch".to_string());
    }
    Ok(())
}

fn view_binding(
    consultation_id: &str,
    kind: M6OrgConsultationViewKind,
) -> Result<ServerResolvedBinding, String> {
    ServerResolvedBinding::from_server_canonical(
        sealed_ref(
            "actor",
            &format!(
                "syn.m6.org.multi-view/{consultation_id}/{}/actor",
                kind.as_str()
            ),
        ),
        sealed_ref(
            "role",
            &format!("syn.m6.org.multi-view/role/{}", kind.as_str()),
        ),
        sealed_ref("scope", "syn.m6.org.multi-view/scope/global-read-only"),
        sealed_ref(
            "object",
            &format!("syn.m6.org.multi-view/consultation/{consultation_id}"),
        ),
        sealed_ref(
            "channel",
            "syn.m6.org.multi-view/channel/internal-consultation",
        ),
        sealed_ref(
            "permission",
            "syn.m6.org.multi-view/permission/read-only-no-project-write",
        ),
    )
    .map_err(|_| "m6_org_multi_view_binding_invalid".to_string())
}

fn m3_metadata(
    repository: &M3RoleSessionSqliteRepository,
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
        .map_err(|_| "m6_org_multi_view_metadata_invalid".to_string())?,
        request_idempotency_key: RequestIdempotencyKey::try_from_canonical(sealed_ref(
            "request",
            &format!("{material}/idempotency"),
        ))
        .map_err(|_| "m6_org_multi_view_metadata_invalid".to_string())?,
        occurred_at: repository
            .capture_server_utc_now()
            .map_err(|error| format!("m6_org_multi_view_clock:{}", error.code))?,
    })
}

fn validate_start_request(
    request: &M6OrgStartMultiViewConsultationRequest,
    now_ms: i64,
) -> Result<(), String> {
    if now_ms < 0 {
        return Err("m6_org_multi_view_clock_invalid".to_string());
    }
    validate_ref("question_ref", &request.question_ref)?;
    validate_ref("idempotency_key", &request.idempotency_key)?;
    validate_refs("source_refs", &request.source_refs, false)?;
    if request.source_refs.len() > MAX_SOURCE_REFS {
        return Err("m6_org_multi_view_source_refs_too_many".to_string());
    }
    if !request.escalation_trigger.requires_multi_view() {
        if !request.view_kinds.is_empty()
            || request.budget_limit_ref.is_some()
            || request.budget_limit_units.is_some()
            || request.deadline_at_ms.is_some()
        {
            return Err("m6_org_multi_view_routine_must_use_single_role".to_string());
        }
        return Ok(());
    }
    if request.view_kinds.len() < MIN_MULTI_VIEW_COUNT
        || request.view_kinds.len() > MAX_MULTI_VIEW_COUNT
        || request
            .view_kinds
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .len()
            != request.view_kinds.len()
    {
        return Err("m6_org_multi_view_independent_views_invalid".to_string());
    }
    let budget_ref = request
        .budget_limit_ref
        .as_deref()
        .ok_or_else(|| "m6_org_multi_view_budget_ref_required".to_string())?;
    validate_ref("budget_limit_ref", budget_ref)?;
    let budget = request
        .budget_limit_units
        .ok_or_else(|| "m6_org_multi_view_budget_required".to_string())?;
    if budget == 0 || budget > MAX_BUDGET_UNITS {
        return Err("m6_org_multi_view_budget_invalid".to_string());
    }
    let deadline = request
        .deadline_at_ms
        .ok_or_else(|| "m6_org_multi_view_deadline_required".to_string())?;
    if deadline <= now_ms {
        return Err("m6_org_multi_view_deadline_invalid".to_string());
    }
    Ok(())
}

fn validate_independent_views(
    packet: &M6OrgQuestionPacket,
    views: &[M6OrgConsultationView],
) -> Result<(), String> {
    if views.len() < MIN_MULTI_VIEW_COUNT {
        return Err("m6_org_multi_view_independent_views_required".to_string());
    }
    let mut role_sessions = BTreeSet::new();
    let mut workcells = BTreeSet::new();
    let mut contexts = BTreeSet::new();
    for view in views {
        if view.question_packet_id != packet.question_packet_id
            || view.question_packet_hash != packet.packet_hash
            || view.peer_conclusions_readable_before_submit
            || view.dispatch_input_refs
                != vec![
                    packet.question_packet_id.clone(),
                    view.context_packet_ref.clone(),
                ]
            || !role_sessions.insert(view.role_session_id.as_str())
            || !workcells.insert(view.workcell_ref.as_str())
            || !contexts.insert(view.context_packet_ref.as_str())
        {
            return Err("m6_org_multi_view_independence_violation".to_string());
        }
    }
    Ok(())
}

fn validate_submit_request(
    request: &M6OrgSubmitConsultationViewRequest,
    now_ms: i64,
) -> Result<(), String> {
    if now_ms < 0 || request.peer_conclusions_readable_before_submit {
        return Err("m6_org_multi_view_peer_read_before_submit_rejected".to_string());
    }
    for (field, value) in [
        ("consultation_id", request.consultation_id.as_str()),
        ("view_id", request.view_id.as_str()),
        ("role_session_id", request.role_session_id.as_str()),
        ("workcell_ref", request.workcell_ref.as_str()),
        ("context_packet_ref", request.context_packet_ref.as_str()),
        ("question_packet_id", request.question_packet_id.as_str()),
        (
            "runtime_final_answer_ref",
            request.runtime_final_answer_ref.as_str(),
        ),
        ("idempotency_key", request.idempotency_key.as_str()),
    ] {
        validate_ref(field, value)?;
    }
    validate_sha256("question_packet_hash", &request.question_packet_hash)?;
    validate_sha256(
        "runtime_final_answer_hash",
        &request.runtime_final_answer_hash,
    )?;
    if request.claims.is_empty() || request.claims.len() > MAX_CLAIMS_PER_VIEW {
        return Err("m6_org_multi_view_claims_invalid".to_string());
    }
    let mut topics = BTreeSet::new();
    for claim in &request.claims {
        validate_ref("claim_topic_ref", &claim.topic_ref)?;
        validate_refs("claim_evidence_refs", &claim.evidence_refs, false)?;
        if !topics.insert(claim.topic_ref.as_str()) {
            return Err("m6_org_multi_view_claim_topic_duplicate".to_string());
        }
    }
    Ok(())
}

fn validate_submission_binding(
    header: &M6OrgConsultationHeader,
    target: &M6OrgConsultationView,
    request: &M6OrgSubmitConsultationViewRequest,
) -> Result<(), String> {
    if target.submitted {
        return Err("m6_org_multi_view_view_already_submitted".to_string());
    }
    if target.consultation_id != request.consultation_id
        || target.view_id != request.view_id
        || target.role_session_id != request.role_session_id
        || target.workcell_ref != request.workcell_ref
        || target.context_packet_ref != request.context_packet_ref
        || target.question_packet_id != request.question_packet_id
        || target.question_packet_hash != request.question_packet_hash
        || request.runtime_input_refs != target.dispatch_input_refs
        || target.peer_conclusions_readable_before_submit
    {
        return Err("m6_org_multi_view_submission_binding_mismatch".to_string());
    }
    let sources = header
        .question_packet
        .source_refs
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if request
        .claims
        .iter()
        .flat_map(|claim| claim.evidence_refs.iter())
        .any(|reference| !sources.contains(reference.as_str()))
    {
        return Err("m6_org_multi_view_claim_evidence_not_in_question_packet".to_string());
    }
    Ok(())
}

fn build_indexes(
    views: &[M6OrgConsultationView],
) -> Result<
    (
        Vec<M6OrgConsensusEntry>,
        Vec<M6OrgDisagreementEntry>,
        Vec<M6OrgEvidenceIndexEntry>,
    ),
    String,
> {
    if views.len() < MIN_MULTI_VIEW_COUNT || views.iter().any(|view| !view.submitted) {
        return Err("m6_org_multi_view_partial_result_not_assemblable".to_string());
    }
    let mut topics = BTreeSet::new();
    let mut evidence_index = Vec::new();
    for view in views {
        let conclusion_ref = view
            .conclusion_ref
            .as_ref()
            .ok_or_else(|| "m6_org_multi_view_conclusion_ref_missing".to_string())?;
        for claim in &view.claims {
            topics.insert(claim.topic_ref.clone());
            evidence_index.push(M6OrgEvidenceIndexEntry {
                view_id: view.view_id.clone(),
                conclusion_ref: conclusion_ref.clone(),
                topic_ref: claim.topic_ref.clone(),
                evidence_refs: claim.evidence_refs.clone(),
            });
        }
    }
    evidence_index.sort_by(|left, right| {
        (&left.topic_ref, &left.view_id).cmp(&(&right.topic_ref, &right.view_id))
    });
    let mut consensus = Vec::new();
    let mut disagreements = Vec::new();
    for topic in topics {
        let mut positions = Vec::new();
        for view in views {
            let claim = view.claims.iter().find(|claim| claim.topic_ref == topic);
            positions.push((view, claim));
        }
        let all_present = positions.iter().all(|(_, claim)| claim.is_some());
        let first_position = positions
            .first()
            .and_then(|(_, claim)| claim.as_ref())
            .map(|claim| claim.position);
        let all_equal = first_position.is_some()
            && positions
                .iter()
                .all(|(_, claim)| claim.as_ref().map(|claim| claim.position) == first_position);
        if all_present && all_equal {
            consensus.push(M6OrgConsensusEntry {
                topic_ref: topic,
                position: first_position.expect("checked present"),
                view_conclusion_refs: positions
                    .iter()
                    .map(|(view, _)| view.conclusion_ref.clone().expect("submitted conclusion"))
                    .collect(),
                evidence_refs: positions
                    .iter()
                    .flat_map(|(_, claim)| claim.expect("checked present").evidence_refs.clone())
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect(),
            });
        } else {
            disagreements.push(M6OrgDisagreementEntry {
                topic_ref: topic.clone(),
                view_positions: positions
                    .into_iter()
                    .map(|(view, claim)| M6OrgDisagreementViewPosition {
                        view_id: view.view_id.clone(),
                        conclusion_ref: view.conclusion_ref.clone().expect("submitted conclusion"),
                        position: claim.map(|claim| claim.position),
                        evidence_refs: claim
                            .map(|claim| claim.evidence_refs.clone())
                            .unwrap_or_default(),
                    })
                    .collect(),
            });
        }
    }
    Ok((consensus, disagreements, evidence_index))
}

fn consultation_from(
    header: M6OrgConsultationHeader,
    views: Vec<M6OrgConsultationView>,
    decision_request: Option<M6OrgConsultationDecisionRequest>,
) -> M6OrgMultiViewConsultation {
    M6OrgMultiViewConsultation {
        consultation_id: header.consultation_id,
        question_packet: header.question_packet,
        escalation_trigger: header.escalation_trigger,
        views,
        consensus_index_ref: header.consensus_index_ref,
        consensus: header.consensus,
        disagreement_index_ref: header.disagreement_index_ref,
        disagreements: header.disagreements,
        evidence_index_ref: header.evidence_index_ref,
        evidence_index: header.evidence_index,
        budget_limit_ref: header.budget_limit_ref,
        budget_limit_units: header.budget_limit_units,
        budget_state: header.budget_state,
        deadline_at_ms: header.deadline_at_ms,
        timeout_state: header.timeout_state,
        result_state: header.result_state,
        user_pending_decision_request_id: header.user_pending_decision_request_id,
        decision_request,
        produces_command: header.produces_command,
        produces_grant: header.produces_grant,
        produces_fact: header.produces_fact,
        revision: header.revision,
        created_at_ms: header.created_at_ms,
        updated_at_ms: header.updated_at_ms,
    }
}

fn insert_receipt(
    transaction: &rusqlite::Transaction<'_>,
    idempotency_key: &str,
    operation: &str,
    request_hash: &str,
    consultation_id: &str,
    now_ms: i64,
) -> Result<(), String> {
    transaction
        .execute(
            "INSERT INTO m6_consultation_command_receipts (
                idempotency_key,operation,request_hash,consultation_id,recorded_at_ms
             ) VALUES (?1,?2,?3,?4,?5)",
            params![
                idempotency_key,
                operation,
                request_hash,
                consultation_id,
                now_ms
            ],
        )
        .map_err(|error| format!("m6_org_multi_view_receipt_insert:{error}"))?;
    Ok(())
}

fn insert_audit(
    transaction: &rusqlite::Transaction<'_>,
    event_id: &str,
    event_type: &str,
    target_ref: &str,
    payload: &impl Serialize,
    now_ms: i64,
) -> Result<(), String> {
    transaction
        .execute(
            "INSERT INTO m6_org_audit_events (
                event_id,event_type,target_ref,payload_json,created_at_ms
             ) VALUES (?1,?2,?3,?4,?5)",
            params![
                event_id,
                event_type,
                target_ref,
                encode(payload, "m6_org_multi_view_audit")?,
                now_ms
            ],
        )
        .map_err(|error| format!("m6_org_multi_view_audit_insert:{error}"))?;
    Ok(())
}

fn load_json_required<P, T>(
    connection: &Connection,
    sql: &str,
    params: P,
    prefix: &str,
) -> Result<T, String>
where
    P: rusqlite::Params,
    T: DeserializeOwned,
{
    let payload = connection
        .query_row(sql, params, |row| row.get::<_, String>(0))
        .optional()
        .map_err(|error| format!("{prefix}:{error}"))?
        .ok_or_else(|| format!("{prefix}:not_found"))?;
    serde_json::from_str(&payload).map_err(|error| format!("{prefix}_decode:{error}"))
}

fn load_json_optional<P, T>(
    connection: &Connection,
    sql: &str,
    params: P,
    prefix: &str,
) -> Result<Option<T>, String>
where
    P: rusqlite::Params,
    T: DeserializeOwned,
{
    let payload = connection
        .query_row(sql, params, |row| row.get::<_, String>(0))
        .optional()
        .map_err(|error| format!("{prefix}:{error}"))?;
    payload
        .map(|payload| {
            serde_json::from_str(&payload).map_err(|error| format!("{prefix}_decode:{error}"))
        })
        .transpose()
}

fn validate_refs(field: &str, refs: &[String], allow_empty: bool) -> Result<(), String> {
    if !allow_empty && refs.is_empty() {
        return Err(format!("m6_org_multi_view_{field}_required"));
    }
    let mut unique = BTreeSet::new();
    for reference in refs {
        validate_ref(field, reference)?;
        if !unique.insert(reference.as_str()) {
            return Err(format!("m6_org_multi_view_{field}_duplicate"));
        }
    }
    Ok(())
}

fn validate_ref(field: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty()
        || value.len() > MAX_REF_LEN
        || value.chars().any(|character| character.is_control())
    {
        return Err(format!("m6_org_multi_view_{field}_invalid"));
    }
    Ok(())
}

fn validate_sha256(field: &str, value: &str) -> Result<(), String> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("m6_org_multi_view_{field}_invalid"));
    }
    Ok(())
}

fn stable_hash(value: &impl Serialize, prefix: &str) -> Result<String, String> {
    let bytes = serde_json::to_vec(value).map_err(|error| format!("{prefix}_serialize:{error}"))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn digest_id(namespace: &str, parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"syn.m6.org.multi-view/v1\0");
    hasher.update(namespace.as_bytes());
    for part in parts {
        hasher.update(b"\0");
        hasher.update(part.as_bytes());
    }
    format!("{namespace}:sha256:{:x}", hasher.finalize())
}

fn sealed_ref(namespace: &str, material: &str) -> String {
    format!(
        "{namespace}:sha256:{}",
        Sha256Digest::of_bytes(material.as_bytes()).as_str()
    )
}

fn opaque_ref(namespace: &str, material: &str) -> Result<OpaqueRef, String> {
    OpaqueRef::try_from_canonical(sealed_ref(namespace, material))
        .map_err(|_| "m6_org_multi_view_metadata_invalid".to_string())
}

fn encode(value: &impl Serialize, prefix: &str) -> Result<String, String> {
    serde_json::to_string(value).map_err(|error| format!("{prefix}_serialize:{error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    const NOW_MS: i64 = 1_787_097_600_000;
    static SCRATCH_SEQUENCE: AtomicU64 = AtomicU64::new(1);

    struct Fixture {
        root: PathBuf,
        app_data_root: PathBuf,
        state: crate::AppState,
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    fn fixture(label: &str) -> Fixture {
        let sequence = SCRATCH_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "syn-m6d07-{label}-{}-{sequence}",
            std::process::id()
        ));
        let app_data_root = root.join(crate::m1_project_index::M1_ORDINARY_APP_DATA_DIR_NAME);
        std::fs::create_dir_all(&app_data_root).expect("create M6D07 app-data root");
        let app_data_root =
            std::fs::canonicalize(&app_data_root).expect("canonical M6D07 app-data root");
        let seeds = root.join("synthetic-ordinary-product-seeds");
        std::fs::create_dir_all(&seeds).expect("create M6D07 seeds");
        let index_seed = seeds.join("codex-index.json");
        let tasks_seed = seeds.join("README.md");
        std::fs::write(&index_seed, r#"{"projects":[]}"#).expect("write index seed");
        std::fs::write(&tasks_seed, "# synthetic M6D07 tasks\n").expect("write tasks seed");
        let state = crate::AppState::try_new_with_tauri_ordinary_product_seeds(
            &app_data_root,
            &index_seed,
            &tasks_seed,
        )
        .expect("ordinary M6D07 AppState");
        Fixture {
            root,
            app_data_root,
            state,
        }
    }

    fn source_ref(material: &str) -> String {
        digest_id("summary-ref", &[material])
    }

    fn multi_request(label: &str) -> M6OrgStartMultiViewConsultationRequest {
        M6OrgStartMultiViewConsultationRequest {
            question_ref: digest_id("question-ref", &[label]),
            source_refs: vec![source_ref("project-a"), source_ref("project-b")],
            escalation_trigger: M6OrgConsultationEscalationTrigger::CrossProjectConflict,
            view_kinds: vec![
                M6OrgConsultationViewKind::RiskAnalyst,
                M6OrgConsultationViewKind::CounterfactualReviewer,
            ],
            budget_limit_ref: Some(digest_id("budget-limit", &[label])),
            budget_limit_units: Some(20),
            deadline_at_ms: Some(NOW_MS + 10_000),
            idempotency_key: format!("m6d07-start-{label}"),
        }
    }

    fn unwrap_multi(response: M6OrgConsultationRouteResponse) -> M6OrgMultiViewConsultation {
        match response {
            M6OrgConsultationRouteResponse::MultiView { consultation } => consultation,
            M6OrgConsultationRouteResponse::SingleRole { .. } => panic!("expected multi-view"),
        }
    }

    fn submit_request(
        consultation: &M6OrgMultiViewConsultation,
        index: usize,
        idempotency_key: &str,
        reported_cost_units: u64,
        shared_position: M6OrgConsultationClaimPosition,
        disputed_position: M6OrgConsultationClaimPosition,
    ) -> M6OrgSubmitConsultationViewRequest {
        let view = &consultation.views[index];
        M6OrgSubmitConsultationViewRequest {
            consultation_id: consultation.consultation_id.clone(),
            view_id: view.view_id.clone(),
            role_session_id: view.role_session_id.clone(),
            workcell_ref: view.workcell_ref.clone(),
            context_packet_ref: view.context_packet_ref.clone(),
            question_packet_id: view.question_packet_id.clone(),
            question_packet_hash: view.question_packet_hash.clone(),
            runtime_input_refs: view.dispatch_input_refs.clone(),
            runtime_final_answer_ref: digest_id("runtime-final-candidate", &[idempotency_key]),
            runtime_final_answer_hash: format!("{:x}", Sha256::digest(idempotency_key.as_bytes())),
            claims: vec![
                M6OrgConsultationClaim {
                    topic_ref: "topic:shared-risk".to_string(),
                    position: shared_position,
                    evidence_refs: vec![consultation.question_packet.source_refs[0].clone()],
                },
                M6OrgConsultationClaim {
                    topic_ref: "topic:disputed-plan".to_string(),
                    position: disputed_position,
                    evidence_refs: vec![consultation.question_packet.source_refs[1].clone()],
                },
            ],
            reported_cost_units,
            peer_conclusions_readable_before_submit: false,
            idempotency_key: idempotency_key.to_string(),
        }
    }

    fn m3_role_session_count(fixture: &Fixture) -> i64 {
        let connection = Connection::open(
            fixture
                .app_data_root
                .join(crate::m3_role_session_repository::M3_ORDINARY_ROLE_SESSION_RELATIVE_PATH),
        )
        .expect("open M3 store");
        connection
            .query_row("SELECT COUNT(*) FROM m3_role_sessions", [], |row| {
                row.get(0)
            })
            .expect("count role sessions")
    }

    fn file_hash(path: &Path) -> String {
        match std::fs::read(path) {
            Ok(bytes) => format!("present:sha256:{:x}", Sha256::digest(bytes)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => "absent".to_string(),
            Err(error) => panic!("read file {}: {error}", path.display()),
        }
    }

    #[test]
    fn m6d07_routine_question_stays_single_role_without_consultation_or_sessions() {
        let fixture = fixture("routine-single");
        let before_sessions = m3_role_session_count(&fixture);
        let m6_path = fixture.state.m6_org_store_path().expect("M6 path");
        assert!(!m6_path.exists());
        let request = M6OrgStartMultiViewConsultationRequest {
            question_ref: digest_id("question-ref", &["routine"]),
            source_refs: vec![source_ref("routine-source")],
            escalation_trigger: M6OrgConsultationEscalationTrigger::Routine,
            view_kinds: Vec::new(),
            budget_limit_ref: None,
            budget_limit_units: None,
            deadline_at_ms: None,
            idempotency_key: "m6d07-routine".to_string(),
        };
        let response = start_for_state(&fixture.state, &request, NOW_MS).expect("route routine");
        match response {
            M6OrgConsultationRouteResponse::SingleRole {
                question_packet,
                escalation_required,
                consultation_created,
            } => {
                assert!(question_packet.minimal);
                assert!(!escalation_required);
                assert!(!consultation_created);
            }
            _ => panic!("routine question escalated"),
        }
        assert_eq!(m3_role_session_count(&fixture), before_sessions);
        assert!(!m6_path.exists());
    }

    #[test]
    fn m6d07_start_creates_distinct_m3_sessions_workcells_and_context_packets() {
        let fixture = fixture("independent-start");
        let before_sessions = m3_role_session_count(&fixture);
        let consultation = unwrap_multi(
            start_for_state(&fixture.state, &multi_request("independent"), NOW_MS)
                .expect("start multi-view"),
        );
        assert_eq!(consultation.views.len(), 2);
        assert_eq!(m3_role_session_count(&fixture), before_sessions + 2);
        assert_eq!(
            consultation.result_state,
            M6OrgConsultationResultState::InFlight
        );
        let left = &consultation.views[0];
        let right = &consultation.views[1];
        assert_ne!(left.role_session_id, right.role_session_id);
        assert_ne!(left.workcell_ref, right.workcell_ref);
        assert_ne!(left.context_packet_ref, right.context_packet_ref);
        assert_eq!(left.question_packet_id, right.question_packet_id);
        assert_eq!(left.question_packet_hash, right.question_packet_hash);
        assert!(!left.peer_conclusions_readable_before_submit);
        assert!(!right.peer_conclusions_readable_before_submit);

        let connection =
            Connection::open(fixture.state.m6_org_store_path().unwrap()).expect("open M6 store");
        let mut statement = connection
            .prepare(
                "SELECT view_id,payload_json FROM m6_consultation_views
                 WHERE consultation_id=?1 ORDER BY view_id",
            )
            .unwrap();
        let rows = statement
            .query_map([&consultation.consultation_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(rows.len(), 2);
        assert!(!rows[0].1.contains(&rows[1].0));
        assert!(!rows[1].1.contains(&rows[0].0));
    }

    #[test]
    fn m6d07_peer_contamination_and_binding_mismatch_fail_before_submit() {
        let fixture = fixture("peer-reject");
        let consultation = unwrap_multi(
            start_for_state(&fixture.state, &multi_request("peer-reject"), NOW_MS).expect("start"),
        );
        let mut contaminated = submit_request(
            &consultation,
            0,
            "m6d07-submit-contaminated",
            2,
            M6OrgConsultationClaimPosition::Support,
            M6OrgConsultationClaimPosition::Support,
        );
        contaminated.peer_conclusions_readable_before_submit = true;
        assert_eq!(
            submit_view_for_state(&fixture.state, &contaminated, NOW_MS + 1).unwrap_err(),
            "m6_org_multi_view_peer_read_before_submit_rejected"
        );
        let mut mismatched = contaminated.clone();
        mismatched.peer_conclusions_readable_before_submit = false;
        mismatched
            .runtime_input_refs
            .push("conclusion:peer".to_string());
        assert_eq!(
            submit_view_for_state(&fixture.state, &mismatched, NOW_MS + 1).unwrap_err(),
            "m6_org_multi_view_submission_binding_mismatch"
        );
        let unknown_field = serde_json::json!({
            "consultation_id": consultation.consultation_id,
            "view_id": consultation.views[0].view_id,
            "role_session_id": consultation.views[0].role_session_id,
            "workcell_ref": consultation.views[0].workcell_ref,
            "context_packet_ref": consultation.views[0].context_packet_ref,
            "question_packet_id": consultation.views[0].question_packet_id,
            "question_packet_hash": consultation.views[0].question_packet_hash,
            "runtime_input_refs": consultation.views[0].dispatch_input_refs,
            "runtime_final_answer_ref": "answer:ref",
            "runtime_final_answer_hash": "0".repeat(64),
            "claims": [{"topic_ref":"topic:x","position":"SUPPORT","evidence_refs":[source_ref("project-a")]}],
            "reported_cost_units": 1,
            "peer_conclusions_readable_before_submit": false,
            "peer_conclusion_refs": ["conclusion:peer"],
            "idempotency_key": "unknown-field"
        });
        assert!(
            serde_json::from_value::<M6OrgSubmitConsultationViewRequest>(unknown_field).is_err()
        );
        let connection = Connection::open(fixture.state.m6_org_store_path().unwrap()).unwrap();
        let submitted: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM m6_consultation_views WHERE submitted=1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(submitted, 0);
    }

    #[test]
    fn m6d07_all_views_assemble_sourced_consensus_disagreement_and_pending_decision() {
        let fixture = fixture("assemble");
        let product_hashes = [
            file_hash(&fixture.state.index_path),
            file_hash(&fixture.state.tasks_path),
            file_hash(&fixture.state.workflow_state_path),
        ];
        let consultation = unwrap_multi(
            start_for_state(&fixture.state, &multi_request("assemble"), NOW_MS).expect("start"),
        );
        let after_first = submit_view_for_state(
            &fixture.state,
            &submit_request(
                &consultation,
                0,
                "m6d07-submit-a",
                3,
                M6OrgConsultationClaimPosition::Support,
                M6OrgConsultationClaimPosition::Support,
            ),
            NOW_MS + 1,
        )
        .expect("submit first");
        assert_eq!(
            after_first.result_state,
            M6OrgConsultationResultState::InFlight
        );
        assert!(after_first.decision_request.is_none());
        let partial = assemble_for_state(
            &fixture.state,
            &M6OrgAssembleMultiViewConsultationRequest {
                consultation_id: after_first.consultation_id.clone(),
                expected_revision: after_first.revision,
                idempotency_key: "m6d07-assemble-partial".to_string(),
            },
            NOW_MS + 2,
        )
        .expect("partial assembly stays incomplete");
        assert_eq!(partial.result_state, M6OrgConsultationResultState::InFlight);
        assert!(partial.consensus.is_empty());
        assert!(partial.disagreements.is_empty());
        assert!(partial.evidence_index.is_empty());
        assert!(partial.decision_request.is_none());
        let after_second = submit_view_for_state(
            &fixture.state,
            &submit_request(
                &after_first,
                1,
                "m6d07-submit-b",
                4,
                M6OrgConsultationClaimPosition::Support,
                M6OrgConsultationClaimPosition::Oppose,
            ),
            NOW_MS + 3,
        )
        .expect("submit second");
        assert_eq!(
            after_second.result_state,
            M6OrgConsultationResultState::Submitted
        );
        let assembled = assemble_for_state(
            &fixture.state,
            &M6OrgAssembleMultiViewConsultationRequest {
                consultation_id: after_second.consultation_id.clone(),
                expected_revision: after_second.revision,
                idempotency_key: "m6d07-assemble".to_string(),
            },
            NOW_MS + 4,
        )
        .expect("assemble");
        assert_eq!(
            assembled.result_state,
            M6OrgConsultationResultState::Assembled
        );
        assert_eq!(assembled.consensus.len(), 1);
        assert_eq!(assembled.consensus[0].topic_ref, "topic:shared-risk");
        assert_eq!(assembled.disagreements.len(), 1);
        assert_eq!(assembled.disagreements[0].topic_ref, "topic:disputed-plan");
        assert_eq!(assembled.evidence_index.len(), 4);
        let decision = assembled
            .decision_request
            .as_ref()
            .expect("pending decision");
        assert_eq!(decision.status, "PENDING_USER_DECISION");
        assert!(!decision.creates_project_command);
        assert!(!decision.creates_grant);
        assert!(!decision.creates_formal_fact);
        assert!(
            !assembled.produces_command && !assembled.produces_grant && !assembled.produces_fact
        );
        assert_eq!(
            product_hashes,
            [
                file_hash(&fixture.state.index_path),
                file_hash(&fixture.state.tasks_path),
                file_hash(&fixture.state.workflow_state_path),
            ]
        );
    }

    #[test]
    fn m6d07_timeout_is_explicit_and_partial_results_never_assemble() {
        let fixture = fixture("timeout");
        let mut request = multi_request("timeout");
        request.deadline_at_ms = Some(NOW_MS + 5);
        let consultation = unwrap_multi(
            start_for_state(&fixture.state, &request, NOW_MS).expect("start timeout case"),
        );
        let timed_out = submit_view_for_state(
            &fixture.state,
            &submit_request(
                &consultation,
                0,
                "m6d07-timeout-submit",
                1,
                M6OrgConsultationClaimPosition::Support,
                M6OrgConsultationClaimPosition::Support,
            ),
            NOW_MS + 6,
        )
        .expect("explicit timeout");
        assert_eq!(
            timed_out.timeout_state,
            M6OrgConsultationTimeoutState::TimedOut
        );
        assert_eq!(
            timed_out.result_state,
            M6OrgConsultationResultState::TimedOut
        );
        assert!(timed_out.views.iter().all(|view| !view.submitted));
        assert!(timed_out.decision_request.is_none());
        assert!(timed_out.consensus.is_empty());
        assert!(timed_out.disagreements.is_empty());
    }

    #[test]
    fn m6d07_budget_exceeded_is_explicit_and_drops_no_partial_candidate_into_result() {
        let fixture = fixture("budget");
        let mut request = multi_request("budget");
        request.budget_limit_units = Some(10);
        let consultation = unwrap_multi(
            start_for_state(&fixture.state, &request, NOW_MS).expect("start budget case"),
        );
        assert_eq!(consultation.views[0].budget_cap_units, 5);
        let exceeded = submit_view_for_state(
            &fixture.state,
            &submit_request(
                &consultation,
                0,
                "m6d07-over-budget-submit",
                6,
                M6OrgConsultationClaimPosition::Support,
                M6OrgConsultationClaimPosition::Support,
            ),
            NOW_MS + 1,
        )
        .expect("explicit budget state");
        assert_eq!(
            exceeded.budget_state,
            M6OrgConsultationBudgetState::BudgetExceeded
        );
        assert_eq!(
            exceeded.result_state,
            M6OrgConsultationResultState::BudgetExceeded
        );
        assert!(exceeded.views.iter().all(|view| !view.submitted));
        assert!(exceeded.decision_request.is_none());
    }

    #[test]
    fn m6d07_replay_converges_and_persistence_contains_refs_not_answer_body() {
        let fixture = fixture("replay-no-body");
        let request = multi_request("replay-no-body");
        let first =
            unwrap_multi(start_for_state(&fixture.state, &request, NOW_MS).expect("first start"));
        let replay = unwrap_multi(
            start_for_state(&fixture.state, &request, NOW_MS + 50).expect("start replay"),
        );
        assert_eq!(first, replay);
        let submit = submit_request(
            &first,
            0,
            "m6d07-replay-submit",
            2,
            M6OrgConsultationClaimPosition::Support,
            M6OrgConsultationClaimPosition::Support,
        );
        let submitted =
            submit_view_for_state(&fixture.state, &submit, NOW_MS + 1).expect("first submit");
        let submitted_replay =
            submit_view_for_state(&fixture.state, &submit, NOW_MS + 2).expect("submit replay");
        assert_eq!(submitted, submitted_replay);
        let bytes = std::fs::read(fixture.state.m6_org_store_path().unwrap()).expect("read M6 DB");
        let lossy = String::from_utf8_lossy(&bytes);
        assert!(!lossy.contains("THIS WOULD BE A RAW RUNTIME FINAL ANSWER BODY"));
        let connection = Connection::open(fixture.state.m6_org_store_path().unwrap()).unwrap();
        let count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM m6_consultation_views WHERE submitted=1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn m6d07_three_production_commands_are_in_the_ordinary_handler() {
        let registry = include_str!("command_registry.rs");
        let commands = include_str!("commands.rs");
        let library = include_str!("lib.rs");
        for command in [
            "start_global_supervisor_multi_view_consultation",
            "submit_global_supervisor_consultation_view",
            "assemble_global_supervisor_multi_view_consultation",
        ] {
            assert_eq!(registry.matches(command).count(), 1, "registry {command}");
            assert_eq!(commands.matches(&format!("fn {command}")).count(), 1);
        }
        assert_eq!(
            library
                .matches("mod m6_org_multi_view_consultation;")
                .count(),
            1
        );
        let production = include_str!("m6_org_multi_view_consultation.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source");
        assert!(production.contains("create_role_session"));
        assert!(production.contains("load_submission_target"));
        assert!(!production.contains("project_root"));
        assert!(!production.contains("open_m5_store"));
        assert!(!production.contains("provider_response"));
    }
}
