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
    fn project_blackboard_derives_node_less_canonical_chain_process_messages_without_cross_workflow_leakage() {
        let dir = std::env::temp_dir().join(format!(
            "p3-a-chain-events-blackboard-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project_a = fixture_project("/tmp/p3-a-chain-events-project-a");
        let project_b = fixture_project("/tmp/p3-a-chain-events-project-b");
        let workflow_a = default_workflow_id(&project_a.project_root);
        let workflow_b = default_workflow_id(&project_b.project_root);

        bootstrap_project_workflow_at(&path, &project_a).expect("first workflow should exist");
        bootstrap_project_workflow_at(&path, &project_b).expect("second workflow should exist");

        let canonical_events = [
            (
                "run-started",
                "workflow_chain_run_started",
                "主管进度 / 开跑",
                "我开跑了，任务已经排好队。",
            ),
            (
                "node-started",
                "workflow_chain_node_started",
                "主管进度 / 开始处理",
                "我在做下一件事了。",
            ),
            (
                "node-completed",
                "workflow_chain_node_completed",
                "主管进度 / 一项完成",
                "这一件做完了。",
            ),
            (
                "waiting-decision",
                "workflow_chain_node_waiting_decision",
                "主管进度 / 等待你",
                "我先停在这儿了——worker 有话想问你。",
            ),
            (
                "needs-rework",
                "workflow_chain_node_needs_rework",
                "主管进度 / 需要返工",
                "这一件要回去再做一遍。",
            ),
            (
                "run-completed",
                "workflow_chain_run_completed",
                "主管进度 / 已完成",
                "都干完了，结果放你右手边。",
            ),
            (
                "run-stopped",
                "workflow_chain_run_stopped",
                "主管进度 / 已中断",
                "这轮先停下来了，原因在右边。",
            ),
        ];
        let mut value = read_json_file(&path);
        let audit_events = value["audit_events"]
            .as_array_mut()
            .expect("audit events should be an array");
        for (index, (suffix, event_type, _, _)) in canonical_events.iter().enumerate() {
            let created_at = if index < 2 {
                "2026-07-18T00:00:00Z".to_string()
            } else {
                format!("2026-07-18T00:00:{index:02}Z")
            };
            audit_events.push(json!({
                "event_id": format!("audit:p3-a:a:{suffix}"),
                "event_type": event_type,
                "workflow_id": workflow_a.clone(),
                "target_ref": "chain-run:p3-a",
                "actor_ref": "project_director",
                "reason": format!("MACHINE_REASON_P3_A_{suffix}"),
                "created_at": created_at,
            }));
        }
        audit_events.push(json!({
            "event_id": "audit:p3-a:a:noncanonical",
            "event_type": "workflow_chain_node_director_deterministic_completed",
            "workflow_id": workflow_a.clone(),
            "target_ref": "chain-run:p3-a",
            "actor_ref": "project_director",
            "reason": "MACHINE_REASON_P3_A_noncanonical",
            "created_at": "2026-07-18T00:01:00Z",
        }));
        audit_events.push(json!({
            "event_id": "audit:p3-a:b:run-started",
            "event_type": "workflow_chain_run_started",
            "workflow_id": workflow_b.clone(),
            "target_ref": "chain-run:p3-b",
            "actor_ref": "project_director",
            "reason": "MACHINE_REASON_P3_A_B",
            "created_at": "2026-07-18T00:02:00Z",
        }));
        audit_events.push(json!({
            "event_id": "audit:p3-a:b:node-failed",
            "event_type": "workflow_chain_node_failed",
            "workflow_id": workflow_b.clone(),
            "target_ref": "chain-run:p3-b",
            "actor_ref": "project_director",
            "reason": "MACHINE_REASON_P3_A_B_NODE_FAILED",
            "created_at": "2026-07-18T00:02:01Z",
        }));
        audit_events.push(json!({
            "event_id": "audit:p3-a:b:run-failed",
            "event_type": "workflow_chain_run_failed",
            "workflow_id": workflow_b.clone(),
            "target_ref": "chain-run:p3-b",
            "actor_ref": "project_director",
            "reason": "MACHINE_REASON_P3_A_B_RUN_FAILED",
            "created_at": "2026-07-18T00:02:02Z",
        }));
        write_validated_workflow_state(&path, &value).expect("chain event fixture should write");
        let expected_fact_state = read_json_file(&path);

        let snapshot = read_workflow_state_snapshot(&path).expect("snapshot should read");
        assert_eq!(
            read_json_file(&path),
            expected_fact_state,
            "blackboard derive must not write facts or audit records"
        );

        let blackboard_a = snapshot
            .project_blackboards
            .iter()
            .find(|blackboard| blackboard.workflow_id == workflow_a)
            .expect("workflow A blackboard should exist");
        let mut process_entries_a = blackboard_a
            .entries
            .iter()
            .filter(|entry| {
                entry.kind == BlackboardEntryKind::SupervisorMessage
                    && entry
                        .source_refs
                        .iter()
                        .any(|source| source.source_kind == "workflow_chain_event")
            })
            .collect::<Vec<_>>();
        process_entries_a.sort_by(|left, right| left.created_at.cmp(&right.created_at));

        assert_eq!(
            process_entries_a.len(),
            canonical_events.len(),
            "one message per canonical audit event"
        );
        for (index, entry) in process_entries_a.iter().enumerate() {
            let (suffix, event_type, expected_title, expected_summary) = canonical_events[index];
            assert_eq!(entry.title, expected_title);
            assert_eq!(entry.summary, expected_summary);
            assert_eq!(entry.status, "reported");
            assert_eq!(entry.source_status.as_deref(), Some(event_type));
            assert_eq!(entry.question_id, None);
            assert_eq!(
                entry.workflow_node_id, None,
                "production chain audit shape is node-less, so this message can only focus the graph"
            );
            assert_eq!(entry.source_refs.len(), 1);
            assert_eq!(entry.source_refs[0].source_kind, "workflow_chain_event");
            assert_eq!(
                entry.source_refs[0].source_id,
                format!("audit:p3-a:a:{suffix}")
            );
            assert!(!entry.summary.contains("MACHINE_REASON_P3_A"));
            assert_eq!(entry.promotion_decision.status, "not_applicable");
            assert!(entry
                .warnings
                .contains(&"supervisor_message_is_read_model_only".to_string()));
            assert!(entry
                .warnings
                .contains(&"supervisor_message_does_not_advance_workflow".to_string()));
        }
        assert!(process_entries_a.iter().all(|entry| {
            entry.source_refs[0].source_id != "audit:p3-a:a:noncanonical"
        }));

        let derived_a = snapshot
            .project_workflows
            .iter()
            .find(|workflow| workflow.workflow_id == workflow_a)
            .and_then(|workflow| workflow.derived_workflow.as_ref())
            .expect("workflow A read model should exist");
        assert!(derived_a.ledger_entries.iter().any(|entry| {
            entry.audit_refs == vec!["audit:p3-a:a:run-started".to_string()]
                && entry.summary == "MACHINE_REASON_P3_A_run-started"
        }));
        let tied_process_entries = process_entries_a
            .iter()
            .filter(|entry| entry.created_at.as_deref() == Some("2026-07-18T00:00:00Z"))
            .collect::<Vec<_>>();
        assert_eq!(
            tied_process_entries
                .iter()
                .map(|entry| entry.source_refs[0].source_id.as_str())
                .collect::<Vec<_>>(),
            vec!["audit:p3-a:a:run-started", "audit:p3-a:a:node-started"],
            "same-created_at process messages must keep their audit ledger order"
        );
        for entry in tied_process_entries {
            let audit_id = &entry.source_refs[0].source_id;
            let ledger_entry_ordinal = derived_a
                .ledger_entries
                .iter()
                .position(|ledger_entry| ledger_entry.ledger_entry_id == *audit_id)
                .expect("process source audit should retain its ledger ordinal");
            assert_eq!(
                entry.entry_id,
                format!(
                    "blackboard:{workflow_a}:supervisor-process:{ledger_entry_ordinal:08}:{}",
                    stable_id(audit_id)
                )
            );
        }

        let blackboard_b = snapshot
            .project_blackboards
            .iter()
            .find(|blackboard| blackboard.workflow_id == workflow_b)
            .expect("workflow B blackboard should exist");
        let mut process_entries_b = blackboard_b
            .entries
            .iter()
            .filter(|entry| {
                entry.kind == BlackboardEntryKind::SupervisorMessage
                    && entry
                        .source_refs
                        .iter()
                        .any(|source| source.source_kind == "workflow_chain_event")
            })
            .collect::<Vec<_>>();
        process_entries_b.sort_by(|left, right| left.created_at.cmp(&right.created_at));
        assert_eq!(
            process_entries_b.len(),
            2,
            "workflow B must not receive A events or duplicate node_failed"
        );
        assert_eq!(
            process_entries_b[0].source_refs[0].source_id,
            "audit:p3-a:b:run-started"
        );
        assert_eq!(process_entries_b[0].summary, "我开跑了，任务已经排好队。");
        assert_eq!(
            process_entries_b[1].source_refs[0].source_id,
            "audit:p3-a:b:run-failed"
        );
        assert_eq!(process_entries_b[1].title, "主管进度 / 已中断");
        assert_eq!(process_entries_b[1].summary, "这轮先停下来了，原因在右边。");
        assert!(process_entries_b.iter().all(|entry| {
            entry.source_refs[0].source_id != "audit:p3-a:b:node-failed"
        }));

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
    fn workflow_node_session_binding_ids_do_not_collide_on_long_work_item_prefixes() {
        let shared_prefix = "work-item:workflow:users-yoyi-codex-workflow-mario-test:default:project-director:planned-task-supervisor-pilot-";
        let workflow_id = "workflow:users-yoyi-codex-workflow-mario-test:default";
        let node_id = format!("{workflow_id}:node:project-director");
        let first = workflow_node_session_binding_id(
            workflow_id,
            &node_id,
            Some(&format!("{shared_prefix}first")),
            "thread-first",
        );
        let second = workflow_node_session_binding_id(
            workflow_id,
            &node_id,
            Some(&format!("{shared_prefix}second")),
            "thread-second",
        );

        assert_ne!(first, second);
        assert!(first.starts_with("binding:sha256:"));
        assert_eq!(first.len(), "binding:sha256:".len() + 64);
    }

    #[test]
    fn workflow_node_session_binding_migration_repairs_duplicate_legacy_ids() {
        let shared_prefix = "work-item:workflow:users-yoyi-codex-workflow-mario-test:default:project-director:planned-task-supervisor-pilot-";
        let first_work_item = format!("{shared_prefix}first");
        let second_work_item = format!("{shared_prefix}second");
        let legacy_id = legacy_workflow_node_session_binding_id(
            "workflow:long:default",
            "workflow:long:default:node:project-director",
            Some(&first_work_item),
        );
        let mut value = json!({
            "schema_version": "workflow_state_v0",
            "workflow_version": 1,
            "projects": [],
            "agent_adapters": [],
            "workflows": [],
            "nodes": [],
            "edges": [],
            "work_items": [],
            "artifacts": [],
            "reviews": [],
            "audit_events": [],
            "capabilities": [],
            "harness_resources": [],
            "workflow_node_session_bindings": [
                {
                    "binding_id": legacy_id,
                    "workflow_id": "workflow:long:default",
                    "node_id": "workflow:long:default:node:project-director",
                    "work_item_id": first_work_item,
                    "native_thread_id": "thread-first"
                },
                {
                    "binding_id": legacy_id,
                    "workflow_id": "workflow:long:default",
                    "node_id": "workflow:long:default:node:project-director",
                    "work_item_id": second_work_item,
                    "native_thread_id": "thread-second"
                }
            ],
            "workflow_node_dispatches": [
                {
                    "dispatch_id": "dispatch-first",
                    "binding_id": legacy_id,
                    "workflow_id": "workflow:long:default",
                    "node_id": "workflow:long:default:node:project-director",
                    "work_item_id": first_work_item,
                    "native_thread_id": "thread-first"
                },
                {
                    "dispatch_id": "dispatch-second",
                    "binding_id": legacy_id,
                    "workflow_id": "workflow:long:default",
                    "node_id": "workflow:long:default:node:project-director",
                    "work_item_id": second_work_item,
                    "native_thread_id": "thread-second"
                }
            ]
        });

        assert!(validate_workflow_state(&value)
            .iter()
            .any(|warning| warning.contains("重复 binding_id")));
        assert_eq!(
            migrate_legacy_workflow_node_session_binding_ids(&mut value),
            WorkflowBindingIdMigrationCounts {
                bindings: 2,
                dispatches: 2,
                unresolved_dispatches: 0,
            }
        );
        assert!(validate_workflow_state(&value).is_empty());
        assert_ne!(
            value["workflow_node_session_bindings"][0]["binding_id"],
            value["workflow_node_session_bindings"][1]["binding_id"]
        );
        assert_eq!(
            value["workflow_node_dispatches"][0]["binding_id"],
            value["workflow_node_session_bindings"][0]["binding_id"]
        );
        assert_eq!(
            value["workflow_node_dispatches"][1]["binding_id"],
            value["workflow_node_session_bindings"][1]["binding_id"]
        );

        value["workflow_node_dispatches"][0]["binding_id"] = Value::String(legacy_id.clone());
        value["workflow_node_dispatches"][1]
            .as_object_mut()
            .expect("dispatch object")
            .remove("binding_id");
        assert_eq!(
            migrate_legacy_workflow_node_session_binding_ids(&mut value),
            WorkflowBindingIdMigrationCounts {
                bindings: 0,
                dispatches: 2,
                unresolved_dispatches: 0,
            }
        );
        assert!(validate_workflow_state(&value).is_empty());
        assert_eq!(
            value["workflow_node_dispatches"][0]["binding_id"],
            value["workflow_node_session_bindings"][0]["binding_id"]
        );
        assert_eq!(
            value["workflow_node_dispatches"][1]["binding_id"],
            value["workflow_node_session_bindings"][1]["binding_id"]
        );

        let retained_history_id =
            "binding:sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        value["workflow_node_dispatches"][0]["binding_id"] =
            Value::String(retained_history_id.to_string());
        assert_eq!(
            migrate_legacy_workflow_node_session_binding_ids(&mut value),
            WorkflowBindingIdMigrationCounts::default()
        );
        assert_eq!(
            value["workflow_node_dispatches"][0]["binding_id"],
            retained_history_id
        );
    }

    #[test]
    fn workflow_binding_migration_counts_unresolvable_legacy_dispatch_reference() {
        let shared_prefix = "work-item:workflow:users-yoyi-codex-workflow-mario-test:default:project-director:planned-task-supervisor-pilot-";
        let first_work_item = format!("{shared_prefix}first");
        let second_work_item = format!("{shared_prefix}second");
        let missing_work_item = format!("{shared_prefix}missing");
        let legacy_id = legacy_workflow_node_session_binding_id(
            "workflow:long:default",
            "node:long",
            Some(&first_work_item),
        );
        let mut value = json!({
            "schema_version": "workflow_state_v0",
            "workflow_version": 1,
            "projects": [],
            "agent_adapters": [],
            "workflows": [],
            "nodes": [],
            "edges": [],
            "work_items": [],
            "artifacts": [],
            "reviews": [],
            "audit_events": [],
            "capabilities": [],
            "harness_resources": [],
            "workflow_node_session_bindings": [
                {
                    "binding_id": legacy_id,
                    "workflow_id": "workflow:long:default",
                    "node_id": "node:long",
                    "work_item_id": first_work_item,
                    "native_thread_id": "thread:first"
                },
                {
                    "binding_id": legacy_id,
                    "workflow_id": "workflow:long:default",
                    "node_id": "node:long",
                    "work_item_id": second_work_item,
                    "native_thread_id": "thread:second"
                }
            ],
            "workflow_node_dispatches": [{
                "binding_id": legacy_id,
                "workflow_id": "workflow:long:default",
                "node_id": "node:long",
                "work_item_id": missing_work_item
            }]
        });

        assert_eq!(
            migrate_legacy_workflow_node_session_binding_ids(&mut value),
            WorkflowBindingIdMigrationCounts {
                bindings: 2,
                dispatches: 0,
                unresolved_dispatches: 1,
            }
        );
        let dir = std::env::temp_dir().join(format!(
            "binding-migration-ambiguous-{}",
            unix_timestamp_string()
        ));
        fs::create_dir_all(&dir).expect("create migration fixture dir");
        let path = dir.join("workflow-state.v0.json");
        let before = serde_json::to_vec_pretty(&value).expect("serialize ambiguous fixture");
        fs::write(&path, &before).expect("write ambiguous fixture");

        let error = migrate_legacy_workflow_node_session_binding_ids_at(&path)
            .expect_err("ambiguous legacy dispatch must fail before write");
        assert!(error.contains("1 条旧 dispatch 引用无法唯一映射"));
        assert_eq!(fs::read(&path).expect("read rejected fixture"), before);
        assert!(!dir.join("backups").exists());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn supervisor_fresh_session_binding_records_control_core_provenance() {
        let dir = std::env::temp_dir().join(format!(
            "supervisor-fresh-binding-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let draft = fixture_task_draft_request(&project.project_root, "主管新会话绑定");
        bootstrap_project_workflow_at(&path, &project).expect("workflow");
        create_task_draft_at(&path, &draft).expect("work item");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id");
        let workflow_id = default_workflow_id(&project.project_root);
        let node_id = format!("{workflow_id}:node:codex-dev");
        let session = fixture_session("thread-supervisor-fresh", &project.project_root, true);
        let request = fixture_node_session_bind_request(
            &project.project_root,
            &node_id,
            Some(&work_item_id),
            &session.thread_id,
        );

        bind_workflow_node_codex_session_with_provenance_at(
            &path,
            &request,
            &session,
            &WorkflowNodeSessionBindingProvenance::fresh_task_session(
                "supervisor_orchestrator",
            ),
        )
        .expect("fresh binding");

        let state = read_json_file(&path);
        let binding = &state["workflow_node_session_bindings"][0];
        assert_eq!(binding["binding_mode"], "create_fresh_task_session");
        assert_eq!(binding["binding_source"], "fresh_task_session_bound");
        let event = state["audit_events"]
            .as_array()
            .expect("audit events")
            .iter()
            .rev()
            .find(|event| event["event_type"] == "workflow_node_session_bound")
            .expect("binding audit");
        assert_eq!(event["actor_ref"], "supervisor_orchestrator");
        assert_eq!(event["permission_level"], "authorized_supervisor_execution");
        assert!(event["reason"]
            .as_str()
            .expect("reason")
            .contains("全新 Codex 会话"));
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
        let state = read_json_file(&path);
        let audit_events = state["audit_events"]
            .as_array()
            .expect("blackboard fixture should retain audit events");
        assert_eq!(
            project_blackboards_from_workflows(&snapshot.project_workflows, audit_events),
            snapshot.project_blackboards
        );

        let _ = fs::remove_dir_all(dir);
    }
