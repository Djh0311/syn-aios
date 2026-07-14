import { Badge } from "../../components/Badge";
import { formatDate, pathTail } from "../../lib/format";
import { summarizePlanAuthorizationStore } from "../../lib/planAuthorization";
import { summarizeProjectConsultationProposalStore } from "../../lib/projectConsultationProposal";
import type {
  PendingAction,
  ProjectRecord,
  SessionRecord,
  TaskDraftSummary,
  WorkflowStateSnapshot,
} from "../../lib/types";
import {
  buildGlobalFinalResultReviewRequest,
  buildProcessFactDecisionRequest,
  buildStageCAcceptanceSummaryRequest,
  buildUserResultDecisionRequest,
  buildWorkerStructuredReportRequest,
  dedupeUiStrings,
  directorDecisionLabel,
  directorReviewSummary,
  dispatchNodeIdForWorkItem,
  globalFinalReviewActionLabel,
  globalFinalReviewStatusLabel,
  permissionDecisionLabel,
  permissionStatusLabel,
  processFactDecisionLabel,
  projectWorkflowDispatchesForCurrentWorkItem,
  userResultDecisionActionLabel,
  userResultDecisionStatusLabel,
  workflowNodeLabel,
} from "./ProjectWorkflowExecutionHelpers";
import {
  DetailLine,
  roleLabel,
  stateActionLabel,
  stateLabel,
} from "./projectWorkflowLabels";

export { ProjectUnifiedExecutionStateCard } from "./ProjectWorkflowUnifiedExecutionCard";

