// M5R07 isolated scratch acceptance. Uses isolated app-data directories and
// fake roles/runtime only. Does not launch a desktop window.

use crate::m5_agent_runtime::{RuntimeFault, SynNativeAgentRuntime, WorkcellRun};
use crate::m5_claim_ledger::{
    load_fact_by_claim, record_claim, record_result_decision, record_review,
};
use crate::m5_controlled_execution::run_authorized_workcell;
use crate::m5_dto::{legacy_execution_manifest, M5GlobalAdviceFixture};
use crate::m5_orchestration_service::{
    prepare_and_dispatch, AuthorizedExecutionRequest, ChainFault,
};
use crate::m5_orchestration_store::M5OrchestrationStore;
use crate::m5_product_commands::{
    open_project_supervisor_command, read_project_summary_command, static_supervisor_session,
    supervisor_turn_command,
};
use crate::m5_project_summary::SummaryConsumer;
use crate::m5_project_supervisor::{
    approve_supervisor_proposal, ProjectSupervisorRoleSessionPort, SupervisorSessionRef,
};
use crate::m5_runtime_receipt::RuntimeReceipt;
use crate::worker_report::{ExecutionReceipt, M5WorkerReport, TrustedActor, WorkerReport};
use serde::Serialize;
use std::collections::HashMap;
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

struct MapSessions(HashMap<String, SupervisorSessionRef>);

impl ProjectSupervisorRoleSessionPort for MapSessions {
    fn load(&self, role_session_id: &str) -> Result<SupervisorSessionRef, String> {
        self.0
            .get(role_session_id)
            .cloned()
            .ok_or_else(|| "missing_session".into())
    }
}

pub(crate) fn run_isolated_acceptance(app_data: &Path) -> Result<IsolatedAcceptanceReport, String> {
    std::fs::create_dir_all(app_data).map_err(|e| e.to_string())?;
    let a = run_scene_a(&app_data.join("scratch-a.sqlite"))?;
    let b = run_scene_b(&app_data.join("scratch-b.sqlite"))?;
    let passed = a.passed && b.passed;
    Ok(IsolatedAcceptanceReport {
        schema: "syn.m5r07.isolated-acceptance.v1".into(),
        passed,
        scenes: vec![a, b],
        window_capture: "NOT_EXECUTED".into(),
        legacy_paths_physically_deleted: legacy_execution_manifest()
            .iter()
            .any(|m| m.physically_deleted),
    })
}

fn run_scene_a(path: &Path) -> Result<IsolatedSceneResult, String> {
    let store = M5OrchestrationStore::open(path)?;
    let sessions = MapSessions(HashMap::from([(
        "rs-a".into(),
        static_supervisor_session("rs-a", "scratch-a", "actor-a"),
    )]));
    let open = open_project_supervisor_command(
        &store,
        &sessions,
        crate::m5_dto::M5SupervisorOpenRequest {
            project_id: "scratch-a".into(),
            role_session_id: "rs-a".into(),
        },
        1000,
    )?;
    let chat = supervisor_turn_command(
        &store,
        &open.project_id,
        &open.binding_id,
        crate::m5_dto::M5SupervisorTurnRequest {
            binding_id: open.binding_id.clone(),
            project_id: "scratch-a".into(),
            kind: "chat".into(),
            text: "what is open?".into(),
        },
        1100,
    )?;
    let proposal = supervisor_turn_command(
        &store,
        &open.project_id,
        &open.binding_id,
        crate::m5_dto::M5SupervisorTurnRequest {
            binding_id: open.binding_id.clone(),
            project_id: "scratch-a".into(),
            kind: "submit_proposal".into(),
            text: "do not run".into(),
        },
        1200,
    )?;
    let grants: i64 = store
        .connection()
        .query_row("SELECT COUNT(*) FROM m5_execution_grants", [], |row| {
            row.get(0)
        })
        .unwrap_or(0);
    let passed = !chat.created_grant && !chat.spawned && proposal.created_proposal && grants == 0;
    Ok(IsolatedSceneResult {
        scene: "scratch-a-readonly-and-user-reject".into(),
        passed,
        notes: vec![
            "chat_zero_spawn".into(),
            "proposal_left_draft_no_grant".into(),
        ],
    })
}

