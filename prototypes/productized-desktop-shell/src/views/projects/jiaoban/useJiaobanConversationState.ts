import { useEffect, useRef, useState } from "react";
import { submitSupervisorResidentAnswer } from "../../../lib/tauri";
import type { ProjectConsultationProposal, ProjectWorkflowSummary, WorkflowStateSnapshot } from "../../../lib/types";
import type { JiaobanPhase } from "./JiaobanArtifactViews";
import {
  CONVERSATION_STREAM_ANCHOR_ID,
  HONEST_SHUTDOWN_NON_TEST_PROJECT_MESSAGE,
  humanizeResidentAnswerOutcome,
  latestWaitingQuestionIdOf,
  supervisorConversationEntriesForProject,
  type JiaobanComposerRoute,
  type JiaobanConversationUserTurn,
} from "./JiaobanConversation";
import { proposalAgeDays } from "./jiaobanTime";

type JiaobanConversationCache = {
  answerDrafts: Record<string, string>;
  answerReceipts: Record<string, string>;
  answerErrors: Record<string, string>;
  composerDraft: string;
  userTurns: JiaobanConversationUserTurn[];
};
const conversationCacheByProject = new Map<string, JiaobanConversationCache>();

export function scrollToConversationMessage(messageId: string | null) {
  if (!messageId) return;
  window.requestAnimationFrame(() => {
    window.requestAnimationFrame(() => {
      const target = document.getElementById(messageId);
      // 锚点可能落在「更早的 N 单对话」折叠段里；先展开沿途所有 <details> 再滚，否则隐藏内容滚不到位。
      for (let node = target?.parentElement ?? null; node; node = node.parentElement) {
        if (node instanceof HTMLDetailsElement) node.open = true;
      }
      target?.scrollIntoView({ behavior: "smooth", block: "start" });
    });
  });
}

// A3·视口停最新：进入对话/切项目/新消息落地都停在最新——signal 随调用方判「当前单有没有新东西」变。
export function useConversationAutoScroll(signal: unknown) {
  useEffect(() => {
    document.getElementById(CONVERSATION_STREAM_ANCHOR_ID)?.scrollIntoView({ block: "end" });
  }, [signal]);
}

