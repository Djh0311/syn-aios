import type { WorkbenchPageReadModelInventory } from "../../src/lib/pageReadModel";

export function pageReadModelInventoryFixture(): WorkbenchPageReadModelInventory {
  return {
    schema_version: "workbench_page_read_model_inventory.v1",
    generated_at: "2026-06-11T00:00:00Z",
    status: "contract_only",
    source_policy: "R4-A1 only records page data contracts; pages still read the existing WorkbenchSnapshot.",
    contracts: [
      {
        page_id: "home",
        page_label: "首页",
        user_facing_data: ["主对象入口", "运行中摘要", "待处理摘要", "索引状态"],
        developer_internal_data: ["snapshot source", "diagnostics refs"],
        must_not_show_as_primary: ["raw sidecar", "full audit path", "schema dump"],
        current_source: "workbench_snapshot",
        planned_read_model: "HomePageReadModel",
        migration_status: "contract_only",
        next_step: "R4-A2+ can introduce a page query or frontend selector without changing layout.",
      },
      {
        page_id: "agents",
        page_label: "智能体",
        user_facing_data: ["项目选择", "会话选择", "对话流", "输入/执行 readiness"],
        developer_internal_data: ["adapter descriptors", "operation/provider/session boundary"],
        must_not_show_as_primary: ["控制中心式全量边界面板", "未实现执行按钮"],
        current_source: "workbench_snapshot",
        planned_read_model: "AgentsPageReadModel",
        migration_status: "contract_only",
        next_step: "Keep normal UI conversation-first; move boundary data behind developer details.",
      },
    ],
    warnings: ["r4_a1_contract_only_no_page_query", "workbench_snapshot_still_active"],
  };
}
