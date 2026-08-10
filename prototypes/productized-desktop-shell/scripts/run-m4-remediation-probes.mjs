import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const shellRoot = join(scriptDirectory, "..");
const repositoryRoot = join(shellRoot, "..", "..");

const argumentValue = (name, fallback) => {
  const prefix = `--${name}=`;
  const argument = process.argv.slice(2).find((candidate) => candidate.startsWith(prefix));
  return argument ? argument.slice(prefix.length) : fallback;
};

const expectedState = argumentValue("expect", "current").toLowerCase();
const selectedProbe = argumentValue("only", "all").toLowerCase();

if (!["current", "red", "green"].includes(expectedState)) {
  throw new Error("--expect must be current, red, or green");
}

const fileCache = new Map();
async function source(relativePath) {
  if (!fileCache.has(relativePath)) {
    try {
      fileCache.set(relativePath, await readFile(join(repositoryRoot, relativePath), "utf8"));
    } catch (error) {
      if (error?.code !== "ENOENT") throw error;
      fileCache.set(relativePath, "");
    }
  }
  return fileCache.get(relativePath);
}

async function sha256(relativePath) {
  const content = await readFile(join(repositoryRoot, relativePath));
  return createHash("sha256").update(content).digest("hex");
}

async function sha256OrMissing(relativePath) {
  try {
    return await sha256(relativePath);
  } catch (error) {
    if (error?.code !== "ENOENT") throw error;
    return null;
  }
}

const paths = {
  app: "prototypes/productized-desktop-shell/src/App.tsx",
  commandRegistry: "prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs",
  commands: "prototypes/productized-desktop-shell/src-tauri/src/commands.rs",
  conversationRuntime:
    "prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_conversation.rs",
  legacyReaders:
    "prototypes/productized-desktop-shell/src-tauri/src/m4_legacy_readers.rs",
  lib: "prototypes/productized-desktop-shell/src-tauri/src/lib.rs",
  proposalStore:
    "prototypes/productized-desktop-shell/src-tauri/src/project_consultation_proposal_store.rs",
  projectsView: "prototypes/productized-desktop-shell/src/views/ProjectsView.tsx",
  readModel:
    "prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_read_model.rs",
  repository:
    "prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_repository.rs",
  routeResolver:
    "prototypes/productized-desktop-shell/src-tauri/src/m4_source_route_resolver.rs",
  sourceDispatcher:
    "prototypes/productized-desktop-shell/src-tauri/src/m4_source_dispatcher.rs",
  tauri: "prototypes/productized-desktop-shell/src/lib/tauri.ts",
  workbenchShell:
    "prototypes/productized-desktop-shell/src/components/WorkbenchShell.tsx",
  workflowDispatch:
    "prototypes/productized-desktop-shell/src-tauri/src/workflow_run_dispatch_entrypoints.rs",
};

const checks = [];
function literal(id, relativePath, needle, explanation) {
  checks.push({ id, relativePath, mode: "PRESENT", needle, explanation });
}

function absent(id, relativePath, needle, explanation) {
  checks.push({ id, relativePath, mode: "ABSENT", needle, explanation });
}

function absentBetween(id, relativePath, start, end, needle, explanation) {
  checks.push({
    id,
    relativePath,
    mode: "ABSENT_BETWEEN",
    start,
    end,
    needle,
    explanation,
  });
}

literal(
  "source.work-item-command-registration",
  paths.commandRegistry,
  "update_work_item_state",
  "required marker: ordinary WorkItem command registration",
);
literal(
  "source.proposal-create-command-registration",
  paths.commandRegistry,
  "create_project_consultation_proposal",
  "required marker: ordinary proposal create command registration",
);
literal(
  "source.proposal-decision-command-registration",
  paths.commandRegistry,
  "record_project_consultation_proposal_decision",
  "required marker: ordinary proposal decision command registration",
);
literal(
  "source.work-item-owner-revision-marker",
  paths.workflowDispatch,
  "m2_result.receipt.committed_revision",
  "required marker: WorkItem owner committed revision read",
);
literal(
  "source.work-item-owner-event-marker",
  paths.workflowDispatch,
  "m2_result.event.event_id",
  "required marker: WorkItem owner event id read",
);
literal(
  "source.work-item-publication-edge",
  paths.workflowDispatch,
  "append_m4_work_item_source_publication",
  "required marker: WorkItem owner transaction publication call",
);
literal(
  "source.provenance-validator",
  paths.sourceDispatcher,
  "validate_work_item_source_provenance",
  "required marker: owner revision/event/watermark provenance validator",
);
literal(
  "source.work-item-consumer-checkpoint",
  paths.sourceDispatcher,
  "M4WorkItemSourceConsumerCheckpointV1",
  "required marker: WorkItem M4 consumer checkpoint type",
);
literal(
  "source.registered-owner-mapper",
  paths.sourceDispatcher,
  "RegisteredWorkItemSourceOwnerMapper",
  "required marker: registered WorkItem owner mapper",
);
literal(
  "source.adapter-ingest-edge",
  paths.sourceDispatcher,
  "ingest_workflow_attention_source",
  "required marker: registered mapper to M4 ingestion edge",
);
literal(
  "source.proposal-decision-outbox",
  paths.proposalStore,
  "M4SourceOwnerOutboxEnvelopeV1",
  "required marker: proposal typed Decision source envelope",
);
literal(
  "source.production-dispatcher",
  paths.lib,
  "dispatch_pending_m4_source_owner_outbox",
  "required marker: ordinary AppState dispatcher startup/tail drain",
);

