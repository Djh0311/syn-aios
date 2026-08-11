import { assert } from "./helpers/offlineInteractionTestUtils";

const nodeProcess = (globalThis as typeof globalThis & {
  process?: { cwd?: () => string };
}).process;
if (!nodeProcess?.cwd) throw new Error("M4R06 静态验收需要 Node cwd");
const nodeFsSpecifier: string = "node:fs";
const { readFileSync } = await import(nodeFsSpecifier) as {
  readFileSync: (path: string, encoding: "utf8") => string;
};
const root = nodeProcess.cwd();
const driver = readFileSync(
  `${root}/src-tauri/src/m4r06_ordinary_legacy_read_driver.rs`,
  "utf8",
);
const main = readFileSync(`${root}/src/main.tsx`, "utf8");
const host = readFileSync(
  `${root}/src-tauri/src/index_host_app_entrypoints.rs`,
  "utf8",
);
const lib = readFileSync(`${root}/src-tauri/src/lib.rs`, "utf8");
const secretaryAgent = readFileSync(`${root}/src-tauri/src/secretary_agent.rs`, "utf8");

for (const token of [
  '"syn.m4.remediation.behavior-receipt.v1"',
  '"ordinary-real-legacy-read-parity-v1"',
  '"read_and_replay"',
  '"syn_m4r06_ordinary_legacy_read_ipc.v1"',
  '"ordinary_zero_arg_load_secretary_legacy_read_compatibility_report_ipc"',
  '"SECRETARY_READ_MODEL_DETERMINISTIC_SUMMARY"',
  '"RIGHT_RAIL_NOTIFICATION_AND_TODO_PROJECTION"',
  '"RUNTIME_ATTENTION_PROJECTION"',
  '"REACT_PENDING_ACTION_VISIBILITY"',
  '"MEMORY_DAILY_INBOX_CANDIDATE"',
  '"m4-legacy-reader:secretary-read-model/v1"',
  '"m4-legacy-reader:right-rail-work-item/v1"',
  '"m4-legacy-reader:runtime-attention/v1"',
  '"m4-legacy-reader:react-pending-action/v1"',
  '"m4-legacy-reader:memory-daily-inbox/v1"',
  '"SERVER_LEGACY_SECRETARY_READ_MODEL_PRIMITIVES"',
  '"M2_WORK_ITEM_RIGHT_RAIL_PROJECTION"',
  '"SERVER_RUNTIME_ATTENTION_PROJECTION"',
  '"RENDERER_LOCAL_PENDING_ACTION_VISIBILITY"',
  '"SERVER_MEMORY_DAILY_CANDIDATE_STORE"',
  '"M4R06_EMPTY_SERVER_SURFACE"',
  '"M4R06_UNJOINABLE_NO_EXACT_TUPLE"',
  '"M4R06_READER_UNAVAILABLE"',
  '"M4R06_READER_REJECTED"',
  'OpenFlags::SQLITE_OPEN_NOFOLLOW',
  'PRAGMA query_only = ON',
  'source_revision = ?4',
  'source_revision = ?3',
  '"READER_RELATED_M4_EXCLUDING_INDEPENDENT_DAILY_SCHEDULER"',
  'independent_daily_scheduler_tables_excluded: true',
  'read_only_query_only_connection_count != 10',
  'const EARLY_PROCESS_DEADLINE: Duration = Duration::from_secs(110)',
  'capture_pre_renderer_database_baseline(app)?',
  'take_pre_renderer_database_baseline()?',
  'synthetic_home_unavailable_trigger: Some(synthetic_home_unavailable_trigger)',
  'actual_ui_fallback_visible: Some(true)',
  'actual_legacy_report_load_calls: Some(actual_legacy_report_load_calls)',
  'WORK_ITEM_SOURCE_OBJECT_TYPE',
  'subject.get("work_item_state").and_then(Value::as_str)',
  'Some("ready_to_dispatch")',
]) {
  assert(driver.includes(token), `M4R06 Rust driver 缺少 ${token}`);
}

const reportValidatorStart = driver.indexOf("fn validate_report(");
const readerParserStart = driver.indexOf("fn parse_reader_receipt(", reportValidatorStart);
const workItemVerifierStart = driver.indexOf(
  "fn verify_work_item_parity(",
  readerParserStart,
);
const databaseStart = driver.indexOf("fn database_evidence(", workItemVerifierStart);
assert(
  reportValidatorStart >= 0
    && readerParserStart > reportValidatorStart
    && workItemVerifierStart > readerParserStart
    && databaseStart > workItemVerifierStart,
  "报告→五 reader receipt→WorkItem DB→零增量证据必须顺序冻结",
);
const readerParser = driver.slice(readerParserStart, workItemVerifierStart);
for (const token of [
  "LEGACY_READER_SPECS",
  "reader_id != spec.reader_id || source_surface_code != spec.source_surface_code",
  '"OBSERVED" =>',
  "expected_kind == WORK_ITEM_LEGACY_SOURCE_KIND",
  "complete_tuple_count == candidate_count",
  '"EMPTY" =>',
  "EMPTY_SERVER_SURFACE_REASON",
  '"UNJOINABLE" =>',
  "UNJOINABLE_NO_EXACT_TUPLE_REASON",
  '"QUARANTINED" =>',
  "READER_UNAVAILABLE_REASON | READER_REJECTED_REASON",
]) {
  assert(readerParser.includes(token), `reader receipt frozen matrix 缺少 ${token}`);
}
assert(
  driver.includes("frozen_reader_receipt_contract_rejects_cross_kind_surface_and_state_matrix"),
  "reader receipt 必须有跨 kind/surface/state 反例测试",
);

