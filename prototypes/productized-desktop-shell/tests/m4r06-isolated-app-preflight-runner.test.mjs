import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
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

for (const token of [
  'const M4R06_ORDINARY_LEGACY_READ_MODE_ARG =\n  "--m4r06-ordinary-legacy-read"',
  '"SYN_M4R06_ORDINARY_LEGACY_READ_DRIVER"',
  '"SYN_M4R06_ORDINARY_LEGACY_READ_PHASE"',
  '"SYN_M4R06_ORDINARY_LEGACY_READ_NONCE"',
  '"ordinary-real-legacy-read-parity-v1"',
  '"m4r06-ordinary-legacy-read-read_and_replay.json"',
  '"m4r06-ordinary-legacy-read-composite-receipt.json"',
  '"../../../docs/harness/reports/M4R06-real-legacy-shadow-parity-fallback-behavior-receipt.json"',
  'const M4R06_ORDINARY_LEGACY_READ_PHASE_TIMEOUT_MS = 125 * 1000',
  'const M4R06_ORDINARY_LEGACY_READ_READER_SPECS = [',
  '"READER_RELATED_M4_EXCLUDING_INDEPENDENT_DAILY_SCHEDULER"',
  '"ready_to_dispatch"',
  'value.read_only_query_only_connection_count === 10',
  '"after_ui_fallback"',
  '"ui_fallback_zero_owner_delta"',
  '"actual_legacy_report_load_calls"',
  '"synthetic_home_unavailable_trigger"',
  '"actual_ui_fallback_visible"',
]) {
  assert.ok(launcher.includes(token), `M4R06 runner 合同缺少 ${token}`);
}

const readerStart = launcher.indexOf("async function readM4R06OrdinaryLegacyReadReceipt(");
const spawnStart = launcher.indexOf("function spawnM4R06OrdinaryLegacyReadApp(", readerStart);
const closeStart = launcher.indexOf("async function closeM4R06AppAtDeadline(", spawnStart);
const phaseStart = launcher.indexOf("async function runM4R06OrdinaryLegacyReadPhase(", closeStart);
const suiteStart = launcher.indexOf("async function runM4R06OrdinaryLegacyReadSuite(", phaseStart);
const policyStart = launcher.indexOf("function normalizeInheritedMarkerNames(", suiteStart);
assert.ok(
  readerStart >= 0
    && spawnStart > readerStart
    && closeStart > spawnStart
    && phaseStart > closeStart
    && suiteStart > phaseStart
    && policyStart > suiteStart,
  "M4R06 receipt/spawn/phase/suite/CLI policy 不能只是 constants-only",
);

const reader = launcher.slice(readerStart, spawnStart);
const identityStart = reader.indexOf("const identityFailure = m4r06ReceiptIdentityFailure({");
const rejectedStart = reader.indexOf('value?.outcome === "REJECTED"');
assert.ok(
  identityStart >= 0
    && rejectedStart > identityStart
    && reader.includes("expectedNonceSha256")
    && reader.includes("expectedProfileFingerprint")
    && reader.includes("expectedProcessIdSha256")
    && reader.includes("m4r06RejectedReceiptContractFailure(value)")
    && reader.includes("rejected_receipt_")
    && reader.includes("raw_rejected_receipt"),
  "stale/异进程 REJECTED receipt 必须先做 schema/task/phase/PID/profile/nonce binding，再解释失败",
);

const spawn = launcher.slice(spawnStart, closeStart);
assert.ok(
  spawn.includes("spawn(debugAppExecutablePath, [], {")
    && spawn.includes("[M4R06_ORDINARY_LEGACY_READ_DRIVER_ENV]")
    && spawn.includes("[M4R06_ORDINARY_LEGACY_READ_PHASE_ENV]")
    && spawn.includes("[M4R06_ORDINARY_LEGACY_READ_NONCE_ENV]")
    && !spawn.includes("MACOS_OPEN_PATH"),
  "R06 必须只启动一个真实 bundle executable，并注入冻结 marker",
);

