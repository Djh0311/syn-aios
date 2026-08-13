import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import {
  chmod,
  link,
  lstat,
  mkdir,
  mkdtemp,
  open,
  readFile,
  readdir,
  realpath,
  rename,
  rm,
  stat,
  symlink,
  unlink,
  writeFile,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, isAbsolute, join, relative, resolve, sep } from "node:path";
import { pathToFileURL } from "node:url";
import { runInNewContext } from "node:vm";

const root = process.cwd();
const launcher = readFileSync(
  `${root}/scripts/run-r4-isolated-app-preflight.mjs`,
  "utf8",
);
const offline = readFileSync(
  `${root}/scripts/run-offline-interaction-test.mjs`,
  "utf8",
);
const packageJson = JSON.parse(readFileSync(`${root}/package.json`, "utf8"));

function between(startToken, endToken, label) {
  const start = launcher.indexOf(startToken);
  const end = launcher.indexOf(endToken, start + startToken.length);
  assert.ok(start >= 0 && end > start, `${label} source slice missing`);
  return launcher.slice(start, end);
}

function sourceFunction(name, nextToken) {
  const asyncStart = launcher.indexOf(`async function ${name}(`);
  const normalStart = launcher.indexOf(`function ${name}(`);
  const start = [asyncStart, normalStart]
    .filter((index) => index >= 0)
    .sort((left, right) => left - right)[0];
  const end = start === undefined ? -1 : launcher.indexOf(nextToken, start);
  assert.ok(start !== undefined && end > start, `${name} source slice missing`);
  return launcher.slice(start, end);
}

function sha(number) {
  return number.toString(16).padStart(64, "0");
}

function sourceStringArray(name) {
  const match = launcher.match(new RegExp(`const ${name} = \\[([\\s\\S]*?)\\n\\];`));
  assert.ok(match, `${name} source array missing`);
  return [...match[1].matchAll(/"([^"]+)"/g)].map((entry) => entry[1]);
}

const modeBlock = between(
  "const m4r07OrdinaryProductReacceptanceMode =",
  "const inheritedM2ReferenceSliceMarkers =",
  "R07 CLI mode",
);
assert.ok(
  launcher.includes(
    'const M4R07_ORDINARY_PRODUCT_REACCEPTANCE_MODE_ARG =\n  "--m4r07-isolated-product-reacceptance"',
  )
    && modeBlock.includes("M4R07_ORDINARY_PRODUCT_REACCEPTANCE_MODE_ARG")
    && launcher.includes("inheritedM4R07OrdinaryProductCloseoutMarkers")
    && launcher.includes("M4R07_ORDINARY_PRODUCT_REACCEPTANCE_MODE_CONFLICT"),
  "R07 must register one explicit CLI mode and inherited-marker conflict boundary",
);

const conflictPolicy = between(
  "function resolveLauncherModeConflict(",
  "const initialHome =",
  "mode conflict policy",
);
assert.ok(
  conflictPolicy.includes("m4r07OrdinaryProductReacceptanceMode")
    && conflictPolicy.includes("inheritedM4R07OrdinaryProductCloseoutMarkers.length > 0")
    && conflictPolicy.includes("M4R07_ORDINARY_PRODUCT_REACCEPTANCE_MODE_CONFLICT"),
  "inherited R07 markers must fail before fixture/build/child side effects",
);

const launcherPreflightDirectory = await mkdtemp(join(tmpdir(), "m4r07-esm-preflight-"));
try {
  const launcherTmpDirectory = join(launcherPreflightDirectory, "child-tmp");
  const spawnProbePath = join(launcherPreflightDirectory, "spawn-probe.log");
  const spawnPreloadPath = join(launcherPreflightDirectory, "spawn-preload.cjs");
  await mkdir(launcherTmpDirectory, { recursive: true });
  await writeFile(
    spawnPreloadPath,
    [
      'const fs = require("node:fs");',
      'const childProcess = require("node:child_process");',
      'const { syncBuiltinESMExports } = require("node:module");',
      'const probePath = process.env.M4R07_LAUNCHER_SPAWN_PROBE_PATH;',
      'fs.appendFileSync(probePath, "preloader_loaded\\n", { mode: 0o600 });',
      'childProcess.spawn = function forbiddenPreflightSpawn(...args) {',
      '  fs.appendFileSync(probePath, `spawn:${String(args[0])}\\n`);',
      '  throw new Error("m4r07_preflight_spawn_forbidden");',
      '};',
      'syncBuiltinESMExports();',
      "",
    ].join("\n"),
    { mode: 0o600 },
  );
  const preflight = spawnSync(
    process.execPath,
    [
      `${root}/scripts/run-r4-isolated-app-preflight.mjs`,
      "--m4r07-isolated-product-reacceptance",
      "--m4r06-ordinary-legacy-read",
    ],
    {
      encoding: "utf8",
      env: {
        ...process.env,
        M4R07_LAUNCHER_SPAWN_PROBE_PATH: spawnProbePath,
        NODE_OPTIONS: `--require=${spawnPreloadPath}`,
        TMPDIR: launcherTmpDirectory,
      },
      timeout: 10_000,
    },
  );
  assert.equal(preflight.error, undefined, "launcher preflight process must start");
  assert.equal(preflight.status, 1, "two mode arguments must take the bounded preflight exit");
  assert.equal(preflight.signal, null, "bounded preflight must not be terminated");
  assert.equal(preflight.stdout, "", "bounded preflight must not emit a receipt");
  assert.equal(preflight.stderr, "", "bounded preflight must not emit a launcher exception");
  assert.doesNotMatch(
    `${preflight.stdout}${preflight.stderr}`,
    /(?:ReferenceError|MODE_0600|before initialization)/,
    "full ESM evaluation must not hit a MODE_0600 temporal-dead-zone failure",
  );
  assert.deepEqual(
    await readdir(launcherTmpDirectory),
    [],
    "preflight rejection must create no isolated root or build workspace",
  );
  assert.equal(
    await readFile(spawnProbePath, "utf8"),
    "preloader_loaded\n",
    "preflight rejection must make zero child/build/App spawn calls",
  );
} finally {
  await rm(launcherPreflightDirectory, { recursive: true, force: true });
}

const environmentScrub = between(
  "const normalBuildEnvironment = { ...process.env };",
  "const bundleBuildStartedAtMs =",
  "normal environment scrub",
);
for (const marker of [
  "M4R07_ORDINARY_PRODUCT_CLOSEOUT_ENV",
  "M4R07_RECOVERY_UI_CAPTURE_ENV",
  "M4R02_ORDINARY_COMPOSITION_DRIVER_ENV",
  "M4R03_ORDINARY_CLOCK_DRIVER_ENV",
  "M4R04_ORDINARY_ROUTE_DRIVER_ENV",
  "M4R05_ORDINARY_CONVERSATION_DRIVER_ENV",
  "M4R06_ORDINARY_LEGACY_READ_DRIVER_ENV",
]) {
  assert.ok(
    environmentScrub.includes(`delete normalBuildEnvironment[${marker}]`),
    `normal build environment must scrub ${marker}`,
  );
}

