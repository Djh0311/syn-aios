import type { MouseEvent as ReactMouseEvent } from "react";

// N2R-R3E D1：活动栏由「可断行的中文文字按钮」收敛为紧凑 icon ribbon。
//
// 这里只改呈现：可访问名称、aria-pressed、dispatch 与 overlay 行为全部由调用方
// 原样传入，本组件不持有任何业务状态，也不知道自己点开的是哪个面板。
//
// 图标一律是本仓自绘的极简几何路径，取 currentColor，不引入任何图标库、字体
// 图标或第三方资产；svg 一律 aria-hidden，可访问名称只由 button 的 aria-label
// 承担，绝不靠 title 提示替代。

export type KnowledgeActivityRailIcon =
  | "files"
  | "search"
  | "graph"
  | "canvas"
  | "command"
  | "maintenance"
  | "sources"
  | "context";

export type KnowledgeActivityRailItem = Readonly<{
  icon: KnowledgeActivityRailIcon;
  /** 可访问名称。必须与 R2 已验证的八个入口逐字一致。 */
  label: string;
  /** 有状态入口才给 boolean：undefined 时 React 会整条略掉 aria-pressed。 */
  active?: boolean;
  onSelect: (event: ReactMouseEvent<HTMLButtonElement>) => void;
}>;

export function KnowledgeActivityRail({ items }: { items: ReadonlyArray<KnowledgeActivityRailItem> }) {
  return (
    <>
      {items.map((item) => (
        <button
          key={item.label}
          className={`native-activity-button${item.active ? " is-active" : ""}`}
          type="button"
          aria-label={item.label}
          aria-pressed={item.active}
          data-activity-entry={item.icon}
          onClick={item.onSelect}
        >
          <KnowledgeActivityIcon name={item.icon} />
        </button>
      ))}
    </>
  );
}

function KnowledgeActivityIcon({ name }: { name: KnowledgeActivityRailIcon }) {
  return (
    <svg
      className="native-activity-icon"
      viewBox="0 0 20 20"
      width="18"
      height="18"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.4"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      focusable="false"
    >
      {knowledgeActivityIconGeometry(name)}
    </svg>
  );
}

/** 每个图标都是几条直线 / 圆 / 矩形，靠轮廓区分，不承担文字信息。 */
function knowledgeActivityIconGeometry(name: KnowledgeActivityRailIcon) {
  switch (name) {
    case "files":
      // 一页纸：右上折角 + 两条正文线
      return (
        <>
          <path d="M11.2 2.8H5.6v14.4h8.8V6z" />
          <path d="M11.2 2.8V6h3.2" />
          <path d="M7.8 10.6h4.4M7.8 13.4h3" />
        </>
      );
    case "search":
      // 放大镜：圆 + 斜柄
      return (
        <>
          <circle cx="8.8" cy="8.8" r="4.6" />
          <path d="M12.3 12.3 16.6 16.6" />
        </>
      );
    case "graph":
      // 关系：三个节点两条边
      return (
        <>
          <circle cx="4.6" cy="6" r="1.9" />
          <circle cx="15.2" cy="4.9" r="1.9" />
          <circle cx="9.9" cy="15.1" r="1.9" />
          <path d="M6.4 6.7 8.6 13.3M13.9 6.5 11.3 13.4" />
        </>
      );
    case "canvas":
      // 画布：外框 + 一条竖分隔
      return (
        <>
          <rect x="2.8" y="4.2" width="14.4" height="11.6" rx="1.4" />
          <path d="M11.4 4.2v11.6" />
        </>
      );
    case "command":
      // 命令提示符：> 与下划线
      return (
        <>
          <path d="M5.4 6.6 9.2 10l-3.8 3.4" />
          <path d="M10.6 13.8h4.2" />
        </>
      );
    case "maintenance":
      // 维护：两条推子轨 + 两个滑块
      return (
        <>
          <path d="M3.2 7.2h13.6M3.2 12.8h13.6" />
          <circle cx="7.6" cy="7.2" r="1.8" />
          <circle cx="12.6" cy="12.8" r="1.8" />
        </>
      );
    case "sources":
      // 来源：三层堆叠
      return (
        <>
          <rect x="3.4" y="3.6" width="13.2" height="3.4" rx="1.2" />
          <rect x="3.4" y="8.3" width="13.2" height="3.4" rx="1.2" />
          <rect x="3.4" y="13" width="13.2" height="3.4" rx="1.2" />
        </>
      );
    case "context":
      // 右侧上下文：外框 + 靠右的分栏线
      return (
        <>
          <rect x="2.8" y="4.2" width="14.4" height="11.6" rx="1.4" />
          <path d="M12.6 4.2v11.6" />
        </>
      );
  }
}