export function WorkItemOrchestrationCard({
  project,
  projectId,
  sessions,
  bindings,
  dispatches,
  directorReviews,
  executionControls,
  permissionRequests,
  executionAttempts,
  derivedWorkflow,
  projectConsultationProposalSummary,
  planAuthorizationSummary,
  workflowRevision,
  observationStoreRevision,
  workItem,
  onRequestAction,
  onOpenAgentSession,
}: {
  project: ProjectRecord;
  projectId: string;
  sessions: SessionRecord[];
  bindings: WorkflowStateSnapshot["project_workflows"][number]["node_session_bindings"];
  dispatches: WorkflowStateSnapshot["project_workflows"][number]["node_dispatches"];
  directorReviews: WorkflowStateSnapshot["project_workflows"][number]["director_reviews"];
  executionControls: WorkflowStateSnapshot["project_workflows"][number]["execution_controls"];
  permissionRequests: WorkflowStateSnapshot["project_workflows"][number]["permission_requests"];
  executionAttempts: WorkflowStateSnapshot["project_workflows"][number]["execution_attempts"];
  derivedWorkflow: NonNullable<WorkflowStateSnapshot["project_workflows"][number]["derived_workflow"]> | null;
  projectConsultationProposalSummary: ReturnType<typeof summarizeProjectConsultationProposalStore>;
  planAuthorizationSummary: ReturnType<typeof summarizePlanAuthorizationStore>;
  workflowRevision: number | null;
  observationStoreRevision: number;
  workItem: TaskDraftSummary;
  onRequestAction: (action: PendingAction) => void;
  onOpenAgentSession: (threadId: string) => void;
}) {
  const currentNodeId = workItem.current_node_id || "";
  const dispatchNodeId = dispatchNodeIdForWorkItem(workItem);
  const currentBinding =
    bindings.find((binding) => binding.node_id === dispatchNodeId && binding.work_item_id === workItem.work_item_id) ??
    bindings.find((binding) => binding.node_id === dispatchNodeId && !binding.work_item_id) ??
    bindings.find((binding) => binding.node_id === currentNodeId && binding.work_item_id === workItem.work_item_id) ??
    bindings.find((binding) => binding.node_id === currentNodeId && !binding.work_item_id) ??
    null;
  const projectSessions = sessions.filter((session) => session.project_root === project.project_root);
  const candidateSessions = projectSessions.length ? projectSessions : sessions;
  const recentDispatch =
    projectWorkflowDispatchesForCurrentWorkItem(dispatches, workItem.workflow_id, workItem.work_item_id)[0] ?? null;
  const completedDispatch = dispatches.find(
    (dispatch) =>
      dispatch.workflow_id === workItem.workflow_id &&
      dispatch.work_item_id === workItem.work_item_id &&
      dispatch.state === "completed",
  );
  const recentDirectorReview =
    directorReviews.find(
      (review) => review.workflow_id === workItem.workflow_id && review.work_item_id === workItem.work_item_id,
    ) ?? null;
  const executionControl =
    executionControls.find((control) => control.workflow_id === workItem.workflow_id && control.work_item_id === workItem.work_item_id) ??
    null;
  const workItemPermissionRequests = permissionRequests.filter(
    (request) => request.workflow_id === workItem.workflow_id && request.work_item_id === workItem.work_item_id,
  );
  const workItemExecutionAttempts = executionAttempts.filter(
    (attempt) => attempt.workflow_id === workItem.workflow_id && attempt.work_item_id === workItem.work_item_id,
  );
  const userReviewedInstruction = executionControl?.user_reviewed_instruction ?? null;

  return (
    <article className="work-item-orchestration-card">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">当前工作项</p>
          <h3>{workItem.title}</h3>
          <details className="agent-boundary-details">
            <summary className="agent-boundary-summary">开发者详情</summary>
            <p className="path-text">工作项 ID：{workItem.work_item_id}</p>
          </details>
        </div>
        <Badge tone={workItem.state === "accepted" ? "candidate" : "unknown"}>{stateLabel(workItem.state)}</Badge>
      </div>
      <div className="workflow-draft-grid">
        <DetailLine label="负责角色" value={roleLabel(workItem.assigned_role_id)} />
        <DetailLine label="当前位置" value={workflowNodeLabel(workItem.current_node_id)} />
        <DetailLine label="派发位置" value={workflowNodeLabel(dispatchNodeId)} />
        <DetailLine label="下一步" value={workItem.next_action_label || "缺少状态规则"} />
        {/* 会话绑定值隐掉机器码 project_binding_source（如 index_inferred）后缀，只留中文会话标题；中文字段保留。 */}
        <DetailLine
          label="会话绑定"
          value={currentBinding ? currentBinding.session_title : "未绑定；请选择已有 Codex 会话"}
        />
      </div>
      <details className="work-item-secondary-fold">
        <summary>
          <span>会话绑定</span>
          <em>选择 / 解除节点 Codex 会话（次要，默认收起）</em>
        </summary>
      <div className="node-session-binding-box">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">节点会话绑定</p>
            <h3>{currentBinding ? "派发位置已有绑定" : "选择已有 Codex 会话"}</h3>
          </div>
          <Badge tone={currentBinding ? "candidate" : "unknown"}>{currentBinding ? currentBinding.binding_source : "未绑定"}</Badge>
        </div>
        {currentBinding ? (
          <div className="binding-current-card">
            <strong>{currentBinding.session_title}</strong>
            <details className="project-inline-dev-detail">
              <summary>会话标识</summary>
              <span>{currentBinding.native_thread_id}</span>
            </details>
            <span>更新时间：{formatDate(currentBinding.session_updated_at_ms)}</span>
            <span>读取状态：{currentBinding.rollout_exists ? "可读取" : "缺回放记录"}</span>
            {currentBinding.warnings.length ? <em>警告：{currentBinding.warnings.join("，")}</em> : null}
            <div className="workflow-state-actions">
              <button className="secondary-button" type="button" onClick={() => onOpenAgentSession(currentBinding.native_thread_id)}>
                打开会话
              </button>
              <button
                className="secondary-button"
                type="button"
                onClick={() =>
                  onRequestAction({
                    kind: "unbind-node-session",
                    label: "解除节点会话绑定",
                    path: project.project_root,
                    source: "索引内项目路径",
                    boundary:
                      "只解除工作台自己的 workflow-state.v0.json 绑定并追加审计事件；不删除、不移动、不归档 Codex 原始会话；不写 .codex 或 Codex 状态库。",
                    nodeSessionUnbinding: {
                      project_root: project.project_root,
                      binding_id: currentBinding.binding_id,
                    },
                  })
                }
              >
                解除绑定
              </button>
            </div>
          </div>
        ) : null}
        <div className="binding-candidate-list" aria-label="候选 Codex 会话">
          {candidateSessions.slice(0, 4).map((session) => (
            <button
              className="binding-candidate-item"
              key={session.thread_id}
              type="button"
              onClick={() =>
                onRequestAction({
                  kind: "bind-node-session",
                  label: "绑定节点 Codex 会话",
                  path: project.project_root,
                  source: "索引内项目路径",
                  boundary:
                    "只把已有索引 Codex 会话绑定到工作台自己的 workflow-state.v0.json；不启动 Codex、不发送消息、不恢复会话、不读取完整会话正文、不写 Codex 状态库。",
                  nodeSessionBinding: {
                    project_root: project.project_root,
                    node_id: dispatchNodeId,
                    work_item_id: workItem.work_item_id,
                    thread_id: session.thread_id,
                  },
                })
              }
            >
              <strong>{session.title || "未知标题"}</strong>
              <span>{session.project_root ? pathTail(session.project_root) : "未关联项目"}</span>
              <em>项目归属来源：索引推断 / {session.rollout_exists ? "可读取" : "缺回放记录"}</em>
            </button>
          ))}
          {!candidateSessions.length ? <p className="muted small-note">当前索引没有可绑定的 Codex 会话。</p> : null}
        </div>
      </div>
      </details>
      {/* B·保留：派发指令框含「派发结果」(recentDispatch)——KEEP 常驻。 */}
      <div className="node-dispatch-box">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">派发指令</p>
            <h3>{currentBinding ? "节点派发" : "缺少节点会话绑定"}</h3>
          </div>
        </div>
        {userReviewedInstruction ? (
          <div className="dispatch-preview-block">
            <span>用户审核业务指令</span>
            <strong>{userReviewedInstruction.summary || "未填写摘要"}</strong>
            <em>执行目录：{userReviewedInstruction.execution_cwd || "未登记"}</em>
            <em>沙箱模式：{userReviewedInstruction.sandbox_mode || "未登记"}</em>
            <em>允许写入根目录：{userReviewedInstruction.allowed_write_roots.join("；") || "未登记"}</em>
            <em>允许读取：{userReviewedInstruction.allowed_reads.join("；") || "未登记"}</em>
            <em>允许写入：{userReviewedInstruction.allowed_writes.join("；") || "未登记"}</em>
            <em>禁止事项：{userReviewedInstruction.forbidden_actions.join("；") || "未登记"}</em>
            <em>超时 / 重试：{userReviewedInstruction.timeout_seconds ?? "未登记"} 秒 / {userReviewedInstruction.max_retries} 次</em>
          </div>
        ) : null}
        {recentDispatch ? (
          <div className="dispatch-result-card">
            <strong>{recentDispatch.state}</strong>
            <span>退出码：{recentDispatch.exit_code ?? "未完成"}</span>
            <span>事件：{recentDispatch.transcript_event_count ?? "未回读"} / 命中：{recentDispatch.transcript_target_hits ?? "未回读"}</span>
            {recentDispatch.last_message_summary ? <em>{recentDispatch.last_message_summary}</em> : null}
            {recentDispatch.warnings.length ? <em>警告：{recentDispatch.warnings.join("，")}</em> : null}
            <details className="agent-boundary-details">
              <summary className="agent-boundary-summary">开发者详情</summary>
              <span>派发 ID：{recentDispatch.dispatch_id}</span>
              <span>会话：{recentDispatch.native_thread_id}</span>
            </details>
          </div>
        ) : (
          <p className="muted small-note">当前工作项还没有节点派发记录。</p>
        )}
      </div>
      {/* B·折：权限请求队列（安全闸，次要块默认收起）。可控执行协议 / 执行尝试明细已裁掉。 */}
      <details className="work-item-secondary-fold">
        <summary>
          <span>权限请求队列</span>
          <em>会话绑定外的权限请求安全闸（默认收起）</em>
        </summary>
      <ExecutionControlPanel
        control={executionControl}
        attempts={workItemExecutionAttempts}
        permissionRequests={workItemPermissionRequests}
        projectRoot={project.project_root}
        workItem={workItem}
        onRequestAction={onRequestAction}
      />
      </details>
      <ProcessFactConfirmationPanel
        project={project}
        projectId={projectId}
        workItem={workItem}
        dispatchNodeId={dispatchNodeId}
        recentDispatch={recentDispatch}
        derivedWorkflow={derivedWorkflow}
        permissionRequests={workItemPermissionRequests}
        executionAttempts={workItemExecutionAttempts}
        workflowRevision={workflowRevision}
        observationStoreRevision={observationStoreRevision}
        onRequestAction={onRequestAction}
      />
      {/* B·折：C6 结果 / 阶段验收（全局复核 / 用户决定）次要，默认收起。汇报(C5)与回收(总指导)保持常驻。 */}
      <details className="work-item-secondary-fold">
        <summary>
          <span>C6 结果 / 全局复核 / 用户决定</span>
          <em>阶段验收门禁与最终复核次要块（默认收起）</em>
        </summary>
      <WorkflowResultSummaryPanel
        project={project}
        projectId={projectId}
        workItem={workItem}
        derivedWorkflow={derivedWorkflow}
        projectConsultationProposalSummary={projectConsultationProposalSummary}
        planAuthorizationSummary={planAuthorizationSummary}
        workflowRevision={workflowRevision}
        onRequestAction={onRequestAction}
      />
      </details>
      <div className="director-review-box">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">总指导回收</p>
            <h3>{recentDirectorReview ? directorDecisionLabel(recentDirectorReview.decision) : "记录派发结果判断"}</h3>
          </div>
          <Badge tone={workItem.state === "ready_for_review" ? "candidate" : "unknown"}>
            {workItem.state === "ready_for_review" ? "待回收" : stateLabel(workItem.state)}
          </Badge>
        </div>
        {/* 简化：只留一句「收没收」的中文状态，砍掉派发结果 / 复核明细卡（含机器码、ID、命中数等）。 */}
        {completedDispatch ? (
          recentDirectorReview ? (
            <p className="muted small-note">已回收：{directorDecisionLabel(recentDirectorReview.decision)}。</p>
          ) : (
            <p className="muted small-note">已完成派发，待回收结果。</p>
          )
        ) : (
          <p className="state-warning">当前工作项还没有已完成派发记录，不能记录总指导回收。</p>
        )}
        <div className="workflow-state-actions" aria-label="总指导回收动作">
          {(["accepted", "needs_changes", "paused", "discarded"] as const).map((decision) => (
            <button
              className="secondary-button"
              key={decision}
              type="button"
              disabled={!completedDispatch || workItem.state !== "ready_for_review"}
              onClick={() => {
                if (!completedDispatch) return;
                onRequestAction({
                  kind: "record-director-review",
                  label: `记录总指导回收：${directorDecisionLabel(decision)}`,
                  path: project.project_root,
                  source: "索引内项目路径",
                  boundary:
                    "只写真实 workflow-state.v0.json 的复核记录和审计事件；不启动 Codex、不恢复会话、不发送消息、不写 /Users/yoyi/.codex、不读取完整会话记录。",
                  directorReview: {
                    project_root: project.project_root,
                    work_item_id: workItem.work_item_id,
                    dispatch_id: completedDispatch.dispatch_id,
                    decision,
                    summary: directorReviewSummary(decision, completedDispatch),
                  },
                });
              }}
            >
              {directorDecisionLabel(decision)}
            </button>
          ))}
        </div>
      </div>
      <div className="workflow-state-actions" aria-label="工作项下一步动作">
        {workItem.next_states.map((nextState) => (
          <button
            className="secondary-button"
            key={nextState}
            type="button"
            onClick={() =>
              onRequestAction({
                kind: "advance-work-item-state",
                label: `推进工作项到${stateLabel(nextState)}`,
                path: project.project_root,
                source: "索引内项目路径",
                boundary:
                  "只写工作台自己的 workflow-state.v0.json；追加审计事件；不启动 Codex 命令行、不恢复会话、不派发真实 Codex 会话、不运行运行器、不写 .codex 或 Codex 状态库。",
                workItemStateUpdate: {
                  project_root: project.project_root,
                  work_item_id: workItem.work_item_id,
                  next_state: nextState,
                },
              })
            }
          >
            {stateActionLabel(nextState)}
          </button>
        ))}
      </div>
      {/* 裁掉「最近审计事件」整块（审计明细列表）——只动展示层。 */}
    </article>
  );
}

