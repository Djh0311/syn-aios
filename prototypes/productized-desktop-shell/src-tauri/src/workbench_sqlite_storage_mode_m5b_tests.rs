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

fn m5f1_workflow_audit(event_id: &str, event_type: &str) -> Value {
    json!({
        "event_id": event_id,
        "event_type": event_type,
        "target_ref": "m5f1-workflow-state",
        "actor_ref": "m5f1-test",
        "source_kind": "workspace_state",
        "permission_level": "user_confirmed_write",
        "created_at": "1700000001200",
        "reason": "exercise blocked JSON-only workflow-state fallback"
    })
}

fn write_m5f1_blocked_batch(
    batch: &str,
    path: &Path,
    phase: &str,
    candidate: &Value,
) -> Result<(), String> {
    match batch {
        "batch1" => crate::write_m5b_batch1_workflow_state(path, phase, candidate),
        "batch2" => crate::write_m5b_batch2_workflow_state(path, phase, candidate),
        _ => panic!("unknown M5-F1 batch fixture: {batch}"),
    }
}

fn m5f1_json_leading_blocked_fixture(label: &str) -> (DbPrimaryFixture, Vec<(&'static str, i64)>) {
    clear_storage_mode_cache_for_tests();
    let fixture = db_primary_fixture(label);
    let db_before_fallback = db_primary_row_counts(&fixture.config);
    let mut json_leading = crate::read_workflow_state_value(&fixture.state_path)
        .expect("read workflow state for JSON-leading fixture");
    append_workflow_state_row(
        &mut json_leading,
        "audit_events",
        m5f1_workflow_audit("audit:m5f1:json-leading", "m5f1_json_leading_fixture"),
    );
    crate::write_validated_workflow_state(&fixture.state_path, &json_leading)
        .expect("inject JSON-leading workflow state");
    clear_storage_mode_cache_for_tests();
    let startup_error = initialize_for_startup(&fixture.state_path)
        .expect_err("JSON-leading state must freeze DB-primary writes");
    assert!(
        startup_error.contains("db_primary_projection_blocked"),
        "{startup_error}"
    );
    (fixture, db_before_fallback)
}

#[test]
fn m5f1_auxiliary_degradation_failures_commit_original_candidate_once_for_both_batches() {
    let _serial = test_lock().lock().expect("storage mode test lock");
    for batch in ["batch1", "batch2"] {
        for failure in ["audit_append", "backup"] {
            clear_storage_mode_cache_for_tests();
            super::m5f1::clear_deferred_blocked_json_only_audit_append_failure_for_tests();
            let case = format!("m5f1-{batch}-{failure}");
            let (fixture, db_before_fallback) = m5f1_json_leading_blocked_fixture(&case);
            let mut candidate = crate::read_workflow_state_value(&fixture.state_path)
                .expect("read workflow state before blocked fallback");
            let expected_revision = candidate["revision"]
                .as_i64()
                .expect("workflow state revision");
            let business_event_id = format!("audit:{case}:business");
            append_workflow_state_row(
                &mut candidate,
                "audit_events",
                m5f1_workflow_audit(&business_event_id, "m5f1_original_business_written"),
            );

            let backups_path = fixture
                .state_path
                .parent()
                .expect("workflow-state parent")
                .join("backups");
            if failure == "audit_append" {
                super::m5f1::inject_deferred_blocked_json_only_audit_append_failure_for_tests();
            } else {
                if backups_path.exists() {
                    if backups_path.is_dir() {
                        fs::remove_dir_all(&backups_path)
                            .expect("remove fixture backups directory");
                    } else {
                        fs::remove_file(&backups_path).expect("remove fixture backups file");
                    }
                }
                fs::write(&backups_path, "m5f1 backup failure fixture")
                    .expect("replace fixture backups directory with file");
            }

            write_m5f1_blocked_batch(batch, &fixture.state_path, &case, &candidate)
                .expect("auxiliary audit failure must preserve the original business candidate");

            let persisted = crate::read_workflow_state_value(&fixture.state_path)
                .expect("read persisted blocked fallback state");
            assert_eq!(
                persisted["revision"].as_i64(),
                Some(expected_revision + 1),
                "{case} must consume exactly the one existing business CAS revision"
            );
            assert!(persisted["audit_events"]
                .as_array()
                .expect("workflow audit array")
                .iter()
                .any(|event| {
                    event.get("event_id").and_then(Value::as_str)
                        == Some(business_event_id.as_str())
                }));
            assert!(
                degradation_audits(&fixture.state_path).is_empty(),
                "{case} must leave an unpersisted degradation audit eligible for the next write"
            );
            assert_eq!(db_primary_row_counts(&fixture.config), db_before_fallback);
            assert_db_primary_health_blocked(
                &fixture.state_path,
                "db_json_reconciliation_not_green",
            );

            if failure == "backup" {
                fs::remove_file(&backups_path)
                    .expect("restore fixture backups path to absent state");
            }
            let mut fresh_candidate = crate::read_workflow_state_value(&fixture.state_path)
                .expect("read fresh state after auxiliary audit failure");
            let fresh_event_id = format!("audit:{case}:fresh");
            append_workflow_state_row(
                &mut fresh_candidate,
                "audit_events",
                m5f1_workflow_audit(&fresh_event_id, "m5f1_fresh_business_written"),
            );
            write_m5f1_blocked_batch(batch, &fixture.state_path, &case, &fresh_candidate)
                .expect("fresh candidate must persist the deferred degradation audit once");
            assert_eq!(
                degradation_audits(&fixture.state_path).len(),
                1,
                "{case} must retain an unrecorded degradation audit after preparation failure"
            );
            assert_eq!(db_primary_row_counts(&fixture.config), db_before_fallback);
            super::m5f1::clear_deferred_blocked_json_only_audit_append_failure_for_tests();
        }
    }
    clear_storage_mode_cache_for_tests();
}

#[test]
fn m5f1_blocked_json_only_commits_same_batch_workflow_write_with_one_degradation_audit() {
    let _serial = test_lock().lock().expect("storage mode test lock");
    clear_storage_mode_cache_for_tests();
    let fixture = db_primary_fixture("m5f1-blocked-same-batch");
    let db_before_fallback = db_primary_row_counts(&fixture.config);
    let mut json_leading = crate::read_workflow_state_value(&fixture.state_path)
        .expect("read workflow state for JSON-leading fixture");
    append_workflow_state_row(
        &mut json_leading,
        "audit_events",
        m5f1_workflow_audit("audit:m5f1:json-leading", "m5f1_json_leading_fixture"),
    );
    crate::write_validated_workflow_state(&fixture.state_path, &json_leading)
        .expect("inject JSON-leading workflow state");
    clear_storage_mode_cache_for_tests();
    let startup_error = initialize_for_startup(&fixture.state_path)
        .expect_err("JSON-leading state must freeze DB-primary writes");
    assert!(
        startup_error.contains("db_primary_projection_blocked"),
        "{startup_error}"
    );

    let mut candidate = crate::read_workflow_state_value(&fixture.state_path)
        .expect("read workflow state before blocked fallback");
    let expected_revision = candidate["revision"]
        .as_i64()
        .expect("workflow state revision");
    append_workflow_state_row(
        &mut candidate,
        "audit_events",
        m5f1_workflow_audit("audit:m5f1:same-batch", "m5f1_same_batch_business_written"),
    );

    crate::write_m5b_batch2_workflow_state(&fixture.state_path, "m5f1_same_batch", &candidate)
        .expect("blocked Batch 2 write must combine its audit with the business candidate");

    let persisted = crate::read_workflow_state_value(&fixture.state_path)
        .expect("read persisted blocked fallback state");
    assert_eq!(
        persisted["revision"].as_i64(),
        Some(expected_revision + 1),
        "the combined JSON-only write must consume one existing CAS revision"
    );
    assert!(persisted["audit_events"]
        .as_array()
        .expect("workflow audit array")
        .iter()
        .any(|event| event["event_id"] == "audit:m5f1:same-batch"));
    assert_eq!(degradation_audits(&fixture.state_path).len(), 1);
    assert_eq!(db_primary_row_counts(&fixture.config), db_before_fallback);
    assert_db_primary_health_blocked(&fixture.state_path, "db_json_reconciliation_not_green");

    let mut second_candidate = crate::read_workflow_state_value(&fixture.state_path)
        .expect("read state for Batch 1 fallback");
    append_workflow_state_row(
        &mut second_candidate,
        "audit_events",
        m5f1_workflow_audit("audit:m5f1:batch1", "m5f1_batch1_business_written"),
    );
    crate::write_m5b_batch1_workflow_state(&fixture.state_path, "m5f1_batch1", &second_candidate)
        .expect("blocked Batch 1 write must retain JSON-only fallback");
    assert_eq!(
        degradation_audits(&fixture.state_path).len(),
        1,
        "same-process fallback must not duplicate degradation audit"
    );
    assert_eq!(db_primary_row_counts(&fixture.config), db_before_fallback);

    clear_storage_mode_cache_for_tests();
    let restart_error = initialize_for_startup(&fixture.state_path)
        .expect_err("JSON-leading fallback must still block DB-primary restart");
    assert!(
        restart_error.contains("db_primary_projection_blocked"),
        "{restart_error}"
    );
    clear_storage_mode_cache_for_tests();
}

#[test]
fn m5f1_blocked_json_only_never_rebases_over_external_revision_conflict() {
    let _serial = test_lock().lock().expect("storage mode test lock");
    for batch in ["batch1", "batch2"] {
        clear_storage_mode_cache_for_tests();
        let case = format!("m5f1-external-conflict-{batch}");
        let fixture = db_primary_fixture(&case);
        let db_before_fallback = db_primary_row_counts(&fixture.config);
        let stale_event_id = format!("audit:{case}:stale");
        let mut stale_candidate = crate::read_workflow_state_value(&fixture.state_path)
            .expect("read workflow state for stale candidate");
        append_workflow_state_row(
            &mut stale_candidate,
            "audit_events",
            m5f1_workflow_audit(&stale_event_id, "m5f1_stale_business_must_not_write"),
        );

        let external_event_id = format!("audit:{case}:external");
        let mut external = crate::read_workflow_state_value(&fixture.state_path)
            .expect("read workflow state for external writer");
        append_workflow_state_row(
            &mut external,
            "audit_events",
            m5f1_workflow_audit(&external_event_id, "m5f1_external_writer"),
        );
        crate::write_validated_workflow_state(&fixture.state_path, &external)
            .expect("independent writer advances the real workflow revision");
        block_db_primary_writes(
            &fixture.state_path,
            &case,
            "injected blocked fixture reason",
        );

        let error = write_m5f1_blocked_batch(batch, &fixture.state_path, &case, &stale_candidate)
            .expect_err("real external revision conflict must not be retried or rebased");
        assert!(
            error.contains("workflow_state_revision_conflict"),
            "{error}"
        );
        let after_conflict = crate::read_workflow_state_value(&fixture.state_path)
            .expect("read state after preserved conflict");
        let audit_events = after_conflict["audit_events"]
            .as_array()
            .expect("workflow audit array");
        assert!(audit_events.iter().any(|event| {
            event.get("event_id").and_then(Value::as_str) == Some(external_event_id.as_str())
        }));
        assert!(!audit_events.iter().any(|event| {
            event.get("event_id").and_then(Value::as_str) == Some(stale_event_id.as_str())
        }));
        assert!(
            degradation_audits(&fixture.state_path).is_empty(),
            "failed CAS must not claim that its deferred audit was persisted"
        );
        assert_eq!(db_primary_row_counts(&fixture.config), db_before_fallback);
        assert_db_primary_health_blocked(&fixture.state_path, &case);

        let mut fresh_candidate = crate::read_workflow_state_value(&fixture.state_path)
            .expect("read fresh candidate after conflict");
        append_workflow_state_row(
            &mut fresh_candidate,
            "audit_events",
            m5f1_workflow_audit(
                &format!("audit:{case}:fresh"),
                "m5f1_fresh_business_written",
            ),
        );
        write_m5f1_blocked_batch(batch, &fixture.state_path, &case, &fresh_candidate)
            .expect("fresh JSON-only candidate can leave the required degradation audit");
        assert_eq!(degradation_audits(&fixture.state_path).len(), 1);
        assert_eq!(db_primary_row_counts(&fixture.config), db_before_fallback);
    }
    clear_storage_mode_cache_for_tests();
}
