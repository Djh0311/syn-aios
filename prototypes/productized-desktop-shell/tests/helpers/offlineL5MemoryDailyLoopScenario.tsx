import React from "react";
import { PermissionDialog } from "../../src/components/PermissionDialog";
import { DailyMemoryCandidateInbox } from "../../src/components/DailyMemoryCandidateInbox";
import { deriveDailyMemoryCandidateInbox } from "../../src/lib/memoryDailyLoop";
import type { PendingAction, ProjectRecord, WorkbenchSnapshot, WorkflowStateSnapshot } from "../../src/lib/types";
import { assert, findButtonByText, visibleText } from "./offlineInteractionTestUtils";
import { memoryCenterCoreFixtures } from "./offlineMemoryCenterCoreFixtures";

// G4 宿主迁移(2026-07-20·预登记 M1)：原宿主=RunningWorkflowsView(死视图整删)，
// 迁移到被测组件本体 DailyMemoryCandidateInbox 直渲——断言逐条平移,覆盖不丢。
// 尾块 operation control 按钮断言随死视图壳退役(预登记 R8·理由与替代覆盖在档)。
export function runL5MemoryDailyLoopScenario({
  snapshot: _snapshot,
  workflowStateWithProjectWorkflow: _workflowStateWithProjectWorkflow,
  project,
}: {
  snapshot: WorkbenchSnapshot;
  workflowStateWithProjectWorkflow: WorkflowStateSnapshot;
  project: ProjectRecord;
}) {
  const { formalMemoryStore, memoryCandidateStore } = memoryCenterCoreFixtures();
  const inbox = deriveDailyMemoryCandidateInbox({ memoryCandidateStore });
  const inboxText = visibleText(
    <DailyMemoryCandidateInbox
      inbox={inbox}
      projectRoot={project.project_root}
      candidateStoreRevision={memoryCandidateStore.revision ?? null}
      formalStoreRevision={formalMemoryStore.revision ?? null}
    />,
  );

  assert(inboxText.includes("日常记忆候选收件箱"), "L5 收件箱应有日常记忆候选收件箱");
  assert(inboxText.includes("1 条记忆候选待确认"), "L5 收件箱应显示待确认候选数量");
  assert(inboxText.includes("候选不是正式记忆，采纳前必须确认"), "L5 收件箱必须说明候选边界");

  const capturedActions: PendingAction[] = [];
  const captureInbox = (candidateStore: typeof memoryCandidateStore) => (
    <DailyMemoryCandidateInbox
      inbox={deriveDailyMemoryCandidateInbox({ memoryCandidateStore: candidateStore })}
      projectRoot={project.project_root}
      candidateStoreRevision={candidateStore.revision ?? null}
      formalStoreRevision={formalMemoryStore.revision ?? null}
      onRequestAction={(action) => {
        capturedActions.push(action);
      }}
    />
  );
  const runningInbox = captureInbox(memoryCandidateStore);
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
  const reviewInbox = captureInbox(reviewCandidateStore);
  const rejectButton = findButtonByText(reviewInbox, "拒绝候选");
  assert(rejectButton, "L5 收件箱应提供拒绝候选入口");
  const rejectClick = rejectButton.props?.onClick;
  assert(typeof rejectClick === "function", "L5 拒绝候选按钮应有 onClick");
  rejectClick({ preventDefault() {}, stopPropagation() {} });
  const rejectAction = capturedActions[0];
  assert(rejectAction?.kind === "record-memory-candidate-decision", "L5 拒绝候选必须复用候选决策路径");
  assert(rejectAction.memoryCandidateDecision?.requested_status === "candidate_rejected", "L5 拒绝候选应写 rejected 状态");
}
