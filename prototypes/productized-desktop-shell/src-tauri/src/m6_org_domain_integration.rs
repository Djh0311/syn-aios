//! M6D08 domain-layer integration regression.
//!
//! This test module exercises the ordinary composition with synthetic M5
//! carriers and fake M3 participants.  It deliberately excludes the M3 and M6
//! coordination databases from its write-spy; every other file under the
//! isolated fixture, including M5 and explicit project-side surfaces, must be
//! byte-for-byte unchanged.

use crate::m5_project_summary::{ensure_summary_schema, ProjectSummary, SourceRef};
use crate::m6_org_consult_handoff::bind_fake_secretary_for_member_directory;
use crate::m6_org_cross_project_advisory::{
    adopt_for_state, policy_allow_ref_for, project_owner_ref_for, summary_id_for,
};
use crate::m6_org_dto::{
    M6OrgAdvisoryAdoptionRequest, M6OrgConsultDecision,
    M6OrgGlobalSupervisorConsultDecisionRequest, M6OrgProjectSummaryQueryInput,
    M6OrgSecretaryConsultStartRequest,
};
use crate::m6_org_member_directory::{
    contact_for_state, export_for_state, list_for_state, observe_availability_for_state,
    register_for_state, update_for_state, M6OrgAvailabilityState, M6OrgAvailabilityTtl,
    M6OrgCapabilityPermissionKind, M6OrgCapabilityPermissionRef, M6OrgContactStableMemberRequest,
    M6OrgHeuristicCandidateKind, M6OrgListStableMembersRequest, M6OrgMemberContactBinding,
    M6OrgMemberRegistrationDisposition, M6OrgObserveMemberAvailabilityRequest,
    M6OrgRegisterStableMemberRequest, M6OrgRoleAssignment, M6OrgScopeAssignment,
    M6OrgStableIdentityEvidence, M6OrgUpdateStableMemberRequest,
};
use crate::m6_org_multi_view_consultation::{
    assemble_for_state, start_for_state, submit_view_for_state,
    M6OrgAssembleMultiViewConsultationRequest, M6OrgConsultationClaim,
    M6OrgConsultationClaimPosition, M6OrgConsultationEscalationTrigger,
    M6OrgConsultationResultState, M6OrgConsultationRouteResponse, M6OrgConsultationViewKind,
    M6OrgMultiViewConsultation, M6OrgStartMultiViewConsultationRequest,
    M6OrgSubmitConsultationViewRequest,
};
use crate::m6_org_temporary_agent_projection::tests::{fixture, seed_execution};
use crate::m6_org_temporary_agent_projection::{
    refresh_for_state, search_for_state, M6OrgSearchTemporaryAgentHistoryRequest,
    M6OrgTemporaryAgentSourceState,
};
use rusqlite::params;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const NOW_MS: i64 = 1_787_097_600_000;
const ACCEPT_BY: &str = "2026-08-19T00:10:00.000Z";
const RETURN_BY: &str = "2026-08-19T00:20:00.000Z";
const LEGACY_PROCESS_FACT_MARKER: &str =
    "legacy-process-fact:must-never-enter-m6-query-inputs:m6d08";

fn sealed_ref(namespace: &str, material: &str) -> String {
    format!(
        "{namespace}:sha256:{:x}",
        Sha256::digest(material.as_bytes())
    )
}

fn summary(
    project_id: &str,
    orchestration_id: &str,
    watermark_ms: i64,
    fact_count: u32,
) -> ProjectSummary {
    ProjectSummary {
        project_id: project_id.to_string(),
        orchestration_id: orchestration_id.to_string(),
        schema_version: "m5.project-summary.v1".to_string(),
        version: 1,
        watermark_ms,
        summary_hash: format!(
            "{:x}",
            Sha256::digest(
                format!("{project_id}:{orchestration_id}:{watermark_ms}:{fact_count}").as_bytes()
            )
        ),
        source_refs: vec![SourceRef {
            source_type: "project_fact".to_string(),
            source_id: format!("fact-source:{project_id}"),
            last_updated_ms: watermark_ms,
        }],
        fact_count,
        unverified_claim_count: 1,
        open_run_count: 1,
        rebuilt_at_ms: watermark_ms,
    }
}

