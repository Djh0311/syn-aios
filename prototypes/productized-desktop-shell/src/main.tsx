import React from "react";
import ReactDOM from "react-dom/client";
import { emit, listen } from "@tauri-apps/api/event";
import { App } from "./App";
import {
  bootstrapProjectWorkflow,
  createProjectConsultationProposal,
  createTaskDraft,
  initializeWorkflowState,
  loadSecretaryHomeContext,
  loadWorkflowStateSnapshot,
  operateSecretaryCoordination,
  operateSecretaryPersonalObject,
  resolveSecretarySourceRoute,
  updateWorkItemState,
} from "./lib/tauri";
import { mintSecretaryCoordinationIdempotencyKey } from "./lib/secretaryReadModel";
import { setTauriWindowTitle } from "./lib/tauriWindow";
import type { WorkItemStateUpdateRequest } from "./lib/types/workflow";
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
// Leave the launcher enough room to observe the arm receipt and SIGKILL the
// exact bundled process while both user-scheduled objects are still pre-due.
const M4R03_STARTUP_DUE_DELAY_MS = 45_000;
// The two ordinary snooze commands are serialized. Keep their shared marker
// beyond the complete operation window so a production tick cannot split them.
const M4R03_TIMER_DUE_DELAY_MS = 30_000;
const M4R03_HOME_READ_TIMEOUT_MS = 15_000;

type M4R03Phase = "arm" | "recovery_timer" | "repeat";
type M4R03Operation =
  | "arm_startup_recovery"
  | "observe_startup_recovery"
  | "arm_timer_tick"
  | "observe_timer_tick"
  | "observe_repeat";

type M4R03OrdinaryClockInvocation = {
  schema_version: typeof M4R03_IPC_SCHEMA_VERSION;
  phase: M4R03Phase;
  operation: M4R03Operation;
  nonce: string;
  startup_due_marker_utc: string | null;
  timer_due_marker_utc: string | null;
};

type M4R03PreparedObjects = {
  openLoopId: string;
  reminderId: string;
  startupDueMarkerUtc: string;
  timerDueMarkerUtc: string | null;
};

let m4r03PreparedObjects: M4R03PreparedObjects | null = null;
let m4r03OperationQueue: Promise<void> = Promise.resolve();
let m4r03WriteCommandsInvoked = 0;

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
    candidate.schema_version !== M4R03_IPC_SCHEMA_VERSION
    || typeof candidate.nonce !== "string"
    || !/^[a-f0-9]{32}$/.test(candidate.nonce)
  ) return false;
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

function m4r03ErrorFamily(error: unknown) {
  const message = error instanceof Error ? error.message : String(error);
  if (message.includes("state_read_timeout")) return "state_read_timeout";
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
    schema_version: M4R03_IPC_SCHEMA_VERSION,
    phase: invocation.phase,
    operation: invocation.operation,
    nonce: invocation.nonce,
    ...result,
  });
}

async function runM4R03OrdinaryClockOperation(
  invocation: M4R03OrdinaryClockInvocation,
) {
  m4r03WriteCommandsInvoked = 0;
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
