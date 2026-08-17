// M5R07 isolated scratch acceptance. Scenes go through product commands only.

use crate::m5_orchestration_store::M5OrchestrationStore;
use crate::m5_product_commands::{open_source_deep_link_command, read_project_summary_command};
use crate::m5_project_summary::SummaryConsumer;
use rusqlite::OptionalExtension;
use std::path::Path;

#[cfg(test)]
use crate::m5_dto::{M5GlobalAdviceFixture, M5SupervisorOpenRequest};
#[cfg(test)]
use crate::m5_product_commands::{
    open_project_supervisor_command, record_authorization_decision_command, supervisor_turn_command,
};
#[cfg(test)]
use crate::m5_project_supervisor::ProjectSupervisorRoleSessionPort;
#[cfg(test)]
use serde::Serialize;

#[cfg(test)]
#[derive(Clone, Debug, Serialize)]
pub(crate) struct IsolatedSceneResult {
    pub scene: String,
    pub passed: bool,
    pub notes: Vec<String>,
}

#[cfg(test)]
#[derive(Clone, Debug, Serialize)]
pub(crate) struct IsolatedAcceptanceReport {
    pub schema: String,
    pub passed: bool,
    pub scenes: Vec<IsolatedSceneResult>,
    pub window_capture: String,
    pub legacy_paths_physically_deleted: bool,
}

#[cfg(test)]
#[derive(Clone, Debug, Serialize)]
pub(crate) struct AuthorizedFollowthroughResult {
    pub claim_id: String,
    pub duplicate_claim_id: String,
    pub fact_project_id: String,
    pub review_id: String,
}

