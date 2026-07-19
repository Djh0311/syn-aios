import { useEffect, useRef, useState } from "react";
import { submitSupervisorResidentAnswer } from "../../../lib/tauri";
import type { ProjectWorkflowSummary, WorkflowStateSnapshot } from "../../../lib/types";
import {
  CONVERSATION_STREAM_ANCHOR_ID,
  HONEST_SHUTDOWN_NON_TEST_PROJECT_MESSAGE,
  type JiaobanComposerRoute,
} from "./JiaobanConversation";

type ResidentMessageOutcome = {
  status: string;
};

type ResidentMessageReconciliation = {
  clearDraft: boolean;
  messageError: string | null;
};

const MESSAGE_NOT_SENT = "这句没送到主管——稍后再试一次。";
const MESSAGE_RECORDED_NO_SUPERVISOR_REPLY = "消息已送到主管，但主管这次没回上来——可以再发一次。";
const MESSAGE_RECORDED_PROPOSAL_FAILED = "主管收到了，但方案卡没有生成——请再说一次“出方案”。";
const MESSAGE_REFRESH_PENDING = "这句已经送到主管，但对话还没刷新。";
const MESSAGE_DELIVERY_UNKNOWN = "消息状态暂时无法确认——请稍后刷新后再试一次。";

// This is deliberately I/O-injected so the offline interaction suite can
// prove the user-facing truth table without creating a second client-side
// message or card.  Production still renders only after the supplied
// canonical workflow/proposal refresh completes.
export async function reconcileResidentMessageSubmission({
  submit,
  refreshCanonicalAndProposal,
}: {
  submit: () => Promise<ResidentMessageOutcome>;
  refreshCanonicalAndProposal: () => Promise<void>;
}): Promise<ResidentMessageReconciliation> {
  let outcome: ResidentMessageOutcome;
  try {
    outcome = await submit();
  } catch {
    // A transport-level rejection cannot prove the canonical append did not
    // happen.  Refresh anyway, but never turn uncertainty into “没送到”.
    try {
      await refreshCanonicalAndProposal();
    } catch {
      // The caller gets the same non-assertive state below.
    }
    return { clearDraft: false, messageError: MESSAGE_DELIVERY_UNKNOWN };
  }

  try {
    await refreshCanonicalAndProposal();
  } catch {
    if (outcome.status === "message_not_recorded") {
      return { clearDraft: false, messageError: MESSAGE_DELIVERY_UNKNOWN };
    }
    return { clearDraft: true, messageError: MESSAGE_REFRESH_PENDING };
  }

  switch (outcome.status) {
    case "message_not_recorded":
      return { clearDraft: false, messageError: MESSAGE_NOT_SENT };
    case "message_recorded_supervisor_incomplete":
      return { clearDraft: true, messageError: MESSAGE_RECORDED_NO_SUPERVISOR_REPLY };
    case "message_sent_proposal_tool_failed":
      return { clearDraft: true, messageError: MESSAGE_RECORDED_PROPOSAL_FAILED };
    default:
      return { clearDraft: true, messageError: null };
  }
}

type JiaobanConversationCache = {
  composerDraft: string;
  messageErrors: Record<string, string>;
  pendingClientRequestId: string | null;
};
const conversationCacheByProject = new Map<string, JiaobanConversationCache>();
const GENERIC_MESSAGE_STATE_KEY = "resident-message";

// A3·视口停最新：进入对话/切项目/新消息落地都停在最新——signal 随调用方判「当前单有没有新东西」变。
export function useConversationAutoScroll(signal: unknown) {
  useEffect(() => {
    document.getElementById(CONVERSATION_STREAM_ANCHOR_ID)?.scrollIntoView({ block: "end" });
  }, [signal]);
}

