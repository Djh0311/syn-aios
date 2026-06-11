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
