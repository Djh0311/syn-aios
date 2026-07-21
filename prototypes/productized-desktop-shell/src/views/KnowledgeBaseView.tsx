import { useCallback, useEffect, useState } from "react";
import { Pill, EmptyState } from "../components/SpecPrimitives";
import { DetailLine } from "../components/WorkbenchPrimitives";
import { deriveKnowledgeBaseSummary, type KnowledgeDocumentReadModel, type KnowledgeMemoryLink } from "../lib/knowledgeBase";
import { deriveKnowledgeBasePageReadModelFromParts } from "../lib/pageSelectors";
import { parseMarkdown, type MdBlock, type MdInline } from "../lib/knowledgeVault";
import {
  knowledgeVaultCreateNote,
  knowledgeVaultListNotes,
  knowledgeVaultReadNote,
  knowledgeVaultWriteNote,
  type KnowledgeVaultNote,
  type KnowledgeVaultNoteSummary,
} from "../lib/tauri";
import type {
  FormalMemoryStoreV1,
  MemoryCaptureStoreV1,
  MemoryCandidateStoreV1,
  PendingAction,
  ProjectRecord,
  WorkflowStateSnapshot,
} from "../lib/types";

export function KnowledgeBaseView({
  projects,
  workflowState,
  formalMemoryStore,
  memoryCaptureStore,
  memoryCandidateStore,
  hasRealSnapshot,
  onRequestAction,
}: {
  projects: ProjectRecord[];
  workflowState: WorkflowStateSnapshot | null;
  formalMemoryStore?: FormalMemoryStoreV1 | null;
  memoryCaptureStore?: MemoryCaptureStoreV1 | null;
  memoryCandidateStore?: MemoryCandidateStoreV1 | null;
  hasRealSnapshot: boolean;
  onRequestAction: (action: PendingAction) => void;
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
    <section className="stage-pad knowledge-base" aria-label="知识库最小入口">
      <div className="pg-head">
        <div>
          <p className="pg-sub">知识库</p>
          <h1 className="pg-title">知识库资料</h1>
        </div>
        <div className="pg-meta">
          <div className="big">{pageReadModel.snapshot_status_label}</div>
          <div>{pageReadModel.boundary_text}</div>
        </div>
      </div>

      <div className="stat-strip knowledge-base-stats">
        <StatCell label="资料" value={`${pageReadModel.document_count}`} helper="知识库资料" />
        <StatCell label="正式记忆" value={`${pageReadModel.formal_memory_link_count}`} helper="关联正式记忆" />
        <StatCell label="候选" value={`${pageReadModel.candidate_link_count}`} helper="关联候选" />
        <StatCell label="任务引用" value={`${pageReadModel.task_reference_count}`} helper="任务包知识引用" />
        <StatCell label="捕获" value={`${pageReadModel.capture_event_count}`} helper="记忆捕获来源" />
      </div>

      <div className="knowledge-base-grid">
        <section className="knowledge-base-panel knowledge-document-list" aria-label="知识库资料列表">
          <div className="panel-heading">
            <div>
              <p className="eyebrow">资料列表</p>
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
                  关联正式记忆 {document.formal_memory_links.length} / 关联候选 {document.candidate_links.length} /{" "}
                  {document.task_reference_summary.display_text}
                </small>
                <details className="agent-boundary-details">
                  <summary className="agent-boundary-summary">开发者详情</summary>
                  <em>{document.source_anchor.source_kind} / {document.source_anchor.path_summary}</em>
                </details>
              </div>
            ))}
            {!summary.documents.length ? (
              <p className="muted small-note">暂无权威文件可作为知识库资料；不会伪造知识库索引。</p>
            ) : null}
          </div>
        </section>

        <section className="knowledge-base-panel knowledge-boundary-panel" aria-label="知识库和记忆边界">
          <div className="panel-heading">
            <div>
              <p className="eyebrow">边界</p>
              <h3>{pageReadModel.obsidian_boundary.label}</h3>
            </div>
            <Pill tone="unknown">占位</Pill>
          </div>
          <div className="workflow-compact-list">
            <div className="workflow-compact-item">
              <strong>{pageReadModel.obsidian_boundary.native_sync_status}</strong>
              <span>{pageReadModel.obsidian_boundary.vault_scan_status}</span>
              <em>{pageReadModel.obsidian_boundary.forbidden_text}</em>
            </div>
            <div className="workflow-compact-item">
              <strong>知识库和正式记忆</strong>
              <span>知识库是材料和笔记空间；正式记忆是经过确认、来源、版本、审计和权限治理的行为上下文。</span>
              <em>知识命中、资料摘要和 Markdown 来源不能绕过候选流程。</em>
            </div>
          </div>
        </section>

        <section className="knowledge-base-panel knowledge-detail-panel" aria-label="知识库资料详情">
          <div className="panel-heading">
            <div>
              <p className="eyebrow">资料详情</p>
              <h3>来源 / 反向引用 / 候选入口</h3>
            </div>
            <Pill tone={summary.documents.length ? "candidate" : "unknown"}>{summary.documents.length ? "可提出候选" : "空"}</Pill>
          </div>
          {summary.documents.length ? (
            summary.documents.map((document) => (
              <KnowledgeDocumentDetail document={document} onRequestAction={onRequestAction} key={document.document_key} />
            ))
          ) : (
            <p className="muted small-note">暂无知识库资料可展示来源、反向引用和候选入口。</p>
          )}
        </section>

        <section className="knowledge-base-panel knowledge-boundary-panel" aria-label="记忆捕获来源">
          <div className="panel-heading">
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
            {!summary.recent_capture_events.length ? (
              <p className="muted small-note">暂无记忆捕获事件；知识库不会伪造候选来源。</p>
            ) : null}
          </div>
        </section>
      </div>

      <KnowledgeVaultNotesPanel />

      {summary.warnings.length ? (
        <div className="knowledge-warning-list" aria-label="知识库读模型警告">
          {summary.warnings.slice(0, 4).map((warning) => (
            <p className="state-warning" key={warning}>{warning}</p>
          ))}
        </div>
      ) : null}
    </section>
  );
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
    boundary: "只会在你确认后写入 memory-candidates.v1.json；只生成候选，不写正式记忆；未执行 Obsidian 原生同步。",
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

