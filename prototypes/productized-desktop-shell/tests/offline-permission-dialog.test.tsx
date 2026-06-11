import React from "react";
import { renderToStaticMarkup } from "react-dom/server.browser";
import {
  assert,
  assertDeepEqual,
  buttonTextsInMarkup,
  findButtonByText,
  findButtonContainingText,
  findElement,
  visibleText,
} from "./helpers/offlineInteractionTestUtils";
import type { ReactElementLike } from "./helpers/offlineInteractionTestUtils";
import { runPermissionScenario } from "./helpers/offlinePermissionScenarioUtils";
import type { CapturedActionState, OfflinePermissionScenario } from "./helpers/offlinePermissionScenarioUtils";
import { authorizationWorkflowFixtures } from "./helpers/offlineAuthorizationWorkflowFixtures";
import {
  directorReviewActionFixture,
  globalBoundaryReviewPayloadFixture,
  globalBoundaryReviewSummary,
  projectConsultationProposalDecisionPayloadFixture,
  projectConsultationProposalDecisionSummary,
  projectDirectorTaskPlanRequestFixture,
} from "./helpers/offlineProjectPlanningActionFixtures";
import {
  buildBootstrapProjectWorkflowAction,
  buildAdvanceWorkItemStateAction,
  buildBindNodeSessionAction,
  buildCopyTaskPreviewAction,
  buildCreateTaskDraftAction,
  buildCorrectDispatchFieldsAction,
  buildGenerateTaskFileAction,
  buildNotReadyDispatchReadiness,
  buildPermissionDecisionAction,
  buildUpdateTaskFieldsAction,
  buildUnbindNodeSessionAction,
  buildUserReviewedInstructionPreviewAction,
  taskDraftFormDataFixture,
  taskDraftFormValues,
  taskFieldCorrectionFixtures,
} from "./helpers/offlineTaskFieldTestUtils";
import {
  runtimeAttentionFixture,
  runtimeAttentionFixtures,
  runtimeSessionSummaryFixture,
  runtimeLogStoreFixture,
} from "./helpers/offlineRuntimeDiagnosticFixtures";
import {
  expectedOfflineRoleDispatchAction,
  missingOfflineDispatchBlock,
  missingOfflineRoleDispatchFormDataFixture,
  offlineRoleDispatchFormDataFixture,
} from "./helpers/offlineRoleOrchestrationFixtures";
import { workbenchBaseFixtures } from "./helpers/offlineWorkbenchBaseFixtures";
import { projectWorkflowStateFixtures } from "./helpers/offlineProjectWorkflowStateFixtures";
import { derivedWorkflowStateFixtures } from "./helpers/offlineDerivedWorkflowFixtures";
import { c6ResultSummaryFixtures } from "./helpers/offlineC6ResultSummaryFixtures";
import { candidateGovernanceFixtures } from "./helpers/offlineCandidateGovernanceFixtures";
import { memoryCenterCoreFixtures } from "./helpers/offlineMemoryCenterCoreFixtures";
import { memoryCenterGovernanceFixtures } from "./helpers/offlineMemoryCenterGovernanceFixtures";
import { memoryPendingActionFixtures } from "./helpers/offlineMemoryPendingActionFixtures";
import { memoryPatternFixtures } from "./helpers/offlineMemoryPatternFixtures";
import {
  knowledgeBaseBoundaryFixtures,
  secretaryReadModelFixtures,
} from "./helpers/offlineKnowledgeSecretaryFixtures";
import {
  sessionCenterHardeningFixtures,
  transcriptCleaningFixtures,
} from "./helpers/offlineTranscriptSessionFixtures";
import { realExecutionProductCommandFixtures } from "./helpers/offlineRealExecutionProductCommandFixtures";
import {
  controlledSessionContinuationLevelAStoreFixture,
  h2DuplicateSessionContinuationStoreFixture,
} from "./helpers/offlineSessionContinuationStoreFixtures";
import {
  rightDetailPanelCommonPropsFixture,
  rightRailPanelSummaryTitles,
} from "./helpers/offlineRightRailFixtures";
import {
  shellDerivedWorkflowExpectedTexts,
  shellProposalDialogExpectedTexts,
  shellScenarioTextFixtures,
} from "./helpers/offlineShellScenarioTextFixtures";
import { stageJRunQueueFixtures } from "./helpers/offlineRunQueueFixtures";
import { workerProtocolFixtureForAdapters } from "./helpers/offlineWorkerProtocolFixtures";
import {
  workflowStateReadyForReviewFixture,
  workflowStateWithCompletedOfflineDispatchFixture,
  workflowStateWithGeneratedTaskFileFixture,
  workflowStateWithPreparedOfflineDispatchFixture,
} from "./helpers/offlineWorkflowStateVariantFixtures";
import { AgentSessionCenter, AgentView, ChatTranscript, filterAgentSessions } from "../src/views/AgentView";
import { HomeView } from "../src/views/HomeView";
import { RunningWorkflowsView } from "../src/views/RunningWorkflowsView";
import { SettingsView } from "../src/views/SettingsView";
import { PermissionDialog } from "../src/components/PermissionDialog";
import { WorkflowStatePanel } from "../src/components/WorkflowStatePanel";
import { deriveAgentAdapterDescriptors } from "../src/lib/adapterCapabilities";
import { deriveProviderAvailabilitySummaries } from "../src/lib/providerAvailability";
import {
  deriveH2RealResumeAuthorizationReadiness,
  deriveH2RealResumeExecutionDecisionSurface,
} from "../src/lib/h2RealResumeAuthorization";
import { deriveSessionContinuationPreviews, inspectSessionContinuationGuard } from "../src/lib/sessionContinuation";
import { deriveSessionOperationDescriptors } from "../src/lib/sessionOperations";
import { summarizePlanAuthorizationStore } from "../src/lib/planAuthorization";
import { summarizeProjectConsultationProposalStore } from "../src/lib/projectConsultationProposal";
import {
  buildOfflineRoleDispatchAction,
  buildOfflineStubResult,
  defaultOfflineDispatchBlock,
  OfflineRoleOrchestrationPanel,
  parseOfflineDispatchBlock,
} from "../src/views/OfflineRoleOrchestrationPanel";
import {
  ProjectDetail,
  ProjectDirectorTaskPlanCard,
  ProjectsView,
  TaskDispatchFieldCorrectionEditor,
  TaskDispatchFieldCorrectionShell,
  TaskDispatchReadinessDetails,
  TaskDispatchReadinessController,
  TaskDispatchReadinessShell,
  TaskFieldCorrectionPreview,
  WorkflowRunCheckDetails,
  buildGlobalBoundaryReviewAction,
  buildPrepareAuthorizedAutoDispatchAction,
  buildProjectConsultationProposalDecisionAction,
  missingCorrectionFields,
  TaskFileGenerationController,
  WorkItemOrchestrationCard,
  filterProjectSessionsForProject,
  nextSelectedWorkItemId,
  selectedTaskDraftFor,
} from "../src/views/ProjectsView";
import { SkillsBoardView } from "../src/views/SkillsBoardView";
import { HarnessBoardView } from "../src/views/HarnessBoardView";
import { RightDetailPanel, workspaceRailItems } from "../src/App";
import { devNavItems, primaryNavItems } from "../src/lib/workbenchNavigation";
import {
  canvasBoundaryForbiddenPhrases,
  experimentCanvasBoundary,
  projectWorkflowCanvasBoundary,
} from "../src/lib/canvasSurfaceBoundaries";
import { deriveProjectWorkflowCanvasReadModel, projectCanvasStateExamples } from "../src/lib/projectCanvas";
import { conversationTurns } from "../src/lib/conversationTurns";
import {
  buildBlackboardCandidateOverlay,
  summarizeObservationStore,
  summarizeFormalMemoryStore,
  summarizeMemoryCandidateStore,
  summarizeMemoryLintStore,
  summarizeTaskPackageMemoryInjection,
  summarizeTaskMemoryPacketPreview,
} from "../src/lib/candidateGovernance";
import { deriveKnowledgeBaseSummary } from "../src/lib/knowledgeBase";
import { deriveMemoryManagementSummary } from "../src/lib/memoryCenter";
import { deriveRunQueueReadModel } from "../src/lib/runQueue";
import { SecretaryBrief } from "../src/components/SecretaryBrief";
import { deriveSecretaryContext } from "../src/lib/secretaryReadModel";
import { KnowledgeBaseView } from "../src/views/KnowledgeBaseView";
import { MemoryCenterView } from "../src/views/MemoryCenterView";
import type {
  PendingAction,
  ProjectConsultationProposalStoreV1,
  RuntimeSessionAttention,
  WorkbenchSnapshot,
  WorkflowStateSnapshot,
} from "../src/lib/types";

const {
  backendAgentAdapterDescriptors,
  backendProviderAvailabilitySummaries,
  backendSessionOperationDescriptors,
  emptyProject,
  otherProjectSession,
  plugin,
  project,
  session,
  skill,
  snapshot,
  workflowId,
  workflowProjectId,
} = workbenchBaseFixtures();

const { workflowState, workflowStateWithProjectWorkflow } = projectWorkflowStateFixtures(project.project_root, session);

const {
  blockedWorkflowRunCheck,
  planAuthorizationStore,
  projectConsultationProposalStore,
  planAuthorizationStorePendingGlobal,
  projectConsultationProposalStoreConfirmed,
  projectConsultationProposalStoreActive,
  projectDirectorTaskPlan,
  runnableWorkflowRunCheck,
} = authorizationWorkflowFixtures(project.project_root, session.thread_id, workflowProjectId, workflowId);

const planAuthorizationSummary = summarizePlanAuthorizationStore(planAuthorizationStore, workflowProjectId, workflowId);
const projectConsultationProposalSummary = summarizeProjectConsultationProposalStore(
  projectConsultationProposalStoreActive,
  planAuthorizationStore,
  workflowProjectId,
  workflowId,
);

const { pendingWorkflowResultSummary, workflowStateWithDerivedWorkflow } = derivedWorkflowStateFixtures({
  projectRoot: project.project_root,
  sessionThreadId: session.thread_id,
  workflowProjectId,
  workflowId,
  workflowStateWithProjectWorkflow,
});
const { workflowStateWithC6ResultSummary } = c6ResultSummaryFixtures({
  workflowProjectId,
  workflowId,
  pendingWorkflowResultSummary,
  workflowStateWithDerivedWorkflow,
});

const workflowStateReadyForReview: WorkflowStateSnapshot =
  workflowStateReadyForReviewFixture(workflowStateWithProjectWorkflow);

const workflowStateWithPreparedOfflineDispatch: WorkflowStateSnapshot = workflowStateWithPreparedOfflineDispatchFixture(
  workflowStateWithProjectWorkflow,
  project.project_root,
);

const workflowStateWithCompletedOfflineDispatch: WorkflowStateSnapshot = workflowStateWithCompletedOfflineDispatchFixture(
  workflowStateWithProjectWorkflow,
  project.project_root,
);

const workflowStateWithGeneratedTaskFile: WorkflowStateSnapshot =
  workflowStateWithGeneratedTaskFileFixture(workflowStateWithProjectWorkflow);

const notReadyDispatchReadiness = buildNotReadyDispatchReadiness(project.project_root);

const scenarios: OfflinePermissionScenario[] = [];

let capturedAction: PendingAction | null = null;

function captureAction(action: PendingAction) {
  capturedAction = action;
}

const capturedActionState: CapturedActionState = {
  get: () => capturedAction,
  set: (action) => {
    capturedAction = action;
  },
};

function main() {
  runShellScenario();
  runProjectCanvasReadModelScenario();
  runAdapterCapabilityScenario();
  runSessionOperationBoundaryScenario();
  runProviderAvailabilityBoundaryScenario();
  runAdapterSdkCliDiagnosticsBoundaryScenario();
  runRealExecutionProductCommandBoundaryScenario();
  runStageJRunQueueScenario();
  runSessionContinuationPreviewScenario();
  runControlledSessionContinuationLevelAScenario();
  runH2RealResumeAuthorizationReadinessScenario();
  runRuntimeSessionAttentionScenario();
  runRuntimeLogBoundaryScenario();
  runCandidateGovernanceScenario();
  runMemoryManagementCenterScenario();
  runKnowledgeBaseBoundaryScenario();
  runSecretaryReadModelScenario();
  runRightRailSecretarySurfaceScenario();
  runOfflineRoleOrchestrationScenario();
  runTranscriptCleaningScenario();
  runSessionCenterHardeningScenario();
  for (const scenario of scenarios) {
    runPermissionScenario(scenario, capturedActionState);
  }
  console.log(`offline interaction tests passed: ${scenarios.length + 14}`);
}

function runRealExecutionProductCommandBoundaryScenario() {
  const readModel = snapshot.real_execution_product_commands;
  assert(readModel, "PCR1 snapshot 应包含真实执行 product command 只读摘要");
  assert(readModel.schema_version === "real_execution_product_commands.v1", "PCR1 read model schema 不匹配");
  assert(readModel.store_available === false, "PCR1 离线 fixture 不应声明真实 sidecar 可用");
  assert(readModel.command_count === 0, "PCR1 离线 fixture 不应包含真实 product command");
  assert(
    readModel.ordinary_product_entry_status === "readiness_only_pcr1_no_execute",
    "PCR1 普通入口只能是 readiness-only",
  );
  assert(
    readModel.legacy_entry_status === "legacy_sealed_blocked_not_product_command",
    "PCR1 旧入口必须保持 legacy / sealed / blocked",
  );
  assert(
    readModel.runner_entry_status === "internal_runner_blocked_until_unified_execute_and_level_b",
    "PCR1 runner 入口必须保持内部阻断直到 Level B",
  );
  assert(readModel.level_b_authorization_required, "PCR1 必须保留 Level B 授权要求");
  assert(readModel.failure_stop_retry_summary.item_count === 0, "PCR7 默认 fixture 不应造失败/停止/重试状态");
  assert(!readModel.failure_stop_retry_summary.retry_requires_new_user_confirmation, "PCR7 默认 fixture 不应要求重新确认");

  const { activeReadModel, activeAutomation, snapshotWithProductCommands } =
    realExecutionProductCommandFixtures({
      snapshot,
      readModel,
      workflowStateWithProjectWorkflow,
      projectRoot: project.project_root,
    });

  const agentNode = (
    <AgentView
      sessions={[session]}
      projects={[project]}
      adapterDescriptors={snapshot.agent_adapters}
      workflowState={workflowStateWithProjectWorkflow}
      realExecutionProductCommands={activeReadModel}
      projectWorkflowAutomation={activeAutomation}
      onRequestAction={captureAction}
    />
  );
  const agentText = visibleText(agentNode);
  const agentMarkup = renderToStaticMarkup(agentNode);
  for (const expectedText of ["项目", "对话", "可以开始对话", "任务输入", "生成发送预览"]) {
    assert(agentText.includes(expectedText), `J5 Agent 对话工作区缺少 ${expectedText}`);
  }
  assert(agentMarkup.includes("agent-conversation-bar"), "J5 Agent 普通区应有项目 / 对话选择条");
  assert(agentMarkup.includes("agent-chat-composer"), "J5 Agent 普通区应有任务输入框");
  assert(
    agentMarkup.indexOf("agent-conversation-bar") < agentMarkup.indexOf("agent-boundary-details"),
    "J5 Agent 普通对话区必须排在开发者详情前面",
  );
  for (const expectedText of ["统一执行链路", "2 条统一命令", "等待确认：1", "受控记录：1", "阻断：1", "读回边界：未知 / 不可用"]) {
    assert(agentText.includes(expectedText), `PCR6 Agent 统一执行链路缺少 ${expectedText}`);
  }
  for (const expectedText of ["自动编排：Level A 闭环已记录", "编排 run units：5", "编排读回未知：5", "编排捕获来源：1", "worker report 已回收"]) {
    assert(agentText.includes(expectedText), `K3 Agent 自动编排摘要缺少 ${expectedText}`);
  }
  for (const expectedText of [
    "Codex 控制",
    "J1-A · 产品命令入口 · 非真实执行",
    "生成预览",
    "写入准备",
    "用户确认",
    "记录 Phase A（不真实执行）",
    "任务正文保存策略",
    "观察 / 候选来源",
    "不会自动写正式记忆",
    "临时运行绑定",
  ]) {
    assert(agentText.includes(expectedText), `J1-A Agent Codex 控制入口缺少 ${expectedText}`);
  }
  for (const expectedText of [
    "需要重新确认",
    "用户已拒绝",
    "被安全边界阻断",
    "被诊断阻断",
    "重复执行已阻断",
    "记忆包缺失或过期",
    "读回不可用",
    "读回失败",
    "执行超时",
    "运行失败",
    "停止请求需受控处理",
  ]) {
    assert(agentText.includes(expectedText), `PCR7 Agent 统一执行链路缺少 ${expectedText}`);
  }
  assert(agentText.includes("结果数：未知/不可用"), "PCR7 Agent readback null 应显示未知 / 不可用");
  assert(!agentText.includes("H6 真实执行状态"), "PCR6 Agent 普通 UI 不应继续显示 H6 阶段标题");

  const runningText = visibleText(
    <RunningWorkflowsView
      snapshot={snapshotWithProductCommands}
      workflowState={workflowStateWithProjectWorkflow}
      workflowStateLoading={false}
      workflowStateError={null}
      onReloadWorkflowState={() => {}}
      onNavigate={() => {}}
    />,
  );
  for (const expectedText of ["统一执行命令", "2", "1 等确认", "最近状态", "受控记录已写入", "不等于真实 Codex 自由运行", "未知 / 不可用不显示成 0"]) {
    assert(runningText.includes(expectedText), `PCR6 Running 统一执行链路缺少 ${expectedText}`);
  }
  for (const expectedText of ["自动编排", "5", "0 等确认", "5 读回未知", "worker report", "捕获来源", "过程观察", "主管复核"]) {
    assert(runningText.includes(expectedText), `K3 Running 自动编排摘要缺少 ${expectedText}`);
  }
  for (const expectedText of ["失败", "读回异常", "停止请求", "需要重新确认", "读回结果：未知 / 不可用"]) {
    assert(runningText.includes(expectedText), `PCR7 Running 统一执行链路缺少 ${expectedText}`);
  }

  const projectDetailText = visibleText(
    <ProjectDetail
      project={project}
      sessions={[session]}
      workflowState={workflowStateWithProjectWorkflow}
      realExecutionProductCommands={activeReadModel}
      projectWorkflowAutomation={activeAutomation}
      selectedTool="workflow"
      onRequestAction={captureAction}
    />,
  );
  for (const expectedText of [
    "统一执行链路",
    "统一命令状态",
    "运行关注",
    "旧派发记录",
    "历史派发记录可见，不是统一产品命令",
    "读回边界",
    "未知 / 不可用",
    "开发者详情：统一命令读模型",
    "失败 / 阻断 / 读回",
    "重新确认",
    "停止请求",
    "读回结果：未知 / 不可用",
    "自动编排",
    "自动编排阶段",
    "编排捕获",
    "编排读回",
    "项目自动编排目标",
    "生成 Level A 编排记录",
    "确认后只写工作台记录、捕获来源和 observation",
    "主管复核",
  ]) {
    assert(projectDetailText.includes(expectedText), `PCR6 Projects 统一执行链路缺少 ${expectedText}`);
  }

  const secretaryContext = deriveSecretaryContext({
    snapshot: snapshotWithProductCommands,
    workflowState: workflowStateWithProjectWorkflow,
    workflowStateError: null,
  });
  const secretaryText = visibleText(<SecretaryBrief context={secretaryContext} />);
  assert(secretaryText.includes("查看统一执行链路"), "PCR6 秘书应提供统一执行链路查看建议");
  const secretaryProductCommandText = [
    ...secretaryContext.risk_signals.map((risk) => risk.summary),
    ...secretaryContext.suggestions.map((suggestion) => suggestion.summary),
  ].join("\n");
  assert(secretaryProductCommandText.includes("用户已拒绝"), "PCR7 秘书 read model 应解释 product command 失败/停止/重试状态");
  assert(secretaryProductCommandText.includes("需要重新确认"), "PCR7 秘书 read model 应提示重新确认边界");
  assert(secretaryContext.risk_signals.some((risk) => risk.kind === "real_execution_product_command_boundary"), "PCR6 秘书风险应包含 product command 边界");
  assert(secretaryContext.risk_signals.some((risk) => risk.kind === "project_workflow_automation_boundary"), "K3 秘书风险应包含自动编排边界");
  assert(
    secretaryContext.risk_signals.some((risk) => risk.summary.includes("不能批准、派发、恢复、重试、停止或重启")),
    "PCR6 秘书风险摘要应声明不生成执行类建议",
  );
  assert(
    secretaryContext.suggestions.some((suggestion) => suggestion.kind === "inspect_real_execution_product_commands"),
    "PCR6 秘书建议应包含统一执行链路查看建议",
  );
  assert(
    secretaryContext.suggestions.some((suggestion) => suggestion.kind === "inspect_project_workflow_automation"),
    "K3 秘书建议应包含自动编排查看建议",
  );
  assert(
    secretaryContext.suggestions.every((suggestion) =>
      !["approve", "dispatch", "retry", "stop", "restart", "resume", "send"].includes(suggestion.kind),
    ),
    "PCR7 秘书 suggestion kind 不应变成执行类动作",
  );
  for (const forbiddenProposalText of ["批准", "派发", "重试", "stop", "resume", "停止", "恢复"]) {
    assert(
      !secretaryContext.action_proposals.some((proposal) => proposal.title.toLowerCase().includes(forbiddenProposalText.toLowerCase())),
      `PCR6 秘书 action proposal 不应生成执行动作：${forbiddenProposalText}`,
    );
  }

  const rightPanelText = visibleText(
    <RightDetailPanel
      activePanel="running"
      snapshot={snapshotWithProductCommands}
      workflowState={workflowStateWithProjectWorkflow}
      notice="offline notice"
      error={false}
      workflowStateError={null}
      secretaryContext={secretaryContext}
      onClose={() => {}}
      onNavigate={() => {}}
      onReloadWorkflowState={() => {}}
    />,
  );
  for (const expectedText of ["统一执行链路", "统一执行命令状态", "最近状态：受控记录已写入", "读回未知 / 不可用不能显示成 0"]) {
    assert(rightPanelText.includes(expectedText), `PCR6 Right rail 统一执行链路缺少 ${expectedText}`);
  }
  for (const expectedText of ["失败", "读回异常", "需确认", "停止请求", "停止请求需受控处理", "读回结果：未知 / 不可用"]) {
    assert(rightPanelText.includes(expectedText), `PCR7 Right rail 统一执行链路缺少 ${expectedText}`);
  }

  const combinedMarkup = renderToStaticMarkup(
    <>
      <AgentView sessions={[session]} realExecutionProductCommands={activeReadModel} onRequestAction={captureAction} />
      <RunningWorkflowsView
        snapshot={snapshotWithProductCommands}
        workflowState={workflowStateWithProjectWorkflow}
        workflowStateLoading={false}
        workflowStateError={null}
        onReloadWorkflowState={() => {}}
        onNavigate={() => {}}
      />
    </>,
  );
  for (const forbiddenText of [
    "H5 命令",
    "H6 真实执行状态",
    "允许一次",
    "结果数：0",
    "runRealExecutionProductCommandPhaseA",
    "runRealExecutionProductCommandPhaseB",
    "run_real_execution_product_command_phase_b",
    "confirmRealExecutionProductCommand",
    "recordRealExecutionProductCommandDecision",
    "prepareRealExecutionProductCommand",
  ]) {
    assert(!combinedMarkup.includes(forbiddenText), `PCR6 UI 不应暴露 ${forbiddenText}`);
  }
}

