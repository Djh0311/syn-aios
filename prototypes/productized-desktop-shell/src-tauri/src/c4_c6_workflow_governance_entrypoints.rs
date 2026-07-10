#[derive(Clone)]
struct ProjectDirectorAuthorizationContext {
    project: ProjectRecord,
    proposal: ProjectConsultationProposal,
    authorization: PlanAuthorization,
    authorization_store: PlanAuthorizationStoreV1,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct ActiveBindingInfo {
    binding_id: String,
    native_thread_id: String,
    rollout_exists: bool,
    warnings: Vec<String>,
}

fn preview_project_director_task_plan_for_index_at(
    path: &Path,
    index: &Value,
    request: &PreviewProjectDirectorTaskPlanInput,
) -> Result<ProjectDirectorTaskPlan, String> {
    let value = read_c4_workflow_value(path, &request.project_root, &request.workflow_id)?;
    let context = project_director_authorization_context(
        path,
        index,
        &request.project_root,
        &request.project_id,
        &request.workflow_id,
        &request.proposal_id,
        &request.authorization_id,
        request.expected_authorization_revision,
    )?;
    let tasks = annotate_project_director_planned_tasks(
        index,
        &value,
        &context,
        deterministic_project_director_planned_tasks(&context),
        unix_timestamp_ms(),
    );
    Ok(project_director_task_plan_from_tasks(
        &request.project_root,
        &request.project_id,
        &request.workflow_id,
        &request.proposal_id,
        &request.authorization_id,
        &request.actor_id,
        tasks,
        &value,
    ))
}

fn prepare_authorized_auto_dispatch_for_index_at(
    path: &Path,
    index: &Value,
    request: &PrepareAuthorizedAutoDispatchInput,
) -> Result<AuthorizedPreparedDispatchResult, String> {
    let mut value = read_c4_workflow_value(path, &request.project_root, &request.workflow_id)?;
    if let Some(expected) = request.expected_workflow_revision {
        let current = i64_value(&value, "workflow_version").unwrap_or_default();
        if current != expected {
            return Err(format!(
                "workflow revision 不匹配：expected {expected}, actual {current}"
            ));
        }
    }
    let context = project_director_authorization_context(
        path,
        index,
        &request.project_root,
        &request.project_id,
        &request.workflow_id,
        &request.proposal_id,
        &request.authorization_id,
        request.expected_authorization_revision,
    )?;
    let timestamp = unix_timestamp_string();
    let timestamp_ms = unix_timestamp_ms();
    let mut backup_path: Option<PathBuf> = None;
    let mut audit_event_id = format!(
        "no-op:authorized-prepared-dispatch:{}",
        request.authorization_id
    );
    let planned_tasks = if request.planned_tasks.is_empty() {
        deterministic_project_director_planned_tasks(&context)
    } else {
        request.planned_tasks.clone()
    };
    let mut annotated = annotate_project_director_planned_tasks(
        index,
        &value,
        &context,
        planned_tasks.clone(),
        timestamp_ms,
    );

    // 2.3·任务级工序图（纯加法）：收「过了 guard 的」任务的 (任务节点 id, planned_task_id, title, depends_on)，
    // 循环后据此建依赖边。task_node_order 只对**已物化**任务递增（blocked 不建节点·不占位·§3 边界语义不动）。
    let mut task_graph: Vec<(String, String, String, Vec<String>)> = Vec::new();
    let mut task_node_order = 0usize;

    for task in &mut annotated {
        if task
            .guard_result
            .as_ref()
            .is_none_or(|guard| guard.status != "authorized")
            || !task.blocked_reasons.is_empty() && task.status == "blocked"
        {
            ensure_c4_backup(path, &timestamp, &mut backup_path)?;
            audit_event_id = push_authorized_prepared_dispatch_blocked_audit(
                &mut value,
                task,
                &context.authorization.authorization_id,
                &timestamp,
                "authorized_prepared_dispatch_blocked",
                "项目主管拆任务授权检查未通过；未创建 prepared dispatch。",
            )?;
            continue;
        }

        let work_item_id =
            c4_work_item_id(&context.authorization.workflow_id, &task.planned_task_id);
        let artifact_id =
            c4_task_package_artifact_id(&context.authorization.workflow_id, &task.planned_task_id);
        let node_id = c4_node_id(&context.authorization.workflow_id, &task.scope.target_role);
        task.work_item_id = Some(work_item_id.clone());
        task.task_package_id = Some(artifact_id.clone());
        task.workflow_node_id = Some(node_id.clone());

        ensure_c4_backup(path, &timestamp, &mut backup_path)?;
        ensure_project_director_worker_node(&mut value, &node_id, task, &timestamp)?;
        // 2.3·加法：给「过了 guard 的」每个任务额外落一个任务级节点（≠上面的 role 节点·带 :task: 后缀·无执行口），
        // 并记进 task_graph 供循环后建依赖边。老 role 节点/work_item/binding/workflow_node_id 锚全部原样。
        let task_level_node_id =
            ensure_project_director_task_level_node(&mut value, task, task_node_order, &timestamp)?;
        task_node_order += 1;
        task_graph.push((
            task_level_node_id,
            task.planned_task_id.clone(),
            task.title.clone(),
            task.depends_on.clone(),
        ));
        ensure_project_director_work_item(
            &mut value,
            &work_item_id,
            &artifact_id,
            task,
            &timestamp,
        )?;
        let memory_snapshot = project_director_memory_snapshot(
            path,
            &request.project_root,
            task,
            &work_item_id,
            &artifact_id,
            &timestamp,
        )?;
        task.memory_packet_snapshot_id = Some(memory_snapshot.snapshot_id.clone());
        ensure_project_director_task_package_artifact(
            &mut value,
            &context.project,
            &work_item_id,
            &artifact_id,
            task,
            &memory_snapshot,
            &timestamp,
        )?;

        let binding = active_binding_for_planned_task(index, &value, &node_id, &work_item_id);
        // C1·chain_binds_per_task（canon 2026-07-09·架构收官）：链会每任务 create_and_bind 真会话 → 无绑定/
        // rollout 缺时**不判 needs_binding**，改产 prepared·thread 延迟（下方 binding_id/native_thread_id 置 null +
        // thread_binding_deferred 标记 + 审计变体·透明不吞）。**只放宽「有无会话」就绪判定·授权/安全一条不松**
        // （guard_result 授权检查照旧）。false 路（手动挡/existing/前端预 prepare）needs_binding 判定**逐字不变**。
        let thread_deferred = request.chain_binds_per_task
            && binding.as_ref().map(|b| !b.rollout_exists).unwrap_or(true);
        if !thread_deferred {
            let Some(binding) = binding.as_ref() else {
                task.status = "needs_binding".to_string();
                push_unique(&mut task.blocked_reasons, "等待绑定会话后才能准备派发。");
                audit_event_id = push_authorized_prepared_dispatch_blocked_audit(
                    &mut value,
                    task,
                    &context.authorization.authorization_id,
                    &timestamp,
                    "authorized_prepared_dispatch_blocked",
                    "项目主管拆任务已生成任务包草案和记忆快照，但目标 worker 节点缺少 active binding；未创建 prepared dispatch。",
                )?;
                continue;
            };
            if !binding.rollout_exists {
                task.status = "needs_binding".to_string();
                push_unique(
                    &mut task.blocked_reasons,
                    "绑定会话 rollout 不可用，等待重新绑定。",
                );
                audit_event_id = push_authorized_prepared_dispatch_blocked_audit(
                    &mut value,
                    task,
                    &context.authorization.authorization_id,
                    &timestamp,
                    "authorized_prepared_dispatch_blocked",
                    "项目主管拆任务已生成任务包草案和记忆快照，但绑定会话不可用；未创建 prepared dispatch。",
                )?;
                continue;
            }
        }

        if let Some(existing_dispatch) = existing_prepared_dispatch_for_planned_task(
            &value,
            &task.planned_task_id,
            &work_item_id,
            &context.authorization.authorization_id,
        ) {
            task.status = "prepared".to_string();
            task.prepared_dispatch_id = optional_string_from(existing_dispatch, "dispatch_id");
            continue;
        }

        let prompt_preview = render_project_director_prepared_prompt(task, &memory_snapshot);
        let dispatch_id = format!(
            "authorized-prepared-dispatch:{}:{}",
            stable_id(&task.planned_task_id),
            timestamp
        );
        let authorization_check = task.guard_result.clone().ok_or_else(|| {
            "项目主管 planned task 缺少授权检查结果，不能创建 prepared dispatch".to_string()
        })?;
        // C1 延迟：thread_deferred 时 binding_id/native_thread_id 置 null（链每任务补真会话·下游据
        // thread_binding_deferred 知道）；非延迟（含 false 路真绑定）用真绑定。授权检查上面已取·不受影响。
        let (binding_id_json, native_thread_json, binding_warnings): (Value, Value, Vec<String>) =
            match binding.as_ref() {
                Some(existing) if !thread_deferred => (
                    json!(existing.binding_id),
                    json!(existing.native_thread_id),
                    existing.warnings.clone(),
                ),
                _ => (Value::Null, Value::Null, Vec::new()),
            };
        array_mut(&mut value, "workflow_node_dispatches")?.push(json!({
          "dispatch_id": dispatch_id,
          "project_id": context.authorization.project_id,
          "workflow_id": context.authorization.workflow_id,
          "node_id": node_id,
          "work_item_id": work_item_id,
          "binding_id": binding_id_json,
          "native_thread_id": native_thread_json,
          "thread_binding_deferred": thread_deferred,
          "prompt_preview": prompt_preview,
          "prompt_kind": "authorized_prepared_auto_dispatch",
          "memory_packet_snapshot_id": memory_snapshot.snapshot_id,
          "memory_packet_fingerprint": memory_snapshot.fingerprint,
          "plan_authorization_id": context.authorization.authorization_id,
          "authorization_check": serde_json::to_value(&authorization_check).unwrap_or(Value::Null),
          "state": "prepared",
          "started_at_ms": Value::Null,
          "ended_at_ms": Value::Null,
          "exit_code": Value::Null,
          "last_message_path": Value::Null,
          "last_message_summary": Value::Null,
          "transcript_event_count": Value::Null,
          "transcript_target_hits": Value::Null,
          "c4_planned_task_id": task.planned_task_id,
          "warnings": dedupe_strings({
              let mut warnings = vec![
                  "prepared_only_no_worker_execution".to_string(),
                  "task_memory_packet_snapshot_attached".to_string()
              ];
              if thread_deferred {
                  warnings.push("thread_binding_deferred_chain_binds_per_task".to_string());
              }
              warnings.extend(binding_warnings.clone());
              warnings
          }),
          "created_at_ms": timestamp_ms,
          "updated_at_ms": timestamp_ms
        }));
        task.status = "prepared".to_string();
        task.prepared_dispatch_id = Some(dispatch_id.clone());
        // C1 延迟：透明审计新变体（不吞·主导线可见「thread 由链补」）；非延迟走原 created 审计。
        audit_event_id = if thread_deferred {
            push_authorized_prepared_dispatch_thread_deferred_audit(
                &mut value,
                task,
                &dispatch_id,
                &context.authorization.authorization_id,
                &timestamp,
            )?
        } else {
            push_authorized_prepared_dispatch_created_audit(
                &mut value,
                task,
                &dispatch_id,
                &context.authorization.authorization_id,
                &timestamp,
            )?
        };
    }

    // 2.3·依赖边（循环后·所有任务级节点都已建·才能 title→节点 id 映射）：depends_on 按 title 连（同链的先例·
    // director_agent run_director_task_chain）。悬空依赖（title 不在已物化任务集里·如指向 blocked 任务）→记
    // warning **不建边**。纯加法·edge_exists 幂等（重跑 prepare 不重复建）。写在 backup-write 之前 → 随现成 write 落盘。
    let mut graph_warnings: Vec<String> = Vec::new();
    let title_to_node: std::collections::BTreeMap<&str, (&str, &str)> = task_graph
        .iter()
        .map(|(node_id, planned_task_id, title, _)| {
            (title.as_str(), (node_id.as_str(), planned_task_id.as_str()))
        })
        .collect();
    for (to_node_id, to_planned_task_id, to_title, depends_on) in &task_graph {
        for dep_title in depends_on {
            match title_to_node.get(dep_title.as_str()) {
                Some((from_node_id, from_planned_task_id)) => {
                    ensure_project_director_task_dep_edge(
                        &mut value,
                        &request.workflow_id,
                        from_node_id,
                        to_node_id,
                        from_planned_task_id,
                        to_planned_task_id,
                        &timestamp,
                    )?;
                }
                None => graph_warnings.push(format!(
                    "任务「{to_title}」依赖「{dep_title}」未在已授权任务里建节点（悬空/被阻断），未建依赖边。"
                )),
            }
        }
    }

    if backup_path.is_some() {
        value["updated_at"] = Value::String(timestamp.clone());
        write_validated_workflow_state(path, &value)?;
    }

    let updated_value = read_workflow_state_value(path)?;
    let final_tasks = annotate_project_director_planned_tasks(
        index,
        &updated_value,
        &context,
        planned_tasks,
        timestamp_ms,
    );
    let plan = project_director_task_plan_from_tasks(
        &request.project_root,
        &request.project_id,
        &request.workflow_id,
        &request.proposal_id,
        &request.authorization_id,
        &request.actor_id,
        final_tasks,
        &updated_value,
    );
    let prepared_dispatches = prepared_dispatch_read_models_from_plan(&plan, &updated_value);
    let snapshot = read_workflow_state_snapshot(path)?;
    // 2.3：把工序图悬空依赖 warning 并入返回（现成 plan.warnings 之外的加法·不改 plan 组装）。
    let mut warnings = plan.warnings.clone();
    warnings.extend(graph_warnings);
    Ok(AuthorizedPreparedDispatchResult {
        message: format!(
            "项目主管拆任务已准备：planned {} / prepared {} / needs_binding {} / blocked {}；已准备；仍未执行 worker。",
            plan.planned_task_count,
            plan.prepared_dispatch_count,
            plan.needs_binding_count,
            plan.blocked_count
        ),
        path: path.display().to_string(),
        backup_path: backup_path.map(|path| path.display().to_string()),
        audit_event_id,
        plan,
        prepared_dispatches,
        snapshot,
        warnings,
    })
}

fn record_worker_structured_report_at(
    path: &Path,
    request: &WorkerStructuredReportInput,
) -> Result<WorkflowStateMutationResult, String> {
    if !path.exists() {
        return Err("工作流状态文件不存在；无法记录 worker 结构化汇报".to_string());
    }
    validate_worker_structured_report_input(request)?;
    let timestamp = unix_timestamp_string();
    let mut value = read_workflow_state_value(path)?;
    if let Some(expected) = request.expected_workflow_revision {
        let current = i64_value(&value, "workflow_version").unwrap_or_default();
        if current != expected {
            return Err(format!(
                "workflow revision 不匹配：expected {expected}, actual {current}"
            ));
        }
    }
    let validation_warnings = validate_workflow_state(&value);
    if !validation_warnings.is_empty() {
        return Err(format!(
            "当前状态文件未通过 schema 校验：{}",
            validation_warnings.join(", ")
        ));
    }
    if !workflow_exists(&value, &request.workflow_id) {
        return Err("当前项目还没有目标 workflow；无法记录 worker 结构化汇报".to_string());
    }
    if find_work_item(&value, &request.workflow_id, &request.work_item_id).is_none() {
        return Err("当前 workflow 下找不到该 work item；无法记录 worker 结构化汇报".to_string());
    }
    if !node_exists(&value, &request.workflow_id, &request.workflow_node_id) {
        return Err("当前 workflow 下找不到该 worker node；无法记录 worker 结构化汇报".to_string());
    }
    if let Some(dispatch_id) = request.dispatch_id.as_deref() {
        let dispatch = find_workflow_node_dispatch(&value, dispatch_id)
            .ok_or_else(|| "找不到关联 dispatch；无法记录 worker 结构化汇报".to_string())?;
        if optional_string_from(dispatch, "workflow_id").as_deref()
            != Some(request.workflow_id.as_str())
            || optional_string_from(dispatch, "work_item_id").as_deref()
                != Some(request.work_item_id.as_str())
        {
            return Err(
                "关联 dispatch 不属于目标 work item，已拒绝记录 worker 结构化汇报".to_string(),
            );
        }
        let state = optional_string_from(dispatch, "state").unwrap_or_default();
        if !matches!(
            state.as_str(),
            "prepared" | "completed" | "failed" | "timed_out" | "cancelled"
        ) {
            return Err(format!(
                "关联 dispatch 状态 {state} 不能作为 C5 worker 汇报来源"
            ));
        }
    }

    let backup = backup_workflow_state_file(path, &timestamp)?;
    let report_id = format!(
        "worker-report:{}:{}:{}",
        stable_id(&request.work_item_id),
        stable_id(&request.summary),
        timestamp
    );
    array_mut(&mut value, "audit_events")?.push(json!({
      "event_id": report_id,
      "event_type": "worker_structured_report_recorded",
      "target_ref": request.work_item_id,
      "project_id": request.project_id,
      "workflow_id": request.workflow_id,
      "node_id": request.workflow_node_id,
      "work_item_id": request.work_item_id,
      "dispatch_id": request.dispatch_id,
      "actor_ref": request.actor_role,
      "source_kind": "worker_handoff",
      "permission_level": "workflow_event_record",
      "executed_what": request.executed_what.trim(),
      "changed_what": request.changed_what.trim(),
      "reason": request.summary.trim(),
      "evidence_refs": request.evidence_refs,
      "open_issues": request.open_issues,
      "permission_requests": request.permission_requests,
      "direction_risks": request.direction_risks,
      "follow_up_suggestions": request.follow_up_suggestions,
      "acceptance_status": request.acceptance_status.trim(),
      "source_refs": request.source_refs,
      "created_at": timestamp,
      "warnings": [
        "worker_report_is_not_formal_fact",
        "worker_report_is_not_formal_memory",
        "project_director_confirmation_required"
      ]
    }));
    value["updated_at"] = Value::String(timestamp);
    write_validated_workflow_state(path, &value)?;
    let snapshot = read_workflow_state_snapshot(path)?;
    Ok(WorkflowStateMutationResult {
        message: "worker 结构化汇报已记录；仍不是正式事实或正式记忆。".to_string(),
        path: path.display().to_string(),
        backup_path: Some(backup.display().to_string()),
        audit_event_id: report_id,
        first_initialize: false,
        snapshot,
    })
}

fn record_project_director_process_fact_decision_at(
    path: &Path,
    request: &ProjectDirectorProcessFactDecisionInput,
) -> Result<ProjectDirectorProcessFactDecisionResult, String> {
    if !path.exists() {
        return Err("工作流状态文件不存在；无法记录过程事实确认".to_string());
    }
    validate_process_fact_decision_input(request)?;
    let timestamp = unix_timestamp_string();
    let mut value = read_workflow_state_value(path)?;
    if let Some(expected) = request.expected_workflow_revision {
        let current = i64_value(&value, "workflow_version").unwrap_or_default();
        if current != expected {
            return Err(format!(
                "workflow revision 不匹配：expected {expected}, actual {current}"
            ));
        }
    }
    let validation_warnings = validate_workflow_state(&value);
    if !validation_warnings.is_empty() {
        return Err(format!(
            "当前状态文件未通过 schema 校验：{}",
            validation_warnings.join(", ")
        ));
    }
    if !workflow_exists(&value, &request.workflow_id) {
        return Err("当前项目还没有目标 workflow；无法记录过程事实确认".to_string());
    }
    let report = find_worker_report_event(&value, &request.report_id)
        .ok_or_else(|| "找不到 worker 汇报；无法记录过程事实确认".to_string())?
        .clone();
    if optional_string_from(&report, "workflow_id").as_deref() != Some(request.workflow_id.as_str())
    {
        return Err("worker 汇报不属于目标 workflow，已拒绝确认过程事实".to_string());
    }
    if process_fact_decision_exists(&value, &request.report_id, &request.accepted_facts) {
        return Err("process_fact_duplicate: 同一 report / process fact 已确认过".to_string());
    }

    let mut observations = Vec::new();
    let mut observation_store_revision = request.expected_observation_store_revision;
    if request.decision == "confirm_process_fact" {
        for fact in &request.accepted_facts {
            validate_process_fact_candidate(fact, request)?;
            let observation_input = CreateObservationInput {
                project_root: request.project_root.clone(),
                project_id: Some(request.project_id.clone()),
                workflow_id: Some(request.workflow_id.clone()),
                scope: fact.scope.clone(),
                observation_type: fact.proposed_observation_type.clone(),
                summary: fact.summary.trim().to_string(),
                source_refs: fact.source_refs.clone(),
                generated_by_role: "project_director".to_string(),
                actor_id: request.actor_id.clone(),
                risk_level: fact.risk_level.clone(),
                sensitive_level: fact.sensitive_level.clone(),
                reason: request.summary.trim().to_string(),
                expected_store_revision: observation_store_revision,
            };
            let output = create_observation_at(
                path,
                &observation_input,
                &timestamp,
                &format!(
                    "write-c5-process-fact-observation-{}-{}",
                    stable_id(&fact.process_fact_id),
                    unix_timestamp_nanos()
                ),
            )?;
            observation_store_revision = Some(output.store_revision);
            observations.push(output.observation);
        }
    }

    let backup = backup_workflow_state_file(path, &timestamp)?;
    let decision_record_id = format!(
        "process-fact-decision:{}:{}",
        stable_id(&request.report_id),
        timestamp
    );
    let audit_event_id = format!(
        "audit:process-fact-decision:{}:{}",
        stable_id(&request.report_id),
        timestamp
    );
    let observation_ids = observations
        .iter()
        .map(|observation| observation.observation_id.clone())
        .collect::<Vec<_>>();
    let accepted_fact_ids = request
        .accepted_facts
        .iter()
        .map(|fact| fact.process_fact_id.clone())
        .collect::<Vec<_>>();
    let evidence_refs = request
        .accepted_facts
        .iter()
        .flat_map(|fact| fact.evidence_refs.clone())
        .collect::<Vec<_>>();
    array_mut(&mut value, "reviews")?.push(json!({
      "review_id": decision_record_id,
      "project_id": request.project_id,
      "workflow_id": request.workflow_id,
      "work_item_id": optional_string_from(&report, "work_item_id"),
      "dispatch_id": optional_string_from(&report, "dispatch_id"),
      "workflow_node_id": optional_string_from(&report, "node_id"),
      "report_id": request.report_id,
      "reviewer_role": "project_director",
      "decision": request.decision,
      "summary": request.summary.trim(),
      "accepted_fact_ids": accepted_fact_ids,
      "rejected_fact_ids": request.rejected_fact_ids,
      "observation_ids": observation_ids,
      "evidence_refs": evidence_refs,
      "created_at": timestamp,
      "updated_at": timestamp,
      "warnings": [
        "process_fact_observation_is_not_formal_memory",
        "worker_report_not_direct_formal_fact",
        "global_director_final_review_still_required"
      ]
    }));
    array_mut(&mut value, "audit_events")?.push(json!({
      "event_id": audit_event_id,
      "event_type": "project_director_process_fact_decision_recorded",
      "target_ref": request.report_id,
      "project_id": request.project_id,
      "workflow_id": request.workflow_id,
      "actor_ref": request.actor_id,
      "source_kind": "project_director_confirmation",
      "permission_level": "workflow_event_record",
      "decision": request.decision,
      "accepted_fact_ids": request.accepted_facts.iter().map(|fact| fact.process_fact_id.clone()).collect::<Vec<_>>(),
      "rejected_fact_ids": request.rejected_fact_ids,
      "observation_ids": observations.iter().map(|observation| observation.observation_id.clone()).collect::<Vec<_>>(),
      "created_at": timestamp,
      "reason": request.summary.trim()
    }));
    value["updated_at"] = Value::String(timestamp);
    write_validated_workflow_state(path, &value)?;
    let snapshot = read_workflow_state_snapshot(path)?;
    let message = match request.decision.as_str() {
        "confirm_process_fact" => {
            "项目主管已确认过程事实；已记录为观察，仍不是正式记忆。".to_string()
        }
        "request_rework" => "项目主管已要求返工；未写入过程事实 observation。".to_string(),
        "block_and_escalate" => "项目主管已阻断并上报；未写入过程事实 observation。".to_string(),
        _ => "项目主管过程事实决定已记录。".to_string(),
    };
    Ok(ProjectDirectorProcessFactDecisionResult {
        message,
        path: path.display().to_string(),
        backup_path: Some(backup.display().to_string()),
        audit_event_id,
        decision_record_id,
        observations,
        observation_store_revision,
        snapshot,
        warnings: vec![
            "worker_report_not_direct_formal_fact".to_string(),
            "observation_is_not_formal_memory".to_string(),
        ],
    })
}

fn record_global_final_result_review_at(
    path: &Path,
    request: &GlobalFinalResultReviewInput,
) -> Result<WorkflowStateMutationResult, String> {
    if !path.exists() {
        return Err("工作流状态文件不存在；无法记录全局最终复核".to_string());
    }
    validate_global_final_result_review_input(request)?;
    let timestamp = unix_timestamp_string();
    let mut value = read_workflow_state_value(path)?;
    validate_expected_workflow_revision(&value, request.expected_workflow_revision)?;
    let validation_warnings = validate_workflow_state(&value);
    if !validation_warnings.is_empty() {
        return Err(format!(
            "当前状态文件未通过 schema 校验：{}",
            validation_warnings.join(", ")
        ));
    }
    validate_c6_prerequisites_for_final_review(path, &value, request)?;
    let confirmed_fact_ids = confirmed_process_fact_ids_for_workflow(&value, &request.workflow_id);
    if request.decision == "accepted" {
        if request.accepted_process_fact_ids.is_empty() {
            return Err("全局最终复核 accepted 必须引用已确认过程事实".to_string());
        }
        for fact_id in &request.accepted_process_fact_ids {
            if !confirmed_fact_ids
                .iter()
                .any(|confirmed| confirmed == fact_id)
            {
                return Err(format!("全局最终复核引用了未确认的过程事实：{fact_id}"));
            }
        }
        let unresolved = unresolved_process_fact_decisions(&value, &request.workflow_id);
        if !unresolved.is_empty() {
            return Err(format!(
                "仍有返工 / 阻断过程事实未处理，不能记录 accepted：{}",
                unresolved.join("；")
            ));
        }
    }

    let backup = backup_workflow_state_file(path, &timestamp)?;
    let review_id = format!(
        "global-final-review:{}:{}:{}",
        stable_id(&request.workflow_id),
        stable_id(&request.decision),
        timestamp
    );
    let audit_event_id = format!(
        "audit:global-final-review:{}:{}",
        stable_id(&review_id),
        timestamp
    );
    array_mut(&mut value, "reviews")?.push(json!({
      "review_id": review_id,
      "project_id": request.project_id,
      "workflow_id": request.workflow_id,
      "reviewer_role": "global_director",
      "review_target": "global_final_result",
      "proposal_id": request.proposal_id,
      "authorization_id": request.authorization_id,
      "decision": request.decision,
      "summary": request.summary.trim(),
      "accepted_fact_ids": request.accepted_process_fact_ids,
      "observation_ids": [],
      "evidence_refs": request.evidence_refs,
      "open_issues": request.open_issues,
      "deferred_items": request.deferred_items,
      "created_at": timestamp,
      "updated_at": timestamp,
      "warnings": [
        "global_final_review_is_not_user_acceptance",
        "process_fact_observation_is_not_formal_memory",
        "stage_c_acceptance_summary_still_required"
      ]
    }));
    array_mut(&mut value, "audit_events")?.push(json!({
      "event_id": audit_event_id,
      "event_type": "global_final_result_review_recorded",
      "target_ref": request.workflow_id,
      "project_id": request.project_id,
      "workflow_id": request.workflow_id,
      "actor_ref": request.actor_id,
      "source_kind": "global_director_final_review",
      "permission_level": "workflow_event_record",
      "decision": request.decision,
      "review_id": review_id,
      "proposal_id": request.proposal_id,
      "authorization_id": request.authorization_id,
      "accepted_process_fact_ids": request.accepted_process_fact_ids,
      "created_at": timestamp,
      "reason": request.summary.trim()
    }));
    value["updated_at"] = Value::String(timestamp);
    write_validated_workflow_state(path, &value)?;
    let snapshot = read_workflow_state_snapshot(path)?;
    Ok(WorkflowStateMutationResult {
        message: "全局主管已完成最终复核；这不代表用户已接受，也不写正式记忆。".to_string(),
        path: path.display().to_string(),
        backup_path: Some(backup.display().to_string()),
        audit_event_id,
        first_initialize: false,
        snapshot,
    })
}

fn record_user_result_decision_at(
    path: &Path,
    request: &UserResultDecisionInput,
) -> Result<WorkflowStateMutationResult, String> {
    if !path.exists() {
        return Err("工作流状态文件不存在；无法记录用户结果决定".to_string());
    }
    validate_user_result_decision_input(request)?;
    let timestamp = unix_timestamp_string();
    let mut value = read_workflow_state_value(path)?;
    validate_expected_workflow_revision(&value, request.expected_workflow_revision)?;
    let validation_warnings = validate_workflow_state(&value);
    if !validation_warnings.is_empty() {
        return Err(format!(
            "当前状态文件未通过 schema 校验：{}",
            validation_warnings.join(", ")
        ));
    }
    if !workflow_exists(&value, &request.workflow_id) {
        return Err("当前项目还没有目标 workflow；无法记录用户结果决定".to_string());
    }
    let global_review = latest_global_final_review(&value, &request.workflow_id)
        .ok_or_else(|| "缺少全局主管最终复核，不能记录用户结果决定".to_string())?;
    let global_review_id = optional_string_from(global_review, "review_id")
        .unwrap_or_else(|| "global-final-review:missing".to_string());
    if let Some(accepted_review_id) = request.accepted_review_id.as_deref() {
        if accepted_review_id != global_review_id {
            return Err("用户结果决定引用的全局最终复核不是当前最新复核".to_string());
        }
    }
    if request.decision == "accept_result"
        && optional_string_from(global_review, "decision").as_deref() != Some("accepted")
    {
        return Err("全局最终复核未 accepted，用户不能记录接受结果".to_string());
    }

    let backup = backup_workflow_state_file(path, &timestamp)?;
    let decision_id = format!(
        "user-result-decision:{}:{}:{}",
        stable_id(&request.workflow_id),
        stable_id(&request.decision),
        timestamp
    );
    let audit_event_id = format!(
        "audit:user-result-decision:{}:{}",
        stable_id(&decision_id),
        timestamp
    );
    array_mut(&mut value, "reviews")?.push(json!({
      "review_id": decision_id,
      "project_id": request.project_id,
      "workflow_id": request.workflow_id,
      "reviewer_role": "user",
      "review_target": "user_result_decision",
      "accepted_review_id": request.accepted_review_id,
      "decision": request.decision,
      "summary": request.summary.trim(),
      "accepted_fact_ids": [],
      "observation_ids": [],
      "evidence_refs": [global_review_id],
      "requested_changes": request.requested_changes,
      "created_at": timestamp,
      "updated_at": timestamp,
      "warnings": [
        "user_result_decision_is_for_this_result_only",
        "user_decision_does_not_write_formal_memory"
      ]
    }));
    array_mut(&mut value, "audit_events")?.push(json!({
      "event_id": audit_event_id,
      "event_type": "user_result_decision_recorded",
      "target_ref": request.workflow_id,
      "project_id": request.project_id,
      "workflow_id": request.workflow_id,
      "actor_ref": request.actor_id,
      "source_kind": "user_result_view",
      "permission_level": "workflow_event_record",
      "decision": request.decision,
      "accepted_review_id": request.accepted_review_id,
      "created_at": timestamp,
      "reason": request.summary.trim()
    }));
    value["updated_at"] = Value::String(timestamp);
    write_validated_workflow_state(path, &value)?;
    let snapshot = read_workflow_state_snapshot(path)?;
    Ok(WorkflowStateMutationResult {
        message: "用户已查看结果并作出决定；只适用于本次结果，不写正式记忆。".to_string(),
        path: path.display().to_string(),
        backup_path: Some(backup.display().to_string()),
        audit_event_id,
        first_initialize: false,
        snapshot,
    })
}

fn generate_stage_c_acceptance_summary_at(
    path: &Path,
    request: &GenerateStageCAcceptanceSummaryInput,
) -> Result<WorkflowStateMutationResult, String> {
    if !path.exists() {
        return Err("工作流状态文件不存在；无法生成阶段 C 验收摘要".to_string());
    }
    validate_generate_stage_c_acceptance_summary_input(request)?;
    let timestamp = unix_timestamp_string();
    let mut value = read_workflow_state_value(path)?;
    validate_expected_workflow_revision(&value, request.expected_workflow_revision)?;
    let validation_warnings = validate_workflow_state(&value);
    if !validation_warnings.is_empty() {
        return Err(format!(
            "当前状态文件未通过 schema 校验：{}",
            validation_warnings.join(", ")
        ));
    }
    if !workflow_exists(&value, &request.workflow_id) {
        return Err("当前项目还没有目标 workflow；无法生成阶段 C 验收摘要".to_string());
    }
    let summary =
        build_stage_c_acceptance_summary(path, &value, &request.project_id, &request.workflow_id)?;
    let backup = backup_workflow_state_file(path, &timestamp)?;
    let artifact_id = format!(
        "stage-c-acceptance-summary:{}:{}",
        stable_id(&request.workflow_id),
        timestamp
    );
    let audit_event_id = format!(
        "audit:stage-c-acceptance-summary:{}:{}",
        stable_id(&request.workflow_id),
        timestamp
    );
    array_mut(&mut value, "artifacts")?.push(json!({
      "artifact_id": artifact_id,
      "artifact_type": "stage_c_acceptance_summary",
      "project_id": request.project_id,
      "workflow_id": request.workflow_id,
      "source_kind": "stage_c_acceptance",
      "source_ref": request.workflow_id,
      "permission_level": "workflow_event_record",
      "title": "阶段 C 验收摘要",
      "brief": if summary.accepted_as_stage_c_complete { "阶段 C gates 已通过，后置项已列明。" } else { "阶段 C gates 尚未全部通过。" },
      "stage_c_acceptance_summary": summary,
      "created_at": timestamp,
      "updated_at": timestamp,
      "warnings": [
        "stage_c_summary_does_not_complete_middle_version",
        "deferred_items_remain_out_of_scope_for_c6"
      ]
    }));
    array_mut(&mut value, "audit_events")?.push(json!({
      "event_id": audit_event_id,
      "event_type": "stage_c_acceptance_summary_generated",
      "target_ref": request.workflow_id,
      "project_id": request.project_id,
      "workflow_id": request.workflow_id,
      "actor_ref": "control_core",
      "source_kind": "stage_c_acceptance",
      "permission_level": "workflow_event_record",
      "created_at": timestamp,
      "reason": "生成阶段 C 验收 gate 摘要；不执行真实 worker，不写正式记忆。"
    }));
    value["updated_at"] = Value::String(timestamp);
    write_validated_workflow_state(path, &value)?;
    let snapshot = read_workflow_state_snapshot(path)?;
    Ok(WorkflowStateMutationResult {
        message: if snapshot
            .project_workflows
            .iter()
            .find(|workflow| workflow.workflow_id == request.workflow_id)
            .and_then(|workflow| workflow.derived_workflow.as_ref())
            .is_some_and(|workflow| {
                workflow
                    .result_summary
                    .stage_c_acceptance
                    .accepted_as_stage_c_complete
            }) {
            "阶段 C 验收摘要已生成：gates 已通过；这不代表中间版本整体完成。".to_string()
        } else {
            "阶段 C 验收摘要已生成：仍有缺口、修改项、阻断或后置项。".to_string()
        },
        path: path.display().to_string(),
        backup_path: Some(backup.display().to_string()),
        audit_event_id,
        first_initialize: false,
        snapshot,
    })
}

fn validate_expected_workflow_revision(value: &Value, expected: Option<i64>) -> Result<(), String> {
    if let Some(expected) = expected {
        let current = i64_value(value, "workflow_version").unwrap_or_default();
        if current != expected {
            return Err(format!(
                "workflow revision 不匹配：expected {expected}, actual {current}"
            ));
        }
    }
    Ok(())
}

fn validate_global_final_result_review_input(
    request: &GlobalFinalResultReviewInput,
) -> Result<(), String> {
    if request.actor_role != "global_director" {
        return Err("只有全局主管可以记录最终结果复核".to_string());
    }
    for (label, value) in [
        ("project_root", request.project_root.as_str()),
        ("project_id", request.project_id.as_str()),
        ("workflow_id", request.workflow_id.as_str()),
        ("authorization_id", request.authorization_id.as_str()),
        ("proposal_id", request.proposal_id.as_str()),
        ("actor_id", request.actor_id.as_str()),
        ("summary", request.summary.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(format!("全局最终复核缺少必填字段：{label}"));
        }
    }
    match request.decision.as_str() {
        "accepted" | "needs_changes" | "blocked" => {}
        other => return Err(format!("未知全局最终复核 decision：{other}")),
    }
    if request.evidence_refs.is_empty() {
        return Err("全局最终复核缺少 evidence_refs，已拒绝记录".to_string());
    }
    if matches!(request.decision.as_str(), "needs_changes" | "blocked")
        && request.open_issues.is_empty()
    {
        return Err("needs_changes / blocked 必须写明 open_issues".to_string());
    }
    Ok(())
}

fn validate_user_result_decision_input(request: &UserResultDecisionInput) -> Result<(), String> {
    if request.actor_role != "user" {
        return Err("只有用户可以记录用户结果决定".to_string());
    }
    for (label, value) in [
        ("project_root", request.project_root.as_str()),
        ("project_id", request.project_id.as_str()),
        ("workflow_id", request.workflow_id.as_str()),
        ("actor_id", request.actor_id.as_str()),
        ("summary", request.summary.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(format!("用户结果决定缺少必填字段：{label}"));
        }
    }
    match request.decision.as_str() {
        "accept_result" | "request_changes" | "reject_result" => {}
        other => return Err(format!("未知用户结果决定：{other}")),
    }
    if matches!(
        request.decision.as_str(),
        "request_changes" | "reject_result"
    ) && request.requested_changes.is_empty()
    {
        return Err("用户要求修改 / 拒绝结果必须写明 requested_changes".to_string());
    }
    Ok(())
}

fn validate_generate_stage_c_acceptance_summary_input(
    request: &GenerateStageCAcceptanceSummaryInput,
) -> Result<(), String> {
    for (label, value) in [
        ("project_root", request.project_root.as_str()),
        ("project_id", request.project_id.as_str()),
        ("workflow_id", request.workflow_id.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(format!("阶段 C 验收摘要缺少必填字段：{label}"));
        }
    }
    Ok(())
}

fn validate_c6_prerequisites_for_final_review(
    path: &Path,
    value: &Value,
    request: &GlobalFinalResultReviewInput,
) -> Result<(), String> {
    if !workflow_exists(value, &request.workflow_id) {
        return Err("当前项目还没有目标 workflow；无法记录全局最终复核".to_string());
    }
    let timestamp_ms = unix_timestamp_ms();
    let proposal_store = project_consultation_proposal_store::load_store(path, timestamp_ms)?;
    let proposal = proposal_store
        .proposals
        .iter()
        .find(|proposal| proposal.proposal_id == request.proposal_id)
        .ok_or_else(|| format!("找不到 C2 项目咨询方案：{}", request.proposal_id))?;
    if proposal.project_id != request.project_id || proposal.workflow_id != request.workflow_id {
        return Err("C2 proposal 不属于目标 project/workflow".to_string());
    }
    if proposal.status != ProjectConsultationProposalStatus::UserConfirmed {
        return Err("C2 proposal 尚未由用户确认，不能记录全局最终复核".to_string());
    }
    if proposal.plan_authorization_id.as_deref() != Some(request.authorization_id.as_str()) {
        return Err("C2 proposal 与 C1 authorization 回链不匹配".to_string());
    }

    let authorization_store = plan_authorization_store::load_store(path, timestamp_ms)?;
    let authorization = authorization_store
        .authorizations
        .iter()
        .find(|authorization| authorization.authorization_id == request.authorization_id)
        .ok_or_else(|| format!("找不到 C1 方案授权对象：{}", request.authorization_id))?;
    if authorization.project_id != request.project_id
        || authorization.workflow_id != request.workflow_id
    {
        return Err("C1 authorization 不属于目标 project/workflow".to_string());
    }
    if authorization.source_proposal_id.as_deref() != Some(request.proposal_id.as_str()) {
        return Err("C1 authorization source_proposal_id 与 C2 proposal 不匹配".to_string());
    }
    if authorization.user_confirmation.is_none() {
        return Err("C1 authorization 缺少用户确认，不能记录全局最终复核".to_string());
    }
    if authorization.status != PlanAuthorizationStatus::Active {
        return Err("C3 全局边界复核未让 authorization active，不能记录最终复核".to_string());
    }
    if !authorization
        .global_boundary_review
        .as_ref()
        .is_some_and(|review| review.status == "approved")
    {
        return Err("缺少 C3 approved global boundary review".to_string());
    }
    if !has_c4_prepared_dispatch(value, &request.workflow_id, &request.authorization_id) {
        return Err("缺少 C4 prepared dispatch 记录，不能记录全局最终复核".to_string());
    }
    if !has_c4_task_package_artifact(value, &request.workflow_id) {
        return Err("缺少 C4 task package artifact，不能记录全局最终复核".to_string());
    }
    if !has_worker_report_for_workflow(value, &request.workflow_id) {
        return Err("缺少 C5 worker 结构化汇报，不能记录全局最终复核".to_string());
    }
    if !has_process_fact_decision_for_workflow(value, &request.workflow_id) {
        return Err("缺少 C5 项目主管过程事实决定，不能记录全局最终复核".to_string());
    }
    Ok(())
}

fn has_c4_prepared_dispatch(value: &Value, workflow_id: &str, authorization_id: &str) -> bool {
    value
        .get("workflow_node_dispatches")
        .and_then(Value::as_array)
        .is_some_and(|dispatches| {
            dispatches.iter().any(|dispatch| {
                optional_string_from(dispatch, "workflow_id").as_deref() == Some(workflow_id)
                    && optional_string_from(dispatch, "plan_authorization_id").as_deref()
                        == Some(authorization_id)
                    && matches!(
                        optional_string_from(dispatch, "state").as_deref(),
                        Some("prepared" | "completed" | "failed" | "timed_out" | "cancelled")
                    )
            })
        })
}

fn has_c4_task_package_artifact(value: &Value, workflow_id: &str) -> bool {
    value
        .get("artifacts")
        .and_then(Value::as_array)
        .is_some_and(|artifacts| {
            artifacts.iter().any(|artifact| {
                artifact_belongs_to_workflow(value, artifact, workflow_id)
                    && optional_string_from(artifact, "artifact_type").as_deref()
                        == Some("task_package")
                    && optional_string_from(artifact, "source_kind").as_deref()
                        == Some("project_director_task_plan")
            })
        })
}

fn artifact_belongs_to_workflow(value: &Value, artifact: &Value, workflow_id: &str) -> bool {
    if optional_string_from(artifact, "workflow_id").as_deref() == Some(workflow_id) {
        return true;
    }
    optional_string_from(artifact, "source_ref")
        .as_deref()
        .is_some_and(|source_ref| find_work_item(value, workflow_id, source_ref).is_some())
}

fn has_worker_report_for_workflow(value: &Value, workflow_id: &str) -> bool {
    value
        .get("audit_events")
        .and_then(Value::as_array)
        .is_some_and(|events| {
            events.iter().any(|event| {
                optional_string_from(event, "workflow_id").as_deref() == Some(workflow_id)
                    && matches!(
                        optional_string_from(event, "event_type").as_deref(),
                        Some("worker_structured_report_recorded" | "subagent_report")
                    )
            })
        })
}

fn process_fact_reviews_for_workflow<'a>(value: &'a Value, workflow_id: &str) -> Vec<&'a Value> {
    value
        .get("reviews")
        .and_then(Value::as_array)
        .map(|reviews| {
            reviews
                .iter()
                .filter(|review| {
                    optional_string_from(review, "workflow_id").as_deref() == Some(workflow_id)
                        && optional_string_from(review, "reviewer_role").as_deref()
                            == Some("project_director")
                        && matches!(
                            optional_string_from(review, "decision").as_deref(),
                            Some("confirm_process_fact" | "request_rework" | "block_and_escalate")
                        )
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn has_process_fact_decision_for_workflow(value: &Value, workflow_id: &str) -> bool {
    !process_fact_reviews_for_workflow(value, workflow_id).is_empty()
}

fn confirmed_process_fact_ids_for_workflow(value: &Value, workflow_id: &str) -> Vec<String> {
    process_fact_reviews_for_workflow(value, workflow_id)
        .into_iter()
        .filter(|review| {
            optional_string_from(review, "decision").as_deref() == Some("confirm_process_fact")
        })
        .flat_map(|review| string_array(review, "accepted_fact_ids"))
        .collect()
}

fn unresolved_process_fact_decisions(value: &Value, workflow_id: &str) -> Vec<String> {
    process_fact_reviews_for_workflow(value, workflow_id)
        .into_iter()
        .filter(|review| {
            matches!(
                optional_string_from(review, "decision").as_deref(),
                Some("request_rework" | "block_and_escalate")
            )
        })
        .filter_map(|review| {
            let decision = optional_string_from(review, "decision")?;
            let summary = optional_string_from(review, "summary").unwrap_or_default();
            Some(format!("{decision}: {summary}"))
        })
        .collect()
}

fn latest_global_final_review<'a>(value: &'a Value, workflow_id: &str) -> Option<&'a Value> {
    value
        .get("reviews")
        .and_then(Value::as_array)
        .and_then(|reviews| {
            reviews.iter().rev().find(|review| {
                optional_string_from(review, "workflow_id").as_deref() == Some(workflow_id)
                    && optional_string_from(review, "reviewer_role").as_deref()
                        == Some("global_director")
                    && optional_string_from(review, "review_target").as_deref()
                        == Some("global_final_result")
            })
        })
}

fn latest_user_result_decision<'a>(value: &'a Value, workflow_id: &str) -> Option<&'a Value> {
    value
        .get("reviews")
        .and_then(Value::as_array)
        .and_then(|reviews| {
            reviews.iter().rev().find(|review| {
                optional_string_from(review, "workflow_id").as_deref() == Some(workflow_id)
                    && optional_string_from(review, "reviewer_role").as_deref() == Some("user")
                    && optional_string_from(review, "review_target").as_deref()
                        == Some("user_result_decision")
            })
        })
}

