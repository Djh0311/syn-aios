import { Badge } from "../components/Badge";
import { DetailLine } from "../components/WorkbenchPrimitives";
import { deriveKnowledgeBaseSummary, type KnowledgeDocumentReadModel, type KnowledgeMemoryLink } from "../lib/knowledgeBase";
import { deriveKnowledgeBasePageReadModelFromParts } from "../lib/pageSelectors";
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
            <Badge tone="neutral">{summary.documents.length}</Badge>
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
            <Badge tone="unknown">占位</Badge>
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
            <Badge tone={summary.documents.length ? "candidate" : "unknown"}>{summary.documents.length ? "可提出候选" : "空"}</Badge>
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
            <Badge tone={summary.capture_event_count ? "candidate" : "unknown"}>{summary.capture_event_count}</Badge>
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
