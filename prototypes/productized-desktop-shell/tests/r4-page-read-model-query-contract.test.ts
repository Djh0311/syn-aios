import type { PageReadModelQueryInput, PageReadModelQueryResult } from "../src/lib/pageReadModel";
import { pageReadModelInventoryFixture } from "./fixtures/pageReadModelFixture";

const request: PageReadModelQueryInput = { page_id: "agents" };
const agentsContract = pageReadModelInventoryFixture().contracts.find(
  (contract) => contract.page_id === request.page_id,
);

assert(agentsContract, "agents contract fixture should exist");

const result: PageReadModelQueryResult = {
  schema_version: "workbench_page_read_model_query.v1",
  generated_at: "2026-06-11T00:00:00Z",
  status: "selector_contract_only",
  requested_page_id: request.page_id,
  page_label: agentsContract.page_label,
  contract: agentsContract,
  selector_plan: {
    selector_id: "agents_selector_contract",
    selector_kind: "page_read_model_selector_contract",
    planned_read_model: agentsContract.planned_read_model,
    data_migration_status: "not_migrated",
    ui_consumption_status: "not_connected_to_pages",
    next_step: agentsContract.next_step,
  },
  source_boundary: {
    current_source: agentsContract.current_source,
    workbench_snapshot_active: true,
    returns_business_data: false,
    writes_stores: false,
    tauri_command_migrates_page: false,
    warnings: [
      "r4_a2_selector_contract_only_no_business_data",
      "workbench_snapshot_still_active",
      "page_ui_not_migrated",
    ],
  },
  warnings: [
    "r4_a2_skeleton_no_page_data_query",
    "workbench_snapshot_still_active",
    "do_not_claim_workbench_snapshot_deprecated",
  ],
};

assert(result.status === "selector_contract_only", "R4-A2 result should stay contract-only");
assert(result.source_boundary.workbench_snapshot_active, "WorkbenchSnapshot should remain active");
assert(!result.source_boundary.returns_business_data, "R4-A2 skeleton must not return business data");
assert(!result.source_boundary.writes_stores, "R4-A2 skeleton must not write stores");
assert(!result.source_boundary.tauri_command_migrates_page, "R4-A2 skeleton must not migrate page UI");
assert(
  result.warnings.includes("do_not_claim_workbench_snapshot_deprecated"),
  "R4-A2 should warn against claiming WorkbenchSnapshot deprecation",
);

console.log("r4 page read model query contract test passed");

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}
