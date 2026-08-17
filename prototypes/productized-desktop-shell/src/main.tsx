import React from "react";
import ReactDOM from "react-dom/client";
import { emit, listen } from "@tauri-apps/api/event";
import { App } from "./App";
import {
  bootstrapProjectWorkflow,
  createProjectConsultationProposal,
  createTaskDraft,
  initializeWorkflowState,
  loadSecretaryConversation,
  loadSecretaryDailyReport,
  loadSecretaryHomeContext,
  loadSecretaryLegacyReadCompatibilityReport,
  loadWorkflowStateSnapshot,
  operateSecretaryCoordination,
  operateSecretaryPersonalObject,
  resolveSecretarySourceRoute,
  sendSecretaryMessage,
  updateWorkItemState,
} from "./lib/tauri";
import { mintSecretaryCoordinationIdempotencyKey } from "./lib/secretaryReadModel";
import { setTauriWindowTitle } from "./lib/tauriWindow";
import {
  loadM5IsolatedAcceptanceStatus,
  writeM5IsolatedUiReceipt,
} from "./lib/m5ProjectSupervisor";
import type { WorkItemStateUpdateRequest } from "./lib/types/workflow";
import type {
  M4SecretaryConversation,
  M4SecretaryConversationTurn,
  M4SecretaryMessageSendOutcome,
} from "./lib/types/m4SecretaryConversation";
import "./styles.css";
import "./manualRelay.css";
import "./components/sourceStylePlaceholder.css";
import "./views/memory/memoryCenter.css";
import "./views/projects/projectWorkflowSidePanel.css";
import "./views/projects/projectReferencePanels.css";

type BootErrorBoundaryProps = {
  children: React.ReactNode;
};

type BootErrorBoundaryState = {
  error: Error | null;
};

const BOOT_VISIBLE_PROBE_ID = "tauri-boot-visible-probe";
const bootProbeEnabled = import.meta.env.DEV;

// M2 DAT-008 accepts exactly one debug/R4 fixture command through the same
// Tauri IPC wrapper used by the product UI.  The Rust host emits this request
// only after a validated isolated profile reaches runtime readiness; this
// frontend bridge is otherwise inert and exposes no new command.
const M2_R4_IPC_READY_EVENT = "syn-m2-r4-reference-slice-ui-ready";
const M2_R4_IPC_INVOKE_EVENT = "syn-m2-r4-reference-slice-invoke";
const M2_R4_IPC_RESULT_EVENT = "syn-m2-r4-reference-slice-result";
const M2_R4_IPC_SCHEMA_VERSION = "syn_m2_r4_tauri_ipc.v1";

type M2R4IpcInvocation = {
  schema_version: typeof M2_R4_IPC_SCHEMA_VERSION;
  operation: "update_work_item_state";
  attempt: string;
  nonce: string;
  request: WorkItemStateUpdateRequest;
};

function isM2R4IpcInvocation(value: unknown): value is M2R4IpcInvocation {
  if (!value || typeof value !== "object") return false;
  const candidate = value as Partial<M2R4IpcInvocation>;
  return (
    candidate.schema_version === M2_R4_IPC_SCHEMA_VERSION &&
    candidate.operation === "update_work_item_state" &&
    typeof candidate.attempt === "string" &&
    /^[a-z0-9-]{1,48}$/.test(candidate.attempt) &&
    typeof candidate.nonce === "string" &&
    /^[a-f0-9]{32}$/.test(candidate.nonce) &&
    Boolean(candidate.request) &&
    typeof candidate.request?.project_root === "string" &&
    typeof candidate.request?.work_item_id === "string" &&
    typeof candidate.request?.next_state === "string"
  );
}

async function installM2R4TauriIpcBridge() {
  if (!("__TAURI_INTERNALS__" in window)) return;
  try {
    await listen<M2R4IpcInvocation>(M2_R4_IPC_INVOKE_EVENT, async ({ payload }) => {
      if (!isM2R4IpcInvocation(payload)) return;
      try {
        // Two separate frontend invokes intentionally exercise the registered
        // command's replay contract.  They are not a Rust helper call.
        const first = await updateWorkItemState(payload.request);
        const replay = await updateWorkItemState(payload.request);
        if (!first.receipt_id || first.receipt_id !== replay.receipt_id) {
          throw new Error("m2_r4_reference_slice_ipc_replay_receipt_mismatch");
        }
        await emit(M2_R4_IPC_RESULT_EVENT, {
          schema_version: M2_R4_IPC_SCHEMA_VERSION,
          operation: payload.operation,
          attempt: payload.attempt,
          nonce: payload.nonce,
          receipt_id: first.receipt_id,
          replay_receipt_id: replay.receipt_id,
          outcome: "PASS",
        });
      } catch (error) {
        // Do not put product state or raw errors into the cross-process
        // acceptance event.  Rust persists only a value-free failure family.
        const message = error instanceof Error ? error.message : String(error);
        await emit(M2_R4_IPC_RESULT_EVENT, {
          schema_version: M2_R4_IPC_SCHEMA_VERSION,
          operation: payload.operation,
          attempt: payload.attempt,
          nonce: payload.nonce,
          outcome: "REJECTED",
          error_family: message.includes("acceptance_injected_failure:projection-fail")
            ? "projection_fail"
            : "command_rejected",
        });
      }
    });
    await emit(M2_R4_IPC_READY_EVENT, {
      schema_version: M2_R4_IPC_SCHEMA_VERSION,
      surface: "registered_tauri_command_ipc",
    });
  } catch {
    // The Rust driver has its own bounded readiness timeout and fails closed.
  }
}

// M4R02 generic-profile proof. The host may only orchestrate this bridge in a
// debug isolated profile. Every product mutation still crosses the same
// tauri.ts wrappers and registered commands used by the ordinary renderer.
const M4R02_IPC_READY_EVENT = "syn-m4r02-ordinary-composition-ui-ready";
const M4R02_IPC_INVOKE_EVENT = "syn-m4r02-ordinary-composition-invoke";
const M4R02_IPC_RESULT_EVENT = "syn-m4r02-ordinary-composition-result";
const M4R02_IPC_SCHEMA_VERSION = "syn_m4r02_ordinary_composition_ipc.v1";
const M4R02_TASK_TITLE = "SYN M4R02 ordinary product composition";
const M4R02_TASK_OBJECTIVE = "isolated generic-profile proof through ordinary product commands";
const M4R02_TASK_ASSIGNED_ROLE = "codex-dev";
const M4R02_PERSONAL_ACTION_TITLE = "SYN M4R02 ordinary personal follow-up";
const M4R02_REMINDER_SCHEDULED_FOR_UTC = "2099-01-01T00:00:00Z";
const M4R02_HOME_READ_TIMEOUT_MS = 12_000;

type M4R02Operation =
  | "initialize"
  | "prepare_mutation"
  | "apply_mutation"
  | "apply_personal_objects"
  | "readback";

type M4R02TaskInput = {
  title: typeof M4R02_TASK_TITLE;
  objective: typeof M4R02_TASK_OBJECTIVE;
  assigned_role: typeof M4R02_TASK_ASSIGNED_ROLE;
};

type M4R02OrdinaryCompositionInvocation = {
  schema_version: typeof M4R02_IPC_SCHEMA_VERSION;
  phase: "initialize" | "mutate" | "readback";
  operation: M4R02Operation;
  nonce: string;
  project_root: string;
  task: M4R02TaskInput | null;
  request: WorkItemStateUpdateRequest | null;
};

type M4R02PreparedMutation = {
  nonce: string;
  projectRoot: string;
  workItemId: string;
};

let m4r02PreparedMutation: M4R02PreparedMutation | null = null;
let m4r02OperationQueue: Promise<void> = Promise.resolve();

function hasExactKeys(value: object, expected: readonly string[]) {
  const actual = Object.keys(value).sort();
  const sortedExpected = [...expected].sort();
  return actual.length === sortedExpected.length
    && actual.every((key, index) => key === sortedExpected[index]);
}

function isM4R02Task(value: unknown): value is M4R02TaskInput {
  if (!value || typeof value !== "object") return false;
  const task = value as Partial<M4R02TaskInput>;
  return hasExactKeys(value, ["assigned_role", "objective", "title"])
    && task.title === M4R02_TASK_TITLE
    && task.objective === M4R02_TASK_OBJECTIVE
    && task.assigned_role === M4R02_TASK_ASSIGNED_ROLE;
}

function isM4R02FixedUpdateRequest(
  value: unknown,
  projectRoot: string,
  nonce: string,
): value is WorkItemStateUpdateRequest {
  if (!value || typeof value !== "object") return false;
  const request = value as Partial<WorkItemStateUpdateRequest>;
  return hasExactKeys(value, [
    "client_request_ref",
    "next_state",
    "project_root",
    "work_item_id",
  ])
    && request.project_root === projectRoot
    && typeof request.work_item_id === "string"
    && request.work_item_id.length > 0
    && request.work_item_id.length <= 512
    && !/[\\/\r\n]/.test(request.work_item_id)
    && request.next_state === "ready_to_dispatch"
    && request.client_request_ref === nonce;
}

function isM4R02Invocation(value: unknown): value is M4R02OrdinaryCompositionInvocation {
  if (!value || typeof value !== "object") return false;
  const candidate = value as Partial<M4R02OrdinaryCompositionInvocation>;
  if (!hasExactKeys(value, [
    "nonce",
    "operation",
    "phase",
    "project_root",
    "request",
    "schema_version",
    "task",
  ])) return false;
  if (
    candidate.schema_version !== M4R02_IPC_SCHEMA_VERSION
    || typeof candidate.nonce !== "string"
    || !/^[a-f0-9]{32}$/.test(candidate.nonce)
    || typeof candidate.project_root !== "string"
    || candidate.project_root.length === 0
    || candidate.project_root.length > 1024
    || /[\r\n]/.test(candidate.project_root)
  ) return false;
  if (candidate.phase === "initialize" && candidate.operation === "initialize") {
    return candidate.task === null && candidate.request === null;
  }
  if (candidate.phase === "mutate" && candidate.operation === "prepare_mutation") {
    return isM4R02Task(candidate.task) && candidate.request === null;
  }
  if (candidate.phase === "mutate" && candidate.operation === "apply_mutation") {
    return candidate.task === null
      && isM4R02FixedUpdateRequest(candidate.request, candidate.project_root, candidate.nonce);
  }
  if (candidate.phase === "mutate" && candidate.operation === "apply_personal_objects") {
    return candidate.task === null && candidate.request === null;
  }
  return candidate.phase === "readback"
    && candidate.operation === "readback"
    && isM4R02Task(candidate.task)
    && candidate.request === null;
}

function findM4R02Task(
  snapshot: Awaited<ReturnType<typeof loadWorkflowStateSnapshot>>,
  projectRoot: string,
) {
  const matches = snapshot.project_workflows
    .filter((workflow) => workflow.project_root === projectRoot)
    .flatMap((workflow) => workflow.task_drafts)
    .filter((task) => task.title === M4R02_TASK_TITLE);
  if (matches.length !== 1) throw new Error("m4r02_task_cardinality_invalid");
  return matches[0];
}

function delayM4R02(milliseconds: number) {
  return new Promise<void>((resolve) => window.setTimeout(resolve, milliseconds));
}

async function waitForM4R02Notification(
  workItemId: string,
  expectedStatus: "DELIVERED" | "READ" | "DISMISSED",
) {
  const deadline = Date.now() + M4R02_HOME_READ_TIMEOUT_MS;
  while (Date.now() < deadline) {
    const home = await loadSecretaryHomeContext();
    if (home.status === "READY") {
      const matches = home.application_outcome.local_objects.notifications.filter(
        (notification) => notification.source_ref.canonical_source_object_id === workItemId,
      );
      if (matches.length > 1) throw new Error("m4r02_notification_cardinality_invalid");
      if (matches[0]?.status === expectedStatus) return { home, notification: matches[0] };
    }
    await delayM4R02(250);
  }
  throw new Error("m4r02_home_context_timeout");
}

async function loadM4R02ReadyHome() {
  const home = await loadSecretaryHomeContext();
  if (home.status !== "READY") throw new Error("m4r02_home_context_not_ready");
  return home;
}

function findM4R02PersonalAction(
  home: Awaited<ReturnType<typeof loadM4R02ReadyHome>>,
) {
  const matches = home.application_outcome.local_objects.personal_actions.filter(
    (item) => item.title === M4R02_PERSONAL_ACTION_TITLE,
  );
  if (matches.length !== 1) throw new Error("m4r02_personal_action_cardinality_invalid");
  return matches[0];
}

function findM4R02Reminder(
  home: Awaited<ReturnType<typeof loadM4R02ReadyHome>>,
  personalActionId: string,
) {
  const matches = home.application_outcome.local_objects.reminders.filter(
    (item) => item.owner_ref === personalActionId,
  );
  if (matches.length !== 1) throw new Error("m4r02_reminder_cardinality_invalid");
  return matches[0];
}

function m4r02ErrorFamily(error: unknown) {
  const message = error instanceof Error ? error.message : String(error);
  if (message.includes("home_context_timeout")) return "home_read_timeout";
  if (message.includes("m4_secretary_home_")) return "home_read_contract";
  if (message.includes("task_cardinality")) return "task_readback";
  if (message.includes("notification_cardinality")) return "notification_readback";
  if (message.includes("replay_receipt")) return "duplicate_replay";
  if (message.includes("prepared_mutation")) return "prepare_binding";
  return "command_rejected";
}

async function emitM4R02Result(
  invocation: M4R02OrdinaryCompositionInvocation,
  result: Record<string, unknown>,
) {
  await emit(M4R02_IPC_RESULT_EVENT, {
    schema_version: M4R02_IPC_SCHEMA_VERSION,
    phase: invocation.phase,
    operation: invocation.operation,
    nonce: invocation.nonce,
    ...result,
  });
}

