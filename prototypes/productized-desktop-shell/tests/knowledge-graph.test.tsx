import React from "react";
import { renderToStaticMarkup } from "react-dom/server.browser";
import {
  deterministicKnowledgeGraphPosition,
  KnowledgeGraphNodeAction,
  KnowledgeGraphView,
  KNOWLEDGE_GRAPH_MIN_NODE_GAP,
  KNOWLEDGE_GRAPH_NODE_HEIGHT,
  KNOWLEDGE_GRAPH_NODE_PITCH,
  KNOWLEDGE_GRAPH_NODE_WIDTH,
  KNOWLEDGE_GRAPH_READABLE_ZOOM,
  knowledgeGraphLayoutRadius,
  knowledgeGraphMinZoom,
  knowledgeGraphNodeAccessibleLabel,
  knowledgeGraphRingCapacity,
  knowledgeGraphRingPlan,
  loadKnowledgeGraph,
  nextKnowledgeGraphOpenRequest,
  openKnowledgeGraphNode,
} from "../src/views/knowledge/KnowledgeGraphView";
import type {
  KnowledgeWorkspaceGraphNode,
  KnowledgeWorkspaceGraphOptions,
  KnowledgeWorkspaceGraphResponse,
} from "../src/lib/tauri";

let assertionCount = 0;
function assert(condition: unknown, message: string): asserts condition {
  assertionCount += 1;
  if (!condition) throw new Error(`[knowledge-graph] ${message}`);
}

function assertDeep(actual: unknown, expected: unknown, message: string) {
  assert(JSON.stringify(actual) === JSON.stringify(expected), `${message}; actual=${JSON.stringify(actual)}`);
}

const graphProjection: KnowledgeWorkspaceGraphResponse = {
  scope: "global",
  focus_relative_path: null,
  query: null,
  tag: null,
  nodes: [
    { id: "opaque-node-id", relative_path: "research/field-note.md", title: "田野笔记", tags: ["研究"] },
    { id: "archive/isolated.md", relative_path: "archive/isolated.md", title: "独立资料", tags: [] },
  ],
  edges: [],
  diagnostics: [
    {
      code: "knowledge_workspace_graph_node_limit",
      relative_path: null,
      message: "图谱节点超过本阶段上限，剩余节点和相关边未显示。",
    },
  ],
  truncated: true,
};

// SSR only receives a static, read-only projection. It must not attempt to
// invoke the fixed client or touch Tauri/vault state.
let staticGraphCalls = 0;
const staticClient = {
  graph: async () => {
    staticGraphCalls += 1;
    return graphProjection;
  },
};
const staticMarkup = renderToStaticMarkup(<KnowledgeGraphView client={staticClient} staticGraph={graphProjection} />);

assert(staticGraphCalls === 0, "SSR 图谱壳不得调用 Tauri 或读取 vault");
assert(staticMarkup.includes("Syn 原生关系图"), "静态壳必须把图谱明确为 Syn 原生关系投影");
assert(staticMarkup.includes("全局") && staticMarkup.includes("局部"), "静态壳必须保留全局/局部范围入口");
assert(!staticMarkup.includes("<h2"), "中央标签已表达关系图，静态壳不得重复大标题");
assert(staticMarkup.includes('aria-pressed="true"'), "全局范围必须具有真实选中语义");
assert(
  staticMarkup.includes('aria-expanded="false"') && staticMarkup.includes("打开关系图筛选"),
  "文字、标签和局部焦点必须收进默认关闭的筛选入口",
);
assert(!staticMarkup.includes("<input") && !staticMarkup.includes("<select"), "默认 compact chrome 不得常驻详细筛选表单");
assert(staticMarkup.includes("独立笔记") && staticMarkup.includes("独立资料"), "没有边的 Markdown 也必须作为独立笔记显示");
assert(staticMarkup.includes("已截断") && staticMarkup.includes("剩余节点和相关边未显示"), "上限诊断必须保留给用户");
assert(!staticMarkup.includes("Obsidian") && !staticMarkup.includes("外部打开") && !staticMarkup.includes("工作流"), "图谱不得伪装为外部或工作流入口");

const deterministicPositions = Array.from({ length: 6 }, (_, index) => deterministicKnowledgeGraphPosition(index, 6));
assertDeep(
  deterministicPositions,
  [
    { x: 162, y: 0 },
    { x: 302, y: 81 },
    { x: 302, y: 243 },
    { x: 162, y: 324 },
    { x: 22, y: 243 },
    { x: 22, y: 81 },
  ],
  "轻量节点布局必须是固定输入对应固定坐标的确定性关系舞台",
);
assert(
  new Set(deterministicPositions.map(({ x, y }) => `${x}:${y}`)).size === deterministicPositions.length,
  "确定性关系舞台不得把不同节点叠到同一坐标",
);

// N2R-R3E D3：布局必须随节点数增长，而不是把半轴写死在 110/160。
// 半轴恒定时 6 节点刚好排满、12 节点起互压，后端上限 512 必然堆叠。
assert(
  KNOWLEDGE_GRAPH_NODE_PITCH === KNOWLEDGE_GRAPH_NODE_WIDTH + KNOWLEDGE_GRAPH_MIN_NODE_GAP
    && KNOWLEDGE_GRAPH_NODE_WIDTH === 136
    && KNOWLEDGE_GRAPH_NODE_HEIGHT === 40,
  "节点盒常量必须与 styles.css 的 .native-graph-node 同源，间距下界由盒宽推出",
);
assert(
  Math.abs(KNOWLEDGE_GRAPH_READABLE_ZOOM - 28 / KNOWLEDGE_GRAPH_NODE_HEIGHT) < 1e-12,
  "达标缩放必须由 28px hit target 与节点盒高推出，不得手填",
);

