import { useCallback, useEffect, useRef, useState, type ReactNode } from "react";
import { Pill, EmptyState } from "../components/SpecPrimitives";
import { DetailLine } from "../components/WorkbenchPrimitives";
import { deriveKnowledgeBaseSummary, type KnowledgeDocumentReadModel, type KnowledgeMemoryLink } from "../lib/knowledgeBase";
import { deriveKnowledgeBasePageReadModelFromParts } from "../lib/pageSelectors";
import { parseMarkdown, type MdBlock, type MdInline } from "../lib/knowledgeVault";
import {
  knowledgeWorkspaceAsyncCommitDisposition,
  knowledgeWorkspaceDraftNavigationDisposition,
  knowledgeWorkspaceDraftRefreshDisposition,
} from "../lib/knowledgeWorkspace";
import { NativeKnowledgeWorkspace } from "./knowledge/NativeKnowledgeWorkspace";
import {
  humanizeObsidianIntegrationError,
  obsidianIntegration,
  obsidianStatusLabel,
  type ObsidianIntegrationActionReceipt,
  type ObsidianIntegrationStatusSnapshot,
} from "../lib/obsidianIntegration";
import {
  knowledgeVaultCreateNote,
  knowledgeVaultListNotes,
  knowledgeVaultReadNote,
  knowledgeVaultWriteNote,
  type KnowledgeVaultNote,
  type KnowledgeVaultNoteSummary,
} from "../lib/tauri";
import type { KnowledgeOpenRelayIntent, KnowledgeOpenRelayOutcome } from "../lib/knowledgeOpenRelay";
import type {
  FormalMemoryStoreV1,
  MemoryCaptureStoreV1,
  MemoryCandidateStoreV1,
  PendingAction,
  ProjectRecord,
  WorkflowStateSnapshot,
} from "../lib/types";

type KnowledgeVaultEditorAsyncRequest = Readonly<{
  draftRevision: number;
  generation: number;
  currentRelativePath: string | null;
}>;

export function KnowledgeBaseView({
  projects,
  workflowState,
  formalMemoryStore,
  memoryCaptureStore,
  memoryCandidateStore,
  hasRealSnapshot,
  onRequestAction,
  knowledgeOpenIntent = null,
  onKnowledgeOpenIntentOutcome,
}: {
  projects: ProjectRecord[];
  workflowState: WorkflowStateSnapshot | null;
  formalMemoryStore?: FormalMemoryStoreV1 | null;
  memoryCaptureStore?: MemoryCaptureStoreV1 | null;
  memoryCandidateStore?: MemoryCandidateStoreV1 | null;
  hasRealSnapshot: boolean;
  onRequestAction: (action: PendingAction) => void;
  knowledgeOpenIntent?: KnowledgeOpenRelayIntent | null;
  onKnowledgeOpenIntentOutcome?: (intent: KnowledgeOpenRelayIntent, outcome: KnowledgeOpenRelayOutcome) => Promise<boolean>;
}) {
  const summary = deriveKnowledgeBaseSummary({
    projects,
    workflowState,
    formalMemoryStore,
    memoryCaptureStore,
    memoryCandidateStore,
  });
  const pageReadModel = deriveKnowledgeBasePageReadModelFromParts({ summary, hasRealSnapshot });

  return (
    <SynNativeKnowledgeWorkspaceArea
      knowledgeOpenIntent={knowledgeOpenIntent}
      onKnowledgeOpenIntentOutcome={onKnowledgeOpenIntentOutcome}
      sourceSidebar={<KnowledgeSourceSidebar summary={summary} onRequestAction={onRequestAction} />}
      sourceContext={(selectedRelativePath) => (
        <KnowledgeSourceContext
          summary={summary}
          onRequestAction={onRequestAction}
          selectedRelativePath={selectedRelativePath}
        />
      )}
      statusContent={
        <>
          <span>{pageReadModel.snapshot_status_label}</span>
          <span>{pageReadModel.boundary_text}</span>
          {summary.warnings.slice(0, 2).map((warning) => <span className="state-warning" key={warning}>{warning}</span>)}
        </>
      }
    />
  );
}

function SynNativeKnowledgeWorkspaceArea({
  knowledgeOpenIntent,
  onKnowledgeOpenIntentOutcome,
  sourceSidebar,
  sourceContext,
  statusContent,
}: {
  knowledgeOpenIntent: KnowledgeOpenRelayIntent | null;
  onKnowledgeOpenIntentOutcome?: (intent: KnowledgeOpenRelayIntent, outcome: KnowledgeOpenRelayOutcome) => Promise<boolean>;
  sourceSidebar: ReactNode;
  sourceContext: (selectedRelativePath: string | null) => ReactNode;
  statusContent: ReactNode;
}) {
  return (
    <NativeKnowledgeWorkspace
      knowledgeOpenIntent={knowledgeOpenIntent}
      onKnowledgeOpenIntentOutcome={onKnowledgeOpenIntentOutcome}
      sourceSidebar={sourceSidebar}
      sourceContext={sourceContext}
      statusContent={statusContent}
    />
  );
}

