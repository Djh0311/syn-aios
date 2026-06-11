use serde::Serialize;

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
}
