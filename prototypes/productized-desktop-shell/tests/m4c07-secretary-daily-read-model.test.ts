import { parseSecretaryDailyReportEnvelope } from "../src/lib/secretaryReadModel";
import { assert, assertDeepEqual } from "./helpers/offlineInteractionTestUtils";

const DAILY_SCHEMA_VERSION = "syn.m4.secretary.daily.v1";

function hash(character: string): string {
  if (!/^[a-f0-9]$/.test(character)) throw new Error("daily fixture hash digit must be lowercase hex");
  return character.repeat(64);
}

function deterministicRef(kind: string, character: string): string {
  return `${kind}:${hash(character)}`;
}

function timezoneRulesVersion(character: string): string {
  return `timezone-rules:${hash(character)}`;
}

function readyEnvelope() {
  const currentWindow = deterministicRef("daily-window", "a");
  const lastClosedWindow = deterministicRef("daily-window", "b");
  return {
    schema_version: DAILY_SCHEMA_VERSION,
    status: "READY",
    scheduler: {
      configuration_revision: "2",
      iana_timezone: "Asia/Shanghai",
      timezone_rules_version: timezoneRulesVersion("c"),
      current_daily_window_id: currentWindow,
      last_closed_daily_window_id: lastClosedWindow,
      catch_up_pending_count: 0,
      pending_catch_up_receipt_refs: [] as string[],
      status: "IDLE",
    },
    daily_brief: {
      daily_window_id: currentWindow,
      scope_source_watermark: hash("e"),
      projector_version: "1",
      // This is intentionally not ref-lexical order. It is the server's
      // priority/due/source-change projection order and must cross unchanged.
      ordered_item_refs: [
        deterministicRef("open-loop", "a"),
        deterministicRef("inbox", "b"),
      ],
      generated_at_utc: "2026-08-10T09:00:00Z",
    },
    daily_report: {
      daily_report_id: deterministicRef("daily-report", "c"),
      daily_window_id: lastClosedWindow,
      report_version: "1",
      status: "GENERATED",
      scope_source_watermark: hash("d"),
      projector_version: "1",
      ordered_item_refs: [
        deterministicRef("source-event", "d"),
        deterministicRef("personal-action", "e"),
      ],
      supersedes_report_ref: null,
      generated_at_utc: "2026-08-10T00:05:00Z",
    },
    last_run: {
      scheduler_run_id: deterministicRef("scheduler-run", "f"),
      configuration_revision: "2",
      window_ref: lastClosedWindow,
      scope_source_watermark_before: hash("c"),
      scope_source_watermark_after: hash("d"),
      admitted_material_event_count: 0,
      agent_turn_count: 0,
      model_invocation_count: 0,
      outcome_code: "EMPTY_WINDOW",
      recorded_at_utc: "2026-08-10T00:05:00Z",
    },
    recovery_code: null,
  };
}

function assertParseRejects(value: unknown, message: string) {
  let rejected = false;
  try {
    parseSecretaryDailyReportEnvelope(value);
  } catch {
    rejected = true;
  }
  assert(rejected, message);
}

function notReadyEnvelope(status: "UNAVAILABLE" | "DISABLED", reason: string) {
  return {
    schema_version: DAILY_SCHEMA_VERSION,
    status,
    reason,
  };
}

// 1) A valid ready envelope accepts service ordering exactly as projected; the
// parser must never sort refs lexically or mutate the server payload.
{
  const raw = readyEnvelope();
  const rawBefore = JSON.stringify(raw);
  const parsed = parseSecretaryDailyReportEnvelope(raw);
  if (parsed.status !== "READY") throw new Error("ready fixture must remain READY");

  assertDeepEqual(
    parsed.daily_brief.ordered_item_refs,
    [deterministicRef("open-loop", "a"), deterministicRef("inbox", "b")],
    "DailyBrief preserves the server's non-lexical priority order",
  );
  assertDeepEqual(
    parsed.daily_report.ordered_item_refs,
    [deterministicRef("source-event", "d"), deterministicRef("personal-action", "e")],
    "DailyReport preserves the server's non-lexical priority order",
  );
  assert(
    parsed.daily_brief.ordered_item_refs.join("|")
      !== [...parsed.daily_brief.ordered_item_refs].sort().join("|"),
    "fixture order must prove it is not ref-lexical order",
  );
  assert(JSON.stringify(raw) === rawBefore, "daily parser does not mutate a server-owned envelope");
}

