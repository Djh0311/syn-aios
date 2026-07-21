// 设计系统基座(2026-07-15 施工·设计定稿 `docs/design/2026-07-14-stage-b-hifi-fullapp-v1.html` 的定式组件化)。
// 规则正本=DESIGN.md 信息规范:四问+两禁令+三面定式。此后新页面一律引用这里,禁止临场发挥。
import type { ReactNode } from "react";

// 事实行:左键右值,数值 tabular-nums(交货卡「干了什么」/体检单/项目事实卡/弹层三要素通用)。
export function FactRow({ k, children, bad = false }: { k: string; children: ReactNode; bad?: boolean }) {
  return (
    <div className="spec-fact-row">
      <span className="spec-fact-k">{k}</span>
      <span className={bad ? "spec-fact-v spec-bad" : "spec-fact-v"}>{children}</span>
    </div>
  );
}

// 段标题(卡内分组:「干了什么」「复核体检单」「会动什么」)。
export function SegTitle({ children }: { children: ReactNode }) {
  return <p className="spec-seg-title">{children}</p>;
}

// pill 行+pill(状态概览:已交货/⚠N 项不符/全程 N 分钟)。ariaLabel 可选(G3 补回:交货卡概览行「这单概览」)。
export function PillRow({ children, ariaLabel }: { children: ReactNode; ariaLabel?: string }) {
  return (
    <div className="spec-pill-row" aria-label={ariaLabel}>
      {children}
    </div>
  );
}
export function Pill({
  tone = "plain",
  children,
}: {
  tone?: "plain" | "ok" | "warn" | "run" | "candidate" | "unknown" | "bad";
  children: ReactNode;
}) {
  return <span className={`spec-pill spec-pill-${tone}`}>{children}</span>;
}

// 三元素列表行(回顾面定式:状态徽章+一句 claim+时间;其余归详情)。
// onSelect 可选=纯展示;带 onSelect=可选中(cursor+选中态)。无 hooks,离线测试可平铺调用。
export function ListRow({
  badge,
  claim,
  time,
  selected = false,
  onSelect,
}: {
  badge?: ReactNode;
  claim: ReactNode;
  time?: ReactNode;
  selected?: boolean;
  onSelect?: () => void;
}) {
  return (
    <div
      className={`spec-list-row${selected ? " is-selected" : ""}${onSelect ? " is-selectable" : ""}`}
      onClick={onSelect}
      role={onSelect ? "button" : undefined}
      aria-pressed={onSelect ? selected : undefined}
    >
      {badge != null ? <span className="spec-list-badge">{badge}</span> : null}
      <span className="spec-list-claim">{claim}</span>
      {time != null ? <span className="spec-list-time">{time}</span> : null}
    </div>
  );
}

// 空态(宪法 D7:必答「下一步做什么」;禁止只说「这里没有东西」)。
export function EmptyState({ what, next }: { what: string; next: string }) {
  return (
    <p className="spec-empty muted small-note">
      {what}
      {next ? `；${next}` : ""}
    </p>
  );
}

// 展开控件(统一「展开剩余 N 条…」;状态由父组件持有——保无 hooks 约定)。
export function ExpandRest({ hidden, onShow }: { hidden: number; onShow: () => void }) {
  if (hidden <= 0) return null;
  return (
    <button className="jiaoban-linklike spec-expand" type="button" onClick={onShow}>
      展开剩余 {hidden} 条…
    </button>
  );
}
