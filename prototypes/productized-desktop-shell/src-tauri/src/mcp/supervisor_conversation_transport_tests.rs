fn shared_supervisor_conversation_config(fixture: &Fixture) -> McpServerConfig {
    let mut config = fixture.config.clone();
    config.run_id = "supervisor-conversation:offline-capability-plane".to_string();
    config
}

fn new_shared_supervisor_binding(config: &McpServerConfig) -> ConversationTurnBinding {
    ConversationTurnBinding::establish_supervisor_read_only(
        super::super::supervisor_conversation_binding::SupervisorConversationTurnInput {
            project_id: crate::project_id(PROJECT),
            project_root: PROJECT.to_string(),
            workflow_id: WORKFLOW.to_string(),
            turn_id: "turn:offline-capability-plane".to_string(),
            transport_attempt: 1,
            run_id: config.run_id.clone(),
            user_message_snapshot: "请整理一份待用户确认的离线方案。".to_string(),
            created_at_ms: now_ms(),
            max_runtime_minutes: super::super::supervisor_conversation_binding::SUPERVISOR_CONVERSATION_MAX_RUNTIME_MINUTES,
        },
    )
    .expect("host creates a frozen supervisor-read-only binding")
}

fn establish_starting_shared_supervisor_binding(config: &McpServerConfig) {
    establish_supervisor_conversation_turn_binding(config, new_shared_supervisor_binding(config))
        .expect("persist binding only in the fixture supervisor store");
}

fn establish_active_shared_supervisor_binding(config: &McpServerConfig) {
    establish_starting_shared_supervisor_binding(config);
    activate_supervisor_conversation_turn_binding(config, "thread:offline-capability-plane")
        .expect("host-observed thread activates the fixture binding");
}

struct DbPrimarySupervisorFixture {
    root: PathBuf,
    db_path: PathBuf,
    config: McpServerConfig,
}

impl Drop for DbPrimarySupervisorFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn db_primary_supervisor_fixture(existing_sessions: usize) -> DbPrimarySupervisorFixture {
    let root = std::env::temp_dir().join(format!(
        "shared-supervisor-binding-db-primary-{}-{}",
        std::process::id(),
        FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed),
    ));
    let state_dir = root.join("workflow-state");
    let runtime_dir = root.join("runtime-artifacts");
    fs::create_dir_all(&state_dir).expect("DB-primary fixture workflow-state directory");
    fs::create_dir_all(&runtime_dir).expect("DB-primary fixture runtime directory");
    let state_path = state_dir.join("workflow-state.v0.json");
    let workflow_state = json!({
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
    });
    fs::write(
        &state_path,
        serde_json::to_vec_pretty(&workflow_state)
            .expect("serialize DB-primary fixture workflow state"),
    )
    .expect("DB-primary fixture workflow state");
    let state_path = fs::canonicalize(state_path).expect("canonical DB-primary workflow state");
    let db_path = runtime_dir.join("workbench-state.v1.sqlite");
    fs::File::create(&db_path).expect("create DB-primary fixture database");
    let db_path = fs::canonicalize(db_path).expect("canonical DB-primary fixture database");
    let storage_mode_path = root.join("runtime-artifacts").join("storage-mode.v1.json");
    fs::write(
        storage_mode_path,
        serde_json::to_vec(&json!({
            "schema_version": "storage-mode.v1",
            "mode": "db_primary_json_projection",
            "workflow_state_path": state_path,
            "confirmed_workflow_state_path": state_path,
            "db_path": db_path,
            "confirmed_db_path": db_path,
            "denied_path_markers": [],
        }))
        .expect("serialize DB-primary fixture storage mode"),
    )
    .expect("write DB-primary fixture storage mode");
    crate::workbench_sqlite_storage_mode::clear_storage_mode_cache_for_tests();

    let repository = crate::workbench_sqlite_repository::WorkbenchSqliteRepository::open_confirmed(
        &crate::workbench_sqlite_repository::ConfirmedWorkbenchSqliteRepositoryConfig {
            db_path: db_path.clone(),
            confirmed_db_path: db_path.clone(),
            denied_path_markers: vec![],
        },
    )
    .expect("open DB-primary fixture repository");
    let mut store = empty_store(now_ms());
    for index in 0..existing_sessions {
        store.sessions.push(SupervisorSession {
            run_id: format!("historical-supervisor-session:{index}"),
            ..SupervisorSession::default()
        });
    }
    let sidecar = state_dir.join("supervisor-orchestrator.v1.json");
    write_store_atomic(&sidecar, &store, "seed-shared-supervisor-db-primary")
        .expect("write DB-primary fixture supervisor sidecar");
    for session in &store.sessions {
        let session =
            serde_json::to_value(session).expect("serialize historical supervisor session");
        repository
            .record_supervisor_orchestrator_delta(Some(&session), &[], None)
            .expect("seed historical supervisor DB session");
    }
    repository
        .record_workflow_state_delta_with_audit(&json!({}), &workflow_state, None)
        .expect("seed DB-primary workflow projection");
    crate::workbench_sqlite_storage_mode::clear_storage_mode_cache_for_tests();
    crate::workbench_sqlite_storage_mode::initialize_for_startup(&state_path)
        .expect("initialize DB-primary fixture before binding write");
    let config = McpServerConfig {
        role: super::super::McpRole::SupervisorOrchestrator,
        run_id: "supervisor-conversation:db-primary-fixture".to_string(),
        node_id: None,
        supervisor_workflow_state_path: Some(state_path.clone()),
        supervisor_quota_limits: Some(SupervisorQuotaLimits {
            max_active_workers: 1,
            max_follow_ups_per_worker: 1,
            max_runtime_minutes: 1,
        }),
        knowledge_open_relay: None,
    };
    DbPrimarySupervisorFixture {
        root,
        db_path,
        config,
    }
}

