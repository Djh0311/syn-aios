// Diagnostics, provider availability, session continuation, adapter descriptor, and session operation helpers split out during Root Treatment R2-B8.
// This file is included at crate root so helper visibility and behavior stay unchanged.

#[allow(clippy::too_many_arguments)]
fn derive_diagnostic_summary(
    state: &AppState,
    index: &Value,
    _projects: &[ProjectRecord],
    sessions: &[SessionRecord],
    agent_adapters: &[AgentAdapterDescriptor],
    provider_availability: &[ProviderAvailabilitySummary],
    session_continuation_store: &SessionContinuationStoreV1,
    runtime_session_attention: &[RuntimeSessionAttention],
    session_run_status_summaries: &[SessionRunStatusSummary],
    runtime_log_store: &RuntimeLogStoreV1,
    top_level_warning_count: usize,
    context_warning_count: usize,
    generated_at: &str,
) -> DiagnosticSummary {
    let mut store_integrity = vec![
        workflow_state_integrity(&state.workflow_state_path),
        json_file_integrity("index", "索引快照", &state.index_path, Some(index)),
        text_file_integrity("tasks", "任务入口", &state.tasks_path),
        sidecar_integrity(
            "session_continuation",
            "会话继续 sidecar",
            session_continuation_store
                .scope
                .sidecar_path
                .as_deref()
                .map(PathBuf::from),
            Some(&session_continuation_store.schema_version),
            session_continuation_store.continuations.len()
                + session_continuation_store.attempts.len()
                + session_continuation_store.audit_events.len(),
            session_continuation_store.warnings.len(),
            "G2 只读检查 continuation sidecar；不重试、不发送 prompt、不写 .codex。",
        ),
        sidecar_integrity(
            "runtime_log",
            "运行日志 sidecar",
            runtime_log_store
                .scope
                .sidecar_path
                .as_deref()
                .map(PathBuf::from),
            Some(&runtime_log_store.schema_version),
            runtime_log_store.entries.len(),
            runtime_log_store.warnings.len(),
            "Runtime log 只记录脱敏运行摘要；不能替代 audit event。",
        ),
    ];
    store_integrity.extend(derived_store_integrity_findings(&state.workflow_state_path));

    let mut degraded_states = Vec::new();
    if top_level_warning_count + context_warning_count > 0 {
        degraded_states.push(ServiceDegradedState {
            state_id: "diagnostic:index_warnings".to_string(),
            kind: "index_warning".to_string(),
            severity: "warning".to_string(),
            title: "索引或项目上下文存在 warning".to_string(),
            summary: format!(
                "顶层 warning {top_level_warning_count} 个，项目上下文 warning {context_warning_count} 个。"
            ),
            user_action_required: false,
            blocks_real_execution: false,
            source_refs: vec!["workbench_snapshot.diagnostics".to_string()],
            recommended_next_step: "在管理入口查看 warning 摘要；G2 不自动修复索引。".to_string(),
        });
    }
    let planned_adapters = agent_adapters
        .iter()
        .filter(|adapter| {
            adapter.status == "planned"
                || adapter.execution_status == "not_implemented"
                || adapter.credential_status == "not_configured"
                || adapter.model_access_status == "not_verified"
        })
        .count();
    if planned_adapters > 0 {
        degraded_states.push(ServiceDegradedState {
            state_id: "diagnostic:planned_adapters".to_string(),
            kind: "adapter_unavailable".to_string(),
            severity: "warning".to_string(),
            title: "存在 planned / unavailable adapter".to_string(),
            summary: format!("{planned_adapters} 个 adapter 仍是计划中、未配置凭据或模型未验证。"),
            user_action_required: false,
            blocks_real_execution: true,
            source_refs: vec!["workbench_snapshot.agent_adapters".to_string()],
            recommended_next_step: "保持只读展示；真实接入必须另拆任务包并取得授权。".to_string(),
        });
    }
    let provider_unavailable = provider_availability
        .iter()
        .filter(|provider| provider.availability_status != "available_readonly")
        .count();
    if provider_unavailable > 0 {
        degraded_states.push(ServiceDegradedState {
            state_id: "diagnostic:provider_boundary".to_string(),
            kind: "provider_unverified".to_string(),
            severity: "warning".to_string(),
            title: "Provider / 模型 / 凭据只读边界未解除".to_string(),
            summary: format!(
                "{provider_unavailable} 个 provider availability 条目不是可用只读状态。"
            ),
            user_action_required: false,
            blocks_real_execution: true,
            source_refs: vec!["workbench_snapshot.provider_availability".to_string()],
            recommended_next_step: "不要探测 token 或调用 provider；后续如需验证凭据必须单独授权。"
                .to_string(),
        });
    }
    let blocked_runtime = runtime_session_attention
        .iter()
        .filter(|attention| {
            attention.blocks_continuation
                || matches!(
                    attention.readback_boundary.status.as_str(),
                    "readback_unavailable"
                        | "readback_failed"
                        | "readback_timed_out"
                        | "blocked_by_guard"
                        | "duplicate_blocked"
                        | "stale_cancelled"
                        | "timed_out"
                )
        })
        .count();
    if blocked_runtime > 0 {
        degraded_states.push(ServiceDegradedState {
            state_id: "diagnostic:runtime_attention".to_string(),
            kind: "runtime_attention".to_string(),
            severity: "warning".to_string(),
            title: "运行关注存在阻断或 readback 边界".to_string(),
            summary: format!(
                "{blocked_runtime} 条运行关注需要解释；readback unavailable 不是 0 条结果。"
            ),
            user_action_required: runtime_session_attention
                .iter()
                .any(|attention| attention.requires_user_action),
            blocks_real_execution: runtime_session_attention
                .iter()
                .any(|attention| attention.blocks_continuation),
            source_refs: vec!["workbench_snapshot.runtime_session_attention".to_string()],
            recommended_next_step: "查看运行中入口和管理入口摘要；G2 不自动 resume、retry 或修复。"
                .to_string(),
        });
    }
    let runtime_errors = runtime_log_store
        .entries
        .iter()
        .filter(|entry| entry.severity == "error")
        .count();
    if runtime_errors > 0 {
        degraded_states.push(ServiceDegradedState {
            state_id: "diagnostic:runtime_log_errors".to_string(),
            kind: "runtime_log_error".to_string(),
            severity: "degraded".to_string(),
            title: "运行日志存在 error 摘要".to_string(),
            summary: format!("运行日志中有 {runtime_errors} 条 error 级别脱敏摘要。"),
            user_action_required: false,
            blocks_real_execution: false,
            source_refs: vec!["workbench_snapshot.runtime_log_store".to_string()],
            recommended_next_step: "打开管理入口查看最近错误；不要把 runtime log 当审计。"
                .to_string(),
        });
    }
    if sessions.is_empty() {
        degraded_states.push(ServiceDegradedState {
            state_id: "diagnostic:tauri_bridge_or_session_index".to_string(),
            kind: "tauri_bridge_or_session_index".to_string(),
            severity: "warning".to_string(),
            title: "Tauri bridge / session index 缺少可验证会话数据".to_string(),
            summary: "当前 snapshot 没有可展示 session；G2 只能提示桥接或索引数据缺失，不能自动探测 .codex。".to_string(),
            user_action_required: false,
            blocks_real_execution: false,
            source_refs: vec!["workbench_snapshot.sessions".to_string()],
            recommended_next_step: "后续 G3 用真实 Tauri 验收桥接；G2 不读取完整 transcript 或 .codex secret。".to_string(),
        });
    }
    if std::env::var("CI").is_err() && std::env::var("TAURI_ENV_DEBUG").is_err() {
        degraded_states.push(ServiceDegradedState {
            state_id: "diagnostic:test_environment_unverified".to_string(),
            kind: "test_environment_unverified".to_string(),
            severity: "warning".to_string(),
            title: "测试 / Tauri 环境未由 G2 真实验证".to_string(),
            summary: "G2 仅提供读模型；真实窗口、截图权限、端口和 Tauri bridge 验收留给 G3。"
                .to_string(),
            user_action_required: false,
            blocks_real_execution: false,
            source_refs: vec!["stage_g_g3_deferred".to_string()],
            recommended_next_step: "执行 G3 时再启动真实 Tauri 并保存截图证据。".to_string(),
        });
    }
    degraded_states.push(ServiceDegradedState {
        state_id: "diagnostic:bundle_reference".to_string(),
        kind: "diagnostic_bundle_reference".to_string(),
        severity: "info".to_string(),
        title: "诊断 bundle 为只读引用".to_string(),
        summary: "G2 在 WorkbenchSnapshot.diagnostic_summary 中提供可引用诊断 bundle；不导出 secret、不生成新文件。".to_string(),
        user_action_required: false,
        blocks_real_execution: false,
        source_refs: vec![
            "workbench_snapshot.diagnostic_summary".to_string(),
            "workbench_snapshot.runtime_log_store".to_string(),
        ],
        recommended_next_step: "evidence 可引用该读模型；如需落盘导出 bundle，必须另拆任务并定义脱敏规则。".to_string(),
    });
    degraded_states.extend(
        store_integrity
            .iter()
            .filter(|finding| matches!(finding.status.as_str(), "warning" | "degraded" | "missing"))
            .map(|finding| ServiceDegradedState {
                state_id: format!("diagnostic:store:{}", finding.store_id),
                kind: "store_integrity".to_string(),
                severity: finding.severity.clone(),
                title: format!("{} 状态：{}", finding.label, finding.status),
                summary: finding.summary.clone(),
                user_action_required: finding.status == "degraded",
                blocks_real_execution: finding.status == "degraded",
                source_refs: vec![finding
                    .path
                    .clone()
                    .unwrap_or_else(|| finding.store_id.clone())],
                recommended_next_step: "只读记录诊断；如需修复 store，必须另拆维护任务。"
                    .to_string(),
            }),
    );

    let blocked_count = degraded_states
        .iter()
        .filter(|state| state.blocks_real_execution)
        .count();
    let degraded_count = degraded_states
        .iter()
        .filter(|state| state.severity == "degraded" || state.severity == "error")
        .count();
    let warning_count = degraded_states
        .iter()
        .filter(|state| state.severity == "warning")
        .count()
        + store_integrity
            .iter()
            .filter(|finding| finding.status == "warning")
            .count();
    let healthy_count = store_integrity
        .iter()
        .filter(|finding| finding.status == "ok")
        .count()
        + session_run_status_summaries
            .iter()
            .filter(|summary| summary.blocking_count == 0 && summary.needs_user_count == 0)
            .count();
    let overall_severity = if blocked_count > 0 || degraded_count > 0 {
        "degraded"
    } else if warning_count > 0 {
        "warning"
    } else {
        "healthy"
    }
    .to_string();
    let status = if overall_severity == "healthy" {
        "healthy"
    } else {
        "degraded_readonly"
    }
    .to_string();
    let recent_error_summaries = runtime_log_store
        .entries
        .iter()
        .filter(|entry| entry.severity == "error" || entry.status.contains("failed"))
        .take(5)
        .map(|entry| format!("{} · {}", entry.category, entry.summary))
        .collect();

    DiagnosticSummary {
        status,
        generated_at: generated_at.to_string(),
        overall_severity,
        healthy_count,
        warning_count,
        degraded_count,
        blocked_count,
        store_integrity,
        degraded_states,
        recent_error_summaries,
        boundary_notes: vec![
            "G2 是只读诊断，不自动修复 store、不自动重试、不调用 provider。".to_string(),
            "readback_unavailable 表示无法读回，不能显示成 0 条结果。".to_string(),
            "runtime log 记录运行摘要；audit event 记录可追责决定，两者不可互相替代。".to_string(),
            "真实 Tauri 截图验收仍属于 G3，不由 G2 冒领。".to_string(),
        ],
    }
}
fn workflow_state_integrity(workflow_state_path: &Path) -> StoreIntegrityFinding {
    let mut finding =
        json_file_integrity("workflow_state", "工作流事实层", workflow_state_path, None);
    finding.boundary = "只读解析 workflow-state.v0.json；G2 不修改状态枚举或顶层结构。".to_string();
    finding
}

