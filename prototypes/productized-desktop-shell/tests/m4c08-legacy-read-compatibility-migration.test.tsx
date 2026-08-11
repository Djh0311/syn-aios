import { renderToStaticMarkup } from "react-dom/server.browser";
import { RightDetailPanel } from "../src/components/RightDetailPanel";
import { SecretaryBoardView } from "../src/components/SecretaryBoardView";
import { SecretaryBrief } from "../src/components/SecretaryBrief";
import { emptySnapshot } from "../src/lib/emptySnapshot";
import {
  M4_LEGACY_READ_SOURCE_KINDS,
  canOperateSecretaryReadModel,
  createSecretaryGuardedLegacyReadOnlyFallback,
  deriveSecretaryHomeReadModel,
  emptySecretaryContext,
  isSecretaryLegacyReadFallback,
  isSecretarySourceOwnerResolved,
  parseSecretaryLegacyReadCompatibilityReportEnvelope,
  parseSecretaryHomeContextEnvelope,
  shouldUseSecretaryLegacyReadFallback,
} from "../src/lib/secretaryReadModel";
import { assert } from "./helpers/offlineInteractionTestUtils";

const hash = (character: string) => (/^[a-f0-9]$/.test(character) ? character : "a").repeat(64);
const opaque = (namespace: string, character: string) => `${namespace}:sha256:${hash(character)}`;

function readyEnvelope() {
  return {
    status: "READY",
    application_outcome: {
      context: {
        context_ref: "secretary-context:c08",
        role_session_ref: "role-session:c08",
        scope_ref: "scope:personal:primary",
        scope_source_watermark: hash("w"),
        snapshot_hash: hash("s"),
        reconstruction_code: "DETERMINISTIC_REBUILD",
      },
      deterministic_brief: {
        brief_ref: "secretary-brief:c08",
        brief_hash: hash("b"),
        context_ref: "secretary-context:c08",
        scope_source_watermark: hash("w"),
        attention_items: [{
          item_ref: "open-loop:c08",
          item_kind_code: "OPEN_LOOP",
          source_owner_ref: "owner:workflow",
          source_object_ref: "workflow:c08",
          source_object_type: "WORKFLOW",
          source_route_ref: opaque("route", "a"),
          source_summary_ref: opaque("summary", "b"),
          why_code: "WAITING_FOR_OWNER",
          priority_rank: 1,
          priority_code: "EXTERNAL_COMMITMENT",
          status_code: "OPEN",
          source_status_code: "ACTIVE",
          coordination_revision: "8",
          due_at_utc: null,
          last_change_at_utc: "2026-08-10T10:00:00Z",
          change_hash: hash("a"),
        }],
        personal_actions: [],
      },
      local_objects: {
        personal_actions: [],
        notifications: [],
        reminders: [],
        decisions: [],
        reminder_owner_refs: [],
      },
      model_enhancement: null,
    },
  };
}

