#[test]
fn workflow_authorization_plan_authorization_guard_blocks_without_authorization() {
    let timestamp_ms = 1_765_000_000_000;
    let project_root = "/tmp/c1-plan-auth-project";
    let store = PlanAuthorizationStoreV1 {
        schema_version: "plan_authorization_store.v1".to_string(),
        revision: 0,
        authorizations: vec![],
        audit_events: vec![],
        updated_at_ms: timestamp_ms,
        warnings: vec![],
    };
    let input = fixture_plan_authorization_guard_input(project_root);

    let result = control_core::inspect_auto_dispatch_scope(&store, &input, timestamp_ms);

    assert_eq!(result.status, "blocked");
    assert!(result
        .reasons
        .iter()
        .any(|reason| reason.contains("缺少有效方案授权")));
    assert!(result.required_user_confirmation);
    assert!(result.required_global_review);
}

#[test]
fn plan_authorization_guard_needs_review_before_user_and_global_approval() {
    let timestamp_ms = 1_765_000_000_000;
    let project_root = "/tmp/c1-plan-auth-project";
    let input = fixture_plan_authorization_guard_input(project_root);
    let pending_user = fixture_plan_authorization_store_with_status(
        project_root,
        PlanAuthorizationStatus::PendingUserConfirmation,
        timestamp_ms,
    );
    let pending_global = fixture_plan_authorization_store_with_status(
        project_root,
        PlanAuthorizationStatus::PendingGlobalBoundaryReview,
        timestamp_ms,
    );

    let user_result =
        control_core::inspect_auto_dispatch_scope(&pending_user, &input, timestamp_ms);
    let global_result =
        control_core::inspect_auto_dispatch_scope(&pending_global, &input, timestamp_ms);

    assert_eq!(user_result.status, "needs_review");
    assert!(user_result.required_user_confirmation);
    assert!(user_result
        .reasons
        .iter()
        .any(|reason| reason.contains("待用户确认")));
    assert_eq!(global_result.status, "needs_review");
    assert!(global_result.required_global_review);
    assert!(global_result
        .reasons
        .iter()
        .any(|reason| reason.contains("待全局边界复核")));
}

#[test]
fn plan_authorization_guard_authorizes_matching_active_scope() {
    let timestamp_ms = 1_765_000_000_000;
    let project_root = "/tmp/c1-plan-auth-project";
    let store = fixture_plan_authorization_store_with_status(
        project_root,
        PlanAuthorizationStatus::Active,
        timestamp_ms,
    );
    let input = fixture_plan_authorization_guard_input(project_root);

    let result = control_core::inspect_auto_dispatch_scope(&store, &input, timestamp_ms);

    assert_eq!(result.status, "authorized", "{:?}", result.reasons);
    assert_eq!(
        result.authorization_id.as_deref(),
        Some("plan-auth:c1-fixture")
    );
    assert!(result.reasons.is_empty());
}

#[test]
fn plan_authorization_guard_blocks_write_scope_escape() {
    let timestamp_ms = 1_765_000_000_000;
    let project_root = "/tmp/c1-plan-auth-project";
    let store = fixture_plan_authorization_store_with_status(
        project_root,
        PlanAuthorizationStatus::Active,
        timestamp_ms,
    );
    let mut input = fixture_plan_authorization_guard_input(project_root);
    input.requested_write_roots = vec!["/tmp/c1-plan-auth-outside".to_string()];

    let result = control_core::inspect_auto_dispatch_scope(&store, &input, timestamp_ms);

    assert_eq!(result.status, "blocked");
    assert!(result
        .reasons
        .iter()
        .any(|reason| reason.contains("写入范围超出方案授权")));
}