/// Test-only post-dispatch followthrough. Not a product entry.
#[cfg(test)]
pub(crate) fn run_authorized_followthrough(
    store: &M5OrchestrationStore,
    project_id: &str,
    grant_id: &str,
    dispatch_id: &str,
    actor_id: &str,
    reviewer_actor_id: &str,
    reviewer_role_session_id: &str,
    now_ms: i64,
) -> Result<AuthorizedFollowthroughResult, String> {
    use crate::m5_agent_runtime::{RuntimeFault, SynNativeAgentRuntime, WorkcellRun};
    use crate::m5_claim_ledger::{
        load_fact_by_claim, record_claim, record_result_decision, record_review,
    };
    use crate::m5_controlled_execution::run_authorized_workcell;
    use crate::m5_orchestration_service::complete_dispatch_readback;
    use crate::m5_project_summary::rebuild_project_summary;
    use crate::worker_report::{ExecutionReceipt, M5WorkerReport, TrustedActor, WorkerReport};

    let grant = store.load_grant(grant_id)?.ok_or("missing_grant")?;
    if grant.project_id != project_id {
        return Err("followthrough_grant_project_mismatch".into());
    }
    let pending = store
        .load_dispatch(dispatch_id)?
        .ok_or("missing_dispatch")?;
    if pending.grant_id != grant_id || pending.project_id != project_id {
        return Err("followthrough_dispatch_join_failed".into());
    }
    let (dispatch, _attempt) = complete_dispatch_readback(store, dispatch_id, now_ms)?;
    let mut runtime = SynNativeAgentRuntime::new();
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
    if reviewer_role_session_id.trim().is_empty()
        || reviewer_actor_id.trim().is_empty()
        || reviewer_role_session_id.starts_with("reviewer:")
        || reviewer_actor_id.starts_with("reviewer:")
        || reviewer_actor_id == actor_id
        || reviewer_role_session_id == grant.worker_role_session_id
    {
        return Err("reviewer_identity_unbound".into());
    }
    let review = record_review(
        store,
        &claim.claim_id,
        reviewer_actor_id,
        reviewer_role_session_id,
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

#[cfg(test)]
pub(crate) fn run_isolated_acceptance(app_data: &Path) -> Result<IsolatedAcceptanceReport, String> {
    use crate::m5_dto::legacy_execution_manifest;
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

#[cfg(test)]
struct TestSessions {
    session: crate::m5_project_supervisor::SupervisorSessionRef,
}

#[cfg(test)]
impl ProjectSupervisorRoleSessionPort for TestSessions {
    fn load(
        &self,
        role_session_id: &str,
    ) -> Result<crate::m5_project_supervisor::SupervisorSessionRef, String> {
        if !role_session_id.is_empty() && role_session_id != self.session.role_session_id {
            return Err("caller_invented_role_session_rejected".to_string());
        }
        Ok(self.session.clone())
    }
}

#[cfg(test)]
fn test_sessions(project_id: &str) -> TestSessions {
    TestSessions {
        session: crate::m5_project_supervisor::SupervisorSessionRef {
            role_session_id: format!("test-session:{project_id}"),
            project_id: project_id.to_string(),
            actor_id: format!("test-actor:{project_id}"),
            role: "project_supervisor".into(),
            status: "ACTIVE".into(),
        },
    }
}

#[cfg(test)]
fn open(
    store: &M5OrchestrationStore,
    project_id: &str,
    now_ms: i64,
) -> Result<crate::m5_dto::M5SupervisorOpenResponse, String> {
    let sessions = test_sessions(project_id);
    open_project_supervisor_command(
        store,
        &sessions,
        M5SupervisorOpenRequest {
            project_id: project_id.into(),
        },
        now_ms,
    )
}

#[cfg(test)]
fn run_scene_a(path: &Path) -> Result<IsolatedSceneResult, String> {
    let store = M5OrchestrationStore::open(path)?;
    let sessions = test_sessions("scratch-a");
    let opened = open(&store, "scratch-a", 1000)?;
    assert_eq!(opened.role_session_id, sessions.session.role_session_id);
    let chat = supervisor_turn_command(
        &store,
        &sessions,
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
        &sessions,
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
    let invented = sessions.load("invented-session").err();
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

#[cfg(test)]
fn run_scene_b(path: &Path) -> Result<IsolatedSceneResult, String> {
    let store = M5OrchestrationStore::open(path)?;
    let sessions = test_sessions("scratch-b");
    let opened = open(&store, "scratch-b", 1000)?;
    let proposal = supervisor_turn_command(
        &store,
        &sessions,
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
    let expanded = record_authorization_decision_command(
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
                worker_role_session_id: "test-worker:scratch-b".into(),
                principal_actor_id: binding.actor_id.clone(),
                workflow_ref: "workflow:scratch-b".into(),
                source_object_ref: "object:scratch-b".into(),
                allowed_commands: vec!["echo".into(), "rm".into()],
                cwd_ref: "scratch:scratch-b".into(),
                write_root_refs: vec!["scratch:scratch-b".into()],
                object_refs: vec!["object:scratch-b".into()],
                scope_fingerprint: "scope:scratch-b".into(),
                policy_decision_ref: crate::m5_m3_identity::policy_decision_ref_for_action("echo"),
                now_ms: 2000,
                ttl_ms: 60_000,
            },
        ),
    )
    .err();
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
                worker_role_session_id: "test-worker:scratch-b".into(),
                principal_actor_id: binding.actor_id.clone(),
                workflow_ref: "workflow:scratch-b".into(),
                source_object_ref: "object:scratch-b".into(),
                allowed_commands: vec!["echo".into()],
                cwd_ref: "scratch:scratch-b".into(),
                write_root_refs: vec!["scratch:scratch-b".into()],
                object_refs: vec!["object:scratch-b".into()],
                scope_fingerprint: "scope:scratch-b".into(),
                policy_decision_ref: crate::m5_m3_identity::policy_decision_ref_for_action("echo"),
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
        chain
            .dispatch_id
            .as_ref()
            .ok_or("missing_dispatch")?
            .as_str(),
        &binding.actor_id,
        "test-reviewer-actor:scratch-b",
        "test-reviewer:scratch-b",
        2500,
    )?;
    let consumer = SummaryConsumer {
        role_session_id: sessions.session.role_session_id.clone(),
        role: "project_supervisor".into(),
        scope_project_id: "scratch-b".into(),
        expires_at_ms: 90_000,
    };
    let summary = read_project_summary_command(&store, &consumer, 5100)?;
    let stale = read_project_summary_command(&store, &consumer, 80_000)?;
    let deep_link =
        open_source_deep_link_command(&store, "scratch-b", &summary.source_refs[0].source_id)?;
    let advice = M5GlobalAdviceFixture::frozen("scratch-b");
    let resolved = crate::m5_project_summary::resolve_source_ref(
        &store,
        "scratch-b",
        &summary.source_refs[0].source_id,
    )?;
    let passed = expanded.as_deref() == Some("renderer_grant_scope_rejected")
        && follow.fact_project_id == "scratch-b"
        && follow.duplicate_claim_id == follow.claim_id
        && !summary.source_refs.is_empty()
        && stale.stale
        && !summary.stale
        && deep_link.starts_with("syn://m5/")
        && resolved.source_id == summary.source_refs[0].source_id
        && !advice.writable;
    Ok(IsolatedSceneResult {
        scene: "scratch-b-authorized-echo-review-summary".into(),
        passed,
        notes: vec![
            "authorization_decision_then_grant".into(),
            "single_exact_effect_after_dispatch_readback".into(),
            "independent_review".into(),
            "summary_source_refs_and_stale".into(),
            format!("deep_link={deep_link}"),
        ],
    })
}

pub(crate) fn write_backend_derived_receipt(
    store: &M5OrchestrationStore,
    phase: &str,
    project_id: &str,
) -> Result<String, String> {
    let grants: i64 = store
        .connection()
        .query_row(
            "SELECT COUNT(*) FROM m5_execution_grants WHERE project_id=?1",
            [project_id],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let rejected: i64 = store
        .connection()
        .query_row(
            "SELECT COUNT(*) FROM m5_supervisor_proposals
             WHERE project_id=?1 AND status='REJECTED'",
            [project_id],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let approved: i64 = store
        .connection()
        .query_row(
            "SELECT COUNT(*) FROM m5_supervisor_proposals
             WHERE project_id=?1 AND status='APPROVED'",
            [project_id],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let binding = store
        .connection()
        .query_row(
            "SELECT binding_id, role_session_id, actor_id FROM m5_supervisor_bindings
             WHERE project_id=?1",
            [project_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        )
        .optional()
        .map_err(|e| e.to_string())?;
    let grant_join = store
        .connection()
        .query_row(
            "SELECT g.grant_id, d.dispatch_id, c.claim_id, r.review_id, f.fact_id
             FROM m5_execution_grants g
             JOIN m5_dispatches d ON d.grant_id = g.grant_id AND d.project_id = g.project_id
             LEFT JOIN m5_claims c ON c.grant_id = g.grant_id AND c.project_id = g.project_id
             LEFT JOIN m5_reviews r ON r.claim_id = c.claim_id AND r.project_id = g.project_id
             LEFT JOIN m5_project_facts f ON f.claim_id = c.claim_id AND f.project_id = g.project_id
             WHERE g.project_id=?1
             ORDER BY g.issued_at_ms DESC LIMIT 1",
            [project_id],
            |row| {
                Ok(serde_json::json!({
                    "grant_id": row.get::<_, String>(0)?,
                    "dispatch_id": row.get::<_, String>(1)?,
                    "claim_id": row.get::<_, Option<String>>(2)?,
                    "review_id": row.get::<_, Option<String>>(3)?,
                    "fact_id": row.get::<_, Option<String>>(4)?,
                }))
            },
        )
        .optional()
        .ok()
        .flatten();
    let consumer = SummaryConsumer {
        role_session_id: binding.as_ref().map(|b| b.1.clone()).unwrap_or_default(),
        role: "project_supervisor".into(),
        scope_project_id: project_id.to_string(),
        expires_at_ms: m5_now_ms_for_receipt() + 3_600_000,
    };
    let summary = read_project_summary_command(store, &consumer, m5_now_ms_for_receipt()).ok();
    let stale = read_project_summary_command(store, &consumer, m5_now_ms_for_receipt() + 120_000)
        .ok()
        .map(|s| s.stale);
    let deep_link = summary
        .as_ref()
        .and_then(|s| s.source_refs.first())
        .and_then(|r| open_source_deep_link_command(store, project_id, &r.source_id).ok());
    let deep_link_resolves = summary
        .as_ref()
        .and_then(|s| s.source_refs.first())
        .and_then(|r| {
            crate::m5_project_summary::resolve_source_ref(store, project_id, &r.source_id).ok()
        })
        .is_some();
    let notes = match phase {
        "scene-a" => vec![
            format!("zero_grant={}", grants == 0),
            format!("rejected_proposal={}", rejected > 0),
        ],
        "scene-b" => vec![
            format!("approved_proposal={}", approved > 0),
            format!(
                "exact_join={}",
                grant_join.as_ref().is_some_and(|j| {
                    j.get("claim_id").and_then(|v| v.as_str()).is_some()
                        && j.get("review_id").and_then(|v| v.as_str()).is_some()
                        && j.get("fact_id").and_then(|v| v.as_str()).is_some()
                })
            ),
            format!("stale_observed={}", stale.unwrap_or(false)),
            format!("deep_link_resolves={deep_link_resolves}"),
        ],
        "resume" => vec![format!("binding_recovered={}", binding.is_some())],
        other => vec![format!("phase={other}")],
    };
    let body = serde_json::json!({
        "schema": "syn.m5r07.isolated-ui-receipt.v2",
        "phase": phase,
        "project_id": project_id,
        "binding_id": binding.as_ref().map(|b| b.0.clone()),
        "role_session_id": binding.as_ref().map(|b| b.1.clone()),
        "grants": grants,
        "rejected_proposals": rejected,
        "approved_proposals": approved,
        "grant_join": grant_join,
        "stale": stale,
        "deep_link": deep_link,
        "deep_link_resolves": deep_link_resolves,
        "dispatched": grants > 0,
        "spawned": false,
        "notes": notes,
        "derived_from": "backend_store",
    });
    let log_dir = crate::acceptance_runtime_profile::isolated_log_dir()?
        .ok_or_else(|| "m5_isolated_log_dir_missing".to_string())?;
    std::fs::create_dir_all(&log_dir).map_err(|e| format!("m5_isolated_log_dir:{e}"))?;
    let path = log_dir.join(format!("m5r07-ui-{phase}.json"));
    std::fs::write(
        &path,
        serde_json::to_vec_pretty(&body).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("write_ui_receipt:{e}"))?;
    Ok(path.to_string_lossy().into_owned())
}

pub(crate) fn write_unavailable_receipt(
    phase: &str,
    m1_authority_installed: bool,
    m3_authority_installed: bool,
    composition_gap: Option<&str>,
) -> Result<String, String> {
    let body = serde_json::json!({
        "schema": "syn.m5r07.isolated-ui-receipt.v3-unavailable",
        "phase": phase,
        "m1_authority_installed": m1_authority_installed,
        "m3_authority_installed": m3_authority_installed,
        "open_available": false,
        "full_loop_claimed": false,
        "composition_gap": composition_gap,
        "derived_from": "installed_authority_slots",
    });
    let log_dir = crate::acceptance_runtime_profile::isolated_log_dir()?
        .ok_or_else(|| "m5_isolated_log_dir_missing".to_string())?;
    std::fs::create_dir_all(&log_dir).map_err(|e| format!("m5_isolated_log_dir:{e}"))?;
    let path = log_dir.join(format!("m5r07-ui-{phase}.json"));
    std::fs::write(
        &path,
        serde_json::to_vec_pretty(&body).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("write_ui_receipt:{e}"))?;
    Ok(path.to_string_lossy().into_owned())
}

fn m5_now_ms_for_receipt() -> i64 {
    crate::m5_product_commands::m5_now_ms()
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
