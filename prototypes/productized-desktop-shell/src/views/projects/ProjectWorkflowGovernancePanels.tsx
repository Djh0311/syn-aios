import { useEffect, useState } from "react";
import { Badge } from "../../components/Badge";
import {
  globalBoundaryReviewStatusLabels,
  summarizeAutoDispatchGuardResult,
  summarizeGlobalBoundaryReview,
  summarizePlanAuthorizationStore,
} from "../../lib/planAuthorization";
import {
  projectDirectorPlannedTaskStatusLabels,
  summarizeProjectDirectorTaskPlan,
} from "../../lib/projectDirectorTaskPlan";
import {
  projectConsultationProposalStatusLabels,
  summarizeProjectConsultationProposalStore,
} from "../../lib/projectConsultationProposal";
import type {
  AutoDispatchGuardResult,
  GlobalBoundaryReviewStatus,
  PendingAction,
  PreviewProjectDirectorTaskPlanInput,
  ProjectConsultationProposal,
  ProjectConsultationProposalDecisionKind,
  ProjectDirectorTaskPlan,
  ProjectRecord,
  TaskDraftSummary,
  TaskPackage,
  WorkflowStateSnapshot,
} from "../../lib/types";
import { DetailLine } from "./projectWorkflowLabels";

export function ProjectDirectorTaskPlanCard({
  project,
  request,
  plan,
  loading,
  error,
  workflowRevision,
  onPreview,
  onRequestAction,
}: {
  project: ProjectRecord;
  request: PreviewProjectDirectorTaskPlanInput | null;
  plan: ProjectDirectorTaskPlan | null;
  loading: boolean;
  error: string | null;
  workflowRevision: number | null;
  onPreview: () => void;
  onRequestAction: (action: PendingAction) => void;
}) {
  const summary = summarizeProjectDirectorTaskPlan(plan);
  const prepareBlockedReason = projectDirectorPrepareBlockedReason(request, plan, loading, error);
  const canPrepare = !prepareBlockedReason && request && plan;
  const previewDisabled = !request || loading;

  return (
    <section className="project-canvas-detail-card" aria-label="项目主管拆任务与准备派发">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">项目主管拆任务</p>
          <h3>{projectDirectorTaskPlanHeadline(request, plan, loading, error)}</h3>
        </div>
        <Badge tone={projectDirectorTaskPlanTone(plan, loading, error)}>{summary.status_label}</Badge>
      </div>
      <div className="workflow-draft-grid">
        <DetailLine label="active 授权" value={summary.active_authorization_id ?? request?.authorization_id ?? "暂无"} />
        <DetailLine label="planned" value={String(summary.planned_task_count)} />
        <DetailLine label="prepared" value={String(summary.prepared_dispatch_count)} />
        <DetailLine label="needs_binding" value={String(summary.needs_binding_count)} />
        <DetailLine label="blocked" value={String(summary.blocked_count)} />
        <DetailLine label="记忆快照" value={summary.memory_text} />
      </div>
      {error ? <p className="state-warning">拆任务草案读取失败：{error}</p> : null}
      {summary.blocked_reasons.map((reason) => (
        <p className="state-warning" key={reason}>{reason}</p>
      ))}
      {plan ? (
        <div className="workflow-compact-list" aria-label="项目主管计划任务摘要">
          {plan.planned_tasks.slice(0, 3).map((task) => (
            <div className="workflow-compact-item" key={task.planned_task_id}>
              <strong>{task.title}</strong>
              <span>{projectDirectorPlannedTaskStatusLabels[task.status] ?? task.status}</span>
              <em>{task.blocked_reasons.slice(0, 2).join("；") || `${task.scope.target_role} / ${task.scope.task_package_kind}`}</em>
            </div>
          ))}
        </div>
      ) : (
        <p className="muted small-note">生成拆任务草案后才会显示工作者子任务摘要。</p>
      )}
      <div className="workflow-state-actions">
        <button className="secondary-button" type="button" disabled={previewDisabled} onClick={onPreview}>
          {loading ? "正在生成" : "生成拆任务草案"}
        </button>
        <button
          className="primary-button"
          type="button"
          disabled={!canPrepare}
          onClick={() => {
            if (request && plan) {
              onRequestAction(
                buildPrepareAuthorizedAutoDispatchAction({
                  project,
                  request,
                  plan,
                  workflowRevision,
                }),
              );
            }
          }}
        >
          准备授权范围内派发
        </button>
      </div>
      {prepareBlockedReason ? <p className="state-warning">{prepareBlockedReason}</p> : null}
      <p className="muted small-note">只创建准备记录，不启动工作者、不执行 codex exec resume、不写 /Users/yoyi/.codex。</p>
    </section>
  );
}