async function runM4R02OrdinaryCompositionOperation(
  invocation: M4R02OrdinaryCompositionInvocation,
) {
  switch (invocation.operation) {
    case "initialize": {
      const initialized = await initializeWorkflowState();
      if (!initialized.first_initialize || !initialized.snapshot.initialized) {
        throw new Error("m4r02_initialize_result_invalid");
      }
      await emitM4R02Result(invocation, {
        outcome: "PASS",
        initialize_audit_event_id: initialized.audit_event_id,
        first_initialize: initialized.first_initialize,
        workflow_initialized: initialized.snapshot.initialized,
        restart_required: initialized.message.includes("ordinary_product_storage_restart_required"),
        write_commands_invoked: 1,
        client_request_ref_sent: false,
        server_sealed_command_identity: true,
        explicit_identity_fields_sent: false,
      });
      return;
    }
    case "prepare_mutation": {
      const bootstrapped = await bootstrapProjectWorkflow(invocation.project_root);
      const created = await createTaskDraft({
        project_root: invocation.project_root,
        title: M4R02_TASK_TITLE,
        objective: M4R02_TASK_OBJECTIVE,
        assigned_role: M4R02_TASK_ASSIGNED_ROLE,
      });
      const task = findM4R02Task(created.snapshot, invocation.project_root);
      if (task.state !== "draft") throw new Error("m4r02_prepared_mutation_state_invalid");
      m4r02PreparedMutation = {
        nonce: invocation.nonce,
        projectRoot: invocation.project_root,
        workItemId: task.work_item_id,
      };
      await emitM4R02Result(invocation, {
        outcome: "PASS",
        bootstrap_audit_event_id: bootstrapped.audit_event_id,
        task_create_audit_event_id: created.audit_event_id,
        work_item_id: task.work_item_id,
        work_item_state: task.state,
        write_commands_invoked: 2,
        client_request_ref_sent: false,
        server_sealed_command_identity: true,
        explicit_identity_fields_sent: false,
      });
      return;
    }
    case "apply_mutation": {
      const request = invocation.request;
      if (!request) throw new Error("m4r02_prepared_mutation_request_missing");
      const prepared = m4r02PreparedMutation;
      if (
        !prepared
        || prepared.nonce !== invocation.nonce
        || prepared.projectRoot !== invocation.project_root
        || prepared.workItemId !== request.work_item_id
      ) throw new Error("m4r02_prepared_mutation_binding_invalid");
      // Both calls cross the ordinary tauri.ts wrapper and registered command.
      // The fixed nonce-bound client reference makes the second call an exact
      // replay while the backend derives command identity and revision.
      const first = await updateWorkItemState(request);
      const replay = await updateWorkItemState(request);
      if (!first.receipt_id || first.receipt_id !== replay.receipt_id) {
        throw new Error("m4r02_replay_receipt_mismatch");
      }
      const task = findM4R02Task(first.snapshot, invocation.project_root);
      if (task.work_item_id !== prepared.workItemId || task.state !== "ready_to_dispatch") {
        throw new Error("m4r02_updated_task_readback_invalid");
      }
      const { notification } = await waitForM4R02Notification(
        prepared.workItemId,
        "DELIVERED",
      );
      await emitM4R02Result(invocation, {
        outcome: "PASS",
        work_item_id: prepared.workItemId,
        work_item_state: task.state,
        update_receipt_id: first.receipt_id,
        replay_receipt_id: replay.receipt_id,
        notification_id: notification.notification_id,
        notification_status: notification.status,
        write_commands_invoked: 2,
        client_request_ref_sent: true,
        server_sealed_command_identity: true,
        explicit_identity_fields_sent: false,
      });
      return;
    }
    case "apply_personal_objects": {
      const prepared = m4r02PreparedMutation;
      if (
        !prepared
        || prepared.nonce !== invocation.nonce
        || prepared.projectRoot !== invocation.project_root
      ) throw new Error("m4r02_prepared_mutation_binding_invalid");

      const personalActionRequest = {
        action: "PERSONAL_ACTION_CREATE" as const,
        title: M4R02_PERSONAL_ACTION_TITLE,
        idempotency_key: await mintSecretaryCoordinationIdempotencyKey(),
      };
      const personalActionFirst = await operateSecretaryPersonalObject(personalActionRequest);
      const personalActionReplay = await operateSecretaryPersonalObject(personalActionRequest);
      if (
        personalActionFirst.command_receipt_ref !== personalActionReplay.command_receipt_ref
        || personalActionFirst.item_ref !== personalActionReplay.item_ref
        || personalActionFirst.replayed
        || !personalActionReplay.replayed
      ) throw new Error("m4r02_personal_action_replay_receipt_mismatch");
      let home = await loadM4R02ReadyHome();
      const personalAction = findM4R02PersonalAction(home);
      if (
        personalAction.personal_action_id !== personalActionFirst.item_ref
        || personalAction.status !== "OPEN"
      ) throw new Error("m4r02_personal_action_readback_invalid");

      const reminderRequest = {
        action: "REMINDER_CREATE" as const,
        owner_ref: personalAction.personal_action_id,
        scheduled_for_utc: M4R02_REMINDER_SCHEDULED_FOR_UTC,
        iana_timezone: "Asia/Shanghai",
        idempotency_key: await mintSecretaryCoordinationIdempotencyKey(),
      };
      const reminderFirst = await operateSecretaryPersonalObject(reminderRequest);
      const reminderReplay = await operateSecretaryPersonalObject(reminderRequest);
      if (
        reminderFirst.command_receipt_ref !== reminderReplay.command_receipt_ref
        || reminderFirst.item_ref !== reminderReplay.item_ref
        || reminderFirst.replayed
        || !reminderReplay.replayed
      ) throw new Error("m4r02_reminder_replay_receipt_mismatch");
      home = await loadM4R02ReadyHome();
      const reminder = findM4R02Reminder(home, personalAction.personal_action_id);
      if (reminder.reminder_id !== reminderFirst.item_ref || reminder.status !== "SCHEDULED") {
        throw new Error("m4r02_reminder_readback_invalid");
      }

      const delivered = await waitForM4R02Notification(prepared.workItemId, "DELIVERED");
      const notificationRead = await operateSecretaryPersonalObject({
        action: "NOTIFICATION_READ",
        item_ref: delivered.notification.notification_id,
        expected_revision: delivered.notification.revision,
        idempotency_key: await mintSecretaryCoordinationIdempotencyKey(),
      });
      const read = await waitForM4R02Notification(prepared.workItemId, "READ");
      if (notificationRead.item_ref !== read.notification.notification_id) {
        throw new Error("m4r02_notification_read_receipt_mismatch");
      }
      const notificationDismiss = await operateSecretaryPersonalObject({
        action: "NOTIFICATION_DISMISS",
        item_ref: read.notification.notification_id,
        expected_revision: read.notification.revision,
        idempotency_key: await mintSecretaryCoordinationIdempotencyKey(),
      });
      const dismissed = await waitForM4R02Notification(prepared.workItemId, "DISMISSED");
      if (notificationDismiss.item_ref !== dismissed.notification.notification_id) {
        throw new Error("m4r02_notification_dismiss_receipt_mismatch");
      }
      if (JSON.stringify(dismissed.home.application_outcome.deterministic_brief)
        .includes(M4R02_PERSONAL_ACTION_TITLE)) {
        throw new Error("m4r02_personal_action_title_leaked_to_model_brief");
      }

      await emitM4R02Result(invocation, {
        outcome: "PASS",
        work_item_id: prepared.workItemId,
        work_item_state: "ready_to_dispatch",
        notification_id: dismissed.notification.notification_id,
        notification_status: dismissed.notification.status,
        notification_revision: dismissed.notification.revision,
        notification_read_receipt_id: notificationRead.command_receipt_ref,
        notification_dismiss_receipt_id: notificationDismiss.command_receipt_ref,
        personal_action_id: personalAction.personal_action_id,
        personal_action_status: personalAction.status,
        personal_action_revision: personalAction.revision,
        personal_action_receipt_id: personalActionFirst.command_receipt_ref,
        personal_action_replay_receipt_id: personalActionReplay.command_receipt_ref,
        reminder_id: reminder.reminder_id,
        reminder_status: reminder.status,
        reminder_revision: reminder.revision,
        reminder_receipt_id: reminderFirst.command_receipt_ref,
        reminder_replay_receipt_id: reminderReplay.command_receipt_ref,
        personal_action_title_model_brief_absent: true,
        write_commands_invoked: 6,
        client_request_ref_sent: false,
        server_sealed_command_identity: true,
        explicit_identity_fields_sent: false,
      });
      return;
    }
    case "readback": {
      const snapshot = await loadWorkflowStateSnapshot();
      const task = findM4R02Task(snapshot, invocation.project_root);
      if (task.state !== "ready_to_dispatch") throw new Error("m4r02_restart_task_state_invalid");
      const { home, notification } = await waitForM4R02Notification(
        task.work_item_id,
        "DISMISSED",
      );
      const personalAction = findM4R02PersonalAction(home);
      const reminder = findM4R02Reminder(home, personalAction.personal_action_id);
      if (JSON.stringify(home.application_outcome.deterministic_brief)
        .includes(M4R02_PERSONAL_ACTION_TITLE)) {
        throw new Error("m4r02_personal_action_title_leaked_to_model_brief");
      }
      await emitM4R02Result(invocation, {
        outcome: "PASS",
        work_item_id: task.work_item_id,
        work_item_state: task.state,
        notification_id: notification.notification_id,
        notification_status: notification.status,
        notification_revision: notification.revision,
        personal_action_id: personalAction.personal_action_id,
        personal_action_status: personalAction.status,
        personal_action_revision: personalAction.revision,
        reminder_id: reminder.reminder_id,
        reminder_status: reminder.status,
        reminder_revision: reminder.revision,
        personal_action_title_model_brief_absent: true,
        write_commands_invoked: 0,
        client_request_ref_sent: false,
        server_sealed_command_identity: true,
        explicit_identity_fields_sent: false,
      });
    }
  }
}

async function installM4R02OrdinaryCompositionTauriIpcBridge() {
  // `tauri build --debug` still performs a production Vite build, so
  // `import.meta.env.DEV` is false in the isolated product binary.  The
  // privileged enablement gate lives in the Rust host (debug binary + exact
  // profile/driver/phase/nonce); this renderer listener is inert unless that
  // host emits the bound invocation event.
  if (!("__TAURI_INTERNALS__" in window)) return;
  try {
    await listen<M4R02OrdinaryCompositionInvocation>(M4R02_IPC_INVOKE_EVENT, ({ payload }) => {
      if (!isM4R02Invocation(payload)) return;
      // Rust may emit the next operation while the Promise returned by the
      // previous result event is still settling.  Queue every validated
      // invocation so that this hand-off never drops a product command.
      m4r02OperationQueue = m4r02OperationQueue.then(async () => {
        try {
          await runM4R02OrdinaryCompositionOperation(payload);
        } catch (error) {
          await emitM4R02Result(payload, {
            outcome: "REJECTED",
            write_commands_invoked: 0,
            client_request_ref_sent: payload.operation === "apply_mutation",
            server_sealed_command_identity: true,
            explicit_identity_fields_sent: false,
            error_family: m4r02ErrorFamily(error),
          });
        }
      });
    });
    await emit(M4R02_IPC_READY_EVENT, {
      schema_version: M4R02_IPC_SCHEMA_VERSION,
      surface: "ordinary_registered_tauri_command_ipc",
      phases: ["initialize", "mutate", "readback"],
    });
  } catch {
    // The host has bounded readiness/result timeouts and fails closed.
  }
}

// M4R03 generic-profile server-clock proof. The bridge has no clock or fire
// operation: it can only issue the same user-facing snooze/create commands as
// the Home surface and then reread ordinary server state. StartupRecovery and
// TimerTick remain owned entirely by the Rust scheduler.
const M4R03_IPC_READY_EVENT = "syn-m4r03-ordinary-clock-ui-ready";
const M4R03_IPC_INVOKE_EVENT = "syn-m4r03-ordinary-clock-invoke";
const M4R03_IPC_RESULT_EVENT = "syn-m4r03-ordinary-clock-result";
const M4R03_IPC_SCHEMA_VERSION = "syn_m4r03_ordinary_clock_ipc.v1";
const M4R07_POST_TICK_RENDERER_IPC_SCHEMA_VERSION =
  "syn_m4r07_post_tick_renderer_ipc.v1";
const M4R07_POST_TICK_RENDERER_DIAGNOSTIC_IPC_SCHEMA_VERSION =
  "syn_m4r07_post_tick_renderer_diagnostic_ipc.v1";
// Leave the launcher enough room to observe the arm receipt and SIGKILL the
// exact bundled process while both user-scheduled objects are still pre-due.
const M4R03_STARTUP_DUE_DELAY_MS = 45_000;
// The two ordinary snooze commands are serialized. Keep their shared marker
// beyond the complete operation window so a production tick cannot split them.
const M4R03_TIMER_DUE_DELAY_MS = 30_000;
const M4R03_HOME_READ_TIMEOUT_MS = 15_000;
const M4R03_RECOVERY_VISIBLE_MARKERS = [
  "现在要看住什么",
  "已继续同一情境",
  "持续关注",
  "OPEN",
  "FIRED",
] as const;

async function m4r03Sha256(value: string) {
  const digest = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(value));
  return Array.from(new Uint8Array(digest), (byte) => byte.toString(16).padStart(2, "0")).join("");
}

type M4R03Phase = "arm" | "recovery_timer" | "repeat";
type M4R03Operation =
  | "arm_startup_recovery"
  | "observe_startup_recovery"
  | "arm_timer_tick"
  | "observe_timer_tick"
  | "observe_repeat";

type M4R03OrdinaryClockInvocation = {
  schema_version:
    | typeof M4R03_IPC_SCHEMA_VERSION
    | typeof M4R07_POST_TICK_RENDERER_IPC_SCHEMA_VERSION
    | typeof M4R07_POST_TICK_RENDERER_DIAGNOSTIC_IPC_SCHEMA_VERSION;
  phase: M4R03Phase;
  operation: M4R03Operation;
  nonce: string;
  startup_due_marker_utc: string | null;
  timer_due_marker_utc: string | null;
};

const M4R07_POST_TICK_RENDERER_DIAGNOSTIC_CODES = [
  "m4r03_state_read_timeout",
  "m4r03_home_context_not_ready",
  "m4r03_open_loop_cardinality_invalid",
  "m4r03_reminder_cardinality_invalid",
  "m4r03_prepared_binding_invalid",
  "m4r03_home_visible_prior_state_invalid",
  "m4r03_home_refresh_cardinality_invalid",
  "m4r03_home_visible_terminal_state",
  "m4r07_post_tick_refresh_transition_not_observed",
  "m4r07_post_tick_fresh_ready_not_observed",
  "m4r07_post_tick_old_ready_reused",
  "m4r07_post_tick_dom_recovery_markers_not_observed",
  "m4r07_post_tick_screenshot_markers_not_visible",
  "m4r07_post_tick_backend_binding_invalid",
] as const;

type M4R07PostTickRendererDiagnosticCode =
  | (typeof M4R07_POST_TICK_RENDERER_DIAGNOSTIC_CODES)[number]
  | "m4r07_post_tick_renderer_unclassified";

type M4R07PostTickRendererDiagnosticCheckpoint = {
  prior_ready: boolean;
  refresh_clicked: boolean;
  transition_seen: boolean;
  new_ready_seen: boolean;
  dom5_seen: boolean;
  screenshot_pair_seen: boolean;
  old_ready_reused_after_transition: boolean;
};

const M4R07_POST_TICK_RENDERER_DIAGNOSTIC_CHECKPOINT_ORDER = [
  "prior_ready",
  "refresh_clicked",
  "transition_seen",
  "new_ready_seen",
  "dom5_seen",
  "screenshot_pair_seen",
] as const satisfies readonly (keyof M4R07PostTickRendererDiagnosticCheckpoint)[];

type M4R07PostTickRendererDiagnosticCheckpointKey =
  (typeof M4R07_POST_TICK_RENDERER_DIAGNOSTIC_CHECKPOINT_ORDER)[number];

function newM4R07PostTickRendererDiagnosticCheckpoint():
M4R07PostTickRendererDiagnosticCheckpoint {
  return {
    prior_ready: false,
    refresh_clicked: false,
    transition_seen: false,
    new_ready_seen: false,
    dom5_seen: false,
    screenshot_pair_seen: false,
    old_ready_reused_after_transition: false,
  };
}

function advanceM4R07PostTickRendererDiagnosticCheckpoint(
  checkpoint: M4R07PostTickRendererDiagnosticCheckpoint | null,
  key: M4R07PostTickRendererDiagnosticCheckpointKey,
) {
  if (!checkpoint || checkpoint[key]) return;
  const index = M4R07_POST_TICK_RENDERER_DIAGNOSTIC_CHECKPOINT_ORDER.indexOf(key);
  if (
    index < 0
    || M4R07_POST_TICK_RENDERER_DIAGNOSTIC_CHECKPOINT_ORDER
      .slice(0, index)
      .some((prior) => !checkpoint[prior])
  ) throw new Error("m4r07_post_tick_diagnostic_checkpoint_order_invalid");
  checkpoint[key] = true;
}

function m4r07PostTickRendererDiagnosticCode(
  error: unknown,
  checkpoint: M4R07PostTickRendererDiagnosticCheckpoint | null,
): M4R07PostTickRendererDiagnosticCode {
  const message = error instanceof Error ? error.message : "";
  if ((M4R07_POST_TICK_RENDERER_DIAGNOSTIC_CODES as readonly string[])
    .includes(message)
  ) return message as M4R07PostTickRendererDiagnosticCode;
  if (checkpoint?.old_ready_reused_after_transition) {
    return "m4r07_post_tick_old_ready_reused";
  }
  if (checkpoint?.refresh_clicked && !checkpoint.transition_seen) {
    return "m4r07_post_tick_refresh_transition_not_observed";
  }
  if (checkpoint?.transition_seen && !checkpoint.new_ready_seen) {
    return "m4r07_post_tick_fresh_ready_not_observed";
  }
  if (checkpoint?.new_ready_seen && !checkpoint.dom5_seen) {
    return "m4r07_post_tick_dom_recovery_markers_not_observed";
  }
  if (checkpoint?.dom5_seen && !checkpoint.screenshot_pair_seen) {
    return "m4r07_post_tick_screenshot_markers_not_visible";
  }
  return "m4r07_post_tick_renderer_unclassified";
}

function isM4R07PostTickRendererDiagnosticInvocation(
  invocation: M4R03OrdinaryClockInvocation,
) {
  return invocation.schema_version
      === M4R07_POST_TICK_RENDERER_DIAGNOSTIC_IPC_SCHEMA_VERSION
    && invocation.phase === "recovery_timer"
    && invocation.operation === "observe_timer_tick";
}