function KnowledgeSourceSidebar({
  summary,
  onRequestAction,
}: {
  summary: ReturnType<typeof deriveKnowledgeBaseSummary>;
  onRequestAction: (action: PendingAction) => void;
}) {
  return (
    <section className="syn-knowledge-source-panel" aria-label="知识来源">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">来源</p>
          <h3>项目文档 / 来源锚点</h3>
        </div>
        <Pill tone="plain">{summary.documents.length}</Pill>
      </div>
      <div className="workflow-compact-list">
        {summary.documents.map((document) => (
          <div className="knowledge-document-item" key={document.document_key}>
            <span>{document.title}</span>
            <strong>{document.project_name}</strong>
            <small>
              关联正式记忆 {document.formal_memory_links.length} / 关联候选 {document.candidate_links.length} / {document.task_reference_summary.display_text}
            </small>
          </div>
        ))}
        {!summary.documents.length ? <p className="muted small-note">暂无权威文件可作为知识库资料；不会伪造知识库索引。</p> : null}
      </div>
      {summary.documents.length ? (
        <div className="workflow-compact-list" aria-label="来源候选动作">
          {summary.documents.map((document) => (
            <KnowledgeDocumentDetail document={document} onRequestAction={onRequestAction} key={`source-${document.document_key}`} />
          ))}
        </div>
      ) : null}
      <section className="syn-knowledge-source-panel" aria-label="记忆捕获来源">
        <div className="panel-heading compact">
          <div>
            <p className="eyebrow">记忆捕获</p>
            <h3>事件 / 观察 / 候选来源</h3>
          </div>
          <Pill tone={summary.capture_event_count ? "candidate" : "unknown"}>{summary.capture_event_count}</Pill>
        </div>
        <div className="workflow-compact-list">
          {summary.recent_capture_events.map((event) => (
            <div className="workflow-compact-item" key={`${event.label}-${event.created_at}-${event.summary}`}>
              <strong>{event.label}</strong>
              <span>{event.summary}</span>
              <em>{event.policy_label} · {event.boundary}</em>
            </div>
          ))}
          {!summary.recent_capture_events.length ? <p className="muted small-note">暂无记忆捕获事件；知识库不会伪造候选来源。</p> : null}
        </div>
      </section>
      <ObsidianIntegrationPanel />
    </section>
  );
}

function KnowledgeSourceContext({
  summary,
  onRequestAction,
  selectedRelativePath,
}: {
  summary: ReturnType<typeof deriveKnowledgeBaseSummary>;
  onRequestAction: (action: PendingAction) => void;
  selectedRelativePath: string | null;
}) {
  const selectedDocument = selectedRelativePath
    ? summary.documents.find((document) => knowledgeDocumentMatchesRelativePath(document, selectedRelativePath)) ?? null
    : null;
  return (
    <section className="syn-knowledge-source-panel" aria-label="来源上下文">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">来源上下文</p>
          <h3>当前笔记的来源 / 反向引用</h3>
        </div>
        <Pill tone={selectedDocument ? "candidate" : "unknown"}>{selectedDocument ? "当前笔记" : "未映射"}</Pill>
      </div>
      {selectedDocument ? (
        <KnowledgeDocumentDetail document={selectedDocument} onRequestAction={onRequestAction} />
      ) : (
        <p className="muted small-note">当前笔记没有可映射的来源上下文；可在左侧“来源”查看项目资料与候选入口。</p>
      )}
    </section>
  );
}

export function knowledgeDocumentMatchesRelativePath(document: KnowledgeDocumentReadModel, relativePath: string): boolean {
  const sourcePath = document.source_anchor.path_summary;
  return sourcePath === relativePath
    || sourcePath.endsWith(`/${relativePath}`)
    || relativePath.endsWith(`/${sourcePath}`);
}

function KnowledgeDocumentDetail({
  document,
  onRequestAction,
}: {
  document: KnowledgeDocumentReadModel;
  onRequestAction: (action: PendingAction) => void;
}) {
  return (
    <div className="knowledge-document-detail">
      <div className="workflow-draft-grid knowledge-detail-grid">
        <DetailLine label="项目归属" value={document.project_name} />
        <DetailLine label="来源锚点" value={document.source_anchor.anchor_label} />
        <DetailLine label="关联正式记忆" value={`${document.formal_memory_links.length}`} />
        <DetailLine label="关联候选" value={`${document.candidate_links.length}`} />
        <DetailLine label="任务包知识引用" value={`${document.task_reference_summary.reference_count}`} />
        <DetailLine label="边界" value={document.boundary} />
      </div>

      <details className="agent-boundary-details">
        <summary className="agent-boundary-summary">开发者详情</summary>
        <div className="workflow-draft-grid knowledge-detail-grid">
          <DetailLine label="来源类型" value={document.source_anchor.source_kind} />
          <DetailLine label="路径摘要" value={document.source_anchor.path_summary} />
        </div>
      </details>

      <div className="knowledge-action-row">
        <button type="button" className="primary-button" onClick={() => onRequestAction(buildKnowledgeCandidateAction(document))}>
          {document.candidate_draft.label}
        </button>
        <span>{document.candidate_draft.boundary}</span>
      </div>

      <section className="knowledge-links-section" aria-label="知识库反向引用">
        <div className="panel-heading compact">
          <div>
            <p className="eyebrow">反向引用</p>
            <h3>正式记忆 / 候选 / 任务包</h3>
          </div>
        </div>
        <div className="workflow-compact-list">
          {document.formal_memory_links.map((link, index) => (
            <KnowledgeLinkItem link={link} key={`formal-${index}-${link.claim}`} />
          ))}
          {document.candidate_links.map((link, index) => (
            <KnowledgeLinkItem link={link} key={`candidate-${index}-${link.claim}`} />
          ))}
          <div className="workflow-compact-item">
            <strong>{document.task_reference_summary.display_text}</strong>
            <span>{document.task_reference_summary.task_goals.join("；") || "暂无任务包知识引用"}</span>
            <em>任务包引用只是可用资料摘要，不代表资料已经进入正式记忆。</em>
          </div>
        </div>
      </section>
    </div>
  );
}

