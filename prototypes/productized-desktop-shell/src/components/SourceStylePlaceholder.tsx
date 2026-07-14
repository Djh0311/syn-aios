import type { ReactNode } from "react";

export function SourceStylePlaceholder({
  title,
  kicker,
  hasRealSnapshot,
  items,
  summary,
  primaryStat,
  secondaryStat,
  sections,
  boundary,
  lede = null,
}: {
  title: string;
  kicker: string;
  hasRealSnapshot: boolean;
  items: string[];
  summary: string;
  primaryStat: string;
  secondaryStat: string;
  // 可选首屏引导位(K 定稿·想法箱空态用)。不传=旧调用点零变化。
  lede?: ReactNode;
  sections: {
    title: string;
    eyebrow: string;
    items: string[];
    emptyText: string;
  }[];
  boundary: {
    title: string;
    text: string;
    status: string;
  };
}) {
  return (
    <section className="stage-pad source-placeholder">
      <div className="sr-only">
        <p>{kicker}</p>
        <h1>{title}</h1>
        <p>{summary}</p>
      </div>

      <div className="source-entry-hero">
        <div>
          <p className="eyebrow">{kicker}</p>
          <h2>{title}</h2>
          <p>{summary}</p>
        </div>
        <div className="source-entry-stats" aria-label={`${title}入口状态`}>
          <SourceEntryStat label="索引状态" value={hasRealSnapshot ? "已读取" : "未接真实数据"} />
          <SourceEntryStat label="当前内容" value={primaryStat} />
          <SourceEntryStat label="后置能力" value={secondaryStat} />
        </div>
      </div>

      {lede}

      <div className="source-entry-grid">
        {sections.map((section) => (
          <section className="panel source-entry-section" key={section.title}>
            <div className="panel-h">
              {section.title}
              <span className="count">{section.items.length}</span>
            </div>
            <p className="eyebrow">{section.eyebrow}</p>
            {section.items.length ? (
              <div className="source-entry-list">
                {section.items.map((item) => (
                  <div className="source-entry-row" key={item}>
                    <strong>{item.split(" · ")[0]}</strong>
                    <span>{item}</span>
                    <em>只读</em>
                  </div>
                ))}
              </div>
            ) : (
              <p className="muted small-note">{section.emptyText}</p>
            )}
          </section>
        ))}

        <section className="panel source-entry-section source-entry-boundary">
          <div className="panel-h">
            {boundary.title}
            <span className="count">{boundary.status}</span>
          </div>
          <div className="source-entry-boundary-card">
            <strong>当前只保留页面入口和只读索引</strong>
            <span>{boundary.text}</span>
            <em>不足部分不冒充真实完成，不触发执行，不读取敏感凭据。</em>
          </div>
        </section>
      </div>
    </section>
  );
}

function SourceEntryStat({ label, value }: { label: string; value: string }) {
  return (
    <div className="source-entry-stat">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}