function isM4R07PostTickRendererInvocation(
  invocation: M4R03OrdinaryClockInvocation,
) {
  return invocation.schema_version === M4R07_POST_TICK_RENDERER_IPC_SCHEMA_VERSION
    || isM4R07PostTickRendererDiagnosticInvocation(invocation);
}

type M4R03PreparedObjects = {
  openLoopId: string;
  reminderId: string;
  startupDueMarkerUtc: string;
  timerDueMarkerUtc: string | null;
};

let m4r03PreparedObjects: M4R03PreparedObjects | null = null;
let m4r03OperationQueue: Promise<void> = Promise.resolve();
let m4r03WriteCommandsInvoked = 0;
let m4r07PostTickRendererDiagnosticCheckpoint:
M4R07PostTickRendererDiagnosticCheckpoint | null = null;

function isM4R03Utc(value: unknown): value is string {
  return typeof value === "string"
    && /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$/.test(value)
    && !Number.isNaN(Date.parse(value));
}

function isM4R03Invocation(value: unknown): value is M4R03OrdinaryClockInvocation {
  if (!value || typeof value !== "object") return false;
  if (!hasExactKeys(value, [
    "nonce",
    "operation",
    "phase",
    "schema_version",
    "startup_due_marker_utc",
    "timer_due_marker_utc",
  ])) return false;
  const candidate = value as Partial<M4R03OrdinaryClockInvocation>;
  if (
    typeof candidate.nonce !== "string"
    || !/^[a-f0-9]{32}$/.test(candidate.nonce)
  ) return false;
  if (candidate.schema_version === M4R07_POST_TICK_RENDERER_IPC_SCHEMA_VERSION
    || candidate.schema_version
      === M4R07_POST_TICK_RENDERER_DIAGNOSTIC_IPC_SCHEMA_VERSION) {
    return candidate.phase === "recovery_timer"
      && candidate.operation === "observe_timer_tick"
      && isM4R03Utc(candidate.startup_due_marker_utc)
      && isM4R03Utc(candidate.timer_due_marker_utc);
  }
  if (candidate.schema_version !== M4R03_IPC_SCHEMA_VERSION) return false;
  if (candidate.phase === "arm" && candidate.operation === "arm_startup_recovery") {
    return candidate.startup_due_marker_utc === null
      && candidate.timer_due_marker_utc === null;
  }
  if (candidate.phase === "recovery_timer") {
    if (!isM4R03Utc(candidate.startup_due_marker_utc)) return false;
    if (
      candidate.operation === "observe_startup_recovery"
      || candidate.operation === "arm_timer_tick"
    ) return candidate.timer_due_marker_utc === null;
    return candidate.operation === "observe_timer_tick"
      && isM4R03Utc(candidate.timer_due_marker_utc);
  }
  return candidate.phase === "repeat"
    && candidate.operation === "observe_repeat"
    && isM4R03Utc(candidate.startup_due_marker_utc)
    && isM4R03Utc(candidate.timer_due_marker_utc);
}

async function loadM4R03ReadyHome() {
  const home = await loadSecretaryHomeContext();
  if (home.status !== "READY") throw new Error("m4r03_home_context_not_ready");
  return home;
}

function findM4R03OpenLoop(home: Awaited<ReturnType<typeof loadM4R03ReadyHome>>) {
  const matches = home.application_outcome.deterministic_brief.attention_items.filter(
    (item) => item.item_kind_code === "OPEN_LOOP",
  );
  if (matches.length !== 1) throw new Error("m4r03_open_loop_cardinality_invalid");
  return matches[0];
}

function findM4R03PersonalAction(home: Awaited<ReturnType<typeof loadM4R03ReadyHome>>) {
  const matches = home.application_outcome.local_objects.personal_actions.filter(
    (item) => item.title === M4R02_PERSONAL_ACTION_TITLE,
  );
  if (matches.length !== 1) throw new Error("m4r03_personal_action_cardinality_invalid");
  return matches[0];
}

function findM4R03Reminder(
  home: Awaited<ReturnType<typeof loadM4R03ReadyHome>>,
  startupDueMarkerUtc: string,
) {
  const matches = home.application_outcome.local_objects.reminders.filter(
    (item) => item.scheduled_for_utc === startupDueMarkerUtc,
  );
  if (matches.length !== 1) throw new Error("m4r03_reminder_cardinality_invalid");
  return matches[0];
}

async function waitForM4R03State(
  startupDueMarkerUtc: string,
  openLoopStatus: "OPEN" | "SNOOZED",
  reminderStatus: "SCHEDULED" | "SNOOZED" | "FIRED",
) {
  const deadline = Date.now() + M4R03_HOME_READ_TIMEOUT_MS;
  while (Date.now() < deadline) {
    const home = await loadM4R03ReadyHome();
    const openLoop = findM4R03OpenLoop(home);
    const reminder = findM4R03Reminder(home, startupDueMarkerUtc);
    if (openLoop.status_code === openLoopStatus && reminder.status === reminderStatus) {
      return { openLoop, reminder };
    }
    await delayM4R02(250);
  }
  throw new Error("m4r03_state_read_timeout");
}

async function waitForM4R03VisibleRecovery(
  startupDueMarkerUtc: string,
  diagnosticCheckpoint: M4R07PostTickRendererDiagnosticCheckpoint | null = null,
) {
  const priorReadyHomes = document.querySelectorAll<HTMLElement>(
    'main.secretary-home[data-secretary-home-state="ready"]',
  );
  if (priorReadyHomes.length !== 1) throw new Error("m4r03_home_visible_prior_state_invalid");
  const priorReadyHome = priorReadyHomes[0];
  advanceM4R07PostTickRendererDiagnosticCheckpoint(
    diagnosticCheckpoint,
    "prior_ready",
  );
  const refreshButtons = Array.from(
    document.querySelectorAll<HTMLButtonElement>('[data-workbench-refresh="true"]'),
  ).filter((button) => (
    !button.disabled
    && button.getClientRects().length > 0
    && getComputedStyle(button).visibility !== "hidden"
    && getComputedStyle(button).display !== "none"
  ));
  if (refreshButtons.length !== 1) throw new Error("m4r03_home_refresh_cardinality_invalid");

  const deadline = Date.now() + M4R03_HOME_READ_TIMEOUT_MS;
  let transitionObserved = false;
  const observer = new MutationObserver(() => {
    transitionObserved ||= !priorReadyHome.isConnected
      || priorReadyHome.dataset.secretaryHomeState !== "ready"
      || Boolean(document.querySelector(
        'main.secretary-home[data-secretary-home-state="loading"]',
      ));
  });
  observer.observe(document.body, {
    attributes: true,
    attributeFilter: ["data-secretary-home-state"],
    childList: true,
    subtree: true,
  });
  refreshButtons[0].click();
  advanceM4R07PostTickRendererDiagnosticCheckpoint(
    diagnosticCheckpoint,
    "refresh_clicked",
  );
  try {
    while (Date.now() < deadline) {
      transitionObserved ||= !priorReadyHome.isConnected
        || priorReadyHome.dataset.secretaryHomeState !== "ready";
      if (transitionObserved) {
        advanceM4R07PostTickRendererDiagnosticCheckpoint(
          diagnosticCheckpoint,
          "transition_seen",
        );
      }
      const readyHomes = document.querySelectorAll<HTMLElement>(
        'main.secretary-home[data-secretary-home-state="ready"]',
      );
      if (transitionObserved && readyHomes.length === 1) {
        if (readyHomes[0] === priorReadyHome) {
          if (diagnosticCheckpoint) {
            diagnosticCheckpoint.old_ready_reused_after_transition = true;
            throw new Error("m4r07_post_tick_old_ready_reused");
          }
          await delayM4R02(100);
          continue;
        }
        advanceM4R07PostTickRendererDiagnosticCheckpoint(
          diagnosticCheckpoint,
          "new_ready_seen",
        );
        const openLoops = Array.from(
          readyHomes[0].querySelectorAll<HTMLElement>('[data-item-kind="OPEN_LOOP"]'),
        );
        const expectedReminderMarker = startupDueMarkerUtc.replace("T", " ").replace("Z", " UTC");
        const reminders = Array.from(
          readyHomes[0].querySelectorAll<HTMLElement>(
            '.secretary-personal-actions[aria-labelledby="secretary-reminders-title"] > ul > li',
          ),
        ).filter((element) => (
          element.querySelector<HTMLElement>(":scope > strong")?.textContent?.trim()
            === expectedReminderMarker
        ));
        const rendered = (element: HTMLElement) => (
          (() => {
            const style = getComputedStyle(element);
            return element.getClientRects().length > 0
              && (typeof element.checkVisibility !== "function" || element.checkVisibility())
              && style.visibility !== "hidden"
              && style.display !== "none"
              && style.opacity !== "0";
          })()
        );
        const exactRenderedText = (element: HTMLElement | null, expected: string) => (
          Boolean(element)
          && element?.textContent?.trim() === expected
          && rendered(element)
        );
        const exactMarkerNodes = [
          [readyHomes[0].querySelector<HTMLElement>("#secretary-home-title"), "现在要看住什么"],
          [readyHomes[0].querySelector<HTMLElement>(".secretary-context-status"), "已继续同一情境"],
          [readyHomes[0].querySelector<HTMLElement>("#secretary-attention-title"), "持续关注"],
          [openLoops[0]?.querySelector<HTMLElement>(".secretary-spine-status > code:first-of-type") ?? null, "OPEN"],
          [reminders[0]?.querySelector<HTMLElement>("span > code") ?? null, "FIRED"],
        ] as const;
        if (
          openLoops.length === 1
          && reminders.length === 1
          && exactMarkerNodes.every(([element, expected]) => exactRenderedText(element, expected))
        ) {
          advanceM4R07PostTickRendererDiagnosticCheckpoint(
            diagnosticCheckpoint,
            "dom5_seen",
          );
          const reminderHeading = readyHomes[0].querySelector<HTMLElement>(
            "#secretary-reminders-title",
          );
          const reminderStatus = exactMarkerNodes[4][0];
          reminderStatus?.scrollIntoView({ block: "center", inline: "nearest" });
          await new Promise<void>((resolveFrame) => requestAnimationFrame(() => {
            requestAnimationFrame(() => resolveFrame());
          }));
          const fullyVisibleInHome = (element: HTMLElement | null) => {
            if (!element || !rendered(element)) return false;
            const rect = element.getBoundingClientRect();
            const homeRect = readyHomes[0].getBoundingClientRect();
            return rect.left >= Math.max(0, homeRect.left)
              && rect.top >= Math.max(0, homeRect.top)
              && rect.right <= Math.min(window.innerWidth, homeRect.right)
              && rect.bottom <= Math.min(window.innerHeight, homeRect.bottom);
          };
          if (
            !exactRenderedText(reminderHeading, "提醒")
            || !exactRenderedText(reminderStatus, "FIRED")
            || !fullyVisibleInHome(reminderHeading)
            || !fullyVisibleInHome(reminderStatus)
          ) {
            await delayM4R02(100);
            continue;
          }
          advanceM4R07PostTickRendererDiagnosticCheckpoint(
            diagnosticCheckpoint,
            "screenshot_pair_seen",
          );
          const domProjection = JSON.stringify({
            visible_markers: [...M4R03_RECOVERY_VISIBLE_MARKERS],
            startup_due_marker_sha256: await m4r03Sha256(startupDueMarkerUtc),
            open_loop_status: "OPEN",
            reminder_status: "FIRED",
            refresh_clicked: true,
            refresh_transition_observed: true,
            scroll_performed: true,
            scroll_settled: true,
          });
          const screenshotProjection = JSON.stringify({
            visible_markers: ["提醒", "FIRED"],
          });
          return {
            domRecoveryMarkersSha256: await m4r03Sha256(domProjection),
            screenshotVisibleMarkersSha256: await m4r03Sha256(screenshotProjection),
          };
        }
      }
      const terminalHome = document.querySelector<HTMLElement>(
        'main.secretary-home[data-secretary-home-state="error"],'
        + 'main.secretary-home[data-secretary-home-state="degraded"],'
        + 'main.secretary-home[data-secretary-home-state="empty"]',
      );
      if (terminalHome) throw new Error("m4r03_home_visible_terminal_state");
      await delayM4R02(100);
    }
  } finally {
    observer.disconnect();
  }
  throw new Error("m4r03_home_visible_recovery_timeout");
}

function m4r03ErrorFamily(error: unknown) {
  const message = error instanceof Error ? error.message : String(error);
  if (message.includes("state_read_timeout")) return "state_read_timeout";
  if (message.includes("home_visible")) return "home_visible_contract";
  if (message.includes("home_refresh")) return "home_visible_contract";
  if (message.includes("home_context")) return "home_read_contract";
  if (message.includes("cardinality")) return "object_cardinality";
  if (message.includes("prepared")) return "prepared_binding";
  return "command_rejected";
}

async function emitM4R03Result(
  invocation: M4R03OrdinaryClockInvocation,
  result: Record<string, unknown>,
) {
  await emit(M4R03_IPC_RESULT_EVENT, {
    schema_version: invocation.schema_version,
    phase: invocation.phase,
    operation: invocation.operation,
    nonce: invocation.nonce,
    ui_refresh_clicked: false,
    ui_refresh_transition_observed: false,
    ui_recovery_dom_projection_sha256: null,
    ui_recovery_screenshot_projection_sha256: null,
    ...result,
  });
}