function KnowledgeLinkItem({ link }: { link: KnowledgeMemoryLink }) {
  return (
    <div className={`workflow-compact-item knowledge-link-item ${link.kind}`}>
      <strong>{link.label}</strong>
      <span>{link.claim}</span>
      <em>{link.boundary}</em>
      <details className="agent-boundary-details">
        <summary className="agent-boundary-summary">开发者详情</summary>
        <em>{link.status}</em>
      </details>
    </div>
  );
}

function buildKnowledgeCandidateAction(document: KnowledgeDocumentReadModel): PendingAction {
  return {
    kind: "create-memory-candidate",
    label: document.candidate_draft.label,
    path: "memory-candidates.v1.json",
    source: "Tauri 应用数据目录",
    boundary: "只会在你确认后写入 memory-candidates.v1.json；只生成候选，不写正式记忆；知识库材料仍需经候选与用户确认。",
    memoryCandidateCreation: document.candidate_draft.input,
  };
}

function StatCell({ label, value, helper }: { label: string; value: string; helper: string }) {
  return (
    <div className="stat-cell">
      <div className="lbl">{label}</div>
      <div className="val mono">{value}</div>
      <div className="memory-stat-helper">{helper}</div>
    </div>
  );
}

// ── L3 第二片：官方 Obsidian 只作为开放格式的可选外部打开。Syn 原生工作区不依赖
// 安装状态；展开兼容入口后才调用受限 bridge。 ──

const KNOWLEDGE_VAULT_SAVED_EVENT = "syn-knowledge-vault-saved";

export function subscribeToKnowledgeVaultSavedEvent(
  target: Pick<EventTarget, "addEventListener" | "removeEventListener">,
  onWorkspaceMutation: () => void,
): () => void {
  target.addEventListener(KNOWLEDGE_VAULT_SAVED_EVENT, onWorkspaceMutation);
  return () => target.removeEventListener(KNOWLEDGE_VAULT_SAVED_EVENT, onWorkspaceMutation);
}

export type ObsidianIntegrationCommands = {
  status: () => Promise<ObsidianIntegrationStatusSnapshot>;
  openVault: () => Promise<ObsidianIntegrationActionReceipt>;
  openNote: (slug: string) => Promise<ObsidianIntegrationActionReceipt>;
  openSearch: (query: string) => Promise<ObsidianIntegrationActionReceipt>;
};

const defaultObsidianIntegrationCommands: ObsidianIntegrationCommands = {
  status: obsidianIntegration.status,
  openVault: obsidianIntegration.openVault,
  openNote: obsidianIntegration.openNote,
  openSearch: obsidianIntegration.openSearch,
};

type ObsidianIntegrationLoadState = "loading" | "ready" | "unavailable";
type ObsidianIntegrationBusyAction = "open_vault" | "open_note" | "open_search" | null;

export function ObsidianIntegrationPanel({
  commands = defaultObsidianIntegrationCommands,
  selectedNote = null,
}: {
  commands?: ObsidianIntegrationCommands;
  selectedNote?: Pick<KnowledgeVaultNote, "slug" | "title"> | null;
}) {
  // Offline evidence renders components directly outside a browser. Keep that
  // path hook-free: the optional bridge is not probed until a user expands it.
  if (typeof window === "undefined") {
    return <ObsidianCompatibilityDisclosure />;
  }
  return <ObsidianIntegrationPanelInner commands={commands} selectedNote={selectedNote} />;
}

function ObsidianCompatibilityDisclosure({
  children,
  onToggle,
}: {
  children?: React.ReactNode;
  onToggle?: (expanded: boolean) => void;
}) {
  return (
    <details
      className="knowledge-compatibility-panel"
      aria-label="Markdown 和 Obsidian 兼容"
      onToggle={onToggle ? (event) => onToggle(event.currentTarget.open) : undefined}
    >
      <summary className="knowledge-compatibility-summary">
        <span>可选兼容与外部打开</span>
        <span>Syn 原生知识工作区无需安装 Obsidian</span>
      </summary>
      {children ?? (
        <p className="muted small-note knowledge-compatibility-copy">
          Markdown / JSON Canvas 文件兼容；展开后可查看官方 Obsidian 的可选外部打开状态。
        </p>
      )}
    </details>
  );
}

function ObsidianIntegrationPanelInner({
  commands,
  selectedNote,
}: {
  commands: ObsidianIntegrationCommands;
  selectedNote: Pick<KnowledgeVaultNote, "slug" | "title"> | null;
}) {
  const [expanded, setExpanded] = useState(false);

  return (
    <ObsidianCompatibilityDisclosure onToggle={setExpanded}>
      {expanded ? <ObsidianIntegrationPanelContents commands={commands} selectedNote={selectedNote} /> : null}
    </ObsidianCompatibilityDisclosure>
  );
}

