// M5-C keeps every test artifact under fresh_root()/DbPrimaryFixture and uses only
// the replaceable Phase-B process runner.  The mock's own warning is the evidence
// that no real process was spawned; do not infer that from modeled attempt flags.

struct M5cTempRoot(PathBuf);

impl Drop for M5cTempRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct M5cTempMockPhaseBRunner;

impl crate::codex_local_runner::CodexLocalPhaseBProcessRunner for M5cTempMockPhaseBRunner {
    fn run_phase_b(
        &self,
        _request: &crate::CodexLocalExecutionRequest,
        _command_plan: &crate::CodexLocalCommandPlan,
        _prompt_body: &str,
        last_message_path: &Path,
        _timeout_ms: Option<i64>,
    ) -> crate::codex_local_runner::CodexLocalPhaseBProcessResult {
        if let Some(parent) = last_message_path.parent() {
            fs::create_dir_all(parent).expect("create temp mock last-message parent");
        }
        fs::write(last_message_path, "M5-C temp mock runner only\n")
            .expect("write temp mock last-message");
        crate::codex_local_runner::CodexLocalPhaseBProcessResult {
            runner_kind: "m5c_temp_mock_phase_b_runner".to_string(),
            status: "succeeded".to_string(),
            exit_code: Some(0),
            timed_out: false,
            prompt_sent: false,
            real_codex_executed: false,
            writes_codex_home: false,
            writes_project_files: false,
            readback_status: "succeeded".to_string(),
            readback_attempted: true,
            readback_result_count: Some(1),
            last_message_path: Some(last_message_path.display().to_string()),
            failure_code: None,
            failure_message: None,
            retryable: false,
            user_action_required: false,
            warnings: vec!["m5c_temp_mock_no_real_process_spawned".to_string()],
        }
    }
}

fn m5c_temp_k2_config(
    project_root: &str,
    project_id: &str,
    workflow_id: &str,
    work_item_id: &str,
    execution_point_id: &str,
) -> crate::real_execution_command::K2ExecutionPointConfig {
    let mut config = crate::real_execution_command::k2_execution_point_config(execution_point_id)
        .expect("load canonical K2 fixture config");
    config.project_root = project_root.to_string();
    config.project_id = project_id.to_string();
    config.workflow_id = workflow_id.to_string();
    config.run_unit_id = work_item_id.to_string();
    config.node_id = format!("node:m5c-temp:{}", config.operation);
    config.baseline_hashes = vec!["m5c_temp_fixture_only".to_string()];
    crate::real_execution_command::validate_k2_execution_point_config(&config)
        .expect("temp K2 config must preserve frozen safety shape");
    config
}

fn m5c_product_store_revision(workflow_state_path: &Path, generated_at: &str) -> i64 {
    crate::real_execution_command::load_real_execution_product_command_store(
        workflow_state_path,
        generated_at,
    )
    .expect("read product command store")
    .0
    .revision
}

fn assert_m5c_fake_runner_evidence(workflow_state_path: &Path, continuation_id: &str) {
    let continuation_store =
        crate::session_continuation_store::load_store(workflow_state_path, "2026-07-14T00:10:00Z")
            .expect("read continuation sidecar after mock Phase B");
    let attempt = continuation_store
        .attempts
        .iter()
        .rev()
        .find(|attempt| attempt.continuation_id == continuation_id)
        .expect("mock Phase B continuation attempt");
    assert_eq!(attempt.runner_kind, "m5c_temp_mock_phase_b_runner");
    assert!(
        attempt
            .warnings
            .iter()
            .any(|warning| warning == "m5c_temp_mock_no_real_process_spawned"),
        "mock runner warning is the no-real-process evidence: {:?}",
        attempt.warnings
    );
}