fn latest_stage_c_acceptance_artifact<'a>(
    artifacts: &'a [Value],
    workflow_id: &str,
) -> Option<&'a Value> {
    artifacts.iter().rev().find(|artifact| {
        optional_string_from(artifact, "workflow_id").as_deref() == Some(workflow_id)
            && optional_string_from(artifact, "artifact_type").as_deref()
                == Some("stage_c_acceptance_summary")
    })
}

fn build_stage_c_acceptance_summary(
    path: &Path,
    value: &Value,
    project_id_value: &str,
    workflow_id: &str,
) -> Result<StageCAcceptanceSummary, String> {
    let timestamp_ms = unix_timestamp_ms();
    let proposal_store = project_consultation_proposal_store::load_store(path, timestamp_ms)?;
    let authorization_store = plan_authorization_store::load_store(path, timestamp_ms)?;
    let proposal = proposal_store
        .proposals
        .iter()
        .filter(|proposal| {
            proposal.project_id == project_id_value && proposal.workflow_id == workflow_id
        })
        .last();
    let authorization = proposal
        .and_then(|proposal| proposal.plan_authorization_id.as_deref())
        .and_then(|authorization_id| {
            authorization_store
                .authorizations
                .iter()
                .find(|authorization| authorization.authorization_id == authorization_id)
        })
        .or_else(|| {
            authorization_store
                .authorizations
                .iter()
                .filter(|authorization| {
                    authorization.project_id == project_id_value
                        && authorization.workflow_id == workflow_id
                })
                .rev()
                .find(|authorization| authorization.status == PlanAuthorizationStatus::Active)
        });
    let authorization_id = authorization
        .map(|authorization| authorization.authorization_id.as_str())
        .unwrap_or("");
    let final_review = latest_global_final_review(value, workflow_id);
    let user_decision = latest_user_result_decision(value, workflow_id);
    let final_review_status = final_review
        .and_then(|review| optional_string_from(review, "decision"))
        .unwrap_or_else(|| "pending".to_string());
    let user_decision_status = user_decision
        .and_then(|review| optional_string_from(review, "decision"))
        .unwrap_or_else(|| "pending".to_string());
    let process_decisions = process_fact_reviews_for_workflow(value, workflow_id);
    let mut gates = Vec::new();
    gates.push(stage_c_gate(
        "c1-plan-authorization",
        "C1 方案授权",
        if authorization.is_some_and(|authorization| authorization.user_confirmation.is_some()) {
            "passed"
        } else {
            "missing_evidence"
        },
        if let Some(authorization) = authorization {
            format!(
                "authorization {} / status {:?}",
                authorization.authorization_id, authorization.status
            )
        } else {
            "缺少方案授权对象或用户确认。".to_string()
        },
        authorization
            .map(|authorization| vec![authorization.authorization_id.clone()])
            .unwrap_or_default(),
    ));
    gates.push(stage_c_gate(
        "c2-user-confirmed-proposal",
        "C2 用户确认方案",
        if proposal.is_some_and(|proposal| {
            proposal.status == ProjectConsultationProposalStatus::UserConfirmed
        }) {
            "passed"
        } else {
            "missing_evidence"
        },
        if let Some(proposal) = proposal {
            format!(
                "proposal {} / status {:?}",
                proposal.proposal_id, proposal.status
            )
        } else {
            "缺少用户确认过的项目咨询方案。".to_string()
        },
        proposal
            .map(|proposal| vec![proposal.proposal_id.clone()])
            .unwrap_or_default(),
    ));
    gates.push(stage_c_gate(
        "c3-global-boundary-review",
        "C3 全局边界复核",
        if authorization.is_some_and(|authorization| {
            authorization.status == PlanAuthorizationStatus::Active
                && authorization
                    .global_boundary_review
                    .as_ref()
                    .is_some_and(|review| review.status == "approved")
        }) {
            "passed"
        } else {
            "missing_evidence"
        },
        "authorization 必须 active 且 global boundary review 为 approved。".to_string(),
        authorization
            .map(|authorization| vec![authorization.authorization_id.clone()])
            .unwrap_or_default(),
    ));
    gates.push(stage_c_gate(
        "c4-prepared-dispatch",
        "C4 项目主管拆任务 / prepared dispatch",
        if has_c4_prepared_dispatch(value, workflow_id, authorization_id)
            && has_c4_task_package_artifact(value, workflow_id)
        {
            "passed"
        } else {
            "missing_evidence"
        },
        "需要 task package artifact 和 prepared dispatch 记录。".to_string(),
        evidence_refs_for_c4(value, workflow_id),
    ));
    let c5_status = if process_decisions.iter().any(|review| {
        optional_string_from(review, "decision").as_deref() == Some("block_and_escalate")
    }) {
        "blocked"
    } else if process_decisions
        .iter()
        .any(|review| optional_string_from(review, "decision").as_deref() == Some("request_rework"))
    {
        "needs_changes"
    } else if has_worker_report_for_workflow(value, workflow_id)
        && process_decisions.iter().any(|review| {
            optional_string_from(review, "decision").as_deref() == Some("confirm_process_fact")
        })
    {
        "passed"
    } else {
        "missing_evidence"
    };
    gates.push(stage_c_gate(
        "c5-worker-report-process-fact",
        "C5 worker 汇报 / 过程事实确认",
        c5_status,
        "worker report 必须由项目主管确认过程事实；observation 仍不是正式记忆。".to_string(),
        evidence_refs_for_c5(value, workflow_id),
    ));
    gates.push(stage_c_gate(
        "c6-global-final-review",
        "C6 全局最终复核",
        match final_review_status.as_str() {
            "accepted" => "passed",
            "needs_changes" => "needs_changes",
            "blocked" => "blocked",
            _ => "missing_evidence",
        },
        "全局主管最终复核不能代表用户已接受。".to_string(),
        final_review
            .and_then(|review| optional_string_from(review, "review_id"))
            .into_iter()
            .collect(),
    ));
    gates.push(stage_c_gate(
        "c6-user-result-decision",
        "C6 用户结果决定",
        match user_decision_status.as_str() {
            "accept_result" => "passed",
            "request_changes" => "needs_changes",
            "reject_result" => "blocked",
            _ => "missing_evidence",
        },
        "用户决定只适用于本次结果，不代表未来任务默认接受。".to_string(),
        user_decision
            .and_then(|review| optional_string_from(review, "review_id"))
            .into_iter()
            .collect(),
    ));
    gates.push(stage_c_gate(
        "stage-c-deferred-real-worker",
        "后置：真实 worker / Codex 执行",
        "deferred",
        "C6 默认不执行真实 worker、codex exec 或 codex exec resume。".to_string(),
        vec![],
    ));
    gates.push(stage_c_gate(
        "stage-c-deferred-ops-tauri-retry",
        "后置：真实 Tauri 全面验收 / 自动重试 / 运维日志",
        "deferred",
        "真实窗口全面验收、完整自动重试和运行日志体系后续单独拆任务。".to_string(),
        vec![],
    ));

    let deferred_items = dedupe_strings({
        let mut items = vec![
            "真实 worker / Codex 执行仍需单独授权任务包。".to_string(),
            "真实 Tauri 全面截图验收仍是后置项。".to_string(),
            "完整自动重试、运行日志和运维诊断仍是后置项。".to_string(),
            "M7-M13 完整记忆系统仍未完成。".to_string(),
        ];
        if let Some(review) = final_review {
            items.extend(string_array(review, "deferred_items"));
        }
        items
    });
    let open_blockers = gates
        .iter()
        .filter(|gate| !matches!(gate.status.as_str(), "passed" | "deferred"))
        .map(|gate| format!("{}：{}", gate.label, gate.reason))
        .collect::<Vec<_>>();
    let accepted_as_stage_c_complete = open_blockers.is_empty()
        && final_review_status == "accepted"
        && user_decision_status == "accept_result";
    Ok(StageCAcceptanceSummary {
        project_id: project_id_value.to_string(),
        workflow_id: workflow_id.to_string(),
        gates,
        final_review_status,
        user_decision_status,
        accepted_as_stage_c_complete,
        deferred_items,
        open_blockers,
        warnings: vec![
            "stage_c_acceptance_does_not_complete_middle_version".to_string(),
            "process_fact_observation_is_not_formal_memory".to_string(),
        ],
    })
}

