import { useEffect, useMemo, useState } from "react";
import { pathTail } from "../../lib/format";
import { deriveAgentsPageReadModelFromParts } from "../../lib/pageSelectors";
import {
  confirmRealExecutionProductCommand,
  prepareRealExecutionProductCommand,
  previewRealExecutionProductCommand,
  runRealExecutionProductCommandPhaseA,
} from "../../lib/tauri";
import type {
  CodexControlCommandInput,
  H2RealResumeExecutionDecisionSurface,
  ProjectRecord,
  ProjectWorkflowAutomationReadModel,
  RealExecutionProductCommandDecisionOutput,
  RealExecutionProductCommandPhaseAOutput,
  RealExecutionProductCommandPrepareOutput,
  RealExecutionProductCommandPreview,
  RealExecutionProductCommandReadModel,
  RuntimeSessionAttention,
  SessionContinuationStoreV1,
  SessionRecord,
  SessionRunStatusSummary,
  WorkflowStateSnapshot,
} from "../../lib/types";
import { readbackCountLabel } from "./TranscriptViews";
import {
  attemptStatusLabel,
  automationRunUnitLabel,
  automationStatusLabel,
  automationUnitStatusLabel,
  codexControlPreviewLabel,
  codexControlReasonLabel,
  h2DecisionStatusLabel,
  J1_DEFAULT_DENIED_PATHS,
  j1ControlSlug,
  messageOf,
  productCommandStatusLabel,
  productEntryStatusLabel,
  readbackStatusLabel,
  runtimeAttentionLabel,
  sha256HexText,
} from "./agentLabels";