const runtimeStart = driver.indexOf("fn run_after_runtime_ready(");
const invokeStart = driver.indexOf("fn invoke_renderer_operation(", runtimeStart);
const runtime = driver.slice(runtimeStart, invokeStart);
const baseline = runtime.indexOf("let baseline = take_pre_renderer_database_baseline()");
const uiFallback = runtime.indexOf('invoke_renderer_operation(app_handle, "ui_fallback"', baseline);
const firstRead = runtime.indexOf('invoke_renderer_operation(app_handle, "first_read"', uiFallback);
const replay = runtime.indexOf('invoke_renderer_operation(app_handle, "exact_replay"', firstRead);
const database = runtime.indexOf("let database = database_evidence(", replay);
assert(
  baseline >= 0 && uiFallback > baseline && firstRead > uiFallback && replay > firstRead && database > replay,
  "必须先取 pre-render baseline，再真实 UI fallback、零参 first/replay IPC，最后读 DB 零增量",
);
assert(
  runtime.includes("validate_r02_preparation(&paths)?")
    && runtime.includes("first_report != replay_report")
    && runtime.includes("first.reader_receipts != replay.reader_receipts"),
  "R06 必须绑定同 profile R02 readback 并要求 exact replay",
);
const installStart = driver.indexOf("pub(crate) fn install_after_runtime_ready(");
const installEnd = driver.indexOf("fn valid_ready_payload", installStart);
const install = driver.slice(installStart, installEnd);
assert(
  install.indexOf("capture_pre_renderer_database_baseline(app)?")
    < install.indexOf("app.listen_any(TAURI_IPC_READY_EVENT"),
  "pre-render DB baseline 必须在 renderer ready listener 前锁定，覆盖自动 Home fallback read",
);

const mainStart = main.indexOf("function m4r06ErrorFamily(error: unknown): string");
const mainEnd = main.indexOf("class BootErrorBoundary", mainStart);
const bridge = main.slice(mainStart, mainEnd);
assert(
  bridge.includes("zeroArgLoadCalls += 1")
    && bridge.includes("loadSecretaryLegacyReadCompatibilityReport()")
    && bridge.includes("zero_arg_load_calls: zeroArgLoadCalls")
    && bridge.includes('new Error("m4r06_report_not_ready")')
    && bridge.includes('message.includes("report_not_ready")')
    && main.includes('operation: "ui_fallback" | "first_read" | "exact_replay"')
    && bridge.includes("m4r06ObserveActualUiFallback")
    && bridge.includes('[data-secretary-compatibility-fallback="true"]')
    && bridge.includes('[data-secretary-source-route-action="OPEN"]')
    && bridge.includes("button.secretary-brief-source-link")
    && bridge.includes("getBoundingClientRect")
    && !bridge.includes("loadSecretaryHomeContext()"),
  "renderer 必须观测 ordinary Home fallback DOM、如实记录 bridge 调用次数，且不自行伪造 Home IPC",
);
assert(
  secretaryAgent.includes("consume_synthetic_home_unavailable_trigger()?")
    && secretaryAgent.includes("SecretaryHomeContextEnvelope::unavailable()"),
  "R06 synthetic Home trigger 必须只复用现有 unavailable envelope",
);

for (const token of [
  "m4r06_ordinary_legacy_read_driver::requested()",
  "m4r06_ordinary_legacy_read_driver::start_early_process_watchdog()",
  "m4r06_ordinary_legacy_read_driver::mark_ordinary_constructor_ready()",
  "m4r06_ordinary_legacy_read_driver::install_after_runtime_ready(app)",
  "m4r06_ordinary_legacy_read_driver::reject_early_setup(",
]) {
  assert(host.includes(token), `ordinary host 接线缺少 ${token}`);
}
assert(
  lib.includes("mod m4r06_ordinary_legacy_read_driver;"),
  "R06 driver 必须纳入普通 lib module",
);
assert(
  !driver.includes("m4_acceptance::")
    && !driver.includes("m4_legacy_test_right_rail_observed_batch")
    && !driver.includes("insert into")
    && !driver.includes("INSERT INTO"),
  "R06 driver 只能做普通 IPC + 只读证据，不能 seed/manual legacy candidate/acceptance 写入",
);

console.log("m4r06 ordinary legacy-read driver: ok");