fn db_supervisor_session_counts(fixture: &DbPrimarySupervisorFixture) -> (i64, i64) {
    let connection =
        rusqlite::Connection::open(&fixture.db_path).expect("open fixture DB for count");
    connection
        .query_row(
            "SELECT COUNT(*), SUM(CASE WHEN json_extract(record_json, '$.conversation_turn_binding') IS NOT NULL THEN 1 ELSE 0 END) FROM supervisor_orchestrator_sessions",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("count fixture DB supervisor sessions")
}

fn db_supervisor_binding_lifecycle(fixture: &DbPrimarySupervisorFixture) -> String {
    let connection =
        rusqlite::Connection::open(&fixture.db_path).expect("open fixture DB for lifecycle");
    connection
        .query_row(
            "SELECT json_extract(record_json, '$.conversation_turn_binding.lifecycle') FROM supervisor_orchestrator_sessions WHERE run_id = ?1",
            rusqlite::params![fixture.config.run_id],
            |row| row.get(0),
        )
        .expect("read fixture DB binding lifecycle")
}

fn json_supervisor_binding_lifecycle(config: &McpServerConfig) -> ConversationTurnLifecycle {
    session(
        &load_store(config).expect("read temporary fixture supervisor store"),
        &config.run_id,
    )
    .and_then(|session| session.conversation_turn_binding.as_ref())
    .map(|binding| binding.lifecycle)
    .expect("fixture binding lifecycle")
}

fn assert_shared_tools_closed(config: &McpServerConfig) {
    assert!(
        tool_names(&list_tools(config)).is_empty(),
        "failed or unconfirmed binding must not publish tools"
    );
    call_tool_with_invoker(
        config,
        json!({"name": "submit_proposal", "arguments": {}}),
        &FakeInvoker,
    )
    .expect_err("failed or unconfirmed binding must reject tools/call");
}

fn copy_fixture_artifact(source_root: &Path, target_root: &Path, name: &str) {
    fs::copy(source_root.join(name), target_root.join(name))
        .unwrap_or_else(|error| panic!("copy approved private fixture artifact {name}: {error}"));
}

#[test]
#[ignore = "requires SHARED_SUPERVISOR_BINDING_AUDIT_COPY pointing to an approved private temporary copy"]
fn shared_supervisor_real_copy_replay_persists_starting_binding_before_transport() {
    let _serial = crate::workbench_sqlite_storage_mode::storage_mode_test_lock()
        .lock()
        .expect("storage mode test lock");
    let source_root = std::env::var_os("SHARED_SUPERVISOR_BINDING_AUDIT_COPY")
        .map(PathBuf::from)
        .expect("approved private temporary copy path is required");
    let root = std::env::temp_dir().join(format!(
        "shared-supervisor-binding-real-copy-replay-{}-{}",
        std::process::id(),
        FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed),
    ));
    let state_dir = root.join("workflow-state");
    let db_dir = root.join("production-db");
    let runtime_dir = root.join("runtime-artifacts");
    fs::create_dir_all(&state_dir).expect("create replay workflow-state directory");
    fs::create_dir_all(&db_dir).expect("create replay DB directory");
    fs::create_dir_all(&runtime_dir).expect("create replay runtime directory");
    copy_fixture_artifact(&source_root, &state_dir, "workflow-state.v0.json");
    copy_fixture_artifact(&source_root, &state_dir, "supervisor-orchestrator.v1.json");
    copy_fixture_artifact(&source_root, &db_dir, "workbench-state.v1.sqlite");
    copy_fixture_artifact(&source_root, &db_dir, "workbench-state.v1.sqlite-wal");
    copy_fixture_artifact(&source_root, &db_dir, "workbench-state.v1.sqlite-shm");
    let state_path = fs::canonicalize(state_dir.join("workflow-state.v0.json"))
        .expect("canonical replay workflow state");
    let db_path =
        fs::canonicalize(db_dir.join("workbench-state.v1.sqlite")).expect("canonical replay DB");
    fs::write(
        runtime_dir.join("storage-mode.v1.json"),
        serde_json::to_vec(&json!({
            "schema_version": "storage-mode.v1",
            "mode": "db_primary_json_projection",
            "workflow_state_path": state_path,
            "confirmed_workflow_state_path": state_path,
            "db_path": db_path,
            "confirmed_db_path": db_path,
            "denied_path_markers": [],
        }))
        .expect("serialize replay storage mode"),
    )
    .expect("write replay storage mode");
    crate::workbench_sqlite_storage_mode::clear_storage_mode_cache_for_tests();
    crate::workbench_sqlite_storage_mode::initialize_for_startup(&state_path)
        .expect("approved private copy must reconcile before DB-primary replay");
    let fixture = DbPrimarySupervisorFixture {
        root,
        db_path,
        config: McpServerConfig {
            role: super::super::McpRole::SupervisorOrchestrator,
            run_id: "supervisor-conversation:real-copy-offline-replay".to_string(),
            node_id: None,
            supervisor_workflow_state_path: Some(state_path),
            supervisor_quota_limits: Some(SupervisorQuotaLimits {
                max_active_workers: 1,
                max_follow_ups_per_worker: 1,
                max_runtime_minutes: 1,
            }),
            knowledge_open_relay: None,
        },
    };
    let replay_project = fixture.root.join("safe-project");
    fs::create_dir_all(&replay_project).expect("create replay-safe project root");
    let replay_project = fs::canonicalize(replay_project)
        .expect("canonical replay-safe project root")
        .display()
        .to_string();
    let before_store =
        load_store(&fixture.config).expect("load approved private copy supervisor store");
    let before_db = db_supervisor_session_counts(&fixture);
    assert_eq!(before_store.sessions.len(), 25);
    assert_eq!(before_db, (25, 0));
    let binding = ConversationTurnBinding::establish_supervisor_read_only(
        super::super::supervisor_conversation_binding::SupervisorConversationTurnInput {
            project_id: crate::project_id(&replay_project),
            project_root: replay_project,
            workflow_id: "workflow:real-copy-offline-replay".to_string(),
            turn_id: "turn:real-copy-offline-replay".to_string(),
            transport_attempt: 1,
            run_id: fixture.config.run_id.clone(),
            user_message_snapshot: "offline replay only".to_string(),
            created_at_ms: now_ms(),
            max_runtime_minutes: 1,
        },
    )
    .expect("construct safe offline replay binding");
    establish_supervisor_conversation_turn_binding(&fixture.config, binding)
        .expect("persist replay binding before transport");
    let after_store = load_store(&fixture.config).expect("load replayed supervisor store");
    let after_db = db_supervisor_session_counts(&fixture);
    assert_eq!(after_store.sessions.len(), before_store.sessions.len() + 1);
    assert_eq!(after_db.0, before_db.0 + 1);
    assert_eq!(after_db.1, before_db.1 + 1);
    let replayed = after_store
        .sessions
        .iter()
        .find(|session| session.run_id == fixture.config.run_id)
        .expect("replay binding session");
    assert_eq!(
        replayed
            .conversation_turn_binding
            .as_ref()
            .map(|binding| binding.lifecycle),
        Some(ConversationTurnLifecycle::Starting),
    );
}

