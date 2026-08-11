use rusqlite::{params, Connection, OpenFlags};
use serde_json::{json, Value};
use std::fs;

struct FixtureRoot(std::path::PathBuf);

impl Drop for FixtureRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn replace_exact_string(value: &mut Value, before: &str, after: &str) {
    match value {
        Value::String(current) if current == before => *current = after.to_string(),
        Value::Array(values) => {
            for value in values {
                replace_exact_string(value, before, after);
            }
        }
        Value::Object(values) => {
            for value in values.values_mut() {
                replace_exact_string(value, before, after);
            }
        }
        _ => {}
    }
}

fn owner_counts(connection: &Connection) -> [i64; 6] {
    [
        "events",
        "command_receipts",
        "current_snapshots",
        "work_items",
        "workflow_audit_events",
        "m4_source_owner_publications",
    ]
    .map(|table| {
        connection
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .unwrap_or_else(|error| panic!("count {table}: {error}"))
    })
}

fn seed_work_item_fixture(
    label: &str,
    replacement_work_item_id: Option<&str>,
) -> (
    FixtureRoot,
    std::path::PathBuf,
    std::path::PathBuf,
    String,
    String,
) {
    crate::workbench_sqlite_storage_mode::clear_storage_mode_cache_for_tests();
    let root = std::env::temp_dir().join(format!(
        "m4-owner-rejection-{label}-{}",
        crate::unix_timestamp_nanos()
    ));
    fs::create_dir_all(&root).expect("create candidate rejection fixture root");
    let root = fs::canonicalize(root).expect("canonical candidate rejection fixture root");
    let guard = FixtureRoot(root.clone());
    let project_root = root.join("project");
    fs::create_dir_all(&project_root).expect("create candidate rejection project root");
    let project_root = fs::canonicalize(project_root)
        .expect("canonical candidate rejection project root")
        .display()
        .to_string();
    let state_parent = root.join("workflow-state");
    fs::create_dir_all(&state_parent).expect("create workflow state parent");
    let state_parent = fs::canonicalize(state_parent).expect("canonical workflow state parent");
    let state_path = state_parent.join("workflow-state.v0.json");
    let project = crate::ProjectRecord {
        project_root: project_root.clone(),
        name: "M4 candidate rejection fixture".to_string(),
        active_hint: true,
        thread_count: 0,
        active_thread_count: 0,
        archived_thread_count: 0,
        latest_updated_at_ms: None,
        authority_files: vec![],
        handoff_files: vec![],
        evidence_files: vec![],
        harness_candidates: vec![],
        harness_resources: vec![],
        context_warnings: vec![],
        warnings: vec![],
    };
    crate::bootstrap_project_workflow_at(&state_path, &project)
        .expect("bootstrap rejection workflow");
    crate::create_task_draft_at(
        &state_path,
        &crate::TaskDraftRequest {
            project_root: project_root.clone(),
            title: "candidate rejection task".to_string(),
            objective: "exercise owner rollback".to_string(),
            assigned_role: Some("codex-dev".to_string()),
        },
    )
    .expect("create rejection work item");
    let initial = crate::read_workflow_state_value(&state_path).expect("read new work item");
    let opaque_id = initial["work_items"][0]["work_item_id"]
        .as_str()
        .expect("new opaque work item id")
        .to_string();
    crate::update_work_item_state_at(
        &state_path,
        &crate::WorkItemStateUpdateRequest {
            project_root: project_root.clone(),
            work_item_id: opaque_id.clone(),
            next_state: "ready_to_dispatch".to_string(),
            client_request_ref: None,
            command_id: Some("m4-rejection-json-seed-command".to_string()),
            idempotency_key: Some("m4-rejection-json-seed-key".to_string()),
            expected_revision: None,
        },
    )
    .expect("make rejection work item dispatchable");

    // Historical IDs remain readable. Reproduce one whose old owner key
    // contains sensitive/path material, then import it before DB activation.
    let work_item_id = replacement_work_item_id
        .map(str::to_string)
        .unwrap_or_else(|| opaque_id.clone());
    let mut state =
        crate::read_workflow_state_value(&state_path).expect("read dispatchable rejection fixture");
    replace_exact_string(&mut state, &opaque_id, &work_item_id);
    fs::write(
        &state_path,
        serde_json::to_vec_pretty(&state).expect("serialize sensitive legacy fixture"),
    )
    .expect("write sensitive legacy fixture");

    let runtime_parent = root.join("runtime-artifacts");
    fs::create_dir_all(&runtime_parent).expect("create runtime parent");
    let runtime_parent = fs::canonicalize(runtime_parent).expect("canonical runtime parent");
    let db_path = runtime_parent.join("workbench.sqlite");
    let repository = crate::workbench_sqlite_repository::WorkbenchSqliteRepository::open_confirmed(
        &crate::workbench_sqlite_repository::ConfirmedWorkbenchSqliteRepositoryConfig {
            db_path: db_path.clone(),
            confirmed_db_path: db_path.clone(),
            denied_path_markers: vec![],
        },
    )
    .expect("open candidate rejection DB");
    let empty = json!({
        "workflows": [],
        "nodes": [],
        "edges": [],
        "work_items": [],
        "artifacts": [],
        "reviews": [],
        "workflow_node_session_bindings": [],
        "workflow_node_dispatches": [],
        "audit_events": []
    });
    repository
        .record_workflow_state_delta_with_audit(&empty, &state, None)
        .expect("seed candidate rejection workflow projection");
    let state_text = serde_json::to_string(&state).expect("serialize sidecar meta state");
    let connection = Connection::open(&db_path).expect("open candidate rejection DB meta");
    connection
        .execute(
            "INSERT INTO workflow_state_meta (
                workspace_id, source_root_hash, schema_version, workflow_version,
                revision, source_id, meta_json
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                format!(
                    "m4-owner-rejection:{}",
                    crate::utils::hash::sha256_hex(&state_path.display().to_string())
                ),
                crate::utils::hash::sha256_hex(&state_text),
                state
                    .get("schema_version")
                    .and_then(Value::as_str)
                    .unwrap_or("workflow_state_v0"),
                state
                    .get("workflow_version")
                    .and_then(Value::as_i64)
                    .unwrap_or(1),
                state.get("revision").and_then(Value::as_i64).unwrap_or(0),
                crate::workbench_sqlite_repository::WORKFLOW_STATE_SIDECAR_REPOSITORY_SOURCE_ID,
                serde_json::to_string(&json!({
                    "schema_version": state.get("schema_version"),
                    "workflow_version": state.get("workflow_version"),
                    "revision": state.get("revision"),
                    "fixture_provenance": "db_primary_projection_writer"
                }))
                .expect("serialize sidecar meta"),
            ],
        )
        .expect("seed candidate rejection sidecar meta");
    drop(connection);
    let config = crate::workbench_sqlite_storage_mode::DbPrimaryJsonProjectionConfig {
        workflow_state_path: state_path.clone(),
        confirmed_workflow_state_path: state_path.clone(),
        db_path: db_path.clone(),
        confirmed_db_path: db_path.clone(),
        denied_path_markers: vec![],
    };
    crate::workbench_sqlite_storage_mode::install_db_primary_config_create_new(&config)
        .expect("activate candidate rejection DB primary mode");
    crate::workbench_sqlite_storage_mode::clear_storage_mode_cache_for_path_for_tests(&state_path);
    crate::workbench_sqlite_storage_mode::initialize_for_startup(&state_path)
        .expect("reconcile candidate rejection DB primary fixture");
    (guard, state_path, db_path, project_root, work_item_id)
}

