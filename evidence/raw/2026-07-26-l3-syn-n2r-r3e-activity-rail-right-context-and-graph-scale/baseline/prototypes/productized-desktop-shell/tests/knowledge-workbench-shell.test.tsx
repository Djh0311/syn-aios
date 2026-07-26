import React from "react";
import { renderToStaticMarkup } from "react-dom/server.browser";
import {
  KnowledgeBaseView,
  knowledgeDocumentMatchesRelativePath,
} from "../src/views/KnowledgeBaseView";
import { NativeKnowledgeWorkspace } from "../src/views/knowledge/NativeKnowledgeWorkspace";
import { KnowledgeWorkbenchShell } from "../src/views/knowledge/KnowledgeWorkbenchShell";
import {
  initialKnowledgeWorkbenchLayoutState,
  knowledgeWorkbenchOverlayDismissesForKey,
  reduceKnowledgeWorkbenchLayout,
  restoreKnowledgeWorkbenchOverlayFocus,
} from "../src/lib/knowledgeWorkbenchLayout";
import * as knowledgeWorkbenchLayoutContracts from "../src/lib/knowledgeWorkbenchLayout";
import * as knowledgeWorkspaceContracts from "../src/lib/knowledgeWorkspace";
import type { KnowledgeDocumentReadModel } from "../src/lib/knowledgeBase";
import type { PendingAction } from "../src/lib/types";
import { findElement, visibleText } from "./helpers/offlineInteractionTestUtils";
import { knowledgeBaseBoundaryFixtures } from "./helpers/offlineKnowledgeSecretaryFixtures";
import { offlineScenarioEnvironmentFixtures } from "./helpers/offlineScenarioEnvironmentFixtures";

const nodeProcess = (globalThis as typeof globalThis & { process?: { cwd?: () => string } }).process;
if (!nodeProcess?.cwd) throw new Error("[knowledge-workbench-shell] 离线合同需要 Node cwd 才能读取同项目样式");
const nodeFsSpecifier: string = "node:fs";
const nodeFs = await import(nodeFsSpecifier) as { readFileSync: (path: string, encoding: "utf8") => string };
const stylesheet = nodeFs.readFileSync(`${nodeProcess.cwd()}/src/styles.css`, "utf8");

function scopedMediaBlock(css: string, maxWidth: number): string {
  const marker = `@media (max-width: ${maxWidth}px)`;
  const start = css.indexOf(marker);
  const openingBrace = css.indexOf("{", start);
  if (start < 0 || openingBrace < 0) return "";
  let depth = 0;
  for (let index = openingBrace; index < css.length; index += 1) {
    if (css[index] === "{") depth += 1;
    if (css[index] === "}") depth -= 1;
    if (depth === 0) return css.slice(start, index + 1);
  }
  return "";
}

function assertWorkbenchContract(condition: boolean, message: string, failures: string[]) {
  if (!condition) failures.push(message);
}

function count(markup: string, marker: string): number {
  return markup.split(marker).length - 1;
}

const markup = renderToStaticMarkup(
  <KnowledgeBaseView
    projects={[]}
    workflowState={null}
    hasRealSnapshot={false}
    onRequestAction={() => {}}
  />,
);

const failures: string[] = [];
const r3bFailures: string[] = [];
let r3bAssertionCount = 0;

function assertR3bContract(test: () => boolean, message: string) {
  r3bAssertionCount += 1;
  let passed = false;
  try {
    passed = test();
  } catch {
    passed = false;
  }
  if (!passed) {
    r3bFailures.push(message);
    failures.push(message);
  }
}

assertWorkbenchContract(
  count(markup, 'data-knowledge-shell="syn-workbench"') === 1,
  "知识库必须只渲染一个 Syn 单壳根",
  failures,
);

for (const region of ["activity", "left", "central", "right", "status"]) {
  assertWorkbenchContract(
    count(markup, `data-knowledge-region="${region}"`) === 1,
    `单壳必须唯一包含 ${region} 区域`,
    failures,
  );
}

