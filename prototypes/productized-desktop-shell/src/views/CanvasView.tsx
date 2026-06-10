import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  ReactFlow,
  ReactFlowProvider,
  Background,
  Controls,
  MiniMap,
  addEdge,
  applyEdgeChanges,
  applyNodeChanges,
  type Connection,
  type Edge,
  type EdgeChange,
  type Node,
  type NodeChange,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";

import {
  canvasAbortRun,
  canvasLoad,
  canvasRunStatus,
  canvasSave,
  type CanvasRunStatus,
} from "../lib/tauri";
import { experimentCanvasBoundary, type CanvasSurfaceBoundary } from "../lib/canvasSurfaceBoundaries";
import type {
  CanvasDefinition,
  CanvasNode,
  CanvasNodeRole,
  SessionRecord,
} from "../lib/types";

type CanvasViewProps = {
  canvasId: string;
  sessions: SessionRecord[];
  onNotice: (msg: string) => void;
};

type FlowNode = Node<{
  label: string;
  role: CanvasNodeRole;
  skill?: string | null;
  session_id?: string | null;
}>;

const ROLE_STYLES: Record<CanvasNodeRole, { bg: string; border: string; tone: string }> = {
  director: { bg: "#fff3e6", border: "#c8602b", tone: "实验主管" },
  subagent: { bg: "#f4f1e8", border: "#5a6f4a", tone: "子 agent" },
};

function toFlowNodes(canvas: CanvasDefinition): FlowNode[] {
  return canvas.nodes.map((n) => ({
    id: n.id,
    position: n.position,
    data: { label: n.label, role: n.role, skill: n.skill, session_id: n.session_id },
    style: nodeStyle(n.role),
    type: "default",
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

function nodeStyle(role: CanvasNodeRole): React.CSSProperties {
  const s = ROLE_STYLES[role];
  return {
    background: s.bg,
    border: `2px solid ${s.border}`,
    padding: 8,
    borderRadius: 6,
    fontSize: 13,
    minWidth: 140,
    textAlign: "left",
  };
}

function fromFlow(canvas: CanvasDefinition, nodes: FlowNode[], edges: Edge[]): CanvasDefinition {
  const merged: CanvasNode[] = nodes.map((fn) => {
    const prior = canvas.nodes.find((n) => n.id === fn.id);
    return {
      id: fn.id,
      role: fn.data.role,
      label: fn.data.label,
      skill: fn.data.skill ?? null,
      session_id: fn.data.session_id ?? null,
      position: { x: fn.position.x, y: fn.position.y },
      warnings: prior?.warnings ?? [],
    };
  });
  return {
    ...canvas,
    nodes: merged,
    edges: edges.map((e) => ({ id: e.id, from: String(e.source), to: String(e.target) })),
    updated_at: new Date().toISOString(),
  };
}

export function CanvasView({ canvasId, sessions, onNotice }: CanvasViewProps) {
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

  useEffect(() => {
    void reload();
  }, [reload]);

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

  const addNode = useCallback((role: CanvasNodeRole) => {
    const id = `${role}-${Date.now().toString(36).slice(-5)}`;
    const label = role === "director" ? "实验主管" : "新子 agent";
    const offset = nodes.length * 30;
    const fn: FlowNode = {
      id,
      position: { x: 80 + offset, y: 80 + offset },
      data: { label, role, skill: role === "subagent" ? "" : null, session_id: null },
      style: nodeStyle(role),
    };
    setNodes((curr) => [...curr, fn]);
    setSelected(id);
    setDirty(true);
  }, [nodes.length]);

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
        <ExperimentCanvasBoundaryPanel boundary={experimentCanvasBoundary} />
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
        </div>
        <span className="canvas-id">canvas_id={canvas.canvas_id}</span>
      </header>
      <div className="canvas-body">
        <aside className="canvas-side">
          <ExperimentCanvasBoundaryPanel boundary={experimentCanvasBoundary} />
          <fieldset>
            <legend>新增节点</legend>
            <button onClick={() => addNode("director")} disabled={busy}>+ 实验主管</button>
            <button onClick={() => addNode("subagent")} disabled={busy}>+ 子智能体</button>
          </fieldset>
          <fieldset>
            <legend>节点编辑</legend>
            {selectedNode ? (
              <NodeEditor
                node={selectedNode}
                sessions={sessions}
                onChange={updateSelected}
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
        <div className="canvas-flow">
          <ReactFlow
            nodes={nodes}
            edges={edges}
            onNodesChange={onNodesChange}
            onEdgesChange={onEdgesChange}
            onConnect={onConnect}
            onSelectionChange={({ nodes: sel }) => setSelected(sel[0]?.id ?? null)}
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

export function CanvasViewWithProvider(props: CanvasViewProps) {
  return (
    <ReactFlowProvider>
      <CanvasView {...props} />
    </ReactFlowProvider>
  );
}

function NodeEditor({
  node,
  sessions,
  onChange,
  disabled,
}: {
  node: FlowNode;
  sessions: SessionRecord[];
  onChange: (patch: Partial<FlowNode["data"]>) => void;
  disabled: boolean;
}) {
  return (
    <div className="canvas-node-editor">
      <p>
        <strong>编号：</strong> <code>{node.id}</code>
      </p>
      <label>
        角色
        <select
          value={node.data.role}
          onChange={(e) => onChange({ role: e.target.value as CanvasNodeRole })}
          disabled={disabled}
        >
          <option value="director">实验主管</option>
          <option value="subagent">子智能体</option>
        </select>
      </label>
      <label>
        显示名
        <input
          type="text"
          value={node.data.label}
          onChange={(e) => onChange({ label: e.target.value })}
          disabled={disabled}
        />
      </label>
      {node.data.role === "subagent" && (
        <label>
          技能 / 岗位
          <input
            type="text"
            value={node.data.skill ?? ""}
            onChange={(e) => onChange({ skill: e.target.value })}
            disabled={disabled}
          />
        </label>
      )}
      <label>
        Codex 会话
        <select
          value={node.data.session_id ?? ""}
          onChange={(e) => onChange({ session_id: e.target.value || null })}
          disabled={disabled}
        >
          <option value="">— 未挂会话 —</option>
          {sessions.map((s) => (
            <option key={s.thread_id} value={s.thread_id}>
              {sessionLabel(s)}
            </option>
          ))}
        </select>
      </label>
      <p className="canvas-hint">
        v1 暂不支持画布内新建会话；先在 Codex 命令行或智能体页起好会话，再回来这里挂上。
      </p>
    </div>
  );
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

function messageOf(e: unknown): string {
  if (typeof e === "string") return e;
  if (e instanceof Error) return e.message;
  try {
    return JSON.stringify(e);
  } catch {
    return String(e);
  }
}
