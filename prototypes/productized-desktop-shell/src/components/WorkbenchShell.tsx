import type { Dispatch, ReactNode, SetStateAction } from "react";
import { PermissionDialog } from "./PermissionDialog";
import { RightDetailPanel } from "./RightDetailPanel";
import type { SecretaryContext } from "../lib/secretaryReadModel";
import type {
  MemoryCandidateStoreV1,
  MemoryCaptureStoreV1,
  PendingAction,
  WorkbenchSnapshot,
  WorkflowStateSnapshot,
} from "../lib/types";
import {
  primaryNavGroups,
  primaryNavItems,
  settingsNavItem,
  workspaceRailItems,
  type RightPanelKey,
  type ViewKey,
} from "../lib/workbenchNavigation";

type WorkbenchShellStat = {
  label: string;
  value: number;
};

export function WorkbenchShell({
  activeRightPanel,
  activeView,
  actionBusy,
  children,
  displaySnapshot,
  error,
  isDeveloperView,
  memoryCandidateStore,
  memoryCaptureStore,
  notice,
  pendingAction,
  query,
  rightStats,
  secretaryContext,
  topbarReviewCount,
  workflowState,
  workflowStateError,
  workflowStateLoading,
  onActiveRightPanelChange,
  onActiveViewChange,
  onCancelAction,
  onConfirmAction,
  onQueryChange,
  onReload,
  onReloadWorkflowState,
}: {
  activeRightPanel: RightPanelKey | null;
  activeView: ViewKey;
  actionBusy: boolean;
  children: ReactNode;
  displaySnapshot: WorkbenchSnapshot;
  error: boolean;
  isDeveloperView: boolean;
  memoryCandidateStore: MemoryCandidateStoreV1 | null;
  memoryCaptureStore: MemoryCaptureStoreV1 | null;
  notice: string;
  pendingAction: PendingAction | null;
  query: string;
  rightStats: WorkbenchShellStat[];
  secretaryContext: SecretaryContext;
  topbarReviewCount: number;
  workflowState: WorkflowStateSnapshot | null;
  workflowStateError: string | null;
  workflowStateLoading: boolean;
  onActiveRightPanelChange: Dispatch<SetStateAction<RightPanelKey | null>>;
  onActiveViewChange: (view: ViewKey) => void;
  onCancelAction: () => void;
  onConfirmAction: () => void | Promise<void>;
  onQueryChange: (value: string) => void;
  onReload: () => void | Promise<void>;
  onReloadWorkflowState: () => void;
}) {
  return (
    <div
      className={`app-shell ${activeRightPanel ? "right-pane-open" : ""} ${
        activeRightPanel === "secretary" ? "right-pane-secretary" : ""
      }`}
    >
      <WorkbenchTopbar
        displaySnapshot={displaySnapshot}
        error={error}
        query={query}
        topbarReviewCount={topbarReviewCount}
        onActiveRightPanelChange={onActiveRightPanelChange}
        onActiveViewChange={onActiveViewChange}
        onQueryChange={onQueryChange}
        onReload={onReload}
      />

      <WorkbenchSidebar
        activeView={activeView}
        isDeveloperView={isDeveloperView}
        onActiveViewChange={onActiveViewChange}
      />

      <main
        className={`main-panel stage ${activeView === "projects" ? "project-stage" : ""} ${
          activeView === "agents" ? "agent-stage" : ""
        }`}
      >
        {notice || error ? (
          <section className={`notice-panel ${error ? "error" : ""}`} aria-live="polite">
            <strong>{error ? "需要处理" : "状态"}</strong>
            <span>{notice}</span>
          </section>
        ) : null}
        {children}
      </main>

      <WorkbenchStatusRail
        activeRightPanel={activeRightPanel}
        activeView={activeView}
        displaySnapshot={displaySnapshot}
        error={error}
        memoryCandidateStore={memoryCandidateStore}
        memoryCaptureStore={memoryCaptureStore}
        notice={notice}
        rightStats={rightStats}
        secretaryContext={secretaryContext}
        workflowState={workflowState}
        workflowStateError={workflowStateError}
        workflowStateLoading={workflowStateLoading}
        onActiveRightPanelChange={onActiveRightPanelChange}
        onActiveViewChange={onActiveViewChange}
        onReloadWorkflowState={onReloadWorkflowState}
      />

      <WorkbenchDock onActiveRightPanelChange={onActiveRightPanelChange} />

      <PermissionDialog
        action={pendingAction}
        busy={actionBusy}
        onCancel={onCancelAction}
        onConfirm={() => void onConfirmAction()}
      />

      <button
        className={`secretary-float ${activeView === "secretary_board" ? "active" : ""}`}
        type="button"
        aria-label="打开秘书看板"
        onClick={() => onActiveViewChange("secretary_board")}
      >
        <span aria-hidden="true">秘</span>
        {topbarReviewCount > 0 ? <i>{topbarReviewCount}</i> : null}
      </button>
    </div>
  );
}

