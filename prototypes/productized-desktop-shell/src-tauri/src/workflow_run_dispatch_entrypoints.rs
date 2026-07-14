// Workflow run, binding, and legacy dispatch entrypoints split out during Root Treatment R2-B4.
// This file is included at crate root so helper visibility and behavior stay unchanged.

fn inspect_workflow_run_check_at(
    path: &Path,
    project: &ProjectRecord,
    request: &WorkflowRunCheckRequest,
) -> Result<WorkflowRunCheck, String> {
    if !path.exists() {
        return Ok(blocked_workflow_run_check(
            &project.project_root,
            request.workflow_id.as_deref(),
            "missing_workflow",
            "没有工作流",
            "工作流状态文件不存在；不会自动创建或补编。",
        ));
    }

    let value = read_workflow_state_value(path)?;
    let validation_warnings = validate_workflow_state(&value);
    let workflow_id = request
        .workflow_id
        .clone()
        .unwrap_or_else(|| default_workflow_id(&request.project_root));
    let workflow = value
        .get("workflows")
        .and_then(Value::as_array)
        .and_then(|workflows| {
            workflows.iter().find(|workflow| {
                optional_string_from(workflow, "workflow_id").as_deref()
                    == Some(workflow_id.as_str())
            })
        });
    let Some(workflow) = workflow else {
        let mut check = blocked_workflow_run_check(
            &project.project_root,
            Some(&workflow_id),
            "missing_workflow",
            "没有工作流",
            "当前项目没有匹配的 workflow；不能运行、派发或标记准备完成。",
        );
        check.warnings.extend(validation_warnings);
        return Ok(check);
    };

    let nodes = value
        .get("nodes")
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
    let bindings = value
        .get("workflow_node_session_bindings")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut check = inspect_workflow_run_check_from_value(
        &project.project_root,
        Some(&workflow_id),
        workflow,
        &nodes,
        &work_items,
        &artifacts,
        &bindings,
    );
    check.warnings.extend(validation_warnings);
    Ok(check)
}

fn inspect_workflow_run_check_from_value(
    project_root: &str,
    workflow_id: Option<&str>,
    workflow: &Value,
    nodes: &[Value],
    work_items: &[Value],
    artifacts: &[Value],
    bindings: &[Value],
) -> WorkflowRunCheck {
    let workflow_id_string = workflow_id.map(str::to_string);
    let mut checks = Vec::new();
    let workflow_id_value = workflow_id.unwrap_or_default();

    let has_director = optional_string_from(workflow, "owner_role")
        .is_some_and(|role| !role.trim().is_empty())
        || nodes.iter().any(|node| {
            optional_string_from(node, "workflow_id").as_deref() == Some(workflow_id_value)
                && (optional_string_from(node, "node_type").as_deref() == Some("director")
                    || optional_string_from(node, "node_id")
                        .is_some_and(|node_id| node_id.ends_with(":node:director")))
        });
    push_check(
        &mut checks,
        "missing_owner",
        "项目主管",
        if has_director { "pass" } else { "blocked" },
        if has_director {
            "工作流存在 director 节点或 owner_role。"
        } else {
            "没有项目主管；不能运行、派发或标记准备完成。"
        },
        workflow_id,
    );

    let workflow_items = work_items
        .iter()
        .filter(|item| {
            optional_string_from(item, "workflow_id").as_deref() == Some(workflow_id_value)
        })
        .collect::<Vec<_>>();
    if workflow_items.is_empty() {
        push_check(
            &mut checks,
            "missing_work_item",
            "工作项",
            "warning",
            "当前 workflow 还没有工作项；可以查看和编辑草稿，但不能派发。",
            workflow_id,
        );
    }

    for work_item in &workflow_items {
        let work_item_id = optional_string_from(work_item, "work_item_id")
            .unwrap_or_else(|| "work-item:missing".to_string());
        let artifact =
            find_task_package_artifact_for_work_item(artifacts, &work_item_id, work_item);
        let assigned_role = optional_string_from(work_item, "assigned_role_id");
        let dispatch_node_id = assigned_role
            .as_deref()
            .map(|role| format!("{workflow_id_value}:node:{role}"))
            .or_else(|| optional_string_from(work_item, "current_node_id"));
        let has_binding = dispatch_node_id.as_deref().is_some_and(|node_id| {
            bindings.iter().any(|binding| {
                optional_string_from(binding, "workflow_id").as_deref() == Some(workflow_id_value)
                    && optional_string_from(binding, "node_id").as_deref() == Some(node_id)
                    && optional_string_from(binding, "lifecycle").as_deref() == Some("active")
                    && bool_value(binding, "rollout_exists")
            })
        });
        push_check(
            &mut checks,
            "missing_session",
            "绑定会话",
            if has_binding { "pass" } else { "blocked" },
            if has_binding {
                "要派发的节点已有 active 会话绑定。"
            } else {
                "要派发的节点没有 active 会话绑定。"
            },
            Some(&work_item_id),
        );

        let model_id = artifact.and_then(|artifact| optional_string_from(artifact, "model_id"));
        push_check(
            &mut checks,
            "missing_model",
            "模型",
            if model_id
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
            {
                "pass"
            } else {
                "blocked"
            },
            if model_id.is_some() {
                "任务包已显式指定模型。"
            } else {
                "缺模型；系统不会自动选择模型。"
            },
            Some(&work_item_id),
        );

        let allowed_read = artifact
            .map(|artifact| string_array(artifact, "allowed_read_scope"))
            .unwrap_or_default();
        push_check(
            &mut checks,
            "missing_read_scope",
            "读取范围",
            if allowed_read.is_empty() {
                "blocked"
            } else {
                "pass"
            },
            if allowed_read.is_empty() {
                "没有读范围；不能运行。"
            } else {
                "任务包已登记允许读取范围。"
            },
            Some(&work_item_id),
        );

        let allowed_write = artifact
            .map(|artifact| string_array(artifact, "allowed_write"))
            .unwrap_or_default();
        push_check(
            &mut checks,
            "missing_write_scope",
            "写入范围",
            if allowed_write.is_empty() {
                "blocked"
            } else {
                "pass"
            },
            if allowed_write.is_empty() {
                "会写文件但没有写范围；不能运行。"
            } else {
                "任务包已登记允许写入范围。"
            },
            Some(&work_item_id),
        );

        let acceptance = artifact
            .map(|artifact| string_array(artifact, "acceptance_criteria"))
            .unwrap_or_default();
        push_check(
            &mut checks,
            "missing_acceptance_criteria",
            "验收标准",
            if acceptance.is_empty() {
                "blocked"
            } else {
                "pass"
            },
            if acceptance.is_empty() {
                "没有验收标准；不能运行。"
            } else {
                "任务包已登记验收标准。"
            },
            Some(&work_item_id),
        );

        let requires_tools =
            artifact.is_some_and(|artifact| bool_value(artifact, "requires_tools"));
        let tools = artifact
            .map(|artifact| string_array(artifact, "callable_tool_capabilities"))
            .unwrap_or_default();
        push_check(
            &mut checks,
            "missing_tool_whitelist",
            "工具白名单",
            if requires_tools && tools.is_empty() {
                "blocked"
            } else if tools.is_empty() {
                "warning"
            } else {
                "pass"
            },
            if requires_tools && tools.is_empty() {
                "节点声明需要工具但没有工具白名单。"
            } else if tools.is_empty() {
                "节点没有声明工具；工具白名单为空。"
            } else {
                "任务包已登记可调用工具白名单。"
            },
            Some(&work_item_id),
        );

        let requires_harness =
            artifact.is_some_and(|artifact| bool_value(artifact, "requires_harness"));
        let harness = artifact
            .map(|artifact| string_array(artifact, "harness_requirements"))
            .unwrap_or_default();
        push_check(
            &mut checks,
            "missing_harness_requirement",
            "Harness",
            if requires_harness && harness.is_empty() {
                "blocked"
            } else if harness.is_empty() {
                "warning"
            } else {
                "pass"
            },
            if requires_harness && harness.is_empty() {
                "节点要求 harness 但没有配置。"
            } else if harness.is_empty() {
                "节点未要求 harness；harness 要求为空。"
            } else {
                "任务包已登记 harness 要求。"
            },
            Some(&work_item_id),
        );

        let forbidden = artifact
            .map(|artifact| string_array(artifact, "forbidden_actions"))
            .unwrap_or_default();
        let policy_blocked = forbidden
            .iter()
            .any(|line| contains_conflicting_generation_ban(line))
            || allowed_write
                .iter()
                .any(|line| line.contains("/Users/yoyi/.codex"));
        push_check(
            &mut checks,
            "policy_violation",
            "权限冲突",
            if policy_blocked { "blocked" } else { "pass" },
            if policy_blocked {
                "存在权限冲突或历史禁令冲突。"
            } else {
                "没有发现权限冲突。"
            },
            Some(&work_item_id),
        );

        let requires_knowledge =
            artifact.is_some_and(|artifact| bool_value(artifact, "requires_knowledge_refs"));
        let knowledge = artifact
            .map(|artifact| string_array(artifact, "available_knowledge_refs"))
            .unwrap_or_default();
        if requires_knowledge && knowledge.is_empty() {
            push_check(
                &mut checks,
                "missing_knowledge_refs",
                "知识库引用",
                "blocked",
                "任务包声明需要知识库引用，但没有显式引用。",
                Some(&work_item_id),
            );
        }
        let requires_memory =
            artifact.is_some_and(|artifact| bool_value(artifact, "requires_memory_refs"));
        let memory = artifact
            .map(|artifact| string_array(artifact, "available_memory_refs"))
            .unwrap_or_default();
        if requires_memory && memory.is_empty() {
            push_check(
                &mut checks,
                "missing_memory_refs",
                "记忆引用",
                "blocked",
                "任务包声明需要记忆作为依据，但没有确认记忆引用。",
                Some(&work_item_id),
            );
        }
    }

    let blocked_reasons = checks
        .iter()
        .filter(|check| check.status == "blocked")
        .map(|check| check.reason.clone())
        .collect::<Vec<_>>();
    let warnings = checks
        .iter()
        .filter(|check| check.status == "warning")
        .map(|check| check.reason.clone())
        .collect::<Vec<_>>();
    let status = if !blocked_reasons.is_empty() {
        "blocked"
    } else if !warnings.is_empty() {
        "warning"
    } else {
        "runnable"
    };

    WorkflowRunCheck {
        project_root: project_root.to_string(),
        workflow_id: workflow_id_string,
        status: status.to_string(),
        checks,
        blocked_reasons,
        warnings,
        evidence_completeness: if status == "runnable" {
            "complete".to_string()
        } else {
            "missing".to_string()
        },
    }
}

