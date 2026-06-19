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
  executionControlStateLabel,
  failureVisibilityLabel,
  globalFinalReviewActionLabel,
  globalFinalReviewStatusLabel,
  permissionDecisionLabel,
  permissionStatusLabel,
  permissionVisibilityLabel,
  processFactDecisionLabel,
  processFactReviewLabel,
  projectWorkflowDispatchesForCurrentWorkItem,
  readbackVisibilityLabel,
  stageGateStatusLabel,
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
          <p className="path-text">{workItem.work_item_id}</p>
        </div>
        <Badge tone={workItem.state === "accepted" ? "candidate" : "unknown"}>{stateLabel(workItem.state)}</Badge>
      </div>
      <div className="workflow-draft-grid">
        <DetailLine label="负责角色" value={roleLabel(workItem.assigned_role_id)} />
        <DetailLine label="当前位置" value={workflowNodeLabel(workItem.current_node_id)} />
        <DetailLine label="派发位置" value={workflowNodeLabel(dispatchNodeId)} />
        <DetailLine label="下一步" value={workItem.next_action_label || "缺少状态规则"} />
        <DetailLine
          label="会话绑定"
          value={currentBinding ? `${currentBinding.session_title} / ${currentBinding.project_binding_source}` : "未绑定；请选择已有 Codex 会话"}
        />
      </div>
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
            <span>项目归属来源：{currentBinding.project_binding_source}</span>
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
      <div className="node-dispatch-box">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">派发指令</p>
            <h3>{currentBinding ? "节点派发" : "缺少节点会话绑定"}</h3>
          </div>
          <Badge tone="unknown">旧入口已封存</Badge>
        </div>
        <div className="dispatch-preview-block">
          <span>安全探针提示词</span>
          <strong>请只回复这一句：WORKFLOW_NODE_DISPATCH_OK_2026_05_29</strong>
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
        <p className="state-warning">
          旧节点派发入口已在 K2.5 封存；项目真实执行必须走统一 Product Command，不再从这里调用 legacy wrapper。
        </p>
        <div className="workflow-state-actions">
          <button
            className="primary-button"
            type="button"
            disabled
          >
            旧安全派发已封存
          </button>
          <button
            className="secondary-button"
            type="button"
            disabled
          >
            旧业务派发已封存
          </button>
        </div>
        {recentDispatch ? (
          <div className="dispatch-result-card">
            <strong>{recentDispatch.state}</strong>
            <span>{recentDispatch.dispatch_id}</span>
            <span>会话：{recentDispatch.native_thread_id}</span>
            <span>退出码：{recentDispatch.exit_code ?? "未完成"}</span>
            <span>事件：{recentDispatch.transcript_event_count ?? "未回读"} / 命中：{recentDispatch.transcript_target_hits ?? "未回读"}</span>
            {recentDispatch.last_message_summary ? <em>{recentDispatch.last_message_summary}</em> : null}
            {recentDispatch.warnings.length ? <em>警告：{recentDispatch.warnings.join("，")}</em> : null}
          </div>
        ) : (
          <p className="muted small-note">当前工作项还没有节点派发记录。</p>
        )}
      </div>
      <div className="node-dispatch-box">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">工作流机器</p>
            <h3>总指导循环闭环</h3>
          </div>
          <Badge tone="unknown">旧入口已封存</Badge>
        </div>
        <div className="dispatch-preview-block">
          <span>闭环顺序</span>
          <strong>总指导 → 开发线 → 验证线 → 回收线 → 总指导结论 → 下一轮</strong>
          <em>目标完成才收口；否则继续下一轮，最多 3 轮。</em>
        </div>
        <p className="state-warning">旧工作流机器入口已封存；K3 自动化编排必须走统一 Product Command 主路径。</p>
        <div className="workflow-state-actions">
          <button
            className="primary-button"
            type="button"
            disabled
          >
            旧闭环已封存
          </button>
        </div>
      </div>
      <ExecutionControlPanel
        control={executionControl}
        attempts={workItemExecutionAttempts}
        permissionRequests={workItemPermissionRequests}
        projectRoot={project.project_root}
        workItem={workItem}
        onRequestAction={onRequestAction}
      />
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
        {completedDispatch ? (
          <div className="dispatch-result-card">
            <strong>{completedDispatch.last_message_summary || "无最终回复摘要"}</strong>
            <span>派发：{completedDispatch.dispatch_id}</span>
            <span>事件：{completedDispatch.transcript_event_count ?? "未回读"}</span>
            <span>命中：{completedDispatch.transcript_target_hits ?? "未回读"}</span>
            {completedDispatch.warnings.length ? <em>警告：{completedDispatch.warnings.join("，")}</em> : null}
          </div>
        ) : (
          <p className="state-warning">当前工作项还没有已完成派发记录，不能记录总指导回收。</p>
        )}
        {recentDirectorReview ? (
          <div className="dispatch-result-card">
            <strong>{directorDecisionLabel(recentDirectorReview.decision)}</strong>
            <span>{recentDirectorReview.review_id}</span>
            <em>{recentDirectorReview.summary}</em>
          </div>
        ) : null}
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
      <div className="audit-summary-list" aria-label="最近审计事件">
        <p className="eyebrow">最近审计事件</p>
        {workItem.recent_audit_events.length ? (
          workItem.recent_audit_events.map((event) => (
            <div className="audit-summary-item" key={event.event_id}>
              <strong>{event.event_type}</strong>
              <span>{stateLabel(event.before_state || "")} 到 {stateLabel(event.after_state || "")}</span>
              <em>{event.reason || event.created_at || event.event_id}</em>
            </div>
          ))
        ) : (
          <p className="muted small-note">当前工作项还没有状态推进审计事件。</p>
        )}
      </div>
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
      <div className="workflow-draft-grid">
        <DetailLine label="汇报数量" value={`${reports.length} 条`} />
        <DetailLine label="待确认事实" value={`${pendingConfirmationCount} 条`} />
        <DetailLine label="已确认事实" value={`${confirmedFactCount} 条`} />
        <DetailLine label="读回" value={readbackVisibilityLabel(recentDispatch)} />
        <DetailLine label="权限" value={permissionVisibilityLabel(permissionRequests)} />
        <DetailLine label="失败" value={failureVisibilityLabel(executionAttempts, reports)} />
      </div>
      {latestReport ? (
        <div className="dispatch-result-card">
          <strong>{latestReport.actor_role || "工作者"} / {latestReport.acceptance_status}</strong>
          <span>{latestReport.summary}</span>
          <em>证据：{latestReport.evidence_refs.join("；") || "未登记"} / 问题：{latestReport.open_issues.slice(0, 3).join("；") || "无"}</em>
        </div>
      ) : (
        <p className="muted small-note">当前工作项还没有工作者结构化汇报；准备派发不能解释为真实工作者产出。</p>
      )}
      {processFactReviews.slice(0, 2).map((review) => (
        <div className="dispatch-result-card" key={review.review_id}>
          <strong>{processFactReviewLabel(review.result)}</strong>
          <span>{review.summary || "未登记摘要"}</span>
          <em>{review.observation_ids.length ? "已记录为观察，仍不是正式记忆" : "未写观察"}</em>
        </div>
      ))}
      {openIssues.length ? (
        <ul className="state-warning-list">
          {openIssues.slice(0, 3).map((issue) => (
            <li key={issue}>{issue}</li>
          ))}
        </ul>
      ) : null}
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
  const passedCount = stageSummary?.gates.filter((gate) => gate.status === "passed").length ?? 0;
  const blockedCount = stageSummary?.gates.filter((gate) => gate.status === "blocked").length ?? 0;
  const needsChangesCount = stageSummary?.gates.filter((gate) => gate.status === "needs_changes").length ?? 0;
  const missingCount = stageSummary?.gates.filter((gate) => gate.status === "missing_evidence").length ?? 0;
  const deferredCount = stageSummary?.gates.filter((gate) => gate.status === "deferred").length ?? 0;
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
      <div className="workflow-draft-grid">
        <DetailLine label="最终复核" value={globalFinalReviewStatusLabel(resultSummary?.final_review_status ?? "pending")} />
        <DetailLine label="用户决定" value={userResultDecisionStatusLabel(resultSummary?.user_decision_status ?? "pending")} />
        <DetailLine label="阶段门禁" value={`${passedCount} 通过 / ${missingCount} 缺证据 / ${needsChangesCount} 需改 / ${blockedCount} 阻断 / ${deferredCount} 后置`} />
        <DetailLine label="过程事实" value={`${confirmedFactIds.length} 条`} />
      </div>
      {resultSummary?.final_review_id ? (
        <div className="dispatch-result-card">
          <strong>全局主管已完成最终复核</strong>
          <span>{globalFinalReviewStatusLabel(resultSummary.final_review_status)}</span>
          <em>{resultSummary.final_review_id}</em>
        </div>
      ) : (
        <p className="muted small-note">全局主管尚未记录最终复核；C5 观察只能作为过程事实证据，仍不是正式记忆。</p>
      )}
      {resultSummary?.user_decision_id ? (
        <div className="dispatch-result-card">
          <strong>用户已查看结果并作出决定</strong>
          <span>{userResultDecisionStatusLabel(resultSummary.user_decision_status)}</span>
          <em>{resultSummary.user_decision_id}</em>
        </div>
      ) : (
        <p className="muted small-note">用户结果决定尚未记录；全局最终复核不能自动代表用户接受。</p>
      )}
      {stageSummary ? (
        <div className="dispatch-result-card">
          <strong>{stageSummary.accepted_as_stage_c_complete ? "阶段 C 验收门禁已通过" : "阶段 C 门禁摘要"}</strong>
          <span>{stageSummary.gates.slice(0, 5).map((gate) => `${gate.label}：${stageGateStatusLabel(gate.status)}`).join(" / ")}</span>
          <em>{stageSummary.accepted_as_stage_c_complete ? "仍不代表中间版本整体完成" : "缺口和后置项仍需单独处理"}</em>
        </div>
      ) : null}
      {openItems.length ? (
        <ul className="state-warning-list">
          {openItems.slice(0, 5).map((issue) => (
            <li key={issue}>{issue}</li>
          ))}
        </ul>
      ) : null}
      {deferredItems.length ? (
        <ul className="state-warning-list">
          {deferredItems.slice(0, 5).map((item) => (
            <li key={item}>{item}</li>
          ))}
        </ul>
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
  const instruction = control?.user_reviewed_instruction ?? null;
  return (
    <div className="execution-control-box">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">可控执行协议</p>
          <h3>{control ? executionControlStateLabel(control.control_state) : "协议未登记"}</h3>
        </div>
        <Badge tone={control ? "candidate" : "unknown"}>{control ? executionControlStateLabel(control.long_task_state) : "只读占位"}</Badge>
      </div>
      <div className="workflow-draft-grid">
        <DetailLine label="重试" value={control ? `${control.retry_count}/${control.max_retries}` : "未登记"} />
        <DetailLine label="超时" value={control?.timeout_seconds ? `${control.timeout_seconds} 秒` : "未登记"} />
        <DetailLine label="取消" value={control?.cancel_requested_at || "未请求"} />
        <DetailLine label="失败原因" value={control?.failure_reason || "无"} />
      </div>
      <p className="muted small-note">
        这里只展示协议能力和用户审核边界；不执行真实业务任务、不恢复会话、不发送 Codex 消息。
      </p>
      {instruction ? (
        <div className="instruction-preview-card">
          <span>用户审核业务指令</span>
          <strong>{instruction.summary || "未填写摘要"}</strong>
          <em>{instruction.objective || "未填写目标"}</em>
          <pre>{instruction.preview_markdown}</pre>
          <div className="workflow-state-actions">
            <button
              className="secondary-button"
              type="button"
              onClick={() =>
                onRequestAction({
                  kind: "preview-user-reviewed-instruction",
                  label: "确认用户审核业务指令边界",
                  path: projectRoot,
                  source: "索引内项目路径",
                  boundary:
                    "只确认用户审核业务指令的结构化预览和边界；本版本不执行 codex exec resume、不发送 Codex 消息、不写 /Users/yoyi/.codex、不读取完整会话记录。",
                  userReviewedInstruction: instruction,
                })
              }
            >
              确认指令边界
            </button>
          </div>
        </div>
      ) : (
        <p className="state-warning">还没有用户审核业务指令结构；真实业务派发保持阻塞。</p>
      )}
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
      <div className="attempt-list" aria-label="执行尝试记录">
        <p className="eyebrow">失败 / 重试 / 超时 / 取消</p>
        {attempts.length ? (
          attempts.map((attempt) => (
            <div className="attempt-card" key={attempt.attempt_id}>
              <strong>第 {attempt.attempt_no} 次 / {executionControlStateLabel(attempt.state)}</strong>
              <span>{attempt.failure_reason || attempt.retry_scheduled_at || attempt.timed_out_at || attempt.cancel_requested_at || "无异常记录"}</span>
              {attempt.warnings.length ? <em>警告：{attempt.warnings.join("，")}</em> : null}
            </div>
          ))
        ) : (
          <p className="muted small-note">当前没有执行尝试记录。</p>
        )}
      </div>
    </div>
  );
}
