import { useCallback, useEffect, useMemo, useState } from "react";
import { loadCodexSessionPage } from "../../lib/tauri";
import type { CodexSessionPage, CodexSessionPageRequest, SessionRecord } from "../../lib/types";
import { messageOf } from "./agentLabels";
import type { SessionReadFilter } from "./AgentSessionList";

const AGENT_SESSION_PAGE_SIZE = 100;

export function useAgentSessionPage(
  initialSessions: SessionRecord[],
  searchQuery = "",
  loadSessionPageFromSource: (request: CodexSessionPageRequest) => Promise<CodexSessionPage> = loadCodexSessionPage,
) {
  const [shellSessions, setShellSessions] = useState<SessionRecord[]>(initialSessions);
  const [sessionPageOffset, setSessionPageOffset] = useState(0);
  const [sessionPageHasMore, setSessionPageHasMore] = useState(false);
  const [sessionPageSource, setSessionPageSource] = useState<string | null>(null);
  const [sessionPageWarnings, setSessionPageWarnings] = useState<string[]>([]);
  const [sessionPageStatus, setSessionPageStatus] = useState<"idle" | "loading" | "error">("idle");
  const [loadingMoreSessions, setLoadingMoreSessions] = useState(false);
  const [sessionPageReadFilter, setSessionPageReadFilter] = useState<SessionReadFilter>("readable");

  useEffect(() => {
    setShellSessions(initialSessions);
    setSessionPageOffset(initialSessions.length);
    setSessionPageHasMore(false);
    setSessionPageSource(null);
    setSessionPageWarnings([]);
    setSessionPageStatus("idle");
  }, [initialSessions]);

  const sessionPageArchiveOptions = useMemo(
    () => ({
      include_archived: false,
      archived_only: sessionPageReadFilter === "archived",
    }),
    [sessionPageReadFilter],
  );

  const loadSessionPage = useCallback(
    async (offset: number, mode: "replace" | "append", queryOverride?: string) => {
      setSessionPageStatus("loading");
      setLoadingMoreSessions(mode === "append");
      try {
        const page = await loadSessionPageFromSource({
          page_size: AGENT_SESSION_PAGE_SIZE,
          offset,
          query: (queryOverride ?? searchQuery).trim() || null,
          ...sessionPageArchiveOptions,
        });
        setShellSessions((current) => (mode === "append" ? [...current, ...page.sessions] : page.sessions));
        setSessionPageOffset(offset + page.sessions.length);
        setSessionPageHasMore(page.has_more);
        setSessionPageSource(page.source);
        setSessionPageWarnings(page.warnings);
        setSessionPageStatus("idle");
        return page;
      } catch (error) {
        if (mode === "replace") {
          setShellSessions(initialSessions);
          setSessionPageOffset(initialSessions.length);
        }
        setSessionPageHasMore(false);
        setSessionPageSource("snapshot_fallback");
        setSessionPageWarnings([messageOf(error)]);
        setSessionPageStatus("error");
        return null;
      } finally {
        setLoadingMoreSessions(false);
      }
    },
    [initialSessions, searchQuery, sessionPageArchiveOptions, loadSessionPageFromSource],
  );

  useEffect(() => {
    void loadSessionPage(0, "replace");
  }, [loadSessionPage]);

  return {
    shellSessions,
    sessionPageOffset,
    sessionPageHasMore,
    sessionPageSource,
    sessionPageWarnings,
    sessionPageStatus,
    loadingMoreSessions,
    loadSessionPage,
    setSessionPageReadFilter,
  };
}
