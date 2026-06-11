export type PageReadModelContract = {
  page_id: string;
  page_label: string;
  user_facing_data: string[];
  developer_internal_data: string[];
  must_not_show_as_primary: string[];
  current_source: string;
  planned_read_model: string;
  migration_status: string;
  next_step: string;
};

export type WorkbenchPageReadModelInventory = {
  schema_version: "workbench_page_read_model_inventory.v1" | string;
  generated_at: string;
  status: string;
  source_policy: string;
  contracts: PageReadModelContract[];
  warnings: string[];
};

export type PageReadModelQueryInput = {
  page_id: string;
};

export type PageReadModelSelectorPlan = {
  selector_id: string;
  selector_kind: string;
  planned_read_model: string;
  data_migration_status: string;
  ui_consumption_status: string;
  next_step: string;
};

export type PageReadModelSourceBoundary = {
  current_source: string;
  workbench_snapshot_active: boolean;
  returns_business_data: boolean;
  writes_stores: boolean;
  tauri_command_migrates_page: boolean;
  warnings: string[];
};

export type PageReadModelQueryResult = {
  schema_version: "workbench_page_read_model_query.v1" | string;
  generated_at: string;
  status: string;
  requested_page_id: string;
  page_label: string;
  contract: PageReadModelContract;
  selector_plan: PageReadModelSelectorPlan;
  source_boundary: PageReadModelSourceBoundary;
  warnings: string[];
};
