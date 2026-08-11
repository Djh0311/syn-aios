import type { Dispatch, ReactNode, SetStateAction } from "react";
import { PermissionDialog } from "./PermissionDialog";
import { RightDetailPanel, deriveRightPanelFeedCounts } from "./RightDetailPanel";
import type { SecretaryContext } from "../lib/secretaryReadModel";
import type { SecretaryHomeReadModel, SecretarySourceRouteViewState, SecretaryTypedDeepLinkDescriptor } from "../lib/types/m4Secretary";
import type {
  MemoryCandidateStoreV1,
  MemoryCaptureStoreV1,
  PendingAction,
  WorkbenchSnapshot,
  WorkflowStateSnapshot,
} from "../lib/types";
import type { SystemStatusReadModel } from "../lib/tauri";
import {
  primaryNavGroups,
  primaryNavItems,
  settingsNavItem,
  workspaceRailItems,
  type NavigateHandler,
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
  secretaryHome,
  secretaryHomePresentationState,
  secretarySourceRouteState,
  systemStatus,
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
  onReloadSecretaryHome,
  onReloadWorkflowState,
  onOpenSecretaryDeepLink,
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
  secretaryHome: SecretaryHomeReadModel;
  secretaryHomePresentationState: "loading" | "error" | null;
  secretarySourceRouteState?: SecretarySourceRouteViewState;
  systemStatus: SystemStatusReadModel | null;
  topbarReviewCount: number;
  workflowState: WorkflowStateSnapshot | null;
  workflowStateError: string | null;
  workflowStateLoading: boolean;
  onActiveRightPanelChange: Dispatch<SetStateAction<RightPanelKey | null>>;
  onActiveViewChange: NavigateHandler;
  onCancelAction: () => void;
  onConfirmAction: () => void | Promise<void>;
  onQueryChange: (value: string) => void;
  onReload: () => void | Promise<void>;
  onReloadSecretaryHome: () => void | Promise<void>;
  onReloadWorkflowState: () => void;
  onOpenSecretaryDeepLink: (descriptor: SecretaryTypedDeepLinkDescriptor) => void;
}) {
  return (
    // 07-08 用户二调：秘书摘要归队右侧栏——与通知/待办同一套面板开法，不再单独浮层。
    <div
      className={`app-shell ${activeRightPanel ? "right-pane-open" : ""}`}
      data-active-view={activeView}
      data-secretary-source-route-phase={secretarySourceRouteState?.phase ?? "IDLE"}
      data-secretary-source-route-ref={secretarySourceRouteState?.source_route_ref ?? undefined}
      data-secretary-source-route-error-code={secretarySourceRouteState?.error_code ?? undefined}
    >
      <WorkbenchTopbar
        displaySnapshot={displaySnapshot}
        error={error}
        query={query}
        systemStatus={systemStatus}
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
          <section
            className={`notice-panel ${error ? "error" : ""}`}
            aria-live="polite"
            data-secretary-source-route-notice={secretarySourceRouteState?.phase ?? undefined}
            data-secretary-source-route-notice-ref={secretarySourceRouteState?.source_route_ref ?? undefined}
            data-secretary-source-route-notice-error-code={secretarySourceRouteState?.error_code ?? undefined}
          >
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
        secretaryHome={secretaryHome}
        secretaryHomePresentationState={secretaryHomePresentationState}
        workflowState={workflowState}
        workflowStateError={workflowStateError}
        workflowStateLoading={workflowStateLoading}
        onActiveRightPanelChange={onActiveRightPanelChange}
        onActiveViewChange={onActiveViewChange}
        onOpenSecretaryDeepLink={onOpenSecretaryDeepLink}
        onReloadSecretaryHome={onReloadSecretaryHome}
        onReloadWorkflowState={onReloadWorkflowState}
      />

      <WorkbenchDock onActiveRightPanelChange={onActiveRightPanelChange} onActiveViewChange={onActiveViewChange} />

      <PermissionDialog
        action={pendingAction}
        busy={actionBusy}
        onCancel={onCancelAction}
        onConfirm={() => void onConfirmAction()}
      />

      {/* 浮钮 = 将来「桌面宠物」秘书角色的位（当前界面未显·占位待建）。07-09 用户拍：
          留浮钮、撤它的开看板行为——开看板归 dock 里的「打开秘书」chip。保留 review 计数徽标。 */}
      <div className="secretary-float" role="img" aria-label="秘书助手（桌面宠物·建设中）">
        <span aria-hidden="true">秘</span>
        {topbarReviewCount > 0 ? <i>{topbarReviewCount}</i> : null}
      </div>
    </div>
  );
}

