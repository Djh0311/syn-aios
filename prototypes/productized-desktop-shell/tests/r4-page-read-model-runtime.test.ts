import {
  batchOneWorkbenchPageIds,
  loadWorkbenchSnapshotFromPageQueries,
  pageReadModelSnapshotWarnings,
  snapshotFromPageReadModelResults,
  type BatchOneWorkbenchPageId,
} from "../src/lib/pageReadModelRuntime";
import type { PageReadModelQueryResult } from "../src/lib/pageReadModel";

const requestedPageIds: string[] = [];
const { snapshot, warnings } = await loadWorkbenchSnapshotFromPageQueries(async ({ page_id }) => {
  requestedPageIds.push(page_id);
  const snapshotSlice: Record<string, unknown> = {};
  if (page_id === "settings") {
    snapshotSlice.summary = {
      generated_at: "2026-06-13T00:00:00Z",
      project_count: 1,
      session_count: 1,
      skill_count: 0,
      plugin_count: 0,
      task_count: 0,
      warning_count: 0,
    };
  }
  if (page_id === "projects") {
    snapshotSlice.projects = [
      {
        project_root: "/tmp/project",
        name: "Project",
        active_hint: true,
        thread_count: 1,
        active_thread_count: 1,
        archived_thread_count: 0,
        latest_updated_at_ms: null,
        authority_files: [],
        handoff_files: [],
        evidence_files: [],
        harness_candidates: [],
        harness_resources: [],
        context_warnings: [],
        warnings: [],
      },
    ];
  }
  return pageResult(page_id as BatchOneWorkbenchPageId, snapshotSlice);
});

assert(
  requestedPageIds.join(",") === batchOneWorkbenchPageIds.join(","),
  "runtime should query all six batch-one pages",
);
assert(snapshot.summary.project_count === 1, "settings slice should provide summary");
assert(snapshot.projects.length === 1, "projects slice should provide projects");
assert(snapshot.projects[0]?.name === "Project", "projects slice should be merged into snapshot");
assert(warnings.length === 0, "complete page query set should not emit fallback warnings");

const partial = snapshotFromPageReadModelResults({
  projects: pageResult("projects", {
    projects: [],
  }),
});

assert(partial.projects.length === 0, "known ready slice should overwrite fallback fields");
assert(partial.sessions.length === 0, "missing slices should keep empty fallback without pretending data exists");
assert(
  pageReadModelSnapshotWarnings({
    projects: pageResult("projects", {
      projects: [],
    }),
  }).includes("missing_page_read_model:agents"),
  "missing page slices should emit warnings while falling back",
);

console.log("r4 page read model runtime test passed");

function pageResult(pageId: BatchOneWorkbenchPageId, snapshotSlice: Record<string, unknown>): PageReadModelQueryResult {
  return {
    schema_version: "workbench_page_read_model_query.v1",
    generated_at: "2026-06-13T00:00:00Z",
    status: "page_data_ready",
    requested_page_id: pageId,
    page_label: pageId,
    contract: {
      page_id: pageId,
      page_label: pageId,
      user_facing_data: [],
      developer_internal_data: [],
      must_not_show_as_primary: [],
      current_source: "workbench_snapshot",
      planned_read_model: `${pageId}_read_model`,
      migration_status: "contract_only",
      next_step: "fixture",
    },
    target_schema: null,
    snapshot_field_coverage: [],
    uncovered_snapshot_fields: [],
    page_payload: {
      page_id: pageId,
      schema_version: `${pageId}_page_read_model.v1`,
      generated_from: "workbench_page_query",
      data: {
        snapshot_slice: snapshotSlice,
      },
      warnings: ["snapshot_slice_read_only"],
    },
    selector_plan: {
      selector_id: `${pageId}_selector`,
      selector_kind: "page_read_model_selector_contract",
      planned_read_model: `${pageId}_read_model`,
      data_migration_status: "backend_page_query_ready",
      ui_consumption_status: "page_query_payload_ready",
      next_step: "fixture",
    },
    source_boundary: {
      current_source: "workbench_snapshot",
      workbench_snapshot_active: true,
      returns_business_data: true,
      writes_stores: false,
      tauri_command_migrates_page: false,
      warnings: ["h2_3_snapshot_slice_available"],
    },
    warnings: ["h2_3_page_query_payload_supports_frontend_consumption"],
  };
}

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}
