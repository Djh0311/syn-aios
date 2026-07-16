// 交办·预演画布(批前工序图/运行只读图)+运行节点态工具——阶段3拆巨石第七刀:自 ProjectJiaobanPanel.tsx 原样迁出,零逻辑改动。
import type {
  DirectorChainStep,
  ProjectConsultationProposal,
  ProjectDirectorPlannedTask,
  ProjectDirectorPreviewNodeSessionBinding,
  ProjectDirectorTaskSessionBinding,
  ProjectWorkflowChainStatus,
  SessionRecord,
} from "../../../lib/types";
import { JiaobanRawSessionLink, JiaobanSessionPicker, NEW_SESSION_CHOICE } from "./jiaobanSessionParts";

export type JiaobanPreviewCanvasNode = {
  preview_node_id: string;
  title: string;
  depends_on: string[];
};

export type JiaobanRuntimeNodeState =
  | "pending"
  | "running"
  | "completed"
  | "waiting_decision"
  | "needs_rework"
  | "failed"
  | "skipped"
  | "archived"
  | "unknown";

export type JiaobanRuntimeNodeStateInfo = {
  state: JiaobanRuntimeNodeState;
  detail?: string;
  rawState?: string;
};

const jiaobanRuntimeNodeLabel: Record<JiaobanRuntimeNodeState, string> = {
  pending: "等待",
  running: "正在执行",
  completed: "已完成",
  waiting_decision: "待你决定",
  needs_rework: "需要重做",
  failed: "失败",
  skipped: "没轮到/被跳过",
  archived: "本单已结束",
  unknown: "状态未知",
};

function normalizeJiaobanRuntimeNodeState(
  value: string | null | undefined,
  message: string | null | undefined,
): JiaobanRuntimeNodeStateInfo {
  const rawState = value?.trim() ?? "";
  const detail = message?.trim() ?? "";
  switch (rawState.toLowerCase()) {
    case "pending":
    case "waiting":
      return { state: "pending" };
    case "running":
      return { state: "running" };
    case "completed":
    case "finished":
    case "done":
    case "succeeded":
      return { state: "completed" };
    case "needs_rework":
    case "needs-rework":
      return { state: "needs_rework" };
    case "waiting_decision":
    case "waiting-decision":
      return { state: "waiting_decision" };
    case "failed":
    case "aborted":
    case "stopped":
      return { state: "failed" };
    case "skipped":
      return {
        state: "skipped",
        detail: detail || "skipped；详情看画布",
      };
    case "archived":
      return { state: "archived", detail: detail || undefined };
    default:
      return rawState
        ? {
            state: "unknown",
            rawState,
            detail: `状态未知（${rawState}）；详情看画布`,
          }
        : { state: "pending" };
  }
}

function jiaobanRuntimeNodeStateLabel(state: JiaobanRuntimeNodeStateInfo): string {
  return state.state === "unknown" && state.rawState
    ? `状态未知（${state.rawState}）`
    : jiaobanRuntimeNodeLabel[state.state];
}

// 运行读模型以实时链节点优先；终态若轮询来不及回写，再用本轮 outcome 的步骤兜底。
// 只有单节点简单活才允许把唯一链节点映射给唯一预演节点，避免多节点时猜错归属。
export function jiaobanRuntimeNodeStates(
  nodes: JiaobanPreviewCanvasNode[],
  chainStatus: ProjectWorkflowChainStatus | null,
  chainSteps: DirectorChainStep[] = [],
): Record<string, JiaobanRuntimeNodeStateInfo> {
  return Object.fromEntries(
    nodes.map((node) => {
      const chainNode = chainStatus?.nodes.find((item) => item.node_id === node.preview_node_id);
      const chainStep = chainSteps.find((item) => item.planned_task_id === node.preview_node_id);
      const singleChainNode = nodes.length === 1 ? chainStatus?.nodes[0] : null;
      const singleChainStep = nodes.length === 1 ? chainSteps[0] : null;
      return [
        node.preview_node_id,
        normalizeJiaobanRuntimeNodeState(
          chainNode?.state ?? chainStep?.state ?? singleChainNode?.state ?? singleChainStep?.state,
          chainNode?.message ?? singleChainNode?.message,
        ),
      ];
    }),
  );
}

