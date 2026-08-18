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
        crate::workbench_sqlite_storage_mode::primary_repository_for_m2_t2_fail_closed_write(
            path,
            "workflow_work_item_state_update",
        )?
    {
        return update_work_item_state_db_primary(path, request, &repository);
    }
    if crate::ordinary_product_storage_bootstrap::is_ordinary_product_workflow_state_path(path) {
        return Err(
            crate::ordinary_product_storage_bootstrap::ORDINARY_PRODUCT_STORAGE_RESTART_REQUIRED_MARKER
                .to_string(),
        );
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
        receipt_id: None,
        first_initialize: false,
        snapshot,
    })
}

fn with_explicit_m2_port_provenance(
    mut snapshot: WorkflowStateSnapshot,
    explicit_m2_identity: bool,
    command_id: &str,
) -> WorkflowStateSnapshot {
    if explicit_m2_identity {
        let caller_mode = if command_id.starts_with("workflow-state-sidecar.m2.r4:") {
            "R4_ACCEPTANCE"
        } else if command_id.starts_with("workflow-state-sidecar.product.v1:") {
            "SERVER_SEALED_PRODUCT_REQUEST"
        } else {
            "EXPLICIT_M2_REQUEST"
        };
        snapshot.m2_port_provenance = Some(WorkflowStateM2PortProvenance {
            repository_port_version:
                crate::workbench_sqlite_repository::M2_WORKFLOW_STATE_SIDECAR_PORT_VERSION
                    .to_string(),
            schema_version: crate::workbench_sqlite_schema_m2::M2_SCHEMA_VERSION.to_string(),
            caller_mode: caller_mode.to_string(),
        });
    }
    snapshot
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WorkItemStateCommandIdentityMode {
    GuardedLegacy,
    ExplicitM2,
    ServerSealedProduct,
}

impl WorkItemStateCommandIdentityMode {
    fn carries_versioned_revision(self) -> bool {
        !matches!(self, Self::GuardedLegacy)
    }
}

fn server_sealed_work_item_command_identity(
    request: &WorkItemStateUpdateRequest,
    workflow_id: &str,
    next_state: &str,
    current_node_id: &str,
) -> Result<(String, String), String> {
    let client_request_ref = request
        .client_request_ref
        .as_deref()
        .ok_or_else(|| "work_item_client_request_ref_required".to_string())?;
    if client_request_ref.len() != 32
        || !client_request_ref
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err("work_item_client_request_ref_invalid".to_string());
    }

    // Every semantic owner field has already crossed the index/workflow/work
    // item admission above.  Only fixed hashes leave this server-side seal;
    // project paths and raw client references never become command ids.
    let command_material = serde_json::to_string(&serde_json::json!({
        "domain": "syn.workflow-state-sidecar.server-sealed-command.v1",
        "actor_id": "user",
        "scope_owner": "ordinary_product_workflow",
        "project_root": request.project_root,
        "workflow_id": workflow_id,
        "work_item_id": request.work_item_id,
        "client_request_ref": client_request_ref,
    }))
    .map_err(|error| format!("work_item_server_identity_serialize_failed:{error}"))?;
    let command_state_material = serde_json::to_string(&serde_json::json!({
        "schema_version": "workflow-state-sidecar.command-state.v1",
        "repository_port_version": crate::workbench_sqlite_repository::M2_WORKFLOW_STATE_SIDECAR_PORT_VERSION,
        "after_state": next_state,
        "current_node_id": current_node_id,
    }))
    .map_err(|error| format!("work_item_server_state_serialize_failed:{error}"))?;
    let idempotency_material = serde_json::to_string(&serde_json::json!({
        "domain": "syn.workflow-state-sidecar.server-sealed-idempotency.v1",
        "command_material_sha256": crate::utils::hash::sha256_hex(&command_material),
        "next_state": next_state,
        "command_state_sha256": crate::utils::hash::sha256_hex(&command_state_material),
    }))
    .map_err(|error| format!("work_item_server_idempotency_serialize_failed:{error}"))?;
    Ok((
        format!(
            "workflow-state-sidecar.product.v1:{}",
            crate::utils::hash::sha256_hex(&command_material)
        ),
        format!(
            "idem:workflow-state-sidecar.product.v1:{}",
            crate::utils::hash::sha256_hex(&idempotency_material)
        ),
    ))
}

fn explicit_m2_identity_values_are_bounded(command_id: &str, idempotency_key: &str) -> bool {
    !command_id.is_empty()
        && command_id.trim() == command_id
        && command_id.len() <= 512
        && !idempotency_key.is_empty()
        && idempotency_key.trim() == idempotency_key
        && idempotency_key.len() <= 512
}

fn explicit_m2_idempotency_matches_command(command_id: &str, idempotency_key: &str) -> bool {
    idempotency_key == format!("idem:{command_id}")
}

fn ordinary_product_explicit_m2_identity_allowed(
    command_id: &str,
    idempotency_key: &str,
) -> bool {
    if !explicit_m2_identity_values_are_bounded(command_id, idempotency_key) {
        return false;
    }
    cfg!(test)
        || (explicit_m2_idempotency_matches_command(command_id, idempotency_key)
            && matches!(
                crate::m2_r4_reference_slice_driver::current_reference_command_is_registered(
                    command_id
                ),
                Ok(true)
            ))
}

enum ServerSealedCommandResolution {
    Replay {
        receipt_id: String,
    },
    Fresh {
        command: crate::m2_workflow_state::UpdateWorkItemStateCommand,
        request_hash: String,
        authoritative_snapshot_hash: String,
    },
}

fn resolve_server_sealed_work_item_command_in_transaction(
    port: &crate::workbench_sqlite_repository::WorkflowStateSidecarRepositoryV1<'_>,
    request: &WorkItemStateUpdateRequest,
    workflow_id: &str,
    next_state: &str,
    current_node_id: &str,
    command_id: &str,
    idempotency_key: &str,
) -> Result<
    ServerSealedCommandResolution,
    crate::workbench_sqlite_repository::RepositoryMutationError,