fn json_file_integrity(
    store_id: &str,
    label: &str,
    path: &Path,
    parsed: Option<&Value>,
) -> StoreIntegrityFinding {
    if let Some(value) = parsed {
        return StoreIntegrityFinding {
            store_id: store_id.to_string(),
            label: label.to_string(),
            status: "ok".to_string(),
            severity: "info".to_string(),
            path: Some(path.display().to_string()),
            schema_version: optional_string_from(value, "schema_version"),
            revision: optional_i64_from(value, "revision"),
            item_count: value.as_object().map_or(0, |object| object.len()),
            warning_count: array_len(value, "warnings"),
            error: None,
            summary: format!("{label} 已由现有读取路径解析。"),
            boundary: "只读解析；不写入、不迁移、不修复。".to_string(),
        };
    }
    match fs::read_to_string(path) {
        Ok(text) => match serde_json::from_str::<Value>(&text) {
            Ok(value) => StoreIntegrityFinding {
                store_id: store_id.to_string(),
                label: label.to_string(),
                status: "ok".to_string(),
                severity: "info".to_string(),
                path: Some(path.display().to_string()),
                schema_version: optional_string_from(&value, "schema_version"),
                revision: optional_i64_from(&value, "revision"),
                item_count: value.as_object().map_or(0, |object| object.len()),
                warning_count: array_len(&value, "warnings"),
                error: None,
                summary: format!("{label} 可读取且 JSON 可解析。"),
                boundary: "只读解析；不写入、不迁移、不修复。".to_string(),
            },
            Err(error) => StoreIntegrityFinding {
                store_id: store_id.to_string(),
                label: label.to_string(),
                status: "degraded".to_string(),
                severity: "degraded".to_string(),
                path: Some(path.display().to_string()),
                schema_version: None,
                revision: None,
                item_count: 0,
                warning_count: 1,
                error: Some(format!("JSON 解析失败：{error}")),
                summary: format!("{label} JSON 损坏或不可解析，G2 只报告不覆盖。"),
                boundary: "只读诊断；损坏 JSON 必须拒绝覆盖，修复另拆任务。".to_string(),
            },
        },
        Err(error) => StoreIntegrityFinding {
            store_id: store_id.to_string(),
            label: label.to_string(),
            status: "missing".to_string(),
            severity: "warning".to_string(),
            path: Some(path.display().to_string()),
            schema_version: None,
            revision: None,
            item_count: 0,
            warning_count: 1,
            error: Some(format!("读取失败：{error}")),
            summary: format!("{label} 当前不可读取或不存在。"),
            boundary: "只读诊断；不创建文件、不自动初始化。".to_string(),
        },
    }
}

fn text_file_integrity(store_id: &str, label: &str, path: &Path) -> StoreIntegrityFinding {
    match fs::read_to_string(path) {
        Ok(text) => StoreIntegrityFinding {
            store_id: store_id.to_string(),
            label: label.to_string(),
            status: "ok".to_string(),
            severity: "info".to_string(),
            path: Some(path.display().to_string()),
            schema_version: None,
            revision: None,
            item_count: text.lines().count(),
            warning_count: 0,
            error: None,
            summary: format!("{label} 可读取。"),
            boundary: "只读读取任务入口；不改队列。".to_string(),
        },
        Err(error) => StoreIntegrityFinding {
            store_id: store_id.to_string(),
            label: label.to_string(),
            status: "missing".to_string(),
            severity: "warning".to_string(),
            path: Some(path.display().to_string()),
            schema_version: None,
            revision: None,
            item_count: 0,
            warning_count: 1,
            error: Some(format!("读取失败：{error}")),
            summary: format!("{label} 当前不可读取或不存在。"),
            boundary: "只读诊断；不创建文件。".to_string(),
        },
    }
}

fn sidecar_integrity(
    store_id: &str,
    label: &str,
    path: Option<PathBuf>,
    schema_version: Option<&str>,
    item_count: usize,
    warning_count: usize,
    boundary: &str,
) -> StoreIntegrityFinding {
    let status = if warning_count > 0 { "warning" } else { "ok" };
    StoreIntegrityFinding {
        store_id: store_id.to_string(),
        label: label.to_string(),
        status: status.to_string(),
        severity: if status == "ok" { "info" } else { "warning" }.to_string(),
        path: path.as_ref().map(|path| path.display().to_string()),
        schema_version: schema_version.map(str::to_string),
        revision: None,
        item_count,
        warning_count,
        error: None,
        summary: if status == "ok" {
            format!("{label} 已读取为只读摘要。")
        } else {
            format!("{label} 有 {warning_count} 条 warning，G2 只解释不修复。")
        },
        boundary: boundary.to_string(),
    }
}

