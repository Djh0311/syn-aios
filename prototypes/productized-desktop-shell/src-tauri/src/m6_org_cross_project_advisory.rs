//! M6D03 read-only cross-project ProjectSummary consumption and advisory.
//!
//! The domain core accepts only M5 `ProjectSummaryQueryPort`; it has no
//! project aggregate, project root, workflow mutation, runner, grant, outbox,
//! sidecar, or raw-source port.  M6 persists only its own advisory, decision,
//! projection, and scrubbed audit records.

use crate::m5_project_summary::{
    ProjectSummary, ProjectSummaryQueryPort, QueryError, SourceRef, SummaryConsumer,
};
use crate::m6_org_dto::*;
use crate::m6_org_global_role_session::{
    authorize_attempted_project_write, M6OrgGlobalRoleSessionSlot, M6OrgGlobalRoleSessionStatusDto,
    M6OrgGlobalSummaryConsumerLease, M6_ORG_GLOBAL_SCOPE_KIND,
};
use crate::m6_org_store::M6OrgStore;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const M6_ORG_ADVISORY_MINIMUM_PROJECTS: usize = 2;
pub(crate) const M6_ORG_SUMMARY_OWNER_REQUIRED: &str = "m6_org_summary_owner_required";
pub(crate) const M6_ORG_SUMMARY_WATERMARK_REQUIRED: &str = "m6_org_summary_watermark_required";
pub(crate) const M6_ORG_PROJECT_WRITE_REJECTED: &str =
    "m6_org_global_role_session_project_write_rejected";
pub(crate) const M6_ORG_ADVISORY_SOURCE_OWNER_REF: &str = "global_supervisor_domain";

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct M6OrgCrossProjectAdvisoryResponse {
    pub(crate) global_role_session: M6OrgGlobalRoleSessionStatusDto,
    pub(crate) freshness_judgements: Vec<M6OrgFreshnessJudgement>,
    pub(crate) advisory: Option<M6OrgCrossProjectAdvisory>,
    pub(crate) stale_advisory_refs: Vec<String>,
    pub(crate) blocked_reasons: Vec<String>,
}

#[derive(Clone)]
struct ValidatedProjectQuery {
    summary_id: String,
    project_id: String,
    project_owner_ref: String,
    policy_decision_ref: String,
    expected_schema_version: String,
    expected_version: u64,
    expected_source_watermark: i64,
    expected_summary_hash: String,
}

struct FreshSummary {
    consumed: M6OrgConsumedProjectSummaryRef,
    source_metadata: Vec<SourceRef>,
}

pub(crate) fn run_for_state(
    state: &crate::AppState,
    request: &M6OrgCrossProjectAdvisoryRequest,
    now_ms: i64,
) -> Result<M6OrgCrossProjectAdvisoryResponse, String> {
    // Validate every caller-carried exact-join field before opening either
    // persistence carrier. Missing owner/watermark therefore has zero write.
    let _ = validate_request_shape(request)?;
    let lease = state
        .m6_org_global_role_session
        .summary_consumer_lease(now_ms)?;
    let m6_path = state.m6_org_store_path()?;
    state.with_m5_project_summary_query_port(|port| {
        let mut m6_store = M6OrgStore::open(&m6_path)?;
        run_with_port(
            &state.m6_org_global_role_session,
            &lease,
            port,
            &mut m6_store,
            request,
            now_ms,
        )
    })
}

fn run_with_port<P: ProjectSummaryQueryPort + ?Sized>(
    role_session: &M6OrgGlobalRoleSessionSlot,
    lease: &M6OrgGlobalSummaryConsumerLease,
    port: &P,
    m6_store: &mut M6OrgStore,
    request: &M6OrgCrossProjectAdvisoryRequest,
    now_ms: i64,
) -> Result<M6OrgCrossProjectAdvisoryResponse, String> {
    let mut queries = validate_request_shape(request)?;
    if lease.scope_kind != M6_ORG_GLOBAL_SCOPE_KIND || now_ms >= lease.consumer_expires_at_ms {
        return Err("m6_org_summary_consumer_gate_denied".to_string());
    }
    queries.sort_by(|left, right| left.project_id.cmp(&right.project_id));
    let request_hash = request_hash(request, &queries)?;
    let existing = m6_store.load_advisory_by_idempotency(&request.idempotency_key)?;
    if let Some(existing) = &existing {
        if existing.request_hash != request_hash {
            return Err("m6_org_advisory_idempotency_collision".to_string());
        }
    }

    let mut judgements = Vec::new();
    let mut fresh = Vec::new();
    let mut observed_current = Vec::new();
    let mut blocked_reasons = Vec::new();
    for query in &queries {
        let consumer = SummaryConsumer {
            role_session_id: lease.role_session_id.clone(),
            role: "global_supervisor".to_string(),
            // The M5 owner gate remains per-project. The surrounding M6 gate
            // has already proved the server-fixed GLOBAL RoleSession.
            scope_project_id: query.project_id.clone(),
            expires_at_ms: lease.consumer_expires_at_ms,
        };
        match port.get_summary(&query.project_id, &consumer, now_ms) {
            Ok(summary) => match admit_summary(query, summary) {
                Ok(admitted) => {
                    judgements.push(freshness_judgement(
                        query,
                        lease,
                        now_ms,
                        M6OrgFreshnessState::Fresh,
                        "summary_exact_and_current",
                    ));
                    observed_current.push(admitted.consumed.clone());
                    fresh.push(admitted);
                }
                Err((state, reason, current)) => {
                    judgements.push(freshness_judgement(query, lease, now_ms, state, &reason));
                    blocked_reasons.push(reason);
                    if let Some(current) = current {
                        observed_current.push(current);
                    }
                }
            },
            Err(error) => {
                let (state, reason) = map_query_error(error);
                judgements.push(freshness_judgement(query, lease, now_ms, state, &reason));
                blocked_reasons.push(reason);
            }
        }
    }

    let stale_advisory_refs = if observed_current.is_empty() {
        Vec::new()
    } else {
        m6_store.mark_issued_advisories_stale_for_source_changes(&observed_current, now_ms)?
    };

    if fresh.len() != queries.len() || fresh.len() < M6_ORG_ADVISORY_MINIMUM_PROJECTS {
        blocked_reasons.sort();
        blocked_reasons.dedup();
        return Ok(M6OrgCrossProjectAdvisoryResponse {
            global_role_session: role_session.status(),
            freshness_judgements: judgements,
            advisory: None,
            stale_advisory_refs,
            blocked_reasons,
        });
    }

    fresh.sort_by(|left, right| left.consumed.project_id.cmp(&right.consumed.project_id));
    let consumed_summaries = fresh
        .iter()
        .map(|entry| entry.consumed.clone())
        .collect::<Vec<_>>();
    let source_links = build_source_links(&fresh);
    let findings = build_findings(&consumed_summaries, &source_links);
    if let Some(existing) = existing {
        if existing.lifecycle_status != "ISSUED"
            || existing.freshness_state != M6OrgFreshnessState::Fresh
        {
            return Ok(M6OrgCrossProjectAdvisoryResponse {
                global_role_session: role_session.status(),
                freshness_judgements: judgements,
                advisory: None,
                stale_advisory_refs,
                blocked_reasons: vec!["m6_org_idempotent_advisory_not_current".to_string()],
            });
        }
        let (summary_refs, source_refs) = context_refs(&consumed_summaries);
        let global_role_session =
            role_session.status_with_minimal_context(summary_refs, source_refs)?;
        return Ok(M6OrgCrossProjectAdvisoryResponse {
            global_role_session,
            freshness_judgements: judgements,
            advisory: Some(existing),
            stale_advisory_refs,
            blocked_reasons: Vec::new(),
        });
    }
    let generated_at_ms = consumed_summaries
        .iter()
        .map(|summary| summary.source_watermark)
        .max()
        .ok_or_else(|| "m6_org_advisory_sources_missing".to_string())?;
    let policy_decision_ref = format!(
        "m6-policy-bundle:{}",
        sha_hex(
            &consumed_summaries
                .iter()
                .map(|summary| summary.policy_decision_ref.as_str())
                .collect::<Vec<_>>()
                .join("|")
        )
    );
    let advisory_id = format!(
        "m6-advisory:{}",
        sha_hex(&format!("{request_hash}:{policy_decision_ref}"))
    );
    let advisory = M6OrgCrossProjectAdvisory {
        advisory_id,
        global_role_session_id: lease.role_session_id.clone(),
        consult_handoff_ref: request.consult_handoff.consult_handoff_ref.clone(),
        consumed_summaries,
        policy_decision_ref,
        generated_at_ms,
        source_links,
        findings,
        lifecycle_status: "ISSUED".to_string(),
        freshness_state: M6OrgFreshnessState::Fresh,
        revision: 1,
        idempotency_key: request.idempotency_key.clone(),
        request_hash,
        created_at_ms: generated_at_ms,
    };
    let advisory = m6_store.record_advisory(&advisory)?;
    let (summary_refs, source_refs) = context_refs(&advisory.consumed_summaries);
    let global_role_session =
        role_session.status_with_minimal_context(summary_refs, source_refs)?;
    Ok(M6OrgCrossProjectAdvisoryResponse {
        global_role_session,
        freshness_judgements: judgements,
        advisory: Some(advisory),
        stale_advisory_refs,
        blocked_reasons: Vec::new(),
    })
}

