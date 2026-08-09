import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  loadAgentRoleSessionDetail,
  loadAgentRoleSessionDirectory,
  loadCodexSessionPage,
} from "../../lib/tauri";
import {
  createRoleSessionRequestNonce,
  mergeRoleSessionDirectoryPage,
  normalizeRoleSessionReadError,
  resolveRoleSessionDirectorySelection,
  roleSessionDirectoryPageHasCompatibleProjection,
  roleSessionDetailMatchesCurrentSelection,
  roleSessionDirectoryMatchesRequest,
  type RoleSessionDetail,
  type RoleSessionDirectory,
  type RoleSessionReadError,
} from "../../lib/roleSessionReadModel";
import type { CodexSessionPage, CodexSessionPageRequest, SessionRecord } from "../../lib/types";
import { messageOf } from "./agentLabels";
import type { SessionReadFilter } from "./AgentSessionList";

const AGENT_SESSION_PAGE_SIZE = 100;

export type AgentRoleSessionReadStatus =
  | "idle"
  | "loading"
  | "ready"
  | "empty"
  | "selection_required"
  | "error";

export type AgentRoleSessionReadState = Readonly<{
  status: AgentRoleSessionReadStatus;
  project_locator: string;
  directory: RoleSessionDirectory | null;
  detail: RoleSessionDetail | null;
  // This opaque value may only come from the currently loaded server
  // directory. It is never inferred from a legacy SessionRecord or history
  // ordering.
  selected_selection: string | null;
  loading_more: boolean;
  selection_error: string | null;
  error: RoleSessionReadError | null;
  // SessionRecord / Codex transcript data is compatibility reading material
  // only. It never turns into a continuation target in this hook.
  legacy_display_only: true;
}>;