fn derived_store_integrity_findings(workflow_state_path: &Path) -> Vec<StoreIntegrityFinding> {
    let sidecars = [
        (
            "formal_memory",
            "正式记忆 sidecar",
            "formal-memories.v1.json",
        ),
        (
            "memory_candidate",
            "候选记忆 sidecar",
            "memory-candidates.v1.json",
        ),
        (
            "blackboard_candidate",
            "黑板候选 sidecar",
            "blackboard-candidates.v1.json",
        ),
        ("observation", "观察来源 sidecar", "observations.v1.json"),
        ("memory_lint", "记忆 lint sidecar", "memory-lint.v1.json"),
        (
            "memory_entity_relation",
            "实体关系 sidecar",
            "memory-entity-relations.v1.json",
        ),
        (
            "memory_pattern",
            "成熟模式 sidecar",
            "memory-patterns.v1.json",
        ),
        (
            "plan_authorization",
            "方案授权 sidecar",
            "plan-authorizations.v1.json",
        ),
        (
            "project_proposal",
            "项目咨询方案 sidecar",
            "project-proposals.v1.json",
        ),
    ];
    let Some(parent) = workflow_state_path.parent() else {
        return vec![StoreIntegrityFinding {
            store_id: "workflow_sidecar_parent".to_string(),
            label: "工作流 sidecar 目录".to_string(),
            status: "degraded".to_string(),
            severity: "degraded".to_string(),
            path: Some(workflow_state_path.display().to_string()),
            schema_version: None,
            revision: None,
            item_count: 0,
            warning_count: 1,
            error: Some("workflow state 路径缺少父目录".to_string()),
            summary: "无法推导 sidecar 目录。".to_string(),
            boundary: "只读诊断；不创建目录。".to_string(),
        }];
    };
    let mut findings = sidecars
        .iter()
        .map(|(store_id, label, file_name)| {
            let path = parent.join(file_name);
            if !path.exists() {
                return StoreIntegrityFinding {
                    store_id: (*store_id).to_string(),
                    label: (*label).to_string(),
                    status: "missing".to_string(),
                    severity: "warning".to_string(),
                    path: Some(path.display().to_string()),
                    schema_version: None,
                    revision: None,
                    item_count: 0,
                    warning_count: 1,
                    error: Some("sidecar_missing".to_string()),
                    summary: format!("{label} 不存在；G2 只记录缺失，不自动初始化。"),
                    boundary: "只读诊断；缺失 sidecar 由对应阶段或维护任务处理。".to_string(),
                };
            }
            match fs::read_to_string(&path) {
                Ok(text) => match serde_json::from_str::<Value>(&text) {
                    Ok(value) => {
                        let item_count = value.as_object().map_or(0, |object| {
                            object
                                .values()
                                .filter_map(Value::as_array)
                                .map(Vec::len)
                                .sum::<usize>()
                        });
                        let warning_count = array_len(&value, "warnings");
                        StoreIntegrityFinding {
                            store_id: (*store_id).to_string(),
                            label: (*label).to_string(),
                            status: if warning_count > 0 { "warning" } else { "ok" }.to_string(),
                            severity: if warning_count > 0 { "warning" } else { "info" }
                                .to_string(),
                            path: Some(path.display().to_string()),
                            schema_version: optional_string_from(&value, "schema_version"),
                            revision: optional_i64_from(&value, "revision"),
                            item_count,
                            warning_count,
                            error: None,
                            summary: if warning_count > 0 {
                                format!("{label} 可解析，但存在 {warning_count} 条 warning。")
                            } else {
                                format!("{label} 可解析。")
                            },
                            boundary: "只读 JSON integrity probe；不修改 sidecar。".to_string(),
                        }
                    }
                    Err(error) => StoreIntegrityFinding {
                        store_id: (*store_id).to_string(),
                        label: (*label).to_string(),
                        status: "degraded".to_string(),
                        severity: "degraded".to_string(),
                        path: Some(path.display().to_string()),
                        schema_version: None,
                        revision: None,
                        item_count: 0,
                        warning_count: 1,
                        error: Some(format!("JSON 解析失败：{error}")),
                        summary: format!("{label} JSON 损坏，G2 拒绝覆盖。"),
                        boundary: "只读诊断；损坏 JSON 必须另拆修复任务。".to_string(),
                    },
                },
                Err(error) => StoreIntegrityFinding {
                    store_id: (*store_id).to_string(),
                    label: (*label).to_string(),
                    status: "degraded".to_string(),
                    severity: "degraded".to_string(),
                    path: Some(path.display().to_string()),
                    schema_version: None,
                    revision: None,
                    item_count: 0,
                    warning_count: 1,
                    error: Some(format!("读取失败：{error}")),
                    summary: format!("{label} 无法读取，G2 不修复。"),
                    boundary: "只读诊断；不修改权限或文件。".to_string(),
                },
            }
        })
        .collect::<Vec<_>>();
    findings.extend(memory_consistency::derive_store_integrity_findings(
        workflow_state_path,
        &unix_timestamp_string(),
    ));
    findings
}

fn derive_provider_availability_summaries(
    adapters: &[AgentAdapterDescriptor],
    session_operations: &[SessionOperationDescriptor],
) -> Vec<ProviderAvailabilitySummary> {
    adapters
        .iter()
        .map(|adapter| provider_availability_for_adapter(adapter, session_operations))
        .collect()
}

fn provider_availability_for_adapter(
    adapter: &AgentAdapterDescriptor,
    session_operations: &[SessionOperationDescriptor],
) -> ProviderAvailabilitySummary {
    let is_codex = adapter.adapter_id == "codex-local";
    let planned_adapter =
        adapter.status == "planned" || adapter.execution_status == "not_implemented";
    let operations_need_future_task = session_operations.iter().any(|operation| {
        operation.adapter_id == adapter.adapter_id
            && matches!(
                operation.current_status.as_str(),
                "requires_future_task" | "blocked" | "blocked_destructive" | "planned"
            )
    });
    let mut warnings = vec![
        "provider_availability_read_model_only".to_string(),
        "credential_secret_not_read".to_string(),
        "model_not_verified".to_string(),
        "cost_not_estimated".to_string(),
        "provider_availability_not_project_authorization".to_string(),
        "no_external_provider_call_in_e3".to_string(),
    ];
    if operations_need_future_task {
        warnings.push("session_operation_requires_future_task".to_string());
    }

    if is_codex {
        let availability_status = match adapter.status.as_str() {
            "available" => "available_readonly",
            "degraded" => "unknown",
            "not_connected" => "not_connected",
            _ => "unknown",
        };
        return ProviderAvailabilitySummary {
            adapter_id: adapter.adapter_id.clone(),
            provider_id: "local-codex-cli".to_string(),
            provider_label: "Codex 本地 CLI".to_string(),
            provider_kind: "local_cli".to_string(),
            adapter_status: adapter.status.clone(),
            availability_status: availability_status.to_string(),
            credential_status: "not_required_by_workbench".to_string(),
            model_status: "local_cli_managed".to_string(),
            external_call_status: "not_needed_for_readonly".to_string(),
            cost_risk_status: "unknown".to_string(),
            user_visible_reason:
                "Codex 由本地 CLI 管理；工作台只读取索引和边界状态，不读取凭据、不验证模型、不发起 provider 调用。"
                    .to_string(),
            safe_to_display: true,
            requires_user_configuration: adapter.requires_user_setup,
            requires_future_task: operations_need_future_task,
            warnings,
        };
    }

    if planned_adapter {
        warnings.push("planned_adapter_not_connected".to_string());
        warnings.push("external_call_blocked".to_string());
    }

    ProviderAvailabilitySummary {
        adapter_id: adapter.adapter_id.clone(),
        provider_id: adapter.provider.clone(),
        provider_label: adapter.display_name.clone(),
        provider_kind: provider_kind_for_adapter(&adapter.adapter_id).to_string(),
        adapter_status: adapter.status.clone(),
        availability_status: if planned_adapter {
            "planned"
        } else {
            "unknown"
        }
        .to_string(),
        credential_status: if planned_adapter {
            "credential_missing"
        } else {
            "unknown"
        }
        .to_string(),
        model_status: if planned_adapter {
            "model_unverified"
        } else {
            "unknown"
        }
        .to_string(),
        external_call_status: if planned_adapter {
            "external_call_blocked"
        } else {
            "requires_future_authorization"
        }
        .to_string(),
        cost_risk_status: if planned_adapter {
            "blocked_until_authorized"
        } else {
            "unknown"
        }
        .to_string(),
        user_visible_reason: format!(
            "{} 仍是 planned descriptor；没有真实命令、会话、凭据或模型访问，外发调用已阻断。",
            adapter.display_name
        ),
        safe_to_display: true,
        requires_user_configuration: true,
        requires_future_task: true,
        warnings,
    }
}