function runStageJRunQueueScenario() {
  const { j4Snapshot, memoryCaptureStore, memoryCandidateStore } = stageJRunQueueFixtures({
    snapshot,
    workflowStateWithProjectWorkflow,
    projectRoot: project.project_root,
  });

  const readModel = deriveRunQueueReadModel({
    snapshot: j4Snapshot,
    workflowState: workflowStateWithProjectWorkflow,
    memoryCaptureStore,
    memoryCandidateStore,
  });
  assert(readModel.schema_version === "run_queue_read_model.v1", "J4 run queue schema 不匹配");
  assert(readModel.operation_control_summary.schema_version === "operation_control_summary.v1", "K5 operation control schema 不匹配");
  assert(readModel.operation_control_summary.true_operation_available === false, "K5 操作控制摘要不应声明真实操作可用");
  assert(readModel.operation_control_summary.retry_proposal_count >= 1, "K5 应汇总重试确认或重试提案");
  assert(readModel.operation_control_summary.stop_request_count >= 1, "K5 应汇总停止 / 取消确认");
  assert(readModel.operation_control_summary.restart_readiness_count === 0, "K5 不应声明真实重启准备完成");
  assert(readModel.operation_control_summary.resume_readiness_count === 0, "K5 不应声明真实恢复准备完成");
  assert(readModel.operation_control_summary.readback_issue_count >= 2, "K5 应汇总读回异常");
  assert(readModel.operation_control_summary.duplicate_blocked_count >= 1, "K5 应汇总重复阻断");
  assert(readModel.operation_control_summary.manual_review_count >= 1, "K5 应汇总人工复核事项");
  assert(
    readModel.operation_control_summary.warnings.includes("no_auto_retry_stop_restart_resume"),
    "K5 操作控制必须声明不会自动重试/停止/重启/恢复",
  );
  assert(readModel.run_queue_items.some((item) => item.status === "readback_unavailable" && item.readback_result_count === null), "J4 readback_unavailable 应保持 result_count=null");
  assert(readModel.run_queue_items.some((item) => item.status === "readback_failed" && item.readback_result_count === null), "J4 readback_failed 应保持 result_count=null");
  assert(readModel.failure_control_summaries.some((item) => item.status === "timed_out" && item.readback_result_count === null), "J4 timed_out 应保持 result_count=null");
  assert(readModel.failure_control_summaries.some((item) => item.classification === "duplicate_blocked"), "J4 duplicate blocked 应进入失败控制");
  assert(readModel.user_confirmation_queue.some((item) => item.kind === "execute_confirmation"), "J4 应包含执行确认");
  assert(readModel.user_confirmation_queue.some((item) => item.kind === "retry_confirmation"), "J4 应包含重试确认");
  assert(readModel.user_confirmation_queue.some((item) => item.kind === "stop_cancel_confirmation"), "J4 应包含停止 / 取消确认");
  assert(readModel.user_confirmation_queue.some((item) => item.kind === "result_confirmation"), "J4 应包含结果确认");
  assert(readModel.user_confirmation_queue.some((item) => item.kind === "process_fact_confirmation"), "J4 应包含过程事实确认");
  assert(readModel.user_confirmation_queue.some((item) => item.kind === "memory_candidate_confirmation"), "J4 应包含记忆候选确认");
  assert(readModel.user_confirmation_queue.some((item) => item.kind === "memory_formalization_confirmation"), "J4 应包含正式化确认");
  assert(readModel.user_confirmation_queue.some((item) => item.kind === "capture_compensation_confirmation"), "J4 应包含 capture 补偿确认");
  assert(readModel.capture_compensation_count === 1, "J4 capture 半完成状态应进入补偿摘要");
  assert(
    readModel.user_confirmation_queue.every((item) => item.confirmation_command_kind !== "runner_call"),
    "J4 确认队列不应直接调用 runner",
  );
  assert(
    readModel.user_confirmation_queue.every((item) => !item.writes_codex_home),
    "J4 默认确认队列不应声明写 .codex",
  );

  const runningText = visibleText(
    <RunningWorkflowsView
      snapshot={j4Snapshot}
      workflowState={workflowStateWithProjectWorkflow}
      workflowStateLoading={false}
      workflowStateError={null}
      memoryCaptureStore={memoryCaptureStore}
      memoryCandidateStore={memoryCandidateStore}
      onReloadWorkflowState={() => {}}
      onNavigate={() => {}}
    />,
  );
  for (const expectedText of [
    "运行队列",
    "待确认",
    "失败控制",
    "操作控制 / 恢复建议",
    "重试提案",
    "停止请求",
    "重启准备",
    "恢复准备",
    "只读建议",
    "后续任务",
    "单独授权",
    "不执行真实恢复命令",
    "不清理真实 Codex 本地状态",
    "重试确认",
    "停止 / 取消确认",
    "过程事实确认",
    "记忆候选确认",
    "正式化确认",
    "捕获补偿确认",
    "重复执行已阻断",
    "候选不是正式记忆",
    "结果数：未知 / 不可用",
    "不会自动调用 runner",
  ]) {
    assert(runningText.includes(expectedText), `K5 Running UI 缺少 ${expectedText}`);
  }

  const secretaryContext = deriveSecretaryContext({
    snapshot: j4Snapshot,
    workflowState: workflowStateWithProjectWorkflow,
    memoryCaptureStore,
    memoryCandidateStore,
  });
  assert(secretaryContext.risk_signals.some((risk) => risk.kind === "run_queue_boundary"), "J4 秘书风险应包含运行队列边界");
  assert(
    secretaryContext.risk_signals.some((risk) => risk.kind === "run_queue_boundary" && risk.summary.includes("捕获补偿 1")),
    "J4 秘书风险应包含 capture compensation 计数",
  );
  assert(secretaryContext.suggestions.some((suggestion) => suggestion.kind === "inspect_run_queue"), "J4 秘书建议应包含查看运行队列");
  assert(
    secretaryContext.action_proposals.every((proposal) => !["retry", "stop", "restart", "resume", "send"].includes(proposal.kind)),
    "J4 秘书 action proposal 不应变成执行动作",
  );

  const rightPanelText = visibleText(
    <RightDetailPanel
      activePanel="running"
      snapshot={j4Snapshot}
      workflowState={workflowStateWithProjectWorkflow}
      notice="offline notice"
      error={false}
      workflowStateError={null}
      memoryCaptureStore={memoryCaptureStore}
      memoryCandidateStore={memoryCandidateStore}
      secretaryContext={secretaryContext}
      onClose={() => {}}
      onNavigate={() => {}}
      onReloadWorkflowState={() => {}}
    />,
  );
  for (const expectedText of ["运行队列", "待确认", "失败控制", "捕获补偿", "不自动执行", "记忆候选确认", "正式化确认", "捕获补偿确认"]) {
    assert(rightPanelText.includes(expectedText), `J4 Right rail 缺少 ${expectedText}`);
  }

  const combinedMarkup = renderToStaticMarkup(
    <>
      <RunningWorkflowsView
        snapshot={j4Snapshot}
        workflowState={workflowStateWithProjectWorkflow}
        workflowStateLoading={false}
        workflowStateError={null}
        memoryCaptureStore={memoryCaptureStore}
        memoryCandidateStore={memoryCandidateStore}
        onReloadWorkflowState={() => {}}
        onNavigate={() => {}}
      />
      <RightDetailPanel
        activePanel="running"
        snapshot={j4Snapshot}
        workflowState={workflowStateWithProjectWorkflow}
        notice="offline notice"
        error={false}
        workflowStateError={null}
        memoryCaptureStore={memoryCaptureStore}
        memoryCandidateStore={memoryCandidateStore}
        secretaryContext={secretaryContext}
        onClose={() => {}}
        onNavigate={() => {}}
        onReloadWorkflowState={() => {}}
      />
    </>,
  );
  for (const forbiddenText of ["自动重试中", "已自动修复", "已写正式记忆", "结果数：0", "runner_call", "codex exec resume", "已停止", "已重启", "已恢复", "已 resume"]) {
    assert(!combinedMarkup.includes(forbiddenText), `K5 UI 不应出现误导文案：${forbiddenText}`);
  }
}

function runProjectCanvasReadModelScenario() {
  const boundaryText = JSON.stringify([experimentCanvasBoundary, projectWorkflowCanvasBoundary]);
  assert(experimentCanvasBoundary.context_kind === "experiment_canvas", "F4 一级画布边界应声明 experiment canvas 语境");
  assert(projectWorkflowCanvasBoundary.context_kind === "project_workflow_canvas", "F4 项目画布边界应声明 project workflow canvas 语境");
  for (const expectedText of [
    "实验 / 模板画布",
    "experiment / template / canvas library",
    "不会写项目事实",
    "不会写正式记忆",
    "不会写项目工作流状态",
    "不是项目 workflow 事实源",
    "MCP canvas run 非默认项目工作流",
    "项目工作流画布",
    "工作流状态派生读模型",
    "方案授权 / 控制核心 / 权限 / 审计",
    "React Flow 仅负责渲染",
    "实验画布不会写入本项目事实",
  ]) {
    assert(boundaryText.includes(expectedText), `F4 画布边界声明缺少 ${expectedText}`);
  }
  for (const forbiddenText of canvasBoundaryForbiddenPhrases) {
    assert(!boundaryText.includes(forbiddenText), `F4 画布边界声明不应出现误导文案 ${forbiddenText}`);
  }

  const projectWorkflow = workflowStateWithDerivedWorkflow.project_workflows[0];
  const selectedTask = projectWorkflow.task_drafts[0];
  const model = deriveProjectWorkflowCanvasReadModel({
    project,
    projectWorkflow,
    projectBlackboard: workflowStateWithDerivedWorkflow.project_blackboards?.[0] ?? null,
    selectedTask,
    workflowStatePath: workflowStateWithDerivedWorkflow.path,
    runtimeSessionAttention: [
      {
        ...runtimeAttentionFixture(
          session.thread_id,
          "canvas-readback-unavailable",
          "readback_unavailable",
          "warning",
          "readback_unavailable",
          "not_attempted_stub",
          true,
          false,
        ),
        project_id: projectWorkflow.project_id,
        workflow_id: projectWorkflow.workflow_id,
        node_id: "workflow:offline-fixture-projects-codex-workbench:default:node:codex-dev",
      },
    ],
  });

  assert(model.schema_version === "project_workflow_canvas.v1", "项目画布读模型 schema 不匹配");
  assert(model.source.source_kind === "workflow_state_read_model", "项目画布必须声明来自 workflow state 读模型");
  assert(model.status === "waiting_for_permission", "pending 权限请求应让画布进入等待权限状态");
  assert(model.status_reason.label === "等待权限", "画布缺少状态原因标签");
  assert(model.attention_items.some((item) => item.kind === "waiting_for_permission"), "画布 attention 缺少权限待处理");
  assert(model.attention_items.some((item) => item.kind === "readback_unavailable"), "画布 attention 缺少 readback unavailable");
  assert(model.nodes.some((node) => node.node_type === "project_goal"), "项目画布缺少项目目标节点");
  assert(model.nodes.some((node) => node.node_type === "director"), "项目画布缺少总指导节点");
  assert(model.nodes.some((node) => node.node_type === "dev_line"), "项目画布缺少开发线节点");
  assert(model.nodes.some((node) => node.node_type === "validation_line"), "项目画布缺少验证线节点");
  assert(model.nodes.some((node) => node.node_type === "review_line"), "项目画布缺少回收线节点");
  assert(model.nodes.some((node) => node.node_type === "permission_request"), "项目画布缺少权限请求 sidecar 节点");
  assert(model.nodes.some((node) => node.node_type === "blackboard_candidate"), "项目画布缺少黑板候选 sidecar 节点");
  assert(model.edges.some((edge) => edge.edge_type === "responsibility_flow"), "项目画布缺少责任流转边");
  assert(model.edges.some((edge) => edge.edge_type === "blocking_relation"), "项目画布缺少阻塞关系边");
  assert(model.viewport_hint.selected_node_id.includes(":canvas:codex-dev"), "默认选中节点应落在当前派发角色");
  assert(model.edit_boundary.source_kind === "frontend_read_model", "F3 编辑边界应来自前端只读模型");
  assert(model.edit_boundary.layout_boundary.react_flow_source_of_truth === false, "画布渲染层不应成为 workflow authority");
  assert(model.edit_boundary.layout_boundary.writes_workflow_state === false, "布局不应写 workflow state");
  assert(model.edit_boundary.layout_boundary.persists_layout === false, "F3 不应持久化布局");
  assert(
    model.edit_boundary.capabilities.some((capability) => capability.kind === "local_layout_preview" && capability.status === "allowed" && !capability.changes_workflow_facts),
    "F3 应允许本地视图布局预览且不改事实",
  );
  assert(
    model.edit_boundary.capabilities.some((capability) => capability.kind === "personal_layout_preference" && capability.status === "requires_future_task"),
    "F3 不应实现个人布局持久化",
  );
  for (const mutationKind of ["workflow_node_mutation", "workflow_edge_mutation"] as const) {
    assert(
      model.edit_boundary.capabilities.some(
        (capability) =>
          capability.kind === mutationKind &&
          capability.status === "preview_only" &&
          capability.changes_workflow_facts &&
          capability.requires_proposal &&
          capability.requires_control_core &&
          capability.requires_audit,
      ),
      `F3 ${mutationKind} 应只允许 proposal preview`,
    );
  }
  assert(
    model.edit_boundary.capabilities.some(
      (capability) =>
        capability.kind === "permission_or_model_mutation" &&
        capability.status === "blocked" &&
        capability.requires_confirmation &&
        capability.requires_control_core &&
        capability.requires_audit,
    ),
    "F3 高风险权限 / 模型变更必须被阻断并要求确认、控制核心和审计",
  );
  assert(
    model.edit_boundary.capabilities.some((capability) => capability.kind === "execution_mutation" && capability.status === "blocked"),
    "F3 不应允许执行变更",
  );
  assert(
    model.edit_boundary.proposal_previews.some(
      (preview) => preview.change_kind === "workflow_node_mutation" && preview.status === "preview_only" && preview.requires_proposal,
    ),
    "F3 节点变更缺少 proposal preview",
  );

  const selectedNode = model.nodes.find((node) => node.node_id === model.viewport_hint.selected_node_id);
  assert(selectedNode, "默认选中节点不存在");
  const detail = selectedNode ? model.detail_panels[selectedNode.detail_panel_id] : null;
  assert(detail, "默认选中节点缺少详情面板");
  const detailKinds = detail?.sections.map((section) => section.kind) ?? [];
  const detailLayers = detail?.sections.map((section) => section.layer) ?? [];
  for (const expectedLayer of ["user_summary", "project_director", "technical_details"]) {
    assert(detailLayers.includes(expectedLayer as never), `节点详情缺少 ${expectedLayer} 层`);
  }
  for (const expectedKind of ["summary", "task_package", "memory_packet", "session_binding", "dispatch", "readback", "permission_requests", "blackboard_entries", "completion_gate", "audit_refs"]) {
    assert(detailKinds.includes(expectedKind as never), `节点详情缺少 ${expectedKind}`);
  }
  const userSummary = detail?.sections.find((section) => section.layer === "user_summary");
  for (const expectedLabel of ["当前节点", "当前状态", "为什么停下", "谁能处理", "下一步"]) {
    assert(userSummary?.items.some((item) => item.label === expectedLabel), `用户摘要缺少 ${expectedLabel}`);
  }
  assert(
    detail?.sections.some((section) => section.kind === "source_refs" && section.layer === "technical_details"),
    "技术详情缺少 source refs 摘要",
  );
  assert(
    detail?.allowed_actions.some((action) => action.action_kind === "record_permission_decision" && action.enabled),
    "待权限节点详情应暴露权限结论动作说明",
  );
  assert(!model.source.derived_from.some((source) => source.kind === "audit_event" && source.id.includes("transcript")), "画布读模型不应引用完整 transcript");
  assert(
    detail?.sections.some((section) =>
      section.kind === "memory_packet" &&
      section.items.some((item) => item.value.includes("候选和观察不会当作正式记忆注入") || item.item_id === "memory-snapshot"),
    ),
    "节点详情缺少任务记忆包摘要边界",
  );
  assert(
    detail?.sections.some((section) =>
      section.kind === "readback" &&
      section.items.some((item) => item.value.includes("0 条") || item.value.includes("有摘要") || item.value.includes("events")),
    ),
    "节点详情缺少读回摘要",
  );

  const examples = projectCanvasStateExamples();
  assertDeepEqual(
    examples.map((example) => example.example_id),
    ["empty", "four_roles", "prepared", "running", "needs_review", "waiting_permission", "blocked", "failed", "timed_out", "readback_unavailable", "reviewing", "accepted"],
    "画布组件状态样例清单不匹配",
  );
  assert(examples.some((example) => example.permission_queue === "pending"), "状态样例缺少权限队列 pending 态");
  assert(examples.some((example) => example.detail_sections.includes("blackboard_entries") || example.description.includes("候选")), "状态样例缺少黑板候选基准");
  assert(examples.some((example) => example.status === "prepared"), "状态样例缺少 prepared 态");
  assert(examples.some((example) => example.status === "readback_unavailable"), "状态样例缺少 readback unavailable 态");

  const preparedWorkflow = {
    ...projectWorkflow,
    permission_requests: [],
    execution_attempts: [],
    task_drafts: [{ ...selectedTask, state: "prepared" }],
    node_dispatches: [
      {
        ...projectWorkflow.node_dispatches[0],
        state: "prepared",
        last_message_summary: null,
        transcript_event_count: null,
        transcript_target_hits: null,
        warnings: ["prepared_dispatch_is_not_worker_execution"],
      },
    ],
  };
  const preparedModel = deriveProjectWorkflowCanvasReadModel({
    project,
    projectWorkflow: preparedWorkflow,
    projectBlackboard: null,
    selectedTask: preparedWorkflow.task_drafts[0],
  });
  assert(preparedModel.status === "prepared", "prepared dispatch 应进入准备派发状态");
  assert(preparedModel.attention_items.some((item) => item.summary.includes("仍未启动工作者")), "准备态关注项不应暗示工作者已执行");

  const emptyModel = deriveProjectWorkflowCanvasReadModel({
    project,
    projectWorkflow: null,
    projectBlackboard: null,
    selectedTask: null,
  });
  assert(emptyModel.status === "empty", "缺 workflow 应进入空态");
  assert(emptyModel.attention_items.some((item) => item.summary.includes("不补编任务")), "空态不应补编工作项");
}

function runAdapterCapabilityScenario() {
  const descriptors = deriveAgentAdapterDescriptors({
    sessions: [session, otherProjectSession],
    projects: [project],
    workflowState: workflowStateWithProjectWorkflow,
  });
  assert(descriptors.length === 5, "E1 应返回 Codex 和四个计划中的 adapter descriptor");
  const codex = descriptors[0];
  assert(codex.adapter_id === "codex-local", "Codex adapter id 不匹配");
  assert(codex.agent_type === "codex", "Codex adapter agent_type 不匹配");
  assert(codex.status === "available", "有 Codex 会话和绑定时 adapter 应 available");
  assert(codex.source_kind === "frontend_read_model", "适配器能力声明应是前端读模型");
  assert(codex.execution_status === "available_with_user_confirmation", "Codex 执行状态应要求用户确认");
  assert(codex.credential_status === "not_read", "Codex descriptor 不应读取凭据");
  assert(codex.model_access_status === "local_read_model_only", "Codex 模型状态只能是本地读模型摘要");
  assert(codex.warnings.includes("adapter_descriptor_frontend_fallback_used"), "前端派生 helper 应声明 fallback 警告");
  assert(codex.hidden_unimplemented_adapters.includes("openclaw"), "未实现 OpenClaw 应隐藏");
  assert(codex.hidden_unimplemented_adapters.includes("claude-code"), "未实现 Claude Code 应隐藏");
  assert(codex.hidden_unimplemented_adapters.includes("opencode-like"), "OpenCode-like 应进入未实现清单");
  assert(codex.implemented_action_kinds.includes("bind-node-session"), "Codex adapter 缺少节点绑定动作声明");
  assert(codex.implemented_action_kinds.includes("execute-node-dispatch"), "Codex adapter 缺少派发动作声明");
  const plannedAdapters = descriptors.filter((descriptor) => descriptor.adapter_id !== "codex-local");
  assert(plannedAdapters.length === 4, "应包含四个计划中的 adapter descriptor");
  for (const planned of plannedAdapters) {
    assert(planned.status === "planned", `${planned.adapter_id} 必须是计划中状态`);
    assert(planned.execution_status === "not_implemented", `${planned.adapter_id} 不能有真实执行能力`);
    assert(planned.credential_status === "not_configured", `${planned.adapter_id} 凭据状态必须是未配置`);
    assert(planned.model_access_status === "not_verified", `${planned.adapter_id} 模型访问状态必须是未验证`);
    assert(planned.implemented_action_kinds.length === 0, `${planned.adapter_id} 不能声明已实现动作`);
    assert(planned.capabilities.every((capability) => capability.status !== "available"), `${planned.adapter_id} 不能有 available 能力`);
    assert(planned.warnings.includes("no_execution_button"), `${planned.adapter_id} 必须声明无执行按钮边界`);
  }
  for (const expectedCapability of [
    "session_index_read",
    "session_transcript_read",
    "workflow_node_binding",
    "safe_probe_dispatch",
    "user_reviewed_dispatch",
    "workflow_machine_run",
    "permission_decision_record",
    "harness_resource_index",
  ]) {
    assert(
      codex.capabilities.some((capability) => capability.kind === expectedCapability),
      `Codex adapter 缺少能力声明 ${expectedCapability}`,
    );
  }
  assert(
    codex.capabilities
      .filter((capability) => capability.status === "requires_confirmation")
      .every((capability) => capability.boundary.includes("本轮只声明能力") || capability.boundary.includes("控制核心") || capability.boundary.includes("工作流状态")),
    "需确认能力必须标明声明边界或控制核心边界",
  );
}

function runSessionOperationBoundaryScenario() {
  const descriptors = deriveAgentAdapterDescriptors({
    sessions: [session, otherProjectSession],
    projects: [project],
    workflowState: workflowStateWithProjectWorkflow,
  });
  const operations = deriveSessionOperationDescriptors(descriptors);
  assert(operations.length === 40, "H3.1 后应为 5 个 adapter 派生 40 条会话操作边界");

  const expectedOperationIds = ["new_session", "send_message", "stop", "restart", "resume", "export", "delete", "favorite"] as const;
  for (const operationId of expectedOperationIds) {
    assert(
      operations.filter((operation) => operation.operation_id === operationId).length === 5,
      `E2 缺少 ${operationId} per-adapter 边界`,
    );
  }
  assert(
    operations.every((operation) => !["available", "available_to_execute", "executable"].includes(operation.current_status)),
    "E2 不允许任何会话操作进入可执行状态",
  );
  assert(
    operations.every((operation) => operation.warnings.includes("session_operation_boundary_read_model_only")),
    "会话操作必须声明只读边界",
  );
  assert(
    operations.every((operation) => operation.warnings.includes("no_session_operation_execution_in_e2")),
    "会话操作必须声明 E2 不执行",
  );

  const codexNewSession = operations.find((operation) => operation.adapter_id === "codex-local" && operation.operation_id === "new_session");
  assert(codexNewSession?.current_status === "requires_future_task", "Codex 新会话必须需要后续任务");
  assert(codexNewSession?.writes_codex_home, "新会话真实实现前应显式声明 Codex home 写入影响");
  assert(codexNewSession?.requires_user_confirmation, "新会话真实实现前必须要求用户确认");
  assert(
    codexNewSession?.warnings.includes("h3_1_new_session_noop_only"),
    "H3.1 新会话必须声明 no-op only",
  );

  const codexSend = operations.find((operation) => operation.adapter_id === "codex-local" && operation.operation_id === "send_message");
  assert(codexSend?.current_status === "requires_future_task", "Codex 发消息必须需要后续任务");
  assert(codexSend?.writes_codex_home, "发消息真实实现前应显式声明 Codex home 写入影响");
  assert(codexSend?.requires_user_confirmation, "发消息真实实现前必须要求用户确认");

  const codexResume = operations.find((operation) => operation.adapter_id === "codex-local" && operation.operation_id === "resume");
  assert(codexResume?.current_status === "requires_future_task", "Codex resume 必须需要后续任务");
  assert(
    codexResume?.warnings.includes("workflow_dispatch_is_not_session_center_resume"),
    "workflow dispatch resume 不能被等同为会话中心 resume",
  );

  const deleteOperations = operations.filter((operation) => operation.operation_id === "delete");
  assert(
    deleteOperations.every((operation) => operation.current_status === "blocked_destructive" && operation.risk_level === "destructive"),
    "删除必须全部是破坏性阻断",
  );
  assert(
    deleteOperations.every((operation) => operation.warnings.includes("destructive_operation_blocked")),
    "删除必须包含破坏性阻断 warning",
  );

  const plannedOperations = operations.filter((operation) => operation.adapter_id !== "codex-local");
  assert(
    plannedOperations.every((operation) => operation.warnings.includes("planned_adapter_operation_not_available")),
    "planned adapter 会话操作必须保持不可用",
  );
  assert(
    plannedOperations.every((operation) => operation.applies_to_session_state === "planned_adapter_without_session_source"),
    "planned adapter 不应伪造会话事实源",
  );

  const agentView = (
    <AgentView
      sessions={[session]}
      projects={[project]}
      adapterDescriptors={backendAgentAdapterDescriptors}
      sessionOperationDescriptors={operations}
      workflowState={workflowStateWithProjectWorkflow}
      onRequestAction={captureAction}
    />
  );
  const agentViewText = visibleText(agentView);
  for (const expectedText of [
    "会话操作边界",
    "只读历史浏览器",
    "新会话预览",
    "H3.1 只实现新会话 request",
    "发消息",
    "需要后续任务",
    "停止",
    "当前不可执行",
    "resume",
    "会话中心通用 resume",
    "导出",
    "计划中",
    "删除",
    "破坏性阻断",
    "收藏",
    "计划中不可执行",
    "不执行新建会话、发消息、停止、重启、恢复、导出、删除或收藏",
  ]) {
    assert(agentViewText.includes(expectedText), `会话操作边界 UI 缺少 ${expectedText}`);
  }
  const agentViewMarkup = renderToStaticMarkup(agentView);
  for (const forbiddenButtonText of ["新建会话", "新会话预览", "发消息", "停止", "重启", "resume", "导出", "删除", "收藏"]) {
    assert(!buttonTextsInMarkup(agentViewMarkup).includes(forbiddenButtonText), `会话操作边界不应渲染可点击按钮：${forbiddenButtonText}`);
  }
  for (const forbiddenText of ["真实新会话已创建", "已创建真实会话", "已发送", "已停止", "已重启", "已 resume", "已导出", "已删除", "已收藏"]) {
    assert(!agentViewText.includes(forbiddenText), `会话操作边界不应出现误导文案：${forbiddenText}`);
  }

  const secretaryContext = deriveSecretaryContext({
    snapshot: {
      ...snapshot,
      session_operations: operations,
    },
    workflowState: workflowStateWithProjectWorkflow,
  });
  assert(
    secretaryContext.risk_signals.some((risk) => risk.kind === "session_operation_boundary"),
    "秘书风险应包含会话操作边界提醒",
  );
  assert(
    secretaryContext.suggestions.some((suggestion) => suggestion.kind === "inspect_session_operation_boundary"),
    "秘书建议应包含查看会话操作边界",
  );
  for (const forbiddenProposalText of ["新建会话", "发消息", "停止", "重启", "resume", "导出", "删除", "收藏"]) {
    assert(
      !secretaryContext.action_proposals.some((proposal) => proposal.title.includes(forbiddenProposalText)),
      `秘书 action proposal 不应变成会话操作：${forbiddenProposalText}`,
    );
  }
}

