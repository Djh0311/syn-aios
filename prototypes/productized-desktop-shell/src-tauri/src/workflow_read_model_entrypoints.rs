// Workflow read model, dispatch summary, and readback stats helpers split out during Root Treatment R2-B5.
// This file is included at crate root so helper visibility and behavior stay unchanged.

fn workflow_state_counts(value: &Value) -> WorkflowStateCounts {
    WorkflowStateCounts {
        projects: array_len(value, "projects"),
        agent_adapters: array_len(value, "agent_adapters"),
        workflows: array_len(value, "workflows"),
        nodes: array_len(value, "nodes"),
        edges: array_len(value, "edges"),
        work_items: array_len(value, "work_items"),
        artifacts: array_len(value, "artifacts"),
        reviews: array_len(value, "reviews"),
        audit_events: array_len(value, "audit_events"),
        capabilities: array_len(value, "capabilities"),
        harness_resources: array_len(value, "harness_resources"),
    }
}

fn empty_workflow_state_snapshot(path: &Path, warnings: Vec<String>) -> WorkflowStateSnapshot {
    WorkflowStateSnapshot {
        exists: false,
        path: path.display().to_string(),
        schema_version: None,
        workflow_version: None,
        workspace_id: None,
        updated_at: None,
        initialized: false,
        counts: WorkflowStateCounts {
            projects: 0,
            agent_adapters: 0,
            workflows: 0,
            nodes: 0,
            edges: 0,
            work_items: 0,
            artifacts: 0,
            reviews: 0,
            audit_events: 0,
            capabilities: 0,
            harness_resources: 0,
        },
        project_workflows: vec![],
        project_blackboards: vec![],
        warnings,
    }
}

