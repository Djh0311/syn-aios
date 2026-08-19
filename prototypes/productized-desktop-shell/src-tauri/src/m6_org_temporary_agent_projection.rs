//! M6D06 temporary-agent execution-history projection.
//!
//! M5 remains the immutable execution-fact owner.  Refresh opens that store
//! read-only, accepts only a complete independently joined execution envelope,
//! and writes ref-only history into the M6 organization store.  A report body,
//! runtime trace, session name, or parent/child naming convention can never
//! manufacture execution identity here.

use crate::m6_org_member_directory::{M6OrgRegisterStableMemberRequest, M6OrgStableMember};
use crate::m6_org_schema::ensure_m6_org_schema;
use rusqlite::{params, Connection, OpenFlags, OptionalExtension, Transaction};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::path::Path;

const MAX_SEARCH_LIMIT: u32 = 200;
const TEMPORARY_AGENT_PREFIX: &str = "temporary_agent_";
const TEMPORARY_AGENT_PAYLOAD_SCHEMA: &str = "syn.m6.org.temporary-agent/v1";
const M5_TEMPORARY_AGENT_REQUIRED_TABLES: [&str; 11] = [
    "m5_claims",
    "m5_work_items",
    "m5_prepared_attempts",
    "m5_execution_grants",
    "m5_dispatches",
    "m5_worker_role_session_bindings",
    "m5_execution_attempt_readbacks",
    "m5_command_receipts",
    "m5_events",
    "m5_audit_records",
    "m5_durable_operations",
];
const M5_SCHEMA_CARRIER_MISMATCH: &str = "m6_org_temporary_agent_m5_schema_carrier_mismatch";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum M6OrgTemporaryAgentLifecycle {
    Projected,
    Retained,
}