function WorkbenchTopbar({
  displaySnapshot,
  error,
  query,
  systemStatus,
  topbarReviewCount,
  onActiveRightPanelChange,
  onActiveViewChange,
  onQueryChange,
  onReload,
}: {
  displaySnapshot: WorkbenchSnapshot;
  error: boolean;
  query: string;
  systemStatus: SystemStatusReadModel | null;
  topbarReviewCount: number;
  onActiveRightPanelChange: Dispatch<SetStateAction<RightPanelKey | null>>;
  onActiveViewChange: NavigateHandler;
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
        <button
          className="secondary-button icon-button"
          type="button"
          data-workbench-refresh="true"
          onClick={() => void onReload()}
          aria-label="重新读取"
        >
          ↺
        </button>
        <span
          className={`top-health-dot ${error || systemStatus?.storage_healthy === false ? "error" : ""}`}
          title={error || systemStatus?.storage_healthy === false ? "需处理" : "可用"}
          aria-label={error || systemStatus?.storage_healthy === false ? "需处理" : "可用"}
        />
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
  onActiveViewChange: NavigateHandler;
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
  secretaryHome,
  secretaryHomePresentationState,
  workflowState,
  workflowStateError,
  workflowStateLoading,
  onActiveRightPanelChange,
  onActiveViewChange,
  onOpenSecretaryDeepLink,
  onReloadSecretaryHome,
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
  secretaryHome: SecretaryHomeReadModel;
  secretaryHomePresentationState: "loading" | "error" | null;
  workflowState: WorkflowStateSnapshot | null;
  workflowStateError: string | null;
  workflowStateLoading: boolean;
  onActiveRightPanelChange: Dispatch<SetStateAction<RightPanelKey | null>>;
  onActiveViewChange: NavigateHandler;
  onOpenSecretaryDeepLink: (descriptor: SecretaryTypedDeepLinkDescriptor) => void;
  onReloadSecretaryHome: () => void | Promise<void>;
  onReloadWorkflowState: () => void;
}) {
  // 角标数 = 抽屉里真数得出来的条数（同一处派生，见 deriveRightPanelFeedCounts）。
  const feedCounts = deriveRightPanelFeedCounts({
    snapshot: displaySnapshot,
    workflowState,
    notice,
    error: error || Boolean(workflowStateError),
    memoryCaptureStore,
    memoryCandidateStore,
    secretaryContext,
  });

  return (
    <aside className="status-rail ink-shell" aria-label="工作台入口">
      <div className="right-icon-strip">
        {/* 秘书入口（07-09 定）：右侧栏图标 = 开侧边栏摘要（浮层）；dock「打开秘书」chip = 开看板；
            浮钮（.secretary-float）已撤开看板行为、留作将来「桌面宠物」角色位。 */}
        {workspaceRailItems.map((item) => {
          const badgeCount = railBadgeCount(item.key, feedCounts);
          const label = item.key === "secretary" ? "秘书摘要" : item.label;
          const labelWithCount = badgeCount > 0 ? `${label} ${badgeCount}` : label;
          return (
            <button
              className={`rail-icon-button ${activeRightPanel === item.key ? "active" : ""}`}
              key={item.key}
              type="button"
              title={labelWithCount}
              aria-label={item.key === "secretary" ? "打开侧边栏摘要" : labelWithCount}
              aria-expanded={activeRightPanel === item.key}
              onClick={() => onActiveRightPanelChange((current) => (current === item.key ? null : item.key))}
            >
              <span aria-hidden="true">{item.glyph}</span>
              {badgeCount > 0 ? <i className="rail-icon-badge" aria-hidden="true">{badgeCount}</i> : null}
            </button>
          );
        })}
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
          secretaryHome={secretaryHome}
          secretaryHomePresentationState={secretaryHomePresentationState}
          onClose={() => onActiveRightPanelChange(null)}
          onNavigate={onActiveViewChange}
          onOpenSecretaryDeepLink={onOpenSecretaryDeepLink}
          onReloadSecretaryHome={onReloadSecretaryHome}
          onReloadWorkflowState={onReloadWorkflowState}
        />
      ) : null}
    </aside>
  );
}

// 计数角标只挂宪法 §三.2「常显级」明列的那几类（干态进度 / 待批队列计数 / 系统健康异常），
// 正好=定稿 D rail 上画了角标的 知 / 待 / 行 三项。
// 「管」= 记账级（§三.3 默认沉默、可查）→ 不挂角标，定稿 D 也没画；
// 「秘」「想」无常显级依据 → 同样不挂。数为 0 时不渲染角标（没事就别占注意力）。
function railBadgeCount(
  key: RightPanelKey,
  counts: Record<Exclude<RightPanelKey, "secretary">, number>,
): number {
  if (key === "notifications") return counts.notifications;
  if (key === "todos") return counts.todos;
  if (key === "running") return counts.running;
  return 0;
}

function WorkbenchDock({
  onActiveRightPanelChange,
  onActiveViewChange,
}: {
  onActiveRightPanelChange: Dispatch<SetStateAction<RightPanelKey | null>>;
  onActiveViewChange: NavigateHandler;
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
          disabled
          placeholder="持续消息发送尚未接入"
          aria-label="持续 Secretary 消息发送尚未接入"
          aria-describedby="secretary-composer-unavailable"
        />
        <div className="dock-chips" aria-label="秘书快捷入口">
          <span id="secretary-composer-unavailable" className="dock-unavailable">消息发送未接入</span>
          <button className="chip send" type="button" onClick={() => onActiveViewChange("secretary_board")}>查看情境</button>
        </div>
      </div>
    </footer>
  );
}