fn project_workflow_summaries(value: &Value) -> Vec<ProjectWorkflowSummary> {
    let projects = value
        .get("projects")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let workflows = value
        .get("workflows")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let nodes = value
        .get("nodes")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let edges = value
        .get("edges")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let work_items = value
        .get("work_items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let artifacts = value
        .get("artifacts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let node_session_bindings = value
        .get("workflow_node_session_bindings")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let node_dispatches = value
        .get("workflow_node_dispatches")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let director_reviews = value
        .get("reviews")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let execution_controls = value
        .get("workflow_execution_controls")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let permission_requests = value
        .get("permission_requests")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let execution_attempts = value
        .get("execution_attempts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let audit_events = value
        .get("audit_events")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    workflows
        .iter()
        .filter_map(|workflow| {
            let workflow_id = optional_string_from(workflow, "workflow_id")?;
            let project_id = optional_string_from(workflow, "project_id")?;
            let project_root = projects
                .iter()
                .find(|project| {
                    optional_string_from(project, "project_id").as_deref()
                        == Some(project_id.as_str())
                })
                .and_then(|project| optional_string_from(project, "root_path"))
                .unwrap_or_default();
            Some(ProjectWorkflowSummary {
                task_draft_count: work_items
                    .iter()
                    .filter(|item| {
                        optional_string_from(item, "workflow_id").as_deref()
                            == Some(workflow_id.as_str())
                    })
                    .count(),
                task_drafts: task_draft_summaries(
                    &workflow_id,
                    &work_items,
                    &artifacts,
                    &audit_events,
                ),
                node_session_bindings: workflow_node_session_binding_summaries(
                    &workflow_id,
                    &node_session_bindings,
                ),
                node_dispatches: workflow_node_dispatch_summaries(&workflow_id, &node_dispatches),
                director_reviews: workflow_dispatch_director_review_summaries(
                    &workflow_id,
                    &director_reviews,
                ),
                execution_controls: workflow_execution_control_summaries(
                    &workflow_id,
                    &execution_controls,
                ),
                permission_requests: workflow_permission_request_summaries(
                    &workflow_id,
                    &permission_requests,
                ),
                execution_attempts: workflow_execution_attempt_summaries(
                    &workflow_id,
                    &execution_attempts,
                ),
                derived_workflow: Some(derive_workflow_read_model(
                    workflow,
                    &project_id,
                    &project_root,
                    &nodes,
                    &edges,
                    &work_items,
                    &artifacts,
                    &node_session_bindings,
                    &node_dispatches,
                    &director_reviews,
                    &permission_requests,
                    &execution_attempts,
                    &audit_events,
                )),
                node_count: nodes
                    .iter()
                    .filter(|node| {
                        optional_string_from(node, "workflow_id").as_deref()
                            == Some(workflow_id.as_str())
                    })
                    .count(),
                edge_count: edges
                    .iter()
                    .filter(|edge| {
                        optional_string_from(edge, "workflow_id").as_deref()
                            == Some(workflow_id.as_str())
                    })
                    .count(),
                title: optional_string_from(workflow, "title")
                    .unwrap_or_else(|| "未命名工作流".to_string()),
                state: optional_string_from(workflow, "state")
                    .unwrap_or_else(|| "unknown".to_string()),
                project_id,
                project_root,
                workflow_id,
            })
        })
        .collect()
}

fn project_blackboards_from_workflows(
    workflows: &[ProjectWorkflowSummary],
    audit_events: &[Value],
) -> Vec<ProjectBlackboard> {
    workflow_read_model::derive_project_blackboards(workflows, |summary| {
        project_blackboard_from_workflow(summary, audit_events)
    })
}

fn workflow_chain_audit_event_types_for_workflow(
    audit_events: &[Value],
    workflow_id: &str,
) -> BTreeMap<String, String> {
    audit_events
        .iter()
        .filter_map(|event| {
            let event_type = optional_string_from(event, "event_type")?;
            if !event_type.starts_with("workflow_chain_") {
                return None;
            }
            let belongs_to_workflow = match optional_string_from(event, "workflow_id") {
                Some(audit_workflow_id) => audit_workflow_id == workflow_id,
                None => optional_string_from(event, "target_ref")
                    .is_some_and(|target_ref| target_ref.contains(workflow_id)),
            };
            if !belongs_to_workflow {
                return None;
            }
            Some((optional_string_from(event, "event_id")?, event_type))
        })
        .collect()
}

fn project_blackboard_from_workflow(
    summary: &ProjectWorkflowSummary,
    audit_events: &[Value],
) -> ProjectBlackboard {
    let mut entries = Vec::new();
    let chain_audit_event_types =
        workflow_chain_audit_event_types_for_workflow(audit_events, &summary.workflow_id);

    if let Some(workflow) = &summary.derived_workflow {
        for report in &workflow.subagent_reports {
            entries.push(blackboard_candidate_entry(
                format!("blackboard:{}:report:{}", workflow.workflow_id, stable_id(&report.report_id)),
                &summary.project_id,
                &summary.workflow_id,
                None,
                report.workflow_node_id.clone(),
                BlackboardEntryKind::SubagentReport,
                format!("子智能体汇报 / {}", report.actor_role.as_deref().unwrap_or("unknown")),
                report.summary.clone(),
                None,
                {
                    let mut refs = vec![blackboard_source_ref("subagent_report", &report.report_id, "子智能体汇报")];
                    refs.extend(report.evidence_refs.iter().map(|evidence| {
                        blackboard_source_ref("evidence", evidence, "汇报证据")
                    }));
                    refs
                },
                None,
                "workflow_fact",
                "子智能体汇报只进入项目黑板；必须经控制核心和项目主管确认后，才可能升级为正式事实或状态变化。",
                {
                    let mut warnings = report.warnings.clone();
                    warnings.push("subagent_report_does_not_complete_workflow_node".to_string());
                    warnings
                },
            ));

            for (index, risk) in report.direction_risks.iter().enumerate() {
                entries.push(blackboard_candidate_entry(
                    format!(
                        "blackboard:{}:risk:{}:{index}",
                        workflow.workflow_id,
                        stable_id(&report.report_id)
                    ),
                    &summary.project_id,
                    &summary.workflow_id,
                    None,
                    report.workflow_node_id.clone(),
                    BlackboardEntryKind::Risk,
                    "方向风险候选".to_string(),
                    risk.clone(),
                    Some(report.acceptance_status.clone()),
                    vec![blackboard_source_ref(
                        "subagent_report",
                        &report.report_id,
                        "风险来源",
                    )],
                    None,
                    "workflow_risk",
                    "风险只作为黑板候选；不会直接推进工作流状态。",
                    vec!["risk_candidate_not_workflow_state_transition".to_string()],
                ));
            }
        }

        for request in &summary.permission_requests {
            entries.push(blackboard_candidate_entry(
                format!(
                    "blackboard:{}:permission:{}",
                    workflow.workflow_id,
                    stable_id(&request.request_id)
                ),
                &summary.project_id,
                &summary.workflow_id,
                Some(request.work_item_id.clone()),
                None,
                BlackboardEntryKind::PermissionRequest,
                format!("权限请求 / {}", request.permission_kind),
                request.reason.clone(),
                Some(request.status.clone()),
                {
                    let mut refs = vec![blackboard_source_ref(
                        "permission_request",
                        &request.request_id,
                        "权限请求",
                    )];
                    if let Some(dispatch_id) = &request.dispatch_id {
                        refs.push(blackboard_source_ref("dispatch", dispatch_id, "关联派发"));
                    }
                    refs
                },
                Some(request.requested_at.clone()),
                "permission_decision",
                "权限请求在黑板中只是待处理项；不能由黑板直接批准、拒绝或推进状态。",
                {
                    let mut warnings = request.warnings.clone();
                    warnings.push("permission_request_requires_control_core_decision".to_string());
                    warnings
                },
            ));
        }

        for ledger_entry in workflow
            .ledger_entries
            .iter()
            .filter(|entry| entry.entry_type == "tool_call_summary")
        {
            entries.push(blackboard_candidate_entry(
                format!(
                    "blackboard:{}:tool:{}",
                    workflow.workflow_id,
                    stable_id(&ledger_entry.ledger_entry_id)
                ),
                &summary.project_id,
                &summary.workflow_id,
                ledger_entry.source_refs.first().cloned(),
                ledger_entry.workflow_node_id.clone(),
                BlackboardEntryKind::ToolSummary,
                "工具摘要候选".to_string(),
                ledger_entry.summary.clone(),
                None,
                {
                    let mut refs = vec![blackboard_source_ref(
                        "ledger_entry",
                        &ledger_entry.ledger_entry_id,
                        "账本摘要",
                    )];
                    refs.extend(ledger_entry.tool_call_refs.iter().map(|tool_ref| {
                        blackboard_source_ref("tool_call", tool_ref, "工具调用引用")
                    }));
                    refs
                },
                ledger_entry.created_at.clone(),
                "audit_event",
                "工具摘要只保留摘要和引用；不会把工具全文直接升级为审计事件或事实。",
                {
                    let mut warnings = ledger_entry.risk_flags.clone();
                    warnings.push("tool_summary_is_not_full_tool_output".to_string());
                    warnings
                },
            ));
        }

        let answered_question_refs = workflow
            .ledger_entries
            .iter()
            .filter(|entry| entry.entry_type == "user_reply")
            .flat_map(|entry| entry.source_refs.iter().cloned())
            .collect::<std::collections::BTreeSet<_>>();
        for ledger_entry in workflow.ledger_entries.iter().filter(|entry| {
            matches!(
                entry.entry_type.as_str(),
                "supervisor_question" | "user_reply"
            )
        }) {
            let is_question = ledger_entry.entry_type == "supervisor_question";
            let status = if is_question
                && !ledger_entry
                    .source_refs
                    .iter()
                    .any(|source_ref| answered_question_refs.contains(source_ref))
            {
                "waiting_user"
            } else {
                "answered"
            };
            let title = if is_question {
                format!("主管问题 / {status}")
            } else {
                "用户答复 / 已记录".to_string()
            };
            let mut source_refs = vec![blackboard_source_ref(
                "ledger_entry",
                &ledger_entry.ledger_entry_id,
                "主管对话账本",
            )];
            source_refs.extend(ledger_entry.audit_refs.iter().map(|audit_ref| {
                blackboard_source_ref("workflow_audit", audit_ref, "主管对话审计")
            }));
            entries.push(blackboard_supervisor_message_entry(
                format!(
                    "blackboard:{}:supervisor-message:{}",
                    workflow.workflow_id,
                    stable_id(&ledger_entry.ledger_entry_id)
                ),
                &summary.project_id,
                &summary.workflow_id,
                None,
                supervisor_message_question_id(
                    &ledger_entry.workflow_id,
                    &ledger_entry.source_refs,
                ),
                BlackboardEntryKind::SupervisorMessage,
                title,
                ledger_entry.summary.clone(),
                status,
                Some(status.to_string()),
                "主管问题和用户答复是会话事实，不是黑板候选，也不会推进工作流状态。",
                source_refs,
                ledger_entry.created_at.clone(),
            ));
        }

        for (ledger_entry_ordinal, ledger_entry) in workflow.ledger_entries.iter().enumerate() {
            let Some(audit_ref) = ledger_entry.audit_refs.first() else {
                continue;
            };
            let Some(source_event_type) = chain_audit_event_types.get(audit_ref) else {
                continue;
            };
            let Some((title, message_summary)) =
                supervisor_process_message_template(source_event_type)
            else {
                continue;
            };
            entries.push(blackboard_supervisor_message_entry(
                format!(
                    "blackboard:{}:supervisor-process:{ledger_entry_ordinal:08}:{}",
                    workflow.workflow_id,
                    stable_id(&ledger_entry.ledger_entry_id)
                ),
                &summary.project_id,
                &summary.workflow_id,
                ledger_entry.workflow_node_id.clone(),
                None,
                BlackboardEntryKind::SupervisorMessage,
                title.to_string(),
                message_summary.to_string(),
                "reported",
                Some(source_event_type.to_string()),
                "主管过程短讯是纯派生读模型，不是黑板候选，也不会推进工作流状态。",
                vec![blackboard_source_ref(
                    "workflow_chain_event",
                    audit_ref,
                    "链事件审计",
                )],
                ledger_entry.created_at.clone(),
            ));
        }

        for task_package in &workflow.task_packages {
            for memory_ref in &task_package.available_memory_refs {
                entries.push(blackboard_candidate_entry(
                    format!(
                        "blackboard:{}:memory-candidate:{}:{}",
                        workflow.workflow_id,
                        stable_id(&task_package.task_package_id),
                        stable_id(memory_ref)
                    ),
                    &summary.project_id,
                    &summary.workflow_id,
                    None,
                    Some(task_package.workflow_node_id.clone()),
                    BlackboardEntryKind::MemoryCandidate,
                    "记忆候选".to_string(),
                    format!("{memory_ref} 来自任务包显式记忆引用；这里只作为候选，不写正式记忆。"),
                    Some(
                        if task_package.stale {
                            "stale_task_package"
                        } else {
                            "fresh_task_package"
                        }
                        .to_string(),
                    ),
                    vec![
                        blackboard_source_ref(
                            "task_package",
                            &task_package.task_package_id,
                            "任务包",
                        ),
                        blackboard_source_ref("memory_candidate", memory_ref, "记忆候选"),
                    ],
                    None,
                    "formal_memory",
                    "记忆候选必须经用户或控制核心确认，不能由黑板直接写正式记忆。",
                    vec!["memory_candidate_not_formal_memory".to_string()],
                ));
            }

            for knowledge_ref in &task_package.available_knowledge_refs {
                entries.push(blackboard_candidate_entry(
                    format!(
                        "blackboard:{}:knowledge-ref:{}:{}",
                        workflow.workflow_id,
                        stable_id(&task_package.task_package_id),
                        stable_id(knowledge_ref)
                    ),
                    &summary.project_id,
                    &summary.workflow_id,
                    None,
                    Some(task_package.workflow_node_id.clone()),
                    BlackboardEntryKind::KnowledgeRef,
                    "知识引用".to_string(),
                    format!("{knowledge_ref} 是任务包显式资料来源；不会被当作记忆写入。"),
                    Some(
                        if task_package.stale {
                            "stale_task_package"
                        } else {
                            "fresh_task_package"
                        }
                        .to_string(),
                    ),
                    vec![
                        blackboard_source_ref(
                            "task_package",
                            &task_package.task_package_id,
                            "任务包",
                        ),
                        blackboard_source_ref("knowledge_ref", knowledge_ref, "知识引用"),
                    ],
                    None,
                    "knowledge_reference",
                    "知识引用只作为资料来源；不能被黑板直接升级为正式记忆。",
                    vec!["knowledge_ref_is_not_memory".to_string()],
                ));
            }
        }
    }

    entries.sort_by(|left, right| left.entry_id.cmp(&right.entry_id));

    ProjectBlackboard {
        project_id: summary.project_id.clone(),
        project_root: summary.project_root.clone(),
        workflow_id: summary.workflow_id.clone(),
        entries,
        warnings: vec![
            "project_blackboard_is_read_model_only".to_string(),
            "blackboard_promotion_requires_control_core_confirmation".to_string(),
        ],
    }
}

#[allow(clippy::too_many_arguments)]
fn blackboard_candidate_entry(
    entry_id: String,
    project_id: &str,
    workflow_id: &str,
    work_item_id: Option<String>,
    workflow_node_id: Option<String>,
    kind: BlackboardEntryKind,
    title: String,
    summary: String,
    source_status: Option<String>,
    source_refs: Vec<BlackboardSourceRef>,
    created_at: Option<String>,
    target_kind: &str,
    reason: &str,
    mut warnings: Vec<String>,
) -> BlackboardEntry {
    warnings.push("blackboard_entry_is_candidate_only".to_string());
    BlackboardEntry {
        promotion_decision: blackboard_promotion_decision(&entry_id, target_kind, reason),
        entry_id,
        project_id: project_id.to_string(),
        workflow_id: workflow_id.to_string(),
        work_item_id,
        workflow_node_id,
        question_id: None,
        kind,
        title,
        summary,
        status: "candidate".to_string(),
        source_status,
        source_refs,
        created_at,
        warnings,
    }
}

fn blackboard_supervisor_message_entry(
    entry_id: String,
    project_id: &str,
    workflow_id: &str,
    workflow_node_id: Option<String>,
    question_id: Option<String>,
    kind: BlackboardEntryKind,
    title: String,
    summary: String,
    status: &str,
    source_status: Option<String>,
    promotion_reason: &str,
    source_refs: Vec<BlackboardSourceRef>,
    created_at: Option<String>,
) -> BlackboardEntry {
    BlackboardEntry {
        promotion_decision: BlackboardPromotionDecision {
            decision_id: format!("promotion:{entry_id}"),
            status: "not_applicable".to_string(),
            target_kind: None,
            decided_by_role: None,
            decided_at: None,
            reason: promotion_reason.to_string(),
            audit_refs: vec![],
            warnings: vec!["supervisor_message_not_a_promotion_candidate".to_string()],
        },
        entry_id,
        project_id: project_id.to_string(),
        workflow_id: workflow_id.to_string(),
        work_item_id: None,
        workflow_node_id,
        question_id,
        kind,
        title,
        summary,
        status: status.to_string(),
        source_status,
        source_refs,
        created_at,
        warnings: vec![
            "supervisor_message_is_read_model_only".to_string(),
            "supervisor_message_does_not_advance_workflow".to_string(),
        ],
    }
}

fn supervisor_process_message_template(event_type: &str) -> Option<(&'static str, &'static str)> {
    match event_type {
        "workflow_chain_run_started" => Some(("主管进度 / 开跑", "我开跑了，任务已经排好队。")),
        "workflow_chain_node_started" => Some(("主管进度 / 开始处理", "我在做下一件事了。")),
        "workflow_chain_node_completed" => Some(("主管进度 / 一项完成", "这一件做完了。")),
        "workflow_chain_node_waiting_decision" => {
            Some(("主管进度 / 等待你", "我先停在这儿了——worker 有话想问你。"))
        }
        "workflow_chain_node_needs_rework" => {
            Some(("主管进度 / 需要返工", "这一件要回去再做一遍。"))
        }
        "workflow_chain_run_completed" => Some(("主管进度 / 已完成", "都干完了，结果放你右手边。")),
        "workflow_chain_run_failed" | "workflow_chain_run_stopped" => {
            Some(("主管进度 / 已中断", "这轮先停下来了，原因在右边。"))
        }
        _ => None,
    }
}

fn supervisor_message_question_id(workflow_id: &str, source_refs: &[String]) -> Option<String> {
    let prefix = format!("{workflow_id}:resident-question:");
    source_refs.iter().find_map(|source_ref| {
        source_ref.strip_prefix(&prefix).and_then(|question_id| {
            let question_id = question_id.trim();
            (!question_id.is_empty()).then(|| question_id.to_string())
        })
    })
}

fn blackboard_source_ref(source_kind: &str, source_id: &str, label: &str) -> BlackboardSourceRef {
    BlackboardSourceRef {
        source_kind: source_kind.to_string(),
        source_id: source_id.to_string(),
        label: label.to_string(),
    }
}

fn blackboard_promotion_decision(
    entry_id: &str,
    target_kind: &str,
    reason: &str,
) -> BlackboardPromotionDecision {
    BlackboardPromotionDecision {
        decision_id: format!("promotion:{entry_id}"),
        status: "candidate_pending_control_core".to_string(),
        target_kind: Some(target_kind.to_string()),
        decided_by_role: None,
        decided_at: None,
        reason: reason.to_string(),
        audit_refs: vec![],
        warnings: vec!["not_promoted_without_control_core_confirmation".to_string()],
    }
}

fn task_draft_summaries(
    workflow_id: &str,
    work_items: &[Value],
    artifacts: &[Value],
    audit_events: &[Value],
) -> Vec<TaskDraftSummary> {
    work_items
        .iter()
        .filter(|item| optional_string_from(item, "workflow_id").as_deref() == Some(workflow_id))
        .filter_map(|item| {
            let work_item_id = optional_string_from(item, "work_item_id")?;
            let artifact_type = artifacts
                .iter()
                .find(|artifact| {
                    optional_string_from(artifact, "source_ref").as_deref()
                        == Some(work_item_id.as_str())
                })
                .and_then(|artifact| optional_string_from(artifact, "artifact_type"));
            let artifact_path = artifacts
                .iter()
                .find(|artifact| {
                    optional_string_from(artifact, "source_ref").as_deref()
                        == Some(work_item_id.as_str())
                })
                .and_then(|artifact| optional_string_from(artifact, "path"));
            let state =
                optional_string_from(item, "state").unwrap_or_else(|| "unknown".to_string());
            Some(TaskDraftSummary {
                workflow_id: workflow_id.to_string(),
                title: optional_string_from(item, "title")
                    .unwrap_or_else(|| "未命名任务草稿".to_string()),
                state: state.clone(),
                assigned_role_id: optional_string_from(item, "assigned_role_id"),
                current_node_id: optional_string_from(item, "current_node_id")
                    .or_else(|| Some(workflow_node_for_work_item_state(workflow_id, &state))),
                next_states: next_work_item_states(&state),
                next_action_label: next_action_label(&state),
                artifact_type,
                artifact_path,
                recent_audit_events: recent_audit_events_for(&work_item_id, audit_events, 3),
                work_item_id,
            })
        })
        .collect()
}

fn derive_workflow_read_model(
    workflow: &Value,
    project_id: &str,
    project_root: &str,
    nodes: &[Value],
    edges: &[Value],
    work_items: &[Value],
    artifacts: &[Value],
    node_session_bindings: &[Value],
    node_dispatches: &[Value],
    director_reviews: &[Value],
    permission_requests: &[Value],
    execution_attempts: &[Value],
    audit_events: &[Value],
) -> Workflow {
    let workflow_id = optional_string_from(workflow, "workflow_id").unwrap_or_default();
    let workflow_nodes = derive_workflow_nodes(
        &workflow_id,
        nodes,
        edges,
        work_items,
        artifacts,
        node_session_bindings,
    );
    let task_packages = derive_task_packages(
        &workflow_id,
        project_id,
        project_root,
        work_items,
        artifacts,
        node_session_bindings,
    );
    let ledger_entries = derive_workflow_ledger_entries(
        &workflow_id,
        audit_events,
        node_dispatches,
        director_reviews,
        permission_requests,
    );
    // 运行性检查剔除 canvas_run 临时 work_item（跑链/跑节点的产物、会累积、无任务包 artifact）——
    // 与 submit 侧一致，否则跑过一次后状态条会一直显 blocked、跟「能存草案」对不上。真任务包件照查。
    let work_items_for_check: Vec<Value> = work_items
        .iter()
        .filter(|wi| optional_string_from(wi, "source_kind").as_deref() != Some("canvas_run"))
        .cloned()
        .collect();
    let check = inspect_workflow_run_check_from_value(
        project_root,
        Some(&workflow_id),
        workflow,
        nodes,
        &work_items_for_check,
        artifacts,
        node_session_bindings,
    );
    let review_results = derive_review_results(&workflow_id, director_reviews);
    let subagent_reports = derive_subagent_reports(
        &workflow_id,
        node_dispatches,
        audit_events,
        permission_requests,
    );
    let exceptions = derive_workflow_exceptions(
        &workflow_id,
        artifacts,
        permission_requests,
        execution_attempts,
        &review_results,
    );
    let state_machine =
        workflow_state_machine_summary(&task_packages, &review_results, &exceptions);
    let acceptance_scenarios = workflow_acceptance_scenarios(
        &task_packages,
        &subagent_reports,
        &review_results,
        &exceptions,
    );
    let result_summary = derive_workflow_result_summary_read_model(
        project_id,
        &workflow_id,
        artifacts,
        director_reviews,
        audit_events,
    );

    Workflow {
        workflow_id: workflow_id.clone(),
        project_id: project_id.to_string(),
        title: optional_string_from(workflow, "title")
            .unwrap_or_else(|| "未命名工作流".to_string()),
        source_proposal_id: optional_string_from(workflow, "source_proposal_id"),
        status: optional_string_from(workflow, "state").unwrap_or_else(|| "unknown".to_string()),
        view_mode: optional_string_from(workflow, "view_mode"),
        created_by_role: optional_string_from(workflow, "created_by_role"),
        owner_role: optional_string_from(workflow, "owner_role"),
        current_stage: optional_string_from(workflow, "current_stage"),
        run_check_status: check.status,
        risk_level: optional_string_from(workflow, "risk_level"),
        created_at: optional_string_from(workflow, "created_at"),
        updated_at: optional_string_from(workflow, "updated_at"),
        nodes: workflow_nodes,
        task_packages,
        ledger_entries,
        subagent_reports,
        review_results,
        exceptions,
        result_summary,
        interface_boundaries: workflow_interface_boundaries(),
        state_machine,
        acceptance_scenarios,
        warnings: vec!["derived_from_workflow_state_v0_missing_fields_are_not_guessed".to_string()],
    }
}

fn derive_workflow_nodes(
    workflow_id: &str,
    nodes: &[Value],
    edges: &[Value],
    work_items: &[Value],
    artifacts: &[Value],
    node_session_bindings: &[Value],
) -> Vec<WorkflowNode> {
    nodes
        .iter()
        .filter(|node| optional_string_from(node, "workflow_id").as_deref() == Some(workflow_id))
        .map(|node| {
            let node_id =
                optional_string_from(node, "node_id").unwrap_or_else(|| "node:missing".to_string());
            let assigned_role = node_id
                .split(":node:")
                .nth(1)
                .map(str::to_string)
                .or_else(|| optional_string_from(node, "assigned_role"));
            let assigned_session_id = node_session_bindings
                .iter()
                .find(|binding| {
                    optional_string_from(binding, "workflow_id").as_deref() == Some(workflow_id)
                        && optional_string_from(binding, "node_id").as_deref()
                            == Some(node_id.as_str())
                        && optional_string_from(binding, "lifecycle").as_deref() == Some("active")
                })
                .and_then(|binding| optional_string_from(binding, "native_thread_id"));
            let task_package_id = work_items
                .iter()
                .find(|item| {
                    optional_string_from(item, "current_node_id").as_deref()
                        == Some(node_id.as_str())
                })
                .and_then(|item| {
                    let work_item_id = optional_string_from(item, "work_item_id")?;
                    artifacts
                        .iter()
                        .find(|artifact| {
                            optional_string_from(artifact, "source_ref").as_deref()
                                == Some(work_item_id.as_str())
                        })
                        .and_then(|artifact| optional_string_from(artifact, "artifact_id"))
                });
            let acceptance_criteria = task_package_id
                .as_deref()
                .and_then(|artifact_id| {
                    artifacts.iter().find(|artifact| {
                        optional_string_from(artifact, "artifact_id").as_deref()
                            == Some(artifact_id)
                    })
                })
                .map(|artifact| string_array(artifact, "acceptance_criteria"))
                .unwrap_or_default();
            let mut missing_fields = Vec::new();
            if assigned_session_id.is_none()
                && assigned_role
                    .as_deref()
                    .is_some_and(|role| role == "codex-dev")
            {
                missing_fields.push("assigned_session_id".to_string());
            }
            if acceptance_criteria.is_empty() && task_package_id.is_some() {
                missing_fields.push("acceptance_criteria".to_string());
            }
            WorkflowNode {
                workflow_node_id: node_id.clone(),
                workflow_id: workflow_id.to_string(),
                node_type: optional_string_from(node, "node_type")
                    .unwrap_or_else(|| "unknown".to_string()),
                title: optional_string_from(node, "title")
                    .unwrap_or_else(|| "未命名节点".to_string()),
                assigned_role,
                assigned_session_id,
                status: optional_string_from(node, "state")
                    .unwrap_or_else(|| "unknown".to_string()),
                task_package_id,
                depends_on: edges
                    .iter()
                    .filter(|edge| {
                        optional_string_from(edge, "to_node_id").as_deref()
                            == Some(node_id.as_str())
                    })
                    .filter_map(|edge| optional_string_from(edge, "from_node_id"))
                    .collect(),
                harness_requirements: string_array(node, "harness_requirements"),
                review_requirements: string_array(node, "review_requirements"),
                acceptance_criteria,
                created_at: optional_string_from(node, "created_at"),
                updated_at: optional_string_from(node, "updated_at"),
                missing_fields,
                warnings: string_array(node, "warnings"),
            }
        })
        .collect()
}

fn derive_task_packages(
    workflow_id: &str,
    project_id_value: &str,
    project_root: &str,
    work_items: &[Value],
    artifacts: &[Value],
    node_session_bindings: &[Value],
) -> Vec<TaskPackage> {
    work_items
        .iter()
        .filter(|item| optional_string_from(item, "workflow_id").as_deref() == Some(workflow_id))
        .filter_map(|work_item| {
            let work_item_id = optional_string_from(work_item, "work_item_id")?;
            let artifact = artifacts.iter().find(|artifact| {
                optional_string_from(artifact, "artifact_type").as_deref() == Some("task_package")
                    && optional_string_from(artifact, "source_ref").as_deref()
                        == Some(work_item_id.as_str())
            })?;
            let artifact_id = optional_string_from(artifact, "artifact_id")
                .unwrap_or_else(|| format!("task-package:{work_item_id}"));
            let node_id = optional_string_from(work_item, "current_node_id").unwrap_or_else(|| {
                workflow_node_for_work_item_state(
                    workflow_id,
                    optional_string_from(work_item, "state")
                        .as_deref()
                        .unwrap_or("draft"),
                )
            });
            let target_role = optional_string_from(work_item, "assigned_role_id");
            let target_session_id = target_role.as_deref().and_then(|role| {
                let expected_node_id = format!("{workflow_id}:node:{role}");
                node_session_bindings
                    .iter()
                    .find(|binding| {
                        optional_string_from(binding, "node_id").as_deref()
                            == Some(expected_node_id.as_str())
                            && optional_string_from(binding, "lifecycle").as_deref()
                                == Some("active")
                    })
                    .and_then(|binding| optional_string_from(binding, "native_thread_id"))
            });
            let allowed_read_scope = string_array(artifact, "allowed_read_scope");
            let allowed_write_scope = string_array(artifact, "allowed_write");
            let acceptance_criteria = string_array(artifact, "acceptance_criteria");
            let report_format = string_array(artifact, "report_format");
            let mut missing_fields = Vec::new();
            if allowed_read_scope.is_empty() {
                missing_fields.push("allowed_read_scope".to_string());
            }
            if allowed_write_scope.is_empty() {
                missing_fields.push("allowed_write_scope".to_string());
            }
            if acceptance_criteria.is_empty() {
                missing_fields.push("acceptance_criteria".to_string());
            }
            if report_format.is_empty() {
                missing_fields.push("report_format".to_string());
            }
            if target_role.is_none() {
                missing_fields.push("target_role".to_string());
            }
            if target_session_id.is_none() {
                missing_fields.push("target_session_id".to_string());
            }
            let stale = bool_value(artifact, "stale")
                || optional_string_from(artifact, "path").is_none()
                    && optional_string_from(artifact, "last_generated_fingerprint").is_some();
            let memory_injection_summary = task_memory_injection::snapshot_from_artifact(artifact)
                .map(|snapshot| {
                    let mut stale_reasons = snapshot.stale_reasons.clone();
                    if bool_value(artifact, "memory_packet_stale") && stale_reasons.is_empty() {
                        stale_reasons.push("task_memory_packet_snapshot_marked_stale".to_string());
                    }
                    task_memory_injection::summary_from_snapshot(&snapshot, stale_reasons)
                })
                .unwrap_or_else(task_memory_injection::missing_summary);
            Some(TaskPackage {
                task_package_id: artifact_id,
                workflow_id: workflow_id.to_string(),
                workflow_node_id: node_id,
                project_id: project_id_value.to_string(),
                target_session_id,
                target_role,
                task_goal: optional_string_from(artifact, "task_goal")
                    .or_else(|| optional_string_from(work_item, "title")),
                allowed_read_scope,
                allowed_write_scope,
                available_skills: string_array(artifact, "available_skills"),
                available_knowledge_refs: string_array(artifact, "available_knowledge_refs"),
                available_memory_refs: string_array(artifact, "available_memory_refs"),
                callable_tool_capabilities: string_array(artifact, "callable_tool_capabilities"),
                model_id: optional_string_from(artifact, "model_id"),
                harness_requirements: string_array(artifact, "harness_requirements"),
                forbidden_actions: string_array(artifact, "forbidden_actions"),
                acceptance_criteria,
                report_format,
                timeout_policy: optional_string_from(artifact, "timeout_policy")
                    .or_else(|| Some("未登记".to_string())),
                failure_policy: optional_string_from(artifact, "failure_policy")
                    .or_else(|| Some("未登记".to_string())),
                version: i64_value(artifact, "version").unwrap_or(1),
                stale,
                stale_reasons: string_array(artifact, "stale_reasons"),
                missing_fields,
                export_includes_internal_audit: bool_value(
                    artifact,
                    "export_includes_internal_audit",
                ),
                memory_injection_summary,
                warnings: {
                    let mut warnings = string_array(artifact, "warnings");
                    if project_root.trim().is_empty() {
                        warnings.push("missing_project_root".to_string());
                    }
                    warnings
                },
            })
        })
        .collect()
}

fn derive_workflow_ledger_entries(
    workflow_id: &str,
    audit_events: &[Value],
    node_dispatches: &[Value],
    director_reviews: &[Value],
    permission_requests: &[Value],
) -> Vec<WorkflowLedgerEntry> {
    let mut entries = workflow_read_model::derive_workflow_ledger_entries(
        workflow_id,
        audit_events,
        node_dispatches,
        director_reviews,
        permission_requests,
        workflow_read_model::WorkflowLedgerDerivationFns {
            optional_string_from,
            string_array,
            i64_value,
            ledger_entry_type_from_audit,
            compact_ledger_summary,
        },
    );
    for entry in &mut entries {
        validate_ledger_entry_type(entry);
    }
    entries
}

fn derive_subagent_reports(
    workflow_id: &str,
    node_dispatches: &[Value],
    audit_events: &[Value],
    _permission_requests: &[Value],
) -> Vec<SubagentReport> {
    let mut reports = node_dispatches
        .iter()
        .filter(|dispatch| {
            optional_string_from(dispatch, "workflow_id").as_deref() == Some(workflow_id)
        })
        .filter(|dispatch| optional_string_from(dispatch, "state").as_deref() == Some("completed"))
        .map(|dispatch| {
            let dispatch_id = optional_string_from(dispatch, "dispatch_id")
                .unwrap_or_else(|| "dispatch:missing".to_string());
            let work_item_id = optional_string_from(dispatch, "work_item_id").unwrap_or_default();
            let workflow_node_id = optional_string_from(dispatch, "node_id");
            let worker_report = matching_worker_report_event(
                workflow_id,
                &dispatch_id,
                &work_item_id,
                workflow_node_id.as_deref(),
                audit_events,
            );
            let warnings = string_array(dispatch, "warnings");
            SubagentReport {
                report_id: format!("report:{dispatch_id}"),
                workflow_id: workflow_id.to_string(),
                workflow_node_id: workflow_node_id.clone(),
                actor_role: dispatch_role_from_node(workflow_node_id.as_deref()),
                executed_what: optional_string_from(dispatch, "prompt_preview")
                    .map(|value| compact_ledger_summary(&value))
                    .unwrap_or_else(|| "未登记执行内容".to_string()),
                changed_what: optional_string_from(dispatch, "last_message_summary")
                    .unwrap_or_else(|| "未登记改动内容".to_string()),
                summary: optional_string_from(dispatch, "last_message_summary")
                    .unwrap_or_else(|| "子智能体汇报缺摘要".to_string()),
                evidence_refs: optional_string_from(dispatch, "last_message_path")
                    .into_iter()
                    .collect(),
                open_issues: worker_report
                    .map(|event| string_array(event, "open_issues"))
                    .unwrap_or_default(),
                permission_requests: worker_report
                    .map(|event| string_array(event, "permission_requests"))
                    .unwrap_or_default(),
                direction_risks: worker_report
                    .map(|event| string_array(event, "direction_risks"))
                    .unwrap_or_default(),
                follow_up_suggestions: worker_report
                    .map(|event| string_array(event, "follow_up_suggestions"))
                    .unwrap_or_default(),
                acceptance_status: worker_report
                    .and_then(|event| optional_string_from(event, "acceptance_status"))
                    .or_else(|| optional_string_from(dispatch, "acceptance_status"))
                    .unwrap_or_else(|| "reported_not_completed".to_string()),
                warnings,
            }
        })
        .collect::<Vec<_>>();

    for event in audit_events.iter().filter(|event| {
        matches!(
            optional_string_from(event, "event_type").as_deref(),
            Some("subagent_report") | Some("worker_structured_report_recorded")
        ) && optional_string_from(event, "workflow_id")
            .as_deref()
            .map_or_else(
                || {
                    optional_string_from(event, "target_ref")
                        .is_some_and(|target| target.contains(workflow_id))
                },
                |event_workflow_id| event_workflow_id == workflow_id,
            )
    }) {
        reports.push(SubagentReport {
            report_id: optional_string_from(event, "event_id")
                .unwrap_or_else(|| "report:audit:missing".to_string()),
            workflow_id: workflow_id.to_string(),
            workflow_node_id: optional_string_from(event, "node_id"),
            actor_role: optional_string_from(event, "actor_ref"),
            executed_what: optional_string_from(event, "executed_what")
                .unwrap_or_else(|| "未登记执行内容".to_string()),
            changed_what: optional_string_from(event, "changed_what")
                .unwrap_or_else(|| "未登记改动内容".to_string()),
            summary: optional_string_from(event, "reason")
                .unwrap_or_else(|| "子智能体汇报缺摘要".to_string()),
            evidence_refs: string_array(event, "evidence_refs"),
            open_issues: string_array(event, "open_issues"),
            permission_requests: string_array(event, "permission_requests"),
            direction_risks: string_array(event, "direction_risks"),
            follow_up_suggestions: string_array(event, "follow_up_suggestions"),
            acceptance_status: optional_string_from(event, "acceptance_status")
                .unwrap_or_else(|| "reported_not_completed".to_string()),
            warnings: string_array(event, "warnings"),
        });
    }
    reports
}

fn matching_worker_report_event<'a>(
    workflow_id: &str,
    dispatch_id: &str,
    work_item_id: &str,
    workflow_node_id: Option<&str>,
    audit_events: &'a [Value],
) -> Option<&'a Value> {
    audit_events.iter().rev().find(|event| {
        if optional_string_from(event, "event_type").as_deref()
            != Some("worker_structured_report_recorded")
            || !audit_event_matches_workflow(event, workflow_id)
        {
            return false;
        }
        if let Some(event_dispatch_id) = optional_string_from(event, "dispatch_id") {
            return event_dispatch_id == dispatch_id;
        }
        if let Some(event_work_item_id) = optional_string_from(event, "work_item_id") {
            return !work_item_id.is_empty() && event_work_item_id == work_item_id;
        }
        workflow_node_id.is_some_and(|node_id| {
            optional_string_from(event, "node_id").as_deref() == Some(node_id)
        })
    })
}

fn audit_event_matches_workflow(event: &Value, workflow_id: &str) -> bool {
    optional_string_from(event, "workflow_id")
        .as_deref()
        .map_or_else(
            || {
                optional_string_from(event, "target_ref")
                    .is_some_and(|target| target.contains(workflow_id))
            },
            |event_workflow_id| event_workflow_id == workflow_id,
        )
}

fn dispatch_role_from_node(node_id: Option<&str>) -> Option<String> {
    node_id
        .and_then(|id| id.split(":node:").nth(1))
        .map(str::to_string)
}

fn derive_review_results(workflow_id: &str, director_reviews: &[Value]) -> Vec<ReviewResult> {
    director_reviews
        .iter()
        .filter(|review| {
            optional_string_from(review, "workflow_id").as_deref() == Some(workflow_id)
        })
        .map(|review| {
            let result = optional_string_from(review, "decision")
                .unwrap_or_else(|| "not_required".to_string());
            let mut warnings = string_array(review, "warnings");
            if result == "accepted" {
                warnings
                    .push("review_passed_but_director_still_confirms_node_completion".to_string());
            }
            ReviewResult {
                review_id: optional_string_from(review, "review_id")
                    .unwrap_or_else(|| "review:missing".to_string()),
                workflow_id: workflow_id.to_string(),
                workflow_node_id: optional_string_from(review, "workflow_node_id"),
                reviewer_role: optional_string_from(review, "reviewer_role"),
                report_id: optional_string_from(review, "report_id"),
                accepted_fact_ids: string_array(review, "accepted_fact_ids"),
                observation_ids: string_array(review, "observation_ids"),
                result: review_result_label(&result).to_string(),
                summary: optional_string_from(review, "summary").unwrap_or_default(),
                evidence_refs: string_array(review, "evidence_refs"),
                requires_director_confirmation: result != "confirm_process_fact",
                can_complete_node: false,
                warnings,
            }
        })
        .collect()
}

fn derive_workflow_result_summary_read_model(
    project_id_value: &str,
    workflow_id: &str,
    artifacts: &[Value],
    reviews: &[Value],
    audit_events: &[Value],
) -> WorkflowResultSummaryReadModel {
    let final_review = reviews.iter().rev().find(|review| {
        optional_string_from(review, "workflow_id").as_deref() == Some(workflow_id)
            && optional_string_from(review, "reviewer_role").as_deref() == Some("global_director")
            && optional_string_from(review, "review_target").as_deref()
                == Some("global_final_result")
    });
    let user_decision = reviews.iter().rev().find(|review| {
        optional_string_from(review, "workflow_id").as_deref() == Some(workflow_id)
            && optional_string_from(review, "reviewer_role").as_deref() == Some("user")
            && optional_string_from(review, "review_target").as_deref()
                == Some("user_result_decision")
    });
    let final_review_status = final_review
        .and_then(|review| optional_string_from(review, "decision"))
        .unwrap_or_else(|| "pending".to_string());
    let user_decision_status = user_decision
        .and_then(|review| optional_string_from(review, "decision"))
        .unwrap_or_else(|| "pending".to_string());
    let final_review_id = final_review.and_then(|review| optional_string_from(review, "review_id"));
    let user_decision_id =
        user_decision.and_then(|review| optional_string_from(review, "review_id"));
    let stage_c_acceptance = latest_stage_c_acceptance_artifact(artifacts, workflow_id)
        .and_then(|artifact| artifact.get("stage_c_acceptance_summary").cloned())
        .and_then(|summary| serde_json::from_value::<StageCAcceptanceSummary>(summary).ok())
        .unwrap_or_else(|| {
            pending_stage_c_acceptance_summary(
                project_id_value,
                workflow_id,
                &final_review_status,
                &user_decision_status,
            )
        });
    let open_issues = final_review
        .map(|review| string_array(review, "open_issues"))
        .unwrap_or_else(|| {
            audit_events
                .iter()
                .filter(|event| {
                    optional_string_from(event, "workflow_id").as_deref() == Some(workflow_id)
                        && matches!(
                            optional_string_from(event, "event_type").as_deref(),
                            Some(
                                "workflow_execution_failed"
                                    | "workflow_execution_timed_out"
                                    | "workflow_execution_cancelled"
                            )
                        )
                })
                .filter_map(|event| optional_string_from(event, "reason"))
                .take(5)
                .collect()
        });
    let deferred_items = if stage_c_acceptance.deferred_items.is_empty() {
        final_review
            .map(|review| string_array(review, "deferred_items"))
            .unwrap_or_default()
    } else {
        stage_c_acceptance.deferred_items.clone()
    };
    WorkflowResultSummaryReadModel {
        project_id: project_id_value.to_string(),
        workflow_id: workflow_id.to_string(),
        final_review_status,
        final_review_id,
        user_decision_status,
        user_decision_id,
        stage_c_acceptance,
        open_issues,
        deferred_items,
        warnings: vec![
            "workflow_result_summary_is_read_model".to_string(),
            "stage_c_acceptance_does_not_complete_middle_version".to_string(),
        ],
    }
}

fn pending_stage_c_acceptance_summary(
    project_id_value: &str,
    workflow_id: &str,
    final_review_status: &str,
    user_decision_status: &str,
) -> StageCAcceptanceSummary {
    StageCAcceptanceSummary {
        project_id: project_id_value.to_string(),
        workflow_id: workflow_id.to_string(),
        gates: vec![stage_c_gate(
            "stage-c-summary-artifact",
            "阶段 C 验收摘要",
            "missing_evidence",
            "尚未生成阶段 C gate 摘要。".to_string(),
            vec![],
        )],
        final_review_status: final_review_status.to_string(),
        user_decision_status: user_decision_status.to_string(),
        accepted_as_stage_c_complete: false,
        deferred_items: vec![
            "真实 worker / Codex 执行仍需单独授权任务包。".to_string(),
            "真实 Tauri 全面截图验收仍是后置项。".to_string(),
        ],
        open_blockers: vec!["尚未生成阶段 C 验收摘要。".to_string()],
        warnings: vec!["stage_c_summary_not_generated".to_string()],
    }
}

fn review_result_label(decision: &str) -> &str {
    match decision {
        "accepted" | "passed" => "passed",
        "needs_changes" | "returned" => "returned",
        "blocked" => "blocked",
        "failed" | "discarded" => "failed",
        "confirm_process_fact" => "process_fact_confirmed",
        "request_rework" => "rework_requested",
        "block_and_escalate" => "blocked_and_escalated",
        "accept_result" => "user_result_accepted",
        "request_changes" => "user_changes_requested",
        "reject_result" => "user_result_rejected",
        "not_required" => "not_required",
        _ => "not_required",
    }
}

fn derive_workflow_exceptions(
    workflow_id: &str,
    artifacts: &[Value],
    permission_requests: &[Value],
    execution_attempts: &[Value],
    review_results: &[ReviewResult],
) -> Vec<WorkflowException> {
    let mut exceptions = Vec::new();
    for attempt in execution_attempts.iter().filter(|attempt| {
        optional_string_from(attempt, "workflow_id").as_deref() == Some(workflow_id)
    }) {
        let state = optional_string_from(attempt, "state").unwrap_or_default();
        if state == "timed_out" || state == "failed" {
            exceptions.push(WorkflowException {
                exception_id: format!(
                    "exception:attempt:{}",
                    optional_string_from(attempt, "attempt_id")
                        .unwrap_or_else(|| "missing".to_string())
                ),
                workflow_id: workflow_id.to_string(),
                workflow_node_id: optional_string_from(attempt, "node_id"),
                exception_type: if state == "timed_out" {
                    "subagent_timeout"
                } else {
                    "subagent_failed"
                }
                .to_string(),
                summary: optional_string_from(attempt, "failure_reason")
                    .unwrap_or_else(|| format!("执行尝试进入 {state}")),
                status: "open".to_string(),
                warnings: string_array(attempt, "warnings"),
            });
        }
    }
    for request in permission_requests.iter().filter(|request| {
        optional_string_from(request, "workflow_id").as_deref() == Some(workflow_id)
    }) {
        if optional_string_from(request, "status").as_deref() == Some("pending") {
            exceptions.push(WorkflowException {
                exception_id: format!(
                    "exception:permission:{}",
                    optional_string_from(request, "request_id")
                        .unwrap_or_else(|| "missing".to_string())
                ),
                workflow_id: workflow_id.to_string(),
                workflow_node_id: None,
                exception_type: "long_permission_wait".to_string(),
                summary: optional_string_from(request, "reason")
                    .unwrap_or_else(|| "权限请求仍在等待".to_string()),
                status: "open".to_string(),
                warnings: string_array(request, "warnings"),
            });
        }
    }
    let returned_count = review_results
        .iter()
        .filter(|result| result.result == "returned")
        .count();
    if returned_count >= REVIEW_RETURN_EXCEPTION_THRESHOLD {
        exceptions.push(WorkflowException {
            exception_id: format!("exception:review-return:{workflow_id}"),
            workflow_id: workflow_id.to_string(),
            workflow_node_id: None,
            exception_type: "repeated_review_return".to_string(),
            summary: format!("审查退回已达到 {returned_count} 次。"),
            status: "open".to_string(),
            warnings: vec!["review_agent_cannot_end_node".to_string()],
        });
    }
    for artifact in artifacts.iter().filter(|artifact| {
        optional_string_from(artifact, "workflow_id").as_deref() == Some(workflow_id)
            || optional_string_from(artifact, "artifact_type").as_deref() == Some("task_package")
    }) {
        if bool_value(artifact, "harness_blocked") {
            exceptions.push(WorkflowException {
                exception_id: format!(
                    "exception:harness:{}",
                    optional_string_from(artifact, "artifact_id")
                        .unwrap_or_else(|| "artifact:missing".to_string())
                ),
                workflow_id: workflow_id.to_string(),
                workflow_node_id: optional_string_from(artifact, "node_id"),
                exception_type: "harness_blocked".to_string(),
                summary: "harness 阻断完成判定。".to_string(),
                status: "open".to_string(),
                warnings: string_array(artifact, "warnings"),
            });
        }
    }
    exceptions
}

const REVIEW_RETURN_EXCEPTION_THRESHOLD: usize = 2;

const LEDGER_ENTRY_TYPES: &[&str] = &[
    "task_package_created",
    "subagent_started",
    "permission_requested",
    "permission_granted",
    "permission_denied",
    "tool_call_summary",
    "subagent_report",
    "review_result",
    "node_returned",
    "node_failed",
    "node_passed",
    "director_summary",
    "user_decision",
    "waiting_decision",
    "node_skipped",
    "node_cancelled",
    "supervisor_question",
    "user_reply",
    "reply_injected",
];

fn is_valid_ledger_entry_type(entry_type: &str) -> bool {
    LEDGER_ENTRY_TYPES.contains(&entry_type)
}

fn validate_ledger_entry_type(entry: &mut WorkflowLedgerEntry) {
    if is_valid_ledger_entry_type(&entry.entry_type) {
        return;
    }
    let warning = format!("invalid_ledger_entry_type:{}", entry.entry_type);
    if !entry.risk_flags.contains(&warning) {
        entry.risk_flags.push(warning);
    }
}

fn ledger_entry_type_from_audit(event_type: &str) -> String {
    match event_type {
        "task_draft_created"
        | "task_node_state_updated"
        | "task_package_fields_updated"
        | "task_package_fields_corrected_for_dispatch"
        | "task_package_file_generated"
        | "task_memory_packet_injected_into_task_package" => "task_package_created",
        "workflow_permission_decision_recorded" => "user_decision",
        "workflow_dispatch_director_review_recorded" | "offline_director_review_recorded" => {
            "review_result"
        }
        "offline_role_dispatch_prepared"
        | "workflow_chain_node_started"
        | "workflow_chain_run_started"
        | "workflow_node_dispatch_prepared"
        | "workflow_node_dispatch_started" => "subagent_started",
        "offline_role_result_handoff_recorded"
        | "workflow_node_dispatch_completed"
        | "workflow_node_dispatch_readback_completed" => "subagent_report",
        "workflow_chain_node_completed"
        | "workflow_chain_node_director_deterministic_completed"
        | "workflow_chain_node_director_lm_completed"
        | "workflow_chain_run_completed" => "node_passed",
        "workflow_chain_node_failed"
        | "workflow_chain_run_failed"
        | "workflow_execution_failed"
        | "workflow_execution_timed_out"
        | "workflow_node_dispatch_failed" => "node_failed",
        "workflow_chain_node_needs_rework"
        | "workflow_chain_node_failed_action_rework"
        | "workflow_chain_run_stopped"
        | "workflow_chain_run_superseded" => "node_returned",
        "workflow_chain_node_failed_action_archive"
        | "workflow_chain_node_failed_action_change_session"
        | "workflow_chain_node_failed_action_retry"
        | "workflow_chain_run_stop_requested" => "user_decision",
        "workflow_chain_node_waiting_decision" | "workflow_chain_run_waiting_decision" => {
            "waiting_decision"
        }
        "workflow_chain_node_skipped" => "node_skipped",
        "workflow_chain_node_cancelled" | "workflow_execution_cancelled" => "node_cancelled",
        "workflow_chain_director_summary" => "director_summary",
        "supervisor_resident_question_asked" => "supervisor_question",
        "supervisor_resident_question_answered" => "user_reply",
        "supervisor_resident_reply_injected" => "reply_injected",
        _ => event_type,
    }
    .to_string()
}

fn compact_ledger_summary(summary: &str) -> String {
    let mut text = summary.lines().next().unwrap_or(summary).trim().to_string();
    if text.chars().count() > 240 {
        text = text.chars().take(240).collect::<String>();
        text.push_str("...");
    }
    text
}

fn workflow_state_machine_summary(
    task_packages: &[TaskPackage],
    review_results: &[ReviewResult],
    exceptions: &[WorkflowException],
) -> WorkflowStateMachineSummary {
    WorkflowStateMachineSummary {
        workflow_allowed_transitions: WORKFLOW_ALLOWED_TRANSITIONS
            .iter()
            .map(|(from, to)| format!("{from}->{to}"))
            .collect(),
        workflow_rejected_transitions: vec![
            "draft->running".to_string(),
            "waiting_decision->completed".to_string(),
            "failed->running_without_retry_or_reopen".to_string(),
        ],
        node_allowed_transitions: NODE_ALLOWED_TRANSITIONS
            .iter()
            .map(|(from, to)| format!("{from}->{to}"))
            .collect(),
        node_rejected_transitions: vec![
            "subagent->passed".to_string(),
            "waiting_decision->running_without_director".to_string(),
            "failed->running_without_retry_or_reopen".to_string(),
        ],
        completion_gate: director_completion_gate(
            task_packages.first(),
            review_results,
            exceptions,
        ),
        warnings: vec![
            "passed_not_equal_completed".to_string(),
            "subagent_and_review_agent_cannot_complete_node".to_string(),
            "waiting_decision_requires_manual_confirmation".to_string(),
        ],
    }
}

const WORKFLOW_ALLOWED_TRANSITIONS: &[(&str, &str)] = &[
    ("draft", "ready"),
    ("ready", "running"),
    ("running", "paused"),
    ("paused", "running"),
    ("running", "waiting_decision"),
    ("waiting_decision", "running"),
    ("waiting_decision", "archived"),
    ("running", "completed"),
    ("running", "failed"),
    ("completed", "archived"),
    ("failed", "running"),
    ("failed", "archived"),
    ("stopped", "archived"),
];

const NODE_ALLOWED_TRANSITIONS: &[(&str, &str)] = &[
    ("not_started", "waiting"),
    ("waiting", "running"),
    ("running", "waiting_permission"),
    ("waiting_permission", "running"),
    ("running", "waiting_decision"),
    ("waiting_decision", "running"),
    ("waiting_decision", "cancelled"),
    ("running", "reviewing"),
    ("reviewing", "passed"),
    ("reviewing", "returned"),
    ("returned", "running"),
    ("running", "failed"),
    ("failed", "running"),
    ("failed", "needs_rework"),
    ("failed", "archived"),
    ("needs_rework", "running"),
    ("needs_rework", "needs_rework"),
    ("needs_rework", "archived"),
    ("running", "paused"),
    ("paused", "running"),
    ("waiting", "skipped"),
];

fn workflow_transition_allowed(from: &str, to: &str, explicit_retry_or_reopen: bool) -> bool {
    if matches!(from, "failed" | "stopped") && to == "running" {
        return explicit_retry_or_reopen;
    }
    WORKFLOW_ALLOWED_TRANSITIONS
        .iter()
        .any(|(allowed_from, allowed_to)| *allowed_from == from && *allowed_to == to)
}

fn workflow_node_transition_allowed(
    from: &str,
    to: &str,
    actor_role: &str,
    explicit_retry_or_reopen: bool,
) -> bool {
    if to == "passed" && actor_role != "review" && actor_role != "project_director" {
        return false;
    }
    if from == "waiting_decision"
        && (to == "running" || to == "cancelled")
        && actor_role != "project_director"
    {
        return false;
    }
    if matches!(from, "failed" | "needs_rework") && to == "running" {
        return actor_role == "project_director" && explicit_retry_or_reopen;
    }
    if matches!(from, "failed" | "needs_rework")
        && (to == "needs_rework" || to == "archived")
    {
        return actor_role == "project_director";
    }
    NODE_ALLOWED_TRANSITIONS
        .iter()
        .any(|(allowed_from, allowed_to)| *allowed_from == from && *allowed_to == to)
}

fn director_completion_gate(
    task_package: Option<&TaskPackage>,
    review_results: &[ReviewResult],
    _exceptions: &[WorkflowException],
) -> DirectorCompletionGate {
    let required = vec![
        "task_goal_completed".to_string(),
        "acceptance_criteria_met".to_string(),
        "evidence_refs_exist".to_string(),
        "review_or_harness_passed_when_required".to_string(),
        "memory_candidate_step_recorded".to_string(),
        "final_user_report_need_recorded".to_string(),
    ];
    let mut missing = Vec::new();
    match task_package {
        Some(package) => {
            if package
                .task_goal
                .as_deref()
                .is_none_or(|goal| goal.trim().is_empty())
            {
                missing.push("task_goal_completed".to_string());
            }
            if package.acceptance_criteria.is_empty() {
                missing.push("acceptance_criteria_met".to_string());
            }
            if package.report_format.is_empty() {
                missing.push("final_user_report_need_recorded".to_string());
            }
            if package.available_memory_refs.is_empty() {
                missing.push("memory_candidate_step_recorded".to_string());
            }
            if !review_results
                .iter()
                .any(|result| result.result == "passed")
            {
                if package.harness_requirements.is_empty() {
                    missing.push("review_or_harness_passed_when_required".to_string());
                } else {
                    // 07-14 诚实化(L2 板4 小修):配置了 harness 要求≠harness 通过。结果机器
                    // 未建前,配置且无通过评审=如实标「已配置·结果未验证」,不再静默视为满足。
                    missing.push("harness_configured_result_unverified".to_string());
                }
            }
        }
        None => missing.extend([
            "task_goal_completed".to_string(),
            "acceptance_criteria_met".to_string(),
            "evidence_refs_exist".to_string(),
        ]),
    }
    if !review_results
        .iter()
        .any(|result| !result.evidence_refs.is_empty())
    {
        missing.push("evidence_refs_exist".to_string());
    }
    missing.sort();
    missing.dedup();
    DirectorCompletionGate {
        can_complete: missing.is_empty(),
        required,
        missing,
        warnings: vec![
            "only_project_director_can_mark_complete".to_string(),
            "passed_not_equal_completed".to_string(),
        ],
    }
}

fn workflow_interface_boundaries() -> WorkflowInterfaceBoundaries {
    WorkflowInterfaceBoundaries {
        proposal_interface: interface_boundary(
            "proposal_interface",
            vec!["explicit_proposal_refs", "director_decision_request"],
            vec!["subagent_direct_user_decision", "implicit_direction_change"],
        ),
        memory_candidate_interface: interface_boundary(
            "memory_candidate_interface",
            vec![
                "confirmed_memory_refs",
                "memory_candidates_after_director_summary",
            ],
            vec!["auto_write_formal_memory"],
        ),
        knowledge_refs_interface: interface_boundary(
            "knowledge_refs_interface",
            vec!["explicit_material_refs"],
            vec!["auto_scan_knowledge_base", "obsidian_native_without_design"],
        ),
        tool_capability_registry: interface_boundary(
            "tool_capability_registry",
            vec!["static_whitelist", "registered_tool_capabilities"],
            vec!["tool_without_whitelist", "tool_output_fulltext_in_ledger"],
        ),
        model_pool_selector: interface_boundary(
            "model_pool_selector",
            vec!["explicit_model_id"],
            vec!["silent_auto_model_selection"],
        ),
        harness_requirement_provider: interface_boundary(
            "harness_requirement_provider",
            vec!["run_check", "task_package_template", "completion_gate"],
            vec!["ordinary_workflow_node", "auto_run_harness"],
        ),
        audit_refs_interface: interface_boundary(
            "audit_refs_interface",
            vec!["summary_refs", "evidence_refs", "handoff_refs"],
            vec!["full_tool_output_in_workflow_ledger"],
        ),
        warnings: vec![
            "multiple_harness_conflict_policy_open".to_string(),
            "harness_failure_policy_open".to_string(),
            "harness_output_ui_detail_open".to_string(),
            "tool_call_summary_retention_granularity_open".to_string(),
        ],
    }
}

fn interface_boundary(
    interface_id: &str,
    allowed: Vec<&str>,
    blocked: Vec<&str>,
) -> InterfaceBoundary {
    InterfaceBoundary {
        interface_id: interface_id.to_string(),
        status: "conservative_stub".to_string(),
        allowed: allowed.into_iter().map(str::to_string).collect(),
        blocked: blocked.into_iter().map(str::to_string).collect(),
        source_refs: vec!["workflow-task-package-design-v1-confirmed-boundary".to_string()],
        warnings: vec![],
    }
}

fn workflow_acceptance_scenarios(
    task_packages: &[TaskPackage],
    subagent_reports: &[SubagentReport],
    review_results: &[ReviewResult],
    _exceptions: &[WorkflowException],
) -> Vec<WorkflowAcceptanceScenario> {
    vec![
        WorkflowAcceptanceScenario {
            scenario_id: "10.1".to_string(),
            title: "子智能体发现方向风险".to_string(),
            status: if subagent_reports
                .iter()
                .any(|report| !report.direction_risks.is_empty())
            {
                "covered_by_fixture".to_string()
            } else {
                "not_triggered_in_current_state".to_string()
            },
            expected: vec![
                "subagent_report_writes_direction_risk".to_string(),
                "node_enters_waiting_decision".to_string(),
                "subagent_does_not_ask_user_directly".to_string(),
            ],
            evidence_refs: subagent_reports
                .iter()
                .filter(|report| !report.direction_risks.is_empty())
                .map(|report| report.report_id.clone())
                .collect(),
            warnings: vec![],
        },
        WorkflowAcceptanceScenario {
            scenario_id: "10.2".to_string(),
            title: "任务包限制上下文".to_string(),
            status: if task_packages
                .iter()
                .any(|package| package.missing_fields.is_empty())
            {
                "covered_by_fixture".to_string()
            } else {
                "blocked_until_package_complete".to_string()
            },
            expected: vec![
                "explicit_memory_refs".to_string(),
                "explicit_knowledge_refs".to_string(),
                "explicit_tool_capabilities".to_string(),
                "missing_scope_means_not_allowed".to_string(),
            ],
            evidence_refs: task_packages
                .iter()
                .map(|package| package.task_package_id.clone())
                .collect(),
            warnings: vec![],
        },
        WorkflowAcceptanceScenario {
            scenario_id: "10.3".to_string(),
            title: "子智能体完成并汇报".to_string(),
            status: if subagent_reports.is_empty() {
                "not_triggered_in_current_state".to_string()
            } else {
                "covered_by_fixture".to_string()
            },
            expected: vec![
                "report_enters_workflow_ledger".to_string(),
                "memory_candidate_can_be_generated".to_string(),
                "formal_memory_not_written_automatically".to_string(),
            ],
            evidence_refs: subagent_reports
                .iter()
                .map(|report| report.report_id.clone())
                .collect(),
            warnings: vec![],
        },
        WorkflowAcceptanceScenario {
            scenario_id: "10.4".to_string(),
            title: "审查智能体通过".to_string(),
            status: if review_results
                .iter()
                .any(|result| result.result == "passed")
            {
                "covered_by_fixture".to_string()
            } else {
                "not_triggered_in_current_state".to_string()
            },
            expected: vec![
                "review_result_stored".to_string(),
                "project_director_still_marks_node_completion".to_string(),
            ],
            evidence_refs: review_results
                .iter()
                .map(|result| result.review_id.clone())
                .collect(),
            warnings: vec![],
        },
        WorkflowAcceptanceScenario {
            scenario_id: "10.5".to_string(),
            title: "harness 不是普通节点".to_string(),
            status: "covered_by_rules".to_string(),
            expected: vec![
                "harness_affects_run_check".to_string(),
                "harness_affects_task_package_template".to_string(),
                "harness_affects_completion_gate".to_string(),
                "harness_not_main_workflow_node".to_string(),
            ],
            evidence_refs: vec![
                "workflow_interface_boundaries.harness_requirement_provider".to_string()
            ],
            warnings: vec![],
        },
    ]
}

fn recent_audit_events_for(
    target_ref: &str,
    audit_events: &[Value],
    limit: usize,
) -> Vec<AuditEventSummary> {
    audit_events
        .iter()
        .rev()
        .filter(|event| optional_string_from(event, "target_ref").as_deref() == Some(target_ref))
        .take(limit)
        .map(|event| AuditEventSummary {
            event_id: optional_string_from(event, "event_id")
                .unwrap_or_else(|| "unknown".to_string()),
            event_type: optional_string_from(event, "event_type")
                .unwrap_or_else(|| "unknown".to_string()),
            before_state: optional_string_from(event, "before_state"),
            after_state: optional_string_from(event, "after_state"),
            created_at: optional_string_from(event, "created_at"),
            reason: optional_string_from(event, "reason"),
        })
        .collect()
}

fn workflow_node_session_binding_summaries(
    workflow_id: &str,
    bindings: &[Value],
) -> Vec<WorkflowNodeSessionBinding> {
    bindings
        .iter()
        .filter(|binding| {
            optional_string_from(binding, "workflow_id").as_deref() == Some(workflow_id)
                && optional_string_from(binding, "lifecycle").as_deref() == Some("active")
        })
        .filter_map(|binding| {
            Some(WorkflowNodeSessionBinding {
                binding_id: optional_string_from(binding, "binding_id")?,
                project_id: optional_string_from(binding, "project_id")?,
                workflow_id: optional_string_from(binding, "workflow_id")?,
                node_id: optional_string_from(binding, "node_id")?,
                work_item_id: optional_string_from(binding, "work_item_id"),
                agent_type: optional_string_from(binding, "agent_type")
                    .unwrap_or_else(|| "codex".to_string()),
                adapter_id: optional_string_from(binding, "adapter_id")
                    .unwrap_or_else(|| "codex-local".to_string()),
                native_thread_id: optional_string_from(binding, "native_thread_id")?,
                native_rollout_path: optional_string_from(binding, "native_rollout_path"),
                session_title: optional_string_from(binding, "session_title")
                    .unwrap_or_else(|| "未知标题".to_string()),
                session_updated_at_ms: i64_value(binding, "session_updated_at_ms"),
                rollout_exists: bool_value(binding, "rollout_exists"),
                project_binding_source: optional_string_from(binding, "project_binding_source")
                    .unwrap_or_else(|| "unknown".to_string()),
                binding_source: optional_string_from(binding, "binding_source")
                    .unwrap_or_else(|| "workflow_bound".to_string()),
                binding_mode: optional_string_from(binding, "binding_mode")
                    .unwrap_or_else(|| "select_existing_session".to_string()),
                lifecycle: optional_string_from(binding, "lifecycle")
                    .unwrap_or_else(|| "active".to_string()),
                created_at_ms: i64_value(binding, "created_at_ms").unwrap_or_default(),
                updated_at_ms: i64_value(binding, "updated_at_ms").unwrap_or_default(),
                warnings: string_array(binding, "warnings"),
            })
        })
        .collect()
}

fn workflow_node_dispatch_summaries(
    workflow_id: &str,
    dispatches: &[Value],
) -> Vec<WorkflowNodeDispatchRecord> {
    dispatches
        .iter()
        .rev()
        .filter(|dispatch| {
            optional_string_from(dispatch, "workflow_id").as_deref() == Some(workflow_id)
        })
        .filter_map(|dispatch| parse_workflow_node_dispatch_record(dispatch).ok())
        .take(5)
        .collect()
}

fn workflow_dispatch_director_review_summaries(
    workflow_id: &str,
    reviews: &[Value],
) -> Vec<WorkflowDispatchDirectorReviewRecord> {
    reviews
        .iter()
        .rev()
        .filter(|review| {
            optional_string_from(review, "workflow_id").as_deref() == Some(workflow_id)
                && optional_string_from(review, "reviewer_role").as_deref() == Some("director")
        })
        .filter_map(|review| parse_workflow_dispatch_director_review_record(review).ok())
        .take(5)
        .collect()
}

fn workflow_execution_control_summaries(
    workflow_id: &str,
    controls: &[Value],
) -> Vec<WorkflowExecutionControlRecord> {
    controls
        .iter()
        .rev()
        .filter(|control| {
            optional_string_from(control, "workflow_id").as_deref() == Some(workflow_id)
        })
        .filter_map(|control| parse_workflow_execution_control_record(control).ok())
        .take(10)
        .collect()
}

fn workflow_permission_request_summaries(
    workflow_id: &str,
    requests: &[Value],
) -> Vec<WorkflowPermissionRequestRecord> {
    requests
        .iter()
        .rev()
        .filter(|request| {
            optional_string_from(request, "workflow_id").as_deref() == Some(workflow_id)
        })
        .filter_map(|request| parse_workflow_permission_request_record(request).ok())
        .take(10)
        .collect()
}

fn workflow_execution_attempt_summaries(
    workflow_id: &str,
    attempts: &[Value],
) -> Vec<WorkflowExecutionAttemptRecord> {
    attempts
        .iter()
        .rev()
        .filter(|attempt| {
            optional_string_from(attempt, "workflow_id").as_deref() == Some(workflow_id)
        })
        .filter_map(|attempt| parse_workflow_execution_attempt_record(attempt).ok())
        .take(10)
        .collect()
}

fn parse_workflow_dispatch_director_review_record(
    value: &Value,
) -> Result<WorkflowDispatchDirectorReviewRecord, String> {
    Ok(WorkflowDispatchDirectorReviewRecord {
        review_id: optional_string_from(value, "review_id")
            .ok_or_else(|| "review 缺 review_id".to_string())?,
        project_id: optional_string_from(value, "project_id")
            .ok_or_else(|| "review 缺 project_id".to_string())?,
        workflow_id: optional_string_from(value, "workflow_id")
            .ok_or_else(|| "review 缺 workflow_id".to_string())?,
        work_item_id: optional_string_from(value, "work_item_id")
            .ok_or_else(|| "review 缺 work_item_id".to_string())?,
        dispatch_id: optional_string_from(value, "dispatch_id")
            .ok_or_else(|| "review 缺 dispatch_id".to_string())?,
        reviewer_role: optional_string_from(value, "reviewer_role")
            .unwrap_or_else(|| "director".to_string()),
        decision: optional_string_from(value, "decision").unwrap_or_else(|| "unknown".to_string()),
        summary: optional_string_from(value, "summary").unwrap_or_default(),
        evidence_refs: string_array(value, "evidence_refs"),
        handoff_refs: string_array(value, "handoff_refs"),
        created_at: optional_string_from(value, "created_at").unwrap_or_default(),
        updated_at: optional_string_from(value, "updated_at").unwrap_or_default(),
        warnings: string_array(value, "warnings"),
    })
}

fn parse_workflow_execution_control_record(
    value: &Value,
) -> Result<WorkflowExecutionControlRecord, String> {
    Ok(WorkflowExecutionControlRecord {
        control_id: optional_string_from(value, "control_id")
            .ok_or_else(|| "execution control 缺 control_id".to_string())?,
        project_id: optional_string_from(value, "project_id")
            .ok_or_else(|| "execution control 缺 project_id".to_string())?,
        workflow_id: optional_string_from(value, "workflow_id")
            .ok_or_else(|| "execution control 缺 workflow_id".to_string())?,
        work_item_id: optional_string_from(value, "work_item_id")
            .ok_or_else(|| "execution control 缺 work_item_id".to_string())?,
        control_state: optional_string_from(value, "control_state")
            .unwrap_or_else(|| "not_started".to_string()),
        long_task_state: optional_string_from(value, "long_task_state")
            .unwrap_or_else(|| "not_started".to_string()),
        retry_count: i64_value(value, "retry_count").unwrap_or_default().max(0) as usize,
        max_retries: i64_value(value, "max_retries").unwrap_or_default().max(0) as usize,
        timeout_seconds: i64_value(value, "timeout_seconds"),
        cancel_requested_at: optional_string_from(value, "cancel_requested_at"),
        failure_reason: optional_string_from(value, "failure_reason"),
        user_reviewed_instruction: value
            .get("user_reviewed_instruction")
            .and_then(|instruction| parse_workflow_user_reviewed_instruction(instruction).ok()),
        audit_event_types: string_array(value, "audit_event_types"),
        warnings: string_array(value, "warnings"),
    })
}

fn parse_workflow_user_reviewed_instruction(
    value: &Value,
) -> Result<WorkflowUserReviewedInstruction, String> {
    let summary = optional_string_from(value, "summary").unwrap_or_default();
    let objective = optional_string_from(value, "objective").unwrap_or_default();
    let execution_cwd = optional_string_from(value, "execution_cwd").unwrap_or_default();
    let sandbox_mode = optional_string_from(value, "sandbox_mode").unwrap_or_default();
    let allowed_write_roots = string_array(value, "allowed_write_roots");
    let allowed_reads = string_array(value, "allowed_reads");
    let allowed_writes = string_array(value, "allowed_writes");
    let forbidden_actions = string_array(value, "forbidden_actions");
    let required_return = string_array(value, "required_return");
    let preview_markdown = optional_string_from(value, "preview_markdown").unwrap_or_else(|| {
        render_user_reviewed_instruction_preview(
            &summary,
            &objective,
            &execution_cwd,
            &sandbox_mode,
            &allowed_write_roots,
            &allowed_reads,
            &allowed_writes,
            &forbidden_actions,
            &required_return,
        )
    });
    Ok(WorkflowUserReviewedInstruction {
        instruction_id: optional_string_from(value, "instruction_id")
            .ok_or_else(|| "user reviewed instruction 缺 instruction_id".to_string())?,
        summary,
        objective,
        execution_cwd,
        sandbox_mode,
        allowed_write_roots,
        allowed_reads,
        allowed_writes,
        forbidden_actions,
        required_return,
        approval_state: optional_string_from(value, "approval_state")
            .unwrap_or_else(|| "draft".to_string()),
        preview_markdown,
    })
}

fn parse_workflow_permission_request_record(
    value: &Value,
) -> Result<WorkflowPermissionRequestRecord, String> {
    Ok(WorkflowPermissionRequestRecord {
        request_id: optional_string_from(value, "request_id")
            .ok_or_else(|| "permission request 缺 request_id".to_string())?,
        project_id: optional_string_from(value, "project_id")
            .ok_or_else(|| "permission request 缺 project_id".to_string())?,
        workflow_id: optional_string_from(value, "workflow_id")
            .ok_or_else(|| "permission request 缺 workflow_id".to_string())?,
        work_item_id: optional_string_from(value, "work_item_id")
            .ok_or_else(|| "permission request 缺 work_item_id".to_string())?,
        dispatch_id: optional_string_from(value, "dispatch_id"),
        permission_kind: optional_string_from(value, "permission_kind")
            .unwrap_or_else(|| "unknown".to_string()),
        reason: optional_string_from(value, "reason").unwrap_or_default(),
        status: optional_string_from(value, "status").unwrap_or_else(|| "pending".to_string()),
        requested_at: optional_string_from(value, "requested_at").unwrap_or_default(),
        decided_at: optional_string_from(value, "decided_at"),
        decision: optional_string_from(value, "decision"),
        warnings: string_array(value, "warnings"),
    })
}

fn parse_workflow_execution_attempt_record(
    value: &Value,
) -> Result<WorkflowExecutionAttemptRecord, String> {
    Ok(WorkflowExecutionAttemptRecord {
        attempt_id: optional_string_from(value, "attempt_id")
            .ok_or_else(|| "execution attempt 缺 attempt_id".to_string())?,
        project_id: optional_string_from(value, "project_id")
            .ok_or_else(|| "execution attempt 缺 project_id".to_string())?,
        workflow_id: optional_string_from(value, "workflow_id")
            .ok_or_else(|| "execution attempt 缺 workflow_id".to_string())?,
        work_item_id: optional_string_from(value, "work_item_id")
            .ok_or_else(|| "execution attempt 缺 work_item_id".to_string())?,
        dispatch_id: optional_string_from(value, "dispatch_id"),
        attempt_no: i64_value(value, "attempt_no").unwrap_or(1).max(1) as usize,
        state: optional_string_from(value, "state").unwrap_or_else(|| "unknown".to_string()),
        started_at: optional_string_from(value, "started_at"),
        ended_at: optional_string_from(value, "ended_at"),
        failure_reason: optional_string_from(value, "failure_reason"),
        retry_scheduled_at: optional_string_from(value, "retry_scheduled_at"),
        timed_out_at: optional_string_from(value, "timed_out_at"),
        cancel_requested_at: optional_string_from(value, "cancel_requested_at"),
        warnings: string_array(value, "warnings"),
    })
}

fn parse_workflow_node_dispatch_record(
    value: &Value,
) -> Result<WorkflowNodeDispatchRecord, String> {
    Ok(WorkflowNodeDispatchRecord {
        dispatch_id: optional_string_from(value, "dispatch_id")
            .ok_or_else(|| "dispatch 缺 dispatch_id".to_string())?,
        project_id: optional_string_from(value, "project_id")
            .ok_or_else(|| "dispatch 缺 project_id".to_string())?,
        workflow_id: optional_string_from(value, "workflow_id")
            .ok_or_else(|| "dispatch 缺 workflow_id".to_string())?,
        node_id: optional_string_from(value, "node_id")
            .ok_or_else(|| "dispatch 缺 node_id".to_string())?,
        work_item_id: optional_string_from(value, "work_item_id")
            .ok_or_else(|| "dispatch 缺 work_item_id".to_string())?,
        binding_id: optional_string_from(value, "binding_id")
            .ok_or_else(|| "dispatch 缺 binding_id".to_string())?,
        native_thread_id: optional_string_from(value, "native_thread_id")
            .ok_or_else(|| "dispatch 缺 native_thread_id".to_string())?,
        prompt_preview: optional_string_from(value, "prompt_preview").unwrap_or_default(),
        prompt_kind: optional_string_from(value, "prompt_kind")
            .unwrap_or_else(|| "safe_probe".to_string()),
        memory_packet_snapshot_id: optional_string_from(value, "memory_packet_snapshot_id"),
        memory_packet_fingerprint: optional_string_from(value, "memory_packet_fingerprint"),
        plan_authorization_id: optional_string_from(value, "plan_authorization_id"),
        authorization_check: value
            .get("authorization_check")
            .and_then(|check| serde_json::from_value(check.clone()).ok()),
        offline_role_dispatch: value
            .get("offline_role_dispatch")
            .and_then(|dispatch| serde_json::from_value(dispatch.clone()).ok()),
        user_reviewed_instruction: value
            .get("user_reviewed_instruction")
            .and_then(|instruction| parse_workflow_user_reviewed_instruction(instruction).ok()),
        state: optional_string_from(value, "state").unwrap_or_else(|| "unknown".to_string()),
        started_at_ms: i64_value(value, "started_at_ms"),
        ended_at_ms: i64_value(value, "ended_at_ms"),
        exit_code: i64_value(value, "exit_code").map(|code| code as i32),
        last_message_path: optional_string_from(value, "last_message_path"),
        last_message_summary: optional_string_from(value, "last_message_summary"),
        transcript_event_count: i64_value(value, "transcript_event_count")
            .map(|count| count as usize),
        transcript_target_hits: i64_value(value, "transcript_target_hits")
            .map(|count| count as usize),
        warnings: string_array(value, "warnings"),
    })
}

fn dispatch_result_from_state(
    path: &Path,
    backup_path: Option<PathBuf>,
    audit_event_id: &str,
    dispatch_id: &str,
    message: &str,
) -> Result<WorkflowNodeDispatchResult, String> {
    let snapshot = read_workflow_state_snapshot(path)?;
    let value = read_workflow_state_value(path)?;
    let dispatch = parse_workflow_node_dispatch_record(
        find_workflow_node_dispatch(&value, dispatch_id)
            .ok_or_else(|| "写入派发记录后重新读取失败".to_string())?,
    )?;
    Ok(WorkflowNodeDispatchResult {
        message: message.to_string(),
        path: path.display().to_string(),
        backup_path: backup_path.map(|path| path.display().to_string()),
        audit_event_id: audit_event_id.to_string(),
        product_command_boundary: legacy_product_command_boundary("workflow_node_dispatch_result"),
        dispatch,
        snapshot,
    })
}

fn dispatch_readback_stats(
    index: Option<&Value>,
    readback_db_path: &Path,
    context: &WorkflowNodeDispatchContext,
) -> Result<CodexDispatchReadbackStats, String> {
    dispatch_readback_stats_native(
        index,
        readback_db_path,
        &context.native_thread_id,
        safe_probe_target(),
    )
}

fn dispatch_readback_stats_native(
    index: Option<&Value>,
    readback_db_path: &Path,
    thread_id: &str,
    target: &str,
) -> Result<CodexDispatchReadbackStats, String> {
    #[cfg(test)]
    DISPATCH_READBACK_NATIVE_READ_COUNT.with(|count| count.set(count.get() + 1));

    let transcript = load_codex_session_transcript_with_optional_catalog(
        index,
        thread_id,
        readback_db_path,
        None,
    );
    match transcript {
        Ok(transcript) => Ok(dispatch_readback_stats_from_transcript(&transcript, target)),
        Err(_) => Ok(CodexDispatchReadbackStats {
            transcript_event_count: 0,
            transcript_target_hits: 0,
        }),
    }
}

fn dispatch_readback_stats_from_transcript(
    transcript: &CodexTranscript,
    target: &str,
) -> CodexDispatchReadbackStats {
    CodexDispatchReadbackStats {
        transcript_event_count: transcript.summary.total_events,
        transcript_target_hits: transcript
            .events
            .iter()
            .filter(|event| {
                event
                    .text
                    .as_deref()
                    .is_some_and(|text| text.contains(target))
                    || event
                        .stdout
                        .as_deref()
                        .is_some_and(|text| text.contains(target))
            })
            .count(),
    }
}

#[cfg(test)]
thread_local! {
    static DISPATCH_READBACK_NATIVE_READ_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}
