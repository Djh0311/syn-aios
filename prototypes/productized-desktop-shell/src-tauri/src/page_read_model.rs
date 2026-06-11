use serde::{Deserialize, Serialize};

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

    Ok(PageReadModelQueryResult {
        schema_version: "workbench_page_read_model_query.v1".to_string(),
        generated_at: generated_at.to_string(),
        status: "selector_contract_only".to_string(),
        requested_page_id: page_id.to_string(),
        page_label: contract.page_label.clone(),
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
            "r4_a2_skeleton_no_page_data_query".to_string(),
            "workbench_snapshot_still_active".to_string(),
            "do_not_claim_workbench_snapshot_deprecated".to_string(),
        ],
    })
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
}