> {
    if let Some((receipt_id, existing_idempotency_key, _existing_hash, status)) =
        port.find_command_receipt_by_command_id(command_id)?
    {
        if existing_idempotency_key != idempotency_key {
            return Err(
                crate::workbench_sqlite_repository::RepositoryMutationError::Message(
                    "work_item_client_request_ref_identity_conflict".to_string(),
                ),
            );
        }
        if matches!(
            status.as_str(),
            "COMMITTED" | "EXTERNAL_PENDING" | "EXTERNAL_RESULT"
        ) {
            return Ok(ServerSealedCommandResolution::Replay { receipt_id });
        }
        return Err(
            crate::workbench_sqlite_repository::RepositoryMutationError::Message(format!(
                "m2_existing_receipt_not_successful:receipt_id={receipt_id},status={status}"
            )),
        );
    }

    // This is intentionally after the receipt lookup and in the same
    // IMMEDIATE transaction. A concurrent exact request either owns the first
    // revision or observes its immutable receipt; it never invents rev+1 for
    // the same logical command.
    let authoritative_revision = port.authoritative_revision(workflow_id, &request.work_item_id)?;
    let authoritative_snapshot_hash = port.authoritative_snapshot_hash(
        &request.project_root,
        workflow_id,
        authoritative_revision,
    )?;
    let command_state_json = serde_json::to_string(&serde_json::json!({
        "schema_version": "workflow-state-sidecar.command-state.v1",
        "repository_port_version": crate::workbench_sqlite_repository::M2_WORKFLOW_STATE_SIDECAR_PORT_VERSION,
        "after_state": next_state,
        "current_node_id": current_node_id,
    }))
    .map_err(|error| {
        crate::workbench_sqlite_repository::RepositoryMutationError::Message(format!(
            "m2_command_state_serialize_failed:{error}"
        ))
    })?;
    let command = crate::m2_workflow_state::UpdateWorkItemStateCommand {
        command_id: command_id.to_string(),
        idempotency_key: idempotency_key.to_string(),
        actor_id: "user".to_string(),
        scope_ref: format!("workflow:{}", request.project_root),
        project_id: request.project_root.clone(),
        workflow_id: workflow_id.to_string(),
        work_item_id: request.work_item_id.clone(),
        expected_revision: Some(authoritative_revision),
        new_status: Some(crate::m2_workflow_state::WorkItemStatus::from_str(
            next_state,
        )),
        new_state_json: Some(command_state_json),
    };
    let request_hash =
        crate::m2_update_work_item_state::update_work_item_state_request_hash(&command);
    Ok(ServerSealedCommandResolution::Fresh {
        command,
        request_hash,
        authoritative_snapshot_hash,
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
    let current_node_id = workflow_node_for_work_item_state(&workflow_id, next_state);
    // Three disjoint compatibility modes exist. Ordinary product callers send
    // only a retry reference; the server seals every command field and reads
    // the CAS revision under the owning transaction. Historical no-identity
    // calls retain their guarded behavior. Raw explicit identities are kept
    // only for unit fixtures and the already-registered R4 driver.
    let client_request_ref = request
        .client_request_ref
        .as_deref()
        .filter(|value| !value.trim().is_empty());
    if request.client_request_ref.is_some() && client_request_ref.is_none() {
        return Err("work_item_client_request_ref_invalid".to_string());
    }
    // Mode selection is based on field presence, not normalized content.
    // An explicitly supplied empty identity must never collapse into the
    // legacy no-identity path; the explicit-mode validators reject it below.
    let supplied_command_id = request.command_id.as_deref();
    let supplied_idempotency_key = request.idempotency_key.as_deref();
    let (command_id, idempotency_key, expected_revision, identity_mode) = match (
        client_request_ref,
        supplied_command_id,
        supplied_idempotency_key,
        request.expected_revision,
    ) {
        (None, None, None, None) => {
            let command_id = format!(
                "workflow-state-sidecar.m2.v1:{}",
                crate::m2_clock::uuid_v7()
            );
            (
                command_id.clone(),
                format!("idem:{command_id}"),
                None,
                WorkItemStateCommandIdentityMode::GuardedLegacy,
            )
        }
        (None, Some(command_id), Some(idempotency_key), Some(expected_revision)) => {
            if !ordinary_product_explicit_m2_identity_allowed(command_id, idempotency_key) {
                return Err("work_item_explicit_m2_identity_reserved".to_string());
            }
            (
                command_id.to_string(),
                idempotency_key.to_string(),
                Some(expected_revision),
                WorkItemStateCommandIdentityMode::ExplicitM2,
            )
        }
        (Some(_), None, None, None) => {
            let (command_id, idempotency_key) = server_sealed_work_item_command_identity(
                request,
                &workflow_id,
                next_state,
                &current_node_id,
            )?;
            (
                command_id,
                idempotency_key,
                None,
                WorkItemStateCommandIdentityMode::ServerSealedProduct,
            )
        }
        _ => return Err("work_item_command_identity_mode_conflict".to_string()),
    };
    let explicit_m2_identity = identity_mode.carries_versioned_revision();
    let command_state_json = serde_json::to_string(&serde_json::json!({
        "schema_version": "workflow-state-sidecar.command-state.v1",
        "repository_port_version": crate::workbench_sqlite_repository::M2_WORKFLOW_STATE_SIDECAR_PORT_VERSION,
        "after_state": next_state,
        "current_node_id": current_node_id,
    }))
    .map_err(|error| format!("m2_command_state_serialize_failed:{error}"))?;
    let m2_command = (!matches!(
        identity_mode,
        WorkItemStateCommandIdentityMode::ServerSealedProduct
    ))
    .then(|| crate::m2_workflow_state::UpdateWorkItemStateCommand {
        command_id: command_id.clone(),
        idempotency_key: idempotency_key.clone(),
        actor_id: "user".to_string(),
        scope_ref: format!("workflow:{}", request.project_root),
        project_id: request.project_root.clone(),
        workflow_id: workflow_id.clone(),
        work_item_id: request.work_item_id.clone(),
        expected_revision,
        new_status: Some(crate::m2_workflow_state::WorkItemStatus::from_str(
            next_state,
        )),
        new_state_json: Some(command_state_json),
    });
    let resolved_request_hash = std::cell::RefCell::new(None::<String>);
    let rejection_authoritative_snapshot_hash = std::cell::RefCell::new(None::<String>);

    if let Some(m2_command) = m2_command.as_ref() {
        let request_hash =
            crate::m2_update_work_item_state::update_work_item_state_request_hash(m2_command);
        *resolved_request_hash.borrow_mut() = Some(request_hash.clone());

        // Explicit/legacy compatibility preflight remains unchanged. The
        // server-sealed mode performs this decision only inside IMMEDIATE.
        match repository.find_command_receipt_for_idempotency(&command_id, &idempotency_key)? {
            Some((existing_receipt_id, existing_hash, existing_status))
                if existing_hash == request_hash =>
            {
                if matches!(
                    existing_status.as_str(),
                    "COMMITTED" | "EXTERNAL_PENDING" | "EXTERNAL_RESULT"
                ) {
                    let snapshot = with_explicit_m2_port_provenance(
                        read_workflow_state_snapshot(path)?,
                        explicit_m2_identity,
                        &command_id,
                    );
                    return Ok(WorkflowStateMutationResult {
                        message: format!(
                            "幂等重放：该状态推进命令已处理，返回既有 receipt，未新增任何变更：{} -> {}",
                            work_item_state_label(&before_state),
                            work_item_state_label(next_state)
                        ),
                        path: path.display().to_string(),
                        backup_path: None,
                        audit_event_id: format!("idempotent-replay:{existing_receipt_id}"),
                        receipt_id: Some(existing_receipt_id),
                        first_initialize: false,
                        snapshot,
                    });
                }
                return Err(format!(
                    "m2_existing_receipt_not_successful:receipt_id={existing_receipt_id},status={existing_status}"
                ));
            }
            Some((_, existing_hash, _)) => {
                return Err(format!(
                    "idempotent_conflict: command_id={}, idempotency_key={}, existing_hash={}, new_hash={}",
                    command_id, idempotency_key, existing_hash, request_hash
                ));
            }
            None => {}
        }

        let authoritative_revision =
            repository.m2_workflow_state_sidecar_revision(&workflow_id, &request.work_item_id)?;
        if matches!(identity_mode, WorkItemStateCommandIdentityMode::ExplicitM2)
            && expected_revision != Some(authoritative_revision)
        {
            return Err(format!(
                "m2_workflow_state_expected_revision_stale:expected={},actual={authoritative_revision}",
                expected_revision.expect("explicit M2 identity has expected revision")
            ));
        }
        *rejection_authoritative_snapshot_hash.borrow_mut() =
            Some(repository.m2_workflow_state_authoritative_snapshot_hash(
                &request.project_root,
                &workflow_id,
                authoritative_revision,
            )?);
    }

    // Policy 预检（真闸：control_core 状态转换表）；非法转换走 M2 denial receipt（同一事务落盘），
    // 零 domain/event/outbox mutation，JSON 业务状态不变，命令以错误返回
    if let Err(policy_reason) =
        control_core::validate_work_item_state_transition(&before_state, next_state)
    {
        let replay_receipt_id = std::cell::RefCell::new(None::<String>);
        repository
            .with_m2_reference_command_transaction(
                "update_work_item_state_m2_denial",
                &command_id,
                None,
                |transaction| {
                let port = crate::workbench_sqlite_repository::WorkflowStateSidecarRepositoryV1::new(
                    transaction,
                    crate::workbench_sqlite_repository::M2WorkflowStateSidecarConsumerId::UpdateWorkItemStateDbPrimary,
                );
                let command = if let Some(command) = m2_command.as_ref() {
                    command.clone()
                } else {
                    match resolve_server_sealed_work_item_command_in_transaction(
                        &port,
                        request,
                        &workflow_id,
                        next_state,
                        &current_node_id,
                        &command_id,
                        &idempotency_key,
                    )? {
                        ServerSealedCommandResolution::Replay { receipt_id } => {
                            *replay_receipt_id.borrow_mut() = Some(receipt_id);
                            return Ok(());
                        }
                        ServerSealedCommandResolution::Fresh { command, .. } => command,
                    }
                };
                let result = port.execute_update(command)?;
                if result.receipt.status != crate::m2_dto::CommandReceiptStatus::Denied {
                    return Err(crate::workbench_sqlite_repository::RepositoryMutationError::Message(
                        "m2_policy_denial_receipt_missing".to_string(),
                    ));
                }
                Ok(()) as Result<(), crate::workbench_sqlite_repository::RepositoryMutationError>
            },
            )
            .map_err(|e| format!("update_work_item_state_m2_denial: {}", e))?;
        if let Some(receipt_id) = replay_receipt_id.into_inner() {
            let snapshot = with_explicit_m2_port_provenance(
                read_workflow_state_snapshot(path)?,
                explicit_m2_identity,
                &command_id,
            );
            return Ok(WorkflowStateMutationResult {
                message: "幂等重放：该状态推进命令已处理，返回既有 receipt，未新增任何变更。"
                    .to_string(),
                path: path.display().to_string(),
                backup_path: None,
                audit_event_id: format!("idempotent-replay:{receipt_id}"),
                receipt_id: Some(receipt_id),
                first_initialize: false,
                snapshot,
            });
        }
        return Err(policy_reason);
    }

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

    // Same immediate transaction: the UoW is the authoritative idempotency
    // decision.  The advisory precheck above reduces work but cannot decide a
    // concurrent race; a replay must not append another repository audit,
    // mutate state, or start a JSON projection.
    let replay_receipt_id = std::cell::RefCell::new(None::<String>);
    let applied_receipt_id = std::cell::RefCell::new(None::<String>);
    let applied_workflow_revision = std::cell::RefCell::new(None::<i64>);
    let applied_event_id = std::cell::RefCell::new(None::<String>);
    let applied_snapshot_hash = std::cell::RefCell::new(None::<String>);
    let m4_conflict_candidate = std::cell::RefCell::new(
        None::<crate::m4_source_owner_schema::M4SourceOwnerOutboxEnvelopeV1>,
    );
    let transaction_result = repository
        .with_m2_reference_command_transaction(
            "update_work_item_state_m2_wired",
            &command_id,
            None,
            |transaction| {
            let port = crate::workbench_sqlite_repository::WorkflowStateSidecarRepositoryV1::new(
                transaction,
                crate::workbench_sqlite_repository::M2WorkflowStateSidecarConsumerId::UpdateWorkItemStateDbPrimary,
            );
            let command = if let Some(command) = m2_command.as_ref() {
                command.clone()
            } else {
                match resolve_server_sealed_work_item_command_in_transaction(
                    &port,
                    request,
                    &workflow_id,
                    next_state,
                    &current_node_id,
                    &command_id,
                    &idempotency_key,
                )? {
                    ServerSealedCommandResolution::Replay { receipt_id } => {
                        *replay_receipt_id.borrow_mut() = Some(receipt_id);
                        return Ok(()) as Result<
                            (),
                            crate::workbench_sqlite_repository::RepositoryMutationError,
                        >;
                    }
                    ServerSealedCommandResolution::Fresh {
                        command,
                        request_hash,
                        authoritative_snapshot_hash,
                    } => {
                        *resolved_request_hash.borrow_mut() = Some(request_hash);
                        *rejection_authoritative_snapshot_hash.borrow_mut() =
                            Some(authoritative_snapshot_hash);
                        command
                    }
                }
            };
            // M2 UoW 全链（policy → idempotency → domain state → event → audit → receipt → snapshot）
            let m2_result = port.execute_update(command)?;
            if m2_result.event.event_type == "WorkItemStateUpdateIdempotent" {
                *replay_receipt_id.borrow_mut() = Some(m2_result.receipt.receipt_id);
                return Ok(())
                    as Result<(), crate::workbench_sqlite_repository::RepositoryMutationError>;
            }
            if m2_result.receipt.status == crate::m2_dto::CommandReceiptStatus::Denied {
                return Err(
                    crate::workbench_sqlite_repository::RepositoryMutationError::Message(
                        "m2_chain_denied_after_fresh_transaction_check".to_string(),
                    ),
                );
            }
            let committed_revision = m2_result.receipt.committed_revision.ok_or_else(|| {
                crate::workbench_sqlite_repository::RepositoryMutationError::Message(
                    "m2_workflow_state_revision_missing".to_string(),
                )
            })?;
            // M1 JSON records can legitimately predate the M2 revision field.
            // Persist it only on records this M2 command changed, with the same
            // receipt-backed revision as the UoW.  On a post-commit projection
            // failure this is the causal ordering proof that lets startup replay
            // the DB-primary state without treating an unversioned legacy record
            // as an arbitrary hash conflict.
            let mut committed_work_item_after = work_item_after.clone();
            let mut committed_node_after = node_after.clone();
            committed_work_item_after["workflow_revision_after"] =
                Value::Number(committed_revision.into());
            committed_node_after["workflow_revision_after"] =
                Value::Number(committed_revision.into());
            *applied_receipt_id.borrow_mut() = Some(m2_result.receipt.receipt_id.clone());
            *applied_workflow_revision.borrow_mut() = Some(committed_revision);

            // repository work_item + node 状态更新（同一事务）
            port.write_domain_state(
                &committed_work_item_after,
                &committed_node_after,
                &before_state,
            )?;

            let authoritative_snapshot = port.record_authoritative_snapshot(
                &request.project_root,
                &workflow_id,
                committed_revision,
                &m2_result.event.event_id,
                crate::unix_timestamp_ms(),
            )?;
            *applied_event_id.borrow_mut() = Some(m2_result.event.event_id.clone());
            *applied_snapshot_hash.borrow_mut() = Some(authoritative_snapshot.snapshot_hash.clone());

            // repository audit record（同一事务）
            port.append_owning_audit(
                &crate::workbench_sqlite_repository::RepositoryAuditEntry {
                    event_id: audit_event_id.clone(),
                    target_kind: "workflow_state".to_string(),
                    target_id: request.work_item_id.clone(),
                    payload: audit_event.clone(),
                },
            )?;

            // M4R02 registered-source publication is an owning fact of this
            // ordinary WorkItem command.  Build it by rereading the just-
            // committed native event/receipt/snapshot/domain rows, then append
            // it before the same IMMEDIATE transaction can commit.  A failed
            // provenance check therefore rolls the WorkItem mutation back as
            // one UoW; there is no post-command best-effort wrapper.
            let m4_publication =
                crate::m4_source_owner_schema::build_m4_work_item_source_publication(
                    transaction,
                    &m2_result.event.event_id,
                    &m2_result.receipt.receipt_id,
                    &request.work_item_id,
                    next_state,
                )?;
            *m4_conflict_candidate.borrow_mut() = Some(m4_publication.clone());
            crate::m4_source_owner_schema::append_m4_work_item_source_publication(
                transaction,
                &m4_publication,
            )?;

            // DAT-004/008 has exactly one optional external-effect branch:
            // the debug R4 driver must prove its full attempt/nonce/command
            // binding before this production UoW reaches the repository.  The
            // declaration reuses this command's receipt/event/audit/domain
            // facts and creates no synthetic owning command.
            #[cfg(debug_assertions)]
            if crate::m2_r4_reference_slice_driver::current_reference_effect_is_armed(
                &command_id,
            )
            .map_err(crate::workbench_sqlite_repository::RepositoryMutationError::Message)?
            {
                port.declare_armed_r4_effect(
                    &crate::workbench_sqlite_repository::M2R4ArmedReferenceEffectDeclaration {
                        owning_command_id: &command_id,
                        owning_receipt_id: m2_result.receipt.receipt_id.as_str(),
                        owning_event_id: m2_result.event.event_id.as_str(),
                        actor_id: m2_result.receipt.actor_id.as_str(),
                        scope_ref: m2_result.receipt.scope_ref.as_str(),
                        subject_ref: &format!("work-item:{}", request.work_item_id),
                        payload_hash: authoritative_snapshot.snapshot_hash.as_str(),
                        correlation_id: &command_id,
                        causation_id: &command_id,
                    },
                    crate::unix_timestamp_ms(),
                )?;
            }

            Ok(()) as Result<(), crate::workbench_sqlite_repository::RepositoryMutationError>
        },
        );
    if let Err(error) = transaction_result {
        if error.contains("m4_source_owner_identifier_invalid") {
            let request_hash = resolved_request_hash.borrow().clone().ok_or_else(|| {
                "ordinary_product_work_item_source_rejection_hash_missing".to_string()
            })?;
            let authoritative_snapshot_hash = rejection_authoritative_snapshot_hash
                .borrow()
                .clone()
                .ok_or_else(|| {
                    "ordinary_product_work_item_source_rejection_snapshot_missing".to_string()
                })?;
            let candidate = crate::m4_source_owner_schema::build_m4_work_item_candidate_rejection(
                &command_id,
                &idempotency_key,
                &request_hash,
                &authoritative_snapshot_hash,
                next_state,
            )
            .map_err(|_| "ordinary_product_work_item_source_rejection_record_failed".to_string())?;
            repository
                .record_m4_source_owner_candidate_rejection(
                    &candidate,
                    "OWNER_PUBLICATION_IDENTIFIER_REJECTED",
                    crate::unix_timestamp_ms(),
                )
                .map_err(|_| {
                    "ordinary_product_work_item_source_rejection_record_failed".to_string()
                })?;
            return Err("ordinary_product_work_item_source_publication_rejected".to_string());
        }
        if error.contains("m4_source_owner_publication_idempotency_conflict") {
            if let Some(candidate) = m4_conflict_candidate.borrow().as_ref() {
                repository.record_m4_source_owner_candidate_conflict(
                    candidate,
                    "IDEMPOTENCY_PAYLOAD_CONFLICT",
                    crate::unix_timestamp_ms(),
                )?;
            }
        }
        return Err(format!("update_work_item_state_m2_wired: {error}"));
    }

    if let Some(receipt_id) = replay_receipt_id.into_inner() {
        let snapshot = with_explicit_m2_port_provenance(
            read_workflow_state_snapshot(path)?,
            explicit_m2_identity,
            &command_id,
        );
        return Ok(WorkflowStateMutationResult {
            message: format!(
                "幂等重放：同一 receipt 已在事务内确认，未新增业务、审计或投影：{} -> {}",
                work_item_state_label(&before_state),
                work_item_state_label(next_state)
            ),
            path: path.display().to_string(),
            backup_path: None,
            audit_event_id: format!("idempotent-replay:{receipt_id}"),
            receipt_id: Some(receipt_id),
            first_initialize: false,
            snapshot,
        });
    }

    let committed_workflow_revision = applied_workflow_revision.into_inner().ok_or_else(|| {
        crate::workbench_sqlite_storage_mode::block_db_primary_writes(
            path,
            "workflow_state_revision_missing",
            "accepted M2 command returned without a committed workflow revision",
        );
        "m2_workflow_state_revision_missing".to_string()
    })?;
    value["work_items"]
        .as_array_mut()
        .and_then(|items| items.get_mut(work_item_index))
        .ok_or_else(|| "M2 投影找不到已提交的 work item".to_string())?["workflow_revision_after"] =
        Value::Number(committed_workflow_revision.into());
    value["nodes"]
        .as_array_mut()
        .and_then(|nodes| {
            nodes.iter_mut().find(|node| {
                optional_string_from(node, "node_id").as_deref() == Some(current_node_id.as_str())
            })
        })
        .ok_or_else(|| "M2 投影找不到已提交的 workflow node".to_string())?
        ["workflow_revision_after"] = Value::Number(committed_workflow_revision.into());

    // T2 崩溃恢复验收门（debug-only）：commit 已落盘、JSON 投影未开始的确定性窗口。
    // 操作者在窗口内 SIGKILL → 重启走 DB-leading replay 恢复投影。
    let projection_result =
        crate::workbench_sqlite_storage_mode::complete_db_primary_json_projection(
            path,
            "work_item_state_transition",
            || {
                // This gate is deliberately inside the projection/freeze wrapper.
                // A timeout after DB commit must block subsequent DB-primary
                // writes until startup reconciliation, not return while leaving a
                // writable DB-leading process behind.
                #[cfg(debug_assertions)]
                crate::m2_r4_reference_slice_driver::wait_for_current_reference_command_gate(
                    "post-commit",
                    &command_id,
                )?;
                // T2 投影失败验收门（debug-only）：武装时注入确定性投影失败，验证 fail-closed/降级语义。
                #[cfg(debug_assertions)]
                if let Some(injected) =
                    crate::m2_r4_reference_slice_driver::injected_current_reference_command_failure(
                        "projection-fail",
                        &command_id,
                    )?
                {
                    return Err(injected);
                }
                let backup = crate::workflow_state_store::backup_file(path, &timestamp)?;
                write_validated_workflow_state(path, &value)?;
                let snapshot = with_explicit_m2_port_provenance(
                    read_workflow_state_snapshot(path)?,
                    explicit_m2_identity,
                    &command_id,
                );
                if !snapshot.exists {
                    return Err("推进工作项状态后重新读取校验失败".to_string());
                }
                // The parity formula must be derived from bytes that were
                // written and parsed back from the internal projection, not
                // the pre-write in-memory candidate.  A disk tamper or a
                // serializer divergence therefore cannot advance a checkpoint
                // or make the DB accept JSON as a source of truth.
                let persisted_value = read_workflow_state_value(path)?;
                let persisted_warnings = validate_workflow_state(&persisted_value);
                if !persisted_warnings.is_empty() {
                    return Err(format!(
                        "m2_workflow_state_projection_persisted_schema_invalid:{}",
                        persisted_warnings.join(",")
                    ));
                }
                let projection_snapshot = crate::workbench_sqlite_repository::
                    m2_workflow_state_sidecar_snapshot_from_projection(
                        &request.project_root,
                        &workflow_id,
                        committed_workflow_revision,
                        &persisted_value,
                    )?;
                let projection_hash = projection_snapshot.snapshot_hash;
                let expected_snapshot_hash = applied_snapshot_hash
                    .borrow()
                    .clone()
                    .ok_or_else(|| "m2_workflow_state_snapshot_hash_missing".to_string())?;
                if projection_hash != expected_snapshot_hash {
                    return Err(format!(
                        "m2_workflow_state_projection_snapshot_hash_mismatch:expected={expected_snapshot_hash},actual={projection_hash}"
                    ));
                }
                let event_id = applied_event_id
                    .borrow()
                    .clone()
                    .ok_or_else(|| "m2_workflow_state_event_id_missing".to_string())?;
                let receipt_id = applied_receipt_id
                    .borrow()
                    .clone()
                    .ok_or_else(|| "m2_workflow_state_receipt_id_missing".to_string())?;

                repository
                    .with_immediate_transaction(
                        "workflow_state_internal_projection_checkpoint",
                        None,
                        |transaction| {
                            crate::workbench_sqlite_repository::WorkflowStateSidecarRepositoryV1::new(
                                transaction,
                                crate::workbench_sqlite_repository::M2WorkflowStateSidecarConsumerId::UpdateWorkItemStateDbPrimary,
                            )
                            .record_projection_checkpoint(
                                &projection_snapshot.object_ref,
                                committed_workflow_revision,
                                &event_id,
                                &receipt_id,
                                &projection_hash,
                                crate::unix_timestamp_ms(),
                            )
                        },
                    )
                    .map_err(|error| format!("workflow_state_internal_projection_checkpoint:{error}"))?;

                Ok(WorkflowStateMutationResult {
                    message: format!(
                        "已推进工作项状态：{} -> {}",
                        work_item_state_label(&before_state),
                        work_item_state_label(next_state)
                    ),
                    path: path.display().to_string(),
                    backup_path: Some(backup.display().to_string()),
                    audit_event_id,
                    receipt_id: applied_receipt_id.borrow().clone(),
                    first_initialize: false,
                    snapshot,
                })
            },
        );
    projection_result
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

const M6P00_WORKFLOW_PROJECT_ID_MISMATCH: &str = "m6p00_workflow_project_id_mismatch";

/// Select a target record in the canonical project namespace.  Same-key
/// path-derived or foreign records are never treated as an idempotent hit.
/// Ownerless records remain a guarded legacy fixture only; any explicit
/// non-canonical owner wins over that fixture and fails closed.  This fallback
/// expires when the workflow-state schema migration stamps project_id on
/// nodes, work items, bindings, dispatches, and task-package artifacts.
fn canonical_owned_record_index<F>(
    value: &Value,
    collection: &str,
    canonical_project_id: &str,
    target: &str,
    matches_target: F,
) -> Result<Option<usize>, String>
where
    F: Fn(&Value) -> bool,
{
    let Some(records) = value.get(collection).and_then(Value::as_array) else {
        return Ok(None);
    };
    let mut ownerless = None;
    let mut explicit_mismatch = None;
    for (index, record) in records.iter().enumerate() {
        if !matches_target(record) {
            continue;
        }
        match optional_string_from(record, "project_id") {
            Some(owner) if owner == canonical_project_id => return Ok(Some(index)),
            Some(owner) => {
                explicit_mismatch.get_or_insert(owner);
            }
            None => {
                ownerless.get_or_insert(index);
            }
        }
    }
    if let Some(owner) = explicit_mismatch {
        return Err(format!(
            "{M6P00_WORKFLOW_PROJECT_ID_MISMATCH}:{target}:expected={canonical_project_id}:actual={owner}"
        ));
    }
    Ok(ownerless)
}

fn canonical_workflow_record_index(
    value: &Value,
    workflow_id: &str,
    canonical_project_id: &str,
) -> Result<Option<usize>, String> {
    canonical_owned_record_index(
        value,
        "workflows",
        canonical_project_id,
        "workflow",
        |workflow| optional_string_from(workflow, "workflow_id").as_deref() == Some(workflow_id),
    )
}

fn canonical_node_record_index(
    value: &Value,
    workflow_id: &str,
    node_id: &str,
    canonical_project_id: &str,
) -> Result<Option<usize>, String> {
    canonical_owned_record_index(value, "nodes", canonical_project_id, "node", |node| {
        optional_string_from(node, "workflow_id").as_deref() == Some(workflow_id)
            && optional_string_from(node, "node_id").as_deref() == Some(node_id)
    })
}

fn canonical_work_item_record_index(
    value: &Value,
    workflow_id: &str,
    work_item_id: &str,
    canonical_project_id: &str,
) -> Result<Option<usize>, String> {
    canonical_owned_record_index(
        value,
        "work_items",
        canonical_project_id,
        "work_item",
        |work_item| {
            optional_string_from(work_item, "workflow_id").as_deref() == Some(workflow_id)
                && optional_string_from(work_item, "work_item_id").as_deref() == Some(work_item_id)
        },
    )
}

fn canonical_binding_record_index(
    value: &Value,
    workflow_id: &str,
    node_id: &str,
    work_item_id: Option<&str>,
    canonical_project_id: &str,
) -> Result<Option<usize>, String> {
    canonical_owned_record_index(
        value,
        "workflow_node_session_bindings",
        canonical_project_id,
        "workflow_node_session_binding",
        |binding| {
            optional_string_from(binding, "workflow_id").as_deref() == Some(workflow_id)
                && optional_string_from(binding, "node_id").as_deref() == Some(node_id)
                && optional_string_from(binding, "lifecycle").as_deref() == Some("active")
                && optional_string_from(binding, "work_item_id").as_deref() == work_item_id
        },
    )
}

fn canonical_dispatch_record_index(
    value: &Value,
    dispatch_id: &str,
    canonical_project_id: &str,
) -> Result<Option<usize>, String> {
    canonical_owned_record_index(
        value,
        "workflow_node_dispatches",
        canonical_project_id,
        "workflow_node_dispatch",
        |dispatch| optional_string_from(dispatch, "dispatch_id").as_deref() == Some(dispatch_id),
    )
}

fn canonical_task_package_artifact_record_index(
    value: &Value,
    work_item_id: &str,
    work_item: &Value,
    canonical_project_id: &str,
) -> Result<Option<usize>, String> {
    let source_artifact_id = optional_string_from(work_item, "source_ref");
    canonical_owned_record_index(
        value,
        "artifacts",
        canonical_project_id,
        "task_package_artifact",
        |artifact| {
            optional_string_from(artifact, "artifact_type").as_deref() == Some("task_package")
                && (optional_string_from(artifact, "source_ref").as_deref() == Some(work_item_id)
                    || source_artifact_id.as_deref().is_some_and(|source_id| {
                        optional_string_from(artifact, "artifact_id").as_deref() == Some(source_id)
                    }))
        },
    )
}

fn update_canonical_node_state_for_id(
    value: &mut Value,
    workflow_id: &str,
    node_id: &str,
    canonical_project_id: &str,
    state: &str,
    timestamp: &str,
) -> Result<(), String> {
    let node_index =
        canonical_node_record_index(value, workflow_id, node_id, canonical_project_id)?;
    if let Some(node_index) = node_index {
        let node = array_mut(value, "nodes")?
            .get_mut(node_index)
            .ok_or_else(|| "canonical workflow node disappeared before update".to_string())?;
        node["state"] = Value::String(state.to_string());
        node["updated_at"] = Value::String(timestamp.to_string());
    }
    Ok(())
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
    // Legacy fixture/guarded callers still persist the historical path-derived
    // owner.  This wrapper expires when those callers receive the M1 canonical
    // ProjectId explicitly.  Formal M6P00 commands bypass it.
    migrate_legacy_workflow_node_session_binding_ids_at(path)?;
    bind_workflow_node_codex_session_with_canonical_project_id_at(
        path,
        request,
        session,
        provenance,
        &project_id(&request.project_root),
    )
}

fn bind_workflow_node_codex_session_with_canonical_project_id_at(
    path: &Path,
    request: &WorkflowNodeSessionBindRequest,
    session: &SessionRecord,
    provenance: &WorkflowNodeSessionBindingProvenance,
    canonical_project_id: &str,
) -> Result<WorkflowStateMutationResult, String> {
    if !path.exists() {
        return Err("工作流状态文件不存在；无法绑定节点会话".to_string());
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

    // 后置C#2：workflow_id 从 node_id（`{workflow_id}:node:…`）解析、退回 default——让非默认工作流
    // 的节点也能绑（默认工作流 node_id 照样解析出 default，向后兼容）。
    let workflow_id = request
        .node_id
        .split_once(":node:")
        .map(|(prefix, _)| prefix.to_string())
        .filter(|prefix| !prefix.is_empty())
        .unwrap_or_else(|| default_workflow_id(&request.project_root));
    if canonical_workflow_record_index(&value, &workflow_id, canonical_project_id)?.is_none() {
        return Err("当前项目还没有本地 workflow；无法绑定节点会话".to_string());
    }
    if canonical_node_record_index(&value, &workflow_id, &request.node_id, canonical_project_id)?
        .is_none()
    {
        return Err("当前 workflow 下找不到该 node；无法绑定节点会话".to_string());
    }
    if let Some(work_item_id) = request.work_item_id.as_deref() {
        if canonical_work_item_record_index(
            &value,
            &workflow_id,
            work_item_id,
            canonical_project_id,
        )?
        .is_none()
        {
            return Err("当前 workflow 下找不到该 work item；无法绑定节点会话".to_string());
        }
    }

    let existing_active_index = canonical_binding_record_index(
        &value,
        &workflow_id,
        &request.node_id,
        request.work_item_id.as_deref(),
        canonical_project_id,
    )?;
    let backup = crate::workflow_state_store::backup_file(path, &timestamp)?;
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
      "project_id": canonical_project_id,
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
        receipt_id: None,
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
        receipt_id: None,
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
    // Guarded legacy/fixed-test route.  Formal M6P00 commands call the
    // canonical variant below and never mint this path-derived owner.
    execute_workflow_node_dispatch_with_canonical_project_id_at(
        path,
        index,
        readback_db_path,
        runner,
        request,
        &project_id(&request.project_root),
    )
}

fn execute_workflow_node_dispatch_with_canonical_project_id_at(
    path: &Path,
    index: &Value,
    readback_db_path: &Path,
    runner: &dyn CodexResumeRunner,
    request: &WorkflowNodeDispatchExecuteRequest,
    canonical_project_id: &str,
) -> Result<WorkflowNodeDispatchResult, String> {
    execute_workflow_node_dispatch_with_authorization_and_canonical_project_id_at(
        path,
        index,
        readback_db_path,
        runner,
        request,
        None,
        canonical_project_id,
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
    // Guarded legacy/fixed-test route; see the canonical variant below.
    execute_workflow_node_dispatch_with_authorization_and_canonical_project_id_at(
        path,
        index,
        readback_db_path,
        runner,
        request,
        prepared_authorization,
        &project_id(&request.project_root),
    )
}

fn execute_workflow_node_dispatch_with_authorization_and_canonical_project_id_at(
    path: &Path,
    index: &Value,
    readback_db_path: &Path,
    runner: &dyn CodexResumeRunner,
    request: &WorkflowNodeDispatchExecuteRequest,
    prepared_authorization: Option<&PreparedDispatchAuthorization>,
    canonical_project_id: &str,
) -> Result<WorkflowNodeDispatchResult, String> {
    let prepare_request = WorkflowNodeDispatchPrepareRequest {
        project_root: request.project_root.clone(),
        node_id: request.node_id.clone(),
        work_item_id: request.work_item_id.clone(),
        prompt_kind: request.prompt_kind.clone(),
        user_reviewed_instruction: request.user_reviewed_instruction.clone(),
    };
    let mut context = workflow_node_dispatch_context_with_canonical_project_id(
        path,
        index,
        &prepare_request,
        canonical_project_id,
    )?;
    if let Some(prepared_authorization) = prepared_authorization {
        if prepared_authorization.m2_execution_grant_required {
            // The M2 grant route must attach to the exact work-item binding
            // created by C1.  Frozen M1 prepared dispatches continue to use
            // their existing role-binding behavior and never mint a grant.
            let current = read_workflow_state_value(path)?;
            let binding_index = canonical_binding_record_index(
                &current,
                &context.workflow_id,
                &context.node_id,
                Some(&context.work_item_id),
                canonical_project_id,
            )?
            .ok_or_else(|| "execution_grant_exact_work_item_binding_required".to_string())?;
            let exact_binding = current
                .get("workflow_node_session_bindings")
                .and_then(Value::as_array)
                .and_then(|bindings| bindings.get(binding_index))
                .ok_or_else(|| "execution_grant_exact_work_item_binding_missing".to_string())?;
            if optional_string_from(exact_binding, "binding_id").as_deref()
                != Some(context.binding_id.as_str())
                || optional_string_from(exact_binding, "native_thread_id").as_deref()
                    != Some(context.native_thread_id.as_str())
            {
                return Err("execution_grant_exact_work_item_binding_stale".to_string());
            }
        }
        context.plan_authorization_id = Some(prepared_authorization.authorization_id.clone());
        context.authorization_check = Some(prepared_authorization.authorization_check.clone());
        context.prepared_dispatch_id = Some(prepared_authorization.prepared_dispatch_id.clone());
        context
            .warnings
            .push("authorized_prepared_dispatch_inherited".to_string());
        if prepared_authorization.m2_execution_grant_required {
            // Exact prepared authorization is bound either to its immutable C4
            // binding snapshot or (only when C4 explicitly deferred it) to
            // the live per-task binding resolved by this invocation.
            if prepared_authorization.thread_binding_deferred {
                if prepared_authorization.authorization_binding_id.is_some()
                    || prepared_authorization
                        .authorization_native_thread_id
                        .is_some()
                {
                    return Err(
                        "execution_grant_prepared_deferred_binding_snapshot_invalid".to_string()
                    );
                }
            } else if prepared_authorization.authorization_binding_id.as_deref()
                != Some(context.binding_id.as_str())
                || prepared_authorization
                    .authorization_native_thread_id
                    .as_deref()
                    != Some(context.native_thread_id.as_str())
            {
                return Err("execution_grant_prepared_binding_snapshot_stale".to_string());
            }
        }
    }
    control_core::validate_dispatch_start(&context.work_item_state)?;
    if context.prepared_dispatch_id.is_none() {
        let _prepared = write_prepared_dispatch(path, context.clone())?;
    }
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
            write_completed_dispatch(
                path,
                &dispatch_id,
                &context.project_id,
                result.exit_code,
                stats,
            )
        }
        Ok((result, _options)) => write_failed_dispatch(
            path,
            &dispatch_id,
            &context.project_id,
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
            &context.project_id,
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
    prepared_dispatch_id: String,
    authorization_binding_id: Option<String>,
    authorization_native_thread_id: Option<String>,
    thread_binding_deferred: bool,
    m2_execution_grant_required: bool,
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
    prepared_dispatch_id: Option<String>,
    user_reviewed_instruction: Option<UserReviewedInstructionInput>,
    warnings: Vec<String>,
}

fn workflow_node_dispatch_context(
    path: &Path,
    index: &Value,
    request: &WorkflowNodeDispatchPrepareRequest,
) -> Result<WorkflowNodeDispatchContext, String> {
    // Guarded legacy/fixed-test route.  Formal commands supply the canonical
    // M1 identity to the variant below.
    workflow_node_dispatch_context_with_canonical_project_id(
        path,
        index,
        request,
        &project_id(&request.project_root),
    )
}

fn workflow_node_dispatch_context_with_canonical_project_id(
    path: &Path,
    index: &Value,
    request: &WorkflowNodeDispatchPrepareRequest,
    canonical_project_id: &str,
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
    if canonical_workflow_record_index(&value, &workflow_id, canonical_project_id)?.is_none() {
        return Err("当前项目还没有本地 workflow；无法准备节点派发".to_string());
    }
    if canonical_node_record_index(&value, &workflow_id, &request.node_id, canonical_project_id)?
        .is_none()
    {
        return Err("当前 workflow 下找不到该 node；无法准备节点派发".to_string());
    }
    let work_item_index = canonical_work_item_record_index(
        &value,
        &workflow_id,
        &request.work_item_id,
        canonical_project_id,
    )?
    .ok_or_else(|| "当前 workflow 下找不到该 work item；无法准备节点派发".to_string())?;
    let work_item = value
        .get("work_items")
        .and_then(Value::as_array)
        .and_then(|work_items| work_items.get(work_item_index))
        .ok_or_else(|| "当前 workflow 下找不到该 work item；无法准备节点派发".to_string())?;
    let work_item_state =
        optional_string_from(work_item, "state").unwrap_or_else(|| "unknown".to_string());
    let exact_binding_index = canonical_binding_record_index(
        &value,
        &workflow_id,
        &request.node_id,
        Some(&request.work_item_id),
        canonical_project_id,
    )?;
    let binding_index = match exact_binding_index {
        Some(index) => Some(index),
        None => canonical_binding_record_index(
            &value,
            &workflow_id,
            &request.node_id,
            None,
            canonical_project_id,
        )?,
    }
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
    let memory_artifact_index = canonical_task_package_artifact_record_index(
        &value,
        &request.work_item_id,
        work_item,
        canonical_project_id,
    )?;
    let memory_snapshot = memory_artifact_index
        .and_then(|index| value.get("artifacts").and_then(Value::as_array)?.get(index))
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
        project_id: canonical_project_id.to_string(),
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
        prepared_dispatch_id: None,
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
    // Legacy readiness helper retained for crate-root fixtures.  The formal
    // command resolves M1 first and calls the canonical variant below.  Remove
    // this wrapper when the old lifecycle fixture accepts an explicit owner.
    inspect_task_package_authorization_with_canonical_project_id_at(
        path,
        project,
        workflow_id,
        work_item,
        artifact,
        fields,
        dispatch_kind,
        &project_id(&project.project_root),
    )
}

fn inspect_task_package_authorization_with_canonical_project_id_at(
    path: &Path,
    _project: &ProjectRecord,
    workflow_id: &str,
    work_item: &Value,
    artifact: &Value,
    fields: &RenderTaskPackageFields,
    dispatch_kind: &str,
    canonical_project_id: &str,
) -> Result<AutoDispatchGuardResult, String> {
    let work_item_id = fields.work_item_id.clone();
    let target_role_id = optional_string_from(work_item, "assigned_role_id")
        .or_else(|| optional_string_from(artifact, "target_role"))
        .unwrap_or_else(|| assigned_line_id(&fields.assigned_line).to_string());
    let input = AutoDispatchGuardInput {
        project_id: canonical_project_id.to_string(),
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

#[cfg(test)]
mod m4_source_owner_candidate_rejection_tests {
    use super::*;
    include!("workflow_run_dispatch_entrypoints_m4r02_tests.rs");
}
