import { useCallback, useEffect, useMemo, useState } from "react";
import {
  ReactFlow,
  Background,
  Controls,
  MiniMap,
  Handle,
  Position,
  addEdge,
  applyEdgeChanges,
  applyNodeChanges,
  useReactFlow,
  type Connection,
  type Edge,
  type EdgeChange,
  type Node,
  type NodeChange,
  type NodeProps,
  type NodeTypes,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";

import {
  canvasLoad,
  canvasSave,
  deleteWorkflowTemplate,
  executeWorkflowNodeDispatch,
  executeExperimentNodeDispatch,
  listWorkflowTemplates,
  loadWorkflowTemplate,
  saveWorkflowTemplate,
} from "../lib/tauri";
import type { CanvasSurfaceBoundary } from "../lib/canvasSurfaceBoundaries";
import type { CanvasSurfaceConfig } from "../lib/canvasSurfaceConfig";
import { pathTail } from "../lib/format";
import {
  NODE_KIND_PRESETS,
  SANDBOX_PRESETS,
  STATUS_PRESETS,
  buildNodeDispatchRequest,
  buildExperimentNodeDispatchRequest,
  canvasNodeToData,
  canvasScope,
  createNodeData,
  dataToCanvasNode,
  instantiateTemplateGraph,
  kindAccent,
  kindLabel,
  nodeRunReadiness,
  statusTone,
  type CanvasCustomField,
  type CanvasNodeData,
  type SessionPolicy,
} from "../lib/canvasNodeData";
import type {
  CanvasDefinition,
  SessionRecord,
  WorkflowTemplate,
  WorkflowTemplateSummary,
} from "../lib/types";

type WorkflowCanvasEngineProps = {
  config: CanvasSurfaceConfig;
  canvasId: string;
  sessions: SessionRecord[];
  onNotice: (msg: string) => void;
};

type FlowNode = Node<CanvasNodeData>;

function toFlowNodes(canvas: CanvasDefinition): FlowNode[] {
  return canvas.nodes.map((n) => ({
    id: n.id,
    position: n.position,
    data: canvasNodeToData(n),
    type: "canvasNode",
  }));
}

function toFlowEdges(canvas: CanvasDefinition): Edge[] {
  return canvas.edges.map((e) => ({
    id: e.id,
    source: e.from,
    target: e.to,
    animated: false,
  }));
}

function fromFlow(canvas: CanvasDefinition, nodes: FlowNode[], edges: Edge[]): CanvasDefinition {
  const merged = nodes.map((fn) => {
    const prior = canvas.nodes.find((n) => n.id === fn.id);
    return dataToCanvasNode(fn.id, fn.data, fn.position, prior?.warnings ?? []);
  });
  return {
    ...canvas,
    // Stamp scope explicitly on save: legacy canvases (no scope) get migrated to a
    // concrete value via the project_root fallback; an explicit scope is preserved.
    scope: canvasScope(canvas),
    nodes: merged,
    edges: edges.map((e) => ({ id: e.id, from: String(e.source), to: String(e.target) })),
    updated_at: new Date().toISOString(),
  };
}

export function WorkflowCanvasEngine({ config, canvasId, sessions, onNotice }: WorkflowCanvasEngineProps) {
  const [canvas, setCanvas] = useState<CanvasDefinition | null>(null);
  const [nodes, setNodes] = useState<FlowNode[]>([]);
  const [edges, setEdges] = useState<Edge[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [dirty, setDirty] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [templates, setTemplates] = useState<WorkflowTemplateSummary[]>([]);
  const [templateTitle, setTemplateTitle] = useState("");
  const [confirmDeleteId, setConfirmDeleteId] = useState<string | null>(null);
  const [bindOpen, setBindOpen] = useState(false);
  const [bindInput, setBindInput] = useState("");
  // 实验画布「清空 / 新建画布」两步确认（Tauri webview 不弹 window.confirm）。
  const [confirmReset, setConfirmReset] = useState<null | "clear" | "new">(null);

  const reload = useCallback(async () => {
    setBusy(true);
    setError(null);
    try {
      const loaded = await canvasLoad(canvasId);
      // Project surface: bind the canvas to its project + stamp scope on load so
      // a fresh project canvas isn't mis-derived as experiment (default-by-surface
      // per plan B). Persists on the next save (fromFlow stamps scope).
      const c =
        config.kind === "project" && config.projectRoot && !loaded.project_root
          ? { ...loaded, project_root: config.projectRoot, scope: "project" as const }
          : loaded;
      setCanvas(c);
      setNodes(toFlowNodes(c));
      setEdges(toFlowEdges(c));
      setDirty(false);
    } catch (e) {
      setError(messageOf(e));
    } finally {
      setBusy(false);
    }
  }, [canvasId, config.kind, config.projectRoot]);

  const refreshTemplates = useCallback(async () => {
    try {
      setTemplates(await listWorkflowTemplates());
    } catch (e) {
      // Non-fatal: templates panel just stays empty if the store can't be read.
      onNotice(`读取成熟模式失败：${messageOf(e)}`);
    }
  }, [onNotice]);

  useEffect(() => {
    void reload();
    void refreshTemplates();
  }, [reload, refreshTemplates]);

  const onNodesChange = useCallback((changes: NodeChange<FlowNode>[]) => {
    setNodes((curr) => applyNodeChanges(changes, curr));
    if (changes.some((c) => c.type === "position" || c.type === "remove")) {
      setDirty(true);
    }
  }, []);

  const onEdgesChange = useCallback((changes: EdgeChange[]) => {
    setEdges((curr) => applyEdgeChanges(changes, curr));
    if (changes.some((c) => c.type === "remove")) {
      setDirty(true);
    }
  }, []);

  const onConnect = useCallback((params: Connection) => {
    setEdges((curr) =>
      addEdge(
        { ...params, id: `e-${Date.now()}-${Math.random().toString(36).slice(2, 6)}` },
        curr,
      ),
    );
    setDirty(true);
  }, []);

  const { screenToFlowPosition } = useReactFlow();

  const addNode = useCallback((kind: string, position?: { x: number; y: number }) => {
    const id = `${kind}-${Date.now().toString(36).slice(-5)}-${Math.random().toString(36).slice(2, 5)}`;
    setNodes((curr) => {
      const offset = curr.length * 28;
      const fn: FlowNode = {
        id,
        position: position ?? { x: 80 + offset, y: 80 + offset },
        data: createNodeData(kind),
        type: "canvasNode",
      };
      return [...curr, fn];
    });
    setSelected(id);
    setDirty(true);
  }, []);

  const onPaneDoubleClick = useCallback(
    (event: React.MouseEvent) => {
      // Double-click empty canvas → drop a fresh node where the cursor is. Only
      // when the pane itself was hit, so double-clicking a node still selects it.
      const target = event.target as HTMLElement | null;
      if (!target || !target.classList.contains("react-flow__pane")) return;
      const position = screenToFlowPosition({ x: event.clientX, y: event.clientY });
      addNode("subagent", position);
    },
    [addNode, screenToFlowPosition],
  );

  const updateSelected = useCallback(
    (patch: Partial<FlowNode["data"]>) => {
      if (!selected) return;
      setNodes((curr) =>
        curr.map((n) =>
          n.id === selected ? { ...n, data: { ...n.data, ...patch } } : n,
        ),
      );
      setDirty(true);
    },
    [selected],
  );

  const save = useCallback(async () => {
    if (!canvas) return;
    setBusy(true);
    try {
      const next = fromFlow(canvas, nodes, edges);
      await canvasSave(next);
      setCanvas(next);
      setDirty(false);
      onNotice("画布已保存。");
    } catch (e) {
      setError(messageOf(e));
    } finally {
      setBusy(false);
    }
  }, [canvas, nodes, edges, onNotice]);

  // B2 · 把当前画布图存成可复用的「成熟模式」（workflow template）。纯数据，不执行。
  const saveAsTemplate = useCallback(async () => {
    if (!canvas) return;
    const title = templateTitle.trim() || canvas.display_name || "未命名工作流";
    setBusy(true);
    try {
      const now = new Date().toISOString();
      const graph = fromFlow(canvas, nodes, edges);
      const template: WorkflowTemplate = {
        schema_version: "workflow-template-v1",
        template_id: `wft-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 7)}`,
        title: title.trim() || "未命名工作流",
        scope: canvas.project_root ? "project" : "global",
        project_root: canvas.project_root ?? null,
        source_canvas_id: canvas.canvas_id,
        version: 1,
        nodes: graph.nodes,
        edges: graph.edges,
        created_at: now,
        updated_at: now,
        warnings: [],
      };
      await saveWorkflowTemplate(template);
      await refreshTemplates();
      setTemplateTitle("");
      onNotice(`已存为成熟模式：${template.title}`);
    } catch (e) {
      setError(messageOf(e));
    } finally {
      setBusy(false);
    }
  }, [canvas, nodes, edges, onNotice, refreshTemplates, templateTitle]);

  // B3 · 从成熟模式起一张新工作流：节点 id 全部重置，连线随新 id 重映射，载入当前画布供编辑。
  const instantiateFromTemplate = useCallback(
    async (templateId: string) => {
      setBusy(true);
      try {
        const template = await loadWorkflowTemplate(templateId);
        const graph = instantiateTemplateGraph(
          template.nodes,
          template.edges,
          (node) => `${node.kind ?? node.role}-${Date.now().toString(36).slice(-5)}-${Math.random().toString(36).slice(2, 5)}`,
        );
        setNodes(graph.nodes.map((n) => ({ id: n.id, position: n.position, data: n.data, type: "canvasNode" })));
        setEdges(graph.edges.map((e) => ({ id: e.id, source: e.from, target: e.to, animated: false })));
        setSelected(null);
        setDirty(true);
        onNotice(`已从成熟模式「${template.title}」起新工作流（${graph.nodes.length} 节点）；记得保存。`);
      } catch (e) {
        setError(messageOf(e));
      } finally {
        setBusy(false);
      }
    },
    [onNotice],
  );

  const removeTemplate = useCallback(
    async (templateId: string, title: string) => {
      setConfirmDeleteId(null);
      setBusy(true);
      try {
        await deleteWorkflowTemplate(templateId);
        await refreshTemplates();
        onNotice(`已删除成熟模式：${title}`);
      } catch (e) {
        setError(messageOf(e));
      } finally {
        setBusy(false);
      }
    },
    [onNotice, refreshTemplates],
  );

  // P3 · 运行此节点。前端不判闸——默认安全态由后端 path-lock 守（非固定测试项目 → blocked、零执行）。
  //   实验面（A 映射）：调 execute_experiment_node_dispatch——目标后端硬锁固定测试项目、自动建临时
  //     work_item，无需手填票号/绑项目；实验节点自由 id 对不上 workflow-state，不能走下面那条。
  //   项目面（C 映射）：节点=workflow-state work_item，走 execute_workflow_node_dispatch（带 project_root
  //     + work_item_id + 已绑会话）。
  const runSelectedNode = useCallback(async () => {
    const node = nodes.find((n) => n.id === selected);
    if (!node || !canvas) return;
    const surface = config.kind === "experiment" ? "experiment" : "project";
    const readiness = nodeRunReadiness(node.data, surface);
    if (!readiness.ready) {
      onNotice(`无法运行：${readiness.reason}`);
      return;
    }
    setBusy(true);
    try {
      if (surface === "experiment") {
        const result = await executeExperimentNodeDispatch(buildExperimentNodeDispatchRequest(node.data));
        onNotice(`已在固定测试项目派发实验节点「${node.data.name}」。返回：${summarizeRunResult(result)}`);
      } else {
        const request = buildNodeDispatchRequest({
          nodeId: node.id,
          projectRoot: canvas.project_root ?? "",
          data: node.data,
          instructionId: `canvas-run-${node.id}-${Date.now().toString(36)}`,
        });
        const result = await executeWorkflowNodeDispatch(request);
        onNotice(`已派发节点「${node.data.name}」。返回：${summarizeRunResult(result)}`);
      }
    } catch (e) {
      // 后端 path-lock 闸对非固定测试项目返回 Err（blocked message）；实验「新建会话」不启用（resume-only）也会
      // 在此报清楚（拦路石②）。任一情况都没有 codex 真跑。
      onNotice(`运行被拦截或失败：${messageOf(e)}`);
    } finally {
      setBusy(false);
    }
  }, [nodes, selected, canvas, config.kind, onNotice]);

  // P2 · 作用域升级：实验画布「绑定到项目」→ 设 project_root，画布变项目画布。
  // 只改定义里的 project_root（保存后持久化）；不碰双闸——真跑权限仍由后端定。
  // 页面内输入（Tauri webview 不弹 window.prompt、旧版会静默失败）：点按钮开内联输入框。
  const openBind = useCallback(() => {
    if (!canvas) return;
    setBindInput(canvas.project_root ?? "");
    setBindOpen(true);
  }, [canvas]);
  // scope is explicit + persisted now: binding graduates the canvas to a project
  // surface, unbinding returns it to experiment. (P3 真跑权限仍由后端双闸定。)
  const applyBind = useCallback(() => {
    if (!canvas) return;
    const trimmed = bindInput.trim();
    setCanvas({ ...canvas, project_root: trimmed || null, scope: trimmed ? "project" : "experiment" });
    setDirty(true);
    setBindOpen(false);
    onNotice(trimmed ? `画布已绑定项目：${trimmed}（记得保存）` : "已取消项目绑定，回到实验画布（记得保存）");
  }, [canvas, bindInput, onNotice]);

  // 真机反馈：实验画布需要「清空画布 / 新建画布」。清空 = 倒空当前图（留 id/名）；
  // 新建 = 重置成一张空白新工作流（清图 + 名归位 + 退回实验作用域）。都置 dirty，存盘生效；
  // 纯前端、零执行。（实验面单画布模型；多画布库后置。）
  const clearCanvas = useCallback(() => {
    setNodes([]);
    setEdges([]);
    setSelected(null);
    setDirty(true);
    setConfirmReset(null);
    onNotice("画布已清空（记得保存）。");
  }, [onNotice]);
  const newCanvas = useCallback(() => {
    setCanvas((curr) => (curr ? { ...curr, display_name: "新工作流", project_root: null, scope: "experiment" } : curr));
    setNodes([]);
    setEdges([]);
    setSelected(null);
    setDirty(true);
    setConfirmReset(null);
    onNotice("已重置为新的空白实验画布（记得保存）。");
  }, [onNotice]);

  const selectedNode = useMemo(
    () => nodes.find((n) => n.id === selected) ?? null,
    [nodes, selected],
  );

  const nodeTypes = useMemo<NodeTypes>(() => ({ canvasNode: CanvasFlowNode }), []);

  // Surface-driven chrome (one engine, two调用): the project surface relabels and
  // (when embedded) lets the host render the head / rule bar / view toggle.
  const isProject = config.kind === "project";
  const surfaceEyebrow = isProject ? "项目工作流画布" : "实验 / 模板画布";

  if (error) {
    return (
      <section className="canvas-view canvas-load-fallback">
        <header className="canvas-head">
          <div>
            <p className="eyebrow">{surfaceEyebrow}</p>
            <h2>画 布 暂 未 接 入</h2>
          </div>
          <span className="canvas-id">canvas_id={canvasId}</span>
        </header>
        <ExperimentCanvasBoundaryPanel boundary={config.boundary} />
        <div className="canvas-fallback-stage">
          <svg className="threads" viewBox="0 0 1000 520" preserveAspectRatio="none" aria-hidden="true">
            <path className="lit" d="M190 250 C310 150 430 150 500 245 S710 350 820 210" />
            <path d="M500 245 C420 330 350 375 240 395" />
            <path d="M500 245 C585 325 655 380 780 390" />
            <path className="feather" d="M230 395 C370 410 640 415 785 390" />
          </svg>
          <div className="canvas-fallback-node main">
            <span>模板</span>
            <strong>当前页面不在 Tauri 窗口中运行</strong>
            <em>{error}</em>
          </div>
          <div className="canvas-fallback-node left">
            <span>边界</span>
            <strong>不伪造数据</strong>
            <em>需要桌面壳读取本地画布</em>
          </div>
          <div className="canvas-fallback-node right">
            <span>动作</span>
            <strong>重新读取</strong>
            <button onClick={() => void reload()}>重试</button>
          </div>
        </div>
      </section>
    );
  }

  if (!canvas) {
    return <section className="canvas-view">载入{isProject ? "项目工作流" : "实验 / 模板"}画布……</section>;
  }

  const scope = canvasScope(canvas);

  return (
    <section className="canvas-view canvas-view-hud" aria-label={surfaceEyebrow}>
      {config.embedded ? null : (
        <header className="canvas-head">
          <div>
            <p className="eyebrow">{surfaceEyebrow}</p>
            <h2>{canvas.display_name}</h2>
            <div className="canvas-scope" data-scope={scope}>
              {scope === "project" ? (
                <span className="canvas-scope-chip project" title={canvas.project_root ?? undefined}>
                  项目画布{canvas.project_root ? ` · ${pathTail(canvas.project_root)}` : " · 草案（未绑项目）"}
                </span>
              ) : (
                <span className="canvas-scope-chip experiment">实验画布 · 未绑项目（真跑只打固定测试项目）</span>
              )}
              {/* 绑定/改绑是实验面把草案「毕业」到项目的入口；项目面已由 surface 绑定，不重复提供。 */}
              {isProject ? null : bindOpen ? (
                <span className="canvas-scope-bind-edit">
                  <input
                    type="text"
                    value={bindInput}
                    onChange={(e) => setBindInput(e.target.value)}
                    placeholder="项目根目录绝对路径（留空=保持实验画布）"
                    disabled={busy}
                  />
                  <button type="button" className="canvas-scope-bind" onClick={() => void applyBind()} disabled={busy}>确认</button>
                  <button type="button" onClick={() => setBindOpen(false)} disabled={busy}>取消</button>
                </span>
              ) : (
                <button type="button" className="canvas-scope-bind" onClick={() => void openBind()} disabled={busy}>
                  {canvas.project_root ? "改绑项目" : "绑定到项目"}
                </button>
              )}
            </div>
          </div>
          <span className="canvas-id">canvas_id={canvas.canvas_id}</span>
        </header>
      )}
      {/* P2 四周分布（fullbleed-hud 重设计 §2）：画布吃满 .canvas-engine-stage（position:relative;
          flex:1），调色板 / 节点编辑 / 动作条作为四边绝对定位 HUD 悬浮。HUD 容器 pointer-events:none、
          内部控件 pointer-events:auto，不挡画布平移缩放 / 空白双击建节点。共享引擎：实验 + 项目编辑
          同一套，两面都拿到同一个四边布局。 */}
      <div className="canvas-engine-stage" onDoubleClick={onPaneDoubleClick}>
        {nodes.length === 0 ? (
          <div className="canvas-empty-guide" role="note">
            <strong>空白画布</strong>
            <span>左边「节点调色板」点一个种类，或在空白处双击，建第一个节点。</span>
          </div>
        ) : null}
        <ReactFlow
          nodes={nodes}
          edges={edges}
          nodeTypes={nodeTypes}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onConnect={onConnect}
          onSelectionChange={({ nodes: sel }) => setSelected(sel[0]?.id ?? null)}
          panOnScroll
          zoomOnScroll={false}
          zoomOnPinch
          zoomOnDoubleClick={false}
          nodesDraggable
          nodesConnectable
          elementsSelectable
          fitView
        >
          <Background />
          <Controls />
          <MiniMap />
        </ReactFlow>

        {/* 左边 HUD：节点调色板（编辑态竖排）。点种类建节点；空白双击也可建。 */}
        <div className="canvas-engine-hud canvas-engine-hud-left" aria-label="节点调色板">
          <div className="canvas-palette">
            {NODE_KIND_PRESETS.map((preset) => (
              <button
                key={preset.kind}
                type="button"
                className="canvas-palette-chip"
                style={{ borderColor: preset.accent }}
                onClick={() => addNode(preset.kind)}
                disabled={busy}
                title={preset.hint}
              >
                <span className="canvas-palette-dot" style={{ background: preset.accent }} aria-hidden="true" />
                {preset.label}
              </button>
            ))}
          </div>
        </div>

        {/* 右边 HUD：选中节点才出现的节点面板（名称/提示词等；开发者字段折进默认收起区）。 */}
        {selectedNode ? (
          <div className="canvas-engine-hud canvas-engine-hud-right" aria-label="节点编辑">
            <NodeEditor
              node={selectedNode}
              sessions={sessions}
              surface={config.kind === "experiment" ? "experiment" : "project"}
              onChange={updateSelected}
              onRun={() => void runSelectedNode()}
              disabled={busy}
            />
          </div>
        ) : null}

        {/* 底边 HUD：▶运行选中节点 + 保存 + 紧凑动作（重载/清空/新建）+ 成熟模式折进默认收起区。 */}
        <div className="canvas-engine-hud canvas-engine-hud-bottom" aria-label="画布动作">
          <button
            type="button"
            className="canvas-engine-run"
            onClick={() => void runSelectedNode()}
            disabled={busy || !selectedNode || !nodeRunReadiness(selectedNode.data, config.kind === "experiment" ? "experiment" : "project").ready}
            title={selectedNode ? "运行选中节点（经双闸；默认安全态被后端挡下、零执行）" : "先选中一个节点"}
          >
            ▶ 运行选中节点
          </button>
          <button type="button" className="canvas-engine-save" onClick={() => void save()} disabled={busy || !dirty}>
            {dirty ? "保存（未保存）" : "保存"}
          </button>
          <button type="button" onClick={() => void reload()} disabled={busy}>重载</button>
          {/* 实验画布「清空 / 新建画布」（项目面用项目页的「新建工作流」，这里不重复）。*/}
          {!isProject ? (
            confirmReset ? (
              <span className="canvas-reset-confirm">
                <span className="canvas-reset-confirm-text">
                  确认{confirmReset === "clear" ? "清空" : "重置"}？未保存改动会丢失。
                </span>
                <button onClick={() => (confirmReset === "clear" ? clearCanvas() : newCanvas())} disabled={busy}>
                  确认
                </button>
                <button onClick={() => setConfirmReset(null)} disabled={busy}>取消</button>
              </span>
            ) : (
              <>
                <button onClick={() => setConfirmReset("clear")} disabled={busy || nodes.length === 0}>清空画布</button>
                <button onClick={() => setConfirmReset("new")} disabled={busy}>新建画布</button>
              </>
            )
          ) : null}

          {/* P3 砍杂项：成熟模式 + 边界说明折进默认收起的「更多」区，日常看不到。 */}
          <details className="canvas-engine-more">
            <summary>更多（成熟模式 / 边界）</summary>
            <fieldset>
              <legend>成熟模式</legend>
              <label>
                模式标题
                <input
                  type="text"
                  value={templateTitle}
                  onChange={(e) => setTemplateTitle(e.target.value)}
                  placeholder={canvas?.display_name || "未命名工作流"}
                  disabled={busy}
                />
              </label>
              <button onClick={() => void saveAsTemplate()} disabled={busy || nodes.length === 0}>
                ＋ 把这张存成成熟模式
              </button>
              {templates.length === 0 ? null : (
                <ul className="canvas-template-list">
                  {templates.map((tpl) => (
                    <li key={tpl.template_id} className="canvas-template-item">
                      <div className="ct-meta">
                        <strong>{tpl.title}</strong>
                        <span>{tpl.scope === "project" ? "项目私有" : "全局"} · {tpl.node_count} 节点 / {tpl.edge_count} 连线</span>
                      </div>
                      <div className="ct-actions">
                        <button onClick={() => void instantiateFromTemplate(tpl.template_id)} disabled={busy}>
                          起新工作流
                        </button>
                        {confirmDeleteId === tpl.template_id ? (
                          <>
                            <button
                              className="ct-delete"
                              onClick={() => void removeTemplate(tpl.template_id, tpl.title)}
                              disabled={busy}
                            >
                              确认删除
                            </button>
                            <button onClick={() => setConfirmDeleteId(null)} disabled={busy}>
                              取消
                            </button>
                          </>
                        ) : (
                          <button
                            className="ct-delete"
                            onClick={() => setConfirmDeleteId(tpl.template_id)}
                            disabled={busy}
                            aria-label="删除成熟模式"
                          >
                            ×
                          </button>
                        )}
                      </div>
                    </li>
                  ))}
                </ul>
              )}
            </fieldset>
            <ExperimentCanvasBoundaryPanel boundary={config.boundary} />
          </details>
        </div>
      </div>
    </section>
  );
}

// A1 · custom React Flow node. Beyond a colour block: title + kind badge +
// status light + a few key fields, with explicit handles so any node can be
// freely wired to any other.
function CanvasFlowNode({ data, selected }: NodeProps<FlowNode>) {
  const accent = kindAccent(data.kind);
  const preview = data.prompt.trim().split(/\r?\n/)[0] ?? "";
  return (
    <div className={`canvas-node-card${selected ? " selected" : ""}`} style={{ borderColor: accent }}>
      <Handle type="target" position={Position.Left} />
      <header className="cnc-head">
        <span className="cnc-kind" style={{ background: accent }}>{kindLabel(data.kind)}</span>
        <span className="cnc-status" title={`状态：${data.status}`}>
          <span className="cnc-status-dot" style={{ background: statusTone(data.status) }} aria-hidden="true" />
          {data.status}
        </span>
      </header>
      <strong className="cnc-name">{data.name || "未命名节点"}</strong>
      <dl className="cnc-fields">
        {data.skill ? (
          <div><dt>技能</dt><dd>{data.skill}</dd></div>
        ) : null}
        <div><dt>沙箱</dt><dd>{data.sandbox}</dd></div>
        <div><dt>会话</dt><dd>{sessionPolicyLabel(data.session)}</dd></div>
        {preview ? (
          <div className="cnc-prompt"><dt>提示</dt><dd>{preview}</dd></div>
        ) : null}
        {data.fields.length > 0 ? (
          <div><dt>自定义</dt><dd>{data.fields.length} 项</dd></div>
        ) : null}
      </dl>
      <Handle type="source" position={Position.Right} />
    </div>
  );
}

function NodeEditor({
  node,
  sessions,
  surface,
  onChange,
  onRun,
  disabled,
}: {
  node: FlowNode;
  sessions: SessionRecord[];
  surface: "experiment" | "project";
  onChange: (patch: Partial<CanvasNodeData>) => void;
  onRun: () => void;
  disabled: boolean;
}) {
  const data = node.data;
  const readiness = nodeRunReadiness(data, surface);
  const updateField = (id: string, patch: Partial<CanvasCustomField>) => {
    onChange({ fields: data.fields.map((f) => (f.id === id ? { ...f, ...patch } : f)) });
  };
  const addField = () => {
    const id = `field-${Date.now().toString(36).slice(-5)}-${Math.random().toString(36).slice(2, 5)}`;
    onChange({ fields: [...data.fields, { id, key: "", value: "" }] });
  };
  const removeField = (id: string) => {
    onChange({ fields: data.fields.filter((f) => f.id !== id) });
  };
  return (
    <div className="canvas-node-editor">
      <p className="cne-id">
        <strong>编号：</strong> <code>{node.id}</code>
      </p>
      <label>
        名称
        <input
          type="text"
          value={data.name}
          onChange={(e) => onChange({ name: e.target.value })}
          disabled={disabled}
        />
      </label>
      <label>
        提示词 prompt
        <textarea
          value={data.prompt}
          onChange={(e) => onChange({ prompt: e.target.value })}
          rows={3}
          disabled={disabled}
        />
      </label>
      {/* P3 砍杂项：种类 / 状态 / 技能 / 自定义字段折进默认收起的「字段 / 高级」区，日常看不到。 */}
      <details className="canvas-advanced-details">
        <summary>字段 / 高级（种类 · 状态 · 技能 · 自定义字段）</summary>
        <label>
          种类（自由）
          <input
            type="text"
            list="canvas-node-kinds"
            value={data.kind}
            onChange={(e) => onChange({ kind: e.target.value })}
            disabled={disabled}
          />
          <datalist id="canvas-node-kinds">
            {NODE_KIND_PRESETS.map((preset) => (
              <option key={preset.kind} value={preset.kind} />
            ))}
          </datalist>
        </label>
        <label>
          状态灯
          <input
            type="text"
            list="canvas-node-status"
            value={data.status}
            onChange={(e) => onChange({ status: e.target.value })}
            disabled={disabled}
          />
          <datalist id="canvas-node-status">
            {STATUS_PRESETS.map((status) => (
              <option key={status} value={status} />
            ))}
          </datalist>
        </label>
        <label>
          技能 / 岗位
          <input
            type="text"
            value={data.skill ?? ""}
            onChange={(e) => onChange({ skill: e.target.value })}
            disabled={disabled}
          />
        </label>
        <fieldset className="canvas-custom-fields">
          <legend>自定义字段</legend>
          {data.fields.map((field) => (
            <div className="canvas-custom-field-row" key={field.id}>
              <input
                type="text"
                aria-label="字段名"
                placeholder="字段名"
                value={field.key}
                onChange={(e) => updateField(field.id, { key: e.target.value })}
                disabled={disabled}
              />
              <input
                type="text"
                aria-label="字段值"
                placeholder="字段值"
                value={field.value}
                onChange={(e) => updateField(field.id, { value: e.target.value })}
                disabled={disabled}
              />
              <button type="button" onClick={() => removeField(field.id)} disabled={disabled} aria-label="删除字段">
                ×
              </button>
            </div>
          ))}
          <button type="button" onClick={addField} disabled={disabled}>+ 字段</button>
        </fieldset>
      </details>
      <details className="canvas-exec-details">
        <summary>接执行（真跑用）</summary>
        <label>
          沙箱
          <select
            value={data.sandbox}
            onChange={(e) => onChange({ sandbox: e.target.value })}
            disabled={disabled}
          >
            {SANDBOX_PRESETS.map((sandbox) => (
              <option key={sandbox} value={sandbox}>{sandbox}</option>
            ))}
          </select>
        </label>
        <div className="canvas-session-policy">
          <span className="csp-label">会话（执行上下文）</span>
          <div className="canvas-segmented" role="group" aria-label="会话策略">
            <button
              type="button"
              className={`csp-seg${data.session.mode === "new" ? " active" : ""}`}
              onClick={() => onChange({ session: { mode: "new" } })}
              disabled={disabled}
              aria-pressed={data.session.mode === "new"}
            >
              新建会话
            </button>
            <button
              type="button"
              className={`csp-seg${data.session.mode === "resume" ? " active" : ""}`}
              onClick={() =>
                onChange({
                  session: {
                    mode: "resume",
                    thread_id: data.session.mode === "resume" ? data.session.thread_id : "",
                  },
                })
              }
              disabled={disabled}
              aria-pressed={data.session.mode === "resume"}
            >
              续已有会话
            </button>
          </div>
          {data.session.mode === "resume" ? (
            <label className="csp-resume">
              续哪条会话
              <input
                type="text"
                list="canvas-sessions"
                value={data.session.thread_id}
                onChange={(e) => onChange({ session: { mode: "resume", thread_id: e.target.value } })}
                placeholder="粘贴 / 搜 thread id"
                disabled={disabled}
              />
              <datalist id="canvas-sessions">
                {sessions.map((s) => (
                  <option key={s.thread_id} value={s.thread_id}>{sessionLabel(s)}</option>
                ))}
              </datalist>
            </label>
          ) : null}
        </div>
        <label>
          工作项 ID
          <input
            type="text"
            value={data.work_item_id}
            onChange={(e) => onChange({ work_item_id: e.target.value })}
            placeholder="workflow-state work_item_id"
            disabled={disabled}
          />
        </label>
        <div className="canvas-node-run">
          <button
            type="button"
            className="canvas-node-run-btn"
            onClick={onRun}
            disabled={disabled || !readiness.ready}
            title={readiness.reason ?? "经双闸命令运行此节点"}
          >
            ▶ 运行此节点
          </button>
          {readiness.ready ? null : (
            <p className="canvas-hint">运行前提未满足：{readiness.reason}</p>
          )}
        </div>
      </details>
    </div>
  );
}

function sessionPolicyLabel(session: SessionPolicy): string {
  if (session.mode === "new") return "新建";
  return session.thread_id ? `续 ${session.thread_id.slice(0, 8)}` : "续(未选)";
}

function sessionLabel(s: SessionRecord): string {
  const title = s.title || s.thread_id;
  const tail = s.thread_id.slice(0, 8);
  return `${title} (${tail})`;
}

function ExperimentCanvasBoundaryPanel({ boundary }: { boundary: CanvasSurfaceBoundary }) {
  return (
    <section className="canvas-boundary-panel" aria-label="实验画布边界">
      <div className="canvas-boundary-heading">
        <div>
          <p className="eyebrow">{boundary.eyebrow}</p>
          <h3>{boundary.title}</h3>
          <p>{boundary.summary}</p>
        </div>
        <span>只读边界</span>
      </div>
      <div className="canvas-boundary-badges">
        {boundary.badges.map((badge) => (
          <span key={badge}>{badge}</span>
        ))}
      </div>
      <dl className="canvas-boundary-list">
        {boundary.items.map((item) => (
          <div key={item.item_id}>
            <dt>{item.label}</dt>
            <dd>{item.value}</dd>
          </div>
        ))}
      </dl>
    </section>
  );
}

function summarizeRunResult(result: unknown): string {
  if (result && typeof result === "object") {
    const rec = result as Record<string, unknown>;
    const dispatch = rec.dispatch;
    if (dispatch && typeof dispatch === "object") {
      const d = dispatch as Record<string, unknown>;
      const id = typeof d.dispatch_id === "string" ? d.dispatch_id : "?";
      const state = typeof d.state === "string" ? d.state : "?";
      return `dispatch_id=${id} state=${state}`;
    }
  }
  try {
    return JSON.stringify(result);
  } catch {
    return String(result);
  }
}

function messageOf(e: unknown): string {
  if (typeof e === "string") return e;
  if (e instanceof Error) return e.message;
  try {
    return JSON.stringify(e);
  } catch {
    return String(e);
  }
}