async function runM4R03OrdinaryClockOperation(
  invocation: M4R03OrdinaryClockInvocation,
) {
  m4r03WriteCommandsInvoked = 0;
  m4r07PostTickRendererDiagnosticCheckpoint =
    isM4R07PostTickRendererDiagnosticInvocation(invocation)
      ? newM4R07PostTickRendererDiagnosticCheckpoint()
      : null;
  switch (invocation.operation) {
    case "arm_startup_recovery": {
      const home = await loadM4R03ReadyHome();
      const openLoop = findM4R03OpenLoop(home);
      const personalAction = findM4R03PersonalAction(home);
      if (openLoop.status_code !== "OPEN") throw new Error("m4r03_open_loop_not_open");
      const startupDueMarkerUtc = new Date(
        Date.now() + M4R03_STARTUP_DUE_DELAY_MS,
      ).toISOString();
      const openLoopIdempotencyKey = await mintSecretaryCoordinationIdempotencyKey();
      m4r03WriteCommandsInvoked += 1;
      const openLoopReceipt = await operateSecretaryCoordination({
        action: "OPEN_LOOP_SNOOZE",
        item_ref: openLoop.item_ref,
        expected_revision: openLoop.coordination_revision,
        snoozed_until_utc: startupDueMarkerUtc,
        idempotency_key: openLoopIdempotencyKey,
      });
      const reminderIdempotencyKey = await mintSecretaryCoordinationIdempotencyKey();
      m4r03WriteCommandsInvoked += 1;
      const reminderReceipt = await operateSecretaryPersonalObject({
        action: "REMINDER_CREATE",
        owner_ref: personalAction.personal_action_id,
        scheduled_for_utc: startupDueMarkerUtc,
        iana_timezone: "Asia/Shanghai",
        idempotency_key: reminderIdempotencyKey,
      });
      const armed = await waitForM4R03State(startupDueMarkerUtc, "SNOOZED", "SCHEDULED");
      if (
        openLoopReceipt.item_ref !== armed.openLoop.item_ref
        || reminderReceipt.item_ref !== armed.reminder.reminder_id
        || openLoopReceipt.aggregate_kind_code !== "OPEN_LOOP"
        || reminderReceipt.aggregate_kind_code !== "REMINDER"
        || openLoopReceipt.coordination_revision !== armed.openLoop.coordination_revision
        || reminderReceipt.coordination_revision !== armed.reminder.revision
        || openLoopReceipt.outcome_code !== "APPLIED"
        || reminderReceipt.outcome_code !== "CREATED"
        || openLoopReceipt.replayed
        || reminderReceipt.replayed
      ) throw new Error("m4r03_arm_receipt_binding_invalid");
      m4r03PreparedObjects = {
        openLoopId: armed.openLoop.item_ref,
        reminderId: armed.reminder.reminder_id,
        startupDueMarkerUtc,
        timerDueMarkerUtc: null,
      };
      await emitM4R03Result(invocation, {
        outcome: "PASS",
        startup_due_marker_utc: startupDueMarkerUtc,
        timer_due_marker_utc: null,
        open_loop_id: armed.openLoop.item_ref,
        open_loop_status: armed.openLoop.status_code,
        open_loop_revision: armed.openLoop.coordination_revision,
        reminder_id: armed.reminder.reminder_id,
        reminder_status: armed.reminder.status,
        reminder_revision: armed.reminder.revision,
        reminder_last_fired_at_utc: armed.reminder.last_fired_at_utc,
        open_loop_command_receipt_ref: openLoopReceipt.command_receipt_ref,
        reminder_command_receipt_ref: reminderReceipt.command_receipt_ref,
        write_commands_invoked: 2,
      });
      return;
    }
    case "observe_startup_recovery": {
      const startupDueMarkerUtc = invocation.startup_due_marker_utc;
      if (!startupDueMarkerUtc) throw new Error("m4r03_startup_marker_missing");
      const recovered = await waitForM4R03State(startupDueMarkerUtc, "OPEN", "FIRED");
      m4r03PreparedObjects = {
        openLoopId: recovered.openLoop.item_ref,
        reminderId: recovered.reminder.reminder_id,
        startupDueMarkerUtc,
        timerDueMarkerUtc: null,
      };
      await emitM4R03Result(invocation, {
        outcome: "PASS",
        startup_due_marker_utc: startupDueMarkerUtc,
        timer_due_marker_utc: null,
        open_loop_id: recovered.openLoop.item_ref,
        open_loop_status: recovered.openLoop.status_code,
        open_loop_revision: recovered.openLoop.coordination_revision,
        reminder_id: recovered.reminder.reminder_id,
        reminder_status: recovered.reminder.status,
        reminder_revision: recovered.reminder.revision,
        reminder_last_fired_at_utc: recovered.reminder.last_fired_at_utc,
        open_loop_command_receipt_ref: null,
        reminder_command_receipt_ref: null,
        write_commands_invoked: 0,
      });
      return;
    }
    case "arm_timer_tick": {
      const prepared = m4r03PreparedObjects;
      if (!prepared || prepared.startupDueMarkerUtc !== invocation.startup_due_marker_utc) {
        throw new Error("m4r03_prepared_binding_invalid");
      }
      const home = await loadM4R03ReadyHome();
      const openLoop = findM4R03OpenLoop(home);
      const reminder = findM4R03Reminder(home, prepared.startupDueMarkerUtc);
      if (
        openLoop.item_ref !== prepared.openLoopId
        || reminder.reminder_id !== prepared.reminderId
        || openLoop.status_code !== "OPEN"
        || reminder.status !== "FIRED"
      ) throw new Error("m4r03_timer_arm_state_invalid");
      const timerDueMarkerUtc = new Date(Date.now() + M4R03_TIMER_DUE_DELAY_MS).toISOString();
      const openLoopIdempotencyKey = await mintSecretaryCoordinationIdempotencyKey();
      m4r03WriteCommandsInvoked += 1;
      const openLoopReceipt = await operateSecretaryCoordination({
        action: "OPEN_LOOP_SNOOZE",
        item_ref: openLoop.item_ref,
        expected_revision: openLoop.coordination_revision,
        snoozed_until_utc: timerDueMarkerUtc,
        idempotency_key: openLoopIdempotencyKey,
      });
      const reminderIdempotencyKey = await mintSecretaryCoordinationIdempotencyKey();
      m4r03WriteCommandsInvoked += 1;
      const reminderReceipt = await operateSecretaryPersonalObject({
        action: "REMINDER_SNOOZE",
        item_ref: reminder.reminder_id,
        expected_revision: reminder.revision,
        snoozed_until_utc: timerDueMarkerUtc,
        idempotency_key: reminderIdempotencyKey,
      });
      const armed = await waitForM4R03State(prepared.startupDueMarkerUtc, "SNOOZED", "SNOOZED");
      if (
        openLoopReceipt.item_ref !== armed.openLoop.item_ref
        || reminderReceipt.item_ref !== armed.reminder.reminder_id
        || openLoopReceipt.aggregate_kind_code !== "OPEN_LOOP"
        || reminderReceipt.aggregate_kind_code !== "REMINDER"
        || openLoopReceipt.coordination_revision !== armed.openLoop.coordination_revision
        || reminderReceipt.coordination_revision !== armed.reminder.revision
        || openLoopReceipt.outcome_code !== "APPLIED"
        || reminderReceipt.outcome_code !== "APPLIED"
        || openLoopReceipt.replayed
        || reminderReceipt.replayed
      ) throw new Error("m4r03_timer_arm_receipt_binding_invalid");
      prepared.timerDueMarkerUtc = timerDueMarkerUtc;
      await emitM4R03Result(invocation, {
        outcome: "PASS",
        startup_due_marker_utc: prepared.startupDueMarkerUtc,
        timer_due_marker_utc: timerDueMarkerUtc,
        open_loop_id: armed.openLoop.item_ref,
        open_loop_status: armed.openLoop.status_code,
        open_loop_revision: armed.openLoop.coordination_revision,
        reminder_id: armed.reminder.reminder_id,
        reminder_status: armed.reminder.status,
        reminder_revision: armed.reminder.revision,
        reminder_last_fired_at_utc: armed.reminder.last_fired_at_utc,
        open_loop_command_receipt_ref: openLoopReceipt.command_receipt_ref,
        reminder_command_receipt_ref: reminderReceipt.command_receipt_ref,
        write_commands_invoked: 2,
      });
      return;
    }
    case "observe_timer_tick": {
      const prepared = m4r03PreparedObjects;
      if (
        !prepared
        || prepared.startupDueMarkerUtc !== invocation.startup_due_marker_utc
        || prepared.timerDueMarkerUtc !== invocation.timer_due_marker_utc
      ) throw new Error("m4r03_prepared_binding_invalid");
      const advanced = await waitForM4R03State(
        prepared.startupDueMarkerUtc,
        "OPEN",
        "FIRED",
      );
      if (
        advanced.openLoop.item_ref !== prepared.openLoopId
        || advanced.reminder.reminder_id !== prepared.reminderId
      ) throw new Error("m4r03_prepared_binding_invalid");
      const postTickRendererInvocation = isM4R07PostTickRendererInvocation(invocation);
      const uiRecoveryProjection = postTickRendererInvocation
        ? await waitForM4R03VisibleRecovery(
            prepared.startupDueMarkerUtc,
            m4r07PostTickRendererDiagnosticCheckpoint,
          )
        : null;
      await emitM4R03Result(invocation, {
        outcome: "PASS",
        startup_due_marker_utc: prepared.startupDueMarkerUtc,
        timer_due_marker_utc: prepared.timerDueMarkerUtc,
        open_loop_id: advanced.openLoop.item_ref,
        open_loop_status: advanced.openLoop.status_code,
        open_loop_revision: advanced.openLoop.coordination_revision,
        reminder_id: advanced.reminder.reminder_id,
        reminder_status: advanced.reminder.status,
        reminder_revision: advanced.reminder.revision,
        reminder_last_fired_at_utc: advanced.reminder.last_fired_at_utc,
        open_loop_command_receipt_ref: null,
        reminder_command_receipt_ref: null,
        write_commands_invoked: 0,
        ui_refresh_clicked: postTickRendererInvocation,
        ui_refresh_transition_observed: postTickRendererInvocation,
        ui_recovery_dom_projection_sha256:
          uiRecoveryProjection?.domRecoveryMarkersSha256 ?? null,
        ui_recovery_screenshot_projection_sha256:
          uiRecoveryProjection?.screenshotVisibleMarkersSha256 ?? null,
        ...(isM4R07PostTickRendererDiagnosticInvocation(invocation)
          ? {
              diagnostic_code: null,
              diagnostic_checkpoint: m4r07PostTickRendererDiagnosticCheckpoint,
            }
          : {}),
      });
      return;
    }
    case "observe_repeat": {
      const startupDueMarkerUtc = invocation.startup_due_marker_utc;
      if (!startupDueMarkerUtc) throw new Error("m4r03_startup_marker_missing");
      const stable = await waitForM4R03State(startupDueMarkerUtc, "OPEN", "FIRED");
      await emitM4R03Result(invocation, {
        outcome: "PASS",
        startup_due_marker_utc: startupDueMarkerUtc,
        timer_due_marker_utc: invocation.timer_due_marker_utc,
        open_loop_id: stable.openLoop.item_ref,
        open_loop_status: stable.openLoop.status_code,
        open_loop_revision: stable.openLoop.coordination_revision,
        reminder_id: stable.reminder.reminder_id,
        reminder_status: stable.reminder.status,
        reminder_revision: stable.reminder.revision,
        reminder_last_fired_at_utc: stable.reminder.last_fired_at_utc,
        open_loop_command_receipt_ref: null,
        reminder_command_receipt_ref: null,
        write_commands_invoked: 0,
      });
    }
  }
}

async function installM4R03OrdinaryClockTauriIpcBridge() {
  if (!("__TAURI_INTERNALS__" in window)) return;
  try {
    await listen<M4R03OrdinaryClockInvocation>(M4R03_IPC_INVOKE_EVENT, ({ payload }) => {
      if (!isM4R03Invocation(payload)) return;
      m4r03OperationQueue = m4r03OperationQueue.then(async () => {
        try {
          await runM4R03OrdinaryClockOperation(payload);
        } catch (error) {
          const diagnosticInvocation =
            isM4R07PostTickRendererDiagnosticInvocation(payload);
          await emitM4R03Result(payload, {
            outcome: "REJECTED",
            startup_due_marker_utc: payload.startup_due_marker_utc,
            timer_due_marker_utc: payload.timer_due_marker_utc,
            open_loop_id: null,
            open_loop_status: null,
            open_loop_revision: null,
            reminder_id: null,
            reminder_status: null,
            reminder_revision: null,
            reminder_last_fired_at_utc: null,
            open_loop_command_receipt_ref: null,
            reminder_command_receipt_ref: null,
            write_commands_invoked: m4r03WriteCommandsInvoked,
            error_family: m4r03ErrorFamily(error),
            ...(diagnosticInvocation
              ? {
                  diagnostic_code: m4r07PostTickRendererDiagnosticCode(
                    error,
                    m4r07PostTickRendererDiagnosticCheckpoint,
                  ),
                  diagnostic_checkpoint:
                    m4r07PostTickRendererDiagnosticCheckpoint,
                }
              : {}),
          });
        }
      });
    });
    await emit(M4R03_IPC_READY_EVENT, {
      schema_version: M4R03_IPC_SCHEMA_VERSION,
      surface: "ordinary_registered_tauri_command_ipc",
      phases: ["arm", "recovery_timer", "repeat"],
    });
  } catch {
    // The host owns bounded timeouts and a terminal, value-free receipt.
  }
}

// M4R04 ordinary-product route proof. The host sends only bounded orchestration
// events. Source resolution remains the registered product wrapper reached by
// a real Secretary DOM click; this bridge never reconstructs a target route.
const M4R04_IPC_READY_EVENT = "syn-m4r04-ordinary-route-ui-ready";
const M4R04_IPC_INVOKE_EVENT = "syn-m4r04-ordinary-route-invoke";
const M4R04_IPC_RESULT_EVENT = "syn-m4r04-ordinary-route-result";
const M4R04_IPC_SCHEMA_VERSION = "syn_m4r04_ordinary_route_ipc.v1";
const M4R04_WORK_ITEM_OWNER = "owner:m2-workflow-state-work-item:v1";
const M4R04_PROPOSAL_OWNER = "owner:project-consultation-proposal:v1";
const M4R04_HOME_LINK_TYPE = "workflow_attention";
const M4R04_WORK_ITEM_NATIVE_TYPE = "workflow_attention";
const M4R04_PROPOSAL_NATIVE_TYPE = "proposal_decision";
const M4R04_DOM_WAIT_MS = 25_000;

type M4R04Phase = "work_item" | "proposal" | "restart_negative";
type M4R04Operation =
  | "click_work_item_route"
  | "create_proposal_source"
  | "click_proposal_route"
  | "click_restart_work_item"
  | "click_restart_proposal"
  | "advance_check_negatives_and_click_current";

type M4R04OrdinaryRouteInvocation = {
  schema_version: typeof M4R04_IPC_SCHEMA_VERSION;
  phase: M4R04Phase;
  operation: M4R04Operation;
  nonce: string;
  project_root: string;
};

type M4R04RouteAction = {
  source_owner_ref: string;
  source_object_type: string;
  canonical_source_object_id: string;
  source_route_ref: string;
  source_action_dom_count: number;
};

type M4R04RouteObservation = M4R04RouteAction & {
  source_revision: string | null;
  source_action_seen: boolean;
  route_action_clicks: number;
  consumed_marker_count: number;
  active_view: string;
  route_phase: string;
  success_notice_count: number;
};

type M4R04NegativeObservation = {
  stale_error_code: string;
  tampered_error_code: string;
  resolver_wrapper_calls: number;
  stale_ui_phase: string;
  stale_notice_error_code: string;
  stale_route_action_clicks: number;
  active_view_before: string;
  active_view_after: string;
  route_phase_before: string;
  route_phase_after: string;
  consumed_marker_count_before: number;
  consumed_marker_count_after: number;
  success_notice_count_before: number;
  success_notice_count_after: number;
  zero_navigation: boolean;
  zero_consume_delta: boolean;
  zero_success_delta: boolean;
};

type M4R04PreparedState = {
  nonce: string;
  projectRoot: string;
  workItem: M4R04RouteObservation | null;
  proposal: M4R04RouteObservation | null;
};

const m4r04Counters = {
  proposalCreateCalls: 0,
  workItemUpdateCalls: 0,
  routeActionClicks: 0,
  navigationClicks: 0,
  refreshClicks: 0,
  resolverWrapperCalls: 0,
};
let m4r04PreparedState: M4R04PreparedState | null = null;
let m4r04OperationQueue = Promise.resolve();

function isM4R04Invocation(value: unknown): value is M4R04OrdinaryRouteInvocation {
  if (!value || typeof value !== "object") return false;
  const candidate = value as Partial<M4R04OrdinaryRouteInvocation>;
  if (!hasExactKeys(value, ["nonce", "operation", "phase", "project_root", "schema_version"])) {
    return false;
  }
  if (
    candidate.schema_version !== M4R04_IPC_SCHEMA_VERSION
    || typeof candidate.nonce !== "string"
    || !/^[a-f0-9]{32}$/.test(candidate.nonce)
    || typeof candidate.project_root !== "string"
    || candidate.project_root.length === 0
    || candidate.project_root.length > 1024
    || /[\r\n]/.test(candidate.project_root)
  ) return false;
  return (
    (candidate.phase === "work_item" && candidate.operation === "click_work_item_route")
    || (candidate.phase === "proposal"
      && (candidate.operation === "create_proposal_source"
        || candidate.operation === "click_proposal_route"))
    || (candidate.phase === "restart_negative"
      && (candidate.operation === "click_restart_work_item"
        || candidate.operation === "click_restart_proposal"
        || candidate.operation === "advance_check_negatives_and_click_current"))
  );
}

function m4r04Delay(milliseconds: number) {
  return new Promise<void>((resolve) => window.setTimeout(resolve, milliseconds));
}

function m4r04AppShell() {
  const shell = document.querySelector<HTMLElement>(".app-shell[data-active-view]");
  if (!shell) throw new Error("m4r04_app_shell_missing");
  return shell;
}

function m4r04ActiveView() {
  return m4r04AppShell().dataset.activeView ?? "";
}

function m4r04RoutePhase() {
  return m4r04AppShell().dataset.secretarySourceRoutePhase ?? "";
}

function m4r04ConsumedMarkerCount() {
  return document.querySelectorAll('[data-secretary-source-focus-status="CONSUMED"]').length;
}

function m4r04SuccessNoticeCount() {
  return document.querySelectorAll('[data-secretary-source-route-notice="CONSUMED"]').length;
}

function m4r04ReadAction(element: HTMLElement): Omit<M4R04RouteAction, "source_action_dom_count"> {
  const sourceRouteRef = element.dataset.secretarySourceRouteRef ?? "";
  const sourceOwnerRef = element.dataset.secretarySourceOwner ?? "";
  const sourceObjectType = element.dataset.secretarySourceObjectType ?? "";
  const canonicalSourceObjectId = element.dataset.secretarySourceObjectId ?? "";
  if (
    !/^source-route:sha256:[a-f0-9]{64}$/.test(sourceRouteRef)
    || sourceOwnerRef.length === 0
    || sourceObjectType.length === 0
    || canonicalSourceObjectId.length === 0
    || canonicalSourceObjectId.length > 512
    || /[\\/\r\n]/.test(canonicalSourceObjectId)
  ) throw new Error("m4r04_source_action_binding_invalid");
  return {
    source_owner_ref: sourceOwnerRef,
    source_object_type: sourceObjectType,
    canonical_source_object_id: canonicalSourceObjectId,
    source_route_ref: sourceRouteRef,
  };
}

