import { useCallback, useEffect, useRef, useState } from "react";
import { mergeOlderTranscriptPage } from "../../lib/conversationEngine";
import type { CodexTranscript, CodexTranscriptPageRequest, SessionRecord } from "../../lib/types";
import { messageOf } from "./agentLabels";

const DEFAULT_TRANSCRIPT_PAGE_LIMIT = 80;

export function useAgentTranscriptLoader({
  onLoadTranscript,
  onLoadTranscriptPage,
  selectedSession,
}: {
  onLoadTranscript?: (threadId: string) => Promise<CodexTranscript>;
  onLoadTranscriptPage?: (request: CodexTranscriptPageRequest) => Promise<CodexTranscript>;
  selectedSession: SessionRecord | null;
}) {
  const [transcript, setTranscript] = useState<CodexTranscript | null>(null);
  const [transcriptCache, setTranscriptCache] = useState<Map<string, CodexTranscript>>(() => new Map());
  const [loadingThreadId, setLoadingThreadId] = useState<string | null>(null);
  const [loadingOlderThreadId, setLoadingOlderThreadId] = useState<string | null>(null);
  const [transcriptError, setTranscriptError] = useState<string | null>(null);
  const selectedThreadIdRef = useRef<string | null>(selectedSession?.thread_id ?? null);

  useEffect(() => {
    selectedThreadIdRef.current = selectedSession?.thread_id ?? null;
  }, [selectedSession?.thread_id]);

  const loadTranscript = useCallback(
    async (threadId: string) => {
      setTranscriptError(null);
      if (!onLoadTranscript) {
        if (onLoadTranscriptPage) {
          setLoadingThreadId(threadId);
          try {
            const nextTranscript = await onLoadTranscriptPage({
              before_line: null,
              limit: DEFAULT_TRANSCRIPT_PAGE_LIMIT,
              thread_id: threadId,
            });
            setTranscriptCache((current) => new Map(current).set(threadId, nextTranscript));
            setTranscript((current) => (threadId === selectedThreadIdRef.current ? nextTranscript : current));
          } catch (error) {
            setTranscriptError((current) => (threadId === selectedThreadIdRef.current ? messageOf(error) : current));
          } finally {
            setLoadingThreadId((current) => (current === threadId ? null : current));
          }
          return;
        }
        setTranscriptError("当前运行环境没有接入会话记录读取入口。");
        return;
      }
      setLoadingThreadId(threadId);
      try {
        const nextTranscript = onLoadTranscriptPage
          ? await onLoadTranscriptPage({
              before_line: null,
              limit: DEFAULT_TRANSCRIPT_PAGE_LIMIT,
              thread_id: threadId,
            })
          : await onLoadTranscript(threadId);
        setTranscriptCache((current) => new Map(current).set(threadId, nextTranscript));
        setTranscript((current) => (threadId === selectedThreadIdRef.current ? nextTranscript : current));
      } catch (error) {
        setTranscriptError((current) => (threadId === selectedThreadIdRef.current ? messageOf(error) : current));
      } finally {
        setLoadingThreadId((current) => (current === threadId ? null : current));
      }
    },
    [onLoadTranscript, onLoadTranscriptPage],
  );

  const loadOlderTranscript = useCallback(
    async (threadId: string) => {
      if (!onLoadTranscriptPage) return;
      const currentTranscript =
        transcript?.thread_id === threadId ? transcript : transcriptCache.get(threadId) ?? null;
      const cursor = currentTranscript?.pagination?.older_before_line;
      if (!currentTranscript?.pagination?.has_older || !cursor) return;
      setTranscriptError(null);
      setLoadingOlderThreadId(threadId);
      try {
        const olderPage = await onLoadTranscriptPage({
          before_line: cursor,
          limit: currentTranscript.pagination.page_size || DEFAULT_TRANSCRIPT_PAGE_LIMIT,
          thread_id: threadId,
        });
        const mergedTranscript = mergeOlderTranscriptPage(currentTranscript, olderPage);
        setTranscriptCache((current) => new Map(current).set(threadId, mergedTranscript));
        setTranscript((current) => (threadId === selectedThreadIdRef.current ? mergedTranscript : current));
      } catch (error) {
        setTranscriptError((current) => (threadId === selectedThreadIdRef.current ? messageOf(error) : current));
      } finally {
        setLoadingOlderThreadId((current) => (current === threadId ? null : current));
      }
    },
    [onLoadTranscriptPage, transcript, transcriptCache],
  );

  const selectedTranscript =
    transcript?.thread_id === selectedSession?.thread_id
      ? transcript
      : selectedSession
        ? transcriptCache.get(selectedSession.thread_id) ?? null
        : null;

  useEffect(() => {
    if (!selectedSession?.rollout_exists || !selectedSession.rollout_path) return;
    if (selectedTranscript?.thread_id === selectedSession.thread_id || loadingThreadId === selectedSession.thread_id) return;
    void loadTranscript(selectedSession.thread_id);
  }, [loadTranscript, loadingThreadId, selectedSession, selectedTranscript?.thread_id]);

  return {
    loadingOlderThreadId,
    loadingThreadId,
    loadOlderTranscript,
    loadTranscript,
    selectedTranscript,
    transcriptError,
  };
}
