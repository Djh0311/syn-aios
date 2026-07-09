    #[test]
    fn workflow_ledger_derives_summary_entries_without_tool_output_fulltext() {
        let workflow_id = default_workflow_id("/tmp/indexed-project");
        let long_tool_output = "工具输出全文".repeat(80);
        let audit_events = vec![json!({
          "event_id": "audit:tool-summary:001",
          "event_type": "tool_call_summary",
          "target_ref": workflow_id,
          "actor_ref": "codex-dev",
          "reason": "工具调用摘要：读取了允许范围内的 README。",
          "tool_output_fulltext": long_tool_output,
          "risk_flags": ["allowed_scope_only"],
          "created_at": "2026-06-01T00:00:00Z"
        })];
        let dispatches = vec![json!({
          "dispatch_id": "dispatch:tool:001",
          "workflow_id": workflow_id,
          "node_id": format!("{}:node:codex-dev", default_workflow_id("/tmp/indexed-project")),
          "work_item_id": "work-item:001",
          "native_thread_id": "thread-001",
          "prompt_kind": "tool_call_summary",
          "prompt_preview": format!("摘要，不是全文。{long_tool_output}"),
          "tool_call_ref": "tool-call:001",
          "warnings": ["tool_output_trimmed"]
        })];
        let entries =
            derive_workflow_ledger_entries(&workflow_id, &audit_events, &dispatches, &[], &[]);

        assert!(entries
            .iter()
            .any(|entry| entry.entry_type == "tool_call_summary"));
        assert!(entries
            .iter()
            .any(|entry| entry.tool_call_refs == vec!["tool-call:001".to_string()]));
        assert!(entries
            .iter()
            .all(|entry| !entry.summary.contains(&long_tool_output)));
        assert!(entries
            .iter()
            .any(|entry| entry.audit_refs == vec!["audit:tool-summary:001".to_string()]));
    }

    #[test]
    fn workflow_ledger_maps_chain_audit_entry_types_exactly() {
        let workflow_id = default_workflow_id("/tmp/indexed-project");
        let events = vec![
            ("started", "workflow_chain_node_started", "subagent_started"),
            ("completed", "workflow_chain_node_completed", "node_passed"),
            (
                "deterministic",
                "workflow_chain_node_director_deterministic_completed",
                "node_passed",
            ),
            (
                "lm",
                "workflow_chain_node_director_lm_completed",
                "node_passed",
            ),
            ("failed", "workflow_chain_node_failed", "node_failed"),
            (
                "needs-rework",
                "workflow_chain_node_needs_rework",
                "node_returned",
            ),
            (
                "failed-action-rework",
                "workflow_chain_node_failed_action_rework",
                "node_returned",
            ),
            (
                "failed-action-archive",
                "workflow_chain_node_failed_action_archive",
                "user_decision",
            ),
            (
                "failed-action-retry",
                "workflow_chain_node_failed_action_retry",
                "user_decision",
            ),
            (
                "failed-action-change-session",
                "workflow_chain_node_failed_action_change_session",
                "user_decision",
            ),
            (
                "waiting-decision",
                "workflow_chain_node_waiting_decision",
                "waiting_decision",
            ),
            ("skipped", "workflow_chain_node_skipped", "node_skipped"),
            ("cancelled", "workflow_chain_node_cancelled", "node_cancelled"),
            ("summary", "workflow_chain_director_summary", "director_summary"),
            ("run-started", "workflow_chain_run_started", "subagent_started"),
            ("run-completed", "workflow_chain_run_completed", "node_passed"),
            ("run-failed", "workflow_chain_run_failed", "node_failed"),
            (
                "run-waiting",
                "workflow_chain_run_waiting_decision",
                "waiting_decision",
            ),
            ("run-stopped", "workflow_chain_run_stopped", "node_returned"),
            ("run-superseded", "workflow_chain_run_superseded", "node_returned"),
            (
                "run-stop-requested",
                "workflow_chain_run_stop_requested",
                "user_decision",
            ),
        ];
        let audit_events = events
            .iter()
            .map(|(suffix, event_type, _)| {
                json!({
                  "event_id": format!("audit:{suffix}"),
                  "event_type": event_type,
                  "target_ref": workflow_id,
                  "actor_ref": "project_director",
                  "reason": format!("ledger mapping fixture {suffix}"),
                  "created_at": "2026-07-09T00:00:00Z"
                })
            })
            .collect::<Vec<_>>();
        let entries = derive_workflow_ledger_entries(&workflow_id, &audit_events, &[], &[], &[]);

        for (suffix, event_type, expected_entry_type) in events {
            let entry = entries
                .iter()
                .find(|entry| entry.ledger_entry_id == format!("audit:{suffix}"))
                .unwrap_or_else(|| panic!("{event_type} should enter workflow ledger"));
            assert_eq!(
                entry.entry_type, expected_entry_type,
                "{event_type} should map by exact event_type"
            );
            assert!(
                !entry
                    .risk_flags
                    .iter()
                    .any(|warning| warning.starts_with("invalid_ledger_entry_type:")),
                "{event_type} should be in the ledger entry_type vocabulary: {:?}",
                entry.risk_flags
            );
        }
    }

    #[test]
    fn workflow_ledger_validates_entry_type_without_panicking() {
        assert!(is_valid_ledger_entry_type("waiting_decision"));
        assert!(is_valid_ledger_entry_type("node_skipped"));
        assert!(is_valid_ledger_entry_type("node_cancelled"));
        assert!(!is_valid_ledger_entry_type("workflow_chain_custom_future_event"));

        let workflow_id = default_workflow_id("/tmp/indexed-project");
        let audit_events = vec![json!({
          "event_id": "audit:custom-future",
          "event_type": "workflow_chain_custom_future_event",
          "target_ref": workflow_id,
          "actor_ref": "project_director",
          "reason": "未知未来链事件应软着陆。",
          "created_at": "2026-07-09T00:00:00Z"
        })];
        let entries = derive_workflow_ledger_entries(&workflow_id, &audit_events, &[], &[], &[]);
        let entry = entries
            .iter()
            .find(|entry| entry.ledger_entry_id == "audit:custom-future")
            .expect("unknown workflow audit still enters ledger");

        assert_eq!(entry.entry_type, "workflow_chain_custom_future_event");
        assert!(entry.risk_flags.contains(
            &"invalid_ledger_entry_type:workflow_chain_custom_future_event".to_string()
        ));
    }

    #[test]
    fn subagent_report_projects_help_fields_from_worker_report_truth_source() {
        let workflow_id = default_workflow_id("/tmp/indexed-project");
        let dispatches = vec![json!({
          "dispatch_id": "dispatch:report:001",
          "workflow_id": workflow_id,
          "node_id": format!("{workflow_id}:node:codex-dev"),
          "work_item_id": "work-item:001",
          "native_thread_id": "thread-001",
          "prompt_preview": "执行：修改 README。",
          "state": "completed",
          "last_message_summary": "改了 README，发现 direction risk。",
          "last_message_path": "/tmp/report.md",
          "warnings": ["direction_risk:旧 warning 不是真源"],
          "acceptance_status": "reported_not_completed"
        })];
        let audit_events = vec![json!({
          "event_id": "audit:worker-report:001",
          "event_type": "worker_structured_report_recorded",
          "workflow_id": workflow_id,
          "node_id": format!("{workflow_id}:node:codex-dev"),
          "work_item_id": "work-item:001",
          "dispatch_id": "dispatch:report:001",
          "actor_ref": "codex-dev",
          "reason": "worker blocked",
          "open_issues": ["缺少验收口径"],
          "permission_requests": ["请授权读取 /secure"],
          "direction_risks": ["方向A 可能错"],
          "follow_up_suggestions": ["请主管裁决方向"],
          "acceptance_status": "blocked"
        })];
        let reports = derive_subagent_reports(&workflow_id, &dispatches, &audit_events, &[]);

        let report = reports
            .iter()
            .find(|report| report.report_id == "report:dispatch:report:001")
            .expect("dispatch 派生报告应存在");
        assert_eq!(report.actor_role.as_deref(), Some("codex-dev"));
        assert!(report.executed_what.contains("修改 README"));
        assert!(report.changed_what.contains("改了 README"));
        assert_eq!(report.evidence_refs, vec!["/tmp/report.md".to_string()]);
        assert_eq!(report.open_issues, vec!["缺少验收口径".to_string()]);
        assert_eq!(
            report.permission_requests,
            vec!["请授权读取 /secure".to_string()]
        );
        assert_eq!(report.direction_risks, vec!["方向A 可能错".to_string()]);
        assert_eq!(report.follow_up_suggestions, vec!["请主管裁决方向".to_string()]);
        assert_eq!(report.acceptance_status, "blocked");
        assert_eq!(
            report.warnings,
            vec!["direction_risk:旧 warning 不是真源".to_string()]
        );
    }

    #[test]
    fn subagent_report_does_not_infer_help_fields_from_dispatch_warnings() {
        let workflow_id = default_workflow_id("/tmp/indexed-project");
        let dispatches = vec![json!({
          "dispatch_id": "dispatch:report:002",
          "workflow_id": workflow_id,
          "node_id": format!("{workflow_id}:node:codex-dev"),
          "work_item_id": "work-item:002",
          "native_thread_id": "thread-002",
          "prompt_preview": "执行：修改 README。",
          "state": "completed",
          "last_message_summary": "改了 README。",
          "last_message_path": "/tmp/report.md",
          "warnings": ["direction risk 只是运行警告"],
          "follow_up_suggestions": ["旧 dispatch 建议不是真源"],
          "acceptance_status": "reported_completed"
        })];
        let permission_requests = vec![json!({
          "request_id": "permission:002",
          "workflow_id": workflow_id,
          "work_item_id": "work-item:002",
          "status": "pending",
          "reason": "结构性权限请求，不是 worker 自述。",
          "requested_at": "2026-06-01T00:00:00Z"
        })];
        let reports = derive_subagent_reports(&workflow_id, &dispatches, &[], &permission_requests);
        let report = reports
            .iter()
            .find(|report| report.report_id == "report:dispatch:report:002")
            .expect("dispatch 派生报告应存在");

        assert!(report.open_issues.is_empty());
        assert!(report.permission_requests.is_empty());
        assert!(report.direction_risks.is_empty());
        assert!(report.follow_up_suggestions.is_empty());
        assert_eq!(
            report.warnings,
            vec!["direction risk 只是运行警告".to_string()]
        );
    }

    #[test]
    fn subagent_report_does_not_cross_project_help_fields_by_same_node() {
        let workflow_id = default_workflow_id("/tmp/indexed-project");
        let dispatches = vec![json!({
          "dispatch_id": "dispatch:report:003",
          "workflow_id": workflow_id,
          "node_id": format!("{workflow_id}:node:codex-dev"),
          "work_item_id": "work-item:003",
          "native_thread_id": "thread-003",
          "prompt_preview": "执行：修改 README。",
          "state": "completed",
          "last_message_summary": "改了 README。",
          "last_message_path": "/tmp/report.md",
          "warnings": [],
          "acceptance_status": "reported_completed"
        })];
        let audit_events = vec![json!({
          "event_id": "audit:worker-report:other",
          "event_type": "worker_structured_report_recorded",
          "workflow_id": workflow_id,
          "node_id": format!("{workflow_id}:node:codex-dev"),
          "work_item_id": "work-item:other",
          "dispatch_id": "dispatch:other",
          "actor_ref": "codex-dev",
          "reason": "other worker report",
          "open_issues": ["别的任务卡点"],
          "permission_requests": ["别的任务权限"],
          "direction_risks": ["别的任务方向风险"],
          "follow_up_suggestions": ["别的任务建议"],
          "acceptance_status": "blocked"
        })];
        let reports = derive_subagent_reports(&workflow_id, &dispatches, &audit_events, &[]);
        let report = reports
            .iter()
            .find(|report| report.report_id == "report:dispatch:report:003")
            .expect("dispatch 派生报告应存在");

        assert!(report.open_issues.is_empty());
        assert!(report.permission_requests.is_empty());
        assert!(report.direction_risks.is_empty());
        assert!(report.follow_up_suggestions.is_empty());
        assert_eq!(report.acceptance_status, "reported_completed");
    }

    #[test]
    fn review_result_cannot_directly_complete_node() {
        let workflow_id = default_workflow_id("/tmp/indexed-project");
        let reviews = vec![json!({
          "review_id": "review:001",
          "workflow_id": workflow_id,
          "workflow_node_id": format!("{workflow_id}:node:review"),
          "decision": "accepted",
          "summary": "审查通过。",
          "evidence_refs": ["/tmp/evidence.md"],
          "warnings": []
        })];
        let results = derive_review_results(&workflow_id, &reviews);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].result, "passed");
        assert!(results[0].requires_director_confirmation);
        assert!(!results[0].can_complete_node);
        assert!(results[0]
            .warnings
            .contains(&"review_passed_but_director_still_confirms_node_completion".to_string()));
    }

    #[test]
    fn workflow_exception_ignores_dead_unresolved_direction_artifact_field() {
        let workflow_id = default_workflow_id("/tmp/indexed-project");
        let artifacts = vec![json!({
          "artifact_id": "artifact:001",
          "artifact_type": "task_package",
          "workflow_id": workflow_id,
          "unresolved_direction_risk": true,
          "risk_flags": ["unresolved_direction_risk"],
          "harness_blocked": true,
          "warnings": ["fixture"]
        })];
        let permission_requests = vec![json!({
          "request_id": "permission:001",
          "workflow_id": workflow_id,
          "work_item_id": "work-item:001",
          "status": "pending",
          "reason": "等待权限。",
          "requested_at": "2026-06-01T00:00:00Z"
        })];
        let attempts = vec![json!({
          "attempt_id": "attempt:001",
          "workflow_id": workflow_id,
          "state": "timed_out",
          "failure_reason": "超时。"
        })];
        let review_results = vec![
            ReviewResult {
                review_id: "review:001".to_string(),
                workflow_id: workflow_id.clone(),
                workflow_node_id: None,
                reviewer_role: Some("director".to_string()),
                report_id: None,
                accepted_fact_ids: vec![],
                observation_ids: vec![],
                result: "returned".to_string(),
                summary: "退回一次".to_string(),
                evidence_refs: vec![],
                requires_director_confirmation: true,
                can_complete_node: false,
                warnings: vec![],
            },
            ReviewResult {
                review_id: "review:002".to_string(),
                workflow_id: workflow_id.clone(),
                workflow_node_id: None,
                reviewer_role: Some("director".to_string()),
                report_id: None,
                accepted_fact_ids: vec![],
                observation_ids: vec![],
                result: "returned".to_string(),
                summary: "退回两次".to_string(),
                evidence_refs: vec![],
                requires_director_confirmation: true,
                can_complete_node: false,
                warnings: vec![],
            },
        ];
        let exceptions = derive_workflow_exceptions(
            &workflow_id,
            &artifacts,
            &permission_requests,
            &attempts,
            &review_results,
        );
        let types = exceptions
            .iter()
            .map(|exception| exception.exception_type.as_str())
            .collect::<Vec<_>>();

        assert!(types.contains(&"subagent_timeout"));
        assert!(types.contains(&"long_permission_wait"));
        assert!(types.contains(&"repeated_review_return"));
        assert!(!types.contains(&"unresolved_direction_risk"));
        assert!(types.contains(&"harness_blocked"));
    }

    #[test]
    fn workflow_state_transition_enforces_confirmed_table() {
        assert!(workflow_transition_allowed("draft", "ready", false));
        assert!(workflow_transition_allowed(
            "running",
            "waiting_decision",
            false
        ));
        assert!(!workflow_transition_allowed("draft", "running", false));
        assert!(!workflow_transition_allowed(
            "waiting_decision",
            "completed",
            false
        ));
        assert!(!workflow_transition_allowed("failed", "running", false));
        assert!(workflow_transition_allowed("failed", "running", true));
    }

    #[test]
    fn workflow_node_state_transition_enforces_actor_boundaries() {
        assert!(workflow_node_transition_allowed(
            "waiting",
            "running",
            "project_director",
            false
        ));
        assert!(workflow_node_transition_allowed(
            "reviewing",
            "passed",
            "review",
            false
        ));
        assert!(!workflow_node_transition_allowed(
            "reviewing",
            "passed",
            "subagent",
            false
        ));
        assert!(!workflow_node_transition_allowed(
            "waiting_decision",
            "running",
            "subagent",
            false
        ));
        assert!(workflow_node_transition_allowed(
            "waiting_decision",
            "running",
            "project_director",
            false
        ));
        assert!(!workflow_node_transition_allowed(
            "waiting_decision",
            "cancelled",
            "subagent",
            false
        ));
        assert!(workflow_node_transition_allowed(
            "waiting_decision",
            "cancelled",
            "project_director",
            false
        ));
        assert!(!workflow_node_transition_allowed(
            "failed",
            "running",
            "project_director",
            false
        ));
        assert!(workflow_node_transition_allowed(
            "failed",
            "running",
            "project_director",
            true
        ));
    }

    #[test]
    fn director_completion_gate_requires_evidence_review_and_no_risk() {
        let package = TaskPackage {
            task_package_id: "task-package:001".to_string(),
            workflow_id: "workflow:001".to_string(),
            workflow_node_id: "node:001".to_string(),
            project_id: "project:001".to_string(),
            target_session_id: Some("thread-001".to_string()),
            target_role: Some("codex-dev".to_string()),
            task_goal: Some("完成目标".to_string()),
            allowed_read_scope: vec!["/tmp/project".to_string()],
            allowed_write_scope: vec!["/tmp/project/README.md".to_string()],
            available_skills: vec![],
            available_knowledge_refs: vec![],
            available_memory_refs: vec!["memory:confirmed:001".to_string()],
            callable_tool_capabilities: vec![],
            model_id: Some("codex-test-model".to_string()),
            harness_requirements: vec![],
            forbidden_actions: vec!["不写 .codex".to_string()],
            acceptance_criteria: vec!["验收通过".to_string()],
            report_format: vec!["做了什么".to_string()],
            timeout_policy: Some("600s".to_string()),
            failure_policy: Some("return_to_director".to_string()),
            version: 1,
            stale: false,
            stale_reasons: vec![],
            missing_fields: vec![],
            export_includes_internal_audit: false,
            memory_injection_summary: task_memory_injection::missing_summary(),
            warnings: vec![],
        };
        let reviews = vec![ReviewResult {
            review_id: "review:001".to_string(),
            workflow_id: "workflow:001".to_string(),
            workflow_node_id: Some("node:review".to_string()),
            reviewer_role: Some("director".to_string()),
            report_id: None,
            accepted_fact_ids: vec![],
            observation_ids: vec![],
            result: "passed".to_string(),
            summary: "审查通过".to_string(),
            evidence_refs: vec!["evidence:001".to_string()],
            requires_director_confirmation: true,
            can_complete_node: false,
            warnings: vec![],
        }];
        let gate = director_completion_gate(Some(&package), &reviews, &[]);
        assert!(gate.can_complete);

        let ignored_dead_direction_exception = director_completion_gate(
            Some(&package),
            &reviews,
            &[WorkflowException {
                exception_id: "exception:direction".to_string(),
                workflow_id: "workflow:001".to_string(),
                workflow_node_id: None,
                exception_type: "unresolved_direction_risk".to_string(),
                summary: "方向风险".to_string(),
                status: "waiting_decision".to_string(),
                warnings: vec![],
            }],
        );
        assert!(ignored_dead_direction_exception.can_complete);
        assert!(!ignored_dead_direction_exception
            .required
            .contains(&"no_unresolved_risk".to_string()));
        assert!(!ignored_dead_direction_exception
            .missing
            .contains(&"no_unresolved_risk".to_string()));
    }

    #[test]
    fn project_director_task_package_uses_v2_canonical_names_and_defaults() {
        let workflow_id = "workflow:c2:naming";
        let work_item_id = "work-item:c2:naming:1";
        let artifact_id = "artifact:c2:naming:1";
        let project_root = "/tmp/c2-naming-project";
        let project = fixture_project(project_root);
        let mut state = json!({
          "artifacts": [],
          "work_items": [{
            "work_item_id": work_item_id,
            "workflow_id": workflow_id,
            "title": "C2 命名",
            "state": "ready_to_dispatch",
            "assigned_role_id": "codex-dev",
            "current_node_id": format!("{workflow_id}:node:codex-dev")
          }]
        });
        let task = ProjectDirectorPlannedTask {
            planned_task_id: "planned-task:c2:naming:1".to_string(),
            title: "C2 命名".to_string(),
            task_goal: "把任务包字段统一到正本命名。".to_string(),
            scope: ProjectDirectorTaskScope {
                project_id: project_id(project_root),
                workflow_id: workflow_id.to_string(),
                target_role: "codex-dev".to_string(),
                task_package_kind: "task_package".to_string(),
                allowed_read_scope: vec![project_root.to_string()],
                allowed_write_scope: vec![format!("{project_root}/src")],
                callable_tool_capabilities: vec!["read_file".to_string()],
                required_checks: vec!["cargo test".to_string()],
                stop_conditions: vec!["发现沙箱失配就停".to_string()],
                timeout_policy: Some("600s".to_string()),
                failure_policy: Some("return_to_director".to_string()),
                available_skills: vec!["rust".to_string()],
                available_knowledge_refs: vec!["docs/workflow-task-package-design-v1.md#3.4".to_string()],
                forbidden_actions: vec!["不改 execute 本体".to_string()],
                model_id: Some("codex-c2".to_string()),
            },
            depends_on: vec![],
            acceptance_criteria: vec!["旧三键不再物化".to_string()],
            report_format: vec!["做了什么".to_string()],
            status: "planned".to_string(),
            guard_result: None,
            work_item_id: Some(work_item_id.to_string()),
            workflow_node_id: Some(format!("{workflow_id}:node:codex-dev")),
            task_package_id: Some(artifact_id.to_string()),
            memory_packet_snapshot_id: None,
            prepared_dispatch_id: None,
            blocked_reasons: vec![],
        };
        let memory_snapshot = test_empty_memory_snapshot(workflow_id, work_item_id, artifact_id);

        ensure_project_director_task_package_artifact(
            &mut state,
            &project,
            work_item_id,
            artifact_id,
            &task,
            &memory_snapshot,
            "2026-07-09T00:00:00Z",
        )
        .expect("project director task package should materialize");

        let artifact = state["artifacts"][0].as_object().expect("artifact object");
        assert_eq!(
            artifact.get("task_goal").and_then(Value::as_str),
            Some("把任务包字段统一到正本命名。")
        );
        assert_eq!(artifact.get("report_format").and_then(Value::as_array).unwrap().len(), 1);
        assert_eq!(
            artifact
                .get("allowed_read_scope")
                .and_then(Value::as_array)
                .unwrap()
                .len(),
            1
        );
        assert!(artifact.get("brief").is_none());
        assert!(artifact.get("required_return").is_none());
        assert!(artifact.get("allowed_read").is_none());
        assert_eq!(
            artifact.get("forbidden_actions").and_then(Value::as_array).unwrap()[0],
            "不改 execute 本体"
        );
        assert_eq!(artifact.get("model_id").and_then(Value::as_str), Some("codex-c2"));

        let packages = derive_task_packages(
            workflow_id,
            &project_id(project_root),
            project_root,
            state["work_items"].as_array().unwrap(),
            state["artifacts"].as_array().unwrap(),
            &[],
        );
        assert_eq!(packages.len(), 1);
        assert_eq!(
            packages[0].task_goal.as_deref(),
            Some("把任务包字段统一到正本命名。")
        );
        assert_eq!(packages[0].report_format, vec!["做了什么".to_string()]);
        assert_eq!(packages[0].timeout_policy.as_deref(), Some("600s"));
        assert_eq!(
            packages[0].failure_policy.as_deref(),
            Some("return_to_director")
        );
        assert_eq!(packages[0].available_skills, vec!["rust".to_string()]);
        assert_eq!(
            packages[0].available_knowledge_refs,
            vec!["docs/workflow-task-package-design-v1.md#3.4".to_string()]
        );

        let old_scope: ProjectDirectorTaskScope = serde_json::from_value(json!({
          "project_id": "project:c2:old",
          "workflow_id": "workflow:c2:old",
          "target_role": "codex-dev",
          "task_package_kind": "task_package",
          "allowed_read_scope": [project_root],
          "allowed_write_scope": [format!("{project_root}/src")],
          "callable_tool_capabilities": [],
          "required_checks": [],
          "stop_conditions": []
        }))
        .expect("old planned task scope json should default new fields");
        assert!(old_scope.timeout_policy.is_none());
        assert!(old_scope.failure_policy.is_none());
        assert!(old_scope.available_skills.is_empty());
        assert!(old_scope.available_knowledge_refs.is_empty());
        assert!(old_scope.forbidden_actions.is_empty());
        assert!(old_scope.model_id.is_none());

        let default_task = ProjectDirectorPlannedTask {
            planned_task_id: "planned-task:c2:naming:default".to_string(),
            title: "C2 默认兜底".to_string(),
            task_goal: "验证默认 forbidden/model。".to_string(),
            scope: old_scope,
            depends_on: vec![],
            acceptance_criteria: vec!["默认保护仍在".to_string()],
            report_format: vec!["证据".to_string()],
            status: "planned".to_string(),
            guard_result: None,
            work_item_id: Some("work-item:c2:naming:default".to_string()),
            workflow_node_id: Some(format!("{workflow_id}:node:codex-dev")),
            task_package_id: Some("artifact:c2:naming:default".to_string()),
            memory_packet_snapshot_id: None,
            prepared_dispatch_id: None,
            blocked_reasons: vec![],
        };
        ensure_project_director_task_package_artifact(
            &mut state,
            &project,
            "work-item:c2:naming:default",
            "artifact:c2:naming:default",
            &default_task,
            &test_empty_memory_snapshot(
                workflow_id,
                "work-item:c2:naming:default",
                "artifact:c2:naming:default",
            ),
            "2026-07-09T00:00:00Z",
        )
        .expect("default task package should materialize");
        let default_artifact = state["artifacts"][1].as_object().expect("artifact object");
        assert_eq!(
            default_artifact
                .get("forbidden_actions")
                .and_then(Value::as_array)
                .unwrap()
                .len(),
            4
        );
        assert_eq!(
            default_artifact.get("model_id").and_then(Value::as_str),
            Some("codex-local-prepared")
        );
    }

    fn test_empty_memory_snapshot(
        workflow_id: &str,
        work_item_id: &str,
        artifact_id: &str,
    ) -> TaskPackageMemoryPacketSnapshot {
        TaskPackageMemoryPacketSnapshot {
            snapshot_id: format!("snapshot:{work_item_id}"),
            schema_version: "task_package_memory_packet_snapshot.v1".to_string(),
            source_packet_id: "packet:c2:naming".to_string(),
            project_id: Some("project:c2:naming".to_string()),
            workflow_id: Some(workflow_id.to_string()),
            work_item_id: work_item_id.to_string(),
            task_package_artifact_id: Some(artifact_id.to_string()),
            role_id: "codex-dev".to_string(),
            retrieval_intent: "worker_task".to_string(),
            included_memories: vec![],
            excluded_items: vec![],
            review_materials: vec![],
            store_revisions: TaskPackageMemoryPacketStoreRevisions {
                formal_store_revision: 0,
                candidate_store_revision: 0,
                observation_store_revision: 0,
                lint_store_revision: Some(0),
                entity_relation_store_revision: Some(0),
            },
            estimated_tokens: 0,
            max_estimated_tokens: 2000,
            fingerprint: "fingerprint:c2:naming".to_string(),
            generated_at: "2026-07-09T00:00:00Z".to_string(),
            stale: false,
            stale_reasons: vec![],
            warnings: vec![],
        }
    }

    #[test]
    fn workflow_interfaces_keep_conservative_boundaries() {
        let boundaries = workflow_interface_boundaries();
        assert!(boundaries
            .memory_candidate_interface
            .blocked
            .contains(&"auto_write_formal_memory".to_string()));
        assert!(boundaries
            .knowledge_refs_interface
            .blocked
            .contains(&"auto_scan_knowledge_base".to_string()));
        assert!(boundaries
            .model_pool_selector
            .blocked
            .contains(&"silent_auto_model_selection".to_string()));
        assert!(boundaries
            .harness_requirement_provider
            .blocked
            .contains(&"ordinary_workflow_node".to_string()));
        assert!(boundaries
            .tool_capability_registry
            .blocked
            .contains(&"tool_output_fulltext_in_ledger".to_string()));
    }