fn stage_c_gate(
    gate_id: &str,
    label: &str,
    status: &str,
    reason: String,
    evidence_refs: Vec<String>,
) -> StageCAcceptanceGate {
    StageCAcceptanceGate {
        gate_id: gate_id.to_string(),
        label: label.to_string(),
        status: status.to_string(),
        reason,
        evidence_refs,
    }
}

fn evidence_refs_for_c4(value: &Value, workflow_id: &str) -> Vec<String> {
    let mut refs = Vec::new();
    if let Some(dispatches) = value
        .get("workflow_node_dispatches")
        .and_then(Value::as_array)
    {
        refs.extend(dispatches.iter().filter_map(|dispatch| {
            (optional_string_from(dispatch, "workflow_id").as_deref() == Some(workflow_id))
                .then(|| optional_string_from(dispatch, "dispatch_id"))
                .flatten()
        }));
    }
    if let Some(artifacts) = value.get("artifacts").and_then(Value::as_array) {
        refs.extend(artifacts.iter().filter_map(|artifact| {
            (artifact_belongs_to_workflow(value, artifact, workflow_id)
                && optional_string_from(artifact, "artifact_type").as_deref()
                    == Some("task_package"))
            .then(|| optional_string_from(artifact, "artifact_id"))
            .flatten()
        }));
    }
    dedupe_strings(refs)
}

