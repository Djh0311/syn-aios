    #[test]
    fn project_director_task_plan_rejects_without_active_c3_authorization() {
        let timestamp_ms = 1_765_300_000_000;
        let dir = test_temp_dir("project-director-task-plan-no-c3-active");
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/c4-no-c3-active");
        let thread_id = "thread-c4-no-c3";
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let mut proposal_input = fixture_project_consultation_proposal_input(&project.project_root);
        proposal_input.scope_draft.allowed_agent_ids = vec![thread_id.to_string()];
        let created = project_consultation_proposal_store::create_proposal(
            &path,
            &proposal_input,
            timestamp_ms,
            "write-c4-no-c3-proposal-create",
        )
        .expect("proposal should create");
        let confirmed = project_consultation_proposal_store::record_decision(
            &path,
            &RecordProjectConsultationProposalDecisionInput {
                project_root: project.project_root.clone(),
                proposal_id: created.proposal.proposal_id.clone(),
                actor_id: "user-fixture".to_string(),
                decision: ProjectConsultationProposalDecisionKind::Confirm,
                summary: "用户确认 C4 fixture 方案；尚未全局复核。".to_string(),
                expected_proposal_store_revision: Some(created.store_revision),
                expected_plan_authorization_store_revision: None,
            },
            timestamp_ms + 1,
            "write-c4-no-c3-proposal-confirm",
            "write-c4-no-c3-plan-auth",
            "write-c4-no-c3-plan-auth-user",
        )
        .expect("proposal confirmation should create pending authorization");
        let authorization = confirmed
            .plan_authorization
            .expect("confirmed proposal should link authorization");
        let revision = confirmed
            .plan_authorization_store_revision
            .expect("confirmed proposal should return revision");
        let index = fixture_dispatch_index(&project.project_root, thread_id);
        let preview_input = fixture_project_director_preview_input(
            &project.project_root,
            &confirmed.proposal.proposal_id,
            &authorization.authorization_id,
            revision,
        );
        let prepare_input = fixture_project_director_prepare_input(
            &project.project_root,
            &confirmed.proposal.proposal_id,
            &authorization.authorization_id,
            revision,
            vec![],
        );

        let preview_error =
            preview_project_director_task_plan_for_index_at(&path, &index, &preview_input)
                .expect_err("preview should reject missing C3 approval");
        let prepare_error =
            prepare_authorized_auto_dispatch_for_index_at(&path, &index, &prepare_input)
                .expect_err("prepare should reject missing C3 approval");

        assert!(preview_error.contains("C3 approved"), "{preview_error}");
        assert!(prepare_error.contains("C3 approved"), "{prepare_error}");

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn project_director_task_plan_rejects_proposal_authorization_mismatch() {
        let timestamp_ms = 1_765_300_000_000;
        let dir = test_temp_dir("project-director-task-plan-mismatch");
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/c4-proposal-authorization-mismatch");
        let thread_id = "thread-c4-mismatch";
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let (proposal, authorization, revision) =
            create_active_project_director_authorization_fixture(
                &path,
                &project.project_root,
                thread_id,
                timestamp_ms,
            );
        let mut proposal_store =
            project_consultation_proposal_store::load_store(&path, timestamp_ms + 4)
                .expect("proposal store should load");
        proposal_store.proposals[0].plan_authorization_id = Some("plan-auth:wrong".to_string());
        fs::write(
            project_consultation_proposal_store::sidecar_path(&path)
                .expect("proposal sidecar path"),
            serde_json::to_string_pretty(&proposal_store).expect("proposal store should serialize"),
        )
        .expect("mutated proposal store should write");
        let index = fixture_dispatch_index(&project.project_root, thread_id);
        let preview_input = fixture_project_director_preview_input(
            &project.project_root,
            &proposal.proposal_id,
            &authorization.authorization_id,
            revision,
        );

        let err = preview_project_director_task_plan_for_index_at(&path, &index, &preview_input)
            .expect_err("mismatched back link should reject C4 preview");

        assert!(err.contains("授权回链"), "{err}");

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn project_director_task_plan_blocks_out_of_scope_planned_task() {
        let timestamp_ms = 1_765_300_000_000;
        let dir = test_temp_dir("project-director-task-plan-out-of-scope");
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/c4-out-of-scope");
        let thread_id = "thread-c4-out-of-scope";
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let (proposal, authorization, revision) =
            create_active_project_director_authorization_fixture(
                &path,
                &project.project_root,
                thread_id,
                timestamp_ms,
            );
        let index = fixture_dispatch_index(&project.project_root, thread_id);
        let preview_input = fixture_project_director_preview_input(
            &project.project_root,
            &proposal.proposal_id,
            &authorization.authorization_id,
            revision,
        );
        let plan = preview_project_director_task_plan_for_index_at(&path, &index, &preview_input)
            .expect("preview should build deterministic plan");
        let mut planned_task = plan.planned_tasks[0].clone();
        planned_task.scope.allowed_write_scope = vec!["/tmp/c4-outside-write".to_string()];
        planned_task.scope.callable_tool_capabilities = vec!["network_access".to_string()];
        planned_task.scope.required_checks = vec!["npm run deploy".to_string()];
        planned_task.scope.task_package_kind = "unapproved_kind".to_string();
        let prepare_input = fixture_project_director_prepare_input(
            &project.project_root,
            &proposal.proposal_id,
            &authorization.authorization_id,
            revision,
            vec![planned_task],
        );

        let result = prepare_authorized_auto_dispatch_for_index_at(&path, &index, &prepare_input)
            .expect("blocked planned task should record blocked summary");
        let updated = read_json_file(&path);

        assert_eq!(result.plan.blocked_count, 1);
        assert_eq!(result.plan.prepared_dispatch_count, 0);
        assert!(result.plan.blocked_reasons.iter().any(|reason| {
            reason.contains("写入范围超出方案授权")
                || reason.contains("工具超出方案授权")
                || reason.contains("task package kind 超出方案授权")
        }));
        assert!(updated["workflow_node_dispatches"]
            .as_array()
            .map_or(true, Vec::is_empty));
        assert!(updated["audit_events"]
            .as_array()
            .expect("audit events should be array")
            .iter()
            .any(|event| event["event_type"] == "authorized_prepared_dispatch_blocked"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn authorized_prepared_dispatch_needs_binding_without_executable_dispatch() {
        let timestamp_ms = 1_765_300_000_000;
        let dir = test_temp_dir("authorized-prepared-dispatch-needs-binding");
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/c4-needs-binding");
        let thread_id = "thread-c4-needs-binding";
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let (proposal, authorization, revision) =
            create_active_project_director_authorization_fixture(
                &path,
                &project.project_root,
                thread_id,
                timestamp_ms,
            );
        let index = fixture_dispatch_index(&project.project_root, thread_id);
        let prepare_input = fixture_project_director_prepare_input(
            &project.project_root,
            &proposal.proposal_id,
            &authorization.authorization_id,
            revision,
            vec![],
        );

        let result = prepare_authorized_auto_dispatch_for_index_at(&path, &index, &prepare_input)
            .expect("missing binding should write setup artifacts but no prepared dispatch");
        let updated = read_json_file(&path);

        assert_eq!(result.plan.needs_binding_count, 1);
        assert_eq!(result.plan.prepared_dispatch_count, 0);
        assert!(result
            .plan
            .blocked_reasons
            .iter()
            .any(|reason| reason.contains("等待绑定会话")));
        assert!(updated["work_items"]
            .as_array()
            .expect("work items should be array")
            .iter()
            .any(|item| item["source_kind"] == "project_director_task_plan"));
        assert!(updated["artifacts"]
            .as_array()
            .expect("artifacts should be array")
            .iter()
            .any(|artifact| artifact["memory_packet_snapshot"].is_object()));
        assert!(updated["workflow_node_dispatches"]
            .as_array()
            .map_or(true, Vec::is_empty));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn authorized_prepared_dispatch_creates_memory_snapshot_and_remains_unexecuted_and_idempotent()
    {
        let timestamp_ms = 1_765_300_000_000;
        let dir = test_temp_dir("authorized-prepared-dispatch-created");
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/c4-prepared-dispatch");
        let thread_id = "thread-c4-prepared";
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let (proposal, authorization, revision) =
            create_active_project_director_authorization_fixture(
                &path,
                &project.project_root,
                thread_id,
                timestamp_ms,
            );
        let index = fixture_dispatch_index(&project.project_root, thread_id);
        let workflow_id = default_workflow_id(&project.project_root);
        let node_id = format!("{workflow_id}:node:codex-dev");
        bind_workflow_node_codex_session_for_index_at(
            &path,
            &index,
            &fixture_node_session_bind_request(&project.project_root, &node_id, None, thread_id),
        )
        .expect("node-level binding should write");
        let prepare_input = fixture_project_director_prepare_input(
            &project.project_root,
            &proposal.proposal_id,
            &authorization.authorization_id,
            revision,
            vec![],
        );

        let first = prepare_authorized_auto_dispatch_for_index_at(&path, &index, &prepare_input)
            .expect("active binding should create prepared dispatch");
        let second = prepare_authorized_auto_dispatch_for_index_at(&path, &index, &prepare_input)
            .expect("repeated prepare should be idempotent");
        let updated = read_json_file(&path);
        let dispatches = updated["workflow_node_dispatches"]
            .as_array()
            .expect("dispatches should be array");
        let dispatch = dispatches.first().expect("prepared dispatch should exist");
        let artifact = updated["artifacts"]
            .as_array()
            .expect("artifacts should be array")
            .iter()
            .find(|artifact| artifact["source_kind"] == "project_director_task_plan")
            .expect("task package artifact should exist");

        assert_eq!(first.plan.prepared_dispatch_count, 1);
        assert_eq!(first.plan.needs_binding_count, 0);
        assert_eq!(first.prepared_dispatches.len(), 1);
        assert_eq!(second.plan.prepared_dispatch_count, 1);
        assert_eq!(
            dispatches.len(),
            1,
            "duplicate prepare must not duplicate dispatch"
        );
        assert_eq!(dispatch["state"], "prepared");
        assert_eq!(dispatch["prompt_kind"], "authorized_prepared_auto_dispatch");
        assert_eq!(
            dispatch["plan_authorization_id"],
            authorization.authorization_id
        );
        assert_eq!(dispatch["authorization_check"]["status"], "authorized");
        assert!(dispatch["prompt_preview"]
            .as_str()
            .unwrap_or("")
            .contains("prepared dispatch 只是工作台准备态记录"));
        assert!(dispatch["memory_packet_snapshot_id"].is_string());
        assert!(dispatch["memory_packet_fingerprint"].is_string());
        assert!(dispatch["started_at_ms"].is_null());
        assert!(dispatch["ended_at_ms"].is_null());
        assert!(dispatch["exit_code"].is_null());
        assert!(dispatch["last_message_path"].is_null());
        assert!(dispatch["last_message_summary"].is_null());
        assert!(dispatch["transcript_event_count"].is_null());
        assert!(dispatch["transcript_target_hits"].is_null());
        assert!(artifact["memory_packet_snapshot"].is_object());
        assert_eq!(
            artifact["memory_packet_snapshot"]["schema_version"],
            "task_package_memory_packet_snapshot.v1"
        );
        assert!(updated["audit_events"]
            .as_array()
            .expect("audit events should be array")
            .iter()
            .any(|event| event["event_type"] == "authorized_prepared_dispatch_created"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn worker_structured_report_rejects_missing_evidence_and_ordinary_chat_source() {
        let (dir, path, project, work_item_id, dispatch_id, node_id) =
            setup_c5_worker_report_fixture("c5-worker-report-invalid");
        let mut input = fixture_c5_worker_report_input(
            &project.project_root,
            &work_item_id,
            &dispatch_id,
            &node_id,
        );
        input.evidence_refs.clear();

        let err = record_worker_structured_report_at(&path, &input)
            .expect_err("worker report without evidence should be rejected");
        assert!(err.contains("evidence_refs"), "{err}");

        let mut input = fixture_c5_worker_report_input(
            &project.project_root,
            &work_item_id,
            &dispatch_id,
            &node_id,
        );
        input.source_refs[0].source_kind = "ordinary_chat".to_string();
        let err = record_worker_structured_report_at(&path, &input)
            .expect_err("ordinary chat source should be rejected");
        assert!(err.contains("普通聊天来源"), "{err}");

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn worker_structured_report_records_audit_without_observation_or_formal_memory() {
        let (dir, path, project, work_item_id, dispatch_id, node_id) =
            setup_c5_worker_report_fixture("c5-worker-report-audit-only");
        let input = fixture_c5_worker_report_input(
            &project.project_root,
            &work_item_id,
            &dispatch_id,
            &node_id,
        );

        let output = record_worker_structured_report_at(&path, &input)
            .expect("worker report should write audit event only");
        let updated = read_json_file(&path);
        let snapshot = read_workflow_state_snapshot(&path).expect("snapshot should read");
        let report = snapshot.project_workflows[0]
            .derived_workflow
            .as_ref()
            .expect("derived workflow should exist")
            .subagent_reports
            .iter()
            .find(|report| report.report_id == output.audit_event_id)
            .expect("worker structured report should derive as subagent report");

        assert_eq!(report.acceptance_status, "reported_completed");
        assert!(report
            .warnings
            .contains(&"worker_report_is_not_formal_fact".to_string()));
        assert!(updated["audit_events"]
            .as_array()
            .expect("audit events should be array")
            .iter()
            .any(
                |event| event["event_type"] == "worker_structured_report_recorded"
                    && event["event_id"] == output.audit_event_id
            ));
        assert!(
            !observation_store::sidecar_path(&path)
                .expect("observation sidecar path")
                .exists(),
            "worker report must not automatically create observation store"
        );
        assert!(
            !formal_memory_store::sidecar_path(&path)
                .expect("formal memory sidecar path")
                .exists(),
            "worker report must not create formal memory"
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn project_director_process_fact_confirmation_writes_recorded_observation_only() {
        let (dir, path, project, work_item_id, dispatch_id, node_id) =
            setup_c5_worker_report_fixture("c5-process-fact-confirm");
        let report = record_worker_structured_report_at(
            &path,
            &fixture_c5_worker_report_input(
                &project.project_root,
                &work_item_id,
                &dispatch_id,
                &node_id,
            ),
        )
        .expect("worker report should write");
        let input = fixture_c5_process_fact_decision_input(
            &project.project_root,
            &report.audit_event_id,
            &dispatch_id,
            "confirm_process_fact",
        );

        let output = record_project_director_process_fact_decision_at(&path, &input)
            .expect("project director should confirm low-risk process fact");
        let updated = read_json_file(&path);
        let observation_store = observation_store::load_store(&path, "2026-06-04T00:00:01Z")
            .expect("observation store should load");
        let derived = output.snapshot.project_workflows[0]
            .derived_workflow
            .as_ref()
            .expect("derived workflow should exist");

        assert_eq!(output.observations.len(), 1);
        assert_eq!(output.observations[0].observation_type, "process_fact");
        assert_eq!(output.observations[0].status, ObservationStatus::Recorded);
        assert_eq!(output.observations[0].generated_by_role, "project_director");
        assert!(output.message.contains("仍不是正式记忆"));
        assert_eq!(observation_store.observations.len(), 1);
        assert!(updated["reviews"]
            .as_array()
            .expect("reviews should be array")
            .iter()
            .any(|review| review["decision"] == "confirm_process_fact"
                && review["report_id"] == report.audit_event_id
                && review["warnings"]
                    .as_array()
                    .is_some_and(|warnings| warnings.iter().any(
                        |warning| warning == "process_fact_observation_is_not_formal_memory"
                    ))));
        assert!(derived.review_results.iter().any(|review| {
            review.report_id.as_deref() == Some(report.audit_event_id.as_str())
                && review.result == "process_fact_confirmed"
                && review.observation_ids.len() == 1
        }));
        assert!(
            !formal_memory_store::sidecar_path(&path)
                .expect("formal memory sidecar path")
                .exists(),
            "process fact observation must not create formal memory"
        );
        assert!(
            !memory_candidate_store::sidecar_path(&path)
                .expect("candidate sidecar path")
                .exists(),
            "process fact confirmation must not automatically create memory candidate"
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn project_director_process_fact_decision_rejects_wrong_actor_and_unsafe_facts() {
        let (dir, path, project, work_item_id, dispatch_id, node_id) =
            setup_c5_worker_report_fixture("c5-process-fact-boundaries");
        let report = record_worker_structured_report_at(
            &path,
            &fixture_c5_worker_report_input(
                &project.project_root,
                &work_item_id,
                &dispatch_id,
                &node_id,
            ),
        )
        .expect("worker report should write");

        let mut wrong_actor = fixture_c5_process_fact_decision_input(
            &project.project_root,
            &report.audit_event_id,
            &dispatch_id,
            "confirm_process_fact",
        );
        wrong_actor.actor_role = "secretary".to_string();
        let err = record_project_director_process_fact_decision_at(&path, &wrong_actor)
            .expect_err("secretary must not confirm process fact");
        assert!(err.contains("只有项目主管"), "{err}");

        let mut high_risk = fixture_c5_process_fact_decision_input(
            &project.project_root,
            &report.audit_event_id,
            &dispatch_id,
            "confirm_process_fact",
        );
        high_risk.accepted_facts[0].risk_level = "high".to_string();
        let err = record_project_director_process_fact_decision_at(&path, &high_risk)
            .expect_err("high risk process fact should require higher confirmation");
        assert!(err.contains("high / medium risk"), "{err}");

        let mut secret = fixture_c5_process_fact_decision_input(
            &project.project_root,
            &report.audit_event_id,
            &dispatch_id,
            "confirm_process_fact",
        );
        secret.accepted_facts[0].sensitive_level = "secret".to_string();
        let err = record_project_director_process_fact_decision_at(&path, &secret)
            .expect_err("secret process fact should require user confirmation");
        assert!(err.contains("secret / sensitive"), "{err}");

        let mut cross_project = fixture_c5_process_fact_decision_input(
            &project.project_root,
            &report.audit_event_id,
            &dispatch_id,
            "confirm_process_fact",
        );
        cross_project.accepted_facts[0].scope.project_id = Some("project:other".to_string());
        let err = record_project_director_process_fact_decision_at(&path, &cross_project)
            .expect_err("cross-project process fact should be rejected");
        assert!(err.contains("cross-project"), "{err}");

        assert!(
            !observation_store::sidecar_path(&path)
                .expect("observation sidecar path")
                .exists(),
            "rejected C5 decisions must not create observations"
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn process_fact_duplicate_is_rejected_and_rework_does_not_write_observation() {
        let (dir, path, project, work_item_id, dispatch_id, node_id) =
            setup_c5_worker_report_fixture("c5-process-fact-duplicate");
        let report = record_worker_structured_report_at(
            &path,
            &fixture_c5_worker_report_input(
                &project.project_root,
                &work_item_id,
                &dispatch_id,
                &node_id,
            ),
        )
        .expect("worker report should write");
        let confirm = fixture_c5_process_fact_decision_input(
            &project.project_root,
            &report.audit_event_id,
            &dispatch_id,
            "confirm_process_fact",
        );
        record_project_director_process_fact_decision_at(&path, &confirm)
            .expect("first process fact confirmation should write");

        let duplicate = record_project_director_process_fact_decision_at(&path, &confirm)
            .expect_err("duplicate process fact confirmation should be rejected");
        assert!(duplicate.contains("process_fact_duplicate"), "{duplicate}");

        let (
            rework_dir,
            rework_path,
            rework_project,
            rework_item_id,
            rework_dispatch_id,
            rework_node_id,
        ) = setup_c5_worker_report_fixture("c5-process-fact-rework");
        let rework_report = record_worker_structured_report_at(
            &rework_path,
            &fixture_c5_worker_report_input(
                &rework_project.project_root,
                &rework_item_id,
                &rework_dispatch_id,
                &rework_node_id,
            ),
        )
        .expect("worker report should write for rework");
        let rework = record_project_director_process_fact_decision_at(
            &rework_path,
            &fixture_c5_process_fact_decision_input(
                &rework_project.project_root,
                &rework_report.audit_event_id,
                &rework_dispatch_id,
                "request_rework",
            ),
        )
        .expect("rework decision should write review only");
        let rework_snapshot =
            read_workflow_state_snapshot(&rework_path).expect("rework snapshot should read");
        let rework_derived = rework_snapshot.project_workflows[0]
            .derived_workflow
            .as_ref()
            .expect("derived workflow should exist");

        assert!(rework.observations.is_empty());
        assert!(
            !observation_store::sidecar_path(&rework_path)
                .expect("observation sidecar path")
                .exists(),
            "rework decision must not create process fact observation"
        );
        assert!(rework_derived.review_results.iter().any(|review| {
            review.report_id.as_deref() == Some(rework_report.audit_event_id.as_str())
                && review.result == "rework_requested"
                && review.observation_ids.is_empty()
        }));

        let _ = fs::remove_dir_all(dir);
        let _ = fs::remove_dir_all(rework_dir);
    }

    #[test]
    fn global_final_result_review_rejects_missing_c2_and_c3_prerequisites() {
        let dir = test_temp_dir("c6-final-review-missing-c2");
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/c6-final-review-missing-c2");
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let missing_c2 = fixture_global_final_result_review_input(
            &project.project_root,
            "proposal:missing",
            "plan-auth:missing",
            "process-fact:missing",
            "accepted",
        );

        let err = record_global_final_result_review_at(&path, &missing_c2)
            .expect_err("missing C2 proposal should reject final review");
        assert!(err.contains("找不到 C2"), "{err}");

        let timestamp_ms = 1_765_600_000_000;
        let confirmed =
            create_confirmed_proposal_for_global_review(&path, &project.project_root, timestamp_ms);
        let missing_c3 = fixture_global_final_result_review_input(
            &project.project_root,
            &confirmed.0.proposal_id,
            &confirmed.1.authorization_id,
            "process-fact:missing",
            "accepted",
        );
        let err = record_global_final_result_review_at(&path, &missing_c3)
            .expect_err("missing active C3 authorization should reject final review");
        assert!(err.contains("C3") || err.contains("active"), "{err}");

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn global_final_result_review_records_review_without_memory_or_user_acceptance() {
        let (dir, path, project, proposal, authorization, fact_id) =
            setup_c6_complete_fixture("c6-final-review-records");
        let input = fixture_global_final_result_review_input(
            &project.project_root,
            &proposal.proposal_id,
            &authorization.authorization_id,
            &fact_id,
            "accepted",
        );

        let output = record_global_final_result_review_at(&path, &input)
            .expect("global director should record accepted final review");
        let updated = read_json_file(&path);
        let derived = output.snapshot.project_workflows[0]
            .derived_workflow
            .as_ref()
            .expect("derived workflow should exist");

        assert!(updated["reviews"]
            .as_array()
            .expect("reviews should be array")
            .iter()
            .any(|review| review["review_target"] == "global_final_result"
                && review["reviewer_role"] == "global_director"
                && review["decision"] == "accepted"));
        assert!(updated["audit_events"]
            .as_array()
            .expect("audit events should be array")
            .iter()
            .any(|event| event["event_type"] == "global_final_result_review_recorded"));
        assert_eq!(derived.result_summary.final_review_status, "accepted");
        assert_eq!(derived.result_summary.user_decision_status, "pending");
        assert!(
            !formal_memory_store::sidecar_path(&path)
                .expect("formal memory sidecar path")
                .exists(),
            "final review must not write formal memory"
        );
        assert!(
            !memory_candidate_store::sidecar_path(&path)
                .expect("candidate sidecar path")
                .exists(),
            "final review must not generate memory candidate"
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn global_final_result_review_rejects_wrong_actor() {
        let (dir, path, project, proposal, authorization, fact_id) =
            setup_c6_complete_fixture("c6-final-review-wrong-actor");
        let mut input = fixture_global_final_result_review_input(
            &project.project_root,
            &proposal.proposal_id,
            &authorization.authorization_id,
            &fact_id,
            "accepted",
        );
        input.actor_role = "project_director".to_string();

        let err = record_global_final_result_review_at(&path, &input)
            .expect_err("project director must not record global final review");
        assert!(err.contains("只有全局主管"), "{err}");

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn user_result_decision_requires_user_and_does_not_write_memory() {
        let (dir, path, project, proposal, authorization, fact_id) =
            setup_c6_complete_fixture("c6-user-result-decision");
        record_global_final_result_review_at(
            &path,
            &fixture_global_final_result_review_input(
                &project.project_root,
                &proposal.proposal_id,
                &authorization.authorization_id,
                &fact_id,
                "accepted",
            ),
        )
        .expect("global final review should write");
        let actual_review_id = read_json_file(&path)["reviews"]
            .as_array()
            .expect("reviews should be array")
            .iter()
            .rev()
            .find(|review| review["review_target"] == "global_final_result")
            .and_then(|review| optional_string_from(review, "review_id"))
            .expect("global final review id should exist");
        let mut wrong_actor = fixture_user_result_decision_input(
            &project.project_root,
            &actual_review_id,
            "accept_result",
        );
        wrong_actor.actor_role = "secretary".to_string();
        let err = record_user_result_decision_at(&path, &wrong_actor)
            .expect_err("secretary must not accept result for user");
        assert!(err.contains("只有用户"), "{err}");

        let output = record_user_result_decision_at(
            &path,
            &fixture_user_result_decision_input(
                &project.project_root,
                &actual_review_id,
                "accept_result",
            ),
        )
        .expect("user should accept accepted final review");
        let derived = output.snapshot.project_workflows[0]
            .derived_workflow
            .as_ref()
            .expect("derived workflow should exist");

        assert_eq!(derived.result_summary.user_decision_status, "accept_result");
        assert!(
            !formal_memory_store::sidecar_path(&path)
                .expect("formal memory sidecar path")
                .exists(),
            "user decision must not write formal memory"
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn stage_c_acceptance_summary_records_gates_and_deferred_items() {
        let (dir, path, project, proposal, authorization, fact_id) =
            setup_c6_complete_fixture("c6-stage-acceptance-summary");
        record_global_final_result_review_at(
            &path,
            &fixture_global_final_result_review_input(
                &project.project_root,
                &proposal.proposal_id,
                &authorization.authorization_id,
                &fact_id,
                "accepted",
            ),
        )
        .expect("global final review should write");
        let review_id = read_json_file(&path)["reviews"]
            .as_array()
            .expect("reviews should be array")
            .iter()
            .rev()
            .find(|review| review["review_target"] == "global_final_result")
            .and_then(|review| optional_string_from(review, "review_id"))
            .expect("global final review id should exist");
        record_user_result_decision_at(
            &path,
            &fixture_user_result_decision_input(&project.project_root, &review_id, "accept_result"),
        )
        .expect("user decision should write");

        let output = generate_stage_c_acceptance_summary_at(
            &path,
            &GenerateStageCAcceptanceSummaryInput {
                project_root: project.project_root.clone(),
                project_id: project_id(&project.project_root),
                workflow_id: default_workflow_id(&project.project_root),
                expected_workflow_revision: None,
            },
        )
        .expect("stage C summary should write artifact");
        let updated = read_json_file(&path);
        let derived = output.snapshot.project_workflows[0]
            .derived_workflow
            .as_ref()
            .expect("derived workflow should exist");

        assert!(updated["artifacts"]
            .as_array()
            .expect("artifacts should be array")
            .iter()
            .any(
                |artifact| artifact["artifact_type"] == "stage_c_acceptance_summary"
                    && artifact["stage_c_acceptance_summary"]["accepted_as_stage_c_complete"]
                        == true
            ));
        assert!(
            derived
                .result_summary
                .stage_c_acceptance
                .accepted_as_stage_c_complete
        );
        assert!(derived
            .result_summary
            .stage_c_acceptance
            .gates
            .iter()
            .any(|gate| gate.status == "deferred"));
        assert!(derived
            .result_summary
            .deferred_items
            .iter()
            .any(|item| item.contains("真实 worker")));

        let _ = fs::remove_dir_all(dir);
    }
