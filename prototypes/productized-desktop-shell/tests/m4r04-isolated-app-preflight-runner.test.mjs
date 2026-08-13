import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const root = process.cwd();
const launcher = readFileSync(
  `${root}/scripts/run-r4-isolated-app-preflight.mjs`,
  "utf8",
);
const offlineRunner = readFileSync(
  `${root}/scripts/run-offline-interaction-test.mjs`,
  "utf8",
);

for (const token of [
  'const M4R04_ORDINARY_ROUTE_MODE_ARG = "--m4r04-ordinary-route"',
  '"SYN_M4R04_ORDINARY_ROUTE_DRIVER"',
  '"SYN_M4R04_ORDINARY_ROUTE_PHASE"',
  '"SYN_M4R04_ORDINARY_ROUTE_NONCE"',
  '"ordinary-registered-source-route-v1"',
  '"syn_m4r04_ordinary_route_driver_receipt.v1"',
  '"syn.m4.remediation.behavior-receipt.v1"',
  '"m4r04-registered-owner-route-composite-receipt.json"',
  '"../../../docs/harness/reports/M4R04-registered-owner-exact-source-return-behavior-receipt.json"',
  "const M4R04_ORDINARY_ROUTE_PHASE_TIMEOUT_MS = 210 * 1000",
  "const M4R04_REPOSITORY_PROBE_EXPECTED_TESTS = 1",
  '"m4_source_route_resolver::tests::full_registry_returns_fixed_failures_for_stale_revision_missing_and_tamper"',
  '"m4_source_route_resolver::tests::full_registry_resolves_real_delivered_work_item_and_proposal_owner_collision"',
]) {
  assert.ok(launcher.includes(token), `M4R04 launcher 合同缺少 ${token}`);
}

const receiptFieldsStart = launcher.indexOf(
  "const M4R04_ORDINARY_ROUTE_PASS_RECEIPT_FIELDS = [",
);
const routeFieldsStart = launcher.indexOf(
  "const M4R04_ORDINARY_ROUTE_SLOT_FIELDS = [",
  receiptFieldsStart,
);
const negativeFieldsStart = launcher.indexOf(
  "const M4R04_ORDINARY_ROUTE_NEGATIVE_FIELDS = [",
  routeFieldsStart,
);
const negativeFieldsEnd = launcher.indexOf("];", negativeFieldsStart);
assert.ok(
  receiptFieldsStart >= 0
    && routeFieldsStart > receiptFieldsStart
    && negativeFieldsStart > routeFieldsStart,
  "M4R04 exact receipt field registries 缺失",
);
const receiptFields = launcher.slice(receiptFieldsStart, routeFieldsStart);
const routeFields = launcher.slice(routeFieldsStart, negativeFieldsStart);
const negativeFields = [
  ...launcher
    .slice(negativeFieldsStart, negativeFieldsEnd)
    .matchAll(/"([a-z0-9_]+)"/g),
].map((match) => match[1]);
assert.deepEqual(negativeFields, [
  "stale_error_code",
  "tampered_error_code",
  "resolver_wrapper_calls",
  "stale_ui_phase",
  "stale_notice_error_code",
  "stale_route_action_clicks",
  "active_view_before",
  "active_view_after",
  "route_phase_before",
  "route_phase_after",
  "consumed_marker_count_before",
  "consumed_marker_count_after",
  "success_notice_count_before",
  "success_notice_count_after",
  "zero_navigation",
  "zero_consume_delta",
  "zero_success_delta",
  "stale_historical_rows",
  "stale_current_rows",
  "stale_current_route_mismatch",
  "stale_revision_advanced",
]);
for (const field of [
  "schema_version",
  "phase",
  "launch_ordinal",
  "process_id_sha256",
  "profile_fingerprint",
  "nonce_sha256",
  "previous_phase_receipt_sha256",
  "ordinary_constructor",
  "ordinary_composition",
  "command_registry_surface",
  "acceptance_wrapper_calls",
  "direct_repository_seed_calls",
  "direct_resolver_calls",
  "route_action_clicks",
  "resolver_wrapper_calls",
  "work_item",
  "proposal",
  "current_work_item",
  "negative",
  "restart_continuity",
  "error_family",
]) {
  assert.ok(receiptFields.includes(`"${field}"`), `phase receipt 缺少 ${field}`);
}
for (const field of [
  "source_owner_ref",
  "source_object_type",
  "target_kind",
  "canonical_source_object_id_sha256",
  "source_revision",
  "source_route_ref_sha256",
  "source_action_dom_count",
  "route_action_clicks",
  "consumed_marker_count",
  "active_view",
  "route_phase",
  "success_notice_count",
  "raw_capability_fields_present",
  "m4_provenance_rows",
  "m4_ingestion_rows",
  "owner_publication_status",
  "owner_terminal_receipt_present",
  "current_route_match",
  "revision_advanced",
  "route_binding_match",
]) {
  assert.ok(routeFields.includes(`"${field}"`), `route slot 缺少 ${field}`);
}

