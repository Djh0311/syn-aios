    #[test]
    fn path_whitelist_accepts_only_index_projects_and_rollouts() {
        let index = json!({
          "projects": [
            { "project_root": "/Users/yoyi/workspace" },
            { "project_root": null }
          ],
          "threads": [
            { "rollout_path": "/Users/yoyi/.codex/sessions/sample.jsonl" },
            { "rollout_path": 12 }
          ]
        });

        let allowed = allowed_paths(&index);

        assert!(allowed.projects.contains("/Users/yoyi/workspace"));
        assert!(allowed
            .rollouts
            .contains("/Users/yoyi/.codex/sessions/sample.jsonl"));
        assert!(allowed.can_copy("/Users/yoyi/workspace"));
        assert!(!allowed.can_copy("/Users/yoyi/.codex/auth.json"));
    }

    #[test]
    fn snapshot_keeps_metadata_without_session_body() {
        let dir = test_temp_dir("snapshot-metadata");
        fs::create_dir_all(&dir).expect("create temp dir");
        let state = AppState {
            index_path: dir.join("codex-index.json"),
            tasks_path: dir.join("tasks.md"),
            workflow_state_path: dir.join("workflow-state.v0.json"),
        };
        let index = json!({
          "generated_at": "2026-05-27T10:23:52Z",
          "projects": [
            {
              "project_root": "/Users/yoyi/workspace",
              "thread_count": 2,
              "authority_files": [{ "kind": "readme", "path": "/Users/yoyi/workspace/README.md" }]
            }
          ],
          "threads": [
            {
              "thread_id": "abc",
              "title": "truncated title",
              "rollout_path": "/Users/yoyi/.codex/sessions/sample.jsonl",
              "rollout_exists": true
            }
          ],
          "skills": [{ "skill_id": "one", "title": "One", "path": "/skills/one", "source_type": "user" }],
          "plugins": [{ "plugin_name": "browser", "plugin_version": "1", "skill_paths": ["/a"] }],
          "warnings": []
        });

        let snapshot = build_snapshot_with_session_source(
            &state,
            &index,
            "## 待派发\n\n- `task.md`：说明正文\n",
            SessionSourceMode::IndexOnly,
        );

        assert_eq!(snapshot.summary.project_count, 1);
        assert_eq!(snapshot.summary.session_count, 1);
        assert_eq!(snapshot.projects[0].authority_files.len(), 1);
        assert_eq!(snapshot.tasks[0].title, "task.md");
        assert_eq!(snapshot.diagnostics.allowed_rollout_path_count, 1);
        assert_eq!(snapshot.diagnostic_summary.status, "degraded_readonly");
        assert!(snapshot
            .diagnostic_summary
            .boundary_notes
            .iter()
            .any(|note| note.contains("只读诊断")));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn g2_diagnostic_summary_reports_degraded_store_without_repair() {
        let dir = test_temp_dir("g2-diagnostic-summary");
        fs::create_dir_all(&dir).expect("create temp dir");
        let workflow_state_path = dir.join("workflow-state.v0.json");
        let workflow_state = json!({
          "schema_version": "workflow_state_v0",
          "workflow_version": 1,
          "workspace_id": "workspace:g2",
          "updated_at": "2026-06-07T00:00:00Z",
          "projects": [{ "project_id": "project:g2", "root_path": "/tmp/g2-project" }],
          "agent_adapters": [{ "adapter_id": "codex-local", "agent_type": "codex" }],
          "workflows": [{ "workflow_id": "workflow:g2", "project_id": "project:g2", "title": "G2", "state": "running" }],
          "nodes": [{ "node_id": "node:g2", "workflow_id": "workflow:g2", "node_type": "dev_line", "title": "G2 node", "state": "running" }],
          "edges": [],
          "work_items": [],
          "artifacts": [],
          "reviews": [],
          "audit_events": [],
          "capabilities": [],
          "harness_resources": [],
          "workflow_node_session_bindings": [],
          "workflow_node_dispatches": [],
          "workflow_execution_controls": [],
          "permission_requests": [],
          "execution_attempts": []
        });
        fs::write(
            &workflow_state_path,
            serde_json::to_string_pretty(&workflow_state).expect("serialize workflow state"),
        )
        .expect("write workflow state");
        fs::write(dir.join("formal-memories.v1.json"), "{broken json")
            .expect("write broken formal memory sidecar");
        let state = AppState {
            index_path: dir.join("codex-index.json"),
            tasks_path: dir.join("tasks.md"),
            workflow_state_path,
        };
        fs::write(&state.tasks_path, "- `g2.md`：G2\n").expect("write tasks");
        let index = json!({
          "generated_at": "2026-06-07T00:00:00Z",
          "projects": [{ "project_root": "/tmp/g2-project" }],
          "threads": []
        });

        let snapshot =
            build_snapshot_with_session_source(&state, &index, "", SessionSourceMode::IndexOnly);

        assert_eq!(snapshot.diagnostic_summary.status, "degraded_readonly");
        assert!(snapshot.diagnostic_summary.blocked_count > 0);
        assert!(snapshot
            .diagnostic_summary
            .store_integrity
            .iter()
            .any(|finding| finding.store_id == "formal_memory"
                && finding.status == "degraded"
                && finding.error.as_deref().unwrap_or("").contains("JSON")));
        assert!(snapshot
            .diagnostic_summary
            .degraded_states
            .iter()
            .any(|state| state.kind == "adapter_unavailable" && state.blocks_real_execution));
        assert!(snapshot
            .diagnostic_summary
            .boundary_notes
            .iter()
            .any(|note| note.contains("readback_unavailable")));
        assert_eq!(
            fs::read_to_string(dir.join("formal-memories.v1.json")).unwrap(),
            "{broken json"
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn workbench_snapshot_includes_backend_agent_adapter_descriptor() {
        let dir = test_temp_dir("adapter-backend-read-model");
        fs::create_dir_all(&dir).expect("create temp dir");
        let workflow_state_path = dir.join("workflow-state.v0.json");
        let workflow_state = json!({
          "schema_version": "workflow_state_v0",
          "workflow_version": 1,
          "workspace_id": "workspace:test",
          "updated_at": "2026-06-03T00:00:00Z",
          "projects": [{ "project_id": "project:test", "root_path": "/tmp/adapter-project" }],
          "agent_adapters": [{ "adapter_id": "codex-local", "agent_type": "codex" }],
          "workflows": [{ "workflow_id": "workflow:test", "project_id": "project:test", "title": "Adapter test", "state": "running" }],
          "nodes": [{ "node_id": "node:codex-dev", "workflow_id": "workflow:test", "node_type": "dev_line", "title": "开发线", "state": "running" }],
          "edges": [],
          "work_items": [{ "work_item_id": "work:test", "workflow_id": "workflow:test", "project_id": "project:test", "title": "Adapter task", "state": "ready_to_dispatch", "assigned_role_id": "codex-dev" }],
          "artifacts": [],
          "reviews": [],
          "audit_events": [],
          "capabilities": [],
          "harness_resources": [],
          "workflow_node_session_bindings": [{
            "binding_id": "binding:codex-dev",
            "project_id": "project:test",
            "workflow_id": "workflow:test",
            "node_id": "node:codex-dev",
            "work_item_id": "work:test",
            "agent_type": "codex",
            "adapter_id": "codex-local",
            "native_thread_id": "thread:adapter",
            "native_rollout_path": "/tmp/adapter-thread.jsonl",
            "session_title": "Adapter thread",
            "rollout_exists": true,
            "lifecycle": "active",
            "created_at_ms": 1,
            "updated_at_ms": 2
          }],
          "workflow_node_dispatches": [{
            "dispatch_id": "dispatch:safe-probe",
            "project_id": "project:test",
            "workflow_id": "workflow:test",
            "node_id": "node:codex-dev",
            "work_item_id": "work:test",
            "binding_id": "binding:codex-dev",
            "native_thread_id": "thread:adapter",
            "prompt_kind": "safe_probe",
            "state": "completed"
          }],
          "workflow_execution_controls": [{
            "control_id": "control:reviewed",
            "project_id": "project:test",
            "workflow_id": "workflow:test",
            "work_item_id": "work:test",
            "user_reviewed_instruction": {
              "instruction_id": "instruction:reviewed",
              "summary": "reviewed",
              "objective": "只测试读模型",
              "execution_cwd": "/tmp/adapter-project",
              "sandbox_mode": "workspace-write",
              "approval_state": "reviewed"
            }
          }],
          "permission_requests": [{
            "request_id": "permission:one",
            "project_id": "project:test",
            "workflow_id": "workflow:test",
            "work_item_id": "work:test",
            "permission_kind": "write_workflow_state",
            "reason": "test",
            "status": "pending",
            "requested_at": "2026-06-03T00:00:00Z"
          }],
          "execution_attempts": []
        });
        fs::write(
            &workflow_state_path,
            serde_json::to_string_pretty(&workflow_state).expect("serialize workflow state"),
        )
        .expect("write workflow state");
        let state = AppState {
            index_path: dir.join("codex-index.json"),
            tasks_path: dir.join("tasks.md"),
            workflow_state_path,
        };
        let index = json!({
          "generated_at": "2026-06-03T00:00:00Z",
          "projects": [{
            "project_root": "/tmp/adapter-project",
            "harness_resources": [{
              "root_path": "/tmp/adapter-project/harness",
              "adapter_id": "codex-local"
            }]
          }],
          "threads": [{
            "thread_id": "thread:adapter",
            "title": "Adapter thread",
            "project_root": "/tmp/adapter-project",
            "thread_source": "codex",
            "rollout_path": "/tmp/adapter-thread.jsonl",
            "rollout_exists": true
          }]
        });

        let snapshot =
            build_snapshot_with_session_source(&state, &index, "", SessionSourceMode::IndexOnly);

        assert_eq!(snapshot.agent_adapters.len(), 5);
        let adapter = snapshot
            .agent_adapters
            .iter()
            .find(|descriptor| descriptor.adapter_id == "codex-local")
            .expect("codex-local descriptor");
        assert_eq!(adapter.adapter_id, "codex-local");
        assert_eq!(adapter.source_kind, "backend_read_model");
        assert_eq!(adapter.status, "available");
        assert_eq!(adapter.execution_status, "available_with_user_confirmation");
        assert_eq!(adapter.credential_status, "not_read");
        assert_eq!(adapter.model_access_status, "local_read_model_only");
        assert!(adapter
            .warnings
            .contains(&"adapter_descriptor_is_backend_read_model_only".to_string()));
        assert!(adapter
            .hidden_unimplemented_adapters
            .contains(&"claude-code".to_string()));
        assert!(adapter
            .hidden_unimplemented_adapters
            .contains(&"openclaw".to_string()));
        assert!(adapter
            .hidden_unimplemented_adapters
            .contains(&"opencode-like".to_string()));
        assert!(adapter.capabilities.iter().any(|capability| capability.kind
            == "workflow_node_binding"
            && capability.status == "requires_confirmation"
            && capability
                .evidence_refs
                .contains(&"binding:codex-dev".to_string())));
        assert!(adapter
            .capabilities
            .iter()
            .filter(|capability| [
                "safe_probe_dispatch",
                "user_reviewed_dispatch",
                "workflow_machine_run"
            ]
            .contains(&capability.kind.as_str()))
            .all(|capability| capability.status == "requires_confirmation"));
        let planned = snapshot
            .agent_adapters
            .iter()
            .find(|descriptor| descriptor.adapter_id == "claude-code")
            .expect("planned claude-code descriptor");
        assert_eq!(planned.status, "planned");
        assert_eq!(planned.execution_status, "not_implemented");
        assert_eq!(planned.credential_status, "not_configured");
        assert_eq!(planned.model_access_status, "not_verified");
        assert_eq!(planned.implemented_action_kinds.len(), 0);
        assert_eq!(planned.capabilities.len(), 0);
        assert!(planned.requires_user_setup);
        assert!(planned.unavailable_reason.is_some());
        assert!(planned
            .warnings
            .contains(&"planned_adapter_not_connected".to_string()));
        assert!(planned
            .warnings
            .contains(&"no_execution_button".to_string()));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn backend_agent_adapter_descriptor_is_stable_without_codex_signals() {
        let dir = test_temp_dir("adapter-backend-empty");
        fs::create_dir_all(&dir).expect("create temp dir");
        let state = AppState {
            index_path: dir.join("codex-index.json"),
            tasks_path: dir.join("tasks.md"),
            workflow_state_path: dir.join("missing-workflow-state.v0.json"),
        };
        let snapshot = build_snapshot_with_session_source(
            &state,
            &json!({ "projects": [], "threads": [] }),
            "",
            SessionSourceMode::IndexOnly,
        );

        assert_eq!(snapshot.agent_adapters.len(), 5);
        let adapter = snapshot
            .agent_adapters
            .iter()
            .find(|descriptor| descriptor.adapter_id == "codex-local")
            .expect("codex-local descriptor");
        assert_eq!(adapter.adapter_id, "codex-local");
        assert_eq!(adapter.source_kind, "backend_read_model");
        assert_eq!(adapter.status, "not_connected");
        assert_eq!(adapter.execution_status, "not_connected");
        assert_eq!(
            adapter.unavailable_reason.as_deref(),
            Some("codex_signal_missing")
        );
        assert!(adapter
            .warnings
            .contains(&"workflow_state_snapshot_missing_for_adapter_descriptor".to_string()));
        assert!(adapter.capabilities.iter().any(|capability| {
            capability.kind == "session_index_read"
                && capability.status == "blocked"
                && capability
                    .warnings
                    .contains(&"codex_session_index_empty".to_string())
        }));
        assert!(snapshot.agent_adapters.iter().any(|descriptor| {
            descriptor.adapter_id == "opencode-like"
                && descriptor.status == "planned"
                && descriptor.implemented_action_kinds.is_empty()
                && descriptor.credential_status == "not_configured"
        }));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn session_operation_descriptors_cover_e2_boundary_matrix() {
        let descriptors = derive_agent_adapter_descriptors(&[], &[], None, None);
        let operations = derive_session_operation_descriptors(&descriptors);

        assert_eq!(descriptors.len(), 5);
        assert_eq!(operations.len(), 40);

        let required_operations = [
            "new_session",
            "send_message",
            "stop",
            "restart",
            "resume",
            "export",
            "delete",
            "favorite",
        ];
        for operation_id in required_operations {
            assert_eq!(
                operations
                    .iter()
                    .filter(|operation| operation.operation_id == operation_id)
                    .count(),
                5,
                "{operation_id} should be present for every adapter"
            );
        }

        assert!(operations.iter().all(|operation| ![
            "available",
            "available_to_execute",
            "executable"
        ]
        .contains(&operation.current_status.as_str())));
        assert!(operations.iter().all(|operation| operation
            .warnings
            .contains(&"session_operation_boundary_read_model_only".to_string())));
        assert!(operations.iter().all(|operation| operation
            .warnings
            .contains(&"no_session_operation_execution_in_e2".to_string())));

        let codex_new_session = operations
            .iter()
            .find(|operation| {
                operation.adapter_id == "codex-local" && operation.operation_id == "new_session"
            })
            .expect("codex new_session operation");
        assert_eq!(codex_new_session.current_status, "requires_future_task");
        assert!(codex_new_session.requires_user_confirmation);
        assert!(codex_new_session.writes_codex_home);
        assert!(codex_new_session.writes_workbench_state);
        assert_eq!(
            codex_new_session.applies_to_session_state,
            "work_item_without_native_session"
        );
        assert!(codex_new_session
            .warnings
            .contains(&"h3_1_new_session_noop_only".to_string()));

        let codex_send = operations
            .iter()
            .find(|operation| {
                operation.adapter_id == "codex-local" && operation.operation_id == "send_message"
            })
            .expect("codex send_message operation");
        assert_eq!(codex_send.current_status, "requires_future_task");
        assert!(codex_send.requires_user_confirmation);
        assert!(codex_send.writes_codex_home);
        assert!(codex_send.writes_workbench_state);

        let codex_resume = operations
            .iter()
            .find(|operation| {
                operation.adapter_id == "codex-local" && operation.operation_id == "resume"
            })
            .expect("codex resume operation");
        assert_eq!(codex_resume.current_status, "requires_future_task");
        assert!(codex_resume
            .warnings
            .contains(&"workflow_dispatch_is_not_session_center_resume".to_string()));
        assert!(codex_resume
            .unavailable_reason
            .contains("不等于会话中心通用 resume"));

        let delete_operations = operations
            .iter()
            .filter(|operation| operation.operation_id == "delete")
            .collect::<Vec<_>>();
        assert_eq!(delete_operations.len(), 5);
        assert!(delete_operations.iter().all(|operation| {
            operation.current_status == "blocked_destructive"
                && operation.risk_level == "destructive"
                && operation.writes_codex_home
                && operation
                    .warnings
                    .contains(&"destructive_operation_blocked".to_string())
        }));

        let planned_operations = operations
            .iter()
            .filter(|operation| operation.adapter_id != "codex-local")
            .collect::<Vec<_>>();
        assert_eq!(planned_operations.len(), 32);
        assert!(planned_operations.iter().all(|operation| operation
            .warnings
            .contains(&"planned_adapter_operation_not_available".to_string())));
        assert!(planned_operations
            .iter()
            .all(|operation| operation.applies_to_session_state
                == "planned_adapter_without_session_source"));
    }

    #[test]
    fn provider_availability_summaries_cover_e3_boundary_matrix() {
        let descriptors = derive_agent_adapter_descriptors(&[], &[], None, None);
        let operations = derive_session_operation_descriptors(&descriptors);
        let summaries = derive_provider_availability_summaries(&descriptors, &operations);

        assert_eq!(descriptors.len(), 5);
        assert_eq!(summaries.len(), 5);
        assert!(summaries.iter().all(|summary| summary.safe_to_display));
        assert!(summaries.iter().all(|summary| summary
            .warnings
            .contains(&"provider_availability_read_model_only".to_string())));
        assert!(summaries.iter().all(|summary| summary
            .warnings
            .contains(&"credential_secret_not_read".to_string())));
        assert!(summaries.iter().all(|summary| summary
            .warnings
            .contains(&"provider_availability_not_project_authorization".to_string())));

        let codex = summaries
            .iter()
            .find(|summary| summary.adapter_id == "codex-local")
            .expect("codex provider summary");
        assert_eq!(codex.provider_kind, "local_cli");
        assert_eq!(codex.credential_status, "not_required_by_workbench");
        assert_eq!(codex.model_status, "local_cli_managed");
        assert_eq!(codex.external_call_status, "not_needed_for_readonly");
        assert_eq!(codex.cost_risk_status, "unknown");
        assert!(codex.requires_future_task);
        assert!(codex.user_visible_reason.contains("不读取凭据"));
        assert!(codex.user_visible_reason.contains("不验证模型"));

        let planned = summaries
            .iter()
            .filter(|summary| summary.adapter_id != "codex-local")
            .collect::<Vec<_>>();
        assert_eq!(planned.len(), 4);
        assert!(planned.iter().all(|summary| {
            summary.availability_status == "planned"
                && summary.credential_status == "credential_missing"
                && summary.model_status == "model_unverified"
                && summary.external_call_status == "external_call_blocked"
                && summary.cost_risk_status == "blocked_until_authorized"
                && summary.requires_user_configuration
                && summary.requires_future_task
                && summary
                    .warnings
                    .contains(&"planned_adapter_not_connected".to_string())
                && summary
                    .warnings
                    .contains(&"external_call_blocked".to_string())
        }));
        assert!(summaries.iter().all(|summary| ![
            "model_available",
            "credential_configured",
            "available_to_execute",
            "provider_verified"
        ]
        .contains(&summary.availability_status.as_str())));
    }

    #[test]
    fn session_continuation_guard_covers_e4_boundary_matrix() {
        let descriptors = derive_agent_adapter_descriptors(&[], &[], None, None);
        let operations = derive_session_operation_descriptors(&descriptors);
        let summaries = derive_provider_availability_summaries(&descriptors, &operations);
        let codex = descriptors
            .iter()
            .find(|descriptor| descriptor.adapter_id == "codex-local")
            .expect("codex adapter descriptor");
        let codex_send = operations
            .iter()
            .find(|operation| {
                operation.adapter_id == "codex-local" && operation.operation_id == "send_message"
            })
            .expect("codex send operation");
        let codex_provider = summaries
            .iter()
            .find(|summary| summary.adapter_id == "codex-local");
        let request = SessionContinuationRequest {
            adapter_id: "codex-local".to_string(),
            operation_id: "send_message".to_string(),
            project_id: Some("project:fixture".to_string()),
            project_root: Some("/workspace/project".to_string()),
            workflow_id: Some("workflow:fixture".to_string()),
            node_id: Some("node:dev".to_string()),
            session_id: Some("thread-fixture".to_string()),
            work_item_id: Some("work-item:fixture".to_string()),
            target_cwd: Some("/workspace/project".to_string()),
            allowed_write_roots: vec!["/workspace/project".to_string()],
            sandbox: "workspace-write-preview-only".to_string(),
            prompt_source_kind: "workflow_followup".to_string(),
            prompt_summary: "E4 prompt summary preview only".to_string(),
            readback_strategy: "required".to_string(),
            requested_by: "test".to_string(),
            user_confirmation_state: "missing".to_string(),
        };

        let needs_confirmation = inspect_session_continuation_guard(
            &request,
            Some(codex),
            Some(codex_send),
            codex_provider,
        );
        assert_eq!(needs_confirmation.status, "needs_user_confirmation");
        assert!(needs_confirmation.allows_preview);
        assert!(needs_confirmation.blocks_execution);
        assert!(needs_confirmation.requires_user_confirmation);
        assert!(needs_confirmation
            .reasons
            .contains(&"user_confirmation_required_before_execution".to_string()));

        let confirmed = inspect_session_continuation_guard(
            &SessionContinuationRequest {
                user_confirmation_state: "confirmed".to_string(),
                ..request.clone()
            },
            Some(codex),
            Some(codex_send),
            codex_provider,
        );
        assert_eq!(confirmed.status, "allowed_preview");
        assert!(confirmed.blocks_execution);
        assert!(!confirmed.requires_user_confirmation);

        let missing_project = inspect_session_continuation_guard(
            &SessionContinuationRequest {
                project_id: None,
                ..request.clone()
            },
            Some(codex),
            Some(codex_send),
            codex_provider,
        );
        assert_eq!(missing_project.status, "blocked");
        assert!(missing_project
            .reasons
            .contains(&"missing_project_binding".to_string()));

        let out_of_scope = inspect_session_continuation_guard(
            &SessionContinuationRequest {
                target_cwd: Some("/workspace/other".to_string()),
                ..request.clone()
            },
            Some(codex),
            Some(codex_send),
            codex_provider,
        );
        assert_eq!(out_of_scope.status, "blocked");
        assert!(out_of_scope
            .reasons
            .contains(&"cwd_out_of_scope_blocked".to_string()));

        let sensitive = inspect_session_continuation_guard(
            &SessionContinuationRequest {
                target_cwd: Some("/workspace/project/.env".to_string()),
                ..request.clone()
            },
            Some(codex),
            Some(codex_send),
            codex_provider,
        );
        assert_eq!(sensitive.status, "blocked");
        assert!(sensitive
            .reasons
            .iter()
            .any(|reason| reason.starts_with("sensitive_path_blocked")));

        let no_readback = inspect_session_continuation_guard(
            &SessionContinuationRequest {
                readback_strategy: "not_defined".to_string(),
                ..request.clone()
            },
            Some(codex),
            Some(codex_send),
            codex_provider,
        );
        assert_eq!(no_readback.status, "blocked");
        assert!(no_readback
            .reasons
            .contains(&"readback_strategy_required".to_string()));

        let codex_new_session = operations
            .iter()
            .find(|operation| {
                operation.adapter_id == "codex-local" && operation.operation_id == "new_session"
            })
            .expect("codex new_session operation");
        let new_session = inspect_session_continuation_guard(
            &SessionContinuationRequest {
                operation_id: "new_session".to_string(),
                session_id: None,
                prompt_source_kind: "h3_new_session_task_package".to_string(),
                ..request.clone()
            },
            Some(codex),
            Some(codex_new_session),
            codex_provider,
        );
        assert_eq!(new_session.status, "needs_user_confirmation");
        assert!(new_session.blocks_execution);
        assert!(!new_session
            .reasons
            .contains(&"missing_session_binding".to_string()));
        assert!(new_session
            .warnings
            .contains(&"new_session_does_not_require_existing_session".to_string()));

        let missing_work_item = inspect_session_continuation_guard(
            &SessionContinuationRequest {
                operation_id: "new_session".to_string(),
                session_id: None,
                work_item_id: None,
                prompt_source_kind: "h3_new_session_task_package".to_string(),
                ..request.clone()
            },
            Some(codex),
            Some(codex_new_session),
            codex_provider,
        );
        assert_eq!(missing_work_item.status, "blocked");
        assert!(missing_work_item
            .reasons
            .contains(&"missing_work_item_binding".to_string()));

        let planned_adapter = descriptors
            .iter()
            .find(|descriptor| descriptor.adapter_id == "claude-code")
            .expect("planned adapter descriptor");
        let planned_operation = operations
            .iter()
            .find(|operation| {
                operation.adapter_id == "claude-code" && operation.operation_id == "send_message"
            })
            .expect("planned send operation");
        let planned_provider = summaries
            .iter()
            .find(|summary| summary.adapter_id == "claude-code");
        let planned = inspect_session_continuation_guard(
            &SessionContinuationRequest {
                adapter_id: "claude-code".to_string(),
                ..request
            },
            Some(planned_adapter),
            Some(planned_operation),
            planned_provider,
        );
        assert_eq!(planned.status, "blocked");
        assert!(planned
            .reasons
            .iter()
            .any(|reason| reason.contains("planned_adapter_blocked")));
        assert!(planned
            .warnings
            .contains(&"provider_availability_not_execution_authorization".to_string()));
    }
