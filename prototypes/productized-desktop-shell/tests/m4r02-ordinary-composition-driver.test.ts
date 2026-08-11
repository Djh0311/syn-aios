import { assert } from "./helpers/offlineInteractionTestUtils";

const nodeProcess = (globalThis as typeof globalThis & {
  process?: { cwd?: () => string };
}).process;
if (!nodeProcess?.cwd) throw new Error("M4R02 ordinary composition 静态验收需要 Node cwd");
const nodeFsSpecifier: string = "node:fs";
const { readFileSync } = await import(nodeFsSpecifier) as {
  readFileSync: (path: string, encoding: "utf8") => string;
};
const root = nodeProcess.cwd();
const driver = readFileSync(`${root}/src-tauri/src/m4r02_ordinary_composition_driver.rs`, "utf8");
const main = readFileSync(`${root}/src/main.tsx`, "utf8");
const launcher = readFileSync(`${root}/scripts/run-r4-isolated-app-preflight.mjs`, "utf8");
const lib = readFileSync(`${root}/src-tauri/src/lib.rs`, "utf8");
const indexHost = readFileSync(`${root}/src-tauri/src/index_host_app_entrypoints.rs`, "utf8");
const workflowTypes = readFileSync(`${root}/src/lib/types/workflow.ts`, "utf8");
const workflowPanels = readFileSync(`${root}/src/views/projects/ProjectWorkflowExecutionPanels.tsx`, "utf8");
const app = readFileSync(`${root}/src/App.tsx`, "utf8");

for (const token of [
  "SYN_M4R02_ORDINARY_COMPOSITION_DRIVER",
  "ordinary-product-composition-v1",
  "syn-m4r02-ordinary-composition-ui-ready",
  "syn-m4r02-ordinary-composition-invoke",
  "syn-m4r02-ordinary-composition-result",
  "ordinary_registered_tauri_command_ipc",
  "install_after_runtime_ready",
]) {
  assert(driver.includes(token), `M4R02 driver 缺少 ${token}`);
}

const requestIndex = indexHost.indexOf("m4r02_ordinary_composition_driver::requested()");
const ordinaryStateIndex = indexHost.indexOf("AppState::try_new_with_isolated_product_profile(paths)");
const ordinaryStorageIndex = indexHost.indexOf("initialize_for_ordinary_startup(", ordinaryStateIndex);
const dispatcherIndex = indexHost.indexOf("start_m4_source_owner_dispatcher(", ordinaryStorageIndex);
const manageIndex = indexHost.indexOf("app.manage(state)", dispatcherIndex);
const logIndex = indexHost.indexOf("app.handle().plugin(log_plugin)", manageIndex);
const installIndex = indexHost.indexOf("m4r02_ordinary_composition_driver::install_after_runtime_ready(app)", logIndex);
assert(
  lib.includes("mod m4r02_ordinary_composition_driver;")
    && requestIndex >= 0
    && ordinaryStateIndex > requestIndex
    && ordinaryStorageIndex > ordinaryStateIndex
    && dispatcherIndex > ordinaryStorageIndex
    && manageIndex > dispatcherIndex
    && logIndex > manageIndex
    && installIndex > logIndex,
  "M4R02 必须独立 requested，并在普通 AppState/storage/dispatcher/manage/log ready 后安装 driver",
);
const legacyRuntimeExpression = indexHost.slice(
  indexHost.indexOf("let legacy_acceptance_runtime_requested"),
  indexHost.indexOf("let acceptance_state"),
);
assert(
  !legacyRuntimeExpression.includes("m4r02_ordinary_composition_requested"),
  "M4R02 generic-profile driver 不得进入 legacy acceptance runtime boolean",
);

assert(
  !driver.includes("m4_acceptance::")
    && !driver.includes("dispatch_pending_m4_source_owner_outbox(")
    && !driver.includes("ingest_registered_source_publication(")
    && !driver.includes("append_m4_work_item_source_publication(")
    && !driver.includes("WorkbenchSqliteRepository::open"),
  "M4R02 driver 只能编排普通 IPC 和做只读佐证，不得成为 acceptance/repository/adapter 写入口",
);
assert(
  driver.includes("OpenFlags::SQLITE_OPEN_READ_ONLY")
    && driver.includes("command_receipt_rows")
    && driver.includes("owner_event_rows")
    && driver.includes("duplicate_owner_outbox_delta")
    && driver.includes("duplicate_m4_effect_delta"),
  "M4R02 duplicate 证据必须是只读结构核证，并证明 receipt/event/outbox/M4 零重复",
);
assert(
  !driver.includes("read_expected_work_item_revision")
    && !driver.includes("workflow_state_meta AS meta"),
  "M4R02 driver 的命令 authority 不得来自 SQLite revision 直读",
);
for (const field of [
  "source_revision",
  "owner_native_watermark_sha256",
  "sealed_source_owner_watermark_sha256",
  "ingestion_adapter_id",
]) {
  assert(driver.includes(field), `M4R02 source receipt 缺少 ${field}`);
  assert(launcher.includes(field), `M4R02 composite receipt 缺少 ${field}`);
}