fn run_m5c_resume_fake_chain(
    workflow_state_path: &Path,
    config: &crate::real_execution_command::K2ExecutionPointConfig,
    timestamp_prefix: &str,
) {
    let prepare = crate::real_execution_command::prepare_real_execution_product_command_at(
        workflow_state_path,
        &crate::real_execution_command::k2_prepare_input(
            config,
            Some(m5c_product_store_revision(
                workflow_state_path,
                timestamp_prefix,
            )),
            Some(format!("{timestamp_prefix}:00Z")),
        )
        .expect("build temp resume prepare input"),
    )
    .expect("prepare temp resume product command");
    assert_eq!(prepare.status, "prepared");
    let product_command_id = prepare
        .product_command_id
        .clone()
        .expect("prepared resume command id");

    let decision =
        crate::real_execution_command::record_real_execution_product_command_decision_at(
            workflow_state_path,
            &crate::real_execution_command::k2_decision_input(
                &product_command_id,
                prepare.store_revision,
                Some(format!("{timestamp_prefix}:01Z")),
            ),
        )
        .expect("record temp resume decision");
    assert_eq!(decision.status, "decision_recorded");

    let phase_a = crate::real_execution_command::run_real_execution_product_command_phase_a_at(
        workflow_state_path,
        &crate::real_execution_command::k2_phase_a_input(
            &product_command_id,
            decision.store_revision,
            Some(format!("{timestamp_prefix}:02Z")),
        ),
        &format!("{timestamp_prefix}:02Z"),
        "m5c-temp-resume-phase-a",
    )
    .expect("record temp resume Phase A");
    assert_eq!(phase_a.status, "phase_a_completed");

    let phase_b_input = crate::real_execution_command::k2_resume_phase_b_input(
        workflow_state_path,
        config,
        &product_command_id,
        phase_a.product_command_store_revision,
        phase_a
            .session_continuation_store_revision
            .expect("resume Phase A continuation revision"),
        Some(format!("{timestamp_prefix}:03Z")),
    )
    .expect("build temp resume Phase B input");
    let last_message_path = workflow_state_path
        .parent()
        .expect("temp workflow state parent")
        .join("runtime")
        .join("m5c-temp-resume.last-message.txt");
    let phase_b =
        crate::real_execution_command::run_real_execution_product_command_phase_b_with_runner(
            workflow_state_path,
            &phase_b_input,
            &format!("{timestamp_prefix}:03Z"),
            "m5c-temp-resume-phase-b",
            &last_message_path,
            &M5cTempMockPhaseBRunner,
        )
        .expect("record temp resume Phase B through mock runner");
    assert_eq!(phase_b.status, "phase_b_completed");
    assert_m5c_fake_runner_evidence(
        workflow_state_path,
        phase_b
            .continuation_id
            .as_deref()
            .expect("resume mock continuation id"),
    );
    assert!(
        last_message_path.exists(),
        "mock runner writes only temp evidence"
    );
}

fn run_m5c_new_session_fake_chain(
    workflow_state_path: &Path,
    config: &crate::real_execution_command::K2ExecutionPointConfig,
    timestamp_prefix: &str,
) {
    let prepare = crate::real_execution_command::prepare_real_execution_product_command_at(
        workflow_state_path,
        &crate::real_execution_command::k2_prepare_input(
            config,
            Some(m5c_product_store_revision(
                workflow_state_path,
                timestamp_prefix,
            )),
            Some(format!("{timestamp_prefix}:00Z")),
        )
        .expect("build temp new-session prepare input"),
    )
    .expect("prepare temp new-session product command");
    assert_eq!(prepare.status, "prepared");
    let product_command_id = prepare
        .product_command_id
        .clone()
        .expect("prepared new-session command id");

    let decision =
        crate::real_execution_command::record_real_execution_product_command_decision_at(
            workflow_state_path,
            &crate::real_execution_command::k2_decision_input(
                &product_command_id,
                prepare.store_revision,
                Some(format!("{timestamp_prefix}:01Z")),
            ),
        )
        .expect("record temp new-session decision");
    assert_eq!(decision.status, "decision_recorded");

    let phase_a = crate::real_execution_command::run_real_execution_product_command_phase_a_at(
        workflow_state_path,
        &crate::real_execution_command::k2_phase_a_input(
            &product_command_id,
            decision.store_revision,
            Some(format!("{timestamp_prefix}:02Z")),
        ),
        &format!("{timestamp_prefix}:02Z"),
        "m5c-temp-new-session-phase-a",
    )
    .expect("record temp new-session Phase A");
    assert_eq!(phase_a.status, "phase_a_completed");

    let phase_b_input = crate::real_execution_command::k2_new_session_phase_b_input(
        workflow_state_path,
        config,
        &product_command_id,
        phase_a.product_command_store_revision,
        phase_a
            .session_continuation_store_revision
            .expect("new-session Phase A continuation revision"),
        Some(format!("{timestamp_prefix}:03Z")),
    )
    .expect("build temp new-session Phase B input");
    let last_message_path = workflow_state_path
        .parent()
        .expect("temp workflow state parent")
        .join("runtime")
        .join("m5c-temp-new-session.last-message.txt");
    let phase_b = crate::real_execution_command::run_real_execution_product_command_new_session_phase_b_with_runner(
        workflow_state_path,
        &phase_b_input,
        &format!("{timestamp_prefix}:03Z"),
        "m5c-temp-new-session-phase-b",
        &last_message_path,
        &M5cTempMockPhaseBRunner,
    )
    .expect("record temp new-session Phase B through mock runner");
    assert_eq!(phase_b.status, "phase_b_completed");
    assert_m5c_fake_runner_evidence(
        workflow_state_path,
        phase_b
            .continuation_id
            .as_deref()
            .expect("new-session mock continuation id"),
    );
    assert!(
        last_message_path.exists(),
        "mock runner writes only temp evidence"
    );
}