assertWorkbenchContract(!markup.includes('class="pg-head"'), "页面级 pg-head 必须从知识主页面退场", failures);
assertWorkbenchContract(!markup.includes("knowledge-base-stats"), "旧统计条不能继续作为主页面容器", failures);
assertWorkbenchContract(!markup.includes("knowledge-base-grid"), "旧知识三栏不能继续作为主页面容器", failures);
assertWorkbenchContract(!markup.includes("knowledge-vault-notes"), "第二套 Vault Notes 编辑器不能继续常驻", failures);
assertWorkbenchContract(!markup.includes("native-knowledge-graph"), "Graph 不能在静态单壳根级纵向追加", failures);
assertWorkbenchContract(!markup.includes("native-knowledge-canvas"), "Canvas 不能在静态单壳根级纵向追加", failures);
assertWorkbenchContract(!markup.includes("native-knowledge-maintenance"), "维护区不能在静态单壳根级纵向追加", failures);
assertWorkbenchContract(markup.includes("快速打开") && markup.includes("Syn 命令"), "临时 overlay 的入口必须保留", failures);
assertWorkbenchContract(markup.includes("Markdown 源码") && markup.includes("渲染预览"), "中央工作区必须保留 Markdown 与预览模式", failures);

function renderShellSurface(
  surface: string,
  {
    leftCollapsed = false,
    rightCollapsed = false,
    overlay = null,
  }: { leftCollapsed?: boolean; rightCollapsed?: boolean; overlay?: React.ReactNode } = {},
): string {
  return renderToStaticMarkup(
    <KnowledgeWorkbenchShell
      activityRail={<button type="button">文件</button>}
      leftSidebar={<button type="button" data-knowledge-sidebar-control="left">左栏可聚焦控件</button>}
      centralWorkspace={<div data-central-surface={surface}>{surface}</div>}
      rightSidebar={<button type="button" data-knowledge-sidebar-control="right">右栏可聚焦控件</button>}
      statusBar={<span>状态</span>}
      leftCollapsed={leftCollapsed}
      rightCollapsed={rightCollapsed}
      overlay={overlay}
    />,
  );
}

for (const surface of ["Markdown", "Graph", "Canvas"]) {
  const surfaceMarkup = renderShellSurface(surface);
  const centralStart = surfaceMarkup.indexOf('data-knowledge-region="central"');
  const centralEnd = surfaceMarkup.indexOf("</section>", centralStart);
  const surfaceMarker = surfaceMarkup.indexOf(`data-central-surface="${surface}"`);
  assertWorkbenchContract(
    count(surfaceMarkup, 'data-knowledge-shell="syn-workbench"') === 1
      && count(surfaceMarkup, 'data-knowledge-region="central"') === 1
      && centralStart >= 0
      && centralEnd > centralStart
      && surfaceMarker > centralStart
      && surfaceMarker < centralEnd,
    `${surface} 必须在唯一中央槽位渲染，不能成为根级兄弟`,
    failures,
  );
}

const collapsedMarkup = renderShellSurface("Markdown", { leftCollapsed: true });
assertWorkbenchContract(
  collapsedMarkup.includes("is-left-collapsed")
    && count(collapsedMarkup, 'data-knowledge-region="activity"') === 1
    && count(collapsedMarkup, 'data-knowledge-region="left"') === 1,
  "左栏折叠后活动栏和唯一结构槽位必须仍然存在",
  failures,
);

const overlayMarkup = renderShellSurface("Markdown", {
  overlay: <div role="dialog" aria-label="快速打开">临时快速打开</div>,
});
assertWorkbenchContract(
  overlayMarkup.includes('role="dialog"')
    && overlayMarkup.indexOf('role="dialog"') < overlayMarkup.lastIndexOf("</section>"),
  "快速打开必须作为唯一壳内的临时 overlay，而非主页面常驻面板",
  failures,
);

const knowledgeShellCssStart = stylesheet.indexOf("/* N2R-R1：唯一桌面知识壳");
const knowledgeShellCssEnd = stylesheet.indexOf("/* L3 N3：", knowledgeShellCssStart);
const knowledgeShellCss = stylesheet.slice(knowledgeShellCssStart, knowledgeShellCssEnd);
const mediumKnowledgeShellCss = scopedMediaBlock(knowledgeShellCss, 1180);
const narrowKnowledgeShellCss = scopedMediaBlock(knowledgeShellCss, 900);
assertWorkbenchContract(
  !/\.syn-knowledge-shell\s*\{[^}]*grid-template-columns:[^}]*\s0;/.test(mediumKnowledgeShellCss)
    && mediumKnowledgeShellCss.includes("--syn-knowledge-right-track"),
  "1180px 断点不能无条件把已展开右栏压成 0，必须保留由壳状态控制的非零轨宽",
  failures,
);
assertWorkbenchContract(
  !narrowKnowledgeShellCss.includes(".syn-knowledge-shell,")
    && !/\.syn-knowledge-shell\s*\{[^}]*grid-template-columns:/.test(narrowKnowledgeShellCss)
    && narrowKnowledgeShellCss.includes("--syn-knowledge-left-track")
    && narrowKnowledgeShellCss.includes("--syn-knowledge-right-track"),
  "900px 断点不能覆盖全部 reducer 状态为双零轨，必须只收紧由壳状态消费的左右轨宽",
  failures,
);