impl M6OrgTemporaryAgentLifecycle {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Projected => "PROJECTED",
            Self::Retained => "RETAINED",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct M6OrgExecutionHashes {
    pub(crate) grant_hash: String,
    pub(crate) report_hash: String,
    pub(crate) receipt_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct M6OrgExecutionEnvelope {
    pub(crate) project_id: String,
    pub(crate) orchestration_id: String,
    pub(crate) workflow_run_id: String,
    pub(crate) work_item_id: String,
    pub(crate) node_id: String,
    pub(crate) dispatch_id: String,
    pub(crate) attempt_id: String,
    pub(crate) grant_id: String,
    pub(crate) worker_role_session_id: String,
    pub(crate) authoritative_receipt_ref: String,
    pub(crate) trusted_actor_id: String,
    pub(crate) hashes: M6OrgExecutionHashes,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct M6OrgChildRunRef {
    pub(crate) child_run_ref: String,
    pub(crate) parent_workcell_ref: String,
    pub(crate) attempt_id: String,
    pub(crate) grant_id: String,
    pub(crate) trace_ref: String,
    pub(crate) reference_only: bool,
    pub(crate) creates_stable_member: bool,
    pub(crate) creates_temporary_agent: bool,
    pub(crate) creates_organization_hierarchy: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct M6OrgTemporaryAgent {
    pub(crate) schema: String,
    pub(crate) temporary_agent_id: String,
    pub(crate) execution_envelope: M6OrgExecutionEnvelope,
    pub(crate) membership_lifecycle: M6OrgTemporaryAgentLifecycle,
    pub(crate) display_name_ref: String,
    pub(crate) claim_ref: String,
    pub(crate) task_ref: String,
    pub(crate) task_state: String,
    pub(crate) result_ref: String,
    pub(crate) result_state: String,
    pub(crate) failure_ref: Option<String>,
    pub(crate) source_refs: Vec<String>,
    pub(crate) child_run_ref: Option<M6OrgChildRunRef>,
    pub(crate) report_body_copied: bool,
    pub(crate) auto_stabilize: bool,
    pub(crate) created_at: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct M6OrgTemporaryAgentQuarantineRecord {
    pub(crate) quarantine_ref: String,
    pub(crate) source_claim_ref: String,
    pub(crate) reason_code: String,
    pub(crate) source_refs: Vec<String>,
    pub(crate) payload_mode: String,
    pub(crate) recorded_at: i64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum M6OrgTemporaryAgentSourceState {
    CompatibleExecutionHistory,
    NoExecutionHistory,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct M6OrgTemporaryAgentRefreshResponse {
    pub(crate) source_state: M6OrgTemporaryAgentSourceState,
    pub(crate) projected_count: usize,
    pub(crate) retained_count: usize,
    pub(crate) quarantined_count: usize,
    pub(crate) ignored_non_execution_count: usize,
    pub(crate) records: Vec<M6OrgTemporaryAgent>,
    pub(crate) quarantines: Vec<M6OrgTemporaryAgentQuarantineRecord>,
    pub(crate) m5_opened_read_only: bool,
    pub(crate) report_bodies_copied: bool,
    pub(crate) automatic_promotions: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct M6OrgSearchTemporaryAgentHistoryRequest {
    pub(crate) query: String,
    pub(crate) limit: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct M6OrgTemporaryAgentHistoryView {
    pub(crate) temporary_agent: M6OrgTemporaryAgent,
    pub(crate) promoted_member_id: Option<String>,
    pub(crate) presented_as_temporary: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct M6OrgSearchTemporaryAgentHistoryResponse {
    pub(crate) matches: Vec<M6OrgTemporaryAgentHistoryView>,
    pub(crate) total_projected: usize,
    pub(crate) quarantine_count: usize,
    pub(crate) report_bodies_present: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE", deny_unknown_fields)]
pub(crate) enum M6OrgTemporaryAgentPromotionTarget {
    CreateStableMember {
        registration: M6OrgRegisterStableMemberRequest,
    },
    BindExistingStableMember {
        member_id: String,
        expected_revision: u64,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct M6OrgPromoteTemporaryAgentRequest {
    pub(crate) temporary_agent_id: String,
    pub(crate) promoted_by_actor_id: String,
    pub(crate) explicit_human_command: bool,
    pub(crate) target: M6OrgTemporaryAgentPromotionTarget,
    pub(crate) idempotency_key: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct M6OrgPromotionBinding {
    pub(crate) binding_id: String,
    pub(crate) member_id: String,
    pub(crate) promoted_from: String,
    pub(crate) promoted_by_actor_id: String,
    pub(crate) explicit_human_command: bool,
    pub(crate) created_at: i64,
    pub(crate) source_temporary_agent_type_unchanged: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct M6OrgTemporaryAgentPromotionResponse {
    pub(crate) binding: M6OrgPromotionBinding,
    pub(crate) stable_member: M6OrgStableMember,
    pub(crate) source_temporary_agent: M6OrgTemporaryAgent,
    pub(crate) replayed: bool,
}

#[derive(Clone, Debug)]
struct M5ClaimHeader {
    claim_id: String,
    report_kind: String,
    claim_status: String,
    project_id: String,
    orchestration_id: String,
    workflow_run_id: Option<String>,
    work_item_id: Option<String>,
    node_id: Option<String>,
    dispatch_id: Option<String>,
    attempt_id: Option<String>,
    grant_id: Option<String>,
    worker_role_session_id: Option<String>,
    authoritative_receipt_ref: Option<String>,
    report_hash: String,
    authenticated_actor_id: Option<String>,
    created_at_ms: i64,
}

#[derive(Clone, Debug)]
struct WorkItemSource {
    project_id: String,
    orchestration_id: String,
    workflow_run_id: String,
    source_object_ref: String,
    node_id: String,
    status: String,
}

#[derive(Clone, Debug)]
struct AttemptSource {
    state: String,
    project_id: String,
    orchestration_id: String,
    workflow_run_id: String,
    work_item_id: String,
    node_id: String,
    worker_role_session_id: String,
    grant_id: Option<String>,
    revision: i64,
}

#[derive(Clone, Debug)]
struct GrantSource {
    project_id: String,
    orchestration_id: String,
    workflow_run_id: String,
    work_item_id: String,
    attempt_id: String,
    principal_actor_id: String,
    worker_role_session_id: String,
    status: String,
    revoked_at_ms: Option<i64>,
    revision: i64,
    effect_key: String,
    grant_hash: String,
}

#[derive(Clone, Debug)]
struct DispatchSource {
    project_id: String,
    orchestration_id: String,
    workflow_run_id: String,
    work_item_id: String,
    node_id: String,
    attempt_id: String,
    grant_id: String,
    grant_revision: i64,
    worker_role_session_id: String,
    effect_id: String,
    state: String,
}

#[derive(Clone, Debug)]
struct BindingSource {
    project_id: String,
    orchestration_id: String,
    workflow_run_id: String,
    work_item_id: String,
    attempt_id: String,
    worker_role_session_id: String,
    principal_actor_id: String,
}

#[derive(Clone, Debug)]
struct ReadbackSource {
    grant_id: String,
    attempt_id: String,
    dispatch_id: String,
    effect_id: String,
    trace_hash: String,
    actor_binding: String,
    enforcement_status: String,
    outcome: String,
    derived_attempt_state: String,
    source_attempt_revision: i64,
    committed_attempt_revision: i64,
    canonical_readback_hash: String,
    recording_command_receipt_ref: String,
}

#[derive(Clone, Debug)]
struct ReceiptCarrier {
    command_id: String,
    actor_id: String,
    scope_ref: String,
    current_object_ref: Option<String>,
    status: String,
    result_ref: Option<String>,
    result_hash: Option<String>,
    committed_revision: Option<i64>,
}

#[derive(Clone, Debug)]
struct EventCarrier {
    event_type: String,
    actor_id: String,
    scope_ref: String,
    source_ref: String,
    source_revision: Option<String>,
    command_id: Option<String>,
    payload_hash: Option<String>,
}

#[derive(Clone, Debug)]
struct AuditCarrier {
    action: String,
    decision: String,
    actor_id: String,
    scope_ref: String,
    subject_ref: Option<String>,
    command_id: Option<String>,
    source_refs: Option<String>,
}

#[derive(Clone, Debug)]
struct DurableOperationSource {
    operation_id: String,
    attempt_id: String,
    project_id: String,
    orchestration_id: String,
    workflow_run_id: String,
    grant_id: String,
    dispatch_id: String,
    effect_id: String,
    last_receipt_id: Option<String>,
    error: Option<String>,
}

pub(crate) fn refresh_for_state(
    state: &crate::AppState,
    now_ms: i64,
) -> Result<M6OrgTemporaryAgentRefreshResponse, String> {
    require_temporary_agent_runtime(state)?;
    let m5_path = state
        .m5_store_path
        .as_deref()
        .ok_or_else(|| "m5_runtime_unavailable".to_string())?;
    let m6_path = state.m6_org_store_path()?;
    refresh_from_paths(m5_path, &m6_path, now_ms)
}

fn refresh_from_paths(
    m5_path: &Path,
    m6_path: &Path,
    now_ms: i64,
) -> Result<M6OrgTemporaryAgentRefreshResponse, String> {
    let m5 = Connection::open_with_flags(
        m5_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|error| format!("m6_org_temporary_agent_m5_read_only_open:{error}"))?;
    let mut missing_tables = Vec::new();
    for table in M5_TEMPORARY_AGENT_REQUIRED_TABLES {
        if !table_exists(&m5, table)? {
            missing_tables.push(table.to_string());
        }
    }
    if !missing_tables.is_empty() {
        return Err(M5_SCHEMA_CARRIER_MISMATCH.to_string());
    }
    let claims = load_claim_headers(&m5)?;
    let source_state = if claims.is_empty() {
        M6OrgTemporaryAgentSourceState::NoExecutionHistory
    } else {
        M6OrgTemporaryAgentSourceState::CompatibleExecutionHistory
    };
    let mut store = M6OrgTemporaryAgentStore::open(m6_path)?;

    let mut projected_count = 0usize;
    let mut retained_count = 0usize;
    let mut quarantined_count = 0usize;
    let mut ignored_non_execution_count = 0usize;
    for claim in claims {
        if claim.report_kind != "executed" {
            ignored_non_execution_count += 1;
            continue;
        }
        let projection = project_exact_claim(&m5, &claim);
        match projection {
            Ok(record) => {
                if store.record_projection(&record)? {
                    projected_count += 1;
                } else {
                    retained_count += 1;
                }
            }
            Err(reason_code) => {
                let quarantine = quarantine_for_claim(&claim, &reason_code, now_ms);
                if store.record_quarantine(&quarantine)? {
                    quarantined_count += 1;
                }
            }
        }
    }

    Ok(M6OrgTemporaryAgentRefreshResponse {
        source_state,
        projected_count,
        retained_count,
        quarantined_count,
        ignored_non_execution_count,
        records: store.list_agents()?,
        quarantines: store.list_quarantines()?,
        m5_opened_read_only: true,
        report_bodies_copied: false,
        automatic_promotions: 0,
    })
}

pub(crate) fn search_for_state(
    state: &crate::AppState,
    request: &M6OrgSearchTemporaryAgentHistoryRequest,
) -> Result<M6OrgSearchTemporaryAgentHistoryResponse, String> {
    require_temporary_agent_runtime(state)?;
    let query = request.query.trim().to_ascii_lowercase();
    if query.is_empty() {
        return Err("m6_org_temporary_agent_search_query_required".to_string());
    }
    if request.limit == 0 || request.limit > MAX_SEARCH_LIMIT {
        return Err("m6_org_temporary_agent_search_limit_invalid".to_string());
    }
    let store = M6OrgTemporaryAgentStore::open(&state.m6_org_store_path()?)?;
    let all = store.list_agents()?;
    let total_projected = all.len();
    let mut matches = Vec::new();
    for temporary_agent in all {
        if searchable_text(&temporary_agent).contains(&query) {
            let promoted_member_id =
                store.promoted_member_id(&temporary_agent.temporary_agent_id)?;
            matches.push(M6OrgTemporaryAgentHistoryView {
                temporary_agent,
                promoted_member_id,
                presented_as_temporary: true,
            });
            if matches.len() >= request.limit as usize {
                break;
            }
        }
    }
    Ok(M6OrgSearchTemporaryAgentHistoryResponse {
        matches,
        total_projected,
        quarantine_count: store.list_quarantines()?.len(),
        report_bodies_present: false,
    })
}

pub(crate) fn promote_for_state(
    state: &crate::AppState,
    request: &M6OrgPromoteTemporaryAgentRequest,
    now_ms: i64,
) -> Result<M6OrgTemporaryAgentPromotionResponse, String> {
    require_temporary_agent_runtime(state)?;
    validate_ref("temporary_agent_id", &request.temporary_agent_id)?;
    if !request
        .temporary_agent_id
        .starts_with(TEMPORARY_AGENT_PREFIX)
    {
        return Err("m6_org_temporary_agent_id_invalid".to_string());
    }
    validate_ref("promoted_by_actor_id", &request.promoted_by_actor_id)?;
    validate_ref("idempotency_key", &request.idempotency_key)?;
    if !request.explicit_human_command {
        return Err("m6_org_temporary_agent_promotion_requires_human_command".to_string());
    }
    let request_hash = stable_hash(request)?;
    let mut store = M6OrgTemporaryAgentStore::open(&state.m6_org_store_path()?)?;
    if let Some(mut response) = store
        .load_command_response::<M6OrgTemporaryAgentPromotionResponse>(
            &request.idempotency_key,
            "promote_temporary_agent",
            &request_hash,
        )?
    {
        response.replayed = true;
        return Ok(response);
    }
    let source = store
        .load_agent(&request.temporary_agent_id)?
        .ok_or_else(|| "m6_org_temporary_agent_not_found".to_string())?;
    let source_before = encode(&source, "m6_org_temporary_agent_source_before")?;
    if store
        .promoted_member_id(&request.temporary_agent_id)?
        .is_some()
    {
        return Err("m6_org_temporary_agent_already_promoted".to_string());
    }

    let stable_member = match &request.target {
        M6OrgTemporaryAgentPromotionTarget::CreateStableMember { registration } => {
            crate::m6_org_member_directory::register_promoted_for_state(
                state,
                registration,
                &request.temporary_agent_id,
                now_ms,
            )?
        }
        M6OrgTemporaryAgentPromotionTarget::BindExistingStableMember {
            member_id,
            expected_revision,
        } => crate::m6_org_member_directory::bind_existing_promotion_for_state(
            state,
            member_id,
            &request.temporary_agent_id,
            *expected_revision,
            &format!("temporary-agent-promotion:{}", request.idempotency_key),
            now_ms,
        )?,
    };
    if stable_member.promoted_from.as_deref() != Some(request.temporary_agent_id.as_str()) {
        return Err("m6_org_temporary_agent_promotion_binding_missing".to_string());
    }

    // Reopen after the directory helper committed through its own connection.
    store = M6OrgTemporaryAgentStore::open(&state.m6_org_store_path()?)?;
    let source_after = store
        .load_agent(&request.temporary_agent_id)?
        .ok_or_else(|| "m6_org_temporary_agent_source_lost".to_string())?;
    if source_before != encode(&source_after, "m6_org_temporary_agent_source_after")? {
        return Err("m6_org_temporary_agent_source_rewritten".to_string());
    }
    let binding = M6OrgPromotionBinding {
        binding_id: digest_id(
            "promotion-binding",
            &[&request.temporary_agent_id, &stable_member.member_id],
        ),
        member_id: stable_member.member_id.clone(),
        promoted_from: request.temporary_agent_id.clone(),
        promoted_by_actor_id: request.promoted_by_actor_id.clone(),
        explicit_human_command: true,
        created_at: now_ms,
        source_temporary_agent_type_unchanged: true,
    };
    let response = M6OrgTemporaryAgentPromotionResponse {
        binding,
        stable_member,
        source_temporary_agent: source_after,
        replayed: false,
    };
    store.record_promotion(&request.idempotency_key, &request_hash, &response, now_ms)?;
    Ok(response)
}

fn project_exact_claim(
    m5: &Connection,
    claim: &M5ClaimHeader,
) -> Result<M6OrgTemporaryAgent, String> {
    if claim.claim_status != "RECORDED_UNVERIFIED" {
        return Err("m6_org_temporary_agent_claim_not_authoritative".to_string());
    }
    let workflow_run_id = required(&claim.workflow_run_id)?;
    let work_item_id = required(&claim.work_item_id)?;
    let node_id = required(&claim.node_id)?;
    let dispatch_id = required(&claim.dispatch_id)?;
    let attempt_id = required(&claim.attempt_id)?;
    let grant_id = required(&claim.grant_id)?;
    let worker_role_session_id = required(&claim.worker_role_session_id)?;
    let authoritative_receipt_ref = required(&claim.authoritative_receipt_ref)?;
    let trusted_actor_id = required(&claim.authenticated_actor_id)?;
    for value in [
        claim.project_id.as_str(),
        claim.orchestration_id.as_str(),
        workflow_run_id,
        work_item_id,
        node_id,
        dispatch_id,
        attempt_id,
        grant_id,
        worker_role_session_id,
        authoritative_receipt_ref,
        trusted_actor_id,
    ] {
        if value.trim().is_empty() {
            return Err("m6_org_temporary_agent_incomplete_execution_envelope".to_string());
        }
    }

    let work = load_work_item(m5, work_item_id)?;
    let attempt = load_attempt(m5, attempt_id)?;
    let grant = load_grant(m5, grant_id)?;
    let dispatch = load_dispatch(m5, dispatch_id)?;
    let binding = load_binding(m5, attempt_id)?;
    let readback = load_readback(m5, authoritative_receipt_ref)?;
    let receipt = load_receipt_carrier(m5, &readback.recording_command_receipt_ref)?;
    let event = load_event_carrier(m5, authoritative_receipt_ref)?;
    let audit = load_audit_carrier(m5, authoritative_receipt_ref)?;

    let exact = work.project_id == claim.project_id
        && work.orchestration_id == claim.orchestration_id
        && work.workflow_run_id == workflow_run_id
        && work.node_id == node_id
        && attempt.project_id == claim.project_id
        && attempt.orchestration_id == claim.orchestration_id
        && attempt.workflow_run_id == workflow_run_id
        && attempt.work_item_id == work_item_id
        && attempt.node_id == node_id
        && attempt.worker_role_session_id == worker_role_session_id
        && attempt.grant_id.as_deref() == Some(grant_id)
        && grant.project_id == claim.project_id
        && grant.orchestration_id == claim.orchestration_id
        && grant.workflow_run_id == workflow_run_id
        && grant.work_item_id == work_item_id
        && grant.attempt_id == attempt_id
        && grant.principal_actor_id == trusted_actor_id
        && grant.worker_role_session_id == worker_role_session_id
        && grant.status == "ACTIVE"
        && grant.revoked_at_ms.is_none()
        && dispatch.project_id == claim.project_id
        && dispatch.orchestration_id == claim.orchestration_id
        && dispatch.workflow_run_id == workflow_run_id
        && dispatch.work_item_id == work_item_id
        && dispatch.node_id == node_id
        && dispatch.attempt_id == attempt_id
        && dispatch.grant_id == grant_id
        && dispatch.grant_revision == grant.revision
        && dispatch.worker_role_session_id == worker_role_session_id
        && dispatch.effect_id == grant.effect_key
        && dispatch.state == "DISPATCHED"
        && binding.project_id == claim.project_id
        && binding.orchestration_id == claim.orchestration_id
        && binding.workflow_run_id == workflow_run_id
        && binding.work_item_id == work_item_id
        && binding.attempt_id == attempt_id
        && binding.worker_role_session_id == worker_role_session_id
        && binding.principal_actor_id == trusted_actor_id
        && readback.grant_id == grant_id
        && readback.attempt_id == attempt_id
        && readback.dispatch_id == dispatch_id
        && readback.effect_id == dispatch.effect_id
        && readback.actor_binding == worker_role_session_id
        && readback.enforcement_status == "OK"
        && readback.derived_attempt_state == attempt.state
        && readback.committed_attempt_revision == attempt.revision
        && readback.source_attempt_revision + 1 == readback.committed_attempt_revision
        && readback.recording_command_receipt_ref
            == format!("rcpt-record-execution-attempt-readback-{authoritative_receipt_ref}")
        && readback.canonical_readback_hash
            == canonical_readback_hash(authoritative_receipt_ref, &readback)
        && receipt.command_id
            == format!("cmd-record-execution-attempt-readback-{authoritative_receipt_ref}")
        && receipt.actor_id == trusted_actor_id
        && receipt.scope_ref == claim.project_id
        && receipt.current_object_ref.as_deref() == Some(format!("attempt:{attempt_id}").as_str())
        && receipt.status == "COMMITTED"
        && receipt.result_ref.as_deref()
            == Some(format!("receipt:{authoritative_receipt_ref}").as_str())
        && receipt.result_hash.as_deref() == Some(readback.canonical_readback_hash.as_str())
        && receipt.committed_revision == Some(readback.committed_attempt_revision)
        && event.event_type == "ExecutionAttemptReadbackRecorded"
        && event.actor_id == trusted_actor_id
        && event.scope_ref == claim.project_id
        && event.source_ref == format!("attempt:{attempt_id}")
        && event.source_revision.as_deref()
            == Some(readback.committed_attempt_revision.to_string().as_str())
        && event.command_id.as_deref() == Some(receipt.command_id.as_str())
        && event.payload_hash.as_deref() == Some(readback.canonical_readback_hash.as_str())
        && audit.action == "COMMITTED"
        && audit.decision == "SCRUBBED_ATTEMPT_RECORD"
        && audit.actor_id == trusted_actor_id
        && audit.scope_ref == claim.project_id
        && audit.subject_ref.as_deref() == Some(format!("attempt:{attempt_id}").as_str())
        && audit.command_id.as_deref() == Some(receipt.command_id.as_str());
    if !exact {
        return Err("m6_org_temporary_agent_exact_join_mismatch".to_string());
    }
    let audit_refs = audit.source_refs.as_deref().unwrap_or_default();
    for expected in [
        format!("attempt:{attempt_id}"),
        format!("grant:{grant_id}"),
        format!("dispatch:{dispatch_id}"),
        format!("receipt:{authoritative_receipt_ref}"),
    ] {
        if !audit_refs.split(';').any(|actual| actual == expected) {
            return Err("m6_org_temporary_agent_audit_join_mismatch".to_string());
        }
    }
    for hash in [
        grant.grant_hash.as_str(),
        claim.report_hash.as_str(),
        readback.canonical_readback_hash.as_str(),
        readback.trace_hash.as_str(),
    ] {
        validate_hash(hash)?;
    }

    let envelope = M6OrgExecutionEnvelope {
        project_id: claim.project_id.clone(),
        orchestration_id: claim.orchestration_id.clone(),
        workflow_run_id: workflow_run_id.to_string(),
        work_item_id: work_item_id.to_string(),
        node_id: node_id.to_string(),
        dispatch_id: dispatch_id.to_string(),
        attempt_id: attempt_id.to_string(),
        grant_id: grant_id.to_string(),
        worker_role_session_id: worker_role_session_id.to_string(),
        authoritative_receipt_ref: authoritative_receipt_ref.to_string(),
        trusted_actor_id: trusted_actor_id.to_string(),
        hashes: M6OrgExecutionHashes {
            grant_hash: grant.grant_hash,
            report_hash: claim.report_hash.clone(),
            receipt_hash: readback.canonical_readback_hash.clone(),
        },
    };
    let child_run_ref = load_exact_child_run_ref(m5, &envelope, &dispatch.effect_id, &readback)?;
    let failure_ref = if readback.derived_attempt_state == "SUCCEEDED" {
        None
    } else {
        Some(format!(
            "attempt-state:{attempt_id}:{}",
            readback.derived_attempt_state
        ))
    };
    let mut source_refs = BTreeSet::from([
        format!("claim:{}", claim.claim_id),
        format!("work-item:{work_item_id}"),
        format!("attempt:{attempt_id}"),
        format!("grant:{grant_id}"),
        format!("dispatch:{dispatch_id}"),
        format!("receipt:{authoritative_receipt_ref}"),
        format!("command-receipt:{}", readback.recording_command_receipt_ref),
        format!("event:evt-execution-attempt-readback-{authoritative_receipt_ref}"),
        format!("audit:aud-execution-attempt-readback-{authoritative_receipt_ref}"),
        format!("actor:{trusted_actor_id}"),
        format!("role-session:{worker_role_session_id}"),
    ]);
    if let Some(child) = &child_run_ref {
        source_refs.insert(format!("child-run:{}", child.child_run_ref));
    }
    let temporary_agent_id = format!("{TEMPORARY_AGENT_PREFIX}{}", stable_hash(&envelope)?);
    Ok(M6OrgTemporaryAgent {
        schema: TEMPORARY_AGENT_PAYLOAD_SCHEMA.to_string(),
        temporary_agent_id,
        execution_envelope: envelope,
        membership_lifecycle: M6OrgTemporaryAgentLifecycle::Projected,
        display_name_ref: format!("actor:{trusted_actor_id}"),
        claim_ref: format!("claim:{}", claim.claim_id),
        task_ref: work.source_object_ref,
        task_state: work.status,
        result_ref: format!("receipt:{authoritative_receipt_ref}"),
        result_state: readback.outcome,
        failure_ref,
        source_refs: source_refs.into_iter().collect(),
        child_run_ref,
        report_body_copied: false,
        auto_stabilize: false,
        created_at: claim.created_at_ms,
    })
}

fn load_exact_child_run_ref(
    connection: &Connection,
    envelope: &M6OrgExecutionEnvelope,
    effect_id: &str,
    readback: &ReadbackSource,
) -> Result<Option<M6OrgChildRunRef>, String> {
    let operation = connection
        .query_row(
            "SELECT operation_id,attempt_id,project_id,orchestration_id,workflow_run_id,
                    grant_id,dispatch_id,effect_id,last_receipt_id,error
             FROM m5_durable_operations WHERE effect_id=?1",
            [effect_id],
            |row| {
                Ok(DurableOperationSource {
                    operation_id: row.get(0)?,
                    attempt_id: row.get(1)?,
                    project_id: row.get(2)?,
                    orchestration_id: row.get(3)?,
                    workflow_run_id: row.get(4)?,
                    grant_id: row.get(5)?,
                    dispatch_id: row.get(6)?,
                    effect_id: row.get(7)?,
                    last_receipt_id: row.get(8)?,
                    error: row.get(9)?,
                })
            },
        )
        .optional()
        .map_err(|error| format!("m6_org_temporary_agent_child_run_read:{error}"))?;
    let Some(operation) = operation else {
        return Ok(None);
    };
    let expected_operation_id = format!("op-wc-{}:{}", envelope.attempt_id, envelope.grant_id);
    if operation.operation_id != expected_operation_id
        || operation.attempt_id != envelope.attempt_id
        || operation.project_id != envelope.project_id
        || operation.orchestration_id != envelope.orchestration_id
        || operation.workflow_run_id != envelope.workflow_run_id
        || operation.grant_id != envelope.grant_id
        || operation.dispatch_id != envelope.dispatch_id
        || operation.effect_id != effect_id
        || operation.last_receipt_id.as_deref() != Some(envelope.authoritative_receipt_ref.as_str())
    {
        return Err("m6_org_temporary_agent_child_run_exact_join_mismatch".to_string());
    }
    // The error body remains M5-owned.  Its presence is searchable through a
    // deterministic reference only; M6 never copies the text.
    let _error_ref = operation
        .error
        .as_deref()
        .map(|error| format!("durable-error:sha256:{}", sha_hex(error.as_bytes())));
    Ok(Some(M6OrgChildRunRef {
        child_run_ref: operation.operation_id,
        parent_workcell_ref: format!("wc-{}:{}", envelope.attempt_id, envelope.grant_id),
        attempt_id: envelope.attempt_id.clone(),
        grant_id: envelope.grant_id.clone(),
        trace_ref: format!("trace:sha256:{}", readback.trace_hash),
        reference_only: true,
        creates_stable_member: false,
        creates_temporary_agent: false,
        creates_organization_hierarchy: false,
    }))
}

fn quarantine_for_claim(
    claim: &M5ClaimHeader,
    reason_code: &str,
    now_ms: i64,
) -> M6OrgTemporaryAgentQuarantineRecord {
    let mut source_refs = BTreeSet::from([format!("claim:{}", claim.claim_id)]);
    for (namespace, value) in [
        ("project", Some(claim.project_id.as_str())),
        ("orchestration", Some(claim.orchestration_id.as_str())),
        ("work-item", claim.work_item_id.as_deref()),
        ("attempt", claim.attempt_id.as_deref()),
        ("grant", claim.grant_id.as_deref()),
        ("dispatch", claim.dispatch_id.as_deref()),
        ("receipt", claim.authoritative_receipt_ref.as_deref()),
    ] {
        if let Some(value) = value.filter(|value| !value.trim().is_empty()) {
            source_refs.insert(format!("{namespace}:{value}"));
        }
    }
    M6OrgTemporaryAgentQuarantineRecord {
        quarantine_ref: digest_id("temporary-agent-quarantine", &[&claim.claim_id]),
        source_claim_ref: format!("claim:{}", claim.claim_id),
        reason_code: reason_code.to_string(),
        source_refs: source_refs.into_iter().collect(),
        payload_mode: "REF_ONLY".to_string(),
        recorded_at: now_ms,
    }
}

struct M6OrgTemporaryAgentStore {
    connection: Connection,
}

impl M6OrgTemporaryAgentStore {
    fn open(path: &Path) -> Result<Self, String> {
        let parent = path
            .parent()
            .ok_or_else(|| "m6_org_temporary_agent_store_parent_missing".to_string())?;
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("m6_org_temporary_agent_store_parent_create:{error}"))?;
        let connection = Connection::open(path)
            .map_err(|error| format!("m6_org_temporary_agent_store_open:{error}"))?;
        ensure_m6_org_schema(&connection)?;
        Ok(Self { connection })
    }

    fn record_projection(&mut self, record: &M6OrgTemporaryAgent) -> Result<bool, String> {
        let payload = encode(record, "m6_org_temporary_agent_projection")?;
        let claim_id = record
            .claim_ref
            .strip_prefix("claim:")
            .ok_or_else(|| "m6_org_temporary_agent_claim_ref_invalid".to_string())?;
        let source_fingerprint = stable_hash(&json!({
            "claim_ref": record.claim_ref,
            "execution_envelope": record.execution_envelope,
        }))?;
        let existing = self
            .connection
            .query_row(
                "SELECT temporary_agent_id,source_fingerprint,payload_json
                 FROM m6_temporary_agent_history WHERE claim_id=?1",
                [claim_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .optional()
            .map_err(|error| format!("m6_org_temporary_agent_projection_load:{error}"))?;
        if let Some((existing_id, existing_fingerprint, existing_payload)) = existing {
            if existing_id != record.temporary_agent_id
                || existing_fingerprint != source_fingerprint
                || existing_payload != payload
            {
                return Err("m6_org_temporary_agent_projection_collision".to_string());
            }
            return Ok(false);
        }
        let audit_payload = encode(
            &json!({
                "temporary_agent_id": record.temporary_agent_id,
                "claim_ref": record.claim_ref,
                "source_ref_count": record.source_refs.len(),
                "report_body_copied": false,
                "auto_stabilize": false,
                "project_writeback": false
            }),
            "m6_org_temporary_agent_projection_audit",
        )?;
        let transaction = self
            .connection
            .transaction()
            .map_err(|error| format!("m6_org_temporary_agent_projection_tx:{error}"))?;
        transaction
            .execute(
                "INSERT INTO m6_temporary_agent_history (
                    temporary_agent_id,claim_id,lifecycle_status,source_fingerprint,
                    payload_json,created_at_ms
                 ) VALUES (?1,?2,?3,?4,?5,?6)",
                params![
                    record.temporary_agent_id,
                    claim_id,
                    record.membership_lifecycle.as_str(),
                    source_fingerprint,
                    payload,
                    record.created_at
                ],
            )
            .map_err(|error| format!("m6_org_temporary_agent_projection_insert:{error}"))?;
        insert_audit(
            &transaction,
            &digest_id("audit-temporary-agent", &[&record.temporary_agent_id]),
            "TemporaryAgentProjected",
            &record.temporary_agent_id,
            &audit_payload,
            record.created_at,
        )?;
        transaction
            .commit()
            .map_err(|error| format!("m6_org_temporary_agent_projection_commit:{error}"))?;
        Ok(true)
    }

    fn record_quarantine(
        &mut self,
        quarantine: &M6OrgTemporaryAgentQuarantineRecord,
    ) -> Result<bool, String> {
        let source_refs_json = encode(
            &quarantine.source_refs,
            "m6_org_temporary_agent_quarantine_refs",
        )?;
        let existing = self
            .connection
            .query_row(
                "SELECT quarantine_ref,reason_code,source_refs_json,payload_mode
                 FROM m6_temporary_agent_quarantine WHERE source_claim_ref=?1",
                [&quarantine.source_claim_ref],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                },
            )
            .optional()
            .map_err(|error| format!("m6_org_temporary_agent_quarantine_load:{error}"))?;
        if let Some((quarantine_ref, reason_code, refs, payload_mode)) = existing {
            if quarantine_ref != quarantine.quarantine_ref
                || reason_code != quarantine.reason_code
                || refs != source_refs_json
                || payload_mode != quarantine.payload_mode
            {
                return Err("m6_org_temporary_agent_quarantine_collision".to_string());
            }
            return Ok(false);
        }
        let audit_payload = encode(
            &json!({
                "quarantine_ref": quarantine.quarantine_ref,
                "source_claim_ref": quarantine.source_claim_ref,
                "reason_code": quarantine.reason_code,
                "payload_mode": "REF_ONLY",
                "mapped_to": null
            }),
            "m6_org_temporary_agent_quarantine_audit",
        )?;
        let transaction = self
            .connection
            .transaction()
            .map_err(|error| format!("m6_org_temporary_agent_quarantine_tx:{error}"))?;
        transaction
            .execute(
                "INSERT INTO m6_temporary_agent_quarantine (
                    quarantine_ref,source_claim_ref,reason_code,source_refs_json,
                    payload_mode,recorded_at_ms
                 ) VALUES (?1,?2,?3,?4,?5,?6)",
                params![
                    quarantine.quarantine_ref,
                    quarantine.source_claim_ref,
                    quarantine.reason_code,
                    source_refs_json,
                    quarantine.payload_mode,
                    quarantine.recorded_at
                ],
            )
            .map_err(|error| format!("m6_org_temporary_agent_quarantine_insert:{error}"))?;
        insert_audit(
            &transaction,
            &digest_id(
                "audit-temporary-agent-quarantine",
                &[&quarantine.quarantine_ref],
            ),
            "TemporaryAgentQuarantined",
            &quarantine.source_claim_ref,
            &audit_payload,
            quarantine.recorded_at,
        )?;
        transaction
            .commit()
            .map_err(|error| format!("m6_org_temporary_agent_quarantine_commit:{error}"))?;
        Ok(true)
    }

    fn load_agent(&self, temporary_agent_id: &str) -> Result<Option<M6OrgTemporaryAgent>, String> {
        load_json_optional(
            &self.connection,
            "SELECT payload_json FROM m6_temporary_agent_history WHERE temporary_agent_id=?1",
            temporary_agent_id,
            "m6_org_temporary_agent_load",
        )
    }

    fn list_agents(&self) -> Result<Vec<M6OrgTemporaryAgent>, String> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT payload_json FROM m6_temporary_agent_history
                 ORDER BY created_at_ms,temporary_agent_id",
            )
            .map_err(|error| format!("m6_org_temporary_agent_list_prepare:{error}"))?;
        let rows = statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|error| format!("m6_org_temporary_agent_list_query:{error}"))?;
        decode_rows(rows, "m6_org_temporary_agent_list")
    }

    fn list_quarantines(&self) -> Result<Vec<M6OrgTemporaryAgentQuarantineRecord>, String> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT quarantine_ref,source_claim_ref,reason_code,source_refs_json,
                        payload_mode,recorded_at_ms
                 FROM m6_temporary_agent_quarantine
                 ORDER BY recorded_at_ms,quarantine_ref",
            )
            .map_err(|error| format!("m6_org_temporary_agent_quarantine_list_prepare:{error}"))?;
        let rows = statement
            .query_map([], |row| {
                let refs_json: String = row.get(3)?;
                let source_refs = serde_json::from_str(&refs_json).map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(
                        3,
                        rusqlite::types::Type::Text,
                        Box::new(error),
                    )
                })?;
                Ok(M6OrgTemporaryAgentQuarantineRecord {
                    quarantine_ref: row.get(0)?,
                    source_claim_ref: row.get(1)?,
                    reason_code: row.get(2)?,
                    source_refs,
                    payload_mode: row.get(4)?,
                    recorded_at: row.get(5)?,
                })
            })
            .map_err(|error| format!("m6_org_temporary_agent_quarantine_list_query:{error}"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("m6_org_temporary_agent_quarantine_list_row:{error}"))
    }

    fn promoted_member_id(&self, temporary_agent_id: &str) -> Result<Option<String>, String> {
        self.connection
            .query_row(
                "SELECT member_id FROM m6_temporary_agent_promotion_bindings
                 WHERE temporary_agent_id=?1",
                [temporary_agent_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| format!("m6_org_temporary_agent_promotion_lookup:{error}"))
    }

    fn load_command_response<T: DeserializeOwned>(
        &self,
        idempotency_key: &str,
        operation: &str,
        request_hash: &str,
    ) -> Result<Option<T>, String> {
        let existing = self
            .connection
            .query_row(
                "SELECT operation,request_hash,response_json
                 FROM m6_temporary_agent_command_receipts WHERE idempotency_key=?1",
                [idempotency_key],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .optional()
            .map_err(|error| format!("m6_org_temporary_agent_command_receipt_load:{error}"))?;
        let Some((existing_operation, existing_hash, response_json)) = existing else {
            return Ok(None);
        };
        if existing_operation != operation || existing_hash != request_hash {
            return Err("m6_org_temporary_agent_idempotency_collision".to_string());
        }
        serde_json::from_str(&response_json)
            .map(Some)
            .map_err(|error| format!("m6_org_temporary_agent_command_receipt_decode:{error}"))
    }

    fn record_promotion(
        &mut self,
        idempotency_key: &str,
        request_hash: &str,
        response: &M6OrgTemporaryAgentPromotionResponse,
        now_ms: i64,
    ) -> Result<(), String> {
        let binding_payload = encode(&response.binding, "m6_org_temporary_agent_promotion")?;
        let response_payload = encode(response, "m6_org_temporary_agent_promotion_response")?;
        let audit_payload = encode(
            &json!({
                "binding_id": response.binding.binding_id,
                "member_id": response.binding.member_id,
                "promoted_from": response.binding.promoted_from,
                "promoted_by_actor_id": response.binding.promoted_by_actor_id,
                "explicit_human_command": true,
                "source_temporary_agent_type_unchanged": true
            }),
            "m6_org_temporary_agent_promotion_audit",
        )?;
        let transaction = self
            .connection
            .transaction()
            .map_err(|error| format!("m6_org_temporary_agent_promotion_tx:{error}"))?;
        transaction
            .execute(
                "INSERT INTO m6_temporary_agent_promotion_bindings (
                    binding_id,temporary_agent_id,member_id,promoted_by_actor_id,
                    explicit_human_command,source_temporary_agent_type_unchanged,
                    idempotency_key,request_hash,payload_json,created_at_ms
                 ) VALUES (?1,?2,?3,?4,1,1,?5,?6,?7,?8)",
                params![
                    response.binding.binding_id,
                    response.binding.promoted_from,
                    response.binding.member_id,
                    response.binding.promoted_by_actor_id,
                    idempotency_key,
                    request_hash,
                    binding_payload,
                    now_ms
                ],
            )
            .map_err(|error| format!("m6_org_temporary_agent_promotion_insert:{error}"))?;
        insert_audit(
            &transaction,
            &digest_id(
                "audit-temporary-agent-promotion",
                &[&response.binding.binding_id],
            ),
            "TemporaryAgentPromoted",
            &response.binding.promoted_from,
            &audit_payload,
            now_ms,
        )?;
        transaction
            .execute(
                "INSERT INTO m6_temporary_agent_command_receipts (
                    idempotency_key,operation,request_hash,response_json,recorded_at_ms
                 ) VALUES (?1,'promote_temporary_agent',?2,?3,?4)",
                params![idempotency_key, request_hash, response_payload, now_ms],
            )
            .map_err(|error| format!("m6_org_temporary_agent_command_receipt_insert:{error}"))?;
        transaction
            .commit()
            .map_err(|error| format!("m6_org_temporary_agent_promotion_commit:{error}"))
    }
}

fn load_claim_headers(connection: &Connection) -> Result<Vec<M5ClaimHeader>, String> {
    let mut statement = connection
        .prepare(
            "SELECT claim_id,report_kind,claim_status,project_id,orchestration_id,
                    workflow_run_id,work_item_id,node_id,dispatch_id,attempt_id,grant_id,
                    worker_role_session_id,authoritative_receipt_ref,report_hash,
                    authenticated_actor_id,created_at_ms
             FROM m5_claims ORDER BY created_at_ms,claim_id",
        )
        .map_err(|error| format!("m6_org_temporary_agent_claim_prepare:{error}"))?;
    let rows = statement
        .query_map([], |row| {
            Ok(M5ClaimHeader {
                claim_id: row.get(0)?,
                report_kind: row.get(1)?,
                claim_status: row.get(2)?,
                project_id: row.get(3)?,
                orchestration_id: row.get(4)?,
                workflow_run_id: row.get(5)?,
                work_item_id: row.get(6)?,
                node_id: row.get(7)?,
                dispatch_id: row.get(8)?,
                attempt_id: row.get(9)?,
                grant_id: row.get(10)?,
                worker_role_session_id: row.get(11)?,
                authoritative_receipt_ref: row.get(12)?,
                report_hash: row.get(13)?,
                authenticated_actor_id: row.get(14)?,
                created_at_ms: row.get(15)?,
            })
        })
        .map_err(|error| format!("m6_org_temporary_agent_claim_query:{error}"))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("m6_org_temporary_agent_claim_row:{error}"))
}

fn load_work_item(connection: &Connection, id: &str) -> Result<WorkItemSource, String> {
    connection
        .query_row(
            "SELECT project_id,orchestration_id,workflow_run_id,source_object_ref,node_id,status
             FROM m5_work_items WHERE work_item_id=?1",
            [id],
            |row| {
                Ok(WorkItemSource {
                    project_id: row.get(0)?,
                    orchestration_id: row.get(1)?,
                    workflow_run_id: row.get(2)?,
                    source_object_ref: row.get(3)?,
                    node_id: row.get(4)?,
                    status: row.get(5)?,
                })
            },
        )
        .optional()
        .map_err(|error| format!("m6_org_temporary_agent_work_item_read:{error}"))?
        .ok_or_else(|| "m6_org_temporary_agent_work_item_missing".to_string())
}

fn load_attempt(connection: &Connection, id: &str) -> Result<AttemptSource, String> {
    connection
        .query_row(
            "SELECT state,project_id,orchestration_id,workflow_run_id,work_item_id,node_id,
                    worker_role_session_id,grant_id,revision
             FROM m5_prepared_attempts WHERE attempt_id=?1",
            [id],
            |row| {
                Ok(AttemptSource {
                    state: row.get(0)?,
                    project_id: row.get(1)?,
                    orchestration_id: row.get(2)?,
                    workflow_run_id: row.get(3)?,
                    work_item_id: row.get(4)?,
                    node_id: row.get(5)?,
                    worker_role_session_id: row.get(6)?,
                    grant_id: row.get(7)?,
                    revision: row.get(8)?,
                })
            },
        )
        .optional()
        .map_err(|error| format!("m6_org_temporary_agent_attempt_read:{error}"))?
        .ok_or_else(|| "m6_org_temporary_agent_attempt_missing".to_string())
}

fn load_grant(connection: &Connection, id: &str) -> Result<GrantSource, String> {
    connection
        .query_row(
            "SELECT project_id,orchestration_id,workflow_run_id,work_item_id,attempt_id,
                    principal_actor_id,worker_role_session_id,status,revoked_at_ms,revision,
                    effect_key,grant_hash
             FROM m5_execution_grants WHERE grant_id=?1",
            [id],
            |row| {
                Ok(GrantSource {
                    project_id: row.get(0)?,
                    orchestration_id: row.get(1)?,
                    workflow_run_id: row.get(2)?,
                    work_item_id: row.get(3)?,
                    attempt_id: row.get(4)?,
                    principal_actor_id: row.get(5)?,
                    worker_role_session_id: row.get(6)?,
                    status: row.get(7)?,
                    revoked_at_ms: row.get(8)?,
                    revision: row.get(9)?,
                    effect_key: row.get(10)?,
                    grant_hash: row.get(11)?,
                })
            },
        )
        .optional()
        .map_err(|error| format!("m6_org_temporary_agent_grant_read:{error}"))?
        .ok_or_else(|| "m6_org_temporary_agent_grant_missing".to_string())
}

fn load_dispatch(connection: &Connection, id: &str) -> Result<DispatchSource, String> {
    connection
        .query_row(
            "SELECT project_id,orchestration_id,workflow_run_id,work_item_id,node_id,
                    attempt_id,grant_id,grant_revision,worker_role_session_id,effect_id,state
             FROM m5_dispatches WHERE dispatch_id=?1",
            [id],
            |row| {
                Ok(DispatchSource {
                    project_id: row.get(0)?,
                    orchestration_id: row.get(1)?,
                    workflow_run_id: row.get(2)?,
                    work_item_id: row.get(3)?,
                    node_id: row.get(4)?,
                    attempt_id: row.get(5)?,
                    grant_id: row.get(6)?,
                    grant_revision: row.get(7)?,
                    worker_role_session_id: row.get(8)?,
                    effect_id: row.get(9)?,
                    state: row.get(10)?,
                })
            },
        )
        .optional()
        .map_err(|error| format!("m6_org_temporary_agent_dispatch_read:{error}"))?
        .ok_or_else(|| "m6_org_temporary_agent_dispatch_missing".to_string())
}

fn load_binding(connection: &Connection, attempt_id: &str) -> Result<BindingSource, String> {
    connection
        .query_row(
            "SELECT project_id,orchestration_id,workflow_run_id,work_item_id,attempt_id,
                    worker_role_session_id,principal_actor_id
             FROM m5_worker_role_session_bindings WHERE attempt_id=?1",
            [attempt_id],
            |row| {
                Ok(BindingSource {
                    project_id: row.get(0)?,
                    orchestration_id: row.get(1)?,
                    workflow_run_id: row.get(2)?,
                    work_item_id: row.get(3)?,
                    attempt_id: row.get(4)?,
                    worker_role_session_id: row.get(5)?,
                    principal_actor_id: row.get(6)?,
                })
            },
        )
        .optional()
        .map_err(|error| format!("m6_org_temporary_agent_binding_read:{error}"))?
        .ok_or_else(|| "m6_org_temporary_agent_binding_missing".to_string())
}

fn load_readback(connection: &Connection, receipt_id: &str) -> Result<ReadbackSource, String> {
    connection
        .query_row(
            "SELECT grant_id,attempt_id,dispatch_id,effect_id,trace_hash,actor_binding,
                    enforcement_status,outcome,derived_attempt_state,source_attempt_revision,
                    committed_attempt_revision,canonical_readback_hash,recording_command_receipt_ref
             FROM m5_execution_attempt_readbacks WHERE receipt_id=?1",
            [receipt_id],
            |row| {
                Ok(ReadbackSource {
                    grant_id: row.get(0)?,
                    attempt_id: row.get(1)?,
                    dispatch_id: row.get(2)?,
                    effect_id: row.get(3)?,
                    trace_hash: row.get(4)?,
                    actor_binding: row.get(5)?,
                    enforcement_status: row.get(6)?,
                    outcome: row.get(7)?,
                    derived_attempt_state: row.get(8)?,
                    source_attempt_revision: row.get(9)?,
                    committed_attempt_revision: row.get(10)?,
                    canonical_readback_hash: row.get(11)?,
                    recording_command_receipt_ref: row.get(12)?,
                })
            },
        )
        .optional()
        .map_err(|error| format!("m6_org_temporary_agent_readback_read:{error}"))?
        .ok_or_else(|| "m6_org_temporary_agent_readback_missing".to_string())
}

fn load_receipt_carrier(
    connection: &Connection,
    receipt_id: &str,
) -> Result<ReceiptCarrier, String> {
    connection
        .query_row(
            "SELECT command_id,actor_id,scope_ref,current_object_ref,status,result_ref,
                    result_hash,committed_revision
             FROM m5_command_receipts WHERE receipt_id=?1",
            [receipt_id],
            |row| {
                Ok(ReceiptCarrier {
                    command_id: row.get(0)?,
                    actor_id: row.get(1)?,
                    scope_ref: row.get(2)?,
                    current_object_ref: row.get(3)?,
                    status: row.get(4)?,
                    result_ref: row.get(5)?,
                    result_hash: row.get(6)?,
                    committed_revision: row.get(7)?,
                })
            },
        )
        .optional()
        .map_err(|error| format!("m6_org_temporary_agent_receipt_carrier_read:{error}"))?
        .ok_or_else(|| "m6_org_temporary_agent_receipt_carrier_missing".to_string())
}

fn load_event_carrier(connection: &Connection, receipt_id: &str) -> Result<EventCarrier, String> {
    connection
        .query_row(
            "SELECT event_type,actor_id,scope_ref,source_ref,source_revision,command_id,payload_hash
             FROM m5_events WHERE event_id=?1",
            [format!("evt-execution-attempt-readback-{receipt_id}")],
            |row| {
                Ok(EventCarrier {
                    event_type: row.get(0)?,
                    actor_id: row.get(1)?,
                    scope_ref: row.get(2)?,
                    source_ref: row.get(3)?,
                    source_revision: row.get(4)?,
                    command_id: row.get(5)?,
                    payload_hash: row.get(6)?,
                })
            },
        )
        .optional()
        .map_err(|error| format!("m6_org_temporary_agent_event_carrier_read:{error}"))?
        .ok_or_else(|| "m6_org_temporary_agent_event_carrier_missing".to_string())
}

fn load_audit_carrier(connection: &Connection, receipt_id: &str) -> Result<AuditCarrier, String> {
    connection
        .query_row(
            "SELECT action,decision,actor_id,scope_ref,subject_ref,command_id,source_refs
             FROM m5_audit_records WHERE audit_id=?1",
            [format!("aud-execution-attempt-readback-{receipt_id}")],
            |row| {
                Ok(AuditCarrier {
                    action: row.get(0)?,
                    decision: row.get(1)?,
                    actor_id: row.get(2)?,
                    scope_ref: row.get(3)?,
                    subject_ref: row.get(4)?,
                    command_id: row.get(5)?,
                    source_refs: row.get(6)?,
                })
            },
        )
        .optional()
        .map_err(|error| format!("m6_org_temporary_agent_audit_carrier_read:{error}"))?
        .ok_or_else(|| "m6_org_temporary_agent_audit_carrier_missing".to_string())
}

fn require_temporary_agent_runtime(state: &crate::AppState) -> Result<(), String> {
    let _ = state.m6_org_global_role_session.authority_seed()?;
    Ok(())
}

fn table_exists(connection: &Connection, table: &str) -> Result<bool, String> {
    connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1)",
            [table],
            |row| row.get::<_, bool>(0),
        )
        .map_err(|error| format!("m6_org_temporary_agent_table_probe:{table}:{error}"))
}

fn required(value: &Option<String>) -> Result<&str, String> {
    value
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "m6_org_temporary_agent_incomplete_execution_envelope".to_string())
}

fn validate_hash(value: &str) -> Result<(), String> {
    let value = value.strip_prefix("sha256:").unwrap_or(value);
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("m6_org_temporary_agent_hash_invalid".to_string());
    }
    Ok(())
}

fn canonical_readback_hash(receipt_id: &str, readback: &ReadbackSource) -> String {
    sha_hex(
        format!(
            "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
            receipt_id,
            readback.grant_id,
            readback.attempt_id,
            readback.dispatch_id,
            readback.effect_id,
            readback.trace_hash,
            readback.actor_binding,
            readback.enforcement_status,
            readback.outcome,
            readback.derived_attempt_state,
            readback.source_attempt_revision,
            readback.committed_attempt_revision
        )
        .as_bytes(),
    )
}

fn validate_ref(field: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() || value.len() > 512 || value.chars().any(char::is_control) {
        return Err(format!("m6_org_temporary_agent_{field}_invalid"));
    }
    Ok(())
}

fn searchable_text(record: &M6OrgTemporaryAgent) -> String {
    let mut values = vec![
        record.temporary_agent_id.as_str(),
        record.claim_ref.as_str(),
        record.task_ref.as_str(),
        record.task_state.as_str(),
        record.result_ref.as_str(),
        record.result_state.as_str(),
        record.display_name_ref.as_str(),
        record.execution_envelope.project_id.as_str(),
        record.execution_envelope.orchestration_id.as_str(),
        record.execution_envelope.workflow_run_id.as_str(),
        record.execution_envelope.work_item_id.as_str(),
        record.execution_envelope.node_id.as_str(),
        record.execution_envelope.dispatch_id.as_str(),
        record.execution_envelope.attempt_id.as_str(),
        record.execution_envelope.grant_id.as_str(),
        record.execution_envelope.worker_role_session_id.as_str(),
        record.execution_envelope.authoritative_receipt_ref.as_str(),
        record.execution_envelope.trusted_actor_id.as_str(),
    ];
    if let Some(failure_ref) = &record.failure_ref {
        values.push(failure_ref);
    }
    for source_ref in &record.source_refs {
        values.push(source_ref);
    }
    values.join("\n").to_ascii_lowercase()
}

fn insert_audit(
    transaction: &Transaction<'_>,
    event_id: &str,
    event_type: &str,
    target_ref: &str,
    payload_json: &str,
    created_at_ms: i64,
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
                payload_json,
                created_at_ms
            ],
        )
        .map(|_| ())
        .map_err(|error| format!("m6_org_temporary_agent_audit_insert:{error}"))
}

