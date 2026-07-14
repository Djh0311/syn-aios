use rusqlite::{params, Connection, OpenFlags};

fn m5b_empty_workflow_state() -> Value {
    json!({
        "schema_version": "workflow_state_v0",
        "workflow_version": 1,
        "revision": 0,
        "projects": [],
        "agent_adapters": [],
        "workflows": [],
        "nodes": [],
        "edges": [],
        "work_items": [],
        "artifacts": [],
        "reviews": [],
        "workflow_node_session_bindings": [],
        "workflow_node_dispatches": [],
        "execution_attempts": [],
        "workflow_chain_runs": [],
        "workflow_execution_controls": [],
        "permission_requests": [],
        "capabilities": [],
        "harness_resources": [],
        "audit_events": []
    })
}

fn enable_m5b_db_primary(
    fixture: &mut Fixture,
) -> crate::workbench_sqlite_storage_mode::DbPrimaryJsonProjectionConfig {
    let state = m5b_empty_workflow_state();
    fs::write(
        &fixture.state_path,
        serde_json::to_vec_pretty(&state).expect("serialize M5-B workflow state"),
    )
    .expect("write M5-B workflow state");
    let canonical_state_path = fs::canonicalize(&fixture.state_path).expect("canonical workflow state");
    fixture.state_path = canonical_state_path.clone();
    fixture.config.supervisor_workflow_state_path = Some(canonical_state_path);
    let config_path = crate::workbench_sqlite_storage_mode::storage_mode_path(&fixture.state_path)
        .expect("storage mode path");
    let runtime_artifacts = config_path.parent().expect("runtime artifacts parent");
    fs::create_dir_all(runtime_artifacts).expect("create runtime artifacts");
    let canonical_runtime_artifacts =
        fs::canonicalize(runtime_artifacts).expect("canonical runtime artifacts");
    let config = crate::workbench_sqlite_storage_mode::DbPrimaryJsonProjectionConfig {
        workflow_state_path: fixture.state_path.clone(),
        confirmed_workflow_state_path: fixture.state_path.clone(),
        db_path: canonical_runtime_artifacts.join("workbench.sqlite"),
        confirmed_db_path: canonical_runtime_artifacts.join("workbench.sqlite"),
        denied_path_markers: vec![],
    };
    fs::write(
        config_path,
        serde_json::to_vec_pretty(&json!({
            "schema_version": crate::workbench_sqlite_storage_mode::STORAGE_MODE_SCHEMA_VERSION,
            "mode": "db_primary_json_projection",
            "workflow_state_path": config.workflow_state_path.clone(),
            "confirmed_workflow_state_path": config.confirmed_workflow_state_path.clone(),
            "db_path": config.db_path.clone(),
            "confirmed_db_path": config.confirmed_db_path.clone(),
            "denied_path_markers": config.denied_path_markers.clone(),
        }))
        .expect("serialize storage mode config"),
    )
    .expect("write storage mode config");
    let repository = crate::workbench_sqlite_repository::WorkbenchSqliteRepository::open_confirmed(
        &crate::workbench_sqlite_repository::ConfirmedWorkbenchSqliteRepositoryConfig {
            db_path: config.db_path.clone(),
            confirmed_db_path: config.confirmed_db_path.clone(),
            denied_path_markers: vec![],
        },
    )
    .expect("initialize M5-B fixture DB");
    let authorization_store = crate::plan_authorization_store::load_store(
        &fixture.state_path,
        now_ms(),
    )
    .expect("load active authorization sidecar");
    let connection = Connection::open(&config.db_path).expect("open M5-B fixture DB");
    for authorization in authorization_store.authorizations {
        let value = serde_json::to_value(&authorization).expect("authorization value");
        connection
            .execute(
                "INSERT INTO plan_authorizations (authorization_id, source_proposal_id, source_id, record_hash, record_json)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    authorization.authorization_id,
                    authorization.source_proposal_id,
                    "m5b-supervisor-test-seed",
                    crate::workbench_sqlite_importer::canonical_json_hash(&value),
                    serde_json::to_string(&value).expect("authorization JSON"),
                ],
            )
            .expect("seed active authorization");
    }
    repository
        .record_workflow_state_delta_with_audit(&m5b_empty_workflow_state(), &state, None)
        .expect("seed empty workflow projection");
    drop(connection);
    crate::workbench_sqlite_storage_mode::clear_storage_mode_cache_for_tests();
    crate::workbench_sqlite_storage_mode::initialize_for_startup(&fixture.state_path)
        .expect("initialize M5-B DB primary fixture");
    config
}

