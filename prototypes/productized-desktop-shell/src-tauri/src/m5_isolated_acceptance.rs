// M5R07 isolated scratch acceptance. Scenes go through product commands only.

use crate::m5_agent_runtime::{RuntimeFault, SynNativeAgentRuntime, WorkcellRun};
use crate::m5_claim_ledger::{
    load_fact_by_claim, record_claim, record_result_decision, record_review,
};
use crate::m5_controlled_execution::{retry_operation, run_authorized_workcell};
use crate::m5_dto::{legacy_execution_manifest, M5GlobalAdviceFixture, M5SupervisorOpenRequest};
use crate::m5_orchestration_store::M5OrchestrationStore;
use crate::m5_product_commands::{
    derived_supervisor_session_id, open_project_supervisor_command, open_source_deep_link_command,
    read_project_summary_command, record_authorization_decision_command, supervisor_turn_command,
};
use crate::m5_project_summary::{rebuild_project_summary, SummaryConsumer};
use crate::m5_project_supervisor::PersistentRoleSessionPort;
use crate::worker_report::{ExecutionReceipt, M5WorkerReport, TrustedActor, WorkerReport};
use serde::Serialize;
use std::path::Path;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct IsolatedSceneResult {
    pub scene: String,
    pub passed: bool,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct IsolatedAcceptanceReport {
    pub schema: String,
    pub passed: bool,
    pub scenes: Vec<IsolatedSceneResult>,
    pub window_capture: String,
    pub legacy_paths_physically_deleted: bool,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct AuthorizedFollowthroughResult {
    pub claim_id: String,
    pub duplicate_claim_id: String,
    pub fact_project_id: String,
    pub review_id: String,
}

/// Post-dispatch isolated followthrough. Callers must already have gone through
/// AuthorizationDecision → Grant → Dispatch. This never calls prepare_and_dispatch.
pub(crate) fn run_authorized_followthrough(
    store: &M5OrchestrationStore,
    project_id: &str,
    grant_id: &str,
    dispatch_id: &str,
    actor_id: &str,
    now_ms: i64,
) -> Result<AuthorizedFollowthroughResult, String> {
    let grant = store.load_grant(grant_id)?.ok_or("missing_grant")?;
    if grant.project_id != project_id {
        return Err("followthrough_grant_project_mismatch".into());
    }
    let dispatch = store
        .load_dispatch(dispatch_id)?
        .ok_or("missing_dispatch")?;
    if dispatch.grant_id != grant_id || dispatch.project_id != project_id {
        return Err("followthrough_dispatch_join_failed".into());
    }
    let mut runtime = SynNativeAgentRuntime::new();
    let fail_cell = WorkcellRun {
        workcell_id: format!("wc-{project_id}-fail"),
        profile_digest: "profile:syn-native:v1".into(),
        session_ref: format!("rt-{project_id}"),
        parent_grant_id: grant.grant_id.as_str().into(),
        attempt_id: grant.attempt_id.as_str().into(),
        dispatch_id: dispatch.dispatch_id.clone(),
        effect_id: format!("{}-fail", dispatch.effect_id),
        actor_binding: grant.worker_role_session_id.clone(),
        command: "echo".into(),
        child_depth: 0,
        budget_tokens: 8,
        stop_conditions: vec!["max_tokens".into()],
        dynamic_package_enabled: false,
    };
    run_authorized_workcell(store, &mut runtime, &fail_cell, now_ms, RuntimeFault::Timeout)?;
    retry_operation(store, &format!("op-wc-{project_id}-fail"), now_ms + 100)?;
    let workcell = WorkcellRun {
        workcell_id: format!("wc-{project_id}"),
        profile_digest: "profile:syn-native:v1".into(),
        session_ref: format!("rt-{project_id}"),
        parent_grant_id: grant.grant_id.as_str().into(),
        attempt_id: grant.attempt_id.as_str().into(),
        dispatch_id: dispatch.dispatch_id.clone(),
        effect_id: dispatch.effect_id.clone(),
        actor_binding: grant.worker_role_session_id.clone(),
        command: "echo".into(),
        child_depth: 0,
        budget_tokens: 8,
        stop_conditions: vec!["max_tokens".into()],
        dynamic_package_enabled: false,
    };
    let receipt = run_authorized_workcell(
        store,
        &mut runtime,
        &workcell,
        now_ms + 500,
        RuntimeFault::None,
    )?;
    let report = M5WorkerReport::from_base(WorkerReport {
        status: "ok".into(),
        did: "echoed".into(),
        ..WorkerReport::default()
    })
    .as_execution(
        ExecutionReceipt {
            execution_id: receipt.receipt_id.as_str().into(),
            started_at_ms: now_ms + 500,
            completed_at_ms: Some(now_ms + 600),
            status: "SUCCEEDED".into(),
            exit_code: Some(0),
            output_hash: Some(receipt.trace_hash.clone()),
            cost_tokens: None,
        },
        TrustedActor {
            actor_id: actor_id.to_string(),
            role: "worker".into(),
            actor_type: "syn-native".into(),
            authentication_method: "role-session".into(),
        },
    )
    .bind_project(project_id, grant.orchestration_id.as_str())
    .bind_execution_join(
        grant.workflow_run_id.as_str(),
        grant.work_item_id.as_str(),
        &dispatch.node_id,
        &dispatch.dispatch_id,
        grant.attempt_id.as_str(),
        grant.grant_id.as_str(),
        &grant.worker_role_session_id,
        receipt.receipt_id.as_str(),
        &receipt.trace_hash,
    );
    let claim = record_claim(store, &report, Some(&receipt), now_ms + 1500)?;
    let review = record_review(
        store,
        &claim.claim_id,
        &format!("reviewer:{project_id}"),
        &format!("reviewer-session:{project_id}"),
        "VERIFIED",
        now_ms + 1600,
    )?;
    record_result_decision(
        store,
        &review.review_id,
        actor_id,
        "ACCEPTED_RESULT",
        None,
        now_ms + 1700,
    )?;
    let fact = load_fact_by_claim(store, &claim.claim_id)?.ok_or("missing_fact")?;
    rebuild_project_summary(store, project_id, now_ms + 2500)?;
    let duplicate = record_claim(store, &report, Some(&receipt), now_ms + 1800)?;
    Ok(AuthorizedFollowthroughResult {
        claim_id: claim.claim_id,
        duplicate_claim_id: duplicate.claim_id,
        fact_project_id: fact.project_id,
        review_id: review.review_id,
    })
}

pub(crate) fn run_isolated_acceptance(app_data: &Path) -> Result<IsolatedAcceptanceReport, String> {
    std::fs::create_dir_all(app_data).map_err(|e| e.to_string())?;
    let a = run_scene_a(&app_data.join("scratch-a.sqlite"))?;
    let b = run_scene_b(&app_data.join("scratch-b.sqlite"))?;
    Ok(IsolatedAcceptanceReport {
        schema: "syn.m5r07.isolated-acceptance.v2".into(),
        passed: a.passed && b.passed,
        scenes: vec![a, b],
        window_capture: "NOT_EXECUTED".into(),
        legacy_paths_physically_deleted: legacy_execution_manifest()
            .iter()
            .any(|m| m.physically_deleted),
    })
}

fn port<'a>(store: &'a M5OrchestrationStore) -> PersistentRoleSessionPort<'a> {
    PersistentRoleSessionPort::new(store)
}