function WorkbenchTopbar({
  displaySnapshot,
  error,
  query,
  topbarReviewCount,
  onActiveRightPanelChange,
  onActiveViewChange,
  onQueryChange,
  onReload,
}: {
  displaySnapshot: WorkbenchSnapshot;
  error: boolean;
  query: string;
  topbarReviewCount: number;
  onActiveRightPanelChange: Dispatch<SetStateAction<RightPanelKey | null>>;
  onActiveViewChange: (view: ViewKey) => void;
  onQueryChange: (value: string) => void;
  onReload: () => void | Promise<void>;
}) {
  return (
    <header className="shell-topbar ink-shell">
      <button className="topbar-seal-button" type="button" onClick={() => onActiveViewChange("home")} aria-label="回到首页" title="首页">
        <span className="brand-mark">案</span>
      </button>
      <div className="topbar-actions">
        {displaySnapshot.projects[0] ? (
          <button className="project-switch" type="button" onClick={() => onActiveViewChange("projects")} title={displaySnapshot.projects[0].project_root}>
            <span className="pdot" aria-hidden="true" />
            <span>最近项目</span>
            <b>{displaySnapshot.projects[0].name}</b>
          </button>
        ) : null}
        <label className="search-box">
          <span aria-hidden="true">⌕</span>
          <input
            value={query}
            onChange={(event) => onQueryChange(event.currentTarget.value)}
            placeholder="搜索"
            aria-label="搜索"
          />
        </label>
        {topbarReviewCount > 0 ? (
          <button className="pending-review-button" type="button" onClick={() => onActiveRightPanelChange("todos")}>
            {topbarReviewCount} 待审
          </button>
        ) : null}
        <span className="meta-text">{displaySnapshot.summary.project_count} 项目</span>
        <button className="secondary-button icon-button" type="button" onClick={() => void onReload()} aria-label="重新读取">
          ↺
        </button>
        <span className={`top-health-dot ${error ? "error" : ""}`} title={error ? "需处理" : "可用"} aria-label={error ? "需处理" : "可用"} />
      </div>
    </header>
  );
}

function WorkbenchSidebar({
  activeView,
  isDeveloperView,
  onActiveViewChange,
}: {
  activeView: ViewKey;
  isDeveloperView: boolean;
  onActiveViewChange: (view: ViewKey) => void;
}) {
  return (
    <aside className="sidebar ink-shell">
      <div className="sidebar-inner">
        <nav className="sidebar-nav" aria-label="主导航">
          {primaryNavGroups.map((group) => (
            <section className="nav-group" aria-label={group.label} key={group.key}>
              <p className="nav-section-label">{group.label}</p>
              <div className="nav-list">
                {group.items.map((item) => (
                  <button
                    className={`nav-item ${activeView === item.key ? "active" : ""}`}
                    key={item.key}
                    type="button"
                    onClick={() => onActiveViewChange(item.key)}
                    title={item.label}
                  >
                    <span className="nav-glyph" aria-hidden="true">{item.glyph}</span>
                    <span className="nav-label">{item.label}</span>
                  </button>
                ))}
              </div>
            </section>
          ))}
          <div className="nav-list settings-nav-list" aria-label="设置入口">
            <button
              className={`nav-item ${activeView === settingsNavItem.key || isDeveloperView ? "active" : ""}`}
              type="button"
              onClick={() => onActiveViewChange(settingsNavItem.key)}
              title={settingsNavItem.label}
            >
              <span className="nav-glyph" aria-hidden="true">{settingsNavItem.glyph}</span>
              <span className="nav-label">{settingsNavItem.label}</span>
            </button>
          </div>
        </nav>
      </div>
    </aside>
  );
}