fn assert_m5c_tables_green_and_populated(
    report: &DbJsonReconciliationReport,
    table_names: &[&str],
) {
    assert!(report.is_green(), "{report:?}");
    for table_name in table_names {
        let table = report
            .tables
            .iter()
            .find(|table| table.table_name == *table_name)
            .unwrap_or_else(|| panic!("missing M5-C reconciliation table {table_name}"));
        assert!(table.db_count > 0, "{table:?}");
        assert_eq!(table.db_count, table.json_count, "{table:?}");
        assert_eq!(table.matched_count, table.db_count, "{table:?}");
        assert!(table.db_leading.is_empty(), "{table:?}");
        assert!(table.json_leading.is_empty(), "{table:?}");
        assert!(table.hash_mismatches.is_empty(), "{table:?}");
    }
}

fn m5c_global_review_record(
    fixture: &DbPrimaryFixture,
) -> crate::global_supervisor_review_store::GlobalSupervisorReviewRecord {
    crate::global_supervisor_review_store::GlobalSupervisorReviewRecord {
        review_id: "review:m5c:db-primary".to_string(),
        project_id: fixture.project_id.clone(),
        workflow_id: fixture.workflow_id.clone(),
        chain_started_at: "1700000000000".to_string(),
        status: "ready".to_string(),
        overall: "pass".to_string(),
        summary: "M5-C temp DB-primary supervisor review".to_string(),
        suggested_action: "none".to_string(),
        human_note: String::new(),
        tasks: vec![
            crate::global_supervisor_review_store::GlobalSupervisorTaskVerdict {
                title: "temp fixture".to_string(),
                verdict: "ok".to_string(),
                comment: "DB before JSON projection".to_string(),
            },
        ],
        unavailable_reason: None,
        model: "m5c-test".to_string(),
        profile_version: "m5c-test-v1".to_string(),
        created_at_ms: 0,
        updated_at_ms: 0,
    }
}

fn m5c_global_boundary_record(
    fixture: &DbPrimaryFixture,
) -> crate::global_supervisor_review_store::GlobalSupervisorBoundaryReviewRecord {
    crate::global_supervisor_review_store::GlobalSupervisorBoundaryReviewRecord {
        review_id: "boundary-review:m5c:db-primary".to_string(),
        project_id: fixture.project_id.clone(),
        proposal_id: "proposal:m5c:db-primary".to_string(),
        status: "ready".to_string(),
        verdict: "looks_ok".to_string(),
        points: vec!["temp fixture only".to_string()],
        summary: "M5-C temp DB-primary boundary review".to_string(),
        unavailable_reason: None,
        model: "m5c-test".to_string(),
        profile_version: "m5c-test-v1".to_string(),
        created_at_ms: 0,
        updated_at_ms: 0,
    }
}

#[test]
fn m5c_global_supervisor_mode_on_writes_four_tables_then_db_leading_replays() {
    let _serial = test_lock().lock().expect("storage mode test lock");
    let fixture = db_primary_fixture("m5c-global-supervisor-mode-on");

    crate::global_supervisor_review_store::upsert_review(
        &fixture.state_path,
        m5c_global_review_record(&fixture),
        "m5c-temp-test",
        1_700_000_010_000,
    )
    .expect("write DB-primary supervisor review");
    crate::global_supervisor_review_store::upsert_boundary_review(
        &fixture.state_path,
        m5c_global_boundary_record(&fixture),
        "m5c-temp-test",
        1_700_000_010_001,
    )
    .expect("write DB-primary supervisor boundary review");

    let report = reconcile_db_vs_json(&fixture.config).expect("reconcile supervisor bridge");
    assert_m5c_tables_green_and_populated(
        &report,
        &[
            "supervisor_reviews",
            "supervisor_review_audit_events",
            "supervisor_boundary_reviews",
            "supervisor_boundary_audit_events",
        ],
    );

    let review_sidecar = crate::global_supervisor_review_store::sidecar_path(&fixture.state_path)
        .expect("global supervisor review sidecar path");
    fs::remove_file(&review_sidecar).expect("remove only temp global review sidecar for replay");
    clear_storage_mode_cache_for_path_for_tests(&fixture.state_path);
    initialize_for_startup(&fixture.state_path).expect("replay DB-leading global review sidecar");
    assert!(
        review_sidecar.exists(),
        "DB-leading global review replay writes sidecar"
    );
    let replay_report = reconcile_db_vs_json(&fixture.config).expect("reconcile replayed review");
    assert_m5c_tables_green_and_populated(
        &replay_report,
        &[
            "supervisor_reviews",
            "supervisor_review_audit_events",
            "supervisor_boundary_reviews",
            "supervisor_boundary_audit_events",
        ],
    );
}

