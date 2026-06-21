import { assert, assertDeepEqual } from "./offlineInteractionTestUtils";
import {
  NODE_KIND_PRESETS,
  canvasNodeToData,
  createNodeData,
  dataToCanvasNode,
  instantiateTemplateGraph,
  type CanvasNodeData,
} from "../../src/lib/canvasNodeData";
import type { CanvasEdge, CanvasNode } from "../../src/lib/types";

// Free-canvas node authoring (plan A1–A4) data-layer coverage. React Flow does
// not render under renderToStaticMarkup, so this exercises the pure mapping that
// the custom node / editor / save path run through. A4 persists the free payload,
// so the round-trip is now lossless (front/back CanvasNode types match).
export function runCanvasNodeAuthoringScenario() {
  // A2 · the palette offers free kinds beyond the two old fixed buttons.
  const kinds = NODE_KIND_PRESETS.map((preset) => preset.kind);
  assert(kinds.includes("director") && kinds.includes("subagent"), "A2 调色板应保留原 director/subagent");
  assert(
    kinds.filter((kind) => kind !== "director" && kind !== "subagent").length >= 2,
    "A2 调色板应提供更多自由种类，不止两个固定钮",
  );

  // A3 · a newly created node seeds a free payload with sensible defaults.
  const seeded = createNodeData("reviewer");
  assert(seeded.kind === "reviewer", "A3 新建节点应带自由 kind");
  assert(seeded.status === "draft" && seeded.sandbox === "read-only", "A3 新建节点应有默认状态灯/沙箱");

  // A4 · the full free payload (kind/prompt/sandbox/status/custom fields) survives
  // the persist round-trip unchanged.
  const rich: CanvasNodeData = {
    name: "自定义校验节点",
    kind: "my-custom-kind",
    role: "subagent",
    status: "ready",
    prompt: "检查 README 是否更新\n第二行也保留",
    sandbox: "workspace-write",
    skill: "doc-review",
    session_id: "thread-xyz",
    fields: [
      { id: "f1", key: "模型", value: "claude-opus-4-8" },
      { id: "f2", key: "验收", value: "typecheck 绿" },
    ],
  };
  const persisted: CanvasNode = dataToCanvasNode("node-1", rich, { x: 12, y: 34 }, ["w1"]);
  assert(persisted.kind === "my-custom-kind", "A4 自由 kind 应落进持久化节点");
  assert(persisted.label === "自定义校验节点", "A4 name 应落进持久化 label");
  assertDeepEqual(persisted.position, { x: 12, y: 34 }, "A4 持久化应保留节点坐标");
  assertDeepEqual(persisted.warnings, ["w1"], "A4 持久化应保留既有 warnings");
  const restored = canvasNodeToData(persisted);
  assertDeepEqual(
    restored,
    rich,
    "A4 自由 payload（kind/prompt/sandbox/status/技能/会话/自定义字段）经持久化往返应无损",
  );

  // 向后兼容：A4 之前保存的节点没有 kind/data，仍应正确读出（kind 回落到 role、自由字段默认空）。
  const legacy: CanvasNode = {
    id: "legacy-1",
    role: "director",
    label: "老主管节点",
    skill: null,
    session_id: null,
    position: { x: 0, y: 0 },
    warnings: [],
  };
  const legacyData = canvasNodeToData(legacy);
  assert(legacyData.kind === "director", "向后兼容：无 kind 的旧节点应回落到 role 作为 kind");
  assert(
    legacyData.fields.length === 0 && legacyData.prompt === "" && legacyData.status === "draft",
    "向后兼容：旧节点自由字段应默认空 / 默认状态",
  );

  // B3 · 从成熟模式实例化：节点 id 全部重置、连线随新 id 重映射、节点 payload 带过来。
  const tplNodes: CanvasNode[] = [
    dataToCanvasNode("tpl-a", { ...rich, name: "模板节点 A" }, { x: 1, y: 2 }, []),
    dataToCanvasNode(
      "tpl-b",
      { ...createNodeData("reviewer"), name: "模板节点 B" },
      { x: 3, y: 4 },
      [],
    ),
  ];
  const tplEdges: CanvasEdge[] = [
    { id: "tpl-edge-1", from: "tpl-a", to: "tpl-b" },
    { id: "tpl-edge-dangling", from: "tpl-a", to: "tpl-missing" },
  ];
  const instance = instantiateTemplateGraph(tplNodes, tplEdges, (_node, index) => `new-${index}`);
  assertDeepEqual(
    instance.nodes.map((n) => n.id),
    ["new-0", "new-1"],
    "B3 实例化应给每个节点重置新 id",
  );
  assert(
    !instance.nodes.some((n) => n.id === "tpl-a" || n.id === "tpl-b"),
    "B3 实例化后不得残留模板原 id",
  );
  assert(instance.nodes[0].data.name === "模板节点 A", "B3 实例化应带过节点 payload（name）");
  assert(instance.nodes[0].data.kind === "my-custom-kind", "B3 实例化应带过自由 kind");
  assertDeepEqual(
    instance.edges.map((e) => ({ from: e.from, to: e.to })),
    [{ from: "new-0", to: "new-1" }],
    "B3 连线应重映射到新 id，且丢弃指向缺失节点的悬空边",
  );
}