const leftCollapsedAccessibilityMarkup = renderShellSurface("Markdown", { leftCollapsed: true });
assertWorkbenchContract(
  leftCollapsedAccessibilityMarkup.includes('data-knowledge-region="activity"')
    && leftCollapsedAccessibilityMarkup.includes('aria-hidden="true"')
    && leftCollapsedAccessibilityMarkup.includes("inert=\"\"")
    && !leftCollapsedAccessibilityMarkup.includes('data-knowledge-sidebar-control="left"')
    && renderShellSurface("Markdown").includes('data-knowledge-sidebar-control="left"'),
  "折叠左栏必须退出 Tab/辅助技术路径，同时活动栏仍可恢复其可聚焦内容",
  failures,
);
const rightCollapsedAccessibilityMarkup = renderShellSurface("Markdown", { rightCollapsed: true });
assertWorkbenchContract(
  rightCollapsedAccessibilityMarkup.includes('data-knowledge-region="activity"')
    && rightCollapsedAccessibilityMarkup.includes('aria-hidden="true"')
    && rightCollapsedAccessibilityMarkup.includes("inert=\"\"")
    && !rightCollapsedAccessibilityMarkup.includes('data-knowledge-sidebar-control="right"')
    && renderShellSurface("Markdown").includes('data-knowledge-sidebar-control="right"'),
  "折叠右栏必须退出 Tab/辅助技术路径，同时活动栏仍可恢复其可聚焦内容",
  failures,
);

const collapsedLayout = reduceKnowledgeWorkbenchLayout(initialKnowledgeWorkbenchLayoutState, { type: "collapse-left" });
const reopenedLeftLayout = reduceKnowledgeWorkbenchLayout(collapsedLayout, {
  type: "select-left-view",
  view: "sources",
});
const openedOverlayLayout = reduceKnowledgeWorkbenchLayout(reopenedLeftLayout, {
  type: "open-overlay",
  overlay: "quick-open",
});
const closedOverlayLayout = reduceKnowledgeWorkbenchLayout(openedOverlayLayout, { type: "close-overlay" });
let restoredFocusCount = 0;
let scheduledRestoreCount = 0;
restoreKnowledgeWorkbenchOverlayFocus(
  { focus: () => { restoredFocusCount += 1; } } as HTMLButtonElement,
  (callback) => {
    scheduledRestoreCount += 1;
    callback();
    return 0;
  },
);

assertWorkbenchContract(
  collapsedLayout.leftCollapsed
    && reopenedLeftLayout.leftView === "sources"
    && !reopenedLeftLayout.leftCollapsed
    && openedOverlayLayout.overlay === "quick-open"
    && closedOverlayLayout.overlay === null,
  "左栏折叠/恢复和 overlay 开关必须共用可验证的单壳状态模型",
  failures,
);
assertWorkbenchContract(
  knowledgeWorkbenchOverlayDismissesForKey("Escape")
    && !knowledgeWorkbenchOverlayDismissesForKey("Enter")
    && scheduledRestoreCount === 1
    && restoredFocusCount === 1,
  "overlay 必须只以 Escape 关闭，并在关闭后恢复到触发控件",
  failures,
);

// R3A red-first contract: the UI must consume these helpers for the actual
// keyboard paths; browser evidence below remains the authority for hooks,
// IPC, focus and pixel overflow. Keeping this pure contract here fixes the
// Arrow boundary semantics without turning a source-string scan into a green.
const r3aLayoutContracts = knowledgeWorkbenchLayoutContracts as {
  knowledgeWorkbenchListActionForKey?: (key: string) => "previous" | "next" | "activate" | "dismiss" | null;
  knowledgeWorkbenchMoveListSelection?: (currentIndex: number, itemCount: number, action: "previous" | "next") => number;
  knowledgeWorkbenchShortcutTarget?: (input: Readonly<{ key: string; metaKey: boolean; shiftKey: boolean; altKey?: boolean; ctrlKey?: boolean }>) => "quick-open" | "command" | "left-search" | null;
};
const r3aListActionForKey = r3aLayoutContracts.knowledgeWorkbenchListActionForKey;
const r3aMoveListSelection = r3aLayoutContracts.knowledgeWorkbenchMoveListSelection;
const r3aShortcutTarget = r3aLayoutContracts.knowledgeWorkbenchShortcutTarget;

