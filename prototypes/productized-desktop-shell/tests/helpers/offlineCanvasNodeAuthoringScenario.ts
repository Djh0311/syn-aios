import { assert, assertDeepEqual } from "./offlineInteractionTestUtils";
import {
  NODE_KIND_PRESETS,
  buildNodeDispatchRequest,
  buildExperimentNodeDispatchRequest,
  canvasNodeToData,
  canvasScope,
  createNodeData,
  dataToCanvasNode,
  instantiateTemplateGraph,
  nodeRunReadiness,
  type CanvasNodeData,
} from "../../src/lib/canvasNodeData";
import type { CanvasEdge, CanvasNode } from "../../src/lib/types";
import { experimentCanvasSurfaceConfig, projectCanvasSurfaceConfig } from "../../src/lib/canvasSurfaceConfig";
import { experimentCanvasBoundary, projectWorkflowCanvasBoundary } from "../../src/lib/canvasSurfaceBoundaries";

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
    session: { mode: "resume", thread_id: "thread-xyz" },
    work_item_id: "wi-001",
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
  // §8.1 reuse-bug fix: a template node with a resume policy must NOT carry the
  // template author's thread_id into the instance — it lands as resume-but-unchosen.
  assert(
    instance.nodes[0].data.session.mode === "resume" && instance.nodes[0].data.session.thread_id === "",
    "B3 实例化应清空模板里 resume 的 thread_id，不继承旧会话",
  );
  assertDeepEqual(
    instance.edges.map((e) => ({ from: e.from, to: e.to })),
    [{ from: "new-0", to: "new-1" }],
    "B3 连线应重映射到新 id，且丢弃指向缺失节点的悬空边",
  );

  // C2 / 会话模型 · run readiness (UI-level, NOT a security gate): "new" mints a
  // session at run time so it is ready; "resume" needs a concrete thread_id; a
  // work_item_id is always required.
  assert(nodeRunReadiness(rich).ready, "续已有 + 选了会话 + work_item 的节点应可运行");
  assert(
    nodeRunReadiness({ ...rich, session: { mode: "new" } }).ready,
    "会话模型：new 策略无需预绑会话即可运行（运行时 mint）",
  );
  assert(
    !nodeRunReadiness({ ...rich, session: { mode: "resume", thread_id: "" } }).ready,
    "会话模型：续已有但未选具体会话 → 不能运行",
  );
  assert(
    !nodeRunReadiness({ ...rich, work_item_id: "" }).ready,
    "C2 未绑工作项 ID 的节点不能运行",
  );

  // P3 A 映射 · 实验面 readiness：无需手填 work_item_id（后端自动建临时 work_item），但要 prompt。
  assert(
    nodeRunReadiness({ ...rich, work_item_id: "" }, "experiment").ready,
    "A 映射：实验面无需手填 work_item_id（后端自动建临时票）",
  );
  assert(
    !nodeRunReadiness({ ...rich, prompt: "" }, "experiment").ready,
    "A 映射：实验面节点缺 prompt 不能跑",
  );
  assert(
    !nodeRunReadiness({ ...rich, session: { mode: "resume", thread_id: "" } }, "experiment").ready,
    "A 映射：实验面续已有但未选会话 → 不能跑（拦路石①下仍要先选名册里的会话）",
  );
  assert(
    !nodeRunReadiness({ ...rich, session: { mode: "new" } }, "experiment").ready,
    "resume-only（2026-06-22 决策）：实验面选『开新会话』不可跑（本期未启用）",
  );

  // 会话模型 P1 · default policy is "new"; resume policy round-trips through
  // persistence; legacy top-level session_id migrates to a resume policy.
  assert(createNodeData("subagent").session.mode === "new", "P1 新建节点默认会话策略 = new（无默认偏向偏 resume）");
  assert(
    restored.session.mode === "resume" &&
      restored.session.thread_id === "thread-xyz",
    "P1 resume 会话策略应经持久化往返无损",
  );
  assert(
    persisted.session_id === "thread-xyz",
    "P1 顶层 session_id 仍随 resume 策略写入，供既有 sealed 逻辑读",
  );
  const legacyResume = canvasNodeToData({
    id: "legacy-resume",
    role: "subagent",
    label: "老节点带会话",
    skill: null,
    session_id: "old-thread-001",
    position: { x: 0, y: 0 },
    warnings: [],
  });
  assert(
    legacyResume.session.mode === "resume" && legacyResume.session.thread_id === "old-thread-001",
    "P1 向后兼容：旧节点顶层 session_id 迁移成 resume 策略",
  );

  // C1 · the dispatch request built from node data maps prompt/sandbox/work_item
  // correctly. (It only SHAPES the request — the backend double gate decides
  // blocked vs run; default-safe is proved by the Rust gate test, not here.)
  const request = buildNodeDispatchRequest({
    nodeId: "canvas-node-9",
    projectRoot: "/Users/yoyi/codex-workflow-mario-test",
    data: rich,
    instructionId: "instr-fixed",
  });
  assert(request.node_id === "canvas-node-9", "C1 请求带画布节点 id 作为 node_id");
  assert(request.work_item_id === "wi-001", "C1 请求带节点绑定的 work_item_id");
  assert(request.prompt_kind === "user_reviewed_instruction", "C1 真业务派发用 user_reviewed_instruction");
  assert(
    request.user_reviewed_instruction.objective === "检查 README 是否更新\n第二行也保留",
    "C1 prompt 应映射到 objective",
  );
  assert(request.user_reviewed_instruction.sandbox_mode === "workspace-write", "C1 sandbox 取自节点 data");
  assertDeepEqual(
    request.user_reviewed_instruction.allowed_write_roots,
    ["/Users/yoyi/codex-workflow-mario-test"],
    "C1 workspace-write 才给写入根，限死在 project_root",
  );
  const readOnlyReq = buildNodeDispatchRequest({
    nodeId: "n",
    projectRoot: "/Users/yoyi/codex-workflow-mario-test",
    data: { ...rich, sandbox: "read-only" },
    instructionId: "instr-ro",
  });
  assertDeepEqual(
    readOnlyReq.user_reviewed_instruction.allowed_write_roots,
    [],
    "C1 read-only 节点不给任何写入根",
  );

  // P3 A 映射 · 实验派发请求只带会话策略 + 节点名 + prompt + 沙箱；不带 project_root /
  // work_item_id（后端硬锁固定测试项目 + 自动建临时票）。resume 带 trim 后的 thread_id；new → null。
  const expResume = buildExperimentNodeDispatchRequest({
    ...rich,
    name: "实验节点A",
    prompt: "建实验证明文件",
    sandbox: "workspace-write",
    session: { mode: "resume", thread_id: "  019e7738-thread  " },
  });
  assert(expResume.session_mode === "resume", "A 映射：resume 策略透传");
  assert(expResume.thread_id === "019e7738-thread", "A 映射：resume thread_id 应 trim 后透传");
  assert(expResume.summary === "实验节点A", "A 映射：summary 取节点名");
  assert(expResume.objective === "建实验证明文件", "A 映射：objective 取 prompt");
  assert(expResume.sandbox_mode === "workspace-write", "A 映射：sandbox 取节点 data");
  assert(!("project_root" in expResume) && !("work_item_id" in expResume), "A 映射：请求不带 project_root / work_item_id");
  const expNew = buildExperimentNodeDispatchRequest({ ...rich, session: { mode: "new" } });
  assert(expNew.session_mode === "new" && expNew.thread_id === null, "A 映射：new 策略 thread_id=null（resume-only 决策下 readiness 已拦，builder 仍纯映射）");

  // 引擎抽出 P0 · 行为不变锚点：CanvasView 现在是 WorkflowCanvasEngine + experiment
  // config 的薄壳。experiment 面 config 必须复现旧 CanvasView 的实验语境——它注入
  // 的 boundary 必须就是旧代码硬编码的同一个 experimentCanvasBoundary（引用相同），
  // 能力为自由编排全开、真跑目标=固定测试项目、事实源=自由画布定义、无项目规则条。
  assert(experimentCanvasSurfaceConfig.kind === "experiment", "P0 实验面 config kind=experiment");
  assert(
    experimentCanvasSurfaceConfig.boundary === experimentCanvasBoundary,
    "P0 实验面 config 注入的 boundary 必须是旧 CanvasView 同一个 experimentCanvasBoundary（行为不变）",
  );
  assert(
    experimentCanvasSurfaceConfig.realRunTarget === "fixed_test_project",
    "P0 实验面真跑目标仍为固定测试项目（双闸靶子不变）",
  );
  assert(
    experimentCanvasSurfaceConfig.authority === "free_canvas_def",
    "P0 实验面事实源 = 自由画布定义，非项目事实源",
  );
  assert(
    experimentCanvasSurfaceConfig.capabilities.edit &&
      experimentCanvasSurfaceConfig.capabilities.connect &&
      experimentCanvasSurfaceConfig.capabilities.createNode &&
      experimentCanvasSurfaceConfig.capabilities.saveTemplate &&
      experimentCanvasSurfaceConfig.capabilities.manualOrchestrate,
    "P0 实验面自由编排全功能全开（与旧 CanvasView 一致）",
  );
  assert(
    experimentCanvasSurfaceConfig.showProjectRuleBar === false,
    "P0 实验面无项目规则状态条（§11.2 是项目面的）",
  );

  // B · scope 是显式持久化字段，不再靠 project_root 派生：显式值优先；旧画布（无
  // scope）按 project_root 有无回落迁移；并留出「草案设计好、未绑项目」中间态
  // （scope=project 但 project_root 空）。
  assert(
    canvasScope({ scope: "experiment", project_root: "/some/project" }) === "experiment",
    "B 显式 scope 优先于 project_root 派生",
  );
  assert(
    canvasScope({ scope: "project", project_root: null }) === "project",
    "B 显式 scope=project 即使未绑 project_root 也保留（草案未绑中间态）",
  );
  assert(
    canvasScope({ project_root: "/some/project" }) === "project",
    "B 向后兼容：旧画布无 scope、有 project_root → 派生 project",
  );
  assert(
    canvasScope({}) === "experiment",
    "B 向后兼容：旧画布无 scope、无 project_root → 派生 experiment",
  );

  // C/D · 项目面 config（两面一引擎，2026-06-21 真机反馈版）：与实验面同引擎、不同调用。
  // project kind、事实源 = workflow-state 读模型、默认只读运行状态视图（编辑是动作不是视图，
  // 已删方案/运行切换）、真跑目标 = 绑定项目、有项目规则状态条、embedded（host 提供 chrome）、
  // boundary = 项目工作流边界。
  const projectCfg = projectCanvasSurfaceConfig("/Users/yoyi/some-project");
  assert(projectCfg.kind === "project", "C 项目面 config kind=project");
  assert(projectCfg.boundary === projectWorkflowCanvasBoundary, "C 项目面 config 注入项目工作流边界");
  assert(projectCfg.projectRoot === "/Users/yoyi/some-project", "C 项目面 config 带 projectRoot");
  assert(projectCfg.authority === "workflow_state_read_model", "C 项目面事实源 = workflow-state 派生读模型");
  assert(projectCfg.realRunTarget === "bound_project", "C 项目面真跑目标 = 绑定项目（非固定测试项目）");
  assertDeepEqual(projectCfg.views, ["run_state"], "D 项目面默认只读运行状态视图（编辑是动作不是视图，真机后删切换）");
  assert(projectCfg.showProjectRuleBar === true, "D 项目面有规则状态条（§11.2）");
  assert(projectCfg.embedded === true, "C 项目面 embedded：host 提供 head / 规则条 / 编辑动作");
  // 两面差异（同引擎双调用）：experiment 真跑打固定测试项目 / 自由画布事实源 / 无规则条；
  // project 真跑打绑定项目 / workflow-state 读模型事实源 / 有规则条。（上面逐项已锚定；
  // 两者字面量类型本就不同，TS 编译期即保证有别，无需再运行期比较。）
}