function ObsidianIntegrationPanelContents({
  commands,
  selectedNote,
}: {
  commands: ObsidianIntegrationCommands;
  selectedNote: Pick<KnowledgeVaultNote, "slug" | "title"> | null;
}) {
  const [loadState, setLoadState] = useState<ObsidianIntegrationLoadState>("loading");
  const [status, setStatus] = useState<ObsidianIntegrationStatusSnapshot | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [busyAction, setBusyAction] = useState<ObsidianIntegrationBusyAction>(null);

  const refresh = useCallback(async () => {
    try {
      const nextStatus = await commands.status();
      setStatus(nextStatus);
      setLoadState("ready");
    } catch {
      setStatus(null);
      setLoadState("unavailable");
    }
  }, [commands]);

  useEffect(() => {
    void refresh();
    const refreshAfterFocus = () => void refresh();
    const refreshAfterVaultSave = () => void refresh();
    window.addEventListener("focus", refreshAfterFocus);
    window.addEventListener(KNOWLEDGE_VAULT_SAVED_EVENT, refreshAfterVaultSave);
    return () => {
      window.removeEventListener("focus", refreshAfterFocus);
      window.removeEventListener(KNOWLEDGE_VAULT_SAVED_EVENT, refreshAfterVaultSave);
    };
  }, [refresh]);

  const runAction = useCallback(
    async (
      action: Exclude<ObsidianIntegrationBusyAction, null>,
      operation: () => Promise<ObsidianIntegrationActionReceipt>,
    ) => {
      setBusyAction(action);
      setNotice(null);
      try {
        const receipt = await operation();
        setNotice(receipt.message);
        await refresh();
      } catch (error) {
        setNotice(humanizeObsidianIntegrationError(error));
      } finally {
        setBusyAction(null);
      }
    },
    [refresh],
  );

  return (
    <ObsidianIntegrationView
      loadState={loadState}
      status={status}
      selectedNote={selectedNote}
      notice={notice}
      searchQuery={searchQuery}
      busyAction={busyAction}
      onRefresh={() => void refresh()}
      onOpenVault={() => void runAction("open_vault", commands.openVault)}
      onOpenNote={() => {
        if (selectedNote) void runAction("open_note", () => commands.openNote(selectedNote.slug));
      }}
      onSearchQueryChange={setSearchQuery}
      onOpenSearch={() => {
        const query = searchQuery.trim();
        if (query) void runAction("open_search", () => commands.openSearch(query));
      }}
    />
  );
}

export function ObsidianIntegrationView({
  loadState,
  status,
  selectedNote,
  notice,
  searchQuery,
  busyAction,
  onRefresh,
  onOpenVault,
  onOpenNote,
  onSearchQueryChange,
  onOpenSearch,
}: {
  loadState: ObsidianIntegrationLoadState;
  status: ObsidianIntegrationStatusSnapshot | null;
  selectedNote: Pick<KnowledgeVaultNote, "slug" | "title"> | null;
  notice: string | null;
  searchQuery: string;
  busyAction: ObsidianIntegrationBusyAction;
  onRefresh: () => void;
  onOpenVault: () => void;
  onOpenNote: () => void;
  onSearchQueryChange: (value: string) => void;
  onOpenSearch: () => void;
}) {
  const connectionReady = loadState === "ready" && status?.status === "ready";
  const busy = busyAction !== null;
  const statusLabel =
    loadState === "loading"
      ? "正在检测官方 Obsidian 状态"
      : loadState === "unavailable"
        ? "暂时无法读取 Obsidian 状态"
        : obsidianStatusLabel(status?.status ?? "installed");
  const statusMessage =
    status?.message ??
    (loadState === "unavailable"
      ? "连接状态暂时读不到；Syn 原生知识工作区仍可浏览和编辑。"
      : "Syn 原生知识工作区使用开放 Markdown / JSON Canvas 文件；外部打开是可选兼容。 ");

  return (
    <section className="knowledge-base-panel knowledge-compatibility-status" aria-label="Obsidian 兼容信息">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">可选兼容</p>
          <h3>官方 Obsidian 外部打开</h3>
        </div>
        <Pill tone={obsidianStatusTone(loadState, status)}>{connectionReady ? "可用" : statusLabel}</Pill>
      </div>
      <div className="workflow-compact-list">
        <div className="workflow-compact-item">
          <strong>{statusLabel}</strong>
          <span>{statusMessage}</span>
          <em>不影响 Syn 内的 Markdown 阅读、编辑和链接。</em>
        </div>
        <div className="obsidian-integration-details">
          <div>
            <strong>文件兼容</strong>
            <span>Markdown / JSON Canvas</span>
          </div>
          <div>
            <strong>共享资料</strong>
            <span>{status?.vault_label ?? "Syn 自管 Markdown vault"}</span>
          </div>
        </div>
      </div>
      <div className="action-row knowledge-obsidian-actions">
        <button className="secondary-button" type="button" onClick={onRefresh} disabled={busy}>
          刷新连接状态
        </button>
        <button className="secondary-button" type="button" onClick={onOpenVault} disabled={!connectionReady || busy}>
          {busyAction === "open_vault" ? "正在打开…" : "在官方 Obsidian 中打开 Syn vault"}
        </button>
        {selectedNote ? (
          <button className="secondary-button" type="button" onClick={onOpenNote} disabled={!connectionReady || busy}>
            {busyAction === "open_note" ? "正在打开…" : `在 Obsidian 中打开《${selectedNote.title}》`}
          </button>
        ) : null}
      </div>
      <div className="knowledge-obsidian-search">
        <label htmlFor="knowledge-obsidian-search">在 Obsidian 中搜索</label>
        <input
          id="knowledge-obsidian-search"
          aria-label="在 Obsidian 中搜索"
          value={searchQuery}
          placeholder="输入固定 vault 内的搜索词"
          onChange={(event) => onSearchQueryChange(event.target.value)}
        />
        <button
          className="secondary-button"
          type="button"
          onClick={onOpenSearch}
          disabled={!connectionReady || busy || !searchQuery.trim()}
        >
          {busyAction === "open_search" ? "正在打开…" : "在 Obsidian 中搜索"}
        </button>
      </div>
      {notice ? <p className="state-warning knowledge-obsidian-notice">{notice}</p> : null}
      <div className="workflow-compact-item knowledge-memory-boundary">
        <strong>知识库和正式记忆</strong>
        <span>知识库是材料和笔记空间；正式记忆是经过确认、来源、版本、审计和权限治理的行为上下文。</span>
        <em>知识命中、资料摘要和 Markdown 来源不能绕过候选流程。</em>
      </div>
    </section>
  );
}