// 2) Unsafe/ambiguous refs and any shape or source-binding drift fail closed.
{
  const duplicate = readyEnvelope();
  const firstBriefRef = duplicate.daily_brief.ordered_item_refs[0];
  if (!firstBriefRef) throw new Error("duplicate fixture must have a ref");
  duplicate.daily_brief.ordered_item_refs = [firstBriefRef, firstBriefRef];
  assertParseRejects(duplicate, "duplicate ordered refs are rejected");

  const unsafeRef = readyEnvelope();
  unsafeRef.daily_report.ordered_item_refs = ["https://example.invalid/raw-provider-body"];
  assertParseRejects(unsafeRef, "raw URL/provider-shaped refs are rejected");

  assertParseRejects(
    { ...readyEnvelope(), raw_transcript: "must never become a daily read field" },
    "unknown ready fields are rejected rather than ignored",
  );

  const invalidTimezoneRules = readyEnvelope();
  invalidTimezoneRules.scheduler.timezone_rules_version = "tzdb-2026a";
  assertParseRejects(invalidTimezoneRules, "timezone rules require the frozen opaque id format");

  const crossWindow = readyEnvelope();
  crossWindow.daily_brief.daily_window_id = deterministicRef("daily-window", "f");
  assertParseRejects(crossWindow, "DailyBrief must bind to the scheduler current window");

  const crossRunRef = readyEnvelope();
  crossRunRef.last_run.window_ref = crossRunRef.scheduler.current_daily_window_id;
  assertParseRejects(crossRunRef, "scheduler run must bind to the DailyReport window");

  const crossWatermark = readyEnvelope();
  crossWatermark.last_run.scope_source_watermark_after = hash("e");
  assertParseRejects(crossWatermark, "scheduler run after-watermark must bind to the DailyReport watermark");

  const nonCanonicalRevision = readyEnvelope();
  nonCanonicalRevision.scheduler.configuration_revision = "02";
  assertParseRejects(nonCanonicalRevision, "leading-zero u64 revisions are rejected");

  const invalidUtc = readyEnvelope();
  invalidUtc.daily_report.generated_at_utc = "2026-08-10T00:05:00+08:00";
  assertParseRejects(invalidUtc, "only canonical UTC timestamps are accepted");

  const missingReceipt = readyEnvelope();
  missingReceipt.scheduler.catch_up_pending_count = 1;
  assertParseRejects(missingReceipt, "pending catch-up count requires an opaque recovery receipt");

  const pendingCatchUp = readyEnvelope();
  pendingCatchUp.scheduler.catch_up_pending_count = 3;
  pendingCatchUp.scheduler.pending_catch_up_receipt_refs = [
    deterministicRef("catch-up-truncation", "f"),
  ];
  const parsedPendingCatchUp = parseSecretaryDailyReportEnvelope(pendingCatchUp);
  assert(
    parsedPendingCatchUp.status === "READY"
      && parsedPendingCatchUp.scheduler.pending_catch_up_receipt_refs[0]
        === deterministicRef("catch-up-truncation", "f"),
    "pending catch-up exposes only its deterministic server receipt",
  );

  const duplicateReceipt = readyEnvelope();
  duplicateReceipt.scheduler.catch_up_pending_count = 2;
  duplicateReceipt.scheduler.pending_catch_up_receipt_refs = [
    deterministicRef("catch-up-truncation", "f"),
    deterministicRef("catch-up-truncation", "f"),
  ];
  assertParseRejects(duplicateReceipt, "duplicate catch-up receipts are rejected");
}

// 3) An empty material window is a strict double-zero, but a material window
// does not invent a one-to-one relation between events, turns and model calls.
{
  const nonzeroTurnForEmptyWindow = readyEnvelope();
  nonzeroTurnForEmptyWindow.last_run.agent_turn_count = 1;
  assertParseRejects(nonzeroTurnForEmptyWindow, "empty material window rejects nonzero agent turns");

  const nonzeroModelForEmptyWindow = readyEnvelope();
  nonzeroModelForEmptyWindow.last_run.model_invocation_count = 1;
  assertParseRejects(nonzeroModelForEmptyWindow, "empty material window rejects nonzero model calls");

  const materialWindow = readyEnvelope();
  materialWindow.last_run.admitted_material_event_count = 3;
  materialWindow.last_run.agent_turn_count = 0;
  materialWindow.last_run.model_invocation_count = 5;
  const parsed = parseSecretaryDailyReportEnvelope(materialWindow);
  assert(
    parsed.status === "READY"
      && parsed.last_run.admitted_material_event_count === 3
      && parsed.last_run.agent_turn_count === 0
      && parsed.last_run.model_invocation_count === 5,
    "material windows keep independent event, turn and model-call counters",
  );
}

// 4) Unavailable and disabled are disjoint, scrubbed terminal read shapes.
{
  const unavailable = parseSecretaryDailyReportEnvelope(
    notReadyEnvelope("UNAVAILABLE", "M4_DAILY_STORAGE_UNAVAILABLE"),
  );
  const disabled = parseSecretaryDailyReportEnvelope(
    notReadyEnvelope("DISABLED", "SCHEDULER_CONFIGURATION_DISABLED"),
  );
  assert(
    unavailable.status === "UNAVAILABLE" && disabled.status === "DISABLED",
    "UNAVAILABLE and DISABLED remain distinct terminal states",
  );
  assertDeepEqual(
    Object.keys(unavailable).sort(),
    ["reason", "schema_version", "status"],
    "UNAVAILABLE exposes only its frozen scrubbed shape",
  );
  assertDeepEqual(
    Object.keys(disabled).sort(),
    ["reason", "schema_version", "status"],
    "DISABLED exposes only its frozen scrubbed shape",
  );
  assertParseRejects(
    { ...notReadyEnvelope("UNAVAILABLE", "M4_DAILY_STORAGE_UNAVAILABLE"), scheduler: readyEnvelope().scheduler },
    "not-ready shapes cannot carry ready-only fields",
  );
  assertParseRejects(
    notReadyEnvelope("UNAVAILABLE", "RAW_TRANSCRIPT"),
    "not-ready reasons must remain scrubbed codes",
  );
}

