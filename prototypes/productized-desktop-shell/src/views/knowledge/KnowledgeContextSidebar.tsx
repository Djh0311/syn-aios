import { useId, useState, type ReactNode } from "react";
import type { KnowledgeWorkspaceMarkdownDocument } from "../../lib/tauri";

// N2R-R3E D2：右栏由一路平铺到底的长列表收敛为「属性 / 反向引用 / 来源上下文」
// 三个稳定区块，每区一个可及标题 + aria-expanded 明确的折叠控件。
//
// 边界：
// - 注入方 `KnowledgeBaseView` 冻结只读。来源上下文的层级由这里在**未修改**的
//   注入内容外面包一层产品侧容器实现，注入内容本身逐字照渲。
// - 折叠状态不持久化：不写 store / vault / Markdown，也不新增任何 localStorage
//   键。刷新后回到固定的默认展开集合（三区全展开），可预测。
// - 折叠后内容带 `hidden`，同时退出 Tab 顺序与辅助技术树；空态、loading 与冲突
//   文案只被压进区块里，不允许消失。
// - 本阶段不提供大纲：右栏标题不再声明「大纲」，也不派生任何标题列表。

export function KnowledgeContextSidebar({
  selected,
  rightCollapsed,
  onCollapseRight,
  onOpen,
  sourceContext,
}: {
  selected: KnowledgeWorkspaceMarkdownDocument | null;
  rightCollapsed: boolean;
  onCollapseRight: () => void;
  onOpen: (relativePath: string) => void;
  sourceContext: ((selectedRelativePath: string | null) => ReactNode) | null;
}) {
  return (
    <>
      <div className="syn-knowledge-sidebar-tabs syn-knowledge-sidebar-tabs--right">
        <span>属性 / 反向引用 / 来源上下文</span>
        <button
          className="text-button"
          type="button"
          aria-label="折叠右侧栏"
          aria-expanded={!rightCollapsed}
          onClick={onCollapseRight}
        >
          折叠
        </button>
      </div>
      <div className="native-context-stack">
        <KnowledgeContextSection title="属性" badge={selected ? `${selected.tags.length} 标签` : null}>
          {selected ? (
            <WorkspaceMetadata document={selected} onOpen={onOpen} />
          ) : (
            <p className="muted small-note">打开笔记后会显示安全属性、标签和反向引用。</p>
          )}
        </KnowledgeContextSection>
        <KnowledgeContextSection title="反向引用" badge={selected ? `${selected.backlinks.length}` : null}>
          {selected ? (
            <div className="native-workspace-backlinks">
              {selected.backlinks.length
                ? selected.backlinks.map((relativePath) => (
                    <button type="button" key={relativePath} onClick={() => onOpen(relativePath)}>
                      {relativePath}
                    </button>
                  ))
                : <span>暂无反向引用</span>}
            </div>
          ) : (
            <p className="muted small-note">未选择文件时没有反向引用可投影。</p>
          )}
        </KnowledgeContextSection>
        <KnowledgeContextSection title="来源上下文" badge={null}>
          {sourceContext?.(selected?.relative_path ?? null) ?? <p className="muted small-note">没有接入来源上下文。</p>}
        </KnowledgeContextSection>
      </div>
    </>
  );
}

/**
 * 一个可折叠区块。默认展开——三区全展开是本阶段固定且可预测的默认集合。
 * 折叠只改本区块，不影响其它区块，也不写任何持久化。
 */
function KnowledgeContextSection({
  title,
  badge,
  children,
}: {
  title: string;
  badge: string | null;
  children: ReactNode;
}) {
  const [expanded, setExpanded] = useState(true);
  const bodyId = `${useId()}-body`;
  return (
    <section className="native-context-section" data-context-section={title} data-expanded={expanded}>
      <h3 className="native-context-heading">
        <button
          className="native-context-summary"
          type="button"
          aria-expanded={expanded}
          aria-controls={bodyId}
          onClick={() => setExpanded((current) => !current)}
        >
          <svg
            className="native-context-chevron"
            viewBox="0 0 12 12"
            width="10"
            height="10"
            fill="none"
            stroke="currentColor"
            strokeWidth="1.5"
            strokeLinecap="round"
            strokeLinejoin="round"
            aria-hidden="true"
            focusable="false"
          >
            <path d="M4 2.5 8 6l-4 3.5" />
          </svg>
          <span className="native-context-title">{title}</span>
          {badge ? <span className="native-context-badge">{badge}</span> : null}
        </button>
      </h3>
      <div className="native-context-body" id={bodyId} hidden={!expanded}>
        {children}
      </div>
    </section>
  );
}

function WorkspaceMetadata({
  document,
  onOpen,
}: {
  document: KnowledgeWorkspaceMarkdownDocument;
  onOpen: (relativePath: string) => void;
}) {
  const propertyEntries = Object.entries(document.properties);
  return (
    <div className="native-workspace-metadata">
      <div>
        <strong>标签</strong>
        {document.tags.length ? <span>{document.tags.map((tag) => `#${tag}`).join(" · ")}</span> : <span>未标记</span>}
      </div>
      <div>
        <strong>别名</strong>
        {document.aliases.length ? <span>{document.aliases.join(" · ")}</span> : <span>无</span>}
      </div>
      <div>
        <strong>属性</strong>
        {propertyEntries.length ? (
          <dl>
            {propertyEntries.map(([key, value]) => (
              <div key={key}>
                <dt>{key}</dt>
                <dd>{value}</dd>
              </div>
            ))}
          </dl>
        ) : (
          <span>无声明式属性</span>
        )}
      </div>
      <div>
        <strong>正向链接</strong>
        {document.outlinks.length ? (
          document.outlinks.map((relativePath) => (
            <button type="button" key={relativePath} onClick={() => onOpen(relativePath)}>
              {relativePath}
            </button>
          ))
        ) : (
          <span>暂无链接</span>
        )}
      </div>
    </div>
  );
}