fn load_json_optional<T: DeserializeOwned>(
    connection: &Connection,
    sql: &str,
    key: &str,
    prefix: &str,
) -> Result<Option<T>, String> {
    let payload = connection
        .query_row(sql, [key], |row| row.get::<_, String>(0))
        .optional()
        .map_err(|error| format!("{prefix}:{error}"))?;
    payload
        .map(|payload| {
            serde_json::from_str(&payload).map_err(|error| format!("{prefix}_decode:{error}"))
        })
        .transpose()
}

fn decode_rows<T: DeserializeOwned>(
    rows: rusqlite::MappedRows<'_, impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<String>>,
    prefix: &str,
) -> Result<Vec<T>, String> {
    let mut values = Vec::new();
    for row in rows {
        let payload = row.map_err(|error| format!("{prefix}_row:{error}"))?;
        values.push(
            serde_json::from_str(&payload).map_err(|error| format!("{prefix}_decode:{error}"))?,
        );
    }
    Ok(values)
}

fn stable_hash(value: &impl Serialize) -> Result<String, String> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| format!("m6_org_temporary_agent_hash_serialize:{error}"))?;
    Ok(sha_hex(&bytes))
}

fn sha_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn digest_id(namespace: &str, parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"syn.m6.org.temporary-agent/v1\0");
    hasher.update(namespace.as_bytes());
    for part in parts {
        hasher.update(b"\0");
        hasher.update(part.as_bytes());
    }
    format!("{namespace}:sha256:{:x}", hasher.finalize())
}