const r07Dispatch = between(
  "} else if (m4r07OrdinaryProductReacceptanceMode) {",
  "} else if (m3c07IsolatedMode) {",
  "R07 bounded dispatcher",
);
const r07DiagnosticDispatch = between(
  "} else if (m4r07PostTickRendererDiagnosticMode) {",
  "} else if (m4r07OrdinaryProductReacceptanceMode) {",
  "R07 post-tick renderer diagnostic dispatcher",
);
const positions = [
  "runM4R05OrdinaryConversationSuite({",
  "runM4R02OrdinaryCompositionSuite({",
  "runM4R06OrdinaryLegacyReadSuite({",
  "runM4R03ServerClockSuite({",
  "runM4R04OrdinaryRouteSuite({",
].map((token) => r07Dispatch.indexOf(token));
assert.ok(
  positions.every((position) => position >= 0)
    && positions.every((position, index) => index === 0 || position > positions[index - 1]),
  "R07 launch families must stay in 2+3+1+3+3 order",
);
assert.equal(
  r07Dispatch.match(/runM4R02OrdinaryCompositionSuite\(\{/g)?.length,
  1,
  "R07 may create exactly one R02 preparation",
);
const r07ConsumerLaunches = r07Dispatch.slice(
  positions[2],
  r07Dispatch.indexOf("const flatLedger =", positions[2]),
);
assert.equal(
  r07ConsumerLaunches.match(/r02Preparation: m4r02OrdinaryCompositionSuite/g)?.length,
  3,
  "R06/R03/R04 must all receive the same already-completed R02 preparation",
);
assert.ok(
  r07Dispatch.includes("r07DirectSpawn: true")
    && r07Dispatch.includes("r07Closeout: true")
    && !r07Dispatch.includes("m4r07PrepareUiCaptureContract(")
    && !r07Dispatch.includes("r07UiCaptureContract")
    && r07Dispatch.includes(
      'm4r07AssertLaunch8UiValidationExcludedArtifactsAbsent(\n              root,\n              "exit",',
    )
    && r07Dispatch.includes("m4r07CreateComposite({"),
  "formal R07 must retain its product launches while excluding launch-8 UI/CU validation",
);
const excludedUiArtifactPaths = sourceFunction(
  "m4r07Launch8UiValidationExcludedArtifactPaths",
  "async function m4r07AssertLaunch8UiValidationExcludedArtifactsAbsent(",
);
const excludedUiArtifactGate = sourceFunction(
  "m4r07AssertLaunch8UiValidationExcludedArtifactsAbsent",
  "async function m4r07PrepareUiCaptureContract(",
);
for (const token of [
  "M4R07_ORDINARY_PRODUCT_REACCEPTANCE_PORTABLE_REPORT_PATH",
  "M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_CAPTURE_PATH",
  "M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_ATTESTATION_PATH",
  "M4R07_ORDINARY_PRODUCT_REACCEPTANCE_EVIDENCE_MANIFEST_PATH",
  "M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_CAPTURE_SIGNAL_PATH",
  "M4R07_RECOVERY_UI_CAPTURE_READY_FILE",
  "M4R07_RECOVERY_UI_CAPTURE_ACK_FILE",
]) {
  assert.ok(
    excludedUiArtifactPaths.includes(token),
    `excluded launch-8 UI artifact gate must cover ${token}`,
  );
}
assert.ok(
  excludedUiArtifactGate.includes("m4r07RequireAbsent(")
    && r07Dispatch.includes('"exit"')
    && !r07Dispatch.includes("M4R07_RECOVERY_UI_CAPTURE_ENV"),
  "formal R07 must use a pure absence gate and never grant its launch-8 child capture authority",
);

const r02Phase = between(
  "async function runM4R02OrdinaryCompositionPhase({",
  "async function runM4R02OrdinaryCompositionSuite({",
  "R02 direct-spawn phase",
);
assert.ok(
  r02Phase.includes("const command = r07DirectSpawn ? debugAppExecutablePath : MACOS_OPEN_PATH")
    && r02Phase.includes('m4r07RecordPhysicalAppSpawn("M4R02", phase, synPid)')
    && r02Phase.includes("expectedProcessIdSha256: r07DirectSpawn ? sha256(String(synPid)) : null")
    && r02Phase.includes('r07DirectSpawn ? { timeoutSignal: "SIGKILL" } : undefined'),
  "only R07's R02 path may use direct executable spawn with a bound App PID",
);
assert.equal(
  r07Dispatch.match(/r07DirectSpawn: true/g)?.length,
  1,
  "formal R07 must create exactly one direct R02 preparation",
);
const r07DiagnosticBody = sourceFunction(
  "runM4R07PostTickRendererDiagnosticBody",
  "async function runM4R07PostTickRendererDiagnostic(",
);
assert.equal(
  r07DiagnosticBody.match(/r07DirectSpawn: true/g)?.length,
  1,
  "the bounded diagnostic must create exactly one direct R02 preparation",
);
assert.equal(
  launcher.match(/r07DirectSpawn: true/g)?.length,
  2,
  "direct R02 spawning must stay confined to formal R07 and its diagnostic mode",
);
for (const token of [
  '["M4R02", "initialize"]',
  '["M4R02", "mutate"]',
  '["M4R02", "readback"]',
  '["M4R03", "arm"]',
  '["M4R03", "recovery_timer"]',
  "expected_app_launches: 5",
  "computer_use_attempts: 0",
  "repeat_launched: false",
  "r04_launched: false",
]) {
  assert.ok(
    r07DiagnosticBody.includes(token),
    `bounded diagnostic contract must retain ${token}`,
  );
}
for (const forbiddenToken of [
  "runM4R03ServerClockSuite({",
  "runM4R04OrdinaryRouteSuite({",
  "m4r07PrepareUiCaptureContract(",
]) {
  assert.ok(
    !r07DiagnosticBody.includes(forbiddenToken),
    `bounded diagnostic must not enter ${forbiddenToken}`,
  );
}
assert.ok(
  r07DiagnosticDispatch.includes("runM4R07PostTickRendererDiagnostic({"),
  "the diagnostic CLI branch must dispatch only through its bounded wrapper",
);

const ledgerSource = sourceFunction(
  "m4r07BuildFlatLedger",
  "function m4r07ObservedAppLaunchCount(",
);
for (const token of [
  "M4R07_ORDINARY_PRODUCT_REACCEPTANCE_EXPECTED_APP_LAUNCHES",
  "[\"M4R05\", \"two_rounds_arm\"]",
  "[\"M4R05\", \"restart_continue_failure\"]",
  "[\"M4R02\", \"initialize\"]",
  "[\"M4R06\", \"read_and_replay\"]",
  "[\"M4R03\", \"arm\"]",
  "[\"M4R04\", \"restart_negative\"]",
  "const interruptedOrdinals = new Set([1, 7])",
  'entry.signal === "SIGKILL"',
  "entry.exit_code === 0 && entry.signal === null && entry.timed_out === false",
]) {
  assert.ok(ledgerSource.includes(token), `flat ledger must freeze ${token}`);
}

const ledgerPrelude = `
const M4R07_ORDINARY_PRODUCT_REACCEPTANCE_EXPECTED_APP_LAUNCHES = 12;
function m4r02IsLowerHexSha256(value) {
  return typeof value === "string" && /^[a-f0-9]{64}$/.test(value);
}
function m4r02FirstInvalidField(checks) {
  return checks.find(([, accepted]) => !accepted)?.[0] ?? null;
}
function m4r02HasExactObjectFields(value, fields) {
  return value !== null
    && typeof value === "object"
    && !Array.isArray(value)
    && Object.keys(value).length === fields.length
    && fields.every((field) => Object.hasOwn(value, field));
}
function m4r05CanonicalJson(value) {
  return JSON.stringify(value);
}
`;
const ledgerHelpers = [
  sourceFunction("m4r07LedgerEntry", "function m4r07PhysicalSpawnAuditProjection("),
  sourceFunction(
    "m4r07PhysicalSpawnAuditProjection",
    "function m4r07PhysicalSpawnAuditFailure(",
  ),
  sourceFunction(
    "m4r07PhysicalSpawnAuditFailure",
    "// A suite object is only assigned after its last phase succeeds",
  ),
  sourceFunction("m4r07R06LedgerEntry", "function m4r07BuildFlatLedger("),
  ledgerSource,
].join("\n");
const m4r07BuildFlatLedger = runInNewContext(
  `${ledgerPrelude}\n${ledgerHelpers}\nm4r07BuildFlatLedger;`,
);

const phases = [
  ["M4R05", "two_rounds_arm"],
  ["M4R05", "restart_continue_failure"],
  ["M4R02", "initialize"],
  ["M4R02", "mutate"],
  ["M4R02", "readback"],
  ["M4R06", "read_and_replay"],
  ["M4R03", "arm"],
  ["M4R03", "recovery_timer"],
  ["M4R03", "repeat"],
  ["M4R04", "work_item"],
  ["M4R04", "proposal"],
  ["M4R04", "restart_negative"],
];
const profileFingerprint = sha(900);
const buildIdentity = sha(901);
function launchEntry(index) {
  const [, phase] = phases[index];
  const interrupted = index === 0 || index === 6;
  return {
    phase,
    receipt_sha256: sha(index + 1),
    receipt: {
      outcome: "PASS",
      profile_fingerprint: profileFingerprint,
      nonce_sha256: sha(index + 101),
      process_id_sha256: sha(index + 201),
    },
    launch: {
      launched: true,
      exit_code: interrupted ? null : 0,
      signal: interrupted ? "SIGKILL" : null,
      timed_out: false,
    },
  };
}
function validR07Suites() {
  const entries = phases.map((_, index) => launchEntry(index));
  const r06 = entries[5];
  return {
    r05Suite: { launches: entries.slice(0, 2) },
    r02Preparation: { launches: entries.slice(2, 5) },
    r06Suite: {
      r07_phase_ledger_entry: {
        task_package: "M4R06",
        phase: r06.phase,
        outcome: r06.receipt.outcome,
        profile_fingerprint: r06.receipt.profile_fingerprint,
        receipt_sha256: r06.receipt_sha256,
        nonce_sha256: r06.receipt.nonce_sha256,
        process_id_sha256: r06.receipt.process_id_sha256,
        launched: r06.launch.launched,
        exit_code: r06.launch.exit_code,
        signal: r06.launch.signal,
        timed_out: r06.launch.timed_out,
      },
    },
    r03Suite: { launches: entries.slice(6, 9) },
    r04Suite: { launches: entries.slice(9, 12) },
  };
}
function validPhysicalSpawnAudit() {
  return {
    spawns: phases.map(([taskPackage, phase], index) => ({
      task_package: taskPackage,
      phase,
      app_process_id_sha256: sha(index + 201),
    })),
  };
}
function buildLedger(suites, launchAudit = validPhysicalSpawnAudit()) {
  return m4r07BuildFlatLedger({
    ...suites,
    expectedProfileFingerprint: profileFingerprint,
    buildIdentitySha256: buildIdentity,
    launchAudit,
  });
}

const validLedger = buildLedger(validR07Suites());
assert.deepEqual(
  JSON.parse(JSON.stringify(validLedger.map(({ task_package, phase, launch_ordinal }) => [
    launch_ordinal,
    task_package,
    phase,
  ]))),
  phases.map(([taskPackage, phase], index) => [index + 1, taskPackage, phase]),
  "dynamically extracted ledger builder must preserve the exact twelve App launches",
);
assert.equal(validLedger[0].signal, "SIGKILL");
assert.equal(validLedger[6].signal, "SIGKILL");
for (const index of [1, 2, 3, 4, 5, 7, 8, 9, 10, 11]) {
  assert.equal(validLedger[index].exit_code, 0);
  assert.equal(validLedger[index].signal, null);
}

const r03ArmPhaseSource = sourceFunction(
  "runM4R03ArmPhase",
  "async function runM4R03NormalPhase(",
);
assert.ok(
  r03ArmPhaseSource.includes("...(await m4r03AwaitCloseGrace(process))")
    && r03ArmPhaseSource.includes("timed_out: false"),
  "the successful R03 arm close result must normalize timed_out=false before entering the flat ledger",
);
const rawR03ArmCloseSuites = validR07Suites();
rawR03ArmCloseSuites.r03Suite.launches[0].launch = {
  launched: true,
  exit_code: null,
  signal: "SIGKILL",
};
assert.throws(
  () => buildLedger(rawR03ArmCloseSuites),
  /m4r07_flat_ledger_invalid:terminal_contract/,
  "the raw R03 close-event shape without timed_out must be rejected",
);
rawR03ArmCloseSuites.r03Suite.launches[0].launch = {
  ...rawR03ArmCloseSuites.r03Suite.launches[0].launch,
  timed_out: false,
};
assert.doesNotThrow(
  () => buildLedger(rawR03ArmCloseSuites),
  "the normalized R03 arm close shape must satisfy the exact terminal contract",
);

const duplicateNonceSuites = validR07Suites();
duplicateNonceSuites.r03Suite.launches[0].receipt.nonce_sha256 =
  duplicateNonceSuites.r02Preparation.launches[0].receipt.nonce_sha256;
assert.throws(
  () => buildLedger(duplicateNonceSuites),
  /m4r07_flat_ledger_invalid:nonce_hashes/,
  "a duplicate nonce must reject the complete ledger",
);
const brokenTerminalSuites = validR07Suites();
brokenTerminalSuites.r05Suite.launches[0].launch = {
  launched: true,
  exit_code: 0,
  signal: null,
  timed_out: false,
};
assert.throws(
  () => buildLedger(brokenTerminalSuites),
  /m4r07_flat_ledger_invalid:terminal_contract/,
  "launch #1 cannot silently become a normal exit",
);
const tamperedPhysicalAudit = validPhysicalSpawnAudit();
tamperedPhysicalAudit.spawns[7].app_process_id_sha256 = sha(997);
assert.throws(
  () => buildLedger(validR07Suites(), tamperedPhysicalAudit),
  /m4r07_flat_ledger_invalid:physical_spawn_audit/,
  "the external physical-spawn audit must bind each receipt PID in the same order",
);

const countSource = sourceFunction(
  "m4r07ObservedAppLaunchCount",
  "const M4R07_FROZEN_SENTINEL_CHECK_STAGES =",
);
const m4r07ObservedAppLaunchCount = runInNewContext(
  `${countSource}\nm4r07ObservedAppLaunchCount;`,
);
assert.equal(
  m4r07ObservedAppLaunchCount({
    launchAudit: { spawns: Array.from({ length: 5 }, () => ({ task_package: "M4R02" })) },
  }),
  5,
  "partial R07 failure must report its actual direct-spawn count rather than a completed-suite count",
);
assert.equal(
  m4r07ObservedAppLaunchCount({
    ...validR07Suites(),
  }),
  12,
  "completed R07 suite count must remain exactly twelve",
);

const r07Rejected = between(
  ": m4r07OrdinaryProductReacceptanceMode\n        ? m4r07StdoutReceiptEnvelope(",
  ": m3c07IsolatedMode",
  "R07 bounded rejected receipt",
);
assert.ok(
  r07Rejected.includes('outcome: "REJECTED"')
    && r07Rejected.includes("portable: false")
    && r07Rejected.includes("observed_app_launches: m4r07ObservedAppLaunchCount({")
    && r07Rejected.includes("launchAudit: m4r07ActiveLaunchAudit")
    && !r07Rejected.includes("flat_launch_ledger:"),
  "partial failure must emit only a bounded REJECTED summary, never a fabricated PASS ledger",
);
assert.ok(
  r07Dispatch.includes("typeof error?.failureFamily === \"string\"")
    && r07Dispatch.includes("/^[a-z0-9_:-]{1,160}$/"),
  "R07 failure stage must be a bounded family rather than raw child output",
);

const sqliteRows = sourceFunction(
  "m4r07ReadOnlySqliteRows",
  "async function m4r07ReadOnlySqliteLogicalSha3(",
);
const sqliteSha3 = sourceFunction(
  "m4r07ReadOnlySqliteLogicalSha3",
  "async function m4r07ReadOnlySqliteTableSha3Manifest(",
);
const sqliteTableManifest = sourceFunction(
  "m4r07ReadOnlySqliteTableSha3Manifest",
  "function m4r07ReadOnlyJsonArray(",
);
for (const source of [sqliteRows, sqliteSha3]) {
  assert.ok(
    source.includes('"/usr/bin/sqlite3"')
      && source.includes('"-readonly"')
      && source.includes('".timeout 5000"')
      && source.includes('"PRAGMA query_only=ON; PRAGMA foreign_keys=ON;"')
      && !source.includes("PRAGMA busy_timeout"),
    "R07 read-only SQLite probes must avoid PRAGMA busy_timeout stdout pollution",
  );
}
for (const source of [sqliteRows, sqliteSha3, sqliteTableManifest]) {
  assert.ok(
    source.includes("let stdoutOverflow = false")
      && source.includes("let stderrOverflow = false")
      && source.includes("stdoutOverflow ||")
      && source.includes("stderrOverflow"),
    "every bounded SQLite helper must remember overflow before tail-slicing its output",
  );
}
assert.ok(
  sqliteRows.includes("|| stderr.length > 0"),
  "the JSON row helper must reject any non-empty SQLite stderr even after a zero exit",
);
assert.ok(
  sqliteSha3.includes('".sha3sum --schema --sha3-256"')
    && sqliteSha3.includes("m4r02IsLowerHexSha256(digest)"),
  "the dedicated provider sentinel must use a full logical SHA3-256, not volatile SQLite bytes",
);
assert.ok(
  sqliteTableManifest.includes("tableNames.flatMap((tableName) => [")
    && sqliteTableManifest.includes("`.sha3sum --schema --sha3-256 ${tableName}`")
    && sqliteTableManifest.includes('"SELECT 1 WHERE 0;"')
    && sqliteTableManifest.includes("entries.length !== expectedNames.length")
    && !sqliteTableManifest.includes("tableNames.join"),
  "the M3 manifest must execute one exact SHA3 command per allowlisted table and reject a partial manifest",
);
const sqliteReadOnlyProbes = runInNewContext(
  `
const M4R07_ORDINARY_PRODUCT_REACCEPTANCE_SQLITE_READ_TIMEOUT_MS = 15 * 1000;
const M4R07_ORDINARY_PRODUCT_REACCEPTANCE_SQLITE_READ_MAX_BYTES = 512 * 1024;
function m4r02IsLowerHexSha256(value) {
  return typeof value === "string" && /^[a-f0-9]{64}$/.test(value);
}
${sqliteRows}
${sqliteSha3}
({ m4r07ReadOnlySqliteRows, m4r07ReadOnlySqliteLogicalSha3 });
`,
  {
    clearTimeout,
    setTimeout,
    signalProcess: (pid, signal) => process.kill(pid, signal),
    spawn,
  },
);
const sqliteProbeDirectory = await mkdtemp(join(tmpdir(), "m4r07-read-only-probe-"));
try {
  const sqliteProbePath = join(sqliteProbeDirectory, "fixture.sqlite3");
  const create = spawnSync(
    "/usr/bin/sqlite3",
    [
      sqliteProbePath,
      "CREATE TABLE probe (id INTEGER PRIMARY KEY, answer INTEGER NOT NULL); INSERT INTO probe(answer) VALUES (42);",
    ],
    { encoding: "utf8" },
  );
  assert.equal(create.status, 0, create.stderr || "temporary SQLite fixture setup failed");
  const rows = await sqliteReadOnlyProbes.m4r07ReadOnlySqliteRows({
    databaseTarget: sqliteProbePath,
    label: "temporary_rowset",
    query: "SELECT answer FROM probe ORDER BY id;",
  });
  assert.deepEqual(
    JSON.parse(JSON.stringify(rows)),
    [{ answer: 42 }],
    "the read-only row probe must parse exactly one JSON row without timeout-pragmas on stdout",
  );
  const digest = await sqliteReadOnlyProbes.m4r07ReadOnlySqliteLogicalSha3({
    databaseTarget: sqliteProbePath,
    label: "temporary_sha3",
  });
  assert.match(digest, /^[a-f0-9]{64}$/, "temporary logical SHA3 must be SHA3-256");
} finally {
  await rm(sqliteProbeDirectory, { recursive: true, force: true });
}
const projectionReader = sourceFunction(
  "m4r07ReadM3ProviderBusinessProjection",
  "async function m4r07CreateM3ProviderFrozenSentinel(",
);
const boundedProjectionReaderSource = sourceFunction(
  "m4r07ReadM3ProviderBusinessProjectionBounded",
  "async function m4r07CreateM3ProviderFrozenSentinel(",
);
assert.ok(
  boundedProjectionReaderSource.includes('error.failureFamily = "m3_provider_snapshot_projection_invalid"')
    && launcher.match(/await m4r07ReadM3ProviderBusinessProjectionBounded\(root\)/g)?.length === 2,
  "both baseline and repeated sentinels must route uncategorized projection errors through one bounded family",
);
const unclassifiedProjectionFailure = new Error("raw fixture path and SQL detail");
const boundedProjectionReader = runInNewContext(
  `${boundedProjectionReaderSource}\nm4r07ReadM3ProviderBusinessProjectionBounded;`,
  {
    m4r07ReadM3ProviderBusinessProjection: async () => {
      throw unclassifiedProjectionFailure;
    },
  },
);
await assert.rejects(
  () => boundedProjectionReader("/private/fixture/root"),
  (error) => error?.message === "m4r07_m3_provider_snapshot_projection_invalid"
    && error?.failureFamily === "m3_provider_snapshot_projection_invalid"
    && !error.message.includes("fixture"),
  "an unclassified projection failure must not escape as raw SQL/path detail",
);
assert.ok(
  projectionReader.match(/m4r07ReadOnlySqliteRows\(\{/g)?.length === 2
    && projectionReader.match(/m4r07ReadOnlySqliteLogicalSha3\(\{/g)?.length === 1
    && projectionReader.match(/m4r07ReadOnlySqliteTableSha3Manifest\(\{/g)?.length === 1
    && projectionReader.includes("owned_table_sha3_manifest_sha256")
    && projectionReader.includes("owned_schema_sha256")
    && projectionReader.includes("owned_sequence_sha256")
    && projectionReader.includes("owned_catalog_json")
    && projectionReader.includes("m4r07RequireExactM3OwnedCatalog(")
    && projectionReader.includes("unexpected_trigger_view_rows")
    && projectionReader.includes("owned_sequence_json")
    && /SELECT name, seq FROM sqlite_sequence\s+WHERE lower\(name\) GLOB 'm3\?\*'/.test(
      projectionReader,
    )
    && projectionReader.includes("type IN ('table','index') AND lower(name) GLOB 'm3?*'")
    && projectionReader.includes("name GLOB 'm3_*'")
    && projectionReader.includes("tbl_name GLOB 'm3_*'")
    && projectionReader.includes("lower(COALESCE(sql, '')) GLOB '*m3_*'")
    && projectionReader.includes("type IN ('trigger','view')")
    && projectionReader.includes("type IN ('table','index')")
    && projectionReader.includes("read_only_query_only_connection_count: 4"),
  "the frozen sentinel must reverse-enumerate the exact M3 table set and hash all M3 table/view/index/trigger schema",
);
assert.match(
  projectionReader,
  /m4r07RequireRegularPrivateFile\(\s*m3Path,\s*"m3_read_only_source",\s*0o644,?\s*\)/,
  "R07 M3 source must require the isolated run's observed 0644 mode",
);
assert.match(
  projectionReader,
  /m4r07RequireRegularPrivateFile\(\s*providerPath,\s*"provider_read_only_source",\s*MODE_0600,?\s*\)/,
  "R07 provider source must require its exact private 0600 mode",
);
assert.ok(
  projectionReader.includes('error.failureFamily = "m3_provider_read_only_source_invalid"')
    && projectionReader.includes('"m3_provider_snapshot_read"')
    && projectionReader.includes('"m3_owned_domain_digest"')
    && projectionReader.includes('"provider_domain_digest"')
    && projectionReader.includes('"m3_provider_snapshot_changed"')
    && !projectionReader.includes('error.failureFamily = "m4r07_ordinary_product_reacceptance"'),
  "source, rowset, scoped-M3 digest, provider digest, and drift failures must retain bounded diagnostic families",
);

const privateFileModeGateSource = sourceFunction(
  "m4r07RequireRegularPrivateFile",
  "async function m4r07RequireCanonicalPrivateDirectory(",
);
assert.ok(
  privateFileModeGateSource.includes("(metadata.mode & 0o777) !== expectedMode"),
  "the reusable R07 source-file gate must reject every non-exact mode",
);
const m4r07RequireRegularPrivateFile = runInNewContext(
  `
const MODE_0600 = 0o600;
${privateFileModeGateSource}
m4r07RequireRegularPrivateFile;
`,
  { lstat },
);
const m4r07ReadM3ProviderBusinessProjection = runInNewContext(
  `
const MODE_0600 = 0o600;
${privateFileModeGateSource}
${projectionReader}
m4r07ReadM3ProviderBusinessProjection;
`,
  { join, lstat, Promise },
);
const m3ProviderModeDirectory = await mkdtemp(join(tmpdir(), "m4r07-m3-provider-mode-"));
try {
  const productRoot = join(
    m3ProviderModeDirectory,
    "app-data",
    "local.codex.governance.workbench",
  );
  const m3SourcePath = join(productRoot, "conversation", "m3-role-session-v1.sqlite3");
  const providerSourcePath = join(
    productRoot,
    "m4-secretary",
    "provider-transcript-v1.sqlite3",
  );
  await Promise.all([
    mkdir(join(productRoot, "conversation"), { recursive: true }),
    mkdir(join(productRoot, "m4-secretary"), { recursive: true }),
  ]);
  await Promise.all([
    writeFile(m3SourcePath, "m3", { mode: 0o600 }),
    writeFile(providerSourcePath, "provider", { mode: 0o600 }),
  ]);
  await Promise.all([
    chmod(m3SourcePath, 0o644),
    chmod(providerSourcePath, 0o600),
  ]);
  const [m3Metadata, providerMetadata] = await Promise.all([
    m4r07RequireRegularPrivateFile(m3SourcePath, "m3_read_only_source", 0o644),
    m4r07RequireRegularPrivateFile(providerSourcePath, "provider_read_only_source", 0o600),
  ]);
  assert.equal(m3Metadata.mode & 0o777, 0o644);
  assert.equal(providerMetadata.mode & 0o777, 0o600);

  for (const wrongMode of [0o600, 0o640]) {
    await chmod(m3SourcePath, wrongMode);
    await assert.rejects(
      () => m4r07RequireRegularPrivateFile(
        m3SourcePath,
        "m3_read_only_source",
        0o644,
      ),
      /m4r07_prelaunch_m3_read_only_source_file_invalid/,
      `M3 source mode ${wrongMode.toString(8)} must fail closed`,
    );
  }
  await chmod(m3SourcePath, 0o644);
  for (const wrongMode of [0o644, 0o640]) {
    await chmod(providerSourcePath, wrongMode);
    await assert.rejects(
      () => m4r07RequireRegularPrivateFile(
        providerSourcePath,
        "provider_read_only_source",
        0o600,
      ),
      /m4r07_prelaunch_provider_read_only_source_file_invalid/,
      `provider source mode ${wrongMode.toString(8)} must fail closed`,
    );
  }
  await chmod(m3SourcePath, 0o600);
  await chmod(providerSourcePath, 0o600);
  await assert.rejects(
    () => m4r07ReadM3ProviderBusinessProjection(m3ProviderModeDirectory),
    (error) => error?.message === "m4r07_m3_provider_read_only_source_invalid"
      && error?.failureFamily === "m3_provider_read_only_source_invalid",
    "wrong M3 mode must fail through the real projection's bounded source contract",
  );
  await chmod(m3SourcePath, 0o644);
  await chmod(providerSourcePath, 0o644);
  await assert.rejects(
    () => m4r07ReadM3ProviderBusinessProjection(m3ProviderModeDirectory),
    (error) => error?.message === "m4r07_m3_provider_read_only_source_invalid"
      && error?.failureFamily === "m3_provider_read_only_source_invalid",
    "wrong provider mode must fail through the real projection's bounded source contract",
  );
} finally {
  await rm(m3ProviderModeDirectory, { recursive: true, force: true });
}

const m3OwnedTableRegistry = between(
  "const M4R07_M3_OWNED_TABLE_NAMES = [",
  "const M4R07_M3_OWNED_INDEX_NAMES = [",
  "M3 owned-table allowlist",
);
const m3OwnedIndexRegistry = between(
  "const M4R07_M3_OWNED_INDEX_NAMES = [",
  "const M4R07_ORDINARY_PRODUCT_REACCEPTANCE_MODE_CONFLICT =",
  "M3 owned-index allowlist",
);
const m3OwnedTableNames = [
  ...m3OwnedTableRegistry.matchAll(/^  "([a-z0-9_]+)",$/gm),
].map((match) => match[1]);
assert.deepEqual(
  m3OwnedTableNames,
  [
    "m3_audit_records",
    "m3_command_receipts",
    "m3_conversation_contexts",
    "m3_events",
    "m3_handoff_audit_records",
    "m3_handoff_command_receipts",
    "m3_handoff_events",
    "m3_handoff_permission_descriptors",
    "m3_handoff_receipts",
    "m3_handoff_source_applications",
    "m3_handoff_source_command_fences",
    "m3_handoff_source_validation_proofs",
    "m3_handoff_validation_witnesses",
    "m3_handoffs",
    "m3_provider_effect_attempts",
    "m3_provider_handles",
    "m3_role_sessions",
    "m3_role_turns",
    "m3_schema_markers",
    "m3_session_bindings",
    "m3_shadow_imports",
  ],
  "the scoped M3 digest must freeze the exact sorted set of 21 product-owned tables",
);
const m3OwnedIndexNames = [
  ...m3OwnedIndexRegistry.matchAll(/^  "([a-z0-9_]+)",$/gm),
].map((match) => match[1]);
assert.deepEqual(
  m3OwnedIndexNames,
  [
    "m3_idx_role_session_join",
    "m3_idx_role_sessions_owner",
    "m3_idx_turns_session_state",
    "m3_idx_provider_handle_live_natural",
    "m3_idx_provider_handles_session",
    "m3_idx_session_bindings_handle",
    "m3_idx_session_bindings_current",
    "m3_idx_contexts_session",
    "m3_idx_receipts_idempotency",
    "m3_idx_receipts_aggregate",
    "m3_idx_effects_dispatch",
    "m3_idx_effects_turn",
    "m3_idx_effects_one_unsettled_stop_per_turn",
    "m3_idx_events_aggregate",
    "m3_idx_audits_target",
    "m3_idx_shadow_source_ref",
    "m3_idx_handoff_validation_witness_binding",
    "m3_idx_handoffs_source_status",
    "m3_idx_handoffs_recipient_status",
    "m3_idx_handoff_command_idempotency",
    "m3_idx_handoff_receipts_revision",
    "m3_idx_handoff_source_validation_binding",
    "m3_idx_handoff_events_aggregate",
    "m3_idx_handoff_audits_target",
    "m3_idx_handoff_source_command_fences_handoff",
    "m3_idx_handoff_source_application_applied",
    "m3_idx_handoff_source_applications_result",
  ],
  "the reverse catalog gate must freeze the exact 27 M3-owned index names",
);

const sqliteMetadataProjectionSource = sourceFunction(
  "m4r07SqliteMetadataProjection",
  "async function m4r07OptionalSqliteSidecarMetadata(",
);
assert.ok(
  sqliteMetadataProjectionSource.includes("m4r07SqliteStableFileProjection")
    && sqliteMetadataProjectionSource.includes("sha256(bytes)"),
  "read cuts must fingerprint the complete main and WAL bytes, not metadata alone",
);
const sqliteOptionalSidecarSource = sourceFunction(
  "m4r07OptionalSqliteSidecarMetadata",
  "async function m4r07PrepareSqliteReadCut(",
);
const sqlitePrepareReadCutSource = sourceFunction(
  "m4r07PrepareSqliteReadCut",
  "async function m4r07AssertSqliteReadCutStable(",
);
const sqliteAssertReadCutStableSource = sourceFunction(
  "m4r07AssertSqliteReadCutStable",
  "async function m4r07WithFailureFamily(",
);
const sqliteFailureFamilySource = sourceFunction(
  "m4r07WithFailureFamily",
  "async function m4r07ReadOnlySqliteRows(",
);
const sqliteReadCutHelpers = runInNewContext(
  `
${privateFileModeGateSource}
${sqliteMetadataProjectionSource}
${sqliteOptionalSidecarSource}
${sqlitePrepareReadCutSource}
${sqliteAssertReadCutStableSource}
${sqliteFailureFamilySource}
({
  m4r07PrepareSqliteReadCut,
  m4r07AssertSqliteReadCutStable,
  m4r07WithFailureFamily,
});
`,
  {
    lstat,
    pathToFileURL,
    Promise,
    readFile,
    sha256: (value) => createHash("sha256").update(value).digest("hex"),
  },
);
const sqliteJsonArraySource = sourceFunction(
  "m4r07ReadOnlyJsonArray",
  "function m4r07ReadOnlyCount(",
);
const sqliteJsonShaSource = sourceFunction(
  "m4r07ReadOnlyJsonSha256",
  "function m4r07RequireExactM3OwnedCatalog(",
);
const exactM3OwnedCatalogSource = sourceFunction(
  "m4r07RequireExactM3OwnedCatalog",
  "async function m4r07ReadM3ProviderBusinessProjection(",
);
const sqliteReadOnlyCountSource = sourceFunction(
  "m4r07ReadOnlyCount",
  "function m4r07ReadOnlyJsonSha256(",
);
const m3OwnedDomainGateSource = between(
  "  const ownedCatalog = m4r07RequireExactM3OwnedCatalog(\n    m3.owned_catalog_json,\n  );",
  "  const orderedTurnRefs =",
  "M3 owned-domain runtime gate",
);
const m3OwnedDomainHelpers = runInNewContext(
  `
const M4R07_M3_OWNED_TABLE_NAMES = ${JSON.stringify(m3OwnedTableNames)};
const M4R07_M3_OWNED_INDEX_NAMES = ${JSON.stringify(m3OwnedIndexNames)};
${sqliteJsonArraySource}
${sqliteReadOnlyCountSource}
${sqliteJsonShaSource}
${exactM3OwnedCatalogSource}
function m4r07ValidateM3OwnedDomain(m3) {
${m3OwnedDomainGateSource}
  return { ownedCatalog, forbiddenTriggerViewCount, ownedSchema, ownedSequence };
}
({
  m4r07ReadOnlyJsonArray,
  m4r07ReadOnlyJsonSha256,
  m4r07RequireExactM3OwnedCatalog,
  m4r07ValidateM3OwnedDomain,
});
`,
  {
    sha256: (value) => createHash("sha256").update(value).digest("hex"),
  },
);
const sqliteManifestSpawnArguments = [];
const m4r07ReadOnlySqliteTableSha3Manifest = runInNewContext(
  `
const M4R07_ORDINARY_PRODUCT_REACCEPTANCE_SQLITE_READ_TIMEOUT_MS = 15 * 1000;
const M4R07_ORDINARY_PRODUCT_REACCEPTANCE_SQLITE_READ_MAX_BYTES = 512 * 1024;
${sqliteTableManifest}
m4r07ReadOnlySqliteTableSha3Manifest;
`,
  {
    clearTimeout,
    setTimeout,
    sha256: (value) => createHash("sha256").update(value).digest("hex"),
    signalProcess: (pid, signal) => process.kill(pid, signal),
    spawn: (command, arguments_, options) => {
      sqliteManifestSpawnArguments.push([...arguments_]);
      return spawn(command, arguments_, options);
    },
  },
);

const m3WalDigestDirectory = await mkdtemp(join(tmpdir(), "m4r07-m3-wal-digest-"));
try {
  const databasePath = join(m3WalDigestDirectory, "m3-role-session-v1.sqlite3");
  const walPath = `${databasePath}-wal`;
  const shmPath = `${databasePath}-shm`;
  const fixtureSql = [
    "PRAGMA journal_mode=WAL;",
    ...m3OwnedTableNames.flatMap((tableName) => [
      `CREATE TABLE "${tableName}" (id INTEGER PRIMARY KEY, payload TEXT NOT NULL${
        tableName === "m3_events" ? " UNIQUE" : ""
      });`,
      `INSERT INTO "${tableName}" (id, payload) VALUES (1, 'baseline:${tableName}');`,
    ]),
    ...m3OwnedIndexNames.map((indexName) => (
      `CREATE INDEX "${indexName}" ON "m3_events" (payload);`
    )),
    'CREATE TABLE "unrelated_fixture" (id INTEGER PRIMARY KEY AUTOINCREMENT, payload TEXT NOT NULL);',
    "INSERT INTO unrelated_fixture (id, payload) VALUES (1, 'noise');",
    "PRAGMA wal_checkpoint(TRUNCATE);",
  ].join("\n");
  const create = spawnSync("/usr/bin/sqlite3", [databasePath, fixtureSql], {
    encoding: "utf8",
  });
  assert.equal(create.status, 0, create.stderr || "M3 WAL fixture setup failed");
  await chmod(databasePath, 0o644);
  await Promise.all([
    rm(walPath, { force: true }),
    rm(shmPath, { force: true }),
  ]);
  const databaseHeader = await readFile(databasePath);
  assert.equal(databaseHeader[18], 2, "M3 fixture must retain WAL write-version in its header");
  assert.equal(databaseHeader[19], 2, "M3 fixture must retain WAL read-version in its header");
  assert.equal(existsSync(walPath), false, "the checkpointed WAL-header fixture starts without -wal");
  assert.equal(existsSync(shmPath), false, "the checkpointed WAL-header fixture starts without -shm");

  const absentSidecarCut = await sqliteReadCutHelpers.m4r07PrepareSqliteReadCut({
    databasePath,
    label: "m3_absent_sidecars",
    expectedMode: 0o644,
  });
  assert.equal(absentSidecarCut.metadata.wal, null);
  assert.equal(absentSidecarCut.metadata.shm, null);
  assert.equal(
    absentSidecarCut.target,
    `${pathToFileURL(databasePath).href}?mode=ro&immutable=1`,
    "a checkpointed WAL-header database with absent sidecars must use the immutable read target",
  );
  const absentSidecarRows = await sqliteReadOnlyProbes.m4r07ReadOnlySqliteRows({
    databaseTarget: absentSidecarCut.target,
    label: "m3_absent_sidecars",
    query: "SELECT payload FROM m3_events WHERE id=1;",
  });
  assert.deepEqual(
    JSON.parse(JSON.stringify(absentSidecarRows)),
    [{ payload: "baseline:m3_events" }],
    "the real read-only helper must read a WAL-header database with both sidecars absent",
  );
  await sqliteReadCutHelpers.m4r07AssertSqliteReadCutStable(absentSidecarCut);

  await writeFile(walPath, Buffer.alloc(0), { mode: 0o644 });
  await chmod(walPath, 0o644);
  const zeroWalCut = await sqliteReadCutHelpers.m4r07PrepareSqliteReadCut({
    databasePath,
    label: "m3_zero_wal",
    expectedMode: 0o644,
  });
  assert.equal(zeroWalCut.metadata.wal.bytes, 0);
  assert.equal(zeroWalCut.metadata.shm, null);
  assert.equal(
    zeroWalCut.target,
    `${pathToFileURL(databasePath).href}?mode=ro&immutable=1`,
    "a zero-byte WAL must remain a valid immutable read cut without a shm file",
  );
  const zeroWalRows = await sqliteReadOnlyProbes.m4r07ReadOnlySqliteRows({
    databaseTarget: zeroWalCut.target,
    label: "m3_zero_wal",
    query: "SELECT payload FROM m3_role_turns WHERE id=1;",
  });
  assert.deepEqual(
    JSON.parse(JSON.stringify(zeroWalRows)),
    [{ payload: "baseline:m3_role_turns" }],
    "the real read-only helper must accept a zero-byte WAL without shm",
  );
  await sqliteReadCutHelpers.m4r07AssertSqliteReadCutStable(zeroWalCut);

  await writeFile(walPath, Buffer.alloc(32, 0x5a), { mode: 0o644 });
  await chmod(walPath, 0o644);
  assert.equal(existsSync(shmPath), false);
  await assert.rejects(
    () => sqliteReadCutHelpers.m4r07PrepareSqliteReadCut({
      databasePath,
      label: "m3",
      expectedMode: 0o644,
    }),
    (error) => error?.message === "m4r07_m3_nonempty_wal_without_shm"
      && error?.failureFamily === "m3_provider_snapshot_open",
    "a non-empty WAL without shm must fail closed through the bounded snapshot-open family",
  );
  await rm(walPath, { force: true });

  async function m3OwnedDigest() {
    const cut = await sqliteReadCutHelpers.m4r07PrepareSqliteReadCut({
      databasePath,
      label: "m3_owned_fixture",
      expectedMode: 0o644,
    });
    const digest = await m4r07ReadOnlySqliteTableSha3Manifest({
      databaseTarget: cut.target,
      label: "m3_owned_fixture",
      tableNames: m3OwnedTableNames,
    });
    await sqliteReadCutHelpers.m4r07AssertSqliteReadCutStable(cut);
    return digest;
  }
  function mutateFixture(sql, label) {
    const mutation = spawnSync(
      "/usr/bin/sqlite3",
      [databasePath, `PRAGMA journal_mode=WAL; ${sql} PRAGMA wal_checkpoint(TRUNCATE);`],
      { encoding: "utf8" },
    );
    assert.equal(mutation.status, 0, mutation.stderr || `${label} fixture mutation failed`);
  }
  async function readM3OwnedDomainRegistry() {
    const cut = await sqliteReadCutHelpers.m4r07PrepareSqliteReadCut({
      databasePath,
      label: "m3_owned_registry_fixture",
      expectedMode: 0o644,
    });
    const m3OwnedTableSql = m3OwnedTableNames
      .map((tableName) => `'${tableName}'`)
      .join(",");
    const rows = await sqliteReadOnlyProbes.m4r07ReadOnlySqliteRows({
      databaseTarget: cut.target,
      label: "m3_owned_registry_fixture",
      query: `
        SELECT
          (SELECT COALESCE(json_group_array(json_object(
             'type', type,
             'name', name
           )), '[]') FROM (
             SELECT type, name FROM sqlite_schema
             WHERE type IN ('table','index') AND lower(name) GLOB 'm3?*'
             ORDER BY name
           )) AS owned_catalog_json,
          (SELECT COUNT(*) FROM sqlite_schema
             WHERE type IN ('trigger','view')
               AND (
                 name GLOB 'm3_*'
                 OR tbl_name GLOB 'm3_*'
                 OR lower(COALESCE(sql, '')) GLOB '*m3_*'
               )) AS unexpected_trigger_view_rows,
          (SELECT COALESCE(json_group_array(json_object(
             'type', type,
             'name', name,
             'table_name', tbl_name,
             'sql', sql
           )), '[]') FROM (
             SELECT type, name, tbl_name, sql FROM sqlite_schema
             WHERE type IN ('table','index')
               AND (
                 lower(name) GLOB 'm3?*'
                 OR (type='index' AND tbl_name IN (${m3OwnedTableSql}))
               )
             ORDER BY type, name
           )) AS owned_schema_json,
          (SELECT COALESCE(json_group_array(json_object(
             'name', name,
             'sequence', seq
           )), '[]') FROM (
             SELECT name, seq FROM sqlite_sequence
             WHERE lower(name) GLOB 'm3?*'
             ORDER BY name
           )) AS owned_sequence_json;
      `,
    });
    await sqliteReadCutHelpers.m4r07AssertSqliteReadCutStable(cut);
    assert.equal(rows.length, 1, "M3 domain registry query must return exactly one row");
    return rows[0];
  }

  let ownedDigest = await m3OwnedDigest();
  assert.match(ownedDigest, /^[a-f0-9]{64}$/);
  const firstManifestArguments = sqliteManifestSpawnArguments.at(-1);
  const sha3Commands = firstManifestArguments
    .map((argument, index) => [argument, firstManifestArguments[index + 1]])
    .filter(([argument, next]) => argument === "-cmd" && next?.startsWith(".sha3sum "))
    .map(([, command]) => command);
  assert.deepEqual(
    sha3Commands,
    m3OwnedTableNames.map((tableName) => `.sha3sum --schema --sha3-256 ${tableName}`),
    "the executable manifest probe must issue exactly 21 separate table SHA3 commands",
  );

  mutateFixture(
    "UPDATE unrelated_fixture SET payload='unrelated-row-change' WHERE id=1;",
    "unrelated row",
  );
  assert.equal(
    await m3OwnedDigest(),
    ownedDigest,
    "an unrelated-table row mutation must not perturb the scoped M3 digest",
  );
  mutateFixture(
    "ALTER TABLE unrelated_fixture ADD COLUMN unrelated_schema_probe TEXT;",
    "unrelated schema",
  );
  assert.equal(
    await m3OwnedDigest(),
    ownedDigest,
    "an unrelated-table schema mutation must not perturb the scoped M3 digest",
  );

  for (const [index, tableName] of m3OwnedTableNames.entries()) {
    mutateFixture(
      `UPDATE "${tableName}" SET payload='owned-row-${index}' WHERE id=1;`,
      `${tableName} row`,
    );
    const mutatedDigest = await m3OwnedDigest();
    assert.notEqual(
      mutatedDigest,
      ownedDigest,
      `a row mutation in owned M3 table ${tableName} must change the scoped digest`,
    );
    ownedDigest = mutatedDigest;
  }
  for (const [index, tableName] of m3OwnedTableNames.entries()) {
    mutateFixture(
      `ALTER TABLE "${tableName}" ADD COLUMN "schema_probe_${index}" TEXT;`,
      `${tableName} schema`,
    );
    const mutatedDigest = await m3OwnedDigest();
    assert.notEqual(
      mutatedDigest,
      ownedDigest,
      `a schema mutation in owned M3 table ${tableName} must change the scoped digest`,
    );
    ownedDigest = mutatedDigest;
  }

  const exactOwnedDomain = await readM3OwnedDomainRegistry();
  const expectedOwnedCatalog = [
    ...m3OwnedTableNames.map((name) => ({ type: "table", name })),
    ...m3OwnedIndexNames.map((name) => ({ type: "index", name })),
  ].sort((left, right) => left.name.localeCompare(right.name));
  assert.deepEqual(
    JSON.parse(JSON.stringify(
      m3OwnedDomainHelpers.m4r07RequireExactM3OwnedCatalog(
        exactOwnedDomain.owned_catalog_json,
      ),
    )),
    expectedOwnedCatalog,
    "the executable reverse-enumeration gate must accept exactly 21 typed tables plus 27 typed indexes",
  );
  const exactOwnedProjection = m3OwnedDomainHelpers.m4r07ValidateM3OwnedDomain(
    exactOwnedDomain,
  );
  assert.deepEqual(
    JSON.parse(JSON.stringify(exactOwnedProjection.ownedSequence)),
    [],
    "the exact M3 v1 catalog must have no owned sqlite_sequence rows",
  );
  const ownedSchemaSha256 = m3OwnedDomainHelpers.m4r07ReadOnlyJsonSha256(
    exactOwnedProjection.ownedSchema,
  );
  assert.match(ownedSchemaSha256, /^[a-f0-9]{64}$/);
  assert.equal(
    exactOwnedProjection.ownedSchema.some((entry) => (
      entry.type === "index"
      && entry.name === "sqlite_autoindex_m3_events_1"
      && entry.table_name === "m3_events"
    )),
    true,
    "owned schema must include SQLite autoindexes attached to an exact M3 table",
  );
  assert.equal(exactOwnedDomain.unexpected_trigger_view_rows, 0);

  async function assertCatalogMutationRejected({
    createSql,
    dropSql,
    label,
  }) {
    mutateFixture(createSql, label);
    const mutatedDomain = await readM3OwnedDomainRegistry();
    assert.throws(
      () => m3OwnedDomainHelpers.m4r07ValidateM3OwnedDomain(mutatedDomain),
      (error) => error?.message === "m4r07_m3_owned_table_set_invalid"
        && error?.failureFamily === "m3_owned_domain_digest",
      `${label} must fail the exact typed M3 catalog with a bounded family`,
    );
    mutateFixture(dropSql, `${label} cleanup`);
  }

  for (const tableName of ["m3_rogue", "M3_evil", "m3Xevil"]) {
    await assertCatalogMutationRejected({
      createSql: `CREATE TABLE "${tableName}" (id INTEGER PRIMARY KEY, payload TEXT);`,
      dropSql: `DROP TABLE "${tableName}";`,
      label: `unexpected catalog table ${tableName}`,
    });
  }
  for (const indexName of ["m3_rogue_index", "M3_evil", "m3Xevil"]) {
    await assertCatalogMutationRejected({
      createSql: `CREATE INDEX "${indexName}" ON "unrelated_fixture" (payload);`,
      dropSql: `DROP INDEX "${indexName}";`,
      label: `unexpected catalog index ${indexName}`,
    });
  }

  mutateFixture(
    "INSERT INTO sqlite_sequence(name, seq) VALUES('m3_rogue', 7);",
    "orphan M3 sequence row",
  );
  const sequenceDomain = await readM3OwnedDomainRegistry();
  assert.throws(
    () => m3OwnedDomainHelpers.m4r07ValidateM3OwnedDomain(sequenceDomain),
    (error) => error?.message === "m4r07_m3_owned_sequence_invalid"
      && error?.failureFamily === "m3_owned_domain_digest",
    "an orphan m3_* sqlite_sequence row must fail closed through the bounded M3-domain family",
  );
  mutateFixture(
    "DELETE FROM sqlite_sequence WHERE name='m3_rogue';",
    "orphan M3 sequence row cleanup",
  );

  mutateFixture(
    'CREATE VIEW "m3_rogue_view" AS SELECT id, payload FROM unrelated_fixture;',
    "rogue M3 view",
  );
  const rogueViewDomain = await readM3OwnedDomainRegistry();
  assert.throws(
    () => m3OwnedDomainHelpers.m4r07ValidateM3OwnedDomain(rogueViewDomain),
    (error) => error?.message === "m4r07_m3_unexpected_trigger_or_view"
      && error?.failureFamily === "m3_owned_domain_digest",
    "an m3_* view must be rejected rather than merely folded into a baseline hash",
  );
  mutateFixture('DROP VIEW "m3_rogue_view";', "rogue M3 view cleanup");

  mutateFixture(
    'CREATE INDEX "rogue_owned_index" ON "m3_events" (payload);',
    "non-M3-named index on an M3 table",
  );
  const rogueIndexDomain = await readM3OwnedDomainRegistry();
  const rogueIndexProjection = m3OwnedDomainHelpers.m4r07ValidateM3OwnedDomain(
    rogueIndexDomain,
  );
  const rogueIndexSchemaSha256 = m3OwnedDomainHelpers.m4r07ReadOnlyJsonSha256(
    rogueIndexProjection.ownedSchema,
  );
  assert.notEqual(
    rogueIndexSchemaSha256,
    ownedSchemaSha256,
    "a non-M3-named index attached to an exact M3 table must change the owned-schema digest",
  );
  mutateFixture('DROP INDEX "rogue_owned_index";', "non-M3-named index cleanup");

  mutateFixture(
    'CREATE VIEW "hostile_reference_view" AS SELECT id, payload FROM "m3_role_sessions";',
    "non-M3-named view referencing M3",
  );
  const hostileViewDomain = await readM3OwnedDomainRegistry();
  assert.throws(
    () => m3OwnedDomainHelpers.m4r07ValidateM3OwnedDomain(hostileViewDomain),
    (error) => error?.message === "m4r07_m3_unexpected_trigger_or_view"
      && error?.failureFamily === "m3_owned_domain_digest",
    "a non-M3-named view whose SQL references M3 must be rejected with the bounded family",
  );
  mutateFixture('DROP VIEW "hostile_reference_view";', "hostile view cleanup");

  mutateFixture(
    [
      'CREATE TRIGGER "hostile_reference_trigger"',
      'AFTER INSERT ON "unrelated_fixture"',
      "BEGIN",
      '  UPDATE "m3_role_sessions" SET payload=payload WHERE id=NEW.id;',
      "END;",
    ].join(" "),
    "non-M3-named trigger referencing M3",
  );
  const hostileTriggerDomain = await readM3OwnedDomainRegistry();
  assert.throws(
    () => m3OwnedDomainHelpers.m4r07ValidateM3OwnedDomain(hostileTriggerDomain),
    (error) => error?.message === "m4r07_m3_unexpected_trigger_or_view"
      && error?.failureFamily === "m3_owned_domain_digest",
    "a non-M3-named trigger whose SQL references M3 must be rejected with the bounded family",
  );

  await assert.rejects(
    () => sqliteReadCutHelpers.m4r07WithFailureFamily(
      m4r07ReadOnlySqliteTableSha3Manifest({
        databaseTarget: pathToFileURL(databasePath).href + "?mode=ro&immutable=1",
        label: "m3_owned_missing_table",
        tableNames: [...m3OwnedTableNames, "m3_missing_owned_table"],
      }),
      "m4r07_m3_owned_domain_digest_failed",
      "m3_owned_domain_digest",
    ),
    (error) => error?.message === "m4r07_m3_owned_domain_digest_failed"
      && error?.failureFamily === "m3_owned_domain_digest",
    "an incomplete exact-table manifest must retain its bounded M3-domain failure family",
  );
  await assert.rejects(
    () => sqliteReadCutHelpers.m4r07WithFailureFamily(
      sqliteReadOnlyProbes.m4r07ReadOnlySqliteRows({
        databaseTarget: pathToFileURL(databasePath).href + "?mode=ro&immutable=1",
        label: "m3_missing_row_source",
        query: "SELECT * FROM m3_missing_owned_table;",
      }),
      "m4r07_m3_snapshot_read_failed",
      "m3_provider_snapshot_read",
    ),
    (error) => error?.message === "m4r07_m3_snapshot_read_failed"
      && error?.failureFamily === "m3_provider_snapshot_read",
    "an invalid M3 row projection must retain its bounded snapshot-read family",
  );
  await assert.rejects(
    () => sqliteReadCutHelpers.m4r07WithFailureFamily(
      sqliteReadOnlyProbes.m4r07ReadOnlySqliteLogicalSha3({
        databaseTarget: join(m3WalDigestDirectory, "missing-provider.sqlite3"),
        label: "provider_missing",
      }),
      "m4r07_provider_domain_digest_failed",
      "provider_domain_digest",
    ),
    (error) => error?.message === "m4r07_provider_domain_digest_failed"
      && error?.failureFamily === "provider_domain_digest",
    "a provider digest failure must retain its dedicated bounded family",
  );
} finally {
  await rm(m3WalDigestDirectory, { recursive: true, force: true });
}

const liveWalDirectory = await mkdtemp(join(tmpdir(), "m4r07-live-wal-cut-"));
let liveWalWriter = null;
let liveWalWriterClosed = false;
try {
  const liveWalPath = join(liveWalDirectory, "live-wal.sqlite3");
  const liveWalSidecarPath = `${liveWalPath}-wal`;
  const liveShmSidecarPath = `${liveWalPath}-shm`;
  const createLiveWalFixture = spawnSync(
    "/usr/bin/sqlite3",
    [
      liveWalPath,
      [
        "CREATE TABLE wal_probe (id INTEGER PRIMARY KEY, payload TEXT NOT NULL);",
        "INSERT INTO wal_probe(id, payload) VALUES(1, 'main-baseline');",
      ].join(" "),
    ],
    { encoding: "utf8" },
  );
  assert.equal(
    createLiveWalFixture.status,
    0,
    createLiveWalFixture.stderr || "live WAL fixture setup failed",
  );
  await chmod(liveWalPath, 0o644);
  const baselineTableSha3 = await m4r07ReadOnlySqliteTableSha3Manifest({
    databaseTarget: liveWalPath,
    label: "live_wal_baseline_table",
    tableNames: ["wal_probe"],
  });
  const baselineFullSha3 = await sqliteReadOnlyProbes.m4r07ReadOnlySqliteLogicalSha3({
    databaseTarget: liveWalPath,
    label: "live_wal_baseline_full",
  });

  liveWalWriter = spawn("/usr/bin/sqlite3", [liveWalPath], {
    shell: false,
    stdio: ["pipe", "pipe", "pipe"],
  });
  liveWalWriter.once("close", () => {
    liveWalWriterClosed = true;
  });
  let writerStdout = "";
  let writerStderr = "";
  const writerReady = new Promise((resolveReady, rejectReady) => {
    const timeout = setTimeout(
      () => rejectReady(new Error("live WAL writer readiness timeout")),
      5_000,
    );
    liveWalWriter.stdout.on("data", (chunk) => {
      writerStdout += chunk.toString("utf8");
      if (writerStdout.includes("M4R07_WAL_READY")) {
        clearTimeout(timeout);
        resolveReady();
      }
    });
    liveWalWriter.stderr.on("data", (chunk) => {
      writerStderr += chunk.toString("utf8");
    });
    liveWalWriter.once("error", (error) => {
      clearTimeout(timeout);
      rejectReady(error);
    });
    liveWalWriter.once("close", (exitCode, signal) => {
      clearTimeout(timeout);
      rejectReady(new Error(`live WAL writer closed early:${exitCode}:${signal}`));
    });
  });
  liveWalWriter.stdin.write([
    ".bail on",
    "PRAGMA journal_mode=WAL;",
    "PRAGMA wal_autocheckpoint=0;",
    "INSERT INTO wal_probe(id, payload) VALUES(2, 'wal-only-committed');",
    "SELECT 'M4R07_WAL_READY';",
    "",
  ].join("\n"));
  await writerReady;
  assert.equal(writerStderr, "", "live WAL writer must commit without diagnostics");
  await Promise.all([
    chmod(liveWalSidecarPath, 0o644),
    chmod(liveShmSidecarPath, 0o644),
  ]);

  const immutableMainRows = await sqliteReadOnlyProbes.m4r07ReadOnlySqliteRows({
    databaseTarget: `${pathToFileURL(liveWalPath).href}?mode=ro&immutable=1`,
    label: "live_wal_immutable_main",
    query: "SELECT COUNT(*) AS row_count FROM wal_probe;",
  });
  assert.deepEqual(
    JSON.parse(JSON.stringify(immutableMainRows)),
    [{ row_count: 1 }],
    "the second committed row must remain WAL-only when sidecars are ignored",
  );

  const liveCut = await sqliteReadCutHelpers.m4r07PrepareSqliteReadCut({
    databasePath: liveWalPath,
    label: "live_wal",
    expectedMode: 0o644,
  });
  assert.equal(liveCut.target, liveWalPath);
  assert.ok(liveCut.metadata.wal.bytes > 0 && liveCut.metadata.shm.bytes > 0);
  assert.match(liveCut.metadata.main.sha256, /^[a-f0-9]{64}$/);
  assert.match(liveCut.metadata.wal.sha256, /^[a-f0-9]{64}$/);
  assert.equal(Object.hasOwn(liveCut.metadata.shm, "sha256"), false);
  assert.equal(Object.hasOwn(liveCut.metadata.shm, "modified_at_ms"), false);
  assert.equal(Object.hasOwn(liveCut.metadata.shm, "changed_at_ms"), false);

  const liveRows = await sqliteReadOnlyProbes.m4r07ReadOnlySqliteRows({
    databaseTarget: liveCut.target,
    label: "live_wal_rows",
    query: "SELECT id, payload FROM wal_probe ORDER BY id;",
  });
  assert.deepEqual(
    JSON.parse(JSON.stringify(liveRows)),
    [
      { id: 1, payload: "main-baseline" },
      { id: 2, payload: "wal-only-committed" },
    ],
    "the production row helper must include the committed WAL-only row",
  );
  const liveTableSha3 = await m4r07ReadOnlySqliteTableSha3Manifest({
    databaseTarget: liveCut.target,
    label: "live_wal_table",
    tableNames: ["wal_probe"],
  });
  const liveFullSha3 = await sqliteReadOnlyProbes.m4r07ReadOnlySqliteLogicalSha3({
    databaseTarget: liveCut.target,
    label: "live_wal_full",
  });
  assert.notEqual(liveTableSha3, baselineTableSha3);
  assert.notEqual(liveFullSha3, baselineFullSha3);
  await sqliteReadCutHelpers.m4r07AssertSqliteReadCutStable(liveCut);

  const [mainBytesAfter, walBytesAfter] = await Promise.all([
    readFile(liveWalPath),
    readFile(liveWalSidecarPath),
  ]);
  assert.equal(mainBytesAfter.length, liveCut.metadata.main.bytes);
  assert.equal(walBytesAfter.length, liveCut.metadata.wal.bytes);
  assert.equal(
    createHash("sha256").update(mainBytesAfter).digest("hex"),
    liveCut.metadata.main.sha256,
    "read-only WAL probes must not alter the main database bytes",
  );
  assert.equal(
    createHash("sha256").update(walBytesAfter).digest("hex"),
    liveCut.metadata.wal.sha256,
    "read-only WAL probes must not alter committed WAL bytes",
  );
} finally {
  if (liveWalWriter && !liveWalWriterClosed) {
    liveWalWriter.stdin.end(".exit\n");
    await new Promise((resolveClose) => {
      const timeout = setTimeout(() => {
        if (!liveWalWriterClosed && Number.isSafeInteger(liveWalWriter.pid)) {
          try {
            process.kill(liveWalWriter.pid, "SIGKILL");
          } catch {
            // The SQLite fixture writer may close between the check and signal.
          }
        }
        resolveClose();
      }, 2_000);
      liveWalWriter.once("close", () => {
        clearTimeout(timeout);
        resolveClose();
      });
    });
  }
  await rm(liveWalDirectory, { recursive: true, force: true });
}

const sentinelExact = sourceFunction(
  "m4r07FrozenSentinelChecksExact",
  "function m4r07BuildIdentityChecksExact(",
);
const m4r07FrozenSentinelChecksExact = runInNewContext(
  `
const M4R07_FROZEN_SENTINEL_CHECK_STAGES = [
  "after_launch_2_r05", "after_launch_5_r02", "after_launch_6_r06",
  "after_launch_9_r03", "after_launch_12_r04_final_read_only",
];
${sentinelExact}
m4r07FrozenSentinelChecksExact;
`,
);
const exactChecks = [
  "after_launch_2_r05",
  "after_launch_5_r02",
  "after_launch_6_r06",
  "after_launch_9_r03",
  "after_launch_12_r04_final_read_only",
].map((stage) => ({
  stage,
  read_only_query_only_connection_count: 4,
  logical_projection_exact: true,
}));
assert.equal(m4r07FrozenSentinelChecksExact(exactChecks), true);
assert.equal(
  m4r07FrozenSentinelChecksExact([
    ...exactChecks.slice(0, 4),
    { ...exactChecks[4], read_only_query_only_connection_count: 3 },
  ]),
  false,
  "a sentinel check with fewer than four read-only connections must fail",
);

const buildIdentitySource = sourceFunction(
  "m4r07BuildIdentity",
  "async function m4r07CreateBuildIdentitySentinel(",
);
for (const token of [
  "const bundleMetadata = await lstat(debugAppBundlePath)",
  "!bundleMetadata.isDirectory()",
  "bundleMetadata.isSymbolicLink()",
  "await realpath(debugAppBundlePath) !== debugAppBundlePath",
  'debugAppExecutablePath,\n    "build_identity",\n    1024 * 1024 * 1024',
  'debugAppInfoPlistPath,\n    "build_identity_info_plist",\n    64 * 1024',
  "bundleIdentifierMatches.length !== 1",
  "bundleIdentifierMatches[0][1].trim() !== DEBUG_APP_BUNDLE_IDENTIFIER",
  "sha256(infoPlistBytes) !== infoPlist.sha256",
  "bundle_identifier: DEBUG_APP_BUNDLE_IDENTIFIER",
  "bundle_info_plist_sha256: infoPlist.sha256",
]) {
  assert.ok(buildIdentitySource.includes(token), `build identity must freeze ${token}`);
}
const buildSentinelSource = sourceFunction(
  "m4r07CreateBuildIdentitySentinel",
  "async function m4r07AssertBuildIdentityFrozen(",
);
const buildFreezeSource = sourceFunction(
  "m4r07AssertBuildIdentityFrozen",
  "function m4r07RawEvidenceLeak(",
);
for (const token of [
  "debug_executable_bytes: baseline.bytes",
  "debug_executable_sha256: baseline.sha256",
  "bundle_identifier: baseline.bundle_identifier",
  "bundle_info_plist_sha256: baseline.bundle_info_plist_sha256",
]) {
  assert.ok(buildSentinelSource.includes(token), `build sentinel must retain ${token}`);
}
for (const token of [
  "current.bytes !== sentinel.debug_executable_bytes",
  "current.sha256 !== sentinel.debug_executable_sha256",
  "current.bundle_identifier !== sentinel.bundle_identifier",
  "current.bundle_info_plist_sha256",
  "sentinel.bundle_info_plist_sha256",
]) {
  assert.ok(buildFreezeSource.includes(token), `build freeze must compare ${token}`);
}
const buildSpawnIndex = launcher.indexOf("buildResult = await runChild(\n      tauriCliPath,");
const freshBundleIndex = launcher.indexOf(
  "await assertFreshDebugAppExecutable(bundleBuildStartedAtMs)",
  buildSpawnIndex,
);
const sealedBundleIndex = launcher.indexOf(
  "await sealAndVerifyDebugAppBundle(normalBuildEnvironment)",
  freshBundleIndex,
);
const r07BuildSentinelIndex = launcher.indexOf(
  "m4r07BuildIdentitySentinel = await m4r07CreateBuildIdentitySentinel()",
  sealedBundleIndex,
);
const r07LaunchOneIndex = launcher.indexOf(
  "await runM4R05OrdinaryConversationSuite({",
  r07BuildSentinelIndex,
);
const r07AdmissionIndex = launcher.indexOf(
  'm4r07AssertLaunch8UiValidationExcludedArtifactsAbsent(\n        root,\n        "admission",',
);
assert.ok(
  r07AdmissionIndex >= 0
    && buildSpawnIndex > r07AdmissionIndex
    && freshBundleIndex > buildSpawnIndex
    && sealedBundleIndex > freshBundleIndex
    && r07BuildSentinelIndex > sealedBundleIndex
    && r07LaunchOneIndex > r07BuildSentinelIndex
    && !r07Dispatch.includes("m4r07PrepareUiCaptureContract("),
  "R07 must prove excluded UI artifacts absent before build and never prepare capture before App launch #1",
);

const r03Spawn = between(
  "function spawnM4R03OrdinaryClockApp({",
  "async function closeM4R03AppAtDeadline(",
  "R03 capture spawn",
);
const r03Close = between(
  "async function m4r03AwaitCloseGrace(",
  "async function m4r03KillAndAwaitCloseGrace(",
  "R03 close grace",
);
const r03Normal = between(
  "async function runM4R03NormalPhase({",
  "async function runM4R03ServerClockSuite({",
  "R03 capture-ready phase",
);
assert.ok(
  launcher.includes("const M4R03_ORDINARY_CLOCK_CHILD_CLOSE_GRACE_MS = 2 * 1000")
    && r03Close.includes("M4R03_ORDINARY_CLOCK_CHILD_CLOSE_GRACE_MS")
    && r03Spawn.includes("M4R07_RECOVERY_UI_CAPTURE_ENV")
    && !r03Spawn.includes("awaitR07CaptureReady")
    && !r03Spawn.includes("captureReadyPromise")
    && r03Normal.includes("r07UiCaptureContract && phase !== \"recovery_timer\"")
    && r03Normal.includes("m4r07AwaitRootCaptureReady({")
    && r03Normal.includes("m4r07WriteCaptureAck(")
    && r03Normal.includes("expectedFingerprints: r07PrevalidatedCapture.fingerprints")
    && r03Normal.includes("expectedNonceSha256: sha256(nonce)")
    && r03Normal.includes("expectedProcessIdSha256: sha256(String(pid))"),
  "the UI capture must use the live-child root-file handshake and preserve the 2s close grace",
);

const captureReadyAnnouncement = sourceFunction(
  "m4r07AnnounceUiCaptureReady",
  "async function m4r07AwaitRootCaptureReady(",
);
assert.deepEqual(
  sourceStringArray("M4R07_PUBLIC_UI_CAPTURE_READY_FIELDS"),
  [
    "schema_version",
    "event",
    "capture_semantics",
    "repository_relative_path",
    "capture_method",
    "capture_disable_diff",
    "capture_call_count",
    "canonical_bundle_identifier",
    "app_selector_kind",
    "app_selector_repository_relative_path",
    "app_selector_sha256",
    "expected_app_state_app_sha256",
    "bundle_info_plist_sha256",
    "app_selector_executable_sha256",
    "phase",
    "nonce_sha256",
    "app_process_id_sha256",
    "state_sha256",
    "dom_recovery_markers_sha256",
    "screenshot_visible_markers_sha256",
    "ready_file_sha256",
    "signal_not_before_at_ms",
    "capture_deadline_at_ms",
  ],
  "the public v3 capture-ready signal must have one exact portable field set",
);
assert.ok(
  launcher.includes(
    'const M4R07_PUBLIC_UI_CAPTURE_READY_SCHEMA =\n  "syn.m4r07.ui-capture-ready.v3"',
  )
    && launcher.includes(
      'const M4R07_RECOVERY_UI_CAPTURE_ATTESTATION_SCHEMA =\n  "syn.m4r07.computer-use-ui-capture-attestation.v3"',
    )
    && captureReadyAnnouncement.includes(
      "schema_version: M4R07_PUBLIC_UI_CAPTURE_READY_SCHEMA",
    )
    && captureReadyAnnouncement.includes(
      "canonical_bundle_identifier: contract.canonical_bundle_identifier",
    )
    && captureReadyAnnouncement.includes("app_selector_kind: contract.app_selector_kind")
    && captureReadyAnnouncement.includes(
      "contract.app_selector_repository_relative_path",
    )
    && captureReadyAnnouncement.includes("app_selector_sha256: contract.app_selector_sha256")
    && captureReadyAnnouncement.includes(
      "expected_app_state_app_sha256: contract.app_selector_sha256",
    )
    && captureReadyAnnouncement.includes("M4R07_PUBLIC_UI_CAPTURE_READY_FIELDS")
    && !captureReadyAnnouncement.includes("debugAppBundlePath")
    && !captureReadyAnnouncement.includes("app_selector:")
    && !/^\s+app_state_app_sha256:/m.test(captureReadyAnnouncement),
  "the public v3 capture-ready signal must publish only the portable selector identity projection",
);

const selectorIdentityContractSource = sourceFunction(
  "m4r07UiCaptureAppSelectorContractFailure",
  "function m4r07LiveUiCaptureAppSelectorContractFailure(",
);
const selectorLiveContractSource = sourceFunction(
  "m4r07LiveUiCaptureAppSelectorContractFailure",
  "async function m4r07PrepareUiCaptureContract(",
);
const selectorDesktopRoot = "/portable-checkout/prototypes/productized-desktop-shell";
const selectorRepositoryRelativePath =
  "prototypes/productized-desktop-shell/src-tauri/target/debug/bundle/macos/CodexGovernanceWorkbench.app";
const selectorDebugAppBundlePath = resolve(
  selectorDesktopRoot,
  "src-tauri/target/debug/bundle/macos/CodexGovernanceWorkbench.app",
);
const selectorSha256 = createHash("sha256")
  .update(selectorDebugAppBundlePath)
  .digest("hex");
const m4r07LiveUiCaptureAppSelectorContractFailure = runInNewContext(
  `
const DEBUG_APP_BUNDLE_IDENTIFIER = "local.codex.governance.workbench";
const M4R07_RECOVERY_UI_CAPTURE_APP_SELECTOR_KIND = "absolute_app_bundle_path";
const M4R07_ORDINARY_PRODUCT_REACCEPTANCE_APP_SELECTOR_REPOSITORY_RELATIVE_PATH =
  ${JSON.stringify(selectorRepositoryRelativePath)};
const desktopRoot = ${JSON.stringify(selectorDesktopRoot)};
const debugAppBundlePath = ${JSON.stringify(selectorDebugAppBundlePath)};
function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}
function m4r02FirstInvalidField(checks) {
  return checks.find(([, accepted]) => !accepted)?.[0] ?? null;
}
function m4r02IsLowerHexSha256(value) {
  return typeof value === "string" && /^[a-f0-9]{64}$/.test(value);
}
${selectorIdentityContractSource}
${selectorLiveContractSource}
m4r07LiveUiCaptureAppSelectorContractFailure;
`,
  { createHash, resolve },
);
const validSelectorContract = {
  capture_method: "sky.get_app_state",
  capture_disable_diff: true,
  capture_call_count: 1,
  canonical_bundle_identifier: "local.codex.governance.workbench",
  app_selector_kind: "absolute_app_bundle_path",
  app_selector_repository_relative_path: selectorRepositoryRelativePath,
  app_selector_sha256: selectorSha256,
  app_state_app_sha256: selectorSha256,
};
assert.equal(
  m4r07LiveUiCaptureAppSelectorContractFailure(validSelectorContract),
  null,
  "the selector identity must bind the canonical bundle, relative path, and exact full-path hash",
);
for (const [field, tamperedValue] of [
  ["capture_method", "sky.get_screenshot"],
  ["capture_disable_diff", false],
  ["capture_call_count", 2],
  ["canonical_bundle_identifier", "local.example.wrong"],
  ["app_selector_kind", "bundle_identifier"],
  ["app_selector_repository_relative_path", "prototypes/wrong.app"],
  ["app_selector_sha256", sha(799)],
  ["app_state_app_sha256", sha(798)],
]) {
  assert.equal(
    m4r07LiveUiCaptureAppSelectorContractFailure({
      ...validSelectorContract,
      [field]: tamperedValue,
    }),
    field,
    `a tampered ${field} must be rejected`,
  );
}

const captureValidation = sourceFunction(
  "m4r07ValidateUiCapture",
  "async function assertPrelaunchRootLayout(",
);
assert.deepEqual(
  sourceStringArray("M4R07_RECOVERY_UI_CAPTURE_ATTESTATION_FIELDS"),
  [
    "schema_version",
    "capture_semantics",
    "capture_tool",
    "capture_method",
    "capture_disable_diff",
    "capture_call_count",
    "canonical_bundle_identifier",
    "app_selector_kind",
    "app_selector_repository_relative_path",
    "app_selector_sha256",
    "app_state_app_sha256",
    "bundle_info_plist_sha256",
    "app_selector_executable_sha256",
    "phase",
    "nonce_sha256",
    "process_id_sha256",
    "driver_state_sha256",
    "dom_recovery_markers_sha256",
    "screenshot_visible_markers_sha256",
    "ready_file_sha256",
    "public_signal_sha256",
    "accessibility_tree_sha256",
    "screenshot_sha256",
    "screenshot_bytes",
    "captured_at_utc",
    "window_only_capture",
    "expected_accessibility_due_recovery_markers_observed",
    "expected_screenshot_markers_visible",
  ],
  "the raw v3 attestation must have one exact post-tick field set",
);
for (const token of [
  "m4r07ReadStablePrivateArtifact(",
  "pngSignature",
  "bytes.readUInt32BE(16)",
  "bytes.readUInt32BE(20)",
  "attestation.nonce_sha256 !== contract.recovery_timer_nonce_sha256",
  "attestation.process_id_sha256 !== contract.recovery_timer_app_process_id_sha256",
  "attestation.driver_state_sha256 !== contract.recovery_timer_state_sha256",
  "m4r07LiveUiCaptureAppSelectorContractFailure(attestation) !== null",
  "attestation.screenshot_sha256 !== captureSha256",
  "attestation.window_only_capture !== true",
  "attestation.expected_accessibility_due_recovery_markers_observed !== true",
  "attestation.expected_screenshot_markers_visible !== true",
]) {
  assert.ok(captureValidation.includes(token), `UI evidence validation must include ${token}`);
}
assert.equal(
  captureValidation.includes('"app_selector",'),
  false,
  "v3 raw attestation must reject the legacy raw app_selector field",
);
assert.ok(
  r03Normal.indexOf("m4r07WriteCaptureAck(")
    < r03Normal.indexOf("closeM4R03AppAtDeadline(")
    && r03Normal.indexOf("expectedFingerprints: r07PrevalidatedCapture.fingerprints")
      < r03Normal.indexOf("return {")
    && r03Normal.includes("m4r07CleanupCaptureHandshake(r07UiCaptureContract)"),
  "prevalidation and ack must happen while child #8 is live, then post-close fingerprint validation and cleanup must precede #9",
);
assert.ok(
  r03Normal.indexOf("let r07PrevalidatedCapture = null;")
    < r03Normal.indexOf("try {")
    && r03Normal.indexOf("m4r07CleanupCaptureHandshake(r07UiCaptureContract)")
      < r03Normal.indexOf("m4r07CleanupValidatedCaptureEvidence(")
    && r03Normal.includes("(primaryError || handshakeCleanupError)")
    && r03Normal.includes("if (handshakeCleanupError) throw handshakeCleanupError"),
  "every final #8 failure, including handshake cleanup failure, must remove only fully prevalidated evidence without losing the primary error",
);

const uiContractSource = sourceFunction(
  "m4r07UiEvidenceContractFailure",
  "const M4R07_PHASE_RECEIPT_BINDING_FIELDS =",
);
const m4r07UiEvidenceContractFailure = runInNewContext(
  `
const M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_CAPTURE_REPOSITORY_RELATIVE_PATH =
  "docs/harness/reports/M4R07-isolated-product-reacceptance-evidence/home-due-recovery.png";
const DEBUG_APP_BUNDLE_IDENTIFIER = "local.codex.governance.workbench";
const M4R07_RECOVERY_UI_CAPTURE_APP_SELECTOR_KIND = "absolute_app_bundle_path";
const M4R07_ORDINARY_PRODUCT_REACCEPTANCE_APP_SELECTOR_REPOSITORY_RELATIVE_PATH =
  ${JSON.stringify(selectorRepositoryRelativePath)};
const desktopRoot = ${JSON.stringify(selectorDesktopRoot)};
const debugAppBundlePath = ${JSON.stringify(selectorDebugAppBundlePath)};
const M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_CAPTURE_MAX_BYTES = 10 * 1024 * 1024;
const M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_CAPTURE_MAX_DIMENSION = 8192;
const M4R07_RECOVERY_UI_CAPTURE_ATTESTATION_SCHEMA =
  "syn.m4r07.computer-use-ui-capture-attestation.v3";
const M4R07_RECOVERY_UI_CAPTURE_SEMANTICS =
  "post_tick_fresh_home_visible_recovery.v1";
const M4R07_SCREENSHOT_VISIBLE_MARKERS_SHA256 =
  ${JSON.stringify(createHash("sha256").update(JSON.stringify({ visible_markers: ["提醒", "FIRED"] })).digest("hex"))};
function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}
function m4r02IsLowerHexSha256(value) {
  return typeof value === "string" && /^[a-f0-9]{64}$/.test(value);
}
function m4r02FirstInvalidField(checks) {
  return checks.find(([, accepted]) => !accepted)?.[0] ?? null;
}
function m4r02HasExactObjectFields(value, fields) {
  return value !== null
    && typeof value === "object"
    && !Array.isArray(value)
    && Object.keys(value).length === fields.length
    && fields.every((field) => Object.hasOwn(value, field));
}
${uiContractSource}
m4r07UiEvidenceContractFailure;
`,
  { createHash, resolve },
);
const validUiEvidence = {
  repository_relative_path:
    "docs/harness/reports/M4R07-isolated-product-reacceptance-evidence/home-due-recovery.png",
  mime_type: "image/png",
  bytes: 24,
  sha256: sha(800),
  width: 1,
  height: 1,
  recovery_timer_nonce_sha256: validLedger[7].nonce_sha256,
  recovery_timer_app_process_id_sha256: validLedger[7].app_process_id_sha256,
  recovery_timer_state_sha256: sha(801),
  computer_use_capture_attestation: {
    schema_version: "syn.m4r07.computer-use-ui-capture-attestation.v3",
    capture_semantics: "post_tick_fresh_home_visible_recovery.v1",
    capture_tool: "computer-use:@oai/sky",
    ...validSelectorContract,
    bundle_info_plist_sha256: sha(804),
    app_selector_executable_sha256: sha(805),
    nonce_sha256: validLedger[7].nonce_sha256,
    process_id_sha256: validLedger[7].app_process_id_sha256,
    driver_state_sha256: sha(801),
    dom_recovery_markers_sha256: sha(806),
    screenshot_visible_markers_sha256: createHash("sha256")
      .update(JSON.stringify({ visible_markers: ["提醒", "FIRED"] }))
      .digest("hex"),
    ready_file_sha256: sha(808),
    public_signal_sha256: sha(809),
    accessibility_tree_sha256: sha(802),
    screenshot_sha256: sha(800),
    screenshot_bytes: 24,
    attestation_sha256: sha(803),
    capture_time_bound: true,
    window_only_capture: true,
    expected_accessibility_due_recovery_markers_observed: true,
    expected_screenshot_markers_visible: true,
  },
};
const validUiBuild = {
  bundle_identifier: "local.codex.governance.workbench",
  bundle_info_plist_sha256: sha(804),
  executable_sha256: sha(805),
};
const validateUiEvidence = (uiEvidence, build = validUiBuild) => m4r07UiEvidenceContractFailure(
  uiEvidence,
  validLedger,
  build,
);
assert.equal(validateUiEvidence(validUiEvidence), null);
const missingSelectorIdentityUiEvidence = structuredClone(validUiEvidence);
delete missingSelectorIdentityUiEvidence.computer_use_capture_attestation
  .canonical_bundle_identifier;
assert.equal(
  validateUiEvidence(missingSelectorIdentityUiEvidence),
  "ui_attestation_fields",
  "a v3 projection missing a selector identity field must be rejected",
);
assert.equal(
  validateUiEvidence(
    {
      ...validUiEvidence,
      computer_use_capture_attestation: {
        ...validUiEvidence.computer_use_capture_attestation,
        app_selector: "local.codex.governance.workbench",
      },
    },
  ),
  "ui_attestation_fields",
  "a legacy app_selector extra field must be rejected from v3",
);
for (const [field, tamperedValue] of [
  ["capture_method", "sky.get_screenshot"],
  ["capture_disable_diff", false],
  ["capture_call_count", 2],
  ["canonical_bundle_identifier", "local.example.wrong"],
  ["app_selector_kind", "bundle_identifier"],
  ["app_selector_repository_relative_path", "prototypes/wrong.app"],
  ["app_selector_sha256", sha(799)],
  ["app_state_app_sha256", sha(798)],
  ["bundle_info_plist_sha256", sha(797)],
  ["app_selector_executable_sha256", sha(796)],
  ["screenshot_visible_markers_sha256", sha(793)],
]) {
  assert.equal(
    validateUiEvidence(
      {
        ...validUiEvidence,
        computer_use_capture_attestation: {
          ...validUiEvidence.computer_use_capture_attestation,
          [field]: tamperedValue,
        },
      },
    ),
    "ui_attestation",
    `portable evidence with tampered ${field} must be rejected`,
  );
}
for (const [field, tamperedValue] of [
  ["bundle_identifier", "local.example.wrong"],
  ["bundle_info_plist_sha256", sha(795)],
  ["executable_sha256", sha(794)],
]) {
  assert.equal(
    validateUiEvidence(validUiEvidence, {
      ...validUiBuild,
      [field]: tamperedValue,
    }),
    "ui_attestation",
    `portable evidence must reject a build-side ${field} cross-binding mismatch`,
  );
}
assert.equal(
  validateUiEvidence(
    {
      ...validUiEvidence,
      computer_use_capture_attestation: {
        schema_version: "syn.m4r07.computer-use-ui-capture-attestation.v1",
        capture_tool: "computer-use:@oai/sky",
        app_selector: "local.codex.governance.workbench",
        nonce_sha256: validLedger[7].nonce_sha256,
        process_id_sha256: validLedger[7].app_process_id_sha256,
        driver_state_sha256: sha(801),
        accessibility_tree_sha256: sha(802),
        screenshot_sha256: sha(800),
        screenshot_bytes: 24,
        attestation_sha256: sha(803),
        capture_time_bound: true,
        window_only_capture: true,
        expected_due_recovery_markers_visible: true,
      },
    },
  ),
  "ui_attestation_fields",
  "the v1 bundle-id app_selector projection must be rejected",
);
assert.equal(
  validateUiEvidence({
    ...validUiEvidence,
    computer_use_capture_attestation: {
      ...validUiEvidence.computer_use_capture_attestation,
      schema_version: "syn.m4r07.computer-use-ui-capture-attestation.v2",
    },
  }),
  "ui_attestation",
  "the pre-tick v2 attestation schema must be rejected",
);
assert.equal(
  validateUiEvidence(
    { ...validUiEvidence, recovery_timer_nonce_sha256: sha(999) },
  ),
  "ui_r03_launch8_binding",
  "a capture whose nonce is not launch #8 must be rejected",
);
assert.equal(
  validateUiEvidence(
    {
      ...validUiEvidence,
      computer_use_capture_attestation: {
        ...validUiEvidence.computer_use_capture_attestation,
        screenshot_sha256: sha(998),
      },
    },
  ),
  "ui_attestation",
  "a PNG/attestation hash mismatch must be rejected",
);

const rawScanner = sourceFunction(
  "m4r07RawEvidenceLeak",
  "function m4r07ProjectRouteSlot(",
);
const m4r07RawEvidenceLeak = runInNewContext(
  `${rawScanner}\nm4r07RawEvidenceLeak;`,
);
assert.notEqual(
  m4r07RawEvidenceLeak({ nested: { nonce: "raw-secret" } }),
  null,
  "portable contract must reject raw nonce values",
);
assert.equal(
  m4r07RawEvidenceLeak(validUiEvidence),
  null,
  "the valid v3 portable UI projection must not contain the absolute selector path",
);
assert.notEqual(
  m4r07RawEvidenceLeak({
    ...validUiEvidence,
    computer_use_capture_attestation: {
      ...validUiEvidence.computer_use_capture_attestation,
      exact_app_selector: selectorDebugAppBundlePath,
    },
  }),
  null,
  "portable evidence containing the actual absolute App selector must be rejected",
);
assert.notEqual(
  m4r07RawEvidenceLeak({ nested: "C:\\private\\receipt" }),
  null,
  "portable contract must reject a Windows raw path too",
);
assert.equal(
  m4r07RawEvidenceLeak({ evidence_sha256: sha(1), rows: 0 }),
  null,
  "hash-only evidence is allowed by the raw evidence scanner",
);

const prelaunchExpectedSource = sourceFunction(
  "m4r07ExpectedPrelaunchAbsentRelativePaths",
  "function m4r07PrelaunchManifestContractFailure(",
);
const prelaunchContractSource = sourceFunction(
  "m4r07PrelaunchManifestContractFailure",
  "function m4r07HistoricalArtifactsContractFailure(",
);
const prelaunchValidators = runInNewContext(
  `
const PRELAUNCH_ROOT_ENTRY_NAMES = [
  "profile.json", "fixture", "workflow-state", "app-data", "codex-db", "logs",
];
const M4R02_ORDINARY_COMPOSITION_PHASES = ["initialize", "mutate", "readback"];
const M4R02_ORDINARY_COMPOSITION_RECEIPT_PREFIX = "m4r02-ordinary-composition-";
const M4R03_ORDINARY_CLOCK_PHASES = ["arm", "recovery_timer", "repeat"];
const M4R03_ORDINARY_CLOCK_RECEIPT_PREFIX = "m4r03-ordinary-clock-";
const M4R04_ORDINARY_ROUTE_PHASES = ["work_item", "proposal", "restart_negative"];
const M4R04_ORDINARY_ROUTE_RECEIPT_PREFIX = "m4r04-ordinary-route-";
const M4R05_ORDINARY_CONVERSATION_PHASES = ["two_rounds_arm", "restart_continue_failure"];
const M4R05_ORDINARY_CONVERSATION_RECEIPT_PREFIX = "m4r05-ordinary-conversation-";
const M4R06_ORDINARY_LEGACY_READ_RECEIPT_FILE = "m4r06-ordinary-legacy-read-read_and_replay.json";
const M4R07_ORDINARY_PRODUCT_REACCEPTANCE_COMPOSITE_FILE =
  "M4R07-isolated-product-reacceptance-closeout-behavior-receipt.json";
function m4r02IsLowerHexSha256(value) {
  return typeof value === "string" && /^[a-f0-9]{64}$/.test(value);
}
function m4r02HasExactObjectFields(value, fields) {
  return value !== null
    && typeof value === "object"
    && !Array.isArray(value)
    && Object.keys(value).length === fields.length
    && fields.every((field) => Object.hasOwn(value, field));
}
function m4r02FirstInvalidField(checks) {
  return checks.find(([, accepted]) => !accepted)?.[0] ?? null;
}
function m4r05CanonicalJson(value) { return JSON.stringify(value); }
${prelaunchExpectedSource}
${prelaunchContractSource}
({ m4r07ExpectedPrelaunchAbsentRelativePaths, m4r07PrelaunchManifestContractFailure });
`,
  { join },
);
const validPrelaunchManifest = {
  schema_version: "syn.m4r07.prelaunch-root-manifest.v1",
  root_entries: ["app-data", "codex-db", "fixture", "logs", "profile.json", "workflow-state"],
  fixture_catalog_sha256: sha(710),
  profile_sha256: sha(711),
  fixture_project_empty: true,
  app_data_empty: true,
  codex_db_empty: true,
  logs_empty: true,
  absent_relative_paths: JSON.parse(JSON.stringify(
    prelaunchValidators.m4r07ExpectedPrelaunchAbsentRelativePaths(),
  )),
  canonical_fixture_profile_purpose: true,
};
assert.equal(
  prelaunchValidators.m4r07PrelaunchManifestContractFailure(validPrelaunchManifest),
  null,
);
assert.equal(
  prelaunchValidators.m4r07PrelaunchManifestContractFailure({
    ...validPrelaunchManifest,
    absent_relative_paths: validPrelaunchManifest.absent_relative_paths.slice(1),
  }),
  "prelaunch_empty_and_absent",
  "a prelaunch manifest missing one required absence must be rejected",
);

const historyContractSource = sourceFunction(
  "m4r07HistoricalArtifactsContractFailure",
  "function m4r07PhaseBindingsContractFailure(",
);
const historyValidator = runInNewContext(
  `
const M4R07_HISTORICAL_ARTIFACT_PATHS = [{ label: "R01" }, { label: "R02" }];
const M4R07_HISTORICAL_ARTIFACT_ALLOWED_MODES = new Set([0o600, 0o644]);
function m4r02IsLowerHexSha256(value) {
  return typeof value === "string" && /^[a-f0-9]{64}$/.test(value);
}
function m4r02HasExactObjectFields(value, fields) {
  return value !== null
    && typeof value === "object"
    && !Array.isArray(value)
    && Object.keys(value).length === fields.length
    && fields.every((field) => Object.hasOwn(value, field));
}
function m4r02FirstInvalidField(checks) {
  return checks.find(([, accepted]) => !accepted)?.[0] ?? null;
}
function m4r05CanonicalJson(value) { return JSON.stringify(value); }
${historyContractSource}
m4r07HistoricalArtifactsContractFailure;
`,
);
const validHistory = {
  before: [
    { label: "R01", bytes: 1, sha256: sha(720), mode: 0o600, nlink: 1 },
    { label: "R02", bytes: 2, sha256: sha(721), mode: 0o644, nlink: 1 },
  ],
  after: [
    { label: "R01", bytes: 1, sha256: sha(720), mode: 0o600, nlink: 1 },
    { label: "R02", bytes: 2, sha256: sha(721), mode: 0o644, nlink: 1 },
  ],
  unchanged: true,
};
assert.equal(historyValidator(validHistory), null);
assert.equal(
  historyValidator({
    ...validHistory,
    after: [{ ...validHistory.after[0], sha256: sha(722) }, validHistory.after[1]],
  }),
  "history_exact",
  "historical R01-R06 SHA drift must be rejected",
);
assert.equal(
  historyValidator({
    ...validHistory,
    after: [{ ...validHistory.after[0], mode: 0o700 }, validHistory.after[1]],
  }),
  "history_shape",
  "historical artifact mode drift outside 0600/0644 must be rejected",
);

const phaseBindingSource = sourceFunction(
  "m4r07PhaseBindingsContractFailure",
  "function m4r07R05EvidenceContractFailure(",
);
const m4r07PhaseBindingsContractFailure = runInNewContext(
  `
const M4R07_PHASE_RECEIPT_BINDING_FIELDS = [
  "r05_two_rounds_arm", "r05_restart_continue_failure", "r02_initialize",
  "r02_mutate", "r02_readback", "r06_read_and_replay", "r03_arm",
  "r03_recovery_timer", "r03_repeat", "r04_work_item", "r04_proposal",
  "r04_restart_negative",
];
function m4r02HasExactObjectFields(value, fields) {
  return value !== null
    && typeof value === "object"
    && !Array.isArray(value)
    && Object.keys(value).length === fields.length
    && fields.every((field) => Object.hasOwn(value, field));
}
${phaseBindingSource}
m4r07PhaseBindingsContractFailure;
`,
);
const validPhaseBindings = Object.fromEntries([
  "r05_two_rounds_arm",
  "r05_restart_continue_failure",
  "r02_initialize",
  "r02_mutate",
  "r02_readback",
  "r06_read_and_replay",
  "r03_arm",
  "r03_recovery_timer",
  "r03_repeat",
  "r04_work_item",
  "r04_proposal",
  "r04_restart_negative",
].map((field, index) => [field, validLedger[index].receipt_sha256]));
assert.equal(m4r07PhaseBindingsContractFailure(validPhaseBindings, validLedger), null);
assert.equal(
  m4r07PhaseBindingsContractFailure(
    { ...validPhaseBindings, r03_recovery_timer: sha(723) },
    validLedger,
  ),
  "r03_recovery_timer",
  "a phase receipt binding that does not match ledger launch #8 must be rejected",
);

const r05EvidenceSource = sourceFunction(
  "m4r07R05EvidenceContractFailure",
  "function m4r07R06EvidenceContractFailure(",
);
const m4r07R05EvidenceContractFailure = runInNewContext(
  `
function m4r02IsLowerHexSha256(value) {
  return typeof value === "string" && /^[a-f0-9]{64}$/.test(value);
}
function m4r02HasExactObjectFields(value, fields) {
  return value !== null
    && typeof value === "object"
    && !Array.isArray(value)
    && Object.keys(value).length === fields.length
    && fields.every((field) => Object.hasOwn(value, field));
}
function m4r02FirstInvalidField(checks) {
  return checks.find(([, accepted]) => !accepted)?.[0] ?? null;
}
${r05EvidenceSource}
m4r07R05EvidenceContractFailure;
  `,
);
const r05SuiteCreatorSource = sourceFunction(
  "runM4R05OrdinaryConversationSuite",
  "function m4r06OrdinaryLegacyReadReceiptPath(",
);
const rawR05ArmReceipt = {
  initial_turn_count: 0,
  final_turn_count: 2,
  succeeded_turn_count: 2,
  dom_submit_clicks: 2,
  exact_replay_observed: true,
  exact_replay_turn_ref_sha256: sha(730),
  role_session_ref_sha256: sha(731),
  history_ref_sha256: sha(732),
  final_conversation_sha256: sha(733),
  database_evidence: {
    baseline: {},
    final_state: { provider: { read_transcript_calls: 7 } },
    exact_replay_zero_dispatch: true,
    restart_load_zero_dispatch: null,
  },
};
const rawR05RestartReceipt = {
  initial_turn_count: 2,
  final_turn_count: 4,
  succeeded_turn_count: 3,
  failed_turn_count: 1,
  failure_turn_ordinal: 4,
  failure_error_code: "M4_SECRETARY_PROVIDER_FAILURE",
  role_session_ref_sha256: sha(731),
  history_ref_sha256: sha(734),
  final_conversation_sha256: sha(735),
  database_evidence: {
    baseline: { provider: { read_transcript_calls: 7 } },
    final_state: {},
    exact_replay_zero_dispatch: null,
    restart_load_zero_dispatch: true,
  },
};
assert.ok(
  !Object.hasOwn(rawR05ArmReceipt, "exact_replay_zero_dispatch")
    && !Object.hasOwn(rawR05RestartReceipt, "restart_load_zero_dispatch"),
  "the regression fixture must retain the real receipt shape and never prefill the missing top-level fields",
);
let r05NonceByte = 0;
const createR05SuiteFromRawReceipts = runInNewContext(
  `${r05SuiteCreatorSource}\nrunM4R05OrdinaryConversationSuite;`,
  {
    Buffer,
    M4R05_ORDINARY_CONVERSATION_COMPOSITE_SCHEMA:
      "syn.m4.remediation.behavior-receipt.v1",
    M4R05_ORDINARY_CONVERSATION_PHASES: [
      "two_rounds_arm",
      "restart_continue_failure",
    ],
    m4r02FirstInvalidField: (checks) => (
      checks.find(([, accepted]) => !accepted)?.[0] ?? null
    ),
    m4r05CanonicalJson: JSON.stringify,
    m4r05RawEvidenceLeak: () => null,
    m4r05SnapshotWithoutReadTranscript: (value) => value,
    randomBytes: (length) => Buffer.alloc(length, r05NonceByte += 1),
    readFile: async () => Buffer.from("r05-real-receipt-shape"),
    runM4R05OrdinaryConversationPhase: async ({ phase }) => (
      phase === "two_rounds_arm"
        ? {
            phase,
            app_pid_sha256: sha(736),
            launch: { exit_code: null, launched: true, signal: "SIGKILL", timed_out: false },
            receipt_sha256: validPhaseBindings.r05_two_rounds_arm,
            receipt: rawR05ArmReceipt,
          }
        : {
            phase,
            app_pid_sha256: sha(737),
            launch: { exit_code: 0, launched: true, signal: null, timed_out: false },
            receipt_sha256: validPhaseBindings.r05_restart_continue_failure,
            receipt: rawR05RestartReceipt,
          }
    ),
    sha256: (value) => createHash("sha256").update(value).digest("hex"),
  },
);
const projectedR05Suite = await createR05SuiteFromRawReceipts({
  root: "/not-read-by-stub",
  normalBuildEnvironment: {},
  profilePath: "/not-read-by-stub/profile.json",
  reentryCapability: "not-exported",
  buildResult: { launched: true, exit_code: 0, signal: null },
});
assert.deepEqual(
  Object.keys(projectedR05Suite.actual_app.two_rounds).sort(),
  [
    "dom_submit_clicks",
    "exact_replay_observed",
    "exact_replay_turn_ref_sha256",
    "exact_replay_zero_dispatch",
    "final_turn_count",
    "initial_turn_count",
    "succeeded_turn_count",
  ],
);
assert.deepEqual(
  Object.keys(projectedR05Suite.actual_app.restart_continue_failure).sort(),
  [
    "failed_turn_count",
    "failure_error_code",
    "failure_turn_ordinal",
    "final_turn_count",
    "recovered_turn_count",
    "restart_load_zero_dispatch",
    "succeeded_turn_count",
  ],
);
assert.equal(projectedR05Suite.actual_app.two_rounds.exact_replay_zero_dispatch, true);
assert.equal(
  projectedR05Suite.actual_app.restart_continue_failure.restart_load_zero_dispatch,
  true,
);
assert.equal(
  m4r07R05EvidenceContractFailure(
    {
      phase_receipt_sha256: projectedR05Suite.phase_receipt_sha256,
      actual_app: projectedR05Suite.actual_app,
      empty_event_before_first_message: {},
      isolated_fake_provider: {},
    },
    validPhaseBindings,
  ),
  null,
  "the real nested receipt shape must project both booleans and pass the portable R05 validator",
);
function validR05Evidence() {
  return {
    phase_receipt_sha256: {
      two_rounds_arm: validPhaseBindings.r05_two_rounds_arm,
      restart_continue_failure: validPhaseBindings.r05_restart_continue_failure,
    },
    actual_app: {
      two_rounds: {
        initial_turn_count: 0,
        final_turn_count: 2,
        succeeded_turn_count: 2,
        dom_submit_clicks: 2,
        exact_replay_observed: true,
        exact_replay_zero_dispatch: true,
        exact_replay_turn_ref_sha256: sha(730),
      },
      restart_continue_failure: {
        recovered_turn_count: 2,
        final_turn_count: 4,
        succeeded_turn_count: 3,
        failed_turn_count: 1,
        failure_turn_ordinal: 4,
        failure_error_code: "M4_SECRETARY_PROVIDER_FAILURE",
        restart_load_zero_dispatch: true,
      },
      role_session_ref_sha256: sha(731),
      history_ref_sha256: sha(732),
      final_conversation_sha256: sha(733),
      same_profile: true,
      distinct_app_processes: true,
      phase_one_sigkill_confirmed: true,
      phase_two_exit_zero: true,
    },
    empty_event_before_first_message: {},
    isolated_fake_provider: {},
  };
}
assert.equal(
  m4r07R05EvidenceContractFailure(validR05Evidence(), validPhaseBindings),
  null,
);
const r05FailedTurnTamper = validR05Evidence();
r05FailedTurnTamper.actual_app.restart_continue_failure.failed_turn_count = 0;
assert.equal(
  m4r07R05EvidenceContractFailure(r05FailedTurnTamper, validPhaseBindings),
  "r05_restart_result",
  "R05 must reject a rewritten failed-turn total",
);
const r05ReplayTamper = validR05Evidence();
r05ReplayTamper.actual_app.two_rounds.exact_replay_zero_dispatch = false;
assert.equal(
  m4r07R05EvidenceContractFailure(r05ReplayTamper, validPhaseBindings),
  "r05_two_round_result",
  "R05 must reject a replay that claims dispatch side effects",
);

const r06EvidenceSource = sourceFunction(
  "m4r07R06EvidenceContractFailure",
  "function m4r07EvidenceProjectionContractFailure(",
);
const m4r07R06EvidenceContractFailure = runInNewContext(
  `
const M4R06_ORDINARY_LEGACY_READ_READER_SPECS = [];
const M4R06_ORDINARY_LEGACY_READ_R02_PREPARATION_FIELDS = [
  "r02_readback_receipt_sha256", "r02_ingestion_adapter_id_sha256",
  "same_profile", "ingestion_adapter_matches_work_item_reader",
];
const M4R06_ORDINARY_LEGACY_READ_WORK_ITEM_FIELDS = [
  "legacy_source_kind", "canonical_source_object_id_sha256", "source_owner_ref_sha256",
  "source_revision", "r02_ingestion_adapter_id_sha256",
  "reader_adapter_matches_r02_ingestion", "owner_publication_rows", "m4_current_rows",
  "m4_provenance_rows", "parity_primary_rows",
];
const M4R06_ORDINARY_LEGACY_READ_GUARDED_FALLBACK_FIELDS = [
  "eligible_row_count", "eligible_rows_all_parity_primary",
];
const M4R06_ORDINARY_LEGACY_READ_R07_UI_FALLBACK_FIELDS = [
  "open_conversation_clicks", "compatibility_fallback_roots",
  "parity_primary_attention_rows", "non_parity_rows_visible", "source_route_controls",
  "nested_summary_source_route_controls", "board_coordination_action_controls",
  "board_personal_action_controls", "source_route_clicks", "source_route_ref_sha256",
  "source_owner_ref_sha256", "source_object_type", "canonical_source_object_id_sha256",
  "source_revision", "exact_work_item_parity_binding", "consumed_marker_count",
  "success_notice_count", "active_view", "route_phase", "consumed_source_revision",
  "exact_consumed_binding",
];
const M4R06_ORDINARY_LEGACY_READ_WORK_ITEM_SOURCE_KIND =
  "RIGHT_RAIL_NOTIFICATION_AND_TODO_PROJECTION";
const M4R06_ORDINARY_LEGACY_READ_WORK_ITEM_SOURCE_OBJECT_TYPE = "workflow_attention";
function m4r02IsLowerHexSha256(value) {
  return typeof value === "string" && /^[a-f0-9]{64}$/.test(value);
}
function m4r02IsCanonicalRevision(value) {
  return typeof value === "string" && /^[1-9][0-9]*$/.test(value);
}
function m4r02HasExactObjectFields(value, fields) {
  return value !== null
    && typeof value === "object"
    && !Array.isArray(value)
    && Object.keys(value).length === fields.length
    && fields.every((field) => Object.hasOwn(value, field));
}
function m4r02FirstInvalidField(checks) {
  return checks.find(([, accepted]) => !accepted)?.[0] ?? null;
}
function m4r06ReaderReceiptContractFailure() { return null; }
function m4r06DatabaseContractFailure() { return null; }
function m4r06R07DailyReportContractFailure() { return null; }
${r06EvidenceSource}
m4r07R06EvidenceContractFailure;
`,
);
const r06UiFallbackFields = [
  "open_conversation_clicks", "compatibility_fallback_roots",
  "parity_primary_attention_rows", "non_parity_rows_visible", "source_route_controls",
  "nested_summary_source_route_controls", "board_coordination_action_controls",
  "board_personal_action_controls", "source_route_clicks", "source_route_ref_sha256",
  "source_owner_ref_sha256", "source_object_type", "canonical_source_object_id_sha256",
  "source_revision", "exact_work_item_parity_binding", "consumed_marker_count",
  "success_notice_count", "active_view", "route_phase", "consumed_source_revision",
  "exact_consumed_binding",
];
function validR06Evidence() {
  const adapter = sha(740);
  const owner = sha(741);
  const object = sha(742);
  const uiFallback = Object.fromEntries(r06UiFallbackFields.map((field) => [field, null]));
  Object.assign(uiFallback, {
    open_conversation_clicks: 1,
    compatibility_fallback_roots: 1,
    parity_primary_attention_rows: 1,
    non_parity_rows_visible: 0,
    source_route_controls: 1,
    nested_summary_source_route_controls: 0,
    board_coordination_action_controls: 0,
    board_personal_action_controls: 0,
    source_route_clicks: 1,
    source_route_ref_sha256: sha(743),
    source_owner_ref_sha256: owner,
    source_object_type: "workflow_attention",
    canonical_source_object_id_sha256: object,
    source_revision: "7",
    exact_work_item_parity_binding: true,
    consumed_marker_count: 1,
    success_notice_count: 1,
    active_view: "projects",
    route_phase: "CONSUMED",
    consumed_source_revision: "7",
    exact_consumed_binding: true,
  });
  return {
    phase_receipt_sha256: validPhaseBindings.r06_read_and_replay,
    synthetic_home_unavailable_trigger: true,
    synthetic_trigger_scope: "HOME_UNAVAILABLE_ONE_SHOT",
    ordinary_reader_report_observed: true,
    ordinary_dom_fallback_observed: true,
    r02_preparation: {
      r02_readback_receipt_sha256: validPhaseBindings.r02_readback,
      r02_ingestion_adapter_id_sha256: adapter,
      same_profile: true,
      ingestion_adapter_matches_work_item_reader: true,
    },
    report_evidence: {
      first_report_sha256: sha(744),
      exact_replay_report_sha256: sha(744),
      exact_replay_matches_first_read: true,
      zero_arg_load_calls: 2,
      actual_legacy_report_load_calls: 3,
      reader_receipts: [],
    },
    work_item_parity: {
      legacy_source_kind: "RIGHT_RAIL_NOTIFICATION_AND_TODO_PROJECTION",
      canonical_source_object_id_sha256: object,
      source_owner_ref_sha256: owner,
      source_revision: "7",
      r02_ingestion_adapter_id_sha256: adapter,
      reader_adapter_matches_r02_ingestion: true,
      owner_publication_rows: 1,
      m4_current_rows: 1,
      m4_provenance_rows: 1,
      parity_primary_rows: 1,
    },
    guarded_fallback: {
      eligible_row_count: 1,
      eligible_rows_all_parity_primary: true,
    },
    ui_fallback: uiFallback,
    database: {},
    daily_report: {},
  };
}
assert.equal(
  m4r07R06EvidenceContractFailure(validR06Evidence(), validPhaseBindings),
  null,
);
const r06RouteTamper = validR06Evidence();
r06RouteTamper.ui_fallback.route_phase = "READY";
assert.equal(
  m4r07R06EvidenceContractFailure(r06RouteTamper, validPhaseBindings),
  "r06_ui_fallback_parity_and_consumption",
  "R06 fallback must be CONSUMED after its one exact source-route click",
);
const r06BindingTamper = validR06Evidence();
r06BindingTamper.ui_fallback.exact_consumed_binding = false;
assert.equal(
  m4r07R06EvidenceContractFailure(r06BindingTamper, validPhaseBindings),
  "r06_ui_fallback_parity_and_consumption",
  "R06 fallback must retain the exact consumed tuple binding",
);

assert.deepEqual(
  sourceStringArray("M4R07_LAUNCH_8_UI_VALIDATION_FIELDS"),
  [
    "schema_version",
    "launch_ordinal",
    "phase",
    "required_by_current_contract",
    "execution_status",
    "acceptance_result",
    "reason_code",
    "product_recovery_validation_retained",
    "recovery_timer_receipt_sha256",
    "real_timer_wait_seconds",
    "computer_use_attempts",
    "screenshot_written",
    "attestation_written",
    "capture_ready_signal_written",
  ],
  "launch-8 UI validation cancellation must use one frozen exact field set",
);
const launch8UiValidationContractSource = sourceFunction(
  "m4r07Launch8UiValidationContractFailure",
  "function m4r07CreateComposite(",
);
const launch8UiValidationContract = runInNewContext(
  `
const M4R07_LAUNCH_8_UI_VALIDATION_FIELDS = ${JSON.stringify(
    sourceStringArray("M4R07_LAUNCH_8_UI_VALIDATION_FIELDS"),
  )};
const M4R07_LAUNCH_8_UI_VALIDATION_SCOPE_SCHEMA =
  "syn.m4r07.launch-8-ui-validation-scope.v1";
const M4R03_ORDINARY_CLOCK_REAL_TIMER_WAIT_SECONDS = 98;
function m4r02HasExactObjectFields(value, fields) {
  return value !== null
    && typeof value === "object"
    && !Array.isArray(value)
    && Object.keys(value).length === fields.length
    && fields.every((field) => Object.hasOwn(value, field));
}
function m4r02FirstInvalidField(checks) {
  return checks.find(([, accepted]) => !accepted)?.[0] ?? null;
}
${launch8UiValidationContractSource}
m4r07Launch8UiValidationContractFailure;
`,
);
const launch8RecoveryReceiptSha256 = sha(730);
const launch8Ledger = Array.from({ length: 8 }, (_, index) => ({
  launch_ordinal: index + 1,
  task_package: index === 7 ? "M4R03" : "OTHER",
  phase: index === 7 ? "recovery_timer" : "other",
  receipt_sha256: index === 7 ? launch8RecoveryReceiptSha256 : sha(731 + index),
}));
const launch8R03Evidence = { timer_tick: { real_timer_wait_seconds: 98 } };
function validLaunch8UiValidation() {
  return {
    schema_version: "syn.m4r07.launch-8-ui-validation-scope.v1",
    launch_ordinal: 8,
    phase: "recovery_timer",
    required_by_current_contract: false,
    execution_status: "NOT_EXECUTED",
    acceptance_result: "NOT_APPLICABLE",
    reason_code: "USER_SCOPE_EXCLUDED_LAUNCH_8_UI_VALIDATION",
    product_recovery_validation_retained: true,
    recovery_timer_receipt_sha256: launch8RecoveryReceiptSha256,
    real_timer_wait_seconds: 98,
    computer_use_attempts: 0,
    screenshot_written: false,
    attestation_written: false,
    capture_ready_signal_written: false,
  };
}
assert.equal(
  launch8UiValidationContract(
    validLaunch8UiValidation(),
    launch8Ledger,
    launch8R03Evidence,
  ),
  null,
);
for (const [field, tamperedValue] of [
  ["schema_version", "syn.m4r07.launch-8-ui-validation-scope.v0"],
  ["launch_ordinal", 7],
  ["phase", "repeat"],
  ["required_by_current_contract", true],
  ["execution_status", "PASS"],
  ["acceptance_result", "PASS"],
  ["reason_code", "LEGACY_UI_CAPTURE"],
  ["product_recovery_validation_retained", false],
  ["recovery_timer_receipt_sha256", sha(799)],
  ["real_timer_wait_seconds", 97],
  ["computer_use_attempts", 1],
  ["screenshot_written", true],
  ["attestation_written", true],
  ["capture_ready_signal_written", true],
]) {
  const tampered = validLaunch8UiValidation();
  tampered[field] = tamperedValue;
  assert.notEqual(
    launch8UiValidationContract(tampered, launch8Ledger, launch8R03Evidence),
    null,
    `launch-8 UI validation cancellation must reject tampered ${field}`,
  );
}
const launch8ExtraField = validLaunch8UiValidation();
launch8ExtraField.legacy_ui_evidence = {};
assert.equal(
  launch8UiValidationContract(launch8ExtraField, launch8Ledger, launch8R03Evidence),
  "launch_8_ui_validation_fields",
  "launch-8 UI validation cancellation must reject legacy or extra fields",
);

const portableContract = sourceFunction(
  "m4r07PortableReceiptContractFailure",
  "async function m4r07SyncDirectory(",
);
const portableFieldRegistry = between(
  "const M4R07_PORTABLE_RECEIPT_FIELDS = [",
  "function m4r07ExpectedPrelaunchAbsentRelativePaths(",
  "portable top-level field registry",
);
assert.deepEqual(
  [...portableFieldRegistry.matchAll(/^  "([a-z0-9_]+)",$/gm)].map((match) => match[1]),
  [
    "schema_version",
    "task_package",
    "outcome",
    "portable",
    "evidence_level",
    "ordinary_composition",
    "expected_app_launches",
    "observed_app_launches",
    "prelaunch_root_manifest",
    "historical_r01_r06_artifacts",
    "build",
    "flat_launch_ledger",
    "physical_spawn_audit",
    "phase_receipt_bindings",
    "launch_8_ui_validation",
    "m3_provider_frozen_sentinel",
    "evidence",
    "isolation_boundary",
  ],
  "portable receipt must have one frozen exact top-level schema",
);
for (const token of [
  "M4R07_PORTABLE_RECEIPT_FIELDS",
  "m4r07RawEvidenceLeak(value)",
  "m4r07FlatLedgerContractFailure(ledger, buildSha256)",
  "m4r07Launch8UiValidationContractFailure(",
  "value?.launch_8_ui_validation,",
  "value?.evidence?.r03_server_due_recovery,",
  "m4r06R07DailyReportContractFailure",
  "m4r07PrelaunchManifestContractFailure",
  "m4r07HistoricalArtifactsContractFailure",
  "m4r07PhaseBindingsContractFailure",
  "m4r07EvidenceProjectionContractFailure",
  "m4r07FrozenSentinelChecksExact(sentinel?.checks)",
  "logical_domain_sha256",
  "m3_owned_table_count",
  "m3_owned_index_count",
  "m3_owned_catalog_count",
  "m3_owned_catalog",
  "m3_forbidden_trigger_view_count",
  "m3_owned_table_sha3_manifest",
  "m3_owned_schema",
  "m3_owned_sequence_count",
  "m3_owned_sequence",
  "provider_full_database_sha3",
]) {
  assert.ok(portableContract.includes(token), `portable contract must gate ${token}`);
}
const portableExpectedM3OwnedCatalog = [
  ...m3OwnedTableNames.map((name) => ({ type: "table", name })),
  ...m3OwnedIndexNames.map((name) => ({ type: "index", name })),
].sort((left, right) => left.name.localeCompare(right.name));
const emptyArraySha256 = createHash("sha256")
  .update(JSON.stringify([]))
  .digest("hex");
const portableExpectedM3OwnedCatalogSha256 = createHash("sha256")
  .update(JSON.stringify(portableExpectedM3OwnedCatalog))
  .digest("hex");
const portableIsolationValidator = runInNewContext(
  `
const M4R07_PORTABLE_RECEIPT_FIELDS = [
  "schema_version", "task_package", "outcome", "portable", "evidence_level",
  "ordinary_composition", "expected_app_launches", "observed_app_launches",
  "prelaunch_root_manifest", "historical_r01_r06_artifacts", "build",
  "flat_launch_ledger", "physical_spawn_audit", "phase_receipt_bindings",
  "launch_8_ui_validation", "m3_provider_frozen_sentinel", "evidence", "isolation_boundary",
];
const M4R07_ORDINARY_PRODUCT_REACCEPTANCE_COMPOSITE_SCHEMA =
  "syn.m4.isolated-product-reacceptance.behavior-receipt.v2";
const DEBUG_APP_BUNDLE_IDENTIFIER = "local.codex.governance.workbench";
const M4R07_ORDINARY_PRODUCT_REACCEPTANCE_EXPECTED_APP_LAUNCHES = 12;
const M4R07_M3_OWNED_TABLE_NAMES = ${JSON.stringify(m3OwnedTableNames)};
const M4R07_M3_OWNED_INDEX_NAMES = ${JSON.stringify(m3OwnedIndexNames)};
function m4r07ExpectedM3OwnedCatalog() {
  return [
    ...M4R07_M3_OWNED_TABLE_NAMES.map((name) => ({ type: "table", name })),
    ...M4R07_M3_OWNED_INDEX_NAMES.map((name) => ({ type: "index", name })),
  ].sort((left, right) => left.name.localeCompare(right.name));
}
function m4r02IsLowerHexSha256(value) {
  return typeof value === "string" && /^[a-f0-9]{64}$/.test(value);
}
function m4r02HasExactObjectFields(value, fields) {
  return value !== null
    && typeof value === "object"
    && !Array.isArray(value)
    && Object.keys(value).length === fields.length
    && fields.every((field) => Object.hasOwn(value, field));
}
function m4r02FirstInvalidField(checks) {
  return checks.find(([, accepted]) => !accepted)?.[0] ?? null;
}
function m4r05CanonicalJson(value) { return JSON.stringify(value); }
function m4r07RawEvidenceLeak() { return null; }
function m4r07FlatLedgerContractFailure() { return null; }
function m4r07PhysicalSpawnAuditProjection(entries) { return entries; }
function m4r07Launch8UiValidationContractFailure() { return null; }
function m4r06R07DailyReportContractFailure() { return null; }
function m4r07PrelaunchManifestContractFailure() { return null; }
function m4r07HistoricalArtifactsContractFailure() { return null; }
function m4r07PhaseBindingsContractFailure() { return null; }
function m4r07EvidenceProjectionContractFailure() { return null; }
function m4r07BuildIdentityChecksExact() { return true; }
function m4r07FrozenSentinelChecksExact() { return true; }
${portableContract}
m4r07PortableReceiptContractFailure;
`,
  {
    sha256: (value) => createHash("sha256").update(value).digest("hex"),
  },
);
function portableIsolationFixture() {
  return {
    schema_version: "syn.m4.isolated-product-reacceptance.behavior-receipt.v2",
    task_package: "M4R07",
    outcome: "PASS",
    portable: true,
    evidence_level: "ISOLATED_PRODUCT_APP",
    ordinary_composition: true,
    expected_app_launches: 12,
    observed_app_launches: 12,
    prelaunch_root_manifest: {},
    historical_r01_r06_artifacts: { before: [], after: [], unchanged: true },
    build: {
      launched: true,
      exit_code: 0,
      signal: null,
      executable_bytes: 1,
      executable_sha256: sha(760),
      bundle_identifier: "local.codex.governance.workbench",
      bundle_info_plist_sha256: sha(767),
      identity_checks: [],
    },
    flat_launch_ledger: [],
    physical_spawn_audit: {
      count: 12,
      exact_ledger_binding: true,
      physical_spawn_audit_sha256: emptyArraySha256,
    },
    phase_receipt_bindings: {},
    launch_8_ui_validation: validLaunch8UiValidation(),
    m3_provider_frozen_sentinel: {
      business_snapshot_sha256: sha(761),
      r05_receipt_snapshot_sha256: sha(762),
      logical_domain_sha256: {
        m3_owned_table_count: 21,
        m3_owned_index_count: 27,
        m3_owned_catalog_count: 48,
        m3_owned_catalog: portableExpectedM3OwnedCatalogSha256,
        m3_forbidden_trigger_view_count: 0,
        m3_owned_table_sha3_manifest: sha(763),
        m3_owned_schema: sha(764),
        m3_owned_sequence_count: 0,
        m3_owned_sequence: emptyArraySha256,
        provider_full_database_sha3: sha(766),
      },
      checks: [],
      final_read_only_check: true,
    },
    evidence: {
      r05_persistent_conversation: {
        empty_event_before_first_message: {
          exact_empty: true,
          m3_turn_rows: 0,
          m3_start_turn_effect_rows: 0,
          provider_start_session_calls: 0,
          provider_continue_turn_calls: 0,
          provider_poll_calls: 0,
          provider_read_transcript_calls: 0,
          provider_resume_readback_calls: 0,
          provider_stop_calls: 0,
          m4_model_invocation_rows: 0,
        },
        isolated_fake_provider: {
          fake_provider_calls_observed: true,
          fake_provider_turn_rows: 1,
          fake_provider_start_session_calls: 1,
          fake_provider_continue_turn_calls: 1,
        },
      },
      r06_closeout_read_and_daily: {
        synthetic_home_unavailable_trigger: true,
        synthetic_trigger_scope: "HOME_UNAVAILABLE_ONE_SHOT",
        ordinary_reader_report_observed: true,
        ordinary_dom_fallback_observed: true,
      },
    },
    isolation_boundary: {
      real_model_attempts: 0,
      real_provider_attempts: 0,
      isolated_fake_provider_attempts_observed: true,
      external_connector_attempts: 0,
      external_network_writes: 0,
      real_codex_message_attempts: 0,
    },
  };
}
assert.equal(portableIsolationValidator(portableIsolationFixture()), null);
const legacyV1Portable = portableIsolationFixture();
legacyV1Portable.schema_version =
  "syn.m4.isolated-product-reacceptance.behavior-receipt.v1";
assert.equal(
  portableIsolationValidator(legacyV1Portable),
  "schema",
  "the cancelled-scope contract must reject the legacy v1 portable receipt",
);
const legacyUiEvidencePortable = portableIsolationFixture();
delete legacyUiEvidencePortable.launch_8_ui_validation;
legacyUiEvidencePortable.ui_evidence = {};
assert.equal(
  portableIsolationValidator(legacyUiEvidencePortable),
  "top_level_fields",
  "the cancelled-scope contract must reject the legacy ui_evidence top-level shape",
);
const m3OwnedCountTamper = portableIsolationFixture();
m3OwnedCountTamper.m3_provider_frozen_sentinel.logical_domain_sha256
  .m3_owned_table_count = 20;
assert.equal(
  portableIsolationValidator(m3OwnedCountTamper),
  "m3_provider_domain_logical_sha3",
  "portable acceptance must bind the frozen sentinel to all 21 M3-owned tables",
);
for (const [field, tamperedValue] of [
  ["m3_owned_index_count", 26],
  ["m3_owned_catalog_count", 47],
  ["m3_owned_catalog", sha(799)],
  ["m3_forbidden_trigger_view_count", 1],
  ["m3_owned_sequence_count", 1],
  ["m3_owned_sequence", sha(798)],
]) {
  const domainTamper = portableIsolationFixture();
  domainTamper.m3_provider_frozen_sentinel.logical_domain_sha256[field] = tamperedValue;
  assert.equal(
    portableIsolationValidator(domainTamper),
    "m3_provider_domain_logical_sha3",
    `portable acceptance must reject tampered ${field}`,
  );
}
const externalWriteTamper = portableIsolationFixture();
externalWriteTamper.isolation_boundary.external_network_writes = 1;
assert.notEqual(
  portableIsolationValidator(externalWriteTamper),
  null,
  "portable acceptance must reject one external network write even when all other gates pass",
);

const publicationWriter = sourceFunction(
  "publishM4R07Artifacts",
  "function m4r07StdoutReceiptEnvelope(",
);
const stdoutReceiptEnvelopeSource = sourceFunction(
  "m4r07StdoutReceiptEnvelope",
  "function m4r07FormalPublicationCandidate(",
);
const formalCandidateSelectorSource = sourceFunction(
  "m4r07FormalPublicationCandidate",
  "async function m4r07PublishFormalCandidate(",
);
const formalCandidatePublisherSource = sourceFunction(
  "m4r07PublishFormalCandidate",
  "// This policy is intentionally pure:",
);
const publicationDirectoryAndCleanupSource = sourceFunction(
  "m4r07SyncDirectory",
  "async function publishM4R07Artifacts(",
);
const manifestPublish = "const publishedManifest = await m4r07PublishPrivateNoClobber({";
const portablePublish = "const publishedPortable = await m4r07PublishPrivateNoClobber({";
const publicationSuccessPath = publicationWriter.slice(
  publicationWriter.indexOf(manifestPublish),
  publicationWriter.indexOf("  } catch (error) {"),
);
assert.ok(
  publicationWriter.includes("m4r07CleanupOwnedPublicationArtifact")
    && publicationWriter.includes("m4r07EnsureRepositoryPublicationDirectory")
    && publicationWriter.includes(
      "M4R07_ORDINARY_PRODUCT_REACCEPTANCE_EVIDENCE_MANIFEST_PATH",
    )
    && publicationWriter.includes("M4R07_ORDINARY_PRODUCT_REACCEPTANCE_PORTABLE_REPORT_PATH")
    && [...publicationWriter.matchAll(
      /m4r07AssertLaunch8UiValidationExcludedArtifactsAbsent\(/g,
    )].length === 2
    && publicationWriter.includes('"publication_initial"')
    && publicationWriter.includes('"publication_final"')
    && publicationWriter.includes("m4r07AssertPublicationReady")
    && publicationWriter.includes("root_composite_must_remain_absent")
    && !publicationWriter.includes("await rename(")
    && publicationWriter.indexOf(manifestPublish) >= 0
    && publicationWriter.indexOf(portablePublish) > publicationWriter.indexOf(manifestPublish)
    && publicationSuccessPath.trimEnd().endsWith("return;")
    && publicationDirectoryAndCleanupSource.includes("String(current.dev)")
    && publicationDirectoryAndCleanupSource.includes("String(current.ino)")
    && publicationDirectoryAndCleanupSource.includes("await realpath(current) !== current")
    && publicationDirectoryAndCleanupSource.includes("await handle.sync()"),
  "R07 v2 must use canonical parents, inode-owned cleanup, and manifest→portable no-clobber publication",
);

function sha256Bytes(value) {
  return createHash("sha256").update(value).digest("hex");
}

function canonicalJson(value) {
  if (Array.isArray(value)) return `[${value.map(canonicalJson).join(",")}]`;
  if (value && typeof value === "object") {
    return `{${Object.keys(value).sort().map((key) => (
      `${JSON.stringify(key)}:${canonicalJson(value[key])}`
    )).join(",")}}`;
  }
  return JSON.stringify(value);
}

const publicationHelperSource = [
  sourceFunction(
    "m4r07FileFingerprint",
    "function m4r07Launch8UiValidationExcludedArtifactPaths",
  ),
  sourceFunction(
    "m4r07Launch8UiValidationExcludedArtifactPaths",
    "async function m4r07AssertLaunch8UiValidationExcludedArtifactsAbsent(",
  ),
  excludedUiArtifactGate,
  publicationDirectoryAndCleanupSource,
  publicationWriter,
].join("\n");

function m4r07PublicationReceiptFixture() {
  return portableIsolationFixture();
}

const createStdoutReceiptEnvelope = runInNewContext(
  `${stdoutReceiptEnvelopeSource}\nm4r07StdoutReceiptEnvelope;`,
);
const selectFormalPublicationCandidate = runInNewContext(
  `${formalCandidateSelectorSource}\nm4r07FormalPublicationCandidate;`,
);

function createFormalCandidatePublisher(publishFormalArtifacts) {
  return runInNewContext(
    `${formalCandidatePublisherSource}\nm4r07PublishFormalCandidate;`,
    {
      m4r07PortableReceiptContractFailure: portableIsolationValidator,
      publishM4R07Artifacts: publishFormalArtifacts,
    },
  );
}

async function runM4R07PublicationInjection({
  directory,
  failBeforePortable = false,
  legacyManifest = false,
  preexistingExcludedArtifact = false,
  raceTakeover = false,
  replaceOwnedManifestBeforeCleanup = false,
  symlinkReportParent = false,
  tamperManifestSameLength = false,
  tamperPortableSameLength = false,
}) {
  await mkdir(directory, { recursive: true });
  const canonicalDirectory = await realpath(directory);
  const rootDirectory = join(canonicalDirectory, "root");
  const reportDirectory = join(canonicalDirectory, "reports");
  const evidenceDirectory = join(reportDirectory, "evidence");
  await mkdir(rootDirectory, { recursive: true });
  if (symlinkReportParent) {
    const symlinkTarget = join(canonicalDirectory, "symlink-report-target");
    await mkdir(symlinkTarget, { recursive: true });
    await symlink(symlinkTarget, reportDirectory, "dir");
  } else {
    await mkdir(evidenceDirectory, { recursive: true });
  }

  const rootFormalPath = join(rootDirectory, "M4R07-closeout.json");
  const portableFormalPath = join(reportDirectory, "M4R07-closeout.json");
  const manifestFormalPath = join(evidenceDirectory, "manifest.json");
  const screenshotFormalPath = join(evidenceDirectory, "home-due-recovery.png");
  const attestationFormalPath = join(
    evidenceDirectory,
    "home-due-recovery.attestation.json",
  );
  const captureSignalPath = join(evidenceDirectory, ".M4R07-ui-capture-ready.signal.json");
  const readyFormalPath = join(rootDirectory, "m4r07-ui-capture-ready.json");
  const ackFormalPath = join(rootDirectory, "m4r07-ui-capture-ack.json");
  if (preexistingExcludedArtifact) {
    await writeFile(screenshotFormalPath, Buffer.from("legacy-ui"), { mode: 0o600 });
  }
  const receipt = m4r07PublicationReceiptFixture();
  let injectionObserved = false;
  let observedNlinkTwo = 0;
  const externalManifestBytes = Buffer.from("external-manifest-must-survive");
  const publisher = runInNewContext(
    `${publicationHelperSource}\npublishM4R07Artifacts;`,
    {
      Buffer,
      Error,
      JSON,
      MODE_0600: 0o600,
      MODE_0700: 0o700,
      M4R07_REPOSITORY_ROOT: canonicalDirectory,
      M4R07_CLOSEOUT_EVIDENCE_MANIFEST_SCHEMA:
        "syn.m4r07.closeout-evidence-manifest.v2",
      M4R07_ORDINARY_PRODUCT_REACCEPTANCE_EVIDENCE_MANIFEST_PATH:
        manifestFormalPath,
      M4R07_ORDINARY_PRODUCT_REACCEPTANCE_PORTABLE_REPORT_PATH: portableFormalPath,
      M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_CAPTURE_PATH: screenshotFormalPath,
      M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_ATTESTATION_PATH:
        attestationFormalPath,
      M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_CAPTURE_SIGNAL_PATH: captureSignalPath,
      M4R07_RECOVERY_UI_CAPTURE_READY_FILE: "m4r07-ui-capture-ready.json",
      M4R07_RECOVERY_UI_CAPTURE_ACK_FILE: "m4r07-ui-capture-ack.json",
      Promise,
      chmod,
      dirname,
      isAbsolute,
      join,
      link: async (from, to) => {
        if (raceTakeover && to === manifestFormalPath) {
          await writeFile(to, externalManifestBytes, { flag: "wx", mode: 0o600 });
          const error = new Error("injected_manifest_race_takeover");
          error.code = "EEXIST";
          throw error;
        }
        if (failBeforePortable && to === portableFormalPath) {
          injectionObserved = true;
          assert.ok(
            !existsSync(rootFormalPath)
              && existsSync(manifestFormalPath)
              && !existsSync(portableFormalPath),
            "the injected pre-portable boundary must have no root receipt and no completion marker",
          );
          throw new Error("injected_before_portable_publish");
        }
        if (replaceOwnedManifestBeforeCleanup && to === portableFormalPath) {
          await unlink(manifestFormalPath);
          await writeFile(manifestFormalPath, externalManifestBytes, {
            flag: "wx",
            mode: 0o600,
          });
          throw new Error("injected_manifest_replacement_before_cleanup");
        }
        await link(from, to);
        if ((await lstat(to)).nlink === 2) observedNlinkTwo += 1;
      },
      lstat,
      mkdir,
      open,
      randomBytes: (length) => Buffer.alloc(length, 0x17),
      readFile,
      realpath,
      relative,
      resolve,
      sep,
      sha256: sha256Bytes,
      stat,
      unlink,
      writeFile: async (path, bytes, options) => {
        let outputBytes = legacyManifest
          && String(path).includes(".m4r07-closeout_evidence_manifest-")
          ? Buffer.from(`${JSON.stringify({
              schema_version: "syn.m4r07.ui-evidence-manifest.v1",
              artifact: {},
              portable_receipt_sha256: sha256Bytes(Buffer.from("legacy")),
              ui_attestation_sha256: "c".repeat(64),
            })}\n`)
          : bytes;
        if (
          (tamperManifestSameLength
            && String(path).includes(".m4r07-closeout_evidence_manifest-"))
          || (tamperPortableSameLength
            && String(path).includes(".m4r07-portable_report-"))
        ) {
          outputBytes = Buffer.from(outputBytes);
          outputBytes[0] ^= 0x01;
        }
        await writeFile(path, outputBytes, options);
      },
      m4r02HasExactObjectFields: (value, fields) => (
        value !== null
        && typeof value === "object"
        && !Array.isArray(value)
        && Object.keys(value).length === fields.length
        && fields.every((field) => Object.hasOwn(value, field))
      ),
      m4r05CanonicalJson: canonicalJson,
      m4r07PortableReceiptContractFailure: portableIsolationValidator,
      m4r07RequireAbsent: async (path, label) => {
        if (existsSync(path)) throw new Error(`unexpected_artifact:${label}`);
      },
      m4r07RequireRegularPrivateFile: async (path, label) => {
        const metadata = await stat(path);
        if (!metadata.isFile() || (metadata.mode & 0o777) !== 0o600) {
          throw new Error(`temporary_file_not_private:${label}`);
        }
        return metadata;
      },
    },
  );
  let publicationError = null;
  const trackedPublisher = async (...arguments_) => {
    try {
      return await publisher(...arguments_);
    } catch (error) {
      publicationError = error;
      throw error;
    }
  };
  const publicationResult = await createFormalCandidatePublisher(trackedPublisher)({
    candidate: receipt,
    suitePassed: true,
    priorFailureStage: null,
    priorExitCode: 0,
    rootCompositePath: rootFormalPath,
  });
  assert.equal(
    publicationResult.publication_completed,
    !(
      failBeforePortable
      || legacyManifest
      || preexistingExcludedArtifact
      || raceTakeover
      || replaceOwnedManifestBeforeCleanup
      || symlinkReportParent
      || tamperManifestSameLength
      || tamperPortableSameLength
    ),
  );
  assert.equal(publicationResult.exit_code, publicationResult.publication_completed ? 0 : 1);
  return {
    ackFormalPath,
    attestationFormalPath,
    captureSignalPath,
    directory: canonicalDirectory,
    externalManifestBytes,
    injectionObserved,
    manifestFormalPath,
    portableFormalPath,
    publicationResult,
    publicationError,
    observedNlinkTwo,
    readyFormalPath,
    receipt,
    rootFormalPath,
    screenshotFormalPath,
  };
}

const atomicPublicationDirectory = await mkdtemp(join(tmpdir(), "m4r07-atomic-publication-"));
try {
  const pureSuite = m4r07PublicationReceiptFixture();
  const stdoutEnvelope = createStdoutReceiptEnvelope(pureSuite, {
    failureStage: null,
    environmentUnchanged: true,
    homeInitialViewConfigPinned: true,
  });
  assert.equal(portableIsolationValidator(pureSuite), null);
  assert.equal(
    portableIsolationValidator(stdoutEnvelope),
    "top_level_fields",
    "stdout-only environment fields must never become formal portable bytes",
  );
  assert.strictEqual(
    selectFormalPublicationCandidate(pureSuite),
    pureSuite,
    "the actual selector must retain the pure suite object as the only formal candidate",
  );
  assert.notStrictEqual(
    selectFormalPublicationCandidate(pureSuite),
    stdoutEnvelope,
    "the stdout envelope must never be selected as formal bytes",
  );
  let invalidCandidatePublishCalls = 0;
  const invalidCandidateResult = await createFormalCandidatePublisher(async () => {
    invalidCandidatePublishCalls += 1;
  })({
    candidate: stdoutEnvelope,
    suitePassed: true,
    priorFailureStage: null,
    priorExitCode: 0,
    rootCompositePath: join(atomicPublicationDirectory, "invalid-candidate.json"),
  });
  assert.equal(invalidCandidatePublishCalls, 0);
  assert.equal(invalidCandidateResult.publication_attempted, false);
  assert.equal(invalidCandidateResult.publication_completed, false);
  assert.equal(invalidCandidateResult.failure_stage, "m4r07_publication_candidate_invalid");
  assert.equal(invalidCandidateResult.exit_code, 1);
  let blockedPublishCalls = 0;
  const blockedPublicationResult = await createFormalCandidatePublisher(async () => {
    blockedPublishCalls += 1;
  })({
    candidate: pureSuite,
    suitePassed: true,
    priorFailureStage: "m4r07_prior_failure",
    priorExitCode: 1,
    rootCompositePath: join(atomicPublicationDirectory, "blocked.json"),
  });
  assert.equal(blockedPublishCalls, 0);
  assert.equal(blockedPublicationResult.publication_completed, false);
  assert.equal(blockedPublicationResult.failure_stage, "m4r07_prior_failure");
  assert.equal(blockedPublicationResult.exit_code, 1);
  let nonzeroExitPublishCalls = 0;
  const nonzeroExitPublicationResult = await createFormalCandidatePublisher(async () => {
    nonzeroExitPublishCalls += 1;
  })({
    candidate: pureSuite,
    suitePassed: true,
    priorFailureStage: null,
    priorExitCode: 2,
    rootCompositePath: join(atomicPublicationDirectory, "nonzero-exit.json"),
  });
  assert.equal(nonzeroExitPublishCalls, 0);
  assert.equal(nonzeroExitPublicationResult.publication_attempted, false);
  assert.equal(nonzeroExitPublicationResult.publication_completed, false);
  assert.equal(
    nonzeroExitPublicationResult.failure_stage,
    "m4r07_publication_not_attempted",
  );
  assert.equal(nonzeroExitPublicationResult.exit_code, 1);

  const failed = await runM4R07PublicationInjection({
    directory: join(atomicPublicationDirectory, "failed"),
    failBeforePortable: true,
  });
  assert.equal(failed.injectionObserved, true, failed.publicationError?.stack);
  assert.ok(
    !existsSync(failed.rootFormalPath) && !existsSync(failed.portableFormalPath),
    "a pre-portable publish failure must leave neither a formal root PASS receipt nor portable completion marker",
  );
  assert.ok(
    !existsSync(failed.manifestFormalPath),
    "the caught pre-portable failure must clean its owned manifest; a crash residue alone is never PASS without portable",
  );

  const legacyManifest = await runM4R07PublicationInjection({
    directory: join(atomicPublicationDirectory, "legacy-manifest"),
    legacyManifest: true,
  });
  assert.ok(
    !existsSync(legacyManifest.rootFormalPath)
      && !existsSync(legacyManifest.manifestFormalPath)
      && !existsSync(legacyManifest.portableFormalPath),
    "the v2 publisher must reject and clean an old v1 UI-evidence manifest before any completion marker exists",
  );

  const preexistingExcludedArtifact = await runM4R07PublicationInjection({
    directory: join(atomicPublicationDirectory, "preexisting-ui-artifact"),
    preexistingExcludedArtifact: true,
  });
  assert.ok(
    existsSync(preexistingExcludedArtifact.screenshotFormalPath)
      && !existsSync(preexistingExcludedArtifact.rootFormalPath)
      && !existsSync(preexistingExcludedArtifact.manifestFormalPath)
      && !existsSync(preexistingExcludedArtifact.portableFormalPath),
    "publication must fail closed without deleting a pre-existing excluded UI artifact",
  );

  const raced = await runM4R07PublicationInjection({
    directory: join(atomicPublicationDirectory, "raced-manifest"),
    raceTakeover: true,
  });
  assert.equal(raced.publicationResult.exit_code, 1);
  assert.ok(
    (await readFile(raced.manifestFormalPath)).equals(raced.externalManifestBytes)
      && !existsSync(raced.portableFormalPath),
    "no-clobber publication must preserve a racing external manifest and publish no completion marker",
  );

  const replaced = await runM4R07PublicationInjection({
    directory: join(atomicPublicationDirectory, "replaced-owned-manifest"),
    replaceOwnedManifestBeforeCleanup: true,
  });
  assert.equal(replaced.publicationResult.exit_code, 1);
  assert.ok(
    (await readFile(replaced.manifestFormalPath)).equals(replaced.externalManifestBytes)
      && !existsSync(replaced.portableFormalPath),
    "dev+ino cleanup must not remove a replacement at a path formerly owned by this run",
  );

  const symlinkParent = await runM4R07PublicationInjection({
    directory: join(atomicPublicationDirectory, "symlink-parent"),
    symlinkReportParent: true,
  });
  assert.equal(symlinkParent.publicationResult.exit_code, 1);
  assert.ok(
    !existsSync(symlinkParent.manifestFormalPath)
      && !existsSync(symlinkParent.portableFormalPath),
    "a symlink publication parent must be rejected before formal writes",
  );

  for (const [label, options] of [
    ["manifest", { tamperManifestSameLength: true }],
    ["portable", { tamperPortableSameLength: true }],
  ]) {
    const tampered = await runM4R07PublicationInjection({
      directory: join(atomicPublicationDirectory, `same-length-${label}-tamper`),
      ...options,
    });
    assert.equal(tampered.publicationResult.exit_code, 1);
    assert.ok(
      !existsSync(tampered.manifestFormalPath)
        && !existsSync(tampered.portableFormalPath),
      `same-length ${label} tamper must fail exact-byte readback and clean only owned finals`,
    );
  }

  const published = await runM4R07PublicationInjection({
    directory: join(atomicPublicationDirectory, "passed"),
  });
  const [portableBytes, manifestBytes] = await Promise.all([
    readFile(published.portableFormalPath),
    readFile(published.manifestFormalPath),
  ]);
  const portableReceipt = JSON.parse(portableBytes.toString("utf8"));
  const formalManifest = JSON.parse(manifestBytes.toString("utf8"));
  assert.ok(
    !existsSync(published.rootFormalPath),
    "even a successful R07 publication must leave the formal root PASS receipt absent",
  );
  assert.equal(portableReceipt.outcome, "PASS");
  assert.equal(portableReceipt.portable, true);
  assert.equal(published.observedNlinkTwo, 2);
  for (const formalPath of [published.manifestFormalPath, published.portableFormalPath]) {
    const metadata = await lstat(formalPath);
    assert.ok(
      metadata.isFile()
        && !metadata.isSymbolicLink()
        && metadata.nlink === 1
        && (metadata.mode & 0o777) === 0o600,
      "each hard-link publication must settle to one private regular final",
    );
  }
  assert.ok(
    portableBytes.equals(Buffer.from(`${JSON.stringify(published.receipt, null, 2)}\n`)),
    "formal portable bytes must be the pure suite, not the expanded stdout envelope",
  );
  assert.deepEqual(Object.keys(formalManifest).sort(), [
    "launch_8_ui_validation_sha256",
    "portable_receipt_sha256",
    "schema_version",
  ]);
  assert.equal(
    formalManifest.schema_version,
    "syn.m4r07.closeout-evidence-manifest.v2",
  );
  assert.equal(formalManifest.portable_receipt_sha256, sha256Bytes(portableBytes));
  assert.equal(
    formalManifest.launch_8_ui_validation_sha256,
    sha256Bytes(canonicalJson(published.receipt.launch_8_ui_validation)),
  );
  assert.ok(
    !Object.hasOwn(formalManifest, "artifact")
      && !Object.hasOwn(formalManifest, "ui_attestation_sha256"),
    "the v2 manifest must not carry the removed UI-evidence payload or attestation binding",
  );
  assert.ok(
    [
      published.screenshotFormalPath,
      published.attestationFormalPath,
      published.captureSignalPath,
      published.readyFormalPath,
      published.ackFormalPath,
    ].every((path) => !existsSync(path)),
    "successful non-visual publication must leave every legacy UI handshake/evidence artifact absent",
  );
  const publishedEntries = [
    ...(await readdir(dirname(published.portableFormalPath))),
    ...(await readdir(dirname(published.manifestFormalPath))),
    ...(await readdir(dirname(published.rootFormalPath))),
  ];
  assert.ok(
    publishedEntries.every((entry) => !entry.endsWith(".tmp") && !entry.startsWith(".m4r07-")),
    "successful publication must leave no private temporary file",
  );
} finally {
  await rm(atomicPublicationDirectory, { recursive: true, force: true });
}

const finalPublishing = between(
  "if (m4r07OrdinaryProductReacceptanceMode) {\n      const m4r07FormalCandidate =",
  "process.stdout.write(`${JSON.stringify(receipt)}\\n`);",
  "R07 final publication",
);
assert.ok(
  finalPublishing.includes("m4r07FormalPublicationCandidate(")
    && finalPublishing.includes("m4r07PublishFormalCandidate({")
    && finalPublishing.includes("candidate: m4r07FormalCandidate")
    && finalPublishing.includes("!m4r07PublicationResult.publication_completed")
    && finalPublishing.includes("process.exitCode = 1")
    && !finalPublishing.includes("publishM4R07Artifacts(receipt, rootCompositePath)")
    && !finalPublishing.includes("await writeM4R07RootComposite(receipt, rootCompositePath)")
    && !finalPublishing.includes("await writeM4R07PortableReport(receipt, rootCompositePath)"),
  "R07 final publishing must select the pure suite and fail closed unless formal publication completes",
);

assert.ok(
  offline.includes('"tests/m4r07-isolated-product-reacceptance-runner.test.mjs"'),
  "offline interaction runner must register the R07 launcher contract test",
);
assert.equal(
  packageJson.scripts["test:offline-interaction"],
  "node scripts/run-offline-interaction-test.mjs",
  "package offline entry must remain the single non-App regression gateway",
);

console.log("m4r07 isolated product reacceptance runner: ok");
