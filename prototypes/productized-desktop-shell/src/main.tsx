import React from "react";
import ReactDOM from "react-dom/client";
import { emit, listen } from "@tauri-apps/api/event";
import { App } from "./App";
import { updateWorkItemState } from "./lib/tauri";
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