const receiptReaderStart = launcher.indexOf(
  "async function readM4R04OrdinaryRouteReceipt(",
);
const spawnStart = launcher.indexOf(
  "function spawnM4R04OrdinaryRouteApp(",
  receiptReaderStart,
);
const closeStart = launcher.indexOf(
  "async function closeM4R04AppAtDeadline(",
  spawnStart,
);
const phaseStart = launcher.indexOf(
  "async function runM4R04OrdinaryRoutePhase(",
  closeStart,
);
const exactProbeStart = launcher.indexOf(
  "async function runM4R04ExactRepositoryTest(",
  phaseStart,
);
const probeStart = launcher.indexOf(
  "async function runM4R04RepositoryIntegrationProbe(",
  exactProbeStart,
);
const suiteStart = launcher.indexOf(
  "async function runM4R04OrdinaryRouteSuite(",
  probeStart,
);
const policyStart = launcher.indexOf(
  "function normalizeInheritedMarkerNames(",
  suiteStart,
);
assert.ok(
  receiptReaderStart >= 0
    && spawnStart > receiptReaderStart
    && closeStart > spawnStart
    && phaseStart > closeStart
    && exactProbeStart > phaseStart
    && probeStart > exactProbeStart
    && suiteStart > probeStart
    && policyStart > suiteStart,
  "M4R04 launcher source slices 缺失或顺序错误",
);
const reader = launcher.slice(receiptReaderStart, spawnStart);
const spawnSlice = launcher.slice(spawnStart, closeStart);
const closeSlice = launcher.slice(closeStart, phaseStart);
const phaseRunner = launcher.slice(phaseStart, exactProbeStart);
const exactProbe = launcher.slice(exactProbeStart, probeStart);
const probe = launcher.slice(probeStart, suiteStart);
const suite = launcher.slice(suiteStart, policyStart);

for (const binding of [
  "value.schema_version === M4R04_ORDINARY_ROUTE_RECEIPT_SCHEMA",
  "value.phase === phase",
  "value.launch_ordinal === expectedLaunchOrdinal",
  "value.nonce_sha256 === expectedNonceSha256",
  "value.profile_fingerprint === expectedProfileFingerprint",
  "value.process_id_sha256 === expectedProcessIdSha256",
  "value.previous_phase_receipt_sha256 === expectedPreviousReceiptSha256",
]) {
  assert.ok(reader.includes(binding), `旧/错绑 receipt 未校验 ${binding}`);
}
assert.ok(
  reader.includes("M4R04_ORDINARY_ROUTE_PASS_RECEIPT_FIELDS")
    && reader.includes("metadata.isSymbolicLink()")
    && reader.includes("metadata.size > M4R04_ORDINARY_ROUTE_RECEIPT_MAX_BYTES")
    && reader.includes("m4r04PassReceiptContractFailure({"),
  "phase receipt 必须先做 exact shape、metadata 与 PASS contract 校验",
);
assert.ok(
  spawnSlice.includes("spawn(debugAppExecutablePath, [], {")
    && !spawnSlice.includes("MACOS_OPEN_PATH")
    && spawnSlice.includes("[M4R04_ORDINARY_ROUTE_DRIVER_ENV]")
    && spawnSlice.includes("[M4R04_ORDINARY_ROUTE_PHASE_ENV]")
    && spawnSlice.includes("[M4R04_ORDINARY_ROUTE_NONCE_ENV]"),
  "M4R04 必须直接 spawn bundle executable 并只注入冻结 driver markers",
);
assert.ok(
  closeSlice.includes('typeof process.child.pid === "number"')
    && closeSlice.includes('signalProcess(process.child.pid, "SIGKILL")')
    && closeSlice.includes("await Promise.race([")
    && closeSlice.includes("M4R04_ORDINARY_ROUTE_CHILD_CLOSE_GRACE_MS")
    && closeSlice.includes('signal: "SIGKILL_UNCONFIRMED"')
    && closeSlice.includes("timed_out: true"),
  "phase deadline 必须只终止并等候 exact App child",
);
const receiptObservedIndex = phaseRunner.indexOf(
  "receipt = await readM4R04OrdinaryRouteReceipt({",
);
const realExitIndex = phaseRunner.indexOf(
  "const launch = await closeM4R04AppAtDeadline(",
);
assert.ok(
  receiptObservedIndex >= 0
    && realExitIndex > receiptObservedIndex
    && phaseRunner.includes("expectedProcessIdSha256: sha256(String(pid))")
    && phaseRunner.includes("launch.exit_code !== 0")
    && phaseRunner.includes("launch.signal !== null")
    && phaseRunner.includes("launch.timed_out"),
  "必须先观察绑定 receipt，再等待真实 App PID 成功退出后进入下一 phase",
);