function m4r04FindSourceAction(
  owner: string,
  objectType: string,
  objectId?: string,
  excludedRouteRef?: string,
): { action: M4R04RouteAction; button: HTMLButtonElement } | null {
  const candidates = Array.from(
    document.querySelectorAll<HTMLElement>('[data-secretary-source-route-action="OPEN"]'),
  ).map((element) => ({ element, binding: m4r04ReadAction(element) }))
    .filter(({ binding }) => (
      binding.source_owner_ref === owner
      && binding.source_object_type === objectType
      && (objectId === undefined || binding.canonical_source_object_id === objectId)
      && (excludedRouteRef === undefined || binding.source_route_ref !== excludedRouteRef)
    ));
  if (candidates.length === 0) return null;
  const distinctRoutes = new Map<string, typeof candidates>();
  for (const candidate of candidates) {
    const group = distinctRoutes.get(candidate.binding.source_route_ref) ?? [];
    group.push(candidate);
    distinctRoutes.set(candidate.binding.source_route_ref, group);
  }
  if (distinctRoutes.size !== 1) throw new Error("m4r04_distinct_source_route_collision");
  const group = Array.from(distinctRoutes.values())[0];
  const first = group[0].binding;
  if (group.some(({ binding }) => (
    binding.source_owner_ref !== first.source_owner_ref
    || binding.source_object_type !== first.source_object_type
    || binding.canonical_source_object_id !== first.canonical_source_object_id
  ))) throw new Error("m4r04_source_route_tuple_collision");
  const button = group.map(({ element }) => element)
    .find((element): element is HTMLButtonElement => (
      element instanceof HTMLButtonElement && !element.disabled
    ));
  if (!button) throw new Error("m4r04_source_route_action_disabled");
  return {
    action: { ...first, source_action_dom_count: group.length },
    button,
  };
}

async function m4r04WaitForSourceAction(
  owner: string,
  objectType: string,
  objectId?: string,
  excludedRouteRef?: string,
) {
  const deadline = Date.now() + M4R04_DOM_WAIT_MS;
  while (Date.now() < deadline) {
    const match = m4r04FindSourceAction(owner, objectType, objectId, excludedRouteRef);
    if (match) return match;
    await m4r04Delay(150);
  }
  throw new Error("m4r04_source_action_timeout");
}

async function m4r04ClickRefresh() {
  const button = document.querySelector<HTMLButtonElement>('[data-workbench-refresh="true"]');
  if (!button || button.disabled) throw new Error("m4r04_refresh_action_missing");
  m4r04Counters.refreshClicks += 1;
  button.click();
  await m4r04Delay(250);
}

async function m4r04NavigateHome() {
  const button = document.querySelector<HTMLButtonElement>('button[aria-label="回到首页"]');
  if (!button || button.disabled) throw new Error("m4r04_home_navigation_missing");
  m4r04Counters.navigationClicks += 1;
  button.click();
  const deadline = Date.now() + M4R04_DOM_WAIT_MS;
  while (Date.now() < deadline) {
    if (
      m4r04ActiveView() === "home"
      && m4r04RoutePhase() === "IDLE"
      && m4r04ConsumedMarkerCount() === 0
      && m4r04SuccessNoticeCount() === 0
    ) return;
    await m4r04Delay(100);
  }
  throw new Error("m4r04_home_navigation_timeout");
}

async function m4r04ClickAndObserveRoute(
  selected: { action: M4R04RouteAction; button: HTMLButtonElement },
  expectedNativeType: string,
): Promise<M4R04RouteObservation> {
  m4r04Counters.routeActionClicks += 1;
  // A successful product route click brackets the ordinary owner reads with
  // two exact validations of the same sealed capability.
  m4r04Counters.resolverWrapperCalls += 2;
  selected.button.click();
  const deadline = Date.now() + M4R04_DOM_WAIT_MS;
  while (Date.now() < deadline) {
    const shell = m4r04AppShell();
    const routeStateMatchesClick = shell.dataset.secretarySourceRouteRef
      === selected.action.source_route_ref;
    if (routeStateMatchesClick && shell.dataset.secretarySourceRoutePhase === "FAILED") {
      const errorCode = shell.dataset.secretarySourceRouteErrorCode ?? "M4_SOURCE_ROUTE_FAILED";
      if (/^M4_SOURCE_[A-Z0-9_]{1,56}$/.test(errorCode)) {
        throw new Error(`m4r04_source_resolver_failed:${errorCode}`);
      }
      if (/^SECRETARY_SOURCE_TARGET_[A-Z0-9_]{1,48}$/.test(errorCode)) {
        throw new Error(`m4r04_source_consumer_failed:${errorCode}`);
      }
      throw new Error("m4r04_source_focus_failed_invalid_code");
    }
    const markers = Array.from(
      document.querySelectorAll<HTMLElement>('[data-secretary-source-focus-status="CONSUMED"]'),
    ).filter((marker) => (
      marker.dataset.secretarySourceOwner === selected.action.source_owner_ref
      && marker.dataset.secretarySourceObjectType === expectedNativeType
      && marker.dataset.secretarySourceObjectId === selected.action.canonical_source_object_id
      && marker.dataset.secretarySourceRouteRef === selected.action.source_route_ref
    ));
    const revision = markers[0]?.dataset.secretarySourceRevision ?? "";
    const notices = m4r04SuccessNoticeCount();
    if (
      shell.dataset.activeView === "projects"
      && shell.dataset.secretarySourceRoutePhase === "CONSUMED"
      && routeStateMatchesClick
      && markers.length === 1
      && /^(0|[1-9][0-9]*)$/.test(revision)
      && notices === 1
    ) {
      return {
        ...selected.action,
        source_object_type: expectedNativeType,
        source_revision: revision,
        source_action_seen: true,
        route_action_clicks: 1,
        consumed_marker_count: 1,
        active_view: "projects",
        route_phase: "CONSUMED",
        success_notice_count: 1,
      };
    }
    await m4r04Delay(100);
  }
  const focusStatuses = Array.from(
    document.querySelectorAll<HTMLElement>("[data-secretary-source-focus-status]"),
  ).map((element) => element.dataset.secretarySourceFocusStatus ?? "");
  if (focusStatuses.includes("PENDING")) {
    throw new Error("m4r04_focus_pending_timeout");
  }
  if (m4r04RoutePhase() === "CONSUMED") {
    throw new Error("m4r04_focus_consumed_contract_timeout");
  }
  throw new Error("m4r04_focus_consumer_missing_timeout");
}

function m4r04ActionObservation(
  action: M4R04RouteAction,
  expectedNativeType: string,
): M4R04RouteObservation {
  return {
    ...action,
    source_object_type: expectedNativeType,
    source_revision: null,
    source_action_seen: true,
    route_action_clicks: 0,
    consumed_marker_count: 0,
    active_view: m4r04ActiveView(),
    route_phase: m4r04RoutePhase(),
    success_notice_count: m4r04SuccessNoticeCount(),
  };
}

function m4r04RequirePrepared(invocation: M4R04OrdinaryRouteInvocation) {
  const prepared = m4r04PreparedState;
  if (
    !prepared
    || prepared.nonce !== invocation.nonce
    || prepared.projectRoot !== invocation.project_root
  ) throw new Error("m4r04_prepared_binding_invalid");
  return prepared;
}

function m4r04FlipRouteDigest(routeRef: string) {
  if (!/^source-route:sha256:[a-f0-9]{64}$/.test(routeRef)) {
    throw new Error("m4r04_route_ref_invalid");
  }
  const last = routeRef.at(-1);
  return `${routeRef.slice(0, -1)}${last === "0" ? "1" : "0"}`;
}

async function m4r04ExpectedResolverFailure(sourceRouteRef: string, expectedCode: string) {
  m4r04Counters.resolverWrapperCalls += 1;
  try {
    await resolveSecretarySourceRoute({ source_route_ref: sourceRouteRef });
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    if (message.includes(expectedCode)) return expectedCode;
    throw new Error("m4r04_resolver_error_code_mismatch");
  }
  throw new Error("m4r04_resolver_unexpected_success");
}

async function m4r04WaitForHomeRoute(
  owner: string,
  objectId: string,
  excludedRouteRef?: string,
): Promise<M4R04RouteAction> {
  const deadline = Date.now() + M4R04_DOM_WAIT_MS;
  while (Date.now() < deadline) {
    const home = await loadSecretaryHomeContext();
    if (home.status === "READY") {
      const matches = home.application_outcome.deterministic_brief.attention_items.filter((item) => (
        item.source_owner_ref === owner
        && item.source_object_type === M4R04_HOME_LINK_TYPE
        && item.source_object_ref === objectId
        && (excludedRouteRef === undefined || item.source_route_ref !== excludedRouteRef)
      ));
      const routes = new Map(matches.map((item) => [item.source_route_ref, item]));
      if (routes.size > 1) throw new Error("m4r04_current_route_collision");
      const item = Array.from(routes.values())[0];
      if (item && /^source-route:sha256:[a-f0-9]{64}$/.test(item.source_route_ref)) {
        return {
          source_owner_ref: item.source_owner_ref,
          source_object_type: item.source_object_type,
          canonical_source_object_id: item.source_object_ref,
          source_route_ref: item.source_route_ref,
          source_action_dom_count: 0,
        };
      }
    }
    await m4r04Delay(150);
  }
  throw new Error("m4r04_current_home_route_timeout");
}

async function m4r04ClickAndObserveStale(
  selected: { action: M4R04RouteAction; button: HTMLButtonElement },
) {
  m4r04Counters.routeActionClicks += 1;
  m4r04Counters.resolverWrapperCalls += 1;
  selected.button.click();
  const deadline = Date.now() + M4R04_DOM_WAIT_MS;
  while (Date.now() < deadline) {
    const shell = m4r04AppShell();
    const notices = Array.from(
      document.querySelectorAll<HTMLElement>('[data-secretary-source-route-notice="FAILED"]'),
    ).filter((notice) => (
      notice.dataset.secretarySourceRouteNoticeErrorCode === "M4_SOURCE_ROUTE_STALE"
    ));
    if (
      shell.dataset.activeView === "home"
      && shell.dataset.secretarySourceRoutePhase === "FAILED"
      && shell.dataset.secretarySourceRouteRef === selected.action.source_route_ref
      && shell.dataset.secretarySourceRouteErrorCode === "M4_SOURCE_ROUTE_STALE"
      && notices.length === 1
      && m4r04ConsumedMarkerCount() === 0
      && m4r04SuccessNoticeCount() === 0
    ) return;
    await m4r04Delay(100);
  }
  throw new Error("m4r04_stale_ui_timeout");
}

function m4r04ErrorFamily(error: unknown) {
  const message = error instanceof Error ? error.message : String(error);
  const fixedSourceFailureFamilies = new Map<string, string>([
    ["M4_SOURCE_ROUTE_INVALID", "resolver_route_invalid"],
    ["M4_SOURCE_ROUTE_TAMPERED", "resolver_route_tampered"],
    ["M4_SOURCE_OWNER_UNREGISTERED", "resolver_owner_unregistered"],
    ["M4_SOURCE_TYPE_UNREGISTERED", "resolver_type_unregistered"],
    ["M4_SOURCE_SCOPE_MISMATCH", "resolver_scope_mismatch"],
    ["M4_SOURCE_ROUTE_STALE", "resolver_route_stale"],
    ["M4_SOURCE_REVISION_MISMATCH", "resolver_revision_mismatch"],
    ["M4_SOURCE_TARGET_MISSING", "resolver_target_missing"],
    ["M4_SOURCE_TARGET_INTEGRITY_FAILED", "resolver_target_integrity"],
    ["M4_SOURCE_ROUTE_REGISTRY_UNAVAILABLE", "resolver_registry_unavailable"],
    ["M4_SOURCE_ROUTE_RESOLUTION_UNAVAILABLE", "resolver_resolution_unavailable"],
    ["M4_SOURCE_ROUTE_RESPONSE_INVALID", "resolver_response_invalid"],
    ["M4_SOURCE_ROUTE_RESOLUTION_FAILED", "resolver_resolution_failed"],
    ["SECRETARY_SOURCE_TARGET_PROJECT_MISSING", "consumer_project_missing"],
    ["SECRETARY_SOURCE_TARGET_AMBIGUOUS", "consumer_ambiguous"],
    ["SECRETARY_SOURCE_TARGET_RECORD_MISSING", "consumer_record_missing"],
  ]);
  for (const [code, family] of fixedSourceFailureFamilies) {
    if (message.endsWith(`:${code}`)) return family;
  }
  if (message.includes("negative_zero_navigation_invalid")) return "negative_zero_navigation";
  if (message.includes("current_route_refresh_binding_invalid")) return "current_route_binding";
  if (message.includes("focus_pending_timeout")) return "focus_pending_timeout";
  if (message.includes("focus_consumed_contract_timeout")) return "focus_consumed_contract_timeout";
  if (message.includes("focus_consumer_missing_timeout")) return "focus_consumer_missing_timeout";
  if (message.includes("timeout")) return "timeout";
  if (message.includes("m4_secretary_home_")) return "home_read_contract";
  if (message.includes("collision") || message.includes("cardinality")) return "cardinality";
  if (message.includes("prepared")) return "prepared_binding";
  if (message.includes("resolver")) return "resolver_contract";
  if (message.includes("navigation")) return "navigation_contract";
  if (message.includes("source_focus") || message.includes("source_action")) return "dom_contract";
  return "command_rejected";
}

async function m4r04EmitResult(
  invocation: M4R04OrdinaryRouteInvocation,
  outcome: "PASS" | "REJECTED",
  errorFamily?: string,
) {
  const prepared = m4r04PreparedState;
  await emit(M4R04_IPC_RESULT_EVENT, {
    schema_version: M4R04_IPC_SCHEMA_VERSION,
    phase: invocation.phase,
    operation: invocation.operation,
    nonce: invocation.nonce,
    outcome,
    proposal_create_calls: m4r04Counters.proposalCreateCalls,
    work_item_update_calls: m4r04Counters.workItemUpdateCalls,
    route_action_clicks: m4r04Counters.routeActionClicks,
    navigation_clicks: m4r04Counters.navigationClicks,
    refresh_clicks: m4r04Counters.refreshClicks,
    resolver_wrapper_calls: m4r04Counters.resolverWrapperCalls,
    work_item: prepared?.workItem ?? null,
    proposal: prepared?.proposal ?? null,
    current_work_item: invocation.operation === "advance_check_negatives_and_click_current"
      ? m4r04CurrentWorkItem
      : null,
    negative: invocation.operation === "advance_check_negatives_and_click_current"
      ? m4r04Negative
      : null,
    error_family: errorFamily ?? null,
  });
}

let m4r04CurrentWorkItem: M4R04RouteObservation | null = null;
let m4r04Negative: M4R04NegativeObservation | null = null;