assertWorkbenchContract(
  typeof r3aListActionForKey === "function" && typeof r3aMoveListSelection === "function",
  "R3A 的 Search/quick-open/command 必须共用可测试的 Arrow、Enter、Escape 列表键盘合同",
  failures,
);
if (r3aListActionForKey && r3aMoveListSelection) {
  assertWorkbenchContract(
    r3aListActionForKey("ArrowUp") === "previous"
      && r3aListActionForKey("ArrowDown") === "next"
      && r3aListActionForKey("Enter") === "activate"
      && r3aListActionForKey("Escape") === "dismiss"
      && r3aListActionForKey("Tab") === null,
    "R3A 列表键盘只把 Arrow、Enter、Escape 映射为导航、激活和关闭",
    failures,
  );
  assertWorkbenchContract(
    r3aMoveListSelection(-1, 0, "next") === -1
      && r3aMoveListSelection(-1, 3, "next") === 0
      && r3aMoveListSelection(0, 3, "previous") === 0
      && r3aMoveListSelection(2, 3, "next") === 2,
    "R3A Arrow 选择必须对空列表和首尾采用可重复的钳制边界",
    failures,
  );
}
assertWorkbenchContract(
  typeof r3aShortcutTarget === "function",
  "R3A 必须用同一快捷键合同路由 ⌘O、⌘P 和 ⌘⇧F",
  failures,
);
if (r3aShortcutTarget) {
  assertWorkbenchContract(
    r3aShortcutTarget({ key: "o", metaKey: true, shiftKey: false }) === "quick-open"
      && r3aShortcutTarget({ key: "p", metaKey: true, shiftKey: false }) === "command"
      && r3aShortcutTarget({ key: "f", metaKey: true, shiftKey: true }) === "left-search"
      && r3aShortcutTarget({ key: "f", metaKey: false, shiftKey: true }) === null,
    "R3A 快捷键只路由已冻结的 ⌘O、⌘P、⌘⇧F，不拦截普通输入",
    failures,
  );
}
type R3bTab = Readonly<{
  id: string;
  kind: "markdown" | "surface";
  relativePath?: string;
  projection?: "source" | "preview";
  surface?: "graph" | "canvas" | "maintenance";
}>;
type R3bGroup = Readonly<{
  id: string;
  tabs: ReadonlyArray<R3bTab>;
  activeTabId: string | null;
}>;
type R3bCentralState = Readonly<{
  groups: ReadonlyArray<R3bGroup>;
  activeGroupId: string;
  splitRatio: number;
}>;
type R3bCentralAction =
  | Readonly<{ type: "open-markdown"; relativePath: string; groupId?: string; projection?: "source" | "preview" }>
  | Readonly<{ type: "open-surface"; surface: "graph" | "canvas" | "maintenance"; groupId?: string }>
  | Readonly<{ type: "activate-tab"; groupId: string; tabId: string }>
  | Readonly<{ type: "close-tab"; groupId: string; tabId: string }>
  | Readonly<{ type: "split-right" }>
  | Readonly<{ type: "merge-groups" }>
  | Readonly<{ type: "set-split-ratio"; ratio: number }>
  | Readonly<{ type: "set-projection"; groupId: string; projection: "source" | "preview" }>;

const r3bLayoutContracts = knowledgeWorkbenchLayoutContracts as typeof knowledgeWorkbenchLayoutContracts & {
  createKnowledgeWorkbenchCentralState?: (input?: Readonly<{
    markdownTabs?: ReadonlyArray<string>;
    selectedRelativePath?: string | null;
    projection?: "source" | "preview";
  }>) => R3bCentralState;
  reduceKnowledgeWorkbenchCentralState?: (state: R3bCentralState, action: R3bCentralAction) => R3bCentralState;
  knowledgeWorkbenchActiveTab?: (state: R3bCentralState, groupId?: string) => R3bTab | null;
  knowledgeWorkbenchCloseFocusTarget?: (group: R3bGroup, tabId: string) => Readonly<
    { kind: "tab"; tabId: string } | { kind: "group-tools" }
  >;
  knowledgeWorkbenchTabShortcutTarget?: (input: Readonly<{
    key: string;
    metaKey: boolean;
    shiftKey: boolean;
    altKey?: boolean;
    ctrlKey?: boolean;
  }>) => "quick-open" | "next-tab" | "previous-tab" | null;
  knowledgeWorkbenchSplitRatioForKey?: (ratio: number, key: string) => number;
};
const createR3bCentralState = r3bLayoutContracts.createKnowledgeWorkbenchCentralState;
const reduceR3bCentralState = r3bLayoutContracts.reduceKnowledgeWorkbenchCentralState;
const r3bActiveTab = r3bLayoutContracts.knowledgeWorkbenchActiveTab;
const r3bCloseFocusTarget = r3bLayoutContracts.knowledgeWorkbenchCloseFocusTarget;
const r3bTabShortcutTarget = r3bLayoutContracts.knowledgeWorkbenchTabShortcutTarget;
const r3bSplitRatioForKey = r3bLayoutContracts.knowledgeWorkbenchSplitRatioForKey;