function projectDirectorTaskPlanHeadline(
  request: PreviewProjectDirectorTaskPlanInput | null,
  plan: ProjectDirectorTaskPlan | null,
  loading: boolean,
  error: string | null,
) {
  if (!request) return "等待用户确认方案和全局边界复核";
  if (loading) return "正在生成拆任务草案";
  if (error) return "拆任务草案未生成";
  if (!plan) return "尚未生成项目主管拆任务草案";
  if (plan.prepared_dispatch_count > 0) return "已准备；仍未执行工作者";
  if (plan.blocked_count > 0) return "越界任务已阻断";
  if (plan.needs_binding_count > 0) return "等待会话绑定后才能准备派发";
  return "授权范围内可准备";
}

function projectDirectorTaskPlanTone(plan: ProjectDirectorTaskPlan | null, loading: boolean, error: string | null) {
  if (error || plan?.blocked_count) return "warning";
  if (loading || !plan) return "unknown";
  if (plan.prepared_dispatch_count > 0 || plan.authorized_task_count > 0) return "candidate";
  return "unknown";
}

function projectDirectorPrepareBlockedReason(
  request: PreviewProjectDirectorTaskPlanInput | null,
  plan: ProjectDirectorTaskPlan | null,
  loading: boolean,
  error: string | null,
) {
  if (!request) return "缺少 active 授权或已确认方案，不能准备派发。";
  if (loading) return "拆任务草案生成中，暂不能准备派发。";
  if (error) return "拆任务草案读取失败，暂不能准备派发。";
  if (!plan) return "请先生成拆任务草案。";
  if (plan.blocked_count > 0) return "越界任务已阻断";
  if (plan.needs_binding_count > 0) return "等待会话绑定后才能准备派发";
  if (plan.prepared_dispatch_count >= plan.planned_task_count && plan.planned_task_count > 0) {
    return "已准备；仍未执行工作者";
  }
  if (plan.planned_task_count === 0) return "没有可准备的工作者子任务。";
  return null;
}

export function buildPrepareAuthorizedAutoDispatchAction({
  project,
  request,
  plan,
  workflowRevision,
}: {
  project: ProjectRecord;
  request: PreviewProjectDirectorTaskPlanInput;
  plan: ProjectDirectorTaskPlan;
  workflowRevision: number | null;
}): PendingAction {
  return {
    kind: "prepare-authorized-auto-dispatch",
    label: "准备授权范围内派发",
    path: project.project_root,
    source: "索引内项目路径",
    boundary:
      "只创建准备派发记录、任务包草案和记忆快照；不启动工作者、不执行 codex exec resume、不写 /Users/yoyi/.codex。",
    authorizedAutoDispatch: {
      project_root: request.project_root,
      project_id: request.project_id,
      workflow_id: request.workflow_id,
      proposal_id: request.proposal_id,
      authorization_id: request.authorization_id,
      actor_id: request.actor_id,
      planned_tasks: plan.planned_tasks,
      expected_workflow_revision: workflowRevision,
      expected_authorization_revision: request.expected_authorization_revision ?? null,
    },
    authorizedAutoDispatchPreview: plan,
  };
}