fn blocked_workflow_run_check(
    project_root: &str,
    workflow_id: Option<&str>,
    check_id: &str,
    label: &str,
    reason: &str,
) -> WorkflowRunCheck {
    let item = WorkflowRunCheckItem {
        check_id: check_id.to_string(),
        label: label.to_string(),
        status: "blocked".to_string(),
        severity: "blocked".to_string(),
        reason: reason.to_string(),
        source_ref: workflow_id.map(str::to_string),
    };
    WorkflowRunCheck {
        project_root: project_root.to_string(),
        workflow_id: workflow_id.map(str::to_string),
        status: "blocked".to_string(),
        checks: vec![item],
        blocked_reasons: vec![reason.to_string()],
        warnings: vec![],
        evidence_completeness: "missing".to_string(),
    }
}

fn push_check(
    checks: &mut Vec<WorkflowRunCheckItem>,
    check_id: &str,
    label: &str,
    status: &str,
    reason: &str,
    source_ref: Option<&str>,
) {
    checks.push(WorkflowRunCheckItem {
        check_id: check_id.to_string(),
        label: label.to_string(),
        status: status.to_string(),
        severity: status.to_string(),
        reason: reason.to_string(),
        source_ref: source_ref.map(str::to_string),
    });
}

fn find_task_package_artifact_for_work_item<'a>(
    artifacts: &'a [Value],
    work_item_id: &str,
    work_item: &Value,
) -> Option<&'a Value> {
    let source_ref = optional_string_from(work_item, "source_ref");
    artifacts.iter().find(|artifact| {
        optional_string_from(artifact, "artifact_type").as_deref() == Some("task_package")
            && (optional_string_from(artifact, "source_ref").as_deref() == Some(work_item_id)
                || source_ref.as_deref().is_some_and(|source_ref| {
                    optional_string_from(artifact, "artifact_id").as_deref() == Some(source_ref)
                }))
    })
}

