#[test]
fn m5b_plan_authorization_inspect_db_primary_reconciles_after_restart() {
    let _serial = test_lock().lock().expect("storage mode test lock");
    let fixture = db_primary_fixture("m5b-plan-authorization-inspect");
    let proposal = crate::project_consultation_proposal_store::create_proposal(
        &fixture.state_path,
        &proposal_input(&fixture),
        1_700_000_001_000,
        "m5b-inspect-proposal",
    )
    .expect("create DB-primary proposal");
    let authorization = crate::plan_authorization_store::create_authorization(
        &fixture.state_path,
        &authorization_input(&fixture, &proposal.proposal.proposal_id),
        1_700_000_001_001,
        "m5b-inspect-authorization",
    )
    .expect("create DB-primary authorization");
    crate::plan_authorization_store::record_user_confirmation(
        &fixture.state_path,
        &crate::RecordPlanAuthorizationUserConfirmationInput {
            project_root: fixture.project_root.clone(),
            authorization_id: authorization.authorization.authorization_id.clone(),
            actor_id: "m5b-test".to_string(),
            confirmation_summary: "M5-B inspect approval".to_string(),
            expected_store_revision: None,
        },
        1_700_000_001_002,
        "m5b-inspect-confirmation",
    )
    .expect("confirm DB-primary authorization");
    crate::plan_authorization_store::record_global_boundary_review(
        &fixture.state_path,
        &crate::RecordPlanAuthorizationGlobalBoundaryReviewInput {
            project_root: fixture.project_root.clone(),
            authorization_id: authorization.authorization.authorization_id,
            actor_id: "m5b-test".to_string(),
            review_status: "approved".to_string(),
            summary: "M5-B inspect review".to_string(),
            source_proposal_id: Some(proposal.proposal.proposal_id),
            checklist: None,
            findings: vec![],
            reviewed_scope_fingerprint: None,
            expected_store_revision: None,
        },
        1_700_000_001_003,
        "m5b-inspect-boundary-review",
    )
    .expect("approve DB-primary authorization");

    let input = crate::AutoDispatchGuardInput {
        project_id: fixture.project_id.clone(),
        workflow_id: fixture.workflow_id.clone(),
        work_item_id: fixture.work_item_id.clone(),
        task_package_id: Some("artifact:m5b-inspect".to_string()),
        task_package_kind: Some("task_package".to_string()),
        target_role_id: "codex-dev".to_string(),
        target_agent_id: None,
        requested_read_roots: vec![fixture.project_root.clone()],
        requested_write_roots: vec![fixture.project_root.clone()],
        requested_tools: vec!["read_file".to_string()],
        requested_checks: vec![],
        triggered_stop_conditions: vec![],
        dispatch_kind: "prepare_offline".to_string(),
    };
    let result = crate::plan_authorization_store::inspect_auto_dispatch_authorization(
        &fixture.state_path,
        &input,
        1_700_000_001_004,
        "m5b-inspect-auto-dispatch",
    )
    .expect("inspect approved scope through DB-primary bridge");
    assert_eq!(result.status, "authorized", "{:?}", result.reasons);
    let authorization_store = crate::plan_authorization_store::load_store(
        &fixture.state_path,
        1_700_000_001_005,
    )
    .expect("read authorization sidecar");
    let audit = authorization_store
        .audit_events
        .iter()
        .find(|event| {
            event.event_type == "auto_dispatch_scope_checked"
                && event.work_item_id.as_deref() == Some(input.work_item_id.as_str())
        })
        .expect("sidecar inspect audit");
    let connection = Connection::open_with_flags(&fixture.config.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .expect("open DB-primary audit table");
    let db_audit_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM workflow_audit_events WHERE event_id = ?1 AND target_kind = 'plan_authorization'",
            params![audit.audit_event_id],
            |row| row.get(0),
        )
        .expect("query bridged inspect audit");
    assert_eq!(db_audit_count, 1, "inspect audit must commit before JSON projection");

    clear_storage_mode_cache_for_tests();
    initialize_for_startup(&fixture.state_path)
        .expect("restart reconciliation after DB-primary inspect");
    let report = reconcile_db_vs_json(&fixture.config).expect("reconcile after restart");
    assert!(report.is_green(), "{report:?}");
}

#[test]
fn m5b_supervisor_orchestrator_db_ahead_replays_sidecar_and_reconciles() {
    let _serial = test_lock().lock().expect("storage mode test lock");
    let fixture = db_primary_fixture("m5b-supervisor-orchestrator-db-ahead");
    let session = json!({
        "run_id": "supervisor:m5b:db-ahead",
        "project_root": fixture.project_root.clone(),
        "workflow_id": fixture.workflow_id.clone(),
        "authorization_id": "plan-auth:m5b-db-ahead",
        "model_id": "",
        "reasoning_effort": "",
        "max_active_workers": 0,
        "max_follow_ups_per_worker": 0,
        "max_runtime_minutes": 0,
        "launch_status": "running",
        "started_at_ms": 1_700_000_001_100_i64,
        "ended_at_ms": Value::Null,
        "termination_reason": "",
        "workers": [],
        "final_marks": []
    });
    let audit = json!({
        "event_id": "audit:m5b:supervisor-db-ahead",
        "actor": "supervisor_orchestrator",
        "run_id": "supervisor:m5b:db-ahead",
        "tool": "m5b_test",
        "parameter_summary": "DB-only supervisor projection fixture",
        "result_summary": "replay this audit into sidecar",
        "result_status": "accepted",
        "created_at_ms": 1_700_000_001_102_i64
    });
    let repository = WorkbenchSqliteRepository::open_confirmed(&fixture.config.repository_config())
        .expect("open DB-primary repository");
    repository
        .record_supervisor_orchestrator_delta(Some(&session), &[audit.clone()], None)
        .expect("write DB-leading supervisor projection fixture");

    clear_storage_mode_cache_for_tests();
    initialize_for_startup(&fixture.state_path)
        .expect("startup must replay DB-leading supervisor projection");
    let (sessions, audits) =
        crate::mcp::supervisor_orchestrator::db_primary_projection_records(&fixture.state_path)
            .expect("read replayed supervisor sidecar");
    assert!(sessions.iter().any(|record| {
        record.get("run_id").and_then(Value::as_str) == Some("supervisor:m5b:db-ahead")
    }));
    assert!(audits.iter().any(|record| {
        record.get("event_id").and_then(Value::as_str)
            == Some("audit:m5b:supervisor-db-ahead")
    }));
    let report = reconcile_db_vs_json(&fixture.config).expect("reconcile replayed supervisor rows");
    assert!(report.is_green(), "{report:?}");
}