export function ProjectConsultationProposalCard({
  project,
  projectWorkflow,
  selectedTask,
  selectedTaskPackage,
  summary,
  planAuthorizationRevision,
  onRequestAction,
}: {
  project: ProjectRecord;
  projectWorkflow: WorkflowStateSnapshot["project_workflows"][number] | null;
  selectedTask: TaskDraftSummary | null;
  selectedTaskPackage: TaskPackage | null;
  summary: ReturnType<typeof summarizeProjectConsultationProposalStore>;
  planAuthorizationRevision: number;
  onRequestAction: (action: PendingAction) => void;
}) {
  const proposal = summary.latest_proposal;
  const [decisionSummary, setDecisionSummary] = useState("");

  useEffect(() => {
    setDecisionSummary("");
  }, [proposal?.proposal_id]);

  const canDecide = proposal && ["draft", "pending_user_confirmation"].includes(proposal.status);
  const defaultDecisionSummary =
    "用户确认项目咨询方案范围；仍需全局主管复核后才可自动推进，本轮不会启动真实工作者。";

  return (
    <section className="project-canvas-detail-card" aria-label="项目咨询方案草案">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">项目咨询方案草案</p>
          <h3>{summary.display_text}</h3>
        </div>
        <Badge tone={proposal?.status === "user_confirmed" ? "candidate" : proposal ? "warning" : "unknown"}>
          {summary.status_label}
        </Badge>
      </div>

      {!proposal ? (
        <>
          <p className="muted small-note">还没有项目咨询方案草案。可以先创建模板草案；不会调用真实项目咨询智能体。</p>
          {projectWorkflow ? (
            <div className="workflow-state-actions">
              <button
                className="secondary-button"
                type="button"
                onClick={() =>
                  onRequestAction(
                    buildProjectConsultationProposalCreationAction(
                      project,
                      projectWorkflow,
                      selectedTask,
                      selectedTaskPackage,
                      summary.revision,
                    ),
                  )
                }
              >
                创建方案草案
              </button>
            </div>
          ) : (
            <p className="state-warning">缺少项目工作流，暂不能创建方案草案。</p>
          )}
        </>
      ) : (
        <>
          <div className="workflow-draft-grid">
            <DetailLine label="状态" value={projectConsultationProposalStatusLabels[proposal.status] ?? proposal.status} />
            <DetailLine label="目标" value={proposal.goal_summary} />
            <DetailLine label="步骤" value={String(proposal.proposed_steps.length)} />
            <DetailLine label="风险" value={String(proposal.risks.length)} />
            <DetailLine label="读写范围" value={`读 ${proposal.scope_draft.allowed_read_roots.length} / 写 ${proposal.scope_draft.allowed_write_roots.length}`} />
            <DetailLine label="工具 / 检查" value={`工具 ${proposal.scope_draft.allowed_tools.length} / 检查 ${proposal.scope_draft.allowed_checks.length}`} />
            <DetailLine label="停止条件" value={String(proposal.scope_draft.stop_conditions.length)} />
            <DetailLine
              label="授权回链"
              value={summary.linked_plan_authorization?.status ?? (proposal.plan_authorization_id ? "缺失" : "未建立")}
            />
          </div>
          <ul className="proposal-scope-list" aria-label="方案主要步骤">
            {proposal.proposed_steps.slice(0, 4).map((step) => (
              <li key={step}>{step}</li>
            ))}
          </ul>
          {summary.authorization_missing_after_confirmation ? (
            <p className="state-warning">方案已确认但缺少 C1 授权回链；不能显示为可自动推进。</p>
          ) : null}
          {proposal.status === "user_confirmed" ? (
            <p className="state-warning">已记录用户确认；仍需全局主管复核后才可自动推进。</p>
          ) : null}
          {canDecide ? (
            <>
              <label className="proposal-decision-field">
                <span>修改 / 拒绝原因</span>
                <textarea
                  value={decisionSummary}
                  onChange={(event) => setDecisionSummary(event.target.value)}
                  placeholder="要求修改或拒绝时填写原因；确认方案可留空。"
                />
              </label>
              <div className="workflow-state-actions">
                <button
                  className="primary-button"
                  type="button"
                  onClick={() =>
                    onRequestAction(
                      buildProjectConsultationProposalDecisionAction({
                        project,
                        proposal,
                        decision: "confirm",
                        summary: defaultDecisionSummary,
                        proposalStoreRevision: summary.revision,
                        planAuthorizationRevision,
                      }),
                    )
                  }
                >
                  确认方案范围
                </button>
                <button
                  className="secondary-button"
                  type="button"
                  onClick={() =>
                    onRequestAction(
                      buildProjectConsultationProposalDecisionAction({
                        project,
                        proposal,
                        decision: "request_changes",
                        summary: decisionSummary.trim() || "用户要求修改项目咨询方案草案。",
                        proposalStoreRevision: summary.revision,
                        planAuthorizationRevision,
                      }),
                    )
                  }
                >
                  要求修改
                </button>
                <button
                  className="secondary-button"
                  type="button"
                  onClick={() =>
                    onRequestAction(
                      buildProjectConsultationProposalDecisionAction({
                        project,
                        proposal,
                        decision: "reject",
                        summary: decisionSummary.trim() || "用户拒绝当前项目咨询方案草案。",
                        proposalStoreRevision: summary.revision,
                        planAuthorizationRevision,
                      }),
                    )
                  }
                >
                  拒绝方案
                </button>
              </div>
            </>
          ) : null}
          {summary.latest_decision ? <p className="muted small-note">最近决定：{summary.latest_decision.summary}</p> : null}
        </>
      )}
      <p className="muted small-note">C2 只记录方案草案和用户决定；本轮不会启动真实工作者。</p>
    </section>
  );
}