async function m4r04RunOperation(invocation: M4R04OrdinaryRouteInvocation) {
  switch (invocation.operation) {
    case "click_work_item_route": {
      await m4r04ClickRefresh();
      const selected = await m4r04WaitForSourceAction(M4R04_WORK_ITEM_OWNER, M4R04_HOME_LINK_TYPE);
      const workItem = await m4r04ClickAndObserveRoute(selected, M4R04_WORK_ITEM_NATIVE_TYPE);
      m4r04PreparedState = {
        nonce: invocation.nonce,
        projectRoot: invocation.project_root,
        workItem,
        proposal: null,
      };
      return;
    }
    case "create_proposal_source": {
      const snapshot = await loadWorkflowStateSnapshot();
      const workflows = snapshot.project_workflows.filter(
        (workflow) => workflow.project_root === invocation.project_root,
      );
      if (workflows.length !== 1) throw new Error("m4r04_workflow_cardinality_invalid");
      const workflow = workflows[0];
      m4r04Counters.proposalCreateCalls += 1;
      const created = await createProjectConsultationProposal({
        project_root: invocation.project_root,
        project_id: workflow.project_id,
        workflow_id: workflow.workflow_id,
        title: "SYN M4R04 ordinary registered owner route",
        user_goal: "Prove exact return from a Secretary source to its registered proposal owner.",
        goal_summary: "Ordinary isolated App registered-owner route evidence.",
        proposed_steps: ["Create the proposal through the product command.", "Open its Secretary source route."],
        scope_draft: {
          allowed_role_ids: ["codex-dev"],
          allowed_agent_ids: [],
          allowed_read_roots: [invocation.project_root],
          allowed_write_roots: [],
          allowed_tools: [],
          allowed_checks: [],
          allowed_task_package_kinds: [],
          stop_conditions: ["Stop after M4R04 route evidence is observed."],
          max_worker_dispatches: 1,
          max_runtime_minutes: 5,
        },
        risks: [],
        worker_acceptance_criteria: ["The proposal owner page consumes the exact registered source."],
        control_core_acceptance_criteria: ["The route remains bound to the sealed owner provenance."],
        supervisor_acceptance_criteria: ["Restart and negative controls preserve the exact owner boundary."],
        acceptance_criteria: ["The registered proposal owner consumes the exact source target."],
        created_by_role: "project_director",
        actor_id: "m4r04-route-driver",
      });
      if (
        created.proposal.project_id !== workflow.project_id
        || created.proposal.workflow_id !== workflow.workflow_id
      ) throw new Error("m4r04_proposal_create_binding_invalid");
      const deliveredRoute = await m4r04WaitForHomeRoute(
        M4R04_PROPOSAL_OWNER,
        created.proposal.proposal_id,
      );
      await m4r04ClickRefresh();
      const selected = await m4r04WaitForSourceAction(
        M4R04_PROPOSAL_OWNER,
        M4R04_HOME_LINK_TYPE,
        created.proposal.proposal_id,
      );
      if (selected.action.source_route_ref !== deliveredRoute.source_route_ref) {
        throw new Error("m4r04_proposal_refresh_route_binding_invalid");
      }
      m4r04PreparedState = {
        nonce: invocation.nonce,
        projectRoot: invocation.project_root,
        workItem: null,
        proposal: m4r04ActionObservation(selected.action, M4R04_PROPOSAL_NATIVE_TYPE),
      };
      return;
    }
    case "click_proposal_route": {
      const prepared = m4r04RequirePrepared(invocation);
      if (!prepared.proposal) throw new Error("m4r04_prepared_proposal_missing");
      const selected = await m4r04WaitForSourceAction(
        prepared.proposal.source_owner_ref,
        M4R04_HOME_LINK_TYPE,
        prepared.proposal.canonical_source_object_id,
      );
      if (selected.action.source_route_ref !== prepared.proposal.source_route_ref) {
        throw new Error("m4r04_prepared_proposal_route_changed");
      }
      prepared.proposal = await m4r04ClickAndObserveRoute(selected, M4R04_PROPOSAL_NATIVE_TYPE);
      return;
    }
    case "click_restart_work_item": {
      await m4r04ClickRefresh();
      const selected = await m4r04WaitForSourceAction(M4R04_WORK_ITEM_OWNER, M4R04_HOME_LINK_TYPE);
      const workItem = await m4r04ClickAndObserveRoute(selected, M4R04_WORK_ITEM_NATIVE_TYPE);
      m4r04PreparedState = {
        nonce: invocation.nonce,
        projectRoot: invocation.project_root,
        workItem,
        proposal: null,
      };
      return;
    }
    case "click_restart_proposal": {
      const prepared = m4r04RequirePrepared(invocation);
      await m4r04NavigateHome();
      await m4r04ClickRefresh();
      const selected = await m4r04WaitForSourceAction(M4R04_PROPOSAL_OWNER, M4R04_HOME_LINK_TYPE);
      prepared.proposal = await m4r04ClickAndObserveRoute(selected, M4R04_PROPOSAL_NATIVE_TYPE);
      return;
    }
    case "advance_check_negatives_and_click_current": {
      const prepared = m4r04RequirePrepared(invocation);
      if (!prepared.workItem || !prepared.proposal) {
        throw new Error("m4r04_restart_prepared_routes_missing");
      }
      await m4r04NavigateHome();
      const staleSelection = await m4r04WaitForSourceAction(
        M4R04_WORK_ITEM_OWNER,
        M4R04_HOME_LINK_TYPE,
        prepared.workItem.canonical_source_object_id,
      );
      if (staleSelection.action.source_route_ref !== prepared.workItem.source_route_ref) {
        throw new Error("m4r04_restart_old_route_changed_before_update");
      }
      m4r04Counters.workItemUpdateCalls += 1;
      await updateWorkItemState({
        project_root: invocation.project_root,
        work_item_id: prepared.workItem.canonical_source_object_id,
        next_state: "running",
        client_request_ref: invocation.nonce,
      });
      // Poll the ordinary server-owned Home read until the replacement route
      // is current, but deliberately leave the rendered old button in place.
      const currentHomeRoute = await m4r04WaitForHomeRoute(
        M4R04_WORK_ITEM_OWNER,
        prepared.workItem.canonical_source_object_id,
        prepared.workItem.source_route_ref,
      );

      const activeViewBefore = m4r04ActiveView();
      const routePhaseBefore = m4r04RoutePhase();
      const consumedBefore = m4r04ConsumedMarkerCount();
      const successBefore = m4r04SuccessNoticeCount();
      const navigationBefore = m4r04Counters.navigationClicks;
      const resolverBefore = m4r04Counters.resolverWrapperCalls;
      await m4r04ClickAndObserveStale(staleSelection);
      const staleCode = "M4_SOURCE_ROUTE_STALE";
      const tamperedCode = await m4r04ExpectedResolverFailure(
        m4r04FlipRouteDigest(currentHomeRoute.source_route_ref),
        "M4_SOURCE_ROUTE_TAMPERED",
      );
      const activeViewAfter = m4r04ActiveView();
      const routePhaseAfter = m4r04RoutePhase();
      const consumedAfter = m4r04ConsumedMarkerCount();
      const successAfter = m4r04SuccessNoticeCount();
      m4r04Negative = {
        stale_error_code: staleCode,
        tampered_error_code: tamperedCode,
        resolver_wrapper_calls: m4r04Counters.resolverWrapperCalls - resolverBefore,
        stale_ui_phase: routePhaseAfter,
        stale_notice_error_code: "M4_SOURCE_ROUTE_STALE",
        stale_route_action_clicks: 1,
        active_view_before: activeViewBefore,
        active_view_after: activeViewAfter,
        route_phase_before: routePhaseBefore,
        route_phase_after: routePhaseAfter,
        consumed_marker_count_before: consumedBefore,
        consumed_marker_count_after: consumedAfter,
        success_notice_count_before: successBefore,
        success_notice_count_after: successAfter,
        zero_navigation: navigationBefore === m4r04Counters.navigationClicks
          && activeViewBefore === activeViewAfter,
        zero_consume_delta: consumedBefore === consumedAfter,
        zero_success_delta: successBefore === successAfter,
      };
      if (
        activeViewBefore !== "home"
        || activeViewAfter !== "home"
        || routePhaseBefore !== "IDLE"
        || routePhaseAfter !== "FAILED"
        || consumedBefore !== 0
        || consumedAfter !== 0
        || successBefore !== 0
        || successAfter !== 0
        || !m4r04Negative.zero_navigation
        || !m4r04Negative.zero_consume_delta
        || !m4r04Negative.zero_success_delta
      ) throw new Error("m4r04_negative_zero_navigation_invalid");
      await m4r04ClickRefresh();
      const current = await m4r04WaitForSourceAction(
        M4R04_WORK_ITEM_OWNER,
        M4R04_HOME_LINK_TYPE,
        prepared.workItem.canonical_source_object_id,
        prepared.workItem.source_route_ref,
      );
      if (current.action.source_route_ref !== currentHomeRoute.source_route_ref) {
        throw new Error("m4r04_current_route_refresh_binding_invalid");
      }
      m4r04CurrentWorkItem = await m4r04ClickAndObserveRoute(
        current,
        M4R04_WORK_ITEM_NATIVE_TYPE,
      );
      return;
    }
  }
}

async function installM4R04OrdinaryRouteTauriIpcBridge() {
  if (!("__TAURI_INTERNALS__" in window)) return;
  try {
    await listen<M4R04OrdinaryRouteInvocation>(M4R04_IPC_INVOKE_EVENT, ({ payload }) => {
      if (!isM4R04Invocation(payload)) return;
      m4r04OperationQueue = m4r04OperationQueue
        .catch(() => undefined)
        .then(async () => {
          try {
            await m4r04RunOperation(payload);
            await m4r04EmitResult(payload, "PASS");
          } catch (error) {
            await m4r04EmitResult(payload, "REJECTED", m4r04ErrorFamily(error));
          }
        })
        .catch(() => undefined);
    });
    await emit(M4R04_IPC_READY_EVENT, {
      schema_version: M4R04_IPC_SCHEMA_VERSION,
      surface: "ordinary_registered_tauri_command_and_dom_click",
      phases: ["work_item", "proposal", "restart_negative"],
    });
  } catch {
    // Rust owns the bounded readiness timeout and terminal receipt.
  }
}

// M4R05 actual-App proof. Product sends still originate from the visible
// composer. The single duplicate request deliberately reuses the second DOM
// turn's client_message_ref through the ordinary typed wrapper; it does not
// expose a test retry control or manufacture renderer state.
const M4R05_IPC_READY_EVENT = "syn-m4r05-ordinary-conversation-ui-ready";
const M4R05_IPC_INVOKE_EVENT = "syn-m4r05-ordinary-conversation-invoke";
const M4R05_IPC_RESULT_EVENT = "syn-m4r05-ordinary-conversation-result";
const M4R05_IPC_SCHEMA_VERSION = "syn_m4r05_ordinary_conversation_ipc.v1";
const M4R05_COMMAND_SURFACE = "ordinary_secretary_conversation_command_and_dom_submit";
const M4R05_CONVERSATION_SCHEMA = "syn.m4.secretary.conversation.v1";
const M4R05_SEND_SCHEMA = "syn.m4.secretary.conversation-send.v1";
const M4R05_DOM_WAIT_MS = 25_000;
const M4R05_MESSAGES = {
  round_one: "SYN M4R05 ordinary conversation round 1",
  round_two: "SYN M4R05 ordinary conversation round 2",
  round_three: "SYN M4R05 ordinary conversation round 3",
  round_four: "SYN M4R05 ordinary conversation round 4",
} as const;

type M4R05Phase = "two_rounds_arm" | "restart_continue_failure";

type M4R05Invocation = {
  schema_version: typeof M4R05_IPC_SCHEMA_VERSION;
  phase: M4R05Phase;
  operation: "run_phase";
  nonce: string;
};

type M4R05DomTurn = {
  turn_ref: string;
  client_message_ref: string;
  state: string;
  user_text: string;
  assistant_text: string | null;
  error_code: string | null;
};

type M4R05DomObservation = {
  role_session_ref: string;
  turn_count: number;
  succeeded_turn_count: number;
  failed_turn_count: number;
  user_message_node_count: number;
  assistant_message_node_count: number;
  pending: boolean;
  turns: M4R05DomTurn[];
};

function m4r05IsInvocation(value: unknown): value is M4R05Invocation {
  if (!value || typeof value !== "object") return false;
  const candidate = value as Partial<M4R05Invocation>;
  return candidate.schema_version === M4R05_IPC_SCHEMA_VERSION
    && (candidate.phase === "two_rounds_arm"
      || candidate.phase === "restart_continue_failure")
    && candidate.operation === "run_phase"
    && typeof candidate.nonce === "string"
    && /^[a-f0-9]{32}$/.test(candidate.nonce);
}

function m4r05BoundedDomText(value: string | undefined, field: string): string {
  if (!value || value.length > 512 || /[\r\n]/.test(value)) {
    throw new Error(`m4r05_${field}_invalid`);
  }
  return value;
}

function m4r05ReadDomConversation(): M4R05DomObservation | null {
  const roots = Array.from(document.querySelectorAll<HTMLElement>(
    '[data-secretary-conversation-state="READY"]',
  ));
  if (roots.length === 0) return null;
  if (roots.length !== 1) throw new Error("m4r05_conversation_root_cardinality");
  const root = roots[0];
  const roleSessionRef = m4r05BoundedDomText(
    root.dataset.secretaryConversationRoleSessionRef,
    "role_session_ref",
  );
  const turnElements = Array.from(root.querySelectorAll<HTMLElement>(
    "[data-secretary-turn-ref]",
  ));
  const turns = turnElements.map((turn): M4R05DomTurn => {
    const userNodes = turn.querySelectorAll<HTMLElement>(
      '[data-secretary-message-role="user"]',
    );
    const assistantNodes = turn.querySelectorAll<HTMLElement>(
      '[data-secretary-message-role="assistant"]',
    );
    if (userNodes.length !== 1 || assistantNodes.length > 1) {
      throw new Error("m4r05_message_node_cardinality");
    }
    const userParagraphs = userNodes[0].querySelectorAll(":scope > p");
    const assistantParagraphs = assistantNodes.length === 1
      ? assistantNodes[0].querySelectorAll(":scope > p")
      : [];
    if (userParagraphs.length !== 1
      || (assistantNodes.length === 1 && assistantParagraphs.length !== 1)) {
      throw new Error("m4r05_message_text_cardinality");
    }
    const errorNode = turn.matches("[data-secretary-conversation-error-code]")
      ? turn
      : turn.querySelector<HTMLElement>("[data-secretary-conversation-error-code]");
    return {
      turn_ref: m4r05BoundedDomText(turn.dataset.secretaryTurnRef, "turn_ref"),
      client_message_ref: m4r05BoundedDomText(
        turn.dataset.secretaryClientMessageRef,
        "client_message_ref",
      ),
      state: m4r05BoundedDomText(turn.dataset.secretaryTurnState, "turn_state"),
      user_text: userParagraphs[0].textContent?.trim() ?? "",
      assistant_text: assistantNodes.length === 1
        ? assistantParagraphs[0].textContent?.trim() ?? ""
        : null,
      error_code: errorNode?.dataset.secretaryConversationErrorCode ?? null,
    };
  });
  const pendingNodes = Array.from(document.querySelectorAll<HTMLElement>(
    "[data-secretary-send-pending]",
  ));
  if (pendingNodes.length !== 1) throw new Error("m4r05_pending_state_cardinality");
  const pendingValue = pendingNodes[0].dataset.secretarySendPending;
  if (pendingValue !== "true" && pendingValue !== "false") {
    throw new Error("m4r05_pending_state_invalid");
  }
  return {
    role_session_ref: roleSessionRef,
    turn_count: turns.length,
    succeeded_turn_count: turns.filter((turn) => turn.state === "SUCCEEDED").length,
    failed_turn_count: turns.filter((turn) => turn.state === "FAILED").length,
    user_message_node_count: turns.length,
    assistant_message_node_count: turns.filter((turn) => turn.assistant_text !== null).length,
    pending: pendingValue === "true",
    turns,
  };
}

function m4r05ConversationMatchesDom(
  conversation: M4SecretaryConversation,
  observation: M4R05DomObservation,
): boolean {
  return conversation.schema_version === M4R05_CONVERSATION_SCHEMA
    && conversation.role_session_ref === observation.role_session_ref
    && conversation.turns.length === observation.turn_count
    && conversation.turns.every((turn, index) => {
      const dom = observation.turns[index];
      return Boolean(dom)
        && turn.turn_ref === dom.turn_ref
        && turn.client_message_ref === dom.client_message_ref
        && turn.state === dom.state
        && turn.user_message.text === dom.user_text
        && (turn.assistant_message?.text ?? null) === dom.assistant_text
        && turn.error_code === dom.error_code;
    });
}

function m4r05ExpectedTurnContract(
  turn: M4SecretaryConversationTurn,
  message: string,
  state: "SUCCEEDED" | "FAILED",
): boolean {
  return turn.user_message.text === message
    && turn.state === state
    && (state === "SUCCEEDED"
      ? turn.assistant_message !== null && turn.error_code === null
      : turn.assistant_message === null
        && turn.error_code === "M4_SECRETARY_PROVIDER_FAILURE");
}

async function m4r05WaitForDom(
  predicate: (observation: M4R05DomObservation) => boolean,
  family: string,
): Promise<M4R05DomObservation> {
  return m4r05WaitForDomUntil(predicate, family, Date.now() + M4R05_DOM_WAIT_MS);
}

