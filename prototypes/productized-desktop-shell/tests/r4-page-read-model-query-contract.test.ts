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
  target_schema: {
    page_id: "agents",
    page_label: "智能体",
    read_model_type: "AgentsPageReadModel",
    schema_version: "agents_page_read_model.v1",
    snapshot_fields: [
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
    ],
    workflow_state_fields: ["project_workflows", "session_bindings"],
    external_store_inputs: ["session-continuations.v1.json", "real-execution-product-commands.v1.json"],
    output_sections: ["project picker", "session picker", "conversation readiness", "collapsed developer boundary"],
    migration_status: "schema_only",
    workbench_snapshot_active: true,
    returns_business_data: false,
    page_ui_migrated: false,
    next_step: "H2-2 should keep the agents page conversation-first while exposing boundary data as page data.",
  },
  snapshot_field_coverage: [
    {
      field_name: "sessions",
      covered_by_pages: ["projects", "agents", "running_workflows"],
      coverage_status: "covered_by_page_schema",
      notes: "fixture coverage",
    },
  ],
  uncovered_snapshot_fields: [],
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
assert(result.target_schema?.schema_version === "agents_page_read_model.v1", "H2-1 should attach page schema");
assert(!result.target_schema.returns_business_data, "H2-1 schema must not claim business payload");
assert(result.target_schema.snapshot_fields.includes("session_continuation_store"), "schema should list agent page snapshot fields");
assert(result.uncovered_snapshot_fields?.length === 0, "H2-1 coverage should not leave snapshot fields uncovered");
assert(!result.source_boundary.writes_stores, "R4-A2 skeleton must not write stores");
assert(!result.source_boundary.tauri_command_migrates_page, "R4-A2 skeleton must not migrate page UI");
assert(
  result.warnings.includes("do_not_claim_workbench_snapshot_deprecated"),
  "R4-A2 should warn against claiming WorkbenchSnapshot deprecation",
);

const backendPayloadResult: PageReadModelQueryResult = {
  ...result,
  status: "page_data_ready",
  source_boundary: {
    ...result.source_boundary,
    returns_business_data: true,
    writes_stores: false,
    tauri_command_migrates_page: false,
  },
  page_payload: {
    page_id: "agents",
    schema_version: "agents_page_read_model.v1",
    generated_from: "workbench_page_query",
    data: {
      adapter_count: 2,
      conversation_first: true,
      snapshot_slice: {
        projects: [],
        sessions: [],
        agent_adapters: [],
      },
    },
    warnings: ["backend_page_payload_read_only", "snapshot_slice_read_only"],
  },
  warnings: ["h2_2_backend_page_payload_ready", "h2_3_page_query_payload_supports_frontend_consumption"],
};

assert(backendPayloadResult.status === "page_data_ready", "H2-2 backend query should expose page_data_ready");
assert(backendPayloadResult.source_boundary.returns_business_data, "H2-2 backend query should return business data");
assert(backendPayloadResult.page_payload?.generated_from === "workbench_page_query", "payload source should be explicit");
assert(Boolean(backendPayloadResult.page_payload?.data.snapshot_slice), "H2-3 payload should expose a read-only snapshot slice");
assert(!backendPayloadResult.source_boundary.writes_stores, "H2-2 backend query must stay read-only");
assert(!backendPayloadResult.source_boundary.tauri_command_migrates_page, "H2-2 must not claim page UI migration");

console.log("r4 page read model query contract test passed");

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}
