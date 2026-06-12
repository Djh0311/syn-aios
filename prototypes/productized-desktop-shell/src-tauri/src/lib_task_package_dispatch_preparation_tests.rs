    #[test]
    fn workflow_run_check_blocks_missing_workflow_and_missing_required_fields() {
        let dir = std::env::temp_dir().join(format!(
            "workflow-run-check-blocked-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let request = WorkflowRunCheckRequest {
            project_root: project.project_root.clone(),
            workflow_id: None,
        };

        let missing = inspect_workflow_run_check_at(&path, &project, &request)
            .expect("missing workflow check should return blocked");
        assert_eq!(missing.status, "blocked");
        assert!(missing
            .blocked_reasons
            .iter()
            .any(|reason| reason.contains("状态文件不存在")));

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(
            &path,
            &fixture_task_draft_request(&project.project_root, "缺字段任务"),
        )
        .expect("task draft should exist");
        let blocked = inspect_workflow_run_check_at(&path, &project, &request)
            .expect("blocked check should inspect");

        assert_eq!(blocked.status, "blocked");
        for expected in [
            "缺模型",
            "没有读范围",
            "没有写范围",
            "没有验收标准",
            "没有 active 会话绑定",
        ] {
            assert!(
                blocked
                    .blocked_reasons
                    .iter()
                    .any(|reason| reason.contains(expected)),
                "blocked reasons should contain {expected}: {:?}",
                blocked.blocked_reasons
            );
        }

        let _ = fs::remove_dir_all(dir);
    }
    #[test]
    fn workflow_run_check_allows_runnable_fixture_without_auto_filling_optional_refs() {
        let dir = std::env::temp_dir().join(format!(
            "workflow-run-check-runnable-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let tasks_dir = dir.join("tasks");
        let project = fixture_project("/tmp/indexed-project");
        let workflow_id = default_workflow_id(&project.project_root);
        let node_id = format!("{workflow_id}:node:codex-dev");
        let thread_id = "thread-001";
        let index = fixture_dispatch_index(&project.project_root, thread_id);

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(
            &path,
            &fixture_task_draft_request(&project.project_root, "可运行任务"),
        )
        .expect("task draft should exist");
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
        bind_workflow_node_codex_session_for_index_at(
            &path,
            &index,
            &fixture_node_session_bind_request(
                &project.project_root,
                &node_id,
                Some(&work_item_id),
                thread_id,
            ),
        )
        .expect("binding should write");

        let check = inspect_workflow_run_check_at(
            &path,
            &project,
            &WorkflowRunCheckRequest {
                project_root: project.project_root.clone(),
                workflow_id: None,
            },
        )
        .expect("run check should inspect");

        assert_eq!(check.status, "warning", "{:?}", check.blocked_reasons);
        assert!(check.blocked_reasons.is_empty());
        assert!(check
            .warnings
            .iter()
            .any(|warning| warning.contains("工具白名单为空")
                || warning.contains("harness 要求为空")));
        let snapshot = read_workflow_state_snapshot(&path).expect("snapshot should read");
        let task_package = &snapshot.project_workflows[0]
            .derived_workflow
            .as_ref()
            .expect("derived workflow should exist")
            .task_packages[0];
        assert!(task_package.available_knowledge_refs.is_empty());
        assert!(task_package.available_memory_refs.is_empty());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_blocks_missing_report_model_and_stale_after_edit() {
        let dir =
            std::env::temp_dir().join(format!("task-package-stale-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let tasks_dir = dir.join("tasks");
        let project = fixture_project("/tmp/indexed-project");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_active_plan_authorization_for_fixture(&path, &project.project_root);
        create_task_draft_at(
            &path,
            &fixture_task_draft_request(&project.project_root, "stale 任务"),
        )
        .expect("task draft should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        update_task_package_draft_fields_at(
            &path,
            &empty_fields_update_request(&project.project_root, &work_item_id),
        )
        .expect("empty fields should save");
        let missing = inspect_task_package_dispatch_readiness_at(
            &path,
            &project,
            &fixture_dispatch_readiness_request(&project.project_root, &work_item_id),
        )
        .expect("missing readiness should inspect");
        assert_eq!(missing.status, "not_ready");
        assert!(missing
            .blocking_reasons
            .iter()
            .any(|reason| reason.contains("缺模型")));
        assert!(missing
            .blocking_reasons
            .iter()
            .any(|reason| reason.contains("report format")));

        update_task_package_draft_fields_at(
            &path,
            &ready_fields_update_request(&project.project_root, &work_item_id),
        )
        .expect("ready fields should save");
        mark_task_package_fixture_ready(&path, "codex-test-model");
        generate_task_package_file_at(
            &path,
            &project,
            &fixture_task_file_generation_request(&project.project_root, &work_item_id),
            &tasks_dir,
        )
        .expect("file should generate");
        let ready = inspect_task_package_dispatch_readiness_at(
            &path,
            &project,
            &fixture_dispatch_readiness_request(&project.project_root, &work_item_id),
        )
        .expect("ready check should inspect");
        assert_eq!(ready.status, "ready");

        let mut changed = ready_fields_update_request(&project.project_root, &work_item_id);
        changed.fields.goals = vec!["人工编辑后必须重新检查。".to_string()];
        update_task_package_draft_fields_at(&path, &changed).expect("edit should mark stale");
        mark_task_package_fixture_ready(&path, "codex-test-model");
        let stale = inspect_task_package_dispatch_readiness_at(
            &path,
            &project,
            &fixture_dispatch_readiness_request(&project.project_root, &work_item_id),
        )
        .expect("stale check should inspect");
        assert_eq!(stale.status, "not_ready");
        assert!(stale
            .blocking_reasons
            .iter()
            .any(|reason| reason.contains("stale")));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_memory_injection_writes_snapshot_to_task_package_artifact() {
        let (dir, path, tasks_dir, project, work_item_id) =
            setup_task_memory_injection_fixture("task-memory-injection-artifact");
        create_formal_memory_for_task(
            &path,
            &project.project_root,
            "派发准备检查必须保留模型显式配置",
            "该正式记忆用于后续 worker 的任务包上下文。",
            "2026-06-04T04:00:00Z",
            "write-m6-artifact-formal",
        );

        let result = generate_task_package_file_at(
            &path,
            &project,
            &fixture_task_file_generation_request(&project.project_root, &work_item_id),
            &tasks_dir,
        )
        .expect("task package should generate with memory snapshot");

        assert_eq!(result.memory_injection_summary.included_count, 1);
        assert_eq!(result.memory_injection_summary.excluded_count, 0);
        assert!(!result.memory_injection_summary.stale);
        let updated = read_json_file(&path);
        let artifact = updated["artifacts"]
            .as_array()
            .expect("artifacts should be array")
            .first()
            .expect("task package artifact should exist");
        let snapshot = artifact
            .get("memory_packet_snapshot")
            .expect("artifact should store frozen memory snapshot");
        let snapshot_id = optional_string_from(snapshot, "snapshot_id")
            .expect("snapshot should include snapshot_id");

        assert_eq!(
            optional_string_from(snapshot, "schema_version").as_deref(),
            Some("task_package_memory_packet_snapshot.v1")
        );
        assert_eq!(
            optional_string_from(snapshot, "retrieval_intent").as_deref(),
            Some("worker_task")
        );
        assert_eq!(
            artifact["memory_packet_fingerprint"],
            snapshot["fingerprint"]
        );
        assert_eq!(
            artifact["memory_packet_generated_at"],
            snapshot["generated_at"]
        );
        assert_eq!(artifact["memory_packet_stale"], false);
        assert_eq!(
            artifact["memory_packet_store_revisions"]["formal_store_revision"],
            json!(1)
        );
        assert_eq!(
            snapshot["included_memories"]
                .as_array()
                .expect("included memories should be array")
                .len(),
            1
        );
        assert!(snapshot["included_memories"][0]["claim"]
            .as_str()
            .unwrap_or("")
            .contains("派发准备检查"));
        assert!(artifact["memory_packet_warnings"]
            .as_array()
            .expect("warnings should be array")
            .iter()
            .any(|warning| warning == "candidate_and_observation_review_materials_only"));
        assert!(updated["audit_events"]
            .as_array()
            .expect("audit events should be array")
            .iter()
            .any(|event| {
                event["event_type"] == "task_memory_packet_injected_into_task_package"
                    && event["reason"]
                        .as_str()
                        .unwrap_or("")
                        .contains(&work_item_id)
                    && event["reason"]
                        .as_str()
                        .unwrap_or("")
                        .contains(&snapshot_id)
                    && event["reason"]
                        .as_str()
                        .unwrap_or("")
                        .contains("included_count=1")
                    && event["reason"]
                        .as_str()
                        .unwrap_or("")
                        .contains("excluded_count=0")
            }));
        let derived_package = &result.snapshot.project_workflows[0]
            .derived_workflow
            .as_ref()
            .expect("derived workflow should exist")
            .task_packages[0];
        assert_eq!(derived_package.memory_injection_summary.included_count, 1);
        assert_eq!(
            derived_package
                .memory_injection_summary
                .snapshot_id
                .as_deref(),
            Some(snapshot_id.as_str())
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_memory_injection_markdown_and_dispatch_prompt_use_same_snapshot() {
        let (dir, path, tasks_dir, project, work_item_id) =
            setup_task_memory_injection_fixture("task-memory-injection-prompt");
        create_formal_memory_for_task(
            &path,
            &project.project_root,
            "派发准备检查必须保留模型显式配置",
            "该正式记忆需要进入任务包 markdown 和派发 prompt。",
            "2026-06-04T04:10:00Z",
            "write-m6-prompt-formal",
        );
        let generated = generate_task_package_file_at(
            &path,
            &project,
            &fixture_task_file_generation_request(&project.project_root, &work_item_id),
            &tasks_dir,
        )
        .expect("task package should generate");
        let markdown = fs::read_to_string(&generated.file_path).expect("markdown should read");
        let state_after_generate = read_json_file(&path);
        let snapshot_id = optional_string_from(
            &state_after_generate["artifacts"][0]["memory_packet_snapshot"],
            "snapshot_id",
        )
        .expect("snapshot id should exist");

        assert!(markdown.contains("## 正式记忆上下文"));
        assert!(markdown.contains(&snapshot_id));
        assert!(markdown.contains("派发准备检查必须保留模型显式配置"));
        assert!(markdown.contains("任务包内容不会回灌成正式记忆"));
        assert!(markdown.contains("候选 / 观察仅作为待审查材料"));
        assert!(!markdown.contains("worker 已收到记忆包"));

        update_work_item_state_at(
            &path,
            &fixture_work_item_state_update_request(
                &project.project_root,
                &work_item_id,
                "ready_to_dispatch",
            ),
        )
        .expect("work item should be ready");
        let workflow_id = default_workflow_id(&project.project_root);
        let node_id = format!("{workflow_id}:node:codex-dev");
        let index = fixture_dispatch_index(&project.project_root, "thread-m6-prompt");
        let session = fixture_session("thread-m6-prompt", &project.project_root, true);
        bind_workflow_node_codex_session_at(
            &path,
            &fixture_node_session_bind_request(
                &project.project_root,
                &node_id,
                Some(&work_item_id),
                "thread-m6-prompt",
            ),
            &session,
        )
        .expect("binding should write");

        let prepared = prepare_workflow_node_dispatch_for_index_at(
            &path,
            &index,
            &fixture_dispatch_prepare_request(&project.project_root, &node_id, &work_item_id),
        )
        .expect("prepared dispatch should include memory block");

        assert_eq!(
            prepared.dispatch.memory_packet_snapshot_id.as_deref(),
            Some(snapshot_id.as_str())
        );
        assert!(prepared
            .dispatch
            .prompt_preview
            .contains("## 正式记忆上下文"));
        assert!(prepared.dispatch.prompt_preview.contains(&snapshot_id));
        assert!(prepared
            .dispatch
            .prompt_preview
            .contains("派发准备检查必须保留模型显式配置"));
        assert!(!prepared
            .dispatch
            .prompt_preview
            .contains("worker 已收到记忆包"));
        let updated = read_json_file(&path);
        assert_eq!(
            updated["workflow_node_dispatches"][0]["memory_packet_snapshot_id"],
            snapshot_id
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_memory_injection_marks_snapshot_stale_on_store_revision_change() {
        let (dir, path, tasks_dir, project, work_item_id) =
            setup_task_memory_injection_fixture("task-memory-injection-stale");
        create_formal_memory_for_task(
            &path,
            &project.project_root,
            "派发准备检查必须保留模型显式配置",
            "第一条正式记忆进入任务包。",
            "2026-06-04T04:20:00Z",
            "write-m6-stale-formal-001",
        );
        generate_task_package_file_at(
            &path,
            &project,
            &fixture_task_file_generation_request(&project.project_root, &work_item_id),
            &tasks_dir,
        )
        .expect("task package should generate");
        create_formal_memory_for_task(
            &path,
            &project.project_root,
            "派发准备检查新增了验证边界",
            "正式记忆 store revision 变化后，旧任务包快照必须 stale。",
            "2026-06-04T04:20:01Z",
            "write-m6-stale-formal-002",
        );

        let readiness = inspect_task_package_dispatch_readiness_at(
            &path,
            &project,
            &fixture_dispatch_readiness_request(&project.project_root, &work_item_id),
        )
        .expect("readiness should inspect stale memory snapshot");

        assert_eq!(readiness.status, "not_ready");
        assert!(readiness.memory_injection_summary.stale);
        assert!(readiness
            .memory_injection_summary
            .stale_reasons
            .iter()
            .any(|reason| reason.contains("formal_store_revision")));
        assert!(readiness
            .blocking_reasons
            .iter()
            .any(|reason| reason.contains("记忆快照已 stale")));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_memory_injection_blocks_readiness_when_required_snapshot_missing() {
        let (dir, path, tasks_dir, project, work_item_id) =
            setup_task_memory_injection_fixture("task-memory-injection-missing");
        create_formal_memory_for_task(
            &path,
            &project.project_root,
            "派发准备检查必须保留模型显式配置",
            "先生成完整任务包，再模拟旧 artifact 缺快照。",
            "2026-06-04T04:30:00Z",
            "write-m6-missing-formal",
        );
        generate_task_package_file_at(
            &path,
            &project,
            &fixture_task_file_generation_request(&project.project_root, &work_item_id),
            &tasks_dir,
        )
        .expect("task package should generate");
        let mut value = read_json_file(&path);
        let artifact = value["artifacts"]
            .as_array_mut()
            .expect("artifacts should be array")
            .first_mut()
            .expect("artifact should exist");
        artifact["requires_memory_refs"] = Value::Bool(true);
        artifact["available_memory_refs"] = json!(["memory:required"]);
        artifact["memory_packet_snapshot"] = Value::Null;
        artifact["memory_packet_fingerprint"] = Value::Null;
        artifact["memory_packet_generated_at"] = Value::Null;
        artifact["memory_packet_store_revisions"] = Value::Null;
        artifact["memory_packet_stale"] = Value::Bool(true);
        artifact["memory_packet_warnings"] = json!(["task_memory_packet_snapshot_missing"]);
        write_validated_workflow_state(&path, &value)
            .expect("fixture missing snapshot should write");

        let readiness = inspect_task_package_dispatch_readiness_at(
            &path,
            &project,
            &fixture_dispatch_readiness_request(&project.project_root, &work_item_id),
        )
        .expect("readiness should inspect missing memory snapshot");

        assert_eq!(readiness.status, "not_ready");
        assert_eq!(readiness.memory_injection_summary.included_count, 0);
        assert!(readiness.memory_injection_summary.stale);
        assert!(readiness
            .blocking_reasons
            .iter()
            .any(|reason| reason.contains("任务包声明需要记忆作为依据")
                && reason.contains("记忆快照缺失")));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_memory_injection_excludes_lint_blocked_formal_memory() {
        let (dir, path, tasks_dir, project, work_item_id) =
            setup_task_memory_injection_fixture("task-memory-injection-lint");
        create_formal_memory_with_source(
            &path,
            &project.project_root,
            "接口缓存必须启用",
            "source:lint:m6:blocking",
            "evidence",
            "2026-06-04T04:40:00Z",
            "write-m6-lint-formal",
        );
        let candidate =
            create_confirmed_candidate_with_claim(&path, &project.project_root, "接口缓存禁止启用");
        let mut lint_input = fixture_memory_lint_run_input(
            &project.project_root,
            MemoryLintRunIntent::CandidateAdoptionGuard,
        );
        lint_input.candidate_key = Some(candidate.candidate_key);
        run_memory_lint_at(
            &path,
            &lint_input,
            "2026-06-04T04:40:02Z",
            "write-m6-lint-run",
        )
        .expect("lint should write blocking finding");
        let mut fields = ready_fields_update_request(&project.project_root, &work_item_id);
        fields.fields.goals = vec!["接口缓存后续处理。".to_string()];
        update_task_package_draft_fields_at(&path, &fields)
            .expect("fields should target lint claim");
        mark_task_package_fixture_ready(&path, "codex-test-model");

        let result = generate_task_package_file_at(
            &path,
            &project,
            &fixture_task_file_generation_request(&project.project_root, &work_item_id),
            &tasks_dir,
        )
        .expect("task package should generate even with lint excluded memory");

        assert_eq!(result.memory_injection_summary.included_count, 0);
        let updated = read_json_file(&path);
        let snapshot = &updated["artifacts"][0]["memory_packet_snapshot"];
        assert_eq!(
            snapshot["included_memories"]
                .as_array()
                .expect("included should be array")
                .len(),
            0
        );
        assert!(snapshot["excluded_items"]
            .as_array()
            .expect("excluded should be array")
            .iter()
            .any(|item| item["reason"] == "conflicted"
                && item["detail"]
                    .as_str()
                    .unwrap_or("")
                    .contains("memory lint open blocking finding")));
        assert!(snapshot["warnings"]
            .as_array()
            .expect("warnings should be array")
            .iter()
            .any(|warning| warning == "memory_lint_blocking_findings_excluded"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_fields_update_rejects_non_index_project() {
        let dir =
            std::env::temp_dir().join(format!("task-fields-non-index-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let index = json!({
          "projects": [{ "project_root": "/tmp/indexed-project" }]
        });
        let request = fixture_fields_update_request("/tmp/not-indexed", "work-item:missing");

        initialize_workflow_state_at(&path).expect("state should exist");
        let result = update_task_package_draft_fields_for_index_project_at(&path, &index, &request);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("项目不在当前索引内"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_fields_update_rejects_missing_state_file() {
        let dir = std::env::temp_dir().join(format!(
            "task-fields-missing-state-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let request = fixture_fields_update_request("/tmp/indexed-project", "work-item:missing");

        let result = update_task_package_draft_fields_at(&path, &request);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("工作流状态文件不存在"));
        assert!(!path.exists());
    }

    #[test]
    fn task_package_fields_update_rejects_missing_workflow() {
        let dir = std::env::temp_dir().join(format!(
            "task-fields-missing-workflow-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let request = fixture_fields_update_request("/tmp/indexed-project", "work-item:missing");

        initialize_workflow_state_at(&path).expect("state should exist");
        let result = update_task_package_draft_fields_at(&path, &request);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("还没有本地 workflow"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_fields_update_rejects_missing_work_item() {
        let dir = std::env::temp_dir().join(format!(
            "task-fields-missing-work-item-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let request = fixture_fields_update_request(&project.project_root, "work-item:missing");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let result = update_task_package_draft_fields_at(&path, &request);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("找不到该 work item"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_fields_update_rejects_missing_task_package_artifact() {
        let dir = std::env::temp_dir().join(format!(
            "task-fields-missing-artifact-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let mut value = read_json_file(&path);
        let workflow_id = default_workflow_id(&project.project_root);
        let work_item_id = format!("work-item:{workflow_id}:manual");
        array_mut(&mut value, "work_items")
            .expect("work_items should exist")
            .push(json!({
              "work_item_id": work_item_id,
              "project_id": project_id(&project.project_root),
              "workflow_id": workflow_id,
              "title": "没有 artifact 的草稿",
              "state": "draft",
              "source_kind": "workspace_state",
              "source_ref": "artifact:missing"
            }));
        atomic_write_json(&path, &value).expect("fixture should write");
        let request = fixture_fields_update_request(&project.project_root, &work_item_id);

        let result = update_task_package_draft_fields_at(&path, &request);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("找不到 task_package artifact"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_fields_update_writes_structured_fields_backup_and_audit() {
        let dir =
            std::env::temp_dir().join(format!("task-fields-update-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let draft_request = fixture_task_draft_request(&project.project_root, "旧标题");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &draft_request).expect("task draft should be created");
        let before_text = fs::read_to_string(&path).expect("state should be readable");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        let request = fixture_fields_update_request(&project.project_root, &work_item_id);

        let result = update_task_package_draft_fields_at(&path, &request)
            .expect("fields update should write");
        let backup_path = result
            .backup_path
            .expect("fields update should back up old state");
        assert!(PathBuf::from(&backup_path).exists());
        let backup_text = fs::read_to_string(backup_path).expect("backup should be readable");
        assert_eq!(backup_text, before_text);

        let updated = read_json_file(&path);
        assert_eq!(updated["work_items"][0]["title"], "字段编辑任务");
        assert_eq!(updated["work_items"][0]["assigned_role_id"], "desktop-app");
        assert_eq!(updated["artifacts"][0]["task_name"], "字段编辑任务");
        assert_eq!(updated["artifacts"][0]["assigned_line"], "桌面应用线");
        assert_eq!(
            updated["artifacts"][0]["template_version"],
            "task_package_v1"
        );
        assert_eq!(updated["artifacts"][0]["path"], Value::Null);
        assert_eq!(updated["artifacts"][0]["background"][0], "来自结构化字段。");
        assert!(updated["audit_events"]
            .as_array()
            .expect("audit events should be array")
            .iter()
            .any(|event| event["event_type"] == "task_package_fields_updated"));

        let preview_request = fixture_task_preview_request(&project.project_root, &work_item_id);
        let preview = render_task_package_preview_at(&path, &project, &preview_request)
            .expect("preview should render updated fields");
        assert!(preview.markdown.contains("# 任务包：字段编辑任务"));
        assert!(preview.markdown.contains("桌面应用线"));
        assert!(preview.markdown.contains("- 来自结构化字段。"));
        assert!(preview.markdown.contains("- 完成字段编辑。"));
        assert!(preview.markdown.contains("- /tmp/indexed-project"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_fields_update_keeps_empty_fields_as_missing_facts() {
        let dir =
            std::env::temp_dir().join(format!("task-fields-empty-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let draft_request = fixture_task_draft_request(&project.project_root, "旧标题");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &draft_request).expect("task draft should be created");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        let request = empty_fields_update_request(&project.project_root, &work_item_id);

        update_task_package_draft_fields_at(&path, &request)
            .expect("empty fields should still save");
        let updated = read_json_file(&path);
        assert_eq!(updated["artifacts"][0]["task_name"], "");
        assert_eq!(
            updated["artifacts"][0]["background"]
                .as_array()
                .unwrap()
                .len(),
            0
        );
        assert!(updated["artifacts"][0]["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|warning| warning == "missing_task_name"));

        let preview_request = fixture_task_preview_request(&project.project_root, &work_item_id);
        let preview = render_task_package_preview_at(&path, &project, &preview_request)
            .expect("preview should render placeholders");
        assert!(preview.markdown.contains("# 任务包：待补充"));
        assert!(preview.markdown.contains("未登记"));
        assert!(preview.markdown.contains("待补充"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_file_generation_rejects_non_index_project() {
        let dir =
            std::env::temp_dir().join(format!("task-file-non-index-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let tasks_dir = dir.join("tasks");
        let index = json!({
          "projects": [{ "project_root": "/tmp/indexed-project" }]
        });
        let request = fixture_task_file_generation_request("/tmp/not-indexed", "work-item:missing");

        initialize_workflow_state_at(&path).expect("state should exist");
        let result =
            generate_task_package_file_for_index_project_at(&path, &index, &request, &tasks_dir);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("项目不在当前索引内"));
        assert!(!tasks_dir.exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_file_generation_rejects_missing_state_file() {
        let dir = std::env::temp_dir().join(format!(
            "task-file-missing-state-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let tasks_dir = dir.join("tasks");
        let project = fixture_project("/tmp/indexed-project");
        let request =
            fixture_task_file_generation_request(&project.project_root, "work-item:missing");

        let result = generate_task_package_file_at(&path, &project, &request, &tasks_dir);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("工作流状态文件不存在"));
        assert!(!path.exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_file_generation_rejects_missing_workflow() {
        let dir = std::env::temp_dir().join(format!(
            "task-file-missing-workflow-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let tasks_dir = dir.join("tasks");
        let project = fixture_project("/tmp/indexed-project");
        let request =
            fixture_task_file_generation_request(&project.project_root, "work-item:missing");

        initialize_workflow_state_at(&path).expect("state should exist");
        let result = generate_task_package_file_at(&path, &project, &request, &tasks_dir);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("还没有本地 workflow"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_file_generation_rejects_missing_work_item() {
        let dir = std::env::temp_dir().join(format!(
            "task-file-missing-work-item-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let tasks_dir = dir.join("tasks");
        let project = fixture_project("/tmp/indexed-project");
        let request =
            fixture_task_file_generation_request(&project.project_root, "work-item:missing");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let result = generate_task_package_file_at(&path, &project, &request, &tasks_dir);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("找不到该 work item"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_file_generation_rejects_missing_task_package_artifact() {
        let dir = std::env::temp_dir().join(format!(
            "task-file-missing-artifact-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let tasks_dir = dir.join("tasks");
        let project = fixture_project("/tmp/indexed-project");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let mut value = read_json_file(&path);
        let workflow_id = default_workflow_id(&project.project_root);
        let work_item_id = format!("work-item:{workflow_id}:manual");
        array_mut(&mut value, "work_items")
            .expect("work_items should exist")
            .push(json!({
              "work_item_id": work_item_id,
              "project_id": project_id(&project.project_root),
              "workflow_id": workflow_id,
              "title": "没有 artifact 的草稿",
              "state": "draft",
              "source_kind": "workspace_state",
              "source_ref": "artifact:missing"
            }));
        atomic_write_json(&path, &value).expect("fixture should write");
        let request = fixture_task_file_generation_request(&project.project_root, &work_item_id);

        let result = generate_task_package_file_at(&path, &project, &request, &tasks_dir);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("找不到 task_package artifact"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_file_generation_writes_file_updates_artifact_and_audit() {
        let dir =
            std::env::temp_dir().join(format!("task-file-generate-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let tasks_dir = dir.join("tasks");
        let project = fixture_project("/tmp/indexed-project");
        let draft_request = fixture_task_draft_request(&project.project_root, "旧标题");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &draft_request).expect("task draft should be created");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        let fields_request = fixture_fields_update_request(&project.project_root, &work_item_id);
        update_task_package_draft_fields_at(&path, &fields_request)
            .expect("fields should be saved before generation");
        let before_text = fs::read_to_string(&path).expect("state should be readable");
        let request = fixture_task_file_generation_request(&project.project_root, &work_item_id);

        let result = generate_task_package_file_at(&path, &project, &request, &tasks_dir)
            .expect("file generation should write");

        let file_path = PathBuf::from(&result.file_path);
        assert!(file_path.exists());
        assert!(file_path.starts_with(&tasks_dir));
        assert!(file_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("2026-05-29-generated-"));
        let markdown = fs::read_to_string(&file_path).expect("generated file should be readable");
        assert!(markdown.contains("# 任务包：字段编辑任务"));
        assert!(markdown.contains("## 目标"));
        assert!(markdown.contains("- 完成字段编辑。"));
        assert!(markdown.contains("## 禁止事项"));
        assert!(markdown.contains("- 不生成真实任务文件。"));
        assert!(markdown.contains("待补充") || !markdown.contains("{{"));

        let updated = read_json_file(&path);
        assert_eq!(updated["artifacts"][0]["path"], result.file_path);
        assert!(updated["artifacts"][0]["updated_at"].as_str().is_some());
        assert!(!updated["artifacts"][0]["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|warning| warning == "draft_only_no_markdown_file"));
        assert!(updated["audit_events"]
            .as_array()
            .expect("audit events should be array")
            .iter()
            .any(|event| event["event_type"] == "task_package_file_generated"
                && event["target_ref"] == work_item_id));
        let backup_text =
            fs::read_to_string(&result.backup_path).expect("backup should be readable");
        assert_eq!(backup_text, before_text);
        assert_eq!(
            result.snapshot.project_workflows[0].task_drafts[0]
                .artifact_path
                .as_deref(),
            Some(result.file_path.as_str())
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_file_generation_uses_suffix_without_overwriting_existing_file() {
        let dir =
            std::env::temp_dir().join(format!("task-file-conflict-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let tasks_dir = dir.join("tasks");
        let project = fixture_project("/tmp/indexed-project");
        let draft_request = fixture_task_draft_request(&project.project_root, "旧标题");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &draft_request).expect("task draft should be created");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        let fields_request = fixture_fields_update_request(&project.project_root, &work_item_id);
        update_task_package_draft_fields_at(&path, &fields_request)
            .expect("fields should be saved before generation");

        fs::create_dir_all(&tasks_dir).expect("tasks fixture dir should exist");
        let conflict = next_available_task_package_path(&tasks_dir, "字段编辑任务", &work_item_id)
            .expect("first generated path should be calculable");
        fs::write(&conflict, "existing file").expect("conflict fixture should write");
        let request = fixture_task_file_generation_request(&project.project_root, &work_item_id);

        let result = generate_task_package_file_at(&path, &project, &request, &tasks_dir)
            .expect("file generation should use suffix");

        assert_eq!(
            fs::read_to_string(&conflict).expect("conflict file should remain"),
            "existing file"
        );
        assert!(result.file_path.ends_with("-2.md"));
        assert_ne!(result.file_path, conflict.display().to_string());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_file_generation_keeps_missing_fields_as_placeholders() {
        let dir = std::env::temp_dir().join(format!(
            "task-file-placeholders-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let tasks_dir = dir.join("tasks");
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
              "source_ref": work_item_id,
              "warnings": ["missing_task_name"]
            }));
        atomic_write_json(&path, &value).expect("fixture should write");
        let request = fixture_task_file_generation_request(&project.project_root, &work_item_id);

        let result = generate_task_package_file_at(&path, &project, &request, &tasks_dir)
            .expect("file generation should keep placeholders");
        let markdown = fs::read_to_string(result.file_path).expect("generated file should read");

        assert!(markdown.contains("# 任务包：待补充"));
        assert!(markdown.contains("未登记"));
        assert!(markdown.contains("业务背景：待补充"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_dispatch_readiness_flags_polluted_generated_draft_as_not_ready() {
        let dir =
            std::env::temp_dir().join(format!("dispatch-polluted-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let tasks_dir = dir.join("tasks");
        let draft_request = TaskDraftRequest {
            project_root: project.project_root.clone(),
            title: "task draft他日smoke".to_string(),
            objective: "待补充：输入法污染他日".to_string(),
            assigned_role: Some("codex-dev".to_string()),
        };

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_active_plan_authorization_for_fixture(&path, &project.project_root);
        create_task_draft_at(&path, &draft_request).expect("task draft should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        generate_task_package_file_at(
            &path,
            &project,
            &fixture_task_file_generation_request(&project.project_root, &work_item_id),
            &tasks_dir,
        )
        .expect("polluted fixture file should generate");
        let readiness = inspect_task_package_dispatch_readiness_at(
            &path,
            &project,
            &fixture_dispatch_readiness_request(&project.project_root, &work_item_id),
        )
        .expect("readiness should inspect");

        assert_eq!(readiness.status, "not_ready");
        assert!(!readiness.can_generate_next_version);
        assert!(readiness
            .blocking_reasons
            .iter()
            .any(|reason| reason.contains("测试草稿")));
        assert!(readiness
            .blocking_reasons
            .iter()
            .any(|reason| reason.contains("目标")));
        assert!(readiness
            .blocking_reasons
            .iter()
            .any(|reason| reason.contains("历史禁令")));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_dispatch_readiness_rejects_missing_fields() {
        let dir =
            std::env::temp_dir().join(format!("dispatch-missing-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let draft_request = fixture_task_draft_request(&project.project_root, "待补充");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &draft_request).expect("task draft should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        update_task_package_draft_fields_at(
            &path,
            &empty_fields_update_request(&project.project_root, &work_item_id),
        )
        .expect("empty fields should save");
        let readiness = inspect_task_package_dispatch_readiness_at(
            &path,
            &project,
            &fixture_dispatch_readiness_request(&project.project_root, &work_item_id),
        )
        .expect("readiness should inspect");

        assert_eq!(readiness.status, "not_ready");
        assert!(readiness
            .blocking_reasons
            .iter()
            .any(|reason| reason.contains("尚未生成")));
        assert!(readiness
            .blocking_reasons
            .iter()
            .any(|reason| reason.contains("允许写入")));
        assert!(readiness
            .blocking_reasons
            .iter()
            .any(|reason| reason.contains("验收标准")));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_dispatch_readiness_rejects_conflicting_generation_ban() {
        let dir = std::env::temp_dir().join(format!(
            "dispatch-conflicting-ban-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let tasks_dir = dir.join("tasks");
        let draft_request = fixture_task_draft_request(&project.project_root, "旧标题");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_active_plan_authorization_for_fixture(&path, &project.project_root);
        create_task_draft_at(&path, &draft_request).expect("task draft should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        update_task_package_draft_fields_at(
            &path,
            &ready_fields_update_request_with_forbidden(
                &project.project_root,
                &work_item_id,
                vec!["不生成真实任务包文件。".to_string()],
            ),
        )
        .expect("fields should save");
        generate_task_package_file_at(
            &path,
            &project,
            &fixture_task_file_generation_request(&project.project_root, &work_item_id),
            &tasks_dir,
        )
        .expect("file should generate");
        let readiness = inspect_task_package_dispatch_readiness_at(
            &path,
            &project,
            &fixture_dispatch_readiness_request(&project.project_root, &work_item_id),
        )
        .expect("readiness should inspect");

        assert_eq!(readiness.status, "not_ready");
        assert!(readiness
            .blocking_reasons
            .iter()
            .any(|reason| reason.contains("历史禁令")));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_dispatch_readiness_can_be_ready_after_field_fix_and_file_generation() {
        let dir = std::env::temp_dir().join(format!("dispatch-ready-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let tasks_dir = dir.join("tasks");
        let draft_request = fixture_task_draft_request(&project.project_root, "旧标题");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_active_plan_authorization_for_fixture(&path, &project.project_root);
        create_task_draft_at(&path, &draft_request).expect("task draft should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        update_task_package_draft_fields_at(
            &path,
            &ready_fields_update_request(&project.project_root, &work_item_id),
        )
        .expect("fields should save");
        mark_task_package_fixture_ready(&path, "codex-test-model");
        let first = generate_task_package_file_at(
            &path,
            &project,
            &fixture_task_file_generation_request(&project.project_root, &work_item_id),
            &tasks_dir,
        )
        .expect("file should generate");
        let readiness = inspect_task_package_dispatch_readiness_at(
            &path,
            &project,
            &fixture_dispatch_readiness_request(&project.project_root, &work_item_id),
        )
        .expect("readiness should inspect");

        assert_eq!(
            readiness.status, "ready",
            "{:?}",
            readiness.blocking_reasons
        );
        assert!(readiness.can_generate_next_version);
        assert!(readiness.blocking_reasons.is_empty());
        assert_eq!(
            readiness.artifact_path.as_deref(),
            Some(first.file_path.as_str())
        );

        let mut next_fields = ready_fields_update_request(&project.project_root, &work_item_id);
        next_fields.fields.task_name = "派发准备检查任务新版".to_string();
        next_fields.fields.goals = vec!["生成修正后的可派发版本。".to_string()];
        update_task_package_draft_fields_at(&path, &next_fields).expect("next fields should save");
        mark_task_package_fixture_ready(&path, "codex-test-model");
        let second = generate_task_package_file_at(
            &path,
            &project,
            &fixture_task_file_generation_request(&project.project_root, &work_item_id),
            &tasks_dir,
        )
        .expect("next file should not overwrite old file");
        assert_ne!(first.file_path, second.file_path);
        assert!(PathBuf::from(&first.file_path).exists());
        assert!(PathBuf::from(&second.file_path).exists());
        let updated = read_json_file(&path);
        assert_eq!(updated["artifacts"][0]["path"], second.file_path);
        assert!(updated["audit_events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["event_type"] == "task_package_file_generated"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn dispatch_field_correction_rejects_non_index_project() {
        let dir =
            std::env::temp_dir().join(format!("correction-non-index-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let index = json!({
          "projects": [{ "project_root": "/tmp/indexed-project" }]
        });
        let request = fixture_dispatch_correction_request("/tmp/not-indexed", "work-item:missing");

        initialize_workflow_state_at(&path).expect("state should exist");
        let result =
            correct_task_package_dispatch_fields_for_index_project_at(&path, &index, &request);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("项目不在当前索引内"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn dispatch_field_correction_rejects_missing_state_file() {
        let dir = std::env::temp_dir().join(format!(
            "correction-missing-state-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let request =
            fixture_dispatch_correction_request("/tmp/indexed-project", "work-item:missing");
        let update_request = TaskPackageFieldsUpdateRequest {
            project_root: request.project_root,
            work_item_id: request.work_item_id,
            fields: request.fields,
        };

        let result = update_task_package_fields_at(
            &path,
            &update_request,
            TaskPackageFieldWriteMode::DispatchCorrection,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("工作流状态文件不存在"));
        assert!(!path.exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn dispatch_field_correction_rejects_missing_workflow_work_item_and_artifact() {
        let dir = std::env::temp_dir().join(format!(
            "correction-missing-parts-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let missing_work_item =
            fixture_dispatch_correction_request(&project.project_root, "work-item:missing");
        let missing_work_item_update = TaskPackageFieldsUpdateRequest {
            project_root: missing_work_item.project_root.clone(),
            work_item_id: missing_work_item.work_item_id.clone(),
            fields: missing_work_item.fields.clone(),
        };

        initialize_workflow_state_at(&path).expect("state should exist");
        let missing_workflow = update_task_package_fields_at(
            &path,
            &missing_work_item_update,
            TaskPackageFieldWriteMode::DispatchCorrection,
        );
        assert!(missing_workflow.is_err());
        assert!(missing_workflow
            .unwrap_err()
            .contains("还没有本地 workflow"));

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let missing_item = update_task_package_fields_at(
            &path,
            &missing_work_item_update,
            TaskPackageFieldWriteMode::DispatchCorrection,
        );
        assert!(missing_item.is_err());
        assert!(missing_item.unwrap_err().contains("找不到该 work item"));

        let mut value = read_json_file(&path);
        let workflow_id = default_workflow_id(&project.project_root);
        let work_item_id = format!("work-item:{workflow_id}:manual");
        array_mut(&mut value, "work_items")
            .expect("work_items should exist")
            .push(json!({
              "work_item_id": work_item_id,
              "project_id": project_id(&project.project_root),
              "workflow_id": workflow_id,
              "title": "没有 artifact 的草稿",
              "state": "draft",
              "source_kind": "workspace_state",
              "source_ref": "artifact:missing"
            }));
        atomic_write_json(&path, &value).expect("fixture should write");
        let missing_artifact_request =
            fixture_dispatch_correction_request(&project.project_root, &work_item_id);
        let missing_artifact_update = TaskPackageFieldsUpdateRequest {
            project_root: missing_artifact_request.project_root,
            work_item_id: missing_artifact_request.work_item_id,
            fields: missing_artifact_request.fields,
        };
        let missing_artifact = update_task_package_fields_at(
            &path,
            &missing_artifact_update,
            TaskPackageFieldWriteMode::DispatchCorrection,
        );
        assert!(missing_artifact.is_err());
        assert!(missing_artifact
            .unwrap_err()
            .contains("找不到 task_package artifact"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn dispatch_field_correction_backs_up_writes_audit_keeps_path_and_rechecks_ready() {
        let dir = std::env::temp_dir().join(format!("correction-save-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let tasks_dir = dir.join("tasks");
        let project = fixture_project("/tmp/indexed-project");
        let draft_request = fixture_task_draft_request(&project.project_root, "旧标题");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_active_plan_authorization_for_fixture(&path, &project.project_root);
        create_task_draft_at(&path, &draft_request).expect("task draft should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        generate_task_package_file_at(
            &path,
            &project,
            &fixture_task_file_generation_request(&project.project_root, &work_item_id),
            &tasks_dir,
        )
        .expect("existing generated path should exist");
        mark_task_package_fixture_ready(&path, "codex-test-model");
        let before_text = fs::read_to_string(&path).expect("state should be readable");
        let before = read_json_file(&path);
        let old_path =
            optional_string_from(&before["artifacts"][0], "path").expect("path should exist");
        let request = fixture_dispatch_correction_request(&project.project_root, &work_item_id);
        let update_request = TaskPackageFieldsUpdateRequest {
            project_root: request.project_root,
            work_item_id: request.work_item_id,
            fields: request.fields,
        };

        let result = update_task_package_fields_at(
            &path,
            &update_request,
            TaskPackageFieldWriteMode::DispatchCorrection,
        )
        .expect("correction should save");
        let backup_text =
            fs::read_to_string(&result.backup_path.unwrap()).expect("backup should be readable");
        assert_eq!(backup_text, before_text);

        let updated = read_json_file(&path);
        assert_eq!(updated["artifacts"][0]["path"], old_path);
        assert_eq!(updated["artifacts"][0]["task_name"], "派发准备检查任务");
        updated["artifacts"][0]["model_id"]
            .as_str()
            .expect("fixture model should remain explicit");
        assert!(updated["audit_events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["event_type"] == "task_package_fields_corrected_for_dispatch"));

        let readiness = inspect_task_package_dispatch_readiness_at(
            &path,
            &project,
            &fixture_dispatch_readiness_request(&project.project_root, &work_item_id),
        )
        .expect("readiness should inspect after save");
        assert_eq!(readiness.status, "not_ready");
        assert!(readiness
            .blocking_reasons
            .iter()
            .any(|reason| reason.contains("stale")));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn dispatch_field_correction_keeps_empty_fields_missing() {
        let dir =
            std::env::temp_dir().join(format!("correction-empty-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let draft_request = fixture_task_draft_request(&project.project_root, "旧标题");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &draft_request).expect("task draft should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        let empty_update = empty_fields_update_request(&project.project_root, &work_item_id);
        update_task_package_fields_at(
            &path,
            &empty_update,
            TaskPackageFieldWriteMode::DispatchCorrection,
        )
        .expect("empty correction should save without inventing");

        let updated = read_json_file(&path);
        assert_eq!(updated["artifacts"][0]["task_name"], "");
        assert!(updated["artifacts"][0]["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|warning| warning == "missing_task_name"));
        let readiness = inspect_task_package_dispatch_readiness_at(
            &path,
            &project,
            &fixture_dispatch_readiness_request(&project.project_root, &work_item_id),
        )
        .expect("readiness should inspect after empty save");
        assert_eq!(readiness.status, "not_ready");

        let _ = fs::remove_dir_all(dir);
    }
