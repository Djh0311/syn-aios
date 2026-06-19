import type { AgentProjectOptionReadModel } from "../../lib/pageSelectors";
import { pathTail } from "../../lib/format";
import type { ManualRelayReceipt, SessionRecord } from "../../lib/types";
import { userFacingAgentError } from "./agentLabels";

export type AgentConversationSendMode = "existing_session" | "new_session";

export function AgentChatComposer({
  sendMode = "existing_session",
  projectOptions = [],
  selectedProjectRoot,
  selectedSession,
  draftPrompt,
  k2PreviewError,
  manualRelayReceipt,
  manualRelayError,
  manualRelayBusy,
  manualRelayPollingPaused = false,
  manualRelayTimedOutLocally = false,
  relayDirectSendEnabled,
  relayDirectSendBlockedReason,
  onChangeDraft,
  onChangeSelectedProjectRoot,
  onSubmitDraft,
  onResumeManualRelayPolling,
  onStopManualRelayAttempt,
  onOpenDeveloperDetails,
}: {
  sendMode?: AgentConversationSendMode;
  projectOptions?: AgentProjectOptionReadModel[];
  selectedProjectRoot: string;
  selectedSession: SessionRecord | null;
  draftPrompt: string;
  k2PreviewError: string | null;
  manualRelayReceipt: ManualRelayReceipt | null;
  manualRelayError: string | null;
  manualRelayBusy: boolean;
  manualRelayPollingPaused?: boolean;
  manualRelayTimedOutLocally?: boolean;
  relayDirectSendEnabled: boolean;
  relayDirectSendBlockedReason?: string | null;
  onChangeDraft: (value: string) => void;
  onChangeSelectedProjectRoot?: (value: string) => void;
  onSubmitDraft: () => void;
  onResumeManualRelayPolling?: () => void;
  onStopManualRelayAttempt: () => void;
  onOpenDeveloperDetails: () => void;
}) {
  const relayIsRunning = manualRelayReceipt?.status === "running";
  const relayLiveEvents = manualRelayReceipt?.live_events ?? [];
  const relayLiveEventCount = relayLiveEvents.length;
  const relayLastLiveTitle = relayLiveEvents.at(-1)?.title ?? null;
  const relayInputLocked = manualRelayBusy || relayIsRunning;
  const isNewSessionMode = sendMode === "new_session";
  const selectedProjectLabel =
    projectOptions.find((project) => project.project_root === selectedProjectRoot)?.label ??
    pathTail(selectedProjectRoot) ??
    selectedProjectRoot;
  const projectPickerOptions =
    selectedProjectRoot && !projectOptions.some((project) => project.project_root === selectedProjectRoot)
      ? [
          {
            project_root: selectedProjectRoot,
            label: selectedProjectLabel,
            active_session_count: 0,
            session_count: 0,
          },
          ...projectOptions,
        ]
      : projectOptions;
  const canDirectSend = Boolean(
    relayDirectSendEnabled &&
      selectedProjectRoot &&
      draftPrompt.trim() &&
      !relayInputLocked &&
      (isNewSessionMode || selectedSession),
  );
  const targetSessionTitle = isNewSessionMode ? "新建对话" : selectedSession?.title || "未命名会话";
  const formSendMode = relayDirectSendEnabled
    ? isNewSessionMode
      ? "manual_relay_new_session"
      : "manual_relay_direct"
    : "decision-only";
  const targetSummary = isNewSessionMode
    ? `新建对话 · ${selectedProjectLabel || "未绑定项目"}`
    : `${targetSessionTitle} · ${selectedProjectLabel || "未绑定项目"}`;
  const runSummary = relayIsRunning
    ? manualRelayPollingPaused
      ? "状态刷新已暂停"
      : `${relayLastLiveTitle ?? "Codex 正在运行"}${relayLiveEventCount > 0 ? ` · ${relayLiveEventCount} 步` : ""}`
    : manualRelayTimedOutLocally
      ? "已超时"
    : manualRelayReceipt?.status && manualRelayReceipt.status !== "running"
      ? `上次运行：${manualRelayReceipt.status}`
      : null;
  const relayErrorInfo = manualRelayError ? userFacingAgentError(manualRelayError) : null;
  return (
    <form
      className="agent-chat-composer"
      data-send-mode={formSendMode}
      aria-label="智能体任务输入"
      onSubmit={(event) => void (async () => {
        event.preventDefault();
        if (canDirectSend) onSubmitDraft();
      })()}
    >
      <div className={`agent-composer-statusline ${relayDirectSendEnabled ? "armed" : "blocked"} ${relayIsRunning ? "running" : ""}`}>
        <div className="agent-composer-target">
          <strong>{isNewSessionMode ? "新建对话" : "继续对话"}</strong>
          {isNewSessionMode ? (
            <label className="agent-composer-project-picker">
              <span>项目</span>
              <select
                aria-label="选择新对话项目"
                value={selectedProjectRoot}
                disabled={relayInputLocked}
                onChange={(event) => onChangeSelectedProjectRoot?.(event.currentTarget.value)}
              >
                <option value="">选择项目</option>
                {projectPickerOptions.map((project) => (
                  <option key={project.project_root} value={project.project_root}>
                    {project.label} ({project.active_session_count}/{project.session_count})
                  </option>
                ))}
              </select>
            </label>
          ) : (
            <span>{targetSummary}</span>
          )}
        </div>
        <div className="agent-composer-runstate" aria-live="polite">
          {runSummary ? <em>{runSummary}</em> : null}
          {!relayDirectSendEnabled && relayDirectSendBlockedReason ? <em>{relayDirectSendBlockedReason}</em> : null}
        </div>
      </div>
      <label>
        <span className="agent-composer-label">给 Codex</span>
        <textarea
          aria-label="输入给 Codex 的任务"
          value={draftPrompt}
          placeholder="写下要让 Codex 做的事。"
          rows={1}
          readOnly={relayInputLocked}
          aria-busy={relayInputLocked}
          onChange={(event) => {
            if (!relayInputLocked) onChangeDraft(event.currentTarget.value);
          }}
          onKeyDown={(event) => {
            if (relayInputLocked) {
              event.preventDefault();
              return;
            }
            if (event.key !== "Enter" || event.shiftKey) return;
            event.preventDefault();
            if (canDirectSend) onSubmitDraft();
          }}
        />
      </label>
      <div className="agent-chat-composer-foot">
        <div className="agent-chat-composer-actions">
          {manualRelayPollingPaused && relayIsRunning ? (
            <button
              className="secondary-button"
              disabled={manualRelayBusy || !onResumeManualRelayPolling}
              type="button"
              onClick={onResumeManualRelayPolling}
            >
              恢复轮询
            </button>
          ) : null}
          {relayIsRunning ? (
            <button
              className="secondary-button"
              disabled={manualRelayBusy}
              type="button"
              onClick={onStopManualRelayAttempt}
            >
              Stop
            </button>
          ) : null}
          <button className="primary-button" disabled={!canDirectSend} type="submit">
            发送
          </button>
        </div>
      </div>
      {k2PreviewError ? <p className="error-text">发送意图记录失败：{k2PreviewError}</p> : null}
      {relayErrorInfo ? (
        <div className="agent-composer-error" role="alert">
          <strong>{relayErrorInfo.title}</strong>
          <span>{relayErrorInfo.nextStep}</span>
          <button className="secondary-button" type="button" onClick={onOpenDeveloperDetails}>
            查看开发者详情
          </button>
        </div>
      ) : null}
    </form>
  );
}


export function readbackStatusLabel(status?: string | null) {
  if (!status) return "未登记";
  if (status === "not_attempted_stub") return "桩执行未读回";
  if (status === "readback_unavailable") return "读回不可用";
  if (status === "readback_failed") return "读回失败";
  if (status === "readback_succeeded") return "读回成功";
  if (status === "not_attempted") return "未读回";
  if (status === "timed_out") return "超时";
  if (status === "codex_state_error") return "Codex 状态不可写";
  if (status === "blocked") return "阻断";
  return status;
}

export function codexControlReasonLabel(reason: string) {
  const labels: Record<string, string> = {
    codex_control_new_session_deferred_in_j1a: "新会话真实启动留到后续执行点授权",
    codex_control_resume_requires_target_session: "恢复已有会话需要选择目标 session",
    codex_control_prompt_hash_invalid: "任务正文摘要校验未生成",
    codex_control_sensitive_denied_paths_missing: "敏感路径拒绝清单不完整",
    codex_control_allowed_write_roots_boundary_missing: "需要项目根作为执行边界根",
  };
  return labels[reason] ?? reason;
}