literal(
  "clock.scheduler-entry",
  paths.lib,
  "start_m4_secretary_scheduler",
  "required marker: ordinary server scheduler entry",
);
literal(
  "clock.due-batch-definition",
  paths.repository,
  "run_due_transition_batch",
  "required marker: deterministic due-transition batch",
);
literal(
  "clock.production-caller",
  paths.repository,
  "self.run_due_transition_batch",
  "required marker: scheduler-cycle to due-batch edge",
);
literal(
  "clock.open-loop-server-reason",
  paths.repository,
  "open_loop_transition_reason",
  "required marker: explicit OpenLoop transition reason selection",
);
literal(
  "clock.reminder-server-reason",
  paths.repository,
  "reminder_transition_reason",
  "required marker: explicit Reminder transition reason selection",
);

literal(
  "route.app-state-registry",
  paths.lib,
  "m4_source_route_registry",
  "required marker: AppState-installed source-owner route registry",
);
literal(
  "route.registered-owner-registry",
  paths.routeResolver,
  "M4RegisteredSourceOwnerRouteRegistry",
  "required marker: registered owner resolver registry",
);
literal(
  "route.revision-target-validator",
  paths.routeResolver,
  "validate_current_source_revision_and_target",
  "required marker: current revision and target validator",
);
literal(
  "route.finite-target-enum",
  paths.routeResolver,
  "M4SourceNavigationTarget",
  "required marker: finite typed navigation target",
);
literal(
  "route.server-command",
  paths.commands,
  "resolve_secretary_source_route",
  "required marker: ordinary resolver command",
);
literal(
  "route.command-registration",
  paths.commandRegistry,
  "resolve_secretary_source_route",
  "required marker: resolver command registration",
);
literal(
  "route.renderer-client",
  paths.tauri,
  "resolveSecretarySourceRoute",
  "required marker: strict renderer resolver client",
);
literal(
  "route.owner-focus-consumer",
  paths.projectsView,
  "secretarySourceFocus",
  "required marker: owner page typed focus consumer",
);
literal(
  "route.app-resolver-call",
  paths.app,
  "resolveSecretarySourceRoute",
  "required marker: App resolver call",
);
literal(
  "route.failure-marker",
  paths.app,
  "SECRETARY_SOURCE_ROUTE_RESOLUTION_FAILED",
  "required marker: explicit route-resolution failure branch",
);
absentBetween(
  "route.old-project-guess-absent",
  paths.app,
  "const openSecretaryDeepLink",
  "const operateSecretaryAction",
  'navigate("projects"',
  "forbidden marker absent: renderer Projects guess inside Secretary deep-link handler",
);