const prepareIndex = main.indexOf("await bootstrapProjectWorkflow(invocation.project_root)");
const createIndex = main.indexOf("await createTaskDraft({");
const firstUpdateIndex = main.indexOf("await updateWorkItemState(request)");
const replayUpdateIndex = main.indexOf("await updateWorkItemState(request)", firstUpdateIndex + 1);
const homeReadIndex = main.indexOf("await waitForM4R02Notification(", replayUpdateIndex);
const personalFirstIndex = main.indexOf("await operateSecretaryPersonalObject(personalActionRequest)", homeReadIndex);
const personalReplayIndex = main.indexOf("await operateSecretaryPersonalObject(personalActionRequest)", personalFirstIndex + 1);
const reminderFirstIndex = main.indexOf("await operateSecretaryPersonalObject(reminderRequest)", personalReplayIndex);
const reminderReplayIndex = main.indexOf("await operateSecretaryPersonalObject(reminderRequest)", reminderFirstIndex + 1);
const notificationReadIndex = main.indexOf('action: "NOTIFICATION_READ"', reminderReplayIndex);
const notificationDismissIndex = main.indexOf('action: "NOTIFICATION_DISMISS"', notificationReadIndex);
assert(
  prepareIndex >= 0
    && createIndex > prepareIndex
    && firstUpdateIndex > createIndex
    && replayUpdateIndex > firstUpdateIndex
    && homeReadIndex > replayUpdateIndex
    && personalFirstIndex > homeReadIndex
    && personalReplayIndex > personalFirstIndex
    && reminderFirstIndex > personalReplayIndex
    && reminderReplayIndex > reminderFirstIndex
    && notificationReadIndex > reminderReplayIndex
    && notificationDismissIndex > notificationReadIndex,
  "renderer 必须按普通 source update/replay→PersonalAction replay→Reminder replay→Notification read/dismiss 顺序走 tauri.ts",
);
for (const token of ["PersonalObjectEvidence", "OwnerInvariantEvidence"]) {
  assert(driver.includes(token), `M4R02 personal/owner evidence 缺少 ${token}`);
}
for (const token of [
  "source_owner_tuple_sha256_before",
  "source_owner_tuple_sha256_after",
  "personal_action_replay_receipt_match",
  "personal_action_receipt_rows",
  "personal_action_event_rows",
  "reminder_replay_receipt_match",
  "reminder_receipt_rows",
  "reminder_event_rows",
  "notification_publication_status",
  "personal_action_title_model_brief_absent",
]) {
  assert(driver.includes(token), `M4R02 personal/owner evidence 缺少 ${token}`);
  assert(launcher.includes(token), `M4R02 composite personal/owner evidence 缺少 ${token}`);
}
assert(
  main.includes("request.client_request_ref === nonce")
    && main.includes("first.receipt_id !== replay.receipt_id")
    && main.includes("client_request_ref_sent: true")
    && main.includes("server_sealed_command_identity: true")
    && main.includes("explicit_identity_fields_sent: false")
    && !main.includes("request.command_id")
    && !main.includes("request.idempotency_key")
    && !main.includes("request.expected_revision"),
  "renderer duplicate probe 必须只发送 nonce-bound client ref，同 receipt，并由服务端封装命令身份",
);
const publicRequestType = workflowTypes.slice(
  workflowTypes.indexOf("export type WorkItemStateUpdateRequest"),
  workflowTypes.indexOf("export type WorkflowNodeSessionBindRequest"),
);
assert(
  publicRequestType.includes("client_request_ref?: string")
    && !publicRequestType.includes("command_id")
    && !publicRequestType.includes("idempotency_key")
    && !publicRequestType.includes("expected_revision"),
  "普通 renderer WorkItem 请求类型只能暴露 client_request_ref，不得暴露服务端 command/CAS 字段",
);
assert(
  workflowPanels.includes('globalThis.crypto.randomUUID().replaceAll("-", "")'),
  "普通 PendingAction 必须一次生成 32 位小写 hex client_request_ref",
);
assert(
  app.includes('if (pendingAction.kind !== "advance-work-item-state")')
    && app.includes("setPendingAction(null)"),
  "WorkItem 模糊传输失败必须保留原 PendingAction，使重试复用同一 client_request_ref",
);
assert(
  !main.includes("m4_acceptance")
    && !main.includes("ingest_registered_source_publication")
    && !main.includes("dispatch_pending_m4_source_owner_outbox")
    && !main.includes('!import.meta.env.DEV || !("__TAURI_INTERNALS__" in window)'),
  "renderer bridge 不得调用 acceptance wrapper、adapter 或 dispatcher",
);