const r3bBaseState = createR3bCentralState?.({
  markdownTabs: ["notes/a.md", "notes/b.md", "notes/c.md"],
  selectedRelativePath: "notes/b.md",
  projection: "source",
}) ?? null;
const r3bSplitState = r3bBaseState && reduceR3bCentralState
  ? reduceR3bCentralState(r3bBaseState, { type: "split-right" })
  : null;
const r3bResizedState = r3bSplitState && reduceR3bCentralState
  ? reduceR3bCentralState(r3bSplitState, { type: "set-split-ratio", ratio: 60 })
  : null;
const r3bMergedState = r3bResizedState && reduceR3bCentralState
  ? reduceR3bCentralState(r3bResizedState, { type: "merge-groups" })
  : null;

assertR3bContract(
  () => Boolean(
    r3bBaseState
    && r3bBaseState.groups.length === 1
    && r3bBaseState.activeGroupId === r3bBaseState.groups[0]?.id
    && (() => {
      const active = r3bActiveTab?.(r3bBaseState);
      return active?.kind === "markdown" && active.relativePath === "notes/b.md";
    })(),
  ),
  "R3B 单组必须有稳定当前组、顺序 Markdown 标签和唯一当前标签",
);
assertR3bContract(
  () => Boolean(
    r3bSplitState
    && r3bSplitState.groups.length === 2
    && r3bSplitState.groups[0]?.tabs.some((tab) => tab.kind === "markdown" && tab.relativePath === "notes/b.md" && tab.projection === "source")
    && r3bSplitState.groups[1]?.tabs.some((tab) => tab.kind === "markdown" && tab.relativePath === "notes/b.md" && tab.projection === "preview"),
  ),
  "R3B 向右分栏必须把同一 Markdown 标签变为左源码、右预览两个投影",
);
assertR3bContract(
  () => Boolean(
    r3bResizedState
    && r3bResizedState.splitRatio === 60
    && r3bMergedState?.groups.length === 1
    && r3bMergedState.groups[0]?.tabs.some((tab) => tab.kind === "markdown" && tab.relativePath === "notes/b.md"),
  ),
  "R3B 必须支持 50→60 调比例并在合并后保留当前 Markdown 标签",
);
assertR3bContract(
  () => Boolean(
    r3bSplitState
    && reduceR3bCentralState
    && reduceR3bCentralState(r3bSplitState, { type: "split-right" }).groups.length === 2
    && reduceR3bCentralState(r3bSplitState, { type: "set-split-ratio", ratio: Number.NaN }).splitRatio === r3bSplitState.splitRatio,
  ),
  "R3B 第三组与非有限 ratio 必须 fail closed",
);
assertR3bContract(
  () => Boolean(
    r3bBaseState
    && reduceR3bCentralState
    && reduceR3bCentralState(r3bBaseState, { type: "open-markdown", relativePath: "notes/b.md" })
      .groups.flatMap((group) => group.tabs)
      .filter((tab) => tab.kind === "markdown" && tab.relativePath === "notes/b.md").length === 1,
  ),
  "R3B 打开已存在 Markdown 必须聚焦既有标签而非复制",
);
assertR3bContract(
  () => Boolean(
    r3bBaseState
    && reduceR3bCentralState
    && (() => {
      const activeGroup = r3bBaseState.groups[0];
      if (!activeGroup) return false;
      const activeTabId = activeGroup.activeTabId;
      if (!activeTabId) return false;
      const closed = reduceR3bCentralState(r3bBaseState, {
        type: "close-tab",
        groupId: activeGroup.id,
        tabId: activeTabId,
      });
      const nextActive = r3bActiveTab?.(closed);
      return nextActive?.kind === "markdown"
        && nextActive.relativePath === "notes/c.md"
        && r3bCloseFocusTarget?.(activeGroup, activeTabId).kind === "tab";
    })(),
  ),
  "R3B 关闭当前标签必须选择右邻居并提供确定性焦点目标",
);
assertR3bContract(
  () => Boolean(
    createR3bCentralState
    && reduceR3bCentralState
    && (() => {
      const only = createR3bCentralState({
        markdownTabs: ["notes/only.md"],
        selectedRelativePath: "notes/only.md",
        projection: "source",
      });
      const group = only.groups[0];
      const tabId = group?.activeTabId;
      if (!group || !tabId) return false;
      const empty = reduceR3bCentralState(only, { type: "close-tab", groupId: group.id, tabId });
      return empty.groups[0]?.activeTabId === null
        && r3bCloseFocusTarget?.(group, tabId).kind === "group-tools";
    })(),
  ),
  "R3B 关闭最后标签必须进入真实空态并把焦点交给组工具",
);
assertR3bContract(
  () => Boolean(
    r3bBaseState
    && reduceR3bCentralState
    && (() => {
      const once = reduceR3bCentralState(r3bBaseState, { type: "open-surface", surface: "graph" });
      const twice = reduceR3bCentralState(once, { type: "open-surface", surface: "graph" });
      const active = r3bActiveTab?.(twice);
      return twice.groups.flatMap((group) => group.tabs).filter((tab) => tab.kind === "surface" && tab.surface === "graph").length === 1
        && active?.kind === "surface"
        && active.surface === "graph";
    })(),
  ),
  "R3B Graph/Canvas/Maintenance 必须作为组内 singleton tab，而非根级兄弟",
);
assertR3bContract(
  () => Boolean(
    r3bTabShortcutTarget
    && r3bTabShortcutTarget({ key: "t", metaKey: true, shiftKey: false, ctrlKey: false }) === "quick-open"
    && r3bTabShortcutTarget({ key: "Tab", metaKey: false, shiftKey: false, ctrlKey: true }) === "next-tab"
    && r3bTabShortcutTarget({ key: "Tab", metaKey: false, shiftKey: true, ctrlKey: true }) === "previous-tab"
    && r3bTabShortcutTarget({ key: "t", metaKey: true, shiftKey: true, ctrlKey: false }) === null
    && r3bTabShortcutTarget({ key: "t", metaKey: false, shiftKey: false, ctrlKey: false }) === null,
  ),
  "R3B 只路由 ⌘T 与 ⌃Tab，不误拦普通输入、⌘⇧T 或无关组合键",
);
assertR3bContract(
  () => Boolean(
    r3bSplitRatioForKey
    && r3bSplitRatioForKey(50, "ArrowRight") === 55
    && r3bSplitRatioForKey(50, "ArrowLeft") === 45
    && r3bSplitRatioForKey(30, "ArrowLeft") === 30
    && r3bSplitRatioForKey(70, "ArrowRight") === 70
    && r3bSplitRatioForKey(50, "Enter") === 50,
  ),
  "R3B 分隔器键盘只响应左右箭头并把比例限制在 30–70",
);