pub(crate) fn adopt_for_state(
    state: &crate::AppState,
    request: &M6OrgAdvisoryAdoptionRequest,
    now_ms: i64,
) -> Result<M6OrgDecisionRequest, String> {
    let _ = state
        .m6_org_global_role_session
        .summary_consumer_lease(now_ms)?;
    if !request.user_confirmed {
        return Err("m6_org_advisory_adoption_user_confirmation_required".to_string());
    }
    require_nonempty("advisory_id", &request.advisory_id)?;
    require_nonempty("actor_ref", &request.actor_ref)?;
    require_nonempty("idempotency_key", &request.idempotency_key)?;
    let m6_path = state.m6_org_store_path()?;
    let mut store = M6OrgStore::open(&m6_path)?;
    adopt_with_store(&mut store, request, now_ms)
}

fn adopt_with_store(
    store: &mut M6OrgStore,
    request: &M6OrgAdvisoryAdoptionRequest,
    now_ms: i64,
) -> Result<M6OrgDecisionRequest, String> {
    if !request.user_confirmed {
        return Err("m6_org_advisory_adoption_user_confirmation_required".to_string());
    }
    require_nonempty("advisory_id", &request.advisory_id)?;
    require_nonempty("actor_ref", &request.actor_ref)?;
    require_nonempty("idempotency_key", &request.idempotency_key)?;
    let advisory = store
        .load_advisory(&request.advisory_id)?
        .ok_or_else(|| "m6_org_advisory_not_found".to_string())?;
    if advisory.lifecycle_status != "ISSUED" {
        return Err("m6_org_advisory_not_issuable_for_adoption".to_string());
    }
    let decision = M6OrgDecisionRequest {
        decision_request_id: format!(
            "m6-decision-request:{}",
            sha_hex(&format!(
                "{}:{}:{}",
                request.advisory_id, request.actor_ref, request.idempotency_key
            ))
        ),
        source_owner_ref: M6_ORG_ADVISORY_SOURCE_OWNER_REF.to_string(),
        source_object_ref: request.advisory_id.clone(),
        source_revision: advisory.revision,
        requesting_actor_id: request.actor_ref.clone(),
        required_actor_ref: request.actor_ref.clone(),
        required_scope_ref: M6_ORG_GLOBAL_SCOPE_KIND.to_string(),
        question_schema_ref: "m6.advisory-adoption.question.v1".to_string(),
        allowed_answer_schema_ref: "m6.advisory-adoption.answer.v1".to_string(),
        decision_command_type: "AdoptCrossProjectAdvisoryDecision".to_string(),
        status: "PENDING".to_string(),
        idempotency_key: request.idempotency_key.clone(),
        created_at_ms: now_ms,
        expires_at_ms: now_ms
            .checked_add(86_400_000)
            .ok_or_else(|| "m6_org_decision_expiry_overflow".to_string())?,
    };
    store.record_decision_request(&decision)
}

pub(crate) fn observe_application_for_state(
    state: &crate::AppState,
    request: &M6OrgApplicationReceiptObservationRequest,
    now_ms: i64,
) -> Result<M6OrgAdvisoryApplicationProjection, String> {
    let _ = state
        .m6_org_global_role_session
        .summary_consumer_lease(now_ms)?;
    validate_application_receipt(request)?;
    state.verify_m5_authoritative_application_receipt(request)?;
    let m6_path = state.m6_org_store_path()?;
    let mut store = M6OrgStore::open(&m6_path)?;
    observe_application_with_store(&mut store, request)
}

fn observe_application_with_store(
    store: &mut M6OrgStore,
    request: &M6OrgApplicationReceiptObservationRequest,
) -> Result<M6OrgAdvisoryApplicationProjection, String> {
    validate_application_receipt(request)?;
    let observation = M6OrgPerProjectApplicationObservation {
        observation_id: format!(
            "m6-application-observation:{}",
            sha_hex(&format!(
                "{}:{}:{}",
                request.advisory_id,
                request.decision_request_id,
                request.authoritative_command_receipt_ref
            ))
        ),
        advisory_id: request.advisory_id.clone(),
        decision_request_id: request.decision_request_id.clone(),
        project_id: request.project_id.clone(),
        project_owner_ref: request.project_owner_ref.clone(),
        authoritative_command_receipt_ref: request.authoritative_command_receipt_ref.clone(),
        grant_ref: request.grant_ref.clone(),
        outcome: request.outcome,
        observed_at_ms: request.observed_at_ms,
        source_receipt_hash: request.source_receipt_hash.clone(),
    };
    store.record_application_observation(&observation)
}

pub(crate) fn reject_project_write_attempt(
    role_session: &M6OrgGlobalRoleSessionSlot,
    _request: &M6OrgProjectWriteAttemptRequest,
) -> Result<(), String> {
    // This must remain the first and only application boundary action. No M5
    // or M6 store, file, sidecar, audit, outbox, grant, or workflow is opened.
    authorize_attempted_project_write(role_session).map_err(|error| {
        if error == M6_ORG_PROJECT_WRITE_REJECTED {
            error
        } else {
            M6_ORG_PROJECT_WRITE_REJECTED.to_string()
        }
    })
}

fn validate_request_shape(
    request: &M6OrgCrossProjectAdvisoryRequest,
) -> Result<Vec<ValidatedProjectQuery>, String> {
    if request.project_queries.len() < M6_ORG_ADVISORY_MINIMUM_PROJECTS {
        return Err("m6_org_advisory_requires_two_projects".to_string());
    }
    require_nonempty("idempotency_key", &request.idempotency_key)?;
    validate_consult_handoff(&request.consult_handoff)?;
    let mut seen_projects = BTreeSet::new();
    let mut validated = Vec::new();
    for query in &request.project_queries {
        require_nonempty("project_id", &query.project_id)?;
        if !seen_projects.insert(query.project_id.clone()) {
            return Err("m6_org_duplicate_project_query".to_string());
        }
        let project_owner_ref = query
            .project_owner_ref
            .as_ref()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| M6_ORG_SUMMARY_OWNER_REQUIRED.to_string())?
            .clone();
        let expected_owner = project_owner_ref_for(&query.project_id);
        if project_owner_ref != expected_owner {
            return Err("m6_org_summary_owner_mismatch".to_string());
        }
        let summary_id = query
            .summary_id
            .as_ref()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| "m6_org_summary_id_required".to_string())?
            .clone();
        if summary_id != summary_id_for(&query.project_id) {
            return Err("m6_org_summary_id_mismatch".to_string());
        }
        let policy_decision_ref = query
            .policy_decision_ref
            .as_ref()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| "m6_org_policy_decision_required".to_string())?
            .clone();
        if policy_decision_ref != policy_allow_ref_for(&query.project_id, &project_owner_ref) {
            return Err("m6_org_policy_decision_denied".to_string());
        }
        let expected_schema_version = query
            .expected_schema_version
            .as_ref()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| "m6_org_summary_schema_version_required".to_string())?
            .clone();
        let expected_version = query
            .expected_version
            .filter(|value| *value > 0)
            .ok_or_else(|| "m6_org_summary_version_required".to_string())?;
        let expected_source_watermark = query
            .expected_source_watermark
            .filter(|value| *value > 0)
            .ok_or_else(|| M6_ORG_SUMMARY_WATERMARK_REQUIRED.to_string())?;
        let expected_summary_hash = query
            .expected_summary_hash
            .as_ref()
            .filter(|value| is_sha256(value))
            .ok_or_else(|| "m6_org_summary_hash_required".to_string())?
            .clone();
        validated.push(ValidatedProjectQuery {
            summary_id,
            project_id: query.project_id.clone(),
            project_owner_ref,
            policy_decision_ref,
            expected_schema_version,
            expected_version,
            expected_source_watermark,
            expected_summary_hash,
        });
    }
    Ok(validated)
}

