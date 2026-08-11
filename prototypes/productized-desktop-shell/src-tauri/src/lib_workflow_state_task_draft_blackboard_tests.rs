    #[test]
    fn missing_workflow_state_returns_empty_without_creating_file() {
        let dir = std::env::temp_dir().join(format!(
            "workflow-state-missing-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");

        let snapshot = read_workflow_state_snapshot(&path)
            .expect("missing state should return empty snapshot");

        assert!(!snapshot.exists);
        assert!(!snapshot.initialized);
        assert_eq!(snapshot.counts.audit_events, 0);
        assert!(!path.exists());
    }
    #[test]
    fn initializes_workflow_state_with_audit_event() {
        let dir =
            std::env::temp_dir().join(format!("workflow-state-init-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");

        let result = initialize_workflow_state_at(&path).expect("initialize should write state");

        assert!(path.exists());
        assert!(result.first_initialize);
        assert!(result.backup_path.is_none());
        assert_eq!(
            result.snapshot.schema_version.as_deref(),
            Some("workflow_state_v0")
        );
        assert_eq!(result.snapshot.workflow_version, Some(1));
        assert_eq!(result.snapshot.counts.audit_events, 1);

        let text = fs::read_to_string(&path).expect("state file should be readable");
        let value: Value = serde_json::from_str(&text).expect("state should be json");
        assert_eq!(
            value["audit_events"][0]["event_type"],
            "workflow_state_initialized"
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn existing_workflow_state_is_backed_up_before_initialize() {
        let dir =
            std::env::temp_dir().join(format!("workflow-state-backup-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        fs::create_dir_all(&dir).expect("fixture dir should be created");
        fs::write(&path, "{\"old\":true}").expect("old state should be written");

        let result =
            initialize_workflow_state_at(&path).expect("initialize should replace old state");
        let backup_path = result
            .backup_path
            .expect("existing state should be backed up");

        assert!(!result.first_initialize);
        assert!(PathBuf::from(&backup_path).exists());
        let backup_text = fs::read_to_string(backup_path).expect("backup should be readable");
        assert_eq!(backup_text, "{\"old\":true}");

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn bootstrap_project_workflow_initializes_missing_state() {
        let dir = std::env::temp_dir().join(format!(
            "workflow-bootstrap-missing-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");

        let result = bootstrap_project_workflow_at(&path, &project)
            .expect("bootstrap should create state and workflow");

        assert!(result.first_initialize);
        assert!(path.exists());
        assert_eq!(result.snapshot.counts.projects, 1);
        assert_eq!(result.snapshot.counts.workflows, 1);
        assert_eq!(result.snapshot.counts.nodes, 7);
        assert_eq!(result.snapshot.counts.edges, 6);
        assert_eq!(result.snapshot.counts.audit_events, 2);

        let value = read_json_file(&path);
        assert_eq!(value["workflows"][0]["state"], "draft");
        assert_eq!(
            value["audit_events"][1]["event_type"],
            "project_default_workflow_bootstrapped"
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn bootstrap_project_workflow_does_not_duplicate_existing_workflow() {
        let dir = std::env::temp_dir().join(format!(
            "workflow-bootstrap-duplicate-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");

        bootstrap_project_workflow_at(&path, &project).expect("first bootstrap should write");
        let second =
            bootstrap_project_workflow_at(&path, &project).expect("second bootstrap should no-op");

        assert_eq!(second.snapshot.counts.workflows, 1);
        assert_eq!(second.snapshot.counts.nodes, 7);
        assert_eq!(second.audit_event_id, "no-op:existing-workflow");

        let value = read_json_file(&path);
        assert_eq!(array_len(&value, "workflows"), 1);
        assert_eq!(array_len(&value, "nodes"), 7);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn bootstrap_project_workflow_rejects_non_index_project() {
        let index = json!({
          "projects": [{ "project_root": "/tmp/indexed-project" }]
        });

        assert!(find_index_project(&index, "/tmp/indexed-project").is_some());
        assert!(find_index_project(&index, "/tmp/not-indexed").is_none());
    }

    #[test]
    fn bootstrap_project_workflow_backs_up_existing_state() {
        let dir = std::env::temp_dir().join(format!(
            "workflow-bootstrap-backup-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");

        initialize_workflow_state_at(&path).expect("initial state should exist");
        let result = bootstrap_project_workflow_at(&path, &project)
            .expect("bootstrap should back up existing state");

        assert!(!result.first_initialize);
        let backup_path = result
            .backup_path
            .expect("existing state should be backed up");
        assert!(PathBuf::from(backup_path).exists());
        assert_eq!(result.snapshot.counts.workflows, 1);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_draft_rejects_missing_workflow_state() {
        let dir = std::env::temp_dir().join(format!(
            "task-draft-missing-state-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let request = fixture_task_draft_request("/tmp/indexed-project", "草稿 A");

        let result = create_task_draft_at(&path, &request);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("工作流状态文件不存在"));
        assert!(!path.exists());
    }

    #[test]
    fn task_draft_rejects_project_without_workflow() {
        let dir = std::env::temp_dir().join(format!(
            "task-draft-missing-workflow-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let request = fixture_task_draft_request("/tmp/indexed-project", "草稿 A");

        initialize_workflow_state_at(&path).expect("state should exist");
        let result = create_task_draft_at(&path, &request);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("还没有本地 workflow"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_draft_creates_work_item_artifact_and_audit() {
        let dir =
            std::env::temp_dir().join(format!("task-draft-create-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let request = fixture_task_draft_request(&project.project_root, "登记任务包草稿");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let result = create_task_draft_at(&path, &request).expect("task draft should be created");

        assert_eq!(result.snapshot.counts.work_items, 1);
        assert_eq!(result.snapshot.counts.artifacts, 1);
        assert_eq!(result.snapshot.project_workflows[0].task_draft_count, 1);
        assert_eq!(
            result.snapshot.project_workflows[0].task_drafts[0].title,
            "登记任务包草稿"
        );
        assert_eq!(
            result.snapshot.project_workflows[0].task_drafts[0]
                .artifact_type
                .as_deref(),
            Some("task_package")
        );

        let value = read_json_file(&path);
        assert_eq!(value["work_items"][0]["title"], "登记任务包草稿");
        assert_eq!(value["work_items"][0]["assigned_role_id"], "codex-dev");
        assert_eq!(value["work_items"][0]["agent_type"], "codex");
        assert_eq!(value["work_items"][0]["adapter_id"], "codex-local");
        assert_eq!(value["artifacts"][0]["artifact_type"], "task_package");
        assert_eq!(
            value["artifacts"][0]["task_goal"],
            "写入 work_items 和 artifacts"
        );
        assert!(value["artifacts"][0]["path"].is_null());
        assert!(value["audit_events"]
            .as_array()
            .expect("audit events should be array")
            .iter()
            .any(|event| event["event_type"] == "task_draft_created"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn new_task_draft_work_item_id_is_fixed_opaque_and_scrubs_owner_inputs() {
        let dir = std::env::temp_dir().join(format!(
            "task-draft-opaque-owner-id-{}",
            unix_timestamp_nanos()
        ));
        let path = dir.join("workflow-state.v0.json");
        let sensitive_path = "/tmp/PASSWORD-project/ACCESS_TOKEN-workspace";
        let project = fixture_project(sensitive_path);
        let long_marker = "SECRET-title-goal-".repeat(64);
        let request = TaskDraftRequest {
            project_root: sensitive_path.to_string(),
            title: long_marker.clone(),
            objective: format!("{long_marker}credential-objective"),
            assigned_role: Some("codex-dev".to_string()),
        };

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &request).expect("sensitive task content remains valid");
        let value = read_json_file(&path);
        let work_item_id = value["work_items"][0]["work_item_id"]
            .as_str()
            .expect("opaque work item id");
        assert!(work_item_id.starts_with("work-item:sha256:"));
        assert_eq!(work_item_id.len(), "work-item:sha256:".len() + 64);
        assert!(work_item_id["work-item:sha256:".len()..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)));
        for marker in [
            "/tmp/",
            "PASSWORD",
            "ACCESS_TOKEN",
            "SECRET",
            "credential",
        ] {
            assert!(!work_item_id.contains(marker));
        }

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_draft_rejects_non_index_project() {
        let dir =
            std::env::temp_dir().join(format!("task-draft-non-index-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let index = json!({
          "projects": [{ "project_root": "/tmp/indexed-project" }]
        });
        let request = fixture_task_draft_request("/tmp/not-indexed", "草稿 A");

        initialize_workflow_state_at(&path).expect("state should exist");
        let result = create_task_draft_for_index_project_at(&path, &index, &request);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("项目不在当前索引内"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_draft_backs_up_existing_state_before_write() {
        let dir =
            std::env::temp_dir().join(format!("task-draft-backup-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let request = fixture_task_draft_request(&project.project_root, "草稿 A");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let before_text = fs::read_to_string(&path).expect("state should be readable");
        let result = create_task_draft_at(&path, &request).expect("task draft should be created");
        let backup_path = result
            .backup_path
            .expect("task draft write should back up old state");

        assert!(PathBuf::from(&backup_path).exists());
        let backup_text = fs::read_to_string(backup_path).expect("backup should be readable");
        assert_eq!(backup_text, before_text);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_draft_does_not_duplicate_same_workflow_title() {
        let dir =
            std::env::temp_dir().join(format!("task-draft-duplicate-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let request = fixture_task_draft_request(&project.project_root, "草稿 A");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &request).expect("first draft should be created");
        let second = create_task_draft_at(&path, &request).expect("duplicate draft should no-op");

        assert_eq!(second.audit_event_id, "no-op:existing-task-draft");
        assert_eq!(second.snapshot.counts.work_items, 1);
        assert_eq!(second.snapshot.counts.artifacts, 1);

        let value = read_json_file(&path);
        assert_eq!(array_len(&value, "work_items"), 1);
        assert_eq!(array_len(&value, "artifacts"), 1);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn work_item_state_update_advances_state_node_and_audit() {
        let dir = std::env::temp_dir().join(format!(
            "work-item-state-advance-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let request = fixture_task_draft_request(&project.project_root, "编排闭环工作项");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &request).expect("work item should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        let update = fixture_work_item_state_update_request(
            &project.project_root,
            &work_item_id,
            "ready_to_dispatch",
        );

        let result =
            update_work_item_state_at(&path, &update).expect("state update should succeed");

        assert_eq!(
            result.snapshot.project_workflows[0].task_drafts[0].state,
            "ready_to_dispatch"
        );
        assert_eq!(
            result.snapshot.project_workflows[0].task_drafts[0].next_states,
            vec!["running".to_string(), "paused".to_string()]
        );
        let updated = read_json_file(&path);
        assert_eq!(updated["work_items"][0]["state"], "ready_to_dispatch");
        assert_eq!(
            updated["work_items"][0]["current_node_id"],
            format!(
                "{}:node:director",
                default_workflow_id(&project.project_root)
            )
        );
        assert!(updated["audit_events"]
            .as_array()
            .expect("audit events should be array")
            .iter()
            .any(|event| event["event_type"] == "work_item_state_changed"
                && event["before_state"] == "draft"
                && event["after_state"] == "ready_to_dispatch"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn work_item_state_update_rejects_illegal_transition() {
        let dir = std::env::temp_dir().join(format!(
            "work-item-state-illegal-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let request = fixture_task_draft_request(&project.project_root, "非法流转工作项");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &request).expect("work item should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        let update =
            fixture_work_item_state_update_request(&project.project_root, &work_item_id, "running");

        let result = update_work_item_state_at(&path, &update);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("非法工作项状态跳转"));
        let updated = read_json_file(&path);
        assert_eq!(updated["work_items"][0]["state"], "draft");

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn workflow_state_store_helpers_preserve_write_and_backup_behavior() {
        let dir = std::env::temp_dir().join(format!(
            "workflow-state-store-boundary-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");

        initialize_workflow_state_at(&path).expect("state should initialize");
        let timestamp = unix_timestamp_string();
        let backup = backup_workflow_state_file(&path, &timestamp).expect("backup should write");
        assert!(backup.exists());

        let value = read_workflow_state_value(&path).expect("state should read");
        assert!(validate_workflow_state(&value).is_empty());
        write_validated_workflow_state(&path, &value).expect("valid state should write");

        let invalid = json!({
            "schema_version": "bad",
            "workflow_version": 1,
        });
        let rejected = write_validated_workflow_state(&path, &invalid);
        assert!(rejected.is_err());
        assert!(rejected.unwrap_err().contains("写入前 schema 校验失败"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn workflow_permission_decision_records_audit_through_control_core() {
        let dir = std::env::temp_dir().join(format!(
            "permission-decision-control-core-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let request = fixture_task_draft_request(&project.project_root, "权限确认工作项");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &request).expect("work item should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        append_fixture_permission_request(&path, &project.project_root, &work_item_id, "pending");

        let result = record_workflow_permission_decision_at(
            &path,
            &WorkflowPermissionDecisionRequest {
                project_root: project.project_root.clone(),
                work_item_id: work_item_id.clone(),
                request_id: "permission:fixture:001".to_string(),
                decision: "approved".to_string(),
            },
        )
        .expect("permission decision should write");

        assert!(result.message.contains("批准"));
        let updated = read_json_file(&path);
        assert_eq!(updated["permission_requests"][0]["status"], "approved");
        assert_eq!(updated["permission_requests"][0]["decision"], "approved");
        assert!(updated["permission_requests"][0]["decided_at"]
            .as_str()
            .is_some());
        assert!(updated["audit_events"]
            .as_array()
            .expect("audit events should be array")
            .iter()
            .any(
                |event| event["event_type"] == "workflow_permission_decision_recorded"
                    && event["target_ref"] == "permission:fixture:001"
                    && event["before_state"] == "pending"
                    && event["after_state"] == "approved"
            ));

        let duplicate = record_workflow_permission_decision_at(
            &path,
            &WorkflowPermissionDecisionRequest {
                project_root: project.project_root.clone(),
                work_item_id,
                request_id: "permission:fixture:001".to_string(),
                decision: "rejected".to_string(),
            },
        );
        assert!(duplicate.is_err());
        assert!(duplicate.unwrap_err().contains("不是 pending"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn workflow_audit_helper_preserves_work_item_state_changed_fields() {
        let event =
            workflow_audit::work_item_state_changed(workflow_audit::WorkItemStateChangedAudit {
                event_id: "audit:fixture".to_string(),
                work_item_id: "work-item:fixture",
                before_state: "draft",
                after_state: "ready_to_dispatch",
                created_at: "2026-06-01T00:00:00Z",
                reason: "fixture reason".to_string(),
            });

        assert_eq!(event["event_id"], "audit:fixture");
        assert_eq!(event["event_type"], "work_item_state_changed");
        assert_eq!(event["target_ref"], "work-item:fixture");
        assert_eq!(event["actor_ref"], "user_confirmed_desktop_shell");
        assert_eq!(event["source_kind"], "workspace_state");
        assert_eq!(event["permission_level"], "user_confirmed_write");
        assert_eq!(event["before_state"], "draft");
        assert_eq!(event["after_state"], "ready_to_dispatch");
        assert_eq!(event["created_at"], "2026-06-01T00:00:00Z");
        assert_eq!(event["reason"], "fixture reason");
    }

    #[test]
    fn workflow_audit_helper_preserves_permission_decision_fields() {
        let event = workflow_audit::workflow_permission_decision_recorded(
            workflow_audit::WorkflowPermissionDecisionRecordedAudit {
                event_id: "audit:permission:fixture".to_string(),
                request_id: "permission:fixture",
                before_state: "pending",
                after_state: "approved",
                created_at: "2026-06-01T00:00:00Z",
            },
        );

        assert_eq!(event["event_id"], "audit:permission:fixture");
        assert_eq!(event["event_type"], "workflow_permission_decision_recorded");
        assert_eq!(event["target_ref"], "permission:fixture");
        assert_eq!(event["actor_ref"], "user_confirmed_desktop_shell");
        assert_eq!(event["source_kind"], "workspace_state_permission_queue");
        assert_eq!(event["permission_level"], "user_confirmed_write");
        assert_eq!(event["before_state"], "pending");
        assert_eq!(event["after_state"], "approved");
        assert_eq!(event["created_at"], "2026-06-01T00:00:00Z");
        assert_eq!(
            event["reason"],
            "用户确认记录权限请求结论；不启动 Codex、不 resume、不发送消息。"
        );
    }

    #[test]
    fn blackboard_candidate_decision_boundary_rejects_direct_promotion() {
        assert_eq!(
            control_core::validate_blackboard_candidate_decision(
                "memory_candidate",
                "formal_memory",
                "mark_pending",
            )
            .expect("pending should be allowed"),
            control_core::BlackboardCandidateDecisionOutcome::Pending
        );
        assert_eq!(
            control_core::validate_blackboard_candidate_decision(
                "risk",
                "workflow_risk",
                "reject_candidate",
            )
            .expect("reject should be allowed"),
            control_core::BlackboardCandidateDecisionOutcome::Rejected
        );
        assert!(control_core::validate_blackboard_candidate_decision(
            "memory_candidate",
            "formal_memory",
            "confirm_candidate",
        )
        .unwrap_err()
        .contains("不能直接写正式记忆"));
        assert!(control_core::validate_blackboard_candidate_decision(
            "knowledge_ref",
            "formal_memory",
            "confirm_candidate",
        )
        .unwrap_err()
        .contains("知识引用不是记忆"));
        assert!(control_core::validate_blackboard_candidate_decision(
            "tool_summary",
            "workflow_state_change",
            "confirm_candidate",
        )
        .unwrap_err()
        .contains("不能直接推进 workflow state"));
        assert_eq!(
            control_core::validate_blackboard_candidate_decision(
                "subagent_report",
                "workflow_fact",
                "candidate_confirmed_for_followup",
            )
            .expect("followup confirmation should be a candidate-only state"),
            control_core::BlackboardCandidateDecisionOutcome::ConfirmedForFollowup
        );
        assert_eq!(
            control_core::validate_blackboard_candidate_decision(
                "risk",
                "workflow_risk",
                "candidate_deferred",
            )
            .expect("defer should be a candidate-only state"),
            control_core::BlackboardCandidateDecisionOutcome::Deferred
        );
        assert_eq!(
            control_core::validate_blackboard_candidate_decision(
                "tool_summary",
                "audit_event",
                "candidate_discarded",
            )
            .expect("discard should be a candidate-only state"),
            control_core::BlackboardCandidateDecisionOutcome::Discarded
        );
        assert!(
            control_core::validate_blackboard_candidate_decision(
                "memory_candidate",
                "formal_memory",
                "candidate_confirmed_for_memory",
            )
            .is_err(),
            "blackboard candidate confirmation must not promote directly to memory"
        );
    }

    #[test]
    fn blackboard_candidate_store_records_candidate_only_decisions() {
        let dir = std::env::temp_dir().join(format!(
            "blackboard-candidate-store-{}",
            unix_timestamp_nanos()
        ));
        fs::create_dir_all(&dir).expect("temp dir should exist");
        let path = dir.join("workflow-state.v0.json");
        let request = RecordBlackboardCandidateDecisionInput {
            project_id: "project:offline".to_string(),
            project_root: "/offline-fixture/projects/codex-workbench".to_string(),
            workflow_id: "workflow:offline:default".to_string(),
            candidate_key: None,
            source_entry_id: Some("blackboard:offline:report:001".to_string()),
            entry_kind: BlackboardEntryKind::SubagentReport,
            target_kind: BlackboardCandidateTargetKind::WorkflowFact,
            requested_state: BlackboardCandidateState::CandidateConfirmedForFollowup,
            reason: "候选值得后续处理；不写正式事实。".to_string(),
            actor_role: "project_director".to_string(),
            actor_session_id: None,
            source_refs: vec![BlackboardCandidateSourceRef {
                source_kind: "subagent_report".to_string(),
                source_id: "report:offline:001".to_string(),
                label: "子智能体汇报".to_string(),
            }],
            expected_store_revision: None,
            title_snapshot: Some("离线子汇报".to_string()),
            summary_snapshot: Some("只确认后续处理。".to_string()),
            source_status: None,
            work_item_id: None,
            workflow_node_id: None,
        };

        let result = blackboard_candidate_store::record_decision(
            &path,
            &request,
            "2026-06-03T00:00:00Z",
            "write-blackboard-001",
        )
        .expect("blackboard candidate decision should write sidecar");
        assert_eq!(result.store_revision, 1);
        assert_eq!(
            result.record.state,
            BlackboardCandidateState::CandidateConfirmedForFollowup
        );
        assert!(path
            .parent()
            .expect("path should have parent")
            .join("blackboard-candidates.v1.json")
            .exists());
        assert!(
            !path.exists(),
            "blackboard sidecar write must not create workflow state JSON"
        );

        let conflict = blackboard_candidate_store::record_decision(
            &path,
            &RecordBlackboardCandidateDecisionInput {
                expected_store_revision: Some(0),
                requested_state: BlackboardCandidateState::CandidateRejected,
                reason: "并发冲突测试".to_string(),
                ..request.clone()
            },
            "2026-06-03T00:00:01Z",
            "write-blackboard-002",
        )
        .unwrap_err();
        assert!(conflict.contains("blackboard_candidate_store_conflict"));
    }
