#[test]
fn observation_store_records_worker_report() {
    let dir = test_temp_dir("observation-store-records-worker");
    let path = dir.join("workflow-state.v0.json");
    let project_root = "/tmp/observation-store-records-worker";
    bootstrap_project_workflow_at(&path, &fixture_project(project_root))
        .expect("workflow state should include project");

    let created = create_recorded_observation(&path, project_root);

    assert_eq!(created.store_revision, 1);
    assert_eq!(created.observation.status, ObservationStatus::Recorded);
    assert_eq!(created.observation.observation_type, "worker_report");
    assert!(!created.observation.source_refs.is_empty());
    assert!(dir.join("observations.v1.json").exists());
    let summary = observation_store::summarize_store(
        &observation_store::load_store(&path, "2026-06-04T00:00:01Z")
            .expect("observation store should load"),
    );
    assert_eq!(summary.recorded_count, 1);
    assert!(summary.display_text.contains("observation 不是正式记忆"));

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn observation_candidate_creation_project_director() {
    let dir = test_temp_dir("observation-candidate-project-director");
    let path = dir.join("workflow-state.v0.json");
    let project_root = "/tmp/observation-candidate-project-director";
    bootstrap_project_workflow_at(&path, &fixture_project(project_root))
        .expect("workflow state should include project");
    let observation = create_recorded_observation(&path, project_root);

    let created = create_memory_candidate_from_observation_at(
        &path,
        &fixture_observation_candidate_input(
            project_root,
            observation.observation.observation_key.clone(),
            Some(1),
            Some(0),
        ),
        "2026-06-04T00:00:02Z",
        "write-observation-candidate",
        "write-candidate-from-observation",
    )
    .expect("project director should create memory candidate from observation");

    assert_eq!(created.observation_store_revision, 2);
    assert_eq!(created.candidate_store_revision, 1);
    assert_eq!(
        created.candidate.status,
        MemoryLifecycleStatus::CandidateNeedsReview
    );
    assert_eq!(
        created.observation.status,
        ObservationStatus::CandidateCreated
    );
    assert_eq!(
        created.observation.candidate_key.as_deref(),
        Some(created.candidate.candidate_key.as_str())
    );
    assert_eq!(
        created.observation_audit_event.event_type,
        "observation_candidate_created"
    );
    let candidate_store = memory_candidate_store::load_store(&path, "2026-06-04T00:00:03Z")
        .expect("candidate store should load");
    assert_eq!(candidate_store.candidates.len(), 1);

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn observation_candidate_creation_rejects_quarantined() {
    let dir = test_temp_dir("observation-candidate-quarantined");
    let path = dir.join("workflow-state.v0.json");
    let project_root = "/tmp/observation-candidate-quarantined";
    bootstrap_project_workflow_at(&path, &fixture_project(project_root))
        .expect("workflow state should include project");
    let observation = create_recorded_observation(&path, project_root);
    overwrite_first_observation_status(&path, ObservationStatus::Quarantined, None);

    let err = create_memory_candidate_from_observation_at(
        &path,
        &fixture_observation_candidate_input(
            project_root,
            observation.observation.observation_key,
            Some(1),
            Some(0),
        ),
        "2026-06-04T00:00:02Z",
        "write-observation-candidate-quarantined",
        "write-candidate-from-quarantined",
    )
    .unwrap_err();

    assert!(err.contains("当前状态：quarantined"));
    assert!(!dir.join("memory-candidates.v1.json").exists());

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn observation_candidate_creation_rejects_ignored() {
    let dir = test_temp_dir("observation-candidate-ignored");
    let path = dir.join("workflow-state.v0.json");
    let project_root = "/tmp/observation-candidate-ignored";
    bootstrap_project_workflow_at(&path, &fixture_project(project_root))
        .expect("workflow state should include project");
    let observation = create_recorded_observation(&path, project_root);
    overwrite_first_observation_status(&path, ObservationStatus::Ignored, None);

    let err = create_memory_candidate_from_observation_at(
        &path,
        &fixture_observation_candidate_input(
            project_root,
            observation.observation.observation_key,
            Some(1),
            Some(0),
        ),
        "2026-06-04T00:00:02Z",
        "write-observation-candidate-ignored",
        "write-candidate-from-ignored",
    )
    .unwrap_err();

    assert!(err.contains("当前状态：ignored"));
    assert!(!dir.join("memory-candidates.v1.json").exists());

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn observation_candidate_creation_rejects_duplicate() {
    let dir = test_temp_dir("observation-candidate-duplicate");
    let path = dir.join("workflow-state.v0.json");
    let project_root = "/tmp/observation-candidate-duplicate";
    bootstrap_project_workflow_at(&path, &fixture_project(project_root))
        .expect("workflow state should include project");
    let observation = create_recorded_observation(&path, project_root);
    let input = fixture_observation_candidate_input(
        project_root,
        observation.observation.observation_key.clone(),
        Some(1),
        Some(0),
    );
    create_memory_candidate_from_observation_at(
        &path,
        &input,
        "2026-06-04T00:00:02Z",
        "write-observation-candidate-first",
        "write-candidate-from-observation-first",
    )
    .expect("first candidate creation should succeed");

    let err = create_memory_candidate_from_observation_at(
        &path,
        &fixture_observation_candidate_input(
            project_root,
            observation.observation.observation_key,
            Some(2),
            Some(1),
        ),
        "2026-06-04T00:00:03Z",
        "write-observation-candidate-second",
        "write-candidate-from-observation-second",
    )
    .unwrap_err();

    assert!(err.contains("已经生成过 candidate"));
    let candidate_store = memory_candidate_store::load_store(&path, "2026-06-04T00:00:04Z")
        .expect("candidate store should load");
    assert_eq!(candidate_store.candidates.len(), 1);

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn observation_creation_rejects_missing_source_refs() {
    let dir = test_temp_dir("observation-missing-source-refs");
    let path = dir.join("workflow-state.v0.json");
    let project_root = "/tmp/observation-missing-source-refs";
    bootstrap_project_workflow_at(&path, &fixture_project(project_root))
        .expect("workflow state should include project");
    let mut input = fixture_observation_input(project_root);
    input.source_refs = vec![];

    let err = create_observation_at(
        &path,
        &input,
        "2026-06-04T00:00:00Z",
        "write-observation-missing-source",
    )
    .unwrap_err();

    assert!(err.contains("缺少 source_refs"));
    assert!(!dir.join("observations.v1.json").exists());

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn observation_creation_rejects_ordinary_chat_auto_capture() {
    let dir = test_temp_dir("observation-ordinary-chat");
    let path = dir.join("workflow-state.v0.json");
    let project_root = "/tmp/observation-ordinary-chat";
    bootstrap_project_workflow_at(&path, &fixture_project(project_root))
        .expect("workflow state should include project");
    let mut input = fixture_observation_input(project_root);
    input.source_refs[0].source_kind = "ordinary_chat".to_string();
    input.source_refs[0].summary = "普通聊天摘要，未被明确确认为工作流事实。".to_string();

    let err = create_observation_at(
        &path,
        &input,
        "2026-06-04T00:00:00Z",
        "write-observation-ordinary-chat",
    )
    .unwrap_err();

    assert!(err.contains("普通聊天不能自动记录为 observation"));
    assert!(!dir.join("observations.v1.json").exists());

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn observation_candidate_does_not_create_formal_memory() {
    let dir = test_temp_dir("observation-candidate-no-formal");
    let path = dir.join("workflow-state.v0.json");
    let project_root = "/tmp/observation-candidate-no-formal";
    bootstrap_project_workflow_at(&path, &fixture_project(project_root))
        .expect("workflow state should include project");
    let observation = create_recorded_observation(&path, project_root);

    create_memory_candidate_from_observation_at(
        &path,
        &fixture_observation_candidate_input(
            project_root,
            observation.observation.observation_key,
            Some(1),
            Some(0),
        ),
        "2026-06-04T00:00:02Z",
        "write-observation-candidate-no-formal",
        "write-candidate-from-observation-no-formal",
    )
    .expect("candidate should be created");

    let formal_store = formal_memory_store::load_store(&path, "2026-06-04T00:00:03Z")
        .expect("formal store should load empty");
    assert_eq!(formal_store.records.len(), 0);
    assert_eq!(formal_store.versions.len(), 0);
    assert_eq!(formal_store.audit_events.len(), 0);
    assert!(!dir.join("formal-memories.v1.json").exists());

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn observation_context_binding_mismatch_rejected() {
    let dir = test_temp_dir("observation-context-mismatch");
    let path = dir.join("workflow-state.v0.json");
    let project_root = "/tmp/observation-context-mismatch";
    bootstrap_project_workflow_at(&path, &fixture_project(project_root))
        .expect("workflow state should include project");
    let mut input = fixture_observation_input(project_root);
    input.scope.project_id = Some("project:other".to_string());

    let err = create_observation_at(
        &path,
        &input,
        "2026-06-04T00:00:00Z",
        "write-observation-context-mismatch",
    )
    .unwrap_err();

    assert!(err.contains("observation 上下文绑定失败"));
    assert!(err.contains("scope.project_id"));
    assert!(!dir.join("observations.v1.json").exists());

    let _ = fs::remove_dir_all(dir);
}