function ProcessFactConfirmationPanel({
  project,
  projectId,
  workItem,
  dispatchNodeId,
  recentDispatch,
  derivedWorkflow,
  permissionRequests,
  executionAttempts,
  workflowRevision,
  observationStoreRevision,
  onRequestAction,
}: {
  project: ProjectRecord;
  projectId: string;
  workItem: TaskDraftSummary;
  dispatchNodeId: string;
  recentDispatch: WorkflowStateSnapshot["project_workflows"][number]["node_dispatches"][number] | null;
  derivedWorkflow: NonNullable<WorkflowStateSnapshot["project_workflows"][number]["derived_workflow"]> | null;
  permissionRequests: WorkflowStateSnapshot["project_workflows"][number]["permission_requests"];
  executionAttempts: WorkflowStateSnapshot["project_workflows"][number]["execution_attempts"];
  workflowRevision: number | null;
  observationStoreRevision: number;
  onRequestAction: (action: PendingAction) => void;
}) {
  const reports = (derivedWorkflow?.subagent_reports ?? []).filter(
    (report) => report.workflow_node_id === dispatchNodeId || report.workflow_node_id === workItem.current_node_id,
  );
  const latestReport = reports[0] ?? null;
  const processFactReviews = (derivedWorkflow?.review_results ?? []).filter(
    (review) => review.reviewer_role === "project_director" && reports.some((report) => report.report_id === review.report_id),
  );
  const confirmedFactCount = processFactReviews.reduce((count, review) => count + review.observation_ids.length, 0);
  const pendingConfirmationCount = Math.max(reports.length - processFactReviews.filter((review) => review.result === "process_fact_confirmed").length, 0);
  const openIssues = [
    ...reports.flatMap((report) => report.open_issues),
    ...reports.flatMap((report) => report.direction_risks),
    ...permissionRequests.filter((request) => request.status === "pending").map((request) => request.reason || request.request_id),
    ...executionAttempts.filter((attempt) => ["failed", "timed_out", "cancelled"].includes(attempt.state)).map((attempt) => attempt.failure_reason || attempt.state),
  ].filter(Boolean);
  const latestReportAlreadyConfirmed = Boolean(
    latestReport &&
      processFactReviews.some(
        (review) => review.report_id === latestReport.report_id && review.result === "process_fact_confirmed",
      ),
  );
  const canRecordReport = Boolean(recentDispatch);
  const canDecideReport = Boolean(latestReport);

  return (
    <div className="node-dispatch-box">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">C5 工作者汇报 / 过程事实</p>
          <h3>{latestReportAlreadyConfirmed ? "过程事实已确认" : latestReport ? "待主管确认" : "等待工作者汇报"}</h3>
        </div>
        <Badge tone={openIssues.length ? "warning" : confirmedFactCount ? "candidate" : "unknown"}>
          {confirmedFactCount ? "已记录观察" : pendingConfirmationCount ? "待确认" : "准备中"}
        </Badge>
      </div>
      {/* 简化：只留最新一条工作者汇报的中文摘要，砍数字栅格 + 过程事实复核明细 + 未决问题列表（含机器码 / 角色码 / 证据 ref）。 */}
      {latestReport ? (
        <div className="dispatch-result-card">
          <span>{latestReport.summary}</span>
        </div>
      ) : (
        <p className="muted small-note">当前工作项还没有工作者结构化汇报；准备派发不能解释为真实工作者产出。</p>
      )}
      <div className="workflow-state-actions" aria-label="C5 工作者汇报动作">
        <button
          className="secondary-button"
          type="button"
          disabled={!canRecordReport}
          onClick={() => {
            if (!recentDispatch) return;
            onRequestAction({
              kind: "record-worker-structured-report",
              label: "记录工作者结构化汇报",
              path: project.project_root,
              source: "索引内项目路径",
              boundary:
                "只写工作者汇报审计事件；不启动工作者、不执行 Codex、不读取完整会话记录、不把汇报写成正式事实或正式记忆。",
              workerStructuredReport: buildWorkerStructuredReportRequest({
                project,
                projectId,
                workItem,
                dispatch: recentDispatch,
                dispatchNodeId,
                workflowRevision,
              }),
            });
          }}
        >
          记录汇报
        </button>
        <button
          className="secondary-button"
          type="button"
          disabled={!canDecideReport || latestReportAlreadyConfirmed}
          onClick={() => {
            if (!latestReport) return;
            onRequestAction({
              kind: "record-project-director-process-fact-decision",
              label: "确认为过程事实",
              path: project.project_root,
              source: "索引内项目路径",
              boundary:
                "只确认低风险本项目过程事实并写入观察存储；不写正式记忆，不完成最终验收，不启动工作者。",
              processFactDecision: buildProcessFactDecisionRequest({
                project,
                projectId,
                workItem,
                report: latestReport,
                dispatch: recentDispatch,
                decision: "confirm_process_fact",
                workflowRevision,
                observationStoreRevision,
              }),
            });
          }}
        >
          确认为过程事实
        </button>
        {(["request_rework", "block_and_escalate"] as const).map((decision) => (
          <button
            className="secondary-button"
            key={decision}
            type="button"
            disabled={!canDecideReport}
            onClick={() => {
              if (!latestReport) return;
              onRequestAction({
                kind: "record-project-director-process-fact-decision",
                label: processFactDecisionLabel(decision),
                path: project.project_root,
                source: "索引内项目路径",
                boundary:
                  "只写项目主管过程事实决定和审计事件；不写观察，不写正式记忆，不启动工作者。",
                processFactDecision: buildProcessFactDecisionRequest({
                  project,
                  projectId,
                  workItem,
                  report: latestReport,
                  dispatch: recentDispatch,
                  decision,
                  workflowRevision,
                  observationStoreRevision,
                }),
              });
            }}
          >
            {processFactDecisionLabel(decision)}
          </button>
        ))}
      </div>
    </div>
  );
}