assert.ok(
  exactProbe.includes('"cargo"')
    && exactProbe.includes('"--offline"')
    && exactProbe.includes('"--lib"')
    && exactProbe.includes("testIdentity")
    && exactProbe.includes('"--exact"')
    && exactProbe.includes('"--nocapture"')
    && exactProbe.includes("M4R04_REPOSITORY_PROBE_TIMEOUT_MS")
    && exactProbe.includes('stdout_sha256: stdoutHash.digest("hex")')
    && exactProbe.includes('stderr_sha256: stderrHash.digest("hex")')
    && exactProbe.includes("test_filter: testIdentity")
    && exactProbe.includes("executed_tests: executedTests")
    && exactProbe.includes("passed_tests: passedTests")
    && exactProbe.includes("identity_sentinel_observed: identitySentinelObserved")
    && exactProbe.includes("`test ${testIdentity} ... ok`")
    && exactProbe.includes("repository_integration_probe_test_count")
    && exactProbe.includes("repository_integration_probe_identity_sentinel")
    && exactProbe.includes("result.exit_code !== 0")
    && exactProbe.includes("executedTests !== M4R04_REPOSITORY_PROBE_EXPECTED_TESTS")
    && exactProbe.includes("passedTests !== M4R04_REPOSITORY_PROBE_EXPECTED_TESTS")
    && probe.includes("M4R04_REPOSITORY_FIXED_ERROR_TEST")
    && probe.includes("M4R04_REPOSITORY_OWNER_COLLISION_TEST")
    && probe.includes('evidence_level: "REPOSITORY_INTEGRATION"')
    && probe.includes('fixture_scope: "isolated_repository_test"')
    && probe.includes("gui_navigation_claim: false")
    && probe.includes("same_object_id_owner_collision: true")
    && probe.includes("owner_collision_probe: ownerCollisionProbe"),
  "repository matrix/collision 必须来自有界 full-identity exact tests，零匹配失败关门",
);
for (const code of [
  "M4_SOURCE_OWNER_UNREGISTERED",
  "M4_SOURCE_TYPE_UNREGISTERED",
  "M4_SOURCE_TARGET_MISSING",
  "M4_SOURCE_REVISION_MISMATCH",
  "M4_SOURCE_SCOPE_MISMATCH",
  "M4_SOURCE_ROUTE_TAMPERED",
  "M4_SOURCE_ROUTE_STALE",
  "M4_SOURCE_TARGET_INTEGRITY_FAILED",
]) {
  assert.ok(probe.includes(`"${code}"`), `repository matrix 缺少 ${code}`);
}

const preparationIndex = suite.indexOf("runM4R02OrdinaryCompositionSuite({");
const repositoryProbeIndex = suite.indexOf(
  "runM4R04RepositoryIntegrationProbe(normalBuildEnvironment)",
);
const workIndex = suite.indexOf('phase: "work_item"');
const proposalIndex = suite.indexOf('phase: "proposal"');
const restartIndex = suite.indexOf('phase: "restart_negative"');
assert.ok(
  preparationIndex >= 0
    && repositoryProbeIndex > preparationIndex
    && workIndex > repositoryProbeIndex
    && proposalIndex > workIndex
    && restartIndex > proposalIndex,
  "R04 必须在同 profile 的 R02 prep 与 repository probe 后顺序运行三阶段",
);
for (const token of [
  "new Set(Object.values(phaseNonces)).size",
  "nonce: phaseNonces.work_item",
  "nonce: phaseNonces.proposal",
  "nonce: phaseNonces.restart_negative",
  "expectedPreviousReceiptSha256: workItem.receipt_sha256",
  "expectedPreviousReceiptSha256: proposal.receipt_sha256",
  '"distinct_app_processes"',
  '"same_profile"',
  '"owner_collision_distinct_owner"',
  '"owner_collision_distinct_route"',
  '"current_work_item_revision_advanced"',
  '"current_work_item_route_rotated"',
]) {
  assert.ok(suite.includes(token), `cross-launch contract 缺少 ${token}`);
}