type R3bUiPreferences = Readonly<{
  version: 2;
  selectedRelativePath: string | null;
  centralState: R3bCentralState;
}>;
const r3bWorkspaceContracts = knowledgeWorkspaceContracts as typeof knowledgeWorkspaceContracts & {
  normalizeKnowledgeWorkspaceCentralUiPreferences?: (value: unknown) => R3bUiPreferences;
  loadKnowledgeWorkspaceCentralUiPreferences?: (
    storage: Readonly<{ getItem: (key: string) => string | null }> | null,
  ) => R3bUiPreferences;
  saveKnowledgeWorkspaceCentralUiPreferences?: (
    storage: Readonly<{ setItem: (key: string, value: string) => void }> | null,
    value: unknown,
  ) => R3bUiPreferences;
  knowledgeWorkspaceCentralTransitionDisposition?: (input: Readonly<{
    draftIsDirty: boolean;
    needsReload: boolean;
    intent: "switch-markdown" | "close-markdown" | "merge-groups";
    wouldDiscardLastDraft?: boolean;
  }>) => "apply" | "preserve";
};
const normalizeR3bPreferences = r3bWorkspaceContracts.normalizeKnowledgeWorkspaceCentralUiPreferences
  ?? (() => ({ version: 1 }) as unknown as R3bUiPreferences);