fn provider_kind_for_adapter(adapter_id: &str) -> &'static str {
    match adapter_id {
        "claude-code" => "external_cli_planned",
        "openclaw" => "external_agent_planned",
        "opencode" => "external_cli_planned",
        "opencode-like" => "compatible_adapter_planned",
        _ => "unknown",
    }
}

fn derive_session_continuation_previews(
    adapters: &[AgentAdapterDescriptor],
    session_operations: &[SessionOperationDescriptor],
    provider_availability: &[ProviderAvailabilitySummary],
    workflow_state: Option<&WorkflowStateSnapshot>,
) -> Vec<SessionContinuationPreview> {
    let mut previews = Vec::new();
    for adapter in adapters {
        let operations = session_operations
            .iter()
            .filter(|operation| {
                operation.adapter_id == adapter.adapter_id
                    && matches!(
                        operation.operation_id.as_str(),
                        "new_session" | "send_message" | "resume"
                    )
            })
            .collect::<Vec<_>>();
        let provider_summary = provider_availability
            .iter()
            .find(|summary| summary.adapter_id == adapter.adapter_id);
        let active_bindings =
            active_session_bindings_for_adapter(workflow_state, &adapter.adapter_id);
        for operation in operations {
            if adapter.adapter_id == "codex-local" && !active_bindings.is_empty() {
                for (workflow, binding) in &active_bindings {
                    previews.push(session_continuation_preview_for_binding(
                        adapter,
                        operation,
                        provider_summary,
                        Some(workflow),
                        Some(binding),
                    ));
                }
            } else {
                previews.push(session_continuation_preview_for_binding(
                    adapter,
                    operation,
                    provider_summary,
                    None,
                    None,
                ));
            }
        }
    }
    previews
}

fn active_session_bindings_for_adapter<'a>(
    workflow_state: Option<&'a WorkflowStateSnapshot>,
    adapter_id: &str,
) -> Vec<(&'a ProjectWorkflowSummary, &'a WorkflowNodeSessionBinding)> {
    workflow_state
        .iter()
        .flat_map(|snapshot| snapshot.project_workflows.iter())
        .flat_map(|workflow| {
            workflow
                .node_session_bindings
                .iter()
                .filter(move |binding| {
                    binding.adapter_id == adapter_id && binding.lifecycle == "active"
                })
                .map(move |binding| (workflow, binding))
        })
        .collect()
}

fn session_continuation_preview_for_binding(
    adapter: &AgentAdapterDescriptor,
    operation: &SessionOperationDescriptor,
    provider_summary: Option<&ProviderAvailabilitySummary>,
    workflow: Option<&&ProjectWorkflowSummary>,
    binding: Option<&&WorkflowNodeSessionBinding>,
) -> SessionContinuationPreview {
    let workflow = workflow.copied();
    let binding = binding.copied();
    let project_root = workflow.map(|workflow| workflow.project_root.clone());
    let target_cwd = project_root.clone();
    let allowed_write_roots = project_root.iter().cloned().collect::<Vec<_>>();
    let work_item_id = binding.and_then(|binding| binding.work_item_id.clone());
    let session_id = if operation.operation_id == "new_session" {
        None
    } else {
        binding.map(|binding| binding.native_thread_id.clone())
    };
    let request = SessionContinuationRequest {
        adapter_id: adapter.adapter_id.clone(),
        operation_id: operation.operation_id.clone(),
        project_id: binding
            .map(|binding| binding.project_id.clone())
            .or_else(|| workflow.map(|workflow| workflow.project_id.clone())),
        project_root: project_root.clone(),
        workflow_id: binding
            .map(|binding| binding.workflow_id.clone())
            .or_else(|| workflow.map(|workflow| workflow.workflow_id.clone())),
        node_id: binding.map(|binding| binding.node_id.clone()),
        session_id: session_id.clone(),
        work_item_id: work_item_id.clone(),
        target_cwd: target_cwd.clone(),
        allowed_write_roots: allowed_write_roots.clone(),
        sandbox: "workspace-write-preview-only".to_string(),
        prompt_source_kind: continuation_prompt_source_kind(&operation.operation_id).to_string(),
        prompt_summary: continuation_prompt_summary(&operation.operation_id, binding),
        readback_strategy: if binding.is_some() {
            "required".to_string()
        } else {
            "not_defined".to_string()
        },
        requested_by: if operation.operation_id == "new_session" {
            "workbench_h3_1_new_session_preview".to_string()
        } else {
            "workbench_e4_preview".to_string()
        },
        user_confirmation_state: "missing".to_string(),
    };
    let guard_result = inspect_session_continuation_guard(
        &request,
        Some(adapter),
        Some(operation),
        provider_summary,
    );
    let preview_id = format!(
        "session-continuation-preview:{}:{}:{}",
        adapter.adapter_id,
        operation.operation_id,
        binding
            .map(|binding| binding.binding_id.as_str())
            .unwrap_or("unbound")
    );
    let mut user_visible_warnings = vec![
        "session_continuation_preview_only".to_string(),
        "no_prompt_sent_in_e4".to_string(),
        "no_codex_home_write_in_e4".to_string(),
        "user_confirmation_required_before_execution".to_string(),
    ];
    user_visible_warnings.extend(guard_result.warnings.clone());
    user_visible_warnings.sort();
    user_visible_warnings.dedup();

    SessionContinuationPreview {
        preview_id,
        adapter_id: adapter.adapter_id.clone(),
        operation_id: operation.operation_id.clone(),
        target_session_id: session_id,
        target_session_title: if operation.operation_id == "new_session" {
            None
        } else {
            binding.map(|binding| binding.session_title.clone())
        },
        project_id: request.project_id.clone(),
        project_root: project_root.clone(),
        workflow_id: request.workflow_id.clone(),
        node_id: request.node_id.clone(),
        binding_id: binding.map(|binding| binding.binding_id.clone()),
        work_item_id,
        target_cwd,
        allowed_write_roots_summary: allowed_write_roots,
        sandbox_summary: request.sandbox.clone(),
        prompt_source_kind: request.prompt_source_kind.clone(),
        prompt_summary: request.prompt_summary.clone(),
        readback_expectation: continuation_readback_expectation(&request),
        failure_handling: continuation_failure_boundary(&request),
        audit_impact: continuation_audit_impact(&request),
        provider_availability_summary: provider_summary.cloned(),
        guard_result,
        request,
        user_visible_warnings,
    }
}

fn continuation_prompt_source_kind(operation_id: &str) -> &'static str {
    match operation_id {
        "new_session" => "h3_new_session_task_package",
        "send_message" => "workflow_followup",
        "resume" => "task_package_summary",
        _ => "not_allowed",
    }
}

fn continuation_prompt_summary(
    operation_id: &str,
    binding: Option<&WorkflowNodeSessionBinding>,
) -> String {
    let target = binding
        .map(|binding| {
            format!(
                "{} / {} / {} / {}",
                binding.project_id,
                binding.node_id,
                binding
                    .work_item_id
                    .as_deref()
                    .unwrap_or("missing-work-item"),
                binding.native_thread_id
            )
        })
        .unwrap_or_else(|| "未绑定 project / workflow / node / work item / session".to_string());
    match operation_id {
        "new_session" => format!(
            "H3.1 新会话预览：为已绑定 work item 准备独立 Codex 新会话请求；目标 {target}。不创建真实会话，不发送 prompt，不写 Codex 原生状态。"
        ),
        "send_message" => format!(
            "E4 只读预览：继续已绑定会话的下一轮项目意图；目标 {target}。不显示 raw prompt，不发送消息。"
        ),
        "resume" => format!(
            "E4 只读预览：resume 只作为会话继续协议检查；目标 {target}。workflow dispatch 经验仅作边界参考，本轮不启动派发。"
        ),
        _ => "E4 只读预览：该操作不属于会话继续范围。".to_string(),
    }
}