fn validate_consult_handoff(handoff: &M6OrgConsultHandoffRefInput) -> Result<(), String> {
    require_nonempty("handoff_id", &handoff.handoff_id)?;
    require_nonempty("status_ref", &handoff.status_ref)?;
    require_nonempty("receipt_ref", &handoff.receipt_ref)?;
    if handoff.handoff_revision == 0 {
        return Err("m6_org_handoff_revision_required".to_string());
    }
    if handoff.status_ref != "ACCEPTED" {
        return Err("m6_org_handoff_not_accepted".to_string());
    }
    let expected = consult_handoff_ref_for(
        &handoff.handoff_id,
        handoff.handoff_revision,
        &handoff.status_ref,
        &handoff.receipt_ref,
    );
    if handoff.consult_handoff_ref != expected {
        return Err("m6_org_handoff_exact_join_mismatch".to_string());
    }
    Ok(())
}

fn admit_summary(
    query: &ValidatedProjectQuery,
    summary: ProjectSummary,
) -> Result<
    FreshSummary,
    (
        M6OrgFreshnessState,
        String,
        Option<M6OrgConsumedProjectSummaryRef>,
    ),
> {
    if summary.project_id != query.project_id {
        return Err((
            M6OrgFreshnessState::Denied,
            "summary_foreign_project".to_string(),
            None,
        ));
    }
    if summary.schema_version.trim().is_empty()
        || summary.version == 0
        || summary.watermark_ms <= 0
        || !is_sha256(&summary.summary_hash)
        || summary.source_refs.is_empty()
        || summary.source_refs.iter().any(|source| {
            source.source_type.trim().is_empty()
                || source.source_id.trim().is_empty()
                || source.last_updated_ms <= 0
        })
    {
        return Err((
            M6OrgFreshnessState::Degraded,
            "summary_required_metadata_incomplete".to_string(),
            None,
        ));
    }
    let source_refs = summary
        .source_refs
        .iter()
        .map(|source| source_context_ref(&summary.project_id, source))
        .collect::<Vec<_>>();
    let consumed = M6OrgConsumedProjectSummaryRef {
        summary_id: query.summary_id.clone(),
        project_id: summary.project_id.clone(),
        project_owner_ref: query.project_owner_ref.clone(),
        schema_version: summary.schema_version.clone(),
        version: summary.version,
        source_watermark: summary.watermark_ms,
        summary_hash: summary.summary_hash.clone(),
        policy_decision_ref: query.policy_decision_ref.clone(),
        freshness_state: M6OrgFreshnessState::Fresh,
        source_refs,
        orchestration_id: summary.orchestration_id.clone(),
        fact_count: summary.fact_count,
        unverified_claim_count: summary.unverified_claim_count,
        open_run_count: summary.open_run_count,
    };
    if summary.schema_version != query.expected_schema_version
        || summary.version != query.expected_version
        || summary.watermark_ms != query.expected_source_watermark
        || summary.summary_hash != query.expected_summary_hash
    {
        return Err((
            M6OrgFreshnessState::Stale,
            "summary_version_watermark_or_hash_stale".to_string(),
            Some(consumed),
        ));
    }
    Ok(FreshSummary {
        consumed,
        source_metadata: summary.source_refs,
    })
}

fn map_query_error(error: QueryError) -> (M6OrgFreshnessState, String) {
    match error {
        QueryError::ProjectNotFound(_) => {
            (M6OrgFreshnessState::Missing, "summary_missing".to_string())
        }
        QueryError::InsufficientPermission(_) | QueryError::ConsumerExpired(_) => (
            M6OrgFreshnessState::Denied,
            "summary_consumer_denied".to_string(),
        ),
        QueryError::SummaryStale(_) => (
            M6OrgFreshnessState::Stale,
            "summary_projector_stale".to_string(),
        ),
        QueryError::StorageError(_) => (
            M6OrgFreshnessState::Degraded,
            "summary_projector_degraded".to_string(),
        ),
    }
}

fn freshness_judgement(
    query: &ValidatedProjectQuery,
    lease: &M6OrgGlobalSummaryConsumerLease,
    now_ms: i64,
    state: M6OrgFreshnessState,
    reason: &str,
) -> M6OrgFreshnessJudgement {
    M6OrgFreshnessJudgement {
        freshness_state: state,
        subject_ref: query.summary_id.clone(),
        reason_code: reason.to_string(),
        judged_at_ms: now_ms,
        consumer_gate_ref: consumer_gate_ref(lease, &query.project_id),
        cache_reuse: false,
    }
}

fn consumer_gate_ref(lease: &M6OrgGlobalSummaryConsumerLease, project_id: &str) -> String {
    format!(
        "m6-summary-consumer-gate:{}:r{}:{}:{}",
        lease.role_session_id,
        lease.role_session_revision,
        lease.scope_kind,
        sha_hex(project_id)
    )
}

fn build_source_links(fresh: &[FreshSummary]) -> Vec<M6OrgAdvisorySourceLink> {
    let mut links = Vec::new();
    for entry in fresh {
        for source in &entry.source_metadata {
            let source_link_id = format!(
                "m6-source-link:{}",
                sha_hex(&format!(
                    "{}:{}:{}:{}",
                    entry.consumed.project_id,
                    entry.consumed.summary_id,
                    source.source_type,
                    source.source_id
                ))
            );
            links.push(M6OrgAdvisorySourceLink {
                source_link_id,
                object_ref: format!("{}:{}", source.source_type, source.source_id),
                project_id: entry.consumed.project_id.clone(),
                summary_id: entry.consumed.summary_id.clone(),
                title_ref: format!("title-ref:{}", source.source_type),
                scrubbed_summary_ref: format!(
                    "summary-ref:{}:v{}",
                    entry.consumed.summary_id, entry.consumed.version
                ),
                deep_link_metadata_ref: format!(
                    "deep-link:{}",
                    sha_hex(&format!(
                        "{}:{}:{}",
                        entry.consumed.project_id, source.source_type, source.source_id
                    ))
                ),
            });
        }
    }
    links.sort_by(|left, right| left.source_link_id.cmp(&right.source_link_id));
    links
}

fn build_findings(
    summaries: &[M6OrgConsumedProjectSummaryRef],
    source_links: &[M6OrgAdvisorySourceLink],
) -> Vec<M6OrgAdvisoryFinding> {
    let links_by_project =
        source_links
            .iter()
            .fold(BTreeMap::<&str, Vec<String>>::new(), |mut grouped, link| {
                grouped
                    .entry(link.project_id.as_str())
                    .or_default()
                    .push(link.source_link_id.clone());
                grouped
            });
    let mut findings = Vec::new();
    for summary in summaries {
        if summary.unverified_claim_count > 0 {
            findings.push(finding(
                "risk",
                "unverified_claims_present",
                90,
                vec![summary.summary_id.clone()],
                links_by_project
                    .get(summary.project_id.as_str())
                    .cloned()
                    .unwrap_or_default(),
            ));
        }
    }
    let mut orchestration_projects = BTreeMap::<&str, Vec<&M6OrgConsumedProjectSummaryRef>>::new();
    for summary in summaries {
        orchestration_projects
            .entry(summary.orchestration_id.as_str())
            .or_default()
            .push(summary);
    }
    for group in orchestration_projects
        .values()
        .filter(|group| group.len() >= 2)
    {
        findings.push(finding(
            "dependency",
            "shared_orchestration_dependency",
            70,
            group.iter().map(|entry| entry.summary_id.clone()).collect(),
            group
                .iter()
                .flat_map(|entry| {
                    links_by_project
                        .get(entry.project_id.as_str())
                        .cloned()
                        .unwrap_or_default()
                })
                .collect(),
        ));
    }
    let active = summaries
        .iter()
        .filter(|summary| summary.open_run_count > 0)
        .collect::<Vec<_>>();
    if active.len() >= 2 {
        findings.push(finding(
            "conflict",
            "concurrent_open_runs_priority_conflict",
            80,
            active
                .iter()
                .map(|entry| entry.summary_id.clone())
                .collect(),
            active
                .iter()
                .flat_map(|entry| {
                    links_by_project
                        .get(entry.project_id.as_str())
                        .cloned()
                        .unwrap_or_default()
                })
                .collect(),
        ));
    }
    if let Some(priority) = summaries.iter().max_by(|left, right| {
        advisory_priority_score(left)
            .cmp(&advisory_priority_score(right))
            .then_with(|| right.project_id.cmp(&left.project_id))
    }) {
        findings.push(finding(
            "priority",
            "deterministic_project_priority",
            60,
            vec![priority.summary_id.clone()],
            links_by_project
                .get(priority.project_id.as_str())
                .cloned()
                .unwrap_or_default(),
        ));
    }
    findings.sort_by(|left, right| {
        right
            .priority
            .cmp(&left.priority)
            .then_with(|| left.finding_id.cmp(&right.finding_id))
    });
    findings
}