function runProviderAvailabilityBoundaryScenario() {
  const descriptors = deriveAgentAdapterDescriptors({
    sessions: [session, otherProjectSession],
    projects: [project],
    workflowState: workflowStateWithProjectWorkflow,
  });
  const operations = deriveSessionOperationDescriptors(descriptors);
  const summaries = deriveProviderAvailabilitySummaries(descriptors, operations);
  assert(summaries.length === 5, "E3 应为 5 个 adapter 派生 provider availability 摘要");
  assert(summaries.every((summary) => summary.safe_to_display), "E3 摘要必须可安全展示");
  assert(
    summaries.every((summary) => summary.warnings.includes("provider_availability_read_model_only")),
    "E3 摘要必须声明只读 provider availability",
  );
  assert(
    summaries.every((summary) => summary.warnings.includes("credential_secret_not_read")),
    "E3 摘要必须声明不读取 secret",
  );
  assert(
    summaries.every((summary) => summary.warnings.includes("provider_availability_not_project_authorization")),
    "E3 摘要必须声明不等于项目授权",
  );

  const codex = summaries.find((summary) => summary.adapter_id === "codex-local");
  assert(codex, "E3 缺少 codex-local provider 摘要");
  assert(codex.provider_kind === "local_cli", "codex-local provider kind 应是 local_cli");
  assert(codex.availability_status === "available_readonly", "codex-local 只能是只读可见");
  assert(codex.credential_status === "not_required_by_workbench", "codex-local 不应要求工作台读取凭据");
  assert(codex.model_status === "local_cli_managed", "codex-local 模型状态应由本地 CLI 管理");
  assert(codex.external_call_status === "not_needed_for_readonly", "codex-local 只读摘要不需要外发调用");
  assert(codex.cost_risk_status === "unknown", "codex-local 成本风险第一版应保持未知");

  const plannedSummaries = summaries.filter((summary) => summary.adapter_id !== "codex-local");
  assert(plannedSummaries.length === 4, "E3 planned provider 数量不匹配");
  assert(
    plannedSummaries.every(
      (summary) =>
        summary.availability_status === "planned" &&
        summary.credential_status === "credential_missing" &&
        summary.model_status === "model_unverified" &&
        summary.external_call_status === "external_call_blocked" &&
        summary.cost_risk_status === "blocked_until_authorized",
    ),
    "planned adapters 必须保持未配置、未验证和外发阻断",
  );
  assert(
    plannedSummaries.every((summary) => summary.requires_user_configuration && summary.requires_future_task),
    "planned provider 必须需要后续任务或用户设置",
  );

  const serializedSummaries = JSON.stringify(summaries);
  for (const forbiddenFragment of ["api_key", "oauth", "keychain", ".env", "available_to_execute", "provider_verified"]) {
    assert(!serializedSummaries.toLowerCase().includes(forbiddenFragment), `E3 摘要不应包含 ${forbiddenFragment}`);
  }

  const agentView = (
    <AgentView
      sessions={[session]}
      projects={[project]}
      adapterDescriptors={descriptors}
      sessionOperationDescriptors={operations}
      providerAvailabilitySummaries={summaries}
      workflowState={workflowStateWithProjectWorkflow}
      onRequestAction={captureAction}
    />
  );
  const agentViewText = visibleText(agentView);
  for (const expectedText of [
    "供应方 / 模型 / 凭据边界",
    "只读供应方可用性",
    "不等于项目授权",
    "Codex 本地 CLI",
    "本地 CLI 管理",
    "工作台不读取",
    "模型未验证",
    "外发调用已阻断",
    "授权前阻断",
    "planned_adapter_not_connected",
    "provider_availability_not_project_authorization",
    "no_external_provider_call_in_e3",
  ]) {
    assert(agentViewText.includes(expectedText), `Provider availability UI 缺少 ${expectedText}`);
  }
  for (const forbiddenText of [
    "已配置凭据",
    "模型已验证",
    "外部模型已可用",
    "Claude Code 已接入",
    "OpenClaw 已接入",
    "OpenCode 已接入",
    "provider 已验证",
    "测试调用成功",
  ]) {
    assert(!agentViewText.includes(forbiddenText), `Provider availability UI 不应出现误导文案：${forbiddenText}`);
  }
  const agentViewMarkup = renderToStaticMarkup(agentView);
  for (const forbiddenButtonText of ["配置凭据", "验证模型", "测试 provider", "调用模型", "dispatch"]) {
    assert(!buttonTextsInMarkup(agentViewMarkup).includes(forbiddenButtonText), `Provider availability 不应渲染可点击按钮：${forbiddenButtonText}`);
  }

  const secretaryContext = deriveSecretaryContext({
    snapshot: {
      ...snapshot,
      agent_adapters: descriptors,
      session_operations: operations,
      provider_availability: summaries,
    },
    workflowState: workflowStateWithProjectWorkflow,
  });
  assert(
    secretaryContext.risk_signals.some((risk) => risk.kind === "provider_availability_boundary"),
    "秘书风险应包含 provider availability 边界提醒",
  );
  assert(
    secretaryContext.suggestions.some((suggestion) => suggestion.kind === "inspect_provider_availability_boundary"),
    "秘书建议应包含查看模型与凭据边界",
  );
  for (const forbiddenProposalText of ["配置凭据", "验证模型", "调用模型", "provider"]) {
    assert(
      !secretaryContext.action_proposals.some((proposal) => proposal.title.includes(forbiddenProposalText)),
      `秘书 action proposal 不应变成 provider/model/credential 动作：${forbiddenProposalText}`,
    );
  }
}

function runAdapterSdkCliDiagnosticsBoundaryScenario() {
  const descriptors = deriveAgentAdapterDescriptors({
    sessions: [session, otherProjectSession],
    projects: [project],
    workflowState: workflowStateWithProjectWorkflow,
  });
  const operations = deriveSessionOperationDescriptors(descriptors);
  const workerProtocol = workerProtocolFixtureForAdapters(descriptors, backendProviderAvailabilitySummaries, operations);
  assert(
    workerProtocol.adapter_contract_checklists.length === descriptors.length,
    "I5 每个 adapter 都应有 contract checklist",
  );
  assert(
    workerProtocol.adapter_contract_checklists.every((checklist) => checklist.control_core_required && checklist.permission_required && checklist.audit_required && checklist.runtime_log_required),
    "I5 checklist 必须要求 control core / permission / audit / runtime log",
  );
  const plannedChecklists = workerProtocol.adapter_contract_checklists.filter((checklist) => checklist.adapter_id !== "codex-local");
  assert(
    plannedChecklists.every((checklist) => checklist.status === "blocked_or_reserved_contract"),
    "planned adapter contract 必须保持阻断或预留",
  );
  assert(
    plannedChecklists.every((checklist) => checklist.missing_items.includes("runtime_connection_not_implemented")),
    "planned adapter contract 必须明确缺 runtime connection",
  );
  assert(
    workerProtocol.controlled_api_cli_semantics.every((semantics) => semantics.universal_api_backdoor_blocked),
    "CLI parity 必须阻断 universal API backdoor",
  );
  assert(
    workerProtocol.diagnostic_event_schemas.every((schema) => schema.redaction_policy === "no_secret_no_raw_transcript_no_provider_payload"),
    "diagnostic schema 必须脱敏，不允许 secret/raw transcript/provider payload",
  );
  assert(
    workerProtocol.adapter_health_summaries
      .filter((summary) => summary.adapter_id !== "codex-local")
      .every((summary) => summary.status === "planned_unavailable"),
    "planned adapter health 必须保持 unavailable",
  );
  assert(
    workerProtocol.adapter_degraded_modes
      .filter((mode) => mode.adapter_id !== "codex-local")
      .every((mode) => mode.blocks_real_execution),
    "planned adapter degraded mode 必须阻断真实执行",
  );
  assert(
    workerProtocol.adapter_data_locations.every((location) => location.secret_policy === "never_read_auth_token_env_keychain_oauth_provider_credentials"),
    "data location descriptor 不能允许读取 secret",
  );

  const agentView = (
    <AgentView
      sessions={[session]}
      projects={[project]}
      adapterDescriptors={descriptors}
      sessionOperationDescriptors={operations}
      workerProtocol={workerProtocol}
      workflowState={workflowStateWithProjectWorkflow}
      onRequestAction={captureAction}
    />
  );
  const agentViewText = visibleText(agentView);
  for (const expectedText of [
    "适配器 SDK / 命令行 / 诊断预留",
    "只定义未来适配器接入的契约",
    "不提供通用执行接口",
    "不绕过控制核心",
    "运行日志",
    "审计",
    "阻断或预留",
    "阻断通用 API 后门",
    "诊断结构",
    "数据位置",
    "契约材料齐备",
    "阻断或预留",
    "runtime_connection_not_implemented",
    "model_boundary_or_verification_missing",
    "data_location_reserved_not_connected",
    "contract_parity_requires_guard",
    "reserved_no_runtime_parity",
    "required_before_runner",
    "explicit_user_confirmation_required_for_real_execution",
    "runtime_log_and_audit_refs_required",
    "后门阻断：是",
    "adapter_health_read_model_only",
    "adapter_data_location_descriptor_read_model_only",
  ]) {
    assert(agentViewText.includes(expectedText), `I5 Adapter SDK / CLI diagnostics UI 缺少 ${expectedText}`);
  }
  for (const forbiddenText of [
    "SDK 已接入",
    "CLI 已可执行",
    "通用真实 send/resume 已完成",
    "provider 已验证",
    "凭据已配置",
    "模型已验证",
    "外部 adapter 已接入",
    "自动派发已开始",
    "worker 执行中",
  ]) {
    assert(!agentViewText.includes(forbiddenText), `I5 UI 不应出现误导文案：${forbiddenText}`);
  }
  const agentViewMarkup = renderToStaticMarkup(agentView);
  for (const forbiddenButtonText of ["配置 SDK", "执行 CLI", "验证 provider", "配置凭据", "测试模型", "send", "resume", "dispatch", "重试"]) {
    assert(!buttonTextsInMarkup(agentViewMarkup).includes(forbiddenButtonText), `I5 不应渲染可执行按钮：${forbiddenButtonText}`);
  }
}

function runSessionContinuationPreviewScenario() {
  const descriptors = deriveAgentAdapterDescriptors({
    sessions: [session, otherProjectSession],
    projects: [project],
    workflowState: workflowStateWithProjectWorkflow,
  });
  const operations = deriveSessionOperationDescriptors(descriptors);
  const summaries = deriveProviderAvailabilitySummaries(descriptors, operations);
  const previews = deriveSessionContinuationPreviews({
    adapterDescriptors: descriptors,
    sessionOperationDescriptors: operations,
    providerAvailabilitySummaries: summaries,
    workflowState: workflowStateWithProjectWorkflow,
  });
  assert(previews.length === 15, "H3.1 后应为 5 个 adapter 的 new_session / send_message / resume 派生预览");
  assert(
    previews.every((preview) => preview.user_visible_warnings.includes("session_continuation_preview_only")),
    "E4 preview 必须声明只预览",
  );
  assert(
    previews.every((preview) => preview.audit_impact.impact_kind === "preview_only_no_execution"),
    "E4 preview 不能写 attempt / dispatch / readback",
  );
  assert(
    previews.every((preview) => !preview.audit_impact.writes_attempt_in_e4 && !preview.audit_impact.writes_dispatch_in_e4 && !preview.audit_impact.writes_readback_in_e4),
    "E4 audit impact 必须保持不写执行态",
  );

  const codexPreviews = previews.filter((preview) => preview.adapter_id === "codex-local");
  assert(codexPreviews.length === 3, "codex-local 应有 new_session / send_message / resume 三条预览");
  assert(
    codexPreviews.every((preview) => preview.guard_result.status === "needs_user_confirmation"),
    "完整绑定的 codex-local 预览应停在需要用户确认",
  );
  const codexNewSessionPreview = codexPreviews.find((preview) => preview.operation_id === "new_session");
  assert(codexNewSessionPreview, "codex-local 应有 new_session 预览");
  assert(codexNewSessionPreview.target_session_id === null, "new_session 预览不应要求已有 target session");
  assert(codexNewSessionPreview.work_item_id === "work-item:offline:001", "new_session 预览必须绑定 work item");
  assert(
    codexNewSessionPreview.request.prompt_source_kind === "h3_new_session_task_package",
    "new_session prompt source 应独立于 send/resume",
  );
  assert(
    codexNewSessionPreview.guard_result.warnings.includes("new_session_does_not_require_existing_session"),
    "new_session guard 应声明不要求已有 session",
  );
  assert(
    codexNewSessionPreview.readback_expectation.expected_sources.includes("future_h3_new_session_last_message"),
    "new_session readback 应指向未来 H3 last-message，而不是现有 session rollout",
  );
  const codexSendResumePreviews = codexPreviews.filter((preview) => preview.operation_id !== "new_session");
  assert(
    codexSendResumePreviews.every((preview) => preview.target_session_id === session.thread_id && preview.project_root === project.project_root),
    "codex-local send/resume preview 应携带 target session 和 project root",
  );
  assert(
    codexPreviews.every((preview) => preview.readback_expectation.strategy === "required"),
    "codex-local continuation preview 必须声明 readback required",
  );

  const plannedPreviews = previews.filter((preview) => preview.adapter_id !== "codex-local");
  assert(
    plannedPreviews.every((preview) => preview.guard_result.status === "blocked" || preview.guard_result.status === "requires_future_task"),
    "planned adapters 必须保持阻断或后续任务状态",
  );
  assert(
    plannedPreviews.every((preview) => preview.guard_result.reasons.some((reason) => reason.includes("planned_adapter_blocked"))),
    "planned adapter continuation preview 必须包含 planned_adapter_blocked 原因",
  );

  const codexOperation = operations.find((operation) => operation.adapter_id === "codex-local" && operation.operation_id === "send_message");
  assert(codexOperation, "缺少 codex send_message operation");
  const codexNewSessionOperation = operations.find((operation) => operation.adapter_id === "codex-local" && operation.operation_id === "new_session");
  assert(codexNewSessionOperation, "缺少 codex new_session operation");
  const codexAdapter = descriptors.find((descriptor) => descriptor.adapter_id === "codex-local");
  assert(codexAdapter, "缺少 codex adapter");
  const codexProvider = summaries.find((summary) => summary.adapter_id === "codex-local");
  const safeRequest = codexPreviews.find((preview) => preview.operation_id === "send_message")?.request;
  assert(safeRequest, "缺少 send_message safe request");
  const confirmedGuard = inspectSessionContinuationGuard(
    { ...safeRequest, user_confirmation_state: "confirmed" },
    codexAdapter,
    codexOperation,
    codexProvider,
  );
  assert(confirmedGuard.status === "allowed_preview", "用户确认后仍只能进入 allowed_preview");
  assert(confirmedGuard.blocks_execution, "allowed_preview 也必须阻断 E4 执行");

  const outOfScopeGuard = inspectSessionContinuationGuard(
    { ...safeRequest, target_cwd: "/offline-fixture/outside", allowed_write_roots: [project.project_root] },
    codexAdapter,
    codexOperation,
    codexProvider,
  );
  assert(outOfScopeGuard.status === "blocked", "cwd 越界必须阻断");
  assert(outOfScopeGuard.reasons.includes("cwd_out_of_scope_blocked"), "cwd 越界应有明确 reason");

  const sensitiveGuard = inspectSessionContinuationGuard(
    { ...safeRequest, target_cwd: `${project.project_root}/.env`, allowed_write_roots: [project.project_root] },
    codexAdapter,
    codexOperation,
    codexProvider,
  );
  assert(sensitiveGuard.status === "blocked", "敏感路径必须阻断");
  assert(
    sensitiveGuard.reasons.some((reason) => reason.startsWith("sensitive_path_blocked")),
    "敏感路径应有明确 reason",
  );

  const noReadbackGuard = inspectSessionContinuationGuard(
    { ...safeRequest, readback_strategy: "not_defined" },
    codexAdapter,
    codexOperation,
    codexProvider,
  );
  assert(noReadbackGuard.status === "blocked", "缺 readback strategy 必须阻断");
  assert(noReadbackGuard.reasons.includes("readback_strategy_required"), "缺 readback 应有明确 reason");

  const newSessionConfirmedGuard = inspectSessionContinuationGuard(
    { ...codexNewSessionPreview.request, user_confirmation_state: "confirmed" },
    codexAdapter,
    codexNewSessionOperation,
    codexProvider,
  );
  assert(newSessionConfirmedGuard.status === "allowed_preview", "new_session 用户确认后仍只能进入 allowed_preview");
  assert(newSessionConfirmedGuard.blocks_execution, "new_session allowed_preview 仍必须阻断真实执行");

  const newSessionMissingWorkItemGuard = inspectSessionContinuationGuard(
    { ...codexNewSessionPreview.request, work_item_id: null },
    codexAdapter,
    codexNewSessionOperation,
    codexProvider,
  );
  assert(newSessionMissingWorkItemGuard.status === "blocked", "new_session 缺 work item 必须阻断");
  assert(
    newSessionMissingWorkItemGuard.reasons.includes("missing_work_item_binding"),
    "new_session 缺 work item 应有明确 reason",
  );

  const agentView = (
    <AgentView
      sessions={[session]}
      projects={[project]}
      adapterDescriptors={descriptors}
      sessionOperationDescriptors={operations}
      providerAvailabilitySummaries={summaries}
      sessionContinuationPreviews={previews}
      workflowState={workflowStateWithProjectWorkflow}
      onRequestAction={captureAction}
    />
  );
  const agentViewText = visibleText(agentView);
  for (const expectedText of [
    "会话继续预览 / 权限预览",
    "E4 / H3.1 预览协议",
    "新会话预览",
    "不会创建真实新会话",
    "不会发送提示词",
    "不会执行恢复",
    "不会写 Codex 原生状态",
    "工作项：work-item:offline:001",
    "执行边界摘要：工作目录",
    "运行器：H3.1 空操作",
    "提示词发送状态：否",
    "真实 Codex 执行状态：否",
    "写入 Codex 主目录：否",
    "需要用户确认",
    "读回：必需",
    "审计影响：仅预览不执行",
    "供应方：只读可见",
    "planned_adapter_blocked",
    "h3_1_no_real_new_session",
    "no_prompt_sent_in_e4",
    "no_codex_home_write_in_e4",
  ]) {
    assert(agentViewText.includes(expectedText), `会话继续预览 UI 缺少 ${expectedText}`);
  }
  assert(!agentViewText.includes("命令计划：codex exec -C"), "会话继续普通 UI 不应暴露裸 codex exec 命令");
  for (const forbiddenText of [
    "真实新会话已创建",
    "已创建真实会话",
    "已发送",
    "已 resume",
    "Codex 已收到任务",
    "自动派发已开始",
    "worker 执行中",
    "readback 已完成",
  ]) {
    assert(!agentViewText.includes(forbiddenText), `会话继续预览 UI 不应出现误导文案：${forbiddenText}`);
  }
  const agentViewMarkup = renderToStaticMarkup(agentView);
  for (const forbiddenButtonText of ["新建会话", "发消息", "发送", "resume", "申请确认", "执行", "重试"]) {
    assert(!buttonTextsInMarkup(agentViewMarkup).includes(forbiddenButtonText), `会话继续预览不应渲染可点击按钮：${forbiddenButtonText}`);
  }

  const secretaryContext = deriveSecretaryContext({
    snapshot: {
      ...snapshot,
      agent_adapters: descriptors,
      session_operations: operations,
      provider_availability: summaries,
      session_continuation_previews: previews,
    },
    workflowState: workflowStateWithProjectWorkflow,
  });
  assert(
    secretaryContext.risk_signals.some((risk) => risk.kind === "session_continuation_boundary"),
    "秘书风险应包含会话继续预览边界",
  );
  assert(
    secretaryContext.suggestions.some((suggestion) => suggestion.kind === "inspect_session_continuation_preview"),
    "秘书建议应包含查看会话继续预览",
  );
  for (const forbiddenProposalText of ["新建会话", "发送", "发消息", "resume", "批准", "确认预览", "重试"]) {
    assert(
      !secretaryContext.action_proposals.some((proposal) => proposal.title.includes(forbiddenProposalText)),
      `秘书 action proposal 不应变成 continuation 执行动作：${forbiddenProposalText}`,
    );
  }
}

function runControlledSessionContinuationLevelAScenario() {
  const descriptors = deriveAgentAdapterDescriptors({
    sessions: [session, otherProjectSession],
    projects: [project],
    workflowState: workflowStateWithProjectWorkflow,
  });
  const operations = deriveSessionOperationDescriptors(descriptors);
  const summaries = deriveProviderAvailabilitySummaries(descriptors, operations);
  const previews = deriveSessionContinuationPreviews({
    adapterDescriptors: descriptors,
    sessionOperationDescriptors: operations,
    providerAvailabilitySummaries: summaries,
    workflowState: workflowStateWithProjectWorkflow,
  });
  const preview = previews.find((item) => item.adapter_id === "codex-local" && item.operation_id === "resume");
  assert(preview, "E5 场景缺少 codex-local resume preview");
  const { store } = controlledSessionContinuationLevelAStoreFixture({
    preview,
    projectRoot: project.project_root,
    sessionThreadId: session.thread_id,
  });

  const agentView = (
    <AgentView
      sessions={[session]}
      projects={[project]}
      adapterDescriptors={descriptors}
      sessionOperationDescriptors={operations}
      providerAvailabilitySummaries={summaries}
      sessionContinuationPreviews={previews}
      sessionContinuationStore={store}
      workflowState={workflowStateWithProjectWorkflow}
      onRequestAction={captureAction}
    />
  );
  const agentViewText = visibleText(agentView);
  for (const expectedText of [
    "受控会话继续 / E5 Level A",
    "桩验收",
    "真实执行未授权",
    "读回不可用",
    "readback_unavailable_is_not_zero_results",
    "提示词发送状态：否",
    "真实 Codex 执行状态：否",
    "写入 Codex 主目录：否",
    "session-continuations.v1.json",
  ]) {
    assert(agentViewText.includes(expectedText), `E5 Level A UI 缺少 ${expectedText}`);
  }
  for (const forbiddenText of [
    "已发送",
    "已 resume",
    "Codex 已收到任务",
    "真实 Codex 已执行",
    "worker 执行中",
    "readback 已完成",
    "0 条读回",
    "0 条结果",
  ]) {
    assert(!agentViewText.includes(forbiddenText), `E5 Level A UI 不应出现误导文案：${forbiddenText}`);
  }
  const agentViewMarkup = renderToStaticMarkup(agentView);
  for (const forbiddenButtonText of ["发消息", "发送", "resume", "执行", "重试", "stub 验收"]) {
    assert(!buttonTextsInMarkup(agentViewMarkup).includes(forbiddenButtonText), `E5 Level A 不应渲染可执行按钮：${forbiddenButtonText}`);
  }

  const secretaryContext = deriveSecretaryContext({
    snapshot: {
      ...snapshot,
      agent_adapters: descriptors,
      session_operations: operations,
      provider_availability: summaries,
      session_continuation_previews: previews,
      session_continuation_store: store,
    },
    workflowState: workflowStateWithProjectWorkflow,
  });
  assert(
    secretaryContext.risk_signals.some((risk) => risk.kind === "controlled_session_continuation_boundary"),
    "秘书风险应包含 E5 controlled continuation 边界",
  );
  assert(
    secretaryContext.suggestions.some((suggestion) => suggestion.kind === "inspect_controlled_session_continuation"),
    "秘书建议应包含查看 E5 controlled continuation",
  );
  for (const forbiddenProposalText of ["发送", "发消息", "resume", "批准", "确认", "重试", "stub"]) {
    assert(
      !secretaryContext.action_proposals.some((proposal) => proposal.title.includes(forbiddenProposalText)),
      `秘书 action proposal 不应变成 E5 continuation 执行动作：${forbiddenProposalText}`,
    );
  }
}

