fn station4_byte_review_authorization(
    allowed_write_roots: Vec<String>,
    allowed_checks: Vec<String>,
) -> crate::PlanAuthorization {
    let project_root = crate::STATION_4_WRITE_PROJECT_ROOT;
    crate::PlanAuthorization {
        authorization_id: "authorization:station4:byte-review".to_string(),
        schema_version: "plan_authorization.v1".to_string(),
        project_id: crate::project_id(project_root),
        workflow_id: "workflow:station4:byte-review".to_string(),
        source_proposal_id: Some("proposal:station4:byte-review".to_string()),
        title: "station4 字节实证夹具".to_string(),
        goal_summary: "写入后做独立字节复核".to_string(),
        status: crate::PlanAuthorizationStatus::Active,
        scope: crate::AuthorizedExecutionScope {
            project_id: crate::project_id(project_root),
            workflow_id: "workflow:station4:byte-review".to_string(),
            allowed_role_ids: vec!["codex-dev".to_string()],
            allowed_agent_ids: vec!["thread:station4:byte-review".to_string()],
            allowed_read_roots: vec![project_root.to_string()],
            allowed_write_roots,
            allowed_tools: vec!["read_file".to_string(), "apply_patch".to_string()],
            allowed_checks,
            allowed_task_package_kinds: vec!["task_package".to_string()],
            max_worker_dispatches: Some(2),
            max_runtime_minutes: Some(30),
            stop_conditions: vec![],
        },
        user_confirmation: None,
        global_boundary_review: None,
        audit_refs: vec![],
        created_at_ms: 0,
        updated_at_ms: 0,
        expires_at_ms: None,
    }
}

fn station4_byte_review_proposal(
    authorization: &crate::PlanAuthorization,
) -> crate::ProjectConsultationProposal {
    let project_root = crate::STATION_4_WRITE_PROJECT_ROOT;
    crate::ProjectConsultationProposal {
        proposal_id: "proposal:station4:byte-review".to_string(),
        schema_version: "project_consultation_proposal.v1".to_string(),
        project_id: crate::project_id(project_root),
        workflow_id: authorization.workflow_id.clone(),
        title: "站4字节复核夹具".to_string(),
        user_goal: "创建文件后核对字节".to_string(),
        user_requirement_snapshot: "创建文件后核对字节".to_string(),
        goal_summary: "先写再独立只读复核".to_string(),
        proposed_steps: vec!["写入文件".to_string(), "只读复核字节".to_string()],
        scope_draft: crate::ProjectConsultationProposalScopeDraft {
            allowed_role_ids: vec!["codex-dev".to_string()],
            allowed_agent_ids: authorization.scope.allowed_agent_ids.clone(),
            allowed_read_roots: authorization.scope.allowed_read_roots.clone(),
            allowed_write_roots: authorization.scope.allowed_write_roots.clone(),
            allowed_tools: authorization.scope.allowed_tools.clone(),
            allowed_checks: authorization.scope.allowed_checks.clone(),
            allowed_task_package_kinds: vec!["task_package".to_string()],
            stop_conditions: vec![],
            max_worker_dispatches: Some(2),
            max_runtime_minutes: Some(30),
        },
        risks: vec![],
        worker_acceptance_criteria: vec!["写入目标文件。".to_string()],
        control_core_acceptance_criteria: vec!["绑定同一授权段。".to_string()],
        supervisor_acceptance_criteria: vec!["只在实证充分时终标。".to_string()],
        acceptance_criteria: vec!["写入目标文件。".to_string()],
        status: crate::ProjectConsultationProposalStatus::UserConfirmed,
        plan_authorization_id: Some(authorization.authorization_id.clone()),
        created_by_role: crate::ProjectConsultationProposalCreatorRole::ProjectConsultant,
        suggest_workflow: true,
        created_at_ms: 0,
        updated_at_ms: 0,
    }
}