function obsidianStatusTone(
  loadState: ObsidianIntegrationLoadState,
  status: ObsidianIntegrationStatusSnapshot | null,
): "plain" | "ok" | "warn" | "unknown" {
  if (loadState === "loading") return "plain";
  if (loadState === "unavailable") return "unknown";
  return status?.status === "ready" ? "ok" : "warn";
}

// ── L3 知识库第一片：vault 笔记区（工作台自管目录·用户手编为主·AI 写入只走弹窗那一下） ──

export type KnowledgeVaultCommands = {
  listNotes: () => Promise<KnowledgeVaultNoteSummary[]>;
  readNote: (slug: string) => Promise<KnowledgeVaultNote>;
  createNote: (title: string) => Promise<{ slug: string; title: string }>;
  writeNote: (slug: string, body: string, expectedMtimeMs: number, expectedContentHash: string) => Promise<unknown>;
};

const defaultVaultCommands: KnowledgeVaultCommands = {
  listNotes: knowledgeVaultListNotes,
  readNote: knowledgeVaultReadNote,
  createNote: knowledgeVaultCreateNote,
  writeNote: knowledgeVaultWriteNote,
};

// 容器（有 hooks·数据读写）。离线/SSR（无 window）不挂 hooks，渲染 loading 静态面——
// 离线断言走下方零 hooks 的 KnowledgeVaultNotesView 本体（同 ProjectDetail 守卫先例）。
export function KnowledgeVaultNotesPanel({ commands = defaultVaultCommands }: { commands?: KnowledgeVaultCommands }) {
  if (typeof window === "undefined") {
    return (
      <>
        <section className="knowledge-base-panel knowledge-vault-notes" aria-label="知识库笔记">
          <div className="panel-heading">
            <div>
              <p className="eyebrow">笔记</p>
              <h3>知识库笔记</h3>
            </div>
          </div>
          <p className="muted small-note">正在读取笔记…</p>
        </section>
        <ObsidianIntegrationPanel />
      </>
    );
  }
  return <KnowledgeVaultNotesPanelInner commands={commands} />;
}