const loadR3bPreferences = r3bWorkspaceContracts.loadKnowledgeWorkspaceCentralUiPreferences;
const saveR3bPreferences = r3bWorkspaceContracts.saveKnowledgeWorkspaceCentralUiPreferences;
const r3bTransitionDisposition = r3bWorkspaceContracts.knowledgeWorkspaceCentralTransitionDisposition;
const migratedR3bPreferences = normalizeR3bPreferences({
  version: 1,
  tabs: ["notes/a.md", "notes/a.md", "../unsafe.md", "notes/b.md"],
  selectedRelativePath: "notes/a.md",
  viewMode: "split",
});
const normalizedR3bPreferences = normalizeR3bPreferences({
  version: 2,
  selectedRelativePath: "notes/safe.md",
  centralState: {
    groups: [
      {
        id: "knowledge-group-primary",
        activeTabId: "markdown:notes/safe.md",
        tabs: [
          { id: "markdown:notes/safe.md", kind: "markdown", relativePath: "notes/safe.md", projection: "source" },
          { id: "markdown:../unsafe.md", kind: "markdown", relativePath: "../unsafe.md", projection: "preview" },
        ],
      },
      {
        id: "knowledge-group-secondary",
        activeTabId: "surface:unknown",
        tabs: [{ id: "surface:unknown", kind: "surface", surface: "unknown" }],
      },
      {
        id: "knowledge-group-third",
        activeTabId: null,
        tabs: [],
      },
    ],
    activeGroupId: "knowledge-group-third",
    splitRatio: Number.NaN,
  },
});
assertR3bContract(
  () => migratedR3bPreferences.version === 2
    && migratedR3bPreferences.centralState.groups.length === 2
    && migratedR3bPreferences.centralState.splitRatio === 50
    && migratedR3bPreferences.centralState.groups.flatMap((group) => group.tabs)
      .filter((tab) => tab.kind === "markdown" && tab.relativePath === "notes/a.md").length === 2,
  "R3B 必须把旧 v1 split 偏好确定性迁移成同一路径的左右源码/预览组",
);
assertR3bContract(
  () => normalizedR3bPreferences.version === 2
    && normalizedR3bPreferences.centralState.groups.length <= 2
    && normalizedR3bPreferences.centralState.splitRatio === 50
    && normalizedR3bPreferences.centralState.groups.flatMap((group) => group.tabs)
      .every((tab) => tab.kind !== "markdown" || tab.relativePath !== "../unsafe.md")
    && normalizedR3bPreferences.centralState.groups.flatMap((group) => group.tabs)
      .every((tab) => tab.kind !== "surface" || tab.surface !== ("unknown" as never)),
  "R3B 新偏好必须归一非法组数、不安全路径、未知 surface、activeGroup 与非有限 ratio",
);
assertR3bContract(
  () => Boolean(
    r3bTransitionDisposition
    && r3bTransitionDisposition({ draftIsDirty: true, needsReload: false, intent: "switch-markdown" }) === "preserve"
    && r3bTransitionDisposition({ draftIsDirty: false, needsReload: true, intent: "close-markdown" }) === "preserve"
    && r3bTransitionDisposition({ draftIsDirty: true, needsReload: false, intent: "merge-groups", wouldDiscardLastDraft: true }) === "preserve"
    && r3bTransitionDisposition({ draftIsDirty: true, needsReload: false, intent: "merge-groups", wouldDiscardLastDraft: false }) === "apply",
  ),
  "R3B 切换/关闭/丢掉最后草稿投影必须复用 dirty/conflict fail-closed，安全合并可保留草稿",
);
assertR3bContract(
  () => {
    const throwingStorage = {
      getItem: () => { throw new Error("storage disabled"); },
      setItem: () => { throw new Error("storage disabled"); },
    };
    const loaded = loadR3bPreferences?.(throwingStorage);
    const saved = saveR3bPreferences?.(throwingStorage, migratedR3bPreferences);
    return loaded?.version === 2 && saved?.version === 2;
  },
  "R3B UI 偏好读写失败必须继续 fail-soft",
);

const sourcePathDocument = {
  source_anchor: { path_summary: "docs/interface-contract.md" },
} as KnowledgeDocumentReadModel;
assertWorkbenchContract(
  knowledgeDocumentMatchesRelativePath(sourcePathDocument, "docs/interface-contract.md")
    && knowledgeDocumentMatchesRelativePath(sourcePathDocument, "research/docs/interface-contract.md")
    && !knowledgeDocumentMatchesRelativePath(sourcePathDocument, "research/unrelated.md")
    && !knowledgeDocumentMatchesRelativePath(sourcePathDocument, "docs/interface-contract.md.bak"),
  "右侧来源上下文只能接收当前笔记的映射资料，不能常驻展示无关项目来源",
  failures,
);

