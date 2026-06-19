import React from "react";
import ReactDOM from "react-dom/client";
import { App } from "./App";
import { setTauriWindowTitle } from "./lib/tauriWindow";
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
