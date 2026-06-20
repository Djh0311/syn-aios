import { assert, assertDeepEqual } from "./offlineInteractionTestUtils";
import {
  NODE_KIND_PRESETS,
  canvasNodeToData,
  createNodeData,
  dataToCanvasNode,
  type CanvasNodeData,
} from "../../src/lib/canvasNodeData";
import type { CanvasNode } from "../../src/lib/types";

// Free-canvas node authoring (plan A1–A3) data-layer coverage. React Flow does
// not render under renderToStaticMarkup, so this exercises the pure mapping that
// the custom node / editor / save path run through. A1–A3 keep the free payload
// session-only: persistence of it is A4, so this asserts the lossy-by-design
// save mapping and that no field the store lacks is ever sent (no half-contract).
export function runCanvasNodeAuthoringScenario() {
  // A2 · the palette offers free kinds beyond the two old fixed buttons.
  const kinds = NODE_KIND_PRESETS.map((preset) => preset.kind);
  assert(kinds.includes("director") && kinds.includes("subagent"), "A2 调色板应保留原 director/subagent");
  assert(
    kinds.filter((kind) => kind !== "director" && kind !== "subagent").length >= 2,
    "A2 调色板应提供更多自由种类，不止两个固定钮",
  );

  // A3 · a newly created (in-session) node seeds a rich free payload.
  const seeded = createNodeData("reviewer");
  assert(seeded.kind === "reviewer", "A3 会话态新建节点应带自由 kind");
  assert(seeded.status === "draft" && seeded.sandbox === "read-only", "A3 新建节点应有默认状态灯/沙箱");

  const rich: CanvasNodeData = {
    name: "自定义校验节点",
    kind: "my-custom-kind",
    role: "subagent",
    status: "ready",
    prompt: "检查 README 是否更新",
    sandbox: "workspace-write",
    skill: "doc-review",
    session_id: "thread-xyz",
    fields: [{ id: "f1", key: "模型", value: "claude-opus-4-8" }],
  };
  const persisted: CanvasNode = dataToCanvasNode("node-1", rich, { x: 12, y: 34 }, ["w1"]);

  // A1–A3 守纯前端：save 映射只发后端 store 已有的字段，不带 kind/data（不留半截契约）。
  assertDeepEqual(
    Object.keys(persisted).sort(),
    ["id", "label", "position", "role", "session_id", "skill", "warnings"],
    "A1–A3 持久化节点只含后端 store 已有字段，不得带 kind/data",
  );
  assert(!("kind" in persisted) && !("data" in persisted), "A1–A3 save 不得向后端发它没有的 kind/data 字段");
  // Supported subset still carries through correctly.
  assert(persisted.label === "自定义校验节点", "A3 name 落进持久化 label");
  assert(persisted.role === "subagent", "A3 role 仍随持久化保留（供已走通逻辑识别）");
  assert(persisted.skill === "doc-review" && persisted.session_id === "thread-xyz", "A3 skill/session 仍持久化");
  assertDeepEqual(persisted.position, { x: 12, y: 34 }, "A3 持久化应保留节点坐标");
  assertDeepEqual(persisted.warnings, ["w1"], "A3 持久化应保留既有 warnings");

  // Lossy-by-design reload: the free payload is session-only in A1–A3, so a
  // reloaded node does NOT recover kind/prompt/status/custom fields (A4 will).
  const reloaded = canvasNodeToData(persisted);
  assert(reloaded.kind === "subagent", "A1–A3 重载后自由 kind 不持久化，回落到 role");
  assert(
    reloaded.prompt === "" && reloaded.status === "draft" && reloaded.fields.length === 0,
    "A1–A3 重载后自由 payload（prompt/status/自定义字段）按设计丢失，留待 A4 持久化",
  );
  assert(reloaded.skill === "doc-review" && reloaded.session_id === "thread-xyz", "重载仍保留 store 已有的 skill/session");

  // 向后兼容：feature 之前保存的节点本就只有这些字段，读出不应报错。
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
  assert(legacyData.kind === "director" && legacyData.name === "老主管节点", "向后兼容：旧节点 role→kind、label→name");
}