fn evidence_refs_for_c5(value: &Value, workflow_id: &str) -> Vec<String> {
    let mut refs = Vec::new();
    if let Some(events) = value.get("audit_events").and_then(Value::as_array) {
        refs.extend(events.iter().filter_map(|event| {
            (optional_string_from(event, "workflow_id").as_deref() == Some(workflow_id)
                && optional_string_from(event, "event_type").as_deref()
                    == Some("worker_structured_report_recorded"))
            .then(|| optional_string_from(event, "event_id"))
            .flatten()
        }));
    }
    refs.extend(
        process_fact_reviews_for_workflow(value, workflow_id)
            .into_iter()
            .filter_map(|review| optional_string_from(review, "review_id")),
    );
    dedupe_strings(refs)
}

fn validate_worker_structured_report_input(
    request: &WorkerStructuredReportInput,
) -> Result<(), String> {
    for (label, value) in [
        ("project_root", request.project_root.as_str()),
        ("project_id", request.project_id.as_str()),
        ("workflow_id", request.workflow_id.as_str()),
        ("workflow_node_id", request.workflow_node_id.as_str()),
        ("work_item_id", request.work_item_id.as_str()),
        ("actor_role", request.actor_role.as_str()),
        ("executed_what", request.executed_what.as_str()),
        ("changed_what", request.changed_what.as_str()),
        ("summary", request.summary.as_str()),
        ("acceptance_status", request.acceptance_status.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(format!("worker report 缺少必填字段：{label}"));
        }
    }
    if request.evidence_refs.is_empty() {
        return Err("worker report 缺少 evidence_refs，已拒绝记录".to_string());
    }
    if request.source_refs.is_empty() {
        return Err("worker report 缺少 source_refs，已拒绝记录".to_string());
    }
    if request.summary.len() > 1200
        || request.executed_what.len() > 1200
        || request.changed_what.len() > 1200
    {
        return Err("worker report 字段过长；不能存入长日志或 raw transcript".to_string());
    }
    match request.acceptance_status.as_str() {
        "reported_completed" | "reported_not_completed" | "blocked" | "needs_rework" => {}
        other => return Err(format!("未知 worker report acceptance_status：{other}")),
    }
    for source in &request.source_refs {
        validate_c5_source_ref(source)?;
    }
    Ok(())
}

