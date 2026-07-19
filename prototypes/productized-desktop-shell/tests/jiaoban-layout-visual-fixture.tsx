import { useEffect, useMemo, useState } from "react";
import ReactDOM from "react-dom/client";
import type { ProjectConsultationProposal, RunHistoryEntry } from "../src/lib/types";
import { JiaobanMergedLayout } from "../src/views/projects/ProjectWorkspaceShell";
import { JiaobanAuthorizeState } from "../src/views/projects/jiaoban/JiaobanAuthorizeStates";
import {
  JiaobanConversationComposer,
  JiaobanConversationStream,
  type JiaobanConversationArtifactNotice,
  type JiaobanConversationUserTurn,
} from "../src/views/projects/jiaoban/JiaobanConversation";
import { JiaobanHistoryDetail, JiaobanProposalIndex } from "../src/views/projects/jiaoban/JiaobanHistory";
import "../src/styles.css";
import "../src/manualRelay.css";
import "../src/components/sourceStylePlaceholder.css";
import "../src/views/memory/memoryCenter.css";
import "../src/views/projects/projectWorkflowSidePanel.css";
import "../src/views/projects/projectReferencePanels.css";

const noop = () => {};
const currentProposalId = "proposal-current";
const baseTime = Date.UTC(2026, 6, 19, 9, 30);

const proposal: ProjectConsultationProposal = {
  proposal_id: currentProposalId,
  schema_version: "project_consultation_proposal.v1",
  project_id: "project:visual-fixture",
  workflow_id: "workflow:visual-fixture",
  title: "重排交办页",
  user_goal: "把左侧历史栏放到右侧方案卡左边，作为方案列表。",
  goal_summary: "交办页改成左侧持续对话、中间历届方案、右侧方案实体",
  proposed_steps: [
    "目标文件：src/views/projects/ProjectWorkspaceShell.tsx",
    "目标文件：src/views/projects/jiaoban/JiaobanHistory.tsx",
    "目标文件：src/views/projects/projectWorkflowSidePanel.css",
  ],
  scope_draft: {
    allowed_role_ids: [],
    allowed_agent_ids: [],
    allowed_read_roots: ["/Users/yoyi/workspace/product-line"],
    allowed_write_roots: ["/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell"],
    allowed_tools: ["apply_patch", "playwright"],
    allowed_checks: ["pnpm typecheck", "pnpm test:offline-interaction"],
    allowed_task_package_kinds: [],
    stop_conditions: ["不改主管传输 Rust 路径"],
  },
  risks: [],
  worker_acceptance_criteria: [
    "1280px 下按对话、历届方案、方案实体从左到右排列",
    "577px 下按同一阅读顺序纵排，页面壳不滚",
    "点历届方案只切换右侧实体，不滚动左侧对话",
  ],
  control_core_acceptance_criteria: ["既有九态只读语义不变"],
  supervisor_acceptance_criteria: ["真实浏览器量尺与截图留证"],
  acceptance_criteria: ["pnpm typecheck", "pnpm test:offline-interaction"],
  status: "pending_user_confirmation",
  created_by_role: "project_consultant",
  created_at_ms: baseTime,
  updated_at_ms: baseTime,
  suggest_workflow: false,
};

const historyStates = [
  ["pending", "等你确认"],
  ["running", "主管正在推进"],
  ["delivered", "已交货"],
  ["blocked", "等待补充边界"],
  ["advice_only", "只读建议已给出"],
  ["confirmed_not_run", "已确认但未运行"],
  ["changes_requested", "按新要求重出"],
  ["declined", "用户先不做"],
  ["superseded", "已有新方案替代"],
] as const;

const historyEntries: RunHistoryEntry[] = Array.from({ length: 15 }, (_, index) => {
  const [state, stateNote] = historyStates[index % historyStates.length];
  const isCurrent = index === 0;
  const isLegacy = index === 13;
  return {
    proposal_id: isCurrent ? currentProposalId : isLegacy ? "proposal-legacy" : `proposal-history-${index}`,
    workflow_id: "workflow:visual-fixture",
    goal_text: isCurrent
      ? "交办页改成左侧对话、中间历届方案、右侧方案实体"
      : [
          "核对主管常驻会话的续接边界",
          "让运行过程短讯回到对话流",
          "收口方案卡上的工程详情",
          "补齐交货态的主管复核说明",
          "核验固定测试项目的只读边界",
        ][index % 5],
    created_at_ms: isLegacy ? Date.UTC(2025, 11, 18, 8, 0) : baseTime - index * 86_400_000,
    state: isCurrent ? "pending" : state,
    state_note: isCurrent ? "等你确认" : stateNote,
    advice_only: state === "advice_only",
    review_flags: state === "delivered" && index % 2 === 0 ? { result_verdict: "pass" } : {},
    correlation: "exact",
  };
});