export function CodexControlEntryPanel({
  sessions,
  projects,
  selectedSession,
  realExecutionProductCommands,
  workflowState,
}: {
  sessions: SessionRecord[];
  projects: ProjectRecord[];
  selectedSession: SessionRecord | null;
  realExecutionProductCommands: RealExecutionProductCommandReadModel | null;
  workflowState: WorkflowStateSnapshot | null;
}) {
  const projectOptions = useMemo(
    () =>
      deriveAgentsPageReadModelFromParts({
        projects,
        sessions,
        adapterDescriptors: [],
        sessionOperationDescriptors: [],
        providerAvailabilitySummaries: [],
      }).project_options,
    [projects, sessions],
  );
  const initialProjectRoot = selectedSession?.project_root ?? projectOptions[0]?.project_root ?? "";
  const [projectRoot, setProjectRoot] = useState(initialProjectRoot);
  const [operationId, setOperationId] = useState<"resume" | "new_session">("resume");
  const [targetSessionId, setTargetSessionId] = useState(selectedSession?.thread_id ?? "");
  const [sandbox, setSandbox] = useState("read-only");
  const [promptSummary, setPromptSummary] = useState("");
  const [promptBody, setPromptBody] = useState("");
  const [draftCreatedAt, setDraftCreatedAt] = useState(() => new Date().toISOString());
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [preview, setPreview] = useState<RealExecutionProductCommandPreview | null>(null);
  const [prepareOutput, setPrepareOutput] = useState<RealExecutionProductCommandPrepareOutput | null>(null);
  const [decisionOutput, setDecisionOutput] = useState<RealExecutionProductCommandDecisionOutput | null>(null);
  const [phaseAOutput, setPhaseAOutput] = useState<RealExecutionProductCommandPhaseAOutput | null>(null);
  const [localReadModel, setLocalReadModel] = useState<RealExecutionProductCommandReadModel | null>(realExecutionProductCommands);

  useEffect(() => {
    setLocalReadModel(realExecutionProductCommands);
  }, [realExecutionProductCommands]);

  useEffect(() => {
    if (!selectedSession) return;
    if (selectedSession.project_root && !projectRoot) setProjectRoot(selectedSession.project_root);
    if (!targetSessionId) setTargetSessionId(selectedSession.thread_id);
  }, [projectRoot, selectedSession, targetSessionId]);

  const projectSessions = useMemo(
    () => sessions.filter((session) => !projectRoot || session.project_root === projectRoot),
    [projectRoot, sessions],
  );
  const selectedProjectWorkflow = useMemo(
    () => workflowState?.project_workflows.find((workflow) => workflow.project_root === projectRoot) ?? null,
    [projectRoot, workflowState],
  );
  const commandId = prepareOutput?.product_command_id ?? preview?.request.product_command_id ?? null;
  const canBuildDraft = Boolean(
    projectRoot.trim() &&
      promptSummary.trim() &&
      promptBody.trim() &&
      (operationId !== "resume" || targetSessionId.trim()),
  );

  async function buildInput(): Promise<CodexControlCommandInput> {
    if (!projectRoot.trim()) throw new Error("请选择项目。");
    if (!promptSummary.trim()) throw new Error("请填写任务摘要。");
    if (!promptBody.trim()) throw new Error("请填写任务正文。");
    if (operationId === "resume" && !targetSessionId.trim()) throw new Error("恢复已有会话需要选择 session。");
    const promptHash = await sha256HexText(promptBody);
    const shortHash = promptHash.slice(0, 12);
    const projectSlug = j1ControlSlug(projectRoot);
    const projectId = selectedProjectWorkflow?.project_id ?? `project:${projectSlug}`;
    const workflowId = selectedProjectWorkflow?.workflow_id ?? `workflow:j1-codex-control:${projectSlug}`;
    const runRef = `j1-codex-control:${projectSlug}:${shortHash}`;
    return {
      project_id: projectId,
      project_root: projectRoot,
      workflow_id: workflowId,
      node_id: `node:${runRef}`,
      work_item_id: `work-item:${runRef}`,
      task_package_ref: `task-package:${runRef}`,
      memory_packet_ref: `memory-packet:${runRef}`,
      adapter_id: "codex-local",
      operation_id: operationId,
      session_mode: operationId === "new_session" ? "new_session_preview_only" : "resume_existing_session",
      target_session_id: operationId === "resume" ? targetSessionId : null,
      sandbox,
      prompt_summary: promptSummary.trim(),
      prompt_ref: `workbench-runtime-prompt:${runRef}`,
      prompt_hash: promptHash,
      allowed_write_roots: [projectRoot],
      denied_paths: J1_DEFAULT_DENIED_PATHS,
      readback_plan: "readback_unavailable_is_not_zero_results",
      timeout_ms: 120_000,
      requested_by: "user",
    };
  }

  async function runStep<T>(label: string, task: () => Promise<T>): Promise<T | null> {
    setBusy(label);
    setError(null);
    try {
      return await task();
    } catch (stepError) {
      setError(messageOf(stepError));
      return null;
    } finally {
      setBusy(null);
    }
  }

  async function handlePreview() {
    const result = await runStep("preview", async () => {
      const codexControl = await buildInput();
      return previewRealExecutionProductCommand({
        source_kind: "codex_control",
        h5_dispatch_preview: null,
        codex_control: codexControl,
        requested_by: "user",
        created_at: draftCreatedAt,
      });
    });
    if (!result) return;
    setPreview(result);
    setPrepareOutput(null);
    setDecisionOutput(null);
    setPhaseAOutput(null);
  }

  async function handlePrepare() {
    const result = await runStep("prepare", async () => {
      const codexControl = await buildInput();
      return prepareRealExecutionProductCommand({
        source_kind: "codex_control",
        h5_dispatch_preview: null,
        codex_control: codexControl,
        expected_store_revision: localReadModel?.store_revision ?? 0,
        requested_by: "user",
        created_at: draftCreatedAt,
      });
    });
    if (!result) return;
    setPrepareOutput(result);
    setPreview(result.preview);
    setLocalReadModel(result.read_model);
    setDecisionOutput(null);
    setPhaseAOutput(null);
  }

  async function handleConfirm() {
    if (!prepareOutput?.product_command_id) return;
    const result = await runStep("confirm", () =>
      confirmRealExecutionProductCommand({
        product_command_id: prepareOutput.product_command_id ?? "",
        expected_store_revision: prepareOutput.store_revision,
        confirmed_by: "user",
        risk_acknowledgement: "用户确认 J1-A 只记录受控命令和 Phase A，不发送 prompt，不执行真实 Codex。",
        allowed_once: true,
        reason: "J1-A controlled Codex control entry confirmation.",
        requested_by: "user",
        confirmed_at: new Date().toISOString(),
      }),
    );
    if (!result) return;
    setDecisionOutput(result);
    setLocalReadModel(result.read_model);
    setPhaseAOutput(null);
  }

  async function handlePhaseA() {
    if (!prepareOutput?.product_command_id || !decisionOutput) return;
    const result = await runStep("phase-a", () =>
      runRealExecutionProductCommandPhaseA({
        product_command_id: prepareOutput.product_command_id ?? "",
        expected_product_command_store_revision: decisionOutput.store_revision,
        expected_session_continuation_store_revision: null,
        actor_role: "user",
        execution_decision: "phase_a_noop",
        timeout_ms: 120_000,
        requested_at: new Date().toISOString(),
      }),
    );
    if (!result) return;
    setPhaseAOutput(result);
    setLocalReadModel(result.read_model);
  }

  function resetDraft() {
    setDraftCreatedAt(new Date().toISOString());
    setPreview(null);
    setPrepareOutput(null);
    setDecisionOutput(null);
    setPhaseAOutput(null);
    setError(null);
  }

  return (
    <section className="codex-control-panel" aria-label="Codex 控制入口">
      <div className="sec-head">
        <h2>Codex 控制</h2>
        <span className="sec-meta">J1-A · 产品命令入口 · 非真实执行</span>
      </div>
      <p className="codex-control-lead">
        在工作台里选择项目和会话，生成受控 Product Command。J1-A 只做预览、准备、用户确认和 Phase A 记录；不会发送任务正文，不会执行真实 Codex。
      </p>
      <div className="codex-control-grid">
        <label>
          <span>项目</span>
          <select value={projectRoot} onChange={(event) => setProjectRoot(event.currentTarget.value)}>
            <option value="">选择项目</option>
            {projectOptions.map((project) => (
              <option key={project.project_root} value={project.project_root}>
                {project.label || pathTail(project.project_root)}
              </option>
            ))}
          </select>
        </label>
        <label>
          <span>运行模式</span>
          <select value={operationId} onChange={(event) => setOperationId(event.currentTarget.value as "resume" | "new_session")}>
            <option value="resume">恢复已有会话</option>
            <option value="new_session">新会话（本阶段暂缓）</option>
          </select>
        </label>
        <label>
          <span>目标会话</span>
          <select
            disabled={operationId !== "resume"}
            value={targetSessionId}
            onChange={(event) => setTargetSessionId(event.currentTarget.value)}
          >
            <option value="">选择 session</option>
            {projectSessions.map((session) => (
              <option key={session.thread_id} value={session.thread_id}>
                {session.title || session.thread_id}
              </option>
            ))}
          </select>
        </label>
        <label>
          <span>沙箱</span>
          <select value={sandbox} onChange={(event) => setSandbox(event.currentTarget.value)}>
            <option value="read-only">只读</option>
            <option value="workspace-write">工作区写入（仅后续授权）</option>
          </select>
        </label>
      </div>
      <label className="codex-control-field">
        <span>任务摘要</span>
        <input
          value={promptSummary}
          placeholder="一句话说明要让 Codex 做什么"
          onChange={(event) => setPromptSummary(event.currentTarget.value)}
        />
      </label>
      <label className="codex-control-field">
        <span>任务正文</span>
        <textarea
          value={promptBody}
          placeholder="这里的正文只用于运行时。J1-A 不发送、不写 sidecar、不写 runtime log、不写记忆。"
          rows={5}
          onChange={(event) => setPromptBody(event.currentTarget.value)}
        />
      </label>
      <div className="codex-control-boundary">
        <span>任务正文保存策略：只计算摘要引用和 sha256；正文不进入工作台 sidecar、runtime log、audit 或记忆。</span>
        <span>记忆影响：本入口后续只产生观察 / 候选来源，不会自动写正式记忆。</span>
        <span>执行边界根：{projectRoot ? pathTail(projectRoot) : "待选择"}；只读沙箱下不代表项目写授权。</span>
        <span>临时运行绑定：{selectedProjectWorkflow ? selectedProjectWorkflow.title : "J1 临时运行"}；Product Command 会绑定项目 / workflow / work item，不作为游离控制台。</span>
      </div>
      <div className="action-row">
        <button className="secondary-button" disabled={!canBuildDraft || !!busy} type="button" onClick={() => void handlePreview()}>
          {busy === "preview" ? "生成中" : "生成预览"}
        </button>
        <button className="secondary-button" disabled={!canBuildDraft || !!busy} type="button" onClick={() => void handlePrepare()}>
          {busy === "prepare" ? "准备中" : "写入准备"}
        </button>
        <button
          className="secondary-button"
          disabled={!prepareOutput?.product_command_id || prepareOutput.status !== "prepared" || !!busy}
          type="button"
          onClick={() => void handleConfirm()}
        >
          {busy === "confirm" ? "确认中" : "用户确认"}
        </button>
        <button
          className="primary-button"
          disabled={!prepareOutput?.product_command_id || !decisionOutput || !!busy}
          type="button"
          onClick={() => void handlePhaseA()}
        >
          {busy === "phase-a" ? "记录中" : "记录 Phase A（不真实执行）"}
        </button>
        <button className="secondary-button" disabled={!!busy} type="button" onClick={resetDraft}>
          重置本轮草稿
        </button>
      </div>
      {error ? <p className="error-text">操作失败：{error}</p> : null}
      <div className="codex-control-status-grid">
        <span>预览：{preview ? codexControlPreviewLabel(preview) : "未生成"}</span>
        <span>准备：{prepareOutput?.status ?? "未写入"}</span>
        <span>确认：{decisionOutput?.status ?? "未确认"}</span>
        <span>Phase A：{phaseAOutput?.status ?? "未记录"}</span>
        <span>命令：{commandId ? "已生成受控命令" : "未生成"}</span>
        <span>store revision：{localReadModel?.store_revision ?? 0}</span>
      </div>
      {preview?.blocked_reasons.length ? (
        <div className="codex-control-warnings">
          {preview.blocked_reasons.map((reason) => (
            <span key={reason}>{codexControlReasonLabel(reason)}</span>
          ))}
        </div>
      ) : null}
      {phaseAOutput ? (
        <div className="codex-control-warnings">
          <span>prompt_sent={String(phaseAOutput.prompt_sent)}</span>
          <span>real_codex_executed={String(phaseAOutput.real_codex_executed)}</span>
          <span>writes_codex_home={String(phaseAOutput.writes_codex_home)}</span>
          <span>writes_project_files={String(phaseAOutput.writes_project_files)}</span>
          <span>读回：{readbackStatusLabel(phaseAOutput.readback_summary.status)} · 结果数 {readbackCountLabel(phaseAOutput.readback_summary.result_count)}</span>
        </div>
      ) : null}
    </section>
  );
}