#[test]
fn shared_supervisor_db_primary_persists_starting_binding_on_existing_multi_session_store() {
    let _serial = crate::workbench_sqlite_storage_mode::storage_mode_test_lock()
        .lock()
        .expect("storage mode test lock");
    let fixture = db_primary_supervisor_fixture(25);
    assert_eq!(
        load_store(&fixture.config)
            .expect("load fixture store")
            .sessions
            .len(),
        25
    );
    assert_eq!(db_supervisor_session_counts(&fixture), (25, 0));

    establish_supervisor_conversation_turn_binding(
        &fixture.config,
        new_shared_supervisor_binding(&fixture.config),
    )
    .expect("DB-primary binding must project onto an existing 25-session store");

    let projected = load_store(&fixture.config).expect("load projected fixture store");
    let binding = session(&projected, &fixture.config.run_id)
        .and_then(|session| session.conversation_turn_binding.as_ref())
        .expect("new session must contain a Starting binding");
    assert_eq!(binding.lifecycle, ConversationTurnLifecycle::Starting);
    assert_eq!(projected.sessions.len(), 26);
    assert_eq!(db_supervisor_session_counts(&fixture), (26, 1));
}

#[test]
fn shared_supervisor_db_primary_failure_does_not_project_or_publish_binding() {
    let _serial = crate::workbench_sqlite_storage_mode::storage_mode_test_lock()
        .lock()
        .expect("storage mode test lock");
    let fixture = db_primary_supervisor_fixture(25);
    let before = db_supervisor_session_counts(&fixture);
    let _failure = force_db_primary_test_failure(DbPrimaryTestFailure::PersistDb);

    let error = establish_supervisor_conversation_turn_binding(
        &fixture.config,
        new_shared_supervisor_binding(&fixture.config),
    )
    .expect_err("a DB delta failure must fail before the transport can start");
    assert_eq!(
        error,
        SupervisorConversationBindingEstablishmentError::BindingPersistDb
    );
    assert_eq!(
        load_store(&fixture.config)
            .expect("load unprojected fixture store")
            .sessions
            .len(),
        25
    );
    assert_eq!(db_supervisor_session_counts(&fixture), before);
    assert!(
        tool_names(&list_tools(&fixture.config)).is_empty(),
        "failed binding must not publish tools"
    );
}