fn finding(
    kind: &str,
    reason: &str,
    priority: u32,
    mut summary_refs: Vec<String>,
    mut source_link_refs: Vec<String>,
) -> M6OrgAdvisoryFinding {
    summary_refs.sort();
    summary_refs.dedup();
    source_link_refs.sort();
    source_link_refs.dedup();
    let identity = sha_hex(&format!(
        "{kind}:{reason}:{}:{}",
        summary_refs.join("|"),
        source_link_refs.join("|")
    ));
    M6OrgAdvisoryFinding {
        finding_id: format!("m6-finding:{identity}"),
        finding_kind: kind.to_string(),
        reason_code: reason.to_string(),
        priority,
        summary_refs,
        source_link_refs,
        explanation_ref: format!("m6-explanation:{identity}"),
    }
}

fn advisory_priority_score(summary: &M6OrgConsumedProjectSummaryRef) -> u64 {
    u64::from(summary.unverified_claim_count) * 100
        + u64::from(summary.open_run_count) * 10
        + u64::from(summary.fact_count)
}

fn context_refs(summaries: &[M6OrgConsumedProjectSummaryRef]) -> (Vec<String>, Vec<String>) {
    let summary_refs = summaries
        .iter()
        .map(|summary| {
            format!(
                "{}:v{}:w{}:h{}",
                summary.summary_id, summary.version, summary.source_watermark, summary.summary_hash
            )
        })
        .collect::<Vec<_>>();
    let source_refs = summaries
        .iter()
        .flat_map(|summary| summary.source_refs.clone())
        .collect::<Vec<_>>();
    (summary_refs, source_refs)
}

fn validate_application_receipt(
    request: &M6OrgApplicationReceiptObservationRequest,
) -> Result<(), String> {
    for (name, value) in [
        ("advisory_id", request.advisory_id.as_str()),
        ("decision_request_id", request.decision_request_id.as_str()),
        ("project_id", request.project_id.as_str()),
        ("project_owner_ref", request.project_owner_ref.as_str()),
        (
            "authoritative_command_receipt_ref",
            request.authoritative_command_receipt_ref.as_str(),
        ),
        ("grant_ref", request.grant_ref.as_str()),
    ] {
        require_nonempty(name, value)?;
    }
    if request.project_owner_ref != project_owner_ref_for(&request.project_id) {
        return Err("m6_org_application_owner_mismatch".to_string());
    }
    if !is_sha256(&request.source_receipt_hash) || request.observed_at_ms <= 0 {
        return Err("m6_org_application_receipt_incomplete".to_string());
    }
    Ok(())
}

fn request_hash(
    request: &M6OrgCrossProjectAdvisoryRequest,
    queries: &[ValidatedProjectQuery],
) -> Result<String, String> {
    let material = serde_json::to_string(&(
        &request.consult_handoff,
        &request.idempotency_key,
        queries
            .iter()
            .map(|query| {
                (
                    &query.summary_id,
                    &query.project_id,
                    &query.project_owner_ref,
                    &query.policy_decision_ref,
                    &query.expected_schema_version,
                    query.expected_version,
                    query.expected_source_watermark,
                    &query.expected_summary_hash,
                )
            })
            .collect::<Vec<_>>(),
    ))
    .map_err(|error| format!("m6_org_request_hash_serialize:{error}"))?;
    Ok(sha_hex(&material))
}

pub(crate) fn project_owner_ref_for(project_id: &str) -> String {
    format!("project-owner:{project_id}")
}

pub(crate) fn summary_id_for(project_id: &str) -> String {
    format!("project-summary:{project_id}")
}

pub(crate) fn policy_allow_ref_for(project_id: &str, owner_ref: &str) -> String {
    format!(
        "m6-policy-allow:{}",
        sha_hex(&format!("{project_id}:{owner_ref}"))
    )
}

pub(crate) fn consult_handoff_ref_for(
    handoff_id: &str,
    revision: u64,
    status_ref: &str,
    receipt_ref: &str,
) -> String {
    format!(
        "m6-consult-handoff:{}",
        sha_hex(&format!(
            "{handoff_id}:{revision}:{status_ref}:{receipt_ref}"
        ))
    )
}

fn source_context_ref(project_id: &str, source: &SourceRef) -> String {
    format!(
        "source-ref:{}:{}:{}:at{}",
        project_id, source.source_type, source.source_id, source.last_updated_ms
    )
}

fn require_nonempty(field: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("m6_org_required_field_missing:{field}"));
    }
    Ok(())
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn sha_hex(material: &str) -> String {
    format!("{:x}", Sha256::digest(material.as_bytes()))
}

#[cfg(test)]
mod m6d03_tests {
    use super::*;
    use crate::m2_dto::{CommandReceiptDto, CommandReceiptStatus};
    use crate::m3_role_session_repository::{
        M3OrdinaryRoleSessionRepositoryConfig, M3RoleSessionSqliteRepository,
        M3_ORDINARY_ROLE_SESSION_RELATIVE_PATH,
    };
    use crate::m5_execution_grant::{ExecutionGrant, GrantMintInput};
    use crate::m5_orchestration_identity::{
        AttemptId, AuthorizationId, OrchestrationId, WorkItemId, WorkflowRunId,
    };
    use crate::m5_project_summary::ensure_summary_schema;
    use crate::m6_org_global_role_session::install_ordinary_product_runtime;
    use crate::AppState;
    use rusqlite::params;
    use sha2::{Digest, Sha256};
    use std::cell::RefCell;
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    const NOW_MS: i64 = 1_000_000;
    static SCRATCH_SEQUENCE: AtomicU64 = AtomicU64::new(1);

    #[derive(Clone, Copy)]
    enum FakeFailure {
        Missing,
        Denied,
        Stale,
        Degraded,
    }

    #[derive(Default)]
    struct FakeProjectSummaryPort {
        summaries: BTreeMap<String, ProjectSummary>,
        failures: BTreeMap<String, FakeFailure>,
        calls: RefCell<Vec<(String, SummaryConsumer, i64)>>,
    }

    impl FakeProjectSummaryPort {
        fn with_summaries(summaries: impl IntoIterator<Item = ProjectSummary>) -> Self {
            Self {
                summaries: summaries
                    .into_iter()
                    .map(|summary| (summary.project_id.clone(), summary))
                    .collect(),
                ..Self::default()
            }
        }
    }

    impl ProjectSummaryQueryPort for FakeProjectSummaryPort {
        fn get_summary(
            &self,
            project_id: &str,
            consumer: &SummaryConsumer,
            now_ms: i64,
        ) -> Result<ProjectSummary, QueryError> {
            self.calls
                .borrow_mut()
                .push((project_id.to_string(), consumer.clone(), now_ms));
            if let Some(failure) = self.failures.get(project_id) {
                return Err(match failure {
                    FakeFailure::Missing => QueryError::ProjectNotFound(project_id.to_string()),
                    FakeFailure::Denied => {
                        QueryError::InsufficientPermission(project_id.to_string())
                    }
                    FakeFailure::Stale => QueryError::SummaryStale(project_id.to_string()),
                    FakeFailure::Degraded => QueryError::StorageError(project_id.to_string()),
                });
            }
            self.summaries
                .get(project_id)
                .cloned()
                .ok_or_else(|| QueryError::ProjectNotFound(project_id.to_string()))
        }
    }

    fn scratch_app_data_root(label: &str) -> (PathBuf, PathBuf) {
        let sequence = SCRATCH_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let fixture_root = std::env::temp_dir().join(format!(
            "syn-m6d03-{label}-{}-{sequence}",
            std::process::id()
        ));
        let app_data_root =
            fixture_root.join(crate::m1_project_index::M1_ORDINARY_APP_DATA_DIR_NAME);
        std::fs::create_dir_all(&app_data_root).expect("create M6D03 scratch app-data root");
        let app_data_root =
            std::fs::canonicalize(&app_data_root).expect("canonicalize M6D03 app-data root");
        (fixture_root, app_data_root)
    }