#[test]
fn m5b_supervisor_orchestrator_db_primary_reserve_and_persist_bridge_then_restart_reconciles() {
    let _serial = crate::workbench_sqlite_storage_mode::storage_mode_test_lock()
        .lock()
        .expect("storage mode test lock");
    let mut fixture = Fixture::new();
    let db_config = enable_m5b_db_primary(&mut fixture);
    record_pilot_session_started(
        &fixture.config,
        &SupervisorPilotSessionLaunch {
            project_root: PROJECT.to_string(),
            workflow_id: WORKFLOW.to_string(),
            authorization_id: AUTH.to_string(),
            model_id: "m5b-test".to_string(),
            reasoning_effort: "medium".to_string(),
            workbench_executable_path: "/tmp/m5b-supervisor-test-workbench".to_string(),
            workbench_build_id: "m5b-supervisor-test-build".to_string(),
            supervisor_contract_version: "supervisor_action_proposal.v1".to_string(),
            supervisor_contract_sha256: "m5b-supervisor-contract".to_string(),
            worker_report_contract_sha256: "m5b-worker-report-contract".to_string(),
        },
    )
    .expect("start DB-primary supervisor session");
    let input = DispatchInput {
        project_root: PROJECT.to_string(),
        workflow_id: WORKFLOW.to_string(),
        authorization_id: AUTH.to_string(),
        node_id: NODE.to_string(),
        work_item_id: "work-1".to_string(),
        allowed_write: vec![PROJECT.to_string()],
    };
    let reservation_id = reserve_dispatch(&fixture.config, &input)
        .expect("reserve dispatch through DB-primary bridge");
    let reserved_store = load_store(&fixture.config).expect("read reserved sidecar session");
    assert!(reserved_store
        .sessions
        .iter()
        .flat_map(|session| session.workers.iter())
        .any(|worker| worker.worker_id == reservation_id && worker.state == "reserved"));
    let connection = Connection::open_with_flags(&db_config.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .expect("open DB-primary supervisor table");
    let reserved_json: String = connection
        .query_row(
            "SELECT record_json FROM supervisor_orchestrator_sessions WHERE run_id = ?1",
            params![fixture.config.run_id],
            |row| row.get(0),
        )
        .expect("reserved session must reach DB before projection");
    assert!(serde_json::from_str::<Value>(&reserved_json)
        .expect("reserved session JSON")["workers"]
        .as_array()
        .is_some_and(|workers| workers.iter().any(|worker| worker["state"] == "reserved")));
    drop(connection);

    complete_dispatch(
        &fixture.config,
        &reservation_id,
        &WorkerLaunch {
            worker_id: "worker-1".to_string(),
            native_thread_id: "thread-1".to_string(),
            dispatch_id: "dispatch-1".to_string(),
            canonical_work_item_id: String::new(),
            state: "completed".to_string(),
            initial_report: None,
            result_summary: "M5-B fake dispatch".to_string(),
        },
    )
    .expect("complete reserved dispatch");
    let mut workflow_state =
        crate::read_workflow_state_value(&fixture.state_path).expect("read workflow state");
    workflow_state["audit_events"]
        .as_array_mut()
        .expect("workflow audit array")
        .push(json!({
            "event_id": "audit:m5b:supervisor-worker-report",
            "event_type": "worker_structured_report_recorded",
            "dispatch_id": "dispatch-1",
            "acceptance_status": "reported_completed",
            "executed_what": "M5-B test worker completed the reserved task",
            "changed_what": "M5-B test worker changed no real files",
            "reason": "M5-B structured report",
            "evidence_refs": ["m5b:test"],
            "open_issues": [],
            "permission_requests": [],
            "direction_risks": [],
            "follow_up_suggestions": []
        }));
    crate::write_m5b_batch2_workflow_state(
        &fixture.state_path,
        "m5b_supervisor_worker_report_seed",
        &workflow_state,
    )
    .expect("persist structured worker report source");
    let report = read_worker_report(&fixture.config, &json!({"worker_id": "worker-1"}))
        .expect("persist worker report through DB-primary bridge");
    assert_eq!(report["acceptance_status"], "reported_completed");
    let persisted_store = load_store(&fixture.config).expect("read persisted supervisor store");
    assert!(persisted_store
        .sessions
        .iter()
        .flat_map(|session| session.workers.iter())
        .find(|worker| worker.worker_id == "worker-1")
        .and_then(|worker| worker.last_report.as_ref())
        .is_some());
    let connection = Connection::open_with_flags(&db_config.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .expect("reopen DB-primary supervisor table");
    let persisted_json: String = connection
        .query_row(
            "SELECT record_json FROM supervisor_orchestrator_sessions WHERE run_id = ?1",
            params![fixture.config.run_id],
            |row| row.get(0),
        )
        .expect("persisted worker report session must reach DB");
    assert!(serde_json::from_str::<Value>(&persisted_json)
        .expect("persisted session JSON")["workers"]
        .as_array()
        .is_some_and(|workers| workers.iter().any(|worker| worker["last_report"].is_object())));
    let audit_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM supervisor_orchestrator_audit_events WHERE run_id = ?1",
            params![fixture.config.run_id],
            |row| row.get(0),
        )
        .expect("query supervisor audit rows");
    assert!(audit_count >= 1, "pilot audit must reach the DB-primary audit table");

    crate::workbench_sqlite_storage_mode::clear_storage_mode_cache_for_tests();
    crate::workbench_sqlite_storage_mode::initialize_for_startup(&fixture.state_path)
        .expect("restart reconciliation after supervisor writes");
    let report = crate::workbench_sqlite_storage_mode::reconcile_db_vs_json(&db_config)
        .expect("reconcile supervisor DB-primary projection");
    assert!(report.is_green(), "{report:?}");
}

#[test]
fn m5b_supervisor_orchestrator_same_run_db_ahead_replays_after_projection_interruption() {
    let _serial = crate::workbench_sqlite_storage_mode::storage_mode_test_lock()
        .lock()
        .expect("storage mode test lock");
    let mut fixture = Fixture::new();
    let db_config = enable_m5b_db_primary(&mut fixture);
    record_pilot_session_started(
        &fixture.config,
        &SupervisorPilotSessionLaunch {
            project_root: PROJECT.to_string(),
            workflow_id: WORKFLOW.to_string(),
            authorization_id: AUTH.to_string(),
            model_id: "m5b-test".to_string(),
            reasoning_effort: "medium".to_string(),
            workbench_executable_path: "/tmp/m5b-supervisor-test-workbench".to_string(),
            workbench_build_id: "m5b-supervisor-test-build".to_string(),
            supervisor_contract_version: "supervisor_action_proposal.v1".to_string(),
            supervisor_contract_sha256: "m5b-supervisor-contract".to_string(),
            worker_report_contract_sha256: "m5b-worker-report-contract".to_string(),
        },
    )
    .expect("start DB-primary supervisor session");
    let store = load_store(&fixture.config).expect("read projected supervisor session");
    let session = store
        .sessions
        .iter()
        .find(|session| session.run_id == fixture.config.run_id)
        .expect("projected supervisor session");
    let mut db_leading_session = serde_json::to_value(session).expect("session JSON");
    let previous_updated_at_ms = db_leading_session["updated_at_ms"]
        .as_i64()
        .expect("DB-primary session records carry freshness");
    db_leading_session["launch_status"] = Value::String("waiting_user".to_string());
    db_leading_session["updated_at_ms"] = Value::from(previous_updated_at_ms + 1);
    let repository = crate::workbench_sqlite_storage_mode::primary_repository_for_write(
        &fixture.state_path,
    )
    .expect("DB-primary repository gate")
    .expect("DB-primary repository");
    repository
        .record_supervisor_orchestrator_delta(Some(&db_leading_session), &[], None)
        .expect("commit DB-only same-run crash-window record");

    crate::workbench_sqlite_storage_mode::clear_storage_mode_cache_for_tests();
    crate::workbench_sqlite_storage_mode::initialize_for_startup(&fixture.state_path)
        .expect("restart replays DB-leading same-run session");
    let replayed_store = load_store(&fixture.config).expect("read replayed supervisor session");
    let replayed = replayed_store
        .sessions
        .iter()
        .find(|session| session.run_id == fixture.config.run_id)
        .expect("replayed same-run session");
    assert_eq!(replayed.launch_status, "waiting_user");
    assert_eq!(replayed.updated_at_ms, previous_updated_at_ms + 1);
    assert!(
        crate::workbench_sqlite_storage_mode::reconcile_db_vs_json(&db_config)
            .expect("reconcile replayed same-run session")
            .is_green()
    );
}
