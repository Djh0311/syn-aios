#[test]
fn station4_dispatch_uses_exact_task_scope_and_keeps_review_provenance() {
    let fixture = Fixture::new();
    fixture.set_allowed_checks(vec!["交付文件必须为 8 字节且末尾无换行".to_string()]);
    fixture.add_read_only_review_task();
    let adapter = TaskScopedAdapter::default();

    execute_supervisor_action(
        &fixture.runtime,
        Fixture::dispatch_proposal(WORK_ITEM),
        &adapter,
    )
    .expect("writer dispatch");
    execute_supervisor_action(
        &fixture.runtime,
        Fixture::dispatch_proposal(REVIEW_WORK_ITEM),
        &adapter,
    )
    .expect("read-only review dispatch");

    let dispatch_scopes = adapter.dispatch_scopes.borrow();
    assert_eq!(
        dispatch_scopes.as_slice(),
        &[
            (WORK_ITEM.to_string(), vec![PROJECT.to_string()]),
            (REVIEW_WORK_ITEM.to_string(), vec![]),
        ]
    );
    let store = load_store(&fixture.path).expect("action store after dispatches");
    let writer_fingerprint = store
        .actions
        .iter()
        .find(|record| record.target_work_item_id.as_deref() == Some(WORK_ITEM))
        .expect("writer dispatch record")
        .task_package_fingerprint
        .clone();
    let review_fingerprint = store
        .actions
        .iter()
        .find(|record| record.target_work_item_id.as_deref() == Some(REVIEW_WORK_ITEM))
        .expect("review dispatch record")
        .task_package_fingerprint
        .clone();
    assert_ne!(writer_fingerprint, review_fingerprint);

    execute_supervisor_action(
        &fixture.runtime,
        Fixture::inspect_proposal("worker-review"),
        &adapter,
    )
    .expect("review inspect");
    let store = load_store(&fixture.path).expect("action store after review inspect");
    let review_inspect = store.actions.last().expect("review inspect record");
    assert_eq!(review_inspect.kind, "inspect_worker");
    assert_eq!(review_inspect.worker_id.as_deref(), Some("worker-review"));
    assert_eq!(review_inspect.task_package_fingerprint, review_fingerprint);

    let final_guard =
        guard_action(&fixture.runtime, &Fixture::proposal("finalize")).expect("finalize guard");
    assert_ne!(final_guard.task_package_fingerprint, writer_fingerprint);
    assert_ne!(final_guard.task_package_fingerprint, review_fingerprint);
    assert!(final_guard.allowed_write_roots.is_empty());
}

#[test]
fn non_byte_legacy_dispatch_keeps_authorization_scope_without_task_write_field() {
    let fixture = Fixture::new();
    fixture.remove_writer_task_allowed_write();
    let adapter = TaskScopedAdapter::default();

    let result = execute_supervisor_action(
        &fixture.runtime,
        Fixture::dispatch_proposal(WORK_ITEM),
        &adapter,
    )
    .expect("legacy non-byte dispatch remains allowed");

    assert_eq!(result.status, "completed");
    assert_eq!(
        adapter.dispatch_scopes.borrow().as_slice(),
        &[(WORK_ITEM.to_string(), vec![PROJECT.to_string()])]
    );
}

#[test]
fn station4_byte_checks_defer_pass_diagnostic_to_control_core_evidence_gate() {
    let fixture = Fixture::new();
    fixture.set_allowed_checks(vec!["交付文件必须为 8 字节且末尾无换行".to_string()]);
    let adapter = FakeAdapter {
        dispatches: Cell::new(0),
    };

    let result =
        execute_supervisor_action(&fixture.runtime, Fixture::proposal("finalize"), &adapter)
            .expect("byte evidence gate is delegated to control core");
    assert_eq!(result.status, "completed");
    assert_eq!(adapter.dispatches.get(), 0);
}