function runH2RealResumeAuthorizationReadinessScenario() {
  const descriptors = deriveAgentAdapterDescriptors({
    sessions: [session, otherProjectSession],
    projects: [project],
    workflowState: workflowStateWithProjectWorkflow,
  });
  const operations = deriveSessionOperationDescriptors(descriptors);
  const summaries = deriveProviderAvailabilitySummaries(descriptors, operations);
  const previews = deriveSessionContinuationPreviews({
    adapterDescriptors: descriptors,
    sessionOperationDescriptors: operations,
    providerAvailabilitySummaries: summaries,
    workflowState: workflowStateWithProjectWorkflow,
  });
  const readiness = deriveH2RealResumeAuthorizationReadiness({
    previews,
    store: snapshot.session_continuation_store,
  });
  const decisionSurface = deriveH2RealResumeExecutionDecisionSurface({
    previews,
    store: snapshot.session_continuation_store,
  });
  assert(readiness.status === "blocked_waiting_authorization", "H2.2 readiness 默认必须等待授权矩阵");
  assert(readiness.missing_count > 0, "H2.2 readiness 必须暴露缺失授权项");
  assert(decisionSurface.status.startsWith("blocked_"), "H2.8 decision surface 默认必须保持阻断态");
  assert(!decisionSurface.final_approval_allowed, "H2.8 decision surface 不得允许 final approval");
  assert(
    decisionSurface.decision_checks.some((check) => check.check_id === "codex_home_scope" && check.blocks_final_approval),
    "H2.8 decision surface 必须把 .codex 最小范围列为阻断",
  );
  assert(
    decisionSurface.decision_checks.some((check) => check.check_id === "rollback" && check.blocks_final_approval),
    "H2.8 decision surface 必须把 rollback 缺失列为阻断",
  );
  assert(
    decisionSurface.readback_boundary.result_count === null &&
      decisionSurface.readback_boundary.warnings.includes("readback_not_attempted_is_not_zero_results"),
    "H2.8 decision surface 必须说明未读回结果数未知",
  );
  assert(
    decisionSurface.permission_preview.denied_paths.some((path) => path.includes("auth/token")),
    "H2.8 permission preview 必须提示 secret / token 禁止展示",
  );
  const missingSessionSurface = deriveH2RealResumeExecutionDecisionSurface({
    previews: previews.map((preview) =>
      preview.adapter_id === "codex-local" && preview.operation_id === "resume"
        ? {
            ...preview,
            target_session_id: null,
            target_session_title: null,
            request: {
              ...preview.request,
              session_id: null,
            },
          }
        : preview,
    ),
    store: snapshot.session_continuation_store,
  });
  assert(
    missingSessionSurface.status === "blocked_waiting_target_session",
    "H2.8 decision surface 缺 target session 时必须明确阻断",
  );
  assert(
    missingSessionSurface.decision_checks.some((check) => check.check_id === "target_session" && check.blocks_final_approval),
    "H2.8 decision surface 必须把 target session 缺失列为阻断",
  );
  assert(
    readiness.readiness_items.some((item) => item.item_id === "prompt_hash_ref" && item.status === "missing"),
    "H2.2 readiness 必须缺少 prompt hash/ref",
  );
  assert(
    readiness.readiness_items.some((item) => item.item_id === "codex_home_scope" && item.status === "missing"),
    "H2.2 readiness 必须缺少 .codex 最小范围",
  );
  assert(
    readiness.readiness_items.some((item) => item.item_id === "user_confirmation" && item.status === "missing"),
    "H2.2 readiness 必须缺少用户确认",
  );
  assert(
    readiness.readiness_items.some((item) => item.item_id === "global_supervisor_confirmation" && item.status === "missing"),
    "H2.2 readiness 必须缺少全局主管确认",
  );

  const agentView = (
    <AgentView
      sessions={[session]}
      projects={[project]}
      adapterDescriptors={descriptors}
      sessionOperationDescriptors={operations}
      providerAvailabilitySummaries={summaries}
      sessionContinuationPreviews={previews}
      sessionContinuationStore={snapshot.session_continuation_store}
      workflowState={workflowStateWithProjectWorkflow}
      onRequestAction={captureAction}
    />
  );
  const agentViewText = visibleText(agentView);
  for (const expectedText of [
    "H2 真实恢复授权准备",
    "H2.8 最终批准决策面",
    "当前不可批准",
    "权限弹层预览",
    "审计 / 运行日志 / 读回预览",
    "未尝试读回",
    "结果数：未知/不可用",
    "permission_preview_is_not_approval",
    "h2_phase_b_final_approval_not_granted",
    "等待授权矩阵",
    "不会发送提示词",
    "不会执行 codex exec resume",
    "不会读写 /Users/yoyi/.codex",
    "目标会话",
    ".codex 最小范围",
    "提示词引用 / 哈希",
    "回滚：",
    "h2_readiness_is_not_execution_authorization",
  ]) {
    assert(agentViewText.includes(expectedText), `H2.2 readiness UI 缺少 ${expectedText}`);
  }
  for (const forbiddenText of [
    "Codex 已收到任务",
    "真实 Codex 已执行",
    "prompt 已发送",
    ".codex 已读写",
    "H2 已完成",
    "H3 可开始",
    "readback 0 条",
    "0 条读回",
    "final approval 已批准",
  ]) {
    assert(!agentViewText.includes(forbiddenText), `H2.2 readiness UI 不应出现误导文案：${forbiddenText}`);
  }
  const agentViewMarkup = renderToStaticMarkup(agentView);
  for (const forbiddenButtonText of ["执行", "resume", "发送", "确认", "授权", "重试"]) {
    assert(
      !buttonTextsInMarkup(agentViewMarkup).includes(forbiddenButtonText),
      `H2.2 readiness 不应渲染执行或授权按钮：${forbiddenButtonText}`,
    );
  }

  const secretaryContext = deriveSecretaryContext({
    snapshot: {
      ...snapshot,
      agent_adapters: descriptors,
      session_operations: operations,
      provider_availability: summaries,
      session_continuation_previews: previews,
    },
    workflowState: workflowStateWithProjectWorkflow,
  });
  assert(
    secretaryContext.risk_signals.some((risk) => risk.kind === "h2_real_resume_decision_boundary"),
    "秘书风险应包含 H2.8 final approval 决策面边界",
  );
  assert(
    secretaryContext.suggestions.some((suggestion) => suggestion.kind === "inspect_h2_real_resume_decision_surface"),
    "秘书建议应包含查看 H2.8 final approval 决策面",
  );
  for (const forbiddenProposalText of ["发送", "发消息", "resume", "批准", "确认", "重试"]) {
    assert(
      !secretaryContext.action_proposals.some((proposal) => proposal.title.includes(forbiddenProposalText)),
      `秘书 action proposal 不应变成 H2.8 执行动作：${forbiddenProposalText}`,
    );
  }

  const confirmedPreview = previews.find((preview) => preview.adapter_id === "codex-local" && preview.operation_id === "resume");
  assert(confirmedPreview, "H2.8 duplicate guard fixture 缺少 codex-local resume preview");
  const duplicateStore = h2DuplicateSessionContinuationStoreFixture({
    baseStore: snapshot.session_continuation_store,
    confirmedPreview,
    projectRoot: project.project_root,
    sessionThreadId: session.thread_id,
  });
  const duplicateSurface = deriveH2RealResumeExecutionDecisionSurface({
    previews,
    store: duplicateStore,
  });
  assert(
    duplicateSurface.status === "blocked_by_duplicate_attempt",
    "H2.8 decision surface 必须优先阻断 queued/running duplicate attempt",
  );
  assert(duplicateSurface.duplicate_attempt_blocked, "H2.8 duplicate attempt 必须阻断 final approval");
  assert(
    duplicateSurface.readback_boundary.result_count === null &&
      duplicateSurface.readback_boundary.warnings.includes("readback_unavailable_is_not_zero_results"),
    "H2.8 duplicate/readback unavailable 必须保持结果数未知",
  );
}

function runRuntimeSessionAttentionScenario() {
  const attention = runtimeAttentionFixtures(session.thread_id);
  const summaries = [runtimeSessionSummaryFixture(session.thread_id, attention)];
  const runtimeSnapshot: WorkbenchSnapshot = {
    ...snapshot,
    runtime_session_attention: attention,
    session_run_status_summaries: summaries,
  };
  const agentView = (
    <AgentView
      sessions={[session]}
      projects={[project]}
      adapterDescriptors={backendAgentAdapterDescriptors}
      sessionOperationDescriptors={backendSessionOperationDescriptors}
      providerAvailabilitySummaries={backendProviderAvailabilitySummaries}
      sessionContinuationPreviews={[]}
      sessionContinuationStore={snapshot.session_continuation_store}
      runtimeSessionAttention={attention}
      sessionRunStatusSummaries={summaries}
      workflowState={workflowStateWithProjectWorkflow}
      onRequestAction={captureAction}
    />
  );
  const agentText = visibleText(agentView);
  for (const expectedText of [
    "运行关注 / E6",
    "等待确认",
    "边界保护阻断",
    "读回不可用",
    "读回失败",
    "结果数：未知/不可用",
    "真实读回：否",
  ]) {
    assert(agentText.includes(expectedText), `E6 runtime attention UI 缺少 ${expectedText}`);
  }
  for (const forbiddenText of [
    "已自动重试",
    "已停止 agent",
    "已重启 agent",
    "真实派发已完成",
    "真实 prompt 已发送",
    "Codex 已收到任务",
    "真实 readback 已完成",
    "readback 0 条",
    "失败已自动恢复",
    "Claude Code 已接管",
    "OpenClaw 已运行",
    "OpenCode 已 resume",
  ]) {
    assert(!agentText.includes(forbiddenText), `E6 runtime attention UI 不应出现误导文案：${forbiddenText}`);
  }
  const agentMarkup = renderToStaticMarkup(agentView);
  for (const forbiddenButtonText of ["发送", "resume", "重试", "停止", "重启"]) {
    assert(!buttonTextsInMarkup(agentMarkup).includes(forbiddenButtonText), `E6 不应渲染执行类按钮：${forbiddenButtonText}`);
  }

  const secretaryContext = deriveSecretaryContext({
    snapshot: runtimeSnapshot,
    workflowState: workflowStateWithProjectWorkflow,
  });
  assert(
    secretaryContext.risk_signals.some((risk) => risk.kind === "runtime_session_attention_boundary"),
    "秘书风险应包含 E6 runtime session attention 边界",
  );
  assert(
    secretaryContext.suggestions.some((suggestion) => suggestion.kind === "inspect_runtime_session_attention"),
    "秘书建议应包含查看 E6 runtime attention",
  );
  for (const forbiddenProposalText of ["发送", "resume", "批准", "确认", "重试", "停止", "重启"]) {
    assert(
      !secretaryContext.action_proposals.some((proposal) => proposal.title.includes(forbiddenProposalText)),
      `秘书 action proposal 不应变成 E6 执行动作：${forbiddenProposalText}`,
    );
  }

  const commonProps = rightDetailPanelCommonPropsFixture({
    snapshot: runtimeSnapshot,
    workflowState: workflowStateWithProjectWorkflow,
    secretaryContext,
  });
  const runningPanelText = visibleText(<RightDetailPanel activePanel="running" {...commonProps} />);
  assert(runningPanelText.includes("运行中摘要"), "运行中入口应使用职责化摘要标题");
  assert(runningPanelText.includes("不停止、恢复、重试或启动真实执行"), "运行中入口应声明只汇总不执行");
  assert(runningPanelText.includes("边界保护阻断"), "运行中入口应显示 E6 session summary");
  assert(runningPanelText.includes("读回不可用"), "运行中入口应显示读回边界");
  const todoPanelText = visibleText(<RightDetailPanel activePanel="todos" {...commonProps} />);
  assert(todoPanelText.includes("待处理事项"), "待办入口应使用职责化摘要标题");
  assert(todoPanelText.includes("不替用户批准、派发或写入状态"), "待办入口不应暗示自动处理");
  assert(todoPanelText.includes("查看 E6"), "待办入口应显示需要用户查看的 runtime attention");
  const notificationPanelText = visibleText(<RightDetailPanel activePanel="notifications" {...commonProps} />);
  assert(notificationPanelText.includes("通知摘要"), "通知入口应使用职责化摘要标题");
  assert(notificationPanelText.includes("读取状态"), "通知入口不应再暴露索引读取状态措辞");
  assert(!notificationPanelText.includes("索引读取状态"), "通知入口不应把普通提示写成索引状态面板");
  assert(notificationPanelText.includes("读回失败"), "通知入口应显示读回失败摘要");
}

function runRuntimeLogBoundaryScenario() {
  const runtimeStore = runtimeLogStoreFixture(project.project_root, session.thread_id);
  const runtimeSnapshot: WorkbenchSnapshot = {
    ...snapshot,
    runtime_log_store: runtimeStore,
  };
  const commonProps = rightDetailPanelCommonPropsFixture({
    snapshot: runtimeSnapshot,
    workflowState: workflowStateWithProjectWorkflow,
    secretaryContext: deriveSecretaryContext({
      snapshot: runtimeSnapshot,
      workflowState: workflowStateWithProjectWorkflow,
    }),
  });
  const managementPanelText = visibleText(<RightDetailPanel activePanel="audit" {...commonProps} />);

  for (const expectedText of [
    "管理",
    "管理摘要",
    "原始材料仍在开发者区或详情中查看",
    "健康 / 诊断边界",
    "degraded_readonly",
    "工作流事实层",
    "读回不可用不是 0 条结果",
    "不自动修复 store",
    "诊断 bundle",
    "日志 / 审计边界",
    "运行日志与审计事件不能互相替代",
    "应用会话",
    "工作流运行",
    "派发尝试",
    "读回",
    "权限等待",
    "诊断事件",
    "审计引用 1",
  ]) {
    assert(managementPanelText.includes(expectedText), `G1 管理入口运行日志摘要缺少 ${expectedText}`);
  }

  const railManagement = workspaceRailItems.find((item) => item.key === "audit");
  assert(railManagement?.label === "管理", "审计和日志应收进右侧管理入口，不新增散开的日志入口");
  assert(!workspaceRailItems.some((item) => item.label.includes("日志") && item.key !== "audit"), "不应新增右侧日志顶级入口");
  assert(!workspaceRailItems.some((item) => item.label.includes("诊断") && item.key !== "audit"), "不应新增右侧诊断顶级入口");

  const serialized = JSON.stringify(runtimeStore);
  for (const forbiddenText of [
    "sk-test-secret",
    "raw provider credential",
    "完整 transcript",
    "full transcript",
    "OAuth",
    "auth.json",
    ".env",
    "keychain",
    "provider credential",
  ]) {
    assert(!serialized.includes(forbiddenText), `G1 runtime log store 不应包含敏感内容：${forbiddenText}`);
    assert(!managementPanelText.includes(forbiddenText), `G1 管理入口不应显示敏感内容：${forbiddenText}`);
  }

  assert(
    runtimeStore.entries.every((entry) => entry.redaction_status === "redacted_safe_summary"),
    "G1 runtime log entries 必须是脱敏摘要",
  );
  assert(
    runtimeStore.entries.some((entry) => entry.audit_refs.length === 1),
    "G1 runtime log 应只保留 audit_refs 引用",
  );
}

function runCandidateGovernanceScenario() {
  const {
    blackboardCandidateStore,
    confirmedMemoryCandidateStore,
    adoptedMemoryCandidateStore,
    observationStore,
    emptyMemoryCandidateStore,
    formalMemoryStore,
    adoptedFormalMemoryStore,
    memoryLintStore,
    taskMemoryPacketPreview,
  } = candidateGovernanceFixtures(project.project_root);
  const blackboardOverlay = buildBlackboardCandidateOverlay({
    store: blackboardCandidateStore,
    entries: workflowStateWithDerivedWorkflow.project_blackboards?.[0].entries ?? [],
  });
  assert(blackboardOverlay.status_by_entry_id["blackboard:offline:report:001"] === "candidate_confirmed_for_followup", "黑板候选 overlay 应能按 source_entry_id 映射确认状态");
  assert(blackboardOverlay.sidecar_name === "blackboard-candidates.v1.json", "黑板候选 sidecar 文件名不匹配");
  assert(!blackboardOverlay.warnings.includes("writes_formal_memory"), "黑板 overlay 不应写正式记忆 warning");

  const memorySummary = summarizeMemoryCandidateStore(confirmedMemoryCandidateStore);
  assert(memorySummary.sidecar_name === "memory-candidates.v1.json", "记忆候选 sidecar 文件名不匹配");
  assert(memorySummary.confirmed_count === 1, "记忆候选确认保留计数不匹配");
  assert(memorySummary.formal_memory_count === 0, "候选确认不应生成正式记忆");
  assert(memorySummary.adopted_count === 0, "普通 candidate_confirmed 不应显示为已采纳");
  assert(!memorySummary.display_text.includes("已记住"), "记忆候选 UI 文案不能说已记住");
  assert(!memorySummary.display_text.includes("正式记忆已写入"), "记忆候选 UI 文案不能说正式记忆已写入");

  const adoptedMemorySummary = summarizeMemoryCandidateStore(adoptedMemoryCandidateStore);
  assert(adoptedMemorySummary.adopted_count === 1, "已采纳候选计数不匹配");
  assert(adoptedMemorySummary.formal_memory_count === 0, "候选 sidecar 不应把采纳候选改成正式状态");
  assert(adoptedMemorySummary.first_adoption?.adopted_memory_id === "mem:formal:offline:002", "采纳摘要缺少 adopted_memory_id");
  assert(adoptedMemorySummary.first_adoption?.adopted_version_id === "memver:formal:offline:002", "采纳摘要缺少 adopted_version_id");
  assert(adoptedMemorySummary.first_adoption?.adopted_audit_event_id === "audit:formal:offline:002", "采纳摘要缺少 adopted_audit_event_id");

  const observationSummary = summarizeObservationStore(observationStore);
  assert(observationSummary.sidecar_name === "observations.v1.json", "observation sidecar 文件名不匹配");
  assert(observationSummary.recorded_count === 1, "recorded observation 计数不匹配");
  assert(observationSummary.candidate_created_count === 1, "candidate_created observation 计数不匹配");
  assert(observationSummary.recent_candidate_key === "memcand:v1:from-observation", "observation 摘要应显示最近 candidate_key");
  assert(observationSummary.display_text.includes("observation 不是正式记忆"), "observation 摘要必须说明不是正式记忆");

  capturedAction = null;
  const observationWorkflowProject = (
    <ProjectDetail
      project={project}
      sessions={[session]}
      workflowState={workflowStateWithDerivedWorkflow}
      observationStore={observationStore}
      memoryCandidateStore={emptyMemoryCandidateStore}
      selectedTool="workflow"
      onRequestAction={captureAction}
    />
  );
  const observationWorkflowText = visibleText(observationWorkflowProject);
  for (const expectedText of [
    "工作流观察",
    "observations.v1.json",
    "recorded 1",
    "candidate_created 1",
    "observation_candidate_created",
    "memcand:v1:from-observation",
    "观察可生成候选",
    "从工作流观察生成候选",
    "候选仍需确认 / 采纳",
    "observation 不是正式记忆",
  ]) {
    assert(observationWorkflowText.includes(expectedText), `工作流观察 UI 缺少 ${expectedText}`);
  }
  for (const forbiddenText of ["系统已记住", "自动学习完成", "observation 已成为正式记忆", "已注入任务包"]) {
    assert(!observationWorkflowText.includes(forbiddenText), `工作流观察 UI 不应出现越界文案：${forbiddenText}`);
  }

  const formalMemorySummary = summarizeFormalMemoryStore(formalMemoryStore);
  assert(formalMemorySummary.sidecar_name === "formal-memories.v1.json", "正式记忆 sidecar 文件名不匹配");
  assert(formalMemorySummary.record_count === 1, "正式记忆 record 计数不匹配");
  assert(formalMemorySummary.version_count === 1, "正式记忆 version 计数不匹配");
  assert(formalMemorySummary.audit_event_count === 1, "正式记忆 audit 计数不匹配");
  assert(formalMemorySummary.active_count === 1, "正式记忆 active 计数不匹配");
  assert(formalMemorySummary.recent_audit_event?.event_type === "memory_record_created", "正式记忆最近审计事件不匹配");
  assert(formalMemorySummary.display_text.includes("创建时写入 version 和 audit"), "正式记忆摘要应说明 version/audit 骨架");
  for (const forbiddenText of ["AI 自动记住", "候选已记住", "秘书已批准", "worker 已写入正式记忆", "完整记忆层完成", "系统已学习", "任务包注入已完成", "正式记忆完整完成"]) {
    assert(!formalMemorySummary.display_text.includes(forbiddenText), `正式记忆摘要不应出现越界文案：${forbiddenText}`);
  }

  const adoptedFormalMemorySummary = summarizeFormalMemoryStore(adoptedFormalMemoryStore);
  assert(adoptedFormalMemorySummary.recent_audit_event?.event_type === "memory_candidate_adopted_to_formal_memory", "正式记忆摘要应识别候选采纳审计");
  assert(adoptedFormalMemorySummary.display_text.includes("候选受控采纳审计已记录"), "正式记忆摘要应显示候选受控采纳审计");
  assert(!adoptedFormalMemorySummary.display_text.includes("M1 不包含候选采纳"), "采纳事件出现后不应继续显示 M1 候选采纳缺口文案");

  const memoryLintSummary = summarizeMemoryLintStore(memoryLintStore);
  assert(memoryLintSummary.sidecar_name === "memory-lint.v1.json", "记忆 lint sidecar 文件名不匹配");
  assert(memoryLintSummary.finding_count === 3, "记忆 lint finding 计数不匹配");
  assert(memoryLintSummary.open_count === 2, "记忆 lint open 计数不匹配");
  assert(memoryLintSummary.blocking_count === 1, "记忆 lint blocking 计数不匹配");
  assert(memoryLintSummary.needs_review_count === 1, "记忆 lint needs_review 计数不匹配");
  assert(memoryLintSummary.recent_run?.status === "blocked", "记忆 lint 最近 run 状态不匹配");
  for (const expectedText of [
    "记忆 lint 阻断摘要",
    "blocking finding 会阻止进入任务包",
    "lint 只生成待处理 finding",
    "不会自动修改正式记忆",
  ]) {
    assert(memoryLintSummary.display_text.includes(expectedText), `记忆 lint 摘要缺少 ${expectedText}`);
  }

  const taskMemoryPacketSummary = summarizeTaskMemoryPacketPreview(taskMemoryPacketPreview);
  assert(taskMemoryPacketSummary.included_count === 1, "任务记忆包预览 included 计数不匹配");
  assert(taskMemoryPacketSummary.excluded_count === 2, "任务记忆包预览 excluded 计数不匹配");
  assert(taskMemoryPacketSummary.review_material_count === 2, "任务记忆包预览待审查材料计数不匹配");
  assert(taskMemoryPacketSummary.reason_counts.candidate_unconfirmed === 1, "任务记忆包预览缺少 candidate_unconfirmed 计数");
  assert(taskMemoryPacketSummary.reason_counts.observation_not_formal_memory === 1, "任务记忆包预览缺少 observation_not_formal_memory 计数");
  assert(taskMemoryPacketSummary.display_text.includes("预览未注入任务包"), "任务记忆包预览摘要必须说明未注入");
  assert(taskMemoryPacketSummary.reason_text.includes("candidate_unconfirmed"), "任务记忆包预览 reason 摘要缺少 candidate_unconfirmed");
  assert(taskMemoryPacketSummary.reason_text.includes("observation_not_formal_memory"), "任务记忆包预览 reason 摘要缺少 observation_not_formal_memory");
  const taskPackageMemoryInjectionSummary = summarizeTaskPackageMemoryInjection(
    workflowStateWithDerivedWorkflow.project_workflows[0].derived_workflow?.task_packages[0].memory_injection_summary,
  );
  assert(taskPackageMemoryInjectionSummary.included_count === 1, "任务包记忆注入 included 计数不匹配");
  assert(taskPackageMemoryInjectionSummary.excluded_count === 2, "任务包记忆注入 excluded 计数不匹配");
  assert(taskPackageMemoryInjectionSummary.review_material_count === 2, "任务包记忆注入 review materials 计数不匹配");
  assert(!taskPackageMemoryInjectionSummary.stale, "任务包记忆注入 fixture 应为 fresh");
  for (const expectedText of [
    "任务包记忆注入摘要",
    "仅活跃正式记忆可进入任务包",
    "候选 / 观察仅作为待审查材料",
    "任务包内容不会回灌成正式记忆",
  ]) {
    assert(taskPackageMemoryInjectionSummary.display_text.includes(expectedText), `任务包记忆注入摘要缺少 ${expectedText}`);
  }

  const taskMemoryPacketWorkflowText = visibleText(
    <ProjectDetail
      project={project}
      sessions={[session]}
      workflowState={workflowStateWithDerivedWorkflow}
      selectedTool="workflow"
      taskMemoryPacketPreview={taskMemoryPacketPreview}
      memoryLintStore={memoryLintStore}
      onRequestAction={captureAction}
    />,
  );
  for (const expectedText of [
    "任务记忆包预览",
    "入选 1",
    "排除 2",
    "待审查材料 2",
    "估算 token 28/8000",
    "candidate_unconfirmed",
    "observation_not_formal_memory",
    "仅启用态正式记忆可入选",
    "候选 / 观察仅作为待审查材料",
    "不进入正式记忆列表",
    "preview_only_not_injected",
    "记忆 lint sidecar",
    "memory-lint.v1.json",
    "记忆 lint 阻断摘要",
    "任务包记忆注入摘要",
    "task-package-memory-packet-snapshot:v1:offline:001",
    "入选正式记忆",
    "快照状态",
    "新鲜",
    "仅启用态正式记忆可进入任务包",
    "任务包内容不会回灌成正式记忆",
    "open 2",
    "blocking 1",
    "needs_review 1",
    "最近检查运行",
    "来源权限撤回",
    "blocking finding 会阻止进入任务包",
    "lint 只生成待处理 finding",
    "不会自动修改正式记忆",
  ]) {
    assert(taskMemoryPacketWorkflowText.includes(expectedText), `任务记忆包预览 UI 缺少 ${expectedText}`);
  }
  for (const forbiddenText of [
    "系统已记住",
    "自动学习完成",
    "候选已进入任务包",
    "observation 已注入任务包",
    "worker 已收到记忆包",
    "真实 worker 已执行",
    "系统已自动记住",
    "任务包内容已写入正式记忆",
    "任务包注入已完成",
    "中间版本记忆层完成",
    "AI 已自动解决冲突",
    "系统已废弃旧记忆",
    "旧记忆已自动更新",
    "正式记忆生命周期完成",
  ]) {
    assert(!taskMemoryPacketWorkflowText.includes(forbiddenText), `任务记忆包预览 UI 不应出现越界文案：${forbiddenText}`);
  }
}