fn validate_c5_source_ref(source: &ObservationSourceRef) -> Result<(), String> {
    if source.source_id.trim().is_empty() || source.summary.trim().is_empty() {
        return Err("source_ref 缺少 source_id 或 summary".to_string());
    }
    match source.source_kind.as_str() {
        "workflow_event" | "worker_report" | "director_review" | "task_package" | "evidence"
        | "handoff" | "user_confirmation" => Ok(()),
        "ordinary_chat" | "chat" => {
            Err("普通聊天来源不能作为 C5 worker report / process fact 来源".to_string())
        }
        other => Err(format!("不支持的 C5 source_kind：{other}")),
    }
}

fn validate_process_fact_decision_input(
    request: &ProjectDirectorProcessFactDecisionInput,
) -> Result<(), String> {
    if request.actor_role != "project_director" {
        return Err("只有项目主管可以确认 worker 汇报中的过程事实".to_string());
    }
    if request.summary.trim().is_empty() {
        return Err("过程事实决定 summary 不能为空".to_string());
    }
    match request.decision.as_str() {
        "confirm_process_fact" => {
            if request.accepted_facts.is_empty() {
                return Err("确认过程事实至少需要一个 accepted_fact".to_string());
            }
        }
        "request_rework" | "block_and_escalate" => {}
        other => return Err(format!("未知过程事实决定：{other}")),
    }
    Ok(())
}

fn validate_process_fact_candidate(
    fact: &ProcessFactCandidate,
    request: &ProjectDirectorProcessFactDecisionInput,
) -> Result<(), String> {
    if fact.process_fact_id.trim().is_empty() || fact.summary.trim().is_empty() {
        return Err("process fact 缺少 id 或 summary".to_string());
    }
    if fact.source_report_id != request.report_id {
        return Err("process fact source_report_id 与当前 report 不匹配".to_string());
    }
    if fact.evidence_refs.is_empty() || fact.source_refs.is_empty() {
        return Err("process fact 缺少 evidence_refs 或 source_refs".to_string());
    }
    if fact.risk_level != "low" {
        return Err(
            "high / medium risk process fact 需要用户或更高层确认，项目主管不能单独确认"
                .to_string(),
        );
    }
    if fact.sensitive_level == "secret" || fact.sensitive_level == "sensitive" {
        return Err(
            "secret / sensitive process fact 需要用户确认，项目主管不能单独确认".to_string(),
        );
    }
    if fact.scope.project_id.as_deref() != Some(request.project_id.as_str()) {
        return Err("cross-project process fact 需要用户确认，项目主管不能单独确认".to_string());
    }
    if fact.scope.workflow_id.as_deref() != Some(request.workflow_id.as_str()) {
        return Err("process fact workflow scope 与当前 workflow 不匹配".to_string());
    }
    if fact.proposed_observation_type != "process_fact"
        && fact.proposed_observation_type != "worker_report"
    {
        return Err(
            "process fact observation_type 必须是 process_fact 或 worker_report".to_string(),
        );
    }
    for source in &fact.source_refs {
        validate_c5_source_ref(source)?;
    }
    Ok(())
}

fn find_worker_report_event<'a>(value: &'a Value, report_id: &str) -> Option<&'a Value> {
    value
        .get("audit_events")
        .and_then(Value::as_array)
        .and_then(|events| {
            events.iter().find(|event| {
                optional_string_from(event, "event_id").as_deref() == Some(report_id)
                    && matches!(
                        optional_string_from(event, "event_type").as_deref(),
                        Some("worker_structured_report_recorded") | Some("subagent_report")
                    )
            })
        })
}

fn process_fact_decision_exists(
    value: &Value,
    report_id: &str,
    facts: &[ProcessFactCandidate],
) -> bool {
    value
        .get("reviews")
        .and_then(Value::as_array)
        .is_some_and(|reviews| {
            reviews.iter().any(|review| {
                optional_string_from(review, "report_id").as_deref() == Some(report_id)
                    && optional_string_from(review, "reviewer_role").as_deref()
                        == Some("project_director")
                    && facts.iter().any(|fact| {
                        string_array(review, "accepted_fact_ids")
                            .iter()
                            .any(|id| id == &fact.process_fact_id)
                    })
            })
        })
}

fn read_c4_workflow_value(
    path: &Path,
    project_root_value: &str,
    workflow_id: &str,
) -> Result<Value, String> {
    if !path.exists() {
        return Err("工作流状态文件不存在；无法做项目主管拆任务。".to_string());
    }
    let value = read_workflow_state_value(path)?;
    let validation_warnings = validate_workflow_state(&value);
    if !validation_warnings.is_empty() {
        return Err(format!(
            "当前状态文件未通过 schema 校验：{}",
            validation_warnings.join(", ")
        ));
    }
    if !workflow_exists(&value, workflow_id) {
        return Err("当前项目还没有匹配 workflow；无法做项目主管拆任务。".to_string());
    }
    let expected_project_id = project_id(project_root_value);
    if !project_exists(&value, &expected_project_id) {
        return Err("workflow state 缺少当前项目记录；无法做项目主管拆任务。".to_string());
    }
    Ok(value)
}

