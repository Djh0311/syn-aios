import { assert } from "./helpers/offlineInteractionTestUtils";

const nodeProcess = (globalThis as typeof globalThis & {
  process?: { cwd?: () => string };
}).process;
if (!nodeProcess?.cwd) throw new Error("M4R05 static 验收需要 Node cwd");
const nodeFsSpecifier: string = "node:fs";
const { readFileSync } = await import(nodeFsSpecifier) as {
  readFileSync: (path: string, encoding: "utf8") => string;
};
const root = nodeProcess.cwd();
const main = readFileSync(`${root}/src/main.tsx`, "utf8");
const driver = readFileSync(
  `${root}/src-tauri/src/m4r05_ordinary_conversation_driver.rs`,
  "utf8",
);
const host = readFileSync(
  `${root}/src-tauri/src/index_host_app_entrypoints.rs`,
  "utf8",
);

for (const token of [
  '"syn_m4r05_ordinary_conversation_ipc.v1"',
  '"syn.m4.secretary.conversation.v1"',
  '"syn.m4.secretary.conversation-send.v1"',
  '"two_rounds_arm"',
  '"restart_continue_failure"',
  'loadSecretaryConversation()',
  'sendSecretaryMessage({',
  'client_message_ref: secondTurn.client_message_ref',
  'replay.replayed',
  '[data-secretary-open-conversation="true"]',
  '[data-secretary-conversation-state="READY"]',
  '[data-secretary-composer="true"]',
  '[data-secretary-send="true"]',
  '[data-secretary-send-pending]',
  '[data-secretary-turn-ref]',
  '[data-secretary-message-role="user"]',
  '[data-secretary-message-role="assistant"]',
  'querySelectorAll(":scope > p")',
  'open_conversation_clicks: 1',
  'blank_submit_disabled: blankSubmitDisabled',
]) {
  assert(main.includes(token), `M4R05 main bridge 缺少 ${token}`);
}

const phaseStart = main.indexOf("async function m4r05RunPhase(");
const installStart = main.indexOf(
  "async function installM4R05OrdinaryConversationTauriIpcBridge()",
  phaseStart,
);
const phase = main.slice(phaseStart, installStart);
const homeWait = phase.indexOf("const controlsDeadline");
const blankDisabled = phase.indexOf("m4r05_blank_submit_not_disabled");
const openClick = phase.indexOf("homeControls.open.click()");
const boardWait = phase.indexOf("const initialDom = await m4r05WaitForDom");
assert(
  homeWait >= 0
    && blankDisabled > homeWait
    && openClick > blankDisabled
    && boardWait > openClick,
  "必须先有界等待 Home controls/验证空载，再真实点击进入 Board 后读 READY history",
);
const domSendStart = main.indexOf("async function m4r05DomSend(");
const phaseBoundary = main.indexOf("async function m4r05RunPhase(", domSendStart);
const domSend = main.slice(domSendStart, phaseBoundary);
assert(
  domSend.includes("const sendDeadline = Date.now() + M4R05_DOM_WAIT_MS")
    && domSend.includes("m4r05WaitForDomUntil(")
    && domSend.includes("sendDeadline,"),
  "submit enable 与 terminal DOM 必须共用一个 25s deadline",
);