fn continuation_readback_expectation(request: &SessionContinuationRequest) -> ReadbackExpectation {
    let expected_sources = if request.operation_id == "new_session" {
        vec![
            "future_h3_new_session_last_message".to_string(),
            "future_h3_attempt_audit".to_string(),
        ]
    } else {
        vec![
            "target_session_rollout_readback".to_string(),
            "future_e5_attempt_audit".to_string(),
        ]
    };
    let unavailable_behavior = if request.operation_id == "new_session" {
        "H3.1 只定义新会话 readback expectation；真实 readback 必须等 H3-B 真实执行后从受控 last-message / audit 读取，不能伪装成 0 条结果。"
    } else {
        "E4 只定义 readback expectation；真实 readback 失败必须在 E5 / G1 显示为 unavailable，不能伪装成 0 条结果。"
    };
    ReadbackExpectation {
        strategy: request.readback_strategy.clone(),
        required: request.readback_strategy == "required",
        expected_sources,
        unavailable_behavior: unavailable_behavior.to_string(),
        warnings: vec!["readback_expectation_only_no_readback_in_e4".to_string()],
    }
}

fn continuation_failure_boundary(
    request: &SessionContinuationRequest,
) -> ContinuationFailureBoundary {
    let user_visible_behavior = if request.operation_id == "new_session" {
        "本轮只展示 H3.1 新会话 guard 和权限预览；真实失败、超时、取消和重试边界进入 H3-B / G1。"
    } else {
        "本轮只展示 guard 和权限预览；真实失败、超时、取消和重试边界进入 E5 / E6 / G1。"
    };
    ContinuationFailureBoundary {
        timeout_policy: "deferred_to_e5_runtime_boundary".to_string(),
        retry_policy: "no_retry_in_e4".to_string(),
        failure_record: "no_attempt_or_runtime_log_written_in_e4".to_string(),
        user_visible_behavior: user_visible_behavior.to_string(),
        warnings: vec!["failure_boundary_preview_only".to_string()],
    }
}

fn continuation_audit_impact(request: &SessionContinuationRequest) -> ContinuationAuditImpact {
    let future_audit_requirement = if request.operation_id == "new_session" {
        "H3-B 真实新会话前必须写用户确认、attempt、runtime log / continuation record、readback 和失败审计。"
    } else {
        "E5 真实 send / resume 前必须写用户确认、attempt、dispatch / continuation record、readback 和失败审计。"
    };
    ContinuationAuditImpact {
        impact_kind: "preview_only_no_execution".to_string(),
        writes_attempt_in_e4: false,
        writes_dispatch_in_e4: false,
        writes_readback_in_e4: false,
        future_audit_requirement: future_audit_requirement.to_string(),
        warnings: vec!["would_require_attempt_audit_in_e5".to_string()],
    }
}

fn inspect_session_continuation_guard(
    request: &SessionContinuationRequest,
    adapter: Option<&AgentAdapterDescriptor>,
    operation: Option<&SessionOperationDescriptor>,
    provider_summary: Option<&ProviderAvailabilitySummary>,
) -> SessionContinuationGuardResult {
    let mut blocked = false;
    let mut requires_future_task = false;
    let mut reasons = vec!["e4_preview_only_no_execution".to_string()];
    let mut required_fixes = Vec::new();
    let mut warnings = vec![
        "session_continuation_preview_only".to_string(),
        "no_prompt_sent_in_e4".to_string(),
        "no_codex_home_write_in_e4".to_string(),
        "h3_1_no_real_new_session".to_string(),
    ];

    match adapter {
        Some(adapter)
            if adapter.adapter_id == "codex-local"
                && adapter.execution_status != "not_implemented"
                && adapter.status != "planned" => {}
        Some(adapter) => {
            blocked = true;
            reasons.push(format!("planned_adapter_blocked:{}", adapter.adapter_id));
            required_fixes
                .push("先完成该 adapter 的真实接入、凭据边界和模型验证任务。".to_string());
            warnings.push("planned_adapter_blocked".to_string());
        }
        None => {
            blocked = true;
            reasons.push("adapter_descriptor_missing".to_string());
            required_fixes.push("必须先选择已登记 adapter descriptor。".to_string());
        }
    }

    match operation {
        Some(operation)
            if matches!(
                operation.operation_id.as_str(),
                "new_session" | "send_message" | "resume"
            ) && operation.adapter_id == request.adapter_id => {}
        Some(operation) => {
            blocked = true;
            reasons.push(format!(
                "operation_not_allowed_in_e4:{}",
                operation.operation_id
            ));
            required_fixes.push(
                "E4/H3.1 只允许 new_session / send_message / resume 的预览协议。".to_string(),
            );
        }
        None => {
            blocked = true;
            reasons.push("session_operation_descriptor_missing".to_string());
            required_fixes.push("必须先有 E2 session operation descriptor。".to_string());
        }
    }

    if request
        .project_id
        .as_deref()
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        blocked = true;
        reasons.push("missing_project_binding".to_string());
        required_fixes.push("必须绑定 project，不能自由会话绕过项目上下文。".to_string());
    }
    if request
        .project_root
        .as_deref()
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        blocked = true;
        reasons.push("missing_project_root".to_string());
        required_fixes.push("必须提供 project root 才能判断 cwd 和 allowed roots。".to_string());
    }
    if request
        .workflow_id
        .as_deref()
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        blocked = true;
        reasons.push("missing_workflow_binding".to_string());
        required_fixes.push("必须绑定 workflow。".to_string());
    }
    if request.node_id.as_deref().unwrap_or("").trim().is_empty() {
        blocked = true;
        reasons.push("missing_node_binding".to_string());
        required_fixes.push("必须绑定 workflow node。".to_string());
    }
    if request.operation_id != "new_session"
        && request
            .session_id
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
    {
        blocked = true;
        reasons.push("missing_session_binding".to_string());
        required_fixes.push("send_message / resume 必须绑定 target session。".to_string());
    }
    if request.operation_id == "new_session"
        && request
            .work_item_id
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
    {
        blocked = true;
        reasons.push("missing_work_item_binding".to_string());
        required_fixes.push("new_session 必须绑定 work item，不能创建自由会话。".to_string());
    }
    if request.operation_id == "new_session"
        && request
            .session_id
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
    {
        warnings.push("new_session_does_not_require_existing_session".to_string());
    }

    if request.prompt_summary.trim().is_empty() {
        blocked = true;
        reasons.push("prompt_summary_missing".to_string());
        required_fixes.push("必须提供用户可理解的 prompt summary，不能展示空预览。".to_string());
    }
    if request.readback_strategy != "required" {
        blocked = true;
        reasons.push("readback_strategy_required".to_string());
        required_fixes
            .push("必须定义 readback strategy，不能把 readback 失败伪装成 0 条结果。".to_string());
        warnings.push("readback_strategy_required".to_string());
    }

    let project_root = request.project_root.as_deref().unwrap_or("");
    let target_cwd = request.target_cwd.as_deref().unwrap_or("");
    if target_cwd.is_empty() {
        blocked = true;
        reasons.push("target_cwd_missing".to_string());
        required_fixes.push("必须提供 target cwd。".to_string());
    } else if sensitive_path_like(target_cwd) {
        blocked = true;
        reasons.push("sensitive_path_blocked:target_cwd".to_string());
        required_fixes.push("target cwd 命中敏感路径，必须更换到项目授权范围。".to_string());
        warnings.push("sensitive_path_blocked".to_string());
    } else if !project_root.is_empty()
        && !path_within_scope(target_cwd, project_root)
        && !request
            .allowed_write_roots
            .iter()
            .any(|root| path_within_scope(target_cwd, root))
    {
        blocked = true;
        reasons.push("cwd_out_of_scope_blocked".to_string());
        required_fixes
            .push("target cwd 必须在 project root 或 allowed write roots 内。".to_string());
        warnings.push("cwd_out_of_scope_blocked".to_string());
    }

    if request
        .allowed_write_roots
        .iter()
        .any(|root| sensitive_path_like(root))
    {
        blocked = true;
        reasons.push("sensitive_path_blocked:allowed_write_roots".to_string());
        required_fixes.push(
            "allowed write roots 不能包含 .codex、.env、auth/token/secret/keychain 等路径。"
                .to_string(),
        );
        warnings.push("sensitive_path_blocked".to_string());
    }

    if let Some(summary) = provider_summary {
        if summary.external_call_status == "external_call_blocked"
            || summary.credential_status == "credential_missing"
            || summary.availability_status == "planned"
        {
            requires_future_task = true;
            reasons.push(format!(
                "provider_availability_requires_future_task:{}",
                summary.adapter_id
            ));
            required_fixes.push("provider availability 只是 guard 输入；planned / credential_missing / external_call_blocked 需要后续任务。".to_string());
            warnings.push("provider_availability_not_execution_authorization".to_string());
        }
    }

    let user_confirmed = request.user_confirmation_state == "confirmed";
    let status = if blocked {
        "blocked"
    } else if requires_future_task {
        "requires_future_task"
    } else if user_confirmed {
        "allowed_preview"
    } else {
        reasons.push("user_confirmation_required_before_execution".to_string());
        required_fixes.push("E5 真实执行前必须经过用户确认；E4 只允许预览。".to_string());
        "needs_user_confirmation"
    };
    let severity = if blocked {
        "high"
    } else if requires_future_task {
        "medium"
    } else if user_confirmed {
        "low"
    } else {
        "medium"
    };

    reasons.sort();
    reasons.dedup();
    required_fixes.sort();
    required_fixes.dedup();
    warnings.sort();
    warnings.dedup();

    SessionContinuationGuardResult {
        status: status.to_string(),
        severity: severity.to_string(),
        blocks_execution: true,
        allows_preview: matches!(status, "allowed_preview" | "needs_user_confirmation"),
        requires_user_confirmation: !user_confirmed && matches!(status, "needs_user_confirmation"),
        reasons,
        required_fixes,
        warnings,
    }
}