fn seed_summaries(state: &crate::AppState, summaries: &[ProjectSummary]) {
    let store = state.open_m5_store().expect("open true-shaped M5 store");
    ensure_summary_schema(&store).expect("ensure M5 ProjectSummary schema");
    for summary in summaries {
        store
            .connection()
            .execute(
                "INSERT OR REPLACE INTO m5_project_summaries (
                    project_id,orchestration_id,schema_version,version,watermark_ms,
                    summary_hash,source_refs_json,fact_count,unverified_claim_count,
                    open_run_count,rebuilt_at_ms
                 ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
                params![
                    summary.project_id,
                    summary.orchestration_id,
                    summary.schema_version,
                    summary.version as i64,
                    summary.watermark_ms,
                    summary.summary_hash,
                    serde_json::to_string(&summary.source_refs).expect("encode source refs"),
                    summary.fact_count,
                    summary.unverified_claim_count,
                    summary.open_run_count,
                    summary.rebuilt_at_ms,
                ],
            )
            .expect("seed ProjectSummary");
    }
}

fn query(summary: &ProjectSummary) -> M6OrgProjectSummaryQueryInput {
    let owner_ref = project_owner_ref_for(&summary.project_id);
    M6OrgProjectSummaryQueryInput {
        summary_id: Some(summary_id_for(&summary.project_id)),
        project_id: summary.project_id.clone(),
        project_owner_ref: Some(owner_ref.clone()),
        policy_decision_ref: Some(policy_allow_ref_for(&summary.project_id, &owner_ref)),
        expected_schema_version: Some(summary.schema_version.clone()),
        expected_version: Some(summary.version),
        expected_source_watermark: Some(summary.watermark_ms),
        expected_summary_hash: Some(summary.summary_hash.clone()),
    }
}

fn stable_member_request(member_id: &str) -> M6OrgRegisterStableMemberRequest {
    M6OrgRegisterStableMemberRequest {
        member_id: member_id.to_string(),
        display_name_ref: "display-name:m6d08-stable-member".to_string(),
        identity_evidence: M6OrgStableIdentityEvidence::ExplicitIdentityContract {
            contract_kind: "syn.m6.org.stable-member-identity/v1".to_string(),
            identity_contract_ref: format!("identity-contract:{member_id}"),
            source_record_ref: format!("identity-source:{member_id}"),
            source_revision: 1,
            observed_at: NOW_MS,
            explicit_human_command: true,
        },
        scope_assignments: vec![M6OrgScopeAssignment {
            assignment_id: format!("scope-assignment:{member_id}:global"),
            member_id: member_id.to_string(),
            scope_ref: "scope:global".to_string(),
            assigned_by_actor_id: "actor:user".to_string(),
            revision: 1,
            assigned_at: NOW_MS,
            revoked_at: None,
        }],
        role_assignments: vec![M6OrgRoleAssignment {
            assignment_id: format!("role-assignment:{member_id}:consultant"),
            member_id: member_id.to_string(),
            role_ref: "role:consultant".to_string(),
            scope_ref: "scope:global".to_string(),
            assigned_by_actor_id: "actor:user".to_string(),
            revision: 1,
            assigned_at: NOW_MS,
            revoked_at: None,
        }],
        capability_permission_refs: vec![M6OrgCapabilityPermissionRef {
            ref_id: "capability:research".to_string(),
            subject_member_id: member_id.to_string(),
            kind: M6OrgCapabilityPermissionKind::Capability,
            source: "policy-owner:m6d08-fixture".to_string(),
            revision: 1,
            observed_at: NOW_MS,
            directory_is_authority: false,
            read_only: true,
        }],
        memory_refs: vec![format!("memory-ref:{member_id}:profile")],
        contact_bindings: vec![M6OrgMemberContactBinding {
            binding_ref: format!("contact-binding:{member_id}"),
            to_role_ref: sealed_ref("role", "m6d08/stable-member/consultant"),
            to_recipient_ref: sealed_ref("actor", "m6d08/stable-member"),
            source: "syn.m6d08.explicit-contact-binding/v1".to_string(),
            revision: 1,
            observed_at: NOW_MS,
        }],
        idempotency_key: "m6d08-register-stable-member".to_string(),
    }
}