const phase = launcher.slice(phaseStart, suiteStart);
assert.ok(
  phase.includes("expectedProcessIdSha256: sha256(String(pid))")
    && phase.includes("expectedNonceSha256: sha256(nonce)")
    && phase.includes("launch.exit_code !== 0")
    && phase.includes("launch.signal !== null"),
  "R06 child PID/nonce/exit 必须由本次第四 launch 精确绑定",
);

const suite = launcher.slice(suiteStart, policyStart);
assert.ok(
  suite.includes("await runM4R02OrdinaryCompositionSuite({")
    && suite.includes("r02Preparation.launches.length === 3")
    && suite.includes('entry.phase === "readback"')
    && suite.includes("expectedR02ReadbackReceiptSha256: r02Readback.receipt_sha256")
    && suite.includes('r02Readback?.receipt?.subject?.work_item_state === "ready_to_dispatch"')
    && suite.includes("allLaunches.length === 4")
    && suite.includes("new Set(allLaunches.map((entry) => entry.receipt.process_id_sha256)).size")
    && suite.includes("m4r06RawEvidenceLeak(composite)"),
  "R02 三 launch prep → 本次 readback SHA → 同 profile R06 第四 launch 的绑定链缺失",
);
assert.ok(
  launcher.includes("M4R06_ORDINARY_LEGACY_READ_READER_SPECS")
    && suite.includes("report_evidence")
    && suite.includes("synthetic_home_unavailable_trigger:")
    && suite.includes("actual_ui_fallback_visible:")
    && suite.includes("ui_fallback:")
    && suite.includes('synthetic_trigger_scope: "HOME_UNAVAILABLE_ONE_SHOT"')
    && suite.includes("ordinary_reader_report_observed: true")
    && suite.includes("ordinary_dom_fallback_observed: true")
    && suite.includes("portable: true"),
  "PASS composite 必须保留 five receipts、实际 fallback/route binding 与 portable 合同",
);
assert.ok(
  launcher.includes('"source_route_ref"')
    && launcher.includes('"opaque_route_ref"')
    && launcher.includes('"source_object_ref"')
    && launcher.includes("exact_work_item_parity_binding"),
  "portable receipt/composite 必须拒绝 DOM raw route tuple，并冻结其 Rust 已验证的 hash binding",
);

const rawScannerStart = launcher.indexOf("function m4r06RawEvidenceLeak(value) {");
const rawScannerEnd = launcher.indexOf(
  "function m4r06FingerprintFailure(value)",
  rawScannerStart,
);
const rawScannerSource = launcher.slice(rawScannerStart, rawScannerEnd);
assert.ok(
  rawScannerStart >= 0
    && rawScannerEnd > rawScannerStart
    && rawScannerSource.includes('current.includes("\\\\")'),
  "R06 raw scanner 必须命中任一单反斜杠，而不是只拒绝连续反斜杠",
);
const m4r06RawEvidenceLeak = runInNewContext(
  `${rawScannerSource}\nm4r06RawEvidenceLeak;`,
);
const windowsRawPath = "C:\\temp\\secret";
for (const [artifact, candidate] of [
  ["receipt", { report_evidence: { accidental_debug: windowsRawPath } }],
  ["composite", { launch_contract: { accidental_debug: windowsRawPath } }],
  ["portable", { published_payload: { accidental_debug: windowsRawPath } }],
]) {
  assert.notEqual(
    m4r06RawEvidenceLeak(candidate),
    null,
    `R06 ${artifact} 含单反斜杠 Windows raw path 必须被拒绝`,
  );
}
assert.equal(
  m4r06RawEvidenceLeak({
    first_report_sha256: "a".repeat(64),
    ui_fallback: {
      source_route_ref_sha256: "b".repeat(64),
      canonical_source_object_id_sha256: "c".repeat(64),
    },
  }),
  null,
  "正常哈希证据不得被 raw scanner 误拒",
);