function WorkflowResultSummaryPanel({
  project,
  projectId,
  workItem,
  derivedWorkflow,
  projectConsultationProposalSummary,
  planAuthorizationSummary,
  workflowRevision,
  onRequestAction,
}: {
  project: ProjectRecord;
  projectId: string;
  workItem: TaskDraftSummary;
  derivedWorkflow: NonNullable<WorkflowStateSnapshot["project_workflows"][number]["derived_workflow"]> | null;
  projectConsultationProposalSummary: ReturnType<typeof summarizeProjectConsultationProposalStore>;
  planAuthorizationSummary: ReturnType<typeof summarizePlanAuthorizationStore>;
  workflowRevision: number | null;
  onRequestAction: (action: PendingAction) => void;
}) {
  const resultSummary = derivedWorkflow?.result_summary ?? null;
  const stageSummary = resultSummary?.stage_c_acceptance ?? null;
  const confirmedFactIds = dedupeUiStrings(
    (derivedWorkflow?.review_results ?? [])
      .filter((review) => review.reviewer_role === "project_director" && review.result === "process_fact_confirmed")
      .flatMap((review) => review.accepted_fact_ids),
  );
  const hasProcessFactDecision = (derivedWorkflow?.review_results ?? []).some(
    (review) =>
      review.reviewer_role === "project_director" &&
      ["process_fact_confirmed", "rework_requested", "blocked_and_escalated"].includes(review.result),
  );
  const c5Issues = [
    ...(derivedWorkflow?.subagent_reports ?? []).flatMap((report) => report.open_issues),
    ...(derivedWorkflow?.subagent_reports ?? []).flatMap((report) => report.direction_risks),
  ].filter(Boolean);
  const openItems = dedupeUiStrings([
    ...(resultSummary?.open_issues ?? []),
    ...(stageSummary?.open_blockers ?? []),
    ...c5Issues,
  ]);
  const deferredItems = dedupeUiStrings(resultSummary?.deferred_items ?? stageSummary?.deferred_items ?? []);
  // 门禁计数：仅 Badge tone 还要 blocked / needs_changes 两项（栅格已砍）。
  const blockedCount = stageSummary?.gates.filter((gate) => gate.status === "blocked").length ?? 0;
  const needsChangesCount = stageSummary?.gates.filter((gate) => gate.status === "needs_changes").length ?? 0;
  const proposal = projectConsultationProposalSummary.latest_proposal;
  const authorization = projectConsultationProposalSummary.linked_plan_authorization;
  const canRecordGlobalReview = Boolean(
    proposal?.status === "user_confirmed" &&
      authorization?.status === "active" &&
      planAuthorizationSummary.active_authorization_id === authorization.authorization_id &&
      hasProcessFactDecision,
  );
  const finalReviewAccepted = resultSummary?.final_review_status === "accepted";
  const canRecordUserDecision = Boolean(resultSummary?.final_review_id);
  const canGenerateStageSummary = Boolean(derivedWorkflow);

  return (
    <div className="node-dispatch-box">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">C6 结果 / 阶段验收</p>
          <h3>{stageSummary?.accepted_as_stage_c_complete ? "阶段 C 验收门禁已通过" : finalReviewAccepted ? "等待用户结果决定 / 门禁摘要" : "待全局主管复核"}</h3>
        </div>
        <Badge tone={stageSummary?.accepted_as_stage_c_complete ? "candidate" : blockedCount || needsChangesCount ? "warning" : "unknown"}>
          {stageSummary?.accepted_as_stage_c_complete ? "阶段 C 已验收" : resultSummary?.final_review_status === "pending" ? "待复核" : "进行中"}
        </Badge>
      </div>
      {/* 简化：只留「结果接没接受」的最终复核 / 用户决定一句话 + 动作按钮；
          砍数字栅格（阶段门禁 / 过程事实计数）、阶段门禁摘要卡、未决项 / 后置项列表。 */}
      {resultSummary?.final_review_id ? (
        <div className="dispatch-result-card">
          <strong>全局主管已完成最终复核</strong>
          <span>{globalFinalReviewStatusLabel(resultSummary.final_review_status)}</span>
          <details className="agent-boundary-details">
            <summary className="agent-boundary-summary">开发者详情</summary>
            <span>复核 ID：{resultSummary.final_review_id}</span>
          </details>
        </div>
      ) : (
        <p className="muted small-note">全局主管尚未记录最终复核；C5 观察只能作为过程事实证据，仍不是正式记忆。</p>
      )}
      {resultSummary?.user_decision_id ? (
        <div className="dispatch-result-card">
          <strong>用户已查看结果并作出决定</strong>
          <span>{userResultDecisionStatusLabel(resultSummary.user_decision_status)}</span>
          <details className="agent-boundary-details">
            <summary className="agent-boundary-summary">开发者详情</summary>
            <span>决定 ID：{resultSummary.user_decision_id}</span>
          </details>
        </div>
      ) : (
        <p className="muted small-note">用户结果决定尚未记录；全局最终复核不能自动代表用户接受。</p>
      )}
      {/* 批1·恢复被砍证据渲染(体检 P0:「支撑判断的证据 UI 根本不存在」)——门禁逐项 ✓✗ + 未决/后置明细。
          宪法交货态:唯一问题=能信吗·证据呢;evidence_refs 等机器细节仍留开发者下钻,不上主脸(DESIGN.md 禁令②)。 */}
      {stageSummary ? (
        <div className="dispatch-result-card" aria-label="验收门禁逐项">
          <strong>验收门禁</strong>
          <ul className="jiaoban-step-report" aria-label="每道门禁的结论">
            {stageSummary.gates.map((gate) => {
              const tone =
                gate.status === "passed" ? "green" : gate.status === "blocked" ? "red" : gate.status === "deferred" ? "gray" : "yellow";
              const word =
                gate.status === "passed"
                  ? "✓ 过"
                  : gate.status === "blocked"
                    ? "✗ 卡住"
                    : gate.status === "needs_changes"
                      ? "⚠ 要改"
                      : gate.status === "deferred"
                        ? "后置"
                        : "⚠ 缺证据";
              return (
                <li key={gate.gate_id} className={`jiaoban-step-row tone-${tone}`}>
                  <span className="jiaoban-step-title">{gate.label}</span>
                  {gate.reason ? <span className="jiaoban-step-say">{gate.reason}</span> : null}
                  <span className={`jiaoban-step-badge tone-${tone}`}>{word}</span>
                </li>
              );
            })}
          </ul>
        </div>
      ) : null}
      {openItems.length ? (
        <div className="dispatch-result-card" aria-label="未决项">
          <strong>还没解决的（{openItems.length}）</strong>
          <ul className="jiaoban-warnings" aria-label="未决项明细">
            {openItems.map((item, index) => (
              <li key={index}>{item}</li>
            ))}
          </ul>
        </div>
      ) : null}
      {deferredItems.length ? (
        <div className="dispatch-result-card" aria-label="后置项">
          <strong>说好以后做的（{deferredItems.length}）</strong>
          <ul className="jiaoban-warnings" aria-label="后置项明细">
            {deferredItems.map((item, index) => (
              <li key={index}>{item}</li>
            ))}
          </ul>
        </div>
      ) : null}
      <div className="workflow-state-actions" aria-label="C6 结果复核动作">
        {(["accepted", "needs_changes", "blocked"] as const).map((decision) => (
          <button
            className="secondary-button"
            key={decision}
            type="button"
            disabled={!canRecordGlobalReview || (decision === "accepted" && !confirmedFactIds.length)}
            onClick={() => {
              const request = buildGlobalFinalResultReviewRequest({
                project,
                projectId,
                workItem,
                derivedWorkflow,
                proposal,
                authorization,
                decision,
                workflowRevision,
                openItems,
                deferredItems,
              });
              if (!request) return;
              onRequestAction({
                kind: "record-global-final-result-review",
                label: `记录全局最终复核：${globalFinalReviewStatusLabel(decision)}`,
                path: project.project_root,
                source: "索引内项目路径",
                boundary:
                  "只写全局主管最终复核和审计；不代表用户已接受，不写正式记忆，不执行真实工作者。",
                globalFinalResultReview: request,
              });
            }}
          >
            {globalFinalReviewActionLabel(decision)}
          </button>
        ))}
        {(["accept_result", "request_changes", "reject_result"] as const).map((decision) => (
          <button
            className="secondary-button"
            key={decision}
            type="button"
            disabled={!canRecordUserDecision || (decision === "accept_result" && !finalReviewAccepted)}
            onClick={() => {
              const request = buildUserResultDecisionRequest({
                project,
                projectId,
                workItem,
                resultSummary,
                decision,
                workflowRevision,
              });
              if (!request) return;
              onRequestAction({
                kind: "record-user-result-decision",
                label: `记录用户结果决定：${userResultDecisionStatusLabel(decision)}`,
                path: project.project_root,
                source: "索引内项目路径",
                boundary:
                  "只记录本次用户结果决定；不代表未来任务默认接受，不写正式记忆，不执行真实工作者。",
                userResultDecision: request,
              });
            }}
          >
            {userResultDecisionActionLabel(decision)}
          </button>
        ))}
        <button
          className="secondary-button"
          type="button"
          disabled={!canGenerateStageSummary}
          onClick={() => {
            const request = buildStageCAcceptanceSummaryRequest({ project, projectId, workItem, workflowRevision });
            onRequestAction({
              kind: "generate-stage-c-acceptance-summary",
              label: "生成阶段 C 验收摘要",
              path: project.project_root,
              source: "索引内项目路径",
              boundary:
                "只生成阶段 C 门禁摘要产物 / 审计；不执行真实 Codex，不写正式记忆，不代表中间版本整体完成。",
              stageCAcceptanceSummary: request,
            });
          }}
        >
          生成验收摘要
        </button>
      </div>
    </div>
  );
}