function runMemoryManagementCenterScenario() {
  const { formalMemoryStore, memoryCandidateStore, observationStore, memoryCaptureStore } =
    memoryCenterCoreFixtures();
  const { memoryLintStore, memoryEntityRelationStore, memoryEntityRelationPreview } =
    memoryCenterGovernanceFixtures();
  const { memoryPatternStore, maturePatternPreview } = memoryPatternFixtures();

  const summary = deriveMemoryManagementSummary({
    projects: [project],
    workflowState: workflowStateWithDerivedWorkflow,
    formalMemoryStore,
    memoryCaptureStore,
    memoryCandidateStore,
    observationStore,
    memoryLintStore,
    memoryEntityRelationStore,
    memoryEntityRelationPreview,
    memoryPatternStore,
    maturePatternPreview,
  });

  assert(summary.source_kind === "frontend_read_model", "记忆中心必须声明前端只读读模型");
  assert(summary.formal_memories.length === 2, "记忆中心正式记忆数量不匹配");
  assert(summary.candidate_memories.length === 2, "记忆中心候选数量不匹配");
  assert(summary.observation_sources.length === 1, "记忆中心观察来源数量不匹配");
  assert(summary.memory_workbench_summary.capture_count === 2, "记忆工作台捕获数量不匹配");
  assert(summary.memory_workbench_summary.capture_compensation_count === 1, "记忆工作台补证数量不匹配");
  assert(summary.memory_workbench_summary.confirmed_pending_formalization_count === 1, "记忆工作台待正式化数量不匹配");
  assert(summary.memory_workbench_summary.task_package_included_count === 1, "记忆工作台任务包入选数量不匹配");
  assert(summary.memory_workbench_summary.task_package_review_material_count === 2, "记忆工作台待审材料数量不匹配");
  assert(summary.memory_workbench_summary.action_items.some((item) => item.kind === "repair_capture_link"), "记忆工作台缺补证行动项");
  assert(summary.memory_workbench_summary.action_items.some((item) => item.kind === "confirm_formalization"), "记忆工作台缺正式化行动项");
  assert(summary.memory_workbench_summary.boundary_text.includes("观察和候选都不是正式记忆"), "记忆工作台缺候选 / 观察边界");
  assert(summary.task_package_summary.included_count === 1, "记忆中心任务包 included 摘要不匹配");
  assert(summary.task_package_summary.review_material_count === 2, "记忆中心任务包待审材料摘要不匹配");
  assert(summary.entity_relation_summary.entity_candidate_count === 1, "实体候选摘要不匹配");
  assert(summary.entity_relation_summary.merge_candidate_count === 1, "实体 dedupe 候选摘要不匹配");
  assert(summary.entity_relation_summary.relation_candidate_count === 2, "关系候选摘要不匹配");
  assert(summary.entity_relation_summary.confirmed_relation_count === 1, "已确认关系摘要不匹配");
  assert(summary.entity_relation_summary.display_text.includes("LLM 推断仅作候选"), "实体关系摘要缺 LLM 候选边界");
  assert(summary.entity_relation_summary.display_text.includes("相似度命中仅作候选"), "实体关系摘要缺相似度候选边界");
  assert(summary.mature_pattern_summary.mature_pattern_candidate_count === 1, "成熟模式候选摘要不匹配");
  assert(summary.mature_pattern_summary.cluster_report_count === 1, "跨项目主题报告摘要不匹配");
  assert(summary.mature_pattern_summary.user_confirmation_required_count === 1, "成熟模式用户确认计数不匹配");
  assert(summary.mature_pattern_summary.acceptance_summary?.passed_count === 3, "M1-M12 gate 摘要不匹配");
  assert(summary.mature_pattern_summary.boundary_text.includes("候选未确认，不会进入任务包"), "成熟模式摘要缺未确认边界");
  assert(summary.project_summaries.some((item) => item.project_name === "codex-workbench"), "记忆中心缺项目相关记忆摘要");

  const includedMemory = summary.formal_memories.find((item) => item.claim === "接口验收必须保留控制核心边界。");
  assert(includedMemory, "记忆中心缺少可入选正式记忆");
  assert(includedMemory.kind_label === "正式记忆", "正式记忆条目必须标识正式记忆");
  assert(includedMemory.task_eligibility.label === "可进入任务包", "active 正式记忆应显示可进入任务包");
  assert(includedMemory.task_eligibility.included_in_task_package, "任务包 available_memory_refs 命中的正式记忆应显示已被快照引用");
  assert(includedMemory.version_summary.includes("v1"), "正式记忆条目缺少版本摘要");
  assert(includedMemory.audit_summary.includes("memory_record_created"), "正式记忆条目缺少审计摘要");

  const blockedMemory = summary.formal_memories.find((item) => item.claim === "撤回来源不能进入任务记忆包。");
  assert(blockedMemory, "记忆中心缺少 lint 阻断正式记忆");
  assert(blockedMemory.task_eligibility.label === "被检查阻断", "未关闭阻断发现应阻断任务包入选");
  assert(blockedMemory.conflict_summary.includes("未关闭阻断"), "检查阻断正式记忆应显示冲突摘要");

  const candidate = summary.candidate_memories.find((item) => item.claim === "候选需要确认要求和风险提示。");
  assert(candidate, "记忆中心缺候选条目");
  assert(candidate.kind_label === "候选记忆", "候选条目必须标识候选记忆");
  assert(candidate.formal_memory_boundary.includes("不是正式记忆"), "候选条目必须说明不是正式记忆");
  assert(candidate.task_position.label === "待审查材料", "未采纳候选应显示待审查材料");
  assert(candidate.confirmation_summary.includes("需要用户确认"), "候选条目缺确认要求");

  const adoptedCandidate = summary.candidate_memories.find((item) => item.claim === "采纳回链必须可见但仍保留候选身份。");
  assert(adoptedCandidate, "记忆中心缺受控采纳候选");
  assert(adoptedCandidate.adoption_summary.includes("候选已被受控采纳"), "候选采纳回链应用允许文案说明");
  assert(adoptedCandidate.kind_label === "候选记忆", "已采纳候选在候选列表仍应显示候选记忆");

  const memoryCenterText = visibleText(
    <MemoryCenterView
      projects={[project]}
      workflowState={workflowStateWithDerivedWorkflow}
      formalMemoryStore={formalMemoryStore}
      memoryCaptureStore={memoryCaptureStore}
      memoryCandidateStore={memoryCandidateStore}
      observationStore={observationStore}
      memoryLintStore={memoryLintStore}
      memoryEntityRelationStore={memoryEntityRelationStore}
      memoryPatternStore={memoryPatternStore}
      onPreviewMemoryEntityRelationCandidates={() => Promise.resolve(memoryEntityRelationPreview)}
      onPreviewMaturePatterns={() => Promise.resolve(maturePatternPreview)}
      hasRealSnapshot
    />,
  );
  for (const expectedText of [
    "正式记忆",
    "记忆工作台",
    "捕获 / 候选 / 任务记忆包",
    "记忆链路",
    "捕获 2 / 观察 1 / 候选 2 / 正式 2",
    "待正式化 1",
    "需补证 1",
    "任务记忆包快照 1 个",
    "确认正式化",
    "补齐捕获链路",
    "候选和观察不会冒充正式记忆",
    "候选记忆",
    "来源",
    "版本 v1",
    "审计 memory_record_created",
    "权限策略未记录",
    "外发 local_only",
    "外发 blocked",
    "可进入任务包",
    "任务包冻结快照已引用",
    "被检查阻断",
    "未关闭阻断",
    "待审查材料",
    "候选已被受控采纳",
    "观察来源",
    "观察不是正式记忆",
    "任务包冻结快照",
    "入选 1",
    "排除 2",
    "待审材料 2",
    "项目相关记忆摘要",
    "codex-workbench",
    "生命周期",
    "编辑提案",
    "废弃",
    "冻结",
    "解冻",
    "归档",
    "合并",
    "拆分",
    "上升为全局",
    "下沉为项目",
    "编辑会创建新版本，不覆盖旧版本",
    "实体候选",
    "关系候选",
    "已确认关系",
    "刷新实体 / 关系候选",
    "相似度命中仅作候选",
    "LLM 推断仅作候选",
    "已确认关系用于解释召回原因",
    "关系候选不会影响任务包入选清单",
    "维护任务",
    "维护任务摘要",
    "运行维护任务",
    "索引状态 stale",
    "维护任务只生成发现",
    "阻断级发现会阻止召回",
    "成熟模式 / 跨项目主题",
    "刷新成熟模式候选",
    "成熟模式候选",
    "跨项目重复边界：控制核心写入必须走确认",
    "需要用户确认",
    "候选未确认，不会进入任务包",
    "用户确认为正式记忆",
    "隔离",
    "要求补来源",
    "跨项目主题报告",
    "报告可下钻来源，但不是正式事实",
    "M1-M12 门禁摘要",
    "尚未生成 M12 预览",
    "最终权威验收仍在后续阶段",
  ]) {
    assert(memoryCenterText.includes(expectedText), `记忆中心 UI 缺少 ${expectedText}`);
  }
  for (const forbiddenText of [
    "已记住",
    "系统已长期记住",
    "候选已成为正式记忆",
    "观察已成为正式记忆",
    "worker 已收到记忆包",
    "中间版本记忆层已完成",
    "编辑正式记忆",
    "删除正式记忆",
    "归档正式记忆",
    "自动合并实体",
    "自动确认关系",
    "图谱已证明",
    "LLM 已确认关系",
    "相似度已合并实体",
    "GraphRAG 已接入",
    "关系候选已成为事实",
    "自动清理记忆",
    "自动修复记忆",
    "维护任务已改正式记忆",
    "成熟模式已自动成为规则",
    "自动成为技能",
    "自动成为全局规则",
    "自动写入全局记忆",
    "跨项目摘要已注入任务包",
    "聚类报告就是事实",
    "成熟模式已生效",
    "M13 已完成",
    "中间版本记忆系统最终验收完成",
  ]) {
    assert(!memoryCenterText.includes(forbiddenText), `记忆中心 UI 不应出现越界文案：${forbiddenText}`);
  }

  const memoryCenterMarkup = renderToStaticMarkup(
    <MemoryCenterView
      projects={[project]}
      workflowState={workflowStateWithDerivedWorkflow}
      formalMemoryStore={formalMemoryStore}
      memoryCaptureStore={memoryCaptureStore}
      memoryCandidateStore={memoryCandidateStore}
      observationStore={observationStore}
      memoryLintStore={memoryLintStore}
      memoryEntityRelationStore={memoryEntityRelationStore}
      memoryPatternStore={memoryPatternStore}
      hasRealSnapshot
    />,
  );
  for (const expectedClass of ["memory-center", "memory-workbench-panel", "formal-memory-item", "candidate-memory-item", "memory-detail-panel", "memory-entity-relation-panel", "memory-maintenance-panel", "memory-mature-pattern-panel"]) {
    assert(memoryCenterMarkup.includes(expectedClass), `记忆中心布局缺少 class ${expectedClass}`);
  }

  const {
    lifecycleAction,
    relationAction,
    maintenanceAction,
    maturePatternAction,
    quarantineMaturePatternAction,
  } = memoryPendingActionFixtures({
    projectRoot: project.project_root,
    formalMemoryStoreRevision: formalMemoryStore.revision,
    memoryCandidateStoreRevision: memoryCandidateStore.revision,
    memoryLintStoreRevision: memoryLintStore.revision,
    memoryEntityRelationStoreRevision: memoryEntityRelationStore.revision,
    memoryPatternStoreRevision: memoryPatternStore.revision,
    includedMemory,
    relationCandidate: memoryEntityRelationPreview.relation_candidates[0],
    maturePatternCandidate: memoryPatternStore.mature_pattern_candidates[0],
  });
  const lifecycleDialogText = visibleText(
    <PermissionDialog action={lifecycleAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of [
    "正式记忆 废弃",
    "formal-memories.v1.json",
    "确认权",
    "project_director_or_user_confirmation",
    "影响 1 条正式记忆",
    "非活跃记忆默认不进任务包",
    "原版本 v1 / 新版本 v2",
    "会新增版本和审计",
  ]) {
    assert(lifecycleDialogText.includes(expectedText), `正式记忆 lifecycle 确认弹层缺少 ${expectedText}`);
  }
  const relationDialogText = visibleText(
    <PermissionDialog action={relationAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of [
    "确认关系候选",
    "memory-entity-relations.v1.json",
    "接口验收必须保留控制核心边界。",
    "接口契约资料",
    "已确认关系用于解释召回原因",
    "关系候选不会作为正式事实影响工作者",
  ]) {
    assert(relationDialogText.includes(expectedText), `关系候选确认弹层缺少 ${expectedText}`);
  }

  const maintenanceDialogText = visibleText(
    <PermissionDialog action={maintenanceAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of [
    "运行记忆维护任务",
    "memory-lint.v1.json",
    "维护运行",
    "维护任务只生成发现 / 报告",
    "阻断级发现会阻止召回",
    "不会自动修改正式记忆",
  ]) {
    assert(maintenanceDialogText.includes(expectedText), `维护任务确认弹层缺少 ${expectedText}`);
  }
  for (const forbiddenText of ["自动清理记忆", "自动修复记忆", "自动合并重复记忆", "维护任务已改正式记忆"]) {
    assert(!maintenanceDialogText.includes(forbiddenText), `维护任务确认弹层不应出现越界文案：${forbiddenText}`);
  }

  assert(maturePatternAction?.kind === "record-mature-pattern-decision", "成熟模式确认按钮应生成 record-mature-pattern-decision action");
  assert(maturePatternAction.maturePatternDecision?.decision === "confirm_as_formal_memory", "成熟模式确认 action 决定类型不匹配");
  assert(maturePatternAction.maturePatternDecision.actor_role === "user", "成熟模式正式化必须由用户角色确认");
  assert(maturePatternAction.maturePatternDecision.confirmed_by === "user", "成熟模式正式化必须 confirmed_by user");
  assert(maturePatternAction.maturePatternDecision.expected_pattern_store_revision === memoryPatternStore.revision, "成熟模式确认 action 缺 M12 revision guard");
  assert(maturePatternAction.maturePatternDecision.expected_formal_store_revision === formalMemoryStore.revision, "成熟模式确认 action 缺 formal memory revision guard");

  const maturePatternDialogText = visibleText(
    <PermissionDialog action={maturePatternAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of [
    "用户确认成熟模式候选",
    "memory-patterns.v1.json / formal-memories.v1.json",
    "跨项目重复边界：控制核心写入必须走确认",
    "用户确认写入正式记忆",
    "候选和跨项目主题报告未确认不进入任务包",
    "写版本、审计和来源引用",
    "只有用户确认正式化时才会联动 formal-memories.v1.json",
  ]) {
    assert(maturePatternDialogText.includes(expectedText), `成熟模式确认弹层缺少 ${expectedText}`);
  }
  for (const forbiddenText of ["自动成为技能", "自动成为全局规则", "自动写入全局记忆", "跨项目摘要已注入任务包", "成熟模式已生效"]) {
    assert(!maturePatternDialogText.includes(forbiddenText), `成熟模式确认弹层不应出现越界文案：${forbiddenText}`);
  }

  const quarantineDialogText = visibleText(
    <PermissionDialog action={quarantineMaturePatternAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of ["隔离成熟模式候选", "memory-patterns.v1.json", "隔离候选", "未确认正式化", "候选和跨项目主题报告未确认不进入任务包"]) {
    assert(quarantineDialogText.includes(expectedText), `成熟模式隔离弹层缺少 ${expectedText}`);
  }
  assert(!quarantineDialogText.includes("memory-patterns.v1.json / formal-memories.v1.json"), "隔离动作不应声明写 formal store");
}

function runKnowledgeBaseBoundaryScenario() {
  const {
    formalMemoryStore,
    knowledgeWorkflowState,
    memoryCandidateStore,
    projectWithKnowledge,
  } = knowledgeBaseBoundaryFixtures(project, workflowStateWithDerivedWorkflow);

  const summary = deriveKnowledgeBaseSummary({
    projects: [projectWithKnowledge],
    workflowState: knowledgeWorkflowState,
    formalMemoryStore,
    memoryCandidateStore,
  });
  assert(summary.source_kind === "frontend_read_model", "知识库读模型必须声明前端只读来源");
  assert(summary.documents.length === 1, "知识库读模型应包含 authority file 文档");
  assert(summary.documents[0].formal_memory_links.length === 1, "知识库文档应反向链接正式记忆");
  assert(summary.documents[0].candidate_links.length === 1, "知识库文档应反向链接候选记忆");
  assert(summary.documents[0].task_reference_summary.reference_count === 1, "知识库文档应统计任务包知识引用");
  assert(summary.documents[0].candidate_draft.input.source_refs[0].source_type === "knowledge_doc", "候选草案来源必须是 knowledge_doc");
  assert(summary.documents[0].candidate_draft.input.generated_from === "knowledge_summary", "知识资料候选必须走 knowledge_summary 来源类型");
  assert(summary.obsidian_boundary.native_sync_status === "未执行 Obsidian 原生同步", "M8 只能显示 Obsidian-compatible 占位");

  capturedAction = null;
  const knowledgeView = (
    <KnowledgeBaseView
      projects={[projectWithKnowledge]}
      workflowState={knowledgeWorkflowState}
      formalMemoryStore={formalMemoryStore}
      memoryCandidateStore={memoryCandidateStore}
      hasRealSnapshot
      onRequestAction={captureAction}
    />
  );
  const knowledgeText = visibleText(knowledgeView);
  for (const expectedText of [
    "知识库资料",
    "Obsidian-compatible 占位",
    "未执行 Obsidian 原生同步",
    "知识库是材料和笔记空间",
    "正式记忆是经过确认、来源、版本、审计和权限治理的行为上下文",
    "接口契约资料",
    "关联正式记忆 1",
    "关联候选 1",
    "任务包知识引用 1",
    "正式记忆引用了该知识库来源",
    "提出记忆候选",
    "只生成候选，不写正式记忆",
  ]) {
    assert(knowledgeText.includes(expectedText), `知识库 UI 缺少 ${expectedText}`);
  }
  for (const forbiddenText of [
    "已接入 Obsidian 原生同步",
    "vault 已自动扫描",
    "知识库已自动记住",
    "文档已成为正式记忆",
    "知识命中已成为正式记忆",
    "知识命中已注入任务包",
    "中间版本记忆层已完成",
  ]) {
    assert(!knowledgeText.includes(forbiddenText), `知识库 UI 不应出现越界文案：${forbiddenText}`);
  }

  const candidateButton = findElement(
    knowledgeView,
    (element) => element.type === "button" && visibleText(element).includes("提出记忆候选"),
  );
  assert(candidateButton, "知识库 UI 缺少提出记忆候选按钮");
  const clickCandidate = candidateButton.props?.onClick;
  assert(typeof clickCandidate === "function", "提出记忆候选按钮缺少 onClick");
  clickCandidate();
  const knowledgeCandidateAction = capturedAction as PendingAction | null;
  assert(knowledgeCandidateAction?.kind === "create-memory-candidate", "知识库候选按钮应生成 create-memory-candidate action");
  assert(knowledgeCandidateAction.memoryCandidateCreation?.source_refs[0].source_type === "knowledge_doc", "候选 action 必须保留 knowledge_doc source_ref");
  assert(knowledgeCandidateAction.memoryCandidateCreation?.generated_from === "knowledge_summary", "候选 action 必须使用 knowledge_summary generated_from");
  assert(knowledgeCandidateAction.boundary?.includes("只生成候选，不写正式记忆"), "候选 action 必须说明只生成候选");
  assert(!knowledgeCandidateAction.boundary?.includes("formal-memories.v1.json"), "候选 action 不应声明写正式记忆 store");

  const actionDialogText = visibleText(
    <PermissionDialog action={knowledgeCandidateAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of ["提出记忆候选", "memory-candidates.v1.json", "只生成候选，不写正式记忆", "knowledge_doc", "接口契约资料"]) {
    assert(actionDialogText.includes(expectedText), `知识库候选确认弹层缺少 ${expectedText}`);
  }
}

function runSecretaryReadModelScenario() {
  const {
    blackboardCandidateStore,
    memoryCandidateStore,
    secretarySnapshot,
  } = secretaryReadModelFixtures(snapshot, project);
  const context = deriveSecretaryContext({
    snapshot: secretarySnapshot,
    workflowState: workflowStateWithDerivedWorkflow,
    blackboardCandidateStore,
    memoryCandidateStore,
    workflowStateError: "离线工作流状态错误",
  });

  assert(context.source_kind === "derived_read_model", "秘书上下文必须声明 derived_read_model");
  assert(context.warnings.includes("secretary_context_is_read_only"), "秘书上下文必须声明只读边界");
  assert(context.risk_signals.some((risk) => risk.kind === "workflow_state_error"), "秘书风险缺少 workflowStateError");
  assert(context.risk_signals.some((risk) => risk.kind === "diagnostic_warning"), "秘书风险缺少 diagnostics warning");
  assert(context.risk_signals.some((risk) => risk.kind === "pending_permission"), "秘书风险缺少待处理权限");
  assert(context.risk_signals.some((risk) => risk.kind === "failed_execution_attempt"), "秘书风险缺少 failed attempt");
  assert(context.risk_signals.some((risk) => risk.kind === "timed_out_execution_attempt"), "秘书风险缺少 timed_out attempt");
  assert(context.risk_signals.some((risk) => risk.kind === "pending_blackboard_candidate"), "秘书风险缺少 pending 黑板候选");
  assert(context.risk_signals.some((risk) => risk.kind === "pending_memory_candidate"), "秘书风险缺少 pending 记忆候选");
  assert(context.risk_signals.some((risk) => risk.kind === "adapter_warning"), "秘书风险缺少 adapter warning");
  assert(context.risk_signals.some((risk) => risk.kind === "session_operation_boundary"), "秘书风险缺少会话操作边界提醒");
  assert(context.suggestions.some((suggestion) => suggestion.kind === "review_permission"), "秘书建议缺少权限确认");
  assert(context.suggestions.some((suggestion) => suggestion.kind === "inspect_failed_workflow"), "秘书建议缺少失败/超时查看");
  assert(context.suggestions.some((suggestion) => suggestion.kind === "review_candidate"), "秘书建议缺少黑板候选治理");
  assert(context.suggestions.some((suggestion) => suggestion.kind === "review_memory_candidate"), "秘书建议缺少记忆候选审查");
  assert(context.suggestions.every((suggestion) => suggestion.requires_user_confirmation), "秘书建议都必须需要用户确认");
  assert(context.suggestions.every((suggestion) => !suggestion.is_fact_change), "秘书建议不能是事实变更");
  assert(context.action_proposals.every((proposal) => !proposal.executable_now), "秘书 action proposal 不能立即执行");
  assert(!context.action_proposals.some((proposal) => proposal.title.includes("adapter")), "计划中 adapter 不能变成秘书可执行 action proposal");
  assert(context.action_proposals.every((proposal) => proposal.requires_user_confirmation), "秘书 action proposal 必须需要确认");
  assert(context.action_proposals.every((proposal) => proposal.blocked_reason.length > 0), "秘书 action proposal 必须说明阻塞原因");
  assert(context.memory_candidates.some((candidate) => candidate.boundary === "候选不等于工作台已经长期记住。"), "秘书记忆候选必须显示候选边界");

  const secretaryText = visibleText(<SecretaryBrief context={context} />);
  for (const expectedText of ["秘书只读摘要", "需要你确认", "候选，不是正式记忆", "建议，不是事实变更"]) {
    assert(secretaryText.includes(expectedText), `秘书摘要缺少 ${expectedText}`);
  }
  for (const forbiddenText of ["秘书已处理", "秘书已执行", "已记住", "正式事实已写入"]) {
    assert(!secretaryText.includes(forbiddenText), `秘书摘要不应出现越界文案：${forbiddenText}`);
  }
}

function runRightRailSecretarySurfaceScenario() {
  const secretaryContext = deriveSecretaryContext({
    snapshot,
    workflowState: workflowStateWithDerivedWorkflow,
    blackboardCandidateStore: null,
    memoryCandidateStore: null,
    workflowStateError: null,
  });
  const secretaryRailItem = workspaceRailItems.find((item) => item.key === "secretary");
  assert(secretaryRailItem, "右侧竖栏缺少秘书独立入口");
  assert(secretaryRailItem.label === "秘书", "秘书入口 label 应保持为独立秘书入口");

  const commonProps = rightDetailPanelCommonPropsFixture({
    snapshot,
    workflowState: workflowStateWithDerivedWorkflow,
    secretaryContext,
  });
  for (const activePanel of ["notifications", "todos", "audit", "running"] as const) {
    const panelText = visibleText(<RightDetailPanel activePanel={activePanel} {...commonProps} />);
    assert(panelText.includes(rightRailPanelSummaryTitles[activePanel]), `${activePanel} 详情应保留自己的职责摘要列表`);
    assert(!panelText.includes("动态"), `${activePanel} 详情不应再使用泛化动态标题`);
    assert(!panelText.includes("秘书只读摘要"), `${activePanel} 详情不应渲染秘书只读摘要`);
    assert(!panelText.includes("建议，不是事实变更"), `${activePanel} 详情不应渲染秘书边界文案`);
    assert(!panelText.includes("候选，不是正式记忆"), `${activePanel} 详情不应渲染秘书记忆边界`);
  }

  const secretaryPanel = <RightDetailPanel activePanel="secretary" {...commonProps} />;
  const secretaryText = visibleText(secretaryPanel);
  for (const expectedText of ["秘书只读摘要", "建议，不是事实变更", "候选，不是正式记忆", "秘书模型只读"]) {
    assert(secretaryText.includes(expectedText), `秘书独立入口缺少 ${expectedText}`);
  }
  for (const forbiddenText of ["动态", "确认执行", "重新读取事实层"]) {
    assert(!secretaryText.includes(forbiddenText), `秘书独立入口不应出现写入或其他中心操作：${forbiddenText}`);
  }
  const secretaryActionButton = findElement(
    secretaryPanel,
    (element) => element.type === "button" && visibleText(element).trim() !== "×",
  );
  assert(!secretaryActionButton, "秘书独立入口除关闭按钮外不应出现任何操作按钮");
}

function runTranscriptCleaningScenario() {
  // A codex rollout carries the same turns twice: the clean event_msg stream and
  // the raw response_item stream that injects the system prompt / environment
  // context as a fake user turn. conversationTurns must keep only event_msg.
  const {
    events,
    mixedStream,
    noisyFallback,
    onlyResponseItems,
  } = transcriptCleaningFixtures();

  const turns = conversationTurns(events);
  const ids = turns.map((event) => event.event_id);
  assertDeepEqual(ids, ["e2", "e3"], "对话清洗应只保留 event_msg 的非空人/Agent消息");
  assert(
    !turns.some((event) => (event.text ?? "").includes("environment_context")),
    "对话清洗不应带出系统提示词/环境上下文注入",
  );

  assertDeepEqual(
    conversationTurns(mixedStream).map((event) => event.event_id),
    ["m1", "m2"],
    "event_msg 不完整时应补 response_item 中缺失的人/Agent轮次",
  );

  // Fallback: a rollout with no event_msg stream still shows its response_item turns.
  assertDeepEqual(
    conversationTurns(onlyResponseItems).map((event) => event.event_id),
    ["r1", "r2"],
    "没有 event_msg 流时应回退到 response_item 对话",
  );

  assertDeepEqual(
    conversationTurns(noisyFallback).map((event) => event.event_id),
    ["n4", "n5"],
    "response_item 回退也应过滤 thinking、system 注入和工具事件",
  );
}

function runSessionCenterHardeningScenario() {
  const {
    archivedSession,
    missingSession,
    sessions,
    transcript,
  } = sessionCenterHardeningFixtures(project, session, otherProjectSession);

  assertDeepEqual(
    filterAgentSessions(sessions, "readable", "Offline interaction").map((item) => item.thread_id),
    [session.thread_id],
    "搜索标题应缩小到匹配会话",
  );
  assertDeepEqual(
    filterAgentSessions(sessions, "all", "other-project").map((item) => item.thread_id),
    [otherProjectSession.thread_id],
    "搜索项目路径末段应缩小到匹配项目",
  );
  assertDeepEqual(
    filterAgentSessions(sessions, "missing", "").map((item) => item.thread_id),
    [missingSession.thread_id],
    "缺回放记录过滤应只显示缺失会话",
  );
  assertDeepEqual(
    filterAgentSessions(sessions, "archived", "").map((item) => item.thread_id),
    [archivedSession.thread_id],
    "已归档过滤应只显示归档会话",
  );
  const center = (
    <AgentSessionCenter
      sessions={sessions}
      selectedThreadId={session.thread_id}
      selectedSession={session}
      transcript={null}
      loadingThreadId={null}
      transcriptError="rollout_outside_allowed_dirs:/tmp/outside.jsonl"
      projectSessionCount={sessions.length}
      onOpenSession={() => {}}
      onRequestAction={captureAction}
    />
  );
  const centerText = visibleText(center);
  for (const expectedText of ["搜索会话", "可读取", "缺回放记录", "已归档", "路径被安全边界拒绝", "rollout_outside_allowed_dirs"]) {
    assert(centerText.includes(expectedText), `会话中心硬化缺少 ${expectedText}`);
  }
  const centerMarkup = renderToStaticMarkup(center);
  for (const expectedClass of ["agent-session-shell", "agent-session-list", "agent-transcript-panel", "session-state-filter"]) {
    assert(centerMarkup.includes(expectedClass), `会话中心固定布局缺少 class ${expectedClass}`);
  }
  assert(centerMarkup.includes("<button") && centerMarkup.includes("session-card"), "会话卡必须是可键盘聚焦的 button");

  const transcriptText = visibleText(<ChatTranscript transcript={transcript} />);
  for (const expectedText of ["已收纳较早 3 条消息", "展开全部", "展开", "开发者详情：过程事件", "复制", "const ok = true;"]) {
    assert(transcriptText.includes(expectedText), `Transcript 展示硬化缺少 ${expectedText}`);
  }
  assert(!transcriptText.includes("should be internal"), "工具事件不应默认进入主对话流");

  const transcriptCenterText = visibleText(
    <AgentSessionCenter
      sessions={sessions}
      selectedThreadId={session.thread_id}
      selectedSession={session}
      transcript={transcript}
      loadingThreadId={null}
      transcriptError={null}
      projectSessionCount={sessions.length}
      onOpenSession={() => {}}
      onRequestAction={captureAction}
    />,
  );
  assert(
    transcriptCenterText.includes("会话来源：只读历史查看，不是执行结果回收。"),
    "transcript viewer 的执行边界说明应收进开发者会话来源详情",
  );

  assert(centerMarkup.includes("session-search") && centerMarkup.includes("session-card"), "会话中心应渲染搜索框和 button 会话卡");
}

function runOfflineRoleOrchestrationScenario() {
  const parsed = parseOfflineDispatchBlock(defaultOfflineDispatchBlock, project.project_root);
  assert(parsed.ok, "默认离线派发块应能解析");
  assert(parsed.proposal.target_role_id === "codex-dev", "开发线应映射到 codex-dev");
  assert(parsed.proposal.task_title === "README 极小修改验证", "默认派发块任务名解析不匹配");
  assert(parsed.proposal.required_return.includes("验证结果"), "默认派发块缺少验证结果回传要求");

  const missing = parseOfflineDispatchBlock(missingOfflineDispatchBlock, project.project_root);
  assert(!missing.ok, "缺字段派发块不应解析成功");
  for (const expectedMissing of ["目标", "执行目录", "允许读取", "允许写入", "禁止事项", "验收标准", "超时", "回传要求"]) {
    assert(missing.missing.includes(expectedMissing), `缺字段派发块没有提示 ${expectedMissing}`);
  }

  const action = buildOfflineRoleDispatchAction(project.project_root, "work-item:offline:001", parsed.proposal);
  assertDeepEqual(
    action,
    expectedOfflineRoleDispatchAction(project.project_root, "work-item:offline:001", parsed.proposal),
    "离线角色派发 action 不匹配",
  );
  const actionDialogText = visibleText(
    <PermissionDialog action={action} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of [
    "离线派发给开发线",
    "目标角色",
    "开发线",
    "任务名",
    "README 极小修改验证",
    "必须回传",
    "验证结果",
    "不启动 Codex",
    "不执行 codex exec resume",
    "不写 /Users/yoyi/.codex",
    "工作流状态",
  ]) {
    assert(actionDialogText.includes(expectedText), `离线派发确认弹层缺少 ${expectedText}`);
  }

  const stubResult = buildOfflineStubResult(parsed.proposal);
  assert(stubResult.role_label === "开发线", "桩结果角色不匹配");
  assert(stubResult.summary.includes("没有执行真实 Codex 会话"), "桩结果必须说明没有真实执行");
  assert(stubResult.returned_to_director.includes("请总指导回收"), "桩结果应回传总指导");

  const handoffActions: PendingAction[] = [];
  const rolePanelWithPreparedDispatch = (
    <OfflineRoleOrchestrationPanel
      project={project}
      projectWorkflow={workflowStateWithPreparedOfflineDispatch.project_workflows[0]}
      sessions={[session]}
      onRequestAction={(action) => {
        handoffActions.push(action);
      }}
    />
  );
  const handoffButton = findElement(
    rolePanelWithPreparedDispatch,
    (element) => element.type === "button" && element.props?.type === "button" && visibleText(element).includes("写入角色回传"),
  );
  assert(handoffButton, "离线角色编排区缺少角色回传按钮");
  const clickHandoff = handoffButton.props?.onClick;
  assert(typeof clickHandoff === "function", "离线角色回传按钮没有 onClick");
  clickHandoff();
  const capturedHandoffAction = handoffActions[0];
  assert(capturedHandoffAction, "角色回传按钮没有捕获 action");
  assert(capturedHandoffAction?.kind === "offline-role-result-handoff", "角色回传按钮应生成离线回传 action");
  assert(
    capturedHandoffAction.offlineRoleResultHandoff?.dispatch_id === "offline-dispatch:fixture:prepared",
    "角色回传应绑定 prepared 离线派发",
  );
  assert(
    capturedHandoffAction.offlineRoleResultHandoff?.summary.includes("已落账离线派发"),
    "角色回传应使用已落账派发块生成摘要",
  );

  const reviewActions: PendingAction[] = [];
  const rolePanelWithCompletedDispatch = (
    <OfflineRoleOrchestrationPanel
      project={project}
      projectWorkflow={workflowStateWithCompletedOfflineDispatch.project_workflows[0]}
      sessions={[session]}
      onRequestAction={(action) => {
        reviewActions.push(action);
      }}
    />
  );
  const reviewPanelText = visibleText(rolePanelWithCompletedDispatch);
  assert(reviewPanelText.includes("ready_for_review"), "完成回传后应保留 ready_for_review 工作项作为账本锚点");
  assert(reviewPanelText.includes("offline-dispatch:fixture:completed"), "完成回传后应显示 completed 离线派发");
  const reviewButton = findElement(
    rolePanelWithCompletedDispatch,
    (element) => element.type === "button" && element.props?.type === "button" && visibleText(element).includes("写入总指导回收"),
  );
  assert(reviewButton, "离线角色编排区缺少总指导回收按钮");
  assert(reviewButton.props?.disabled !== true, "完成回传后总指导回收按钮不应禁用");
  const clickReview = reviewButton.props?.onClick;
  assert(typeof clickReview === "function", "离线总指导回收按钮没有 onClick");
  clickReview();
  const capturedReviewAction = reviewActions[0];
  assert(capturedReviewAction?.kind === "offline-director-review", "总指导回收按钮应生成离线回收 action");
  assert(
    capturedReviewAction.offlineDirectorReview?.dispatch_id === "offline-dispatch:fixture:completed",
    "总指导回收应绑定 completed 离线派发",
  );

  capturedAction = null;
  const rolePanel = (
    <OfflineRoleOrchestrationPanel
      project={project}
      projectWorkflow={workflowStateWithProjectWorkflow.project_workflows[0]}
      sessions={[session]}
      onRequestAction={captureAction}
    />
  );
  const rolePanelText = visibleText(rolePanel);
  for (const expectedText of [
    "Codex 角色编排",
    "总指导派发闭环",
    "总指导",
    "开发线",
    "验证线",
    "回收线",
    "总指导回复里的派发块",
    "写入离线派发",
    "写入角色回传",
    "写入总指导回收",
    "账本锚点",
    "已有任务草稿",
    "派发预览",
    "角色回传",
    "回传总指导",
    "不启动 Codex",
    "不写 /Users/yoyi/.codex",
    "离线编排账本",
    "预览来自默认示例",
  ]) {
    assert(rolePanelText.includes(expectedText), `离线角色编排区缺少 ${expectedText}`);
  }

  const previewOnlyPanelText = visibleText(
    <OfflineRoleOrchestrationPanel project={project} sessions={[session]} onRequestAction={captureAction} />,
  );
  assert(previewOnlyPanelText.includes("离线编排只能预览"), "没有 ready_to_dispatch 工作项时应只允许预览");

  const offlineForm = findElement(rolePanel, (element) => element.type === "form" && element.props?.className === "offline-role-orchestration-panel");
  assert(offlineForm, "离线角色编排区缺少表单");
  const submitOfflineDispatch = offlineForm.props?.onSubmit;
  assert(typeof submitOfflineDispatch === "function", "离线角色派发表单没有 onSubmit");
  const originalFormData = globalThis.FormData;
  globalThis.FormData = offlineRoleDispatchFormDataFixture(defaultOfflineDispatchBlock);
  try {
    submitOfflineDispatch({
      preventDefault() {},
      currentTarget: {},
    });
  } finally {
    globalThis.FormData = originalFormData;
  }
  assertDeepEqual(capturedAction, action, "离线角色派发表单提交 action 不匹配");

  capturedAction = null;
  globalThis.FormData = missingOfflineRoleDispatchFormDataFixture();
  try {
    submitOfflineDispatch({
      preventDefault() {},
      currentTarget: {},
    });
  } finally {
    globalThis.FormData = originalFormData;
  }
  assert(capturedAction === null, "缺字段派发块不应生成离线派发 action");
}

function runShellScenario() {
  const shellTexts = shellScenarioTextFixtures;
  const visited: string[] = [];
  const home = <HomeView snapshot={snapshot} onNavigate={(view) => visited.push(view)} />;
  const homeText = visibleText(home);
  for (const expectedText of shellTexts.homeExpectedTexts) {
    assert(homeText.includes(expectedText), `首页缺少 ${expectedText}`);
  }
  assertDeepEqual(
    primaryNavItems.map((item) => item.label),
    shellTexts.primaryNavLabels,
    "普通主导航应暴露产品级工作对象和素材/记忆入口",
  );
  assertDeepEqual(
    primaryNavItems.map((item) => [item.key, item.glyph]),
    shellTexts.primaryNavGlyphs,
    "左侧主导航应沿用 inkwash-full.html 的水墨 rail 图标语言",
  );
  for (const internalLabel of devNavItems.map((item) => item.label)) {
    assert(!primaryNavItems.some((item) => item.label === internalLabel), `普通主导航不应暴露开发者入口：${internalLabel}`);
  }
  for (const forbiddenText of shellTexts.homeForbiddenTexts) {
    assert(!homeText.includes(forbiddenText), `首页不应显示数量：${forbiddenText}`);
  }

  const settingsText = visibleText(
    <SettingsView
      snapshot={snapshot}
      workflowState={workflowStateWithProjectWorkflow}
      workflowStateError={null}
      hasRealSnapshot={true}
      developerItems={devNavItems}
      onNavigate={(view) => visited.push(view)}
    />,
  );
  for (const expectedText of shellTexts.settingsExpectedTexts) {
    assert(settingsText.includes(expectedText), `设置开发者区缺少 ${expectedText}`);
  }
  for (const forbiddenText of shellTexts.settingsForbiddenTexts) {
    assert(!settingsText.includes(forbiddenText), `设置开发者区不应出现执行或凭据读取文案：${forbiddenText}`);
  }

  const runningWorkflowsText = visibleText(
    <RunningWorkflowsView
      snapshot={snapshot}
      workflowState={workflowStateWithProjectWorkflow}
      workflowStateLoading={false}
      workflowStateError={null}
      onReloadWorkflowState={() => {}}
      onNavigate={(view) => visited.push(view)}
    />,
  );
  for (const expectedText of shellTexts.runningWorkflowsExpectedTexts) {
    assert(runningWorkflowsText.includes(expectedText), `运行中工作流页缺少 ${expectedText}`);
  }

  const agentButton = findButtonByText(home, "打开智能体");
  assert(agentButton, "首页找不到智能体入口按钮");
  const openAgent = agentButton.props?.onClick;
  assert(typeof openAgent === "function", "智能体入口没有 onClick");
  openAgent({ preventDefault() {}, stopPropagation() {} });
  assertDeepEqual(visited, ["agents"], "智能体入口导航不匹配");

  const agentText = visibleText(
    <AgentSessionCenter
      sessions={[session]}
      selectedThreadId={session.thread_id}
      selectedSession={session}
      transcript={null}
      loadingThreadId={null}
      transcriptError={null}
      projectSessionCount={1}
      onOpenSession={() => {}}
      onRequestAction={captureAction}
    />,
  );
  for (const expectedText of shellTexts.agentSessionExpectedTexts) {
    assert(agentText.includes(expectedText), `Agent 页缺少 ${expectedText}`);
  }
  assert(!agentText.includes("启动 OpenClaw"), "未接入 agent 不应出现操作能力");

  const agentViewNode = (
    <AgentView
      sessions={[session]}
      projects={[project]}
      adapterDescriptors={snapshot.agent_adapters}
      workflowState={workflowStateWithProjectWorkflow}
      onRequestAction={captureAction}
    />
  );
  const agentViewText = visibleText(agentViewNode);
  const agentViewMarkup = renderToStaticMarkup(agentViewNode);
  for (const expectedText of shellTexts.agentViewExpectedTexts) {
    assert(agentViewText.includes(expectedText), `AgentView 新方向缺少 ${expectedText}`);
  }
  assert(agentViewMarkup.includes("agent-conversation-bar"), "AgentView 新方向应有项目 / 对话选择条");
  assert(agentViewMarkup.includes("agent-chat-composer"), "AgentView 新方向应有任务输入框");
  assert(
    agentViewMarkup.indexOf("agent-session-shell") < agentViewMarkup.indexOf("agent-boundary-details"),
    "AgentView 新方向应先展示对话界面，再展示开发者详情",
  );
  const fallbackAgentViewText = visibleText(
    <AgentView
      sessions={[session]}
      projects={[project]}
      workflowState={workflowStateWithProjectWorkflow}
      onRequestAction={captureAction}
    />,
  );
  assert(fallbackAgentViewText.includes("adapter_descriptor_frontend_fallback_used"), "AgentView 没有后端 descriptor 时应保留前端 fallback");
  for (const forbiddenText of shellTexts.agentViewForbiddenTexts) {
    assert(!agentViewText.includes(forbiddenText), `AgentView 新方向不应出现 ${forbiddenText}`);
  }

  const projectText = visibleText(<ProjectDetail project={project} sessions={[session]} onRequestAction={captureAction} />);
  for (const expectedText of shellTexts.projectOverviewExpectedTexts) {
    assert(projectText.includes(expectedText), `项目工作流缺少 ${expectedText}`);
  }
  for (const forbiddenText of shellTexts.projectOverviewForbiddenTexts) {
    assert(!projectText.includes(forbiddenText), `项目工作台主导航不应出现 ${forbiddenText}`);
  }

  const projectAgentButton = findButtonByText(
    <ProjectDetail project={project} sessions={[session]} onRequestAction={captureAction} />,
    "在智能体中打开",
  );
  assert(projectAgentButton, "项目总览缺少智能体会话入口");
  const filteredProjectSessions = filterProjectSessionsForProject([session, otherProjectSession], project);
  assertDeepEqual(filteredProjectSessions, [session], "项目 Agent 会话应只保留 project_root 等于当前项目的会话");

  const projectAgentSessionText = visibleText(
    <AgentSessionCenter
      scope="project"
      eyebrow="项目 Agent"
      title="项目内 Agent 会话"
      description={`只显示 project_root 等于当前项目的 Codex 会话；项目归属来源为索引推断。当前项目：${project.name}`}
      emptyTitle="没有索引推断关联的 Codex 会话"
      emptyMessage="当前项目没有索引推断关联的 Codex 会话。"
      sessions={[session]}
      selectedThreadId={session.thread_id}
      selectedSession={session}
      transcript={null}
      loadingThreadId={null}
      transcriptError={null}
      projectSessionCount={1}
      onOpenSession={() => {}}
      onRequestAction={captureAction}
    />,
  );
  for (const expectedText of shellTexts.projectAgentSessionExpectedTexts) {
    assert(projectAgentSessionText.includes(expectedText), `项目 Agent 会话面板缺少 ${expectedText}`);
  }
  assert(!projectAgentSessionText.includes(otherProjectSession.title), "项目 Agent 会话面板不应显示其他项目会话");
  for (const forbiddenText of shellTexts.projectAgentSessionForbiddenTexts) {
    assert(!projectAgentSessionText.includes(forbiddenText), `项目 Agent 会话面板不应出现危险入口：${forbiddenText}`);
  }

  const emptyProjectAgentSessionText = visibleText(
    <AgentSessionCenter
      scope="project"
      eyebrow="项目 Agent"
      title="项目内 Agent 会话"
      description={`只显示 project_root 等于当前项目的 Codex 会话；项目归属来源为索引推断。当前项目：${emptyProject.name}`}
      emptyTitle="没有索引推断关联的 Codex 会话"
      emptyMessage="当前项目没有索引推断关联的 Codex 会话。"
      sessions={[]}
      selectedThreadId={null}
      selectedSession={null}
      transcript={null}
      loadingThreadId={null}
      transcriptError={null}
      projectSessionCount={0}
      onOpenSession={() => {}}
      onRequestAction={captureAction}
    />,
  );
  for (const expectedText of shellTexts.emptyProjectAgentSessionExpectedTexts) {
    assert(emptyProjectAgentSessionText.includes(expectedText), `空项目 Agent 会话面板缺少 ${expectedText}`);
  }

  capturedAction = null;
  const workflowProject = (
    <ProjectDetail
      project={project}
      sessions={[session]}
      workflowState={workflowState}
      selectedTool="task-packages"
      onRequestAction={captureAction}
    />
  );
  const workflowProjectText = visibleText(workflowProject);
  for (const expectedText of shellTexts.workflowProjectDraftExpectedTexts) {
    assert(workflowProjectText.includes(expectedText), `项目工作流草稿区缺少 ${expectedText}`);
  }
  const bootstrapButton = findButtonByText(workflowProject, "创建默认工作流草稿");
  assert(bootstrapButton, "项目页缺少创建默认工作流按钮");
  const bootstrap = bootstrapButton.props?.onClick;
  assert(typeof bootstrap === "function", "创建默认工作流按钮没有 onClick");
  bootstrap({ preventDefault() {}, stopPropagation() {} });
  assertDeepEqual(
    capturedAction,
    buildBootstrapProjectWorkflowAction(project.project_root),
    "创建默认工作流待确认动作不匹配",
  );
  const bootstrapDialog = (
    <PermissionDialog
      action={capturedAction}
      busy={false}
      onCancel={() => {
        capturedAction = null;
      }}
      onConfirm={() => {
        throw new Error("离线测试不应确认创建默认工作流");
      }}
    />
  );
  const bootstrapDialogText = visibleText(bootstrapDialog);
  for (const expectedText of [project.project_root, ...shellTexts.bootstrapDialogExpectedTexts]) {
    assert(bootstrapDialogText.includes(expectedText), `创建默认工作流确认弹层缺少 ${expectedText}`);
  }
  const cancelBootstrap = findButtonByText(bootstrapDialog, "取消");
  assert(cancelBootstrap, "创建默认工作流确认弹层缺少取消按钮");
  const cancelBootstrapClick = cancelBootstrap.props?.onClick;
  assert(typeof cancelBootstrapClick === "function", "创建默认工作流取消按钮没有 onClick");
  cancelBootstrapClick({ preventDefault() {}, stopPropagation() {} });
  assert(capturedAction === null, "取消创建默认工作流不应保留待确认动作");

  capturedAction = null;
  const workflowProjectWithDraft = (
    <ProjectDetail
      project={project}
      sessions={[session]}
      workflowState={workflowStateWithProjectWorkflow}
      selectedTool="task-packages"
      onRequestAction={captureAction}
    />
  );
  const workflowProjectWithDraftText = visibleText(workflowProjectWithDraft);
  for (const expectedText of shellTexts.workflowProjectWithDraftExpectedTexts) {
    assert(workflowProjectWithDraftText.includes(expectedText), `任务草稿区缺少 ${expectedText}`);
  }

  const workflowCanvasWithDraft = (
    <ProjectDetail
      project={project}
      sessions={[session]}
      workflowState={workflowStateWithProjectWorkflow}
      planAuthorizationStore={planAuthorizationStore}
      projectConsultationProposalStore={projectConsultationProposalStore}
      selectedTool="workflow"
      onRequestAction={captureAction}
    />
  );
  const workflowCanvasWithDraftText = visibleText(workflowCanvasWithDraft);
  for (const expectedText of shellTexts.workflowCanvasWithDraftExpectedTexts) {
    assert(workflowCanvasWithDraftText.includes(expectedText), `工作流画布缺少 ${expectedText}`);
  }
  for (const forbiddenText of shellTexts.workflowCanvasWithDraftForbiddenTexts) {
    assert(!workflowCanvasWithDraftText.includes(forbiddenText), `项目工作流页不应显示开发样例文案：${forbiddenText}`);
  }
  const proposalAction = buildProjectConsultationProposalDecisionAction({
    project,
    proposal: projectConsultationProposalStore.proposals[0],
    decision: "confirm",
    summary: projectConsultationProposalDecisionSummary,
    proposalStoreRevision: projectConsultationProposalStore.revision,
    planAuthorizationRevision: planAuthorizationStore.revision,
  });
  assert(proposalAction.kind === "record-project-consultation-proposal-decision", "确认方案范围应生成 proposal decision action");
  assert(
    proposalAction.projectConsultationProposalDecision?.decision === "confirm",
    "确认方案范围 action 应记录 confirm 决定",
  );
  assertDeepEqual(
    proposalAction.projectConsultationProposalDecision,
    projectConsultationProposalDecisionPayloadFixture(project.project_root),
    "确认方案范围 action payload 不匹配",
  );
  const proposalDialogText = visibleText(
    <PermissionDialog action={proposalAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of shellProposalDialogExpectedTexts(project.project_root)) {
    assert(proposalDialogText.includes(expectedText), `项目咨询方案确认弹层缺少 ${expectedText}`);
  }

  capturedAction = null;
  const workflowCanvasWithConfirmedProposal = (
    <ProjectDetail
      project={project}
      sessions={[session]}
      workflowState={workflowStateWithProjectWorkflow}
      planAuthorizationStore={planAuthorizationStorePendingGlobal}
      projectConsultationProposalStore={projectConsultationProposalStoreConfirmed}
      selectedTool="workflow"
      onRequestAction={captureAction}
    />
  );
  const workflowCanvasWithConfirmedProposalText = visibleText(workflowCanvasWithConfirmedProposal);
  for (const expectedText of shellTexts.confirmedProposalExpectedTexts) {
    assert(workflowCanvasWithConfirmedProposalText.includes(expectedText), `全局边界复核卡片缺少 ${expectedText}`);
  }
  for (const forbiddenText of shellTexts.confirmedProposalForbiddenTexts) {
    assert(!workflowCanvasWithConfirmedProposalText.includes(forbiddenText), `全局边界复核卡片不应显示 ${forbiddenText}`);
  }
  const globalReviewAction = buildGlobalBoundaryReviewAction({
    project,
    proposal: projectConsultationProposalStoreConfirmed.proposals[0],
    authorization: planAuthorizationStorePendingGlobal.authorizations[0],
    reviewStatus: "approved",
    summary: globalBoundaryReviewSummary,
    authorizationRevision: planAuthorizationStorePendingGlobal.revision,
  });
  assert(globalReviewAction.kind === "record-global-boundary-review", "批准并生效应生成全局边界复核 action");
  assertDeepEqual(
    globalReviewAction.globalBoundaryReview,
    globalBoundaryReviewPayloadFixture(project.project_root),
    "批准并生效 action payload 不匹配",
  );
  const globalReviewDialogText = visibleText(
    <PermissionDialog action={globalReviewAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of shellTexts.globalReviewDialogExpectedTexts) {
    assert(globalReviewDialogText.includes(expectedText), `全局边界复核确认弹层缺少 ${expectedText}`);
  }

  const projectDirectorTaskPlanRequest = projectDirectorTaskPlanRequestFixture(
    project.project_root,
    planAuthorizationStore.revision,
  );
  capturedAction = null;
  const projectDirectorTaskPlanCard = (
    <ProjectDirectorTaskPlanCard
      project={project}
      request={projectDirectorTaskPlanRequest}
      plan={projectDirectorTaskPlan}
      loading={false}
      error={null}
      workflowRevision={workflowStateWithProjectWorkflow.workflow_version ?? null}
      onPreview={() => {}}
      onRequestAction={captureAction}
    />
  );
  const projectDirectorTaskPlanCardText = visibleText(projectDirectorTaskPlanCard);
  for (const expectedText of shellTexts.projectDirectorTaskPlanCardExpectedTexts) {
    assert(projectDirectorTaskPlanCardText.includes(expectedText), `C4 项目主管卡片缺少 ${expectedText}`);
  }
  for (const forbiddenText of shellTexts.projectDirectorTaskPlanCardForbiddenTexts) {
    assert(!projectDirectorTaskPlanCardText.includes(forbiddenText), `C4 项目主管卡片不应出现 ${forbiddenText}`);
  }
  const prepareAuthorizedButton = findButtonByText(projectDirectorTaskPlanCard, "准备授权范围内派发");
  assert(prepareAuthorizedButton, "C4 项目主管卡片缺少准备派发按钮");
  const prepareAuthorizedClick = prepareAuthorizedButton.props?.onClick;
  assert(typeof prepareAuthorizedClick === "function", "C4 准备派发按钮没有 onClick");
  prepareAuthorizedClick({ preventDefault() {}, stopPropagation() {} });
  const expectedPrepareAuthorizedAction = buildPrepareAuthorizedAutoDispatchAction({
    project,
    request: projectDirectorTaskPlanRequest,
    plan: projectDirectorTaskPlan,
    workflowRevision: workflowStateWithProjectWorkflow.workflow_version ?? null,
  });
  assertDeepEqual(capturedAction, expectedPrepareAuthorizedAction, "C4 准备派发 action payload 不匹配");
  const prepareAuthorizedDialogText = visibleText(
    <PermissionDialog action={capturedAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of shellTexts.prepareAuthorizedDialogExpectedTexts) {
    assert(prepareAuthorizedDialogText.includes(expectedText), `C4 准备派发确认弹层缺少 ${expectedText}`);
  }
  for (const forbiddenText of shellTexts.prepareAuthorizedDialogForbiddenTexts) {
    assert(!prepareAuthorizedDialogText.includes(forbiddenText), `C4 准备派发确认弹层不应出现 ${forbiddenText}`);
  }

  const workflowProjectWithDerived = (
    <ProjectDetail
      project={project}
      sessions={[session]}
      workflowState={workflowStateWithDerivedWorkflow}
      selectedTool="workflow"
      onRequestAction={captureAction}
      onInspectWorkflowRunCheck={async () => blockedWorkflowRunCheck}
    />
  );
  const workflowProjectWithDerivedText = visibleText(workflowProjectWithDerived);
  for (const expectedText of shellDerivedWorkflowExpectedTexts(project.project_root)) {
    assert(workflowProjectWithDerivedText.includes(expectedText), `派生工作流展示缺少 ${expectedText}`);
  }
  const workflowProjectWithDerivedMarkup = renderToStaticMarkup(workflowProjectWithDerived);
  assert(
    !workflowProjectWithDerivedMarkup.includes('class="project-candidate-governance"'),
    "项目工作流主区域不应再把候选治理作为独立 strip",
  );
  assert(
    workflowProjectWithDerivedMarkup.includes("project-candidate-governance-card"),
    "候选治理仍应保留为项目画布侧栏详情卡",
  );
  for (const forbiddenText of [
    ...shellTexts.derivedWorkflowForbiddenTexts,
    ...canvasBoundaryForbiddenPhrases,
  ]) {
    assert(!workflowProjectWithDerivedText.includes(forbiddenText), `F3/F4 项目画布不应出现误导文案 ${forbiddenText}`);
  }

  const blockedRunCheckText = visibleText(<WorkflowRunCheckDetails runCheck={blockedWorkflowRunCheck} />);
  for (const expectedText of shellTexts.blockedRunCheckExpectedTexts) {
    assert(blockedRunCheckText.includes(expectedText), `blocked 运行前检查展示缺少 ${expectedText}`);
  }

  const runnableRunCheckText = visibleText(<WorkflowRunCheckDetails runCheck={runnableWorkflowRunCheck} />);
  for (const expectedText of shellTexts.runnableRunCheckExpectedTexts) {
    assert(runnableRunCheckText.includes(expectedText), `runnable 运行前检查展示缺少 ${expectedText}`);
  }
  assert(!runnableRunCheckText.includes("自动选择模型"), "runnable 检查不应出现自动补模型文案");

  const projectsViewText = visibleText(
    <ProjectsView
      projects={[project]}
      sessions={[session]}
      workflowState={workflowStateWithProjectWorkflow}
      onRequestAction={captureAction}
      onLoadTranscript={async () => {
        throw new Error("主路径静态渲染不应读取 transcript");
      }}
    />,
  );
  for (const expectedText of shellTexts.projectsViewExpectedTexts) {
    assert(projectsViewText.includes(expectedText), `ProjectsView 项目入口缺少 ${expectedText}`);
  }
  assert(!projectsViewText.includes("节点会话绑定"), "ProjectsView 默认入口不应直接进入项目工作台");
  assert(!projectsViewText.includes("任务包"), "ProjectsView 默认入口不应把任务包作为主模块展示");
  assert(!projectsViewText.includes("项目工作流草稿"), "ProjectsView 默认工作流页不应混入任务包页内容");

  const selectedAfterMissing = nextSelectedWorkItemId(workflowStateWithProjectWorkflow.project_workflows[0].task_drafts, "work-item:missing");
  assert(selectedAfterMissing === "work-item:offline:001", "缺失选择态应回到第一个草稿");
  const selectedAfterSwitch = nextSelectedWorkItemId(workflowStateWithProjectWorkflow.project_workflows[0].task_drafts, "work-item:offline:002");
  assert(selectedAfterSwitch === "work-item:offline:002", "切换选择态后应保留第二个草稿");
  const selectedSecondDraft = selectedTaskDraftFor(workflowStateWithProjectWorkflow.project_workflows[0].task_drafts, selectedAfterSwitch);
  assert(selectedSecondDraft?.work_item_id === "work-item:offline:002", "选择态解析应返回第二个草稿");

  const workflowControlCardWithDraft = (
    <WorkItemOrchestrationCard
      project={project}
      projectId={workflowStateWithDerivedWorkflow.project_workflows[0].project_id}
      sessions={[session]}
      bindings={workflowStateWithDerivedWorkflow.project_workflows[0].node_session_bindings}
      dispatches={workflowStateWithDerivedWorkflow.project_workflows[0].node_dispatches}
      directorReviews={workflowStateWithDerivedWorkflow.project_workflows[0].director_reviews}
      executionControls={workflowStateWithDerivedWorkflow.project_workflows[0].execution_controls}
      permissionRequests={workflowStateWithDerivedWorkflow.project_workflows[0].permission_requests}
      executionAttempts={workflowStateWithDerivedWorkflow.project_workflows[0].execution_attempts}
      derivedWorkflow={workflowStateWithDerivedWorkflow.project_workflows[0].derived_workflow ?? null}
      projectConsultationProposalSummary={projectConsultationProposalSummary}
      planAuthorizationSummary={planAuthorizationSummary}
      workflowRevision={workflowStateWithDerivedWorkflow.workflow_version ?? null}
      observationStoreRevision={0}
      workItem={workflowStateWithDerivedWorkflow.project_workflows[0].task_drafts[0]}
      onRequestAction={captureAction}
      onOpenAgentSession={() => {}}
    />
  );

  capturedAction = null;
  const instructionBoundaryButton = findButtonByText(workflowControlCardWithDraft, "确认指令边界");
  assert(instructionBoundaryButton, "可控执行协议区缺少确认指令边界按钮");
  const previewInstruction = instructionBoundaryButton.props?.onClick;
  assert(typeof previewInstruction === "function", "确认指令边界按钮没有 onClick");
  previewInstruction({ preventDefault() {}, stopPropagation() {} });
  const userReviewedInstruction = workflowStateWithProjectWorkflow.project_workflows[0].execution_controls[0].user_reviewed_instruction;
  assert(userReviewedInstruction, "用户审核业务指令 fixture 缺失");
  assertDeepEqual(
    capturedAction,
    buildUserReviewedInstructionPreviewAction(project.project_root, userReviewedInstruction),
    "用户审核业务指令边界动作不匹配",
  );
  const instructionDialogText = visibleText(
    <PermissionDialog action={capturedAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of shellTexts.instructionDialogExpectedTexts) {
    assert(instructionDialogText.includes(expectedText), `用户审核业务指令确认弹层缺少 ${expectedText}`);
  }

  const c5PanelText = visibleText(workflowControlCardWithDraft);
  for (const expectedText of shellTexts.c5PanelExpectedTexts) {
    assert(c5PanelText.includes(expectedText), `C5 过程事实面板缺少 ${expectedText}`);
  }
  for (const forbiddenText of shellTexts.c5PanelForbiddenTexts) {
    assert(!c5PanelText.includes(forbiddenText), `C5 面板不应显示 ${forbiddenText}`);
  }

  const workflowC6ControlCard = (
    <WorkItemOrchestrationCard
      project={project}
      projectId={workflowStateWithC6ResultSummary.project_workflows[0].project_id}
      sessions={[session]}
      bindings={workflowStateWithC6ResultSummary.project_workflows[0].node_session_bindings}
      dispatches={workflowStateWithC6ResultSummary.project_workflows[0].node_dispatches}
      directorReviews={workflowStateWithC6ResultSummary.project_workflows[0].director_reviews}
      executionControls={workflowStateWithC6ResultSummary.project_workflows[0].execution_controls}
      permissionRequests={workflowStateWithC6ResultSummary.project_workflows[0].permission_requests}
      executionAttempts={workflowStateWithC6ResultSummary.project_workflows[0].execution_attempts}
      derivedWorkflow={workflowStateWithC6ResultSummary.project_workflows[0].derived_workflow ?? null}
      projectConsultationProposalSummary={projectConsultationProposalSummary}
      planAuthorizationSummary={planAuthorizationSummary}
      workflowRevision={workflowStateWithC6ResultSummary.workflow_version ?? null}
      observationStoreRevision={0}
      workItem={workflowStateWithC6ResultSummary.project_workflows[0].task_drafts[0]}
      onRequestAction={captureAction}
      onOpenAgentSession={() => {}}
    />
  );
  const c6PanelText = visibleText(workflowC6ControlCard);
  for (const expectedText of shellTexts.c6PanelExpectedTexts) {
    assert(c6PanelText.includes(expectedText), `C6 结果摘要面板缺少 ${expectedText}`);
  }
  for (const forbiddenText of shellTexts.c6PanelForbiddenTexts) {
    assert(!c6PanelText.includes(forbiddenText), `C6 面板不应显示越界文案：${forbiddenText}`);
  }

  capturedAction = null;
  const globalFinalReviewButton = findButtonByText(workflowC6ControlCard, "记录最终复核通过");
  assert(globalFinalReviewButton, "C6 面板缺少全局最终复核按钮");
  const recordGlobalFinalReview = globalFinalReviewButton.props?.onClick;
  assert(typeof recordGlobalFinalReview === "function", "C6 全局最终复核按钮没有 onClick");
  recordGlobalFinalReview({ preventDefault() {}, stopPropagation() {} });
  const globalFinalReviewAction = capturedAction as unknown as PendingAction;
  assert(globalFinalReviewAction.kind === "record-global-final-result-review", "C6 全局最终复核 action kind 不匹配");
  assert(globalFinalReviewAction.globalFinalResultReview?.actor_role === "global_director", "C6 全局最终复核 actor_role 不匹配");
  assert(globalFinalReviewAction.globalFinalResultReview?.decision === "accepted", "C6 全局最终复核 decision 不匹配");
  assert(globalFinalReviewAction.globalFinalResultReview?.proposal_id === "proposal:offline:001", "C6 全局最终复核 proposal 不匹配");
  assert(globalFinalReviewAction.globalFinalResultReview?.authorization_id === "plan-auth:offline:active", "C6 全局最终复核 authorization 不匹配");
  assert(globalFinalReviewAction.globalFinalResultReview?.accepted_process_fact_ids.includes("process-fact:offline:001"), "C6 全局最终复核缺少 process fact");
  assert(globalFinalReviewAction.boundary?.includes("不代表用户已接受"), "C6 全局最终复核 action 边界缺少用户接受限制");
  const globalFinalReviewDialogText = visibleText(
    <PermissionDialog action={globalFinalReviewAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of shellTexts.globalFinalReviewDialogExpectedTexts) {
    assert(globalFinalReviewDialogText.includes(expectedText), `C6 全局最终复核确认弹层缺少 ${expectedText}`);
  }

  capturedAction = null;
  const userDecisionButton = findButtonByText(workflowC6ControlCard, "记录用户接受");
  assert(userDecisionButton, "C6 面板缺少用户结果决定按钮");
  const recordUserDecision = userDecisionButton.props?.onClick;
  assert(typeof recordUserDecision === "function", "C6 用户结果决定按钮没有 onClick");
  recordUserDecision({ preventDefault() {}, stopPropagation() {} });
  const userDecisionAction = capturedAction as unknown as PendingAction;
  assert(userDecisionAction.kind === "record-user-result-decision", "C6 用户结果决定 action kind 不匹配");
  assert(userDecisionAction.userResultDecision?.actor_role === "user", "C6 用户结果决定 actor_role 不匹配");
  assert(userDecisionAction.userResultDecision?.decision === "accept_result", "C6 用户结果决定 decision 不匹配");
  assert(userDecisionAction.userResultDecision?.accepted_review_id === "global-final-review:offline:001", "C6 用户结果决定 review id 不匹配");
  assert(userDecisionAction.boundary?.includes("不代表未来任务默认接受"), "C6 用户结果决定 action 边界缺少未来任务限制");
  const userDecisionDialogText = visibleText(
    <PermissionDialog action={userDecisionAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of shellTexts.userDecisionDialogExpectedTexts) {
    assert(userDecisionDialogText.includes(expectedText), `C6 用户结果决定确认弹层缺少 ${expectedText}`);
  }

  capturedAction = null;
  const stageSummaryButton = findButtonByText(workflowC6ControlCard, "生成验收摘要");
  assert(stageSummaryButton, "C6 面板缺少阶段 C 验收摘要按钮");
  const generateStageSummary = stageSummaryButton.props?.onClick;
  assert(typeof generateStageSummary === "function", "C6 阶段 C 验收摘要按钮没有 onClick");
  generateStageSummary({ preventDefault() {}, stopPropagation() {} });
  const stageSummaryAction = capturedAction as unknown as PendingAction;
  assert(stageSummaryAction.kind === "generate-stage-c-acceptance-summary", "C6 阶段 C 验收摘要 action kind 不匹配");
  assert(stageSummaryAction.stageCAcceptanceSummary?.project_id === "project:offline-fixture-projects-codex-workbench", "C6 阶段 C 验收摘要 project_id 不匹配");
  assert(stageSummaryAction.stageCAcceptanceSummary?.workflow_id === "workflow:offline-fixture-projects-codex-workbench:default", "C6 阶段 C 验收摘要 workflow_id 不匹配");
  assert(stageSummaryAction.boundary?.includes("不执行真实 Codex"), "C6 阶段 C 验收摘要 action 边界缺少真实 Codex 限制");
  const stageSummaryDialogText = visibleText(
    <PermissionDialog action={stageSummaryAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of shellTexts.stageSummaryDialogExpectedTexts) {
    assert(stageSummaryDialogText.includes(expectedText), `C6 阶段 C 验收摘要确认弹层缺少 ${expectedText}`);
  }
  for (const forbiddenText of shellTexts.c6PanelForbiddenTexts) {
    assert(!globalFinalReviewDialogText.includes(forbiddenText), `C6 全局最终复核弹层不应显示越界文案：${forbiddenText}`);
    assert(!userDecisionDialogText.includes(forbiddenText), `C6 用户结果决定弹层不应显示越界文案：${forbiddenText}`);
    assert(!stageSummaryDialogText.includes(forbiddenText), `C6 阶段 C 验收摘要弹层不应显示越界文案：${forbiddenText}`);
  }

  capturedAction = null;
  const recordWorkerReportButton = findButtonByText(workflowControlCardWithDraft, "记录汇报");
  assert(recordWorkerReportButton, "C5 面板缺少记录汇报按钮");
  const recordWorkerReport = recordWorkerReportButton.props?.onClick;
  assert(typeof recordWorkerReport === "function", "C5 记录汇报按钮没有 onClick");
  recordWorkerReport({ preventDefault() {}, stopPropagation() {} });
  const workerReportAction = capturedAction as unknown as PendingAction;
  assert(workerReportAction.kind === "record-worker-structured-report", "C5 记录汇报 action kind 不匹配");
  assert(workerReportAction.workerStructuredReport?.project_id === "project:offline-fixture-projects-codex-workbench", "C5 汇报 project_id 不匹配");
  assert(workerReportAction.workerStructuredReport?.workflow_id === "workflow:offline-fixture-projects-codex-workbench:default", "C5 汇报 workflow_id 不匹配");
  assert(workerReportAction.workerStructuredReport?.dispatch_id === "dispatch:offline:001", "C5 汇报 dispatch_id 不匹配");
  assert(workerReportAction.workerStructuredReport?.evidence_refs[0] === "/tmp/codex-workflow-node-dispatch-v1/offline-last-message.txt", "C5 汇报 evidence_refs 不匹配");
  assert(workerReportAction.workerStructuredReport?.source_refs[0].source_kind === "workflow_event", "C5 汇报 source kind 不匹配");
  assert(workerReportAction.boundary?.includes("不把汇报写成正式事实或正式记忆"), "C5 汇报 action 边界缺少正式事实 / 正式记忆限制");
  const workerReportDialogText = visibleText(
    <PermissionDialog action={workerReportAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of shellTexts.workerReportDialogExpectedTexts) {
    assert(workerReportDialogText.includes(expectedText), `C5 工作者汇报确认弹层缺少 ${expectedText}`);
  }

  capturedAction = null;
  const confirmProcessFactButton = findButtonByText(workflowControlCardWithDraft, "确认为过程事实");
  assert(confirmProcessFactButton, "C5 面板缺少确认为过程事实按钮");
  const confirmProcessFact = confirmProcessFactButton.props?.onClick;
  assert(typeof confirmProcessFact === "function", "C5 确认为过程事实按钮没有 onClick");
  confirmProcessFact({ preventDefault() {}, stopPropagation() {} });
  const processFactAction = capturedAction as unknown as PendingAction;
  assert(processFactAction.kind === "record-project-director-process-fact-decision", "C5 过程事实 action kind 不匹配");
  assert(processFactAction.processFactDecision?.actor_role === "project_director", "C5 过程事实确认必须由项目主管发起");
  assert(processFactAction.processFactDecision?.decision === "confirm_process_fact", "C5 过程事实 decision 不匹配");
  assert(processFactAction.processFactDecision?.accepted_facts[0].proposed_observation_type === "process_fact", "C5 过程事实 observation type 不匹配");
  assert(processFactAction.processFactDecision?.accepted_facts[0].scope.project_id === "project:offline-fixture-projects-codex-workbench", "C5 过程事实 scope project_id 不匹配");
  assert(processFactAction.processFactDecision?.expected_observation_store_revision === 0, "C5 过程事实 observation revision 不匹配");
  assert(processFactAction.boundary?.includes("不写正式记忆"), "C5 过程事实 action 边界缺少正式记忆限制");
  const processFactDialogText = visibleText(
    <PermissionDialog action={processFactAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of shellTexts.processFactDialogExpectedTexts) {
    assert(processFactDialogText.includes(expectedText), `C5 过程事实确认弹层缺少 ${expectedText}`);
  }

  capturedAction = null;
  const approvePermissionButton = findButtonByText(workflowControlCardWithDraft, "批准");
  assert(approvePermissionButton, "权限队列缺少批准按钮");
  const approvePermission = approvePermissionButton.props?.onClick;
  assert(typeof approvePermission === "function", "权限批准按钮没有 onClick");
  approvePermission({ preventDefault() {}, stopPropagation() {} });
  assertDeepEqual(
    capturedAction,
    buildPermissionDecisionAction(project.project_root),
    "权限结论动作不匹配",
  );
  const permissionDialogText = visibleText(
    <PermissionDialog action={capturedAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of shellTexts.permissionDialogExpectedTexts) {
    assert(permissionDialogText.includes(expectedText), `权限结论确认弹层缺少 ${expectedText}`);
  }
  capturedAction = null;

  capturedAction = null;
  const workflowReviewProject = (
    <ProjectDetail
      project={project}
      sessions={[session]}
      workflowState={workflowStateReadyForReview}
      selectedTool="workflow"
      onRequestAction={captureAction}
    />
  );
  const workflowReviewProjectText = visibleText(workflowReviewProject);
  for (const expectedText of shellTexts.workflowReviewProjectExpectedTexts) {
    assert(workflowReviewProjectText.includes(expectedText), `总指导回收区缺少 ${expectedText}`);
  }
  const workflowReviewControlCard = (
    <WorkItemOrchestrationCard
      project={project}
      projectId={workflowStateReadyForReview.project_workflows[0].project_id}
      sessions={[session]}
      bindings={workflowStateReadyForReview.project_workflows[0].node_session_bindings}
      dispatches={workflowStateReadyForReview.project_workflows[0].node_dispatches}
      directorReviews={workflowStateReadyForReview.project_workflows[0].director_reviews}
      executionControls={workflowStateReadyForReview.project_workflows[0].execution_controls}
      permissionRequests={workflowStateReadyForReview.project_workflows[0].permission_requests}
      executionAttempts={workflowStateReadyForReview.project_workflows[0].execution_attempts}
      derivedWorkflow={workflowStateReadyForReview.project_workflows[0].derived_workflow ?? null}
      projectConsultationProposalSummary={projectConsultationProposalSummary}
      planAuthorizationSummary={planAuthorizationSummary}
      workflowRevision={workflowStateReadyForReview.workflow_version ?? null}
      observationStoreRevision={0}
      workItem={workflowStateReadyForReview.project_workflows[0].task_drafts[0]}
      onRequestAction={captureAction}
      onOpenAgentSession={() => {}}
    />
  );
  const directorAcceptButton = findButtonByText(workflowReviewControlCard, "接受");
  assert(directorAcceptButton, "总指导回收区缺少接受按钮");
  const requestDirectorAccept = directorAcceptButton.props?.onClick;
  assert(typeof requestDirectorAccept === "function", "总指导回收接受按钮没有 onClick");
  requestDirectorAccept({ preventDefault() {}, stopPropagation() {} });
  assertDeepEqual(
    capturedAction,
    directorReviewActionFixture(project.project_root),
    "总指导回收待确认动作不匹配",
  );
  const directorDialogText = visibleText(
    <PermissionDialog action={capturedAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of shellTexts.directorDialogExpectedTexts) {
    assert(directorDialogText.includes(expectedText), `总指导回收确认弹层缺少 ${expectedText}`);
  }
  capturedAction = null;

  const bindCandidate = findButtonContainingText(workflowControlCardWithDraft, ["Offline interaction fixture", "项目归属来源：索引推断"]);
  assert(bindCandidate, "工作流编排区缺少候选会话绑定按钮");
  const bindSession = bindCandidate.props?.onClick;
  assert(typeof bindSession === "function", "候选会话绑定按钮没有 onClick");
  bindSession({ preventDefault() {}, stopPropagation() {} });
  assertDeepEqual(
    capturedAction,
    buildBindNodeSessionAction(project.project_root),
    "绑定节点会话待确认动作不匹配",
  );
  const bindDialogText = visibleText(
    <PermissionDialog action={capturedAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of shellTexts.bindDialogExpectedTexts) {
    assert(bindDialogText.includes(expectedText), `绑定节点会话确认弹层缺少 ${expectedText}`);
  }
  capturedAction = null;
  const unbindButton = findButtonByText(workflowControlCardWithDraft, "解除绑定");
  assert(unbindButton, "工作流编排区缺少解除绑定按钮");
  const unbind = unbindButton.props?.onClick;
  assert(typeof unbind === "function", "解除绑定按钮没有 onClick");
  unbind({ preventDefault() {}, stopPropagation() {} });
  assertDeepEqual(
    capturedAction,
    buildUnbindNodeSessionAction(project.project_root),
    "解除节点会话绑定待确认动作不匹配",
  );
  const unbindDialogText = visibleText(
    <PermissionDialog action={capturedAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of shellTexts.unbindDialogExpectedTexts) {
    assert(unbindDialogText.includes(expectedText), `解除节点会话确认弹层缺少 ${expectedText}`);
  }
  capturedAction = null;
  const dispatchButton = findButtonByText(workflowControlCardWithDraft, "旧安全派发已封存");
  assert(dispatchButton, "工作流编排区缺少旧安全派发封存按钮");
  assert(dispatchButton.props?.disabled, "旧安全派发按钮应保持禁用");
  assert(!dispatchButton.props?.onClick, "旧安全派发按钮不应再触发 pending action");
  const businessDispatchButton = findButtonByText(workflowControlCardWithDraft, "旧业务派发已封存");
  assert(businessDispatchButton, "工作流编排区缺少旧业务派发封存按钮");
  assert(businessDispatchButton.props?.disabled, "旧业务派发按钮应保持禁用");
  assert(!businessDispatchButton.props?.onClick, "旧业务派发按钮不应再触发 pending action");
  assert(capturedAction === null, "封存的旧派发按钮不应生成待确认动作");
  capturedAction = null;
  const advanceButton = findButtonByText(workflowControlCardWithDraft, "标记执行中");
  assert(advanceButton, "工作流编排区缺少推进到执行中按钮");
  const advance = advanceButton.props?.onClick;
  assert(typeof advance === "function", "推进状态按钮没有 onClick");
  advance({ preventDefault() {}, stopPropagation() {} });
  assertDeepEqual(
    capturedAction,
    buildAdvanceWorkItemStateAction(project.project_root),
    "推进工作项状态待确认动作不匹配",
  );
  const advanceDialogText = visibleText(
    <PermissionDialog action={capturedAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of shellTexts.advanceDialogExpectedTexts) {
    assert(advanceDialogText.includes(expectedText), `推进状态确认弹层缺少 ${expectedText}`);
  }
  capturedAction = null;
  const taskDraftForm = findElement(workflowProjectWithDraft, (element) => element.type === "form" && element.props?.className === "task-draft-form");
  assert(taskDraftForm, "任务草稿区缺少创建表单");
  const createTask = taskDraftForm.props?.onSubmit;
  assert(typeof createTask === "function", "创建任务包草稿表单没有 onSubmit");
  const formValues = taskDraftFormValues();
  const originalFormData = globalThis.FormData;
  globalThis.FormData = taskDraftFormDataFixture(formValues);
  try {
    createTask({
      preventDefault() {},
      currentTarget: {},
    });
  } finally {
    globalThis.FormData = originalFormData;
  }
  assertDeepEqual(
    capturedAction,
    buildCreateTaskDraftAction(project.project_root),
    "创建任务包草稿待确认动作不匹配",
  );
  let taskCreateConfirmed = false;
  const taskDraftDialog = (
    <PermissionDialog
      action={capturedAction}
      busy={false}
      onCancel={() => {
        capturedAction = null;
      }}
      onConfirm={() => {
        taskCreateConfirmed = true;
      }}
    />
  );
  const taskDraftDialogText = visibleText(taskDraftDialog);
  for (const expectedText of shellTexts.taskDraftDialogExpectedTexts) {
    assert(taskDraftDialogText.includes(expectedText), `创建任务包草稿确认弹层缺少 ${expectedText}`);
  }
  const cancelTaskDraft = findButtonByText(taskDraftDialog, "取消");
  assert(cancelTaskDraft, "创建任务包草稿确认弹层缺少取消按钮");
  const cancelTaskDraftClick = cancelTaskDraft.props?.onClick;
  assert(typeof cancelTaskDraftClick === "function", "创建任务包草稿取消按钮没有 onClick");
  cancelTaskDraftClick({ preventDefault() {}, stopPropagation() {} });
  assert(capturedAction === null, "取消创建任务包草稿不应保留待确认动作");
  assert(!taskCreateConfirmed, "取消确认不应调用创建动作");

  capturedAction = buildCopyTaskPreviewAction(project.project_root, selectedSecondDraft.work_item_id);
  let copyPreviewConfirmed = false;
  const copyPreviewDialog = (
    <PermissionDialog
      action={capturedAction}
      busy={false}
      onCancel={() => {
        capturedAction = null;
      }}
      onConfirm={() => {
        copyPreviewConfirmed = true;
      }}
    />
  );
  const copyPreviewDialogText = visibleText(copyPreviewDialog);
  for (const expectedText of shellTexts.copyPreviewDialogExpectedTexts) {
    assert(copyPreviewDialogText.includes(expectedText), `复制预览确认弹层缺少 ${expectedText}`);
  }
  const cancelCopyPreview = findButtonByText(copyPreviewDialog, "取消");
  assert(cancelCopyPreview, "复制预览确认弹层缺少取消按钮");
  const cancelCopyPreviewClick = cancelCopyPreview.props?.onClick;
  assert(typeof cancelCopyPreviewClick === "function", "复制预览取消按钮没有 onClick");
  cancelCopyPreviewClick({ preventDefault() {}, stopPropagation() {} });
  assert(capturedAction === null, "取消复制预览不应保留待确认动作");
  assert(!copyPreviewConfirmed, "取消复制预览不应执行复制");

  capturedAction = null;
  const taskFileGenerationPanel = (
    <TaskFileGenerationController
      projectRoot={project.project_root}
      selectedTaskDraft={workflowStateWithProjectWorkflow.project_workflows[0].task_drafts[0]}
      onRequestAction={captureAction}
    />
  );
  const taskFileGenerationText = visibleText(taskFileGenerationPanel);
  for (const expectedText of shellTexts.taskFileGenerationExpectedTexts) {
    assert(taskFileGenerationText.includes(expectedText), `任务文件生成区缺少 ${expectedText}`);
  }
  const generateButton = findButtonByText(taskFileGenerationPanel, "生成任务包文件");
  assert(generateButton, "任务草稿区缺少生成任务包文件按钮");
  const generateTaskFile = generateButton.props?.onClick;
  assert(typeof generateTaskFile === "function", "生成任务包文件按钮没有 onClick");
  generateTaskFile({ preventDefault() {}, stopPropagation() {} });
  assertDeepEqual(
    capturedAction,
    buildGenerateTaskFileAction(project.project_root, "work-item:offline:001"),
    "生成任务包文件待确认动作不匹配",
  );
  let generateConfirmed = false;
  const generateDialog = (
    <PermissionDialog
      action={capturedAction}
      busy={false}
      onCancel={() => {
        capturedAction = null;
      }}
      onConfirm={() => {
        generateConfirmed = true;
      }}
    />
  );
  const generateDialogText = visibleText(generateDialog);
  for (const expectedText of shellTexts.generateDialogExpectedTexts) {
    assert(generateDialogText.includes(expectedText), `生成任务包文件确认弹层缺少 ${expectedText}`);
  }
  const cancelGenerate = findButtonByText(generateDialog, "取消");
  assert(cancelGenerate, "生成任务包文件确认弹层缺少取消按钮");
  const cancelGenerateClick = cancelGenerate.props?.onClick;
  assert(typeof cancelGenerateClick === "function", "生成任务包文件取消按钮没有 onClick");
  cancelGenerateClick({ preventDefault() {}, stopPropagation() {} });
  assert(capturedAction === null, "取消生成任务包文件不应保留待确认动作");
  assert(!generateConfirmed, "取消生成任务包文件不应调用生成动作");

  const generatedTaskFilePanel = (
    <TaskFileGenerationController
      projectRoot={project.project_root}
      selectedTaskDraft={workflowStateWithGeneratedTaskFile.project_workflows[0].task_drafts[0]}
      onRequestAction={captureAction}
    />
  );
  const generatedTaskFileText = visibleText(generatedTaskFilePanel);
  for (const expectedText of shellTexts.generatedTaskFileExpectedTexts) {
    assert(generatedTaskFileText.includes(expectedText), `已有 path 时 UI 缺少 ${expectedText}`);
  }
  const generatedButton = findButtonByText(generatedTaskFilePanel, "已生成");
  assert(generatedButton, "已有 path 时缺少已生成按钮");
  assert(generatedButton.props?.disabled === true, "已有 path 时生成按钮应禁用");

  const dispatchReadinessPanel = (
    <TaskDispatchReadinessController
      projectRoot={project.project_root}
      selectedTaskDraft={workflowStateWithGeneratedTaskFile.project_workflows[0].task_drafts[0]}
      onRequestAction={captureAction}
      onInspectDispatchReadiness={async () => notReadyDispatchReadiness}
    />
  );
  const dispatchReadinessElement = dispatchReadinessPanel as ReactElementLike;
  assert(dispatchReadinessElement.props?.selectedTaskDraft === workflowStateWithGeneratedTaskFile.project_workflows[0].task_drafts[0], "派发准备区应绑定选中草稿");
  assert(typeof dispatchReadinessElement.props?.onInspectDispatchReadiness === "function", "派发准备区缺少检查入口");
  const notReadyShell = (
    <TaskDispatchReadinessShell
      readiness={notReadyDispatchReadiness}
      loading={false}
      error={null}
      onInspect={() => {}}
      onGenerateReadyFile={() => {
        throw new Error("not_ready should keep generation disabled");
      }}
    />
  );
  const notReadyShellText = visibleText(notReadyShell);
  for (const expectedText of shellTexts.notReadyShellExpectedTexts) {
    assert(notReadyShellText.includes(expectedText), `派发准备展示缺少 ${expectedText}`);
  }
  const readyFileButton = findButtonByText(notReadyShell, "生成可派发版本");
  assert(readyFileButton, "派发准备区缺少生成可派发版本按钮");
  assert(readyFileButton.props?.disabled === true, "not_ready 时生成可派发版本按钮应禁用");

  const renderedNotReady = visibleText(<TaskDispatchReadinessDetails readiness={notReadyDispatchReadiness} />);
  for (const expectedText of shellTexts.renderedNotReadyExpectedTexts) {
    assert(renderedNotReady.includes(expectedText), `not_ready 原因展示缺少 ${expectedText}`);
  }

  const { correctionFields, missingPreviewFields, fieldValues } = taskFieldCorrectionFixtures(project.project_root);
  const correctionPreviewText = visibleText(<TaskFieldCorrectionPreview fields={correctionFields} />);
  for (const expectedText of shellTexts.correctionPreviewExpectedTexts) {
    assert(correctionPreviewText.includes(expectedText), `字段修正预览缺少 ${expectedText}`);
  }
  assertDeepEqual(missingCorrectionFields(correctionFields), [], "完整字段不应有缺失提示");
  const missingPreviewText = visibleText(<TaskFieldCorrectionPreview fields={missingPreviewFields} />);
  for (const expectedText of shellTexts.missingPreviewExpectedTexts) {
    assert(missingPreviewText.includes(expectedText), `缺字段预览缺少 ${expectedText}`);
  }

  const correctionEditor = (
    <TaskDispatchFieldCorrectionShell
      projectRoot={project.project_root}
      selectedTaskDraft={workflowStateWithGeneratedTaskFile.project_workflows[0].task_drafts[0]}
      previewFields={correctionFields}
      onPreviewFieldsChange={() => {}}
      onRequestAction={captureAction}
    />
  );
  const correctionEditorText = visibleText(correctionEditor);
  for (const expectedText of shellTexts.correctionEditorExpectedTexts) {
    assert(correctionEditorText.includes(expectedText), `字段修正入口缺少 ${expectedText}`);
  }

  capturedAction = buildCorrectDispatchFieldsAction(
    project.project_root,
    workflowStateWithGeneratedTaskFile.project_workflows[0].task_drafts[0].work_item_id,
    correctionFields,
  );
  assertDeepEqual(
    capturedAction,
    {
      kind: "correct-dispatch-fields",
      label: "保存派发字段修正",
      path: project.project_root,
      source: "索引内项目路径",
      boundary:
        "只写工作台自己的 workflow-state.v0.json；不生成真实任务包文件、不派发真实 Codex 会话、不启动 Codex 命令行、不运行运行器、不写 .codex 或 Codex 状态库。",
      dispatchFields: {
        project_root: project.project_root,
        work_item_id: "work-item:offline:001",
        fields: correctionFields,
      },
    },
    "派发字段修正待确认动作不匹配",
  );
  let correctionConfirmed = false;
  const correctionDialog = (
    <PermissionDialog
      action={capturedAction}
      busy={false}
      onCancel={() => {
        capturedAction = null;
      }}
      onConfirm={() => {
        correctionConfirmed = true;
      }}
    />
  );
  const correctionDialogText = visibleText(correctionDialog);
  for (const expectedText of shellTexts.correctionDialogExpectedTexts) {
    assert(correctionDialogText.includes(expectedText), `派发字段修正确认弹层缺少 ${expectedText}`);
  }
  const cancelCorrection = findButtonByText(correctionDialog, "取消");
  assert(cancelCorrection, "派发字段修正确认弹层缺少取消按钮");
  const cancelCorrectionClick = cancelCorrection.props?.onClick;
  assert(typeof cancelCorrectionClick === "function", "派发字段修正取消按钮没有 onClick");
  cancelCorrectionClick({ preventDefault() {}, stopPropagation() {} });
  assert(capturedAction === null, "取消派发字段修正不应保留待确认动作");
  assert(!correctionConfirmed, "取消派发字段修正不应执行保存");

  capturedAction = null;
  capturedAction = buildUpdateTaskFieldsAction(project.project_root, selectedSecondDraft.work_item_id, fieldValues);
  assertDeepEqual(
    capturedAction,
    {
      kind: "update-task-fields",
      label: "保存任务包字段",
      path: project.project_root,
      source: "索引内项目路径",
      boundary: "写入工作台自己的 workflow-state.v0.json；不生成真实任务文件、不派发真实 Codex 会话。",
      taskFields: {
        project_root: project.project_root,
        work_item_id: "work-item:offline:002",
        fields: {
          task_name: "字段编辑任务",
          assigned_line: "桌面应用线",
          background: ["来自结构化字段。"],
          goals: ["完成字段编辑。"],
          allowed_read: ["/tmp/indexed-project"],
          allowed_write: ["工作台状态文件"],
          forbidden_actions: ["不生成真实任务文件。"],
          acceptance_criteria: ["预览使用新字段。"],
          required_return: ["做了什么"],
          review_focus: ["确认结构化字段。"],
        },
      },
    },
    "保存任务字段待确认动作不匹配",
  );
  let saveFieldsConfirmed = false;
  const saveFieldsDialog = (
    <PermissionDialog
      action={capturedAction}
      busy={false}
      onCancel={() => {
        capturedAction = null;
      }}
      onConfirm={() => {
        saveFieldsConfirmed = true;
      }}
    />
  );
  const saveFieldsDialogText = visibleText(saveFieldsDialog);
  for (const expectedText of shellTexts.saveFieldsDialogExpectedTexts) {
    assert(saveFieldsDialogText.includes(expectedText), `保存字段确认弹层缺少 ${expectedText}`);
  }
  const cancelSaveFields = findButtonByText(saveFieldsDialog, "取消");
  assert(cancelSaveFields, "保存字段确认弹层缺少取消按钮");
  const cancelSaveFieldsClick = cancelSaveFields.props?.onClick;
  assert(typeof cancelSaveFieldsClick === "function", "保存字段取消按钮没有 onClick");
  cancelSaveFieldsClick({ preventDefault() {}, stopPropagation() {} });
  assert(capturedAction === null, "取消保存字段不应保留待确认动作");
  assert(!saveFieldsConfirmed, "取消保存字段不应执行保存");

  capturedAction = null;
  let reloadRequested = false;
  const statePanel = (
    <WorkflowStatePanel
      workflowState={workflowState}
      loading={false}
      error={null}
      onReload={() => {
        reloadRequested = true;
      }}
      onRequestAction={captureAction}
    />
  );
  const stateText = visibleText(statePanel);
  for (const expectedText of shellTexts.statePanelExpectedTexts) {
    assert(stateText.includes(expectedText), `事实层面板缺少 ${expectedText}`);
  }
  const reloadButton = findButtonByText(statePanel, "重新读取事实层");
  assert(reloadButton, "事实层面板缺少重新读取按钮");
  const reload = reloadButton.props?.onClick;
  assert(typeof reload === "function", "重新读取按钮没有 onClick");
  reload({ preventDefault() {}, stopPropagation() {} });
  assert(reloadRequested, "重新读取按钮没有触发回调");

  const initButton = findButtonByText(statePanel, "初始化工作流事实层");
  assert(initButton, "事实层面板缺少初始化按钮");
  const init = initButton.props?.onClick;
  assert(typeof init === "function", "初始化按钮没有 onClick");
  init({ preventDefault() {}, stopPropagation() {} });
  assertDeepEqual(
    capturedAction,
    {
      kind: "initialize-workflow-state",
      label: "初始化工作流事实层",
      path: workflowState.path,
      source: "Tauri 应用数据目录",
      boundary: "只写 workflow-state.v0.json 和同目录备份；不写 .codex、不写 Codex 状态库、不写项目业务目录。",
    },
    "初始化待确认动作不匹配",
  );

  const initDialogText = visibleText(
    <PermissionDialog action={capturedAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of shellTexts.initDialogExpectedTexts) {
    assert(initDialogText.includes(expectedText), `初始化确认弹层缺少 ${expectedText}`);
  }

  const skillText = visibleText(<SkillsBoardView skills={[skill]} plugins={[plugin]} projects={[project]} />);
  for (const expectedText of shellTexts.skillExpectedTexts) {
    assert(skillText.includes(expectedText), `Skill 看板缺少 ${expectedText}`);
  }

  const harnessText = visibleText(<HarnessBoardView projects={[project]} />);
  for (const expectedText of shellTexts.harnessExpectedTexts) {
    assert(harnessText.includes(expectedText), `Harness 看板缺少 ${expectedText}`);
  }
}

main();