const knownProposalIds = new Set(historyEntries.map((entry) => entry.proposal_id).filter((id) => id !== "proposal-legacy"));

const userTurns: JiaobanConversationUserTurn[] = Array.from({ length: 9 }, (_, index) => ({
  id: `visual-user-turn-${index}`,
  text: [
    "先核对当前计划，别越过主管传输这一闸。",
    "历史栏不要再控制对话位置，它应该成为方案索引。",
    "中间列表要能看出状态和日期，旧单缺方案也要诚实显示。",
  ][index % 3],
  createdAtMs: baseTime - (18 - index * 2) * 60_000,
}));

const artifactNotices: JiaobanConversationArtifactNotice[] = Array.from({ length: 8 }, (_, index) => ({
  id: `visual-artifact-notice-${index}`,
  kind: index % 3 === 2 ? "delivery" : "proposal",
  copy: index % 3 === 2 ? "这一单干完了，结果在右边。" : "方案好了，放你右手边了——看一眼，能跑就批。",
  createdAtMs: baseTime - (17 - index * 2) * 60_000,
  onActivate: noop,
}));

type ScrollMetric = {
  clientHeight: number;
  clientWidth: number;
  scrollHeight: number;
  scrollWidth: number;
  canScrollY: boolean;
  movedOnProbe: boolean;
};

type RegionBounds = {
  left: number;
  top: number;
  width: number;
  height: number;
};

function FixtureMetrics({ selectedId, activeView }: { selectedId: string | null; activeView: string }) {
  const [metrics, setMetrics] = useState<Record<string, unknown> | null>(null);

  useEffect(() => {
    const readMetric = (selector: string): ScrollMetric => {
      const element = document.querySelector<HTMLElement>(selector);
      if (!element) throw new Error(`fixture missing ${selector}`);
      const initialScrollTop = element.scrollTop;
      const canScrollY = element.scrollHeight > element.clientHeight;
      if (canScrollY) {
        element.scrollTop = initialScrollTop === 0
          ? Math.min(40, element.scrollHeight - element.clientHeight)
          : 0;
      }
      const movedOnProbe = element.scrollTop !== initialScrollTop;
      element.scrollTop = initialScrollTop;
      return {
        clientHeight: element.clientHeight,
        clientWidth: element.clientWidth,
        scrollHeight: element.scrollHeight,
        scrollWidth: element.scrollWidth,
        canScrollY,
        movedOnProbe,
      };
    };
    const readBounds = (selector: string): RegionBounds => {
      const element = document.querySelector<HTMLElement>(selector);
      if (!element) throw new Error(`fixture missing ${selector}`);
      const bounds = element.getBoundingClientRect();
      return {
        left: Math.round(bounds.left),
        top: Math.round(bounds.top),
        width: Math.round(bounds.width),
        height: Math.round(bounds.height),
      };
    };
    const collect = () => {
      const conversationBounds = readBounds('[aria-label="主管对话"]');
      const proposalIndexBounds = readBounds('[aria-label="历届方案索引"]');
      const canvasBounds = readBounds(".jiaoban-merged-canvas-region");
      const narrow = window.innerWidth <= 900;
      setMetrics({
        viewport: { width: window.innerWidth, height: window.innerHeight },
        documentElement: readMetric("html"),
        body: readMetric("body"),
        stage: readMetric("#fixture-stage"),
        layout: readMetric(".jiaoban-merged-layout"),
        conversation: readMetric(".project-jiaoban-main"),
        proposalIndex: readMetric(".jiaoban-history-column-body"),
        canvas: readMetric(".jiaoban-merged-canvas-surface"),
        bounds: {
          conversation: conversationBounds,
          proposalIndex: proposalIndexBounds,
          canvas: canvasBounds,
        },
        orderPass: narrow
          ? conversationBounds.top < proposalIndexBounds.top && proposalIndexBounds.top < canvasBounds.top
          : conversationBounds.left < proposalIndexBounds.left && proposalIndexBounds.left < canvasBounds.left,
        documentOverflowPass:
          document.documentElement.scrollWidth <= document.documentElement.clientWidth &&
          document.documentElement.scrollHeight <= document.documentElement.clientHeight,
        selectedId,
        activeView,
      });
    };
    const frame = requestAnimationFrame(collect);
    window.addEventListener("resize", collect);
    return () => {
      cancelAnimationFrame(frame);
      window.removeEventListener("resize", collect);
    };
  }, [selectedId, activeView]);

  return <pre hidden id="fixture-metrics">{metrics ? JSON.stringify(metrics) : "pending"}</pre>;
}