fn project_director_authorization_context(
    path: &Path,
    index: &Value,
    project_root_value: &str,
    project_id_value: &str,
    workflow_id: &str,
    proposal_id: &str,
    authorization_id: &str,
    expected_authorization_revision: Option<i64>,
) -> Result<ProjectDirectorAuthorizationContext, String> {
    let project = find_index_project(index, project_root_value)
        .ok_or_else(|| "项目不在当前索引内，已拒绝项目主管拆任务。".to_string())?;
    let expected_project_id = project_id(project_root_value);
    if project_id_value != expected_project_id {
        return Err("C4 输入 project_id 与 project_root 推导结果不一致。".to_string());
    }
    // (b) 放开「只认默认工作流」：角色循环可跑在项目内**任意合法（已存在）工作流**上，不再死锚 default_workflow_id。
    // 这是简化假设、**非安全闸**——真执行仍走 execute 的 path-lock（圈测试项目）+ 沙箱 + 四护栏（本包不碰·0-diff）；
    // workflow_id 不影响 path-lock。补合法性闸：workflow_id 必须是该项目内**已存在**的工作流，否则拒（防注入不存在/
    // 跨项目工作流；project_id 仍按 root 推导一致；跨项目还会被后面「方案 project_id/workflow_id 必须匹配」二次拦）。
    let workflow_state_value = read_workflow_state_value(path)?;
    if !workflow_exists(&workflow_state_value, workflow_id) {
        return Err(
            "C4 输入 workflow_id 不是本项目内合法工作流（不存在或未提交）；请先在项目里建/提交该工作流。"
                .to_string(),
        );
    }
    let timestamp_ms = unix_timestamp_ms();
    let proposal_store = project_consultation_proposal_store::load_store(path, timestamp_ms)?;
    let proposal = proposal_store
        .proposals
        .iter()
        .find(|proposal| proposal.proposal_id == proposal_id)
        .cloned()
        .ok_or_else(|| format!("找不到项目咨询方案：{proposal_id}"))?;
    if proposal.status != ProjectConsultationProposalStatus::UserConfirmed {
        return Err("项目咨询方案尚未由用户确认，不能做项目主管拆任务。".to_string());
    }
    if proposal.project_id != project_id_value || proposal.workflow_id != workflow_id {
        return Err("项目咨询方案与 C4 输入 project_id / workflow_id 不一致。".to_string());
    }
    if proposal.plan_authorization_id.as_deref() != Some(authorization_id) {
        return Err("项目咨询方案缺少匹配的 C1 授权回链，不能做项目主管拆任务。".to_string());
    }

    let authorization_store = plan_authorization_store::load_store(path, timestamp_ms)?;
    if let Some(expected) = expected_authorization_revision {
        if authorization_store.revision != expected {
            return Err(format!(
                "方案授权 revision 不匹配：expected {expected}, actual {}",
                authorization_store.revision
            ));
        }
    }
    let authorization = authorization_store
        .authorizations
        .iter()
        .find(|authorization| authorization.authorization_id == authorization_id)
        .cloned()
        .ok_or_else(|| format!("找不到方案授权对象：{authorization_id}"))?;
    if authorization.project_id != project_id_value || authorization.workflow_id != workflow_id {
        return Err("方案授权对象与 C4 输入 project_id / workflow_id 不一致。".to_string());
    }
    if authorization.source_proposal_id.as_deref() != Some(proposal_id) {
        return Err("方案授权 source_proposal_id 与项目咨询方案不匹配。".to_string());
    }
    if authorization.status != PlanAuthorizationStatus::Active
        || authorization.user_confirmation.is_none()
        || !authorization
            .global_boundary_review
            .as_ref()
            .is_some_and(|review| review.status == "approved")
    {
        return Err("方案授权尚未 active 或缺少 C3 approved 全局边界复核。".to_string());
    }

    Ok(ProjectDirectorAuthorizationContext {
        project,
        proposal,
        authorization,
        authorization_store,
    })
}

fn deterministic_project_director_planned_tasks(
    context: &ProjectDirectorAuthorizationContext,
) -> Vec<ProjectDirectorPlannedTask> {
    let scope = &context.authorization.scope;
    let target_role = scope
        .allowed_role_ids
        .iter()
        .find(|role| {
            !matches!(
                normalize_c4_symbol(role).as_str(),
                "project_director" | "director" | "global_director"
            )
        })
        .or_else(|| scope.allowed_role_ids.first())
        .cloned()
        .unwrap_or_else(|| "codex-dev".to_string());
    let task_package_kind = scope
        .allowed_task_package_kinds
        .first()
        .cloned()
        .unwrap_or_else(|| "task_package".to_string());
    let planned_task_id = format!(
        "project-director-planned-task:{}",
        stable_id(&format!(
            "{}:{}:{}:{}",
            context.authorization.authorization_id,
            context.proposal.proposal_id,
            target_role,
            context.proposal.goal_summary
        ))
    );
    let objective = format!(
        "{}\n\n{}",
        context.proposal.goal_summary,
        context
            .proposal
            .proposed_steps
            .iter()
            .enumerate()
            .map(|(index, step)| format!("{}. {}", index + 1, step))
            .collect::<Vec<_>>()
            .join("\n")
    )
    .trim()
    .to_string();
    let stop_conditions = scope
        .stop_conditions
        .iter()
        .map(|condition| condition.summary.clone())
        .collect::<Vec<_>>();
    vec![ProjectDirectorPlannedTask {
        planned_task_id,
        title: format!("执行已授权方案：{}", context.proposal.title),
        task_goal: objective,
        scope: ProjectDirectorTaskScope {
            project_id: scope.project_id.clone(),
            workflow_id: scope.workflow_id.clone(),
            target_role,
            task_package_kind,
            allowed_read_scope: scope.allowed_read_roots.clone(),
            allowed_write_scope: scope.allowed_write_roots.clone(),
            available_skills: vec![],
            available_knowledge_refs: vec![],
            callable_tool_capabilities: scope.allowed_tools.clone(),
            required_checks: scope.allowed_checks.clone(),
            stop_conditions,
            timeout_policy: None,
            failure_policy: None,
            forbidden_actions: vec![],
            model_id: None,
        },
        depends_on: vec![],
        acceptance_criteria: if context.proposal.acceptance_criteria.is_empty() {
            vec!["按任务包完成工作，并以结构化汇报返回证据、风险和后续建议。".to_string()]
        } else {
            context.proposal.acceptance_criteria.clone()
        },
        report_format: standard_project_director_report_format(),
        status: "draft".to_string(),
        guard_result: None,
        work_item_id: None,
        workflow_node_id: None,
        task_package_id: None,
        memory_packet_snapshot_id: None,
        prepared_dispatch_id: None,
        blocked_reasons: vec![],
    }]
}

fn annotate_project_director_planned_tasks(
    index: &Value,
    value: &Value,
    context: &ProjectDirectorAuthorizationContext,
    tasks: Vec<ProjectDirectorPlannedTask>,
    timestamp_ms: i64,
) -> Vec<ProjectDirectorPlannedTask> {
    let authorization_is_read_only = context.authorization.scope.allowed_write_roots.is_empty();
    tasks
        .into_iter()
        .map(|mut task| {
            let work_item_id =
                c4_work_item_id(&context.authorization.workflow_id, &task.planned_task_id);
            let artifact_id = c4_task_package_artifact_id(
                &context.authorization.workflow_id,
                &task.planned_task_id,
            );
            let node_id = c4_node_id(&context.authorization.workflow_id, &task.scope.target_role);
            task.work_item_id = Some(work_item_id.clone());
            task.task_package_id = Some(artifact_id.clone());
            task.workflow_node_id = Some(node_id.clone());
            task.blocked_reasons =
                c4_static_task_blocking_reasons(&task, authorization_is_read_only);

            let binding = active_binding_for_planned_task(index, value, &node_id, &work_item_id);
            let target_agent_id = binding
                .as_ref()
                .map(|binding| binding.native_thread_id.clone())
                .or_else(|| {
                    context
                        .authorization
                        .scope
                        .allowed_agent_ids
                        .first()
                        .cloned()
                });
            let guard_input = AutoDispatchGuardInput {
                project_id: context.authorization.project_id.clone(),
                workflow_id: context.authorization.workflow_id.clone(),
                work_item_id: work_item_id.clone(),
                task_package_id: Some(artifact_id),
                task_package_kind: Some(task.scope.task_package_kind.clone()),
                target_role_id: task.scope.target_role.clone(),
                target_agent_id,
                requested_read_roots: task.scope.allowed_read_scope.clone(),
                requested_write_roots: task.scope.allowed_write_scope.clone(),
                requested_tools: task.scope.callable_tool_capabilities.clone(),
                requested_checks: task.scope.required_checks.clone(),
                triggered_stop_conditions: vec![],
                dispatch_kind: "prepare_real".to_string(),
            };
            let guard_result = control_core::inspect_auto_dispatch_scope(
                &context.authorization_store,
                &guard_input,
                timestamp_ms,
            );
            if guard_result.status != "authorized" {
                task.blocked_reasons.extend(guard_result.reasons.clone());
            }
            task.guard_result = Some(guard_result);

            if let Some(artifact) = find_task_package_artifact_by_id(value, &work_item_id) {
                if let Some(snapshot) = task_memory_injection::snapshot_from_artifact(artifact) {
                    task.memory_packet_snapshot_id = Some(snapshot.snapshot_id);
                }
            }
            if let Some(dispatch) = existing_prepared_dispatch_for_planned_task(
                value,
                &task.planned_task_id,
                &work_item_id,
                &context.authorization.authorization_id,
            ) {
                task.prepared_dispatch_id = optional_string_from(dispatch, "dispatch_id");
                task.status = "prepared".to_string();
            } else if !task.blocked_reasons.is_empty() {
                task.status = "blocked".to_string();
            } else if binding.is_none() {
                task.status = "needs_binding".to_string();
                push_unique(&mut task.blocked_reasons, "等待绑定会话后才能准备派发。");
            } else if binding
                .as_ref()
                .is_some_and(|binding| !binding.rollout_exists)
            {
                task.status = "needs_binding".to_string();
                push_unique(
                    &mut task.blocked_reasons,
                    "绑定会话 rollout 不可用，等待重新绑定。",
                );
            } else {
                task.status = "authorized".to_string();
            }
            task.blocked_reasons = dedupe_strings(task.blocked_reasons);
            task
        })
        .collect()
}

fn project_director_task_plan_from_tasks(
    project_root_value: &str,
    project_id_value: &str,
    workflow_id: &str,
    proposal_id: &str,
    authorization_id: &str,
    actor_id: &str,
    tasks: Vec<ProjectDirectorPlannedTask>,
    value: &Value,
) -> ProjectDirectorTaskPlan {
    let planned_task_count = tasks.len();
    let prepared_dispatch_count = tasks
        .iter()
        .filter(|task| task.status == "prepared")
        .count();
    let needs_binding_count = tasks
        .iter()
        .filter(|task| task.status == "needs_binding")
        .count();
    let blocked_count = tasks.iter().filter(|task| task.status == "blocked").count();
    let authorized_task_count = tasks
        .iter()
        .filter(|task| {
            task.guard_result
                .as_ref()
                .is_some_and(|guard| guard.status == "authorized")
        })
        .count();
    let blocked_reasons = dedupe_strings(
        tasks
            .iter()
            .flat_map(|task| task.blocked_reasons.clone())
            .take(6)
            .collect(),
    );
    let memory_snapshot_summary = aggregate_project_director_memory_summary(value, &tasks);
    ProjectDirectorTaskPlan {
        project_root: project_root_value.to_string(),
        project_id: project_id_value.to_string(),
        workflow_id: workflow_id.to_string(),
        proposal_id: proposal_id.to_string(),
        authorization_id: authorization_id.to_string(),
        actor_id: actor_id.to_string(),
        planned_tasks: tasks,
        planned_task_count,
        authorized_task_count,
        prepared_dispatch_count,
        blocked_count,
        needs_binding_count,
        blocked_reasons,
        memory_snapshot_summary,
        display_text: format!(
            "项目主管拆任务：planned {planned_task_count} / authorized {authorized_task_count} / prepared {prepared_dispatch_count} / needs_binding {needs_binding_count} / blocked {blocked_count}；prepared 仍未执行 worker。"
        ),
        warnings: vec!["prepared_dispatch_is_not_worker_execution".to_string()],
    }
}

fn aggregate_project_director_memory_summary(
    value: &Value,
    tasks: &[ProjectDirectorPlannedTask],
) -> TaskPackageMemoryInjectionSummary {
    let mut snapshot_ids = Vec::new();
    let mut included_count = 0;
    let mut excluded_count = 0;
    let mut review_material_count = 0;
    let mut stale = false;
    let mut stale_reasons = Vec::new();
    let mut warnings = Vec::new();
    for task in tasks {
        let Some(work_item_id) = task.work_item_id.as_deref() else {
            continue;
        };
        let Some(artifact) = find_task_package_artifact_by_id(value, work_item_id) else {
            continue;
        };
        let Some(snapshot) = task_memory_injection::snapshot_from_artifact(artifact) else {
            continue;
        };
        snapshot_ids.push(snapshot.snapshot_id);
        included_count += snapshot.included_memories.len();
        excluded_count += snapshot.excluded_items.len();
        review_material_count += snapshot.review_materials.len();
        stale = stale || snapshot.stale;
        stale_reasons.extend(snapshot.stale_reasons);
        warnings.extend(snapshot.warnings);
    }
    if snapshot_ids.is_empty() {
        return task_memory_injection::missing_summary();
    }
    TaskPackageMemoryInjectionSummary {
        snapshot_id: Some(snapshot_ids.join(",")),
        included_count,
        excluded_count,
        review_material_count,
        stale,
        stale_reasons: dedupe_strings(stale_reasons),
        display_text: format!(
            "任务包记忆快照：{} 个 snapshot；使用了 {} 条正式记忆；排除了 {} 条候选 / 观察 / lint 阻断项；{} 条待审查材料。",
            snapshot_ids.len(),
            included_count,
            excluded_count,
            review_material_count
        ),
        warnings: dedupe_strings(warnings),
    }
}