#[test]
fn m5c_global_supervisor_mode_off_keeps_json_only_and_creates_no_temp_db() {
    let _serial = test_lock().lock().expect("storage mode test lock");
    let root = fresh_root("m5c-global-supervisor-json-only");
    let _cleanup = M5cTempRoot(root.clone());
    let project_root = root.join("fixture-project").display().to_string();
    let (state_path, project_id, workflow_id, _work_item_id) =
        bootstrap_json_state(&root, &project_root);
    let fixture = DbPrimaryFixture {
        root: root.clone(),
        state_path: state_path.clone(),
        project_root,
        project_id,
        workflow_id,
        work_item_id: "work-item:m5c-json-only".to_string(),
        config: db_primary_config(&state_path),
    };
    clear_storage_mode_cache_for_path_for_tests(&fixture.state_path);

    crate::global_supervisor_review_store::upsert_review(
        &fixture.state_path,
        m5c_global_review_record(&fixture),
        "m5c-temp-test",
        1_700_000_011_000,
    )
    .expect("write JSON-only supervisor review");
    crate::global_supervisor_review_store::upsert_boundary_review(
        &fixture.state_path,
        m5c_global_boundary_record(&fixture),
        "m5c-temp-test",
        1_700_000_011_001,
    )
    .expect("write JSON-only supervisor boundary review");

    assert!(
        crate::global_supervisor_review_store::sidecar_path(&fixture.state_path)
            .expect("JSON-only global review sidecar")
            .exists()
    );
    assert!(
        !fixture.config.db_path.exists(),
        "JSON-only mode must not create a temp workbench SQLite DB"
    );
}

#[test]
fn m5c_global_supervisor_projection_failure_blocks_then_db_leading_replays() {
    let _serial = test_lock().lock().expect("storage mode test lock");
    let fixture = db_primary_fixture("m5c-global-supervisor-projection-failure");
    let review_sidecar = crate::global_supervisor_review_store::sidecar_path(&fixture.state_path)
        .expect("global supervisor review sidecar path");

    // Make only the temporary projection target a directory.  The bridge first commits its
    // four-table Immediate transaction, then its real complete_db_primary_json_projection
    // closure fails while trying to replace this path.
    fs::create_dir_all(&review_sidecar).expect("inject temp JSON projection failure target");
    let error = crate::global_supervisor_review_store::upsert_review(
        &fixture.state_path,
        m5c_global_review_record(&fixture),
        "m5c-temp-test",
        1_700_000_012_000,
    )
    .expect_err("DB commit followed by JSON projection failure must fail closed");
    assert!(
        error.contains("全局主管复核") || error.contains("global"),
        "actual M5-C projection error: {error}"
    );
    assert_db_primary_health_blocked(&fixture.state_path, "global_supervisor_review");

    let connection =
        Connection::open_with_flags(&fixture.config.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .expect("open DB after M5-C projection failure");
    let review_rows: i64 = connection
        .query_row("SELECT COUNT(*) FROM supervisor_reviews", [], |row| {
            row.get(0)
        })
        .expect("count Immediate-committed supervisor review rows");
    let audit_rows: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM supervisor_review_audit_events",
            [],
            |row| row.get(0),
        )
        .expect("count Immediate-committed supervisor audit rows");
    assert_eq!(
        review_rows, 1,
        "repository commit precedes projection failure"
    );
    assert_eq!(
        audit_rows, 1,
        "repository audit commit precedes projection failure"
    );

    fs::remove_dir(&review_sidecar).expect("clear only temp injected projection failure target");
    clear_storage_mode_cache_for_path_for_tests(&fixture.state_path);
    initialize_for_startup(&fixture.state_path)
        .expect("startup must replay M5-C DB-leading rows after projection failure");
    assert!(
        review_sidecar.exists(),
        "startup replay restores global review sidecar"
    );
    let report = reconcile_db_vs_json(&fixture.config).expect("reconcile recovered M5-C bridge");
    assert_m5c_tables_green_and_populated(
        &report,
        &["supervisor_reviews", "supervisor_review_audit_events"],
    );
}