export function previewFallbackNode(proposal: ProjectConsultationProposal): JiaobanPreviewCanvasNode {
  return {
    preview_node_id: `proposal:${proposal.proposal_id}:single`,
    title: proposal.goal_summary || proposal.user_goal || "这项任务",
    depends_on: [],
  };
}

export function previewCanvasNodesFor(
  proposal: ProjectConsultationProposal | null,
  previewTasks: ProjectDirectorPlannedTask[] | null,
): JiaobanPreviewCanvasNode[] {
  if (!proposal) return [];
  if (previewTasks?.length) {
    return previewTasks.map((task) => ({
      preview_node_id: task.planned_task_id,
      title: task.title,
      depends_on: task.depends_on,
    }));
  }
  return [previewFallbackNode(proposal)];
}

export function previewNodeBinding(
  previewNodeId: string,
  sessionChoice: string | null,
): ProjectDirectorPreviewNodeSessionBinding {
  return sessionChoice && sessionChoice !== NEW_SESSION_CHOICE
    ? { preview_node_id: previewNodeId, session_choice: "existing", session_id: sessionChoice }
    : { preview_node_id: previewNodeId, session_choice: "new" };
}

export function runCanvasBindingsFor(
  nodes: JiaobanPreviewCanvasNode[],
  previewBindings: ProjectDirectorPreviewNodeSessionBinding[],
  taskBindings: ProjectDirectorTaskSessionBinding[],
): ProjectDirectorPreviewNodeSessionBinding[] {
  return nodes.map((node) => {
    const taskBinding = taskBindings.find((binding) => binding.planned_task_id === node.preview_node_id);
    if (taskBinding) {
      return taskBinding.session_choice === "existing"
        ? {
            preview_node_id: node.preview_node_id,
            session_choice: "existing",
            session_id: taskBinding.session_id,
          }
        : { preview_node_id: node.preview_node_id, session_choice: "new" };
    }
    return (
      previewBindings.find((binding) => binding.preview_node_id === node.preview_node_id) ??
      previewNodeBinding(node.preview_node_id, NEW_SESSION_CHOICE)
    );
  });
}

