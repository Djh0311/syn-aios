import { useEffect, useRef, useState } from "react";
import { submitSupervisorResidentAnswer } from "../../../lib/tauri";
import type { ProjectWorkflowSummary, WorkflowStateSnapshot } from "../../../lib/types";
import {
  CONVERSATION_STREAM_ANCHOR_ID,
  HONEST_SHUTDOWN_NON_TEST_PROJECT_MESSAGE,
  type JiaobanComposerRoute,
} from "./JiaobanConversation";

type JiaobanConversationCache = {
  composerDraft: string;
  messageErrors: Record<string, string>;
};
const conversationCacheByProject = new Map<string, JiaobanConversationCache>();
const GENERIC_MESSAGE_STATE_KEY = "resident-message";

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
  projectRoot,
  onProposalStoreRefresh,
  humanizeAnswerError,
}: {
  projectWorkflow: ProjectWorkflowSummary | null;
  workflowState: WorkflowStateSnapshot | null;
  projectRoot: string;
  onProposalStoreRefresh?: () => Promise<void> | void;
  humanizeAnswerError: (error: unknown) => string;
}) {
  const cached = conversationCacheByProject.get(projectRoot);
  const [composerDraft, setComposerDraft] = useState(() => cached?.composerDraft ?? "");
  const [messageErrors, setMessageErrors] = useState<Record<string, string>>(() => cached?.messageErrors ?? {});
  const [messageBusy, setMessageBusy] = useState(false);
  const messageSubmittingRef = useRef(false);

  // ProjectWorkspaceShell 换 tab 会卸载本面板；按 project_root 留住草稿和失败提示。
  // 已发送消息只从后端 canonical 黑板重新派生，不保留前端副本。
  useEffect(() => {
    conversationCacheByProject.set(projectRoot, { composerDraft, messageErrors });
  }, [projectRoot, composerDraft, messageErrors]);

  async function submitMessage() {
    if (!projectWorkflow || messageSubmittingRef.current) return;
    const messageText = composerDraft.trim();
    if (!messageText) return;

    messageSubmittingRef.current = true;
    setMessageBusy(true);
    setMessageErrors({});
    try {
      await submitSupervisorResidentAnswer({
        project_id: projectWorkflow.project_id,
        workflow_id: projectWorkflow.workflow_id,
        message_text: messageText,
      });
      // 成功后的可见口供只来自后端 canonical blackboard；不在前端乐观追加一条用户消息，避免重读后重复。
      setComposerDraft("");
      try {
        // App 传入的是 reloadProposalAndWorkflowState：同次只重读 canonical workflow + proposal，不另造客户端事实。
        await onProposalStoreRefresh?.();
      } catch (error) {
        setMessageErrors({
          [GENERIC_MESSAGE_STATE_KEY]: `这句已经送到主管，但对话还没刷新——${humanizeAnswerError(error)}`,
        });
      }
    } catch (error) {
      setMessageErrors({ [GENERIC_MESSAGE_STATE_KEY]: humanizeAnswerError(error) });
    } finally {
      messageSubmittingRef.current = false;
      setMessageBusy(false);
    }
  }

  // S1：同一个常驻框在说/批/跑/交货/卡住都只走 user-message 命令。
  function makeConversationComposer({ isTestProject }: { isTestProject: boolean }) {
    // P1-E 诚实关门不随 S1 放开：非固定测试项目仍不能向已退役的旧入口伪装发送。
    if (!isTestProject) {
      return {
        route: { kind: "disabled" as const, reason: HONEST_SHUTDOWN_NON_TEST_PROJECT_MESSAGE },
        draft: composerDraft,
        busy: false,
        onDraftChange: setComposerDraft,
        onSubmit: () => {},
      };
    }
    const route: JiaobanComposerRoute = { kind: "message" };
    return {
      route,
      draft: composerDraft,
      busy: messageBusy,
      onDraftChange: (value: string) => {
        setComposerDraft(value);
        if (messageErrors[GENERIC_MESSAGE_STATE_KEY]) setMessageErrors({});
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
    setComposerDraft,
  };
}