#[test]
fn m5c_bcd_db_primary_temp_mock_e2e_hits_resume_and_new_session_then_reconciles() {
    let _serial = test_lock().lock().expect("storage mode test lock");
    let fixture = db_primary_fixture("m5c-bcd-temp-mock-e2e");
    let resume = m5c_temp_k2_config(
        &fixture.project_root,
        &fixture.project_id,
        &fixture.workflow_id,
        &fixture.work_item_id,
        "stage-k-k2-r1-mario-test-resume-read-only",
    );
    let new_session = m5c_temp_k2_config(
        &fixture.project_root,
        &fixture.project_id,
        &fixture.workflow_id,
        &fixture.work_item_id,
        "stage-k-k2-n1-isolated-new-session-read-only",
    );

    // Together these paths exercise D's prepare, decision, both Phase-A branches,
    // resume Phase-B and new-session Phase-B writes.  The only runner is the mock above.
    run_m5c_resume_fake_chain(&fixture.state_path, &resume, "2026-07-14T00:00");
    run_m5c_new_session_fake_chain(&fixture.state_path, &new_session, "2026-07-14T00:01");

    let table_names = [
        "session_continuations",
        "session_continuation_attempts",
        "session_continuation_audit_events",
        "runtime_log_entries",
        "runtime_log_summaries",
        "product_commands",
        "product_command_previews",
        "product_command_decisions",
        "product_command_attempts",
    ];
    clear_storage_mode_cache_for_path_for_tests(&fixture.state_path);
    initialize_for_startup(&fixture.state_path)
        .expect("restart reconciliation after temp mock E2E");
    let report = reconcile_db_vs_json(&fixture.config).expect("reconcile temp mock E2E");
    assert_m5c_tables_green_and_populated(&report, &table_names);

    // Delete only fixture sidecars, then prove DB-leading startup restores B/C/D.
    let sidecars = [
        crate::session_continuation_store::sidecar_path(&fixture.state_path)
            .expect("temp continuation sidecar path"),
        crate::runtime_log_store::sidecar_path(&fixture.state_path)
            .expect("temp runtime sidecar path"),
        crate::real_execution_command::real_execution_product_command_sidecar_path(
            &fixture.state_path,
        )
        .expect("temp product command sidecar path"),
    ];
    for sidecar in &sidecars {
        fs::remove_file(sidecar).expect("remove only temp M5-C sidecar for DB-leading replay");
    }
    clear_storage_mode_cache_for_path_for_tests(&fixture.state_path);
    initialize_for_startup(&fixture.state_path).expect("DB-leading B/C/D replay after restart");
    for sidecar in &sidecars {
        assert!(
            sidecar.exists(),
            "DB-leading replay restores {}",
            sidecar.display()
        );
    }
    let replay_report = reconcile_db_vs_json(&fixture.config).expect("reconcile replayed B/C/D");
    assert_m5c_tables_green_and_populated(&replay_report, &table_names);
}

#[test]
fn m5c_bcd_json_only_temp_mock_counterpart_creates_no_sqlite_db() {
    let _serial = test_lock().lock().expect("storage mode test lock");
    let root = fresh_root("m5c-bcd-json-only");
    let _cleanup = M5cTempRoot(root.clone());
    let project_root = root.join("fixture-project").display().to_string();
    let (state_path, project_id, workflow_id, work_item_id) =
        bootstrap_json_state(&root, &project_root);
    let config = m5c_temp_k2_config(
        &project_root,
        &project_id,
        &workflow_id,
        &work_item_id,
        "stage-k-k2-r1-mario-test-resume-read-only",
    );
    clear_storage_mode_cache_for_path_for_tests(&state_path);
    run_m5c_resume_fake_chain(&state_path, &config, "2026-07-14T00:02");

    assert!(crate::session_continuation_store::sidecar_path(&state_path)
        .expect("JSON-only continuation sidecar path")
        .exists());
    assert!(crate::runtime_log_store::sidecar_path(&state_path)
        .expect("JSON-only runtime sidecar path")
        .exists());
    assert!(
        crate::real_execution_command::real_execution_product_command_sidecar_path(&state_path)
            .expect("JSON-only product command sidecar path")
            .exists()
    );
    assert!(
        !db_primary_config(&state_path).db_path.exists(),
        "JSON-only B/C/D counterpart must not create SQLite"
    );
}