fn unwrap_multi(response: M6OrgConsultationRouteResponse) -> M6OrgMultiViewConsultation {
    match response {
        M6OrgConsultationRouteResponse::MultiView { consultation } => consultation,
        M6OrgConsultationRouteResponse::SingleRole { .. } => panic!("expected multi-view route"),
    }
}

fn submit_request(
    consultation: &M6OrgMultiViewConsultation,
    index: usize,
    position: M6OrgConsultationClaimPosition,
) -> M6OrgSubmitConsultationViewRequest {
    let view = &consultation.views[index];
    let key = format!("m6d08-submit-{index}");
    M6OrgSubmitConsultationViewRequest {
        consultation_id: consultation.consultation_id.clone(),
        view_id: view.view_id.clone(),
        role_session_id: view.role_session_id.clone(),
        workcell_ref: view.workcell_ref.clone(),
        context_packet_ref: view.context_packet_ref.clone(),
        question_packet_id: view.question_packet_id.clone(),
        question_packet_hash: view.question_packet_hash.clone(),
        runtime_input_refs: view.dispatch_input_refs.clone(),
        runtime_final_answer_ref: sealed_ref("runtime-final-candidate", &key),
        runtime_final_answer_hash: format!("{:x}", Sha256::digest(key.as_bytes())),
        claims: vec![
            M6OrgConsultationClaim {
                topic_ref: "topic:shared-risk".to_string(),
                position: M6OrgConsultationClaimPosition::Support,
                evidence_refs: vec![consultation.question_packet.source_refs[0].clone()],
            },
            M6OrgConsultationClaim {
                topic_ref: "topic:project-conflict".to_string(),
                position,
                evidence_refs: vec![consultation.question_packet.source_refs[1].clone()],
            },
        ],
        reported_cost_units: 2,
        peer_conclusions_readable_before_submit: false,
        idempotency_key: key,
    }
}

fn is_coordination_file(path: &Path, coordination_dbs: &[PathBuf]) -> bool {
    coordination_dbs.iter().any(|database| {
        path == database
            || ["-wal", "-shm", "-journal"]
                .iter()
                .any(|suffix| path == PathBuf::from(format!("{}{}", database.display(), suffix)))
    })
}

fn snapshot_except_coordination(
    root: &Path,
    coordination_dbs: &[PathBuf],
) -> BTreeMap<String, String> {
    fn walk(
        root: &Path,
        current: &Path,
        coordination_dbs: &[PathBuf],
        snapshot: &mut BTreeMap<String, String>,
    ) {
        let mut entries = std::fs::read_dir(current)
            .unwrap_or_else(|error| panic!("read fixture directory {}: {error}", current.display()))
            .collect::<Result<Vec<_>, _>>()
            .expect("collect fixture directory");
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            let file_type = entry.file_type().expect("fixture file type");
            assert!(
                !file_type.is_symlink(),
                "unexpected fixture symlink: {}",
                path.display()
            );
            if file_type.is_dir() {
                walk(root, &path, coordination_dbs, snapshot);
            } else if file_type.is_file() && !is_coordination_file(&path, coordination_dbs) {
                let relative = path
                    .strip_prefix(root)
                    .expect("fixture relative path")
                    .to_string_lossy()
                    .to_string();
                let bytes = std::fs::read(&path).expect("read write-spy file");
                snapshot.insert(relative, format!("{:x}", Sha256::digest(bytes)));
            }
        }
    }

    let mut snapshot = BTreeMap::new();
    walk(root, root, coordination_dbs, &mut snapshot);
    snapshot
}