export function GlobalBoundaryReviewCard({
  project,
  projectWorkflow,
  proposalSummary,
  planAuthorizationSummary,
  guardResult,
  guardError,
  onRequestAction,
}: {
  project: ProjectRecord;
  projectWorkflow: WorkflowStateSnapshot["project_workflows"][number] | null;
  proposalSummary: ReturnType<typeof summarizeProjectConsultationProposalStore>;
  planAuthorizationSummary: ReturnType<typeof summarizePlanAuthorizationStore>;
  guardResult: AutoDispatchGuardResult | null;
  guardError: string | null;
  onRequestAction: (action: PendingAction) => void;
}) {
  const summary = summarizeGlobalBoundaryReview(proposalSummary, planAuthorizationSummary, guardResult);
  const [reviewSummary, setReviewSummary] = useState("");

  useEffect(() => {
    setReviewSummary("");
  }, [summary.authorization?.authorization_id, summary.review?.status]);

  const canReview = Boolean(projectWorkflow && summary.proposal && summary.authorization && summary.canReview);
  const approvedSummary = "全局主管复核通过方案边界；授权有效，仍未派发工作者。";

  return (
    <section className="project-canvas-detail-card" aria-label="全局边界复核">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">全局边界复核</p>
          <h3>{summary.display_text}</h3>
        </div>
        <Badge tone={summary.authorization?.status === "active" ? "candidate" : summary.review?.status === "blocked" ? "warning" : "unknown"}>
          {summary.status_label}
        </Badge>
      </div>
      <div className="workflow-draft-grid">
        <DetailLine label="用户确认" value={summary.proposal?.status === "user_confirmed" ? "已确认" : "未就绪"} />
        <DetailLine label="复核状态" value={summary.review?.status ? globalBoundaryReviewStatusLabels[summary.review.status as GlobalBoundaryReviewStatus] ?? summary.review.status : "待全局复核"} />
        <DetailLine label="授权对象" value={summary.authorization?.authorization_id ?? "未建立"} />
        <DetailLine label="有效授权" value={summary.active_authorization_id ?? "暂无"} />
        <DetailLine label="守卫验证" value={summary.guard_display_text} />
        <DetailLine label="发现" value={String(summary.findings.length)} />
      </div>
      {guardError ? <p className="state-warning">授权检查读取失败：{guardError}</p> : null}
      {summary.blocked_reasons.map((reason) => (
        <p className="state-warning" key={reason}>{reason}</p>
      ))}
      {summary.guard_reasons.map((reason) => (
        <p className="state-warning" key={reason}>{reason}</p>
      ))}
      {summary.findings.map((finding) => (
        <p className="state-warning" key={finding.finding_id}>{finding.summary}</p>
      ))}
      {canReview && summary.proposal && summary.authorization ? (
        <>
          <label className="proposal-decision-field">
            <span>修改 / 阻断原因</span>
            <textarea
              value={reviewSummary}
              onChange={(event) => setReviewSummary(event.target.value)}
              placeholder="要求修改或阻断方案时填写原因；批准并生效可留空。"
            />
          </label>
          <div className="workflow-state-actions">
            <button
              className="primary-button"
              type="button"
              onClick={() =>
                onRequestAction(
                  buildGlobalBoundaryReviewAction({
                    project,
                    proposal: summary.proposal!,
                    authorization: summary.authorization!,
                    reviewStatus: "approved",
                    summary: approvedSummary,
                    authorizationRevision: planAuthorizationSummary.revision,
                  }),
                )
              }
            >
              批准并生效
            </button>
            <button
              className="secondary-button"
              type="button"
              onClick={() =>
                onRequestAction(
                  buildGlobalBoundaryReviewAction({
                    project,
                    proposal: summary.proposal!,
                    authorization: summary.authorization!,
                    reviewStatus: "needs_changes",
                    summary: reviewSummary.trim() || "全局主管要求修改方案边界；不能自动推进。",
                    authorizationRevision: planAuthorizationSummary.revision,
                  }),
                )
              }
            >
              要求修改
            </button>
            <button
              className="secondary-button"
              type="button"
              onClick={() =>
                onRequestAction(
                  buildGlobalBoundaryReviewAction({
                    project,
                    proposal: summary.proposal!,
                    authorization: summary.authorization!,
                    reviewStatus: "blocked",
                    summary: reviewSummary.trim() || "全局主管阻断当前方案；不能自动推进。",
                    authorizationRevision: planAuthorizationSummary.revision,
                  }),
                )
              }
            >
              阻断方案
            </button>
          </div>
        </>
      ) : null}
      <p className="muted small-note">C3 只记录全局边界复核和授权状态；不会启动工作者。</p>
    </section>
  );
}