fn sensitive_path_like(path: &str) -> bool {
    let normalized = path.to_ascii_lowercase();
    normalized.contains("/.codex")
        || normalized.contains("\\.codex")
        || normalized.ends_with(".codex")
        || normalized.contains("/.ssh")
        || normalized.contains("\\.ssh")
        || normalized.contains(".env")
        || normalized.contains("keychain")
        || normalized.contains("oauth")
        || normalized.contains("provider credential")
        || normalized.contains("token")
        || normalized.contains("secret")
        || normalized.contains("/auth")
        || normalized.contains("\\auth")
}

fn path_within_scope(path: &str, root: &str) -> bool {
    if root.trim().is_empty() {
        return false;
    }
    let path = Path::new(path);
    let root = Path::new(root);
    path == root || path.starts_with(root)
}

fn derive_agent_adapter_descriptors(
    sessions: &[SessionRecord],
    projects: &[ProjectRecord],
    workflow_state: Option<&WorkflowStateSnapshot>,
    workflow_state_error: Option<String>,
) -> Vec<AgentAdapterDescriptor> {
    let codex_sessions = sessions
        .iter()
        .filter(|session| software_key_of_session(session) == "codex")
        .collect::<Vec<_>>();
    let readable_sessions = codex_sessions
        .iter()
        .copied()
        .filter(|session| session.rollout_exists && session.rollout_path.is_some())
        .collect::<Vec<_>>();
    let active_bindings = workflow_state
        .iter()
        .flat_map(|snapshot| snapshot.project_workflows.iter())
        .flat_map(|workflow| workflow.node_session_bindings.iter())
        .filter(|binding| binding.adapter_id == "codex-local" && binding.lifecycle == "active")
        .collect::<Vec<_>>();
    let dispatches = workflow_state
        .iter()
        .flat_map(|snapshot| snapshot.project_workflows.iter())
        .flat_map(|workflow| workflow.node_dispatches.iter())
        .collect::<Vec<_>>();
    let execution_controls = workflow_state
        .iter()
        .flat_map(|snapshot| snapshot.project_workflows.iter())
        .flat_map(|workflow| workflow.execution_controls.iter())
        .collect::<Vec<_>>();
    let permission_requests = workflow_state
        .iter()
        .flat_map(|snapshot| snapshot.project_workflows.iter())
        .flat_map(|workflow| workflow.permission_requests.iter())
        .collect::<Vec<_>>();
    let harness_resources = projects
        .iter()
        .flat_map(|project| project.harness_resources.iter())
        .filter(|resource| resource.adapter_id.as_deref() == Some("codex-local"))
        .collect::<Vec<_>>();
    let workflow_adapter_count = workflow_state
        .map(|snapshot| snapshot.counts.agent_adapters)
        .unwrap_or(0);
    let has_codex_signal = !codex_sessions.is_empty()
        || !active_bindings.is_empty()
        || !harness_resources.is_empty()
        || workflow_adapter_count > 0;

    let mut warnings = vec![
        "adapter_descriptor_is_backend_read_model_only".to_string(),
        "does_not_change_codex_execution_semantics".to_string(),
        "unimplemented_adapters_hidden".to_string(),
    ];
    if let Some(error) = workflow_state_error {
        warnings.push(format!(
            "workflow_state_snapshot_unreadable_for_adapter_descriptor:{error}"
        ));
    } else if workflow_state.is_some_and(|snapshot| !snapshot.exists) {
        warnings.push("workflow_state_snapshot_missing_for_adapter_descriptor".to_string());
    }

    let status = if has_codex_signal {
        "available"
    } else if warnings
        .iter()
        .any(|warning| warning.starts_with("workflow_state_snapshot_unreadable"))
    {
        "degraded"
    } else {
        "not_connected"
    };

    let mut descriptors = vec![AgentAdapterDescriptor {
        adapter_id: "codex-local".to_string(),
        agent_type: "codex".to_string(),
        agent_id: "codex-local".to_string(),
        display_name: "Codex".to_string(),
        provider: "local-codex-index".to_string(),
        status: status.to_string(),
        permission_level: "read_only".to_string(),
        source_kind: "backend_read_model".to_string(),
        capabilities: vec![
            adapter_capability(
                "session_index_read",
                "会话索引读取",
                if codex_sessions.is_empty() {
                    "blocked"
                } else {
                    "read_only"
                },
                &format!("{} 条 Codex 会话索引", codex_sessions.len()),
                "只读取已进入工作台索引的会话元数据，不读取完整 transcript。",
                codex_sessions
                    .iter()
                    .take(3)
                    .map(|session| session.thread_id.clone())
                    .collect(),
                if codex_sessions.is_empty() {
                    vec!["codex_session_index_empty".to_string()]
                } else {
                    vec![]
                },
            ),
            adapter_capability(
                "session_transcript_read",
                "会话正文只读",
                if readable_sessions.is_empty() {
                    "blocked"
                } else {
                    "read_only"
                },
                &format!(
                    "{} 条会话带 rollout，可在用户打开会话时读取",
                    readable_sessions.len()
                ),
                "只读展示会话正文；不发送消息、不 resume、不写 Codex 状态库。",
                readable_sessions
                    .iter()
                    .take(3)
                    .map(|session| session.thread_id.clone())
                    .collect(),
                if readable_sessions.is_empty() {
                    vec!["readable_codex_session_missing".to_string()]
                } else {
                    vec![]
                },
            ),
            adapter_capability(
                "workflow_node_binding",
                "工作流节点绑定",
                if active_bindings.is_empty() {
                    "available"
                } else {
                    "requires_confirmation"
                },
                &format!("{} 个活跃 Codex 节点绑定", active_bindings.len()),
                "只通过工作台确认动作写工作台自己的 workflow state；不启动 Codex。",
                active_bindings
                    .iter()
                    .take(3)
                    .map(|binding| binding.binding_id.clone())
                    .collect(),
                vec![],
            ),
            adapter_capability(
                "safe_probe_dispatch",
                "安全测试派发",
                "requires_confirmation",
                &format!(
                    "{} 条历史 safe probe 派发记录",
                    dispatches
                        .iter()
                        .filter(|dispatch| dispatch.prompt_kind == "safe_probe")
                        .count()
                ),
                "高风险动作；必须用户确认；会执行 codex exec resume。本轮只声明能力，不执行。",
                dispatches
                    .iter()
                    .filter(|dispatch| dispatch.prompt_kind == "safe_probe")
                    .take(3)
                    .map(|dispatch| dispatch.dispatch_id.clone())
                    .collect(),
                vec!["declared_only_not_executed_in_this_slice".to_string()],
            ),
            adapter_capability(
                "user_reviewed_dispatch",
                "用户审核业务派发",
                "requires_confirmation",
                &format!(
                    "{} 条已审核指令记录",
                    execution_controls
                        .iter()
                        .filter(|control| {
                            control
                                .user_reviewed_instruction
                                .as_ref()
                                .is_some_and(|instruction| instruction.approval_state == "reviewed")
                        })
                        .count()
                ),
                "高风险动作；必须用户确认；可能写业务路径。本轮只声明能力，不执行。",
                execution_controls
                    .iter()
                    .take(3)
                    .map(|control| control.control_id.clone())
                    .collect(),
                vec!["declared_only_not_executed_in_this_slice".to_string()],
            ),
            adapter_capability(
                "workflow_machine_run",
                "四角色工作流机器",
                "requires_confirmation",
                "现有路径支持四角色循环，但启动必须用户确认",
                "高风险动作；会调用绑定 Codex 会话。本轮只声明能力，不执行。",
                active_bindings
                    .iter()
                    .take(4)
                    .map(|binding| binding.binding_id.clone())
                    .collect(),
                vec!["declared_only_not_executed_in_this_slice".to_string()],
            ),
            adapter_capability(
                "permission_decision_record",
                "权限结论记录",
                if permission_requests.is_empty() {
                    "available"
                } else {
                    "requires_confirmation"
                },
                &format!("{} 条权限请求记录", permission_requests.len()),
                "只通过控制核心记录权限结论并写工作台 workflow state；不启动 Codex。",
                permission_requests
                    .iter()
                    .take(3)
                    .map(|request| request.request_id.clone())
                    .collect(),
                vec![],
            ),
            adapter_capability(
                "harness_resource_index",
                "Harness 资源索引",
                if harness_resources.is_empty() {
                    "blocked"
                } else {
                    "read_only"
                },
                &format!("{} 个 Codex harness 资源索引", harness_resources.len()),
                "只展示索引字段；不运行 harness，不证明资源可用。",
                harness_resources
                    .iter()
                    .take(3)
                    .map(|resource| resource.root_path.clone())
                    .collect(),
                if harness_resources.is_empty() {
                    vec!["codex_harness_resource_missing".to_string()]
                } else {
                    vec![]
                },
            ),
        ],
        implemented_action_kinds: vec![
            "reveal-rollout".to_string(),
            "bind-node-session".to_string(),
            "unbind-node-session".to_string(),
            "execute-node-dispatch".to_string(),
            "record-permission-decision".to_string(),
            "run-workflow-machine".to_string(),
        ],
        hidden_unimplemented_adapters: vec![
            "claude-code".to_string(),
            "openclaw".to_string(),
            "opencode".to_string(),
            "opencode-like".to_string(),
        ],
        warnings,
        execution_status: if has_codex_signal {
            "available_with_user_confirmation".to_string()
        } else {
            "not_connected".to_string()
        },
        credential_status: "not_read".to_string(),
        model_access_status: "local_read_model_only".to_string(),
        permission_boundary:
            "Codex 高风险动作仍必须用户确认；E1 未执行 codex exec 或 codex exec resume。"
                .to_string(),
        unavailable_reason: if has_codex_signal {
            None
        } else {
            Some("codex_signal_missing".to_string())
        },
        requires_user_setup: !has_codex_signal,
    }];
    descriptors.extend(planned_agent_adapter_descriptors("backend_read_model"));
    descriptors
}