const environment = offlineScenarioEnvironmentFixtures();
const knowledgeFixtures = knowledgeBaseBoundaryFixtures(
  environment.project,
  environment.workflowStateWithDerivedWorkflow,
);
let capturedCandidateAction: PendingAction | null = null;
const sourceCandidateView = (
  <KnowledgeBaseView
    projects={[knowledgeFixtures.projectWithKnowledge]}
    workflowState={knowledgeFixtures.knowledgeWorkflowState}
    formalMemoryStore={knowledgeFixtures.formalMemoryStore}
    memoryCandidateStore={knowledgeFixtures.memoryCandidateStore}
    hasRealSnapshot={true}
    onRequestAction={(action) => { capturedCandidateAction = action; }}
  />
);
const sourceCandidateMarkup = renderToStaticMarkup(sourceCandidateView);
const sourceCandidateButton = findElement(
  sourceCandidateView,
  (element) => element.type === "button" && visibleText(element).includes("提出记忆候选"),
);
const sourceCandidateClick = sourceCandidateButton?.props?.onClick;
assertWorkbenchContract(
  sourceCandidateMarkup.includes('aria-label="知识来源"')
    && sourceCandidateMarkup.includes("提出记忆候选")
    && typeof sourceCandidateClick === "function",
  "旧来源和候选动作必须迁入来源视图，不能随旧三栏退场",
  failures,
);
if (typeof sourceCandidateClick === "function") sourceCandidateClick();
assertWorkbenchContract(
  (capturedCandidateAction as PendingAction | null)?.kind === "create-memory-candidate",
  "来源视图中的候选动作必须继续产生受控 create-memory-candidate action",
  failures,
);

function renderBrowserNativeShell(): string {
  const windowDescriptor = Object.getOwnPropertyDescriptor(globalThis, "window");
  Object.defineProperty(globalThis, "window", {
    configurable: true,
    writable: true,
    value: {
      localStorage: { getItem: () => null, setItem: () => {} },
      setTimeout: () => 0,
    },
  });
  try {
    return renderToStaticMarkup(<NativeKnowledgeWorkspace />);
  } finally {
    if (windowDescriptor) Object.defineProperty(globalThis, "window", windowDescriptor);
    else delete (globalThis as unknown as { window?: unknown }).window;
  }
}

const nativeStaticMarkup = renderToStaticMarkup(<NativeKnowledgeWorkspace />);
const nativeBrowserMarkup = renderBrowserNativeShell();
for (const branchMarkup of [nativeStaticMarkup, nativeBrowserMarkup]) {
  assertWorkbenchContract(
    count(branchMarkup, 'data-knowledge-shell="syn-workbench"') === 1
      && ["activity", "left", "central", "right", "status"].every(
        (region) => count(branchMarkup, `data-knowledge-region="${region}"`) === 1,
      ),
    "static/server 与 browser 初始渲染必须同为唯一五区单壳",
    failures,
  );
}
assertR3bContract(
  () => [nativeStaticMarkup, nativeBrowserMarkup].every((branchMarkup) => (
    !branchMarkup.includes("syn-knowledge-central-tabs")
    && !branchMarkup.includes("native-workspace-document--split")
    && count(branchMarkup, 'data-knowledge-tab-group="knowledge-group-primary"') === 1
    && count(branchMarkup, "data-knowledge-group-tablist") === 1
  )),
  "R3B static/browser 初始分支必须只有一组一排真实中央标签，旧全局模式排和内部 split 退场",
);
assertR3bContract(
  () => nativeBrowserMarkup.includes('role="tablist"')
    && nativeBrowserMarkup.includes('aria-label="主标签组"')
    && nativeBrowserMarkup.includes('aria-label="在主标签组中快速打开"'),
  "R3B 中央组必须暴露真实 tablist 和非 tab 的快速打开组工具",
);

if (failures.length) {
  throw new Error(
    `[knowledge-workbench-shell] R3B ${r3bAssertionCount} 项 / ${r3bFailures.length} 失败；全部失败 ${failures.length}：${failures.join("；")}`,
  );
}

console.log(`knowledge workbench shell static convergence contract passed; R3B ${r3bAssertionCount} / 0`);