fn update_work_item_state_at(
    path: &Path,
    request: &WorkItemStateUpdateRequest,
) -> Result<WorkflowStateMutationResult, String> {
    if !path.exists() {
        return Err("工作流状态文件不存在；无法推进工作项状态".to_string());
    }
    if let Some(repository) =
        crate::workbench_sqlite_storage_mode::primary_repository_for_write(path)?
    {
        return update_work_item_state_db_primary(path, request, &repository);
    }

    let timestamp = unix_timestamp_string();
    let mut value = read_workflow_state_value(path)?;
    let validation_warnings = validate_workflow_state(&value);
    if !validation_warnings.is_empty() {
        return Err(format!(
            "当前状态文件未通过 schema 校验：{}",
            validation_warnings.join(", ")
        ));
    }

    let workflow_id = default_workflow_id(&request.project_root);
    if !workflow_exists(&value, &workflow_id) {
        return Err("当前项目还没有本地 workflow；无法推进工作项状态".to_string());
    }

    let work_item_index = find_work_item_index(&value, &workflow_id, &request.work_item_id)
        .ok_or_else(|| "当前 workflow 下找不到该 work item；无法推进工作项状态".to_string())?;
    let before_state = value
        .get("work_items")
        .and_then(Value::as_array)
        .and_then(|items| items.get(work_item_index))
        .and_then(|item| optional_string_from(item, "state"))
        .unwrap_or_else(|| "draft".to_string());
    let next_state = request.next_state.trim();
    control_core::validate_work_item_state_transition(&before_state, next_state)?;

    let backup = crate::workflow_state_store::backup_file(path, &timestamp)?;

    let current_node_id = workflow_node_for_work_item_state(&workflow_id, next_state);
    {
        let work_items = array_mut(&mut value, "work_items")?;
        let work_item = work_items
            .get_mut(work_item_index)
            .ok_or_else(|| "当前 workflow 下找不到该 work item；无法推进工作项状态".to_string())?;
        work_item["state"] = Value::String(next_state.to_string());
        work_item["current_node_id"] = Value::String(current_node_id.clone());
        work_item["updated_at"] = Value::String(timestamp.clone());
    }
    update_node_state_for_id(&mut value, &current_node_id, next_state, &timestamp)?;

    let audit_event_id = crate::workflow_audit::audit_event_identity(
        "work-item-state",
        &request.work_item_id,
        &timestamp,
    );
    array_mut(&mut value, "audit_events")?.push(workflow_audit::work_item_state_changed(
        workflow_audit::WorkItemStateChangedAudit {
            event_id: audit_event_id.clone(),
            work_item_id: &request.work_item_id,
            before_state: &before_state,
            after_state: next_state,
            created_at: &timestamp,
            reason: format!(
                "用户确认推进工作项状态到：{}",
                work_item_state_label(next_state)
            ),
        },
    ));

    value["updated_at"] = Value::String(timestamp);
    let validation_warnings = validate_workflow_state(&value);
    if !validation_warnings.is_empty() {
        return Err(format!(
            "写入前 schema 校验失败：{}",
            validation_warnings.join(", ")
        ));
    }

    write_validated_workflow_state(path, &value)?;
    let snapshot = read_workflow_state_snapshot(path)?;
    if !snapshot.exists {
        return Err("推进工作项状态后重新读取校验失败".to_string());
    }

    Ok(WorkflowStateMutationResult {
        message: format!(
            "已推进工作项状态：{} -> {}",
            work_item_state_label(&before_state),
            work_item_state_label(next_state)
        ),
        path: path.display().to_string(),
        backup_path: Some(backup.display().to_string()),
        audit_event_id,
        first_initialize: false,
        snapshot,
    })
}

fn update_work_item_state_db_primary(
    path: &Path,
    request: &WorkItemStateUpdateRequest,
    repository: &crate::workbench_sqlite_repository::WorkbenchSqliteRepository,
) -> Result<WorkflowStateMutationResult, String> {
    let timestamp = unix_timestamp_string();
    let mut value = read_workflow_state_value(path)?;
    let validation_warnings = validate_workflow_state(&value);
    if !validation_warnings.is_empty() {
        return Err(format!(
            "当前状态文件未通过 schema 校验：{}",
            validation_warnings.join(", ")
        ));
    }

    let workflow_id = default_workflow_id(&request.project_root);
    if !workflow_exists(&value, &workflow_id) {
        return Err("当前项目还没有本地 workflow；无法推进工作项状态".to_string());
    }

    let work_item_index = find_work_item_index(&value, &workflow_id, &request.work_item_id)
        .ok_or_else(|| "当前 workflow 下找不到该 work item；无法推进工作项状态".to_string())?;
    let before_state = value
        .get("work_items")
        .and_then(Value::as_array)
        .and_then(|items| items.get(work_item_index))
        .and_then(|item| optional_string_from(item, "state"))
        .unwrap_or_else(|| "draft".to_string());
    let next_state = request.next_state.trim();
    control_core::validate_work_item_state_transition(&before_state, next_state)?;

    let current_node_id = workflow_node_for_work_item_state(&workflow_id, next_state);
    {
        let work_items = array_mut(&mut value, "work_items")?;
        let work_item = work_items
            .get_mut(work_item_index)
            .ok_or_else(|| "当前 workflow 下找不到该 work item；无法推进工作项状态".to_string())?;
        work_item["state"] = Value::String(next_state.to_string());
        work_item["current_node_id"] = Value::String(current_node_id.clone());
        work_item["updated_at"] = Value::String(timestamp.clone());
    }
    update_node_state_for_id(&mut value, &current_node_id, next_state, &timestamp)?;

    let audit_event_id = crate::workflow_audit::audit_event_identity(
        "work-item-state",
        &request.work_item_id,
        &timestamp,
    );
    let audit_event =
        workflow_audit::work_item_state_changed(workflow_audit::WorkItemStateChangedAudit {
            event_id: audit_event_id.clone(),
            work_item_id: &request.work_item_id,
            before_state: &before_state,
            after_state: next_state,
            created_at: &timestamp,
            reason: format!(
                "用户确认推进工作项状态到：{}",
                work_item_state_label(next_state)
            ),
        });
    array_mut(&mut value, "audit_events")?.push(audit_event.clone());

    value["updated_at"] = Value::String(timestamp.clone());
    let validation_warnings = validate_workflow_state(&value);
    if !validation_warnings.is_empty() {
        return Err(format!(
            "写入前 schema 校验失败：{}",
            validation_warnings.join(", ")
        ));
    }
    let work_item_after = value
        .get("work_items")
        .and_then(Value::as_array)
        .and_then(|items| items.get(work_item_index))
        .cloned()
        .ok_or_else(|| "DB 主写找不到更新后的 work item".to_string())?;
    let node_after = value
        .get("nodes")
        .and_then(Value::as_array)
        .and_then(|nodes| {
            nodes.iter().find(|node| {
                optional_string_from(node, "node_id").as_deref() == Some(current_node_id.as_str())
            })
        })
        .cloned()
        .ok_or_else(|| "DB 主写找不到更新后的 workflow node".to_string())?;
    repository.transition_work_item_with_audit(
        &work_item_after,
        &node_after,
        &before_state,
        &crate::workbench_sqlite_repository::RepositoryAuditEntry {
            event_id: audit_event_id.clone(),
            target_kind: "workflow_state".to_string(),
            target_id: request.work_item_id.clone(),
            payload: audit_event,
        },
        None,
    )?;
    crate::workbench_sqlite_storage_mode::complete_db_primary_json_projection(
        path,
        "work_item_state_transition",
        || {
            let backup = crate::workflow_state_store::backup_file(path, &timestamp)?;
            write_validated_workflow_state(path, &value)?;
            let snapshot = read_workflow_state_snapshot(path)?;
            if !snapshot.exists {
                return Err("推进工作项状态后重新读取校验失败".to_string());
            }

            Ok(WorkflowStateMutationResult {
                message: format!(
                    "已推进工作项状态：{} -> {}",
                    work_item_state_label(&before_state),
                    work_item_state_label(next_state)
                ),
                path: path.display().to_string(),
                backup_path: Some(backup.display().to_string()),
                audit_event_id,
                first_initialize: false,
                snapshot,
            })
        },
    )
}