function guardedReportEnvelope() {
  const scopeWatermark = hash("c");
  const canonicalSource = {
    source_owner_ref: "workflow-owner",
    scope_ref: "scope:personal:primary",
    source_type: "structured_internal_workflow_attention_ref",
    canonical_source_object_id: "workflow-attention:c08",
    source_revision: "18446744073709551615",
    source_owner_watermark: opaque("owner-watermark", "d"),
    source_link: {
      link_kind: "INTERNAL_ROUTE",
      source_owner_ref: "workflow-owner",
      object_type: "workflow_attention",
      canonical_source_object_id: "workflow-attention:c08",
      expected_source_revision: "18446744073709551615",
      opaque_route_ref: opaque("source-route", "e"),
    },
    source_status_code: "WAITING_USER",
    priority_reason_code: "USER_DECISION_OR_BLOCKER",
  };
  const parityRow = {
    legacy_source_kind: "RIGHT_RAIL_NOTIFICATION_AND_TODO_PROJECTION",
    legacy_item_ref: opaque("legacy-item", "f"),
    disposition: "PARITY",
    reason_code: null,
    canonical_source: canonicalSource,
    canonical_scope_source_watermark: scopeWatermark,
    source_matches: true,
    status_matches: true,
    priority_reason_matches: true,
    source_owner_watermark_matches: true,
    scope_source_watermark_matches: true,
    dedupe_key: `legacy-dedupe:${hash("1")}`,
    dedupe_disposition: "PRIMARY",
  };
  return {
    status: "READY",
    report: {
      schema_version: "syn.m4.secretary.legacy-read-compatibility.v1",
      parity_matrix_version: "syn.m4.legacy-read-parity/v1",
      mode: "M4_PRIMARY_LEGACY_READ_ONLY_FALLBACK",
      rollback_mode: "GUARDED_LEGACY_READ_ONLY",
      scope_ref: "scope:personal:primary",
      scope_source_watermark: scopeWatermark,
      inventory: M4_LEGACY_READ_SOURCE_KINDS.map((legacy_source_kind) => ({
        legacy_source_kind,
        compatibility_role: "SOURCE_REF_AND_DEDUPE_CANDIDATE_ONLY",
        write_authority: "NONE",
      })),
      reader_receipts: [
        {
          legacy_source_kind: "SECRETARY_READ_MODEL_DETERMINISTIC_SUMMARY",
          reader_id: "m4-legacy-reader:secretary-read-model/v1",
          source_surface_code: "SERVER_LEGACY_SECRETARY_READ_MODEL_PRIMITIVES",
          read_state: "EMPTY",
          reason_code: "M4R06_EMPTY_SERVER_SURFACE",
          legacy_reader_adapter_id: null,
          candidate_count: 0,
          complete_tuple_count: 0,
        },
        {
          legacy_source_kind: "RIGHT_RAIL_NOTIFICATION_AND_TODO_PROJECTION",
          reader_id: "m4-legacy-reader:right-rail-work-item/v1",
          source_surface_code: "M2_WORK_ITEM_RIGHT_RAIL_PROJECTION",
          read_state: "OBSERVED",
          reason_code: null,
          legacy_reader_adapter_id: "registered-work-item-source-owner-mapper.v1",
          candidate_count: 3,
          complete_tuple_count: 3,
        },
        {
          legacy_source_kind: "RUNTIME_ATTENTION_PROJECTION",
          reader_id: "m4-legacy-reader:runtime-attention/v1",
          source_surface_code: "SERVER_RUNTIME_ATTENTION_PROJECTION",
          read_state: "UNJOINABLE",
          reason_code: "M4R06_UNJOINABLE_NO_EXACT_TUPLE",
          legacy_reader_adapter_id: null,
          candidate_count: 1,
          complete_tuple_count: 0,
        },
        {
          legacy_source_kind: "REACT_PENDING_ACTION_VISIBILITY",
          reader_id: "m4-legacy-reader:react-pending-action/v1",
          source_surface_code: "RENDERER_LOCAL_PENDING_ACTION_VISIBILITY",
          read_state: "QUARANTINED",
          reason_code: "M4R06_READER_REJECTED",
          legacy_reader_adapter_id: null,
          candidate_count: 1,
          complete_tuple_count: 0,
        },
        {
          legacy_source_kind: "MEMORY_DAILY_INBOX_CANDIDATE",
          reader_id: "m4-legacy-reader:memory-daily-inbox/v1",
          source_surface_code: "SERVER_MEMORY_DAILY_CANDIDATE_STORE",
          read_state: "EMPTY",
          reason_code: "M4R06_EMPTY_SERVER_SURFACE",
          legacy_reader_adapter_id: null,
          candidate_count: 0,
          complete_tuple_count: 0,
        },
      ],
      rows: [
        parityRow,
        {
          ...parityRow,
          legacy_source_kind: "RIGHT_RAIL_NOTIFICATION_AND_TODO_PROJECTION",
          legacy_item_ref: opaque("legacy-item", "2"),
          dedupe_disposition: "DUPLICATE_DISPLAY_ONLY",
        },
        {
          legacy_source_kind: "RIGHT_RAIL_NOTIFICATION_AND_TODO_PROJECTION",
          legacy_item_ref: opaque("legacy-item", "3"),
          disposition: "QUARANTINED",
          reason_code: "M4C08_CANONICAL_SOURCE_NOT_FOUND",
          canonical_source: null,
          canonical_scope_source_watermark: null,
          source_matches: false,
          status_matches: false,
          priority_reason_matches: false,
          source_owner_watermark_matches: false,
          scope_source_watermark_matches: false,
          dedupe_key: null,
          dedupe_disposition: "NOT_ELIGIBLE",
        },
      ],
    },
  };
}