fn encode(value: &impl Serialize, prefix: &str) -> Result<String, String> {
    serde_json::to_string(value).map_err(|error| format!("{prefix}_serialize:{error}"))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::m5_agent_runtime::{ObservingFakeRuntime, RuntimeFault, WorkcellRun};
    use crate::m5_claim_ledger::{ensure_claim_schema, record_claim};
    use crate::m5_controlled_execution::run_authorized_workcell;
    use crate::m5_orchestration_service::{
        complete_dispatch_readback, prepare_and_dispatch, record_execution_attempt_readback,
        AuthorizedExecutionRequest, ChainFault, DispatchReadbackSource,
    };
    use crate::m5_runtime_receipt::EnforcementStatus;
    use crate::m6_org_member_directory::{
        M6OrgRegisterStableMemberRequest, M6OrgStableIdentityEvidence,
    };
    use crate::worker_report::{ExecutionReceipt, M5WorkerReport, TrustedActor, WorkerReport};
    use crate::AppState;
    use rusqlite::Connection;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    const NOW_MS: i64 = 1_787_097_600_000;
    static SCRATCH_SEQUENCE: AtomicU64 = AtomicU64::new(1);

    pub(crate) struct Fixture {
        pub(crate) root: PathBuf,
        pub(crate) state: AppState,
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    #[derive(Clone, Debug)]
    pub(crate) struct Seed {
        claim_id: String,
        project_id: String,
        work_item_id: String,
        attempt_id: String,
        grant_id: String,
        dispatch_id: String,
        receipt_id: String,
        worker_role_session_id: String,
        report_body_sentinel: String,
    }

    pub(crate) fn fixture(label: &str) -> Fixture {
        let sequence = SCRATCH_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "syn-m6d06-{label}-{}-{sequence}",
            std::process::id()
        ));
        let app_data_root = root.join(crate::m1_project_index::M1_ORDINARY_APP_DATA_DIR_NAME);
        std::fs::create_dir_all(&app_data_root).expect("create M6D06 app-data root");
        let app_data_root =
            std::fs::canonicalize(&app_data_root).expect("canonical M6D06 app-data root");
        let seed_dir = root.join("synthetic-ordinary-product-seeds");
        std::fs::create_dir_all(&seed_dir).expect("create M6D06 seeds");
        let index_seed = seed_dir.join("codex-index.json");
        let tasks_seed = seed_dir.join("README.md");
        std::fs::write(&index_seed, r#"{"projects":[]}"#).expect("write M6D06 index seed");
        std::fs::write(&tasks_seed, "# synthetic M6D06 tasks\n").expect("write M6D06 tasks seed");
        let state = AppState::try_new_with_tauri_ordinary_product_seeds(
            &app_data_root,
            &index_seed,
            &tasks_seed,
        )
        .expect("ordinary M6D06 AppState");
        Fixture { root, state }
    }

    pub(crate) fn seed_execution(
        fixture: &Fixture,
        label: &str,
        outcome: &str,
        worker_role_session_id: &str,
    ) -> Seed {
        let store = fixture.state.open_m5_store().expect("open M5 store");
        let project_id = format!("project:{label}");
        let principal_actor_id = format!("actor:{label}");
        let chain = prepare_and_dispatch(
            &store,
            AuthorizedExecutionRequest {
                project_id: project_id.clone(),
                proposal_id: format!("proposal:{label}"),
                deciding_actor_id: "actor:user".to_string(),
                worker_role_session_id: worker_role_session_id.to_string(),
                principal_actor_id: principal_actor_id.clone(),
                workflow_ref: format!("workflow:{label}"),
                source_object_ref: format!("task:{label}"),
                allowed_commands: vec!["echo".to_string()],
                cwd_ref: "/tmp/syn-m6d06-fixture".to_string(),
                write_root_refs: vec!["/tmp/syn-m6d06-fixture".to_string()],
                object_refs: vec![format!("task:{label}")],
                scope_fingerprint: format!("scope:{label}"),
                policy_decision_ref: format!("policy:{label}"),
                now_ms: NOW_MS,
                ttl_ms: 120_000,
            },
            ChainFault::None,
        )
        .expect("prepare M5 execution");
        let grant_id = chain.grant_id.expect("grant id").as_str().to_string();
        let dispatch_id = chain.dispatch_id.expect("dispatch id").as_str().to_string();
        let (dispatch, dispatched_attempt) = complete_dispatch_readback(
            &store,
            DispatchReadbackSource::ExactStoredDispatch(&dispatch_id),
            NOW_MS + 100,
        )
        .expect("complete dispatch readback");
        let grant = store
            .load_grant(&grant_id)
            .expect("load grant")
            .expect("persisted grant");
        let mut runtime = ObservingFakeRuntime::new();
        runtime.observe(&dispatch.effect_id, EnforcementStatus::Ok, outcome);
        let workcell = WorkcellRun {
            workcell_id: format!("fixture-child-workcell:{label}"),
            profile_digest: "profile:observing-fake:v1".to_string(),
            session_ref: format!("child-session-looking-name:{label}"),
            parent_grant_id: grant_id.clone(),
            attempt_id: chain.attempt_id.as_str().to_string(),
            dispatch_id: dispatch_id.clone(),
            effect_id: dispatch.effect_id.clone(),
            actor_binding: worker_role_session_id.to_string(),
            command: "echo".to_string(),
            child_depth: 1,
            budget_tokens: 8,
            stop_conditions: vec!["fixture-complete".to_string()],
            dynamic_package_enabled: false,
        };
        let receipt = run_authorized_workcell(
            &store,
            &mut runtime,
            &workcell,
            NOW_MS + 200,
            RuntimeFault::None,
        )
        .expect("run observing fake workcell");
        let (_, readback) = record_execution_attempt_readback(
            &store,
            receipt.clone(),
            dispatched_attempt.revision,
            NOW_MS + 300,
        )
        .expect("record M5 execution readback");
        let report_body_sentinel = format!("REPORT-BODY-MUST-NOT-COPY-{label}");
        let report_hash = sha_hex(format!("report:{label}").as_bytes());
        let report = M5WorkerReport::from_base(WorkerReport {
            did: report_body_sentinel.clone(),
            status: outcome.to_ascii_lowercase(),
            findings: vec![format!("private report finding for {label}")],
            ..WorkerReport::default()
        })
        .as_execution(
            ExecutionReceipt {
                execution_id: receipt.receipt_id.as_str().to_string(),
                started_at_ms: NOW_MS + 200,
                completed_at_ms: Some(NOW_MS + 300),
                status: readback.derived_attempt_state.clone(),
                exit_code: if outcome == "SUCCEEDED" {
                    Some(0)
                } else {
                    Some(124)
                },
                output_hash: Some(receipt.trace_hash.clone()),
                cost_tokens: None,
            },
            TrustedActor {
                actor_id: principal_actor_id,
                role: "worker".to_string(),
                actor_type: "observing-fake".to_string(),
                authentication_method: "m5-role-session-binding".to_string(),
            },
        )
        .bind_project(&project_id, grant.orchestration_id.as_str())
        .bind_execution_join(
            grant.workflow_run_id.as_str(),
            grant.work_item_id.as_str(),
            &dispatch.node_id,
            &dispatch_id,
            chain.attempt_id.as_str(),
            &grant_id,
            worker_role_session_id,
            receipt.receipt_id.as_str(),
            &report_hash,
        );
        let claim = record_claim(&store, &report, Some(&receipt), NOW_MS + 400)
            .expect("record exact M5 claim");
        Seed {
            claim_id: claim.claim_id,
            project_id,
            work_item_id: grant.work_item_id.as_str().to_string(),
            attempt_id: chain.attempt_id.as_str().to_string(),
            grant_id,
            dispatch_id,
            receipt_id: receipt.receipt_id.as_str().to_string(),
            worker_role_session_id: worker_role_session_id.to_string(),
            report_body_sentinel,
        }
    }

    fn explicit_registration(
        member_id: &str,
        idempotency_key: &str,
    ) -> M6OrgRegisterStableMemberRequest {
        M6OrgRegisterStableMemberRequest {
            member_id: member_id.to_string(),
            display_name_ref: format!("display-name:{member_id}"),
            identity_evidence: M6OrgStableIdentityEvidence::ExplicitIdentityContract {
                contract_kind: "syn.m6.org.stable-member-identity/v1".to_string(),
                identity_contract_ref: format!("identity-contract:{member_id}"),
                source_record_ref: format!("identity-source:{member_id}"),
                source_revision: 1,
                observed_at: NOW_MS,
                explicit_human_command: true,
            },
            scope_assignments: Vec::new(),
            role_assignments: Vec::new(),
            capability_permission_refs: Vec::new(),
            memory_refs: Vec::new(),
            contact_bindings: Vec::new(),
            idempotency_key: idempotency_key.to_string(),
        }
    }

    fn file_hash(path: &Path) -> String {
        sha_hex(&std::fs::read(path).expect("read file for hash"))
    }

    #[test]
    fn m6d06_full_envelope_is_read_only_convergent_and_searchable_without_report_body() {
        let fixture = fixture("full-envelope");
        let seed = seed_execution(&fixture, "alpha", "SUCCEEDED", "role-session:worker-alpha");
        let m5_path = fixture.state.m5_store_path().expect("M5 path");
        let before = file_hash(m5_path);
        let first = refresh_for_state(&fixture.state, NOW_MS + 500).expect("first refresh");
        assert_eq!(first.projected_count, 1);
        assert_eq!(first.retained_count, 0);
        assert_eq!(first.quarantined_count, 0);
        assert!(first.m5_opened_read_only);
        assert!(!first.report_bodies_copied);
        assert_eq!(first.automatic_promotions, 0);
        assert_eq!(before, file_hash(m5_path), "refresh must not write M5");
        let record = &first.records[0];
        assert_eq!(record.execution_envelope.project_id, seed.project_id);
        assert_eq!(record.execution_envelope.work_item_id, seed.work_item_id);
        assert_eq!(record.execution_envelope.attempt_id, seed.attempt_id);
        assert_eq!(record.execution_envelope.grant_id, seed.grant_id);
        assert_eq!(record.execution_envelope.dispatch_id, seed.dispatch_id);
        assert_eq!(
            record.execution_envelope.authoritative_receipt_ref,
            seed.receipt_id
        );
        assert_eq!(
            record.execution_envelope.worker_role_session_id,
            seed.worker_role_session_id
        );
        let child = record.child_run_ref.as_ref().expect("exact child run ref");
        assert!(child.reference_only);
        assert!(!child.creates_stable_member);
        assert!(!child.creates_temporary_agent);
        assert!(!child.creates_organization_hierarchy);
        assert!(!encode(record, "test-record")
            .unwrap()
            .contains(&seed.report_body_sentinel));

        let second = refresh_for_state(&fixture.state, NOW_MS + 600).expect("replay refresh");
        assert_eq!(second.projected_count, 0);
        assert_eq!(second.retained_count, 1);
        assert_eq!(second.records.len(), 1);
        for query in ["task:alpha", "succeeded", "role-session:worker-alpha"] {
            let response = search_for_state(
                &fixture.state,
                &M6OrgSearchTemporaryAgentHistoryRequest {
                    query: query.to_string(),
                    limit: 10,
                },
            )
            .expect("search temporary history");
            assert_eq!(response.matches.len(), 1, "query {query}");
            assert!(response.matches[0].presented_as_temporary);
            assert!(!response.report_bodies_present);
        }
        let m6 = Connection::open(fixture.state.m6_org_store_path().unwrap()).unwrap();
        let stable_members: i64 = m6
            .query_row(
                "SELECT COUNT(*) FROM m6_stable_member_identities",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(stable_members, 0, "repeat use must not auto-promote");
    }

    #[test]
    fn m6d06_every_required_envelope_field_and_report_hash_fail_closed() {
        let mutations = [
            ("project_id", "''"),
            ("orchestration_id", "''"),
            ("workflow_run_id", "NULL"),
            ("work_item_id", "NULL"),
            ("node_id", "NULL"),
            ("dispatch_id", "NULL"),
            ("attempt_id", "NULL"),
            ("grant_id", "NULL"),
            ("worker_role_session_id", "NULL"),
            ("authoritative_receipt_ref", "NULL"),
            ("authenticated_actor_id", "NULL"),
            ("report_hash", "''"),
        ];
        for (field, replacement) in mutations {
            let fixture = fixture(&format!("missing-{field}"));
            seed_execution(
                &fixture,
                &format!("missing-{field}"),
                "SUCCEEDED",
                &format!("role-session:missing-{field}"),
            );
            let connection = Connection::open(fixture.state.m5_store_path().unwrap()).unwrap();
            connection
                .execute(&format!("UPDATE m5_claims SET {field}={replacement}"), [])
                .unwrap();
            drop(connection);
            let before = file_hash(fixture.state.m5_store_path().unwrap());
            let response = refresh_for_state(&fixture.state, NOW_MS + 700)
                .unwrap_or_else(|error| panic!("refresh {field}: {error}"));
            assert!(response.records.is_empty(), "field {field}");
            assert_eq!(response.quarantines.len(), 1, "field {field}");
            assert_eq!(response.quarantines[0].payload_mode, "REF_ONLY");
            assert_eq!(before, file_hash(fixture.state.m5_store_path().unwrap()));
        }
    }

    #[test]
    fn m6d06_report_actor_self_claim_and_join_tamper_are_quarantined() {
        let fixture = fixture("self-claim");
        seed_execution(
            &fixture,
            "self-claim",
            "SUCCEEDED",
            "role-session:self-claim",
        );
        let connection = Connection::open(fixture.state.m5_store_path().unwrap()).unwrap();
        connection
            .execute(
                "UPDATE m5_claims SET authenticated_actor_id='actor:report-self-claim'",
                [],
            )
            .unwrap();
        drop(connection);
        let response = refresh_for_state(&fixture.state, NOW_MS + 800).unwrap();
        assert!(response.records.is_empty());
        assert_eq!(response.quarantines.len(), 1);
        assert_eq!(
            response.quarantines[0].reason_code,
            "m6_org_temporary_agent_exact_join_mismatch"
        );
    }

    #[test]
    fn m6d06_child_run_requires_exact_operation_and_never_uses_session_name_heuristics() {
        let exact = fixture("child-exact-mismatch");
        seed_execution(
            &exact,
            "child-exact-mismatch",
            "SUCCEEDED",
            "role-session:child-looking-name",
        );
        let connection = Connection::open(exact.state.m5_store_path().unwrap()).unwrap();
        connection
            .execute(
                "UPDATE m5_durable_operations SET last_receipt_id='receipt:wrong'",
                [],
            )
            .unwrap();
        drop(connection);
        let rejected = refresh_for_state(&exact.state, NOW_MS + 900).unwrap();
        assert!(rejected.records.is_empty());
        assert_eq!(
            rejected.quarantines[0].reason_code,
            "m6_org_temporary_agent_child_run_exact_join_mismatch"
        );

        let heuristic = fixture("child-name-only");
        seed_execution(
            &heuristic,
            "child-name-only",
            "SUCCEEDED",
            "role-session:parent-child-session-looking-name",
        );
        let connection = Connection::open(heuristic.state.m5_store_path().unwrap()).unwrap();
        connection
            .execute("DELETE FROM m5_durable_operations", [])
            .unwrap();
        drop(connection);
        let accepted = refresh_for_state(&heuristic.state, NOW_MS + 900).unwrap();
        assert_eq!(accepted.records.len(), 1);
        assert!(accepted.records[0].child_run_ref.is_none());
    }

    #[test]
    fn m6d06_human_promotion_create_and_bind_preserve_temporary_source() {
        let fixture = fixture("promotion");
        seed_execution(
            &fixture,
            "promotion-create",
            "SUCCEEDED",
            "role-session:promotion-create",
        );
        seed_execution(
            &fixture,
            "promotion-bind",
            "SUCCEEDED",
            "role-session:promotion-bind",
        );
        let refresh = refresh_for_state(&fixture.state, NOW_MS + 1_000).unwrap();
        assert_eq!(refresh.records.len(), 2);
        let create_source = refresh.records[0].clone();
        let bind_source = refresh.records[1].clone();

        let rejected = promote_for_state(
            &fixture.state,
            &M6OrgPromoteTemporaryAgentRequest {
                temporary_agent_id: create_source.temporary_agent_id.clone(),
                promoted_by_actor_id: "actor:user".to_string(),
                explicit_human_command: false,
                target: M6OrgTemporaryAgentPromotionTarget::CreateStableMember {
                    registration: explicit_registration(
                        "member_rejected_promotion",
                        "register-rejected-promotion",
                    ),
                },
                idempotency_key: "promotion-rejected".to_string(),
            },
            NOW_MS + 1_100,
        );
        assert_eq!(
            rejected.unwrap_err(),
            "m6_org_temporary_agent_promotion_requires_human_command"
        );

        let create_request = M6OrgPromoteTemporaryAgentRequest {
            temporary_agent_id: create_source.temporary_agent_id.clone(),
            promoted_by_actor_id: "actor:user".to_string(),
            explicit_human_command: true,
            target: M6OrgTemporaryAgentPromotionTarget::CreateStableMember {
                registration: explicit_registration(
                    "member_promoted_create",
                    "register-promoted-create",
                ),
            },
            idempotency_key: "promotion-create".to_string(),
        };
        let created = promote_for_state(&fixture.state, &create_request, NOW_MS + 1_200).unwrap();
        assert_eq!(
            created.stable_member.promoted_from.as_deref(),
            Some(create_source.temporary_agent_id.as_str())
        );
        assert_eq!(created.source_temporary_agent, create_source);
        assert!(created.binding.source_temporary_agent_type_unchanged);
        let replay = promote_for_state(&fixture.state, &create_request, NOW_MS + 1_300).unwrap();
        assert!(replay.replayed);
        assert_eq!(replay.binding, created.binding);

        let directly_registered = crate::m6_org_member_directory::register_for_state(
            &fixture.state,
            &explicit_registration("member_promoted_bind", "register-promoted-bind"),
            NOW_MS + 1_400,
        )
        .unwrap()
        .member
        .unwrap();
        assert!(directly_registered.promoted_from.is_none());
        let bound = promote_for_state(
            &fixture.state,
            &M6OrgPromoteTemporaryAgentRequest {
                temporary_agent_id: bind_source.temporary_agent_id.clone(),
                promoted_by_actor_id: "actor:user".to_string(),
                explicit_human_command: true,
                target: M6OrgTemporaryAgentPromotionTarget::BindExistingStableMember {
                    member_id: directly_registered.member_id,
                    expected_revision: directly_registered.revision,
                },
                idempotency_key: "promotion-bind".to_string(),
            },
            NOW_MS + 1_500,
        )
        .unwrap();
        assert_eq!(bound.stable_member.revision, 2);
        assert_eq!(
            bound.stable_member.promoted_from.as_deref(),
            Some(bind_source.temporary_agent_id.as_str())
        );
        assert_eq!(bound.source_temporary_agent, bind_source);

        let search = search_for_state(
            &fixture.state,
            &M6OrgSearchTemporaryAgentHistoryRequest {
                query: create_source.temporary_agent_id,
                limit: 10,
            },
        )
        .unwrap();
        assert!(search.matches[0].presented_as_temporary);
        assert_eq!(
            search.matches[0].promoted_member_id.as_deref(),
            Some("member_promoted_create")
        );
    }

    #[test]
    fn m6d06_failure_and_source_are_searchable_by_refs_only() {
        let fixture = fixture("failure-search");
        let seed = seed_execution(
            &fixture,
            "failure-search",
            "TIMED_OUT",
            "role-session:failure-search",
        );
        let response = refresh_for_state(&fixture.state, NOW_MS + 1_600).unwrap();
        assert_eq!(response.records.len(), 1);
        assert!(response.records[0].failure_ref.is_some());
        for query in [
            "timed_out",
            seed.attempt_id.as_str(),
            seed.receipt_id.as_str(),
        ] {
            let result = search_for_state(
                &fixture.state,
                &M6OrgSearchTemporaryAgentHistoryRequest {
                    query: query.to_string(),
                    limit: 10,
                },
            )
            .unwrap();
            assert_eq!(result.matches.len(), 1, "query {query}");
        }
        let serialized = encode(&response.records[0], "failure-record").unwrap();
        assert!(!serialized.contains(&seed.report_body_sentinel));
        assert!(!serialized.contains("private report finding"));
    }

    #[test]
    fn m6d06_unmappable_legacy_is_quarantined_and_manual_is_ignored() {
        let fixture = fixture("legacy-quarantine");
        seed_execution(
            &fixture,
            "legacy-valid",
            "SUCCEEDED",
            "role-session:legacy-valid",
        );
        let store = fixture.state.open_m5_store().unwrap();
        ensure_claim_schema(&store).unwrap();
        let manual = M5WorkerReport::from_base(WorkerReport {
            did: "manual body".to_string(),
            ..WorkerReport::default()
        })
        .as_manual()
        .bind_project("project:manual", "orchestration:manual");
        record_claim(&store, &manual, None, NOW_MS + 1_700).unwrap();
        store
            .connection()
            .execute(
                "INSERT INTO m5_claims (
                    claim_id,report_kind,claim_status,project_id,orchestration_id,
                    workflow_run_id,work_item_id,node_id,dispatch_id,attempt_id,grant_id,
                    worker_role_session_id,authoritative_receipt_ref,report_hash,
                    authenticated_actor_id,created_at_ms
                 ) VALUES (
                    'claim-legacy-unmappable','executed','RECORDED_UNVERIFIED',
                    'project:legacy','orchestration:legacy',NULL,NULL,NULL,NULL,NULL,NULL,
                    NULL,NULL,?1,NULL,?2
                 )",
                params![sha_hex(b"legacy-report"), NOW_MS + 1_800],
            )
            .unwrap();
        drop(store);
        let response = refresh_for_state(&fixture.state, NOW_MS + 1_900).unwrap();
        assert_eq!(response.records.len(), 1);
        assert_eq!(response.ignored_non_execution_count, 1);
        assert_eq!(response.quarantines.len(), 1);
        assert_eq!(
            response.quarantines[0].source_claim_ref,
            "claim:claim-legacy-unmappable"
        );
        assert_eq!(response.quarantines[0].payload_mode, "REF_ONLY");
    }

    #[test]
    fn m6d08_temporary_commands_share_global_gate_before_read_replay_or_write() {
        let fixture = fixture("m6d08-global-gate");
        seed_execution(
            &fixture,
            "m6d08-global-gate",
            "SUCCEEDED",
            "role-session:m6d08-global-gate",
        );
        let refresh = refresh_for_state(&fixture.state, NOW_MS + 2_000).expect("seed projection");
        assert_eq!(
            refresh.source_state,
            M6OrgTemporaryAgentSourceState::CompatibleExecutionHistory
        );
        let source = refresh.records[0].clone();
        let promotion = M6OrgPromoteTemporaryAgentRequest {
            temporary_agent_id: source.temporary_agent_id.clone(),
            promoted_by_actor_id: "actor:user".to_string(),
            explicit_human_command: true,
            target: M6OrgTemporaryAgentPromotionTarget::CreateStableMember {
                registration: explicit_registration(
                    "member_m6d08_global_gate",
                    "register-m6d08-global-gate",
                ),
            },
            idempotency_key: "promote-m6d08-global-gate".to_string(),
        };
        promote_for_state(&fixture.state, &promotion, NOW_MS + 2_100)
            .expect("seed promotion replay");

        let m5_path = fixture
            .state
            .m5_store_path()
            .expect("M5 path")
            .to_path_buf();
        let m6_path = fixture.state.m6_org_store_path().expect("M6 path");
        let m5_before = file_hash(&m5_path);
        let m6_before = file_hash(&m6_path);
        let unavailable = AppState {
            index_path: fixture.state.index_path.clone(),
            tasks_path: fixture.state.tasks_path.clone(),
            workflow_state_path: fixture.state.workflow_state_path.clone(),
            m3_role_session_read_runtime: Default::default(),
            m1_project_index: None,
            m3_project_role_session_authority: None,
            m5_store_path: Some(m5_path.clone()),
            m6_org_global_role_session: Default::default(),
        };
        let expected =
            crate::m6_org_global_role_session::M6_ORG_GLOBAL_ROLE_SESSION_UNAVAILABLE.to_string();
        assert_eq!(
            refresh_for_state(&unavailable, NOW_MS + 2_200).unwrap_err(),
            expected
        );
        assert_eq!(
            search_for_state(
                &unavailable,
                &M6OrgSearchTemporaryAgentHistoryRequest {
                    query: source.temporary_agent_id.clone(),
                    limit: 10,
                },
            )
            .unwrap_err(),
            expected
        );
        assert_eq!(
            promote_for_state(&unavailable, &promotion, NOW_MS + 2_300).unwrap_err(),
            expected,
            "even an exact idempotent replay must pass the Global Supervisor gate first"
        );
        assert_eq!(m5_before, file_hash(&m5_path));
        assert_eq!(m6_before, file_hash(&m6_path));
    }

    #[test]
    fn m6d08_no_history_is_distinct_from_schema_carrier_mismatch() {
        let mismatch = fixture("m6d08-schema-mismatch");
        seed_execution(
            &mismatch,
            "m6d08-schema-mismatch",
            "SUCCEEDED",
            "role-session:m6d08-schema-mismatch",
        );
        let mismatch_m5 = mismatch.state.m5_store_path().expect("M5 path");
        let connection = Connection::open(mismatch_m5).expect("open mismatch M5");
        connection
            .execute("DROP TABLE m5_durable_operations", [])
            .expect("remove required durable-operation carrier");
        drop(connection);
        let mismatch_m6 = mismatch.state.m6_org_store_path().expect("M6 path");
        assert!(!mismatch_m6.exists());
        assert_eq!(
            refresh_for_state(&mismatch.state, NOW_MS + 2_400).unwrap_err(),
            M5_SCHEMA_CARRIER_MISMATCH
        );
        assert!(
            !mismatch_m6.exists(),
            "schema mismatch must fail before opening or writing the M6 store"
        );

        let empty = fixture("m6d08-no-history");
        seed_execution(
            &empty,
            "m6d08-no-history",
            "SUCCEEDED",
            "role-session:m6d08-no-history",
        );
        let empty_m5 = empty.state.m5_store_path().expect("M5 path");
        let connection = Connection::open(empty_m5).expect("open true-shaped M5");
        for table in M5_TEMPORARY_AGENT_REQUIRED_TABLES {
            let present: i64 = connection
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1)",
                    [table],
                    |row| row.get(0),
                )
                .expect("inspect required carrier table");
            assert_eq!(present, 1, "required M5 carrier table {table}");
        }
        connection
            .execute("DELETE FROM m5_claims", [])
            .expect("remove execution history without changing schema");
        drop(connection);
        let response = refresh_for_state(&empty.state, NOW_MS + 2_500)
            .expect("true-shaped store with no claims");
        assert_eq!(
            response.source_state,
            M6OrgTemporaryAgentSourceState::NoExecutionHistory
        );
        assert!(response.records.is_empty());
        assert!(response.quarantines.is_empty());

        let production = include_str!("m6_org_temporary_agent_projection.rs");
        assert_eq!(M5_TEMPORARY_AGENT_REQUIRED_TABLES.len(), 11);
        for required_convention in [
            "ExecutionAttemptReadbackRecorded",
            "SCRUBBED_ATTEMPT_RECORD",
            "recording_command_receipt_ref",
            "canonical_readback_hash",
            "m5_durable_operations",
        ] {
            assert!(
                production.contains(required_convention),
                "missing typed M5 carrier convention {required_convention}"
            );
        }
    }

    #[test]
    fn m6d06_production_commands_are_registered_and_no_protected_legacy_module_is_used() {
        let lib = include_str!("lib.rs");
        let commands = include_str!("commands.rs");
        let registry = include_str!("command_registry.rs");
        assert!(lib.contains("mod m6_org_temporary_agent_projection;"));
        for command in [
            "refresh_global_supervisor_temporary_agent_history",
            "search_global_supervisor_temporary_agent_history",
            "promote_global_supervisor_temporary_agent",
        ] {
            assert!(commands.contains(&format!("fn {command}")));
            assert_eq!(registry.matches(command).count(), 1);
        }
        assert!(!lib.contains("mod m6_temporary_agent_history;"));
    }
}