export function buildProjectConsultationProposalCreationAction(
  project: ProjectRecord,
  projectWorkflow: WorkflowStateSnapshot["project_workflows"][number],
  selectedTask: TaskDraftSummary | null,
  selectedTaskPackage: TaskPackage | null,
  proposalStoreRevision: number,
): PendingAction {
  const goal = selectedTaskPackage?.task_goal?.trim() || selectedTask?.title?.trim() || `围绕 ${project.name} 建立受控自动推进方案。`;
  const title = `项目咨询方案：${selectedTask?.title ?? projectWorkflow.title}`;
  const allowedReadRoots = uniqueNonEmpty([...(selectedTaskPackage?.allowed_read_scope ?? []), project.project_root]);
  const allowedWriteRoots = uniqueNonEmpty(selectedTaskPackage?.allowed_write_scope ?? []);
  const allowedTools = uniqueNonEmpty([...(selectedTaskPackage?.callable_tool_capabilities ?? []), "read_file"]);
  const allowedChecks = uniqueNonEmpty(selectedTaskPackage?.harness_requirements ?? []);
  const allowedRoleIds = uniqueNonEmpty([
    selectedTaskPackage?.target_role ?? null,
    selectedTask?.assigned_role_id ?? null,
    "project_director",
  ]);
  const allowedAgentIds = uniqueNonEmpty([
    selectedTaskPackage?.target_session_id ?? null,
    ...projectWorkflow.node_session_bindings.map((binding) => binding.native_thread_id),
  ]);

  return {
    kind: "create-project-consultation-proposal",
    label: "创建项目咨询方案草案",
    path: project.project_root,
    source: "索引内项目路径",
    boundary:
      "写入工作台自己的 project-proposals.v1.json 辅助状态文件；不调用真实项目咨询智能体、不启动 Codex、不执行工作者、不写 /Users/yoyi/.codex。",
    projectConsultationProposalCreation: {
      project_root: project.project_root,
      project_id: projectWorkflow.project_id,
      workflow_id: projectWorkflow.workflow_id,
      title,
      user_goal: goal,
      goal_summary: goal,
      proposed_steps: [
        "整理用户目标和项目上下文。",
        "确认允许角色、agent、读写范围、工具、检查和停止条件。",
        "用户确认方案范围后，等待全局主管做边界复核。",
        "只有后续 C3/C4 授权生效后，项目主管才可在范围内准备自动推进。",
      ],
      scope_draft: {
        allowed_role_ids: allowedRoleIds,
        allowed_agent_ids: allowedAgentIds,
        allowed_read_roots: allowedReadRoots,
        allowed_write_roots: allowedWriteRoots,
        allowed_tools: allowedTools,
        allowed_checks: allowedChecks,
        allowed_task_package_kinds: ["task_package"],
        stop_conditions: ["出现超出读写范围、权限升级、用户偏好或高风险事实时必须停下请用户确认。"],
        max_worker_dispatches: 3,
        max_runtime_minutes: 60,
      },
      risks: [
        {
          risk_id: "risk:scope-draft-needs-global-review",
          severity: "warning",
          summary: "模板草案只来自当前工作流上下文，仍需全局主管复核边界。",
          mitigation: "用户确认后不自动派发，等待 C3 全局边界复核。",
        },
      ],
      acceptance_criteria: selectedTaskPackage?.acceptance_criteria.length
        ? selectedTaskPackage.acceptance_criteria
        : ["用户能看懂方案范围，并确认或要求修改；确认后授权仍停在待全局复核。"],
      created_by_role: "project_consultant",
      actor_id: "desktop_project_consultation_template",
      expected_store_revision: proposalStoreRevision,
    },
  };
}