function assertRejects(action: () => unknown, message: string) {
  let rejected = false;
  try {
    action();
  } catch {
    rejected = true;
  }
  assert(rejected, message);
}

function countMarkup(markup: string, fragment: string): number {
  return markup.split(fragment).length - 1;
}

// 1) READY is always the normal M4 primary, even if a guarded legacy report
// is already available in memory.
{
  const guarded = createSecretaryGuardedLegacyReadOnlyFallback(
    parseSecretaryLegacyReadCompatibilityReportEnvelope(guardedReportEnvelope()),
  );
  const model = deriveSecretaryHomeReadModel({
    home_context: parseSecretaryHomeContextEnvelope(readyEnvelope()),
    compatibility: guarded,
  });
  assert(model.source_authority === "M4_APPLICATION_SERVICE", "READY M4 home stays primary over C08 compatibility");
  assert(model.scope_source_watermark === hash("w"), "primary read retains the canonical server watermark");
  assert(!isSecretaryLegacyReadFallback(model), "primary read does not activate the legacy surface");
  assert(!shouldUseSecretaryLegacyReadFallback(parseSecretaryHomeContextEnvelope(readyEnvelope())), "READY never selects the legacy read surface");
  assert(model.attention_items.every(isSecretarySourceOwnerResolved), "server-owned source refs remain the primary actionable routes");
  assert(canOperateSecretaryReadModel(model), "only the primary M4 read may expose coordination writes");
}