const scaleSamples = [1, 2, 6, 7, 12, 40, 100, 512];
for (const total of scaleSamples) {
  const points = Array.from({ length: total }, (_, index) => deterministicKnowledgeGraphPosition(index, total));
  let minDistance = Infinity;
  for (let i = 0; i < total; i += 1) {
    for (let j = i + 1; j < total; j += 1) {
      minDistance = Math.min(minDistance, Math.hypot(points[i].x - points[j].x, points[i].y - points[j].y));
    }
  }
  assert(
    total === 1 || minDistance >= KNOWLEDGE_GRAPH_NODE_PITCH,
    `n=${total} 时任意两节点中心距必须 ≥ 节点盒宽 + 最小间距（实算 ${Math.round(minDistance * 100) / 100}）`,
  );
  assert(
    new Set(points.map(({ x, y }) => `${x}:${y}`)).size === total,
    `n=${total} 时不得把不同节点叠到同一坐标`,
  );
  assert(
    points.every((point, index) => {
      const repeat = deterministicKnowledgeGraphPosition(index, total);
      return repeat.x === point.x && repeat.y === point.y;
    }),
    `n=${total} 时同一 (index,total) 必须恒定产出同一坐标`,
  );
}

// 环容量自洽：每环相邻弦长不得低于布局步距，跨环靠半径差保证。
for (const total of scaleSamples) {
  for (const ring of knowledgeGraphRingPlan(total)) {
    const chord = ring.count < 2 ? Infinity : 2 * ring.radius * Math.sin(Math.PI / ring.count);
    assert(
      ring.count <= knowledgeGraphRingCapacity(ring.radius) && chord >= KNOWLEDGE_GRAPH_NODE_PITCH,
      `n=${total} 的半径 ${ring.radius} 环放 ${ring.count} 个节点时相邻弦长 ${Math.round(chord * 100) / 100} 低于下界`,
    );
  }
}
assert(
  knowledgeGraphLayoutRadius(512) > knowledgeGraphLayoutRadius(40)
    && knowledgeGraphLayoutRadius(40) > knowledgeGraphLayoutRadius(6),
  "布局半径必须随节点数增长，不得与 total 无关",
);
assert(
  knowledgeGraphMinZoom(512) < knowledgeGraphMinZoom(40) && knowledgeGraphMinZoom(40) < knowledgeGraphMinZoom(6),
  "minZoom 必须随规模下调，否则大图 fitView 会被下限夹住",
);

// The browser seam stays typed: UI code passes only lower-camel graph options
// to the fixed graph client. It cannot supply a vault root, raw command, or
// arbitrary target.
const graphCalls: KnowledgeWorkspaceGraphOptions[] = [];
const graphClient = {
  graph: async (options: KnowledgeWorkspaceGraphOptions) => {
    graphCalls.push(options);
    return graphProjection;
  },
};
await loadKnowledgeGraph(graphClient, {
  scope: "local",
  focusRelativePath: "research/field-note.md",
  query: "田野",
  tag: "研究",
});
assertDeep(
  graphCalls,
  [{ scope: "local", focusRelativePath: "research/field-note.md", query: "田野", tag: "研究" }],
  "图谱请求必须保持固定 lower-camel payload",
);
assert(!("invoke" in graphClient) && !("root" in graphClient), "图谱 UI 注入面不得暴露原始 command 或 vault root");

// A graph node is an index projection, not a route. The only permitted action
// hands the backend-returned relative_path to the typed parent callback.
const openedPaths: string[] = [];
const graphNode: KnowledgeWorkspaceGraphNode = {
  id: "opaque-id-that-must-not-be-opened",
  relative_path: "research/field-note.md",
  title: "田野笔记",
  tags: [],
};
const accessibleNodeLabel = knowledgeGraphNodeAccessibleLabel(graphNode, true);
assert(
  accessibleNodeLabel.includes(graphNode.title)
    && accessibleNodeLabel.includes(graphNode.relative_path)
    && accessibleNodeLabel.includes("独立笔记"),
  "视觉隐藏的路径和孤立状态必须保留在节点可访问名称中",
);
openKnowledgeGraphNode(graphNode, (relativePath) => openedPaths.push(relativePath));
assertDeep(openedPaths, ["research/field-note.md"], "节点动作只能交出返回的 relative_path");
const firstOpenRequest = nextKnowledgeGraphOpenRequest(null, "research/field-note.md");
const repeatedOpenRequest = nextKnowledgeGraphOpenRequest(firstOpenRequest, "research/field-note.md");
assert(
  repeatedOpenRequest.sequence > firstOpenRequest.sequence && repeatedOpenRequest.relativePath === firstOpenRequest.relativePath,
  "重复选择同一图节点也必须触发一次受限的原生读取请求",
);

const nodeActionMarkup = renderToStaticMarkup(
  <KnowledgeGraphNodeAction
    relativePath={graphNode.relative_path}
    title={graphNode.title}
    tags={graphNode.tags}
    isolated
    current
    onActivate={() => undefined}
  />,
);
assert(
  nodeActionMarkup.startsWith("<button")
    && nodeActionMarkup.includes('type="button"')
    && nodeActionMarkup.includes("native-graph-node-button"),
  "Graph 节点必须以原生 button 的 click/Enter/Space 等价激活语义交给同一 typed handoff",
);
assert(
  nodeActionMarkup.includes('aria-current="page"')
    && nodeActionMarkup.includes("research/field-note.md")
    && nodeActionMarkup.includes("独立笔记"),
  "节点动作必须保留当前态、返回路径和孤立状态的稳定 ARIA",
);

console.log(`native knowledge graph static shell and typed handoff tests passed: ${assertionCount}`);