// M1·合一页右侧纵向工序图。批前节点可选对话；运行/终态复用同一张图，只读显示真实链状态。
export function JiaobanPlanPreviewCanvas({
  nodes,
  bindings,
  sessions,
  waitingForPreview,
  previewError,
  previewWarnings,
  readOnly = false,
  runtimeNodeStates = null,
  onBindingChange,
  onRetryPreview,
  onOpenAgentSession,
}: {
  nodes: JiaobanPreviewCanvasNode[];
  bindings: ProjectDirectorPreviewNodeSessionBinding[];
  sessions: SessionRecord[];
  waitingForPreview: boolean;
  previewError: string | null;
  previewWarnings: string[];
  readOnly?: boolean;
  runtimeNodeStates?: Record<string, JiaobanRuntimeNodeStateInfo> | null;
  onBindingChange: (previewNodeId: string, value: string | null) => void;
  onRetryPreview: () => void;
  onOpenAgentSession: (threadId: string) => void;
}) {
  if (waitingForPreview) {
    return (
      <div className="jiaoban-plan-preview-state" role="status" aria-label="预演工序图绘制中">
        <strong>正在绘制预演工序图…</strong>
        <span>大约 1–7 分钟；你可以照常允许并开始，未完成时会按现场拆分。</span>
        {/* 07-16:虚线占位骨架(七律②:虚线=「这里将来有东西」)——loading 期右区不再大片空白。 */}
        <div className="jiaoban-preview-ghosts" aria-hidden="true">
          <div className="jiaoban-preview-ghost">任务 · 生成中…</div>
          <p className="jiaoban-preview-ghost-arrow">↓</p>
          <div className="jiaoban-preview-ghost">复核 · 独立只读核验</div>
        </div>
      </div>
    );
  }
  if (previewError) {
    return (
      <div className="jiaoban-plan-preview-state is-error" role="note" aria-label="预演工序图暂不可用">
        <strong>预演工序图暂不可用</strong>
        <span>{previewError}。不影响你批准这份方案。</span>
        <button className="secondary-button" type="button" onClick={onRetryPreview}>
          重试画图
        </button>
      </div>
    );
  }
  return (
    <section className="jiaoban-plan-preview" aria-label={readOnly ? "运行工序图" : "方案预演工序图"}>
      <div className="jiaoban-plan-preview-graph" role="list" aria-label={readOnly ? "运行任务与依赖" : "预演任务与依赖"}>
        {nodes.map((node, index) => {
          const binding =
            bindings.find((item) => item.preview_node_id === node.preview_node_id) ??
            previewNodeBinding(node.preview_node_id, NEW_SESSION_CHOICE);
          const session =
            binding.session_choice === "existing"
              ? sessions.find((item) => item.thread_id === binding.session_id) ?? null
              : null;
          const sessionLabel =
            binding.session_choice === "existing"
              ? `接现有 · ${session?.title || binding.session_id || "已选对话"}`
              : "新会话";
          const dependencies = node.depends_on.filter(Boolean);
          const runtimeNodeState = readOnly
            ? runtimeNodeStates?.[node.preview_node_id] ?? { state: "pending" }
            : null;
          const nodeLabel = runtimeNodeState ? jiaobanRuntimeNodeStateLabel(runtimeNodeState) : "预演";
          return (
            <div className="jiaoban-plan-preview-node-wrap" key={node.preview_node_id} role="listitem">
              {index > 0 ? (
                <span className="jiaoban-plan-preview-edge" aria-label={`步骤 ${index + 1} 的前置关系`}>
                  ↓
                </span>
              ) : null}
              <details
                className={`jiaoban-plan-preview-node${runtimeNodeState ? ` is-runtime-node is-${runtimeNodeState.state}` : ""}`}
              >
                <summary className="project-canvas-static-node task preflight">
                  <span>任务 · {nodeLabel}</span>
                  <strong>{node.title}</strong>
                  {dependencies.length ? <em>依赖：{dependencies.join("、")}</em> : <em>可从这里开始</em>}
                  {runtimeNodeState?.detail ? <em>{runtimeNodeState.detail}</em> : null}
                  <small className={binding.session_choice === "existing" ? "is-existing" : ""}>{sessionLabel}</small>
                </summary>
                {readOnly ? (
                  <div className="jiaoban-plan-preview-picker jiaoban-plan-preview-picker--readonly">
                    <p className="muted small-note">
                      {binding.session_choice === "existing"
                        ? `已绑定：${session?.title || binding.session_id || "现有对话"}`
                        : "这一步使用新会话。"}
                    </p>
                    <JiaobanRawSessionLink
                      sessionChoice={binding.session_choice === "existing" ? binding.session_id ?? null : NEW_SESSION_CHOICE}
                      onOpenAgentSession={onOpenAgentSession}
                    />
                  </div>
                ) : (
                  <div className="jiaoban-plan-preview-picker">
                    <JiaobanSessionPicker
                      sessions={sessions}
                      sessionChoice={binding.session_choice === "existing" ? binding.session_id ?? null : NEW_SESSION_CHOICE}
                      onSessionChoiceChange={(value) => onBindingChange(node.preview_node_id, value)}
                      onOpenAgentSession={onOpenAgentSession}
                      label={`给「${node.title}」选择对话`}
                      inputName={`jiaoban-preview-session-${index}`}
                    />
                  </div>
                )}
              </details>
            </div>
          );
        })}
      </div>
      {previewWarnings.length ? (
        <ul className="jiaoban-plan-preview-warnings" aria-label="预演提醒">
          {previewWarnings.map((warning, index) => (
            <li key={index}>{warning}</li>
          ))}
        </ul>
      ) : null}
    </section>
  );
}