fn prepared_dispatch_read_models_from_plan(
    plan: &ProjectDirectorTaskPlan,
    value: &Value,
) -> Vec<PreparedAutoDispatchReadModel> {
    plan.planned_tasks
        .iter()
        .filter_map(|task| {
            let guard = task.guard_result.clone().or_else(|| {
                Some(AutoDispatchGuardResult {
                    status: "blocked".to_string(),
                    authorization_id: Some(plan.authorization_id.clone()),
                    reasons: task.blocked_reasons.clone(),
                    required_user_confirmation: false,
                    required_global_review: false,
                    checked_at_ms: unix_timestamp_ms(),
                })
            })?;
            let dispatch = task
                .prepared_dispatch_id
                .as_deref()
                .and_then(|dispatch_id| find_workflow_node_dispatch(value, dispatch_id));
            Some(PreparedAutoDispatchReadModel {
                dispatch_id: task.prepared_dispatch_id.clone(),
                planned_task_id: task.planned_task_id.clone(),
                work_item_id: task.work_item_id.clone(),
                workflow_node_id: task.workflow_node_id.clone(),
                task_package_id: task.task_package_id.clone(),
                status: task.status.clone(),
                authorization_check: guard,
                memory_packet_snapshot_id: task.memory_packet_snapshot_id.clone(),
                memory_packet_fingerprint: dispatch.and_then(|dispatch| {
                    optional_string_from(dispatch, "memory_packet_fingerprint")
                }),
                binding_status: if task.status == "prepared" {
                    "active_binding_ready".to_string()
                } else if task.status == "needs_binding" {
                    "needs_binding".to_string()
                } else {
                    "not_ready".to_string()
                },
                prompt_preview: dispatch
                    .and_then(|dispatch| optional_string_from(dispatch, "prompt_preview")),
                blocked_reasons: task.blocked_reasons.clone(),
            })
        })
        .collect()
}

fn c4_static_task_blocking_reasons(
    task: &ProjectDirectorPlannedTask,
    authorization_is_read_only: bool,
) -> Vec<String> {
    let mut reasons = Vec::new();
    if task.title.trim().is_empty() {
        reasons.push("planned task 缺少标题。".to_string());
    }
    if task.task_goal.trim().is_empty() {
        reasons.push("planned task 缺少目标说明。".to_string());
    }
    if task.scope.allowed_read_scope.is_empty() {
        reasons.push("授权读取范围为空，不能生成可派发任务包。".to_string());
    }
    if task.scope.allowed_write_scope.is_empty() && !authorization_is_read_only {
        reasons.push("授权写入范围为空，不能生成可派发任务包。".to_string());
    }
    if task.acceptance_criteria.is_empty() {
        reasons.push("planned task 缺少验收标准。".to_string());
    }
    if task.report_format.is_empty() {
        reasons.push("planned task 缺少回传格式。".to_string());
    }
    if task
        .scope
        .allowed_read_scope
        .iter()
        .chain(task.scope.allowed_write_scope.iter())
        .any(|scope| scope.contains("/Users/yoyi/.codex"))
    {
        reasons.push("读写范围包含 /Users/yoyi/.codex，C4 已阻断。".to_string());
    }
    reasons
}

fn ensure_c4_backup(
    path: &Path,
    timestamp: &str,
    backup_path: &mut Option<PathBuf>,
) -> Result<(), String> {
    if backup_path.is_none() {
        *backup_path = Some(backup_workflow_state_file(path, timestamp)?);
    }
    Ok(())
}

fn ensure_project_director_worker_node(
    value: &mut Value,
    node_id: &str,
    task: &ProjectDirectorPlannedTask,
    timestamp: &str,
) -> Result<(), String> {
    if node_exists(value, &task.scope.workflow_id, node_id) {
        update_node_state_for_id(value, node_id, "ready_to_dispatch", timestamp)?;
        return Ok(());
    }
    array_mut(value, "nodes")?.push(json!({
      "node_id": node_id,
      "workflow_id": task.scope.workflow_id,
      "node_type": "actor",
      "title": assigned_role_label(&task.scope.target_role),
      "state": "ready_to_dispatch",
      "source_kind": "project_director_task_plan",
      "source_ref": task.planned_task_id,
      "agent_type": if task.scope.target_role.contains("codex") { "codex" } else { "agent" },
      "adapter_id": if task.scope.target_role.contains("codex") { Value::String("codex-local".to_string()) } else { Value::Null },
      "permission_level": "plan_authorized_prepared",
      "position": {
        "x": 560,
        "y": 120
      },
      "warnings": []
    }));
    let edge_id = format!(
        "{}:edge:project-director-task:{}",
        task.scope.workflow_id,
        stable_id(&task.planned_task_id)
    );
    if !edge_exists(value, &edge_id) {
        array_mut(value, "edges")?.push(json!({
          "edge_id": edge_id,
          "workflow_id": task.scope.workflow_id,
          "from_node_id": format!("{}:node:task", task.scope.workflow_id),
          "to_node_id": node_id,
          "edge_type": "assigned_to",
          "state": "ready_to_dispatch",
          "source_kind": "project_director_task_plan",
          "source_ref": task.planned_task_id,
          "permission_level": "plan_authorized_prepared",
          "created_at": timestamp,
          "updated_at": timestamp,
          "warnings": []
        }));
    }
    Ok(())
}

// 交办·刀2 2.3·任务级节点（纯加法·落画布让用户看工序图）。**与 role 节点分离**：
// - node_id = `{workflow_id}:node:task:{stable_id(planned_task_id)}`——**带后缀·绝不等于 bootstrap 保留节点
//   `{wf}:node:task`**（那个是任务包草稿节点·精确匹配·lib.rs update_task_node_state）。
// - source_ref = planned_task_id（配对锚·链回刷状态按它找；≠ planned_task.workflow_node_id 语义·坑②不动）。
// - 无 work_item / 无 binding 指向它 → read model 天然无 task_package_id → **天然不就绪·无执行口**（§3）。
// - position 按 order_index 错开（role 节点固定 x:560,y:120·任务级铺在下方网格·不叠·坑③）。
// - node_exists 幂等：已存在**不重复建、也不覆盖 state**（state 归链回刷·2.3 尾）——重跑 prepare 不抹进度。
// 返回该任务级节点 node_id（供依赖边配对）。
fn ensure_project_director_task_level_node(
    value: &mut Value,
    task: &ProjectDirectorPlannedTask,
    order_index: usize,
    timestamp: &str,
) -> Result<String, String> {
    let node_id = format!(
        "{}:node:task:{}",
        task.scope.workflow_id,
        stable_id(&task.planned_task_id)
    );
    if node_exists(value, &task.scope.workflow_id, &node_id) {
        return Ok(node_id);
    }
    let x = 200 + ((order_index % 4) as i64) * 240;
    let y = 360 + ((order_index / 4) as i64) * 160;
    array_mut(value, "nodes")?.push(json!({
      "node_id": node_id,
      "workflow_id": task.scope.workflow_id,
      "node_type": "project_director_task",
      "title": task.title,
      "state": "ready_to_dispatch",
      "source_kind": "project_director_task_plan",
      "source_ref": task.planned_task_id,
      "permission_level": "plan_authorized_prepared",
      "position": { "x": x, "y": y },
      "created_at": timestamp,
      "updated_at": timestamp,
      "warnings": []
    }));
    Ok(node_id)
}

// 2.3·任务级依赖边（depends_on·title→任务节点 id 已在调用处映射好）。edge_exists 幂等·edge_type=depends_on；
// read model derive_workflow_nodes 按 to_node_id 反推 depends_on（=from_node_id 集）→ 画布显示依赖箭头。
fn ensure_project_director_task_dep_edge(
    value: &mut Value,
    workflow_id: &str,
    from_node_id: &str,
    to_node_id: &str,
    from_planned_task_id: &str,
    to_planned_task_id: &str,
    timestamp: &str,
) -> Result<(), String> {
    let edge_id = format!(
        "{workflow_id}:edge:director-task-dep:{}:{}",
        stable_id(from_planned_task_id),
        stable_id(to_planned_task_id)
    );
    if edge_exists(value, &edge_id) {
        return Ok(());
    }
    array_mut(value, "edges")?.push(json!({
      "edge_id": edge_id,
      "workflow_id": workflow_id,
      "from_node_id": from_node_id,
      "to_node_id": to_node_id,
      "edge_type": "depends_on",
      "state": "ready_to_dispatch",
      "source_kind": "project_director_task_plan",
      "source_ref": to_planned_task_id,
      "permission_level": "plan_authorized_prepared",
      "created_at": timestamp,
      "updated_at": timestamp,
      "warnings": []
    }));
    Ok(())
}

fn ensure_project_director_work_item(
    value: &mut Value,
    work_item_id: &str,
    artifact_id: &str,
    task: &ProjectDirectorPlannedTask,
    timestamp: &str,
) -> Result<(), String> {
    if let Some(index) = find_work_item_index(value, &task.scope.workflow_id, work_item_id) {
        let current_state = optional_string_from(&value["work_items"][index], "state")
            .unwrap_or_else(|| "draft".to_string());
        if matches!(
            current_state.as_str(),
            "running" | "ready_for_review" | "accepted" | "failed" | "timed_out" | "cancelled"
        ) {
            return Err(format!(
                "已存在 work item 且状态为 {current_state}，C4 拒绝覆盖：{work_item_id}"
            ));
        }
        let work_items = array_mut(value, "work_items")?;
        let work_item = work_items
            .get_mut(index)
            .ok_or_else(|| "更新 C4 work item 时索引失效".to_string())?;
        work_item["title"] = Value::String(task.title.clone());
        work_item["state"] = Value::String("ready_to_dispatch".to_string());
        work_item["assigned_role_id"] = Value::String(task.scope.target_role.clone());
        work_item["current_node_id"] =
            Value::String(c4_node_id(&task.scope.workflow_id, &task.scope.target_role));
        work_item["project_director_planned_task_id"] = Value::String(task.planned_task_id.clone());
        work_item["updated_at"] = Value::String(timestamp.to_string());
        return Ok(());
    }
    array_mut(value, "work_items")?.push(json!({
      "work_item_id": work_item_id,
      "project_id": task.scope.project_id,
      "workflow_id": task.scope.workflow_id,
      "title": task.title,
      "state": "ready_to_dispatch",
      "source_kind": "project_director_task_plan",
      "source_ref": artifact_id,
      "assigned_role_id": task.scope.target_role,
      "current_node_id": c4_node_id(&task.scope.workflow_id, &task.scope.target_role),
      "agent_type": if task.scope.target_role.contains("codex") { "codex" } else { "agent" },
      "adapter_id": if task.scope.target_role.contains("codex") { "codex-local" } else { "adapter-pending" },
      "permission_level": "plan_authorized_prepared",
      "project_director_planned_task_id": task.planned_task_id,
      "created_at": timestamp,
      "updated_at": timestamp
    }));
    array_mut(value, "audit_events")?.push(json!({
      "event_id": format!("audit:project-director-task-plan-created:{}:{timestamp}", stable_id(work_item_id)),
      "event_type": "project_director_task_plan_created",
      "target_ref": work_item_id,
      "actor_ref": "project_director",
      "source_kind": "workspace_state",
      "permission_level": "plan_authorized_prepared",
      "before_state": "missing_project_director_task",
      "after_state": "ready_to_dispatch",
      "created_at": timestamp,
      "reason": "项目主管在 C3 active 授权范围内生成 worker 子任务；未启动 worker。"
    }));
    Ok(())
}