#[test]
fn server_sealed_client_request_ref_advances_revision_and_exactly_replays() {
    let _serial = crate::workbench_sqlite_storage_mode::storage_mode_test_lock()
        .lock()
        .expect("storage mode test lock");
    let (_root, state_path, db_path, project_root, work_item_id) =
        seed_work_item_fixture("server-sealed-replay", None);
    let repository =
        crate::workbench_sqlite_storage_mode::primary_repository_for_write(&state_path)
            .expect("server-sealed DB-primary gate")
            .expect("server-sealed DB-primary repository");
    let workflow_id = crate::default_workflow_id(&project_root);
    let revision_before = repository
        .m2_workflow_state_sidecar_revision(&workflow_id, &work_item_id)
        .expect("read server-owned revision before command");
    let request = crate::WorkItemStateUpdateRequest {
        project_root: project_root.clone(),
        work_item_id: work_item_id.clone(),
        next_state: "running".to_string(),
        client_request_ref: Some("018f8cf04f717c159e2f4f3bcf28e101".to_string()),
        command_id: None,
        idempotency_key: None,
        expected_revision: None,
    };
    let connection = Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .expect("open server-sealed DB before command");
    let counts_before = owner_counts(&connection);
    drop(connection);

    let first = crate::update_work_item_state_at(&state_path, &request)
        .expect("server-sealed ordinary command succeeds");
    let first_receipt = first.receipt_id.expect("server-sealed receipt");
    assert_eq!(
        repository
            .m2_workflow_state_sidecar_revision(&workflow_id, &work_item_id)
            .expect("read advanced server-owned revision"),
        revision_before + 1,
        "Some(server-owned revision) must persist the aggregate"
    );
    let connection = Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .expect("open server-sealed DB after command");
    let counts_after_first = owner_counts(&connection);
    let sealed_identity: (String, String, i64) = connection
        .query_row(
            "SELECT command_id, idempotency_key, committed_revision
             FROM command_receipts WHERE receipt_id = ?1",
            [first_receipt.as_str()],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("read server-sealed receipt identity");
    assert!(sealed_identity
        .0
        .starts_with("workflow-state-sidecar.product.v1:"));
    assert!(sealed_identity
        .1
        .starts_with("idem:workflow-state-sidecar.product.v1:"));
    for sealed in [&sealed_identity.0, &sealed_identity.1] {
        assert!(!sealed.contains(&project_root));
        assert!(!sealed.contains(&work_item_id));
        assert!(!sealed.contains("018f8cf04f717c159e2f4f3bcf28e101"));
    }
    assert_eq!(sealed_identity.2, revision_before + 1);
    drop(connection);

    let replay = crate::update_work_item_state_at(&state_path, &request)
        .expect("exact server-sealed retry replays");
    assert_eq!(replay.receipt_id.as_deref(), Some(first_receipt.as_str()));
    let connection = Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .expect("open server-sealed DB after replay");
    assert_eq!(
        owner_counts(&connection),
        counts_after_first,
        "exact replay cannot add receipt/event/snapshot/owner publication rows"
    );
    assert_eq!(
        counts_after_first[0] - counts_before[0],
        1,
        "one logical command owns one native event"
    );
    assert_eq!(
        counts_after_first[5] - counts_before[5],
        1,
        "one logical command owns one M4 source publication"
    );
}

#[test]
fn server_sealed_client_request_ref_requires_exact_lowercase_hex() {
    for invalid_ref in [
        "018f8cf04f717c159e2f4f3bcf28e10",
        "018F8CF04F717C159E2F4F3BCF28E101",
    ] {
        let error = super::server_sealed_work_item_command_identity(
            &crate::WorkItemStateUpdateRequest {
                project_root: "/synthetic/project".to_string(),
                work_item_id: "work-item:synthetic:001".to_string(),
                next_state: "running".to_string(),
                client_request_ref: Some(invalid_ref.to_string()),
                command_id: None,
                idempotency_key: None,
                expected_revision: None,
            },
            "workflow:synthetic:default",
            "running",
            "workflow:synthetic:default:node:codex-dev",
        )
        .expect_err("short or uppercase client refs must reject");
        assert_eq!(error, "work_item_client_request_ref_invalid");
    }
}

#[test]
fn server_sealed_same_client_ref_cannot_change_the_requested_state() {
    let _serial = crate::workbench_sqlite_storage_mode::storage_mode_test_lock()
        .lock()
        .expect("storage mode test lock");
    let (_root, state_path, db_path, project_root, work_item_id) =
        seed_work_item_fixture("server-sealed-state-conflict", None);
    let repository =
        crate::workbench_sqlite_storage_mode::primary_repository_for_write(&state_path)
            .expect("state-conflict DB-primary gate")
            .expect("state-conflict DB-primary repository");
    let workflow_id = crate::default_workflow_id(&project_root);
    let client_request_ref = "018f8cf04f717c159e2f4f3bcf28e104".to_string();
    let first = crate::update_work_item_state_at(
        &state_path,
        &crate::WorkItemStateUpdateRequest {
            project_root: project_root.clone(),
            work_item_id: work_item_id.clone(),
            next_state: "running".to_string(),
            client_request_ref: Some(client_request_ref.clone()),
            command_id: None,
            idempotency_key: None,
            expected_revision: None,
        },
    )
    .expect("first server-sealed state succeeds");
    assert!(first.receipt_id.is_some());
    let revision_after_first = repository
        .m2_workflow_state_sidecar_revision(&workflow_id, &work_item_id)
        .expect("read state-conflict revision after first command");
    let connection = Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .expect("open DB before state-conflict retry");
    let counts_after_first = owner_counts(&connection);
    drop(connection);

    let error = crate::update_work_item_state_at(
        &state_path,
        &crate::WorkItemStateUpdateRequest {
            project_root,
            work_item_id: work_item_id.clone(),
            next_state: "ready_for_review".to_string(),
            client_request_ref: Some(client_request_ref),
            command_id: None,
            idempotency_key: None,
            expected_revision: None,
        },
    )
    .expect_err("one client ref cannot be rebound to a different state");
    assert!(
        error.contains("work_item_client_request_ref_identity_conflict"),
        "unexpected state-conflict error: {error}"
    );
    assert_eq!(
        repository
            .m2_workflow_state_sidecar_revision(&workflow_id, &work_item_id)
            .expect("read state-conflict final revision"),
        revision_after_first
    );
    let connection = Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .expect("open DB after state-conflict retry");
    assert_eq!(
        owner_counts(&connection),
        counts_after_first,
        "identity conflict cannot append owner or M2 rows"
    );
}

#[test]
fn server_sealed_identity_fields_are_mutually_exclusive_and_fail_before_write() {
    let _serial = crate::workbench_sqlite_storage_mode::storage_mode_test_lock()
        .lock()
        .expect("storage mode test lock");
    let (_root, state_path, db_path, project_root, work_item_id) =
        seed_work_item_fixture("server-sealed-mixed-fields", None);
    let sidecar_before = fs::read(&state_path).expect("read sidecar before mixed identity");
    let connection = Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .expect("open DB before mixed identity");
    let counts_before = owner_counts(&connection);
    drop(connection);
    let error = crate::update_work_item_state_at(
        &state_path,
        &crate::WorkItemStateUpdateRequest {
            project_root,
            work_item_id,
            next_state: "running".to_string(),
            client_request_ref: Some("018f8cf04f717c159e2f4f3bcf28e102".to_string()),
            command_id: Some("renderer-must-not-own-command".to_string()),
            idempotency_key: None,
            expected_revision: None,
        },
    )
    .expect_err("mixed renderer/server identity must reject");
    assert_eq!(error, "work_item_command_identity_mode_conflict");
    assert_eq!(
        fs::read(&state_path).expect("read sidecar after mixed identity"),
        sidecar_before
    );
    let connection = Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .expect("open DB after mixed identity");
    assert_eq!(owner_counts(&connection), counts_before);
}

#[test]
fn explicit_empty_identity_fields_never_fall_back_to_guarded_legacy() {
    let _serial = crate::workbench_sqlite_storage_mode::storage_mode_test_lock()
        .lock()
        .expect("storage mode test lock");
    let (_root, state_path, db_path, project_root, work_item_id) =
        seed_work_item_fixture("server-sealed-empty-explicit-fields", None);
    let sidecar_before = fs::read(&state_path).expect("read sidecar before empty identity");
    let connection = Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .expect("open DB before empty identity");
    let counts_before = owner_counts(&connection);
    drop(connection);

    for (command_id, idempotency_key) in [
        (Some(String::new()), Some(String::new())),
        (Some("   ".to_string()), Some("\t".to_string())),
        (Some(String::new()), None),
    ] {
        let error = crate::update_work_item_state_at(
            &state_path,
            &crate::WorkItemStateUpdateRequest {
                project_root: project_root.clone(),
                work_item_id: work_item_id.clone(),
                next_state: "running".to_string(),
                client_request_ref: None,
                command_id,
                idempotency_key,
                expected_revision: None,
            },
        )
        .expect_err("explicit empty identity presence must reject before owner mutation");
        assert_eq!(error, "work_item_command_identity_mode_conflict");
    }

    let explicit_empty_error = crate::update_work_item_state_at(
        &state_path,
        &crate::WorkItemStateUpdateRequest {
            project_root: project_root.clone(),
            work_item_id: work_item_id.clone(),
            next_state: "running".to_string(),
            client_request_ref: None,
            command_id: Some(String::new()),
            idempotency_key: Some(String::new()),
            expected_revision: Some(0),
        },
    )
    .expect_err("complete explicit identity with empty values must reject before owner mutation");
    assert_eq!(
        explicit_empty_error,
        "work_item_explicit_m2_identity_reserved"
    );
    assert!(super::explicit_m2_idempotency_matches_command(
        "m2-reference-command",
        "idem:m2-reference-command"
    ));
    assert!(!super::explicit_m2_idempotency_matches_command(
        "m2-reference-command",
        "m2-reference-key"
    ));

    assert_eq!(
        fs::read(&state_path).expect("read sidecar after empty identity"),
        sidecar_before
    );
    let connection = Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .expect("open DB after empty identity");
    assert_eq!(owner_counts(&connection), counts_before);
}

#[test]
fn concurrent_server_sealed_exact_requests_share_one_receipt_and_revision() {
    let _serial = crate::workbench_sqlite_storage_mode::storage_mode_test_lock()
        .lock()
        .expect("storage mode test lock");
    let (_root, state_path, db_path, project_root, work_item_id) =
        seed_work_item_fixture("server-sealed-concurrent", None);
    let repository =
        crate::workbench_sqlite_storage_mode::primary_repository_for_write(&state_path)
            .expect("concurrent DB-primary gate")
            .expect("concurrent DB-primary repository");
    let workflow_id = crate::default_workflow_id(&project_root);
    let revision_before = repository
        .m2_workflow_state_sidecar_revision(&workflow_id, &work_item_id)
        .expect("read concurrent initial revision");
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let run = |barrier: std::sync::Arc<std::sync::Barrier>| {
        let state_path = state_path.clone();
        let project_root = project_root.clone();
        let work_item_id = work_item_id.clone();
        std::thread::spawn(move || {
            barrier.wait();
            crate::update_work_item_state_at(
                &state_path,
                &crate::WorkItemStateUpdateRequest {
                    project_root,
                    work_item_id,
                    next_state: "running".to_string(),
                    client_request_ref: Some("018f8cf04f717c159e2f4f3bcf28e103".to_string()),
                    command_id: None,
                    idempotency_key: None,
                    expected_revision: None,
                },
            )
        })
    };
    let first = run(barrier.clone());
    let second = run(barrier);
    let first = first
        .join()
        .expect("first server-sealed thread")
        .expect("first server-sealed result");
    let second = second
        .join()
        .expect("second server-sealed thread")
        .expect("second server-sealed result");
    assert_eq!(first.receipt_id, second.receipt_id);
    assert!(first.receipt_id.is_some());
    assert_eq!(
        repository
            .m2_workflow_state_sidecar_revision(&workflow_id, &work_item_id)
            .expect("read concurrent final revision"),
        revision_before + 1
    );
    let connection = Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .expect("open concurrent final DB");
    assert_eq!(
        connection
            .query_row("SELECT COUNT(*) FROM command_receipts", [], |row| row
                .get::<_, i64>(0))
            .expect("count concurrent receipts"),
        1
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT COUNT(*) FROM m4_source_owner_publications",
                [],
                |row| row.get::<_, i64>(0)
            )
            .expect("count concurrent owner publications"),
        1
    );
}

