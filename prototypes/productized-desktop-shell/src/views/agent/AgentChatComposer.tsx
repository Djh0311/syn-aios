import type { AgentProjectOptionReadModel } from "../../lib/pageSelectors";
import { pathTail } from "../../lib/format";
import type { ManualRelayReceipt, SessionRecord } from "../../lib/types";
import { userFacingAgentError } from "./agentLabels";

export type AgentConversationSendMode = "existing_session" | "new_session";

// ⑥ H：manual relay 的两条真实发送路径(新会话 / 既有会话，AgentConversationShell.tsx:528 / 568)
// 都硬编码这个沙箱模式，写根都是 `[目标项目根]`。composer 上那行「将以 X 写入 Y」必须引**同一个常量**，
// 否则脸上写的和真发的会各改各的、悄悄漂移(脸上写 read-only、实际 workspace-write = 最坏的那种假事实)。
export const MANUAL_RELAY_SANDBOX = "workspace-write";

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
  const formSendMode = relayDirectSendEnabled
    ? isNewSessionMode
      ? "manual_relay_new_session"
      : "manual_relay_direct"
    : "decision-only";
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
  const blockedRunSummary = !relayDirectSendEnabled && relayDirectSendBlockedReason ? relayDirectSendBlockedReason : null;
  // ⑥ H 定稿：写根/沙箱一行**常显**(治体检 P1「批态可见性缺席」；宪法 §一 批态 D5：
  // 「用户能在批面看到写根/工具/边界」的可见性承诺不变)。
  // 旧口径 `isNewSessionMode || !!runSummary || !!blockedRunSummary` 会在**既有会话 + 没在跑**时整条藏掉 ——
  // 那正是用户要往一个真实项目里发写指令的时刻，恰恰最需要看见写根。故改常显。
  const writeTargetLabel = selectedProjectRoot ? pathTail(selectedProjectRoot) : null;
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
        {isNewSessionMode ? (
        <div className="agent-composer-target">
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
        </div>
        ) : null}
        {/* 「将以 …」= 将来时 = **本次发送的意图**，与两条发送路径的硬编码常量同源(MANUAL_RELAY_SANDBOX +
            allowed_write_roots:[目标项目根])，口径一致，不是既成事实的谎报。
            没选到项目根时不许编写根：如实说还不知道会写到哪(此时发送按钮本来就是 disabled)。 */}
        <div className="agent-composer-boundary">
          {writeTargetLabel ? (
            // 只显项目名(pathTail)，不显完整路径 —— 定稿 H 段画的就是「写入 mario test」。
            // 「composer 不常驻显示完整项目路径」是既有拍板(offlineConversationEngineScenario.tsx:536)，
            // 定稿只反转了「项目名」这一项(批态可见性)，没要求把整条路径摆上脸 → 那条断言原样留着，别顺手放宽。
            <span>
              将以 {MANUAL_RELAY_SANDBOX} 写入 {writeTargetLabel}
            </span>
          ) : (
            <span>还没选项目——选了才知道会写到哪，也才能发送</span>
          )}
        </div>
        <div className="agent-composer-runstate" aria-live="polite">
          {runSummary ? <em>{runSummary}</em> : null}
          {blockedRunSummary ? <em>{blockedRunSummary}</em> : null}
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
