import type { ReactNode } from "react";

export function KnowledgeWorkbenchShell({
  activityRail,
  leftSidebar,
  centralWorkspace,
  rightSidebar,
  statusBar,
  overlay = null,
  leftCollapsed = false,
  rightCollapsed = false,
}: {
  activityRail: ReactNode;
  leftSidebar: ReactNode;
  centralWorkspace: ReactNode;
  rightSidebar: ReactNode;
  statusBar: ReactNode;
  overlay?: ReactNode;
  leftCollapsed?: boolean;
  rightCollapsed?: boolean;
}) {
  return (
    <section
      className={`syn-knowledge-shell${leftCollapsed ? " is-left-collapsed" : ""}${rightCollapsed ? " is-right-collapsed" : ""}`}
      aria-label="Syn 知识工作台"
      data-knowledge-shell="syn-workbench"
    >
      <nav className="syn-knowledge-shell__activity" aria-label="知识活动栏" data-knowledge-region="activity">
        {activityRail}
      </nav>
      <aside
        className="syn-knowledge-shell__left"
        aria-label="知识左侧栏"
        aria-hidden={leftCollapsed}
        inert={leftCollapsed}
        data-knowledge-region="left"
      >
        {leftCollapsed ? null : leftSidebar}
      </aside>
      <section className="syn-knowledge-shell__central" aria-label="知识中央工作区" data-knowledge-region="central">
        {centralWorkspace}
      </section>
      <aside
        className="syn-knowledge-shell__right"
        aria-label="知识右侧上下文"
        aria-hidden={rightCollapsed}
        inert={rightCollapsed}
        data-knowledge-region="right"
      >
        {rightCollapsed ? null : rightSidebar}
      </aside>
      <footer className="syn-knowledge-shell__status" aria-label="知识状态栏" data-knowledge-region="status">
        {statusBar}
      </footer>
      {overlay}
    </section>
  );
}