function ExecutionControlPanel({
  control,
  attempts,
  permissionRequests,
  projectRoot,
  workItem,
  onRequestAction,
}: {
  control: WorkflowStateSnapshot["project_workflows"][number]["execution_controls"][number] | null;
  attempts: WorkflowStateSnapshot["project_workflows"][number]["execution_attempts"];
  permissionRequests: WorkflowStateSnapshot["project_workflows"][number]["permission_requests"];
  projectRoot: string;
  workItem: TaskDraftSummary;
  onRequestAction: (action: PendingAction) => void;
}) {
  void control;
  void attempts;
  // 裁掉「可控执行协议」整块（heading + 重试/超时/取消栅格 + 协议说明 + instruction 明细预览）
  // 与「失败 / 重试 / 超时 / 取消」整块（执行尝试记录）——只动展示层。
  // ⚠️ 安全闸保留：下面「权限请求队列」原样常驻。
  return (
    <div className="execution-control-box">
      <div className="permission-queue" aria-label="权限请求队列">
        <p className="eyebrow">权限请求队列</p>
        {permissionRequests.length ? (
          permissionRequests.map((request) => (
            <div className="permission-request-card" key={request.request_id}>
              <strong>{permissionStatusLabel(request.status)} / {request.permission_kind}</strong>
              <span>{request.reason || request.request_id}</span>
              <em>{request.requested_at || "未登记时间"}</em>
              <div className="workflow-state-actions">
                {(["approved", "rejected"] as const).map((decision) => (
                  <button
                    className="secondary-button"
                    key={decision}
                    type="button"
                    disabled={request.status !== "pending"}
                    onClick={() =>
                      onRequestAction({
                        kind: "record-permission-decision",
                        label: `记录权限结论：${permissionDecisionLabel(decision)}`,
                        path: projectRoot,
                        source: "索引内项目路径",
                        boundary:
                          "只在用户确认后通过控制核心记录权限请求结论并追加审计事件；不启动 Codex、不恢复会话、不发送消息、不写 /Users/yoyi/.codex。",
                        permissionDecision: {
                          project_root: projectRoot,
                          work_item_id: workItem.work_item_id,
                          request_id: request.request_id,
                          decision,
                        },
                      })
                    }
                  >
                    {permissionDecisionLabel(decision)}
                  </button>
                ))}
              </div>
            </div>
          ))
        ) : (
          <p className="muted small-note">当前没有待展示的权限请求。</p>
        )}
      </div>
      {/* 裁掉「失败 / 重试 / 超时 / 取消」整块（执行尝试记录列表）——只动展示层。 */}
    </div>
  );
}