fn planned_agent_adapter_descriptors(source_kind: &str) -> Vec<AgentAdapterDescriptor> {
    [
        ("claude-code", "Claude Code", "anthropic-cli-planned"),
        ("openclaw", "OpenClaw", "openclaw-planned"),
        ("opencode", "OpenCode", "opencode-planned"),
        (
            "opencode-like",
            "OpenCode-like",
            "opencode-compatible-planned",
        ),
    ]
    .into_iter()
    .map(|(adapter_id, display_name, provider)| {
        planned_agent_adapter_descriptor(adapter_id, display_name, provider, source_kind)
    })
    .collect()
}

fn planned_agent_adapter_descriptor(
    adapter_id: &str,
    display_name: &str,
    provider: &str,
    source_kind: &str,
) -> AgentAdapterDescriptor {
    AgentAdapterDescriptor {
        adapter_id: adapter_id.to_string(),
        agent_type: adapter_id.to_string(),
        agent_id: adapter_id.to_string(),
        display_name: display_name.to_string(),
        provider: provider.to_string(),
        status: "planned".to_string(),
        permission_level: "read_only".to_string(),
        source_kind: source_kind.to_string(),
        capabilities: Vec::new(),
        implemented_action_kinds: Vec::new(),
        hidden_unimplemented_adapters: Vec::new(),
        warnings: vec![
            "adapter_descriptor_is_read_model_only".to_string(),
            "planned_adapter_not_connected".to_string(),
            "no_execution_button".to_string(),
            "credential_not_configured".to_string(),
            "model_access_not_verified".to_string(),
        ],
        execution_status: "not_implemented".to_string(),
        credential_status: "not_configured".to_string(),
        model_access_status: "not_verified".to_string(),
        permission_boundary:
            "计划中的 adapter 只有只读 descriptor；没有真实命令、会话、凭据或模型调用。".to_string(),
        unavailable_reason: Some(
            "planned_adapter_descriptor_only_no_runtime_connection".to_string(),
        ),
        requires_user_setup: true,
    }
}

fn adapter_capability(
    kind: &str,
    label: &str,
    status: &str,
    description: &str,
    boundary: &str,
    evidence_refs: Vec<String>,
    warnings: Vec<String>,
) -> AdapterCapability {
    AdapterCapability {
        capability_id: format!("codex-local:{kind}"),
        kind: kind.to_string(),
        label: label.to_string(),
        status: status.to_string(),
        description: description.to_string(),
        boundary: boundary.to_string(),
        evidence_refs,
        warnings,
    }
}

#[derive(Clone, Copy)]
struct SessionOperationSpec {
    operation_id: &'static str,
    label: &'static str,
    category: &'static str,
    codex_status: &'static str,
    risk_level: &'static str,
    applies_to_session_state: &'static str,
    requires_user_confirmation: bool,
    writes_codex_home: bool,
    writes_workbench_state: bool,
    writes_project_files: bool,
    reads_full_transcript: bool,
    requires_model_access: bool,
    requires_runtime_handle: bool,
    audit_requirement: &'static str,
    unavailable_reason: &'static str,
    future_task_hint: &'static str,
    warnings: &'static [&'static str],
}

fn derive_session_operation_descriptors(
    adapters: &[AgentAdapterDescriptor],
) -> Vec<SessionOperationDescriptor> {
    adapters
        .iter()
        .flat_map(|adapter| {
            session_operation_specs()
                .iter()
                .map(|spec| session_operation_descriptor_for_adapter(adapter, spec))
                .collect::<Vec<_>>()
        })
        .collect()
}