export function buildProjectConsultationProposalDecisionAction({
  project,
  proposal,
  decision,
  summary,
  proposalStoreRevision,
  planAuthorizationRevision,
}: {
  project: ProjectRecord;
  proposal: ProjectConsultationProposal;
  decision: ProjectConsultationProposalDecisionKind;
  summary: string;
  proposalStoreRevision: number;
  planAuthorizationRevision: number;
}): PendingAction {
  return {
    kind: "record-project-consultation-proposal-decision",
    label: proposalDecisionActionLabel(decision),
    path: project.project_root,
    source: "索引内项目路径",
    boundary:
      "写入 project-proposals.v1.json；确认方案时联动 plan-authorizations.v1.json 并停在待全局复核；不启动真实工作者、不执行 codex exec resume、不写 /Users/yoyi/.codex。",
    projectConsultationProposalDecision: {
      project_root: project.project_root,
      proposal_id: proposal.proposal_id,
      actor_id: "user",
      decision,
      summary,
      expected_proposal_store_revision: proposalStoreRevision,
      expected_plan_authorization_store_revision: planAuthorizationRevision,
    },
    projectConsultationProposalPreview: {
      title: proposal.title,
      goalSummary: proposal.goal_summary,
      allowedReadRoots: proposal.scope_draft.allowed_read_roots,
      allowedWriteRoots: proposal.scope_draft.allowed_write_roots,
      allowedTools: proposal.scope_draft.allowed_tools,
      allowedChecks: proposal.scope_draft.allowed_checks,
      stopConditions: proposal.scope_draft.stop_conditions,
    },
  };
}

function proposalDecisionActionLabel(decision: ProjectConsultationProposalDecisionKind) {
  if (decision === "confirm") return "确认方案范围";
  if (decision === "request_changes") return "要求修改项目咨询方案";
  return "拒绝项目咨询方案";
}

export function buildGlobalBoundaryReviewAction({
  project,
  proposal,
  authorization,
  reviewStatus,
  summary,
  authorizationRevision,
}: {
  project: ProjectRecord;
  proposal: ProjectConsultationProposal;
  authorization: NonNullable<ReturnType<typeof summarizeProjectConsultationProposalStore>["linked_plan_authorization"]>;
  reviewStatus: GlobalBoundaryReviewStatus;
  summary: string;
  authorizationRevision: number;
}): PendingAction {
  const findings =
    reviewStatus === "approved"
      ? []
      : [
          {
            finding_id: `finding:global-boundary-review:${reviewStatus}`,
            severity: reviewStatus === "blocked" ? ("blocking" as const) : ("warning" as const),
            summary,
            recommendation: reviewStatus === "blocked" ? "阻断后不能自动推进。" : "修改方案后再复核。",
          },
        ];
  return {
    kind: "record-global-boundary-review",
    label: globalBoundaryReviewActionLabel(reviewStatus),
    path: project.project_root,
    source: "索引内项目路径",
    boundary:
      reviewStatus === "approved"
        ? "写入 plan-authorizations.v1.json 的全局边界复核，并让授权有效；只让授权生效，不启动工作者、不执行 codex exec、不写 /Users/yoyi/.codex。"
        : "写入 plan-authorizations.v1.json 的全局边界复核，并让授权保持不可自动推进；不启动工作者、不执行 codex exec、不写 /Users/yoyi/.codex。",
    globalBoundaryReview: {
      project_root: project.project_root,
      project_id: proposal.project_id,
      workflow_id: proposal.workflow_id,
      proposal_id: proposal.proposal_id,
      authorization_id: authorization.authorization_id,
      actor_id: "global_director",
      review_status: reviewStatus,
      summary,
      checklist: completeGlobalBoundaryReviewChecklist(),
      findings,
      expected_authorization_revision: authorizationRevision,
    },
    globalBoundaryReviewPreview: {
      proposalTitle: proposal.title,
      goalSummary: proposal.goal_summary,
      reviewStatus,
      readWriteScope: `读 ${authorization.scope.allowed_read_roots.length} / 写 ${authorization.scope.allowed_write_roots.length}`,
      toolsAndChecks: `工具 ${authorization.scope.allowed_tools.length} / 检查 ${authorization.scope.allowed_checks.length}`,
      stopConditions: authorization.scope.stop_conditions.map((condition) => condition.summary),
      findings,
    },
  };
}