// ── L3 知识库第一片：vault 笔记区（工作台自管目录·用户手编为主·AI 写入只走弹窗那一下） ──

export type KnowledgeVaultCommands = {
  listNotes: () => Promise<KnowledgeVaultNoteSummary[]>;
  readNote: (slug: string) => Promise<KnowledgeVaultNote>;
  createNote: (title: string) => Promise<{ slug: string; title: string }>;
  writeNote: (slug: string, body: string) => Promise<unknown>;
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
      <section className="knowledge-base-panel knowledge-vault-notes" aria-label="知识库笔记">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">笔记</p>
            <h3>知识库笔记</h3>
          </div>
        </div>
        <p className="muted small-note">正在读取笔记…</p>
      </section>
    );
  }
  return <KnowledgeVaultNotesPanelInner commands={commands} />;
}

function KnowledgeVaultNotesPanelInner({ commands }: { commands: KnowledgeVaultCommands }) {
  const [loadState, setLoadState] = useState<"loading" | "ready" | "unavailable">("loading");
  const [notes, setNotes] = useState<KnowledgeVaultNoteSummary[]>([]);
  const [selected, setSelected] = useState<KnowledgeVaultNote | null>(null);
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState("");
  const [newTitle, setNewTitle] = useState<string | null>(null);
  const [pendingLinkTitle, setPendingLinkTitle] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

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
    async (slug: string) => {
      try {
        const note = await commands.readNote(slug);
        setSelected(note);
        setEditing(false);
        setDraft(note.body);
      } catch {
        setNotice("这条笔记没读到。");
      }
    },
    [commands],
  );

  const createAndOpen = useCallback(
    async (title: string) => {
      try {
        const created = await commands.createNote(title);
        await reload();
        const note = await commands.readNote(created.slug);
        setSelected(note);
        setDraft(note.body);
        setEditing(true);
        setNewTitle(null);
        setPendingLinkTitle(null);
        setNotice(null);
      } catch {
        setNotice("新建没成功。");
      }
    },
    [commands, reload],
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

  return (
    <KnowledgeVaultNotesView
      loadState={loadState}
      notes={notes}
      selected={selected}
      editing={editing}
      draft={draft}
      newTitle={newTitle}
      pendingLinkTitle={pendingLinkTitle}
      notice={notice}
      onSelect={(slug) => void openNote(slug)}
      onStartNew={() => setNewTitle("")}
      onNewTitleChange={setNewTitle}
      onCreateNew={() => {
        if (newTitle?.trim()) void createAndOpen(newTitle.trim());
      }}
      onCancelNew={() => setNewTitle(null)}
      onStartEdit={() => setEditing(true)}
      onDraftChange={setDraft}
      onSaveEdit={() => {
        if (!selected) return;
        void commands.writeNote(selected.slug, draft).then(async () => {
          await reload();
          const note = await commands.readNote(selected.slug);
          setSelected(note);
          setEditing(false);
          setNotice("已保存。");
        });
      }}
      onCancelEdit={() => {
        setEditing(false);
        if (selected) setDraft(selected.body);
      }}
      onOpenLink={openLink}
      onCreateFromLink={() => {
        if (pendingLinkTitle) void createAndOpen(pendingLinkTitle);
      }}
      onDismissLink={() => setPendingLinkTitle(null)}
    />
  );
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
        <button className="secondary-button" type="button" onClick={onStartNew}>
          新建笔记
        </button>
      </div>
      <p className="muted small-note">笔记存在这台电脑工作台自管的 vault 里（md 文件）；不碰你的其他文件夹，不同步 Obsidian。</p>
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
                    <button className="primary-button" type="button" onClick={onSaveEdit}>
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