function WorkbenchStatusRail({
  activeRightPanel,
  activeView,
  displaySnapshot,
  error,
  memoryCandidateStore,
  memoryCaptureStore,
  notice,
  rightStats,
  secretaryContext,
  workflowState,
  workflowStateError,
  workflowStateLoading,
  onActiveRightPanelChange,
  onActiveViewChange,
  onReloadWorkflowState,
}: {
  activeRightPanel: RightPanelKey | null;
  activeView: ViewKey;
  displaySnapshot: WorkbenchSnapshot;
  error: boolean;
  memoryCandidateStore: MemoryCandidateStoreV1 | null;
  memoryCaptureStore: MemoryCaptureStoreV1 | null;
  notice: string;
  rightStats: WorkbenchShellStat[];
  secretaryContext: SecretaryContext;
  workflowState: WorkflowStateSnapshot | null;
  workflowStateError: string | null;
  workflowStateLoading: boolean;
  onActiveRightPanelChange: Dispatch<SetStateAction<RightPanelKey | null>>;
  onActiveViewChange: (view: ViewKey) => void;
  onReloadWorkflowState: () => void;
}) {
  return (
    <aside className="status-rail ink-shell" aria-label="工作台入口">
      <div className="right-icon-strip">
        {workspaceRailItems.map((item) => (
          <button
            className={`rail-icon-button ${
              (item.key === "secretary" ? activeView === "secretary_board" : activeRightPanel === item.key) ? "active" : ""
            }`}
            key={item.key}
            type="button"
            title={item.key === "secretary" ? "秘书看板" : item.label}
            aria-label={item.key === "secretary" ? "秘书看板" : item.label}
            aria-expanded={item.key === "secretary" ? activeView === "secretary_board" : activeRightPanel === item.key}
            onClick={() =>
              item.key === "secretary"
                ? onActiveViewChange("secretary_board")
                : onActiveRightPanelChange((current) => (current === item.key ? null : item.key))
            }
          >
            <span aria-hidden="true">{item.glyph}</span>
          </button>
        ))}
        <div className="rail-mini-stats" aria-label="工作台状态摘要">
          {rightStats.map((stat) => (
            <span key={stat.label} title={`${stat.label} ${stat.value}`}>
              {stat.value}
            </span>
          ))}
        </div>
        <span
          className={`rail-health-dot ${error || workflowStateError ? "error" : workflowStateLoading ? "loading" : ""}`}
          title={workflowStateError ?? (workflowStateLoading ? "读取中" : "状态可用")}
          aria-label={workflowStateError ?? (workflowStateLoading ? "读取中" : "状态可用")}
        />
      </div>
      {activeRightPanel ? (
        <RightDetailPanel
          activePanel={activeRightPanel}
          snapshot={displaySnapshot}
          workflowState={workflowState}
          notice={notice}
          error={error || Boolean(workflowStateError)}
          workflowStateError={workflowStateError}
          memoryCaptureStore={memoryCaptureStore}
          memoryCandidateStore={memoryCandidateStore}
          secretaryContext={secretaryContext}
          onClose={() => onActiveRightPanelChange(null)}
          onNavigate={onActiveViewChange}
          onReloadWorkflowState={onReloadWorkflowState}
        />
      ) : null}
    </aside>
  );
}

function WorkbenchDock({
  onActiveRightPanelChange,
}: {
  onActiveRightPanelChange: Dispatch<SetStateAction<RightPanelKey | null>>;
}) {
  return (
    <footer className="dock ink-shell" aria-label="秘书对话框">
      <button
        className="secretary secretary-dock-trigger"
        type="button"
        onClick={() => onActiveRightPanelChange("secretary")}
        aria-label="打开秘书对话"
      >
        <span className="secretary-orb" aria-hidden="true" />
        <span>秘 书 · 辅 助</span>
      </button>
      <div className="dock-input-wrap">
        <span className="prompt" aria-hidden="true">›</span>
        <input
          className="dock-input"
          readOnly
          onFocus={() => onActiveRightPanelChange("secretary")}
          onClick={() => onActiveRightPanelChange("secretary")}
          aria-label="秘书对话输入预览，点击打开秘书"
        />
        <div className="dock-chips" aria-label="秘书快捷入口">
          <button className="chip" type="button" onClick={() => onActiveRightPanelChange("secretary")}>解释</button>
          <button className="chip" type="button" onClick={() => onActiveRightPanelChange("secretary")}>整理</button>
          <button className="chip" type="button" onClick={() => onActiveRightPanelChange("secretary")}>提醒</button>
          <button className="chip" type="button" onClick={() => onActiveRightPanelChange("secretary")}>影响面</button>
          <button className="chip send" type="button" onClick={() => onActiveRightPanelChange("secretary")}>打开秘书</button>
        </div>
      </div>
    </footer>
  );
}