fn ensure_project_director_task_package_artifact(
    value: &mut Value,
    project: &ProjectRecord,
    work_item_id: &str,
    artifact_id: &str,
    task: &ProjectDirectorPlannedTask,
    memory_snapshot: &TaskPackageMemoryPacketSnapshot,
    timestamp: &str,
) -> Result<(), String> {
    let artifact_index = value
        .get("artifacts")
        .and_then(Value::as_array)
        .and_then(|artifacts| {
            artifacts.iter().position(|artifact| {
                optional_string_from(artifact, "artifact_id").as_deref() == Some(artifact_id)
            })
        });
    // fix·worker 回程契约：goals 追加主管拆的 report_format 各项（原有数据一直没人用·现在接上）
    // + 确定性契约段（worker 最后必须交且仅交一个 json 块）。**确定性拼接·不经 LM**；task_goal 仍在首位。
    let goals_with_contract =
        worker_report::build_goals_with_contract(&task.task_goal, &task.report_format);
    let forbidden_actions = if task.scope.forbidden_actions.is_empty() {
        vec![
            "不读写 `/Users/yoyi/.codex`。".to_string(),
            "不越过任务包授权范围。".to_string(),
            "不把 worker 汇报直接写成正式事实或正式记忆。".to_string(),
            "触发停止条件时先回报项目主管。".to_string(),
        ]
    } else {
        task.scope.forbidden_actions.clone()
    };
    let model_id = task
        .scope
        .model_id
        .clone()
        .unwrap_or_else(|| "codex-local-prepared".to_string());
    let artifact_value = json!({
      "artifact_id": artifact_id,
      "artifact_type": "task_package",
      "project_id": task.scope.project_id,
      "path": Value::Null,
      "title": task.title,
      "task_goal": task.task_goal,
      "source_kind": "project_director_task_plan",
      "source_ref": work_item_id,
      "permission_level": "plan_authorized_prepared",
      "version": 1,
      "stale": false,
      "stale_reasons": [],
      "task_name": task.title,
      "assigned_line": assigned_role_label(&task.scope.target_role),
      "background": [
        format!("来自项目主管 C4 拆任务；项目：{}", project.name),
        "prepared dispatch 只是准备态记录，仍未执行 worker。"
      ],
      "goals": goals_with_contract,
      "allowed_read_scope": task.scope.allowed_read_scope,
      "allowed_write": task.scope.allowed_write_scope,
      "available_skills": task.scope.available_skills,
      "available_knowledge_refs": task.scope.available_knowledge_refs,
      "forbidden_actions": forbidden_actions,
      "acceptance_criteria": task.acceptance_criteria,
      "report_format": task.report_format,
      "review_focus": [
        "项目主管确认过程事实前，worker 汇报只作为过程材料。",
        "汇报必须带证据、文件变化、风险和下一步建议。"
      ],
      "callable_tool_capabilities": task.scope.callable_tool_capabilities,
      "harness_requirements": task.scope.required_checks,
      "timeout_policy": task.scope.timeout_policy,
      "failure_policy": task.scope.failure_policy,
      "target_role": task.scope.target_role,
      "task_package_kind": task.scope.task_package_kind,
      "model_id": model_id,
      "model_context_policy": "local_only",
      "max_memory_items": 8,
      "max_estimated_tokens": 2000,
      "project_director_planned_task_id": task.planned_task_id,
      // C1·每任务独立会话：物化态先置 null（会话在链派该任务前才由 director 先生后绑建出并回填 thread_id）。
      // 纯加法一键·不碰判决体/guard；无 C1 的旧路径（手动挡/None）保持 null，语义 0-diff。
      "target_session_id": Value::Null,
      "created_at": timestamp,
      "updated_at": timestamp,
      "warnings": ["prepared_dispatch_is_not_worker_execution"]
    });
    let artifacts = array_mut(value, "artifacts")?;
    match artifact_index {
        Some(index) => artifacts[index] = artifact_value,
        None => artifacts.push(artifact_value),
    }
    let artifact = artifacts
        .iter_mut()
        .find(|artifact| {
            optional_string_from(artifact, "artifact_id").as_deref() == Some(artifact_id)
        })
        .ok_or_else(|| "写入 C4 task package artifact 后重新定位失败".to_string())?;
    task_memory_injection::write_snapshot_to_artifact(artifact, memory_snapshot)?;
    Ok(())
}

fn project_director_memory_snapshot(
    path: &Path,
    project_root_value: &str,
    task: &ProjectDirectorPlannedTask,
    work_item_id: &str,
    artifact_id: &str,
    timestamp: &str,
) -> Result<TaskPackageMemoryPacketSnapshot, String> {
    let input = TaskMemoryPacketBuildInput {
        project_root: project_root_value.to_string(),
        project_id: Some(task.scope.project_id.clone()),
        workflow_id: Some(task.scope.workflow_id.clone()),
        task_id: Some(work_item_id.to_string()),
        role_id: task.scope.target_role.clone(),
        task_goal: task.task_goal.clone(),
        retrieval_intent: "worker_task".to_string(),
        target_model_id: Some(
            task.scope
                .model_id
                .clone()
                .unwrap_or_else(|| "codex-local-prepared".to_string()),
        ),
        model_context_policy: "local_only".to_string(),
        max_memory_items: 8,
        max_estimated_tokens: 2000,
        expected_formal_store_revision: None,
        expected_candidate_store_revision: None,
        expected_observation_store_revision: None,
    };
    let output = preview_task_memory_packet_at(path, &input, timestamp)?;
    task_memory_injection::snapshot_from_build_output(
        &output,
        work_item_id,
        Some(artifact_id),
        timestamp,
    )
}

fn active_binding_for_planned_task(
    index: &Value,
    value: &Value,
    node_id: &str,
    work_item_id: &str,
) -> Option<ActiveBindingInfo> {
    let workflow_id = value
        .get("work_items")
        .and_then(Value::as_array)
        .and_then(|work_items| {
            work_items.iter().find(|work_item| {
                optional_string_from(work_item, "work_item_id").as_deref() == Some(work_item_id)
            })
        })
        .and_then(|work_item| optional_string_from(work_item, "workflow_id"))
        .or_else(|| node_id.split(":node:").next().map(str::to_string))?;
    let binding_index =
        workflow_node_session_binding_index(value, &workflow_id, node_id, Some(work_item_id))
            .or_else(|| workflow_node_session_binding_index(value, &workflow_id, node_id, None))?;
    let binding = value
        .get("workflow_node_session_bindings")
        .and_then(Value::as_array)
        .and_then(|bindings| bindings.get(binding_index))?;
    let native_thread_id = optional_string_from(binding, "native_thread_id")?;
    let mut warnings = string_array(binding, "warnings");
    let index_session = find_index_thread(index, &native_thread_id);
    if index_session.is_none() {
        warnings.push("binding_thread_missing_from_index".to_string());
    }
    let rollout_exists = bool_value(binding, "rollout_exists")
        && index_session
            .as_ref()
            .is_none_or(|session| session.rollout_exists);
    Some(ActiveBindingInfo {
        binding_id: optional_string_from(binding, "binding_id")?,
        native_thread_id,
        rollout_exists,
        warnings: dedupe_strings(warnings),
    })
}

fn find_task_package_artifact_by_id<'a>(value: &'a Value, work_item_id: &str) -> Option<&'a Value> {
    value
        .get("artifacts")
        .and_then(Value::as_array)
        .and_then(|artifacts| {
            artifacts.iter().find(|artifact| {
                optional_string_from(artifact, "source_ref").as_deref() == Some(work_item_id)
                    && optional_string_from(artifact, "artifact_type").as_deref()
                        == Some("task_package")
            })
        })
}

fn existing_prepared_dispatch_for_planned_task<'a>(
    value: &'a Value,
    planned_task_id: &str,
    work_item_id: &str,
    authorization_id: &str,
) -> Option<&'a Value> {
    value
        .get("workflow_node_dispatches")
        .and_then(Value::as_array)
        .and_then(|dispatches| {
            dispatches.iter().rev().find(|dispatch| {
                optional_string_from(dispatch, "state").as_deref() == Some("prepared")
                    && optional_string_from(dispatch, "work_item_id").as_deref()
                        == Some(work_item_id)
                    && optional_string_from(dispatch, "plan_authorization_id").as_deref()
                        == Some(authorization_id)
                    && optional_string_from(dispatch, "c4_planned_task_id")
                        .as_deref()
                        .is_none_or(|id| id == planned_task_id)
            })
        })
}

fn push_authorized_prepared_dispatch_created_audit(
    value: &mut Value,
    task: &ProjectDirectorPlannedTask,
    dispatch_id: &str,
    authorization_id: &str,
    timestamp: &str,
) -> Result<String, String> {
    let audit_event_id = format!(
        "audit:authorized-prepared-dispatch-created:{}:{timestamp}",
        stable_id(dispatch_id)
    );
    array_mut(value, "audit_events")?.push(json!({
      "event_id": audit_event_id,
      "event_type": "authorized_prepared_dispatch_created",
      "target_ref": task.work_item_id,
      "actor_ref": "project_director",
      "source_kind": "workspace_state",
      "permission_level": "plan_authorized_prepared",
      "before_state": "ready_to_dispatch",
      "after_state": "prepared",
      "created_at": timestamp,
      "reason": format!("项目主管在 active 授权 {} 范围内创建 prepared dispatch {}；只写准备态记录，仍未执行 worker。", authorization_id, dispatch_id),
      "plan_authorization_id": authorization_id,
      "project_director_planned_task_id": task.planned_task_id
    }));
    Ok(audit_event_id)
}

// C1·thread 延迟审计变体（canon 2026-07-09）：chain_binds_per_task=true 时产 prepared 但 thread 由链每任务补。
// 透明留档（不吞）——与 created 同族·只是 event_type/reason 点明「thread 延迟·链会 create_and_bind」。
fn push_authorized_prepared_dispatch_thread_deferred_audit(
    value: &mut Value,
    task: &ProjectDirectorPlannedTask,
    dispatch_id: &str,
    authorization_id: &str,
    timestamp: &str,
) -> Result<String, String> {
    let audit_event_id = format!(
        "audit:authorized-prepared-dispatch-thread-deferred:{}:{timestamp}",
        stable_id(dispatch_id)
    );
    array_mut(value, "audit_events")?.push(json!({
      "event_id": audit_event_id,
      "event_type": "authorized_prepared_dispatch_thread_deferred",
      "target_ref": task.work_item_id,
      "actor_ref": "project_director",
      "source_kind": "workspace_state",
      "permission_level": "plan_authorized_prepared",
      "before_state": "ready_to_dispatch",
      "after_state": "prepared",
      "created_at": timestamp,
      "reason": format!("项目主管在 active 授权 {} 范围内创建 prepared dispatch {}；C1 自动路——会话未预绑，thread 由链每任务 create_and_bind 补（授权/安全未松·仅放宽「有无会话」就绪判定）。", authorization_id, dispatch_id),
      "plan_authorization_id": authorization_id,
      "project_director_planned_task_id": task.planned_task_id
    }));
    Ok(audit_event_id)
}

fn push_authorized_prepared_dispatch_blocked_audit(
    value: &mut Value,
    task: &ProjectDirectorPlannedTask,
    authorization_id: &str,
    timestamp: &str,
    event_type: &str,
    reason: &str,
) -> Result<String, String> {
    let audit_event_id = format!(
        "audit:{event_type}:{}:{timestamp}",
        stable_id(&task.planned_task_id)
    );
    array_mut(value, "audit_events")?.push(json!({
      "event_id": audit_event_id,
      "event_type": event_type,
      "target_ref": task.work_item_id,
      "actor_ref": "project_director",
      "source_kind": "workspace_state",
      "permission_level": "plan_authorized_prepared",
      "before_state": "planned",
      "after_state": task.status,
      "created_at": timestamp,
      "reason": reason,
      "blocked_reasons": task.blocked_reasons,
      "plan_authorization_id": authorization_id,
      "project_director_planned_task_id": task.planned_task_id
    }));
    Ok(audit_event_id)
}

fn render_project_director_prepared_prompt(
    task: &ProjectDirectorPlannedTask,
    memory_snapshot: &TaskPackageMemoryPacketSnapshot,
) -> String {
    format!(
        "你将接收一个项目主管拆出的 worker 任务包准备态。\n\n任务：{}\n\n目标：\n{}\n\n授权边界：\n- 读取：{}\n- 写入：{}\n- 工具：{}\n- 检查：{}\n\n验收标准：\n{}\n\n必须回传：\n{}\n\n边界：prepared dispatch 只是工作台准备态记录；当前还未执行 worker。\n\n{}",
        task.title,
        task.task_goal,
        markdown_list_or_empty(&task.scope.allowed_read_scope),
        markdown_list_or_empty(&task.scope.allowed_write_scope),
        markdown_list_or_empty(&task.scope.callable_tool_capabilities),
        markdown_list_or_empty(&task.scope.required_checks),
        markdown_list_or_empty(&task.acceptance_criteria),
        markdown_list_or_empty(&task.report_format),
        task_memory_injection::render_prompt_block(memory_snapshot)
    )
}

fn c4_work_item_id(workflow_id: &str, planned_task_id: &str) -> String {
    format!(
        "work-item:{workflow_id}:project-director:{}",
        stable_id(planned_task_id)
    )
}

fn c4_task_package_artifact_id(workflow_id: &str, planned_task_id: &str) -> String {
    format!(
        "artifact:{workflow_id}:task-package:{}",
        stable_id(planned_task_id)
    )
}

fn c4_node_id(workflow_id: &str, role_id: &str) -> String {
    format!("{workflow_id}:node:{role_id}")
}

fn standard_project_director_report_format() -> Vec<String> {
    vec![
        "做了什么".to_string(),
        "改了哪些文件".to_string(),
        "验证命令和结果".to_string(),
        "证据引用".to_string(),
        "风险".to_string(),
        "需要项目主管确认的事项".to_string(),
    ]
}

fn edge_exists(value: &Value, edge_id: &str) -> bool {
    value
        .get("edges")
        .and_then(Value::as_array)
        .is_some_and(|edges| {
            edges
                .iter()
                .any(|edge| optional_string_from(edge, "edge_id").as_deref() == Some(edge_id))
        })
}

fn push_unique(values: &mut Vec<String>, value: &str) {
    if !values.iter().any(|item| item == value) {
        values.push(value.to_string());
    }
}

fn normalize_c4_symbol(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace('-', "_")
}
