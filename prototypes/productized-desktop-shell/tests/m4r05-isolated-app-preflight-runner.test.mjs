import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

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
  'const M4R05_ORDINARY_CONVERSATION_MODE_ARG = "--m4r05-ordinary-conversation"',
  '"SYN_M4R05_ORDINARY_CONVERSATION_DRIVER"',
  '"SYN_M4R05_ORDINARY_CONVERSATION_PHASE"',
  '"SYN_M4R05_ORDINARY_CONVERSATION_NONCE"',
  '"ordinary-persistent-secretary-conversation-v1"',
  '"syn_m4r05_ordinary_conversation_driver_receipt.v1"',
  '"syn.m4.remediation.behavior-receipt.v1"',
  '"m4r05-secretary-conversation-composite-receipt.json"',
  '"../../../docs/harness/reports/M4R05-persistent-secretary-conversation-behavior-receipt.json"',
  "const M4R05_ORDINARY_CONVERSATION_PHASE_TIMEOUT_MS = 210 * 1000",
  "const M4R05_ORDINARY_CONVERSATION_CHILD_CLOSE_GRACE_MS = 2 * 1000",
  "const M4R05_ORDINARY_CONVERSATION_DATABASE_FIELDS = [",
  "const M4R05_ORDINARY_CONVERSATION_DATABASE_SNAPSHOT_FIELDS = [",
  "const M4R05_ORDINARY_CONVERSATION_M3_DATABASE_FIELDS = [",
  "const M4R05_ORDINARY_CONVERSATION_PROVIDER_DATABASE_FIELDS = [",
  "const M4R05_ORDINARY_CONVERSATION_M4_DATABASE_FIELDS = [",
  "const M4R05_ORDINARY_CONVERSATION_WORKBENCH_DATABASE_FIELDS = [",
  "function m4r05DatabaseSnapshotFailure(value)",
  "function m4r05DatabaseContractFailure({",
  "value.read_only_query_only_connection_count === 6",
  "value.formal_objects_unchanged === true",
  "value.m4.formal_objects,\n    17,",
  '"workbench_db_absent"',
  '"workflow_state_absent"',
  '"storage_mode_absent"',
  '"catalog_file_count"',
  '"catalog_labels_and_bytes_sha256"',
  '"workbench_absence_and_catalog_exact"',
  "value.final_state.m3.ordered_turn_refs_sha256",
  "value.final_state.provider.ordered_client_message_refs_sha256",
]) {
  assert.ok(launcher.includes(token), `M4R05 launcher 合同缺少 ${token}`);
}

const readerStart = launcher.indexOf(
  "async function readM4R05OrdinaryConversationReceipt(",
);
const spawnStart = launcher.indexOf(
  "function spawnM4R05OrdinaryConversationApp(",
  readerStart,
);
const closeStart = launcher.indexOf(
  "async function closeM4R05AppAtDeadline(",
  spawnStart,
);
const killStart = launcher.indexOf("async function killM4R05ArmProcess(", closeStart);
const phaseStart = launcher.indexOf(
  "async function runM4R05OrdinaryConversationPhase(",
  killStart,
);
const suiteStart = launcher.indexOf(
  "async function runM4R05OrdinaryConversationSuite(",
  phaseStart,
);
const policyStart = launcher.indexOf(
  "function normalizeInheritedMarkerNames(",
  suiteStart,
);
assert.ok(
  readerStart >= 0
    && spawnStart > readerStart
    && closeStart > spawnStart
    && killStart > closeStart
    && phaseStart > killStart
    && suiteStart > phaseStart
    && policyStart > suiteStart,
  "M4R05 runner source slices 缺失或顺序错误",
);

const reader = launcher.slice(readerStart, spawnStart);
for (const binding of [
  "value.schema_version === M4R05_ORDINARY_CONVERSATION_RECEIPT_SCHEMA",
  "value.phase === phase",
  "value.launch_ordinal === expectedLaunchOrdinal",
  "value.nonce_sha256 === expectedNonceSha256",
  "value.profile_fingerprint === expectedProfileFingerprint",
  "value.process_id_sha256 === expectedProcessIdSha256",
  "value.previous_phase_receipt_sha256",
  "=== expectedPreviousReceiptSha256",
]) {
  assert.ok(reader.includes(binding), `receipt 绑定缺少 ${binding}`);
}
assert.ok(
  reader.includes("metadata.isSymbolicLink()")
    && reader.includes("M4R05_ORDINARY_CONVERSATION_RECEIPT_MAX_BYTES")
    && reader.includes("M4R05_ORDINARY_CONVERSATION_PASS_RECEIPT_FIELDS"),
  "receipt 必须做 metadata/大小/exact keyset 校验",
);

const spawnSlice = launcher.slice(spawnStart, closeStart);
assert.ok(
  spawnSlice.includes("spawn(debugAppExecutablePath, [], {")
    && spawnSlice.includes("[M4R05_ORDINARY_CONVERSATION_DRIVER_ENV]")
    && spawnSlice.includes("[M4R05_ORDINARY_CONVERSATION_PHASE_ENV]")
    && spawnSlice.includes("[M4R05_ORDINARY_CONVERSATION_NONCE_ENV]")
    && !spawnSlice.includes("MACOS_OPEN_PATH"),
  "M4R05 必须直启 bundle executable 并只注入冻结 marker",
);