function KnowledgeVaultNotesPanelInner({ commands }: { commands: KnowledgeVaultCommands }) {
  const draftRevisionRef = useRef(0);
  const editorGenerationRef = useRef(0);
  const selectedSlugRef = useRef<string | null>(null);
  const [loadState, setLoadState] = useState<"loading" | "ready" | "unavailable">("loading");
  const [notes, setNotes] = useState<KnowledgeVaultNoteSummary[]>([]);
  const [selected, setSelected] = useState<KnowledgeVaultNote | null>(null);
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState("");
  const [newTitle, setNewTitle] = useState<string | null>(null);
  const [pendingLinkTitle, setPendingLinkTitle] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [needsReload, setNeedsReload] = useState(false);

  const replaceDraft = useCallback((nextDraft: string) => {
    draftRevisionRef.current += 1;
    setDraft(nextDraft);
  }, []);
  const replaceSelectedNote = useCallback((note: KnowledgeVaultNote) => {
    selectedSlugRef.current = note.slug;
    setSelected(note);
  }, []);
  const beginEditorAsyncRequest = useCallback((): KnowledgeVaultEditorAsyncRequest => {
    const generation = editorGenerationRef.current + 1;
    editorGenerationRef.current = generation;
    return {
      draftRevision: draftRevisionRef.current,
      generation,
      currentRelativePath: selectedSlugRef.current,
    };
  }, []);
  const canCommitEditorAsyncRequest = useCallback((request: KnowledgeVaultEditorAsyncRequest): boolean => (
    knowledgeWorkspaceAsyncCommitDisposition({
      requestDraftRevision: request.draftRevision,
      currentDraftRevision: draftRevisionRef.current,
      requestGeneration: request.generation,
      currentGeneration: editorGenerationRef.current,
      requestCurrentRelativePath: request.currentRelativePath,
      currentRelativePath: selectedSlugRef.current,
    }) === "apply"
  ), []);

  const reload = useCallback(async () => {
    try {
      const list = await commands.listNotes();
      setNotes(list);
      setLoadState("ready");
    } catch {
      setLoadState("unavailable");
    }
  }, [commands]);
  useEffect(() => {
    void reload();
  }, [reload]);

  const openNote = useCallback(
    async (slug: string, { discardDraft = false }: Readonly<{ discardDraft?: boolean }> = {}): Promise<boolean> => {
      const draftIsProtected = selected !== null && (needsReload || (editing && draft !== selected.body));
      if (
        !discardDraft
        && knowledgeWorkspaceDraftNavigationDisposition({ draftIsDirty: draftIsProtected }) === "preserve"
      ) {
        setNotice("当前笔记草稿尚未保存；请先保存或取消编辑后再切换笔记。");
        return false;
      }
      const request = beginEditorAsyncRequest();
      try {
        const note = await commands.readNote(slug);
        if (!canCommitEditorAsyncRequest(request)) {
          setNotice("读取期间本地草稿或当前笔记已变化；已保留，不会回填迟到内容。");
          return false;
        }
        replaceSelectedNote(note);
        setEditing(false);
        replaceDraft(note.body);
        setNeedsReload(false);
        setNotice(null);
        return true;
      } catch {
        setNotice("这条笔记没读到。");
        return false;
      }
    },
    [beginEditorAsyncRequest, canCommitEditorAsyncRequest, commands, draft, editing, needsReload, replaceDraft, replaceSelectedNote, selected],
  );

  const createAndOpen = useCallback(
    async (title: string) => {
      const draftIsProtected = selected !== null && (needsReload || (editing && draft !== selected.body));
      if (knowledgeWorkspaceDraftNavigationDisposition({ draftIsDirty: draftIsProtected }) === "preserve") {
        setNotice("当前笔记草稿尚未保存；请先保存或取消编辑后再新建并打开笔记。");
        return;
      }
      const request = beginEditorAsyncRequest();
      try {
        const created = await commands.createNote(title);
        window.dispatchEvent(new Event(KNOWLEDGE_VAULT_SAVED_EVENT));
        if (!canCommitEditorAsyncRequest(request)) {
          setNotice("新建已写入，但期间本地草稿或当前笔记已变化；未回填旧读取结果。");
          return;
        }
        await reload();
        if (!canCommitEditorAsyncRequest(request)) {
          setNotice("新建后的读取结果已过期；已保留当前草稿，不会回填旧内容。");
          return;
        }
        const note = await commands.readNote(created.slug);
        if (!canCommitEditorAsyncRequest(request)) {
          setNotice("新建后的读取结果已过期；已保留当前草稿，不会回填旧内容。");
          return;
        }
        replaceSelectedNote(note);
        replaceDraft(note.body);
        setEditing(true);
        setNewTitle(null);
        setPendingLinkTitle(null);
        setNeedsReload(false);
        setNotice(null);
      } catch {
        setNotice("新建没成功。");
      }
    },
    [beginEditorAsyncRequest, canCommitEditorAsyncRequest, commands, draft, editing, needsReload, reload, replaceDraft, replaceSelectedNote, selected],
  );

  const openLink = useCallback(
    (title: string) => {
      const hit = notes.find((note) => note.title.trim().toLowerCase() === title.trim().toLowerCase());
      if (hit) {
        setPendingLinkTitle(null);
        void openNote(hit.slug);
      } else {
        setPendingLinkTitle(title);
      }
    },
    [notes, openNote],
  );

  const refreshVault = useCallback(async () => {
    const request = beginEditorAsyncRequest();
    await reload();
    if (!canCommitEditorAsyncRequest(request)) {
      setNotice("刷新期间本地草稿或当前笔记已变化；已保留，不会回填迟到内容。");
      return;
    }
    if (!selected) return;
    try {
      const note = await commands.readNote(selected.slug);
      if (!canCommitEditorAsyncRequest(request)) {
        setNotice("刷新期间本地草稿或当前笔记已变化；已保留，不会回填迟到内容。");
        return;
      }
      const externallyChanged =
        note.mtime_ms !== selected.mtime_ms || note.content_hash !== selected.content_hash;
      const disposition = knowledgeWorkspaceDraftRefreshDisposition({
        draftIsDirty: needsReload || (editing && draft !== selected.body),
        externallyChanged,
      });
      if (disposition === "conflict") {
        setNeedsReload(true);
        setNotice("笔记已在另一窗口或外部程序修改，请先重新读取后再保存。");
        return;
      }
      if (disposition === "preserve") {
        setNotice(needsReload ? "笔记已在另一窗口或外部程序修改，请先重新读取后再保存。" : "笔记目录已刷新；当前草稿尚未保存，已保留。");
        return;
      }
      replaceSelectedNote(note);
      replaceDraft(note.body);
      setNeedsReload(false);
    } catch {
      setNotice("笔记刷新没完成；当前内容没有被覆盖。");
    }
  }, [beginEditorAsyncRequest, canCommitEditorAsyncRequest, commands, draft, editing, needsReload, reload, replaceDraft, replaceSelectedNote, selected]);

  useEffect(() => {
    const refreshAfterFocus = () => void refreshVault();
    window.addEventListener("focus", refreshAfterFocus);
    return () => window.removeEventListener("focus", refreshAfterFocus);
  }, [refreshVault]);

  const reloadSelectedAfterConflict = useCallback(() => {
    if (!selected) return;
    void openNote(selected.slug, { discardDraft: true }).then((reloaded) => {
      if (reloaded) setNotice("已重新读取当前笔记。");
    });
  }, [openNote, selected]);

  const saveEdit = useCallback(async () => {
    if (!selected || needsReload) return;
    const request = beginEditorAsyncRequest();
    try {
      await commands.writeNote(selected.slug, draft, selected.mtime_ms, selected.content_hash);
    } catch (error) {
      if (isKnowledgeVaultWriteConflict(error)) {
        setNeedsReload(true);
        setNotice("笔记已在另一窗口或外部程序修改，请先重新读取后再保存。");
      } else {
        setNotice("保存没成功；当前笔记没有被静默覆盖。");
      }
      return;
    }
    window.dispatchEvent(new Event(KNOWLEDGE_VAULT_SAVED_EVENT));
    if (!canCommitEditorAsyncRequest(request)) {
      if (
        request.currentRelativePath === selectedSlugRef.current
        && request.draftRevision !== draftRevisionRef.current
      ) {
        setNeedsReload(true);
      }
      setNotice("保存已提交，但期间本地草稿或当前笔记已变化；已保留当前草稿，请重新读取后再保存。");
      return;
    }

    try {
      await reload();
      if (!canCommitEditorAsyncRequest(request)) {
        if (
          request.currentRelativePath === selectedSlugRef.current
          && request.draftRevision !== draftRevisionRef.current
        ) {
          setNeedsReload(true);
        }
        setNotice("保存后的读取结果已过期；已保留当前草稿，请重新读取后再保存。");
        return;
      }
      const note = await commands.readNote(selected.slug);
      if (!canCommitEditorAsyncRequest(request)) {
        if (
          request.currentRelativePath === selectedSlugRef.current
          && request.draftRevision !== draftRevisionRef.current
        ) {
          setNeedsReload(true);
        }
        setNotice("保存后的读取结果已过期；已保留当前草稿，请重新读取后再保存。");
        return;
      }
      replaceSelectedNote(note);
      replaceDraft(note.body);
      setEditing(false);
      setNeedsReload(false);
      setNotice("已保存并重新读取。");
    } catch {
      setEditing(false);
      setNeedsReload(true);
      setNotice("已保存，但重新读取暂时没完成；请先刷新笔记再继续编辑。");
    }
  }, [beginEditorAsyncRequest, canCommitEditorAsyncRequest, commands, draft, needsReload, reload, replaceDraft, replaceSelectedNote, selected]);

  return (
    <>
      <KnowledgeVaultNotesView
        loadState={loadState}
        notes={notes}
        selected={selected}
        editing={editing}
        draft={draft}
        newTitle={newTitle}
        pendingLinkTitle={pendingLinkTitle}
        notice={notice}
        needsReload={needsReload}
        onRefresh={() => void refreshVault()}
        onReloadSelected={reloadSelectedAfterConflict}
        onSelect={(slug) => void openNote(slug)}
        onStartNew={() => setNewTitle("")}
        onNewTitleChange={setNewTitle}
        onCreateNew={() => {
          if (newTitle?.trim()) void createAndOpen(newTitle.trim());
        }}
        onCancelNew={() => setNewTitle(null)}
        onStartEdit={() => setEditing(true)}
        onDraftChange={replaceDraft}
        onSaveEdit={() => void saveEdit()}
        onCancelEdit={() => {
          setEditing(false);
          setNeedsReload(false);
          if (selected) replaceDraft(selected.body);
        }}
        onOpenLink={openLink}
        onCreateFromLink={() => {
          if (pendingLinkTitle) void createAndOpen(pendingLinkTitle);
        }}
        onDismissLink={() => setPendingLinkTitle(null)}
      />
      <ObsidianIntegrationPanel selectedNote={selected} />
    </>
  );
}