export function useAgentSessionPage(
  initialSessions: SessionRecord[],
  searchQuery = "",
  loadSessionPageFromSource: (request: CodexSessionPageRequest) => Promise<CodexSessionPage> = loadCodexSessionPage,
  roleSessionProjectLocator = "",
) {
  const [shellSessions, setShellSessions] = useState<SessionRecord[]>(initialSessions);
  const [sessionPageOffset, setSessionPageOffset] = useState(0);
  const [sessionPageHasMore, setSessionPageHasMore] = useState(false);
  const [sessionPageSource, setSessionPageSource] = useState<string | null>(null);
  const [sessionPageWarnings, setSessionPageWarnings] = useState<string[]>([]);
  const [sessionPageStatus, setSessionPageStatus] = useState<"idle" | "loading" | "error">("idle");
  const [loadingMoreSessions, setLoadingMoreSessions] = useState(false);
  const [sessionPageReadFilter, setSessionPageReadFilter] = useState<SessionReadFilter>("readable");
  const [roleSessionRead, setRoleSessionRead] = useState<AgentRoleSessionReadState>({
    status: "idle",
    project_locator: "",
    directory: null,
    detail: null,
    selected_selection: null,
    loading_more: false,
    selection_error: null,
    error: null,
    legacy_display_only: true,
  });
  const roleSessionRequestEpochRef = useRef(0);
  const roleSessionReadRef = useRef(roleSessionRead);

  const setCurrentRoleSessionRead = useCallback((next: AgentRoleSessionReadState) => {
    roleSessionReadRef.current = next;
    setRoleSessionRead(next);
  }, []);

  const nextRoleSessionRequestEpoch = useCallback(() => {
    roleSessionRequestEpochRef.current += 1;
    return roleSessionRequestEpochRef.current;
  }, []);

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

  const requestRoleSessionDetail = useCallback(({
    project_locator,
    directory,
    selection,
  }: {
    project_locator: string;
    directory: RoleSessionDirectory;
    selection: string;
  }) => {
    const resolution = resolveRoleSessionDirectorySelection(directory, selection);
    if (resolution.status !== "explicit" || resolution.selection !== selection) {
      nextRoleSessionRequestEpoch();
      setCurrentRoleSessionRead({
        status: "selection_required",
        project_locator,
        directory,
        detail: null,
        selected_selection: null,
        loading_more: false,
        selection_error: "所选角色会话不在当前服务端目录中；请重新选择。",
        error: null,
        legacy_display_only: true,
      });
      return;
    }

    const epoch = nextRoleSessionRequestEpoch();
    const request = {
      project_locator,
      selection,
      request_nonce: createRoleSessionRequestNonce("agent-detail"),
    };
    setCurrentRoleSessionRead({
      status: "loading",
      project_locator,
      directory,
      detail: null,
      selected_selection: selection,
      loading_more: false,
      selection_error: null,
      error: null,
      legacy_display_only: true,
    });
    void (async () => {
      try {
        const detail = await loadAgentRoleSessionDetail(request);
        const current = roleSessionReadRef.current;
        if (
          roleSessionRequestEpochRef.current !== epoch
          || current.project_locator !== project_locator
        ) {
          return;
        }
        if (!roleSessionDetailMatchesCurrentSelection(detail, request, current.directory, current.selected_selection)) {
          setCurrentRoleSessionRead({
            ...current,
            status: "error",
            detail: null,
            selected_selection: null,
            loading_more: false,
            selection_error: "服务端角色会话详情与当前目录不一致；已清空当前选择。",
            error: {
              code: "M3_ROLE_SESSION_DETAIL_DRIFT",
              user_message: "服务端角色会话详情在读取期间发生变化；当前没有使用旧选择续聊。",
            },
          });
          return;
        }
        setCurrentRoleSessionRead({
          ...current,
          status: "ready",
          detail,
          loading_more: false,
          selection_error: null,
          error: null,
        });
      } catch (error) {
        const current = roleSessionReadRef.current;
        if (
          roleSessionRequestEpochRef.current !== epoch
          || current.project_locator !== project_locator
          || current.selected_selection !== selection
        ) {
          return;
        }
        setCurrentRoleSessionRead({
          ...current,
          status: "error",
          detail: null,
          loading_more: false,
          error: normalizeRoleSessionReadError(error),
        });
      }
    })();
  }, [nextRoleSessionRequestEpoch, setCurrentRoleSessionRead]);

  useEffect(() => {
    const project_locator = roleSessionProjectLocator.trim();
    const epoch = nextRoleSessionRequestEpoch();
    let disposed = false;
    if (!project_locator) {
      setCurrentRoleSessionRead({
        status: "empty",
        project_locator: "",
        directory: null,
        detail: null,
        selected_selection: null,
        loading_more: false,
        selection_error: null,
        error: null,
        legacy_display_only: true,
      });
      return () => {
        disposed = true;
        // The directory effect may have handed its work to a detail request,
        // which has a newer epoch. Unmounting or replacing this effect must
        // invalidate that hand-off too, otherwise a late detail can set state
        // after cleanup.
        nextRoleSessionRequestEpoch();
      };
    }

    setCurrentRoleSessionRead({
      status: "loading",
      project_locator,
      directory: null,
      detail: null,
      selected_selection: null,
      loading_more: false,
      selection_error: null,
      error: null,
      legacy_display_only: true,
    });
    const request = {
      project_locator,
      cursor: null,
      limit: 50,
      request_nonce: createRoleSessionRequestNonce("agent-directory"),
    };
    void (async () => {
      try {
        const directory = await loadAgentRoleSessionDirectory(request);
        const current = roleSessionReadRef.current;
        if (
          disposed
          || roleSessionRequestEpochRef.current !== epoch
          || current.project_locator !== project_locator
        ) {
          return;
        }
        if (!roleSessionDirectoryMatchesRequest(directory, request)) {
          setCurrentRoleSessionRead({
            status: "error",
            project_locator,
            directory: null,
            detail: null,
            selected_selection: null,
            loading_more: false,
            selection_error: "服务端角色会话目录回包与当前请求不一致；已关闭当前选择。",
            error: {
              code: "M3_ROLE_SESSION_DIRECTORY_NONCE_MISMATCH",
              user_message: "服务端角色会话目录回包已失效；当前没有使用旧选择续聊。",
            },
            legacy_display_only: true,
          });
          return;
        }
        const resolution = resolveRoleSessionDirectorySelection(directory);
        if (resolution.status === "empty") {
          setCurrentRoleSessionRead({
            status: "empty",
            project_locator,
            directory,
            detail: null,
            selected_selection: null,
            loading_more: false,
            selection_error: null,
            error: null,
            legacy_display_only: true,
          });
          return;
        }
        if (resolution.status !== "automatic" || !resolution.selection) {
          setCurrentRoleSessionRead({
            status: "selection_required",
            project_locator,
            directory,
            detail: null,
            selected_selection: null,
            loading_more: false,
            selection_error: null,
            error: null,
            legacy_display_only: true,
          });
          return;
        }
        requestRoleSessionDetail({ project_locator, directory, selection: resolution.selection });
      } catch (error) {
        if (disposed || roleSessionRequestEpochRef.current !== epoch) return;
        setCurrentRoleSessionRead({
          status: "error",
          project_locator,
          directory: null,
          detail: null,
          selected_selection: null,
          loading_more: false,
          selection_error: null,
          error: normalizeRoleSessionReadError(error),
          legacy_display_only: true,
        });
      }
    })();
    return () => {
      disposed = true;
      // See the empty-locator cleanup above: this must also invalidate a
      // descendant detail epoch, not only the directory request epoch.
      nextRoleSessionRequestEpoch();
    };
  }, [nextRoleSessionRequestEpoch, requestRoleSessionDetail, roleSessionProjectLocator, setCurrentRoleSessionRead]);

  const selectRoleSession = useCallback((selection: string) => {
    const current = roleSessionReadRef.current;
    if (!current.project_locator || !current.directory) {
      nextRoleSessionRequestEpoch();
      setCurrentRoleSessionRead({
        ...current,
        status: "selection_required",
        detail: null,
        selected_selection: null,
        loading_more: false,
        selection_error: "服务端角色会话目录尚未就绪；请稍后重新选择。",
        error: null,
      });
      return;
    }
    const resolution = resolveRoleSessionDirectorySelection(current.directory, selection);
    if (resolution.status !== "explicit" || !resolution.selection) {
      nextRoleSessionRequestEpoch();
      setCurrentRoleSessionRead({
        ...current,
        status: "selection_required",
        detail: null,
        selected_selection: null,
        loading_more: false,
        selection_error: "所选角色会话不在当前服务端目录中；请重新选择。",
        error: null,
      });
      return;
    }
    requestRoleSessionDetail({
      project_locator: current.project_locator,
      directory: current.directory,
      selection: resolution.selection,
    });
  }, [nextRoleSessionRequestEpoch, requestRoleSessionDetail, setCurrentRoleSessionRead]);

  const loadMoreRoleSessions = useCallback(() => {
    const current = roleSessionReadRef.current;
    const directory = current.directory;
    const project_locator = current.project_locator;
    const cursor = directory?.next_cursor;
    // A page request would invalidate the detail epoch. While a selected
    // detail is resolving, reject pagination rather than leaving that selected
    // opaque handle in a permanent loading state.
    if (!directory || !project_locator || !cursor || current.loading_more || current.status === "loading") return;

    const epoch = nextRoleSessionRequestEpoch();
    const request = {
      project_locator,
      cursor,
      limit: 50,
      request_nonce: createRoleSessionRequestNonce("agent-directory-more"),
    };
    setCurrentRoleSessionRead({ ...current, loading_more: true, selection_error: null });
    void (async () => {
      try {
        const page = await loadAgentRoleSessionDirectory(request);
        const latest = roleSessionReadRef.current;
        if (
          roleSessionRequestEpochRef.current !== epoch
          || latest.project_locator !== project_locator
        ) {
          return;
        }
        if (!roleSessionDirectoryMatchesRequest(page, request)) {
          setCurrentRoleSessionRead({
            ...latest,
            status: "error",
            directory: null,
            detail: null,
            selected_selection: null,
            loading_more: false,
            selection_error: "服务端角色会话目录分页回包与当前请求不一致；已关闭当前选择。",
            error: {
              code: "M3_ROLE_SESSION_DIRECTORY_NONCE_MISMATCH",
              user_message: "服务端角色会话目录分页回包已失效；当前没有使用旧选择续聊。",
            },
          });
          return;
        }
        const currentDetail = latest.detail?.selection === latest.selected_selection ? latest.detail : null;
        if (!roleSessionDirectoryPageHasCompatibleProjection(directory, page, currentDetail)) {
          setCurrentRoleSessionRead({
            ...latest,
            status: "error",
            directory: null,
            detail: null,
            selected_selection: null,
            loading_more: false,
            selection_error: "服务端目录分页投影发生变化；已清空当前选择，请重新读取。",
            error: {
              code: "M3_DIRECTORY_SNAPSHOT_DRIFT",
              user_message: "服务端角色会话目录在分页期间发生变化；当前没有使用旧选择续聊。",
            },
          });
          return;
        }
        const merged = mergeRoleSessionDirectoryPage(directory, page);
        const resolution = resolveRoleSessionDirectorySelection(merged, latest.selected_selection);
        if (!latest.selected_selection && resolution.status === "automatic" && resolution.selection) {
          requestRoleSessionDetail({ project_locator, directory: merged, selection: resolution.selection });
          return;
        }
        const nextStatus: AgentRoleSessionReadStatus = latest.selected_selection
          ? latest.status
          : resolution.status === "empty"
            ? "empty"
            : "selection_required";
        setCurrentRoleSessionRead({
          ...latest,
          status: nextStatus,
          directory: merged,
          detail: latest.detail?.selection === latest.selected_selection ? latest.detail : null,
          loading_more: false,
          selection_error: null,
        });
      } catch {
        const latest = roleSessionReadRef.current;
        if (roleSessionRequestEpochRef.current !== epoch || latest.project_locator !== project_locator) return;
        setCurrentRoleSessionRead({
          ...latest,
          loading_more: false,
          selection_error: "加载更多服务端角色会话失败；没有使用本地缓存替代目录。",
        });
      }
    })();
  }, [nextRoleSessionRequestEpoch, requestRoleSessionDetail, setCurrentRoleSessionRead]);

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
    roleSessionRead,
    selectRoleSession,
    loadMoreRoleSessions,
  };
}
