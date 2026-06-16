use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct PageReadModelContract {
    pub(crate) page_id: String,
    pub(crate) page_label: String,
    pub(crate) user_facing_data: Vec<String>,
    pub(crate) developer_internal_data: Vec<String>,
    pub(crate) must_not_show_as_primary: Vec<String>,
    pub(crate) current_source: String,
    pub(crate) planned_read_model: String,
    pub(crate) migration_status: String,
    pub(crate) next_step: String,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct WorkbenchPageReadModelInventory {
    pub(crate) schema_version: String,
    pub(crate) generated_at: String,
    pub(crate) status: String,
    pub(crate) source_policy: String,
    pub(crate) contracts: Vec<PageReadModelContract>,
    pub(crate) warnings: Vec<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct PageReadModelQueryInput {
    pub(crate) page_id: String,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct PageReadModelSchemaContract {
    pub(crate) page_id: String,
    pub(crate) page_label: String,
    pub(crate) read_model_type: String,
    pub(crate) schema_version: String,
    pub(crate) snapshot_fields: Vec<String>,
    pub(crate) workflow_state_fields: Vec<String>,
    pub(crate) external_store_inputs: Vec<String>,
    pub(crate) output_sections: Vec<String>,
    pub(crate) migration_status: String,
    pub(crate) workbench_snapshot_active: bool,
    pub(crate) returns_business_data: bool,
    pub(crate) page_ui_migrated: bool,
    pub(crate) next_step: String,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct PageSnapshotFieldCoverage {
    pub(crate) field_name: String,
    pub(crate) covered_by_pages: Vec<String>,
    pub(crate) coverage_status: String,
    pub(crate) notes: String,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct WorkbenchPageReadModelSchemaCatalog {
    pub(crate) schema_version: String,
    pub(crate) generated_at: String,
    pub(crate) status: String,
    pub(crate) target_pages: Vec<String>,
    pub(crate) schemas: Vec<PageReadModelSchemaContract>,
    pub(crate) snapshot_field_coverage: Vec<PageSnapshotFieldCoverage>,
    pub(crate) uncovered_snapshot_fields: Vec<String>,
    pub(crate) warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct PageReadModelPayload {
    pub(crate) page_id: String,
    pub(crate) schema_version: String,
    pub(crate) generated_from: String,
    pub(crate) data: Value,
    pub(crate) warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct PageReadModelSelectorPlan {
    pub(crate) selector_id: String,
    pub(crate) selector_kind: String,
    pub(crate) planned_read_model: String,
    pub(crate) data_migration_status: String,
    pub(crate) ui_consumption_status: String,
    pub(crate) next_step: String,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct PageReadModelSourceBoundary {
    pub(crate) current_source: String,
    pub(crate) workbench_snapshot_active: bool,
    pub(crate) returns_business_data: bool,
    pub(crate) writes_stores: bool,
    pub(crate) tauri_command_migrates_page: bool,
    pub(crate) warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct PageReadModelQueryResult {
    pub(crate) schema_version: String,
    pub(crate) generated_at: String,
    pub(crate) status: String,
    pub(crate) requested_page_id: String,
    pub(crate) page_label: String,
    pub(crate) contract: PageReadModelContract,
    pub(crate) target_schema: Option<PageReadModelSchemaContract>,
    pub(crate) snapshot_field_coverage: Vec<PageSnapshotFieldCoverage>,
    pub(crate) uncovered_snapshot_fields: Vec<String>,
    pub(crate) page_payload: Option<PageReadModelPayload>,
    pub(crate) selector_plan: PageReadModelSelectorPlan,
    pub(crate) source_boundary: PageReadModelSourceBoundary,
    pub(crate) warnings: Vec<String>,
}

pub(crate) fn derive_page_read_model_inventory(
    generated_at: &str,
) -> WorkbenchPageReadModelInventory {
    WorkbenchPageReadModelInventory {
        schema_version: "workbench_page_read_model_inventory.v1".to_string(),
        generated_at: generated_at.to_string(),
        status: "contract_only".to_string(),
        source_policy:
            "R4-A1 only records page data contracts; pages still read the existing WorkbenchSnapshot."
                .to_string(),
        contracts: vec![
            contract("home", "首页", &["主对象入口", "运行中摘要", "待处理摘要", "索引状态"], &["snapshot source", "diagnostics refs"], &["raw sidecar", "full audit path", "schema dump"], "HomePageReadModel", "R4-A2+ can introduce a page query or frontend selector without changing layout."),
            contract("projects", "项目", &["项目列表", "项目详情", "工作流画布摘要", "任务包状态", "节点详情摘要"], &["audit/evidence refs", "dispatch/readback diagnostics"], &["raw transcript", "完整 task package 文本", "内部 schema"], "ProjectsPageReadModel", "Split project selectors before moving data access away from the full snapshot."),
            contract("agents", "智能体", &["项目选择", "会话选择", "对话流", "输入/执行 readiness"], &["adapter descriptors", "operation/provider/session boundary"], &["控制中心式全量边界面板", "未实现执行按钮"], "AgentsPageReadModel", "Keep normal UI conversation-first; move boundary data behind developer details."),
            contract("running_workflows", "运行中工作流", &["运行队列", "待确认", "失败/阻断", "readback 状态"], &["runtime refs", "diagnostic refs"], &["raw runtime log", "internal ids 默认铺开"], "RunningWorkflowsPageReadModel", "Prepare a run queue selector that preserves result_count=null as unknown."),
            contract("memory", "记忆层", &["正式记忆", "候选", "观察", "lint", "任务记忆包摘要"], &["revision", "audit refs", "sidecar health"], &["candidate/observation 冒充正式记忆"], "MemoryPageReadModel", "Separate formal memory, candidates, and observations before UI slimming."),
            contract("knowledge", "知识库", &["资料", "笔记", "引用", "关联记忆", "候选入口"], &["index diagnostics", "source refs"], &["知识命中冒充正式记忆"], "KnowledgePageReadModel", "Keep knowledge hits distinct from memory records in the future read model."),
            contract("settings", "设置", &["普通设置", "开发者入口", "系统健康"], &["diagnostics", "developer nav", "data locations"], &["把开发/内部入口放在主导航"], "SettingsPageReadModel", "Settings can host read-model inventory while remaining non-executing."),
            contract("skill", "Skill", &["可复用能力", "适用场景", "可用性", "最近使用"], &["plugin metadata", "字段缺口"], &["首屏字段/schema 堆叠"], "SkillPageReadModel", "Preserve object-first wording and keep field gaps in developer details."),
            contract("harness", "Harness", &["运行器能力", "可运行范围", "最近运行", "配置状态"], &["adapter/resource fields"], &["首屏候选资源/raw config"], "HarnessPageReadModel", "Preserve runner terminology while hiding raw resource details by default."),
        ],
        warnings: vec![
            "r4_a1_contract_only_no_page_query".to_string(),
            "workbench_snapshot_still_active".to_string(),
            "no_visual_redesign_no_layout_change".to_string(),
        ],
    }
}

pub(crate) fn derive_page_read_model_schema_catalog(
    generated_at: &str,
) -> WorkbenchPageReadModelSchemaCatalog {
    let schemas = vec![
        schema(
            "projects",
            "项目",
            "ProjectsPageReadModel",
            "projects_page_read_model.v1",
            &[
                "summary",
                "projects",
                "sessions",
                "tasks",
                "project_workflow_automation",
                "k3_b1_recovery",
            ],
            &["project_workflows"],
            &["workflow-state.v0.json"],
            &[
                "project list",
                "project detail summary",
                "workflow summary counts",
                "task package status",
            ],
            "H2-2 should return projects page data through query_workbench_page_read_model.",
        ),
        schema(
            "agents",
            "智能体",
            "AgentsPageReadModel",
            "agents_page_read_model.v1",
            &[
                "projects",
                "sessions",
                "agent_adapters",
                "session_operations",
                "provider_availability",
                "session_continuation_previews",
                "session_continuation_store",
                "runtime_session_attention",
                "session_run_status_summaries",
                "worker_protocol",
                "real_execution_product_commands",
                "k3_b1_recovery",
                "operation_control",
            ],
            &["project_workflows", "session_bindings"],
            &["session-continuations.v1.json", "real-execution-product-commands.v1.json"],
            &[
                "project picker",
                "session picker",
                "conversation readiness",
                "collapsed developer boundary",
            ],
            "H2-2 should keep the agents page conversation-first while exposing boundary data as page data.",
        ),
        schema(
            "running_workflows",
            "运行中工作流",
            "RunningWorkflowsPageReadModel",
            "running_workflows_page_read_model.v1",
            &[
                "summary",
                "sessions",
                "session_continuation_previews",
                "session_continuation_store",
                "runtime_session_attention",
                "session_run_status_summaries",
                "runtime_log_store",
                "worker_protocol",
                "real_execution_product_commands",
                "project_workflow_automation",
                "k3_b1_recovery",
                "operation_control",
                "diagnostic_summary",
            ],
            &["project_workflows", "task_drafts", "recent_execution_attempts"],
            &[
                "runtime-log.v1.json",
                "session-continuations.v1.json",
                "real-execution-product-commands.v1.json",
            ],
            &[
                "run queue",
                "permission queue",
                "failure and readback summary",
                "operation control summary",
            ],
            "H2-2 should preserve unknown readback result_count as null in page data.",
        ),
        schema(
            "memory",
            "记忆层",
            "MemoryCenterPageReadModel",
            "memory_center_page_read_model.v1",
            &["projects", "tasks"],
            &["project_workflows", "task_packages", "memory_injection_summary"],
            &[
                "formal-memories.v1.json",
                "memory-candidates.v1.json",
                "observations.v1.json",
                "memory-lint.v1.json",
                "memory-patterns.v1.json",
            ],
            &[
                "formal memory summary",
                "candidate memory summary",
                "observation summary",
                "lint and maintenance summary",
                "task memory packet summary",
            ],
            "H2-2 should keep candidate and observation data distinct from formal memory.",
        ),
        schema(
            "knowledge",
            "知识库",
            "KnowledgeBasePageReadModel",
            "knowledge_base_page_read_model.v1",
            &["projects", "tasks"],
            &["project_workflows", "task_package_references"],
            &[
                "formal-memories.v1.json",
                "memory-capture-events.v1.json",
                "memory-candidates.v1.json",
            ],
            &[
                "document summary",
                "formal memory links",
                "candidate links",
                "task references",
                "Obsidian-compatible boundary",
            ],
            "H2-2 should keep knowledge hits from being displayed as formal memories.",
        ),
        schema(
            "settings",
            "设置",
            "SettingsPageReadModel",
            "settings_page_read_model.v1",
            &[
                "summary",
                "skills",
                "plugins",
                "tasks",
                "agent_adapters",
                "session_operations",
                "provider_availability",
                "runtime_log_store",
                "page_read_model_inventory",
                "diagnostic_summary",
                "diagnostics",
            ],
            &["counts", "workflow_state_error"],
            &["runtime-log.v1.json"],
            &[
                "general settings summary",
                "developer entry summary",
                "system health",
                "page contract inventory",
            ],
            "H2-2 should keep settings non-executing and credential-free.",
        ),
    ];
    let snapshot_field_coverage = derive_snapshot_field_coverage();
    let uncovered_snapshot_fields = uncovered_snapshot_fields(&snapshot_field_coverage);

    WorkbenchPageReadModelSchemaCatalog {
        schema_version: "workbench_page_read_model_schema_catalog.v1".to_string(),
        generated_at: generated_at.to_string(),
        status: "schema_defined_no_query_migration".to_string(),
        target_pages: schemas
            .iter()
            .map(|schema| schema.page_id.clone())
            .collect(),
        schemas,
        snapshot_field_coverage,
        uncovered_snapshot_fields,
        warnings: vec![
            "h2_1_schema_only_no_business_page_payload".to_string(),
            "workbench_snapshot_still_active".to_string(),
            "page_ui_not_migrated".to_string(),
        ],
    }
}

pub(crate) fn query_page_read_model(
    input: &PageReadModelQueryInput,
    generated_at: &str,
) -> Result<PageReadModelQueryResult, String> {
    let page_id = input.page_id.trim();
    if page_id.is_empty() {
        return Err("page_id_required".to_string());
    }

    let inventory = derive_page_read_model_inventory(generated_at);
    let contract = inventory
        .contracts
        .into_iter()
        .find(|contract| contract.page_id == page_id)
        .ok_or_else(|| format!("unknown_page_id:{page_id}"))?;
    let schema_catalog = derive_page_read_model_schema_catalog(generated_at);
    let target_schema = schema_catalog
        .schemas
        .iter()
        .find(|schema| schema.page_id == page_id)
        .cloned();

    Ok(PageReadModelQueryResult {
        schema_version: "workbench_page_read_model_query.v1".to_string(),
        generated_at: generated_at.to_string(),
        status: "selector_contract_only".to_string(),
        requested_page_id: page_id.to_string(),
        page_label: contract.page_label.clone(),
        target_schema,
        snapshot_field_coverage: schema_catalog.snapshot_field_coverage,
        uncovered_snapshot_fields: schema_catalog.uncovered_snapshot_fields,
        page_payload: None,
        selector_plan: PageReadModelSelectorPlan {
            selector_id: format!("{}_selector_contract", contract.page_id),
            selector_kind: "page_read_model_selector_contract".to_string(),
            planned_read_model: contract.planned_read_model.clone(),
            data_migration_status: "not_migrated".to_string(),
            ui_consumption_status: "not_connected_to_pages".to_string(),
            next_step: contract.next_step.clone(),
        },
        source_boundary: PageReadModelSourceBoundary {
            current_source: contract.current_source.clone(),
            workbench_snapshot_active: true,
            returns_business_data: false,
            writes_stores: false,
            tauri_command_migrates_page: false,
            warnings: vec![
                "r4_a2_selector_contract_only_no_business_data".to_string(),
                "workbench_snapshot_still_active".to_string(),
                "page_ui_not_migrated".to_string(),
            ],
        },
        contract,
        warnings: vec![
            "h2_1_schema_defined_no_query_migration".to_string(),
            "r4_a2_skeleton_no_page_data_query".to_string(),
            "workbench_snapshot_still_active".to_string(),
            "do_not_claim_workbench_snapshot_deprecated".to_string(),
        ],
    })
}

pub(crate) fn query_page_read_model_with_snapshot_value(
    input: &PageReadModelQueryInput,
    generated_at: &str,
    snapshot: &Value,
    workflow_state: Option<&Value>,
) -> Result<PageReadModelQueryResult, String> {
    let mut output = query_page_read_model(input, generated_at)?;
    let Some(schema) = output.target_schema.clone() else {
        return Ok(output);
    };
    let Some(payload) = derive_page_payload(&schema, generated_at, snapshot, workflow_state) else {
        return Ok(output);
    };

    output.status = "page_data_ready".to_string();
    output.target_schema = output.target_schema.map(|mut schema| {
        schema.migration_status = "page_query_payload_ready".to_string();
        schema.returns_business_data = true;
        schema.page_ui_migrated = true;
        schema
    });
    output.selector_plan.data_migration_status = "backend_page_query_ready".to_string();
    output.selector_plan.ui_consumption_status = "page_query_payload_ready".to_string();
    output.source_boundary.returns_business_data = true;
    output.source_boundary.warnings = vec![
        "h2_2_backend_page_payload_ready".to_string(),
        "h2_3_snapshot_slice_available".to_string(),
        "writes_stores_false".to_string(),
    ];
    output.page_payload = Some(payload);
    output.warnings = vec![
        "h2_2_backend_page_payload_ready".to_string(),
        "h2_3_page_query_payload_supports_frontend_consumption".to_string(),
        "do_not_claim_workbench_snapshot_deprecated".to_string(),
    ];
    Ok(output)
}

fn derive_page_payload(
    schema: &PageReadModelSchemaContract,
    _generated_at: &str,
    snapshot: &Value,
    workflow_state: Option<&Value>,
) -> Option<PageReadModelPayload> {
    let mut data = match schema.page_id.as_str() {
        "projects" => projects_payload(snapshot, workflow_state),
        "agents" => agents_payload(snapshot),
        "running_workflows" => running_workflows_payload(snapshot, workflow_state),
        "memory" => memory_payload(snapshot),
        "knowledge" => knowledge_payload(snapshot),
        "settings" => settings_payload(snapshot),
        _ => return None,
    };
    if let Value::Object(data) = &mut data {
        data.insert(
            "snapshot_slice".to_string(),
            snapshot_slice_for_schema(schema, snapshot),
        );
    }

    Some(PageReadModelPayload {
        page_id: schema.page_id.clone(),
        schema_version: schema.schema_version.clone(),
        generated_from: "workbench_page_query".to_string(),
        data,
        warnings: vec![
            "backend_page_payload_read_only".to_string(),
            "snapshot_slice_read_only".to_string(),
        ],
    })
}

fn snapshot_slice_for_schema(schema: &PageReadModelSchemaContract, snapshot: &Value) -> Value {
    let mut slice = Map::new();
    for field_name in &schema.snapshot_fields {
        slice.insert(
            field_name.clone(),
            snapshot.get(field_name).cloned().unwrap_or(Value::Null),
        );
    }
    Value::Object(slice)
}

fn projects_payload(snapshot: &Value, workflow_state: Option<&Value>) -> Value {
    let sessions_by_project = count_by_optional_string(items(snapshot, "sessions"), "project_root");
    let workflows_by_project = count_by_optional_string(
        items_opt(workflow_state, "project_workflows"),
        "project_root",
    );
    let projects: Vec<Value> = items(snapshot, "projects")
        .into_iter()
        .map(|project| {
            let root = string_field(project, "project_root");
            let session_count = sessions_by_project
                .get(&root)
                .copied()
                .unwrap_or_else(|| usize_field(project, "thread_count"));
            json!({
                "project_root": root,
                "name": string_field(project, "name"),
                "active_hint": bool_field(project, "active_hint"),
                "session_count": session_count,
                "active_session_count": usize_field(project, "active_thread_count"),
                "archived_session_count": usize_field(project, "archived_thread_count"),
                "workflow_count": workflows_by_project.get(&root).copied().unwrap_or(0),
                "evidence_count": array_field_len(project, "evidence_files"),
                "handoff_count": array_field_len(project, "handoff_files"),
                "authority_count": array_field_len(project, "authority_files"),
                "warning_count": array_field_len(project, "context_warnings") + array_field_len(project, "warnings"),
                "latest_updated_at_ms": project.get("latest_updated_at_ms").cloned().unwrap_or(Value::Null),
            })
        })
        .collect();

    json!({
        "schema_version": "projects_page_read_model.v1",
        "project_count": projects.len(),
        "active_project_count": projects.iter().filter(|project| project.get("active_hint").and_then(Value::as_bool).unwrap_or(false)).count(),
        "total_session_count": array_len(snapshot, "sessions"),
        "workflow_summary_count": items_opt(workflow_state, "project_workflows").len(),
        "projects": projects,
        "user_facing_summary": format!(
            "{} 个项目，{} 个会话，{} 条工作流摘要",
            array_len(snapshot, "projects"),
            array_len(snapshot, "sessions"),
            items_opt(workflow_state, "project_workflows").len()
        ),
        "developer_details_collapsed": true,
    })
}

fn agents_payload(snapshot: &Value) -> Value {
    let sessions = items(snapshot, "sessions");
    let readable_count = sessions
        .iter()
        .filter(|session| bool_field(session, "rollout_exists"))
        .count();
    let archived_count = sessions
        .iter()
        .filter(|session| bool_field(session, "archived"))
        .count();
    let project_options = agent_project_options(snapshot);
    let adapter_count = array_len(snapshot, "agent_adapters");
    let available_adapter_count = items(snapshot, "agent_adapters")
        .iter()
        .filter(|adapter| string_field(adapter, "status") == "available")
        .count();
    let planned_adapter_count = items(snapshot, "agent_adapters")
        .iter()
        .filter(|adapter| string_field(adapter, "status") == "planned")
        .count();

    json!({
        "schema_version": "agents_page_read_model.v1",
        "project_options": project_options,
        "session_summary": {
            "readable_count": readable_count,
            "missing_rollout_count": sessions.len().saturating_sub(readable_count),
            "archived_count": archived_count,
            "total_count": sessions.len(),
        },
        "adapter_count": adapter_count,
        "available_adapter_count": available_adapter_count,
        "planned_adapter_count": planned_adapter_count,
        "operation_boundary_count": array_len(snapshot, "session_operations"),
        "provider_boundary_count": array_len(snapshot, "provider_availability"),
        "continuation_preview_count": array_len(snapshot, "session_continuation_previews"),
        "worker_thread_count": array_field_len(snapshot.get("worker_protocol").unwrap_or(&Value::Null), "work_threads"),
        "conversation_first": true,
        "developer_details_collapsed": true,
        "user_facing_summary": format!(
            "{} 个会话，{} 个可读取，{} 个可用 adapter",
            sessions.len(),
            readable_count,
            available_adapter_count
        ),
    })
}

fn running_workflows_payload(snapshot: &Value, workflow_state: Option<&Value>) -> Value {
    let workflows = items_opt(workflow_state, "project_workflows");
    let workflow_focus_count = workflows
        .iter()
        .filter(|workflow| {
            matches!(
                string_field(workflow, "state").as_str(),
                "running" | "prepared" | "waiting_for_permission" | "blocked" | "failed"
            )
        })
        .count();
    let waiting_permission_count: usize = workflows
        .iter()
        .map(|workflow| {
            items_from_value(workflow, "task_drafts")
                .iter()
                .filter(|task| string_field(task, "state") == "waiting_for_permission")
                .count()
        })
        .sum();
    let runtime_attention = items(snapshot, "runtime_session_attention");
    let runtime_attention_count = runtime_attention
        .iter()
        .filter(|item| {
            bool_field(item, "requires_user_action")
                || bool_field(item, "blocks_continuation")
                || matches!(
                    string_field(item, "status").as_str(),
                    "running"
                        | "waiting_permission"
                        | "blocked_by_guard"
                        | "readback_unavailable"
                        | "readback_failed"
                )
        })
        .count();
    let readback_issue_count = runtime_attention
        .iter()
        .filter(|item| {
            let status = item
                .get("readback_boundary")
                .map(|boundary| string_field(boundary, "status"))
                .unwrap_or_default();
            status == "readback_unavailable" || status == "readback_failed"
        })
        .count();
    let product_commands = snapshot
        .get("real_execution_product_commands")
        .unwrap_or(&Value::Null);
    let automation = snapshot
        .get("project_workflow_automation")
        .unwrap_or(&Value::Null);
    let recovery = snapshot.get("k3_b1_recovery").unwrap_or(&Value::Null);
    let operation_control = snapshot.get("operation_control").unwrap_or(&Value::Null);

    json!({
        "schema_version": "running_workflows_page_read_model.v1",
        "workflow_count": workflows.len(),
        "workflow_focus_count": workflow_focus_count,
        "running_attention_count": workflow_focus_count + runtime_attention_count,
        "runtime_attention_count": runtime_attention_count,
        "waiting_permission_count": waiting_permission_count,
        "readback_issue_count": readback_issue_count,
        "session_run_status_count": array_len(snapshot, "session_run_status_summaries"),
        "runtime_log": {
            "entry_count": array_field_len(snapshot.get("runtime_log_store").unwrap_or(&Value::Null), "entries"),
            "summary_count": array_field_len(snapshot.get("runtime_log_store").unwrap_or(&Value::Null), "summaries"),
        },
        "product_command": {
            "command_count": usize_field(product_commands, "command_count"),
            "pending_decision_count": usize_field(product_commands, "pending_decision_count"),
            "blocked_attempt_count": usize_field(product_commands, "blocked_attempt_count"),
            "running_attempt_count": usize_field(product_commands, "running_attempt_count"),
        },
        "automation": {
            "run_unit_count": usize_field(automation, "run_unit_count"),
            "waiting_user_count": usize_field(automation, "waiting_user_count"),
            "blocked_count": usize_field(automation, "blocked_count"),
            "readback_unknown_count": usize_field(automation, "readback_unknown_count"),
        },
        "k3_b1_recovery": {
            "current_state": string_field(recovery, "current_state"),
            "k3_b2_blocked": recovery
                .get("k3_b2_gate")
                .map(|gate| bool_field(gate, "blocked"))
                .unwrap_or(false),
            "readback_status": recovery
                .get("readback_boundary")
                .map(|boundary| string_field(boundary, "status"))
                .unwrap_or_default(),
            "result_count": recovery
                .get("readback_boundary")
                .and_then(|boundary| boundary.get("result_count").cloned())
                .unwrap_or(Value::Null),
        },
        "operation_control": {
            "operation_count": array_field_len(operation_control, "operations"),
            "true_operation_available": bool_field(operation_control, "true_operation_available"),
            "k3_b2_unlocked": bool_field(operation_control, "k3_b2_unlocked"),
            "readback_status": operation_control
                .get("readback_boundary")
                .map(|boundary| string_field(boundary, "status"))
                .unwrap_or_default(),
            "result_count": operation_control
                .get("readback_boundary")
                .and_then(|boundary| boundary.get("result_count").cloned())
                .unwrap_or(Value::Null),
        },
        "diagnostic": {
            "degraded_count": array_field_len(snapshot.get("diagnostic_summary").unwrap_or(&Value::Null), "degraded_states"),
        },
        "user_facing_summary": format!(
            "{} 条工作流，{} 条运行关注，{} 条等待权限",
            workflows.len(),
            workflow_focus_count + runtime_attention_count,
            waiting_permission_count
        ),
        "developer_details_collapsed": true,
    })
}

fn memory_payload(snapshot: &Value) -> Value {
    json!({
        "schema_version": "memory_center_page_read_model.v1",
        "snapshot_status_label": "后端按页读模型",
        "snapshot_context": {
            "project_count": array_len(snapshot, "projects"),
            "task_count": array_len(snapshot, "tasks"),
        },
        "external_store_inputs": [
            "formal-memories.v1.json",
            "memory-candidates.v1.json",
            "observations.v1.json",
            "memory-lint.v1.json",
            "memory-patterns.v1.json"
        ],
        "formal_memory": { "source": "external_store_required" },
        "candidate_memory": { "source": "external_store_required" },
        "observation": { "source": "external_store_required" },
        "lint": { "source": "external_store_required" },
        "user_facing_summary": "记忆页后端查询已提供 snapshot 上下文；正式记忆 / 候选 / 观察仍来自记忆 store。",
        "developer_details_collapsed": true,
    })
}

fn knowledge_payload(snapshot: &Value) -> Value {
    let document_count: usize = items(snapshot, "projects")
        .iter()
        .map(|project| {
            array_field_len(project, "authority_files")
                + array_field_len(project, "handoff_files")
                + array_field_len(project, "evidence_files")
        })
        .sum();
    json!({
        "schema_version": "knowledge_base_page_read_model.v1",
        "snapshot_status_label": "后端按页读模型",
        "document_count": document_count,
        "task_reference_count": array_len(snapshot, "tasks"),
        "formal_memory_link_count": 0,
        "candidate_link_count": 0,
        "capture_event_count": 0,
        "obsidian_boundary": {
            "label": "Obsidian-compatible 占位",
            "native_sync_status": "未执行 Obsidian 原生同步",
            "vault_scan_status": "未自动扫描 vault",
            "forbidden_text": "知识命中不能绕过候选、正式记忆、来源、版本、审计和权限治理。"
        },
        "user_facing_summary": format!("资料 {}，任务引用 {}", document_count, array_len(snapshot, "tasks")),
        "developer_details_collapsed": true,
    })
}

fn settings_payload(snapshot: &Value) -> Value {
    let summary = snapshot.get("summary").unwrap_or(&Value::Null);
    let page_inventory = snapshot
        .get("page_read_model_inventory")
        .unwrap_or(&Value::Null);
    let diagnostic_summary = snapshot.get("diagnostic_summary").unwrap_or(&Value::Null);

    json!({
        "schema_version": "settings_page_read_model.v1",
        "snapshot_status_label": "后端按页读模型",
        "boundary_text": "设置页只整理入口和边界，不读取凭据、不触发执行。",
        "general": {
            "project_count": usize_field(summary, "project_count"),
            "session_count": usize_field(summary, "session_count"),
            "skill_count": usize_field(summary, "skill_count"),
            "workflow_count": 0,
        },
        "developer_boundary": {
            "adapter_count": array_len(snapshot, "agent_adapters"),
            "provider_count": array_len(snapshot, "provider_availability"),
            "diagnostic_count": array_field_len(diagnostic_summary, "degraded_states"),
            "runtime_log_count": array_field_len(snapshot.get("runtime_log_store").unwrap_or(&Value::Null), "entries"),
            "page_contract_count": array_field_len(page_inventory, "contracts"),
            "credential_display_allowed": false,
            "execution_from_settings_allowed": false,
        },
        "page_contract": {
            "count": array_field_len(page_inventory, "contracts"),
            "status": string_field(page_inventory, "status"),
            "source_policy": string_field(page_inventory, "source_policy"),
        },
        "user_facing_summary": format!(
            "{} 个项目，{} 个会话，{} 个页面合同",
            usize_field(summary, "project_count"),
            usize_field(summary, "session_count"),
            array_field_len(page_inventory, "contracts")
        ),
        "developer_details_collapsed": true,
    })
}

fn agent_project_options(snapshot: &Value) -> Vec<Value> {
    let sessions = items(snapshot, "sessions");
    let mut sessions_by_project = count_by_optional_string(sessions, "project_root");
    let mut known_roots = BTreeSet::new();
    let mut options: Vec<Value> = items(snapshot, "projects")
        .into_iter()
        .map(|project| {
            let root = string_field(project, "project_root");
            known_roots.insert(root.clone());
            let session_count = sessions_by_project
                .remove(&root)
                .unwrap_or_else(|| usize_field(project, "thread_count"));
            json!({
                "project_root": root,
                "label": string_field(project, "name"),
                "session_count": session_count,
                "active_session_count": usize_field(project, "active_thread_count"),
            })
        })
        .collect();
    for (root, count) in sessions_by_project {
        if known_roots.contains(&root) {
            continue;
        }
        options.push(json!({
            "project_root": root,
            "label": tail(&root),
            "session_count": count,
            "active_session_count": count,
        }));
    }
    options
}

fn schema(
    page_id: &str,
    page_label: &str,
    read_model_type: &str,
    schema_version: &str,
    snapshot_fields: &[&str],
    workflow_state_fields: &[&str],
    external_store_inputs: &[&str],
    output_sections: &[&str],
    next_step: &str,
) -> PageReadModelSchemaContract {
    PageReadModelSchemaContract {
        page_id: page_id.to_string(),
        page_label: page_label.to_string(),
        read_model_type: read_model_type.to_string(),
        schema_version: schema_version.to_string(),
        snapshot_fields: strings(snapshot_fields),
        workflow_state_fields: strings(workflow_state_fields),
        external_store_inputs: strings(external_store_inputs),
        output_sections: strings(output_sections),
        migration_status: "schema_only".to_string(),
        workbench_snapshot_active: true,
        returns_business_data: false,
        page_ui_migrated: false,
        next_step: next_step.to_string(),
    }
}

fn contract(
    page_id: &str,
    page_label: &str,
    user_facing_data: &[&str],
    developer_internal_data: &[&str],
    must_not_show_as_primary: &[&str],
    planned_read_model: &str,
    next_step: &str,
) -> PageReadModelContract {
    PageReadModelContract {
        page_id: page_id.to_string(),
        page_label: page_label.to_string(),
        user_facing_data: strings(user_facing_data),
        developer_internal_data: strings(developer_internal_data),
        must_not_show_as_primary: strings(must_not_show_as_primary),
        current_source: "workbench_snapshot".to_string(),
        planned_read_model: planned_read_model.to_string(),
        migration_status: "contract_only".to_string(),
        next_step: next_step.to_string(),
    }
}

fn derive_snapshot_field_coverage() -> Vec<PageSnapshotFieldCoverage> {
    vec![
        coverage(
            "summary",
            &["projects", "running_workflows", "settings"],
            "Counts, generated_at, and warning totals feed page summaries.",
        ),
        coverage(
            "projects",
            &["projects", "agents", "memory", "knowledge"],
            "Project context feeds primary project, conversation, memory, and knowledge surfaces.",
        ),
        coverage(
            "sessions",
            &["projects", "agents", "running_workflows"],
            "Sessions feed project counts, agent conversation selection, and runtime summaries.",
        ),
        coverage(
            "skills",
            &["settings"],
            "Skill inventory is covered by settings during the six-page batch; Skill board query is outside batch one.",
        ),
        coverage(
            "plugins",
            &["settings"],
            "Plugin inventory is covered by settings developer inventory during batch one.",
        ),
        coverage(
            "tasks",
            &["projects", "memory", "knowledge", "settings"],
            "Task package and task reference counts feed project, memory, knowledge, and settings summaries.",
        ),
        coverage(
            "agent_adapters",
            &["agents", "settings"],
            "Adapter availability is visible on agents and counted in settings developer details.",
        ),
        coverage(
            "session_operations",
            &["agents", "settings"],
            "Session operation boundary is visible on agents and counted in settings developer details.",
        ),
        coverage(
            "provider_availability",
            &["agents", "settings"],
            "Provider and credential boundary is visible on agents and counted in settings developer details.",
        ),
        coverage(
            "session_continuation_previews",
            &["agents", "running_workflows"],
            "Continuation previews feed conversation readiness and running workflow attention.",
        ),
        coverage(
            "session_continuation_store",
            &["agents", "running_workflows"],
            "Continuation store feeds attempts, audit refs, and runtime queue state.",
        ),
        coverage(
            "runtime_session_attention",
            &["agents", "running_workflows"],
            "Runtime attention feeds conversation warnings and running workflow focus.",
        ),
        coverage(
            "session_run_status_summaries",
            &["agents", "running_workflows"],
            "Run status summaries feed session status and running workflow rollups.",
        ),
        coverage(
            "runtime_log_store",
            &["running_workflows", "settings"],
            "Runtime log summaries feed running workflow state and settings health.",
        ),
        coverage(
            "worker_protocol",
            &["agents", "running_workflows"],
            "Worker protocol feeds adapter/work-thread boundary and run unit diagnostics.",
        ),
        coverage(
            "real_execution_product_commands",
            &["agents", "running_workflows"],
            "Product command state feeds execution readiness and run control summaries.",
        ),
        coverage(
            "project_workflow_automation",
            &["projects", "running_workflows"],
            "Automation plans feed project workflow summaries and running queue state.",
        ),
        coverage(
            "k3_b1_recovery",
            &["projects", "agents", "running_workflows"],
            "K3-B1 blocked recovery feeds the project recovery card, agent boundary, and running workflow blocked state without enabling execution.",
        ),
        coverage(
            "operation_control",
            &["agents", "running_workflows"],
            "L3 operation control feeds confirmable retry/stop/restart/resume product controls without enabling execution.",
        ),
        coverage(
            "page_read_model_inventory",
            &["settings"],
            "Page contract inventory belongs in settings developer details.",
        ),
        coverage(
            "diagnostic_summary",
            &["running_workflows", "settings"],
            "Degraded state summaries feed running workflow attention and settings health.",
        ),
        coverage(
            "diagnostics",
            &["settings"],
            "Diagnostics detail remains developer-facing in settings.",
        ),
    ]
}

fn workbench_snapshot_field_names() -> Vec<&'static str> {
    vec![
        "summary",
        "projects",
        "sessions",
        "skills",
        "plugins",
        "tasks",
        "agent_adapters",
        "session_operations",
        "provider_availability",
        "session_continuation_previews",
        "session_continuation_store",
        "runtime_session_attention",
        "session_run_status_summaries",
        "runtime_log_store",
        "worker_protocol",
        "real_execution_product_commands",
        "project_workflow_automation",
        "k3_b1_recovery",
        "operation_control",
        "page_read_model_inventory",
        "diagnostic_summary",
        "diagnostics",
    ]
}

fn coverage(field_name: &str, covered_by_pages: &[&str], notes: &str) -> PageSnapshotFieldCoverage {
    PageSnapshotFieldCoverage {
        field_name: field_name.to_string(),
        covered_by_pages: strings(covered_by_pages),
        coverage_status: "covered_by_page_schema".to_string(),
        notes: notes.to_string(),
    }
}

fn uncovered_snapshot_fields(coverage: &[PageSnapshotFieldCoverage]) -> Vec<String> {
    workbench_snapshot_field_names()
        .into_iter()
        .filter(|field_name| {
            !coverage
                .iter()
                .any(|entry| entry.field_name == *field_name && !entry.covered_by_pages.is_empty())
        })
        .map(|field_name| field_name.to_string())
        .collect()
}

fn items<'a>(value: &'a Value, key: &str) -> Vec<&'a Value> {
    items_from_value(value, key)
}

fn items_opt<'a>(value: Option<&'a Value>, key: &str) -> Vec<&'a Value> {
    value
        .map(|value| items_from_value(value, key))
        .unwrap_or_default()
}

fn items_from_value<'a>(value: &'a Value, key: &str) -> Vec<&'a Value> {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(|items| items.iter().collect())
        .unwrap_or_default()
}

fn array_len(value: &Value, key: &str) -> usize {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0)
}

fn array_field_len(value: &Value, key: &str) -> usize {
    array_len(value, key)
}

fn string_field(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn bool_field(value: &Value, key: &str) -> bool {
    value.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn usize_field(value: &Value, key: &str) -> usize {
    value
        .get(key)
        .and_then(Value::as_u64)
        .map(|value| value as usize)
        .unwrap_or(0)
}

fn count_by_optional_string(items: Vec<&Value>, key: &str) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for item in items {
        let Some(value) = item.get(key).and_then(Value::as_str) else {
            continue;
        };
        if value.is_empty() {
            continue;
        }
        *counts.entry(value.to_string()).or_insert(0) += 1;
    }
    counts
}

fn tail(path: &str) -> String {
    path.rsplit('/')
        .find(|part| !part.is_empty())
        .unwrap_or(path)
        .to_string()
}

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| item.to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_read_model_inventory_freezes_r4_a1_contracts_only() {
        let inventory = derive_page_read_model_inventory("2026-06-11T00:00:00Z");
        let page_ids: Vec<&str> = inventory
            .contracts
            .iter()
            .map(|contract| contract.page_id.as_str())
            .collect();

        assert_eq!(
            page_ids,
            vec![
                "home",
                "projects",
                "agents",
                "running_workflows",
                "memory",
                "knowledge",
                "settings",
                "skill",
                "harness",
            ]
        );
        assert_eq!(inventory.status, "contract_only");
        assert!(inventory
            .warnings
            .contains(&"workbench_snapshot_still_active".to_string()));
        assert!(inventory.contracts.iter().all(|contract| {
            contract.current_source == "workbench_snapshot"
                && contract.migration_status == "contract_only"
        }));
        assert!(inventory
            .contracts
            .iter()
            .find(|contract| contract.page_id == "agents")
            .expect("agents contract exists")
            .must_not_show_as_primary
            .contains(&"控制中心式全量边界面板".to_string()));
    }

    #[test]
    fn page_read_model_query_returns_selector_contract_for_known_page() {
        let output = query_page_read_model(
            &PageReadModelQueryInput {
                page_id: "agents".to_string(),
            },
            "2026-06-11T00:00:00Z",
        )
        .expect("known page should resolve");

        assert_eq!(output.schema_version, "workbench_page_read_model_query.v1");
        assert_eq!(output.status, "selector_contract_only");
        assert_eq!(output.requested_page_id, "agents");
        assert_eq!(output.page_label, "智能体");
        let target_schema = output
            .target_schema
            .expect("agents target schema should be returned");
        assert_eq!(target_schema.schema_version, "agents_page_read_model.v1");
        assert!(target_schema
            .snapshot_fields
            .contains(&"session_continuation_store".to_string()));
        assert!(!target_schema.returns_business_data);
        assert!(!target_schema.page_ui_migrated);
        assert!(output.uncovered_snapshot_fields.is_empty());
        assert_eq!(output.contract.current_source, "workbench_snapshot");
        assert_eq!(
            output.selector_plan.ui_consumption_status,
            "not_connected_to_pages"
        );
        assert!(output.source_boundary.workbench_snapshot_active);
        assert!(!output.source_boundary.returns_business_data);
        assert!(!output.source_boundary.writes_stores);
        assert!(!output.source_boundary.tauri_command_migrates_page);
        assert!(output
            .warnings
            .contains(&"h2_1_schema_defined_no_query_migration".to_string()));
        assert!(output
            .warnings
            .contains(&"do_not_claim_workbench_snapshot_deprecated".to_string()));
    }

    #[test]
    fn page_read_model_query_rejects_unknown_or_empty_page() {
        let unknown = query_page_read_model(
            &PageReadModelQueryInput {
                page_id: "missing".to_string(),
            },
            "2026-06-11T00:00:00Z",
        )
        .expect_err("unknown page should be rejected");
        assert_eq!(unknown, "unknown_page_id:missing");

        let empty = query_page_read_model(
            &PageReadModelQueryInput {
                page_id: "  ".to_string(),
            },
            "2026-06-11T00:00:00Z",
        )
        .expect_err("empty page should be rejected");
        assert_eq!(empty, "page_id_required");
    }

    #[test]
    fn page_read_model_schema_catalog_defines_batch_one_six_pages() {
        let catalog = derive_page_read_model_schema_catalog("2026-06-13T00:00:00Z");
        let page_ids: Vec<&str> = catalog
            .schemas
            .iter()
            .map(|schema| schema.page_id.as_str())
            .collect();

        assert_eq!(
            page_ids,
            vec![
                "projects",
                "agents",
                "running_workflows",
                "memory",
                "knowledge",
                "settings",
            ]
        );
        assert_eq!(catalog.status, "schema_defined_no_query_migration");
        assert!(catalog.schemas.iter().all(|schema| {
            schema.workbench_snapshot_active
                && !schema.returns_business_data
                && !schema.page_ui_migrated
        }));
    }

    #[test]
    fn page_read_model_schema_catalog_covers_workbench_snapshot_fields() {
        let catalog = derive_page_read_model_schema_catalog("2026-06-13T00:00:00Z");
        let covered_fields: Vec<&str> = catalog
            .snapshot_field_coverage
            .iter()
            .map(|coverage| coverage.field_name.as_str())
            .collect();

        assert_eq!(workbench_snapshot_field_names().len(), 22);
        assert_eq!(catalog.snapshot_field_coverage.len(), 22);
        assert!(catalog.uncovered_snapshot_fields.is_empty());
        for field_name in workbench_snapshot_field_names() {
            assert!(
                covered_fields.contains(&field_name),
                "missing snapshot field coverage for {field_name}"
            );
        }
        assert!(catalog.snapshot_field_coverage.iter().all(|coverage| {
            coverage.coverage_status == "covered_by_page_schema"
                && !coverage.covered_by_pages.is_empty()
        }));
    }

    #[test]
    fn page_read_model_query_with_snapshot_returns_payload_for_batch_one_pages() {
        let snapshot = fixture_snapshot_value();
        let workflow_state = fixture_workflow_state_value();

        for page_id in [
            "projects",
            "agents",
            "running_workflows",
            "memory",
            "knowledge",
            "settings",
        ] {
            let output = query_page_read_model_with_snapshot_value(
                &PageReadModelQueryInput {
                    page_id: page_id.to_string(),
                },
                "2026-06-13T00:00:00Z",
                &snapshot,
                Some(&workflow_state),
            )
            .expect("batch-one page should resolve");

            assert_eq!(output.status, "page_data_ready");
            assert!(output.source_boundary.returns_business_data);
            assert!(!output.source_boundary.writes_stores);
            assert!(!output.source_boundary.tauri_command_migrates_page);
            let payload = output.page_payload.expect("payload should exist");
            assert_eq!(payload.page_id, page_id);
            assert_eq!(payload.generated_from, "workbench_page_query");
            assert!(payload.data.is_object());
            let snapshot_slice = payload
                .data
                .get("snapshot_slice")
                .expect("payload should carry snapshot slice");
            let target_schema = output
                .target_schema
                .expect("batch-one page should have target schema");
            assert!(target_schema.returns_business_data);
            assert!(target_schema.page_ui_migrated);
            for field_name in target_schema.snapshot_fields {
                assert!(
                    snapshot_slice.get(&field_name).is_some(),
                    "snapshot_slice should include {field_name} for {page_id}"
                );
            }
            assert_eq!(
                output.selector_plan.ui_consumption_status,
                "page_query_payload_ready"
            );
            assert!(output
                .warnings
                .contains(&"h2_3_page_query_payload_supports_frontend_consumption".to_string()));
        }
    }

    #[test]
    fn page_read_model_query_with_snapshot_keeps_non_batch_pages_contract_only() {
        let snapshot = fixture_snapshot_value();
        let output = query_page_read_model_with_snapshot_value(
            &PageReadModelQueryInput {
                page_id: "home".to_string(),
            },
            "2026-06-13T00:00:00Z",
            &snapshot,
            None,
        )
        .expect("home contract should still resolve");

        assert_eq!(output.status, "selector_contract_only");
        assert!(!output.source_boundary.returns_business_data);
        assert!(output.page_payload.is_none());
        assert!(output.target_schema.is_none());
    }

    fn fixture_snapshot_value() -> Value {
        serde_json::json!({
            "summary": {
                "generated_at": "2026-06-13T00:00:00Z",
                "project_count": 1,
                "session_count": 2,
                "skill_count": 1,
                "plugin_count": 1,
                "task_count": 1,
                "warning_count": 0
            },
            "projects": [{
                "project_root": "/tmp/mario-test",
                "name": "mario test",
                "active_hint": true,
                "thread_count": 2,
                "active_thread_count": 1,
                "archived_thread_count": 1,
                "latest_updated_at_ms": 100,
                "authority_files": [{"path": "AUTHORITY.md"}],
                "handoff_files": [{"path": "handoff.md"}],
                "evidence_files": [{"path": "evidence.md"}],
                "context_warnings": [],
                "warnings": []
            }],
            "sessions": [
                {"thread_id": "s1", "project_root": "/tmp/mario-test", "archived": false, "rollout_exists": true},
                {"thread_id": "s2", "project_root": "/tmp/mario-test", "archived": true, "rollout_exists": false}
            ],
            "skills": [{"skill_id": "skill-1"}],
            "plugins": [{"plugin_name": "plugin-1"}],
            "tasks": [{"status": "done", "title": "fixture task"}],
            "agent_adapters": [
                {"adapter_id": "codex-local", "status": "available"},
                {"adapter_id": "claude-code-planned", "status": "planned"}
            ],
            "session_operations": [{"operation_id": "resume"}],
            "provider_availability": [{"provider_id": "claude"}],
            "session_continuation_previews": [{"preview_id": "preview-1"}],
            "session_continuation_store": {"continuations": [], "attempts": [], "audit_events": []},
            "runtime_session_attention": [{
                "attention_id": "attn-1",
                "status": "readback_unavailable",
                "requires_user_action": true,
                "blocks_continuation": false,
                "readback_boundary": {"status": "readback_unavailable", "result_count": null}
            }],
            "session_run_status_summaries": [{"session_id": "s1"}],
            "runtime_log_store": {"entries": [{"entry_id": "runtime-1"}], "summaries": []},
            "worker_protocol": {"work_threads": [{"thread_id": "worker-1"}]},
            "real_execution_product_commands": {
                "command_count": 1,
                "pending_decision_count": 1,
                "blocked_attempt_count": 0,
                "running_attempt_count": 0
            },
            "project_workflow_automation": {
                "run_unit_count": 1,
                "waiting_user_count": 1,
                "blocked_count": 0,
                "readback_unknown_count": 1
            },
            "k3_b1_recovery": {
                "schema_version": "k3_b1_recovery_read_model.v1",
                "execution_point_id": "stage-k-k3-b1-mario-test-workflow-read-only",
                "current_state": "blocked_by_safety_review_again",
                "k3_b2_gate": {
                    "blocked": true,
                    "status": "blocked_waiting_k3_b1_recovery_acceptance",
                    "reason": "K3-B1 still blocked"
                },
                "readback_boundary": {
                    "status": "not_attempted_l1_recovery_path_only",
                    "result_count": null,
                    "unavailable_reason": "L1 does not execute Codex",
                    "user_submitted_evidence_only": true
                }
            },
            "page_read_model_inventory": {
                "status": "contract_only",
                "source_policy": "fixture",
                "contracts": [{"page_id": "projects"}]
            },
            "diagnostic_summary": {"degraded_states": [{"state_id": "diag-1"}]},
            "diagnostics": {"notes": []}
        })
    }

    fn fixture_workflow_state_value() -> Value {
        serde_json::json!({
            "project_workflows": [{
                "project_root": "/tmp/mario-test",
                "workflow_id": "wf-1",
                "state": "running",
                "task_drafts": [{"work_item_id": "task-1", "state": "waiting_for_permission"}],
                "recent_execution_attempts": []
            }]
        })
    }
}