export function isKnowledgeVaultWriteConflict(error: unknown): boolean {
  const message = error instanceof Error ? error.message : typeof error === "string" ? error : "";
  return message.includes("knowledge_vault_conflict");
}

export function KnowledgeVaultNotesView({
  loadState,
  notes,
  selected,
  editing,
  draft,
  newTitle,
  pendingLinkTitle,
  notice = null,
  needsReload = false,
  onRefresh = () => {},
  onReloadSelected = () => {},
  onSelect,
  onStartNew,
  onNewTitleChange,
  onCreateNew,
  onCancelNew,
  onStartEdit,
  onDraftChange,
  onSaveEdit,
  onCancelEdit,
  onOpenLink,
  onCreateFromLink,
  onDismissLink,
}: {
  loadState: "loading" | "ready" | "unavailable";
  notes: KnowledgeVaultNoteSummary[];
  selected: KnowledgeVaultNote | null;
  editing: boolean;
  draft: string;
  newTitle: string | null;
  pendingLinkTitle: string | null;
  notice?: string | null;
  needsReload?: boolean;
  onRefresh?: () => void;
  onReloadSelected?: () => void;
  onSelect: (slug: string) => void;
  onStartNew: () => void;
  onNewTitleChange: (value: string) => void;
  onCreateNew: () => void;
  onCancelNew: () => void;
  onStartEdit: () => void;
  onDraftChange: (value: string) => void;
  onSaveEdit: () => void;
  onCancelEdit: () => void;
  onOpenLink: (title: string) => void;
  onCreateFromLink: () => void;
  onDismissLink: () => void;
}) {
  return (
    <section className="knowledge-base-panel knowledge-vault-notes" aria-label="知识库笔记">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">笔记</p>
          <h3>知识库笔记</h3>
        </div>
        <div className="action-row">
          <button className="secondary-button" type="button" onClick={onRefresh}>
            刷新笔记
          </button>
          <button className="secondary-button" type="button" onClick={onStartNew}>
            新建笔记
          </button>
        </div>
      </div>
      <p className="muted small-note">笔记存在这台电脑工作台自管的 vault 里（Markdown 文件）；Syn 原生工作区无需外部应用，不碰你的其他文件夹。</p>
      {loadState === "loading" ? <p className="muted small-note">正在读取笔记…</p> : null}
      {loadState === "unavailable" ? (
        <EmptyState what="笔记只在桌面壳里能读写，这里读不到。" next="用 Tauri 桌面壳打开知识库就能建、看、改" />
      ) : null}
      {loadState === "ready" ? (
        <div className="knowledge-vault-body">
          <div className="knowledge-vault-list" aria-label="笔记列表">
            {notes.map((note) => (
              <button
                className={`knowledge-vault-item${selected?.slug === note.slug ? " is-selected" : ""}`}
                type="button"
                key={note.slug}
                onClick={() => onSelect(note.slug)}
              >
                <strong>{note.title}</strong>
              </button>
            ))}
            {notes.length === 0 ? <EmptyState what="vault 里还没有笔记。" next="点「新建笔记」写第一条" /> : null}
          </div>
          <div className="knowledge-vault-main">
            {newTitle !== null ? (
              <div className="knowledge-vault-new" aria-label="新建笔记">
                <input
                  aria-label="笔记标题"
                  value={newTitle}
                  placeholder="笔记标题"
                  onChange={(event) => onNewTitleChange(event.target.value)}
                />
                <button className="primary-button" type="button" onClick={onCreateNew} disabled={!newTitle.trim()}>
                  创建
                </button>
                <button className="secondary-button" type="button" onClick={onCancelNew}>
                  算了
                </button>
              </div>
            ) : null}
            {pendingLinkTitle ? (
              <div className="knowledge-vault-new" aria-label="未命中双链">
                <span>《{pendingLinkTitle}》还不存在。</span>
                <button className="secondary-button" type="button" onClick={onCreateFromLink}>
                  新建《{pendingLinkTitle}》
                </button>
                <button className="secondary-button" type="button" onClick={onDismissLink}>
                  算了
                </button>
              </div>
            ) : null}
            {notice ? <p className="muted small-note">{notice}</p> : null}
            {needsReload ? (
              <div className="knowledge-vault-conflict" role="alert">
                <span>外部改动已被保护；重新读取后才可保存。</span>
                <button className="secondary-button" type="button" onClick={onReloadSelected}>
                  重新读取
                </button>
              </div>
            ) : null}
            {selected ? (
              editing ? (
                <div className="knowledge-vault-edit">
                  <textarea
                    aria-label="编辑笔记"
                    value={draft}
                    rows={14}
                    onChange={(event) => onDraftChange(event.target.value)}
                  />
                  <div className="action-row">
                    <button className="primary-button" type="button" onClick={onSaveEdit} disabled={needsReload}>
                      保存
                    </button>
                    <button className="secondary-button" type="button" onClick={onCancelEdit}>
                      取消
                    </button>
                  </div>
                </div>
              ) : (
                <div className="knowledge-vault-read">
                  <div className="action-row">
                    <button className="secondary-button" type="button" onClick={onStartEdit}>
                      编辑
                    </button>
                  </div>
                  <MarkdownBlocks body={selected.body} onOpenLink={onOpenLink} />
                </div>
              )
            ) : (
              newTitle === null && <p className="muted small-note">点左边一条看内容；[[双方括号]]是笔记之间的链接。</p>
            )}
          </div>
        </div>
      ) : null}
    </section>
  );
}