async function m4r05WaitForDomUntil(
  predicate: (observation: M4R05DomObservation) => boolean,
  family: string,
  deadline: number,
): Promise<M4R05DomObservation> {
  while (Date.now() < deadline) {
    const observation = m4r05ReadDomConversation();
    if (observation && predicate(observation)) return observation;
    await delayM4R02(100);
  }
  throw new Error(`m4r05_${family}_timeout`);
}

function m4r05SetNativeInputValue(
  input: HTMLInputElement | HTMLTextAreaElement,
  value: string,
) {
  const prototype = input instanceof HTMLTextAreaElement
    ? HTMLTextAreaElement.prototype
    : HTMLInputElement.prototype;
  const setter = Object.getOwnPropertyDescriptor(prototype, "value")?.set;
  if (!setter) throw new Error("m4r05_composer_value_setter_missing");
  setter.call(input, value);
  input.dispatchEvent(new Event("input", { bubbles: true }));
  input.dispatchEvent(new Event("change", { bubbles: true }));
}

async function m4r05DomSend(
  message: string,
  expectedTurnCount: number,
  expectedState: "SUCCEEDED" | "FAILED",
): Promise<M4R05DomObservation> {
  // Submit enablement and the terminal DOM observation share one deadline;
  // they are not two sequential 25-second budgets.
  const sendDeadline = Date.now() + M4R05_DOM_WAIT_MS;
  const inputs = Array.from(document.querySelectorAll<HTMLInputElement | HTMLTextAreaElement>(
    '[data-secretary-composer="true"]',
  ));
  const submits = Array.from(document.querySelectorAll<HTMLButtonElement>(
    '[data-secretary-send="true"]',
  ));
  if (inputs.length !== 1 || submits.length !== 1) {
    throw new Error("m4r05_composer_cardinality");
  }
  m4r05SetNativeInputValue(inputs[0], message);
  while (submits[0].disabled && Date.now() < sendDeadline) {
    await delayM4R02(50);
  }
  if (submits[0].disabled) throw new Error("m4r05_submit_disabled_timeout");
  submits[0].click();
  return m4r05WaitForDomUntil(
    (observation) => {
      const turn = observation.turns.at(-1);
      return !observation.pending
        && observation.turn_count === expectedTurnCount
        && Boolean(turn)
        && turn?.user_text === message
        && turn?.state === expectedState;
    },
    `round_${expectedTurnCount}_${expectedState.toLowerCase()}`,
    sendDeadline,
  );
}

function m4r05RequireConversationContract(
  conversation: M4SecretaryConversation,
  observation: M4R05DomObservation,
  expectedMessages: readonly string[],
  expectedStates: readonly ("SUCCEEDED" | "FAILED")[],
) {
  if (!m4r05ConversationMatchesDom(conversation, observation)
    || conversation.turns.length !== expectedMessages.length
    || observation.pending
    || !conversation.turns.every((turn, index) => m4r05ExpectedTurnContract(
      turn,
      expectedMessages[index],
      expectedStates[index],
    ))) {
    throw new Error("m4r05_conversation_dom_contract_invalid");
  }
}

async function m4r05RunPhase(invocation: M4R05Invocation) {
  const controlsDeadline = Date.now() + M4R05_DOM_WAIT_MS;
  let homeControls: {
    input: HTMLInputElement | HTMLTextAreaElement;
    submit: HTMLButtonElement;
    open: HTMLButtonElement;
  } | null = null;
  while (Date.now() < controlsDeadline) {
    const inputs = Array.from(document.querySelectorAll<HTMLInputElement | HTMLTextAreaElement>(
      '[data-secretary-composer="true"]',
    ));
    const submits = Array.from(document.querySelectorAll<HTMLButtonElement>(
      '[data-secretary-send="true"]',
    ));
    const opens = Array.from(document.querySelectorAll<HTMLButtonElement>(
      '[data-secretary-open-conversation="true"]',
    ));
    if (inputs.length === 1 && submits.length === 1 && opens.length === 1) {
      homeControls = { input: inputs[0], submit: submits[0], open: opens[0] };
      break;
    }
    if (inputs.length > 1 || submits.length > 1 || opens.length > 1) {
      throw new Error("m4r05_home_conversation_controls_cardinality");
    }
    await delayM4R02(100);
  }
  if (!homeControls) throw new Error("m4r05_home_conversation_controls_timeout");
  const blankSubmitDisabled = homeControls.input.value === ""
    && homeControls.submit.disabled;
  if (!blankSubmitDisabled) throw new Error("m4r05_blank_submit_not_disabled");
  homeControls.open.click();
  const expectedInitialCount = invocation.phase === "two_rounds_arm" ? 0 : 2;
  const initialDom = await m4r05WaitForDom(
    (observation) => !observation.pending && observation.turn_count === expectedInitialCount,
    "initial_dom",
  );
  const initialConversation = await loadSecretaryConversation();
  const initialMessages = invocation.phase === "two_rounds_arm"
    ? []
    : [M4R05_MESSAGES.round_one, M4R05_MESSAGES.round_two];
  m4r05RequireConversationContract(
    initialConversation,
    initialDom,
    initialMessages,
    initialMessages.map(() => "SUCCEEDED" as const),
  );

  if (invocation.phase === "two_rounds_arm") {
    await m4r05DomSend(M4R05_MESSAGES.round_one, 1, "SUCCEEDED");
    const secondDom = await m4r05DomSend(M4R05_MESSAGES.round_two, 2, "SUCCEEDED");
    const beforeReplay = await loadSecretaryConversation();
    m4r05RequireConversationContract(
      beforeReplay,
      secondDom,
      [M4R05_MESSAGES.round_one, M4R05_MESSAGES.round_two],
      ["SUCCEEDED", "SUCCEEDED"],
    );
    const secondTurn = beforeReplay.turns[1];
    const replay = await sendSecretaryMessage({
      message: secondTurn.user_message.text,
      client_message_ref: secondTurn.client_message_ref,
    });
    if (replay.schema_version !== M4R05_SEND_SCHEMA
      || !replay.replayed
      || replay.turn_ref !== secondTurn.turn_ref
      || replay.conversation.turns.length !== 2) {
      throw new Error("m4r05_exact_replay_contract_invalid");
    }
    const finalDom = await m4r05WaitForDom(
      (observation) => !observation.pending && observation.turn_count === 2,
      "replay_zero_delta",
    );
    const finalConversation = await loadSecretaryConversation();
    m4r05RequireConversationContract(
      finalConversation,
      finalDom,
      [M4R05_MESSAGES.round_one, M4R05_MESSAGES.round_two],
      ["SUCCEEDED", "SUCCEEDED"],
    );
    if (JSON.stringify(replay.conversation) !== JSON.stringify(finalConversation)) {
      throw new Error("m4r05_exact_replay_readback_mismatch");
    }
    return {
      initial_conversation: initialConversation,
      initial_dom: initialDom,
      final_conversation: finalConversation,
      final_dom: finalDom,
      replay,
      dom_submit_clicks: 2,
      bridge_load_calls: 3,
      bridge_exact_replay_send_calls: 1,
      open_conversation_clicks: 1,
      blank_submit_disabled: blankSubmitDisabled,
    };
  }

  await m4r05DomSend(M4R05_MESSAGES.round_three, 3, "SUCCEEDED");
  const fourthDom = await m4r05DomSend(M4R05_MESSAGES.round_four, 4, "FAILED");
  const finalConversation = await loadSecretaryConversation();
  m4r05RequireConversationContract(
    finalConversation,
    fourthDom,
    [
      M4R05_MESSAGES.round_one,
      M4R05_MESSAGES.round_two,
      M4R05_MESSAGES.round_three,
      M4R05_MESSAGES.round_four,
    ],
    ["SUCCEEDED", "SUCCEEDED", "SUCCEEDED", "FAILED"],
  );
  return {
    initial_conversation: initialConversation,
    initial_dom: initialDom,
    final_conversation: finalConversation,
    final_dom: fourthDom,
    replay: null,
    dom_submit_clicks: 2,
    bridge_load_calls: 2,
    bridge_exact_replay_send_calls: 0,
    open_conversation_clicks: 1,
    blank_submit_disabled: blankSubmitDisabled,
  };
}

function m4r05ErrorFamily(error: unknown): string {
  const message = error instanceof Error ? error.message : String(error);
  const match = message.match(/m4r05_([a-z0-9_]{1,80})/);
  return match?.[1] ?? "renderer_rejected";
}

async function installM4R05OrdinaryConversationTauriIpcBridge() {
  if (!("__TAURI_INTERNALS__" in window)) return;
  try {
    await listen<M4R05Invocation>(M4R05_IPC_INVOKE_EVENT, async ({ payload }) => {
      if (!m4r05IsInvocation(payload)) return;
      try {
        const evidence = await m4r05RunPhase(payload);
        await emit(M4R05_IPC_RESULT_EVENT, {
          schema_version: M4R05_IPC_SCHEMA_VERSION,
          phase: payload.phase,
          operation: payload.operation,
          nonce: payload.nonce,
          outcome: "PASS",
          ...evidence,
          error_family: null,
        });
      } catch (error) {
        await emit(M4R05_IPC_RESULT_EVENT, {
          schema_version: M4R05_IPC_SCHEMA_VERSION,
          phase: payload.phase,
          operation: payload.operation,
          nonce: payload.nonce,
          outcome: "REJECTED",
          initial_conversation: null,
          initial_dom: null,
          final_conversation: null,
          final_dom: null,
          replay: null,
          dom_submit_clicks: null,
          bridge_load_calls: null,
          bridge_exact_replay_send_calls: null,
          open_conversation_clicks: null,
          blank_submit_disabled: null,
          error_family: m4r05ErrorFamily(error),
        });
      }
    });
    await emit(M4R05_IPC_READY_EVENT, {
      schema_version: M4R05_IPC_SCHEMA_VERSION,
      surface: M4R05_COMMAND_SURFACE,
      phases: ["two_rounds_arm", "restart_continue_failure"],
    });
  } catch {
    // Rust owns the bounded readiness timeout and terminal receipt.
  }
}

// M4R06 observes the ordinary App's own guarded Home fallback in one isolated
// debug launch. The backend supplies its fixed existing UNAVAILABLE envelope;
// this bridge only opens the rendered Board and records DOM facts. It never
// invents a Home result or reads the compatibility report for that UI phase.
const M4R06_IPC_READY_EVENT = "syn-m4r06-ordinary-legacy-read-ui-ready";
const M4R06_IPC_INVOKE_EVENT = "syn-m4r06-ordinary-legacy-read-invoke";
const M4R06_IPC_RESULT_EVENT = "syn-m4r06-ordinary-legacy-read-result";
const M4R06_IPC_SCHEMA_VERSION = "syn_m4r06_ordinary_legacy_read_ipc.v1";
const M4R06_COMMAND_SURFACE =
  "ordinary_zero_arg_load_secretary_legacy_read_compatibility_report_ipc";
const M4R06_DOM_WAIT_MS = 20_000;

type M4R06OrdinaryLegacyReadInvocation = {
  schema_version: typeof M4R06_IPC_SCHEMA_VERSION;
  phase: "read_and_replay";
  operation: "ui_fallback" | "first_read" | "exact_replay";
  nonce: string;
  // This is an upper-level R07 observer flag only. It is emitted by the
  // existing R06 driver and never crosses a product command boundary.
  r07_closeout_mode?: true;
};

type M4R06UiFallbackEvidence = {
  open_conversation_clicks: number;
  compatibility_fallback_roots: number;
  parity_primary_attention_rows: number;
  non_parity_rows_visible: number;
  source_route_controls: number;
  nested_summary_source_route_controls: number;
  board_coordination_action_controls: number;
  board_personal_action_controls: number;
  source_route_clicks: number;
  // These raw route/display references cross only this short-lived IPC event.
  // Rust joins them to the server report and serializes hashes only.
  source_route_ref: string;
  source_owner_ref: string;
  source_object_type: string;
  canonical_source_object_id: string;
  // R07-only route consumption observations. They remain ephemeral IPC data;
  // Rust joins them to server-owned records and stores only scrubbed evidence.
  consumed_marker_count?: number;
  success_notice_count?: number;
  active_view?: string;
  route_phase?: string;
  consumed_source_revision?: string;
};

function isM4R06Invocation(value: unknown): value is M4R06OrdinaryLegacyReadInvocation {
  if (!value || typeof value !== "object") return false;
  const candidate = value as Partial<M4R06OrdinaryLegacyReadInvocation>;
  return candidate.schema_version === M4R06_IPC_SCHEMA_VERSION
    && candidate.phase === "read_and_replay"
    && (candidate.operation === "ui_fallback"
      || candidate.operation === "first_read"
      || candidate.operation === "exact_replay")
    && typeof candidate.nonce === "string"
    && (candidate.r07_closeout_mode === undefined || candidate.r07_closeout_mode === true)
    && /^[a-f0-9]{32}$/.test(candidate.nonce);
}

function m4r06ErrorFamily(error: unknown): string {
  const message = error instanceof Error ? error.message : String(error);
  if (message.includes("ui_fallback")) return "ui_fallback_rejected";
  if (message.includes("daily")) return "daily_report_rejected";
  if (message.includes("legacy_read")) return "legacy_read_rejected";
  if (message.includes("report_not_ready") || message.includes("READY")) return "report_not_ready";
  return "renderer_rejected";
}

function m4r06Delay(milliseconds: number) {
  return new Promise<void>((resolve) => window.setTimeout(resolve, milliseconds));
}

function m4r06IsVisibleConnected(element: Element): element is HTMLElement {
  if (!(element instanceof HTMLElement) || !element.isConnected) return false;
  const style = window.getComputedStyle(element);
  const rectangle = element.getBoundingClientRect();
  return style.display !== "none"
    && style.visibility !== "hidden"
    && rectangle.width > 0
    && rectangle.height > 0;
}

async function m4r06WaitForUniqueVisibleElement(
  scope: ParentNode,
  selector: string,
  deadline: number,
  family: string,
  predicate: (element: HTMLElement) => boolean = () => true,
): Promise<HTMLElement> {
  while (Date.now() < deadline) {
    const matches = Array.from(scope.querySelectorAll<HTMLElement>(selector));
    if (matches.length > 1) throw new Error(`${family}_cardinality`);
    if (matches.length === 1 && m4r06IsVisibleConnected(matches[0]) && predicate(matches[0])) {
      return matches[0];
    }
    await m4r06Delay(50);
  }
  const matches = Array.from(scope.querySelectorAll<HTMLElement>(selector));
  if (matches.length !== 1) throw new Error(`${family}_cardinality`);
  throw new Error(`${family}_not_visible_or_not_ready`);
}