for (const token of [
  'Duration::from_secs(20)',
  'Duration::from_secs(140)',
  'Duration::from_secs(190)',
  '"syn_m4r05_ordinary_conversation_driver_receipt.v1"',
  '"ordinary-persistent-secretary-conversation-v1"',
  '"secretary-client-message:"',
  'suffix.len() == 32',
  'raw_text_fields_present: Some(false)',
  'client_message_refs_sha256',
  'user_messages_sha256',
  'assistant_messages_sha256',
  'phase.exit_after_receipt()',
  'const SECRETARY_ROLE_REF: &str = "role:secretary:personal-primary"',
  'const SECRETARY_SCOPE_REF: &str = "scope:personal:primary"',
  'const SECRETARY_CHANNEL_KEY: &str = "daily"',
  'value.turns.len() > ROUND_MESSAGES.len()',
  'ROUND_MESSAGES.get(ordinal)',
  'conversation_identity_transition_valid',
  'initial.history_ref != final_conversation.history_ref',
  'fn read_database_snapshot(',
  'fn validate_database_baseline(',
  'fn validate_database_evidence(',
  'read_only_query_only_connection_count != 6',
  'OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX',
  '.pragma_update(None, "query_only", "ON")',
  '"PRAGMA integrity_check"',
  '"SELECT COUNT(*) FROM pragma_foreign_key_check"',
  'const M3_HANDOFF_WRITE_TABLES: [&str; 10]',
  'const M4_FORMAL_OBJECT_TABLES: [&str; 17]',
  '`m4_daily_events` carries',
  'const WORKBENCH_FRESH_CATALOG_FILES: [&str; 2]',
  'const WORKBENCH_FRESH_CATALOG_DIRECTORIES: [&str; 2]',
  '"index-kernel/codex-index.json"',
  '"tasks/README.md"',
  'fn read_workbench_absence_snapshot(',
  'workbench_db_absent: true',
  'workflow_state_absent: true',
  'storage_mode_absent: true',
  'catalog_labels_and_bytes_sha256',
  'metadata.nlink() != 1',
  'm4r05_ordinary_conversation_workbench_catalog_directory_shape_invalid',
  'empty_workbench_directories_outside_catalog_allowlist_are_rejected',
  '"index-kernel/empty"',
  'write_forbidden_artifact',
  'workbench_artifacts_that_must_remain_absent_are_rejected',
  'm4r05_ordinary_conversation_workbench_db_must_remain_absent',
  'm4r05_ordinary_conversation_workbench_workflow_state_must_remain_absent',
  'm4r05_ordinary_conversation_workbench_storage_mode_must_remain_absent',
  'ordered_turn_refs_sha256',
  'ordered_client_message_refs_sha256',
  'ordered_turn_bindings_sha256',
  'm3_db_path: ordinary_app_data_root',
  'provider_db_path: ordinary_app_data_root',
  'm4_db_path: ordinary_app_data_root',
  'workbench_db_path: workbench_root.join("runtime-artifacts/workbench.sqlite")',
  'history_identity_must_advance_while_session_identity_stays_fixed',
  'fifth_turn_is_rejected_before_round_message_indexing',
  'conversation_identity_is_exact_secretary_personal_daily',
  'create_readback_count_is_one_not_start_turn_count',
]) {
  assert(driver.includes(token), `M4R05 Rust driver 缺少 ${token}`);
}
assert(
  !driver.includes("database_probe_not_frozen"),
  "数据库 probe 已冻结实现，旧 fail-closed stub 必须消失",
);
assert(
  !driver.includes('    "m4_daily_events",'),
  "scheduler 的 TimerFired bookkeeping 不得计入 R05 conversation formal 指纹",
);
const runtimeStart = driver.indexOf("fn run_after_runtime_ready(");
const invokeStart = driver.indexOf("fn invoke_renderer_operation(", runtimeStart);
const runtime = driver.slice(runtimeStart, invokeStart);
const baselineRead = runtime.indexOf("let baseline = read_database_snapshot(");
const rendererInvoke = runtime.indexOf("let result = invoke_renderer_operation(");
const finalRead = runtime.indexOf("let final_state = read_database_snapshot(");
assert(
  baselineRead >= 0
    && rendererInvoke > baselineRead
    && finalRead > rendererInvoke,
  "三库与 Workbench absence baseline 必须在 renderer operation 前读取，final 必须在 renderer 完成后读取",
);
const finalCountsStart = driver.indexOf("fn validate_phase_final_counts(");
const matchesM3Start = driver.indexOf("fn matches_m3_counts(", finalCountsStart);
const finalCounts = driver.slice(finalCountsStart, matchesM3Start);
assert(
  finalCounts.includes(`
        failed,
        1,
        1,
        start_effects,
        start_effects,
        start_effects,
  `),
  "CREATE effect/readback 必须各为 1，不能把 CREATE readback 错绑为 START 计数",
);
for (const binding of [
  'final_state.m3.ordered_turn_refs_sha256 != hash_json(&dto_turn_refs)?',
  'final_state.provider.ordered_turn_refs_sha256 != final_state.m3.ordered_turn_refs_sha256',
  'final_state.provider.ordered_client_message_refs_sha256',
  '!= hash_json(&dto_client_message_refs)?',
  'final_state.provider.ordered_turn_bindings_sha256 != hash_json(&dto_turn_bindings)?',
  'snapshots_match_except_read_transcript(previous_final, &baseline)',
]) {
  assert(driver.includes(binding), `数据库/DTO exact binding 缺少 ${binding}`);
}
assert(
  driver.includes("fs::hard_link(&temporary_path, &output_path)")
    && driver.includes("options.mode(0o600)")
    && driver.includes("write_all(&bytes)")
    && driver.includes("file.sync_all()"),
  "phase receipt 必须 0600 + sync + hard-link 原子发布",
);

const receiptStart = driver.indexOf("struct DriverReceipt {");
const receiptEnd = driver.indexOf("struct OrdinaryConversationPaths", receiptStart);
const receipt = driver.slice(receiptStart, receiptEnd);
for (const forbidden of [
  "client_message_ref: String",
  "user_message: String",
  "assistant_message: String",
  "role_session_ref: String",
  "history_ref: String",
  "command_receipt_ref: String",
]) {
  assert(!receipt.includes(forbidden), `落盘 receipt 泄露 raw field ${forbidden}`);
}

for (const token of [
  "m4r05_ordinary_conversation_driver::requested()",
  "m4r05_ordinary_conversation_driver::start_early_process_watchdog()",
  "m4r05_ordinary_conversation_driver::mark_ordinary_constructor_ready()",
  "m4r05_ordinary_conversation_driver::install_after_runtime_ready(app)",
  "m4r05_ordinary_conversation_driver::reject_early_setup(",
]) {
  assert(host.includes(token), `ordinary host 接线缺少 ${token}`);
}

console.log("m4r05 ordinary conversation driver: ok");