function MarkdownBlocks({ body, onOpenLink }: { body: string; onOpenLink: (title: string) => void }) {
  const blocks = parseMarkdown(body);
  return (
    <div className="knowledge-vault-markdown">
      {blocks.map((block, index) => (
        <MarkdownBlock key={index} block={block} onOpenLink={onOpenLink} />
      ))}
    </div>
  );
}

function MarkdownBlock({ block, onOpenLink }: { block: MdBlock; onOpenLink: (title: string) => void }) {
  if (block.kind === "heading") {
    const Tag = (`h${block.level}`) as "h1";
    return (
      <Tag className="knowledge-vault-heading">
        <InlineSegments inlines={block.inlines} onOpenLink={onOpenLink} />
      </Tag>
    );
  }
  if (block.kind === "code_block") {
    return <pre className="knowledge-vault-code">{block.text}</pre>;
  }
  if (block.kind === "list") {
    const items = block.items.map((item, index) => (
      <li key={index}>
        <InlineSegments inlines={item} onOpenLink={onOpenLink} />
      </li>
    ));
    return block.ordered ? <ol className="knowledge-vault-list-md">{items}</ol> : <ul className="knowledge-vault-list-md">{items}</ul>;
  }
  return (
    <p>
      <InlineSegments inlines={block.inlines} onOpenLink={onOpenLink} />
    </p>
  );
}

function InlineSegments({ inlines, onOpenLink }: { inlines: MdInline[]; onOpenLink: (title: string) => void }) {
  return (
    <>
      {inlines.map((segment, index) => {
        if (segment.kind === "bold") return <strong key={index}>{segment.text}</strong>;
        if (segment.kind === "italic") return <em key={index}>{segment.text}</em>;
        if (segment.kind === "code") return <code key={index}>{segment.text}</code>;
        if (segment.kind === "wikilink") {
          return (
            <button className="knowledge-vault-wikilink" type="button" key={index} onClick={() => onOpenLink(segment.title)}>
              {segment.title}
            </button>
          );
        }
        if (segment.kind === "link") {
          return (
            <a href={segment.url} target="_blank" rel="noreferrer" key={index}>
              {segment.url}
            </a>
          );
        }
        return <span key={index}>{segment.text}</span>;
      })}
    </>
  );
}