    fn write_ordinary_seeds(fixture_root: &Path) -> (PathBuf, PathBuf) {
        let seed_dir = fixture_root.join("synthetic-ordinary-product-seeds");
        std::fs::create_dir_all(&seed_dir).expect("create M6D03 seed dir");
        let index_seed = seed_dir.join("codex-index.json");
        let tasks_seed = seed_dir.join("README.md");
        std::fs::write(&index_seed, r#"{"projects":[]}"#).expect("write index seed");
        std::fs::write(&tasks_seed, "# synthetic M6D03 tasks\n").expect("write tasks seed");
        (index_seed, tasks_seed)
    }

    fn role_slot(app_data_root: &Path) -> M6OrgGlobalRoleSessionSlot {
        let repository = M3RoleSessionSqliteRepository::open_ordinary_product(
            &M3OrdinaryRoleSessionRepositoryConfig {
                app_data_root: app_data_root.to_path_buf(),
                db_path: app_data_root.join(M3_ORDINARY_ROLE_SESSION_RELATIVE_PATH),
            },
        )
        .expect("open scratch M3 role-session repository");
        install_ordinary_product_runtime(repository).expect("install global role session")
    }

    fn ordinary_state(label: &str) -> (PathBuf, PathBuf, AppState) {
        let (fixture_root, app_data_root) = scratch_app_data_root(label);
        let (index_seed, tasks_seed) = write_ordinary_seeds(&fixture_root);
        let state = AppState::try_new_with_tauri_ordinary_product_seeds(
            &app_data_root,
            &index_seed,
            &tasks_seed,
        )
        .expect("ordinary M6D03 AppState");
        (fixture_root, app_data_root, state)
    }

    fn summary(
        project_id: &str,
        orchestration_id: &str,
        version: u64,
        watermark_ms: i64,
        unverified_claim_count: u32,
        open_run_count: u32,
    ) -> ProjectSummary {
        ProjectSummary {
            project_id: project_id.to_string(),
            orchestration_id: orchestration_id.to_string(),
            schema_version: "m5.project-summary.v1".to_string(),
            version,
            watermark_ms,
            summary_hash: sha_hex(&format!(
                "{project_id}:{orchestration_id}:{version}:{watermark_ms}:{unverified_claim_count}:{open_run_count}"
            )),
            source_refs: vec![SourceRef {
                source_type: "project_fact".to_string(),
                source_id: format!("fact-ref-{project_id}"),
                last_updated_ms: watermark_ms,
            }],
            fact_count: version as u32 + 2,
            unverified_claim_count,
            open_run_count,
            rebuilt_at_ms: watermark_ms,
        }
    }

    fn query(summary: &ProjectSummary) -> M6OrgProjectSummaryQueryInput {
        let owner = project_owner_ref_for(&summary.project_id);
        M6OrgProjectSummaryQueryInput {
            summary_id: Some(summary_id_for(&summary.project_id)),
            project_id: summary.project_id.clone(),
            project_owner_ref: Some(owner.clone()),
            policy_decision_ref: Some(policy_allow_ref_for(&summary.project_id, &owner)),
            expected_schema_version: Some(summary.schema_version.clone()),
            expected_version: Some(summary.version),
            expected_source_watermark: Some(summary.watermark_ms),
            expected_summary_hash: Some(summary.summary_hash.clone()),
        }
    }

    fn request(
        summaries: &[ProjectSummary],
        idempotency_key: &str,
    ) -> M6OrgCrossProjectAdvisoryRequest {
        let handoff_id = "consult-handoff-m6d03";
        let receipt_ref = "consult-receipt:m6d03";
        let status_ref = "ACCEPTED";
        M6OrgCrossProjectAdvisoryRequest {
            project_queries: summaries.iter().map(query).collect(),
            consult_handoff: M6OrgConsultHandoffRefInput {
                consult_handoff_ref: consult_handoff_ref_for(
                    handoff_id,
                    1,
                    status_ref,
                    receipt_ref,
                ),
                handoff_id: handoff_id.to_string(),
                handoff_revision: 1,
                status_ref: status_ref.to_string(),
                receipt_ref: receipt_ref.to_string(),
            },
            idempotency_key: idempotency_key.to_string(),
        }
    }

    fn seed_summary(state: &AppState, summary: &ProjectSummary) {
        let store = state
            .open_m5_store()
            .expect("open M5 store for synthetic seed");
        ensure_summary_schema(&store).expect("ensure M5 summary schema");
        let source_refs_json =
            serde_json::to_string(&summary.source_refs).expect("serialize source refs");
        store
            .connection()
            .execute(
                "INSERT OR REPLACE INTO m5_project_summaries (
                    project_id, orchestration_id, schema_version, version, watermark_ms,
                    summary_hash, source_refs_json, fact_count, unverified_claim_count,
                    open_run_count, rebuilt_at_ms
                 ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
                params![
                    summary.project_id,
                    summary.orchestration_id,
                    summary.schema_version,
                    summary.version as i64,
                    summary.watermark_ms,
                    summary.summary_hash,
                    source_refs_json,
                    summary.fact_count,
                    summary.unverified_claim_count,
                    summary.open_run_count,
                    summary.rebuilt_at_ms,
                ],
            )
            .expect("seed synthetic M5 ProjectSummary");
    }

    fn file_hash(path: &Path) -> String {
        format!(
            "{:x}",
            Sha256::digest(std::fs::read(path).expect("read file for hash"))
        )
    }

    fn seed_authoritative_application_witness(
        state: &AppState,
        advisory_id: &str,
        decision_request_id: &str,
        project_id: &str,
    ) -> M6OrgApplicationReceiptObservationRequest {
        let store = state.open_m5_store().expect("open M5 witness store");
        let policy_decision_ref = format!("policy:{project_id}:apply-advisory");
        let principal_actor_id = format!("project-owner-actor:{project_id}");
        let mut grant = ExecutionGrant::mint(GrantMintInput {
            project_id: project_id.to_string(),
            orchestration_id: OrchestrationId::new(format!("orchestration:{project_id}")),
            workflow_run_id: WorkflowRunId::new(format!("workflow-run:{project_id}")),
            work_item_id: WorkItemId::new(format!("work-item:{project_id}")),
            attempt_id: AttemptId::new(format!("attempt:{project_id}")),
            authorization_id: AuthorizationId::new(format!("authorization:{project_id}")),
            authorization_revision: 1,
            principal_actor_id: principal_actor_id.clone(),
            worker_role_session_id: format!("role-session:{project_id}"),
            scope_fingerprint: format!("scope-fingerprint:{project_id}"),
            allowed_commands: vec!["ApplyCrossProjectAdvisoryDecision".to_string()],
            cwd_ref: format!("synthetic-project-root:{project_id}"),
            write_root_refs: vec![format!("synthetic-project-root:{project_id}")],
            object_refs: vec![decision_request_id.to_string(), advisory_id.to_string()],
            policy_decision_ref: policy_decision_ref.clone(),
            issued_at_ms: NOW_MS + 10,
            expires_at_ms: NOW_MS + 60_000,
            idempotency_key: format!("grant:{project_id}:apply-advisory"),
            effect_key: format!("effect:{project_id}:apply-advisory"),
            created_by_command_receipt_ref: format!("grant-create-receipt:{project_id}"),
        })
        .expect("mint synthetic application grant");
        let minted_hash = grant.grant_hash.clone();
        grant
            .confirm_readback(&minted_hash, NOW_MS + 20)
            .expect("activate synthetic application grant");
        store
            .persist_grant(&grant)
            .expect("persist application grant");
        let receipt = CommandReceiptDto {
            receipt_id: format!("application-receipt:{project_id}"),
            command_id: format!("apply-advisory:{project_id}"),
            idempotency_key: format!("application-receipt:{project_id}:once"),
            request_hash: sha_hex(&format!("{advisory_id}:{decision_request_id}:{project_id}")),
            actor_id: principal_actor_id,
            scope_ref: project_id.to_string(),
            current_object_ref: Some(decision_request_id.to_string()),
            policy_decision_ref,
            status: CommandReceiptStatus::Committed,
            correlation_id: Some(format!("correlation:{advisory_id}")),
            accepted_at: "1970-01-01T00:16:40Z".to_string(),
            result_ref: Some(format!("application-result:{project_id}")),
            result_hash: Some(sha_hex(&format!("application-result:{project_id}"))),
            committed_revision: Some(1),
            error_code: None,
            created_at: "1970-01-01T00:16:40Z".to_string(),
        };
        store
            .persist_receipt_once(&receipt)
            .expect("persist application receipt");
        let source_receipt_hash = format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(&receipt).expect("serialize receipt witness"))
        );
        M6OrgApplicationReceiptObservationRequest {
            advisory_id: advisory_id.to_string(),
            decision_request_id: decision_request_id.to_string(),
            project_id: project_id.to_string(),
            project_owner_ref: project_owner_ref_for(project_id),
            authoritative_command_receipt_ref: receipt.receipt_id,
            grant_ref: grant.grant_id.as_str().to_string(),
            outcome: M6OrgApplicationOutcome::Applied,
            observed_at_ms: NOW_MS + 30,
            source_receipt_hash,
        }
    }

    fn file_tree(root: &Path) -> BTreeMap<String, String> {
        fn visit(base: &Path, path: &Path, files: &mut BTreeMap<String, String>) {
            let mut entries = std::fs::read_dir(path)
                .expect("read fixture tree")
                .map(|entry| entry.expect("read fixture entry"))
                .collect::<Vec<_>>();
            entries.sort_by_key(|entry| entry.path());
            for entry in entries {
                let entry_path = entry.path();
                let file_type = entry.file_type().expect("read fixture file type");
                if file_type.is_dir() {
                    visit(base, &entry_path, files);
                } else if file_type.is_file() {
                    let relative = entry_path
                        .strip_prefix(base)
                        .expect("fixture-relative path")
                        .to_string_lossy()
                        .replace('\\', "/");
                    files.insert(relative, file_hash(&entry_path));
                }
            }
        }
        let mut files = BTreeMap::new();
        visit(root, root, &mut files);
        files
    }

    fn ready_context(response: &M6OrgCrossProjectAdvisoryResponse) -> (&Vec<String>, &Vec<String>) {
        match &response.global_role_session {
            M6OrgGlobalRoleSessionStatusDto::Ready { context, .. } => {
                (&context.summary_refs, &context.source_refs)
            }
            M6OrgGlobalRoleSessionStatusDto::Unavailable { error } => {
                panic!("expected ready Global RoleSession, got {error}")
            }
        }
    }

    fn issued_advisory(
        slot: &M6OrgGlobalRoleSessionSlot,
        store: &mut M6OrgStore,
    ) -> M6OrgCrossProjectAdvisory {
        let alpha = summary("project-alpha", "shared-orchestration", 1, 999_000, 1, 1);
        let beta = summary("project-beta", "shared-orchestration", 1, 999_500, 0, 1);
        let port = FakeProjectSummaryPort::with_summaries([alpha.clone(), beta.clone()]);
        run_with_port(
            slot,
            &slot.summary_consumer_lease(NOW_MS).expect("consumer lease"),
            &port,
            store,
            &request(&[alpha, beta], "issue-advisory"),
            NOW_MS,
        )
        .expect("issue advisory")
        .advisory
        .expect("issued advisory")
    }

    #[test]
    fn m6d03_two_project_advisory_is_deterministic_sourced_and_context_minimal() {
        let (fixture_root, app_data_root) = scratch_app_data_root("deterministic");
        let slot = role_slot(&app_data_root);
        let alpha = summary("project-alpha", "shared-orchestration", 1, 999_000, 2, 1);
        let beta = summary("project-beta", "shared-orchestration", 3, 999_500, 0, 2);
        let port = FakeProjectSummaryPort::with_summaries([beta.clone(), alpha.clone()]);
        let request = request(&[beta, alpha], "deterministic-request");
        let lease = slot.summary_consumer_lease(NOW_MS).expect("consumer lease");
        let mut first_store = M6OrgStore::open_in_memory().expect("first M6 store");
        let first = run_with_port(&slot, &lease, &port, &mut first_store, &request, NOW_MS)
            .expect("first advisory");
        let mut second_store = M6OrgStore::open_in_memory().expect("second M6 store");
        let second = run_with_port(&slot, &lease, &port, &mut second_store, &request, NOW_MS)
            .expect("second advisory");
        assert_eq!(first, second, "same inputs must produce the same output");

        let advisory = first.advisory.as_ref().expect("advisory");
        assert_eq!(advisory.consumed_summaries.len(), 2);
        assert!(advisory.source_links.len() >= 2);
        assert_eq!(
            advisory
                .findings
                .iter()
                .map(|finding| finding.finding_kind.as_str())
                .collect::<BTreeSet<_>>(),
            BTreeSet::from(["conflict", "dependency", "priority", "risk"])
        );
        for finding in &advisory.findings {
            assert!(!finding.summary_refs.is_empty());
            assert!(!finding.source_link_refs.is_empty());
        }
        let (summary_refs, source_refs) = ready_context(&first);
        assert_eq!(summary_refs.len(), 2);
        assert_eq!(source_refs.len(), 2);
        assert!(summary_refs
            .iter()
            .all(|value| { value.contains(":v") && value.contains(":w") && value.contains(":h") }));
        assert!(source_refs
            .iter()
            .all(|value| value.starts_with("source-ref:")));

        let calls = port.calls.borrow();
        assert_eq!(calls.len(), 4);
        for (project_id, consumer, observed_now) in calls.iter() {
            assert_eq!(&consumer.scope_project_id, project_id);
            assert_eq!(consumer.role, "global_supervisor");
            assert_eq!(consumer.role_session_id, lease.role_session_id);
            assert_eq!(consumer.expires_at_ms, lease.consumer_expires_at_ms);
            assert_eq!(*observed_now, NOW_MS);
        }
        drop(calls);

        let replay = run_with_port(&slot, &lease, &port, &mut first_store, &request, NOW_MS)
            .expect("idempotent replay");
        assert_eq!(replay, first);
        assert_eq!(
            first_store
                .count_rows("m6_cross_project_advisories")
                .expect("count advisories"),
            1
        );
        let _ = std::fs::remove_dir_all(fixture_root);
    }

    #[test]
    fn m6d03_freshness_states_fail_closed_without_cached_advisory() {
        let (fixture_root, app_data_root) = scratch_app_data_root("freshness");
        let slot = role_slot(&app_data_root);
        let inputs = [
            summary("project-missing", "orch-a", 1, 999_100, 0, 0),
            summary("project-denied", "orch-b", 1, 999_200, 0, 0),
            summary("project-stale", "orch-c", 1, 999_300, 0, 0),
            summary("project-degraded", "orch-d", 1, 999_400, 0, 0),
        ];
        let mut port = FakeProjectSummaryPort::default();
        port.failures
            .insert("project-missing".to_string(), FakeFailure::Missing);
        port.failures
            .insert("project-denied".to_string(), FakeFailure::Denied);
        port.failures
            .insert("project-stale".to_string(), FakeFailure::Stale);
        port.failures
            .insert("project-degraded".to_string(), FakeFailure::Degraded);
        let mut store = M6OrgStore::open_in_memory().expect("M6 store");
        let response = run_with_port(
            &slot,
            &slot.summary_consumer_lease(NOW_MS).expect("consumer lease"),
            &port,
            &mut store,
            &request(&inputs, "freshness-states"),
            NOW_MS,
        )
        .expect("freshness response");
        assert!(response.advisory.is_none());
        assert_eq!(
            response
                .freshness_judgements
                .iter()
                .map(|judgement| judgement.freshness_state)
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([
                M6OrgFreshnessState::Missing,
                M6OrgFreshnessState::Denied,
                M6OrgFreshnessState::Stale,
                M6OrgFreshnessState::Degraded,
            ])
        );
        assert!(response
            .freshness_judgements
            .iter()
            .all(|judgement| !judgement.cache_reuse));
        assert_eq!(
            store
                .count_rows("m6_cross_project_advisories")
                .expect("count advisories"),
            0
        );
        assert_eq!(
            store
                .count_rows("m6_org_audit_events")
                .expect("count audits"),
            0
        );
        let _ = std::fs::remove_dir_all(fixture_root);
    }

    #[test]
    fn m6d03_owner_watermark_policy_scope_and_foreign_summary_are_rejected_before_write() {
        let (fixture_root, app_data_root) = scratch_app_data_root("gates");
        let slot = role_slot(&app_data_root);
        let alpha = summary("project-alpha", "orch-a", 1, 999_000, 0, 0);
        let beta = summary("project-beta", "orch-b", 1, 999_500, 0, 0);
        let port = FakeProjectSummaryPort::with_summaries([alpha.clone(), beta.clone()]);
        let lease = slot.summary_consumer_lease(NOW_MS).expect("consumer lease");
        let mut store = M6OrgStore::open_in_memory().expect("M6 store");

        let mut missing_owner = request(&[alpha.clone(), beta.clone()], "missing-owner");
        missing_owner.project_queries[0].project_owner_ref = None;
        assert_eq!(
            run_with_port(&slot, &lease, &port, &mut store, &missing_owner, NOW_MS)
                .expect_err("missing owner must reject"),
            M6_ORG_SUMMARY_OWNER_REQUIRED
        );

        let mut missing_watermark = request(&[alpha.clone(), beta.clone()], "missing-watermark");
        missing_watermark.project_queries[0].expected_source_watermark = None;
        assert_eq!(
            run_with_port(&slot, &lease, &port, &mut store, &missing_watermark, NOW_MS)
                .expect_err("missing watermark must reject"),
            M6_ORG_SUMMARY_WATERMARK_REQUIRED
        );

        let mut denied_policy = request(&[alpha.clone(), beta.clone()], "denied-policy");
        denied_policy.project_queries[0].policy_decision_ref = Some("deny".to_string());
        assert_eq!(
            run_with_port(&slot, &lease, &port, &mut store, &denied_policy, NOW_MS)
                .expect_err("policy mismatch must reject"),
            "m6_org_policy_decision_denied"
        );

        let mut expired = lease.clone();
        expired.consumer_expires_at_ms = NOW_MS;
        assert_eq!(
            run_with_port(
                &slot,
                &expired,
                &port,
                &mut store,
                &request(&[alpha.clone(), beta.clone()], "expired"),
                NOW_MS
            )
            .expect_err("expired M6 consumer gate must reject"),
            "m6_org_summary_consumer_gate_denied"
        );

        let foreign_alpha = summary("project-foreign", "orch-a", 1, 999_000, 0, 0);
        let foreign_port = FakeProjectSummaryPort {
            summaries: BTreeMap::from([
                ("project-alpha".to_string(), foreign_alpha),
                ("project-beta".to_string(), beta.clone()),
            ]),
            ..FakeProjectSummaryPort::default()
        };
        let foreign = run_with_port(
            &slot,
            &lease,
            &foreign_port,
            &mut store,
            &request(&[alpha, beta], "foreign-summary"),
            NOW_MS,
        )
        .expect("foreign summary judgement");
        assert!(foreign.advisory.is_none());
        assert!(foreign.freshness_judgements.iter().any(|judgement| {
            judgement.freshness_state == M6OrgFreshnessState::Denied
                && judgement.reason_code == "summary_foreign_project"
        }));
        assert_eq!(
            store
                .count_rows("m6_cross_project_advisories")
                .expect("count advisories"),
            0
        );
        assert_eq!(port.calls.borrow().len(), 0);
        let _ = std::fs::remove_dir_all(fixture_root);
    }

    #[test]
    fn m6d03_summary_change_marks_prior_advisory_stale_without_overwrite() {
        let (fixture_root, app_data_root) = scratch_app_data_root("stale-history");
        let slot = role_slot(&app_data_root);
        let alpha_v1 = summary("project-alpha", "orch-a", 1, 999_000, 0, 0);
        let beta = summary("project-beta", "orch-b", 1, 999_100, 0, 0);
        let mut store = M6OrgStore::open_in_memory().expect("M6 store");
        let first = run_with_port(
            &slot,
            &slot.summary_consumer_lease(NOW_MS).expect("consumer lease"),
            &FakeProjectSummaryPort::with_summaries([alpha_v1.clone(), beta.clone()]),
            &mut store,
            &request(&[alpha_v1.clone(), beta.clone()], "stale-v1"),
            NOW_MS,
        )
        .expect("v1 advisory")
        .advisory
        .expect("v1 advisory payload");

        let alpha_v2 = summary("project-alpha", "orch-a", 2, 999_700, 1, 0);
        let second = run_with_port(
            &slot,
            &slot.summary_consumer_lease(NOW_MS).expect("consumer lease"),
            &FakeProjectSummaryPort::with_summaries([alpha_v2.clone(), beta.clone()]),
            &mut store,
            &request(&[alpha_v2, beta], "stale-v2"),
            NOW_MS + 1,
        )
        .expect("v2 advisory");
        assert_eq!(second.stale_advisory_refs, vec![first.advisory_id.clone()]);
        let prior = store
            .load_advisory(&first.advisory_id)
            .expect("load prior advisory")
            .expect("prior advisory remains");
        assert_eq!(prior.lifecycle_status, "STALE");
        assert_eq!(prior.freshness_state, M6OrgFreshnessState::Stale);
        assert_eq!(prior.revision, 2);
        assert_ne!(
            second.advisory.expect("v2 advisory payload").advisory_id,
            first.advisory_id
        );
        assert_eq!(
            store
                .count_rows("m6_cross_project_advisories")
                .expect("count advisories"),
            2
        );
        let current_alpha = summary("project-alpha", "orch-a", 2, 999_700, 1, 0);
        let current_beta = summary("project-beta", "orch-b", 1, 999_100, 0, 0);
        let old_request_after_change = run_with_port(
            &slot,
            &slot.summary_consumer_lease(NOW_MS).expect("consumer lease"),
            &FakeProjectSummaryPort::with_summaries([current_alpha, current_beta.clone()]),
            &mut store,
            &request(&[alpha_v1, current_beta], "stale-v1"),
            NOW_MS + 2,
        )
        .expect("old request after source change");
        assert!(old_request_after_change.advisory.is_none());
        assert!(old_request_after_change
            .freshness_judgements
            .iter()
            .any(|judgement| judgement.freshness_state == M6OrgFreshnessState::Stale));
        assert!(old_request_after_change
            .freshness_judgements
            .iter()
            .all(|judgement| !judgement.cache_reuse));
        let _ = std::fs::remove_dir_all(fixture_root);
    }

    #[test]
    fn m6d03_adoption_only_records_decision_and_receipt_projection_is_append_only() {
        let (fixture_root, app_data_root) = scratch_app_data_root("decision-projection");
        let slot = role_slot(&app_data_root);
        let mut store = M6OrgStore::open_in_memory().expect("M6 store");
        let advisory = issued_advisory(&slot, &mut store);
        let adoption = M6OrgAdvisoryAdoptionRequest {
            advisory_id: advisory.advisory_id.clone(),
            actor_ref: "user:synthetic-owner".to_string(),
            user_confirmed: true,
            idempotency_key: "adopt-once".to_string(),
        };
        let decision =
            adopt_with_store(&mut store, &adoption, NOW_MS + 10).expect("create DecisionRequest");
        assert_eq!(decision.status, "PENDING");
        assert_eq!(decision.source_owner_ref, M6_ORG_ADVISORY_SOURCE_OWNER_REF);
        assert_eq!(decision.source_object_ref, advisory.advisory_id);
        assert_eq!(decision.source_revision, advisory.revision);
        assert_eq!(decision.requesting_actor_id, "user:synthetic-owner");
        assert_eq!(decision.required_actor_ref, "user:synthetic-owner");
        assert_eq!(decision.required_scope_ref, M6_ORG_GLOBAL_SCOPE_KIND);
        assert_eq!(decision.status, "PENDING");
        assert!(decision.expires_at_ms > decision.created_at_ms);
        assert_eq!(
            adopt_with_store(&mut store, &adoption, NOW_MS + 20).expect("idempotent adoption"),
            decision
        );
        assert_eq!(
            store
                .count_rows("m6_decision_requests")
                .expect("count decisions"),
            1
        );
        assert_eq!(
            store
                .count_rows("m6_advisory_application_observations")
                .expect("count observations"),
            0
        );

        let observation = |project_id: &str,
                           receipt: &str,
                           outcome: M6OrgApplicationOutcome,
                           observed_at_ms: i64| {
            M6OrgApplicationReceiptObservationRequest {
                advisory_id: advisory.advisory_id.clone(),
                decision_request_id: decision.decision_request_id.clone(),
                project_id: project_id.to_string(),
                project_owner_ref: project_owner_ref_for(project_id),
                authoritative_command_receipt_ref: format!("command-receipt:{receipt}"),
                grant_ref: format!("grant:{project_id}"),
                outcome,
                observed_at_ms,
                source_receipt_hash: sha_hex(receipt),
            }
        };
        let applied = observation(
            "project-alpha",
            "alpha-applied",
            M6OrgApplicationOutcome::Applied,
            NOW_MS + 30,
        );
        let failed = observation(
            "project-beta",
            "beta-failed",
            M6OrgApplicationOutcome::Failed,
            NOW_MS + 40,
        );
        let rolled_back = observation(
            "project-alpha",
            "alpha-rolled-back",
            M6OrgApplicationOutcome::RolledBack,
            NOW_MS + 50,
        );
        let first_projection =
            observe_application_with_store(&mut store, &applied).expect("observe applied");
        assert!(!first_projection.partial_apply);
        let second_projection =
            observe_application_with_store(&mut store, &failed).expect("observe failed");
        assert!(second_projection.partial_apply);
        assert_eq!(second_projection.projection_revision, 2);
        let final_projection =
            observe_application_with_store(&mut store, &rolled_back).expect("observe compensation");
        assert!(final_projection.partial_apply);
        assert_eq!(final_projection.projection_revision, 3);
        assert_eq!(final_projection.history.len(), 3);
        assert_eq!(final_projection.compensation_observation_refs.len(), 1);
        let replay =
            observe_application_with_store(&mut store, &applied).expect("idempotent receipt");
        assert_eq!(replay.history.len(), 3);
        assert_eq!(
            store
                .count_rows("m6_advisory_application_observations")
                .expect("count observations"),
            3
        );
        let unchanged = store
            .load_advisory(&advisory.advisory_id)
            .expect("load advisory")
            .expect("advisory remains");
        assert_eq!(unchanged.lifecycle_status, "ISSUED");
        assert_eq!(unchanged.revision, 1);
        let _ = std::fs::remove_dir_all(fixture_root);
    }

    #[test]
    fn m6d03_ordinary_appstate_reads_real_summary_port_with_m5_hash_unchanged() {
        let (fixture_root, _app_data_root, state) = ordinary_state("ordinary-read");
        let alpha = summary("project-alpha", "shared-orchestration", 1, 999_000, 1, 1);
        let beta = summary("project-beta", "shared-orchestration", 2, 999_500, 0, 1);
        seed_summary(&state, &alpha);
        seed_summary(&state, &beta);
        let m5_path = state.m5_store_path().expect("M5 path").to_path_buf();
        let m6_path = state.m6_org_store_path().expect("M6 path");
        assert!(!m6_path.exists());
        let m5_before = file_hash(&m5_path);
        let non_m6_before = file_tree(&fixture_root);
        let response = run_for_state(
            &state,
            &request(&[beta, alpha], "ordinary-real-port"),
            NOW_MS,
        )
        .expect("ordinary real-port advisory");
        let advisory_id = response
            .advisory
            .as_ref()
            .expect("ordinary advisory")
            .advisory_id
            .clone();
        assert_eq!(
            file_hash(&m5_path),
            m5_before,
            "M5 database must be byte-stable"
        );
        let non_m6_after = file_tree(&fixture_root)
            .into_iter()
            .filter(|(path, _)| !path.contains("/m6/"))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(
            non_m6_after, non_m6_before,
            "all project/domain/event/audit/outbox/sidecar files outside M6 must be byte-stable"
        );
        let m6_store = M6OrgStore::open(&m6_path).expect("open M6 store");
        assert_eq!(
            m6_store
                .count_rows("m6_cross_project_advisories")
                .expect("count advisories"),
            1
        );
        assert_eq!(
            m6_store
                .count_rows("m6_decision_requests")
                .expect("count decisions"),
            0
        );
        drop(m6_store);

        let decision = adopt_for_state(
            &state,
            &M6OrgAdvisoryAdoptionRequest {
                advisory_id: advisory_id.clone(),
                actor_ref: "user:ordinary-synthetic".to_string(),
                user_confirmed: true,
                idempotency_key: "ordinary-adoption".to_string(),
            },
            NOW_MS + 10,
        )
        .expect("ordinary DecisionRequest");
        let observation = seed_authoritative_application_witness(
            &state,
            &advisory_id,
            &decision.decision_request_id,
            "project-alpha",
        );
        let m5_after_witness_seed = file_hash(&m5_path);
        let projection = observe_application_for_state(&state, &observation, NOW_MS + 40)
            .expect("ordinary authoritative application observation");
        assert_eq!(projection.history.len(), 1);
        assert_eq!(projection.observations[0].project_id, "project-alpha");
        assert_eq!(
            file_hash(&m5_path),
            m5_after_witness_seed,
            "authoritative receipt verification must be read-only"
        );
        let mut forged = observation.clone();
        forged.authoritative_command_receipt_ref = "missing-receipt".to_string();
        assert_eq!(
            observe_application_for_state(&state, &forged, NOW_MS + 41)
                .expect_err("forged receipt must fail before M6 write"),
            "m6_org_application_receipt_not_found"
        );
        let m6_store = M6OrgStore::open(&m6_path).expect("reopen M6 store");
        assert_eq!(
            m6_store
                .count_rows("m6_advisory_application_observations")
                .expect("count observations"),
            1
        );
        drop(m6_store);

        let tree_before_reject = file_tree(&fixture_root);
        let error = reject_project_write_attempt(
            &state.m6_org_global_role_session,
            &M6OrgProjectWriteAttemptRequest {
                project_id: "project-alpha".to_string(),
                mutation_kind: "create_workflow".to_string(),
            },
        )
        .expect_err("Global Supervisor project write must reject");
        assert_eq!(error, M6_ORG_PROJECT_WRITE_REJECTED);
        assert_eq!(
            file_tree(&fixture_root),
            tree_before_reject,
            "project-write rejection must occur before every file/store write"
        );
        let _ = std::fs::remove_dir_all(fixture_root);
    }

    #[test]
    fn m6d03_invalid_request_does_not_create_m6_store_and_commands_are_reachable() {
        let (fixture_root, _app_data_root, state) = ordinary_state("pre-open-reject");
        let alpha = summary("project-alpha", "orch-a", 1, 999_000, 0, 0);
        let beta = summary("project-beta", "orch-b", 1, 999_500, 0, 0);
        let m6_path = state.m6_org_store_path().expect("M6 path");
        let mut invalid = request(&[alpha.clone(), beta.clone()], "invalid-owner");
        invalid.project_queries[0].project_owner_ref = None;
        assert_eq!(
            run_for_state(&state, &invalid, NOW_MS).expect_err("missing owner"),
            M6_ORG_SUMMARY_OWNER_REQUIRED
        );
        assert!(!m6_path.exists());
        let mut invalid = request(&[alpha, beta], "invalid-watermark");
        invalid.project_queries[0].expected_source_watermark = None;
        assert_eq!(
            run_for_state(&state, &invalid, NOW_MS).expect_err("missing watermark"),
            M6_ORG_SUMMARY_WATERMARK_REQUIRED
        );
        assert!(!m6_path.exists());

        let registry = include_str!("command_registry.rs");
        for command in [
            "run_global_supervisor_cross_project_advisory",
            "adopt_global_supervisor_cross_project_advisory",
            "observe_global_supervisor_advisory_application_receipt",
            "attempt_global_supervisor_project_write",
        ] {
            assert_eq!(
                registry.matches(command).count(),
                1,
                "registry entry {command}"
            );
        }
        let commands = include_str!("commands.rs");
        assert!(commands.contains("tauri::State<'_, AppState>"));
        assert!(commands.contains("m6_org_cross_project_advisory::run_for_state"));
        assert!(commands.contains("m6_org_cross_project_advisory::adopt_for_state"));
        assert!(commands.contains("m6_org_cross_project_advisory::observe_application_for_state"));
        assert!(commands.contains("m6_org_cross_project_advisory::reject_project_write_attempt"));
        let _ = std::fs::remove_dir_all(fixture_root);
    }

    #[test]
    fn m6d03_production_core_has_no_project_read_or_guarded_legacy_bypass() {
        let source = include_str!("m6_org_cross_project_advisory.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("production module span");
        assert!(production.contains("ProjectSummaryQueryPort"));
        assert!(production.contains("fn run_with_port<P: ProjectSummaryQueryPort + ?Sized>"));
        for forbidden in [
            "PersistentProjectSummaryPort",
            "open_m5_store",
            "project_root",
            "m5_project_facts",
            "m5_claims",
            "workflow_state",
            "execute_project_workflow_node",
            "provider_response",
            "transcript",
        ] {
            assert!(
                !production.contains(forbidden),
                "production advisory core must not contain bypass marker {forbidden}"
            );
        }
        let commands = include_str!("commands.rs");
        let guarded_start = commands
            .find("fn execute_project_workflow_node(")
            .expect("guarded legacy command");
        let guarded_span = &commands[guarded_start..];
        let guarded_end = guarded_span
            .find("fn execute_project_workflow_node_at(")
            .expect("guarded command end");
        let guarded_command = &guarded_span[..guarded_end];
        assert!(guarded_command.contains("workflow_engine_test_project_unsealed"));
        assert!(!guarded_command.contains("ProjectSummary"));
        assert!(!guarded_command.contains("m6_org"));
    }
}