literal(
  "conversation.app-state-runtime",
  paths.lib,
  "m4_secretary_conversation_runtime",
  "required marker: fixed Secretary conversation runtime in AppState",
);
literal(
  "conversation.m3-turn-edge",
  paths.conversationRuntime,
  "start_role_turn",
  "required marker: existing M3 Turn lifecycle edge",
);
literal(
  "conversation.transcript-read-join",
  paths.conversationRuntime,
  "read_secretary_transcript",
  "required marker: provider-owned transcript read join",
);
literal(
  "conversation.load-command",
  paths.commands,
  "load_secretary_conversation",
  "required marker: ordinary conversation load command",
);
literal(
  "conversation.send-command",
  paths.commands,
  "send_secretary_message",
  "required marker: ordinary explicit-send command",
);
literal(
  "conversation.load-command-registration",
  paths.commandRegistry,
  "load_secretary_conversation",
  "required marker: load command registration",
);
literal(
  "conversation.send-command-registration",
  paths.commandRegistry,
  "send_secretary_message",
  "required marker: send command registration",
);
literal(
  "conversation.load-renderer-client",
  paths.tauri,
  "loadSecretaryConversation",
  "required marker: strict renderer load client",
);
literal(
  "conversation.send-renderer-client",
  paths.tauri,
  "sendSecretaryMessage",
  "required marker: strict renderer send client",
);
literal(
  "conversation.app-send-edge",
  paths.app,
  "sendSecretaryMessage",
  "required marker: App explicit-send edge",
);
literal(
  "conversation.enabled-composer",
  paths.workbenchShell,
  "onSendSecretaryMessage",
  "required marker: Workbench controlled composer callback",
);
absent(
  "conversation.unavailable-placeholder-absent",
  paths.workbenchShell,
  "持续消息发送尚未接入",
  "forbidden marker absent: unavailable composer placeholder",
);
absent(
  "conversation.unavailable-status-absent",
  paths.workbenchShell,
  "消息发送未接入",
  "forbidden marker absent: unavailable composer status",
);

literal(
  "legacy.app-state-registry",
  paths.lib,
  "m4_legacy_read_registry",
  "required marker: AppState-installed legacy reader registry",
);
literal(
  "legacy.five-reader-registry",
  paths.legacyReaders,
  "M4LegacyReadRegistry",
  "required marker: five-source server reader registry",
);
literal(
  "legacy.server-owned-reader",
  paths.legacyReaders,
  "read_server_owned_legacy_candidates",
  "required marker: server-owned candidate reader entry",
);
literal(
  "legacy.ordinary-command-caller",
  paths.commands,
  "read_server_owned_legacy_candidates",
  "required marker: ordinary command to reader-registry edge",
);
literal(
  "legacy.parity-source-reader",
  paths.legacyReaders,
  "WorkItemLegacyShadowReader",
  "required marker: WorkItem-backed shadow reader",
);
literal(
  "legacy.canonical-comparator",
  paths.readModel,
  "build_m4_legacy_shadow_parity_report",
  "required marker: canonical parity comparator",
);
literal(
  "legacy.guarded-fallback",
  paths.readModel,
  "guarded_m4_legacy_read_only_fallback",
  "required marker: guarded read-only fallback",
);
absentBetween(
  "legacy.inventory-only-caller-absent",
  paths.commands,
  "async fn load_secretary_legacy_read_compatibility_report",
  "async fn load_secretary_daily_report",
  "m4_legacy_read_inventory_only_candidates",
  "forbidden marker absent: inventory-only candidate helper in ordinary command",
);

const probeDefinitions = [
  {
    id: "source",
    p1: "P1-A",
    title: "ordinary source owner and personal-object composition",
    checkIds: checks.filter((check) => check.id.startsWith("source.")).map((check) => check.id),
  },
  {
    id: "clock",
    p1: "P1-B",
    title: "server due clock and recovery",
    checkIds: checks.filter((check) => check.id.startsWith("clock.")).map((check) => check.id),
  },
  {
    id: "route",
    p1: "P1-C",
    title: "registered owner exact source return",
    checkIds: checks.filter((check) => check.id.startsWith("route.")).map((check) => check.id),
  },
  {
    id: "conversation",
    p1: "P1-D",
    title: "persistent Secretary conversation",
    checkIds: checks.filter((check) => check.id.startsWith("conversation.")).map((check) => check.id),
  },
  {
    id: "legacy",
    p1: "P1-E",
    title: "real legacy shadow/parity/fallback readers",
    checkIds: checks.filter((check) => check.id.startsWith("legacy.")).map((check) => check.id),
  },
];

if (selectedProbe !== "all" && !probeDefinitions.some((probe) => probe.id === selectedProbe)) {
  throw new Error(`unknown --only probe: ${selectedProbe}`);
}

const selectedDefinitions = probeDefinitions.filter(
  (probe) => selectedProbe === "all" || probe.id === selectedProbe,
);
const selectedCheckIds = new Set(selectedDefinitions.flatMap((probe) => probe.checkIds));