#[test]
fn shared_supervisor_injected_store_prepare_failure_is_staged_and_closed() {
    let _serial = crate::workbench_sqlite_storage_mode::storage_mode_test_lock()
        .lock()
        .expect("storage mode test lock");
    let fixture = db_primary_supervisor_fixture(25);
    let before = db_supervisor_session_counts(&fixture);
    let failure = force_db_primary_test_failure(DbPrimaryTestFailure::StorePrepare);

    let error = establish_supervisor_conversation_turn_binding(
        &fixture.config,
        new_shared_supervisor_binding(&fixture.config),
    )
    .expect_err("injected store preparation failure must prevent binding persistence");
    assert_eq!(
        error,
        SupervisorConversationBindingEstablishmentError::BindingStorePrepare
    );
    drop(failure);
    assert_eq!(
        load_store(&fixture.config)
            .expect("load unprojected fixture store")
            .sessions
            .len(),
        25
    );
    assert_eq!(db_supervisor_session_counts(&fixture), before);
    assert_shared_tools_closed(&fixture.config);
}

#[test]
fn shared_supervisor_injected_activation_failure_finishes_failed_in_db_and_json() {
    let _serial = crate::workbench_sqlite_storage_mode::storage_mode_test_lock()
        .lock()
        .expect("storage mode test lock");
    let fixture = db_primary_supervisor_fixture(25);
    establish_starting_shared_supervisor_binding(&fixture.config);
    let failure = force_supervisor_conversation_binding_lifecycle_test_failure(
        SupervisorConversationBindingLifecycleTestFailure::Activate,
    );

    activate_supervisor_conversation_turn_binding(&fixture.config, "thread:injected-activate")
        .expect_err("injected activation failure must not publish an active binding");
    drop(failure);
    finish_supervisor_conversation_turn_binding(&fixture.config, ConversationTurnLifecycle::Failed)
        .expect("failed activation must be durably closed");

    assert_eq!(
        json_supervisor_binding_lifecycle(&fixture.config),
        ConversationTurnLifecycle::Failed
    );
    assert_eq!(db_supervisor_binding_lifecycle(&fixture), "failed");
    assert_shared_tools_closed(&fixture.config);
}

