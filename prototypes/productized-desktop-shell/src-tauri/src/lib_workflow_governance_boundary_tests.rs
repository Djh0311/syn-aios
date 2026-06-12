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
    fn subagent_report_derives_required_fields_and_direction_risk() {
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
          "warnings": ["direction_risk:需求冲突"],
          "follow_up_suggestions": ["请项目主管裁决方向。"],
          "acceptance_status": "reported_not_completed"
        })];
        let permission_requests = vec![json!({
          "request_id": "permission:001",
          "workflow_id": workflow_id,
          "work_item_id": "work-item:001",
          "status": "pending",
          "reason": "需要写入 README。",
          "requested_at": "2026-06-01T00:00:00Z"
        })];
        let reports = derive_subagent_reports(&workflow_id, &dispatches, &[], &permission_requests);

        assert_eq!(reports.len(), 1);
        let report = &reports[0];
        assert_eq!(report.actor_role.as_deref(), Some("codex-dev"));
        assert!(report.executed_what.contains("修改 README"));
        assert!(report.changed_what.contains("改了 README"));
        assert_eq!(report.evidence_refs, vec!["/tmp/report.md".to_string()]);
        assert_eq!(
            report.permission_requests,
            vec!["permission:001".to_string()]
        );
        assert_eq!(
            report.direction_risks,
            vec!["direction_risk:需求冲突".to_string()]
        );
        assert_eq!(report.acceptance_status, "reported_not_completed");
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
    fn workflow_exception_detects_timeout_permission_review_direction_and_harness() {
        let workflow_id = default_workflow_id("/tmp/indexed-project");
        let artifacts = vec![json!({
          "artifact_id": "artifact:001",
          "artifact_type": "task_package",
          "workflow_id": workflow_id,
          "unresolved_direction_risk": true,
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
        assert!(types.contains(&"unresolved_direction_risk"));
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

        let blocked = director_completion_gate(
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
        assert!(!blocked.can_complete);
        assert!(blocked.missing.contains(&"no_unresolved_risk".to_string()));
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