#[test]
fn plan_authorization_guard_blocks_role_and_agent_escape() {
    let timestamp_ms = 1_765_000_000_000;
    let project_root = "/tmp/c1-plan-auth-project";
    let store = fixture_plan_authorization_store_with_status(
        project_root,
        PlanAuthorizationStatus::Active,
        timestamp_ms,
    );
    let mut input = fixture_plan_authorization_guard_input(project_root);
    input.target_role_id = "validation".to_string();
    input.target_agent_id = Some("agent-2".to_string());

    let result = control_core::inspect_auto_dispatch_scope(&store, &input, timestamp_ms);

    assert_eq!(result.status, "blocked");
    assert!(result
        .reasons
        .iter()
        .any(|reason| reason.contains("目标角色不在授权范围内")));
    assert!(result
        .reasons
        .iter()
        .any(|reason| reason.contains("目标 agent 不在授权范围内")));
}

#[test]
fn plan_authorization_guard_blocks_revoked_paused_and_expired() {
    let timestamp_ms = 1_765_000_000_000;
    let project_root = "/tmp/c1-plan-auth-project";
    let input = fixture_plan_authorization_guard_input(project_root);

    for (status, expected_reason) in [
        (PlanAuthorizationStatus::Revoked, "方案授权已撤销"),
        (PlanAuthorizationStatus::Paused, "方案授权已暂停"),
        (PlanAuthorizationStatus::Expired, "方案授权已过期"),
    ] {
        let store =
            fixture_plan_authorization_store_with_status(project_root, status, timestamp_ms);
        let result = control_core::inspect_auto_dispatch_scope(&store, &input, timestamp_ms);

        assert_eq!(result.status, "blocked");
        assert!(
            result
                .reasons
                .iter()
                .any(|reason| reason.contains(expected_reason)),
            "{:?}",
            result.reasons
        );
    }
}

#[test]
fn plan_authorization_guard_needs_review_when_stop_condition_requires_user() {
    let timestamp_ms = 1_765_000_000_000;
    let project_root = "/tmp/c1-plan-auth-project";
    let store = fixture_plan_authorization_store_with_status(
        project_root,
        PlanAuthorizationStatus::Active,
        timestamp_ms,
    );
    let mut input = fixture_plan_authorization_guard_input(project_root);
    input.triggered_stop_conditions = vec!["requires_user_confirmation".to_string()];

    let result = control_core::inspect_auto_dispatch_scope(&store, &input, timestamp_ms);

    assert_eq!(result.status, "needs_review");
    assert!(result.required_user_confirmation);
    assert!(result
        .reasons
        .iter()
        .any(|reason| reason.contains("触发必须请用户确认的停止条件")));
}