const validatorStart = launcher.indexOf(
  "function m4r04NegativeContractFailure(",
);
const readerBoundary = launcher.indexOf(
  "async function readM4R04OrdinaryRouteReceipt(",
  validatorStart,
);
const validator = launcher.slice(validatorStart, readerBoundary);
for (const token of [
  "resolver_wrapper_calls: 2",
  "resolver_wrapper_calls: 8",
  "route_action_clicks: 4",
  "minimum_refresh_clicks: 3",
  '"ordinary_registered_tauri_command_and_dom_click"',
  'value.direct_resolver_calls === 0',
  'value.route_action_clicks === expectedCounts.route_action_clicks',
  'value.navigation_clicks === expectedCounts.navigation_clicks',
  "value.refresh_clicks >= expectedCounts.minimum_refresh_clicks",
  'value.restart_continuity === true',
  '"M4_SOURCE_ROUTE_STALE"',
  '"M4_SOURCE_ROUTE_TAMPERED"',
  'value.stale_ui_phase === "FAILED"',
  'value.stale_notice_error_code === "M4_SOURCE_ROUTE_STALE"',
  "value.stale_route_action_clicks === 1",
  'value.active_view_before === "home"',
  'value.active_view_after === "home"',
  'value.route_phase_before === "IDLE"',
  'value.route_phase_after === "FAILED"',
  "value.consumed_marker_count_before === 0",
  "value.consumed_marker_count_after === 0",
  "value.success_notice_count_before === 0",
  "value.success_notice_count_after === 0",
]) {
  assert.ok(validator.includes(token), `phase exact count/negative contract 缺少 ${token}`);
}

const rejectedStart = launcher.indexOf("...(m4r04OrdinaryRouteSuite ?? {");
const rejectedEnd = launcher.indexOf(
  "...(failureStage ? { failure_stage: failureStage } : {})",
  rejectedStart,
);
const rejected = launcher.slice(rejectedStart, rejectedEnd);
assert.ok(
  rejectedStart >= 0
    && rejected.includes('outcome: "REJECTED"')
    && rejected.includes("ordinary_composition: false")
    && !rejected.includes("acceptance_wrapper_calls: 0")
    && !rejected.includes("direct_repository_seed_calls: 0")
    && !rejected.includes("direct_resolver_calls: 0")
    && !rejected.includes("zero_navigation: true")
    && !rejected.includes("zero_consume_delta: true")
    && !rejected.includes("zero_success_delta: true"),
  "REJECTED composite 不得写未观察到的成功性 zero claims",
);

const portableStart = launcher.indexOf(
  'm4r04OrdinaryRouteSuite?.outcome === "PASS"',
);
const portableEnd = launcher.indexOf("process.stdout.write", portableStart);
const portable = launcher.slice(portableStart, portableEnd);
assert.ok(
  portableStart >= 0
    && portable.includes("!failureStage")
    && portable.includes("receipt.ordinary_composition === true")
    && portable.includes("receipt.acceptance_wrapper_calls === 0")
    && portable.includes("receipt.direct_repository_seed_calls === 0")
    && portable.includes("receipt.direct_resolver_calls === 0")
    && portable.includes("receipt.repository_integration_error_matrix?.exit_code === 0")
    && portable.includes("writeM4R04PortableReport(receipt)"),
  "portable behavior receipt 只可由 actual-App + repository exact PASS 生成",
);
assert.ok(
  launcher.includes("async function writeM4R04PortableReport(value)")
    && launcher.includes(
      "await rename(temporaryPath, M4R04_ORDINARY_ROUTE_PORTABLE_REPORT_PATH)",
    )
    && launcher.includes("delete normalBuildEnvironment[M4R04_ORDINARY_ROUTE_DRIVER_ENV]")
    && launcher.includes("inheritedM4R04OrdinaryRouteMarkers.length > 0"),
  "portable report 必须原子更新，且 launcher 必须清理/拒绝继承的 R04 markers",
);
assert.ok(
  offlineRunner.includes('"tests/m4r04-secretary-source-route-ui.test.tsx"')
    && offlineRunner.includes(
      '"tests/m4r04-isolated-app-preflight-runner.test.mjs"',
    ),
  "常规 offline runner 必须同时执行 R04 UI/parser/focus 与 launcher static tests",
);

console.log("m4r04 isolated App preflight runner: ok");