export function useJiaobanConversationState({
  projectWorkflow,
  projectRoot,
  onProposalStoreRefresh,
}: {
  projectWorkflow: ProjectWorkflowSummary | null;
  workflowState: WorkflowStateSnapshot | null;
  projectRoot: string;
  onProposalStoreRefresh?: () => Promise<void> | void;
}) {
  const cached = conversationCacheByProject.get(projectRoot);
  const [composerDraft, setComposerDraft] = useState(() => cached?.composerDraft ?? "");
  const [messageErrors, setMessageErrors] = useState<Record<string, string>>(() => cached?.messageErrors ?? {});
  const [pendingClientRequestId, setPendingClientRequestId] = useState<string | null>(
    () => cached?.pendingClientRequestId ?? null,
  );
  const [messageBusy, setMessageBusy] = useState(false);
  const messageSubmittingRef = useRef(false);

  // ProjectWorkspaceShell 换 tab 会卸载本面板；按 project_root 留住草稿和失败提示。
  // 已发送消息只从后端 canonical 黑板重新派生，不保留前端副本。
  useEffect(() => {
    conversationCacheByProject.set(projectRoot, { composerDraft, messageErrors, pendingClientRequestId });
  }, [projectRoot, composerDraft, messageErrors, pendingClientRequestId]);

  async function submitMessage() {
    if (!projectWorkflow || messageSubmittingRef.current) return;
    const messageText = composerDraft.trim();
    if (!messageText) return;
    const clientRequestId = pendingClientRequestId ?? globalThis.crypto?.randomUUID?.();
    if (!clientRequestId) {
      setMessageErrors({ [GENERIC_MESSAGE_STATE_KEY]: MESSAGE_DELIVERY_UNKNOWN });
      return;
    }

    messageSubmittingRef.current = true;
    setMessageBusy(true);
    setMessageErrors({});
    if (!pendingClientRequestId) setPendingClientRequestId(clientRequestId);
    try {
      const reconciliation = await reconcileResidentMessageSubmission({
        submit: () =>
          submitSupervisorResidentAnswer({
            project_id: projectWorkflow.project_id,
            workflow_id: projectWorkflow.workflow_id,
            message_text: messageText,
            client_request_id: clientRequestId,
          }),
        refreshCanonicalAndProposal: async () => {
          await onProposalStoreRefresh?.();
        },
      });
      // 成功后的可见口供只来自后端 canonical blackboard；不在前端乐观追加一条用户消息，避免重读后重复。
      if (reconciliation.clearDraft) {
        setComposerDraft("");
        setPendingClientRequestId(null);
      }
      if (reconciliation.messageError) {
        setMessageErrors({ [GENERIC_MESSAGE_STATE_KEY]: reconciliation.messageError });
      }
    } finally {
      messageSubmittingRef.current = false;
      setMessageBusy(false);
    }
  }

  function updateComposerDraft(value: string) {
    setComposerDraft(value);
    setPendingClientRequestId(null);
    if (messageErrors[GENERIC_MESSAGE_STATE_KEY]) setMessageErrors({});
  }

  // S1：同一个常驻框在说/批/跑/交货/卡住都只走 user-message 命令。
  function makeConversationComposer({ isTestProject }: { isTestProject: boolean }) {
    // P1-E 诚实关门不随 S1 放开：非固定测试项目仍不能向已退役的旧入口伪装发送。
    if (!isTestProject) {
      return {
        route: { kind: "disabled" as const, reason: HONEST_SHUTDOWN_NON_TEST_PROJECT_MESSAGE },
        draft: composerDraft,
        busy: false,
        onDraftChange: updateComposerDraft,
        onSubmit: () => {},
      };
    }
    const route: JiaobanComposerRoute = { kind: "message" };
    return {
      route,
      draft: composerDraft,
      busy: messageBusy,
      onDraftChange: (value: string) => {
        updateComposerDraft(value);
      },
      onSubmit: () => {
        void submitMessage();
      },
    };
  }

  return {
    messageBusy,
    messageErrors,
    makeConversationComposer,
    setComposerDraft: updateComposerDraft,
  };
}