struct WorkflowNodeSessionBindingProvenance {
    binding_source: &'static str,
    binding_mode: &'static str,
    actor_ref: String,
    permission_level: &'static str,
    reason: String,
}

impl WorkflowNodeSessionBindingProvenance {
    fn user_selected_existing() -> Self {
        Self {
            binding_source: "workflow_bound",
            binding_mode: "select_existing_session",
            actor_ref: "user_confirmed_desktop_shell".to_string(),
            permission_level: "user_confirmed_write",
            reason: "用户确认把已有 Codex 会话绑定到工作流节点；没有启动 Codex、没有发送消息、没有读取 transcript 正文。".to_string(),
        }
    }

    fn fresh_task_session(requested_by: &str) -> Self {
        let supervisor = requested_by == "supervisor_orchestrator";
        Self {
            binding_source: "fresh_task_session_bound",
            binding_mode: "create_fresh_task_session",
            actor_ref: requested_by.to_string(),
            permission_level: if supervisor {
                "authorized_supervisor_execution"
            } else {
                "workflow_director_execution"
            },
            reason: if supervisor {
                "Syn 控制核心为已授权主管任务创建全新 Codex 会话，并精确绑定到当前 work item。"
                    .to_string()
            } else {
                "工作流主管为任务创建全新 Codex 会话，并精确绑定到当前 work item。".to_string()
            },
        }
    }
}

fn bind_workflow_node_codex_session_at(
    path: &Path,
    request: &WorkflowNodeSessionBindRequest,
    session: &SessionRecord,
) -> Result<WorkflowStateMutationResult, String> {
    bind_workflow_node_codex_session_with_provenance_at(
        path,
        request,
        session,
        &WorkflowNodeSessionBindingProvenance::user_selected_existing(),
    )
}

fn bind_workflow_node_codex_session_with_provenance_at(
    path: &Path,
    request: &WorkflowNodeSessionBindRequest,
    session: &SessionRecord,
    provenance: &WorkflowNodeSessionBindingProvenance,
) -> Result<WorkflowStateMutationResult, String> {
    if !path.exists() {
        return Err("工作流状态文件不存在；无法绑定节点会话".to_string());
    }

    let timestamp = unix_timestamp_string();
    let timestamp_ms = unix_timestamp_ms();
    migrate_legacy_workflow_node_session_binding_ids_at(path)?;
    let mut value = read_workflow_state_value(path)?;
    ensure_workflow_node_session_bindings_array(&mut value)?;
    let validation_warnings = validate_workflow_state(&value);
    if !validation_warnings.is_empty() {
        return Err(format!(
            "当前状态文件未通过 schema 校验：{}",
            validation_warnings.join(", ")
        ));
    }

    // 后置C#2：workflow_id 从 node_id（`{workflow_id}:node:…`）解析、退回 default——让非默认工作流
    // 的节点也能绑（默认工作流 node_id 照样解析出 default，向后兼容）。
    let workflow_id = request
        .node_id
        .split_once(":node:")
        .map(|(prefix, _)| prefix.to_string())
        .filter(|prefix| !prefix.is_empty())
        .unwrap_or_else(|| default_workflow_id(&request.project_root));
    if !workflow_exists(&value, &workflow_id) {
        return Err("当前项目还没有本地 workflow；无法绑定节点会话".to_string());
    }
    if !node_exists(&value, &workflow_id, &request.node_id) {
        return Err("当前 workflow 下找不到该 node；无法绑定节点会话".to_string());
    }
    if let Some(work_item_id) = request.work_item_id.as_deref() {
        if find_work_item(&value, &workflow_id, work_item_id).is_none() {
            return Err("当前 workflow 下找不到该 work item；无法绑定节点会话".to_string());
        }
    }

    let backup = crate::workflow_state_store::backup_file(path, &timestamp)?;

    let existing_active_index = workflow_node_session_binding_index(
        &value,
        &workflow_id,
        &request.node_id,
        request.work_item_id.as_deref(),
    );
    let before_state = existing_active_index
        .and_then(|index| {
            value
                .get("workflow_node_session_bindings")
                .and_then(Value::as_array)
                .and_then(|bindings| bindings.get(index))
                .and_then(|binding| optional_string_from(binding, "native_thread_id"))
        })
        .unwrap_or_else(|| "unbound".to_string());
    let event_type = if existing_active_index.is_some() {
        "workflow_node_session_rebound"
    } else {
        "workflow_node_session_bound"
    };
    let binding_id = workflow_node_session_binding_id(
        &workflow_id,
        &request.node_id,
        request.work_item_id.as_deref(),
        &session.thread_id,
    );
    let mut warnings = Vec::new();
    if !session.rollout_exists {
        warnings.push("index_session_rollout_missing".to_string());
    }
    if session.project_root.as_deref() != Some(request.project_root.as_str()) {
        warnings.push("session_project_root_differs_from_current_project".to_string());
    }
    let binding = json!({
      "binding_id": binding_id,
      "project_id": project_id(&request.project_root),
      "workflow_id": workflow_id,
      "node_id": request.node_id,
      "work_item_id": request.work_item_id,
      "agent_type": "codex",
      "adapter_id": "codex-local",
      "native_thread_id": session.thread_id,
      "native_rollout_path": session.rollout_path,
      "session_title": session.title,
      "session_updated_at_ms": session.updated_at_ms,
      "rollout_exists": session.rollout_exists,
      "project_binding_source": "index_inferred",
      "binding_source": provenance.binding_source,
      "binding_mode": provenance.binding_mode,
      "lifecycle": "active",
      "created_at_ms": timestamp_ms,
      "updated_at_ms": timestamp_ms,
      "warnings": warnings
    });
    if let Some(index) = existing_active_index {
        let bindings = array_mut(&mut value, "workflow_node_session_bindings")?;
        let old_created_at = bindings
            .get(index)
            .and_then(|binding| i64_value(binding, "created_at_ms"));
        bindings[index] = binding;
        if let Some(created_at) = old_created_at {
            bindings[index]["created_at_ms"] = Value::Number(created_at.into());
        }
    } else {
        array_mut(&mut value, "workflow_node_session_bindings")?.push(binding);
    }

    let audit_event_id = crate::workflow_audit::audit_event_identity(
        "workflow-node-session",
        &request.node_id,
        &timestamp,
    );
    array_mut(&mut value, "audit_events")?.push(json!({
      "event_id": audit_event_id,
      "event_type": event_type,
      "target_ref": request.node_id,
      "actor_ref": provenance.actor_ref,
      "source_kind": "workspace_state",
      "permission_level": provenance.permission_level,
      "before_state": before_state,
      "after_state": session.thread_id,
      "created_at": timestamp,
      "reason": provenance.reason
    }));
    value["updated_at"] = Value::String(timestamp);

    let validation_warnings = validate_workflow_state(&value);
    if !validation_warnings.is_empty() {
        return Err(format!(
            "写入前 schema 校验失败：{}",
            validation_warnings.join(", ")
        ));
    }
    write_m5b_batch1_workflow_state(path, "workflow_node_session_binding", &value)?;
    let snapshot = read_workflow_state_snapshot(path)?;
    if !snapshot.exists {
        return Err("绑定节点会话后重新读取校验失败".to_string());
    }

    Ok(WorkflowStateMutationResult {
        message: format!("已绑定节点会话：{}", session.title),
        path: path.display().to_string(),
        backup_path: Some(backup.display().to_string()),
        audit_event_id,
        first_initialize: false,
        snapshot,
    })
}

