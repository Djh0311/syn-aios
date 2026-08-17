import { useEffect, useState } from "react";
import {
  applyM5ExecutionControl,
  loadM5ExecutionControl,
  loadM5GlobalAdviceFixture,
  loadM5ProjectSummary,
  openM5ProjectSupervisor,
  openM5SourceDeepLink,
  recordM5AuthorizationDecision,
  recordM5IndependentReview,
  recordM5ResultDecision,
  recordM5WorkerReport,
  runM5AuthorizedRuntime,
  submitM5SupervisorTurn,
  type M5ExecutionControlAction,
  type M5ExecutionControlResponse,
  type M5ProjectSummaryRead,
  type M5SupervisorOpenResponse,
} from "../../lib/m5ProjectSupervisor";

export function ProjectSupervisorPanel({ projectId }: { projectId: string }) {
  const [session, setSession] = useState<M5SupervisorOpenResponse | null>(null);
  const [chat, setChat] = useState("");
  const [log, setLog] = useState<string[]>([]);
  const [proposalId, setProposalId] = useState<string | null>(null);
  const [summary, setSummary] = useState<M5ProjectSummaryRead | null>(null);
  const [deepLink, setDeepLink] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [grantId, setGrantId] = useState<string | null>(null);
  const [dispatchId, setDispatchId] = useState<string | null>(null);
  const [adviceWritable, setAdviceWritable] = useState<string | null>(null);
  const [control, setControl] = useState<M5ExecutionControlResponse | null>(null);

  useEffect(() => {
    let cancelled = false;
    openM5ProjectSupervisor(projectId)
      .then(async (opened) => {
        if (cancelled) return;
        setSession(opened);
        try {
          const next = await loadM5ExecutionControl({
            binding_id: opened.binding_id,
            project_id: opened.project_id,
          });
          if (!cancelled) setControl(next);
        } catch (reason: unknown) {
          if (!cancelled) setError(String(reason));
        }
      })
      .catch((reason: unknown) => {
        if (!cancelled) setError(String(reason));
      });
    return () => {
      cancelled = true;
    };
  }, [projectId]);

  useEffect(() => {
    if (!session) return;
    const reload = () => {
      void loadM5ExecutionControl({
        binding_id: session.binding_id,
        project_id: session.project_id,
      })
        .then(setControl)
        .catch((reason: unknown) => setError(String(reason)));
    };
    window.addEventListener("syn-m5-execution-control-refresh", reload);
    return () => {
      window.removeEventListener("syn-m5-execution-control-refresh", reload);
    };
  }, [session]);

  async function sendChat() {
    if (!session) return;
    const turn = await submitM5SupervisorTurn({
      binding_id: session.binding_id,
      project_id: session.project_id,
      kind: "chat",
      text: chat,
    });
    setLog((rows) => [...rows, `chat grant=${String(turn.created_grant)} spawn=${String(turn.spawned)}`]);
    setChat("");
  }

  async function propose() {
    if (!session) return;
    const turn = await submitM5SupervisorTurn({
      binding_id: session.binding_id,
      project_id: session.project_id,
      kind: "submit_proposal",
      text: chat || "echo hello",
    });
    setProposalId(turn.text);
    setLog((rows) => [...rows, `proposal ${turn.text}`]);
  }

  async function decide(decision: "APPROVED" | "REJECTED") {
    if (!session || !proposalId) return;
    const result = await recordM5AuthorizationDecision({
      binding_id: session.binding_id,
      project_id: session.project_id,
      proposal_id: proposalId,
      decision,
    });
    if (result.grant_id) setGrantId(result.grant_id);
    if (result.dispatch_id) setDispatchId(result.dispatch_id);
    setLog((rows) => [
      ...rows,
      `${decision} dispatched=${String(result.dispatched)} grant=${result.grant_id ?? "none"}`,
    ]);
  }

  async function loadSummary() {
    if (!session) return;
    const next = await loadM5ProjectSummary(session.binding_id, session.project_id);
    setSummary(next);
  }

  async function openLink(sourceId: string) {
    if (!session) return;
    const link = await openM5SourceDeepLink(session.binding_id, session.project_id, sourceId);
    setDeepLink(link);
  }

  async function runFormal(step: "runtime" | "report" | "review" | "result") {
    if (!session) return;
    const input = { binding_id: session.binding_id, project_id: session.project_id };
    const outcome =
      step === "runtime"
        ? await runM5AuthorizedRuntime(input)
        : step === "report"
          ? await recordM5WorkerReport(input)
          : step === "review"
            ? await recordM5IndependentReview(input)
            : await recordM5ResultDecision(input);
    if (outcome.grant_id) setGrantId(outcome.grant_id);
    if (outcome.dispatch_id) setDispatchId(outcome.dispatch_id);
    setLog((rows) => [
      ...rows,
      `${outcome.step} receipt=${outcome.receipt_id ?? "none"} claim=${outcome.claim_id ?? "none"} review=${outcome.review_id ?? "none"} result=${String(outcome.result_decision_recorded)}`,
    ]);
    if (outcome.result_decision_recorded) {
      await loadSummary();
    }
    const next = await loadM5ExecutionControl(input);
    setControl(next);
  }

  async function applyControl(action: M5ExecutionControlAction) {
    if (!session || !control) return;
    const allowed =
      action === "STOP" ? control.can_stop : action === "RETRY" ? control.can_retry : control.can_resume;
    if (!allowed) return;
    try {
      const next = await applyM5ExecutionControl({
        binding_id: session.binding_id,
        project_id: session.project_id,
        action,
        expected_control_revision: control.control_revision,
      });
      setControl(next);
      setLog((rows) => [
        ...rows,
        `${action} replayed=${String(next.replayed)} receipt=${next.last_receipt_id ?? "none"} blocked=${next.blocked_reason ?? "none"}`,
      ]);
    } catch (reason: unknown) {
      setError(String(reason));
    }
  }

  return (
    <section
      className="project-supervisor-panel"
      data-m5-supervisor-panel="ready"
      data-m5-session-status={session ? "open" : "loading"}
      data-m5-project-id={session?.project_id ?? ""}
      data-m5-project-alias={projectId}
      data-m5-binding-id={session?.binding_id ?? ""}
      data-m5-role-session-id={session?.role_session_id ?? ""}
      data-m5-deep-link={deepLink ?? ""}
      data-m5-proposal-id={proposalId ?? ""}
      data-m5-grant-id={grantId ?? ""}
      data-m5-dispatch-id={dispatchId ?? ""}
      data-m5-advice-writable={adviceWritable ?? ""}
      data-m5-control-revision={control ? String(control.control_revision) : ""}
      data-m5-control-phase={control?.phase ?? ""}
      data-m5-control-state={control?.durable_state ?? ""}
      data-m5-control-attempt={control?.attempt_state ?? ""}
      data-m5-can-stop={control ? String(control.can_stop) : ""}
      data-m5-can-retry={control ? String(control.can_retry) : ""}
      data-m5-can-resume={control ? String(control.can_resume) : ""}
      data-m5-blocked-reason={control?.blocked_reason ?? ""}
      data-m5-control-receipt={control?.last_receipt_id ?? ""}
      data-m5-control-replayed={control ? String(control.replayed) : ""}
    >
      <h2>项目主管</h2>
      {error ? <p data-m5-supervisor-error={error}>{error}</p> : null}
      <p data-m5-session-status={session ? "open" : "loading"}>
        {session ? `会话 ${session.role_session_id}` : "正在恢复项目主管会话"}
      </p>
      <textarea
        aria-label="项目主管对话"
        data-m5-supervisor-input="1"
        value={chat}
        onChange={(event) => setChat(event.target.value)}
      />
      <div>
        <button type="button" data-m5-action="chat" onClick={() => void sendChat()}>
          只读对话
        </button>
        <button type="button" data-m5-action="propose" onClick={() => void propose()}>
          提出动作
        </button>
        <button type="button" data-m5-action="reject" onClick={() => void decide("REJECTED")}>
          拒绝
        </button>
        <button type="button" data-m5-action="approve" onClick={() => void decide("APPROVED")}>
          批准执行
        </button>
        <button type="button" data-m5-action="runtime" onClick={() => void runFormal("runtime")}>
          运行回执
        </button>
        <button type="button" data-m5-action="report" onClick={() => void runFormal("report")}>
          记录报告
        </button>
        <button type="button" data-m5-action="review" onClick={() => void runFormal("review")}>
          独立审查
        </button>
        <button type="button" data-m5-action="result" onClick={() => void runFormal("result")}>
          结果决定
        </button>
        <button type="button" data-m5-action="summary" onClick={() => void loadSummary()}>
          读取摘要
        </button>
        <button
          type="button"
          data-m5-action="advice"
          onClick={() => {
            if (!session) return;
            void loadM5GlobalAdviceFixture(session.binding_id, session.project_id).then((advice) => {
              setAdviceWritable(String(advice.writable));
              setLog((rows) => [...rows, `advice writable=${String(advice.writable)}`]);
            });
          }}
        >
          只读建议
        </button>
        <button
          type="button"
          data-m5-action="stop"
          disabled={!control?.can_stop}
          onClick={() => void applyControl("STOP")}
        >
          停止
        </button>
        <button
          type="button"
          data-m5-action="retry"
          disabled={!control?.can_retry}
          onClick={() => void applyControl("RETRY")}
        >
          重试
        </button>
        <button
          type="button"
          data-m5-action="resume"
          disabled={!control?.can_resume}
          onClick={() => void applyControl("RESUME")}
        >
          恢复
        </button>
      </div>
      {control ? (
        <aside data-m5-execution-control="1">
          <p>
            控制 {control.phase} / {control.durable_state} rev={control.control_revision}
          </p>
          <p>attempt {control.attempt_state ?? "none"}</p>
          <p>blocked {control.blocked_reason ?? "none"}</p>
          <p>
            receipt {control.last_receipt_id ?? "none"} replayed={String(control.replayed)}
          </p>
        </aside>
      ) : null}
      <pre data-m5-supervisor-log="1">{log.join("\n")}</pre>
      {summary ? (
        <aside data-m5-summary-stale={summary.stale ? "true" : "false"}>
          <p>watermark {summary.watermark_ms} stale={String(summary.stale)}</p>
          <ul>
            {summary.source_refs.map((ref) => (
              <li key={ref.source_id}>
                <button
                  type="button"
                  data-m5-deep-link-source={ref.source_id}
                  onClick={() => void openLink(ref.source_id)}
                >
                  {ref.deep_link}
                </button>
              </li>
            ))}
          </ul>
        </aside>
      ) : null}
    </section>
  );
}