const phase = launcher.slice(phaseStart, suiteStart);
const receiptIndex = phase.indexOf(
  "const receipt = await readM4R05OrdinaryConversationReceipt({",
);
const aliveIndex = phase.indexOf("if (process.isClosed())", receiptIndex);
const killIndex = phase.indexOf("killM4R05ArmProcess(process)", aliveIndex);
assert.ok(
  receiptIndex >= 0
    && aliveIndex > receiptIndex
    && killIndex > aliveIndex
    && phase.includes("expectedProcessIdSha256: sha256(String(pid))")
    && phase.includes('launch.signal !== "SIGKILL"')
    && phase.includes("launch.exit_code !== 0")
    && phase.includes("launch.signal !== null"),
  "必须 receipt→仍活→精确 SIGKILL；第二 phase 必须真实 exit0",
);

const suite = launcher.slice(suiteStart, policyStart);
assert.ok(
  suite.includes('phase: "two_rounds_arm"')
    && suite.includes('phase: "restart_continue_failure"')
    && suite.includes("new Set(Object.values(phaseNonces)).size !== 2")
    && suite.includes("expectedPreviousReceiptSha256: arm.receipt_sha256")
    && suite.includes('arm.launch.signal === "SIGKILL"')
    && suite.includes("restart.launch.exit_code === 0")
    && suite.includes("m4r05RawEvidenceLeak(phase.receipt)")
    && suite.includes("m4r05RawEvidenceLeak(composite)")
    && !suite.includes("runM4R02OrdinaryCompositionSuite({"),
  "两 launch/SHA chain/raw 扫描合同缺失，或错误复用 R02 prep",
);
assert.ok(
  suite.includes('"history_advanced"')
    && suite.includes(
      "arm.receipt.history_ref_sha256 !== restart.receipt.history_ref_sha256",
    )
    && suite.includes('"database_previous_final_exact"')
    && suite.includes("m4r05SnapshotWithoutReadTranscript(")
    && suite.includes('"database_read_transcript_monotonic"'),
  "跨 launch 必须证明 history 前进、phase1 final→phase2 baseline exact，且只放宽 READ_TRANSCRIPT 单调增长",
);
const databaseValidatorStart = launcher.indexOf(
  "function m4r05DatabaseContractFailure({",
);
const receiptValidatorStart = launcher.indexOf(
  "function m4r05PassReceiptContractFailure({",
  databaseValidatorStart,
);
const databaseValidator = launcher.slice(
  databaseValidatorStart,
  receiptValidatorStart,
);
for (const token of [
  "active_role_session_rows: 1",
  "create_role_session_readback_recorded_rows: 1",
  "turn_rows: 2",
  "turn_rows: 4",
  "succeeded_turn_rows: 3",
  "failed_turn_rows: 1",
  "continue_turn_calls: 2",
  "continue_turn_calls: 4",
  "poll_calls: 3",
  "poll_calls: 5",
  "value.previous_final_match === true",
  "value.exact_replay_zero_dispatch === true",
  "value.restart_load_zero_dispatch === true",
]) {
  assert.ok(databaseValidator.includes(token), `数据库 phase exact gate 缺少 ${token}`);
}
const receiptValidator = launcher.slice(receiptValidatorStart, readerStart);
for (const token of [
  "expectedRoleSessionRefSha256: value.role_session_ref_sha256",
  "expectedTurnRefsSha256: value.turn_refs_sha256",
  "expectedClientMessageRefsSha256: value.client_message_refs_sha256",
]) {
  assert.ok(receiptValidator.includes(token), `receipt→DB hash binding 缺少 ${token}`);
}
assert.ok(
  launcher.includes('current.toLowerCase().includes("m4_secretary_fake_")')
    && launcher.includes('current.startsWith("M4_SECRETARY_")')
    && launcher.includes('"M4_SECRETARY_PROVIDER_FAILURE"'),
  "raw/diagnostic scan 必须拒绝 fake 内部码与未登记的裸 provider code",
);

const rejectedStart = launcher.indexOf(
  "...(m4r05OrdinaryConversationSuite ?? {",
);
const rejectedEnd = launcher.indexOf(
  "...(failureStage ? { failure_stage: failureStage } : {})",
  rejectedStart,
);
const rejected = launcher.slice(rejectedStart, rejectedEnd);
assert.ok(
  rejected.includes('outcome: "REJECTED"')
    && rejected.includes("ordinary_composition: false")
    && !rejected.includes("acceptance_wrapper_calls: 0")
    && !rejected.includes("direct_repository_seed_calls: 0")
    && !rejected.includes("external_capability_attempts: 0")
    && !rejected.includes("real_provider_attempts: 0"),
  "REJECTED composite 不得填写未观察的成功性零值",
);

const portableStart = launcher.indexOf(
  'm4r05OrdinaryConversationSuite?.outcome === "PASS"',
);
const portableEnd = launcher.indexOf("process.stdout.write", portableStart);
const portable = launcher.slice(portableStart, portableEnd);
assert.ok(
  portable.includes("!failureStage")
    && portable.includes("receipt.ordinary_composition === true")
    && portable.includes("receipt.raw_text_fields_present === false")
    && portable.includes("m4r05RawEvidenceLeak(receipt) === null")
    && portable.includes("writeM4R05PortableReport(receipt)"),
  "portable receipt 必须为 actual PASS-only 且 raw scan=0",
);
assert.ok(
  offline.includes('"tests/m4r05-ordinary-conversation-driver.test.ts"')
    && offline.includes('"tests/m4r05-isolated-app-preflight-runner.test.mjs"'),
  "offline runner 必须登记 M4R05 bridge/driver 与 launcher static tests",
);

console.log("m4r05 isolated App preflight runner: ok");