// 2) Renderer-branch evidence only: a parsed UNAVAILABLE home plus a parsed
// READY guarded report shows only exact PARITY + PRIMARY. This does not start
// or claim an actual-product-App fallback run. Duplicate display-only and
// quarantined candidates neither appear as active items nor create writes.
{
  const report = parseSecretaryLegacyReadCompatibilityReportEnvelope(guardedReportEnvelope());
  if (report.status !== "READY") throw new Error("fixture must parse as a ready guarded report");
  const guarded = createSecretaryGuardedLegacyReadOnlyFallback(report);
  if (!guarded) throw new Error("fixture must create guarded fallback");
  const unavailable = parseSecretaryHomeContextEnvelope({ status: "UNAVAILABLE", reason: "M4_CONTEXT_UNAVAILABLE" });
  const fallback = deriveSecretaryHomeReadModel({ home_context: unavailable, compatibility: guarded });
  assert(isSecretaryLegacyReadFallback(fallback), "only the guarded report activates the read-only surface");
  assert(fallback.source_authority === "CANONICAL_SNAPSHOT_SUMMARY", "fallback remains a non-coordination source authority");
  assert(fallback.role_session_recovery.status === "UNAVAILABLE", "fallback never recreates a RoleSession or context");
  assert(fallback.scope_source_watermark === hash("c"), "guarded display retains the re-read scope watermark");
  assert(fallback.attention_items.length === 1, "only one PARITY + PRIMARY row renders; duplicate and quarantine do not");
  assert(fallback.attention_items[0]?.source_owner.source_owner_ref === "workflow-owner", "visible item uses the canonical re-read owner");
  assert(fallback.attention_items[0]?.source_status_code === "WAITING_USER", "visible item uses the canonical re-read status");
  assert(fallback.attention_items[0]?.priority_reason_code === "USER_DECISION_OR_BLOCKER", "visible item uses the canonical re-read priority reason");
  assert(isSecretarySourceOwnerResolved(fallback.attention_items[0]!), "guarded exact row retains only its typed canonical source link");
  assert(!canOperateSecretaryReadModel(fallback), "compatibility face cannot issue coordination writes");
  assert(
    report.report.reader_receipts.map((receipt) => receipt.legacy_source_kind).join(",") === M4_LEGACY_READ_SOURCE_KINDS.join(","),
    "five reader receipts remain one-per-kind in the frozen registry order",
  );
  assert(
    report.report.reader_receipts.find((receipt) => receipt.legacy_source_kind === "RIGHT_RAIL_NOTIFICATION_AND_TODO_PROJECTION")?.read_state === "OBSERVED",
    "only the server-owned work-item reader supplies the observed exact tuple receipt",
  );
  assert(
    report.report.reader_receipts.find((receipt) => receipt.legacy_source_kind === "RUNTIME_ATTENTION_PROJECTION")?.candidate_count === 1,
    "an UNJOINABLE receipt may faithfully retain a positive structural candidate count",
  );

  const brief = renderToStaticMarkup(<SecretaryBrief home={fallback} onOpenDeepLink={() => undefined} />);
  const board = renderToStaticMarkup(<SecretaryBoardView home={fallback} onOpenDeepLink={() => undefined} />);
  const rightRail = renderToStaticMarkup(
    <RightDetailPanel
      activePanel="secretary"
      snapshot={emptySnapshot}
      workflowState={null}
      notice=""
      error={false}
      workflowStateError={null}
      secretaryContext={emptySecretaryContext}
      secretaryHome={fallback}
      onClose={() => undefined}
      onNavigate={() => undefined}
      onReloadWorkflowState={() => undefined}
    />,
  );
  assert(countMarkup(board, 'data-secretary-compatibility-fallback="true"') === 1, "parsed UNAVAILABLE home + READY report makes one renderer fallback root visible");
  assert(countMarkup(board, 'data-secretary-source-route-action="OPEN"') === 1, "fallback exposes exactly one PARITY + PRIMARY route control");
  assert(!board.includes("secretary-brief-source-link"), "fallback summary stays text-only and does not mirror a disabled second source control");
  assert(brief.includes("只读兼容回退") && board.includes("只读兼容回退"), "compatibility faces label the guarded read-only surface");
  assert(brief.includes("workflow-owner") && board.includes("workflow-owner"), "only canonical owner reference reaches the UI");
  assert(brief.includes("来源") && board.includes("回到来源"), "exact guarded rows may open only their typed source route");
  assert(!brief.includes("M4C08_CANONICAL_SOURCE_NOT_FOUND") && !board.includes("M4C08_CANONICAL_SOURCE_NOT_FOUND"), "quarantine report rows do not render");
  assert(!board.includes("m4-legacy-reader:") && !board.includes("M4R06_READER_REJECTED"), "reader receipt evidence never becomes renderer text");
  assert(!rightRail.includes("旧摘要风险") && !board.includes("旧摘要风险"), "raw old summary text never reaches the active compatibility UI");
  assert(!brief.includes("标为已读") && !board.includes("标为已读"), "compatibility UI exposes no coordination write entry");
  assert(!board.includes("data-secretary-action=") && !board.includes("data-secretary-personal-action="), "fallback renderer branch contains zero write actions");
}

