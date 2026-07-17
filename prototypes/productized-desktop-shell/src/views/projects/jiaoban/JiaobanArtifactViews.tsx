import type { ReactNode } from "react";

export type JiaobanPhase =
  | "say"
  | "authorize"
  | "binding"
  | "running"
  | "done"
  | "waiting_decision"
  | "blocked";

export type JiaobanCanvasViewSpec = {
  key: string;
  label: string;
  subtitle?: string;
  content: ReactNode;
};

export function buildJiaobanArtifactCanvasViews({
  phase,
  selectedHistoryId,
  activeViewKey,
  proposalInteractive,
  proposalContent,
  deliveryContent,
  graphContent,
  workStateContent = null,
  governanceContent,
  howRunContent,
}: {
  phase: JiaobanPhase;
  selectedHistoryId: string | null;
  activeViewKey: string;
  proposalInteractive: boolean;
  proposalContent: ReactNode;
  deliveryContent: ReactNode;
  graphContent: ReactNode;
  workStateContent?: ReactNode;
  governanceContent: ReactNode;
  howRunContent: ReactNode;
}): JiaobanCanvasViewSpec[] | undefined {
  const graphCarriesWorkState =
    phase === "binding" || phase === "running" || phase === "waiting_decision";
  const proposalView: JiaobanCanvasViewSpec | null = proposalContent
    ? {
        key: "proposal",
        label: "方案",
        subtitle: proposalInteractive ? "定稿和批准动作都在这里" : "这单的方案留在这里",
        content: proposalContent,
      }
    : null;
  const deliveryView: JiaobanCanvasViewSpec | null = deliveryContent
    ? {
        key: "delivery",
        label: "交货",
        subtitle: "结果、体检和主管意见都在这里",
        content: deliveryContent,
      }
    : null;
  const graphView: JiaobanCanvasViewSpec = {
    key: "graph",
    label: "工序图",
    subtitle: phase === "authorize" ? "批准后照这个跑" : "跑到哪亮到哪",
    content: graphCarriesWorkState && workStateContent ? (
      <div className="jiaoban-canvas-work-state">
        {graphContent}
        {workStateContent}
      </div>
    ) : graphContent,
  };

  if (selectedHistoryId != null) {
    const historyViews = [deliveryView, proposalView].filter(
      (view): view is JiaobanCanvasViewSpec => view != null,
    );
    return historyViews.length ? historyViews : undefined;
  }
  if (phase === "authorize" && proposalView) {
    return [
      proposalView,
      graphView,
      { key: "governance", label: "治理保证", subtitle: "这一单里 Syn 和主管对自己的约束", content: governanceContent },
      { key: "howrun", label: "怎么跑", subtitle: "预演 · 执行模式 · 预填对话", content: howRunContent },
    ];
  }
  if (phase === "done" && deliveryView) {
    return [deliveryView, ...(proposalView ? [proposalView] : []), graphView];
  }
  if (graphCarriesWorkState && !proposalView) {
    return [graphView];
  }
  if (phase !== "say" && proposalView) {
    return [graphView, proposalView];
  }
  if (phase === "say" && (activeViewKey === "proposal" || activeViewKey === "delivery")) {
    const artifactViews = [proposalView, deliveryView].filter(
      (view): view is JiaobanCanvasViewSpec => view != null,
    );
    return artifactViews.some((view) => view.key === activeViewKey) ? artifactViews : undefined;
  }
  return undefined;
}