function JiaobanLayoutFixture() {
  const [filter, setFilter] = useState<"all" | "mine" | "running">("all");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [activeView, setActiveView] = useState("proposal");
  const [selectionResult, setSelectionResult] = useState("idle");
  const selectedHistory = historyEntries.find((entry) => entry.proposal_id === selectedId) ?? null;

  const selectHistory = (entry: RunHistoryEntry) => {
    const conversation = document.querySelector<HTMLElement>(".project-jiaoban-main");
    const before = conversation?.scrollTop ?? 0;
    setSelectedId(entry.proposal_id);
    setActiveView(entry.state === "delivered" ? "delivery" : "proposal");
    requestAnimationFrame(() => {
      const after = document.querySelector<HTMLElement>(".project-jiaoban-main")?.scrollTop ?? 0;
      setSelectionResult(after === before ? `pass:${before}:${after}` : `fail:${before}:${after}`);
    });
  };

  const backToCurrent = () => {
    setSelectedId(null);
    setActiveView("proposal");
  };

  const canvasViews = useMemo(() => {
    if (selectedHistory) {
      const key = selectedHistory.state === "delivered" ? "delivery" : "proposal";
      return [{
        key,
        label: key === "delivery" ? "交货" : "方案",
        subtitle: "历届方案只切换右侧实体",
        content: <JiaobanHistoryDetail entry={selectedHistory} onBackToCurrent={backToCurrent} showBackAction={false} />,
      }];
    }
    return [
      {
        key: "proposal",
        label: "方案",
        subtitle: "定稿和批准动作都在这里",
        content: (
          <JiaobanAuthorizeState
            proposal={proposal}
            proposalIsStale={false}
            amendment=""
            onAmend={noop}
            onAuthorizeAndStart={noop}
            onRePlan={noop}
            onDecline={noop}
            starting={false}
            consultLoading={false}
            consultError={null}
            howRunSummary="经典状态机 · 开个新对话 · 预演图关"
            onShowGovernance={() => setActiveView("governance")}
            onShowHowRun={() => setActiveView("howrun")}
            boundaryLoading={false}
            boundaryOutcome={null}
            onBoundaryRetry={noop}
          />
        ),
      },
      {
        key: "graph",
        label: "工序图",
        subtitle: "批准后照这个跑",
        content: <p className="muted">预演图在批准前保持只读。</p>,
      },
      {
        key: "governance",
        label: "治理保证",
        subtitle: "这一单里 Syn 和主管对自己的约束",
        content: <p className="muted">九态语义、历史实体和批准边界保持不变。</p>,
      },
      {
        key: "howrun",
        label: "怎么跑",
        subtitle: "预演 · 执行模式 · 预填对话",
        content: <p className="muted">沿用既有运行方式，不新增命令或 sidecar。</p>,
      },
    ];
  }, [selectedHistory]);

  const proposalIndex = (
    <JiaobanProposalIndex
      entries={historyEntries}
      total={historyEntries.length}
      loading={false}
      filter={filter}
      onFilterChange={setFilter}
      selectedId={selectedId}
      currentProposalId={currentProposalId}
      latestBlockedId={historyEntries.find((entry) => entry.state === "blocked")?.proposal_id ?? null}
      onSelectEntry={selectHistory}
      onBackToCurrent={backToCurrent}
      onNewJiaoban={noop}
      onContinueRun={noop}
      knownProposalIds={knownProposalIds}
    />
  );

  const main = (
    <div className="project-jiaoban-main">
      <div className="project-jiaoban-col" data-conversation-phase="proposal">
        <JiaobanConversationStream
          entries={[]}
          userGoal="把左侧历史栏放到右侧方案卡左边，作为方案列表。"
          userTurns={userTurns}
          artifactNotices={artifactNotices}
          phaseKind="conversation"
          phaseContent={null}
          consultLoading={false}
          messageBusyKey={null}
          messageErrors={{}}
        />
        <JiaobanConversationComposer
          route={{ kind: "message" }}
          draft=""
          busy={false}
          onDraftChange={noop}
          onSubmit={noop}
        />
      </div>
    </div>
  );

  return (
    <div className="project-detail-shell" id="fixture-shell">
      <section className="project-detail-content project-detail-content--fullwindow">
        <div className="project-layout project-layout--jiaoban" id="fixture-stage">
          <section className="project-jiaoban project-jiaoban--split" aria-label="交办布局渲染夹具">
            <JiaobanMergedLayout
              phase="authorize"
              main={main}
              proposalIndex={proposalIndex}
              canvasViews={canvasViews}
              activeCanvasView={activeView}
              onCanvasViewChange={setActiveView}
              onOpenWorkflow={noop}
            />
          </section>
        </div>
      </section>
      <output hidden id="fixture-selection-result">{selectionResult}</output>
      <output hidden id="fixture-selected-proposal">{selectedId ?? "current"}</output>
      <FixtureMetrics selectedId={selectedId} activeView={activeView} />
    </div>
  );
}

ReactDOM.createRoot(document.getElementById("root")!).render(<JiaobanLayoutFixture />);
