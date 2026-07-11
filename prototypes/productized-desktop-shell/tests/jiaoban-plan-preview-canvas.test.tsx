import { renderToStaticMarkup } from "react-dom/server.browser";
import { JiaobanPlanPreviewCanvas } from "../src/views/projects/ProjectJiaobanPanel";
import { JiaobanMergedLayout } from "../src/views/projects/ProjectWorkspaceShell";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(`[jiaoban-plan-preview-canvas] ${message}`);
}

const noop = () => {};
const removedCopy = (...parts: string[]) => parts.join("");
const previewNodes = [
  { preview_node_id: "preview-step-1", title: "搭好页面骨架", depends_on: [] },
  { preview_node_id: "preview-step-2", title: "补上验收", depends_on: ["搭好页面骨架"] },
];

const previewCanvas = (
  <JiaobanPlanPreviewCanvas
    nodes={previewNodes}
    bindings={[
      { preview_node_id: "preview-step-1", session_choice: "new" },
      { preview_node_id: "preview-step-2", session_choice: "new" },
    ]}
    sessions={[]}
    waitingForPreview={false}
    previewError={null}
    previewWarnings={[]}
    onBindingChange={noop}
    onRetryPreview={noop}
    onOpenAgentSession={noop}
  />
);

const output = renderToStaticMarkup(previewCanvas);
assert(output.includes('aria-label="方案预演工序图"'), "画布语义标签应保留");
assert(
  !output.includes(removedCopy("<strong>预演工序图", "</strong>")) && !output.includes(removedCopy("你批的就是", "这份图")),
  "画布应删除教学性标题与副标",
);
assert(output.includes("任务 · 预演"), "节点必须与运行态明确区分");
assert(output.includes("搭好页面骨架") && output.includes("补上验收"), "预演节点应展示任务标题");
assert(output.includes("依赖：搭好页面骨架"), "依赖关系应在节点图中可见");
assert(output.includes("新会话"), "每个节点默认新会话");
assert(output.includes("<details"), "点击节点的可展开会话选择器应在 DOM 中");
assert(output.includes("开个新的（为这单活新建一个对话）"), "复用现有会话选择器");
assert(!output.includes("preview-step-"), "画布不得暴露内部步骤编号");

const simpleOutput = renderToStaticMarkup(
  <JiaobanPlanPreviewCanvas
    nodes={[{ preview_node_id: "simple-work", title: "这单活本身", depends_on: [] }]}
    bindings={[{ preview_node_id: "simple-work", session_choice: "new" }]}
    sessions={[]}
    waitingForPreview={false}
    previewError={null}
    previewWarnings={[]}
    onBindingChange={noop}
    onRetryPreview={noop}
    onOpenAgentSession={noop}
  />,
);
assert(simpleOutput.includes("这单活本身") && simpleOutput.includes("新会话"), "简单活也应有可选会话的单个预演节点");

const merged = renderToStaticMarkup(
  <JiaobanMergedLayout
    phase="authorize"
    history={<div>历史</div>}
    main={<button type="button">允许并开始</button>}
    previewCanvas={previewCanvas}
    workflowPanel={<div>不应展示的运行态画布</div>}
    onOpenWorkflow={noop}
  />,
);
assert(merged.includes('aria-label="方案预演工序图"'), "授权期应把预演放进 M2 画布区域");
assert(
  !merged.includes(removedCopy("<strong>方案预演", "</strong>")) && !merged.includes(removedCopy("尚未执行，可", "逐节点选对话")),
  "合一页应删除预演教学性头部",
);
assert(!merged.includes("不应展示的运行态画布"), "授权期不能把运行态画布混进预演区");

console.log("jiaoban-plan-preview-canvas: 预演节点、依赖、默认新会话和合一画布离线 DOM 断言全过");