fn open(
    store: &M5OrchestrationStore,
    project_id: &str,
    now_ms: i64,
) -> Result<crate::m5_dto::M5SupervisorOpenResponse, String> {
    open_project_supervisor_command(
        store,
        &port(store),
        M5SupervisorOpenRequest {
            project_id: project_id.into(),
            role_session_id: String::new(),
        },
        now_ms,
    )
}

fn run_scene_a(path: &Path) -> Result<IsolatedSceneResult, String> {
    let store = M5OrchestrationStore::open(path)?;
    let opened = open(&store, "scratch-a", 1000)?;
    assert_eq!(
        opened.role_session_id,
        derived_supervisor_session_id("scratch-a")
    );
    let chat = supervisor_turn_command(
        &store,
        &opened.project_id,
        &opened.binding_id,
        crate::m5_dto::M5SupervisorTurnRequest {
            binding_id: opened.binding_id.clone(),
            project_id: "scratch-a".into(),
            kind: "chat".into(),
            text: "what is open?".into(),
        },
        1100,
    )?;
    let proposal = supervisor_turn_command(
        &store,
        &opened.project_id,
        &opened.binding_id,
        crate::m5_dto::M5SupervisorTurnRequest {
            binding_id: opened.binding_id.clone(),
            project_id: "scratch-a".into(),
            kind: "submit_proposal".into(),
            text: "do not run".into(),
        },
        1200,
    )?;
    let rejected = record_authorization_decision_command(
        &store,
        &opened.binding_id,
        "scratch-a",
        &proposal.text,
        "REJECTED",
        None,
    )?;
    let grants: i64 = store
        .connection()
        .query_row("SELECT COUNT(*) FROM m5_execution_grants", [], |row| {
            row.get(0)
        })
        .unwrap_or(0);
    let invented = open_project_supervisor_command(
        &store,
        &port(&store),
        M5SupervisorOpenRequest {
            project_id: "scratch-a".into(),
            role_session_id: "invented-session".into(),
        },
        1300,
    )
    .err();
    Ok(IsolatedSceneResult {
        scene: "scratch-a-readonly-and-user-reject".into(),
        passed: !chat.created_grant
            && !chat.spawned
            && proposal.created_proposal
            && rejected.is_none()
            && grants == 0
            && invented.as_deref() == Some("caller_invented_role_session_rejected"),
        notes: vec![
            "chat_zero_spawn".into(),
            "user_rejected_authorization_zero_grant".into(),
            "invented_role_session_rejected".into(),
        ],
    })
}

