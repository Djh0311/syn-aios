    #[test]
    fn work_item_state_update_rejects_non_index_project() {
        let dir = std::env::temp_dir().join(format!(
            "work-item-state-non-index-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let index = json!({
          "projects": [{ "project_root": "/tmp/indexed-project" }]
        });
        let request = fixture_task_draft_request(&project.project_root, "索引内工作项");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &request).expect("work item should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        let update = fixture_work_item_state_update_request(
            "/tmp/not-indexed",
            &work_item_id,
            "ready_to_dispatch",
        );

        let result = update_work_item_state_for_index_project_at(&path, &index, &update);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("项目不在当前索引内"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn workflow_node_session_binding_binds_rebinds_and_unbinds() {
        let dir =
            std::env::temp_dir().join(format!("node-session-bind-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let draft = fixture_task_draft_request(&project.project_root, "节点绑定工作项");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &draft).expect("work item should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        let workflow_id = default_workflow_id(&project.project_root);
        let node_id = format!("{workflow_id}:node:codex-dev");
        let first_session = fixture_session("thread-001", &project.project_root, true);
        let first_request = fixture_node_session_bind_request(
            &project.project_root,
            &node_id,
            Some(&work_item_id),
            &first_session.thread_id,
        );

        let first = bind_workflow_node_codex_session_at(&path, &first_request, &first_session)
            .expect("binding should write");

        assert_eq!(
            first.snapshot.project_workflows[0]
                .node_session_bindings
                .len(),
            1
        );
        assert_eq!(
            first.snapshot.project_workflows[0].node_session_bindings[0].native_thread_id,
            "thread-001"
        );
        let updated = read_json_file(&path);
        assert_eq!(
            updated["workflow_node_session_bindings"][0]["binding_source"],
            "workflow_bound"
        );
        assert!(updated["audit_events"]
            .as_array()
            .expect("audit events should be array")
            .iter()
            .any(|event| event["event_type"] == "workflow_node_session_bound"));

        let second_session = fixture_session("thread-002", &project.project_root, false);
        let second_request = fixture_node_session_bind_request(
            &project.project_root,
            &node_id,
            Some(&work_item_id),
            &second_session.thread_id,
        );
        let second = bind_workflow_node_codex_session_at(&path, &second_request, &second_session)
            .expect("rebind should write");

        assert_eq!(
            second.snapshot.project_workflows[0]
                .node_session_bindings
                .len(),
            1
        );
        assert_eq!(
            second.snapshot.project_workflows[0].node_session_bindings[0].native_thread_id,
            "thread-002"
        );
        assert_eq!(
            second.snapshot.project_workflows[0].node_session_bindings[0].warnings,
            vec!["index_session_rollout_missing".to_string()]
        );
        let rebound = read_json_file(&path);
        assert!(rebound["audit_events"]
            .as_array()
            .expect("audit events should be array")
            .iter()
            .any(|event| event["event_type"] == "workflow_node_session_rebound"));
        let binding_id =
            optional_string_from(&rebound["workflow_node_session_bindings"][0], "binding_id")
                .expect("binding id should exist");
        let unbind_request =
            fixture_node_session_unbind_request(&project.project_root, &binding_id);
        let unbound = unbind_workflow_node_codex_session_at(&path, &unbind_request)
            .expect("unbind should write");

        assert!(unbound.snapshot.project_workflows[0]
            .node_session_bindings
            .is_empty());
        let detached = read_json_file(&path);
        assert_eq!(
            detached["workflow_node_session_bindings"][0]["lifecycle"],
            "detached"
        );
        assert!(detached["audit_events"]
            .as_array()
            .expect("audit events should be array")
            .iter()
            .any(|event| event["event_type"] == "workflow_node_session_unbound"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn workflow_node_session_binding_rejects_non_index_session_and_missing_node() {
        let dir =
            std::env::temp_dir().join(format!("node-session-reject-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let index = json!({
          "projects": [{ "project_root": "/tmp/indexed-project" }],
          "threads": [{ "thread_id": "thread-001", "project_root": "/tmp/indexed-project", "title": "Indexed" }]
        });
        let workflow_id = default_workflow_id(&project.project_root);
        let node_id = format!("{workflow_id}:node:codex-dev");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let missing_session_request =
            fixture_node_session_bind_request(&project.project_root, &node_id, None, "missing");
        let missing_session =
            bind_workflow_node_codex_session_for_index_at(&path, &index, &missing_session_request);

        assert!(missing_session.is_err());
        assert!(missing_session.unwrap_err().contains("会话不在当前索引内"));

        let session = fixture_session("thread-001", &project.project_root, true);
        // 后置C#2：bind 现在从 node_id 解析 workflow_id；要测「缺节点」就把缺的节点挂在**存在的**
        // 默认工作流上（否则解析出不存在的 workflow，先报「还没有本地 workflow」而非「找不到该 node」）。
        let missing_node_id = format!("{}:node:nope", default_workflow_id(&project.project_root));
        let missing_node_request = fixture_node_session_bind_request(
            &project.project_root,
            &missing_node_id,
            None,
            &session.thread_id,
        );
        let missing_node =
            bind_workflow_node_codex_session_at(&path, &missing_node_request, &session);

        assert!(missing_node.is_err());
        assert!(missing_node.unwrap_err().contains("找不到该 node"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_preview_rejects_non_index_project() {
        let dir = std::env::temp_dir().join(format!(
            "task-preview-non-index-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let index = json!({
          "projects": [{ "project_root": "/tmp/indexed-project" }]
        });
        let request = fixture_task_preview_request("/tmp/not-indexed", "work-item:missing");

        initialize_workflow_state_at(&path).expect("state should exist");
        let result = render_task_package_preview_for_index_project_at(&path, &index, &request);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("项目不在当前索引内"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_preview_rejects_missing_state_file() {
        let dir = std::env::temp_dir().join(format!(
            "task-preview-missing-state-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let request = fixture_task_preview_request(&project.project_root, "work-item:missing");

        let result = render_task_package_preview_at(&path, &project, &request);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("工作流状态文件不存在"));
        assert!(!path.exists());
    }

    #[test]
    fn task_package_preview_rejects_missing_workflow() {
        let dir = std::env::temp_dir().join(format!(
            "task-preview-missing-workflow-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let request = fixture_task_preview_request(&project.project_root, "work-item:missing");

        initialize_workflow_state_at(&path).expect("state should exist");
        let result = render_task_package_preview_at(&path, &project, &request);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("还没有本地 workflow"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_preview_rejects_missing_work_item() {
        let dir = std::env::temp_dir().join(format!(
            "task-preview-missing-work-item-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let request = fixture_task_preview_request(&project.project_root, "work-item:missing");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let result = render_task_package_preview_at(&path, &project, &request);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("找不到该 work item"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_preview_renders_markdown_from_draft() {
        let dir =
            std::env::temp_dir().join(format!("task-preview-render-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let draft_request = fixture_task_draft_request(&project.project_root, "登记任务包草稿");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &draft_request).expect("task draft should be created");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        let preview_request = fixture_task_preview_request(&project.project_root, &work_item_id);
        let preview = render_task_package_preview_at(&path, &project, &preview_request)
            .expect("preview should render");

        assert_eq!(preview.project_root, project.project_root);
        assert_eq!(preview.work_item_id, work_item_id);
        assert!(preview.markdown.contains("# 任务包：登记任务包草稿"));
        assert!(preview.markdown.contains("## 所属开发线"));
        assert!(preview.markdown.contains("Codex 开发线"));
        assert!(preview.markdown.contains("## 背景"));
        assert!(preview.markdown.contains("## 目标"));
        assert!(preview.markdown.contains("写入 work_items 和 artifacts"));
        assert!(preview.markdown.contains("## 允许读取"));
        assert!(preview.markdown.contains("## 允许写入"));
        assert!(preview.markdown.contains("## 禁止事项"));
        assert!(preview.markdown.contains("## 验收标准"));
        assert!(preview.markdown.contains("## 必须回传"));
        assert!(preview.markdown.contains("## 总指导回收重点"));
        assert!(preview.markdown.contains("待补充"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_preview_uses_placeholders_for_missing_fields() {
        let dir = std::env::temp_dir().join(format!(
            "task-preview-placeholders-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let mut value = read_json_file(&path);
        let workflow_id = default_workflow_id(&project.project_root);
        let work_item_id = format!("work-item:{workflow_id}:manual");
        let artifact_id = format!("artifact:{workflow_id}:manual");
        array_mut(&mut value, "work_items")
            .expect("work_items should exist")
            .push(json!({
              "work_item_id": work_item_id,
              "project_id": project_id(&project.project_root),
              "workflow_id": workflow_id,
              "state": "draft",
              "source_kind": "workspace_state",
              "source_ref": artifact_id
            }));
        array_mut(&mut value, "artifacts")
            .expect("artifacts should exist")
            .push(json!({
              "artifact_id": artifact_id,
              "artifact_type": "task_package",
              "project_id": project_id(&project.project_root),
              "source_kind": "workspace_state",
              "source_ref": work_item_id
            }));
        atomic_write_json(&path, &value).expect("fixture should write");

        let request = fixture_task_preview_request(&project.project_root, &work_item_id);
        let preview = render_task_package_preview_at(&path, &project, &request)
            .expect("preview should render");

        assert!(preview.markdown.contains("# 任务包：待补充"));
        assert!(preview.markdown.contains("未登记"));
        assert!(preview.markdown.contains("业务背景：待补充"));
        assert!(preview
            .warnings
            .iter()
            .any(|warning| warning.contains("任务名未登记")));
        assert!(preview
            .warnings
            .iter()
            .any(|warning| warning.contains("所属开发线未登记")));
        assert!(preview
            .warnings
            .iter()
            .any(|warning| warning.contains("目标说明未登记")));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn workflow_task_package_read_model_derives_v1_objects_from_v0_state() {
        let dir =
            std::env::temp_dir().join(format!("workflow-read-model-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let tasks_dir = dir.join("tasks");
        let project = fixture_project("/tmp/indexed-project");
        let draft_request = fixture_task_draft_request(&project.project_root, "派生读模型任务");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &draft_request).expect("task draft should be created");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        let mut fields = ready_fields_update_request(&project.project_root, &work_item_id);
        fields.fields.assigned_line = "Codex 开发线".to_string();
        update_task_package_draft_fields_at(&path, &fields).expect("fields should save");
        mark_task_package_fixture_ready(&path, "codex-test-model");
        generate_task_package_file_at(
            &path,
            &project,
            &fixture_task_file_generation_request(&project.project_root, &work_item_id),
            &tasks_dir,
        )
        .expect("file should generate");
        append_fixture_dispatch(
            &path,
            &project.project_root,
            &work_item_id,
            "completed",
            "thread-001",
        );

        let snapshot = read_workflow_state_snapshot(&path).expect("snapshot should read");
        let derived = snapshot.project_workflows[0]
            .derived_workflow
            .as_ref()
            .expect("derived workflow should exist");

        assert_eq!(
            derived.workflow_id,
            default_workflow_id(&project.project_root)
        );
        assert!(!derived.nodes.is_empty());
        assert_eq!(derived.task_packages.len(), 1);
        assert_eq!(derived.task_packages[0].version, 2);
        assert_eq!(
            derived.task_packages[0].model_id.as_deref(),
            Some("codex-test-model")
        );
        assert!(derived.task_packages[0].available_memory_refs.is_empty());
        assert!(derived.task_packages[0].available_knowledge_refs.is_empty());
        assert!(derived
            .ledger_entries
            .iter()
            .any(|entry| entry.entry_type == "task_package_created"));
        assert!(derived
            .warnings
            .iter()
            .any(|warning| warning.contains("derived_from_workflow_state_v0")));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn project_blackboard_read_model_derives_candidates_without_state_promotion() {
        let dir = std::env::temp_dir().join(format!(
            "project-blackboard-read-model-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let tasks_dir = dir.join("tasks");
        let project = fixture_project("/tmp/indexed-project");
        let draft_request = fixture_task_draft_request(&project.project_root, "黑板候选任务");
        let workflow_id = default_workflow_id(&project.project_root);

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &draft_request).expect("task draft should be created");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        let mut fields = ready_fields_update_request(&project.project_root, &work_item_id);
        fields.fields.assigned_line = "Codex 开发线".to_string();
        update_task_package_draft_fields_at(&path, &fields).expect("fields should save");
        mark_task_package_fixture_ready(&path, "codex-test-model");
        generate_task_package_file_at(
            &path,
            &project,
            &fixture_task_file_generation_request(&project.project_root, &work_item_id),
            &tasks_dir,
        )
        .expect("file should generate");
        append_fixture_dispatch(
            &path,
            &project.project_root,
            &work_item_id,
            "completed",
            "thread-001",
        );

        let mut value = read_json_file(&path);
        let artifact = value["artifacts"]
            .as_array_mut()
            .expect("artifacts should be array")
            .first_mut()
            .expect("task package artifact should exist");
        artifact["available_memory_refs"] = json!(["memory:candidate:001"]);
        artifact["available_knowledge_refs"] = json!(["knowledge:ref:001"]);
        let (dispatch_id_value, node_id_value) = {
            let dispatch = value["workflow_node_dispatches"]
                .as_array_mut()
                .expect("dispatches should be array")
                .first_mut()
                .expect("dispatch should exist");
            dispatch["prompt_kind"] = json!("tool_call_summary");
            dispatch["prompt_preview"] = json!("工具摘要，只保留摘要和引用。");
            dispatch["tool_call_ref"] = json!("tool-call:blackboard:001");
            dispatch["warnings"] = json!(["direction_risk_blackboard"]);
            (dispatch["dispatch_id"].clone(), dispatch["node_id"].clone())
        };
        value["audit_events"]
            .as_array_mut()
            .expect("audit events should be array")
            .push(json!({
                "event_id": "audit:blackboard:worker-report:001",
                "event_type": "worker_structured_report_recorded",
                "workflow_id": workflow_id,
                "node_id": node_id_value,
                "work_item_id": work_item_id,
                "dispatch_id": dispatch_id_value.clone(),
                "actor_ref": "codex-dev",
                "reason": "worker reported direction risk",
                "open_issues": [],
                "permission_requests": [],
                "direction_risks": ["direction_risk_blackboard"],
                "follow_up_suggestions": [],
                "acceptance_status": "blocked",
                "warnings": []
            }));
        if !value
            .get("permission_requests")
            .is_some_and(Value::is_array)
        {
            value["permission_requests"] = json!([]);
        }
        value["permission_requests"]
            .as_array_mut()
            .expect("permission requests should be array")
            .push(json!({
                "request_id": "permission:blackboard:001",
                "project_id": project_id(&project.project_root),
                "workflow_id": workflow_id,
                "work_item_id": work_item_id,
                "dispatch_id": dispatch_id_value,
                "permission_kind": "write_workflow_state",
                "reason": "需要用户确认是否允许写协议字段。",
                "status": "pending",
                "requested_at": "2026-06-01T00:00:00Z",
                "decided_at": Value::Null,
                "decision": Value::Null,
                "warnings": []
            }));
        write_validated_workflow_state(&path, &value).expect("blackboard fixture should write");

        let snapshot = read_workflow_state_snapshot(&path).expect("snapshot should read");
        let blackboard = snapshot
            .project_blackboards
            .first()
            .expect("project blackboard should be derived");

        assert_eq!(blackboard.project_root, project.project_root);
        assert!(blackboard
            .warnings
            .contains(&"blackboard_promotion_requires_control_core_confirmation".to_string()));
        for kind in [
            BlackboardEntryKind::SubagentReport,
            BlackboardEntryKind::Risk,
            BlackboardEntryKind::PermissionRequest,
            BlackboardEntryKind::ToolSummary,
            BlackboardEntryKind::MemoryCandidate,
            BlackboardEntryKind::KnowledgeRef,
        ] {
            assert!(
                blackboard.entries.iter().any(|entry| entry.kind == kind),
                "blackboard should include {kind:?}: {:?}",
                blackboard.entries
            );
        }
        assert!(blackboard
            .entries
            .iter()
            .all(|entry| entry.status == "candidate"));
        assert!(blackboard
            .entries
            .iter()
            .all(|entry| entry.promotion_decision.status == "candidate_pending_control_core"));
        assert!(blackboard.entries.iter().any(|entry| {
            entry.kind == BlackboardEntryKind::MemoryCandidate
                && entry.promotion_decision.target_kind.as_deref() == Some("formal_memory")
                && entry
                    .warnings
                    .contains(&"memory_candidate_not_formal_memory".to_string())
        }));
        assert!(blackboard.entries.iter().any(|entry| {
            entry.kind == BlackboardEntryKind::KnowledgeRef
                && entry
                    .warnings
                    .contains(&"knowledge_ref_is_not_memory".to_string())
        }));
        assert_eq!(
            project_blackboards_from_workflows(&snapshot.project_workflows),
            snapshot.project_blackboards
        );

        let _ = fs::remove_dir_all(dir);
    }