#[test]
fn shared_supervisor_injected_transport_failure_finishes_failed_in_db_and_json() {
    let _serial = crate::workbench_sqlite_storage_mode::storage_mode_test_lock()
        .lock()
        .expect("storage mode test lock");
    let fixture = db_primary_supervisor_fixture(25);
    establish_starting_shared_supervisor_binding(&fixture.config);
    activate_supervisor_conversation_turn_binding(&fixture.config, "thread:injected-transport")
        .expect("fixture host activates binding before transport start");

    let injected_transport_start: Result<(), &str> = Err("injected_transport_start_failure");
    assert!(
        injected_transport_start.is_err(),
        "fixture must take the transport-start failure branch"
    );
    finish_supervisor_conversation_turn_binding(&fixture.config, ConversationTurnLifecycle::Failed)
        .expect("transport startup failure must close the durable binding");

    assert_eq!(
        json_supervisor_binding_lifecycle(&fixture.config),
        ConversationTurnLifecycle::Failed
    );
    assert_eq!(db_supervisor_binding_lifecycle(&fixture), "failed");
    assert_shared_tools_closed(&fixture.config);
}

#[test]
fn shared_supervisor_injected_termination_failure_keeps_tools_closed_without_terminal_claim() {
    let _serial = crate::workbench_sqlite_storage_mode::storage_mode_test_lock()
        .lock()
        .expect("storage mode test lock");
    let fixture = db_primary_supervisor_fixture(25);
    establish_starting_shared_supervisor_binding(&fixture.config);
    activate_supervisor_conversation_turn_binding(&fixture.config, "thread:injected-finish")
        .expect("fixture host activates binding before termination");
    let failure = force_supervisor_conversation_binding_lifecycle_test_failure(
        SupervisorConversationBindingLifecycleTestFailure::Finish,
    );

    finish_supervisor_conversation_turn_binding(&fixture.config, ConversationTurnLifecycle::Failed)
        .expect_err("injected termination failure must not claim a durable Failed binding");
    drop(failure);

    assert_eq!(
        json_supervisor_binding_lifecycle(&fixture.config),
        ConversationTurnLifecycle::Active
    );
    assert_eq!(db_supervisor_binding_lifecycle(&fixture), "active");
    assert_shared_tools_closed(&fixture.config);
}

#[test]
fn shared_supervisor_json_projection_failure_stays_starting_and_unpublished() {
    let _serial = crate::workbench_sqlite_storage_mode::storage_mode_test_lock()
        .lock()
        .expect("storage mode test lock");
    let fixture = db_primary_supervisor_fixture(25);
    let _failure = force_db_primary_test_failure(DbPrimaryTestFailure::ProjectJson);

    let error = establish_supervisor_conversation_turn_binding(
        &fixture.config,
        new_shared_supervisor_binding(&fixture.config),
    )
    .expect_err("a JSON projection failure must fail closed after the DB delta");
    assert_eq!(
        error,
        SupervisorConversationBindingEstablishmentError::BindingProjectJson
    );
    assert_eq!(
        load_store(&fixture.config)
            .expect("JSON sidecar remains unprojected")
            .sessions
            .len(),
        25
    );
    assert_eq!(db_supervisor_session_counts(&fixture), (26, 1));
    let health = crate::workbench_sqlite_storage_mode::db_primary_health_snapshot(
        fixture
            .config
            .supervisor_workflow_state_path
            .as_deref()
            .expect("DB-primary fixture workflow state"),
    )
    .expect("JSON projection failure must block further DB-primary writes")
    .expect_err("JSON projection failure must freeze DB-primary mode");
    assert!(health.contains("db_primary_json_projection_failed:supervisor_orchestrator"));
    assert!(
        tool_names(&list_tools(&fixture.config)).is_empty(),
        "DB-leading Starting binding must not publish tools"
    );
}