fn run_scene_b(path: &Path) -> Result<IsolatedSceneResult, String> {
    let store = M5OrchestrationStore::open(path)?;
    let opened = open(&store, "scratch-b", 1000)?;
    let proposal = supervisor_turn_command(
        &store,
        &opened.project_id,
        &opened.binding_id,
        crate::m5_dto::M5SupervisorTurnRequest {
            binding_id: opened.binding_id.clone(),
            project_id: "scratch-b".into(),
            kind: "submit_proposal".into(),
            text: "echo hello".into(),
        },
        1100,
    )?;
    let binding =
        crate::m5_project_supervisor::load_binding_by_id(&store, &opened.binding_id, "scratch-b")?;
    let chain = record_authorization_decision_command(
        &store,
        &opened.binding_id,
        "scratch-b",
        &proposal.text,
        "APPROVED",
        Some(
            crate::m5_orchestration_service::AuthorizedExecutionRequest {
                project_id: "scratch-b".into(),
                proposal_id: proposal.text.clone(),
                deciding_actor_id: binding.actor_id.clone(),
                worker_role_session_id: "m5:worker:scratch-b".into(),
                principal_actor_id: binding.actor_id.clone(),
                workflow_ref: "m5:workflow:scratch-b".into(),
                source_object_ref: "obj:scratch-b".into(),
                allowed_commands: vec!["echo".into()],
                cwd_ref: "/tmp/scratch-b".into(),
                write_root_refs: vec!["/tmp/scratch-b".into()],
                object_refs: vec!["obj:scratch-b".into()],
                scope_fingerprint: "scope-b".into(),
                policy_decision_ref: "pol-b".into(),
                now_ms: 2000,
                ttl_ms: 60_000,
            },
        ),
    )?
    .ok_or("expected_dispatch")?;
    let follow = run_authorized_followthrough(
        &store,
        "scratch-b",
        chain.grant_id.as_ref().ok_or("missing_grant")?.as_str(),
        chain.dispatch_id.as_ref().ok_or("missing_dispatch")?.as_str(),
        &binding.actor_id,
        2500,
    )?;
    let consumer = SummaryConsumer {
        role_session_id: derived_supervisor_session_id("scratch-b"),
        role: "project_supervisor".into(),
        scope_project_id: "scratch-b".into(),
        expires_at_ms: 90_000,
    };
    let summary = read_project_summary_command(&store, &consumer, 5100)?;
    let stale = read_project_summary_command(&store, &consumer, 80_000)?;
    let deep_link =
        open_source_deep_link_command(&store, &consumer, &summary.source_refs[0].source_id, 5200)?;
    let advice = M5GlobalAdviceFixture::frozen("scratch-b");
    let passed = follow.fact_project_id == "scratch-b"
        && follow.duplicate_claim_id == follow.claim_id
        && !summary.source_refs.is_empty()
        && stale.stale
        && !summary.stale
        && deep_link.starts_with("syn://project/scratch-b/")
        && !advice.writable;
    Ok(IsolatedSceneResult {
        scene: "scratch-b-authorized-echo-review-summary".into(),
        passed,
        notes: vec![
            "authorization_decision_then_grant".into(),
            "timeout_then_retry".into(),
            "independent_review".into(),
            "summary_source_refs_and_stale".into(),
            format!("deep_link={deep_link}"),
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isolated_two_scratch_scenes_pass() {
        let dir = std::env::temp_dir().join(format!("m5r07-{}", uuid::Uuid::new_v4()));
        let report = run_isolated_acceptance(&dir).unwrap();
        assert!(report.passed, "{report:?}");
        assert_eq!(report.scenes.len(), 2);
        assert!(!report.legacy_paths_physically_deleted);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
