import { pathTail } from "../../lib/format";
import type {
  RealExecutionProductCommandDecisionOutput,
  RealExecutionProductCommandPhaseAOutput,
  RealExecutionProductCommandPhaseBOutput,
  RealExecutionProductCommandPrepareOutput,
  RealExecutionProductCommandPreview,
  SessionRecord,
} from "../../lib/types";
import { readbackCountLabel } from "./TranscriptViews";

export function AgentChatComposer({
  selectedProjectRoot,
  selectedSession,
  draftPrompt,
  k2Preview,
  k2PreviewBusy,
  k2ActionBusy,
  k2PrepareOutput,
  k2DecisionOutput,
  k2PhaseAOutput,
  k2PhaseBOutput,
  k2PreviewError,
  k2Operation,
  onChangeDraft,
  onGeneratePreview,
  onPrepareCommand,
  onConfirmCommand,
  onRecordPhaseA,
  onRunPhaseB,
  onOpenDeveloperDetails,
}: {
  selectedProjectRoot: string;
  selectedSession: SessionRecord | null;
  draftPrompt: string;
  k2Preview: RealExecutionProductCommandPreview | null;
  k2PreviewBusy: boolean;
  k2ActionBusy: string | null;
  k2PrepareOutput: RealExecutionProductCommandPrepareOutput | null;
  k2DecisionOutput: RealExecutionProductCommandDecisionOutput | null;
  k2PhaseAOutput: RealExecutionProductCommandPhaseAOutput | null;
  k2PhaseBOutput: RealExecutionProductCommandPhaseBOutput | null;
  k2PreviewError: string | null;
  k2Operation: "resume" | "new_session";
  onChangeDraft: (value: string) => void;
  onGeneratePreview: () => Promise<void>;
  onPrepareCommand: () => Promise<void>;
  onConfirmCommand: () => Promise<void>;
  onRecordPhaseA: () => Promise<void>;
  onRunPhaseB: () => Promise<void>;
  onOpenDeveloperDetails: () => void;
}) {
  const canPreview = Boolean(selectedProjectRoot && draftPrompt.trim() && (k2Operation === "new_session" || selectedSession));
  const canRunPhaseB = Boolean(k2PhaseAOutput && !k2PhaseAOutput.blocked_reasons.length && !k2PhaseBOutput);
  const operationLabel = k2Operation === "new_session" ? "新建对话" : "继续已有对话";
  return (
    <form
      className="agent-chat-composer"
      aria-label="智能体任务输入"
      onSubmit={(event) => void (async () => {
        event.preventDefault();
        await onGeneratePreview();
      })()}
    >
      <label>
        <span>任务输入</span>
        <textarea
          aria-label="输入给 Codex 的任务"
          value={draftPrompt}
          placeholder="写下要让 Codex 做的事。发送前会先让你确认项目、对话、权限和记忆影响。"
          rows={3}
          onChange={(event) => onChangeDraft(event.currentTarget.value)}
        />
      </label>
      <div className="agent-chat-composer-foot">
        <span>
          {k2Operation === "new_session"
            ? "将创建新对话：先生成预览，再确认执行"
            : selectedSession
            ? `当前对话：${selectedSession.title || selectedSession.thread_id}`
            : "请选择一个可读取对话"}
        </span>
        <button className="primary-button" disabled={!canPreview || k2PreviewBusy} type="submit">
          {k2PreviewBusy ? "生成中" : "生成发送预览"}
        </button>
      </div>
      {k2PreviewError ? <p className="error-text">预览失败：{k2PreviewError}</p> : null}
      {k2Preview ? (
        <section className="agent-send-preview" aria-label="发送预览">
          <div>
            <strong>{k2Preview.blocked_reasons.length ? "需要处理后再发送" : "发送前确认材料已生成"}</strong>
            <span>{operationLabel} · 只读沙箱 · 需要用户确认</span>
          </div>
          <div className="agent-send-preview-grid">
            <span>项目：{selectedProjectRoot ? pathTail(selectedProjectRoot) : "未选择"}</span>
            <span>对话：{k2Operation === "new_session" ? "新建对话" : selectedSession?.title || selectedSession?.thread_id || "未选择"}</span>
            <span>读回：{readbackStatusLabel(k2Preview.readback_boundary.status)} · 结果数 {readbackCountLabel(k2Preview.readback_boundary.result_count)}</span>
            <span>记忆：只生成观察 / 候选来源，不自动写正式记忆</span>
            <span>准备：{k2PrepareOutput?.status ?? "未写入"}</span>
            <span>确认：{k2DecisionOutput?.status ?? "未确认"}</span>
            <span>预检：{k2PhaseAOutput?.status ?? "未记录"}</span>
            <span>执行：{k2PhaseBOutput?.status ?? "未执行"}</span>
          </div>
          {k2Preview.blocked_reasons.length ? (
            <div className="agent-send-preview-warnings">
              {k2Preview.blocked_reasons.map((reason) => (
                <span key={reason}>{codexControlReasonLabel(reason)}</span>
              ))}
            </div>
          ) : null}
          <div className="action-row compact">
            <button
              className="secondary-button"
              disabled={!!k2Preview.blocked_reasons.length || !!k2ActionBusy}
              type="button"
              onClick={() => void onPrepareCommand()}
            >
              {k2ActionBusy === "prepare" ? "准备中" : "写入准备"}
            </button>
            <button
              className="secondary-button"
              disabled={k2PrepareOutput?.status !== "prepared" || !!k2ActionBusy}
              type="button"
              onClick={() => void onConfirmCommand()}
            >
              {k2ActionBusy === "confirm" ? "确认中" : "用户确认"}
            </button>
            <button
              className="secondary-button"
              disabled={!k2DecisionOutput || !!k2ActionBusy}
              type="button"
              onClick={() => void onRecordPhaseA()}
            >
              {k2ActionBusy === "phase-a" ? "记录中" : "记录预检"}
            </button>
            <button
              className="primary-button"
              disabled={!canRunPhaseB || !!k2ActionBusy}
              type="button"
              onClick={() => void onRunPhaseB()}
            >
              {k2ActionBusy === "phase-b" ? "执行中" : "确认执行 Codex"}
            </button>
            <button className="secondary-button" type="button" onClick={onOpenDeveloperDetails}>
              查看开发者详情
            </button>
          </div>
          {k2PhaseBOutput ? (
            <p>
              执行结果：{k2PhaseBOutput.status} · 读回 {readbackStatusLabel(k2PhaseBOutput.readback_summary.status)} · 结果数 {readbackCountLabel(k2PhaseBOutput.readback_summary.result_count)}
            </p>
          ) : null}
        </section>
      ) : null}
      <p>
        预览不会直接发送。下一步仍要确认要做什么、影响哪里、是否写入项目，以及是否产生记忆候选。
      </p>
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