// 5) Keep the IPC contract offline: inspect sources instead of importing the
// Tauri runtime. The wrapper is selector-less and invokes only the exact name.
const nodeProcess = (globalThis as typeof globalThis & { process?: { cwd?: () => string } }).process;
if (!nodeProcess?.cwd) throw new Error("M4C07 offline protocol test requires Node cwd");
const nodeFsSpecifier: string = "node:fs";
const nodeFs = await import(nodeFsSpecifier) as { readFileSync: (path: string, encoding: "utf8") => string };
const root = nodeProcess.cwd();
const tauriSource = nodeFs.readFileSync(`${root}/src/lib/tauri.ts`, "utf8");
const registrySource = nodeFs.readFileSync(`${root}/src-tauri/src/command_registry.rs`, "utf8");
const commandsSource = nodeFs.readFileSync(`${root}/src-tauri/src/commands.rs`, "utf8");

const dailyWrapper = /export async function loadSecretaryDailyReport\(\): Promise<M4SecretaryDailyReportEnvelopeDto> \{[\s\S]*?\n\}/.exec(tauriSource);
assert(dailyWrapper, "tauri.ts exports loadSecretaryDailyReport with the daily DTO result");
assert(dailyWrapper[0].includes("ensureTauriRuntime();"), "daily wrapper keeps the native-runtime guard");
assert(
  /parseSecretaryDailyReportEnvelope\(await invoke<unknown>\(\s*"load_secretary_daily_report"\s*\)\)/.test(dailyWrapper[0]),
  "daily wrapper parses only the exact zero-argument load_secretary_daily_report response",
);

assert(
  /(?:^|\n)\s*load_secretary_daily_report\s*,/m.test(registrySource),
  "command_registry.rs registers the exact daily read command",
);
const dailyCommandSignature = /#\[tauri::command\]\s*async fn load_secretary_daily_report\s*\(\s*([\s\S]*?)\s*\)\s*->\s*Result<m4_secretary_read_model::M4SecretaryDailyReportEnvelope,\s*String>/.exec(commandsSource);
assert(dailyCommandSignature, "commands.rs keeps a Tauri daily read command with its frozen result type");
const dailyCommandParameters = dailyCommandSignature[1].trim();
assert(
  /^state:\s*tauri::State<'_,\s*AppState>,?$/.test(dailyCommandParameters),
  "daily command accepts AppState only",
);
for (const rendererField of ["scope", "path", "date", "provider", "model"]) {
  assert(
    !new RegExp(`\\b${rendererField}\\b`, "i").test(dailyCommandParameters),
    `daily command must not accept renderer ${rendererField} input`,
  );
}

const recoveryWrapper = /export async function recoverSecretaryDailyCatchUp\(\s*catchUpTruncationId: string,?\s*\): Promise<M4SecretaryDailyReportEnvelopeDto> \{[\s\S]*?\n\}/.exec(tauriSource);
assert(recoveryWrapper, "tauri.ts exports the opaque-receipt catch-up recovery wrapper");
assert(
  /invoke<unknown>\(\s*"recover_secretary_daily_catch_up",\s*\{ catchUpTruncationId \},\s*\)/.test(recoveryWrapper[0]),
  "catch-up wrapper sends only the exact server-issued receipt reference",
);
assert(
  /(?:^|\n)\s*recover_secretary_daily_catch_up\s*,/m.test(registrySource),
  "command_registry.rs registers the exact catch-up recovery command",
);
const recoveryCommandSignature = /#\[tauri::command\]\s*async fn recover_secretary_daily_catch_up\s*\(\s*([\s\S]*?)\s*\)\s*->\s*Result<m4_secretary_read_model::M4SecretaryDailyReportEnvelope,\s*String>/.exec(commandsSource);
assert(recoveryCommandSignature, "commands.rs keeps a Tauri catch-up recovery command with its frozen result type");
const recoveryParameters = recoveryCommandSignature[1];
assert(/state:\s*tauri::State<'_,\s*AppState>/.test(recoveryParameters), "catch-up recovery keeps AppState");
assert(/catch_up_truncation_id:\s*String/.test(recoveryParameters), "catch-up recovery accepts one opaque receipt");
for (const forbiddenField of ["scope", "timezone", "date", "provider", "model"]) {
  assert(
    !new RegExp(`\\b${forbiddenField}\\b`, "i").test(recoveryParameters),
    `catch-up recovery must not accept renderer ${forbiddenField} input`,
  );
}

console.log("m4c07-secretary-daily-read-model: offline parser and IPC contract assertions passed");
