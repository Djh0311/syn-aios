import { useCallback, useEffect, useState } from "react";
import {
  loadAgentM3C07AcceptanceStatus,
  loadJiaobanM3C07AcceptanceStatus,
  operateAgentM3C07Acceptance,
  operateJiaobanM3C07Acceptance,
  type M3C07AcceptanceAction,
  type M3C07AcceptanceStatus,
} from "../../lib/tauri";

export type M3AcceptancePanelHost = "agent" | "jiaoban";

export type M3AcceptancePanelClient = Readonly<{
  load: (host: M3AcceptancePanelHost) => Promise<M3C07AcceptanceStatus>;
  operate: (
    host: M3AcceptancePanelHost,
    action: M3C07AcceptanceAction,
    requestNonce: string,
  ) => Promise<M3C07AcceptanceStatus>;
}>;

type M3AcceptancePanelProps = {
  host: M3AcceptancePanelHost;
  initialStatus?: M3C07AcceptanceStatus | null;
  initialError?: string | null;
  client?: M3AcceptancePanelClient;
};

function requestNonce(host: M3AcceptancePanelHost, action: M3C07AcceptanceAction): string {
  return `m3c07-ui:${host}:${action}:${Date.now().toString(36)}`;
}

function unavailable(error: unknown): string {
  const text = error instanceof Error ? error.message : String(error);
  return text.includes("M3_BINDING_UNAVAILABLE") ? "M3_BINDING_UNAVAILABLE" : text;
}

const defaultClient: M3AcceptancePanelClient = {
  load: (host) => host === "agent"
    ? loadAgentM3C07AcceptanceStatus()
    : loadJiaobanM3C07AcceptanceStatus(),
  operate: (host, action, nonce) => host === "agent"
    ? operateAgentM3C07Acceptance(action, nonce)
    : operateJiaobanM3C07Acceptance(action, nonce),
};

const immediateActions: ReadonlyArray<Readonly<{
  action: Extract<M3C07AcceptanceAction, "new" | "continue" | "stop">;
  label: string;
}>> = [
  { action: "new", label: "新建" },
  { action: "continue", label: "继续" },
  { action: "stop", label: "停止" },
];

const forcedRestartStages: ReadonlyArray<Readonly<{
  action: Extract<
    M3C07AcceptanceAction,
    "stage_create_pending" | "stage_start_pending" | "stage_stop_pending"
  >;
  label: string;
}>> = [
  { action: "stage_create_pending", label: "落 CREATE pending" },
  { action: "stage_start_pending", label: "落 START pending" },
  { action: "stage_stop_pending", label: "落 STOP pending" },
];

/**
 * Narrow M3C07 desktop-acceptance surface.  It is intentionally separate from
 * both legacy transcript/relay UI and M3C06 continuation controls: the panel
 * can only call the fixed host acceptance endpoints.
 */