#[test]
fn sensitive_legacy_work_item_candidate_rolls_back_owner_and_persists_only_scrubbed_rejection() {
    let _serial = crate::workbench_sqlite_storage_mode::storage_mode_test_lock()
        .lock()
        .expect("storage mode test lock");
    let sensitive_fixture_id = "work-item:legacy:/Users/example/PASSWORD=alpha/ACCESS_TOKEN=beta";
    let (_root, state_path, db_path, project_root, sensitive_id) =
        seed_work_item_fixture("ordinary-uow", Some(sensitive_fixture_id));
    let repository =
        crate::workbench_sqlite_storage_mode::primary_repository_for_write(&state_path)
            .expect("candidate rejection DB-primary gate")
            .expect("candidate rejection DB-primary repository");
    let workflow_id = crate::default_workflow_id(&project_root);
    let expected_revision = repository
        .m2_workflow_state_sidecar_revision(&workflow_id, &sensitive_id)
        .expect("read sensitive legacy owner revision");
    let request = crate::WorkItemStateUpdateRequest {
        project_root,
        work_item_id: sensitive_id.clone(),
        next_state: "running".to_string(),
        client_request_ref: None,
        command_id: Some("m4-sensitive-candidate-command".to_string()),
        idempotency_key: Some("m4-sensitive-candidate-key".to_string()),
        expected_revision: Some(expected_revision),
    };
    let sidecar_before = fs::read(&state_path).expect("read sidecar before rejection");
    let connection = Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .expect("open rejection DB before update");
    let counts_before = owner_counts(&connection);
    assert_eq!(counts_before[0], 0, "fixture starts with no M2 event");
    assert_eq!(counts_before[1], 0, "fixture starts with no owner receipt");
    assert_eq!(counts_before[2], 0, "fixture starts with no M2 snapshot");
    assert_eq!(
        counts_before[5], 0,
        "fixture starts with no M4 owner outbox"
    );
    let owner_record_before: String = connection
        .query_row(
            "SELECT record_json FROM work_items WHERE work_item_id = ?1",
            [sensitive_id.as_str()],
            |row| row.get(0),
        )
        .expect("read owner fact before rejection");
    drop(connection);

    let first_error = crate::update_work_item_state_at(&state_path, &request)
        .expect_err("sensitive owner key must reject publication");
    assert_eq!(
        first_error,
        "ordinary_product_work_item_source_publication_rejected"
    );
    assert_eq!(
        fs::read(&state_path).expect("read sidecar after rejection"),
        sidecar_before,
        "failed owner UoW cannot advance its JSON projection"
    );
    let connection = Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .expect("open rejection DB after update");
    assert_eq!(owner_counts(&connection), counts_before);
    let owner_record_after: String = connection
        .query_row(
            "SELECT record_json FROM work_items WHERE work_item_id = ?1",
            [sensitive_id.as_str()],
            |row| row.get(0),
        )
        .expect("read owner fact after rejection");
    assert_eq!(
        owner_record_after, owner_record_before,
        "failed owner UoW cannot change the authoritative WorkItem fact"
    );
    let rejection: (String, String, String, String, String, String) = connection
        .query_row(
            "SELECT rejection_receipt_ref, adapter_id,
                    sealed_candidate_event_ref, candidate_payload_hash,
                    reason_code, resolution_state
             FROM m4_source_owner_candidate_rejections",
            [],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            },
        )
        .expect("read scrubbed candidate rejection");
    assert_eq!(
        rejection.1,
        crate::m4_source_owner_schema::M4_WORK_ITEM_SOURCE_ADAPTER_ID
    );
    assert_eq!(rejection.4, "OWNER_PUBLICATION_IDENTIFIER_REJECTED");
    assert_eq!(rejection.5, "HELD");
    for stored in [
        &rejection.0,
        &rejection.1,
        &rejection.2,
        &rejection.3,
        &rejection.4,
        &rejection.5,
    ] {
        for marker in ["/Users/", "PASSWORD", "ACCESS_TOKEN", &sensitive_id] {
            assert!(!stored.contains(marker));
        }
    }
    drop(connection);

    let replay_error = crate::update_work_item_state_at(&state_path, &request)
        .expect_err("exact rejected candidate replay remains rejected");
    assert_eq!(replay_error, first_error);
    let connection = Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .expect("open rejection DB after replay");
    assert_eq!(owner_counts(&connection), counts_before);
    let rejection_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM m4_source_owner_candidate_rejections",
            [],
            |row| row.get(0),
        )
        .expect("count idempotent rejected candidates");
    assert_eq!(rejection_count, 1);
    drop(connection);

    let distinct_request = crate::WorkItemStateUpdateRequest {
        project_root: request.project_root.clone(),
        work_item_id: request.work_item_id.clone(),
        next_state: request.next_state.clone(),
        client_request_ref: None,
        command_id: Some("m4-sensitive-candidate-command-distinct".to_string()),
        idempotency_key: Some("m4-sensitive-candidate-key-distinct".to_string()),
        expected_revision: request.expected_revision,
    };
    let distinct_error = crate::update_work_item_state_at(&state_path, &distinct_request)
        .expect_err("different stable rejected candidate remains rejected");
    assert_eq!(distinct_error, first_error);
    let connection = Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .expect("open rejection DB after distinct candidate");
    assert_eq!(owner_counts(&connection), counts_before);
    let distinct_rejection_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM m4_source_owner_candidate_rejections",
            [],
            |row| row.get(0),
        )
        .expect("count distinct rejected candidates");
    assert_eq!(distinct_rejection_count, 2);
}