function completeGlobalBoundaryReviewChecklist() {
  return {
    architecture_boundary_checked: true,
    cross_project_impact_checked: true,
    permission_scope_checked: true,
    read_write_scope_checked: true,
    tool_and_check_scope_checked: true,
    memory_boundary_checked: true,
    stop_conditions_checked: true,
    acceptance_criteria_checked: true,
  };
}

function globalBoundaryReviewActionLabel(reviewStatus: GlobalBoundaryReviewStatus) {
  if (reviewStatus === "approved") return "批准并生效";
  if (reviewStatus === "needs_changes") return "要求修改全局边界";
  return "阻断方案";
}

function uniqueNonEmpty(values: Array<string | null | undefined>) {
  return Array.from(new Set(values.map((value) => value?.trim()).filter((value): value is string => Boolean(value))));
}

export function PlanAuthorizationSummaryCard({
  summary,
  guardResult,
  guardError,
}: {
  summary: ReturnType<typeof summarizePlanAuthorizationStore>;
  guardResult: AutoDispatchGuardResult | null;
  guardError: string | null;
}) {
  const guardSummary = summarizeAutoDispatchGuardResult(guardResult ?? summary.recent_guard_result ?? null);
  const blockedReasons = guardSummary.reasons.slice(0, 3);
  return (
    <section className="project-canvas-detail-card" aria-label="方案授权摘要">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">方案授权摘要</p>
          <h3>{summary.display_text}</h3>
        </div>
        <Badge tone={guardSummary.status === "authorized" ? "candidate" : guardSummary.status === "not_checked" ? "unknown" : "warning"}>
          {guardSummary.status}
        </Badge>
      </div>
      <div className="workflow-draft-grid">
        <DetailLine label="sidecar" value={`${summary.sidecar_name} / rev ${summary.revision}`} />
        <DetailLine label="授权对象" value={summary.latest_authorization_id ?? "未建立"} />
        <DetailLine label="active 授权" value={summary.active_authorization_id ?? "暂无"} />
        <DetailLine label="允许角色" value={String(summary.actor_scope?.allowed_role_ids.length ?? 0)} />
        <DetailLine label="允许 agent" value={String(summary.actor_scope?.allowed_agent_ids.length ?? 0)} />
        <DetailLine label="读写范围" value={`读 ${summary.resource_scope?.allowed_read_roots.length ?? 0} / 写 ${summary.resource_scope?.allowed_write_roots.length ?? 0}`} />
        <DetailLine label="工具 / 检查" value={`工具 ${summary.resource_scope?.allowed_tools.length ?? 0} / 检查 ${summary.resource_scope?.allowed_checks.length ?? 0}`} />
        <DetailLine label="停止条件" value={String(summary.stop_condition_count)} />
        <DetailLine label="当前检查" value={guardSummary.display_text} />
        <DetailLine label="最近审计" value={summary.recent_audit_event_id ?? "暂无"} />
      </div>
      {guardError ? <p className="state-warning">授权检查读取失败：{guardError}</p> : null}
      {blockedReasons.map((reason) => (
        <p className="state-warning" key={reason}>{reason}</p>
      ))}
      <p className="muted small-note">本摘要只读；授权检查由控制核心执行；本轮未执行真实工作者。</p>
    </section>
  );
}