const evaluatedChecks = [];
for (const check of checks.filter((candidate) => selectedCheckIds.has(candidate.id))) {
  const content = await source(check.relativePath);
  let passed = false;
  if (check.mode === "PRESENT") {
    passed = content.includes(check.needle);
  } else if (check.mode === "ABSENT") {
    passed = !content.includes(check.needle);
  } else if (check.mode === "ABSENT_BETWEEN") {
    const startIndex = content.indexOf(check.start);
    const endIndex = startIndex < 0 ? -1 : content.indexOf(check.end, startIndex + check.start.length);
    passed =
      startIndex >= 0 &&
      endIndex > startIndex &&
      !content.slice(startIndex, endIndex).includes(check.needle);
  }
  evaluatedChecks.push({
    id: check.id,
    path: check.relativePath,
    marker_mode: check.mode,
    passed,
    explanation: check.explanation,
  });
}

const probes = selectedDefinitions.map((probe) => {
  const probeChecks = evaluatedChecks.filter((check) => probe.checkIds.includes(check.id));
  return {
    id: probe.id,
    p1: probe.p1,
    title: probe.title,
    status: probeChecks.every((check) => check.passed) ? "GREEN" : "RED",
    checks: probeChecks,
  };
});

const frozenContracts = {
  "docs/contracts/role-session-v1.md":
    "77c82932e728d4982ebb501b167f274cc31d2076957602771904d96dc399b2ca",
  "docs/contracts/handoff-v1.md":
    "3378f02f5dfb06e4db39125b5828eeda9440fc2c25ddbee3fe4e951fa6c386bf",
  "docs/contracts/identity-scope-v1.md":
    "3cb0073c0fffc2423e3450ce9d9e3c683065cdd075bf618e0d406cc1475e3ea4",
  "docs/contracts/event-audit-outbox-v1.md":
    "15a24d8040da054794e340fe7839b273dce0f60a2c1708513d1b998c8e968e99",
  "docs/contracts/m3-role-session-turn-handoff-resolution-v1.md":
    "946c756b30a8e73aaad441e49ba39a5c9cbd7c7d47241ed97fa19d02783bac48",
  "docs/contracts/m4-secretary-attention-daily-resolution-v1.md":
    "4e4d6251d53e1b9b156fb2fd1266d73d6beace38be2086e83e0f05694dec4e51",
};

const frozenContractEvidence = [];
for (const [relativePath, expectedSha256] of Object.entries(frozenContracts)) {
  const actualSha256 = await sha256(relativePath);
  frozenContractEvidence.push({
    path: relativePath,
    expected_sha256: expectedSha256,
    actual_sha256: actualSha256,
    exact: expectedSha256 === actualSha256,
  });
}

const inputPaths = [...new Set(evaluatedChecks.map((check) => check.path))].sort();
const callGraphInputs = [];
for (const relativePath of inputPaths) {
  const inputSha256 = await sha256OrMissing(relativePath);
  callGraphInputs.push({
    path: relativePath,
    sha256: inputSha256,
    ...(inputSha256 === null ? { disposition: "MISSING_AT_RED_BASELINE" } : {}),
  });
}

const receipt = {
  schema_version: "syn.m4.remediation-probes.receipt.v1",
  red_baseline_commit: "7f9c6da717f0ec49c22fcd76327431fcfff0cb4e",
  selected_probe: selectedProbe,
  expected_state: expectedState.toUpperCase(),
  status_semantics: "ALL_REQUIRED_STATIC_MARKERS_MATCH_ONLY",
  evidence_limit:
    "STATIC_LITERAL_MARKER_PRESENCE_ONLY; no reachability, edge, behavior, or bypass absence beyond explicit ABSENT checks is proven",
  probe_script_sha256: await sha256(
    "prototypes/productized-desktop-shell/scripts/run-m4-remediation-probes.mjs",
  ),
  addendum_sha256: await sha256(
    "docs/contracts/m4-independent-remediation-addendum-v1.md",
  ),
  probes: probes.map((probe) => ({
    id: probe.id,
    p1: probe.p1,
    title: probe.title,
    status: probe.status,
    passed_checks: probe.checks.filter((check) => check.passed).map((check) => check.id),
    missing_checks: probe.checks.filter((check) => !check.passed).map((check) => check.id),
  })),
  frozen_contracts_exact: frozenContractEvidence.every((entry) => entry.exact),
  frozen_contracts: frozenContractEvidence,
  call_graph_inputs: callGraphInputs,
};

process.stdout.write(`${JSON.stringify(receipt, null, 2)}\n`);

const probeExpectationMet =
  expectedState === "current" ||
  probes.every((probe) => probe.status === expectedState.toUpperCase());
const contractsExact = receipt.frozen_contracts_exact;
if (!probeExpectationMet || !contractsExact) {
  process.exitCode = 1;
}