for (const token of [
  "--m4r02-ordinary-composition",
  '"initialize",\n  "mutate",\n  "readback"',
  "runM4R02OrdinaryCompositionSuite",
  "m4r02-ordinary-composition-composite-receipt.json",
  "syn.m4.remediation.behavior-receipt.v1",
  "ordinary_composition: true",
  "synthetic_fixture_only: true",
  "acceptance_wrapper_calls: 0",
  "direct_repository_seed_calls: 0",
  "adapter_direct_calls: 0",
]) {
  assert(launcher.includes(token), `M4R02 launcher/composite 缺少 ${token}`);
}

const passContractIndex = launcher.indexOf("function m4r02PassReceiptContractFailure(");
const receiptReaderIndex = launcher.indexOf(
  "async function readM4R02OrdinaryCompositionReceipt(",
  passContractIndex,
);
const phaseRunnerIndex = launcher.indexOf(
  "async function runM4R02OrdinaryCompositionPhase(",
  receiptReaderIndex,
);
assert(
  passContractIndex >= 0
    && receiptReaderIndex > passContractIndex
    && phaseRunnerIndex > receiptReaderIndex
    && launcher.slice(receiptReaderIndex, phaseRunnerIndex).includes(
      "m4r02PassReceiptContractFailure(phase, value)",
    )
    && launcher.slice(receiptReaderIndex, phaseRunnerIndex).includes(
      "receipt_contract_${phase}_${invalidPassField}",
    ),
  "M4R02 每阶段 PASS receipt 必须先过 field-specific exact contract 才可返回",
);

for (const token of [
  "M4R02_ORDINARY_COMPOSITION_PASS_RECEIPT_FIELDS",
  "M4R02_ORDINARY_COMPOSITION_SUBJECT_FIELDS",
  "M4R02_ORDINARY_COMPOSITION_PERSONAL_OBJECT_FIELDS",
  "M4R02_ORDINARY_COMPOSITION_OWNER_INVARIANT_FIELDS",
  "m4r02HasExactObjectFields(",
  '["error_family", value.error_family === null]',
  '["workflow_state_sha256", m4r02IsLowerHexSha256(value.workflow_state_sha256)]',
  '["first_initialize", value.first_initialize === true]',
  '["snapshot_initialized", value.snapshot_initialized === true]',
  '["restart_required", value.restart_required === true]',
  '["write_commands_invoked", value.write_commands_invoked === 10]',
  '["product_read_visible", value.product_read_visible === true]',
  'subject.ingestion_adapter_id\n        === M4R02_ORDINARY_COMPOSITION_SOURCE_ADAPTER_ID',
  '["subject_checkpoint_status", subject.checkpoint_status === "CAUGHT_UP"]',
  '["subject_command_receipt_rows", subject.command_receipt_rows === 1]',
  '["subject_owner_event_rows", subject.owner_event_rows === 1]',
  '["write_commands_invoked", value.write_commands_invoked === 0]',
  '["subject_outbox_delta", value.subject_outbox_delta === 0]',
  '["subject_m4_effect_delta", value.subject_m4_effect_delta === 0]',
  '["restart_continuity", value.restart_continuity === true]',
]) {
  assert(launcher.includes(token), `M4R02 exact PASS receipt contract 缺少 ${token}`);
}