fn install_project_write_spies(root: &Path) {
    let project = root.join("synthetic-project-zero-write-surfaces");
    std::fs::create_dir_all(&project).expect("create project write-spy root");
    for (name, marker) in [
        ("domain-store.sqlite", "project-domain-store"),
        ("events.jsonl", "project-event"),
        ("audit.jsonl", "project-audit"),
        ("outbox.jsonl", "project-outbox"),
        ("sidecar.json", "project-sidecar"),
        ("compatibility-projection.json", "project-compatibility"),
        ("related-file.md", "project-related-file"),
        ("spawn-settings.json", "project-spawn-settings"),
    ] {
        std::fs::write(project.join(name), format!("m6d08-write-spy:{marker}\n"))
            .expect("write project write-spy fixture");
    }
}

#[test]
fn m6d08_domain_end_to_end_is_project_zero_write_and_decision_only() {
    let fixture = fixture("m6d08-domain-integration");
    seed_execution(
        &fixture,
        "m6d08-domain-integration",
        "SUCCEEDED",
        "role-session:m6d08-domain-integration",
    );
    let summaries = [
        summary("project-alpha", "shared-orchestration", NOW_MS - 2_000, 3),
        summary("project-beta", "shared-orchestration", NOW_MS - 1_000, 7),
    ];
    seed_summaries(&fixture.state, &summaries);
    bind_fake_secretary_for_member_directory(&fixture.state).expect("bind fake Secretary");
    install_project_write_spies(&fixture.root);

    if let Some(parent) = fixture.state.workflow_state_path.parent() {
        std::fs::create_dir_all(parent).expect("create synthetic workflow-state parent");
    }
    let mut workflow_state: serde_json::Value = if fixture.state.workflow_state_path.exists() {
        serde_json::from_slice(
            &std::fs::read(&fixture.state.workflow_state_path).expect("read workflow state"),
        )
        .expect("decode workflow state")
    } else {
        json!({"workflow_version": 0, "workflows": [], "audit_events": []})
    };
    workflow_state["legacy_process_fact_decisions"] = json!([{
        "process_fact_id": LEGACY_PROCESS_FACT_MARKER,
        "owner": "record_project_director_process_fact_decision",
        "scope": "single-project-only"
    }]);
    std::fs::write(
        &fixture.state.workflow_state_path,
        serde_json::to_vec_pretty(&workflow_state).expect("encode workflow marker"),
    )
    .expect("seed legacy process-fact marker");

    let app_data_root = fixture
        .root
        .join(crate::m1_project_index::M1_ORDINARY_APP_DATA_DIR_NAME);
    let m3_path = app_data_root
        .join(crate::m3_role_session_repository::M3_ORDINARY_ROLE_SESSION_RELATIVE_PATH);
    let m6_path = fixture.state.m6_org_store_path().expect("M6 path");
    let coordination_dbs = [m3_path, m6_path.clone()];
    let before = snapshot_except_coordination(&fixture.root, &coordination_dbs);

    let temporary = refresh_for_state(&fixture.state, NOW_MS + 2_000)
        .expect("refresh true-shaped M5 temporary history");
    assert_eq!(
        temporary.source_state,
        M6OrgTemporaryAgentSourceState::CompatibleExecutionHistory
    );
    assert_eq!(temporary.records.len(), 1);
    assert!(temporary.records[0].child_run_ref.is_some());
    let temporary_id = temporary.records[0].temporary_agent_id.clone();
    let temporary_lookup = search_for_state(
        &fixture.state,
        &M6OrgSearchTemporaryAgentHistoryRequest {
            query: temporary_id.clone(),
            limit: 10,
        },
    )
    .expect("find exact temporary history");
    assert_eq!(temporary_lookup.matches.len(), 1);

    let registration = register_for_state(
        &fixture.state,
        &stable_member_request("member_m6d08_stable"),
        NOW_MS + 2_100,
    )
    .expect("register stable member");
    assert_eq!(
        registration.disposition,
        M6OrgMemberRegistrationDisposition::Registered
    );
    let established = registration.member.expect("stable member");
    assert_ne!(established.member_id, temporary_id);
    assert!(established.promoted_from.is_none());
    let active = update_for_state(
        &fixture.state,
        &M6OrgUpdateStableMemberRequest {
            member_id: established.member_id.clone(),
            expected_revision: established.revision,
            activate: true,
            display_name_ref: None,
            added_scope_assignments: Vec::new(),
            added_role_assignments: Vec::new(),
            added_capability_permission_refs: Vec::new(),
            added_memory_refs: Vec::new(),
            added_contact_bindings: Vec::new(),
            idempotency_key: "m6d08-activate-stable-member".to_string(),
        },
        NOW_MS + 2_200,
    )
    .expect("activate stable member");

    let runtime_child = register_for_state(
        &fixture.state,
        &M6OrgRegisterStableMemberRequest {
            member_id: "member_runtime_child_candidate".to_string(),
            display_name_ref: "display-name:runtime-child".to_string(),
            identity_evidence: M6OrgStableIdentityEvidence::HeuristicCandidate {
                candidate_kind: M6OrgHeuristicCandidateKind::RuntimeChild,
                source_refs: vec![format!("child-run-ref:{temporary_id}")],
            },
            scope_assignments: Vec::new(),
            role_assignments: Vec::new(),
            capability_permission_refs: Vec::new(),
            memory_refs: Vec::new(),
            contact_bindings: Vec::new(),
            idempotency_key: "m6d08-runtime-child-candidate".to_string(),
        },
        NOW_MS + 2_300,
    )
    .expect("quarantine runtime-child clue");
    assert_eq!(
        runtime_child.disposition,
        M6OrgMemberRegistrationDisposition::Quarantined
    );
    assert!(runtime_child.member.is_none());

    let stale = observe_availability_for_state(
        &fixture.state,
        &M6OrgObserveMemberAvailabilityRequest {
            member_id: active.member_id.clone(),
            source: "fake-provider-runtime:m6d08-stale".to_string(),
            source_revision: 1,
            observed_at: NOW_MS - 10_000,
            ttl: M6OrgAvailabilityTtl { seconds: 5 },
            observed_state: M6OrgAvailabilityState::Available,
            idempotency_key: "m6d08-stale-availability".to_string(),
        },
        NOW_MS,
    )
    .expect("record stale availability");
    assert_eq!(stale.effective_state, M6OrgAvailabilityState::Unknown);
    assert!(!stale.authorizes);
    let capability_lookup = list_for_state(
        &fixture.state,
        &M6OrgListStableMembersRequest {
            include_deactivated: false,
            available_capability_ref: Some("capability:research".to_string()),
        },
        NOW_MS,
    )
    .expect("capability-filtered member lookup");
    assert!(capability_lookup.members.is_empty());
    assert!(!capability_lookup.stale_availability_used_as_capability);
    let all_stable = list_for_state(
        &fixture.state,
        &M6OrgListStableMembersRequest {
            include_deactivated: false,
            available_capability_ref: None,
        },
        NOW_MS,
    )
    .expect("list stable members");
    assert_eq!(all_stable.members.len(), 1);
    assert_eq!(all_stable.members[0].member.member_id, active.member_id);
    assert_ne!(all_stable.members[0].member.member_id, temporary_id);

    let contact_request = M6OrgContactStableMemberRequest {
        member_id: active.member_id.clone(),
        contact_binding_ref: active.contact_binding_refs[0].clone(),
        reason_ref: "reason:m6d08-integration-contact".to_string(),
        source_refs: vec!["source-ref:m6d08-member-directory".to_string()],
        accept_by_utc: ACCEPT_BY.to_string(),
        idempotency_key: "m6d08-contact-stable-member".to_string(),
    };
    let contact =
        contact_for_state(&fixture.state, &contact_request, NOW_MS).expect("contact stable member");
    assert!(!contact.capability_granted);
    assert!(!contact.project_writeback);
    let contact_replay = contact_for_state(&fixture.state, &contact_request, NOW_MS)
        .expect("replay stable member contact");
    assert!(contact_replay.replayed);
    assert_eq!(
        contact_replay.contact_receipt_id,
        contact.contact_receipt_id
    );

    let consult_request = M6OrgSecretaryConsultStartRequest {
        question_ref: "question-ref:m6d08-two-project-conflict".to_string(),
        source_refs: vec![
            "source-ref:project-alpha".to_string(),
            "source-ref:project-beta".to_string(),
        ],
        project_queries: summaries.iter().map(query).collect(),
        accept_by_utc: ACCEPT_BY.to_string(),
        return_by_utc: RETURN_BY.to_string(),
        idempotency_key: "m6d08-secretary-consult".to_string(),
    };
    let started = crate::secretary_agent::start_global_supervisor_consult_for_state(
        &fixture.state,
        &consult_request,
        NOW_MS,
    )
    .expect("start Secretary consult");
    let start_replay = crate::secretary_agent::start_global_supervisor_consult_for_state(
        &fixture.state,
        &consult_request,
        NOW_MS,
    )
    .expect("replay Secretary consult");
    assert!(start_replay.consult.replayed);
    assert_eq!(
        start_replay.consult.handoff.handoff_id,
        started.consult.handoff.handoff_id
    );
    let returned = crate::m6_org_consult_handoff::decide_for_state(
        &fixture.state,
        &M6OrgGlobalSupervisorConsultDecisionRequest {
            handoff_id: started.consult.handoff.handoff_id,
            decision: M6OrgConsultDecision::Accept,
            rejection_reason: None,
        },
        NOW_MS,
    )
    .expect("accept consult and return advisory");
    assert_eq!(returned.handoff.status_ref, "RETURNED");
    assert_eq!(returned.project_command_attempts, 0);
    assert_eq!(returned.provider_invocations, 0);
    let advisory = returned.advisory.expect("cross-project advisory");
    assert_eq!(advisory.consumed_summaries.len(), 2);
    assert_eq!(advisory.source_links.len(), 2);
    assert!(advisory
        .findings
        .iter()
        .any(|finding| finding.reason_code == "concurrent_open_runs_priority_conflict"));
    assert!(advisory.source_links.iter().all(|link| {
        !link.object_ref.is_empty()
            && !link.scrubbed_summary_ref.is_empty()
            && !link.deep_link_metadata_ref.is_empty()
    }));

    let adoption = adopt_for_state(
        &fixture.state,
        &M6OrgAdvisoryAdoptionRequest {
            advisory_id: advisory.advisory_id.clone(),
            actor_ref: "actor:user".to_string(),
            user_confirmed: true,
            idempotency_key: "m6d08-adopt-advisory".to_string(),
        },
        NOW_MS + 3_000,
    )
    .expect("adoption yields DecisionRequest only");
    assert_eq!(adoption.status, "PENDING");
    assert_eq!(adoption.source_object_ref, advisory.advisory_id);
    assert_eq!(
        adoption.decision_command_type,
        "AdoptCrossProjectAdvisoryDecision"
    );

    let consultation = unwrap_multi(
        start_for_state(
            &fixture.state,
            &M6OrgStartMultiViewConsultationRequest {
                question_ref: sealed_ref("question-ref", "m6d08-multi-view"),
                source_refs: advisory
                    .source_links
                    .iter()
                    .map(|link| link.scrubbed_summary_ref.clone())
                    .collect(),
                escalation_trigger: M6OrgConsultationEscalationTrigger::CrossProjectConflict,
                view_kinds: vec![
                    M6OrgConsultationViewKind::RiskAnalyst,
                    M6OrgConsultationViewKind::CounterfactualReviewer,
                ],
                budget_limit_ref: Some(sealed_ref("budget-limit", "m6d08-multi-view")),
                budget_limit_units: Some(20),
                deadline_at_ms: Some(NOW_MS + 20_000),
                idempotency_key: "m6d08-start-multi-view".to_string(),
            },
            NOW_MS + 4_000,
        )
        .expect("start independent multi-view consultation"),
    );
    let after_first = submit_view_for_state(
        &fixture.state,
        &submit_request(&consultation, 0, M6OrgConsultationClaimPosition::Support),
        NOW_MS + 4_001,
    )
    .expect("submit first independent view");
    let after_second = submit_view_for_state(
        &fixture.state,
        &submit_request(&after_first, 1, M6OrgConsultationClaimPosition::Oppose),
        NOW_MS + 4_002,
    )
    .expect("submit second independent view");
    let assembled = assemble_for_state(
        &fixture.state,
        &M6OrgAssembleMultiViewConsultationRequest {
            consultation_id: after_second.consultation_id.clone(),
            expected_revision: after_second.revision,
            idempotency_key: "m6d08-assemble-multi-view".to_string(),
        },
        NOW_MS + 4_003,
    )
    .expect("assemble submitted views");
    assert_eq!(
        assembled.result_state,
        M6OrgConsultationResultState::Assembled
    );
    let decision = assembled.decision_request.expect("pending user decision");
    assert_eq!(decision.status, "PENDING_USER_DECISION");
    assert!(!decision.creates_project_command);
    assert!(!decision.creates_grant);
    assert!(!decision.creates_formal_fact);
    assert!(!assembled.produces_command);
    assert!(!assembled.produces_grant);
    assert!(!assembled.produces_fact);

    let directory_export = export_for_state(&fixture.state, NOW_MS + 5_000)
        .expect("directory export and internal rebuild verification");
    assert_eq!(directory_export.member_history.len(), 2);
    assert_eq!(directory_export.availability_history.len(), 1);
    assert_eq!(directory_export.contact_history.len(), 1);
    assert_eq!(directory_export.quarantines.len(), 1);

    let after = snapshot_except_coordination(&fixture.root, &coordination_dbs);
    assert_eq!(
        after, before,
        "M6 changed a project/M5/file/spawn write surface"
    );
    for required_spy in [
        "domain-store.sqlite",
        "events.jsonl",
        "audit.jsonl",
        "outbox.jsonl",
        "sidecar.json",
        "compatibility-projection.json",
        "related-file.md",
        "spawn-settings.json",
    ] {
        assert!(before.keys().any(|path| path.ends_with(required_spy)));
    }

    let m6_bytes = std::fs::read(&m6_path).expect("read M6 coordination store");
    assert!(!String::from_utf8_lossy(&m6_bytes).contains(LEGACY_PROCESS_FACT_MARKER));
    assert!(advisory
        .source_links
        .iter()
        .all(|link| !link.object_ref.contains(LEGACY_PROCESS_FACT_MARKER)));

    let commands = include_str!("commands.rs");
    let command_start = commands
        .find("fn record_project_director_process_fact_decision(")
        .expect("legacy process-fact command");
    let command_span = &commands[command_start..];
    let command_end = command_span
        .find("fn record_global_final_result_review(")
        .expect("next legacy command boundary");
    let command_span = &command_span[..command_end];
    assert!(command_span.contains("state.workflow_state_path"));
    assert!(!command_span.contains("m6_org"));
    assert!(!command_span.contains("ProjectSummaryQueryPort"));

    let registry = include_str!("command_registry.rs");
    for legacy_entry in [
        "global_supervisor_review_store::load_global_supervisor_review_store",
        "load_agent_role_session_directory",
        "load_agent_role_session_detail",
    ] {
        assert_eq!(
            registry.matches(legacy_entry).count(),
            1,
            "legacy entry {legacy_entry}"
        );
    }
    let advisory_source = include_str!("m6_org_cross_project_advisory.rs");
    let advisory_production = advisory_source
        .split("#[cfg(test)]")
        .next()
        .expect("production advisory span");
    assert!(advisory_production.contains("ProjectSummaryQueryPort"));
    assert!(!advisory_production.contains("record_project_director_process_fact_decision"));
    assert!(!advisory_production.contains("project_root"));
    assert!(!advisory_production.contains("read_to_string"));
}
