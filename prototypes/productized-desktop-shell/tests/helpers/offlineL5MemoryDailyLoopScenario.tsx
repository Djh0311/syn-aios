import React from "react";
import { PermissionDialog } from "../../src/components/PermissionDialog";
import type { PendingAction, ProjectRecord, WorkbenchSnapshot, WorkflowStateSnapshot } from "../../src/lib/types";
import { RunningWorkflowsView } from "../../src/views/RunningWorkflowsView";
import { assert, findButtonByText, visibleText } from "./offlineInteractionTestUtils";
import { memoryCenterCoreFixtures } from "./offlineMemoryCenterCoreFixtures";

export function runL5MemoryDailyLoopScenario({
  snapshot,
  workflowStateWithProjectWorkflow,
  project,
}: {
  snapshot: WorkbenchSnapshot;
  workflowStateWithProjectWorkflow: WorkflowStateSnapshot;
  project: ProjectRecord;
}) {
  const { formalMemoryStore, memoryCaptureStore, memoryCandidateStore } = memoryCenterCoreFixtures();
  const runningText = visibleText(
    <RunningWorkflowsView
      snapshot={snapshot}
      workflowState={workflowStateWithProjectWorkflow}
      workflowStateLoading={false}
      workflowStateError={null}
      memoryCaptureStore={memoryCaptureStore}
      memoryCandidateStore={memoryCandidateStore}
      formalMemoryStore={formalMemoryStore}
      onReloadWorkflowState={() => {}}
      onNavigate={() => {}}
    />,
  );

  assert(runningText.includes("日常记忆候选收件箱"), "L5 运行页应有日常记忆候选收件箱");
  assert(runningText.includes("1 条记忆候选待确认"), "L5 收件箱应显示待确认候选数量");
  assert(runningText.includes("候选不是正式记忆，采纳前必须确认"), "L5 收件箱必须说明候选边界");

  const capturedActions: PendingAction[] = [];
  const runningInbox = (
    <RunningWorkflowsView
      snapshot={snapshot}
      workflowState={workflowStateWithProjectWorkflow}
      workflowStateLoading={false}
      workflowStateError={null}
      memoryCaptureStore={memoryCaptureStore}
      memoryCandidateStore={memoryCandidateStore}
      formalMemoryStore={formalMemoryStore}
      onReloadWorkflowState={() => {}}
      onNavigate={() => {}}
      onRequestAction={(action) => {
        capturedActions.push(action);
      }}
    />
  );
  const adoptButton = findButtonByText(runningInbox, "采纳候选：候选需要确认要求和风险提示。");
  assert(adoptButton, "L5 收件箱应提供单条采纳按钮");
  const adoptClick = adoptButton.props?.onClick;
  assert(typeof adoptClick === "function", "L5 单条采纳按钮应有 onClick");
  adoptClick({ preventDefault() {}, stopPropagation() {} });
  const adoptAction = capturedActions[0];
  assert(adoptAction?.kind === "adopt-memory-candidate-to-formal-memory", "L5 单条采纳必须复用 M2 pending action");
  assert(adoptAction.memoryCandidateAdoption?.candidate_key === "memcand:v1:memory-center-review", "L5 单条采纳 candidate_key 不匹配");
  assert(adoptAction.memoryCandidateAdoption?.actor_role === "user", "L5 需要用户确认的候选必须由 user 采纳");
  assert(adoptAction.path === project.project_root, "L5 单条采纳应绑定当前项目路径");

  const dialogText = visibleText(<PermissionDialog action={adoptAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />);
  assert(dialogText.includes("采纳候选为正式记忆"), "L5 单条采纳应进入既有 M2 权限弹窗");
  assert(!dialogText.includes("已自动记入正式记忆"), "L5 不得出现自动正式化文案");

  capturedActions.length = 0;
  const batchButton = findButtonByText(runningInbox, "批量采纳 1 条候选");
  assert(batchButton, "L5 收件箱应提供批量采纳入口");
  const batchClick = batchButton.props?.onClick;
  assert(typeof batchClick === "function", "L5 批量采纳按钮应有 onClick");
  batchClick({ preventDefault() {}, stopPropagation() {} });
  const batchAction = capturedActions[0];
  assert(batchAction?.kind === "adopt-memory-candidates-to-formal-memory-batch", "L5 批量采纳必须是显式批量确认 action");
  assert(batchAction.memoryCandidateBatchAdoptions?.length === 1, "L5 批量采纳应逐条复用 M2 输入");

  capturedActions.length = 0;
  const deferButton = findButtonByText(runningInbox, "暂不处理");
  assert(deferButton, "L5 收件箱应提供暂不处理入口");
  const deferClick = deferButton.props?.onClick;
  assert(typeof deferClick === "function", "L5 暂不处理按钮应有 onClick");
  deferClick({ preventDefault() {}, stopPropagation() {} });
  const deferAction = capturedActions[0];
  assert(deferAction?.kind === "record-memory-candidate-decision", "L5 暂不处理必须复用候选决策路径");
  assert(deferAction.memoryCandidateDecision?.requested_status === "candidate_discarded", "L5 暂不处理应写候选状态而非正式记忆");

  capturedActions.length = 0;
  const reviewCandidateStore = {
    ...memoryCandidateStore,
    candidates: memoryCandidateStore.candidates.map((candidate) =>
      candidate.candidate_key === "memcand:v1:memory-center-review"
        ? { ...candidate, status: "candidate_needs_review" as const }
        : candidate,
    ),
  };
  const reviewInbox = (
    <RunningWorkflowsView
      snapshot={snapshot}
      workflowState={workflowStateWithProjectWorkflow}
      workflowStateLoading={false}
      workflowStateError={null}
      memoryCaptureStore={memoryCaptureStore}
      memoryCandidateStore={reviewCandidateStore}
      formalMemoryStore={formalMemoryStore}
      onReloadWorkflowState={() => {}}
      onNavigate={() => {}}
      onRequestAction={(action) => {
        capturedActions.push(action);
      }}
    />
  );
  const rejectButton = findButtonByText(reviewInbox, "拒绝候选");
  assert(rejectButton, "L5 收件箱应提供拒绝候选入口");
  const rejectClick = rejectButton.props?.onClick;
  assert(typeof rejectClick === "function", "L5 拒绝候选按钮应有 onClick");
  rejectClick({ preventDefault() {}, stopPropagation() {} });
  const rejectAction = capturedActions[0];
  assert(rejectAction?.kind === "record-memory-candidate-decision", "L5 拒绝候选必须复用候选决策路径");
  assert(rejectAction.memoryCandidateDecision?.requested_status === "candidate_rejected", "L5 拒绝候选应写 rejected 状态");

  capturedActions.length = 0;
  const operationButton = findButtonByText(runningInbox, "确认记录 恢复 决策");
  assert(operationButton, "L5 应保留 L3 operation control 确认入口");
  const operationClick = operationButton.props?.onClick;
  assert(typeof operationClick === "function", "L5 operation control 按钮应有 onClick");
  operationClick({ preventDefault() {}, stopPropagation() {} });
  const operationAction = capturedActions[0];
  assert(operationAction?.kind === "record-operation-control-decision", "L5 operation control action kind 不匹配");
  assert(operationAction.memoryCaptureEvent?.source_type === "operation_control_decision", "L5 operation control 确认应携带 capture input");
  assert(operationAction.memoryCaptureEvent?.candidate_policy === "candidate_allowed", "L5 operation control capture 应生成候选");
  assert(operationAction.memoryCaptureEvent?.candidate?.requires_user_confirmation, "L5 operation control 候选应需要用户确认");
}