const modeStart = launcher.indexOf("const m4r06OrdinaryLegacyReadMode =");
const scrubStart = launcher.indexOf("delete normalBuildEnvironment[M4R06_ORDINARY_LEGACY_READ_DRIVER_ENV]");
const dispatchStart = launcher.indexOf("} else if (m4r06OrdinaryLegacyReadMode) {");
const finalStart = launcher.indexOf(": m4r06OrdinaryLegacyReadMode");
const fileStart = launcher.indexOf("? M4R06_ORDINARY_LEGACY_READ_COMPOSITE_FILE");
assert.ok(
  modeStart >= 0
    && scrubStart > modeStart
    && dispatchStart > scrubStart
    && finalStart > dispatchStart
    && fileStart > finalStart
    && launcher.includes("inheritedM4R06OrdinaryLegacyReadMarkers")
    && launcher.includes("M4R06_ORDINARY_LEGACY_READ_MODE_CONFLICT"),
  "CLI parsing / inherited marker conflict / env scrub / dispatch / rejected composite / filename 必须完整接入",
);
const rejected = launcher.slice(finalStart, launcher.indexOf(": m3c07IsolatedMode", finalStart));
assert.ok(
  rejected.includes('outcome: "REJECTED"')
    && rejected.includes("portable: false")
    && rejected.includes("ordinary_composition: false")
    && rejected.includes("expected_app_launches: 4")
    && !rejected.includes("synthetic_home_unavailable_trigger: true"),
  "REJECTED composite 不得填充未观察到的成功性 fallback/zero evidence",
);

const portableWriterStart = launcher.indexOf(
  "async function writeM4R06PortableReport(value, rootCompositePath)",
);
const portableWriterEnd = launcher.indexOf(
  "async function assertPrelaunchRootLayout",
  portableWriterStart,
);
const portableWriter = launcher.slice(portableWriterStart, portableWriterEnd);
assert.ok(
  portableWriterStart >= 0
    && portableWriterEnd > portableWriterStart
    && portableWriter.includes("m4r06PortableReportContractFailure(value)")
    && portableWriter.includes("await readFile(rootCompositePath)")
    && portableWriter.includes("rootCompositeBytes.equals(portableBytes)")
    && portableWriter.includes("sha256(rootCompositeBytes) !== sha256(portableBytes)")
    && portableWriter.includes("randomBytes(12)")
    && portableWriter.includes("await rename(")
    && portableWriter.includes("M4R06_ORDINARY_LEGACY_READ_PORTABLE_REPORT_PATH")
    && portableWriter.includes("MODE_0600"),
  "R06 portable receipt 必须经 raw/contract gate、同目录临时文件原子 rename，并保持 0600",
);
const portableStart = launcher.indexOf(
  'm4r06OrdinaryLegacyReadSuite?.outcome === "PASS"',
);
const portableEnd = launcher.indexOf("process.stdout.write", portableStart);
const portable = launcher.slice(portableStart, portableEnd);
assert.ok(
  portableStart >= 0
    && portable.includes("!failureStage")
    && portable.includes('receipt.outcome === "PASS"')
    && portable.includes("receipt.ordinary_composition === true")
    && portable.includes("receipt.acceptance_wrapper_calls === 0")
    && portable.includes("receipt.direct_repository_seed_calls === 0")
    && portable.includes("receipt.manual_legacy_candidate_calls === 0")
    && portable.includes("receipt.synthetic_fixture_only === true")
    && portable.includes("receipt.synthetic_home_unavailable_trigger === true")
    && portable.includes('receipt.synthetic_trigger_scope === "HOME_UNAVAILABLE_ONE_SHOT"')
    && portable.includes("receipt.ordinary_reader_report_observed === true")
    && portable.includes("receipt.ordinary_dom_fallback_observed === true")
    && portable.includes("receipt.report_evidence?.zero_arg_load_calls === 2")
    && portable.includes("receipt.report_evidence?.actual_legacy_report_load_calls === 3")
    && portable.includes("m4r06RawEvidenceLeak(receipt) === null")
    && portable.includes("m4r06PortableReportContractFailure(receipt) === null")
    && portable.includes("writeM4R06PortableReport(receipt, rootCompositePath)"),
  "R06 portable receipt 只可由 ordinary PASS、零直连、synthetic scope 与 raw scan 全部成立时发布",
);
assert.ok(
  offline.includes('"tests/m4r06-ordinary-legacy-read-driver.test.ts"')
    && offline.includes('"tests/m4r06-isolated-app-preflight-runner.test.mjs"'),
  "offline runner 必须登记 M4R06 driver 与真实 runner static tests",
);

console.log("m4r06 isolated App preflight runner: ok");
