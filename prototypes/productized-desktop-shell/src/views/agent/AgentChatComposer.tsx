import type { ManualRelayPreview, ManualRelayReceipt, SessionRecord } from "../../lib/types";

export function AgentChatComposer({
  selectedProjectRoot,
  selectedSession,
  draftPrompt,
  k2PreviewError,
  k2Operation,
  manualRelayPreview,
  manualRelayReceipt,
  manualRelayError,
  manualRelayBusy,
  onChangeDraft,
  onSubmitDraft,
  onPreviewManualRelay,
  onRunManualRelayOnce,
  onStopManualRelayAttempt,
  onOpenDeveloperDetails,
}: {
  selectedProjectRoot: string;
  selectedSession: SessionRecord | null;
  draftPrompt: string;
  k2PreviewError: string | null;
  k2Operation: "resume" | "new_session";
  manualRelayPreview: ManualRelayPreview | null;
  manualRelayReceipt: ManualRelayReceipt | null;
  manualRelayError: string | null;
  manualRelayBusy: boolean;
  onChangeDraft: (value: string) => void;
  onSubmitDraft: () => void;
  onPreviewManualRelay: () => void;
  onRunManualRelayOnce: () => void;
  onStopManualRelayAttempt: () => void;
  onOpenDeveloperDetails: () => void;
}) {
  const canPreview = Boolean(selectedProjectRoot && draftPrompt.trim() && (k2Operation === "new_session" || selectedSession));
  const relayGuard = manualRelayPreview?.guard ?? null;
  const relayEnvelope = manualRelayPreview?.envelope ?? null;
  const relayCanRun = Boolean(relayEnvelope && relayGuard && !relayGuard.blocks_execution && !manualRelayBusy);
  const relayIsRunning = manualRelayReceipt?.status === "running";
  return (
    <form
      className="agent-chat-composer"
      data-send-mode="decision-only"
      aria-label="智能体任务输入"
      onSubmit={(event) => void (async () => {
        event.preventDefault();
        onSubmitDraft();
      })()}
    >
      <label>
        <span>任务输入</span>
        <textarea
          aria-label="输入给 Codex 的任务"
          value={draftPrompt}
          placeholder="写下要让 Codex 做的事。Enter 发送；Shift+Enter 换行。"
          rows={3}
          onChange={(event) => onChangeDraft(event.currentTarget.value)}
          onKeyDown={(event) => {
            if (event.key !== "Enter" || event.shiftKey) return;
            event.preventDefault();
            onSubmitDraft();
          }}
        />
      </label>
      <div className="agent-chat-composer-foot">
        <span>
          {k2Operation === "new_session"
            ? "将记录新对话发送意图；真实创建仍需另行授权"
            : selectedSession
            ? `当前对话：${selectedSession.title || selectedSession.thread_id}`
            : "请选择一个可读取对话"}
        </span>
        <button className="primary-button" disabled={!canPreview} type="submit">
          发送
        </button>
      </div>
      {k2PreviewError ? <p className="error-text">发送意图记录失败：{k2PreviewError}</p> : null}
      <button className="secondary-button" type="button" onClick={onOpenDeveloperDetails}>
        查看发送边界
      </button>
      <p>
        已记录发送意图，等待授权执行；本按钮不真跑 Codex、不解锁 K3-B1 / K3-B2。
      </p>
      <section className="manual-relay-panel" data-send-mode="manual_relay" aria-label="Manual relay 中转">
        <div>
          <strong>甲 · Manual relay</strong>
          <p>
            手动一次一发；本包只走 mock / fixture contract，不真跑 Codex、不写 .codex、不自动连环。
          </p>
        </div>
        <div className="manual-relay-actions">
          <button
            className="secondary-button"
            disabled={!canPreview || manualRelayBusy}
            type="button"
            onClick={onPreviewManualRelay}
          >
            预演中转 payload
          </button>
          <button
            className="primary-button"
            disabled={!relayCanRun}
            type="button"
            onClick={onRunManualRelayOnce}
          >
            确认 mock 中转一次
          </button>
          <button
            className="secondary-button"
            disabled={!relayIsRunning || manualRelayBusy}
            type="button"
            onClick={onStopManualRelayAttempt}
          >
            Stop 本 attempt
          </button>
        </div>
        {relayEnvelope ? (
          <div className="manual-relay-preview">
            <div>
              <span>Exact payload</span>
              <pre>{relayEnvelope.payload.effective_prompt}</pre>
            </div>
            <dl>
              <div>
                <dt>Target</dt>
                <dd>{relayEnvelope.target_binding.target_cwd_canonical}</dd>
              </div>
              <div>
                <dt>Session</dt>
                <dd>{relayEnvelope.target_binding.target_session_id ?? "new session"}</dd>
              </div>
              <div>
                <dt>Sandbox</dt>
                <dd>{relayEnvelope.target_binding.sandbox}</dd>
              </div>
              <div>
                <dt>Write roots</dt>
                <dd>{relayEnvelope.target_binding.allowed_write_roots.join(" / ") || "none"}</dd>
              </div>
              <div>
                <dt>Payload layers</dt>
                <dd>{relayEnvelope.payload.payload_layers.length}（v1 必须为空）</dd>
              </div>
              <div>
                <dt>One-shot</dt>
                <dd>{relayEnvelope.policy.manual_once && !relayEnvelope.policy.auto_chain ? "manual_once / auto_chain=false" : "blocked"}</dd>
              </div>
            </dl>
            {relayGuard?.blocks_execution ? (
              <p className="error-text">中转被阻断：{relayGuard.reasons.join(" / ")}</p>
            ) : (
              <p>Guard ready: command plan 使用 stdin prompt，无 shell 拼接；当前仍是 fixture-only。</p>
            )}
          </div>
        ) : null}
        {manualRelayReceipt ? (
          <div className="manual-relay-receipt">
            <strong>回执：{manualRelayReceipt.status}</strong>
            <span>attempt: {manualRelayReceipt.relay_attempt_id}</span>
            <span>real_codex_executed={String(manualRelayReceipt.real_codex_executed)}</span>
            <span>syn_read_codex_home={String(manualRelayReceipt.syn_read_codex_home)}</span>
            <span>killed_by_user={String(manualRelayReceipt.killed_by_user)}</span>
          </div>
        ) : null}
        {manualRelayError ? <p className="error-text">Manual relay 失败：{manualRelayError}</p> : null}
      </section>
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