#[test]
fn station4_byte_check_builds_exact_writer_and_readonly_reviewer_shapes_and_prompt() {
    let project_root = crate::STATION_4_WRITE_PROJECT_ROOT;
    let authorization = station4_byte_review_authorization(
        vec![project_root.to_string()],
        vec!["核对文件大小为 8 字节且末尾无换行".to_string()],
    );
    let proposal = station4_byte_review_proposal(&authorization);
    let request = SupervisorPilotLaunchRequest {
        project_root: project_root.to_string(),
        workflow_id: authorization.workflow_id.clone(),
        authorization_id: authorization.authorization_id.clone(),
        model_id: None,
        reasoning_effort: "medium".to_string(),
    };
    let writer_task_id = supervisor_pilot_planned_task_id(&authorization.authorization_id);
    let reviewer_task = supervisor_pilot_readonly_reviewer_task_for_authorization(
        &writer_task_id,
        &request,
        &proposal,
        &authorization,
    )
    .expect("站4精确写授权命中字节 check 时必须物化只读复核任务");
    let task_write_scopes = vec![
        authorization.scope.allowed_write_roots.clone(),
        reviewer_task.scope.allowed_write_scope.clone(),
    ];
    assert_eq!(task_write_scopes.len(), 2);
    assert_eq!(task_write_scopes[0], vec![project_root.to_string()]);
    assert!(task_write_scopes[1].is_empty());
    assert_eq!(
        reviewer_task.planned_task_id,
        supervisor_pilot_readonly_reviewer_task_id(&authorization.authorization_id)
    );
    assert_eq!(reviewer_task.depends_on, vec![writer_task_id]);
    assert_eq!(
        reviewer_task.scope.required_checks,
        vec!["核对文件大小为 8 字节且末尾无换行".to_string()]
    );
    assert!(reviewer_task
        .scope
        .forbidden_actions
        .contains(&SUPERVISOR_PILOT_READONLY_BYTE_REVIEW_MARKER.to_string()));

    let mut context = fixture_context();
    context.project_root = project_root.to_string();
    context.allowed_write_roots = vec![project_root.to_string()];
    context.allowed_checks = authorization.scope.allowed_checks.clone();
    context.pilot_task = Some(SupervisorPilotTaskReference {
        node_id: "workflow:station4:byte-review:node:codex-dev".to_string(),
        work_item_id: "work-item:station4:writer".to_string(),
        allowed_write: vec![project_root.to_string()],
    });
    context.reviewer_task = Some(SupervisorPilotTaskReference {
        node_id: "workflow:station4:byte-review:node:codex-dev".to_string(),
        work_item_id: "work-item:station4:readonly-review".to_string(),
        allowed_write: vec![],
    });
    let opening_message = assemble_opening_message(&context);
    assert!(opening_message.contains("字节级只读复核纪律"));
    assert!(opening_message.contains("work-item:station4:readonly-review"));
    assert!(opening_message.contains("allowed_write=无"));
    assert!(opening_message.contains(SUPERVISOR_PILOT_READONLY_BYTE_REVIEW_REPORT_FORMAT));
    assert!(opening_message.contains("不得用执行 worker 的 `evidence`"));
}

#[test]
fn byte_review_stays_absent_for_nonbyte_checks_and_station3b_zero_write() {
    let project_root = crate::STATION_4_WRITE_PROJECT_ROOT;
    let no_byte_authorization = station4_byte_review_authorization(
        vec![project_root.to_string()],
        vec!["cargo test --lib".to_string()],
    );
    let no_byte_proposal = station4_byte_review_proposal(&no_byte_authorization);
    let write_request = SupervisorPilotLaunchRequest {
        project_root: project_root.to_string(),
        workflow_id: no_byte_authorization.workflow_id.clone(),
        authorization_id: no_byte_authorization.authorization_id.clone(),
        model_id: None,
        reasoning_effort: "medium".to_string(),
    };
    assert!(supervisor_pilot_readonly_reviewer_task_for_authorization(
        &supervisor_pilot_planned_task_id(&no_byte_authorization.authorization_id),
        &write_request,
        &no_byte_proposal,
        &no_byte_authorization,
    )
    .is_none());

    let readonly_authorization = station4_byte_review_authorization(
        vec![],
        vec!["核对文件大小为 8 字节且末尾无换行".to_string()],
    );
    let readonly_proposal = station4_byte_review_proposal(&readonly_authorization);
    let readonly_request = SupervisorPilotLaunchRequest {
        project_root: crate::STATION_3B_READONLY_PROJECT_ROOT.to_string(),
        workflow_id: readonly_authorization.workflow_id.clone(),
        authorization_id: readonly_authorization.authorization_id.clone(),
        model_id: None,
        reasoning_effort: "medium".to_string(),
    };
    assert!(supervisor_pilot_readonly_reviewer_task_for_authorization(
        &supervisor_pilot_planned_task_id(&readonly_authorization.authorization_id),
        &readonly_request,
        &readonly_proposal,
        &readonly_authorization,
    )
    .is_none());

    let opening_message = assemble_opening_message(&fixture_context());
    assert!(!opening_message.contains("字节级只读复核纪律"));
}
