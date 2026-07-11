import { renderToStaticMarkup } from "react-dom/server.browser";
import {
  JiaobanPlanPreviewCanvas,
  jiaobanRuntimeNodeStates,
} from "../src/views/projects/ProjectJiaobanPanel";
import { JiaobanMergedLayout } from "../src/views/projects/ProjectWorkspaceShell";
import type { ProjectWorkflowChainStatus, SessionRecord } from "../src/lib/types";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(`[jiaoban-running-graph] ${message}`);
}

const noop = () => {};
const removedCopy = (...parts: string[]) => parts.join("");
const nodes = [
  { preview_node_id: "step-completed", title: "搭好页面骨架", depends_on: [] },
  { preview_node_id: "step-running", title: "补上运行进度", depends_on: ["搭好页面骨架"] },
  { preview_node_id: "step-pending", title: "整理交付结果", depends_on: ["补上运行进度"] },
];
const chainStatus: ProjectWorkflowChainStatus = {
  chain_run_id: "running-graph-chain",
  state: "running",
  nodes: [
    { node_id: "step-completed", state: "completed" },
    { node_id: "step-running", state: "running" },
    { node_id: "step-pending", state: "pending" },
  ],
};
const sessions = [{ thread_id: "session-actual", title: "实际已绑定会话" }] as unknown as SessionRecord[];
const runtimeStates = jiaobanRuntimeNodeStates(nodes, chainStatus);

assert(runtimeStates["step-completed"]?.state === "completed", "完成节点应映射 completed");
assert(runtimeStates["step-running"]?.state === "running", "运行节点应映射 running");
assert(runtimeStates["step-pending"]?.state === "pending", "未开始节点应映射 pending");

const runningGraph = (
  <JiaobanPlanPreviewCanvas
    nodes={nodes}
    bindings={[
      { preview_node_id: "step-completed", session_choice: "existing", session_id: "session-actual" },
      { preview_node_id: "step-running", session_choice: "new" },
      { preview_node_id: "step-pending", session_choice: "new" },
    ]}
    sessions={sessions}
    waitingForPreview={false}
    previewError={null}
    previewWarnings={[]}
    readOnly
    runtimeNodeStates={runtimeStates}
    onBindingChange={noop}
    onRetryPreview={noop}
    onOpenAgentSession={noop}
  />
);
const runningOutput = renderToStaticMarkup(runningGraph);

assert(runningOutput.includes('aria-label="运行工序图"'), "运行态应保留纵向图的独立语义");
assert(runningOutput.includes("任务 · 已完成") && runningOutput.includes("任务 · 正在执行") && runningOutput.includes("任务 · 等待"), "节点运行状态应以文字上脸");
assert(runningOutput.includes("is-runtime-node is-completed") && runningOutput.includes("is-runtime-node is-running"), "运行节点应带状态样式类");
assert(runningOutput.includes("实际已绑定会话") && runningOutput.includes("已绑定：实际已绑定会话"), "既有绑定应显示实际会话");
assert(runningOutput.includes("看原始对话"), "运行态节点可只读查看已绑定对话");
assert(!runningOutput.includes("<input") && !runningOutput.includes("给「"), "运行态节点不得再显示会话编辑器");
assert(!runningOutput.includes(removedCopy("只会用于", "这一步；其余节点默认各开新会话。")), "点名文案应删除");

const runningMerged = renderToStaticMarkup(
  <JiaobanMergedLayout
    phase="running"
    history={<div>历史</div>}
    main={<div>正在干</div>}
    previewCanvas={runningGraph}
    workflowPanel={<div>旧 ReactFlow 运行视图</div>}
    onOpenWorkflow={noop}
  />,
);
assert(runningMerged.includes('aria-label="工作流运行工序图"'), "合一页 running 应把同图标为运行工序图");
assert(runningMerged.includes("任务 · 正在执行"), "合一页 running 应继续显示纵向工序图");
assert(!runningMerged.includes("旧 ReactFlow 运行视图"), "合一页 running 不得回退到旧运行视图");

const doneGraph = renderToStaticMarkup(
  <JiaobanPlanPreviewCanvas
    nodes={[nodes[0]]}
    bindings={[{ preview_node_id: "step-completed", session_choice: "new" }]}
    sessions={[]}
    waitingForPreview={false}
    previewError={null}
    previewWarnings={[]}
    readOnly
    runtimeNodeStates={{ "step-completed": { state: "completed" } }}
    onBindingChange={noop}
    onRetryPreview={noop}
    onOpenAgentSession={noop}
  />,
);
assert(doneGraph.includes("任务 · 已完成") && doneGraph.includes("is-completed"), "done 相应保留终态工序图");

const terminalNodes = [
  { preview_node_id: "step-skipped-with-reason", title: "被权限拦住", depends_on: [] },
  { preview_node_id: "step-skipped-without-reason", title: "没留下原因", depends_on: [] },
  { preview_node_id: "step-archived", title: "已结束任务", depends_on: [] },
  { preview_node_id: "step-unknown", title: "未识别任务", depends_on: [] },
];
const terminalStates = jiaobanRuntimeNodeStates(terminalNodes, {
  chain_run_id: "terminal-graph-chain",
  state: "archived",
  nodes: [
    { node_id: "step-skipped-with-reason", state: "skipped", message: "未授权派发，已跳过" },
    { node_id: "step-skipped-without-reason", state: "skipped" },
    { node_id: "step-archived", state: "archived" },
    { node_id: "step-unknown", state: "future_state" },
  ],
});
assert(terminalStates["step-skipped-with-reason"]?.state === "skipped", "skipped 不得回落等待");
assert(
  terminalStates["step-skipped-with-reason"]?.detail === "未授权派发，已跳过",
  "skipped 应透传已有原因",
);
assert(
  terminalStates["step-skipped-without-reason"]?.detail === "skipped；详情看画布",
  "无原因 skipped 应保留机器键并引导看画布",
);
assert(terminalStates["step-archived"]?.state === "archived", "archived 应映射结束态");
assert(
  terminalStates["step-unknown"]?.rawState === "future_state",
  "未知状态应保留原始机器值",
);
const terminalGraph = renderToStaticMarkup(
  <JiaobanPlanPreviewCanvas
    nodes={terminalNodes}
    bindings={terminalNodes.map((node) => ({ preview_node_id: node.preview_node_id, session_choice: "new" as const }))}
    sessions={[]}
    waitingForPreview={false}
    previewError={null}
    previewWarnings={[]}
    readOnly
    runtimeNodeStates={terminalStates}
    onBindingChange={noop}
    onRetryPreview={noop}
    onOpenAgentSession={noop}
  />,
);
assert(terminalGraph.includes("任务 · 没轮到/被跳过") && terminalGraph.includes("未授权派发，已跳过"), "skipped 应明示并上脸原因");
assert(terminalGraph.includes("任务 · 本单已结束"), "archived 应显示本单已结束");
assert(terminalGraph.includes("任务 · 状态未知（future_state）"), "未知状态不得静默回落等待");

console.log("jiaoban-running-graph: 运行态同图、状态映射、只读节点和终态图离线 DOM 断言全过");