export function M3AcceptancePanel({
  host,
  initialStatus = null,
  initialError = null,
  client = defaultClient,
}: M3AcceptancePanelProps) {
  const [status, setStatus] = useState<M3C07AcceptanceStatus | null>(initialStatus);
  const [error, setError] = useState<string | null>(initialError);
  const [busyAction, setBusyAction] = useState<M3C07AcceptanceAction | null>(null);

  const load = useCallback(async () => {
    try {
      const next = await client.load(host);
      setStatus(next);
      setError(null);
    } catch (cause) {
      setStatus(null);
      setError(unavailable(cause));
    }
  }, [client, host]);

  useEffect(() => {
    if (initialStatus || initialError) return;
    void load();
  }, [initialError, initialStatus, load]);

  const operate = useCallback(async (action: M3C07AcceptanceAction) => {
    setBusyAction(action);
    try {
      const next = await client.operate(host, action, requestNonce(host, action));
      setStatus(next);
      setError(null);
    } catch (cause) {
      setError(unavailable(cause));
    } finally {
      setBusyAction(null);
    }
  }, [client, host]);

  // Normal launch intentionally has no M3 binding.  The panel is not a
  // production error surface: the stable fail-closed code means it must leave
  // no acceptance controls or placeholders in Agent/Jiaoban UI.
  if (error === "M3_BINDING_UNAVAILABLE" || (!status && !error)) return null;

  return (
    <section
      className="panel m3c07-acceptance-panel"
      aria-label={`${host === "agent" ? "Agent" : "交办"} M3C07 隔离验收`}
      data-m3c07-host={host}
      data-m3c07-state={status?.lifecycleState ?? "unavailable"}
    >
      <div className="panel-h">
        <strong>M3C07 隔离桌面验收 · {host === "agent" ? "Agent" : "交办"}</strong>
        <button
          className="secondary-button"
          type="button"
          onClick={() => void load()}
          disabled={busyAction !== null}
        >
          刷新 readback
        </button>
      </div>
      {error ? (
        <p data-m3c07-unavailable="true">{error}</p>
      ) : status ? (
        <>
          <p aria-live="polite">
            {status.lifecycleState} · 会话 {status.sessionState} · 回合 {status.turnState}
          </p>
          <dl>
            <dt>角色</dt><dd>{status.labels.role}</dd>
            <dt>项目</dt><dd>{status.labels.project}</dd>
            <dt>对象</dt><dd>{status.labels.object}</dd>
            <dt>通道</dt><dd>{status.labels.channel}</dd>
            <dt>权限</dt><dd>{status.labels.permission}</dd>
          </dl>
          <div className="m3c07-acceptance-actions">
            {immediateActions.map(({ action, label }) => (
              <button
                key={action}
                className="primary-button"
                type="button"
                onClick={() => void operate(action)}
                disabled={busyAction !== null}
              >
                {busyAction === action ? "处理中…" : label}
              </button>
            ))}
            {forcedRestartStages.map(({ action, label }) => (
              <button
                key={action}
                className="secondary-button"
                type="button"
                onClick={() => void operate(action)}
                disabled={busyAction !== null}
                data-m3c07-restart-stage={action}
              >
                {busyAction === action ? "处理中…" : label}
              </button>
            ))}
            <button
              className="secondary-button"
              type="button"
              onClick={() => void operate("restart_readback")}
              disabled={busyAction !== null}
            >
              {busyAction === "restart_readback" ? "处理中…" : "重启恢复 readback"}
            </button>
            <button
              className="secondary-button"
              type="button"
              onClick={() => void operate("failure_injection_rollback")}
              disabled={busyAction !== null}
            >
              {busyAction === "failure_injection_rollback" ? "处理中…" : "审计回滚注入"}
            </button>
            <button
              className="secondary-button"
              type="button"
              onClick={() => void operate("handoff_exact_replay")}
              disabled={busyAction !== null}
            >
              {busyAction === "handoff_exact_replay" ? "处理中…" : "Handoff exact replay"}
            </button>
            <button
              className="secondary-button"
              type="button"
              onClick={() => void operate("object_navigation")}
              disabled={busyAction !== null}
            >
              {busyAction === "object_navigation" ? "处理中…" : "对象导航（fail-closed）"}
            </button>
          </div>
          <small data-m3c07-ledger="true">
            fake dispatch={status.ledger.fakeDispatches} · fake readback={status.ledger.fakeReadbacks}
            {" · "}real provider attempts={status.ledger.realProviderAttempts}
          </small>
          <small
            data-m3c07-receipt="true"
            data-m3c07-replay={String(status.receipt.replayed)}
            data-m3c07-rollback={String(status.receipt.rollbackApplied)}
          >
            receipt {status.receipt.action} / {status.receipt.outcome}
            {" · "}replay={String(status.receipt.replayed)}
            {" · "}rollback={String(status.receipt.rollbackApplied)}
            {" · "}recovery {status.recovery.state}
            {" · "}{status.objectNavigation.state}
          </small>
        </>
      ) : null}
    </section>
  );
}