async function m4r06ObserveActualUiFallback(
  r07CloseoutMode: boolean,
): Promise<M4R06UiFallbackEvidence> {
  // This is one shared deadline: first perform the real navigation click,
  // then wait for the already-triggered guarded read to render the Board.
  const deadline = Date.now() + M4R06_DOM_WAIT_MS;
  const openControl = await m4r06WaitForUniqueVisibleElement(
    document,
    '[data-secretary-open-conversation="true"]',
    deadline,
    "m4r06_open_conversation",
  );
  if (!(openControl instanceof HTMLButtonElement) || openControl.disabled) {
    throw new Error("m4r06_open_conversation_not_interactive");
  }
  openControl.click();

  const board = await m4r06WaitForUniqueVisibleElement(
    document,
    '[data-secretary-compatibility-fallback="true"]',
    deadline,
    "m4r06_compatibility_fallback",
    (element) => element.dataset.secretaryBoardState === "degraded",
  );
  const routeControls = Array.from(
    board.querySelectorAll<HTMLElement>('[data-secretary-source-route-action="OPEN"]'),
  );
  if (routeControls.length !== 1) throw new Error("m4r06_source_route_cardinality");
  const routeControl = routeControls[0];
  if (!(routeControl instanceof HTMLButtonElement)
    || routeControl.disabled
    || !m4r06IsVisibleConnected(routeControl)) {
    throw new Error("m4r06_source_route_not_interactive");
  }
  const attentionRows = Array.from(
    board.querySelectorAll<HTMLElement>(".secretary-board-attention-row"),
  );
  if (attentionRows.length !== 1 || !m4r06IsVisibleConnected(attentionRows[0])) {
    throw new Error("m4r06_guarded_attention_row_contract_invalid");
  }
  const sourceRouteRef = routeControl.dataset.secretarySourceRouteRef ?? "";
  const sourceOwnerRef = routeControl.dataset.secretarySourceOwner ?? "";
  const sourceObjectType = routeControl.dataset.secretarySourceObjectType ?? "";
  const canonicalSourceObjectId = routeControl.dataset.secretarySourceObjectId ?? "";
  if (!sourceRouteRef || !sourceOwnerRef || !sourceObjectType || !canonicalSourceObjectId) {
    throw new Error("m4r06_source_route_display_tuple_missing");
  }
  let sourceRouteClicks = 0;
  const observeRouteClick = () => { sourceRouteClicks += 1; };
  routeControl.addEventListener("click", observeRouteClick);
  let r07CloseoutEvidence: Pick<
    M4R06UiFallbackEvidence,
    "consumed_marker_count"
    | "success_notice_count"
    | "active_view"
    | "route_phase"
    | "consumed_source_revision"
  > | null = null;
  try {
    if (!r07CloseoutMode) {
      // Do not dispatch the source route: archived R06 proves it is visible
      // and bound while keeping the guarded board read-only.
      await m4r06Delay(0);
    } else {
      if (sourceObjectType !== M4R04_WORK_ITEM_NATIVE_TYPE) {
        throw new Error("m4r06_closeout_source_type_invalid");
      }
      const consumed = await m4r04ClickAndObserveRoute({
        action: {
          source_owner_ref: sourceOwnerRef,
          source_object_type: sourceObjectType,
          canonical_source_object_id: canonicalSourceObjectId,
          source_route_ref: sourceRouteRef,
          source_action_dom_count: 1,
        },
        button: routeControl,
      }, M4R04_WORK_ITEM_NATIVE_TYPE);
      if (
        consumed.source_owner_ref !== sourceOwnerRef
        || consumed.source_object_type !== sourceObjectType
        || consumed.canonical_source_object_id !== canonicalSourceObjectId
        || consumed.source_route_ref !== sourceRouteRef
        || consumed.source_revision === null
      ) {
        throw new Error("m4r06_closeout_consumed_tuple_changed");
      }
      r07CloseoutEvidence = {
        consumed_marker_count: consumed.consumed_marker_count,
        success_notice_count: consumed.success_notice_count,
        active_view: consumed.active_view,
        route_phase: consumed.route_phase,
        consumed_source_revision: consumed.source_revision,
      };
    }
  } finally {
    routeControl.removeEventListener("click", observeRouteClick);
  }
  const boardCoordinationActionControls = board.querySelectorAll(
    "[data-secretary-action]",
  ).length;
  const boardPersonalActionControls = board.querySelectorAll(
    "[data-secretary-personal-action]",
  ).length;
  const nestedSummarySourceRouteControls = board.querySelectorAll(
    "button.secretary-brief-source-link",
  ).length;
  if (boardCoordinationActionControls !== 0
    || boardPersonalActionControls !== 0
    || nestedSummarySourceRouteControls !== 0) {
    throw new Error("m4r06_guarded_board_control_visibility_invalid");
  }
  return {
    open_conversation_clicks: 1,
    compatibility_fallback_roots: 1,
    // The guarded renderer leaves exactly its PARITY + PRIMARY WorkItem row
    // visible; a second attention row would expose a non-eligible candidate.
    parity_primary_attention_rows: 1,
    non_parity_rows_visible: 0,
    source_route_controls: 1,
    nested_summary_source_route_controls: nestedSummarySourceRouteControls,
    board_coordination_action_controls: boardCoordinationActionControls,
    board_personal_action_controls: boardPersonalActionControls,
    source_route_clicks: sourceRouteClicks,
    source_route_ref: sourceRouteRef,
    source_owner_ref: sourceOwnerRef,
    source_object_type: sourceObjectType,
    canonical_source_object_id: canonicalSourceObjectId,
    ...(r07CloseoutEvidence ?? {}),
  };
}

async function installM4R06OrdinaryLegacyReadTauriIpcBridge() {
  if (!("__TAURI_INTERNALS__" in window)) return;
  try {
    await listen<M4R06OrdinaryLegacyReadInvocation>(M4R06_IPC_INVOKE_EVENT, async ({ payload }) => {
      if (!isM4R06Invocation(payload)) return;
      let zeroArgLoadCalls = 0;
      let dailyReportLoadCalls = 0;
      const r07CloseoutMode = payload.r07_closeout_mode === true;
      try {
        if (payload.operation === "ui_fallback") {
          const uiFallbackEvidence = await m4r06ObserveActualUiFallback(r07CloseoutMode);
          await emit(M4R06_IPC_RESULT_EVENT, {
            schema_version: M4R06_IPC_SCHEMA_VERSION,
            phase: payload.phase,
            operation: payload.operation,
            nonce: payload.nonce,
            outcome: "PASS",
            zero_arg_load_calls: zeroArgLoadCalls,
            report: null,
            ui_fallback_evidence: uiFallbackEvidence,
            error_family: null,
          });
          return;
        }
        if (r07CloseoutMode) {
          dailyReportLoadCalls += 1;
          const envelope = await loadSecretaryDailyReport();
          if (envelope.status !== "READY") throw new Error("m4r06_daily_report_not_ready");
          await emit(M4R06_IPC_RESULT_EVENT, {
            schema_version: M4R06_IPC_SCHEMA_VERSION,
            phase: payload.phase,
            operation: payload.operation,
            nonce: payload.nonce,
            outcome: "PASS",
            zero_arg_load_calls: zeroArgLoadCalls,
            report: null,
            ui_fallback_evidence: null,
            error_family: null,
            daily_report_load_calls: dailyReportLoadCalls,
            daily_report: envelope,
          });
          return;
        }
        zeroArgLoadCalls += 1;
        const envelope = await loadSecretaryLegacyReadCompatibilityReport();
        if (envelope.status !== "READY") throw new Error("m4r06_report_not_ready");
        await emit(M4R06_IPC_RESULT_EVENT, {
          schema_version: M4R06_IPC_SCHEMA_VERSION,
          phase: payload.phase,
          operation: payload.operation,
          nonce: payload.nonce,
          outcome: "PASS",
          zero_arg_load_calls: zeroArgLoadCalls,
          report: envelope.report,
          ui_fallback_evidence: null,
          error_family: null,
        });
      } catch (error) {
        await emit(M4R06_IPC_RESULT_EVENT, {
          schema_version: M4R06_IPC_SCHEMA_VERSION,
          phase: payload.phase,
          operation: payload.operation,
          nonce: payload.nonce,
          outcome: "REJECTED",
          zero_arg_load_calls: zeroArgLoadCalls,
          report: null,
          ui_fallback_evidence: null,
          error_family: m4r06ErrorFamily(error),
          ...(r07CloseoutMode
            ? {
              daily_report_load_calls: dailyReportLoadCalls,
              daily_report: null,
            }
            : {}),
        });
      }
    });
    await emit(M4R06_IPC_READY_EVENT, {
      schema_version: M4R06_IPC_SCHEMA_VERSION,
      surface: M4R06_COMMAND_SURFACE,
      operations: ["ui_fallback", "first_read", "exact_replay"],
    });
  } catch {
    // Rust owns bounded readiness and terminal receipt publication.
  }
}

class BootErrorBoundary extends React.Component<BootErrorBoundaryProps, BootErrorBoundaryState> {
  state: BootErrorBoundaryState = { error: null };

  static getDerivedStateFromError(error: Error): BootErrorBoundaryState {
    return { error };
  }

  componentDidCatch(error: Error) {
    if (!bootProbeEnabled) return;

    void setTauriWindowTitle(`Codex 治理工作台 · 首屏错误：${error.message || "未知错误"}`.slice(0, 80)).then((available) => {
      if (!available) {
        document.documentElement.dataset.bootErrorTitleProbe = "unavailable";
      }
    });
  }

  render() {
    if (this.state.error) {
      return (
        <div className="boot-diagnostic-shell" role="alert">
          <strong>工作台启动失败</strong>
          <span>前端已加载，但首屏渲染遇到错误。请查看开发者诊断。</span>
          <code>{this.state.error.message || "未知错误"}</code>
        </div>
      );
    }

    return this.props.children;
  }
}

function m5r07Delay(ms: number) {
  return new Promise<void>((resolve) => window.setTimeout(resolve, ms));
}

async function m5r07WaitFor<T>(label: string, pick: () => T | null | undefined, timeoutMs = 30_000): Promise<T> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const value = pick();
    if (value) return value;
    await m5r07Delay(150);
  }
  throw new Error(`m5r07_${label}_timeout`);
}

function m5r07Panel() {
  return document.querySelector<HTMLElement>("[data-m5-supervisor-panel]");
}

async function m5r07Click(selector: string) {
  const button = await m5r07WaitFor(selector, () => {
    const found = document.querySelector<HTMLButtonElement>(selector);
    return found && !found.disabled ? found : null;
  });
  button.click();
  await m5r07Delay(200);
}

async function m5r07OpenSupervisorPanel() {
  const projectNav = await m5r07WaitFor("projects_nav", () => {
    return Array.from(document.querySelectorAll<HTMLButtonElement>(".nav-item")).find(
      (button) => button.title === "项目" || button.textContent?.includes("项目"),
    ) ?? null;
  });
  projectNav.click();
  await m5r07Delay(400);
  const tile = await m5r07WaitFor("project_tile", () => document.querySelector<HTMLButtonElement>(".project-tile"));
  tile.click();
  await m5r07Delay(400);
  const overview = await m5r07WaitFor("overview_tab", () => {
    return Array.from(document.querySelectorAll<HTMLButtonElement>(".project-tool-tabs button")).find(
      (button) => button.textContent?.includes("总览") || button.title?.includes("项目总览"),
    ) ?? null;
  });
  overview.click();
  await m5r07WaitFor("supervisor_open", () => {
    const panel = m5r07Panel();
    return panel?.dataset.m5SessionStatus === "open" ? panel : null;
  });
}

async function m5r07FillChat(text: string) {
  const input = await m5r07WaitFor(
    "supervisor_input",
    () => document.querySelector<HTMLTextAreaElement>("[data-m5-supervisor-input]"),
  );
  const setter = Object.getOwnPropertyDescriptor(window.HTMLTextAreaElement.prototype, "value")?.set;
  setter?.call(input, text);
  input.dispatchEvent(new Event("input", { bubbles: true }));
  await m5r07Delay(100);
}

async function installM5R07IsolatedAcceptanceDriver() {
  try {
    const status = await loadM5IsolatedAcceptanceStatus();
    if (!status.isolated) return;
    document.documentElement.dataset.m5r07Isolated = "1";
    document.documentElement.dataset.m5r07Scene = status.scene;
    document.documentElement.dataset.m5r07OpenAvailable = status.open_available ? "1" : "0";
    if (!status.open_available) {
      await writeM5IsolatedUiReceipt("unavailable");
      return;
    }
    await m5r07WaitFor("app_shell", () => document.querySelector(".app-shell"));
    await m5r07OpenSupervisorPanel();
    const panel = m5r07Panel();
    if (!panel) throw new Error("m5r07_panel_missing");
    const bindingId = panel.dataset.m5BindingId ?? "";
    const roleSessionId = panel.dataset.m5RoleSessionId ?? "";
    const projectId = panel.dataset.m5ProjectId ?? status.project_id;
    if (!bindingId || !roleSessionId) throw new Error("m5r07_binding_missing");

    if (status.scene === "resume") {
      await writeM5IsolatedUiReceipt("resume");
      return;
    }

    await m5r07FillChat("what is open?");
    await m5r07Click('[data-m5-action="chat"]');
    await m5r07FillChat("do not run");
    await m5r07Click('[data-m5-action="propose"]');
    await m5r07WaitFor("proposal", () => {
      const next = m5r07Panel();
      return next?.dataset.m5ProposalId ? next : null;
    });
    await m5r07Click('[data-m5-action="reject"]');
    await writeM5IsolatedUiReceipt("scene-a");

    if (status.scene === "a") return;

    await m5r07FillChat("echo hello");
    await m5r07Click('[data-m5-action="propose"]');
    await m5r07WaitFor("scene_b_proposal", () => {
      const next = m5r07Panel();
      return next?.dataset.m5ProposalId ? next : null;
    });
    await m5r07Click('[data-m5-action="approve"]');
    await m5r07WaitFor("approved_grant", () => {
      const next = m5r07Panel();
      return next?.dataset.m5GrantId && next.dataset.m5DispatchId ? next : null;
    });
    await m5r07Click('[data-m5-action="runtime"]');
    await m5r07WaitFor("runtime_log", () => {
      const log = document.querySelector("[data-m5-supervisor-log]");
      return log?.textContent?.includes("runtime") ? log : null;
    });
    await m5r07Click('[data-m5-action="report"]');
    await m5r07WaitFor("report_log", () => {
      const log = document.querySelector("[data-m5-supervisor-log]");
      return log?.textContent?.includes("report") ? log : null;
    });
    await m5r07Click('[data-m5-action="review"]');
    await m5r07WaitFor("review_log", () => {
      const log = document.querySelector("[data-m5-supervisor-log]");
      return log?.textContent?.includes("review") ? log : null;
    });
    await m5r07Click('[data-m5-action="result"]');
    await m5r07WaitFor("result_log", () => {
      const log = document.querySelector("[data-m5-supervisor-log]");
      return log?.textContent?.includes("result") ? log : null;
    });
    await m5r07Click('[data-m5-action="summary"]');
    await m5r07WaitFor("summary", () => document.querySelector("[data-m5-summary-stale]"));
    await m5r07Click('[data-m5-action="advice"]');
    await m5r07WaitFor("advice", () => {
      const next = m5r07Panel();
      return next?.dataset.m5AdviceWritable === "false" ? next : null;
    });
    const deepLinkButton = document.querySelector<HTMLButtonElement>("[data-m5-deep-link-source]");
    deepLinkButton?.click();
    await m5r07WaitFor("deep_link", () => {
      const next = m5r07Panel();
      return next?.dataset.m5DeepLink?.startsWith("syn://") ? next : null;
    });
    await writeM5IsolatedUiReceipt("scene-b");
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    document.documentElement.dataset.m5r07Error = message;
    try {
      await writeM5IsolatedUiReceipt("failed");
    } catch {
      // Isolated receipt write is best-effort after a driver failure.
    }
  }
}

function renderBootFailure(message: string) {
  document.body.innerHTML = `<div class="boot-diagnostic-shell" role="alert"><strong>工作台启动失败</strong><span>${message}</span></div>`;
}

function mountVisibleBootProbe() {
  if (!bootProbeEnabled) return;
  if (document.getElementById(BOOT_VISIBLE_PROBE_ID)) return;
  const probe = document.createElement("div");
  probe.id = BOOT_VISIBLE_PROBE_ID;
  probe.className = "boot-visible-probe";
  probe.textContent = "启动诊断：前端脚本已运行";
  document.body.appendChild(probe);
}

async function markFrontendLoaded() {
  if (!bootProbeEnabled) return;

  document.documentElement.dataset.frontendBoot = "loaded";

  const titleUpdated = await setTauriWindowTitle("Codex 治理工作台 · 前端已加载");
  if (!titleUpdated) {
    document.documentElement.dataset.frontendTitleProbe = "unavailable";
  }
}

mountVisibleBootProbe();
void markFrontendLoaded();
void installM2R4TauriIpcBridge();
void installM4R02OrdinaryCompositionTauriIpcBridge();
void installM4R03OrdinaryClockTauriIpcBridge();
void installM4R04OrdinaryRouteTauriIpcBridge();
void installM4R05OrdinaryConversationTauriIpcBridge();
void installM4R06OrdinaryLegacyReadTauriIpcBridge();
void installM5R07IsolatedAcceptanceDriver();

window.addEventListener("error", (event) => {
  const root = document.getElementById("root");
  if (root?.childElementCount) return;
  renderBootFailure(event.message || "页面脚本加载失败。");
});

window.addEventListener("unhandledrejection", (event) => {
  const root = document.getElementById("root");
  if (root?.childElementCount) return;
  renderBootFailure(event.reason instanceof Error ? event.reason.message : "页面异步启动失败。");
});

const root = document.getElementById("root");

if (!root) {
  renderBootFailure("页面缺少 root 挂载点。");
} else {
  ReactDOM.createRoot(root).render(
    <React.StrictMode>
      <BootErrorBoundary>
        <App />
      </BootErrorBoundary>
    </React.StrictMode>,
  );
}