export function useJiaobanConversationState({
  projectWorkflow,
  workflowState,
  projectRoot,
  onProposalStoreRefresh,
  onAnswerAccepted,
  humanizeAnswerError,
}: {
  projectWorkflow: ProjectWorkflowSummary | null;
  workflowState: WorkflowStateSnapshot | null;
  projectRoot: string;
  onProposalStoreRefresh?: () => Promise<void> | void;
  onAnswerAccepted: () => void;
  humanizeAnswerError: (error: unknown) => string;
}) {
  const cached = conversationCacheByProject.get(projectRoot);
  const [answerDrafts, setAnswerDrafts] = useState<Record<string, string>>(() => cached?.answerDrafts ?? {});
  const [answerBusyQuestionId, setAnswerBusyQuestionId] = useState<string | null>(null);
  const [answerReceipts, setAnswerReceipts] = useState<Record<string, string>>(() => cached?.answerReceipts ?? {});
  const [answerErrors, setAnswerErrors] = useState<Record<string, string>>(() => cached?.answerErrors ?? {});
  const answerSubmittingRef = useRef<string | null>(null);
  const [userTurns, setUserTurns] = useState<JiaobanConversationUserTurn[]>(() => cached?.userTurns ?? []);
  const [composerDraft, setComposerDraft] = useState(() => cached?.composerDraft ?? "");
  const userTurnSequenceRef = useRef(0);

  // ProjectWorkspaceShell 换 tab 会卸载本面板；按 project_root 留住草稿、回执和本轮真实口供。
  useEffect(() => {
    conversationCacheByProject.set(projectRoot, { answerDrafts, answerReceipts, answerErrors, composerDraft, userTurns });
  }, [projectRoot, answerDrafts, answerReceipts, answerErrors, composerDraft, userTurns]);

  function recordUserTurn(text: string, createdAtMs: number) {
    userTurnSequenceRef.current += 1;
    const turn = {
      id: `jiaoban-user-turn-${createdAtMs}-${userTurnSequenceRef.current}`,
      text,
      createdAtMs,
    };
    setUserTurns((current) => (current.at(-1)?.text === text ? current : [...current, turn]));
  }

  async function submitAnswer(questionId: string) {
    if (!projectWorkflow || answerSubmittingRef.current) return;
    const answerText = answerDrafts[questionId]?.trim() ?? "";
    if (!answerText) return;
    const waitingQuestion = supervisorConversationEntriesForProject(
      workflowState,
      projectRoot,
      projectWorkflow.workflow_id,
    ).find(
      (entry) =>
        entry.title.startsWith("主管问题") &&
        (entry.source_status ?? entry.status) === "waiting_user" &&
        entry.question_id === questionId,
    );
    if (!waitingQuestion) return;

    answerSubmittingRef.current = questionId;
    setAnswerBusyQuestionId(questionId);
    setAnswerReceipts((current) => ({ ...current, [questionId]: "" }));
    setAnswerErrors((current) => ({ ...current, [questionId]: "" }));
    try {
      const result = await submitSupervisorResidentAnswer({
        project_id: projectWorkflow.project_id,
        workflow_id: projectWorkflow.workflow_id,
        question_id: questionId,
        answer_text: answerText,
      });
      onAnswerAccepted();
      setAnswerReceipts((current) => ({
        ...current,
        [questionId]: humanizeResidentAnswerOutcome(result),
      }));
      if (result.status !== "already_answered") {
        setAnswerDrafts((current) => ({ ...current, [questionId]: "" }));
      }
      await onProposalStoreRefresh?.();
    } catch (error) {
      setAnswerErrors((current) => ({ ...current, [questionId]: humanizeAnswerError(error) }));
    } finally {
      answerSubmittingRef.current = null;
      setAnswerBusyQuestionId(null);
    }
  }

  // 常驻输入框(修单3):话按当前状态路由既有三通道,零新命令。工厂制——phase/方案要到
  // Panel 中段才定,而路由状态与草稿缓存留在本 hook(跨 tab 不丢)。
  function makeConversationComposer({
    phase,
    latestProposal,
    consultLoading,
    isTestProject,
    onAmendment,
    onNewGoal,
  }: {
    phase: JiaobanPhase;
    latestProposal: Pick<ProjectConsultationProposal, "created_at_ms"> | null;
    consultLoading: boolean;
    isTestProject: boolean;
    onAmendment: (text: string) => void;
    onNewGoal: (text: string) => void;
  }) {
    // blocked=P3-C 通道不渲常驻框;say 也走同一个框(07-18 用户拍「只要一个对话框」);跑态禁发诚实说明。
    if (phase === "blocked") return null;
    // P1-E 诚实关门（用户拍板 a·不豁免站 3b）：非固定测试项目——常驻框只出这一句人话，说新目标/按我说的改
    // 都会打到后端已退役的塞纸条路；复用既有 disabled 路由形态，判据仍是调用方传入的 path-lock 结果，
    // 这里不新造判断。已批准方案的[允许并开始]执行流走按钮而非本框，不受影响。
    if (!isTestProject) {
      return {
        route: { kind: "disabled" as const, reason: HONEST_SHUTDOWN_NON_TEST_PROJECT_MESSAGE },
        draft: composerDraft,
        busy: false,
        onDraftChange: setComposerDraft,
        onSubmit: () => {},
      };
    }
    const waitingQuestionId = latestWaitingQuestionIdOf(
      supervisorConversationEntriesForProject(workflowState, projectRoot, projectWorkflow?.workflow_id),
    );
    const route: JiaobanComposerRoute = waitingQuestionId
      ? { kind: "answer", questionId: waitingQuestionId }
      : phase === "say"
        ? { kind: "new_goal", placeholder: "想让 AI 干点啥？直接说——例：给这小游戏加个计分板。" }
        : phase === "authorize" && latestProposal
          ? proposalAgeDays(latestProposal.created_at_ms) >= 1
            ? { kind: "new_goal" }
            : { kind: "amendment" }
          : phase === "done"
            ? { kind: "new_goal" }
            : { kind: "disabled", reason: "正在干活——有话等交货或卡住时说" };
    const draft = route.kind === "answer" ? answerDrafts[route.questionId] ?? "" : composerDraft;
    return {
      route,
      draft,
      busy: route.kind === "answer" ? answerBusyQuestionId != null : consultLoading,
      onDraftChange: (value: string) =>
        route.kind === "answer"
          ? setAnswerDrafts((current) => ({ ...current, [route.questionId]: value }))
          : setComposerDraft(value),
      onSubmit: () => {
        if (route.kind === "disabled" || !draft.trim()) return;
        if (route.kind === "answer") {
          void submitAnswer(route.questionId);
          return;
        }
        (route.kind === "amendment" ? onAmendment : onNewGoal)(draft.trim());
        setComposerDraft("");
      },
    };
  }

  return {
    answerDrafts,
    answerBusyQuestionId,
    answerReceipts,
    answerErrors,
    userTurns,
    setAnswerDraft: (questionId: string, value: string) =>
      setAnswerDrafts((current) => ({ ...current, [questionId]: value })),
    recordUserTurn,
    submitAnswer,
    makeConversationComposer,
    setComposerDraft,
  };
}
