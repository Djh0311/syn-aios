import { useCallback, useEffect, useMemo, useRef, useState } from "react";
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
  canvasAbortRun,
  canvasLoad,
  canvasRunStatus,
  canvasSave,
  deleteWorkflowTemplate,
  executeWorkflowNodeDispatch,
  listWorkflowTemplates,
  loadWorkflowTemplate,
  saveWorkflowTemplate,
  type CanvasRunStatus,
} from "../lib/tauri";
import type { CanvasSurfaceBoundary } from "../lib/canvasSurfaceBoundaries";
import type { CanvasSurfaceConfig } from "../lib/canvasSurfaceConfig";
import { pathTail } from "../lib/format";
import {
  NODE_KIND_PRESETS,
  SANDBOX_PRESETS,
  STATUS_PRESETS,
  buildNodeDispatchRequest,
  canvasNodeToData,
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
  const [goal, setGoal] = useState("");
  const [runId, setRunId] = useState<string | null>(null);
  const [runStatus, setRunStatus] = useState<CanvasRunStatus | null>(null);
  const [templates, setTemplates] = useState<WorkflowTemplateSummary[]>([]);
  const [templateTitle, setTemplateTitle] = useState("");
  const [confirmDeleteId, setConfirmDeleteId] = useState<string | null>(null);
  const pollRef = useRef<number | null>(null);

  const reload = useCallback(async () => {
    setBusy(true);
    setError(null);
    try {
      const c = await canvasLoad(canvasId);
      setCanvas(c);
      setNodes(toFlowNodes(c));
      setEdges(toFlowEdges(c));
      setDirty(false);
    } catch (e) {
      setError(messageOf(e));
    } finally {
      setBusy(false);
    }
  }, [canvasId]);

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

  // C1 · 运行此节点：调已存在的双闸命令 execute_workflow_node_dispatch。
  // 前端不判闸——默认安全态由后端守：非固定测试项目 / 未设 env 钥匙 → 返回 blocked
  // message，零执行。绑会话（C2）是 resume 前提，未绑则前端直接拦下不发。
  const runSelectedNode = useCallback(async () => {
    const node = nodes.find((n) => n.id === selected);
    if (!node || !canvas) return;
    const readiness = nodeRunReadiness(node.data);
    if (!readiness.ready) {
      onNotice(`无法运行：${readiness.reason}`);
      return;
    }
    setBusy(true);
    try {
      const request = buildNodeDispatchRequest({
        nodeId: node.id,
        projectRoot: canvas.project_root ?? "",
        data: node.data,
        instructionId: `canvas-run-${node.id}-${Date.now().toString(36)}`,
      });
      const result = await executeWorkflowNodeDispatch(request);
      onNotice(`已派发节点「${node.data.name}」。返回：${summarizeRunResult(result)}`);
    } catch (e) {
      // The backend double gate returns an Err (blocked message) when the fixed
      // test project + env key are not both set — surfaced here, no codex ran.
      onNotice(`运行被拦截或失败：${messageOf(e)}`);
    } finally {
      setBusy(false);
    }
  }, [nodes, selected, canvas, onNotice]);

  // P2 · 作用域升级：实验画布「绑定到项目」→ 设 project_root，画布变项目画布。
  // 只改定义里的 project_root（保存后持久化）；不碰双闸——真跑权限仍由后端定。
  const bindToProject = useCallback(() => {
    if (!canvas) return;
    const next = window.prompt("绑定到项目（项目根目录绝对路径，留空=保持实验画布）", canvas.project_root ?? "");
    if (next === null) return;
    const trimmed = next.trim();
    setCanvas({ ...canvas, project_root: trimmed || null });
    setDirty(true);
    onNotice(trimmed ? `画布已绑定项目：${trimmed}（记得保存）` : "已取消项目绑定，回到实验画布（记得保存）");
  }, [canvas, onNotice]);

  const startRun = useCallback(async () => {
    onNotice("旧实验画布真实运行入口已封存；H 阶段统一产品命令完成前，不能从这里启动 Codex。");
  }, [onNotice]);

  const abort = useCallback(async () => {
    if (!runId) return;
    setBusy(true);
    try {
      const status = await canvasAbortRun(runId, "用户拍停");
      setRunStatus(status);
      onNotice("已停止实验画布运行；未写项目 workflow。");
    } catch (e) {
      setError(messageOf(e));
    } finally {
      setBusy(false);
    }
  }, [runId, onNotice]);

  useEffect(() => {
    if (!runId) return;
    const tick = async () => {
      try {
        const s = await canvasRunStatus(runId);
        setRunStatus(s);
        if (s.run.status !== "running") {
          if (pollRef.current !== null) {
            window.clearInterval(pollRef.current);
            pollRef.current = null;
          }
        }
      } catch {
        // tolerate transient
      }
    };
    void tick();
    pollRef.current = window.setInterval(tick, 2000);
    return () => {
      if (pollRef.current !== null) {
        window.clearInterval(pollRef.current);
        pollRef.current = null;
      }
    };
  }, [runId]);

  const selectedNode = useMemo(
    () => nodes.find((n) => n.id === selected) ?? null,
    [nodes, selected],
  );

  const nodeTypes = useMemo<NodeTypes>(() => ({ canvasNode: CanvasFlowNode }), []);

  if (error) {
    return (
      <section className="canvas-view canvas-load-fallback">
        <header className="canvas-head">
          <div>
            <p className="eyebrow">实验 / 模板画布</p>
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
    return <section className="canvas-view">载入实验 / 模板画布……</section>;
  }

  return (
    <section className="canvas-view" aria-label="实验 / 模板画布">
      <header className="canvas-head">
        <div>
          <p className="eyebrow">实验 / 模板画布</p>
          <h2>{canvas.display_name}</h2>
          <div className="canvas-scope" data-scope={canvas.project_root ? "project" : "experiment"}>
            {canvas.project_root ? (
              <span className="canvas-scope-chip project" title={canvas.project_root}>
                项目画布 · {pathTail(canvas.project_root)}
              </span>
            ) : (
              <span className="canvas-scope-chip experiment">实验画布 · 未绑项目（真跑只打固定测试项目）</span>
            )}
            <button type="button" className="canvas-scope-bind" onClick={() => void bindToProject()} disabled={busy}>
              {canvas.project_root ? "改绑项目" : "绑定到项目"}
            </button>
          </div>
        </div>
        <span className="canvas-id">canvas_id={canvas.canvas_id}</span>
      </header>
      <div className="canvas-body">
        <aside className="canvas-side">
          <ExperimentCanvasBoundaryPanel boundary={config.boundary} />
          <fieldset>
            <legend>节点调色板</legend>
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
            <p className="canvas-hint">点种类建节点；或在空白处双击建节点。种类、字段都可在下方自由改。</p>
          </fieldset>
          <fieldset>
            <legend>节点编辑</legend>
            {selectedNode ? (
              <NodeEditor
                node={selectedNode}
                sessions={sessions}
                onChange={updateSelected}
                onRun={() => void runSelectedNode()}
                disabled={busy}
              />
            ) : (
              <p className="canvas-hint">点选画布上的节点编辑。</p>
            )}
          </fieldset>
          <fieldset>
            <legend>画布</legend>
            <button onClick={() => void save()} disabled={busy || !dirty}>
              {dirty ? "保存（未保存）" : "保存"}
            </button>
            <button onClick={() => void reload()} disabled={busy}>重载</button>
          </fieldset>
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
            {templates.length === 0 ? (
              <p className="canvas-hint">还没有成熟模式。把跑顺的工作流存下来，可一键起新工作流。</p>
            ) : (
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
            <p className="canvas-hint">从成熟模式起的新工作流节点 id 会重置；保存后成为独立画布。</p>
          </fieldset>
          <fieldset>
            <legend>实验运行边界</legend>
            <p className="canvas-hint">
              旧实验画布真实运行入口已封存，不会启动 Codex、不发送 prompt、不推进项目 workflow。后续真实执行必须走 H 阶段统一产品命令。
            </p>
            <label>
              目标草稿
              <textarea
                value={goal}
                onChange={(e) => setGoal(e.target.value)}
                rows={3}
                disabled={busy || !!runId}
              />
            </label>
            <button onClick={() => void startRun()} disabled={busy || !!runId}>
              查看封存边界
            </button>
            <button onClick={() => void abort()} disabled={busy || !runId}>
              停止实验画布运行
            </button>
            {runId && (
              <RunPanel runId={runId} status={runStatus} />
            )}
          </fieldset>
        </aside>
        <div className="canvas-flow" onDoubleClick={onPaneDoubleClick}>
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
  onChange,
  onRun,
  disabled,
}: {
  node: FlowNode;
  sessions: SessionRecord[];
  onChange: (patch: Partial<CanvasNodeData>) => void;
  onRun: () => void;
  disabled: boolean;
}) {
  const data = node.data;
  const readiness = nodeRunReadiness(data);
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
      <p>
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
        提示词 prompt
        <textarea
          value={data.prompt}
          onChange={(e) => onChange({ prompt: e.target.value })}
          rows={3}
          disabled={disabled}
        />
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
        {data.fields.length === 0 ? (
          <p className="canvas-hint">没有自定义字段。</p>
        ) : (
          data.fields.map((field) => (
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
          ))
        )}
        <button type="button" onClick={addField} disabled={disabled}>+ 字段</button>
      </fieldset>
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
          ) : (
            <p className="canvas-hint">运行时在作用域内新建一条 codex 会话（定义层不产生真会话）。</p>
          )}
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
          <p className="canvas-hint">
            {readiness.ready
              ? "经双闸命令派发；默认安全态（非固定测试项目 / 未设 env 钥匙）会被后端挡下、零执行。"
              : `运行前提未满足：${readiness.reason}`}
          </p>
        </div>
        <p className="canvas-hint">
          v1 暂不支持画布内新建会话；先在 Codex 命令行或智能体页起好会话，再回来这里绑上。
        </p>
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

function RunPanel({
  runId,
  status,
}: {
  runId: string;
  status: CanvasRunStatus | null;
}) {
  return (
    <div className="canvas-run-panel">
      <p className="canvas-hint">运行范围：实验画布；不是项目工作流事实源。</p>
      <p>
        <strong>运行编号：</strong> <code>{runId}</code>
      </p>
      <p>状态：{status?.run.status ?? "..."}</p>
      <p>忙碌节点：{status?.run.busy_node_id ?? "—"}</p>
      {status?.run.outbox && (
        <p>
          上次交回：{status.run.outbox.node_id}（{status.run.outbox.summary}）
        </p>
      )}
      {status?.run.finish_summary && (
        <p>收工：{status.run.finish_summary}</p>
      )}
      {status?.run.abort_reason && (
        <p>停止原因：{status.run.abort_reason}</p>
      )}
    </div>
  );
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