for (const field of [
  "work_item_id_sha256",
  "command_id_sha256",
  "idempotency_key_sha256",
  "update_receipt_id_sha256",
  "owner_native_event_id_sha256",
  "owner_publication_id_sha256",
  "owner_terminal_receipt_sha256",
  "source_event_id_sha256",
  "owner_native_watermark_sha256",
  "sealed_source_owner_watermark_sha256",
  "notification_id_sha256",
]) {
  assert(
    launcher.includes(`!m4r02IsLowerHexSha256(subject[field])`)
      && launcher.includes(`"${field}"`),
    `M4R02 subject exact contract 缺少 hash 字段 ${field}`,
  );
}
for (const field of [
  "personal_action_id_sha256",
  "personal_action_receipt_sha256",
  "reminder_id_sha256",
  "reminder_receipt_sha256",
  "notification_read_receipt_sha256",
  "notification_dismiss_receipt_sha256",
  "notification_read_aggregate_id_sha256",
  "notification_read_scope_ref_sha256",
  "notification_dismiss_aggregate_id_sha256",
  "notification_dismiss_scope_ref_sha256",
]) {
  assert(
    launcher.includes(`!m4r02IsLowerHexSha256(personalObjects[field])`)
      && launcher.includes(`"${field}"`),
    `M4R02 personal object exact contract 缺少 hash 字段 ${field}`,
  );
}
for (const token of [
  'personalObjects.personal_action_status === "OPEN"',
  "m4r02IsCanonicalRevision(personalObjects.personal_action_revision)",
  "personalObjects.personal_action_replay_receipt_match === true",
  "personalObjects.personal_action_receipt_rows === 1",
  "personalObjects.personal_action_event_rows === 1",
  'personalObjects.reminder_status === "SCHEDULED"',
  "m4r02IsCanonicalRevision(personalObjects.reminder_revision)",
  "personalObjects.reminder_replay_receipt_match === true",
  "personalObjects.reminder_receipt_rows === 1",
  "personalObjects.reminder_event_rows === 1",
  'personalObjects.notification_publication_status === "DELIVERED"',
  "personalObjects.personal_action_title_model_brief_absent === true",
  'personalObjects.notification_read_command_kind === "NOTIFICATION_READ"',
  'personalObjects.notification_read_event_kind === "NOTIFICATION_READ"',
  'personalObjects.notification_read_aggregate_kind === "NOTIFICATION"',
  'personalObjects.notification_read_expected_revision === "2"',
  'personalObjects.notification_read_receipt_revision === "3"',
  'personalObjects.notification_read_event_revision === "3"',
  "personalObjects.notification_read_receipt_rows === 1",
  "personalObjects.notification_read_event_rows === 1",
  'personalObjects.notification_dismiss_command_kind === "NOTIFICATION_DISMISS"',
  'personalObjects.notification_dismiss_event_kind === "NOTIFICATION_DISMISSED"',
  'personalObjects.notification_dismiss_aggregate_kind === "NOTIFICATION"',
  'personalObjects.notification_dismiss_expected_revision === "3"',
  'personalObjects.notification_dismiss_receipt_revision === "4"',
  'personalObjects.notification_dismiss_event_revision === "4"',
  "personalObjects.notification_dismiss_receipt_rows === 1",
  "personalObjects.notification_dismiss_event_rows === 1",
  "personalObjects.notification_scope_binding_match === true",
  "personalObjects.notification_aggregate_binding_match === true",
  "personalObjects.notification_revision_chain_contiguous === true",
  "personalObjects.notification_final_revision_match === true",
  "personalObjects.notification_read_aggregate_id_sha256\n        === personalObjects.notification_dismiss_aggregate_id_sha256",
  "personalObjects.notification_read_scope_ref_sha256\n        === personalObjects.notification_dismiss_scope_ref_sha256",
  "personalObjects.notification_read_receipt_sha256\n        !== personalObjects.notification_dismiss_receipt_sha256",
  'personalObjects.notification_revision === "4"',
  "m4r02IsLowerHexSha256(ownerInvariant.source_owner_tuple_sha256_before)",
  "m4r02IsLowerHexSha256(ownerInvariant.source_owner_tuple_sha256_after)",
  "m4r02IsCanonicalRevision(ownerInvariant.source_revision_before)",
  "m4r02IsCanonicalRevision(ownerInvariant.source_revision_after)",
  "ownerInvariant.unchanged === true",
  "ownerInvariant.source_owner_tuple_sha256_before\n        === ownerInvariant.source_owner_tuple_sha256_after",
  "ownerInvariant.source_revision_before === ownerInvariant.source_revision_after",
]) {
  assert(launcher.includes(token), `M4R02 personal/owner exact contract 缺少 ${token}`);
}
assert(
  launcher.includes(
    "value.personal_objects.notification_read_aggregate_id_sha256\n          === value.subject.notification_id_sha256",
  )
    && launcher.includes(
      "value.personal_objects.notification_dismiss_aggregate_id_sha256\n            === value.subject.notification_id_sha256",
    )
    && launcher.includes(
      "value.owner_invariant.source_revision_before === value.subject.source_revision",
    )
    && launcher.includes(
      "value.owner_invariant.source_revision_after === value.subject.source_revision",
    ),
  "M4R02 mutate receipt 必须把 Notification aggregate 与 source owner revision 绑定到 subject",
);

for (const token of [
  'error?.code === "ENOENT" && Date.now() < visibilityDeadline',
  "receipt_invalid_io_${ioCode}",
  "m4r02OrdinaryCompositionFailedPhase",
  'error.phase = "readback"',
  'return "launch_services_exit"',
]) {
  assert(launcher.includes(token), `M4R02 bounded/failure 语义回归：缺少 ${token}`);
}
assert(
  launcher.includes('["same_subject", sameSubject]')
    && launcher.includes('["same_personal_objects", samePersonalObjects]')
    && launcher.includes('["same_owner_invariant", sameOwnerInvariant]')
    && launcher.includes(
      "mutate.workflow_state_sha256 === readback.workflow_state_sha256",
    ),
  "M4R02 readback 必须跨真实 App launch 核对同 workflow/subject/personal/owner",
);

console.log("m4r02-ordinary-composition-driver: ok");