fn session_operation_specs() -> Vec<SessionOperationSpec> {
    vec![
        SessionOperationSpec {
            operation_id: "new_session",
            label: "新会话预览",
            category: "interactive_control",
            codex_status: "requires_future_task",
            risk_level: "high",
            applies_to_session_state: "work_item_without_native_session",
            requires_user_confirmation: true,
            writes_codex_home: true,
            writes_workbench_state: true,
            writes_project_files: false,
            reads_full_transcript: false,
            requires_model_access: true,
            requires_runtime_handle: false,
            audit_requirement:
                "必须绑定 work item、prompt 预览、权限信封、结构化 command plan、attempt、readback 和失败审计。",
            unavailable_reason:
                "H3.1 只实现新会话 request / guard / permission envelope / no-op runner；真实 codex exec 新会话未授权。",
            future_task_hint:
                "H3-B 需单独冻结 fixture、权限信封、真实执行范围、readback 和 /Users/yoyi/.codex 读写授权。",
            warnings: &[
                "h3_1_new_session_noop_only",
                "requires_work_item_binding",
                "requires_future_authorization_task",
                "no_real_new_session_in_h3_1",
                "no_session_operation_execution_in_e2",
            ],
        },
        SessionOperationSpec {
            operation_id: "send_message",
            label: "发消息",
            category: "interactive_control",
            codex_status: "requires_future_task",
            risk_level: "high",
            applies_to_session_state: "existing_readonly_session",
            requires_user_confirmation: true,
            writes_codex_home: true,
            writes_workbench_state: true,
            writes_project_files: false,
            reads_full_transcript: false,
            requires_model_access: true,
            requires_runtime_handle: false,
            audit_requirement:
                "必须定义 prompt 预览、用户确认、执行记录、readback 和失败处理审计。",
            unavailable_reason:
                "会话中心仍是只读历史浏览器；发送路径、权限和 readback 尚未单独定义。",
            future_task_hint:
                "E3 或后续任务需定义 adapter runner、用户确认、审计、写入范围和失败恢复。",
            warnings: &[
                "requires_future_authorization_task",
                "no_session_operation_execution_in_e2",
            ],
        },
        SessionOperationSpec {
            operation_id: "stop",
            label: "停止",
            category: "runtime_control",
            codex_status: "blocked",
            risk_level: "high",
            applies_to_session_state: "running_session_only",
            requires_user_confirmation: true,
            writes_codex_home: false,
            writes_workbench_state: true,
            writes_project_files: false,
            reads_full_transcript: false,
            requires_model_access: false,
            requires_runtime_handle: true,
            audit_requirement: "必须有运行句柄、取消协议、幂等记录、超时和失败恢复审计。",
            unavailable_reason: "当前缺少运行进程 registry、运行句柄和取消协议。",
            future_task_hint: "后续任务需先建立运行句柄、取消协议、运行日志和失败恢复模型。",
            warnings: &[
                "runtime_handle_missing",
                "no_session_operation_execution_in_e2",
            ],
        },
        SessionOperationSpec {
            operation_id: "restart",
            label: "重启",
            category: "runtime_control",
            codex_status: "blocked",
            risk_level: "high",
            applies_to_session_state: "existing_or_running_session",
            requires_user_confirmation: true,
            writes_codex_home: true,
            writes_workbench_state: true,
            writes_project_files: false,
            reads_full_transcript: false,
            requires_model_access: true,
            requires_runtime_handle: true,
            audit_requirement: "必须先定义 restart 语义、上下文来源、成本提示、运行日志和审计。",
            unavailable_reason: "restart 语义未定：新建会话、恢复旧会话或重跑任务尚未决策。",
            future_task_hint: "后续任务需明确 restart 语义、上下文来源、权限、日志和成本提示。",
            warnings: &[
                "restart_semantics_not_defined",
                "no_session_operation_execution_in_e2",
            ],
        },
        SessionOperationSpec {
            operation_id: "resume",
            label: "resume",
            category: "interactive_control",
            codex_status: "requires_future_task",
            risk_level: "high",
            applies_to_session_state: "bound_or_existing_session",
            requires_user_confirmation: true,
            writes_codex_home: true,
            writes_workbench_state: true,
            writes_project_files: false,
            reads_full_transcript: false,
            requires_model_access: true,
            requires_runtime_handle: false,
            audit_requirement:
                "必须绑定会话校验、prompt 预览、权限、超时、运行日志和 readback 审计。",
            unavailable_reason:
                "workflow dispatch 的受控 resume 属于项目工作流语境，不等于会话中心通用 resume。",
            future_task_hint:
                "后续任务需决定是否复用 workflow dispatch 或建立单独 session adapter runner。",
            warnings: &[
                "workflow_dispatch_is_not_session_center_resume",
                "requires_future_authorization_task",
                "no_session_operation_execution_in_e2",
            ],
        },
        SessionOperationSpec {
            operation_id: "export",
            label: "导出",
            category: "data_effect",
            codex_status: "planned",
            risk_level: "medium",
            applies_to_session_state: "readable_session",
            requires_user_confirmation: true,
            writes_codex_home: false,
            writes_workbench_state: false,
            writes_project_files: true,
            reads_full_transcript: true,
            requires_model_access: false,
            requires_runtime_handle: false,
            audit_requirement: "必须有导出范围、脱敏策略、目标位置、用户确认和审计。",
            unavailable_reason: "导出格式、脱敏范围和文件写入位置尚未定义。",
            future_task_hint: "后续任务需定义 Markdown/JSON/证据包格式、脱敏和文件写入位置。",
            warnings: &[
                "export_redaction_policy_missing",
                "no_session_operation_execution_in_e2",
            ],
        },
        SessionOperationSpec {
            operation_id: "delete",
            label: "删除",
            category: "destructive_data_effect",
            codex_status: "blocked_destructive",
            risk_level: "destructive",
            applies_to_session_state: "existing_session_or_native_store",
            requires_user_confirmation: true,
            writes_codex_home: true,
            writes_workbench_state: true,
            writes_project_files: false,
            reads_full_transcript: false,
            requires_model_access: false,
            requires_runtime_handle: false,
            audit_requirement: "必须有备份、回滚、双确认、作用域、原生系统兼容和审计。",
            unavailable_reason: "破坏性操作已阻断；本阶段不删除、不移动、不归档原生会话。",
            future_task_hint: "后续任务需单独设计备份、回滚、双确认、审计和原生系统兼容。",
            warnings: &[
                "destructive_operation_blocked",
                "no_session_operation_execution_in_e2",
            ],
        },
        SessionOperationSpec {
            operation_id: "favorite",
            label: "收藏",
            category: "metadata_effect",
            codex_status: "planned",
            risk_level: "low",
            applies_to_session_state: "existing_session",
            requires_user_confirmation: false,
            writes_codex_home: false,
            writes_workbench_state: true,
            writes_project_files: false,
            reads_full_transcript: false,
            requires_model_access: false,
            requires_runtime_handle: false,
            audit_requirement: "必须有工作台自有 metadata store、冲突策略和轻量审计。",
            unavailable_reason: "工作台自有 favorite metadata store 尚未实现。",
            future_task_hint: "后续任务需定义 metadata store、冲突策略、导入导出和审计。",
            warnings: &[
                "favorite_metadata_store_missing",
                "no_session_operation_execution_in_e2",
            ],
        },
    ]
}

fn session_operation_descriptor_for_adapter(
    adapter: &AgentAdapterDescriptor,
    spec: &SessionOperationSpec,
) -> SessionOperationDescriptor {
    let is_codex = adapter.adapter_id == "codex-local";
    let planned_adapter =
        adapter.status == "planned" || adapter.execution_status == "not_implemented";
    let current_status = if is_codex {
        spec.codex_status
    } else if spec.codex_status == "blocked_destructive" {
        "blocked_destructive"
    } else if matches!(spec.operation_id, "export" | "favorite") {
        "planned"
    } else {
        "blocked"
    };
    let mut warnings = vec![
        "session_operation_boundary_read_model_only".to_string(),
        "no_session_operation_execution_in_e2".to_string(),
        "no_codex_home_write_in_e2".to_string(),
    ];
    warnings.extend(spec.warnings.iter().map(|warning| (*warning).to_string()));
    if planned_adapter {
        warnings.push("planned_adapter_operation_not_available".to_string());
    }

    let unavailable_reason = if planned_adapter {
        format!(
            "{}；{} 仍只是 planned descriptor，没有真实命令、会话、凭据或模型访问。",
            spec.unavailable_reason, adapter.display_name
        )
    } else {
        spec.unavailable_reason.to_string()
    };
    let future_task_hint = if planned_adapter {
        format!(
            "{}；必须先完成 {} adapter 真实接入设计和凭据 / 模型只读边界确认。",
            spec.future_task_hint, adapter.display_name
        )
    } else {
        spec.future_task_hint.to_string()
    };

    SessionOperationDescriptor {
        operation_id: spec.operation_id.to_string(),
        label: spec.label.to_string(),
        category: spec.category.to_string(),
        current_status: current_status.to_string(),
        risk_level: spec.risk_level.to_string(),
        adapter_id: adapter.adapter_id.clone(),
        agent_type: adapter.agent_type.clone(),
        applies_to_session_state: if planned_adapter {
            "planned_adapter_without_session_source".to_string()
        } else {
            spec.applies_to_session_state.to_string()
        },
        requires_user_confirmation: spec.requires_user_confirmation,
        writes_codex_home: spec.writes_codex_home,
        writes_workbench_state: spec.writes_workbench_state,
        writes_project_files: spec.writes_project_files,
        reads_full_transcript: spec.reads_full_transcript,
        requires_credential: planned_adapter && spec.operation_id != "favorite",
        requires_model_access: spec.requires_model_access || planned_adapter,
        requires_runtime_handle: spec.requires_runtime_handle,
        audit_requirement: spec.audit_requirement.to_string(),
        unavailable_reason,
        future_task_hint,
        warnings,
    }
}