fn tool_names(toolface: &Value) -> BTreeSet<&str> {
    toolface["tools"]
        .as_array()
        .expect("MCP tool face is an array")
        .iter()
        .filter_map(|tool| tool["name"].as_str())
        .collect()
}

#[test]
fn shared_supervisor_binding_has_one_server_side_toolface_and_denies_variants() {
    let fixture = Fixture::new();
    let config = shared_supervisor_conversation_config(&fixture);
    let before_chain_runs = workflow_chain_runs_for(&fixture);
    establish_active_shared_supervisor_binding(&config);

    assert_eq!(
        tool_names(&list_tools(&config)),
        BTreeSet::from([
            "submit_proposal",
            "knowledge_search",
            "knowledge_read",
            "knowledge_open",
            "knowledge_cite",
        ]),
        "the shared supervisor profile may expose only its frozen proposal plus four fixed-vault read-only knowledge tools"
    );

    for name in [
        "SUBMIT_PROPOSAL",
        "submit_proposal ",
        "KNOWLEDGE_READ",
        "knowledge_read ",
        "knowledge_write",
        "read_worker_report",
        "dispatch_worker",
        "*",
    ] {
        let error = call_tool_with_invoker(
            &config,
            json!({"name": name, "arguments": {}}),
            &FakeInvoker,
        )
        .expect_err("unknown, variant, wildcard, and non-allowlisted calls must fail closed");
        assert!(
            !error.trim().is_empty(),
            "the server must return a human-facing fail-closed error"
        );
    }

    let store = load_store(&config).expect("read temporary fixture supervisor store");
    let session = session(&store, &config.run_id).expect("persisted fixture binding");
    assert!(
        session.workers.is_empty(),
        "denied tools must not create workers"
    );
    assert_eq!(
        workflow_chain_runs_for(&fixture),
        before_chain_runs,
        "the capability test must not create or advance a workflow chain"
    );
}

#[test]
fn shared_supervisor_tools_list_and_call_share_the_exact_knowledge_allowlist() {
    let fixture = Fixture::new();
    let config = shared_supervisor_conversation_config(&fixture);
    let before_chain_runs = workflow_chain_runs_for(&fixture);
    establish_active_shared_supervisor_binding(&config);

    // Every call deliberately uses a handler-level invalid value.  Reaching a
    // schema error (instead of an authorization error or a filesystem read)
    // proves the exact name listed above passed the same server-side binding
    // and registry gate while keeping this test off the real Syn vault.
    for (name, arguments) in [
        ("knowledge_search", json!({"query": "needle*"})),
        ("knowledge_read", json!({"slug": "../outside"})),
        ("knowledge_open", json!({"slug": "../outside"})),
        ("knowledge_cite", json!({"slugs": ["One", "One"]})),
    ] {
        let error = call_tool_with_invoker(
            &config,
            json!({"name": name, "arguments": arguments}),
            &FakeInvoker,
        )
        .expect_err("the fixed-vault handler must reject bad arguments after authorization");
        assert!(
            !error.contains("未获该 capability 授权"),
            "{name} must pass the same registry/binding authorization as tools/list: {error}"
        );
    }

    assert_eq!(
        super::supervisor_conversation_capability_outcome(&config, "knowledge_open")
            .expect("read safe failed knowledge_open settlement"),
        Some(super::ConversationCapabilityOutcome::Failed),
        "a failed read-only open is recorded as failed without changing the durable turn lifecycle"
    );
    let store = load_store(&config).expect("read temporary fixture supervisor store");
    let session = session(&store, &config.run_id).expect("persisted fixture binding");
    assert_eq!(
        session
            .conversation_turn_binding
            .as_ref()
            .map(|binding| binding.lifecycle),
        Some(ConversationTurnLifecycle::Active),
        "a read-only knowledge tool failure must not terminalize the active conversation"
    );
    assert!(
        session.workers.is_empty(),
        "a failed read-only knowledge tool must not create a worker"
    );
    assert!(
        proposal_store_for(&fixture).proposals.is_empty(),
        "a failed read-only knowledge tool must not create a proposal card"
    );
    assert_eq!(
        workflow_chain_runs_for(&fixture),
        before_chain_runs,
        "a failed read-only knowledge tool must not start or advance a chain"
    );
}