fn run_scene_b(path: &Path) -> Result<IsolatedSceneResult, String> {
    let store = M5OrchestrationStore::open(path)?;
    let sessions = MapSessions(HashMap::from([(
        "rs-b".into(),
        static_supervisor_session("rs-b", "scratch-b", "actor-b"),
    )]));
    let open = open_project_supervisor_command(
        &store,
        &sessions,
        crate::m5_dto::M5SupervisorOpenRequest {
            project_id: "scratch-b".into(),
            role_session_id: "rs-b".into(),
        },
        1000,
    )?;
    let proposal = supervisor_turn_command(
        &store,
        &open.project_id,
        &open.binding_id,
        crate::m5_dto::M5SupervisorTurnRequest {
            binding_id: open.binding_id.clone(),
            project_id: "scratch-b".into(),
            kind: "submit_proposal".into(),
            text: "echo hello".into(),
        },
        1100,
    )?;
    let binding = crate::m5_project_supervisor::SupervisorBinding {
        binding_id: open.binding_id.clone(),
        project_id: "scratch-b".into(),
        role_session_id: "rs-b".into(),
        actor_id: "actor-b".into(),
    };
    approve_supervisor_proposal(&store, &binding, &proposal.text)?;
    let chain = prepare_and_dispatch(
        &store,
        AuthorizedExecutionRequest {
            project_id: "scratch-b".into(),
            proposal_id: proposal.text.clone(),
            deciding_actor_id: "actor-b".into(),
            worker_role_session_id: "worker-b".into(),
            principal_actor_id: "actor-b".into(),
            workflow_ref: "wf-scratch-b".into(),
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
        ChainFault::None,
    )?;
    let grant = store
        .load_grant(chain.grant_id.as_ref().unwrap().as_str())?
        .ok_or("missing_grant")?;
    let dispatch = store
        .load_dispatch(chain.dispatch_id.as_ref().unwrap().as_str())?
        .ok_or("missing_dispatch")?;
    let workcell = WorkcellRun {
        workcell_id: "wc-scratch-b".into(),
        profile_digest: "profile:syn-native:v1".into(),
        session_ref: "rt-scratch-b".into(),
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
    let mut runtime = SynNativeAgentRuntime::new();
    let receipt: RuntimeReceipt =
        run_authorized_workcell(&store, &mut runtime, &workcell, 3000, RuntimeFault::None)?;
    let report = M5WorkerReport::from_base(WorkerReport {
        status: "ok".into(),
        did: "echoed".into(),
        ..WorkerReport::default()
    })
    .as_execution(
        ExecutionReceipt {
            execution_id: receipt.receipt_id.as_str().into(),
            started_at_ms: 3000,
            completed_at_ms: Some(3100),
            status: "SUCCEEDED".into(),
            exit_code: Some(0),
            output_hash: Some(receipt.trace_hash.clone()),
            cost_tokens: None,
        },
        TrustedActor {
            actor_id: "actor-b".into(),
            role: "worker".into(),
            actor_type: "syn-native".into(),
            authentication_method: "role-session".into(),
        },
    )
    .bind_project("scratch-b", grant.orchestration_id.as_str())
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
    let claim = record_claim(&store, &report, Some(&receipt), 4000)?;
    let review = record_review(
        &store,
        &claim.claim_id,
        "reviewer-b",
        "reviewer-session-b",
        "VERIFIED",
        4100,
    )?;
    record_result_decision(
        &store,
        &review.review_id,
        "actor-b",
        "ACCEPTED_RESULT",
        None,
        4200,
    )?;
    let fact = load_fact_by_claim(&store, &claim.claim_id)?.ok_or("missing_fact")?;
    let summary = read_project_summary_command(
        &store,
        &SummaryConsumer {
            role_session_id: "rs-secretary".into(),
            role: "secretary".into(),
            scope_project_id: "scratch-b".into(),
            expires_at_ms: 90_000,
        },
        5000,
    )?;
    let advice = M5GlobalAdviceFixture::frozen("scratch-b");
    let duplicate = record_claim(&store, &report, Some(&receipt), 4300)?;
    let passed = fact.project_id == "scratch-b"
        && summary.project_id == "scratch-b"
        && !advice.writable
        && duplicate.claim_id == claim.claim_id
        && receipt.outcome == "SUCCEEDED";
    Ok(IsolatedSceneResult {
        scene: "scratch-b-authorized-echo-review-summary".into(),
        passed,
        notes: vec![
            "whitelist_echo".into(),
            "independent_review".into(),
            "result_decision".into(),
            "summary_and_readonly_advice".into(),
            format!("deep_link={}", advice.source_ref),
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
        assert_eq!(report.window_capture, "NOT_EXECUTED");
        assert!(!report.legacy_paths_physically_deleted);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