export function UnifiedExecutionStatusPanel({
  surface,
  store,
  runtimeSessionAttention,
  sessionRunStatusSummaries,
  realExecutionProductCommands,
  projectWorkflowAutomation,
  projectDispatchCount,
  projectAttemptCount,
}: {
  surface: H2RealResumeExecutionDecisionSurface;
  store: SessionContinuationStoreV1 | null;
  runtimeSessionAttention: RuntimeSessionAttention[];
  sessionRunStatusSummaries: SessionRunStatusSummary[];
  realExecutionProductCommands: RealExecutionProductCommandReadModel | null;
  projectWorkflowAutomation: ProjectWorkflowAutomationReadModel | null;
  projectDispatchCount: number;
  projectAttemptCount: number;
}) {
  const attempts = store?.attempts ?? [];
  const realAttempts = attempts.filter((attempt) => attempt.real_codex_executed);
  const latestRealAttempt = realAttempts[realAttempts.length - 1] ?? null;
  const latestAttempt = attempts[attempts.length - 1] ?? null;
  const leadAttention = runtimeSessionAttention[0] ?? null;
  const readback = latestRealAttempt?.readback_summary ?? latestAttempt?.readback_summary ?? null;
  const runtimeRefCount = runtimeSessionAttention.length + sessionRunStatusSummaries.length;
  const auditRefCount = attempts.reduce((count, attempt) => count + attempt.audit_refs.length, 0);
  const productCommandStatus = productCommandStatusLabel(realExecutionProductCommands);
  const failureStopRetry = realExecutionProductCommands?.failure_stop_retry_summary ?? null;
  const failureStopRetryItems = failureStopRetry?.items ?? [];
  const automationUnits = projectWorkflowAutomation?.latest_plan?.run_units ?? [];

  return (
    <section className="h2-execution-decision-panel unified-execution-panel" aria-label="统一执行链路摘要">
      <div className="sec-head">
        <h2>统一执行链路</h2>
        <span className="sec-meta">
          本地适配器 · {realExecutionProductCommands?.command_count ?? 0} 条统一命令 · {projectDispatchCount} 次历史派发
        </span>
      </div>
      <p className="h2-execution-decision-note">
        本页只展示统一执行链路的准备、确认、受控记录和读回边界；Codex 控制必须走上方产品命令入口，不能使用裸控制台或绕过确认。
      </p>
      <div className="h2-execution-decision-summary">
        <span>统一链路：{productCommandStatus}</span>
        <span>等待确认：{realExecutionProductCommands?.pending_decision_count ?? 0}</span>
        <span>受控记录：{realExecutionProductCommands?.running_attempt_count ?? 0}</span>
        <span>阻断：{realExecutionProductCommands?.blocked_attempt_count ?? 0}</span>
        <span>最近状态：{attemptStatusLabel(realExecutionProductCommands?.last_attempt_status)}</span>
        <span>失败 / 阻断 / 读回：{failureStopRetry?.failure_count ?? 0} / {failureStopRetry?.blocked_count ?? 0} / {failureStopRetry?.readback_issue_count ?? 0}</span>
        <span>重新确认：{failureStopRetry?.retry_requires_new_user_confirmation ? "需要重新确认" : "当前未要求"}</span>
        <span>停止请求：{failureStopRetry?.manual_stop_requested_count ?? 0}</span>
        <span>读回边界：未知 / 不可用（不可用不等于 0）</span>
        <span>适配器：{surface.adapter_id}</span>
        <span>操作：{surface.operation_id}</span>
        <span>目标会话：{surface.target_session_id ?? latestRealAttempt?.continuation_id ?? "待确认"}</span>
        <span>准备状态：{h2DecisionStatusLabel(surface.status)}</span>
        <span>尝试：{attemptStatusLabel(latestRealAttempt?.status ?? latestAttempt?.status)}</span>
        <span>读回：{readbackStatusLabel(readback?.status ?? surface.readback_boundary.status)}</span>
        <span>结果数：{readbackCountLabel(readback?.result_count ?? surface.readback_boundary.result_count)}</span>
        <span>运行 / 审计：{runtimeRefCount} / {auditRefCount}</span>
        <span>自动编排：{automationStatusLabel(projectWorkflowAutomation?.latest_status)}</span>
        <span>编排 run units：{projectWorkflowAutomation?.run_unit_count ?? 0}</span>
        <span>编排读回未知：{projectWorkflowAutomation?.readback_unknown_count ?? 0}</span>
        <span>编排捕获来源：{projectWorkflowAutomation?.capture_event_count ?? 0}</span>
      </div>
      <div className="h2-execution-decision-columns">
        <article className="h2-execution-decision-card">
          <strong>权限 / 准备状态</strong>
          <span>普通入口：{productEntryStatusLabel(realExecutionProductCommands?.ordinary_product_entry_status)}</span>
          <span>旧入口：{productEntryStatusLabel(realExecutionProductCommands?.legacy_entry_status)}</span>
          <span>Level B：{realExecutionProductCommands?.level_b_authorization_required ? "仍需单独授权" : "当前读模型未要求"}</span>
          <span>最终批准：{surface.final_approval_allowed ? "仍需明确确认" : "当前不可批准"}</span>
          <span>重复保护：{surface.duplicate_attempt_blocked ? "阻断" : "无排队 / 运行中的受控尝试"}</span>
          <span>.codex：{surface.permission_preview.codex_home_scope_summary}</span>
          <span>可写根目录：{surface.permission_preview.allowed_write_roots.join(" / ") || "待确认"}</span>
          <span>提示词：{surface.permission_preview.prompt_summary}</span>
        </article>
        <article className="h2-execution-decision-card">
          <strong>运行 / 读回</strong>
          <span>运行状态：{runtimeAttentionLabel(leadAttention?.status ?? "") || "无当前运行关注"}</span>
          <span>是否卡住：{leadAttention?.blocks_continuation ? "是" : "否"}</span>
          <span>需要权限：{leadAttention?.requires_user_action ? "是" : "否"}</span>
          <span>读回边界：{surface.readback_boundary.user_message}</span>
          <span>项目工作流尝试：{projectAttemptCount}</span>
        </article>
      </div>
      {failureStopRetryItems.length ? (
        <div className="workflow-compact-list">
          {failureStopRetryItems.map((item) => (
            <div className="workflow-compact-item" key={item.kind}>
              <strong>{item.title}</strong>
              <span>{item.summary}</span>
              <em>
                {item.count} 条 · {item.requires_new_user_confirmation ? "需要重新确认" : "只读查看"} · 结果数：{readbackCountLabel(item.result_count)}
              </em>
            </div>
          ))}
        </div>
      ) : (
        <p className="muted small-note">当前统一执行链路没有失败、停止或重试相关产品状态。</p>
      )}
      {projectWorkflowAutomation?.latest_plan ? (
        <div className="workflow-compact-list">
          {automationUnits.slice(0, 3).map((unit) => (
            <div className="workflow-compact-item" key={unit.run_unit_id}>
              <strong>{automationRunUnitLabel(unit.run_unit_kind)} · {automationUnitStatusLabel(unit.status)}</strong>
              <span>
                {unit.worker_report_ref ? "worker report 已回收" : unit.summary}
                {unit.capture_event_refs.length ? `；捕获来源 ${unit.capture_event_refs.length}` : ""}
              </span>
              <em>读回 {readbackStatusLabel(unit.readback_status)} · 结果数：{readbackCountLabel(unit.readback_result_count)}</em>
            </div>
          ))}
          <p className="muted small-note">{projectWorkflowAutomation.next_step ?? projectWorkflowAutomation.latest_plan.next_step}</p>
        </div>
      ) : (
        <p className="muted small-note">当前没有关联的项目自动编排摘要。</p>
      )}
      <details className="agent-boundary-details nested-boundary-details">
        <summary className="agent-boundary-summary">开发者详情：统一命令读模型</summary>
        <div className="h2-execution-decision-summary">
          <span>store revision：{realExecutionProductCommands?.store_revision ?? 0}</span>
          <span>sidecar：{realExecutionProductCommands?.sidecar_name ?? "未生成"}</span>
          <span>store：{realExecutionProductCommands?.store_available ? "可用" : "不可用 / 未生成"}</span>
          <span>runner：{productEntryStatusLabel(realExecutionProductCommands?.runner_entry_status)}</span>
        </div>
        {failureStopRetryItems.length ? (
          <div className="h2-execution-decision-summary">
            {failureStopRetryItems.map((item) => (
              <span key={item.kind}>
                {item.kind} · refs {item.source_refs.join(" / ") || "无"} · warnings {item.warnings.join(" / ") || "无"}
              </span>
            ))}
          </div>
        ) : null}
      </details>
      <div className="h2-execution-decision-warnings">
        <span>统一执行链路不新增一级入口。</span>
        <span>读回不可用 / 失败 / 超时保持结果数未知，不显示为 0 条。</span>
        <span>工作者汇报、过程事实 和记忆候选仍需主管确认，不自动写正式事实或正式记忆。</span>
      </div>
    </section>
  );
}
