import React from "react";
import { renderToStaticMarkup } from "react-dom/server.browser";
import { DailyMemoryCandidateInbox } from "../src/components/DailyMemoryCandidateInbox";
import { PermissionDialog } from "../src/components/PermissionDialog";
import { deriveDailyMemoryCandidateInbox } from "../src/lib/memoryDailyLoop";
import type { PendingAction } from "../src/lib/types";
import { MemoryCenterView } from "../src/views/MemoryCenterView";
import { assert, findButtonByText, visibleText } from "./helpers/offlineInteractionTestUtils";
import { memoryCenterCoreFixtures } from "./helpers/offlineMemoryCenterCoreFixtures";
import { offlineScenarioEnvironmentFixtures } from "./helpers/offlineScenarioEnvironmentFixtures";

const { project, workflowStateWithProjectWorkflow } = offlineScenarioEnvironmentFixtures();
const { formalMemoryStore, memoryCandidateStore, memoryCaptureStore, observationStore } = memoryCenterCoreFixtures();
const capturedActions: PendingAction[] = [];
const inbox = deriveDailyMemoryCandidateInbox({ memoryCandidateStore });
const confirmedItem = inbox.items.find((item) => item.can_adopt);

assert(confirmedItem, "离线 fixture 缺少可采纳的已确认候选");

const memoryCenterMarkup = renderToStaticMarkup(
  <MemoryCenterView
    projects={[project]}
    workflowState={workflowStateWithProjectWorkflow}
    formalMemoryStore={formalMemoryStore}
    memoryCaptureStore={memoryCaptureStore}
    memoryCandidateStore={memoryCandidateStore}
    observationStore={observationStore}
    hasRealSnapshot
    onRequestAction={(action) => capturedActions.push(action)}
  />,
);

assert(memoryCenterMarkup.includes("daily-memory-candidate-inbox"), "记忆中心候选区必须挂载日常候选收件箱");
assert(memoryCenterMarkup.includes("采纳为正式记忆"), "候选详情必须显示采纳为正式记忆动作");

const inboxElement = (
  <DailyMemoryCandidateInbox
    inbox={inbox}
    projectRoot={project.project_root}
    candidateStoreRevision={memoryCandidateStore.revision}
    formalStoreRevision={formalMemoryStore.revision}
    onRequestAction={(action) => capturedActions.push(action)}
  />
);
const adoptButton = findButtonByText(inboxElement, `采纳候选：${confirmedItem.claim}`);
assert(adoptButton, "收件箱缺少已确认候选的采纳按钮");
const requestAdoption = adoptButton.props?.onClick;
assert(typeof requestAdoption === "function", "收件箱采纳按钮必须接到既有动作分发器");
requestAdoption({ preventDefault() {}, stopPropagation() {} });

const adoptionAction = capturedActions.at(-1);
assert(adoptionAction?.kind === "adopt-memory-candidate-to-formal-memory", "收件箱必须派发既有 M2 采纳动作");
assert(
  adoptionAction.memoryCandidateAdoption?.candidate_key === confirmedItem.candidate_key,
  "收件箱采纳动作必须保留候选 key",
);
const adoptionDialogText = visibleText(
  <PermissionDialog action={adoptionAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
);
assert(adoptionDialogText.includes("确认采纳"), "采纳动作必须复用既有确认对话");

const reviewCandidateStore = {
  ...memoryCandidateStore,
  candidates: memoryCandidateStore.candidates.map((candidate) =>
    candidate.candidate_key === confirmedItem.candidate_key
      ? { ...candidate, status: "candidate_needs_review" as const }
      : candidate,
  ),
};
const reviewInbox = deriveDailyMemoryCandidateInbox({ memoryCandidateStore: reviewCandidateStore });
const reviewInboxElement = (
  <DailyMemoryCandidateInbox
    inbox={reviewInbox}
    projectRoot={project.project_root}
    candidateStoreRevision={reviewCandidateStore.revision}
    formalStoreRevision={formalMemoryStore.revision}
    onRequestAction={(action) => capturedActions.push(action)}
  />
);
const confirmButton = findButtonByText(reviewInboxElement, "确认候选属实");
assert(confirmButton, "待审查候选必须能先确认属实");
const requestConfirmation = confirmButton.props?.onClick;
assert(typeof requestConfirmation === "function", "候选确认按钮必须接到既有候选决定动作");
requestConfirmation({ preventDefault() {}, stopPropagation() {} });

const confirmationAction = capturedActions.at(-1);
assert(confirmationAction?.kind === "record-memory-candidate-decision", "确认属实必须派发既有候选决定动作");
assert(
  confirmationAction.memoryCandidateDecision?.requested_status === "candidate_confirmed",
  "确认属实必须只将候选转为 candidate_confirmed",
);
