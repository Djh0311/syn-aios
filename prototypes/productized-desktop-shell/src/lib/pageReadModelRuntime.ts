import { emptySnapshot } from "./emptySnapshot";
import type { PageReadModelQueryInput, PageReadModelQueryResult } from "./pageReadModel";
import type { WorkbenchSnapshot } from "./types";

export const batchOneWorkbenchPageIds = [
  "projects",
  "agents",
  "running_workflows",
  "memory",
  "knowledge",
  "settings",
] as const;

export type BatchOneWorkbenchPageId = (typeof batchOneWorkbenchPageIds)[number];

export type BatchOnePageReadModelResults = Partial<Record<BatchOneWorkbenchPageId, PageReadModelQueryResult>>;

type QueryPageReadModel = (request: PageReadModelQueryInput) => Promise<PageReadModelQueryResult>;

export async function loadWorkbenchSnapshotFromPageQueries(
  queryPageReadModel: QueryPageReadModel,
): Promise<{ snapshot: WorkbenchSnapshot; pageReadModels: BatchOnePageReadModelResults; warnings: string[] }> {
  const results = await Promise.all(
    batchOneWorkbenchPageIds.map((pageId) => queryPageReadModel({ page_id: pageId })),
  );
  const pageReadModels = indexPageReadModelResults(results);
  return {
    snapshot: snapshotFromPageReadModelResults(pageReadModels),
    pageReadModels,
    warnings: pageReadModelSnapshotWarnings(pageReadModels),
  };
}

export function indexPageReadModelResults(
  results: PageReadModelQueryResult[],
): BatchOnePageReadModelResults {
  const indexed: BatchOnePageReadModelResults = {};
  for (const result of results) {
    if (isBatchOneWorkbenchPageId(result.requested_page_id)) {
      indexed[result.requested_page_id] = result;
    }
  }
  return indexed;
}

export function snapshotFromPageReadModelResults(
  pageReadModels: BatchOnePageReadModelResults,
  fallback: WorkbenchSnapshot = emptySnapshot,
): WorkbenchSnapshot {
  const snapshot: WorkbenchSnapshot = { ...fallback };
  for (const pageId of batchOneWorkbenchPageIds) {
    const slice = snapshotSliceFromPageReadModel(pageReadModels[pageId]);
    if (slice) Object.assign(snapshot, slice);
  }
  return snapshot;
}

export function pageReadModelSnapshotWarnings(pageReadModels: BatchOnePageReadModelResults): string[] {
  return batchOneWorkbenchPageIds.flatMap((pageId) => {
    const result = pageReadModels[pageId];
    if (!result) return [`missing_page_read_model:${pageId}`];
    if (result.status !== "page_data_ready") return [`page_read_model_not_ready:${pageId}:${result.status}`];
    if (result.page_payload?.generated_from !== "workbench_page_query") {
      return [`page_read_model_unexpected_payload_source:${pageId}`];
    }
    if (!isRecord(result.page_payload.data.snapshot_slice)) {
      return [`page_read_model_missing_snapshot_slice:${pageId}`];
    }
    return [];
  });
}

function snapshotSliceFromPageReadModel(
  result: PageReadModelQueryResult | undefined,
): Partial<WorkbenchSnapshot> | null {
  if (!result || result.status !== "page_data_ready") return null;
  if (result.page_payload?.generated_from !== "workbench_page_query") return null;
  const data = result.page_payload.data;
  const slice = data.snapshot_slice;
  if (!isRecord(slice)) return null;
  return slice as Partial<WorkbenchSnapshot>;
}

function isBatchOneWorkbenchPageId(pageId: string): pageId is BatchOneWorkbenchPageId {
  return batchOneWorkbenchPageIds.includes(pageId as BatchOneWorkbenchPageId);
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}