// 3) Strict response parsing rejects an owner/link/watermark drift and refuses
// to turn a malformed response into an active compatibility item.
{
  const mismatchedWatermark = guardedReportEnvelope();
  mismatchedWatermark.report.rows[0]!.canonical_scope_source_watermark = hash("9");
  assertRejects(
    () => parseSecretaryLegacyReadCompatibilityReportEnvelope(mismatchedWatermark),
    "PARITY row must carry the report scope watermark",
  );
  const unsafeRoute = guardedReportEnvelope();
  const unsafeCanonicalSource = unsafeRoute.report.rows[0]?.canonical_source;
  if (!unsafeCanonicalSource) throw new Error("fixture must include canonical source");
  unsafeCanonicalSource.source_link.opaque_route_ref = "https://example.invalid/raw-route";
  assertRejects(
    () => parseSecretaryLegacyReadCompatibilityReportEnvelope(unsafeRoute),
    "raw route-shaped source link is rejected",
  );
  const overU64Revision = guardedReportEnvelope();
  const overU64CanonicalSource = overU64Revision.report.rows[0]?.canonical_source;
  if (!overU64CanonicalSource) throw new Error("fixture must include canonical source");
  overU64CanonicalSource.source_revision = "18446744073709551616";
  overU64CanonicalSource.source_link.expected_source_revision = "18446744073709551616";
  assertRejects(
    () => parseSecretaryLegacyReadCompatibilityReportEnvelope(overU64Revision),
    "source revision above u64 is rejected while parsing the report DTO",
  );
  const ambiguousLegacyIdentity = guardedReportEnvelope();
  const parityRow = ambiguousLegacyIdentity.report.rows[0];
  const parityCanonicalSource = parityRow?.canonical_source;
  if (!parityRow || !parityCanonicalSource) throw new Error("fixture must include a parity source");
  ambiguousLegacyIdentity.report.rows.push({
    ...parityRow,
    canonical_source: {
      ...parityCanonicalSource,
      source_owner_ref: "workflow-owner-other",
      canonical_source_object_id: "workflow-attention:c08-other",
      source_owner_watermark: opaque("owner-watermark", "8"),
      source_link: {
        ...parityCanonicalSource.source_link,
        source_owner_ref: "workflow-owner-other",
        canonical_source_object_id: "workflow-attention:c08-other",
        opaque_route_ref: opaque("source-route", "9"),
      },
    },
    dedupe_key: `legacy-dedupe:${hash("8")}`,
    dedupe_disposition: "PRIMARY",
  });
  assertRejects(
    () => parseSecretaryLegacyReadCompatibilityReportEnvelope(ambiguousLegacyIdentity),
    "one legacy identity cannot carry PARITY + PRIMARY rows for two canonical sources",
  );
  const reorderedReaderReceipts = guardedReportEnvelope();
  const firstReceipt = reorderedReaderReceipts.report.reader_receipts[0];
  reorderedReaderReceipts.report.reader_receipts[0] = reorderedReaderReceipts.report.reader_receipts[1]!;
  reorderedReaderReceipts.report.reader_receipts[1] = firstReceipt!;
  assertRejects(
    () => parseSecretaryLegacyReadCompatibilityReportEnvelope(reorderedReaderReceipts),
    "reader receipts must retain the one-per-kind frozen registry order",
  );
  const observedAdapterDrift = guardedReportEnvelope();
  observedAdapterDrift.report.reader_receipts[1]!.legacy_reader_adapter_id = null;
  assertRejects(
    () => parseSecretaryLegacyReadCompatibilityReportEnvelope(observedAdapterDrift),
    "the observed work-item receipt requires the registered server adapter",
  );
  const nonObservedAdapterDrift = guardedReportEnvelope();
  nonObservedAdapterDrift.report.reader_receipts[0]!.legacy_reader_adapter_id = "registered-work-item-source-owner-mapper.v1";
  assertRejects(
    () => parseSecretaryLegacyReadCompatibilityReportEnvelope(nonObservedAdapterDrift),
    "only an observed work-item receipt may name the registered adapter",
  );
  const observedCountDrift = guardedReportEnvelope();
  observedCountDrift.report.reader_receipts[1]!.complete_tuple_count = 1;
  assertRejects(
    () => parseSecretaryLegacyReadCompatibilityReportEnvelope(observedCountDrift),
    "observed work-item candidates require exact complete tuple counts",
  );
  const receiptRowCountDrift = guardedReportEnvelope();
  receiptRowCountDrift.report.reader_receipts[1]!.candidate_count = 2;
  receiptRowCountDrift.report.reader_receipts[1]!.complete_tuple_count = 2;
  assertRejects(
    () => parseSecretaryLegacyReadCompatibilityReportEnvelope(receiptRowCountDrift),
    "each receipt complete tuple count must equal its parsed source-kind row count",
  );
  const missingReceiptField = guardedReportEnvelope();
  Reflect.deleteProperty(missingReceiptField.report.reader_receipts[4]!, "candidate_count");
  assertRejects(
    () => parseSecretaryLegacyReadCompatibilityReportEnvelope(missingReceiptField),
    "every receipt must retain its exact frozen fields",
  );
  const invalidReceiptField = guardedReportEnvelope();
  Object.assign(invalidReceiptField.report.reader_receipts[2]!, { legacy_payload: "never rendered" });
  assertRejects(
    () => parseSecretaryLegacyReadCompatibilityReportEnvelope(invalidReceiptField),
    "reader receipt rejects an unknown candidate-like field",
  );
  const rawReport = guardedReportEnvelope();
  Object.assign(rawReport.report, { raw_transcript: "never rendered" });
  assertRejects(
    () => parseSecretaryLegacyReadCompatibilityReportEnvelope(rawReport),
    "unknown report payload is rejected",
  );
}

