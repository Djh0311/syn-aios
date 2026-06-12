    #[test]
    fn memory_candidate_store_keeps_candidates_out_of_formal_memory() {
        let dir =
            std::env::temp_dir().join(format!("memory-candidate-store-{}", unix_timestamp_nanos()));
        fs::create_dir_all(&dir).expect("temp dir should exist");
        let path = dir.join("workflow-state.v0.json");
        let create = CreateMemoryCandidateInput {
            project_root: "/offline-fixture/projects/codex-workbench".to_string(),
            project_id: Some("project:offline".to_string()),
            workflow_id: Some("workflow:offline:default".to_string()),
            scope: MemoryScope {
                scope_id: "scope:user:yoyi".to_string(),
                scope_type: "user_preference".to_string(),
                user_id: Some("yoyi".to_string()),
                project_id: None,
                workflow_id: None,
                session_id: None,
                role_ids: vec![],
                document_refs: vec![],
                permission_policy_ref: None,
                model_export_policy: "local_only".to_string(),
                valid_from: "2026-06-03T00:00:00Z".to_string(),
                valid_until: None,
            },
            memory_type: "user_preference".to_string(),
            claim: "用户要求先指出风险。".to_string(),
            body: "这是候选，不是正式长期记忆。".to_string(),
            source_refs: vec![MemorySourceRef {
                source_ref_id: "source:user-confirmed:001".to_string(),
                source_type: "user_confirmed_proposal".to_string(),
                source_id: Some("task:offline".to_string()),
                source_path: None,
                source_title: Some("离线确认".to_string()),
                anchor: None,
                source_created_at: None,
                captured_at: "2026-06-03T00:00:00Z".to_string(),
                authority_level: "user_confirmed".to_string(),
                sensitive_level: "private".to_string(),
                content_hash: None,
            }],
            generated_by_role: "user".to_string(),
            generated_from: "explicit_user_confirmation".to_string(),
            risk_level: "low".to_string(),
            sensitive_level: "private".to_string(),
            requires_user_confirmation: true,
            review_reason: "离线候选治理测试".to_string(),
            expected_store_revision: None,
        };

        let created = memory_candidate_store::create_candidate(
            &path,
            &create,
            "2026-06-03T00:00:00Z",
            "write-memory-001",
        )
        .expect("memory candidate should be created");
        assert_eq!(created.store_revision, 1);
        assert_eq!(
            created.candidate.status,
            MemoryLifecycleStatus::CandidateNeedsReview
        );
        let decided = memory_candidate_store::record_decision(
            &path,
            &RecordMemoryCandidateDecisionInput {
                project_root: create.project_root.clone(),
                candidate_key: created.candidate.candidate_key.clone(),
                requested_status: MemoryLifecycleStatus::CandidateConfirmed,
                reason: "确认保留候选；不写正式记忆。".to_string(),
                actor_id: "user".to_string(),
                actor_role: "user".to_string(),
                expected_store_revision: Some(1),
            },
            "2026-06-03T00:00:01Z",
            "write-memory-002",
        )
        .expect("memory candidate should be confirmed as candidate");
        assert_eq!(decided.store_revision, 2);
        assert_eq!(
            decided.candidate.status,
            MemoryLifecycleStatus::CandidateConfirmed
        );
        assert!(
            !path.exists(),
            "memory candidate write must not create workflow state JSON"
        );
        assert!(path
            .parent()
            .expect("path should have parent")
            .join("memory-candidates.v1.json")
            .exists());
        assert!(!path
            .parent()
            .expect("path should have parent")
            .join("blackboard-candidates.v1.json")
            .exists());

        let formal = memory_candidate_store::record_decision(
            &path,
            &RecordMemoryCandidateDecisionInput {
                project_root: create.project_root,
                candidate_key: decided.candidate.candidate_key,
                requested_status: MemoryLifecycleStatus::MemoryActive,
                reason: "禁止正式晋升测试".to_string(),
                actor_id: "user".to_string(),
                actor_role: "user".to_string(),
                expected_store_revision: Some(2),
            },
            "2026-06-03T00:00:02Z",
            "write-memory-003",
        )
        .unwrap_err();
        assert!(formal.contains("不能请求正式记忆状态"));
    }

    #[test]
    fn candidate_sidecars_are_isolated_and_damaged_json_is_not_overwritten() {
        let dir = std::env::temp_dir().join(format!(
            "candidate-sidecar-isolation-{}",
            unix_timestamp_nanos()
        ));
        fs::create_dir_all(&dir).expect("temp dir should exist");
        let path = dir.join("workflow-state.v0.json");
        let memory_path = dir.join("memory-candidates.v1.json");
        fs::write(&memory_path, "{not valid json").expect("damaged memory sidecar should write");
        let err = memory_candidate_store::load_store(&path, "2026-06-03T00:00:00Z").unwrap_err();
        assert!(err.contains("JSON 损坏"));
        assert_eq!(
            fs::read_to_string(&memory_path).expect("damaged file should remain"),
            "{not valid json"
        );
        let blackboard = blackboard_candidate_store::load_store(&path, "2026-06-03T00:00:00Z")
            .expect("blackboard store should ignore damaged memory sidecar");
        assert_eq!(blackboard.revision, 0);
        assert!(!dir.join("blackboard-candidates.v1.json").exists());
    }

    #[test]
    fn formal_memory_store_creates_record_version_and_audit() {
        let dir = test_temp_dir("formal-memory-create");
        fs::create_dir_all(&dir).expect("temp dir should exist");
        let path = dir.join("workflow-state.v0.json");
        let input = fixture_formal_memory_input();

        let created = formal_memory_store::create_record(
            &path,
            &input,
            "2026-06-03T00:00:00Z",
            "write-formal-001",
        )
        .expect("formal memory should create record, version, and audit");

        assert_eq!(created.store_revision, 1);
        assert_eq!(created.record.status, MemoryLifecycleStatus::MemoryActive);
        assert_eq!(created.version.memory_id, created.record.memory_id);
        assert_eq!(created.version.version_number, 1);
        assert_eq!(created.version.record_snapshot, created.record);
        assert_eq!(created.audit_event.event_type, "memory_record_created");
        assert_eq!(created.audit_event.status, "succeeded");

        let store = formal_memory_store::load_store(&path, "2026-06-03T00:00:01Z")
            .expect("formal memory store should load");
        assert_eq!(store.store_version, "formal_memory_store.v1");
        assert_eq!(store.revision, 1);
        assert_eq!(store.records.len(), 1);
        assert_eq!(store.versions.len(), 1);
        assert_eq!(store.audit_events.len(), 1);
        assert!(dir.join("formal-memories.v1.json").exists());
        assert!(
            !path.exists(),
            "formal memory sidecar must not create workflow state JSON"
        );

        let read_model = formal_memory_store::summarize_store(&store);
        assert_eq!(read_model.sidecar_name, "formal-memories.v1.json");
        assert_eq!(read_model.record_count, 1);
        assert_eq!(read_model.active_count, 1);
        assert_eq!(read_model.version_count, 1);
        assert_eq!(read_model.audit_event_count, 1);
        assert_eq!(
            read_model
                .recent_audit_event
                .expect("recent audit should exist")
                .event_type,
            "memory_record_created"
        );
    }

    #[test]
    fn formal_memory_context_accepts_matching_project_and_workflow() {
        let dir = test_temp_dir("formal-memory-context-accept");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/formal-memory-context-project";
        let project = fixture_project(project_root);
        bootstrap_project_workflow_at(&path, &project)
            .expect("workflow state should include project");
        let input = fixture_bound_formal_memory_input(project_root);

        let created = create_formal_memory_record_at(
            &path,
            &input,
            "2026-06-03T00:00:00Z",
            "write-formal-context-accept",
        )
        .expect("matching context should create formal memory");

        assert_eq!(created.store_revision, 1);
        assert_eq!(
            created.record.scope.project_id.as_deref(),
            input.project_id.as_deref()
        );
        assert_eq!(
            created.record.scope.workflow_id.as_deref(),
            input.workflow_id.as_deref()
        );
        assert!(dir.join("formal-memories.v1.json").exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn formal_memory_context_rejects_mismatched_project_id() {
        let dir = test_temp_dir("formal-memory-context-project-mismatch");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/formal-memory-context-project";
        let project = fixture_project(project_root);
        bootstrap_project_workflow_at(&path, &project)
            .expect("workflow state should include project");
        let mut input = fixture_bound_formal_memory_input(project_root);
        input.project_id = Some(project_id("/tmp/other-project"));

        let err = create_formal_memory_record_at(
            &path,
            &input,
            "2026-06-03T00:00:00Z",
            "write-formal-context-project-mismatch",
        )
        .unwrap_err();

        assert!(err.contains("project_id 与 project_root 不匹配"));
        assert!(!dir.join("formal-memories.v1.json").exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn formal_memory_context_rejects_mismatched_workflow_id() {
        let dir = test_temp_dir("formal-memory-context-workflow-mismatch");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/formal-memory-context-project";
        let project = fixture_project(project_root);
        bootstrap_project_workflow_at(&path, &project)
            .expect("workflow state should include project");
        let mut input = fixture_bound_formal_memory_input(project_root);
        input.workflow_id = Some(default_workflow_id("/tmp/other-project"));

        let err = create_formal_memory_record_at(
            &path,
            &input,
            "2026-06-03T00:00:00Z",
            "write-formal-context-workflow-mismatch",
        )
        .unwrap_err();

        assert!(err.contains("workflow_id 与 project_root 不匹配"));
        assert!(!dir.join("formal-memories.v1.json").exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn formal_memory_context_rejects_project_director_cross_project() {
        let dir = test_temp_dir("formal-memory-context-cross-project");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/formal-memory-context-project";
        let project = fixture_project(project_root);
        bootstrap_project_workflow_at(&path, &project)
            .expect("workflow state should include project");
        let mut input = fixture_bound_formal_memory_input(project_root);
        input.scope.project_id = Some(project_id("/tmp/other-project"));

        let err = create_formal_memory_record_at(
            &path,
            &input,
            "2026-06-03T00:00:00Z",
            "write-formal-context-cross-project",
        )
        .unwrap_err();

        assert!(err.contains("scope.project_id 与 project_root 不匹配"));
        assert!(!dir.join("formal-memories.v1.json").exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn formal_memory_context_rejects_missing_project_in_workflow_state() {
        let dir = test_temp_dir("formal-memory-context-missing-state-project");
        let path = dir.join("workflow-state.v0.json");
        bootstrap_project_workflow_at(&path, &fixture_project("/tmp/other-project"))
            .expect("workflow state should include only another project");
        let input = fixture_bound_formal_memory_input("/tmp/formal-memory-context-project");

        let err = create_formal_memory_record_at(
            &path,
            &input,
            "2026-06-03T00:00:00Z",
            "write-formal-context-missing-state-project",
        )
        .unwrap_err();

        assert!(err.contains("workflow state projects[] 不包含 project_root"));
        assert!(!dir.join("formal-memories.v1.json").exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn formal_memory_context_keeps_existing_m1_guards() {
        let dir = test_temp_dir("formal-memory-context-keeps-m1");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/formal-memory-context-project";
        let project = fixture_project(project_root);
        bootstrap_project_workflow_at(&path, &project)
            .expect("workflow state should include project");
        let mut input = fixture_bound_formal_memory_input(project_root);
        input.source_refs = vec![];

        let err = create_formal_memory_record_at(
            &path,
            &input,
            "2026-06-03T00:00:00Z",
            "write-formal-context-keeps-m1",
        )
        .unwrap_err();

        assert!(err.contains("正式记忆缺少来源"));
        assert!(!dir.join("formal-memories.v1.json").exists());

        let _ = fs::remove_dir_all(dir);
    }