fn workflow_node_session_binding_id(
    workflow_id: &str,
    node_id: &str,
    work_item_id: Option<&str>,
    native_thread_id: &str,
) -> String {
    let material = serde_json::to_string(&(
        workflow_id,
        node_id,
        work_item_id.unwrap_or("node"),
        native_thread_id,
    ))
    .expect("workflow binding identity is serializable");
    format!(
        "binding:sha256:{}",
        crate::utils::hash::sha256_hex(&material)
    )
}

fn legacy_workflow_node_session_binding_id(
    workflow_id: &str,
    node_id: &str,
    work_item_id: Option<&str>,
) -> String {
    format!(
        "binding:{}:{}:{}",
        stable_id(workflow_id),
        stable_id(node_id),
        stable_id(work_item_id.unwrap_or("node"))
    )
}

#[derive(Debug)]
struct WorkflowBindingIdMigrationCandidate {
    legacy_id: String,
    current_id: String,
    workflow_id: String,
    node_id: String,
    work_item_id: Option<String>,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct WorkflowBindingIdMigrationCounts {
    bindings: usize,
    dispatches: usize,
    unresolved_dispatches: usize,
}

impl WorkflowBindingIdMigrationCounts {
    fn total(&self) -> usize {
        self.bindings + self.dispatches
    }
}

fn migrate_legacy_workflow_node_session_binding_ids(
    value: &mut Value,
) -> WorkflowBindingIdMigrationCounts {
    let mut counts = WorkflowBindingIdMigrationCounts::default();
    let mut candidates = Vec::new();
    if let Some(bindings) = value
        .get_mut("workflow_node_session_bindings")
        .and_then(Value::as_array_mut)
    {
        for binding in bindings {
            let Some(workflow_id) = optional_string_from(binding, "workflow_id") else {
                continue;
            };
            let Some(node_id) = optional_string_from(binding, "node_id") else {
                continue;
            };
            let Some(native_thread_id) = optional_string_from(binding, "native_thread_id") else {
                continue;
            };
            let work_item_id = optional_string_from(binding, "work_item_id");
            let current_id = workflow_node_session_binding_id(
                &workflow_id,
                &node_id,
                work_item_id.as_deref(),
                &native_thread_id,
            );
            candidates.push(WorkflowBindingIdMigrationCandidate {
                legacy_id: legacy_workflow_node_session_binding_id(
                    &workflow_id,
                    &node_id,
                    work_item_id.as_deref(),
                ),
                current_id: current_id.clone(),
                workflow_id,
                node_id,
                work_item_id,
            });
            if optional_string_from(binding, "binding_id").as_deref() != Some(current_id.as_str()) {
                binding["binding_id"] = Value::String(current_id);
                counts.bindings += 1;
            }
        }
    }
    if let Some(dispatches) = value
        .get_mut("workflow_node_dispatches")
        .and_then(Value::as_array_mut)
    {
        for dispatch in dispatches {
            let existing_id = optional_string_from(dispatch, "binding_id");
            if existing_id.as_ref().is_some_and(|binding_id| {
                candidates
                    .iter()
                    .any(|candidate| candidate.current_id == *binding_id)
            }) {
                continue;
            }
            let workflow_id = optional_string_from(dispatch, "workflow_id");
            let node_id = optional_string_from(dispatch, "node_id");
            let work_item_id = optional_string_from(dispatch, "work_item_id");
            let legacy_matches = candidates
                .iter()
                .filter(|candidate| {
                    existing_id
                        .as_ref()
                        .is_some_and(|binding_id| candidate.legacy_id == *binding_id)
                })
                .collect::<Vec<_>>();
            let mut matched = legacy_matches.clone();
            if existing_id.is_some() && matched.len() > 1 {
                matched = legacy_matches
                    .iter()
                    .copied()
                    .filter(|candidate| {
                        workflow_id.as_deref() == Some(candidate.workflow_id.as_str())
                            && node_id.as_deref() == Some(candidate.node_id.as_str())
                            && work_item_id == candidate.work_item_id
                    })
                    .collect::<Vec<_>>();
            } else if existing_id.is_none() {
                matched = candidates
                    .iter()
                    .filter(|candidate| {
                        workflow_id.as_deref() == Some(candidate.workflow_id.as_str())
                            && node_id.as_deref() == Some(candidate.node_id.as_str())
                            && work_item_id == candidate.work_item_id
                    })
                    .collect::<Vec<_>>();
            }
            if let [candidate] = matched.as_slice() {
                dispatch["binding_id"] = Value::String(candidate.current_id.clone());
                counts.dispatches += 1;
            } else if existing_id.is_some() && !legacy_matches.is_empty() {
                counts.unresolved_dispatches += 1;
            }
        }
    }
    counts
}

fn migrate_legacy_workflow_node_session_binding_ids_at(path: &Path) -> Result<usize, String> {
    if !path.exists() {
        return Ok(0);
    }
    let mut value = read_workflow_state_value(path)?;
    let migrated = migrate_legacy_workflow_node_session_binding_ids(&mut value);
    if migrated.unresolved_dispatches > 0 {
        return Err(format!(
            "binding_id 迁移拒绝写入：{} 条旧 dispatch 引用无法唯一映射",
            migrated.unresolved_dispatches
        ));
    }
    if migrated.total() == 0 {
        return Ok(0);
    }
    let validation_warnings = validate_workflow_state(&value);
    if !validation_warnings.is_empty() {
        return Err(format!(
            "binding_id 迁移后 schema 校验失败：{}",
            validation_warnings.join(", ")
        ));
    }
    let timestamp = unix_timestamp_string();
    let backup = backup_workflow_state_file(path, &timestamp)?;
    array_mut(&mut value, "audit_events")?.push(json!({
        "event_id": crate::workflow_audit::audit_event_identity(
            "workflow-binding-id-migrated",
            &path.display().to_string(),
            &timestamp
        ),
        "event_type": "workflow_node_session_binding_ids_migrated",
        "target_ref": path.display().to_string(),
        "actor_ref": "syn_control_core_migration",
        "source_kind": "workspace_state",
        "permission_level": "local_schema_migration",
        "before_state": "legacy_truncated_binding_ids",
        "after_state": "sha256_binding_ids",
        "migrated_count": migrated.total(),
        "migrated_binding_count": migrated.bindings,
        "migrated_dispatch_count": migrated.dispatches,
        "backup_ref": backup.display().to_string(),
        "created_at": timestamp.clone(),
        "reason": "旧 binding_id 会因 96 字符截断碰撞；按完整 workflow/node/work-item/native-thread 身份同步迁移 binding 与 dispatch 引用为 SHA-256。"
    }));
    value["updated_at"] = Value::String(timestamp);
    write_m5b_batch1_workflow_state(path, "workflow_node_session_binding_migration", &value)?;
    Ok(migrated.total())
}

fn unbind_workflow_node_codex_session_at(
    path: &Path,
    request: &WorkflowNodeSessionUnbindRequest,
) -> Result<WorkflowStateMutationResult, String> {
    if !path.exists() {
        return Err("工作流状态文件不存在；无法解绑节点会话".to_string());
    }

    let timestamp = unix_timestamp_string();
    let timestamp_ms = unix_timestamp_ms();
    let mut value = read_workflow_state_value(path)?;
    ensure_workflow_node_session_bindings_array(&mut value)?;
    let validation_warnings = validate_workflow_state(&value);
    if !validation_warnings.is_empty() {
        return Err(format!(
            "当前状态文件未通过 schema 校验：{}",
            validation_warnings.join(", ")
        ));
    }
    let workflow_id = default_workflow_id(&request.project_root);
    if !workflow_exists(&value, &workflow_id) {
        return Err("当前项目还没有本地 workflow；无法解绑节点会话".to_string());
    }

    let binding_index = value
        .get("workflow_node_session_bindings")
        .and_then(Value::as_array)
        .and_then(|bindings| {
            bindings.iter().position(|binding| {
                optional_string_from(binding, "binding_id").as_deref()
                    == Some(request.binding_id.as_str())
                    && optional_string_from(binding, "workflow_id").as_deref()
                        == Some(workflow_id.as_str())
                    && optional_string_from(binding, "lifecycle").as_deref() == Some("active")
            })
        })
        .ok_or_else(|| "当前 workflow 下找不到 active 节点会话绑定；无法解绑".to_string())?;

    let backup = crate::workflow_state_store::backup_file(path, &timestamp)?;

    let before_thread_id = value
        .get("workflow_node_session_bindings")
        .and_then(Value::as_array)
        .and_then(|bindings| bindings.get(binding_index))
        .and_then(|binding| optional_string_from(binding, "native_thread_id"))
        .unwrap_or_else(|| "unknown".to_string());
    {
        let bindings = array_mut(&mut value, "workflow_node_session_bindings")?;
        let binding = bindings
            .get_mut(binding_index)
            .ok_or_else(|| "当前 workflow 下找不到 active 节点会话绑定；无法解绑".to_string())?;
        binding["lifecycle"] = Value::String("detached".to_string());
        binding["updated_at_ms"] = Value::Number(timestamp_ms.into());
    }
    let audit_event_id = crate::workflow_audit::audit_event_identity(
        "workflow-node-session-unbind",
        &request.binding_id,
        &timestamp,
    );
    array_mut(&mut value, "audit_events")?.push(json!({
      "event_id": audit_event_id,
      "event_type": "workflow_node_session_unbound",
      "target_ref": request.binding_id,
      "actor_ref": "user_confirmed_desktop_shell",
      "source_kind": "workspace_state",
      "permission_level": "user_confirmed_write",
      "before_state": before_thread_id,
      "after_state": "detached",
      "created_at": timestamp,
      "reason": "用户确认解除工作台自己的节点会话绑定；没有删除、移动或归档 Codex 原始会话。"
    }));
    value["updated_at"] = Value::String(timestamp);

    let validation_warnings = validate_workflow_state(&value);
    if !validation_warnings.is_empty() {
        return Err(format!(
            "写入前 schema 校验失败：{}",
            validation_warnings.join(", ")
        ));
    }
    write_m5b_batch1_workflow_state(path, "workflow_node_session_unbinding", &value)?;
    let snapshot = read_workflow_state_snapshot(path)?;
    if !snapshot.exists {
        return Err("解绑节点会话后重新读取校验失败".to_string());
    }

    Ok(WorkflowStateMutationResult {
        message: "已解除节点会话绑定；没有删除 Codex 原始会话。".to_string(),
        path: path.display().to_string(),
        backup_path: Some(backup.display().to_string()),
        audit_event_id,
        first_initialize: false,
        snapshot,
    })
}

fn prepare_workflow_node_dispatch_at(
    path: &Path,
    index: &Value,
    request: &WorkflowNodeDispatchPrepareRequest,
) -> Result<WorkflowNodeDispatchResult, String> {
    let mut context = workflow_node_dispatch_context(path, index, request)?;
    control_core::validate_dispatch_prepare(&context.work_item_state)?;
    let authorization_check = inspect_workflow_node_dispatch_authorization(path, &context)?;
    ensure_authorized_for_prepare(&authorization_check)?;
    context.plan_authorization_id = authorization_check.authorization_id.clone();
    context.authorization_check = Some(authorization_check);
    write_prepared_dispatch(path, context)
}

fn execute_workflow_node_dispatch_at(
    path: &Path,
    index: &Value,
    readback_db_path: &Path,
    runner: &dyn CodexResumeRunner,
    request: &WorkflowNodeDispatchExecuteRequest,
) -> Result<WorkflowNodeDispatchResult, String> {
    execute_workflow_node_dispatch_with_authorization_at(
        path,
        index,
        readback_db_path,
        runner,
        request,
        None,
    )
}

fn execute_workflow_node_dispatch_with_authorization_at(
    path: &Path,
    index: &Value,
    readback_db_path: &Path,
    runner: &dyn CodexResumeRunner,
    request: &WorkflowNodeDispatchExecuteRequest,
    prepared_authorization: Option<&PreparedDispatchAuthorization>,
) -> Result<WorkflowNodeDispatchResult, String> {
    let prepare_request = WorkflowNodeDispatchPrepareRequest {
        project_root: request.project_root.clone(),
        node_id: request.node_id.clone(),
        work_item_id: request.work_item_id.clone(),
        prompt_kind: request.prompt_kind.clone(),
        user_reviewed_instruction: request.user_reviewed_instruction.clone(),
    };
    let mut context = workflow_node_dispatch_context(path, index, &prepare_request)?;
    if let Some(prepared_authorization) = prepared_authorization {
        context.plan_authorization_id = Some(prepared_authorization.authorization_id.clone());
        context.authorization_check = Some(prepared_authorization.authorization_check.clone());
        context
            .warnings
            .push("authorized_prepared_dispatch_inherited".to_string());
    }
    control_core::validate_dispatch_start(&context.work_item_state)?;
    let _prepared = write_prepared_dispatch(path, context.clone())?;
    let dispatch = write_started_dispatch(path, &context)?;
    let dispatch_id = dispatch.dispatch_id.clone();
    let last_message_path = dispatch
        .last_message_path
        .as_ref()
        .ok_or_else(|| "派发记录缺少最终回复路径".to_string())
        .map(PathBuf::from)?;

    let run_result = runner.resume_with_options(
        &context.native_thread_id,
        &context.prompt_preview,
        &last_message_path,
        &codex_resume_options_for_context(&context)?,
    );
    match run_result {
        Ok((result, options)) if result.exit_code == 0 => {
            let stats = match options.readback_stats {
                Some(stats) => stats,
                None => dispatch_readback_stats(Some(index), readback_db_path, &context)?,
            };
            write_completed_dispatch(path, &dispatch_id, result.exit_code, stats)
        }
        Ok((result, _options)) => write_failed_dispatch(
            path,
            &dispatch_id,
            result.exit_code,
            classify_codex_resume_failure(
                result.exit_code,
                result.timed_out,
                &context.user_reviewed_instruction,
                result.stderr_summary.as_deref().unwrap_or(""),
            ),
        ),
        Err(error) => write_failed_dispatch(
            path,
            &dispatch_id,
            -1,
            classify_codex_resume_failure(-1, false, &context.user_reviewed_instruction, &error),
        ),
    }
}

fn read_workflow_node_dispatch_result_at(
    path: &Path,
    index: &Value,
    readback_db_path: &Path,
    request: &WorkflowNodeDispatchReadbackRequest,
) -> Result<WorkflowNodeDispatchResult, String> {
    if !path.exists() {
        return Err("工作流状态文件不存在；无法读取节点派发结果".to_string());
    }
    let value = read_workflow_state_value(path)?;
    let dispatch = find_workflow_node_dispatch(&value, &request.dispatch_id)
        .ok_or_else(|| "找不到该节点派发记录".to_string())?;
    let workflow_id = optional_string_from(dispatch, "workflow_id")
        .ok_or_else(|| "节点派发记录缺 workflow_id".to_string())?;
    let work_item_id = optional_string_from(dispatch, "work_item_id")
        .ok_or_else(|| "节点派发记录缺 work_item_id".to_string())?;
    let node_id = optional_string_from(dispatch, "node_id")
        .ok_or_else(|| "节点派发记录缺 node_id".to_string())?;
    let prompt_kind =
        optional_string_from(dispatch, "prompt_kind").unwrap_or_else(|| "safe_probe".to_string());
    let user_reviewed_instruction = if prompt_kind == "user_reviewed_instruction" {
        Some(user_reviewed_instruction_input_from_value(
            dispatch.get("user_reviewed_instruction").ok_or_else(|| {
                "业务派发记录缺少 user_reviewed_instruction，无法回读".to_string()
            })?,
        )?)
    } else {
        None
    };
    let prepare_request = WorkflowNodeDispatchPrepareRequest {
        project_root: request.project_root.clone(),
        node_id,
        work_item_id,
        prompt_kind,
        user_reviewed_instruction,
    };
    let context = workflow_node_dispatch_context(path, index, &prepare_request)?;
    if context.workflow_id != workflow_id {
        return Err("节点派发记录不属于当前项目默认 workflow".to_string());
    }
    let stats = dispatch_readback_stats(Some(index), readback_db_path, &context)?;
    write_readback_dispatch(path, &request.dispatch_id, stats)
}

#[derive(Clone, Debug)]
struct PreparedDispatchAuthorization {
    authorization_id: String,
    authorization_check: AutoDispatchGuardResult,
}

#[derive(Clone, Debug)]
struct WorkflowNodeDispatchContext {
    project_id: String,
    workflow_id: String,
    node_id: String,
    work_item_id: String,
    work_item_state: String,
    binding_id: String,
    native_thread_id: String,
    prompt_preview: String,
    prompt_kind: String,
    memory_packet_snapshot_id: Option<String>,
    memory_packet_fingerprint: Option<String>,
    plan_authorization_id: Option<String>,
    authorization_check: Option<AutoDispatchGuardResult>,
    user_reviewed_instruction: Option<UserReviewedInstructionInput>,
    warnings: Vec<String>,
}

fn workflow_node_dispatch_context(
    path: &Path,
    index: &Value,
    request: &WorkflowNodeDispatchPrepareRequest,
) -> Result<WorkflowNodeDispatchContext, String> {
    if !path.exists() {
        return Err("工作流状态文件不存在；无法准备节点派发".to_string());
    }
    let value = read_workflow_state_value(path)?;
    ensure_valid_dispatch_state(&value)?;
    // 后置C#2：workflow_id 从 node_id（`{workflow_id}:node:…`）解析、退回 default——让非默认
    // （画布建的）工作流节点也能派发（默认工作流 node_id 照样解析出 default，向后兼容）。
    let workflow_id = request
        .node_id
        .split_once(":node:")
        .map(|(prefix, _)| prefix.to_string())
        .filter(|prefix| !prefix.is_empty())
        .unwrap_or_else(|| default_workflow_id(&request.project_root));
    if !workflow_exists(&value, &workflow_id) {
        return Err("当前项目还没有本地 workflow；无法准备节点派发".to_string());
    }
    if !node_exists(&value, &workflow_id, &request.node_id) {
        return Err("当前 workflow 下找不到该 node；无法准备节点派发".to_string());
    }
    let work_item = find_work_item(&value, &workflow_id, &request.work_item_id)
        .ok_or_else(|| "当前 workflow 下找不到该 work item；无法准备节点派发".to_string())?;
    let work_item_state =
        optional_string_from(work_item, "state").unwrap_or_else(|| "unknown".to_string());
    let binding_index = workflow_node_session_binding_index(
        &value,
        &workflow_id,
        &request.node_id,
        Some(&request.work_item_id),
    )
    .or_else(|| workflow_node_session_binding_index(&value, &workflow_id, &request.node_id, None))
    .ok_or_else(|| "当前工作流节点没有 active Codex 会话绑定；无法派发".to_string())?;
    let binding = value
        .get("workflow_node_session_bindings")
        .and_then(Value::as_array)
        .and_then(|bindings| bindings.get(binding_index))
        .ok_or_else(|| "当前工作流节点绑定记录缺失；无法派发".to_string())?;
    let native_thread_id = optional_string_from(binding, "native_thread_id")
        .ok_or_else(|| "节点绑定缺少 Codex thread id；无法派发".to_string())?;
    // 路A：静态快照找不到绑定会话 → 回退实时 sqlite（新 mint/近期会话能被认）。
    let session = find_index_thread_or_sqlite(index, &native_thread_id)
        .ok_or_else(|| "绑定会话不在当前索引内（含实时 sqlite），已拒绝派发".to_string())?;
    if !session.rollout_exists {
        return Err("绑定会话在索引中缺少 rollout，已拒绝派发".to_string());
    }
    let prompt_kind = request.prompt_kind.trim();
    let user_reviewed_instruction = if prompt_kind == "user_reviewed_instruction" {
        let instruction = request
            .user_reviewed_instruction
            .clone()
            .ok_or_else(|| "用户审核模式缺少完整派发字段，已阻止真实业务派发".to_string())?;
        validate_user_reviewed_instruction(&instruction)?;
        Some(instruction)
    } else {
        None
    };
    let base_prompt_preview = if prompt_kind == "safe_probe" {
        safe_probe_prompt()
    } else if prompt_kind == "user_reviewed_instruction" {
        render_user_reviewed_business_prompt(
            user_reviewed_instruction
                .as_ref()
                .ok_or_else(|| "用户审核模式缺少完整派发字段，已阻止真实业务派发".to_string())?,
        )
    } else {
        return Err(format!("未知派发模式：{prompt_kind}"));
    };
    let memory_snapshot = find_task_package_artifact(&value, &request.work_item_id, work_item)
        .and_then(task_memory_injection::snapshot_from_artifact);
    let prompt_preview = memory_snapshot
        .as_ref()
        .map(|snapshot| {
            format!(
                "{}\n\n{}",
                base_prompt_preview,
                task_memory_injection::render_prompt_block(snapshot)
            )
        })
        .unwrap_or(base_prompt_preview);
    let mut warnings = string_array(binding, "warnings");
    warnings.push("legacy_workflow_node_dispatch_not_h5_unified_product_command".to_string());
    warnings.push(
        "legacy_dispatch_requires_h5_continuation_routing_before_product_real_execution"
            .to_string(),
    );
    if session.project_root.as_deref() != Some(request.project_root.as_str()) {
        warnings.push("session_project_root_differs_from_current_project".to_string());
    }
    if let Some(snapshot) = &memory_snapshot {
        warnings.push("task_memory_packet_snapshot_attached".to_string());
        let current =
            task_memory_injection::current_store_revisions(path, &unix_timestamp_string())?;
        if !task_memory_injection::stale_reasons(snapshot, &current).is_empty() {
            warnings.push("task_memory_packet_snapshot_stale".to_string());
        }
    }

    Ok(WorkflowNodeDispatchContext {
        project_id: project_id(&request.project_root),
        workflow_id,
        node_id: request.node_id.clone(),
        work_item_id: request.work_item_id.clone(),
        work_item_state,
        binding_id: optional_string_from(binding, "binding_id")
            .ok_or_else(|| "节点绑定缺少 binding_id；无法派发".to_string())?,
        native_thread_id,
        prompt_preview,
        prompt_kind: prompt_kind.to_string(),
        memory_packet_snapshot_id: memory_snapshot
            .as_ref()
            .map(|snapshot| snapshot.snapshot_id.clone()),
        memory_packet_fingerprint: memory_snapshot
            .as_ref()
            .map(|snapshot| snapshot.fingerprint.clone()),
        plan_authorization_id: None,
        authorization_check: None,
        user_reviewed_instruction,
        warnings,
    })
}

fn inspect_task_package_authorization_at(
    path: &Path,
    project: &ProjectRecord,
    workflow_id: &str,
    work_item: &Value,
    artifact: &Value,
    fields: &RenderTaskPackageFields,
    dispatch_kind: &str,
) -> Result<AutoDispatchGuardResult, String> {
    let work_item_id = fields.work_item_id.clone();
    let target_role_id = optional_string_from(work_item, "assigned_role_id")
        .or_else(|| optional_string_from(artifact, "target_role"))
        .unwrap_or_else(|| assigned_line_id(&fields.assigned_line).to_string());
    let input = AutoDispatchGuardInput {
        project_id: project_id(&project.project_root),
        workflow_id: workflow_id.to_string(),
        work_item_id,
        task_package_id: optional_string_from(artifact, "artifact_id"),
        task_package_kind: optional_string_from(artifact, "artifact_type")
            .or_else(|| Some("task_package".to_string())),
        target_role_id,
        target_agent_id: optional_string_from(artifact, "target_session_id"),
        requested_read_roots: fields.allowed_read.clone(),
        requested_write_roots: fields.allowed_write.clone(),
        requested_tools: string_array(artifact, "callable_tool_capabilities"),
        requested_checks: string_array(artifact, "harness_requirements"),
        triggered_stop_conditions: vec![],
        dispatch_kind: dispatch_kind.to_string(),
    };
    plan_authorization_store::inspect_auto_dispatch_authorization(
        path,
        &input,
        unix_timestamp_ms(),
        &format!("write-task-package-auth-check-{}", unix_timestamp_nanos()),
    )
}