// 4) Keep the IPC registration and App wiring mechanical and reviewable in
// the offline suite without calling Tauri or a real database.
{
  const nodeProcess = (globalThis as typeof globalThis & { process?: { cwd?: () => string } }).process;
  if (!nodeProcess?.cwd) throw new Error("M4C08 offline protocol test requires Node cwd");
  const nodeFs = await import(("node:" + "fs") as string) as { readFileSync: (path: string, encoding: "utf8") => string };
  const root = nodeProcess.cwd();
  const tauriSource = nodeFs.readFileSync(`${root}/src/lib/tauri.ts`, "utf8");
  const appSource = nodeFs.readFileSync(`${root}/src/App.tsx`, "utf8");
  const commandsSource = nodeFs.readFileSync(`${root}/src-tauri/src/commands.rs`, "utf8");
  const registrySource = nodeFs.readFileSync(`${root}/src-tauri/src/command_registry.rs`, "utf8");
  const repositorySource = nodeFs.readFileSync(`${root}/src-tauri/src/m4_secretary_repository.rs`, "utf8");
  const readModelSource = nodeFs.readFileSync(`${root}/src-tauri/src/m4_secretary_read_model.rs`, "utf8");
  const legacyReadersSource = nodeFs.readFileSync(`${root}/src-tauri/src/m4_legacy_readers.rs`, "utf8");

  const facadeStart = tauriSource.indexOf("export async function loadSecretaryLegacyReadCompatibilityReport");
  const facadeEnd = tauriSource.indexOf("export async function loadSecretaryHomeContext", facadeStart);
  assert(facadeStart >= 0 && facadeEnd > facadeStart, "tauri facade has a bounded C08 report function");
  const facadeSource = tauriSource.slice(facadeStart, facadeEnd);
  assert(/export async function loadSecretaryLegacyReadCompatibilityReport\(\):\s*Promise<M4LegacyReadCompatibilityReportEnvelopeDto>/.test(facadeSource), "tauri facade accepts no renderer request DTO");
  assert(/invoke<unknown>\("load_secretary_legacy_read_compatibility_report"\)/.test(facadeSource), "tauri facade invokes the C08 command without a payload");
  for (const removedPublicType of [
    "M4LegacyReadCompatibilityRequestDto",
    "M4LegacyReadCandidateDto",
    "M4LegacyReadSourceLinkCandidateDto",
    "createSecretaryLegacyReadCompatibilityRequest",
    "parseSecretaryLegacyReadCompatibilityRequest",
  ]) {
    assert(!readModelSource.includes(removedPublicType) && !tauriSource.includes(removedPublicType), `C08 removes renderer ${removedPublicType}`);
  }
  for (const forbiddenRendererField of ["raw_transcript", "pendingAction", "pending_action", "cwd", "legacy_item_ref", "source_owner_ref", "scope_ref", "canonical_source_object_id", "reader_receipts", "identity"]) {
    assert(!facadeSource.includes(forbiddenRendererField), `C08 facade has no renderer ${forbiddenRendererField} IPC field`);
  }
  assert(/(?:^|\n)\s*load_secretary_legacy_read_compatibility_report\s*,/m.test(registrySource), "command registry registers the C08 read-only report command");
  const commandSignature = /#\[tauri::command\]\s*async fn load_secretary_legacy_read_compatibility_report\s*\(([\s\S]*?)\)\s*->\s*Result<m4_secretary_read_model::M4LegacyReadCompatibilityReportEnvelope,\s*String>/.exec(commandsSource);
  assert(commandSignature, "commands.rs declares the C08 report command with its frozen envelope");
  const parameters = commandSignature[1]!;
  assert(/^\s*state:\s*tauri::State<'_,\s*AppState>\s*,?\s*$/.test(parameters), "C08 command accepts only AppState");
  const commandStart = commandsSource.indexOf("async fn load_secretary_legacy_read_compatibility_report");
  const commandEnd = commandsSource.indexOf("async fn load_secretary_daily_report", commandStart);
  assert(commandStart >= 0 && commandEnd > commandStart, "C08 command body has a bounded ordinary-product region");
  const ordinaryCommandSource = commandsSource.slice(commandStart, commandEnd);
  const fixedInventoryKinds = /const ALL: \[Self; 5\] = \[([\s\S]*?)\];/.exec(readModelSource);
  assert(fixedInventoryKinds, "backend declares the fixed five-kind C08 inventory");
  for (const sourceKind of [
    "SecretaryReadModelDeterministicSummary",
    "RightRailNotificationAndTodoProjection",
    "RuntimeAttentionProjection",
    "ReactPendingActionVisibility",
    "MemoryDailyInboxCandidate",
  ]) {
    assert(fixedInventoryKinds[1].includes(`Self::${sourceKind}`), `backend fixed inventory contains ${sourceKind}`);
  }
  assert(!ordinaryCommandSource.includes("m4_legacy_read_inventory_only_candidates"), "R06 ordinary command no longer treats an empty inventory as legacy-reader truth");
  assert(
    ordinaryCommandSource.includes("m4_legacy_read_registry") && ordinaryCommandSource.includes("read_server_owned_legacy_candidates"),
    "R06 command obtains the server-owned reader batch only through AppState registry",
  );
  for (const readerMarker of [
    "pub(crate) struct M4LegacyReadRegistry",
    "read_server_owned_legacy_candidates",
    "M4LegacyServerOwnedReadBatch",
    "WorkItemLegacyShadowReader",
  ]) {
    assert(legacyReadersSource.includes(readerMarker), `R06 registry exposes ${readerMarker}`);
  }
  for (const receiptMarker of [
    "M4LegacyReaderReceipt",
    "M4LegacyReaderReadState",
    "reader_receipts",
    "M4R06_EMPTY_SERVER_SURFACE",
    "M4R06_UNJOINABLE_NO_EXACT_TUPLE",
    "M4R06_READER_UNAVAILABLE",
    "M4R06_READER_REJECTED",
  ]) {
    assert(readModelSource.includes(receiptMarker), `R06 report preserves fixed receipt marker ${receiptMarker}`);
  }
  assert(
    commandsSource.includes("read_legacy_read_compatibility_report") && repositorySource.includes("OpenFlags::SQLITE_OPEN_READ_ONLY"),
    "R06 report path is built on the repository read-only connection",
  );
  assert(
    appSource.includes("loadSecretaryLegacyReadCompatibilityReport()")
      && !appSource.includes("deriveSecretaryContext({")
      && !appSource.includes("reader_receipts"),
    "ordinary App uses only the zero-argument guarded report and never promotes receipt evidence to UI truth",
  );
}

console.log("m4c08-legacy-read-compatibility-migration: primary/guarded-parity/quarantine/strict-IPC/renderer-branch assertions passed");