#[test]
fn shared_supervisor_prefix_without_persisted_binding_has_no_tools_and_cannot_call() {
    let fixture = Fixture::new();
    let mut config = shared_supervisor_conversation_config(&fixture);
    config.run_id = "supervisor-conversation:missing-binding".to_string();
    let before_chain_runs = workflow_chain_runs_for(&fixture);

    assert!(
        tool_names(&list_tools(&config)).is_empty(),
        "a run-id prefix alone must never publish a capability"
    );
    let error = call_tool_with_invoker(
        &config,
        json!({"name": "submit_proposal", "arguments": {}}),
        &FakeInvoker,
    )
    .expect_err("missing durable binding must fail before handler dispatch");
    assert!(error.contains("可信 conversation turn binding"), "{error}");
    assert!(
        load_store(&config)
            .expect("read empty fixture store")
            .sessions
            .is_empty(),
        "a denied missing-binding call must not manufacture a session"
    );
    assert_eq!(workflow_chain_runs_for(&fixture), before_chain_runs);
}

#[test]
fn shared_supervisor_starting_binding_stays_unpublished_until_host_binds_thread() {
    let fixture = Fixture::new();
    let config = shared_supervisor_conversation_config(&fixture);
    let before_chain_runs = workflow_chain_runs_for(&fixture);
    establish_starting_shared_supervisor_binding(&config);

    assert!(
        tool_names(&list_tools(&config)).is_empty(),
        "a durable but starting/unbound turn must not publish submit_proposal"
    );
    call_tool_with_invoker(
        &config,
        json!({"name": "submit_proposal", "arguments": {}}),
        &FakeInvoker,
    )
    .expect_err("a starting/unbound turn must fail before proposal handling");

    let store = load_store(&config).expect("read temporary fixture supervisor store");
    let session = session(&store, &config.run_id).expect("starting binding remains persisted");
    assert!(session.workers.is_empty());
    assert_eq!(workflow_chain_runs_for(&fixture), before_chain_runs);
}

#[test]
fn stale_shared_active_binding_expires_and_cannot_publish_or_call_capability() {
    let fixture = Fixture::new();
    let config = shared_supervisor_conversation_config(&fixture);
    let before_chain_runs = workflow_chain_runs_for(&fixture);
    let mut binding = new_shared_supervisor_binding(&config);
    let expired_created_at = now_ms().saturating_sub(60_001);
    binding.created_at_ms = expired_created_at;
    binding.updated_at_ms = expired_created_at;
    establish_supervisor_conversation_turn_binding(&config, binding)
        .expect("persist an offline stale binding fixture");
    activate_supervisor_conversation_turn_binding(&config, "thread:offline-capability-plane")
        .expect("only the host may activate the fixture binding");

    assert!(
        tool_names(&list_tools(&config)).is_empty(),
        "an expired active binding must no longer publish the supervisor capability"
    );
    let error = call_tool_with_invoker(
        &config,
        json!({"name": "submit_proposal", "arguments": {}}),
        &FakeInvoker,
    )
    .expect_err("an expired active binding must fail before proposal handling");
    assert!(error.contains("超过宿主运行时限"), "{error}");

    let store = load_store(&config).expect("read temporary fixture supervisor store");
    let session = session(&store, &config.run_id).expect("stale fixture binding remains persisted");
    assert!(session.workers.is_empty());
    assert_eq!(workflow_chain_runs_for(&fixture), before_chain_runs);
}