#[test]
fn plan_authorization_inspect_writes_auto_dispatch_scope_checked_audit() {
    let timestamp_ms = 1_765_000_000_000;
    let dir = test_temp_dir("plan-authorization-inspect-audit");
    let path = dir.join("workflow-state.v0.json");
    let project = fixture_project("/tmp/c1-plan-auth-inspect-project");

    bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
    create_active_plan_authorization_for_fixture(&path, &project.project_root);
    let mut input = fixture_plan_authorization_guard_input(&project.project_root);
    input.target_agent_id = None;
    input.requested_checks = vec![];

    let result = plan_authorization_store::inspect_auto_dispatch_authorization(
        &path,
        &input,
        timestamp_ms,
        "write-c1-plan-authorization-inspect",
    )
    .expect("inspect should write audit");
    let store = plan_authorization_store::load_store(&path, timestamp_ms + 1)
        .expect("store should load");

    assert_eq!(result.status, "authorized", "{:?}", result.reasons);
    assert!(store.audit_events.iter().any(|event| {
        event.event_type == "auto_dispatch_scope_checked"
            && event.work_item_id.as_deref() == Some(input.work_item_id.as_str())
            && event
                .guard_result
                .as_ref()
                .is_some_and(|guard| guard.status == "authorized")
    }));

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn project_consultation_proposal_create_writes_revision_and_read_model() {
    let timestamp_ms = 1_765_100_000_000;
    let dir = test_temp_dir("project-consultation-proposal-create");
    let path = dir.join("workflow-state.v0.json");
    let project = fixture_project("/tmp/c2-project-consultation-create");
    bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");

    let output = project_consultation_proposal_store::create_proposal(
        &path,
        &fixture_project_consultation_proposal_input(&project.project_root),
        timestamp_ms,
        "write-c2-proposal-create",
    )
    .expect("proposal should create");
    let store = project_consultation_proposal_store::load_store(&path, timestamp_ms + 1)
        .expect("proposal store should load");

    assert_eq!(output.store_revision, 1);
    assert_eq!(store.revision, 1);
    assert_eq!(
        output.proposal.status,
        ProjectConsultationProposalStatus::PendingUserConfirmation
    );
    assert_eq!(output.read_model.proposal_count, 1);
    assert_eq!(
        output.read_model.latest_status,
        Some(ProjectConsultationProposalStatus::PendingUserConfirmation)
    );

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn project_consultation_proposal_rejects_missing_required_fields() {
    let timestamp_ms = 1_765_100_000_000;
    let dir = test_temp_dir("project-consultation-proposal-invalid");
    let path = dir.join("workflow-state.v0.json");
    let project = fixture_project("/tmp/c2-project-consultation-invalid");
    bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
    let mut input = fixture_project_consultation_proposal_input(&project.project_root);
    input.user_goal.clear();
    let err = project_consultation_proposal_store::create_proposal(
        &path,
        &input,
        timestamp_ms,
        "write-c2-proposal-invalid-goal",
    )
    .expect_err("missing user_goal should fail");
    assert!(err.contains("user_goal"));

    let mut input = fixture_project_consultation_proposal_input(&project.project_root);
    input.acceptance_criteria.clear();
    let err = project_consultation_proposal_store::create_proposal(
        &path,
        &input,
        timestamp_ms,
        "write-c2-proposal-invalid-acceptance",
    )
    .expect_err("missing acceptance criteria should fail");
    assert!(err.contains("acceptance criterion"));

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn project_consultation_proposal_confirm_creates_user_confirmed_authorization_not_active() {
    let timestamp_ms = 1_765_100_000_000;
    let dir = test_temp_dir("project-consultation-proposal-confirm");
    let path = dir.join("workflow-state.v0.json");
    let project = fixture_project("/tmp/c2-project-consultation-confirm");
    bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
    let created = project_consultation_proposal_store::create_proposal(
        &path,
        &fixture_project_consultation_proposal_input(&project.project_root),
        timestamp_ms,
        "write-c2-proposal-confirm-create",
    )
    .expect("proposal should create");

    let output = project_consultation_proposal_store::record_decision(
        &path,
        &RecordProjectConsultationProposalDecisionInput {
            project_root: project.project_root.clone(),
            proposal_id: created.proposal.proposal_id.clone(),
            actor_id: "user-fixture".to_string(),
            decision: ProjectConsultationProposalDecisionKind::Confirm,
            summary: "用户确认 C2 测试方案；仍需全局主管复核。".to_string(),
            expected_proposal_store_revision: Some(created.store_revision),
            expected_plan_authorization_store_revision: None,
        },
        timestamp_ms + 1,
        "write-c2-proposal-confirm",
        "write-c2-proposal-confirm-auth",
        "write-c2-proposal-confirm-auth-user",
    )
    .expect("confirm should write proposal and authorization");
    let authorization = output
        .plan_authorization
        .clone()
        .expect("confirm should return linked authorization");

    assert_eq!(
        output.proposal.status,
        ProjectConsultationProposalStatus::UserConfirmed
    );
    assert_eq!(
        authorization.source_proposal_id.as_deref(),
        Some(created.proposal.proposal_id.as_str())
    );
    assert_eq!(
        authorization.status,
        PlanAuthorizationStatus::PendingGlobalBoundaryReview
    );
    assert!(authorization.user_confirmation.is_some());

    let plan_store = plan_authorization_store::load_store(&path, timestamp_ms + 2)
        .expect("plan authorization store should load");
    let guard = control_core::inspect_auto_dispatch_scope(
        &plan_store,
        &fixture_plan_authorization_guard_input(&project.project_root),
        timestamp_ms + 2,
    );
    assert_eq!(guard.status, "needs_review");
    assert!(guard.required_global_review);
    assert!(guard
        .reasons
        .iter()
        .any(|reason| reason.contains("待全局边界复核")));

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn project_consultation_proposal_request_changes_and_reject_do_not_create_authorization() {
    for (decision, suffix) in [
        (
            ProjectConsultationProposalDecisionKind::RequestChanges,
            "request-changes",
        ),
        (ProjectConsultationProposalDecisionKind::Reject, "reject"),
    ] {
        let timestamp_ms = 1_765_100_000_000;
        let dir = test_temp_dir(&format!("project-consultation-proposal-{suffix}"));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project(&format!("/tmp/c2-project-consultation-{suffix}"));
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let created = project_consultation_proposal_store::create_proposal(
            &path,
            &fixture_project_consultation_proposal_input(&project.project_root),
            timestamp_ms,
            &format!("write-c2-proposal-{suffix}-create"),
        )
        .expect("proposal should create");

        let output = project_consultation_proposal_store::record_decision(
            &path,
            &RecordProjectConsultationProposalDecisionInput {
                project_root: project.project_root.clone(),
                proposal_id: created.proposal.proposal_id.clone(),
                actor_id: "user-fixture".to_string(),
                decision,
                summary: "用户未确认当前项目咨询方案。".to_string(),
                expected_proposal_store_revision: Some(created.store_revision),
                expected_plan_authorization_store_revision: None,
            },
            timestamp_ms + 1,
            &format!("write-c2-proposal-{suffix}"),
            &format!("write-c2-proposal-{suffix}-auth"),
            &format!("write-c2-proposal-{suffix}-auth-user"),
        )
        .expect("decision should write");
        let plan_store = plan_authorization_store::load_store(&path, timestamp_ms + 2)
            .expect("empty plan authorization store should load");

        assert!(output.plan_authorization.is_none());
        assert!(plan_store.authorizations.is_empty());
        assert_eq!(
            output.proposal.status,
            if decision == ProjectConsultationProposalDecisionKind::RequestChanges {
                ProjectConsultationProposalStatus::ChangesRequested
            } else {
                ProjectConsultationProposalStatus::Rejected
            }
        );

        let _ = fs::remove_dir_all(dir);
    }
}

#[test]
fn project_consultation_proposal_rejects_repeated_confirmation() {
    let timestamp_ms = 1_765_100_000_000;
    let dir = test_temp_dir("project-consultation-proposal-repeat-confirm");
    let path = dir.join("workflow-state.v0.json");
    let project = fixture_project("/tmp/c2-project-consultation-repeat-confirm");
    bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
    let created = project_consultation_proposal_store::create_proposal(
        &path,
        &fixture_project_consultation_proposal_input(&project.project_root),
        timestamp_ms,
        "write-c2-proposal-repeat-create",
    )
    .expect("proposal should create");
    let first = project_consultation_proposal_store::record_decision(
        &path,
        &RecordProjectConsultationProposalDecisionInput {
            project_root: project.project_root.clone(),
            proposal_id: created.proposal.proposal_id.clone(),
            actor_id: "user-fixture".to_string(),
            decision: ProjectConsultationProposalDecisionKind::Confirm,
            summary: "用户确认 C2 测试方案。".to_string(),
            expected_proposal_store_revision: Some(created.store_revision),
            expected_plan_authorization_store_revision: None,
        },
        timestamp_ms + 1,
        "write-c2-proposal-repeat-confirm",
        "write-c2-proposal-repeat-confirm-auth",
        "write-c2-proposal-repeat-confirm-auth-user",
    )
    .expect("first confirm should work");
    let err = project_consultation_proposal_store::record_decision(
        &path,
        &RecordProjectConsultationProposalDecisionInput {
            project_root: project.project_root.clone(),
            proposal_id: created.proposal.proposal_id,
            actor_id: "user-fixture".to_string(),
            decision: ProjectConsultationProposalDecisionKind::Confirm,
            summary: "重复确认应被拒绝。".to_string(),
            expected_proposal_store_revision: Some(first.store_revision),
            expected_plan_authorization_store_revision: Some(
                first.plan_authorization_store_revision.unwrap_or(0),
            ),
        },
        timestamp_ms + 2,
        "write-c2-proposal-repeat-confirm-2",
        "write-c2-proposal-repeat-confirm-auth-2",
        "write-c2-proposal-repeat-confirm-auth-user-2",
    )
    .expect_err("second confirm should fail");

    assert!(err.contains("不能重复记录用户决定"));

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn global_boundary_review_approved_activates_authorization_and_guard_still_checks_scope() {
    let timestamp_ms = 1_765_200_000_000;
    let dir = test_temp_dir("global-boundary-review-approved");
    let path = dir.join("workflow-state.v0.json");
    let project = fixture_project("/tmp/c3-global-boundary-approved");
    bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
    let (proposal, authorization, revision) =
        create_confirmed_proposal_for_global_review(&path, &project.project_root, timestamp_ms);

    let output = plan_authorization_store::record_global_boundary_review_with_proposal(
        &path,
        &fixture_global_boundary_review_input(
            &project.project_root,
            &proposal.proposal_id,
            &authorization.authorization_id,
            revision,
        ),
        timestamp_ms + 2,
        "write-c3-global-boundary-approved",
    )
    .expect("approved review should activate authorization");
    let store = plan_authorization_store::load_store(&path, timestamp_ms + 3)
        .expect("plan authorization store should load");
    let mut guard_input = fixture_plan_authorization_guard_input(&project.project_root);

    assert_eq!(output.authorization.status, PlanAuthorizationStatus::Active);
    assert_eq!(output.guard_result.status, "authorized");
    assert_eq!(
        output
            .authorization
            .global_boundary_review
            .as_ref()
            .and_then(|review| review.source_proposal_id.as_deref()),
        Some(proposal.proposal_id.as_str())
    );
    assert_eq!(
        control_core::inspect_auto_dispatch_scope(&store, &guard_input, timestamp_ms + 3).status,
        "authorized"
    );

    guard_input.requested_write_roots = vec!["/tmp/c3-global-boundary-outside".to_string()];
    let blocked =
        control_core::inspect_auto_dispatch_scope(&store, &guard_input, timestamp_ms + 3);
    assert_eq!(blocked.status, "blocked");
    assert!(blocked
        .reasons
        .iter()
        .any(|reason| reason.contains("写入范围超出方案授权")));

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn global_boundary_review_rejects_missing_user_confirmation() {
    let timestamp_ms = 1_765_200_000_000;
    let dir = test_temp_dir("global-boundary-review-missing-user");
    let path = dir.join("workflow-state.v0.json");
    let project = fixture_project("/tmp/c3-global-boundary-missing-user");
    bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
    let (proposal, authorization, revision) =
        create_confirmed_proposal_for_global_review(&path, &project.project_root, timestamp_ms);
    let mut store = plan_authorization_store::load_store(&path, timestamp_ms + 2)
        .expect("plan authorization store should load");
    store.authorizations[0].user_confirmation = None;
    fs::write(
        plan_authorization_store::sidecar_path(&path).expect("sidecar path"),
        serde_json::to_string_pretty(&store).expect("store should serialize"),
    )
    .expect("test should write mutated sidecar");

    let err = plan_authorization_store::record_global_boundary_review_with_proposal(
        &path,
        &fixture_global_boundary_review_input(
            &project.project_root,
            &proposal.proposal_id,
            &authorization.authorization_id,
            revision,
        ),
        timestamp_ms + 3,
        "write-c3-global-boundary-missing-user",
    )
    .expect_err("missing user confirmation should fail");

    assert!(err.contains("缺少用户确认"));

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn global_boundary_review_rejects_proposal_authorization_mismatch() {
    let timestamp_ms = 1_765_200_000_000;
    let dir = test_temp_dir("global-boundary-review-mismatch");
    let path = dir.join("workflow-state.v0.json");
    let project = fixture_project("/tmp/c3-global-boundary-mismatch");
    bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
    let (proposal, _authorization, revision) =
        create_confirmed_proposal_for_global_review(&path, &project.project_root, timestamp_ms);

    let err = plan_authorization_store::record_global_boundary_review_with_proposal(
        &path,
        &fixture_global_boundary_review_input(
            &project.project_root,
            &proposal.proposal_id,
            "plan-auth:wrong",
            revision,
        ),
        timestamp_ms + 2,
        "write-c3-global-boundary-mismatch",
    )
    .expect_err("mismatched authorization should fail");

    assert!(err.contains("回链"));

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn global_boundary_review_rejects_incomplete_checklist_and_blocking_finding_for_approved() {
    let timestamp_ms = 1_765_200_000_000;
    for (suffix, mutate) in [("checklist", "checklist"), ("blocking-finding", "finding")] {
        let dir = test_temp_dir(&format!("global-boundary-review-{suffix}"));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project(&format!("/tmp/c3-global-boundary-{suffix}"));
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let (proposal, authorization, revision) =
            create_confirmed_proposal_for_global_review(&path, &project.project_root, timestamp_ms);
        let mut input = fixture_global_boundary_review_input(
            &project.project_root,
            &proposal.proposal_id,
            &authorization.authorization_id,
            revision,
        );
        if mutate == "checklist" {
            input.checklist.read_write_scope_checked = false;
        } else {
            input.findings.push(GlobalBoundaryReviewFinding {
                finding_id: "finding:blocking".to_string(),
                severity: "blocking".to_string(),
                summary: "存在阻断项。".to_string(),
                recommendation: Some("先修改方案。".to_string()),
            });
        }

        let err = plan_authorization_store::record_global_boundary_review_with_proposal(
            &path,
            &input,
            timestamp_ms + 2,
            &format!("write-c3-global-boundary-{suffix}"),
        )
        .expect_err("approved review should validate checklist and findings");

        assert!(
            err.contains("checklist") || err.contains("blocking"),
            "{err}"
        );

        let _ = fs::remove_dir_all(dir);
    }
}

#[test]
fn global_boundary_review_needs_changes_and_blocked_do_not_activate_authorization() {
    let timestamp_ms = 1_765_200_000_000;
    for (status, suffix) in [("needs_changes", "needs-changes"), ("blocked", "blocked")] {
        let dir = test_temp_dir(&format!("global-boundary-review-{suffix}"));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project(&format!("/tmp/c3-global-boundary-{suffix}"));
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let (proposal, authorization, revision) =
            create_confirmed_proposal_for_global_review(&path, &project.project_root, timestamp_ms);
        let mut input = fixture_global_boundary_review_input(
            &project.project_root,
            &proposal.proposal_id,
            &authorization.authorization_id,
            revision,
        );
        input.review_status = status.to_string();
        input.summary = format!("全局主管复核结论为 {status}；不能自动推进。");
        input.findings.push(GlobalBoundaryReviewFinding {
            finding_id: format!("finding:{suffix}"),
            severity: if status == "blocked" {
                "blocking".to_string()
            } else {
                "warning".to_string()
            },
            summary: input.summary.clone(),
            recommendation: Some("调整方案后再复核。".to_string()),
        });

        let output = plan_authorization_store::record_global_boundary_review_with_proposal(
            &path,
            &input,
            timestamp_ms + 2,
            &format!("write-c3-global-boundary-{suffix}"),
        )
        .expect("non-approved review should write paused authorization");
        let store = plan_authorization_store::load_store(&path, timestamp_ms + 3)
            .expect("plan authorization store should load");
        let guard = control_core::inspect_auto_dispatch_scope(
            &store,
            &fixture_plan_authorization_guard_input(&project.project_root),
            timestamp_ms + 3,
        );

        assert_eq!(output.authorization.status, PlanAuthorizationStatus::Paused);
        assert_eq!(guard.status, "blocked");
        assert!(guard
            .reasons
            .iter()
            .any(|reason| reason.contains("方案授权已暂停")));

        let _ = fs::remove_dir_all(dir);
    }
}