#[test]
fn shared_supervisor_submit_proposal_is_idempotent_and_records_only_safe_settlement() {
    let fixture = Fixture::new();
    let config = shared_supervisor_conversation_config(&fixture);
    let project_id = crate::project_id(PROJECT);
    fs::write(
        &fixture.state_path,
        serde_json::to_vec(&json!({
            "projects": [{"project_id": project_id, "root_path": PROJECT}],
            "workflows": [{"workflow_id": WORKFLOW, "project_id": crate::project_id(PROJECT)}],
            "workflow_chain_runs": [],
            "audit_events": []
        }))
        .expect("shared fixture workflow state json"),
    )
    .expect("shared fixture workflow state");
    let before_chain_runs = workflow_chain_runs_for(&fixture);
    establish_active_shared_supervisor_binding(&config);

    let mut arguments = valid_submit_proposal_arguments();
    arguments["user_goal"] = json!("模型参数不得覆盖宿主冻结的本回合用户消息。");
    for _ in 0..2 {
        let response = call_tool_with_invoker(
            &config,
            json!({"name": "submit_proposal", "arguments": arguments.clone()}),
            &FakeInvoker,
        )
        .expect("active shared binding may create or reuse exactly one pending proposal");
        let receipt: Value = serde_json::from_str(
            response["content"][0]["text"]
                .as_str()
                .expect("shared tool receipt text"),
        )
        .expect("shared tool receipt json");
        assert_eq!(receipt["thread_id"], "thread:offline-capability-plane");
    }

    let proposal_store = proposal_store_for(&fixture);
    assert_eq!(
        proposal_store.proposals.len(),
        1,
        "technical retry for one trusted turn must reuse the same Pending card"
    );
    let proposal = &proposal_store.proposals[0];
    assert_eq!(
        proposal.status,
        crate::ProjectConsultationProposalStatus::PendingUserConfirmation
    );
    assert_eq!(
        proposal.user_requirement_snapshot, "请整理一份待用户确认的离线方案。",
        "shared binding must override the model-supplied user_goal with the host snapshot"
    );
    assert_eq!(
        super::supervisor_conversation_capability_outcome(&config, "submit_proposal")
            .expect("read safe shared settlement"),
        Some(super::ConversationCapabilityOutcome::Succeeded),
        "only the server-side dispatcher may settle a shared proposal action"
    );
    let supervisor_store = load_store(&config).expect("read temporary fixture supervisor store");
    let session = session(&supervisor_store, &config.run_id).expect("shared fixture session");
    assert!(
        session.workers.is_empty(),
        "proposal submission must not create workers"
    );
    assert_eq!(
        workflow_chain_runs_for(&fixture),
        before_chain_runs,
        "an unapproved Pending card must not start or advance a chain"
    );
}

#[test]
fn shared_supervisor_audit_failure_does_not_erase_persisted_tool_result() {
    let fixture = Fixture::new();
    let config = shared_supervisor_conversation_config(&fixture);
    let project_id = crate::project_id(PROJECT);
    fs::write(
        &fixture.state_path,
        serde_json::to_vec(&json!({
            "projects": [{"project_id": project_id, "root_path": PROJECT}],
            "workflows": [{"workflow_id": WORKFLOW, "project_id": crate::project_id(PROJECT)}],
            "workflow_chain_runs": [],
            "audit_events": []
        }))
        .expect("shared audit-failure workflow state json"),
    )
    .expect("shared audit-failure workflow state");
    let before_chain_runs = workflow_chain_runs_for(&fixture);
    establish_active_shared_supervisor_binding(&config);
    let _guard = force_shared_supervisor_tool_audit_failure();

    let response = call_tool_with_invoker(
        &config,
        json!({"name": "submit_proposal", "arguments": valid_submit_proposal_arguments()}),
        &FakeInvoker,
    )
    .expect("audit failure must not replace an already persisted proposal result");
    let receipt: Value = serde_json::from_str(
        response["content"][0]["text"]
            .as_str()
            .expect("shared tool receipt text"),
    )
    .expect("shared tool receipt json");
    assert_eq!(receipt["thread_id"], "thread:offline-capability-plane");
    assert_eq!(
        proposal_store_for(&fixture).proposals.len(),
        1,
        "the handler result remains a real Pending card"
    );
    assert_eq!(
        super::supervisor_conversation_capability_outcome(&config, "submit_proposal")
            .expect("read tool settlement after audit failure"),
        Some(super::ConversationCapabilityOutcome::Succeeded)
    );
    assert_eq!(
        super::supervisor_conversation_capability_audit_outcome(&config, "submit_proposal")
            .expect("read separate audit settlement"),
        Some(super::ConversationCapabilityOutcome::Failed)
    );
    assert_eq!(workflow_chain_runs_for(&fixture), before_chain_runs);
}
